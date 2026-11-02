//! Bereich aufnehmen und direkt ins offene T3-Code-Fenster einfügen.
//!
//! T3 Code hängt ein Bild an die Nachricht an, sobald es im Fenster aus der
//! Zwischenablage eingefügt wird (⌘V bzw. Strg+V). Shot-VILLA legt die
//! Aufnahme deshalb in die Zwischenablage, holt das T3-Code-Fenster nach vorn
//! und löst dort das Einfügen aus. Abgesendet wird nichts.

use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::actions::ActionResult;
use crate::capture::{self, WindowInfo};
use crate::imaging::{self, Format, Image};
use crate::settings::{DEFAULT_T3_WINDOW, Store};
use crate::{clipboard, output};

/// Wartezeit, bis die Web-Ansicht des Fensters nach dem Nach-vorn-Holen
/// Tastatureingaben annimmt.
const SETTLE: Duration = Duration::from_millis(150);

#[derive(Debug, thiserror::Error)]
enum PasteError {
    /// macOS lässt Tastendrücke nur mit Freigabe unter „Bedienungshilfen“ zu.
    #[cfg(target_os = "macos")]
    #[error(
        "Shot-VILLA braucht die Freigabe unter Datenschutz & Sicherheit → Bedienungshilfen. Danach Shot-VILLA beenden und neu öffnen"
    )]
    NoPermission,
    #[error("Das Fenster ließ sich nicht nach vorn holen")]
    Activation,
}

/// Legt `image` in die Zwischenablage und fügt es ins T3-Code-Fenster ein.
/// Scheitert das Einfügen, bleibt das Bild in der Zwischenablage.
pub fn paste_image(app: &AppHandle, image: &Image, scale: f64) -> ActionResult {
    let png = imaging::encode(image, Format::Png, scale)?;
    clipboard::write_image(&png);

    let hint = app.state::<Store>().get().t3_window;
    let windows = capture::windows()?;
    let Some(target) = find_window(&windows, &hint) else {
        output::notify(
            app,
            "T3-Code-Fenster nicht gefunden",
            "Das Bild liegt in der Zwischenablage. Öffne T3 Code und füge es mit ⌘V bzw. Strg+V ein.",
        );
        return Ok(());
    };

    match platform::paste_into(target) {
        Ok(()) => Ok(()),
        Err(error) => {
            log::warn!("{error}");
            output::notify(
                app,
                "Nicht in T3 Code eingefügt",
                &format!("{error}. Das Bild liegt in der Zwischenablage."),
            );
            Ok(())
        }
    }
}

/// Das vorderste Fenster, dessen App- oder Fenstername `hint` enthält
/// (Groß-/Kleinschreibung egal). Ein leerer Name steht für „T3 Code“.
/// Die Liste ist von vorn nach hinten sortiert.
fn find_window<'a>(windows: &'a [WindowInfo], hint: &str) -> Option<&'a WindowInfo> {
    let hint = hint.trim();
    let hint = if hint.is_empty() {
        DEFAULT_T3_WINDOW
    } else {
        hint
    }
    .to_lowercase();
    windows.iter().find(|window| {
        window.app.to_lowercase().contains(&hint) || window.title.to_lowercase().contains(&hint)
    })
}

