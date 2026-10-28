//! Aktionen, die über Menüleiste, Hotkeys und Frontend ausgelöst werden.
//!
//! Aufnahmen laufen in eigenen Threads, weil `ScreenCaptureKit` blockierend
//! abgewartet wird.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::capture::{self, Rect, WindowInfo};
use crate::imaging::{self, Format, Image, PixelRect};
use crate::settings::{AreaOpens, CursorCapture, Store};
use crate::shots::Shots;
use crate::{clipboard, files, ocr, output, permission, tray, windows};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Fullscreen,
    Area,
    RepeatArea,
    AnyWindow,
    ActiveWindow,
    Ocr,
    Record,
    RecordFullscreen,
    RecordCancel,
    ShowApp,
    LoadClipboard,
    OpenFile,
    Settings,
}

impl Action {
    pub const ALL: [Self; 13] = [
        Self::Fullscreen,
        Self::Area,
        Self::RepeatArea,
        Self::AnyWindow,
        Self::ActiveWindow,
        Self::Ocr,
        Self::Record,
        Self::RecordFullscreen,
        Self::RecordCancel,
        Self::ShowApp,
        Self::LoadClipboard,
        Self::OpenFile,
        Self::Settings,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Fullscreen => "capture-fullscreen",
            Self::Area => "capture-area",
            Self::RepeatArea => "capture-repeat-area",
            Self::AnyWindow => "capture-any-window",
            Self::ActiveWindow => "capture-active-window",
            Self::Ocr => "recognize-text",
            Self::Record => "record-area",
            Self::RecordFullscreen => "record-fullscreen",
            Self::RecordCancel => "record-cancel",
            Self::ShowApp => "show-app",
            Self::LoadClipboard => "load-clipboard",
            Self::OpenFile => "open-file",
            Self::Settings => "settings",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }
}

/// Woher ein Bild stammt; bestimmt Mauszeiger und Anzeige danach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Fullscreen,
    Area,
    Window,
    Imported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OverlayMode {
    Area,
    Window,
    Ocr,
    Record,
}

/// Ein Overlay-Fenster über einem Bildschirm mit dessen eingefrorenem Bild.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayDisplay {
    #[serde(skip)]
    pub label: String,
    pub display_id: u32,
    pub shot_id: u32,
    pub frame: Rect,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayInfo {
    pub mode: OverlayMode,
    pub display: OverlayDisplay,
    /// Fenster auf diesem Bildschirm, Koordinaten relativ zum Bildschirm.
    pub windows: Vec<WindowInfo>,
}

struct Overlay {
    mode: OverlayMode,
    displays: Vec<OverlayDisplay>,
    windows: Vec<WindowInfo>,
}

/// Eine laufende Bildschirmaufnahme (Video).
struct ActiveRecording {
    recording: capture::Recording,
    path: PathBuf,
    started: Instant,
}

/// Bildschirmaufnahme: Bereich gewählt und bereit, oder sie läuft.
enum RecordingState {
    /// Bereich in Punkten relativ zum Bildschirm `display_id`; wartet auf „Start“.
    Ready {
        display_id: u32,
        rect: Rect,
    },
    Running(ActiveRecording),
}

/// Wo die Bildschirmaufnahme gerade steht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingPhase {
    Idle,
    Ready,
    Running,
}

/// Was aufgenommen werden soll, wenn noch nichts bereitsteht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordingTarget {
    /// Bereich im Overlay wählen.
    Area,
    /// Ganzer Bildschirm unter dem Mauszeiger.
    Fullscreen,
}

/// Ereignis an Rahmen und Leiste: Die Aufnahme läuft jetzt.
const RECORDING_STARTED: &str = "recording-started";

/// Laufzustand der Aufnahme-Abläufe.
#[derive(Default)]
pub struct Session {
    overlay: Mutex<Option<Overlay>>,
    /// Zuletzt gewählter Bereich: Bildschirm und Rechteck relativ dazu.
    last_area: Mutex<Option<(u32, Rect)>>,
    recording: Mutex<Option<RecordingState>>,
}

