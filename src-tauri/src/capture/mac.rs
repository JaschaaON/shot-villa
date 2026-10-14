//! Bildschirm- und Fensteraufnahme über `ScreenCaptureKit`.
//!
//! Alle Funktionen blockieren, bis `ScreenCaptureKit` geantwortet hat. Sie
//! dürfen deshalb nicht auf dem Main-Thread laufen.

use std::ptr::NonNull;
use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use objc2_core_foundation::{CFRetained, CGPoint, CGRect};
use objc2_core_graphics::{CGEvent, CGImage, CGWindowListCreate, CGWindowListOption};
use objc2_foundation::{NSArray, NSError};
use objc2_screen_capture_kit::{
    SCCaptureResolutionType, SCContentFilter, SCDisplay, SCScreenshotManager, SCShareableContent,
    SCStreamConfiguration, SCWindow,
};

use super::{CaptureError, DisplayShot, Rect, WindowInfo, WindowShot};

const TIMEOUT: Duration = Duration::from_secs(10);

/// Nimmt alle Bildschirme auf.
pub fn all_displays(shows_cursor: bool) -> Result<Vec<DisplayShot>, CaptureError> {
    let content = shareable_content()?;
    unsafe { content.displays() }
        .iter()
        .map(|display| capture_display(&content, &display, shows_cursor))
        .collect()
}

/// Nimmt den Bildschirm auf, auf dem der Mauszeiger steht.
pub fn display_under_cursor(shows_cursor: bool) -> Result<DisplayShot, CaptureError> {
    let (x, y) = cursor_location();
    display_where(shows_cursor, |frame, _| frame.contains(x, y))
}

/// Nimmt einen bestimmten Bildschirm auf; fehlt er, den unter dem Mauszeiger.
pub fn display_by_id(id: u32, shows_cursor: bool) -> Result<DisplayShot, CaptureError> {
    let content = shareable_content()?;
    let display = unsafe { content.displays() }
        .iter()
        .find(|d| unsafe { d.displayID() } == id);
    match display {
        Some(display) => capture_display(&content, &display, shows_cursor),
        None => display_under_cursor(shows_cursor),
    }
}

fn display_where(
    shows_cursor: bool,
    matches: impl Fn(Rect, u32) -> bool,
) -> Result<DisplayShot, CaptureError> {
    let content = shareable_content()?;
    let displays = unsafe { content.displays() };
    let display = displays
        .iter()
        .find(|d| matches(unsafe { d.frame() }.into(), unsafe { d.displayID() }))
        .or_else(|| displays.firstObject())
        .ok_or(CaptureError::NoDisplay)?;
    capture_display(&content, &display, shows_cursor)
}

fn capture_display(
    content: &SCShareableContent,
    display: &SCDisplay,
    shows_cursor: bool,
) -> Result<DisplayShot, CaptureError> {
    // Eigene Fenster (Editor, Vorschau, Overlay) dürfen nie in der Aufnahme landen.
    let own_pid = std::process::id();
    let own_apps: Vec<_> = unsafe { content.applications() }
        .iter()
        .filter(|app| u32::try_from(unsafe { app.processID() }).is_ok_and(|pid| pid == own_pid))
        .collect();
    let filter = unsafe {
        SCContentFilter::initWithDisplay_excludingApplications_exceptingWindows(
            SCContentFilter::alloc(),
            display,
            &NSArray::from_retained_slice(&own_apps),
            &NSArray::new(),
        )
    };
    let frame: Rect = unsafe { display.frame() }.into();
    let (scale, image) = screenshot(&filter, frame, shows_cursor)?;
    Ok(DisplayShot {
        display_id: unsafe { display.displayID() },
        frame,
        scale,
        image,
    })
}

/// Sichtbare Fenster anderer Apps, vorderstes zuerst.
pub fn windows() -> Result<Vec<WindowInfo>, CaptureError> {
    let content = shareable_content()?;
    Ok(window_infos(&content))
}

/// Das vorderste Fenster der aktiven App.
pub fn frontmost_window() -> Result<WindowInfo, CaptureError> {
    let pid = NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .map(|app| app.processIdentifier())
        .ok_or(CaptureError::NoWindow)?;
    windows()?
        .into_iter()
        .find(|window| window.pid == i64::from(pid))
        .ok_or(CaptureError::NoWindow)
}

/// Nimmt ein einzelnes Fenster samt Schatten mit transparentem Hintergrund auf.
pub fn window(id: u32, shows_cursor: bool) -> Result<WindowShot, CaptureError> {
    let content = shareable_content()?;
    let window = unsafe { content.windows() }
        .iter()
        .find(|w| unsafe { w.windowID() } == id)
        .ok_or(CaptureError::NoWindow)?;
    let filter = unsafe {
        SCContentFilter::initWithDesktopIndependentWindow(SCContentFilter::alloc(), &window)
    };
    let info = unsafe { SCShareableContent::infoForFilter(&filter) };
    let content_rect: Rect = unsafe { info.contentRect() }.into();
    let (scale, image) = screenshot(&filter, content_rect, shows_cursor)?;
    Ok(WindowShot { scale, image })
}

