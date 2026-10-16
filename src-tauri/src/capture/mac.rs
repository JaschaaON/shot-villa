//! Bildschirm- und Fensteraufnahme über `ScreenCaptureKit`.
//!
//! Alle Funktionen blockieren, bis `ScreenCaptureKit` geantwortet hat. Sie
//! dürfen deshalb nicht auf dem Main-Thread laufen.

use std::path::Path;
use std::ptr::NonNull;
use std::sync::{Mutex, mpsc};
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{AnyThread, DefinedClass, define_class, msg_send};
use objc2_app_kit::NSWorkspace;
use objc2_core_foundation::{CFRetained, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGEvent, CGImage, CGWindowListCreate, CGWindowListOption};
use objc2_core_media::CMTime;
use objc2_foundation::{
    NSArray, NSError, NSOperatingSystemVersion, NSProcessInfo, NSString, NSURL,
};
use objc2_screen_capture_kit::{
    SCCaptureResolutionType, SCContentFilter, SCDisplay, SCRecordingOutput,
    SCRecordingOutputConfiguration, SCRecordingOutputDelegate, SCScreenshotManager,
    SCShareableContent, SCStream, SCStreamConfiguration, SCWindow,
};

use super::{CaptureError, DisplayShot, Recording, Rect, WindowInfo, WindowShot};

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
    let filter = display_filter(content, display);
    let frame: Rect = unsafe { display.frame() }.into();
    let (scale, image) = screenshot(&filter, frame, shows_cursor)?;
    Ok(DisplayShot {
        display_id: unsafe { display.displayID() },
        frame,
        scale,
        image,
    })
}

/// Ganzer Bildschirm ohne eigene Fenster (Editor, Vorschau, Overlay, Aufnahme-Leiste).
fn display_filter(content: &SCShareableContent, display: &SCDisplay) -> Retained<SCContentFilter> {
    let own_pid = std::process::id();
    let own_apps: Vec<_> = unsafe { content.applications() }
        .iter()
        .filter(|app| u32::try_from(unsafe { app.processID() }).is_ok_and(|pid| pid == own_pid))
        .collect();
    unsafe {
        SCContentFilter::initWithDisplay_excludingApplications_exceptingWindows(
            SCContentFilter::alloc(),
            display,
            &NSArray::from_retained_slice(&own_apps),
            &NSArray::new(),
        )
    }
}

/// Bilder pro Sekunde der Videoaufnahme.
const FRAMES_PER_SECOND: i32 = 30;

/// Startet eine Videoaufnahme (H.264 in MP4) vom Bereich `area` des Bildschirms
/// `display_id`; `area` in Punkten relativ zum Bildschirm. Braucht macOS 15.
pub fn start_recording(
    display_id: u32,
    area: Rect,
    path: &Path,
) -> Result<Recording, CaptureError> {
    let macos_15 = NSOperatingSystemVersion {
        majorVersion: 15,
        minorVersion: 0,
        patchVersion: 0,
    };
    if !NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(macos_15) {
        return Err(CaptureError::Unsupported(
            "Bildschirmaufnahmen brauchen macOS 15 oder neuer",
        ));
    }
    let (started_sender, started) = mpsc::channel();
    let (stop, stop_requested) = mpsc::channel::<()>();
    let (finished_sender, finished) = mpsc::channel();
    let path = path.to_owned();
    // Die Systemobjekte bleiben in diesem Thread, bis die Aufnahme endet.
    std::thread::spawn(move || {
        let stream = match ActiveStream::start(display_id, area, &path) {
            Ok(stream) => {
                let _ = started_sender.send(Ok(()));
                stream
            }
            Err(error) => {
                let _ = started_sender.send(Err(error));
                return;
            }
        };
        // Endet auch, wenn die Gegenseite verschwindet.
        let _ = stop_requested.recv();
        let _ = finished_sender.send(stream.finish());
    });
    started
        .recv_timeout(TIMEOUT)
        .map_err(|_| CaptureError::Timeout)??;
    Ok(Recording::new(stop, finished))
}

/// Meldungen der Datei-Ausgabe.
enum RecordingEvent {
    Started,
    Failed(String),
    Finished,
}

define_class!(
    // SAFETY: NSObject stellt keine Anforderungen an Unterklassen, kein Drop.
    #[unsafe(super(NSObject))]
    #[name = "ShotVillaRecordingDelegate"]
    #[ivars = Mutex<mpsc::Sender<RecordingEvent>>]
    struct RecordingDelegate;

    unsafe impl NSObjectProtocol for RecordingDelegate {}

    unsafe impl SCRecordingOutputDelegate for RecordingDelegate {
        #[unsafe(method(recordingOutputDidStartRecording:))]
        fn did_start(&self, _output: &SCRecordingOutput) {
            self.send(RecordingEvent::Started);
        }

        #[unsafe(method(recordingOutput:didFailWithError:))]
        fn did_fail(&self, _output: &SCRecordingOutput, error: &NSError) {
            self.send(RecordingEvent::Failed(error.localizedDescription().to_string()));
        }

        #[unsafe(method(recordingOutputDidFinishRecording:))]
        fn did_finish(&self, _output: &SCRecordingOutput) {
            self.send(RecordingEvent::Finished);
        }
    }
);

