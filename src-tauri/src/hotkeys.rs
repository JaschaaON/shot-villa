//! Globale Tastenkürzel aus den Einstellungen.

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::actions::{self, Action};
use crate::settings::Hotkeys;

/// Kürzel und zugehörige Aktion, in der Reihenfolge der Einstellungen.
pub fn bindings(hotkeys: &Hotkeys) -> Vec<(Action, &str)> {
    [
        (Action::Fullscreen, &hotkeys.fullscreen),
        (Action::Area, &hotkeys.area),
        (Action::RepeatArea, &hotkeys.repeat_area),
        (Action::AnyWindow, &hotkeys.any_window),
        (Action::ActiveWindow, &hotkeys.active_window),
        (Action::Ocr, &hotkeys.ocr),
        (Action::Record, &hotkeys.record),
        (Action::ShowApp, &hotkeys.show_app),
    ]
    .into_iter()
    .filter_map(|(action, shortcut)| shortcut.as_deref().map(|s| (action, s)))
    .collect()
}

/// Registriert alle Kürzel neu. Liefert Meldungen zu Kürzeln, die nicht
/// registriert werden konnten (ungültig oder von anderer App belegt).
pub fn apply(app: &AppHandle, hotkeys: &Hotkeys) -> Vec<String> {
    let shortcuts = app.global_shortcut();
    if let Err(error) = shortcuts.unregister_all() {
        log::warn!("Kürzel konnten nicht gelöst werden: {error}");
    }
    bindings(hotkeys)
        .into_iter()
        .filter_map(|(action, shortcut)| {
            shortcuts
                .on_shortcut(shortcut, move |app, _, event| {
                    if event.state == ShortcutState::Pressed {
                        actions::run(app, action);
                    }
                })
                .err()
                .map(|error| format!("{shortcut}: {error}"))
        })
        .collect()
}

/// Löst alle Kürzel, z. B. während ein neues Kürzel aufgenommen wird.
pub fn suspend(app: &AppHandle) {
    if let Err(error) = app.global_shortcut().unregister_all() {
        log::warn!("Kürzel konnten nicht gelöst werden: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_unset_shortcuts() {
        let hotkeys = Hotkeys {
            fullscreen: Some("CmdOrCtrl+Shift+1".into()),
            area: None,
            repeat_area: None,
            any_window: Some("CmdOrCtrl+Shift+5".into()),
            active_window: None,
            ocr: None,
            record: Some("Control+Alt+CmdOrCtrl+R".into()),
            show_app: None,
        };
        assert_eq!(
            bindings(&hotkeys),
            vec![
                (Action::Fullscreen, "CmdOrCtrl+Shift+1"),
                (Action::AnyWindow, "CmdOrCtrl+Shift+5"),
                (Action::Record, "Control+Alt+CmdOrCtrl+R")
            ]
        );
    }
}
