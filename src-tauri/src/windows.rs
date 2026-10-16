//! Fenster der App. Sie werden bei Bedarf erzeugt; Editor und Einstellungen
//! werden beim Schließen nur versteckt. Das Dock-Icon erscheint nur, solange
//! ein normales Fenster sichtbar ist.

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::capture::Rect;

pub const EDITOR: &str = "editor";
pub const PERMISSION: &str = "permission";
pub const WELCOME: &str = "welcome";
pub const SETTINGS: &str = "settings";
pub const THUMBNAIL: &str = "thumbnail";
pub const OVERLAY_PREFIX: &str = "overlay-";
pub const PIN_PREFIX: &str = "pin-";
pub const RECORDING_FRAME: &str = "recording-frame";
pub const RECORDING_BAR: &str = "recording-bar";

/// Abstand des Aufnahme-Rahmens zum aufgenommenen Bereich, damit er nicht im Video landet.
const RECORDING_FRAME_GAP: f64 = 4.0;
const RECORDING_BAR_WIDTH: f64 = 230.0;
const RECORDING_BAR_HEIGHT: f64 = 48.0;
const RECORDING_BAR_GAP: f64 = 12.0;

const THUMBNAIL_WIDTH: f64 = 260.0;
const THUMBNAIL_MARGIN: f64 = 20.0;

/// Fenster, die beim Schließen nur versteckt werden.
pub fn is_persistent(label: &str) -> bool {
    matches!(label, EDITOR | SETTINGS | PERMISSION)
}

/// Fenster, bei denen die App ein Dock-Icon zeigt.
#[cfg(target_os = "macos")]
fn needs_dock(label: &str) -> bool {
    is_persistent(label) || label == WELCOME
}

pub fn show_editor(app: &AppHandle, always_on_top: bool) -> tauri::Result<WebviewWindow> {
    let window = existing_or(app, EDITOR, |app| {
        let builder = WebviewWindowBuilder::new(app, EDITOR, WebviewUrl::default())
            .title("Shot-VILLA")
            .inner_size(1100.0, 720.0)
            .min_inner_size(560.0, 360.0)
            .visible(false)
            .center();
        // Werkzeugleiste in der Titelleiste wie bei nativen Mac-Apps.
        #[cfg(target_os = "macos")]
        let builder = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true);
        builder.build()
    })?;
    window.set_always_on_top(always_on_top)?;
    present(app, &window)?;
    Ok(window)
}

pub fn show_settings(app: &AppHandle) -> tauri::Result<()> {
    let window = existing_or(app, SETTINGS, |app| {
        WebviewWindowBuilder::new(app, SETTINGS, WebviewUrl::default())
            .title("Einstellungen")
            .inner_size(600.0, 560.0)
            .resizable(false)
            .minimizable(false)
            .visible(false)
            .center()
            .build()
    })?;
    present(app, &window)
}

/// Begrüßung beim ersten Start: Installation hat geklappt, so geht's los.
pub fn show_welcome(app: &AppHandle) -> tauri::Result<()> {
    let window = existing_or(app, WELCOME, |app| {
        WebviewWindowBuilder::new(app, WELCOME, WebviewUrl::default())
            .title("Willkommen bei Shot-VILLA")
            .inner_size(520.0, 500.0)
            .resizable(false)
            .minimizable(false)
            .maximizable(false)
            .visible(false)
            .center()
            .build()
    })?;
    present(app, &window)
}

pub fn show_permission(app: &AppHandle) -> tauri::Result<()> {
    let window = existing_or(app, PERMISSION, |app| {
        WebviewWindowBuilder::new(app, PERMISSION, WebviewUrl::default())
            .title("Bildschirmaufnahme erlauben")
            .inner_size(460.0, 360.0)
            .resizable(false)
            .minimizable(false)
            .visible(false)
            .center()
            .build()
    })?;
    present(app, &window)
}

/// Kleine schwebende Vorschau unten rechts auf dem Bildschirm `screen`.
pub fn show_thumbnail(app: &AppHandle, screen: Rect, aspect: f64) -> tauri::Result<()> {
    const TOOLBAR_HEIGHT: f64 = 40.0;
    let height = (THUMBNAIL_WIDTH / aspect.max(0.2)).min(320.0) + TOOLBAR_HEIGHT;
    let x = screen.x + screen.width - THUMBNAIL_WIDTH - THUMBNAIL_MARGIN;
    let y = screen.y + screen.height - height - THUMBNAIL_MARGIN;
    let window = existing_or(app, THUMBNAIL, |app| {
        WebviewWindowBuilder::new(app, THUMBNAIL, WebviewUrl::default())
            .title("Vorschau")
            .decorations(false)
            .transparent(true)
            .shadow(true)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible_on_all_workspaces(true)
            .accept_first_mouse(true)
            .visible(false)
            .build()
    })?;
    window.set_size(tauri::LogicalSize::new(THUMBNAIL_WIDTH, height))?;
    window.set_position(tauri::LogicalPosition::new(x, y))?;
    window.show()
}

