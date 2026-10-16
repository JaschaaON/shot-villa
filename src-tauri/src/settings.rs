//! Benutzereinstellungen, gespeichert als `settings.json` im App-Config-Ordner.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Hotkeys {
    pub fullscreen: Option<String>,
    pub area: Option<String>,
    pub repeat_area: Option<String>,
    pub any_window: Option<String>,
    pub active_window: Option<String>,
    pub ocr: Option<String>,
    pub record: Option<String>,
    pub show_app: Option<String>,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            fullscreen: Some("CmdOrCtrl+Shift+1".into()),
            area: Some("CmdOrCtrl+Shift+2".into()),
            repeat_area: None,
            any_window: None,
            active_window: None,
            ocr: Some("Control+Alt+CmdOrCtrl+O".into()),
            record: Some("Control+Alt+CmdOrCtrl+R".into()),
            show_app: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SaveFormat {
    Png,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AreaOpens {
    #[default]
    Editor,
    Thumbnail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThumbnailClosing {
    #[default]
    Manual,
    Automatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CursorCapture {
    /// Nur bei Vollbild-Aufnahmen.
    #[default]
    Auto,
    Include,
    Exclude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ColorFormat {
    #[default]
    Hex,
    Rgb,
    CssRgba,
    Oklch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AfterCapture {
    pub show: bool,
    pub copy: bool,
    pub save: bool,
}

impl Default for AfterCapture {
    fn default() -> Self {
        Self {
            show: true,
            copy: false,
            save: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Einstellungen sind unabhängige Schalter"
)]
pub struct Settings {
    pub hotkeys: Hotkeys,
    pub save_folder: Option<PathBuf>,
    pub save_format: SaveFormat,
    pub downscale_on_save: bool,
    pub after_capture: AfterCapture,
    pub area_opens: AreaOpens,
    pub thumbnail_closing: ThumbnailClosing,
    pub capture_cursor: CursorCapture,
    pub ocr_remove_line_breaks: bool,
    pub esc_copies: bool,
    pub esc_saves: bool,
    pub always_on_top: bool,
    pub prefer_actual_size: bool,
    pub color_format: ColorFormat,
    pub launch_at_login: bool,
    /// Begrüßung nach der Installation schon gezeigt?
    pub welcome_shown: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkeys: Hotkeys::default(),
            save_folder: None,
            save_format: SaveFormat::default(),
            downscale_on_save: false,
            after_capture: AfterCapture::default(),
            area_opens: AreaOpens::default(),
            thumbnail_closing: ThumbnailClosing::default(),
            capture_cursor: CursorCapture::default(),
            ocr_remove_line_breaks: false,
            esc_copies: true,
            esc_saves: false,
            always_on_top: false,
            prefer_actual_size: true,
            color_format: ColorFormat::default(),
            launch_at_login: false,
            welcome_shown: false,
        }
    }
}

impl Settings {
    /// Zielordner für gespeicherte Bilder; ohne Einstellung der Schreibtisch.
    pub fn save_folder(&self) -> PathBuf {
        self.save_folder.clone().unwrap_or_else(default_save_folder)
    }
}

/// Schreibtisch des Nutzers; unter Windows auch ein umgeleiteter (etwa in die Cloud).
fn default_save_folder() -> PathBuf {
    dirs::desktop_dir().unwrap_or_else(std::env::temp_dir)
}

/// Einstellungen samt Speicherort.
pub struct Store {
    path: PathBuf,
    current: Mutex<Settings>,
}

impl Store {
    /// Lädt die Einstellungen. Fehlt die Datei, werden die einer früheren
    /// Installation (`legacy`) übernommen und am neuen Ort gespeichert; sonst
    /// gelten die Vorgaben, ebenso bei einer kaputten Datei.
    pub fn load(path: PathBuf, legacy: Option<&Path>) -> Self {
        let current = read(&path)
            .or_else(|| {
                let settings = read(legacy?)?;
                if let Err(error) = write(&path, &settings) {
                    log::warn!("Übernommene Einstellungen nicht gespeichert: {error}");
                }
                Some(settings)
            })
            .unwrap_or_default();
        Self {
            path,
            current: Mutex::new(current),
        }
    }

    pub fn get(&self) -> Settings {
        self.lock().clone()
    }

    pub fn set(&self, settings: Settings) -> std::io::Result<()> {
        write(&self.path, &settings)?;
        *self.lock() = settings;
        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Settings> {
        self.current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn read(path: &Path) -> Option<Settings> {
    let text = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str(&text) {
        Ok(settings) => Some(settings),
        Err(error) => {
            log::warn!("Einstellungen unlesbar, nutze Vorgaben: {error}");
            None
        }
    }
}

fn write(path: &Path, settings: &Settings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    // Erst vollständig schreiben, dann ersetzen: kein halbes JSON nach Absturz.
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, json)?;
    std::fs::rename(temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("shot-villa-test-{}-{name}", std::process::id()))
            .join("settings.json")
    }

    #[test]
    fn takes_over_legacy_settings_once() -> std::io::Result<()> {
        let legacy = temp_path("legacy-old");
        let path = temp_path("legacy-new");
        let mut old = Settings::default();
        old.hotkeys.area = None;
        write(&legacy, &old)?;
        assert_eq!(Store::load(path.clone(), Some(&legacy)).get(), old);
        // Ab jetzt gilt die neue Datei, auch wenn sich die alte ändert.
        write(&legacy, &Settings::default())?;
        assert_eq!(Store::load(path, Some(&legacy)).get(), old);
        Ok(())
    }

    #[test]
    fn missing_file_yields_defaults() {
        let store = Store::load(temp_path("missing"), None);
        assert_eq!(store.get(), Settings::default());
    }

    #[test]
    fn round_trips_through_disk() -> std::io::Result<()> {
        let path = temp_path("roundtrip");
        let store = Store::load(path.clone(), None);
        let mut settings = store.get();
        settings.save_format = SaveFormat::Png;
        settings.hotkeys.area = None;
        store.set(settings.clone())?;
        assert_eq!(Store::load(path, None).get(), settings);
        Ok(())
    }

    #[test]
    fn partial_json_keeps_other_defaults() -> serde_json::Result<()> {
        let settings: Settings = serde_json::from_str(r#"{"escSaves": true}"#)?;
        assert!(settings.esc_saves);
        assert_eq!(settings.hotkeys, Hotkeys::default());
        Ok(())
    }

    #[test]
    fn default_folder_is_desktop() {
        assert!(Settings::default().save_folder().ends_with("Desktop"));
    }
}