#[cfg(target_os = "macos")]
mod platform {
    use std::process::Command;
    use std::time::{Duration, Instant};

    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};
    use objc2_core_graphics::{
        CGEvent, CGEventFlags, CGEventTapLocation, CGPreflightPostEventAccess,
        CGRequestPostEventAccess,
    };

    use super::{PasteError, SETTLE};
    use crate::capture::WindowInfo;

    /// Tastencode von „V“ (`kVK_ANSI_V`).
    const KEY_V: u16 = 9;
    const POLL: Duration = Duration::from_millis(25);

    pub fn paste_into(window: &WindowInfo) -> Result<(), PasteError> {
        // Der Wert von `CGPreflightPostEventAccess` bleibt nach einer Freigabe bis zum
        // Neustart veraltet; die Abfrage der Bedienungshilfen ist sofort aktuell.
        if !(CGPreflightPostEventAccess() || accessibility_trusted()) {
            // Löst den Systemdialog aus und trägt die App in die Liste ein.
            CGRequestPostEventAccess();
            return Err(PasteError::NoPermission);
        }
        let pid = i32::try_from(window.pid).map_err(|_| PasteError::Activation)?;
        activate(pid)?;
        std::thread::sleep(SETTLE);
        post_paste();
        Ok(())
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        /// `Boolean` (0 oder 1): Ist die App für die Bedienungshilfen freigegeben?
        fn AXIsProcessTrusted() -> u8;
    }

    fn accessibility_trusted() -> bool {
        // SAFETY: Die Funktion hat keine Parameter und keine Nebenwirkungen.
        unsafe { AXIsProcessTrusted() != 0 }
    }

    /// Holt die App nach vorn; klappt das nicht direkt, über `LaunchServices`.
    fn activate(pid: i32) -> Result<(), PasteError> {
        let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
            .ok_or(PasteError::Activation)?;
        app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
        if wait_until_frontmost(pid, Duration::from_millis(500)) {
            return Ok(());
        }
        if let Some(identifier) = app.bundleIdentifier() {
            let _ = Command::new("open")
                .arg("-b")
                .arg(identifier.to_string())
                .status();
        }
        if wait_until_frontmost(pid, Duration::from_millis(1500)) {
            Ok(())
        } else {
            Err(PasteError::Activation)
        }
    }

    fn is_frontmost(pid: i32) -> bool {
        NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .is_some_and(|app| app.processIdentifier() == pid)
    }

    fn wait_until_frontmost(pid: i32, limit: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < limit {
            if is_frontmost(pid) {
                return true;
            }
            std::thread::sleep(POLL);
        }
        is_frontmost(pid)
    }

    /// Drückt und löst ⌘V.
    fn post_paste() {
        for down in [true, false] {
            let event = CGEvent::new_keyboard_event(None, KEY_V, down);
            CGEvent::set_flags(event.as_deref(), CGEventFlags::MaskCommand);
            CGEvent::post(CGEventTapLocation::HIDEventTap, event.as_deref());
            std::thread::sleep(Duration::from_millis(15));
        }
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::time::{Duration, Instant};

    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
        VIRTUAL_KEY, VK_CONTROL, VK_MENU, VK_V,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, IsIconic, SW_RESTORE, SetForegroundWindow, ShowWindow,
    };

    use super::{PasteError, SETTLE};
    use crate::capture::WindowInfo;

    const POLL: Duration = Duration::from_millis(25);

    pub fn paste_into(window: &WindowInfo) -> Result<(), PasteError> {
        // xcap nutzt das Fenster-Handle als Kennung.
        let handle = usize::try_from(window.id).map_err(|_| PasteError::Activation)?;
        let hwnd = HWND(std::ptr::with_exposed_provenance_mut::<c_void>(handle));
        bring_to_front(hwnd)?;
        std::thread::sleep(SETTLE);
        send(&[
            key(VK_CONTROL, false),
            key(VK_V, false),
            key(VK_V, true),
            key(VK_CONTROL, true),
        ]);
        Ok(())
    }

    fn is_foreground(hwnd: HWND) -> bool {
        // SAFETY: Fragt nur das aktuelle Vordergrundfenster ab.
        unsafe { GetForegroundWindow() == hwnd }
    }

    fn wait_until_foreground(hwnd: HWND, limit: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < limit {
            if is_foreground(hwnd) {
                return true;
            }
            std::thread::sleep(POLL);
        }
        is_foreground(hwnd)
    }

    fn bring_to_front(hwnd: HWND) -> Result<(), PasteError> {
        // SAFETY: `hwnd` ist ein Fenster-Handle; ein ungültiges lässt die
        // Aufrufe nur fehlschlagen.
        unsafe {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
            let _ = SetForegroundWindow(hwnd);
        }
        if wait_until_foreground(hwnd, Duration::from_millis(300)) {
            return Ok(());
        }
        // Windows lässt Hintergrundprozesse nur nach Eingabe in den Vordergrund.
        // Ein kurzer Alt-Tastendruck hebt diese Sperre auf.
        send(&[key(VK_MENU, false), key(VK_MENU, true)]);
        // SAFETY: Wie oben.
        let _ = unsafe { SetForegroundWindow(hwnd) };
        if wait_until_foreground(hwnd, Duration::from_millis(1500)) {
            Ok(())
        } else {
            Err(PasteError::Activation)
        }
    }

    fn key(code: VIRTUAL_KEY, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: code,
                    wScan: 0,
                    dwFlags: if up {
                        KEYEVENTF_KEYUP
                    } else {
                        KEYBD_EVENT_FLAGS(0)
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn send(inputs: &[INPUT]) {
        let Ok(size) = i32::try_from(size_of::<INPUT>()) else {
            return;
        };
        // SAFETY: `inputs` sind vollständig gefüllte Tastatureingaben.
        let sent = unsafe { SendInput(inputs, size) };
        if usize::try_from(sent).ok() != Some(inputs.len()) {
            log::warn!(
                "Tastatureingabe nur teilweise gesendet ({sent} von {})",
                inputs.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::Rect;

    fn window(app: &str, title: &str) -> WindowInfo {
        WindowInfo {
            id: 1,
            frame: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            app: app.into(),
            title: title.into(),
            pid: 1,
        }
    }

    #[test]
    fn finds_app_by_name_ignoring_case() {
        let windows = [window("Finder", "Dokumente"), window("T3 Code (Beta)", "")];
        assert_eq!(
            find_window(&windows, "t3 code").map(|w| w.app.as_str()),
            Some("T3 Code (Beta)")
        );
    }

    #[test]
    fn finds_browser_tab_by_title() {
        let windows = [window("Safari", "T3 Code (Beta)")];
        assert!(find_window(&windows, "T3 Code").is_some());
    }

    #[test]
    fn takes_the_frontmost_match() {
        let windows = [
            window("Chrome", "T3 Code – eins"),
            window("Chrome", "T3 Code – zwei"),
        ];
        assert_eq!(
            find_window(&windows, "T3 Code").map(|w| w.title.as_str()),
            Some("T3 Code – eins")
        );
    }

    #[test]
    fn empty_hint_means_default_name() {
        let windows = [window("Notes", ""), window("T3 Code", "")];
        assert_eq!(
            find_window(&windows, "  ").map(|w| w.app.as_str()),
            Some("T3 Code")
        );
    }

    #[test]
    fn no_match_yields_none() {
        assert!(find_window(&[window("Finder", "Dokumente")], "T3 Code").is_none());
    }
}