impl Session {
    fn recording(&self) -> MutexGuard<'_, Option<RecordingState>> {
        self.recording
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn recording_phase(&self) -> RecordingPhase {
        match *self.recording() {
            None => RecordingPhase::Idle,
            Some(RecordingState::Ready { .. }) => RecordingPhase::Ready,
            Some(RecordingState::Running(_)) => RecordingPhase::Running,
        }
    }

    /// Wie lange die laufende Bildschirmaufnahme schon dauert.
    pub fn recording_elapsed(&self) -> Option<Duration> {
        match &*self.recording() {
            Some(RecordingState::Running(active)) => Some(active.started.elapsed()),
            _ => None,
        }
    }

    fn overlay(&self) -> MutexGuard<'_, Option<Overlay>> {
        self.overlay.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn last_area(&self) -> MutexGuard<'_, Option<(u32, Rect)>> {
        self.last_area
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// Auswahl im Overlay.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Selection {
    /// Bereich in Punkten relativ zum Bildschirm des Overlays.
    Area {
        rect: Rect,
    },
    Window {
        id: u32,
    },
    Cancel,
}

pub fn run(app: &AppHandle, action: Action) {
    match action {
        Action::ShowApp => report(show_app(app)),
        Action::Settings => report(windows::show_settings(app)),
        Action::Fullscreen => with_screen_access(app, capture_fullscreen),
        Action::Area => with_screen_access(app, |app| start_overlay(app, OverlayMode::Area)),
        Action::AnyWindow => with_screen_access(app, |app| start_overlay(app, OverlayMode::Window)),
        Action::Ocr => with_screen_access(app, |app| start_overlay(app, OverlayMode::Ocr)),
        Action::Record => toggle_recording(app, RecordingTarget::Area),
        Action::RecordFullscreen => toggle_recording(app, RecordingTarget::Fullscreen),
        Action::RecordCancel => in_background(app, cancel_recording),
        Action::RepeatArea => with_screen_access(app, repeat_area),
        Action::ActiveWindow => with_screen_access(app, capture_active_window),
        Action::LoadClipboard => in_background(app, load_clipboard),
        Action::OpenFile => in_background(app, pick_and_open_file),
    }
}

type ActionResult = Result<(), Box<dyn std::error::Error>>;

/// Führt eine Aufnahme im Hintergrund aus, sofern die Bildschirmaufnahme erlaubt
/// ist; sonst erscheint der Hinweis zur Freigabe.
fn with_screen_access(
    app: &AppHandle,
    task: impl FnOnce(&AppHandle) -> ActionResult + Send + 'static,
) {
    in_background(app, move |app| {
        if permission::screen_access() == permission::ScreenAccess::Granted {
            return task(app);
        }
        permission::request_screen_access();
        windows::show_permission(app)?;
        Ok(())
    });
}

fn in_background(app: &AppHandle, task: impl FnOnce(&AppHandle) -> ActionResult + Send + 'static) {
    let app = app.clone();
    std::thread::spawn(move || {
        if let Err(error) = task(&app) {
            log::error!("{error}");
            output::notify(&app, "Aufnahme fehlgeschlagen", &error.to_string());
        }
    });
}

fn shows_cursor(app: &AppHandle, kind: Kind) -> bool {
    match app.state::<Store>().get().capture_cursor {
        CursorCapture::Include => true,
        CursorCapture::Exclude => false,
        CursorCapture::Auto => kind == Kind::Fullscreen,
    }
}

fn capture_fullscreen(app: &AppHandle) -> ActionResult {
    windows::hide_for_capture(app);
    let shot = capture::display_under_cursor(shows_cursor(app, Kind::Fullscreen))?;
    deliver(app, &shot.image, shot.scale, Kind::Fullscreen, shot.frame)
}

fn capture_active_window(app: &AppHandle) -> ActionResult {
    let window = capture::frontmost_window()?;
    let shot = capture::window(window.id, shows_cursor(app, Kind::Window))?;
    deliver(
        app,
        &shot.image,
        shot.scale,
        Kind::Window,
        screen_of(window.frame),
    )
}

