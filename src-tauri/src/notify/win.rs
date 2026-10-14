use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Unter Windows braucht es keine Vorbereitung.
pub fn init() {}

pub fn show(app: &AppHandle, title: &str, body: &str) {
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        log::warn!("Mitteilung fehlgeschlagen: {error}");
    }
}
