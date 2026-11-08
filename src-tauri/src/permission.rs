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
pub use mac::{
    PERMISSION_EPOCH, open_screen_access_settings, request_screen_access, reset_grants,
    screen_access,
};

#[cfg(target_os = "macos")]
mod mac {
    use objc2_core_graphics::{CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess};

    use super::ScreenAccess;

    const SETTINGS_URL: &str =
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";

    /// Stand der Signaturidentität. macOS knüpft Freigaben an die Signatur der App.
    /// Wechselt sie (etwa von ad hoc auf ein festes Zertifikat), steht der Haken
    /// in den Systemeinstellungen noch, gilt aber für die alte Identität und
    /// greift nicht. Wird die Zahl erhöht, setzt die App ihre Freigaben beim
    /// nächsten Start einmal zurück, damit macOS neu fragt und die Freigabe
    /// danach für alle weiteren Updates reicht.
    pub const PERMISSION_EPOCH: u32 = 1;

    /// Freigaben, die Pivolo nutzt: Bildschirmaufnahme, Bedienungshilfen und
    /// das Senden von Tastendrücken (Einfügen in T3 Code).
    const SERVICES: [&str; 3] = ["ScreenCapture", "Accessibility", "PostEvent"];

    /// Setzt die Freigaben der App mit `identifier` zurück (`tccutil reset`).
    pub fn reset_grants(identifier: &str) {
        for service in SERVICES {
            match std::process::Command::new("/usr/bin/tccutil")
                .args(["reset", service, identifier])
                .output()
            {
                Ok(output) if output.status.success() => {}
                Ok(output) => log::warn!(
                    "Freigabe {service} nicht zurückgesetzt: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
                Err(error) => log::warn!("tccutil nicht ausführbar: {error}"),
            }
        }
    }

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
