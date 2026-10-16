//! Menüleisten-Icon mit allen Aktionen.

use tauri::{
    AppHandle, Wry, include_image,
    menu::{Menu, MenuBuilder, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use std::time::Duration;

use tauri::Manager;

use crate::actions::{Action, RecordingPhase, Session};
use crate::settings::Hotkeys;

const TRAY_ID: &str = "main";

pub fn create(app: &AppHandle, hotkeys: &Hotkeys) -> tauri::Result<()> {
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tray_icon())
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Shot-VILLA")
        .menu(&menu(app, hotkeys)?)
        .build(app)?;
    Ok(())
}

/// macOS färbt das schwarze Vorlagen-Symbol passend zur Menüleiste ein; unter
/// Windows wäre es auf dunkler Taskleiste unsichtbar, dort steht das App-Symbol.
fn tray_icon() -> tauri::image::Image<'static> {
    if cfg!(target_os = "macos") {
        include_image!("icons/tray.png")
    } else {
        include_image!("icons/32x32.png")
    }
}

/// Baut das Menü neu, damit es geänderte Kürzel anzeigt.
pub fn refresh(app: &AppHandle, hotkeys: &Hotkeys) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu(app, hotkeys)?))?;
    }
    Ok(())
}

/// Zeigt die Laufzeit der Bildschirmaufnahme neben dem Symbol (nur macOS).
pub fn show_recording_time(app: &AppHandle, elapsed: Option<Duration>) {
    if let Some(tray) = app.tray_by_id(TRAY_ID)
        && let Err(error) = tray.set_title(elapsed.map(format_elapsed).as_deref())
    {
        log::warn!("Laufzeit nicht angezeigt: {error}");
    }
}

/// „● 1:05“ – Minuten und Sekunden der laufenden Aufnahme.
fn format_elapsed(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    format!("● {}:{:02}", seconds / 60, seconds % 60)
}

fn menu(app: &AppHandle, hotkeys: &Hotkeys) -> tauri::Result<Menu<Wry>> {
    let item = |action: Action, title: &str, shortcut: Option<&String>| {
        MenuItem::with_id(app, action.id(), title, true, shortcut.map(String::as_str))
    };
    let record_title = match app.state::<Session>().recording_phase() {
        RecordingPhase::Idle => "Bildschirmaufnahme (Bereich) …",
        RecordingPhase::Ready => "Bildschirmaufnahme starten",
        RecordingPhase::Running => "Bildschirmaufnahme beenden",
    };
    MenuBuilder::new(app)
        .item(&item(
            Action::Record,
            record_title,
            hotkeys.record.as_ref(),
        )?)
        .separator()
        .item(&item(
            Action::Fullscreen,
            "Vollbild aufnehmen",
            hotkeys.fullscreen.as_ref(),
        )?)
        .item(&item(
            Action::Area,
            "Bereich aufnehmen",
            hotkeys.area.as_ref(),
        )?)
        .item(&item(
            Action::RepeatArea,
            "Bereich wiederholen",
            hotkeys.repeat_area.as_ref(),
        )?)
        .item(&item(
            Action::ActiveWindow,
            "Aktives Fenster aufnehmen",
            hotkeys.active_window.as_ref(),
        )?)
        .item(&item(
            Action::AnyWindow,
            "Fenster wählen …",
            hotkeys.any_window.as_ref(),
        )?)
        .separator()
        .item(&item(
            Action::Ocr,
            "Text / QR-Code erkennen",
            hotkeys.ocr.as_ref(),
        )?)
        .item(&item(
            Action::LoadClipboard,
            "Bild aus Zwischenablage",
            None,
        )?)
        .item(&item(Action::OpenFile, "Bild öffnen …", None)?)
        .separator()
        .item(&item(
            Action::ShowApp,
            "Editor zeigen",
            hotkeys.show_app.as_ref(),
        )?)
        .item(&item(Action::Settings, "Einstellungen …", None)?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("Shot-VILLA beenden"))?)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_shows_minutes_and_seconds() {
        assert_eq!(format_elapsed(Duration::from_secs(5)), "● 0:05");
        assert_eq!(format_elapsed(Duration::from_secs(125)), "● 2:05");
    }
}