/// Schwebendes Bild, immer im Vordergrund.
pub fn open_pin(app: &AppHandle, shot_id: u32, frame: Rect) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, format!("{PIN_PREFIX}{shot_id}"), WebviewUrl::default())
        .title("Pin")
        .decorations(false)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .accept_first_mouse(true)
        .inner_size(frame.width, frame.height)
        .position(frame.x, frame.y)
        .build()?;
    Ok(())
}

/// Rahmen um den gewählten Bereich und mittig darunter eine Leiste mit
/// Start/Abbrechen bzw. Laufzeit und Stopp. Beide gehören zur App und
/// erscheinen deshalb nicht im Video.
pub fn show_recording(app: &AppHandle, area: Rect, screen: Rect) -> tauri::Result<()> {
    let gap = RECORDING_FRAME_GAP;
    let frame = floating(app, RECORDING_FRAME, "Aufnahmebereich")
        .inner_size(area.width + gap * 2.0, area.height + gap * 2.0)
        .position(area.x - gap, area.y - gap)
        .focused(false)
        .build()?;
    frame.set_ignore_cursor_events(true)?;
    let (x, y) = recording_bar_position(area, screen);
    // Fokus, damit Esc die noch nicht gestartete Aufnahme verwirft.
    floating(app, RECORDING_BAR, "Bildschirmaufnahme")
        .inner_size(RECORDING_BAR_WIDTH, RECORDING_BAR_HEIGHT)
        .position(x, y)
        .focused(true)
        .accept_first_mouse(true)
        .build()?;
    Ok(())
}

pub fn close_recording(app: &AppHandle) {
    for label in [RECORDING_FRAME, RECORDING_BAR] {
        if let Some(window) = app.get_webview_window(label)
            && let Err(error) = window.destroy()
        {
            log::warn!("{label} ließ sich nicht schließen: {error}");
        }
    }
}

/// Rahmenloses, durchsichtiges Fenster, immer im Vordergrund.
fn floating<'a>(
    app: &'a AppHandle,
    label: &str,
    title: &str,
) -> WebviewWindowBuilder<'a, tauri::Wry, AppHandle> {
    WebviewWindowBuilder::new(app, label, WebviewUrl::default())
        .title(title)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
}

/// Mittig unter dem Bereich; ist dort kein Platz, darüber, sonst innen am unteren Rand.
fn recording_bar_position(area: Rect, screen: Rect) -> (f64, f64) {
    let x = (area.x + (area.width - RECORDING_BAR_WIDTH) / 2.0)
        .clamp(screen.x, screen.x + screen.width - RECORDING_BAR_WIDTH);
    let below = area.y + area.height + RECORDING_FRAME_GAP + RECORDING_BAR_GAP;
    let above = area.y - RECORDING_FRAME_GAP - RECORDING_BAR_GAP - RECORDING_BAR_HEIGHT;
    let y = if below + RECORDING_BAR_HEIGHT <= screen.y + screen.height {
        below
    } else if above >= screen.y {
        above
    } else {
        area.y + area.height - RECORDING_BAR_HEIGHT - RECORDING_BAR_GAP
    };
    (x, y)
}

/// Vollflächiges Fenster über einem Bildschirm, auch über Menü- bzw. Taskleiste.
pub fn open_overlay(
    app: &AppHandle,
    display_id: u32,
    frame: Rect,
    scale: f64,
) -> tauri::Result<WebviewWindow> {
    let window = WebviewWindowBuilder::new(
        app,
        format!("{OVERLAY_PREFIX}{display_id}"),
        WebviewUrl::default(),
    )
    .title("Auswahl")
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .resizable(false)
    .skip_taskbar(true)
    .visible_on_all_workspaces(true)
    .accept_first_mouse(true)
    .inner_size(frame.width, frame.height)
    .position(frame.x, frame.y)
    .visible(false)
    .always_on_top(cfg!(windows))
    .build()?;
    #[cfg(target_os = "macos")]
    raise_above_menu_bar(&window)?;
    // Unter Windows exakt in physischen Pixeln platzieren: Bei gemischten
    // Skalierungen wären logische Koordinaten mehrdeutig.
    #[cfg(windows)]
    {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let physical = |value: f64| (value * scale).round() as i32;
        window.set_position(tauri::PhysicalPosition::new(
            physical(frame.x),
            physical(frame.y),
        ))?;
        window.set_size(tauri::PhysicalSize::new(
            physical(frame.width).unsigned_abs(),
            physical(frame.height).unsigned_abs(),
        ))?;
    }
    #[cfg(target_os = "macos")]
    let _ = scale;
    Ok(window)
}