fn repeat_area(app: &AppHandle) -> ActionResult {
    let Some((display_id, rect)) = *app.state::<Session>().last_area() else {
        output::notify(
            app,
            "Noch kein Bereich",
            "Nimm zuerst einen Bereich auf, dann lässt er sich wiederholen.",
        );
        return Ok(());
    };
    windows::hide_for_capture(app);
    let shot = capture::display_by_id(display_id, shows_cursor(app, Kind::Area))?;
    let image = imaging::crop(&shot.image, pixel_rect(rect, shot.scale))?;
    deliver(app, &image, shot.scale, Kind::Area, shot.frame)
}

fn load_clipboard(app: &AppHandle) -> ActionResult {
    let Some(bytes) = clipboard::read_image() else {
        output::notify(app, "Kein Bild", "Die Zwischenablage enthält kein Bild.");
        return Ok(());
    };
    let (image, scale) = imaging::decode(&bytes)?;
    let (x, y) = capture::cursor_location();
    deliver(
        app,
        &image,
        scale,
        Kind::Imported,
        Rect {
            x,
            y,
            width: 0.0,
            height: 0.0,
        },
    )
}

/// Öffnet eine Bilddatei im Editor.
pub fn open_file(app: &AppHandle, path: &std::path::Path) -> ActionResult {
    let (image, scale) = imaging::decode(&std::fs::read(path)?)?;
    let (x, y) = capture::cursor_location();
    deliver(
        app,
        &image,
        scale,
        Kind::Imported,
        Rect {
            x,
            y,
            width: 0.0,
            height: 0.0,
        },
    )
}

fn pick_and_open_file(app: &AppHandle) -> ActionResult {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Bild öffnen")
        .add_filter(
            "Bilder",
            &["png", "jpg", "jpeg", "gif", "heic", "tiff", "webp"],
        )
        .blocking_pick_file();
    match picked.map(tauri_plugin_dialog::FilePath::into_path) {
        Some(Ok(path)) => open_file(app, &path),
        Some(Err(error)) => Err(error.into()),
        None => Ok(()),
    }
}

fn show_app(app: &AppHandle) -> ActionResult {
    let always_on_top = app.state::<Store>().get().always_on_top;
    windows::show_editor(app, always_on_top)?;
    Ok(())
}

/// Friert alle Bildschirme ein und zeigt darüber die Auswahl.
fn start_overlay(app: &AppHandle, mode: OverlayMode) -> ActionResult {
    if app.state::<Session>().overlay().is_some() {
        return Ok(());
    }
    windows::hide_for_capture(app);
    let shots = capture::all_displays(false)?;
    let all_windows = if mode == OverlayMode::Window {
        capture::windows()?
    } else {
        Vec::new()
    };

    let mut displays = Vec::with_capacity(shots.len());
    for shot in shots {
        let png = imaging::encode(&shot.image, Format::Png, shot.scale)?;
        let info = app
            .state::<Shots>()
            .insert(shot.image, shot.scale, png, Format::Png.mime());
        let window = windows::open_overlay(app, shot.display_id, shot.frame, shot.scale)?;
        displays.push(OverlayDisplay {
            label: window.label().to_owned(),
            display_id: shot.display_id,
            shot_id: info.id,
            frame: shot.frame,
            scale: shot.scale,
        });
    }
    *app.state::<Session>().overlay() = Some(Overlay {
        mode,
        displays,
        windows: all_windows,
    });
    Ok(())
}

/// Daten für das Overlay-Fenster `label`.
pub fn overlay_info(app: &AppHandle, label: &str) -> Option<OverlayInfo> {
    let session = app.state::<Session>();
    let guard = session.overlay();
    let overlay = guard.as_ref()?;
    let display = overlay.displays.iter().find(|d| d.label == label)?.clone();
    let windows = overlay
        .windows
        .iter()
        .filter(|w| intersects(w.frame, display.frame))
        .map(|w| w.relative_to(display.frame))
        .collect();
    Some(OverlayInfo {
        mode: overlay.mode,
        display,
        windows,
    })
}