fn window_infos(content: &SCShareableContent) -> Vec<WindowInfo> {
    let own_pid = std::process::id();
    let order = z_order();
    let mut infos: Vec<(usize, WindowInfo)> = unsafe { content.windows() }
        .iter()
        .filter(|w| is_regular_window(w))
        .filter_map(|w| {
            let app = unsafe { w.owningApplication() }?;
            let pid = unsafe { app.processID() };
            if u32::try_from(pid).is_ok_and(|pid| pid == own_pid) {
                return None;
            }
            let id = unsafe { w.windowID() };
            let rank = order.iter().position(|&other| other == id)?;
            Some((
                rank,
                WindowInfo {
                    id,
                    frame: unsafe { w.frame() }.into(),
                    app: unsafe { app.applicationName() }.to_string(),
                    title: unsafe { w.title() }
                        .map(|t| t.to_string())
                        .unwrap_or_default(),
                    pid: i64::from(pid),
                },
            ))
        })
        .collect();
    infos.sort_by_key(|(rank, _)| *rank);
    infos.into_iter().map(|(_, info)| info).collect()
}

fn is_regular_window(window: &SCWindow) -> bool {
    const MIN_SIZE: f64 = 40.0;
    let frame = unsafe { window.frame() };
    let on_screen = unsafe { window.isOnScreen() };
    let layer = unsafe { window.windowLayer() };
    on_screen && layer == 0 && frame.size.width >= MIN_SIZE && frame.size.height >= MIN_SIZE
}

/// Fenster-IDs auf dem Bildschirm, vorderstes zuerst.
fn z_order() -> Vec<u32> {
    let Some(list) = CGWindowListCreate(
        CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements,
        0,
    ) else {
        return Vec::new();
    };
    (0..list.count())
        // SAFETY: CGWindowListCreate liefert CGWindowIDs direkt als Array-Werte.
        .map(|index| unsafe { list.value_at_index(index) } as usize)
        .filter_map(|value| u32::try_from(value).ok())
        .collect()
}

fn screenshot(
    filter: &SCContentFilter,
    frame: Rect,
    shows_cursor: bool,
) -> Result<(f64, CFRetained<CGImage>), CaptureError> {
    let info = unsafe { SCShareableContent::infoForFilter(filter) };
    let scale = f64::from(unsafe { info.pointPixelScale() });
    let (width, height) = pixel_size(frame, scale);

    let config = unsafe { SCStreamConfiguration::new() };
    unsafe {
        config.setWidth(width);
        config.setHeight(height);
        config.setShowsCursor(shows_cursor);
        config.setCaptureResolution(SCCaptureResolutionType::Best);
        config.setShouldBeOpaque(false);
        config.setIgnoreShadowsSingleWindow(false);
    }

    let image = await_completion(|done| {
        let handler = RcBlock::new(move |image: *mut CGImage, error: *mut NSError| {
            let result = NonNull::new(image)
                // SAFETY: ScreenCaptureKit übergibt ein gültiges CGImage, das wir hier halten.
                .map(|image| unsafe { CFRetained::retain(image) })
                .ok_or_else(|| ns_error(error));
            // Der Empfänger kann nach einem Timeout bereits weg sein.
            let _ = done.send(result);
        });
        unsafe {
            SCScreenshotManager::captureImageWithFilter_configuration_completionHandler(
                filter,
                &config,
                Some(&handler),
            );
        }
    })?;
    Ok((scale, image))
}

/// Gelingt nur mit erteilter Bildschirmaufnahme-Berechtigung.
pub fn can_list_content() -> bool {
    shareable_content().is_ok()
}

fn shareable_content() -> Result<Retained<SCShareableContent>, CaptureError> {
    await_completion(|done| {
        let handler = RcBlock::new(
            move |content: *mut SCShareableContent, error: *mut NSError| {
                // SAFETY: Der Zeiger ist entweder null oder ein gültiges Objekt.
                let result = unsafe { Retained::retain(content) }.ok_or_else(|| ns_error(error));
                let _ = done.send(result);
            },
        );
        unsafe {
            SCShareableContent::getShareableContentExcludingDesktopWindows_onScreenWindowsOnly_completionHandler(
                true, true, &handler,
            );
        }
    })
}

/// Startet einen asynchronen Aufruf und wartet auf dessen Completion-Handler.
fn await_completion<T>(
    start: impl FnOnce(mpsc::Sender<Result<T, CaptureError>>),
) -> Result<T, CaptureError> {
    let (sender, receiver) = mpsc::channel();
    start(sender);
    receiver
        .recv_timeout(TIMEOUT)
        .map_err(|_| CaptureError::Timeout)?
}

fn ns_error(error: *mut NSError) -> CaptureError {
    // SAFETY: Der Zeiger ist entweder null oder ein gültiges NSError.
    let message = unsafe { Retained::retain(error) }.map_or_else(
        || "unbekannter Fehler".to_owned(),
        |e| e.localizedDescription().to_string(),
    );
    CaptureError::System(message)
}

/// Mausposition in globalen Punkten (Ursprung oben links).
pub fn cursor_location() -> (f64, f64) {
    let event = CGEvent::new(None);
    let CGPoint { x, y } = CGEvent::location(event.as_deref());
    (x, y)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn pixel_size(frame: Rect, scale: f64) -> (usize, usize) {
    (
        (frame.width * scale).round() as usize,
        (frame.height * scale).round() as usize,
    )
}

impl From<CGRect> for Rect {
    fn from(rect: CGRect) -> Self {
        Self {
            x: rect.origin.x,
            y: rect.origin.y,
            width: rect.size.width,
            height: rect.size.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn pixel_size_uses_scale() {
        assert_eq!(pixel_size(rect(0.0, 0.0, 1512.0, 982.0), 2.0), (3024, 1964));
        assert_eq!(
            pixel_size(rect(0.0, 0.0, 1920.0, 1080.0), 1.0),
            (1920, 1080)
        );
    }
}