/// Unter Windows nimmt die Aufnahme auch eigene Fenster mit; Editor und
/// Vorschau verschwinden deshalb kurz vorher. macOS schließt sie selbst aus.
pub fn hide_for_capture(app: &AppHandle) {
    #[cfg(windows)]
    {
        let mut hidden = false;
        for label in [EDITOR, THUMBNAIL] {
            if let Some(window) = app.get_webview_window(label)
                && window.is_visible().unwrap_or(false)
            {
                hidden |= window.hide().is_ok();
            }
        }
        if hidden {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
    #[cfg(target_os = "macos")]
    let _ = app;
}

pub fn close_overlays(app: &AppHandle) {
    for (label, window) in app.webview_windows() {
        if label.starts_with(OVERLAY_PREFIX)
            && let Err(error) = window.destroy()
        {
            log::warn!("Overlay {label} ließ sich nicht schließen: {error}");
        }
    }
}

/// Versteckt das Fenster statt es zu zerstören.
pub fn hide<R: Runtime>(app: &AppHandle<R>, window: &tauri::Window<R>) -> tauri::Result<()> {
    window.hide()?;
    update_dock(app, Some(window.label()))
}

/// Dock-Icon nur zeigen, solange ein normales Fenster sichtbar ist.
#[cfg(target_os = "macos")]
pub fn update_dock<R: Runtime>(app: &AppHandle<R>, closing: Option<&str>) -> tauri::Result<()> {
    use tauri::ActivationPolicy;
    let any_visible = app.webview_windows().values().any(|w| {
        Some(w.label()) != closing && needs_dock(w.label()) && w.is_visible().unwrap_or(false)
    });
    app.set_activation_policy(if any_visible {
        ActivationPolicy::Regular
    } else {
        ActivationPolicy::Accessory
    })
}

fn existing_or(
    app: &AppHandle,
    label: &str,
    build: impl FnOnce(&AppHandle) -> tauri::Result<WebviewWindow>,
) -> tauri::Result<WebviewWindow> {
    match app.get_webview_window(label) {
        Some(window) => Ok(window),
        None => build(app),
    }
}

/// Unter Windows zeigt die Taskleiste Fenster ohnehin nur, solange sie offen sind.
#[cfg(windows)]
#[expect(clippy::unnecessary_wraps, reason = "gleiche Signatur wie unter macOS")]
pub const fn update_dock<R: Runtime>(
    _app: &AppHandle<R>,
    _closing: Option<&str>,
) -> tauri::Result<()> {
    Ok(())
}

fn present(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular)?;
    #[cfg(windows)]
    let _ = app;
    window.unminimize()?;
    window.show()?;
    window.set_focus()
}

#[cfg(target_os = "macos")]
fn raise_above_menu_bar(window: &WebviewWindow) -> tauri::Result<()> {
    use objc2_app_kit::{NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior};

    let window = window.clone();
    window.clone().run_on_main_thread(move || {
        let Ok(pointer) = window.ns_window() else {
            return;
        };
        // SAFETY: Tauri liefert einen gültigen NSWindow-Zeiger; wir sind auf dem Main-Thread.
        let ns_window = unsafe { &*pointer.cast::<NSWindow>() };
        ns_window.setLevel(NSScreenSaverWindowLevel);
        ns_window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary,
        );
    })
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
    fn recording_bar_sits_below_above_or_inside() {
        let screen = rect(0.0, 0.0, 1000.0, 800.0);
        assert_eq!(
            recording_bar_position(rect(100.0, 100.0, 430.0, 200.0), screen),
            (200.0, 316.0)
        );
        assert_eq!(
            recording_bar_position(rect(100.0, 600.0, 430.0, 190.0), screen),
            (200.0, 536.0)
        );
        assert_eq!(
            recording_bar_position(rect(0.0, 0.0, 1000.0, 800.0), screen),
            (385.0, 740.0)
        );
    }
}
