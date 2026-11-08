//! Suche nach neuen Versionen und deren Installation.
//!
//! Der Tauri-Updater ruft die statische `latest.json` aus der Release-Adresse in
//! `tauri.conf.json` ab (eine Datei, keine API mit Abruflimit). Das Paket ist mit
//! minisign signiert; der Updater prüft die Signatur gegen den öffentlichen
//! Schlüssel der App, bevor er etwas installiert.

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{output, settings, tray};

/// Pause nach dem Start, damit die erste Suche den Start nicht bremst.
const STARTUP_DELAY: Duration = Duration::from_secs(20);
/// Abstand der Hintergrundsuchen, solange die App läuft.
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// Ereignis mit dem Fortschritt des Downloads.
const PROGRESS_EVENT: &str = "update-progress";

/// Die gefundene Version und welche Version schon gemeldet wurde.
#[derive(Default)]
pub struct Updates {
    found: Mutex<Option<Update>>,
    announced: Mutex<Announced>,
}

impl Updates {
    fn found(&self) -> Option<Update> {
        self.found
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn set_found(&self, update: Option<Update>) {
        *self.found.lock().unwrap_or_else(PoisonError::into_inner) = update;
    }

    /// Version der gefundenen Aktualisierung, falls es eine gibt.
    pub fn found_version(&self) -> Option<String> {
        self.found().map(|update| update.version)
    }
}

/// Merkt sich die zuletzt gemeldete Version, damit dieselbe Mitteilung nicht
/// bei jeder Suche wieder erscheint.
#[derive(Default)]
struct Announced(Option<String>);

impl Announced {
    /// `true`, wenn diese Version noch nicht gemeldet wurde.
    fn first_time(&mut self, version: &str) -> bool {
        if self.0.as_deref() == Some(version) {
            return false;
        }
        self.0 = Some(version.to_owned());
        true
    }
}

/// Eine verfügbare Version, wie das Frontend sie braucht.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    downloaded: u64,
    total: Option<u64>,
}

/// Fragt den Update-Server und merkt sich eine neuere Version.
pub async fn check(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    let update = app
        .updater()
        .map_err(search_failed)?
        .check()
        .await
        .map_err(search_failed)?;
    let info = update.as_ref().map(|update| UpdateInfo {
        version: update.version.clone(),
    });
    app.state::<Updates>().set_found(update);
    refresh_tray(app);
    Ok(info)
}

/// Lädt die gefundene Version, installiert sie und startet die App neu.
/// Unter Windows beendet der Installer die App selbst und startet sie wieder.
pub async fn install(app: &AppHandle) -> Result<(), String> {
    let update = app
        .state::<Updates>()
        .found()
        .ok_or("Es liegt kein Update vor. Bitte zuerst nach Updates suchen.")?;
    let mut downloaded: u64 = 0;
    update
        .download_and_install(
            |chunk, total| {
                downloaded += chunk as u64;
                let progress = Progress { downloaded, total };
                if let Err(error) = app.emit(PROGRESS_EVENT, progress) {
                    log::warn!("Fortschritt nicht gemeldet: {error}");
                }
            },
            || {},
        )
        .await
        .map_err(|error| format!("Das Update konnte nicht installiert werden: {error}"))?;
    app.restart()
}

/// Sucht beim Start und danach täglich im Hintergrund, solange die Einstellung
/// an ist, und meldet eine neue Version einmal per Mitteilung.
pub fn watch(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(STARTUP_DELAY);
        loop {
            if app.state::<settings::Store>().get().auto_update_check {
                background_check(&app);
            }
            std::thread::sleep(CHECK_INTERVAL);
        }
    });
}

fn background_check(app: &AppHandle) {
    match tauri::async_runtime::block_on(check(app)) {
        Ok(Some(info)) => {
            log::info!("Neue Version {} gefunden", info.version);
            let first_time = app
                .state::<Updates>()
                .announced
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .first_time(&info.version);
            if first_time {
                output::notify(
                    app,
                    "Update für Pivolo",
                    &format!(
                        "Version {} ist verfügbar. Öffne das Menü und wähle „Update installieren“.",
                        info.version
                    ),
                );
            }
        }
        Ok(None) => log::info!("Pivolo ist auf dem neuesten Stand"),
        Err(error) => log::info!("Update-Suche ohne Ergebnis: {error}"),
    }
}

/// Zeigt im Menü den Eintrag passend zur gefundenen Version.
fn refresh_tray(app: &AppHandle) {
    let hotkeys = app.state::<settings::Store>().get().hotkeys;
    if let Err(error) = tray::refresh(app, &hotkeys) {
        log::warn!("Menü nicht aktualisiert: {error}");
    }
}

/// Fehlertext für die Anzeige; die häufigen Fälle auf Deutsch.
fn search_failed(error: tauri_plugin_updater::Error) -> String {
    use tauri_plugin_updater::Error;
    match error {
        Error::ReleaseNotFound => {
            "Der Update-Server hat keine Versionsinformation geliefert. Bitte später erneut versuchen."
                .to_owned()
        }
        Error::Reqwest(_) => {
            "Der Update-Server ist nicht erreichbar. Bitte die Internetverbindung prüfen.".to_owned()
        }
        other => format!("Die Update-Suche ist fehlgeschlagen: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announces_each_version_once() {
        let mut announced = Announced::default();
        assert!(announced.first_time("0.4.0"));
        assert!(!announced.first_time("0.4.0"));
        assert!(announced.first_time("0.4.1"));
    }

    #[test]
    fn progress_serializes_in_camel_case() -> serde_json::Result<()> {
        let json = serde_json::to_string(&Progress {
            downloaded: 10,
            total: None,
        })?;
        assert_eq!(json, r#"{"downloaded":10,"total":null}"#);
        Ok(())
    }

    #[test]
    fn no_update_found_by_default() {
        assert_eq!(Updates::default().found_version(), None);
    }
}