/// Zeigt ein Overlay, sobald sein Bild geladen ist; das unter dem Mauszeiger bekommt den Fokus.
pub fn overlay_ready(app: &AppHandle, label: &str) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window(label) else {
        return Ok(());
    };
    window.show()?;
    let (x, y) = capture::cursor_location();
    let under_cursor = app
        .state::<Session>()
        .overlay()
        .as_ref()
        .and_then(|o| {
            o.displays
                .iter()
                .find(|d| d.label == label)
                .map(|d| d.frame)
        })
        .is_some_and(|frame| frame.contains(x, y));
    if under_cursor {
        window.set_focus()?;
    }
    Ok(())
}

/// Beendet die Auswahl und verarbeitet das Ergebnis.
pub fn finish_overlay(app: &AppHandle, label: &str, selection: Selection) {
    let Some(overlay) = app.state::<Session>().overlay().take() else {
        return;
    };
    windows::close_overlays(app);
    let display = overlay.displays.iter().find(|d| d.label == label).cloned();
    let frozen: Vec<u32> = overlay.displays.iter().map(|d| d.shot_id).collect();
    let mode = overlay.mode;

    in_background(app, move |app| {
        let result = match (selection, display) {
            (Selection::Area { rect }, Some(display)) => select_area(app, mode, &display, rect),
            (Selection::Window { id }, _) => select_window(app, id),
            _ => Ok(()),
        };
        let shots = app.state::<Shots>();
        for id in frozen {
            shots.remove(id);
        }
        result
    });
}

fn select_area(
    app: &AppHandle,
    mode: OverlayMode,
    display: &OverlayDisplay,
    rect: Rect,
) -> ActionResult {
    if rect.width < 1.0 || rect.height < 1.0 {
        return Ok(());
    }
    let (frozen, scale) = app
        .state::<Shots>()
        .image(display.shot_id)
        .ok_or("Eingefrorenes Bild fehlt")?;
    let image = imaging::crop(&frozen, pixel_rect(rect, scale))?;
    match mode {
        OverlayMode::Ocr => return recognize_into_clipboard(app, &image),
        OverlayMode::Record => {
            return prepare_recording(app, display.display_id, display.frame, rect);
        }
        OverlayMode::Area | OverlayMode::Window => {}
    }
    *app.state::<Session>().last_area() = Some((display.display_id, rect));
    deliver(app, &image, scale, Kind::Area, display.frame)
}

/// Je nach Stand: Bereich bzw. Bildschirm wählen, die bereite Aufnahme starten
/// oder die laufende beenden. Menü, Kürzel und die Leiste nutzen das.
fn toggle_recording(app: &AppHandle, target: RecordingTarget) {
    match app.state::<Session>().recording_phase() {
        RecordingPhase::Idle => with_screen_access(app, move |app| match target {
            RecordingTarget::Area => start_overlay(app, OverlayMode::Record),
            RecordingTarget::Fullscreen => prepare_fullscreen_recording(app),
        }),
        RecordingPhase::Ready => in_background(app, begin_recording),
        RecordingPhase::Running => in_background(app, stop_recording),
    }
}

/// Bereitet die Aufnahme des ganzen Bildschirms unter dem Mauszeiger vor.
fn prepare_fullscreen_recording(app: &AppHandle) -> ActionResult {
    let (display_id, screen) = capture::display_info_under_cursor()?;
    let whole = Rect {
        x: 0.0,
        y: 0.0,
        ..screen
    };
    prepare_recording(app, display_id, screen, whole)
}

/// Zeigt Rahmen und Leiste mit „Start“; aufgenommen wird erst danach.
/// `rect` liegt in Punkten relativ zum Bildschirm `screen`.
fn prepare_recording(app: &AppHandle, display_id: u32, screen: Rect, rect: Rect) -> ActionResult {
    *app.state::<Session>().recording() = Some(RecordingState::Ready { display_id, rect });
    let area = Rect {
        x: screen.x + rect.x,
        y: screen.y + rect.y,
        ..rect
    };
    windows::show_recording(app, area, screen)?;
    tray::refresh(app, &app.state::<Store>().get().hotkeys)?;
    Ok(())
}

/// Verwirft den gewählten Bereich, solange die Aufnahme noch nicht läuft.
fn cancel_recording(app: &AppHandle) -> ActionResult {
    {
        let session = app.state::<Session>();
        let mut state = session.recording();
        if !matches!(*state, Some(RecordingState::Ready { .. })) {
            return Ok(());
        }
        *state = None;
    }
    windows::close_recording(app);
    tray::refresh(app, &app.state::<Store>().get().hotkeys)?;
    Ok(())
}

