//! App-Menü. Editor-Befehle gehen als Ereignis `menu` an das vorderste Fenster,
//! damit ⌘Z, ⌘S usw. beim Editor ankommen statt bei der Web-Ansicht.

use tauri::{AppHandle, Emitter, Manager, menu::MenuEvent};

use crate::actions::{self, Action};

/// Präfix für Menüpunkte, die das Frontend behandelt.
const FRONTEND: &str = "menu:";

#[cfg(target_os = "macos")]
pub fn build(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{AboutMetadata, MenuBuilder, MenuItem, PredefinedMenuItem, SubmenuBuilder};

    let frontend = |id: &str, title: &str, accelerator: Option<&str>| {
        MenuItem::with_id(app, format!("{FRONTEND}{id}"), title, true, accelerator)
    };
    let action = |action: Action, title: &str, accelerator: Option<&str>| {
        MenuItem::with_id(app, action.id(), title, true, accelerator)
    };

    let app_menu = SubmenuBuilder::new(app, "Screeni")
        .item(&PredefinedMenuItem::about(
            app,
            Some("Über Screeni"),
            Some(AboutMetadata::default()),
        )?)
        .separator()
        .item(&action(
            Action::Settings,
            "Einstellungen …",
            Some("CmdOrCtrl+,"),
        )?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, Some("Screeni ausblenden"))?)
        .item(&PredefinedMenuItem::hide_others(
            app,
            Some("Andere ausblenden"),
        )?)
        .item(&PredefinedMenuItem::show_all(app, Some("Alle einblenden"))?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("Screeni beenden"))?)
        .build()?;

    let file = SubmenuBuilder::new(app, "Ablage")
        .item(&frontend(
            "copy-image",
            "Bild kopieren",
            Some("CmdOrCtrl+Shift+C"),
        )?)
        .item(&frontend("save", "Sichern", Some("CmdOrCtrl+S"))?)
        .item(&frontend(
            "save-as",
            "Sichern unter …",
            Some("CmdOrCtrl+Shift+S"),
        )?)
        .item(&frontend("pin", "Anpinnen", Some("CmdOrCtrl+P"))?)
        .item(&frontend(
            "ocr",
            "Text erkennen",
            Some("CmdOrCtrl+Shift+T"),
        )?)
        .separator()
        .item(&action(Action::OpenFile, "Öffnen …", Some("CmdOrCtrl+O"))?)
        .item(&action(
            Action::LoadClipboard,
            "Aus Zwischenablage",
            Some("CmdOrCtrl+Shift+V"),
        )?)
        .separator()
        .item(&PredefinedMenuItem::close_window(app, Some("Schließen"))?)
        .build()?;

    let edit = SubmenuBuilder::new(app, "Bearbeiten")
        .item(&frontend("undo", "Widerrufen", Some("CmdOrCtrl+Z"))?)
        .item(&frontend("redo", "Wiederholen", Some("CmdOrCtrl+Shift+Z"))?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, Some("Ausschneiden"))?)
        .item(&PredefinedMenuItem::copy(app, Some("Kopieren"))?)
        .item(&PredefinedMenuItem::paste(app, Some("Einfügen"))?)
        .item(&PredefinedMenuItem::select_all(
            app,
            Some("Alles auswählen"),
        )?)
        .separator()
        .item(&frontend("duplicate", "Duplizieren", Some("CmdOrCtrl+D"))?)
        .item(&frontend("crop", "Auf Auswahl zuschneiden", None)?)
        .item(&frontend("reset-crop", "Zuschnitt zurücksetzen", None)?)
        .build()?;

    let view = SubmenuBuilder::new(app, "Darstellung")
        .item(&frontend("zoom-in", "Vergrößern", Some("CmdOrCtrl+="))?)
        .item(&frontend("zoom-out", "Verkleinern", Some("CmdOrCtrl+-"))?)
        .item(&frontend("zoom-fit", "Einpassen", Some("CmdOrCtrl+1"))?)
        .item(&frontend(
            "zoom-actual",
            "Originalgröße",
            Some("CmdOrCtrl+0"),
        )?)
        .separator()
        .item(&PredefinedMenuItem::fullscreen(app, Some("Vollbild"))?)
        .build()?;

    let window = SubmenuBuilder::new(app, "Fenster")
        .item(&PredefinedMenuItem::minimize(app, Some("Im Dock ablegen"))?)
        .build()?;

    MenuBuilder::new(app)
        .items(&[&app_menu, &file, &edit, &view, &window])
        .build()
}

/// Verteilt Menü-Ereignisse aus App-Menü und Menüleisten-Icon.
pub fn handle(app: &AppHandle, event: &MenuEvent) {
    let id = event.id().as_ref();
    if let Some(command) = id.strip_prefix(FRONTEND) {
        let target = app
            .webview_windows()
            .into_values()
            .find(|window| window.is_focused().unwrap_or(false));
        if let Some(window) = target
            && let Err(error) = window.emit_to(window.label(), "menu", command)
        {
            log::warn!("Menübefehl {command} nicht zugestellt: {error}");
        }
    } else if let Some(action) = Action::from_id(id) {
        actions::run(app, action);
    }
}