impl RecordingDelegate {
    fn new(events: mpsc::Sender<RecordingEvent>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(Mutex::new(events));
        // SAFETY: `init` von NSObject.
        unsafe { msg_send![super(this), init] }
    }

    fn send(&self, event: RecordingEvent) {
        if let Ok(events) = self.ivars().lock() {
            // Nach dem Ende der Aufnahme hört niemand mehr zu.
            let _ = events.send(event);
        }
    }
}

/// Laufender Stream mit Datei-Ausgabe.
struct ActiveStream {
    stream: Retained<SCStream>,
    // Stream und Ausgabe halten ihren Delegate nur schwach.
    _output: Retained<SCRecordingOutput>,
    _delegate: Retained<RecordingDelegate>,
    events: mpsc::Receiver<RecordingEvent>,
}

impl ActiveStream {
    fn start(display_id: u32, area: Rect, path: &Path) -> Result<Self, CaptureError> {
        let content = shareable_content()?;
        let display = unsafe { content.displays() }
            .iter()
            .find(|d| unsafe { d.displayID() } == display_id)
            .ok_or(CaptureError::NoDisplay)?;
        let filter = display_filter(&content, &display);
        let info = unsafe { SCShareableContent::infoForFilter(&filter) };
        let scale = f64::from(unsafe { info.pointPixelScale() });
        let (width, height) = even_pixel_size(area, scale);

        let config = unsafe { SCStreamConfiguration::new() };
        unsafe {
            config.setSourceRect(CGRect::new(
                CGPoint::new(area.x, area.y),
                CGSize::new(area.width, area.height),
            ));
            config.setWidth(width);
            config.setHeight(height);
            config.setShowsCursor(true);
            config.setMinimumFrameInterval(CMTime::new(1, FRAMES_PER_SECOND));
        }

        let (sender, events) = mpsc::channel();
        let delegate = RecordingDelegate::new(sender);
        let output_config = unsafe { SCRecordingOutputConfiguration::new() };
        let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
        unsafe { output_config.setOutputURL(&url) };
        let output = unsafe {
            SCRecordingOutput::initWithConfiguration_delegate(
                SCRecordingOutput::alloc(),
                &output_config,
                ProtocolObject::from_ref(&*delegate),
            )
        };
        let stream = unsafe {
            SCStream::initWithFilter_configuration_delegate(
                SCStream::alloc(),
                &filter,
                &config,
                None,
            )
        };
        unsafe { stream.addRecordingOutput_error(&output) }
            .map_err(|error| system_error(&error))?;
        await_completion(|done| {
            let handler = RcBlock::new(move |error: *mut NSError| {
                let _ = done.send(if error.is_null() {
                    Ok(())
                } else {
                    Err(ns_error(error))
                });
            });
            unsafe { stream.startCaptureWithCompletionHandler(Some(&handler)) };
        })?;
        Ok(Self {
            stream,
            _output: output,
            _delegate: delegate,
            events,
        })
    }

    /// Stoppt den Stream und wartet, bis die Datei fertig geschrieben ist.
    fn finish(self) -> Result<(), CaptureError> {
        await_completion(|done| {
            let handler = RcBlock::new(move |error: *mut NSError| {
                let _ = done.send(if error.is_null() {
                    Ok(())
                } else {
                    Err(ns_error(error))
                });
            });
            unsafe { self.stream.stopCaptureWithCompletionHandler(Some(&handler)) };
        })?;
        loop {
            match self.events.recv_timeout(TIMEOUT) {
                Ok(RecordingEvent::Started) => {}
                Ok(RecordingEvent::Finished) => return Ok(()),
                Ok(RecordingEvent::Failed(message)) => return Err(CaptureError::System(message)),
                Err(_) => return Err(CaptureError::Timeout),
            }
        }
    }
}

/// Pixelgröße des Videos; H.264 verlangt gerade Kantenlängen.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn even_pixel_size(area: Rect, scale: f64) -> (usize, usize) {
    let even = |points: f64| (((points * scale) / 2.0).round() as usize * 2).max(2);
    (even(area.width), even(area.height))
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

fn system_error(error: &NSError) -> CaptureError {
    CaptureError::System(error.localizedDescription().to_string())
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
    fn video_size_is_even() {
        assert_eq!(
            even_pixel_size(rect(0.0, 0.0, 300.5, 201.0), 2.0),
            (602, 402)
        );
        assert_eq!(
            even_pixel_size(rect(0.0, 0.0, 301.0, 201.0), 1.0),
            (302, 202)
        );
        assert_eq!(even_pixel_size(rect(0.0, 0.0, 0.2, 0.2), 1.0), (2, 2));
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