fn begin_recording(app: &AppHandle) -> ActionResult {
    let Some((display_id, rect)) = (match *app.state::<Session>().recording() {
        Some(RecordingState::Ready { display_id, rect }) => Some((display_id, rect)),
        _ => None,
    }) else {
        return Ok(());
    };
    let folder = app.state::<Store>().get().save_folder();
    let path = files::new_recording_path(&folder)?;
    let recording = match capture::start_recording(display_id, rect, &path) {
        Ok(recording) => recording,
        Err(error) => {
            // Eine angelegte, aber leere Datei bliebe sonst liegen.
            let _ = std::fs::remove_file(&path);
            cancel_recording(app)?;
            return Err(error.into());
        }
    };
    *app.state::<Session>().recording() = Some(RecordingState::Running(ActiveRecording {
        recording,
        path,
        started: Instant::now(),
    }));
    app.emit(RECORDING_STARTED, ())?;
    tray::refresh(app, &app.state::<Store>().get().hotkeys)?;
    // Laufzeit in der Menüleiste, solange die Aufnahme läuft.
    let app = app.clone();
    std::thread::spawn(move || {
        while let Some(elapsed) = app.state::<Session>().recording_elapsed() {
            tray::show_recording_time(&app, Some(elapsed));
            std::thread::sleep(Duration::from_millis(500));
        }
        tray::show_recording_time(&app, None);
    });
    Ok(())
}

fn stop_recording(app: &AppHandle) -> ActionResult {
    let active = {
        let session = app.state::<Session>();
        let mut state = session.recording();
        match state.take() {
            Some(RecordingState::Running(active)) => active,
            other => {
                *state = other;
                return Ok(());
            }
        }
    };
    windows::close_recording(app);
    tray::refresh(app, &app.state::<Store>().get().hotkeys)?;
    active.recording.stop()?;
    files::mark_as_screen_capture(&active.path)?;
    let name = active
        .path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    output::notify(app, "Bildschirmaufnahme gesichert", &name);
    files::reveal(&active.path);
    Ok(())
}

fn select_window(app: &AppHandle, id: u32) -> ActionResult {
    // Das Overlay muss vom Bildschirm verschwunden sein.
    std::thread::sleep(Duration::from_millis(60));
    let windows = capture::windows()?;
    let frame = windows.iter().find(|w| w.id == id).map(|w| w.frame);
    let shot = capture::window(id, shows_cursor(app, Kind::Window))?;
    let screen = frame.map_or_else(cursor_screen, screen_of);
    deliver(app, &shot.image, shot.scale, Kind::Window, screen)
}

/// Was die Texterkennung gefunden hat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Found {
    Text,
    Code,
    Nothing,
}

#[derive(Debug, Clone, Serialize)]
pub struct Recognized {
    pub found: Found,
    pub text: String,
}

/// Erkennt Text und QR-Codes und legt das Ergebnis in die Zwischenablage.
pub fn recognize_to_clipboard(app: &AppHandle, image: &Image) -> Result<Recognized, ocr::OcrError> {
    let recognition = ocr::recognize(image)?;
    let remove_breaks = app.state::<Store>().get().ocr_remove_line_breaks;
    let (found, text) = if recognition.codes.is_empty() {
        (
            Found::Text,
            ocr::join_lines(&recognition.lines, remove_breaks),
        )
    } else {
        (Found::Code, recognition.codes.join("\n"))
    };
    if text.is_empty() {
        return Ok(Recognized {
            found: Found::Nothing,
            text,
        });
    }
    clipboard::write_text(&text);
    Ok(Recognized { found, text })
}

/// Wie [`recognize_to_clipboard`], meldet das Ergebnis aber per Mitteilung –
/// für Erkennung per Kürzel, wenn kein Fenster offen ist.
pub fn recognize_into_clipboard(app: &AppHandle, image: &Image) -> ActionResult {
    let recognized = recognize_to_clipboard(app, image)?;
    let title = match recognized.found {
        Found::Text => "Text kopiert",
        Found::Code => "QR-Code kopiert",
        Found::Nothing => {
            output::notify(
                app,
                "Kein Text erkannt",
                "Im gewählten Bereich wurde kein Text gefunden.",
            );
            return Ok(());
        }
    };
    output::notify(app, title, &preview(&recognized.text));
    Ok(())
}

