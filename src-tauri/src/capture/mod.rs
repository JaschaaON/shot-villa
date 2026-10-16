//! Bildschirm- und Fensteraufnahme. Die Umsetzung steckt in `mac.rs`
//! (`ScreenCaptureKit`) bzw. `win.rs` (Windows Graphics Capture über `xcap`).
//!
//! Alle Funktionen blockieren, bis das System geantwortet hat. Sie dürfen
//! deshalb nicht auf dem Main-Thread laufen.

use std::sync::mpsc;
use std::time::Duration;

use serde::Serialize;

use crate::imaging::Image;

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::{
    all_displays, can_list_content, cursor_location, display_by_id, display_under_cursor,
    frontmost_window, start_recording, window, windows,
};

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::{
    all_displays, cursor_location, display_by_id, display_under_cursor, frontmost_window,
    start_recording, window, windows,
};

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("Bildschirmaufnahme: {0}")]
    System(String),
    #[error("Die Bildschirmaufnahme hat nicht rechtzeitig geantwortet")]
    Timeout,
    #[error("{0}")]
    Unsupported(&'static str),
    #[error("Kein Bildschirm gefunden")]
    NoDisplay,
    #[error("Kein passendes Fenster gefunden")]
    NoWindow,
}

/// Rechteck in Punkten (logischen Pixeln); globale Koordinaten, Ursprung oben
/// links am Hauptbildschirm.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, serde::Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn contains(self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Aufnahme eines ganzen Bildschirms.
pub struct DisplayShot {
    pub display_id: u32,
    pub frame: Rect,
    /// Pixel pro Punkt.
    pub scale: f64,
    pub image: Image,
}

/// Aufnahme eines einzelnen Fensters.
pub struct WindowShot {
    pub scale: f64,
    pub image: Image,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub id: u32,
    pub frame: Rect,
    pub app: String,
    pub title: String,
    /// Prozess des Fensters; bestimmt das „aktive Fenster“.
    #[serde(skip)]
    pub pid: i64,
}

impl WindowInfo {
    /// Dasselbe Fenster mit Koordinaten relativ zu `origin`.
    pub fn relative_to(&self, origin: Rect) -> Self {
        Self {
            frame: Rect {
                x: self.frame.x - origin.x,
                y: self.frame.y - origin.y,
                ..self.frame
            },
            ..self.clone()
        }
    }
}

/// Eine laufende Videoaufnahme. Ein eigener Thread hält die Systemobjekte;
/// `stop` beendet die Aufnahme und wartet, bis die Datei fertig geschrieben ist.
pub struct Recording {
    stop: mpsc::Sender<()>,
    finished: mpsc::Receiver<Result<(), CaptureError>>,
}

impl Recording {
    /// Längste Wartezeit, bis das Video nach dem Stopp fertig geschrieben ist.
    const FINISH_TIMEOUT: Duration = Duration::from_secs(30);

    #[cfg_attr(
        windows,
        expect(dead_code, reason = "Videoaufnahme gibt es nur unter macOS")
    )]
    fn new(stop: mpsc::Sender<()>, finished: mpsc::Receiver<Result<(), CaptureError>>) -> Self {
        Self { stop, finished }
    }

    pub fn stop(self) -> Result<(), CaptureError> {
        // Ist der Aufnahme-Thread schon beendet, liegt sein Ergebnis bereit.
        let _ = self.stop.send(());
        self.finished
            .recv_timeout(Self::FINISH_TIMEOUT)
            .map_err(|_| CaptureError::Timeout)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_uses_half_open_bounds() {
        let frame = Rect {
            x: 100.0,
            y: -200.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(frame.contains(100.0, -200.0));
        assert!(!frame.contains(150.0, -180.0));
        assert!(!frame.contains(120.0, -150.0));
    }

    #[test]
    fn relative_frames_subtract_origin() {
        let window = WindowInfo {
            id: 1,
            frame: Rect {
                x: 1600.0,
                y: 40.0,
                width: 300.0,
                height: 200.0,
            },
            app: String::new(),
            title: String::new(),
            pid: 0,
        };
        let display = Rect {
            x: 1512.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        assert_eq!(
            window.relative_to(display).frame,
            Rect {
                x: 88.0,
                y: 40.0,
                width: 300.0,
                height: 200.0
            }
        );
    }
}
