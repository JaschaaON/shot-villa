//! Berechtigung „Bildschirmaufnahme“. Nur macOS fragt danach; unter Windows
//! ist die Aufnahme immer erlaubt.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScreenAccess {
    Granted,
    /// Nur macOS verlangt eine Freigabe.
    #[cfg(target_os = "macos")]
    Denied,
}

#[cfg(target_os = "macos")]
pub use mac::{open_screen_access_settings, request_screen_access, screen_access};

#[cfg(target_os = "macos")]
mod mac {
    use objc2_core_graphics::{CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess};

    use super::ScreenAccess;

    const SETTINGS_URL: &str =
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";

    /// Prüft, ob die App den Bildschirm aufnehmen darf. Der Preflight-Wert ist
    /// nach einer Freigabe bis zum Neustart veraltet; deshalb zählt zusätzlich, ob
    /// sich die Liste der aufnehmbaren Inhalte abrufen lässt.
    pub fn screen_access() -> ScreenAccess {
        if CGPreflightScreenCaptureAccess() || crate::capture::can_list_content() {
            ScreenAccess::Granted
        } else {
            ScreenAccess::Denied
        }
    }

    /// Löst beim ersten Aufruf den Systemdialog aus und trägt die App in die
    /// Liste unter Datenschutz → Bildschirmaufnahme ein. macOS übernimmt eine
    /// Erteilung erst nach einem Neustart der App.
    pub fn request_screen_access() -> ScreenAccess {
        if CGRequestScreenCaptureAccess() {
            ScreenAccess::Granted
        } else {
            ScreenAccess::Denied
        }
    }

    /// Öffnet Systemeinstellungen → Datenschutz → Bildschirmaufnahme.
    pub fn open_screen_access_settings() -> std::io::Result<()> {
        std::process::Command::new("open")
            .arg(SETTINGS_URL)
            .status()
            .map(|_| ())
    }
}

#[cfg(windows)]
pub const fn screen_access() -> ScreenAccess {
    ScreenAccess::Granted
}

#[cfg(windows)]
pub const fn request_screen_access() -> ScreenAccess {
    ScreenAccess::Granted
}

#[cfg(windows)]
#[expect(clippy::unnecessary_wraps, reason = "gleiche Signatur wie unter macOS")]
pub const fn open_screen_access_settings() -> std::io::Result<()> {
    Ok(())
}