/// Verarbeitet ein fertiges Bild gemäß „Nach der Aufnahme“.
fn deliver(app: &AppHandle, image: &Image, scale: f64, kind: Kind, screen: Rect) -> ActionResult {
    let settings = app.state::<Store>().get();
    let after = settings.after_capture;
    let png = imaging::encode(image, Format::Png, scale)?;
    if after.copy && kind != Kind::Imported {
        clipboard::write_image(&png);
    }
    if after.save && kind != Kind::Imported {
        let path = output::save(image, scale, &settings)?;
        if !after.show {
            output::notify(app, "Gespeichert", &path.display().to_string());
        }
    }

    let image = image.clone();
    let shots = app.state::<Shots>();
    let info = shots.insert(image, scale, png, Format::Png.mime());
    shots.make_current(info.id);

    let show = after.show || kind == Kind::Imported;
    if !show {
        if after.copy {
            output::notify(app, "Kopiert", "Die Aufnahme liegt in der Zwischenablage.");
        }
        return Ok(());
    }
    #[allow(clippy::cast_precision_loss)]
    let aspect = info.width as f64 / info.height.max(1) as f64;
    if kind == Kind::Area && settings.area_opens == AreaOpens::Thumbnail {
        windows::show_thumbnail(app, cursor_screen_or(screen), aspect)?;
        app.emit_to(windows::THUMBNAIL, "shot-ready", info)?;
    } else {
        windows::show_editor(app, settings.always_on_top)?;
        app.emit_to(windows::EDITOR, "shot-ready", info)?;
    }
    Ok(())
}

/// Bildschirmrahmen, auf dem ein Fenster liegt (für die Vorschau-Position).
fn screen_of(frame: Rect) -> Rect {
    crate::display::frame_containing(frame)
}

fn cursor_screen_or(screen: Rect) -> Rect {
    if screen.width > 0.0 {
        screen
    } else {
        cursor_screen()
    }
}

fn cursor_screen() -> Rect {
    let (x, y) = capture::cursor_location();
    crate::display::frame_at(x, y)
}

fn intersects(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

/// Rechnet einen Bereich in Punkten in ganze Pixel um.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn pixel_rect(rect: Rect, scale: f64) -> PixelRect {
    let px = |value: f64| (value * scale).round().max(0.0) as u32;
    PixelRect {
        x: px(rect.x),
        y: px(rect.y),
        width: px(rect.width),
        height: px(rect.height),
    }
}

fn preview(text: &str) -> String {
    const MAX: usize = 120;
    let mut short: String = text.chars().take(MAX).collect();
    if text.chars().count() > MAX {
        short.push('…');
    }
    short
}

fn report<E: std::fmt::Display>(result: Result<(), E>) {
    if let Err(error) = result {
        log::error!("{error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_ids_round_trip() {
        for action in Action::ALL {
            assert_eq!(Action::from_id(action.id()), Some(action));
        }
        assert_eq!(Action::from_id("unbekannt"), None);
    }

    #[test]
    fn pixel_rect_scales_and_rounds() {
        let rect = pixel_rect(
            Rect {
                x: 10.2,
                y: 5.0,
                width: 100.4,
                height: 50.0,
            },
            2.0,
        );
        assert_eq!(
            rect,
            PixelRect {
                x: 20,
                y: 10,
                width: 201,
                height: 100
            }
        );
    }

    #[test]
    fn intersection_needs_overlap() {
        let screen = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let inside = Rect {
            x: 90.0,
            y: 90.0,
            width: 50.0,
            height: 50.0,
        };
        let outside = Rect { x: 100.0, ..inside };
        assert!(intersects(inside, screen));
        assert!(!intersects(outside, screen));
    }

    #[test]
    fn preview_shortens_long_text() {
        assert_eq!(preview("kurz"), "kurz");
        assert_eq!(preview(&"a".repeat(130)).chars().count(), 121);
    }
}
