//! Befehle, die das Frontend aufruft.

use std::path::PathBuf;

use serde::Deserialize;
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Manager, State, WebviewWindow};

use crate::actions::{self, Action, OverlayInfo, Selection};
use crate::capture::Rect;
use crate::imaging::{self, Format};
use crate::permission::{self, ScreenAccess};
use crate::settings::{Settings, Store};
use crate::shots::{ShotInfo, Shots};
use crate::{autostart, hotkeys, output, tray, windows};

type CommandResult<T = ()> = Result<T, String>;

fn text<E: std::fmt::Display>(error: E) -> String {
    error.to_string()
}

/// Bildbytes und Parameter eines Raw-Aufrufs. Die Bytes kommen als Body,
/// die Parameter als JSON im Header `x-params` (URI-kodiert).
fn raw_request<T: serde::de::DeserializeOwned>(
    request: &Request<'_>,
) -> CommandResult<(Vec<u8>, T)> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("Bilddaten fehlen".into());
    };
    let header = request
        .headers()
        .get("x-params")
        .and_then(|value| value.to_str().ok())
        .ok_or("Parameter fehlen")?;
    let params = serde_json::from_str(&percent_decode(header)).map_err(text)?;
    Ok((bytes.clone(), params))
}

/// Dekodiert `encodeURIComponent`.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let hex = bytes
            .get(index + 1..index + 3)
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match (bytes[index], hex) {
            (b'%', Some(byte)) => {
                decoded.push(byte);
                index += 3;
            }
            (byte, _) => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn current_shot(shots: State<'_, Shots>) -> Option<ShotInfo> {
    shots.current()
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn shot_info(id: u32, shots: State<'_, Shots>) -> Option<ShotInfo> {
    shots.info(id)
}

/// Liefert die kodierten Bildbytes ohne JSON-Umweg.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn shot_png(id: u32, shots: State<'_, Shots>) -> CommandResult<Response> {
    shots
        .bytes(id)
        .map(|(bytes, _)| Response::new(bytes))
        .ok_or_else(|| format!("Bild {id} ist nicht mehr vorhanden"))
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn run_action(id: String, app: AppHandle) -> CommandResult {
    let action = Action::from_id(&id).ok_or_else(|| format!("Unbekannte Aktion {id}"))?;
    actions::run(&app, action);
    Ok(())
}

// Einstellungen

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn get_settings(app: AppHandle, store: State<'_, Store>) -> Settings {
    let mut settings = store.get();
    settings.launch_at_login = autostart::is_enabled(&app);
    settings
}

/// Speichert die Einstellungen und meldet Kürzel, die nicht registriert werden konnten.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn set_settings(
    settings: Settings,
    app: AppHandle,
    store: State<'_, Store>,
) -> CommandResult<Vec<String>> {
    let previous = store.get();
    if settings.launch_at_login != autostart::is_enabled(&app) {
        autostart::set_enabled(&app, settings.launch_at_login)?;
    }
    store.set(settings.clone()).map_err(text)?;
    if let Some(editor) = app.get_webview_window(windows::EDITOR) {
        editor
            .set_always_on_top(settings.always_on_top)
            .map_err(text)?;
    }
    if previous.hotkeys == settings.hotkeys {
        return Ok(Vec::new());
    }
    tray::refresh(&app, &settings.hotkeys).map_err(text)?;
    Ok(hotkeys::apply(&app, &settings.hotkeys))
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn suspend_hotkeys(app: AppHandle) {
    hotkeys::suspend(&app);
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn resume_hotkeys(app: AppHandle, store: State<'_, Store>) -> Vec<String> {
    hotkeys::apply(&app, &store.get().hotkeys)
}

#[tauri::command]
pub async fn choose_save_folder(app: AppHandle) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .set_title("Ordner für Bildschirmfotos")
        .blocking_pick_folder()
        .and_then(|path| path.into_path().ok())
}

// Overlay

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn overlay_info(window: WebviewWindow, app: AppHandle) -> Option<OverlayInfo> {
    actions::overlay_info(&app, window.label())
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn overlay_ready(window: WebviewWindow, app: AppHandle) -> CommandResult {
    actions::overlay_ready(&app, window.label()).map_err(text)
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn overlay_finish(selection: Selection, window: WebviewWindow, app: AppHandle) {
    actions::finish_overlay(&app, window.label(), selection);
}

// Ausgabe

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportTarget {
    Copy,
    Save,
    SaveAs { path: PathBuf },
}

#[derive(Debug, Deserialize)]
pub struct ExportParams {
    scale: f64,
    target: ExportTarget,
}

#[derive(Debug, Deserialize)]
pub struct ScaleParams {
    scale: f64,
}

/// Gibt ein im Editor gerendertes Bild (PNG) aus. Liefert den Pfad beim Speichern.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn export_image(
    request: Request<'_>,
    app: AppHandle,
    store: State<'_, Store>,
) -> CommandResult<Option<PathBuf>> {
    let (png, ExportParams { scale, target }) = raw_request(&request)?;
    match target {
        ExportTarget::Copy => {
            let (image, _) = imaging::decode(&png).map_err(text)?;
            output::copy(&image, scale).map_err(text)?;
            Ok(None)
        }
        ExportTarget::Save => {
            let (image, _) = imaging::decode(&png).map_err(text)?;
            let path = output::save(&image, scale, &store.get()).map_err(text)?;
            output::notify(&app, "Gespeichert", &path.display().to_string());
            Ok(Some(path))
        }
        ExportTarget::SaveAs { path } => {
            let (image, _) = imaging::decode(&png).map_err(text)?;
            output::save_as(&image, scale, &store.get(), &path).map_err(text)?;
            Ok(Some(path))
        }
    }
}

/// Kopiert oder speichert ein Bild ohne Bearbeitung (aus der Vorschau).
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn export_shot(
    id: u32,
    target: ExportTarget,
    app: AppHandle,
    shots: State<'_, Shots>,
    store: State<'_, Store>,
) -> CommandResult<Option<PathBuf>> {
    let (image, scale) = shots.image(id).ok_or("Bild ist nicht mehr vorhanden")?;
    match target {
        ExportTarget::Copy => output::copy(&image, scale).map(|()| None).map_err(text),
        ExportTarget::Save => {
            let path = output::save(&image, scale, &store.get()).map_err(text)?;
            output::notify(&app, "Gespeichert", &path.display().to_string());
            Ok(Some(path))
        }
        ExportTarget::SaveAs { path } => output::save_as(&image, scale, &store.get(), &path)
            .map(|()| Some(path))
            .map_err(text),
    }
}

/// Erkennt Text im übergebenen Bildausschnitt (PNG) und kopiert ihn.
#[tauri::command]
pub async fn recognize_text(
    request: Request<'_>,
    app: AppHandle,
) -> CommandResult<actions::Recognized> {
    let (png, ScaleParams { .. }) = raw_request(&request)?;
    let (image, _) = imaging::decode(&png).map_err(text)?;
    actions::recognize_to_clipboard(&app, &image).map_err(text)
}

/// Pinnt ein gerendertes Bild an den Bildschirm.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn pin_image(request: Request<'_>, app: AppHandle, shots: State<'_, Shots>) -> CommandResult {
    let (png, ScaleParams { scale }) = raw_request(&request)?;
    let (image, _) = imaging::decode(&png).map_err(text)?;
    let info = shots.insert(image, scale, png, Format::Png.mime());
    #[allow(clippy::cast_precision_loss)]
    let (width, height) = (info.width as f64 / scale, info.height as f64 / scale);
    let (x, y) = crate::capture::cursor_location();
    windows::open_pin(
        &app,
        info.id,
        Rect {
            x: x - width / 2.0,
            y: y - height / 2.0,
            width,
            height,
        },
    )
    .map_err(text)
}

/// Bild des Pin-Fensters, aus dessen Label abgeleitet.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn pin_shot(window: WebviewWindow, shots: State<'_, Shots>) -> Option<ShotInfo> {
    let id = window
        .label()
        .strip_prefix(windows::PIN_PREFIX)?
        .parse()
        .ok()?;
    shots.info(id)
}

/// Öffnet den Editor mit der aktuellen Aufnahme (aus der Vorschau).
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn edit_current(app: AppHandle, store: State<'_, Store>) -> CommandResult {
    if let Some(thumbnail) = app.get_webview_window(windows::THUMBNAIL) {
        thumbnail.hide().map_err(text)?;
    }
    windows::show_editor(&app, store.get().always_on_top).map_err(text)?;
    Ok(())
}

/// Öffnet Bilddateien (Drag & Drop oder Dialog).
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn open_images(paths: Vec<PathBuf>, app: AppHandle) -> CommandResult {
    let path = paths.first().ok_or("Keine Datei")?;
    actions::open_file(&app, path).map_err(text)
}

/// Schließt bzw. versteckt das aufrufende Fenster.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn close_window(window: tauri::Window, app: AppHandle) -> CommandResult {
    if windows::is_persistent(window.label()) {
        windows::hide(&app, &window).map_err(text)
    } else {
        window.destroy().map_err(text)
    }
}

#[tauri::command]
pub fn copy_text(text: &str) {
    crate::clipboard::write_text(text);
}

// Berechtigung

#[tauri::command]
pub async fn screen_access() -> ScreenAccess {
    permission::screen_access()
}

#[tauri::command]
pub fn open_screen_access_settings() -> CommandResult {
    permission::open_screen_access_settings().map_err(text)
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri-Commands erhalten Extraktoren per Wert"
)]
pub fn restart_app(app: AppHandle) {
    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_uri_components() {
        assert_eq!(percent_decode("%7B%22a%22%3A1%7D"), r#"{"a":1}"#);
        assert_eq!(percent_decode("Bilder%2F%C3%84rger"), "Bilder/Ärger");
        assert_eq!(percent_decode("100%"), "100%");
    }
}
