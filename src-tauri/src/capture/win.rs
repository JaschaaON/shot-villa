//! Bildschirm- und Fensteraufnahme unter Windows über `xcap`.
//!
//! `xcap` arbeitet mit physischen Pixeln. Nach außen gibt dieses Modul wie auf
//! macOS logische Koordinaten (physisch geteilt durch die Skalierung) und die
//! Skalierung als „Pixel pro Punkt“.

use std::path::Path;
use std::sync::{Arc, mpsc};
use std::time::Duration;

use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use xcap::{Monitor, Window};

use super::{CaptureError, DisplayShot, Recording, Rect, WindowInfo, WindowShot};

const MIN_WINDOW_SIZE: u32 = 40;

fn system<E: std::fmt::Display>(error: E) -> CaptureError {
    CaptureError::System(error.to_string())
}

/// Physisches Rechteck in logische Punkte umrechnen.
fn logical(x: i32, y: i32, width: u32, height: u32, scale: f64) -> Rect {
    Rect {
        x: f64::from(x) / scale,
        y: f64::from(y) / scale,
        width: f64::from(width) / scale,
        height: f64::from(height) / scale,
    }
}

fn monitor_scale(monitor: &Monitor) -> f64 {
    monitor
        .scale_factor()
        .map(f64::from)
        .ok()
        .filter(|scale| *scale > 0.0)
        .unwrap_or(1.0)
}

fn monitor_frame(monitor: &Monitor) -> Result<Rect, CaptureError> {
    Ok(logical(
        monitor.x().map_err(system)?,
        monitor.y().map_err(system)?,
        monitor.width().map_err(system)?,
        monitor.height().map_err(system)?,
        monitor_scale(monitor),
    ))
}

fn capture_monitor(monitor: &Monitor) -> Result<DisplayShot, CaptureError> {
    Ok(DisplayShot {
        display_id: monitor.id().map_err(system)?,
        frame: monitor_frame(monitor)?,
        scale: monitor_scale(monitor),
        image: Arc::new(monitor.capture_image().map_err(system)?),
    })
}

/// Nimmt alle Bildschirme auf. Den Mauszeiger nimmt `xcap` nicht mit auf.
pub fn all_displays(_shows_cursor: bool) -> Result<Vec<DisplayShot>, CaptureError> {
    Monitor::all()
        .map_err(system)?
        .iter()
        .map(capture_monitor)
        .collect()
}

/// Nimmt den Bildschirm auf, auf dem der Mauszeiger steht.
pub fn display_under_cursor(_shows_cursor: bool) -> Result<DisplayShot, CaptureError> {
    let (x, y) = physical_cursor()?;
    let monitor = Monitor::from_point(x, y)
        .or_else(|_| primary_monitor())
        .map_err(|_| CaptureError::NoDisplay)?;
    capture_monitor(&monitor)
}

/// Nimmt einen bestimmten Bildschirm auf; fehlt er, den unter dem Mauszeiger.
pub fn display_by_id(id: u32, shows_cursor: bool) -> Result<DisplayShot, CaptureError> {
    let monitors = Monitor::all().map_err(system)?;
    match monitors
        .iter()
        .find(|m| m.id().is_ok_and(|other| other == id))
    {
        Some(monitor) => capture_monitor(monitor),
        None => display_under_cursor(shows_cursor),
    }
}

fn primary_monitor() -> Result<Monitor, CaptureError> {
    Monitor::all()
        .map_err(system)?
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .ok_or(CaptureError::NoDisplay)
}

/// Sichtbare Fenster anderer Apps, vorderstes zuerst.
pub fn windows() -> Result<Vec<WindowInfo>, CaptureError> {
    let own_pid = std::process::id();
    let mut infos: Vec<(i32, WindowInfo)> = Window::all()
        .map_err(system)?
        .iter()
        .filter(|w| !w.is_minimized().unwrap_or(true))
        .filter(|w| w.pid().is_ok_and(|pid| pid != own_pid))
        .filter_map(|w| Some((w.z().ok()?, window_info(w)?)))
        .filter(|(_, info)| info.frame.width > 0.0 && info.frame.height > 0.0)
        .collect();
    infos.sort_by_key(|(z, _)| std::cmp::Reverse(*z));
    Ok(infos.into_iter().map(|(_, info)| info).collect())
}

fn window_info(window: &Window) -> Option<WindowInfo> {
    let width = window.width().ok()?;
    let height = window.height().ok()?;
    if width < MIN_WINDOW_SIZE || height < MIN_WINDOW_SIZE {
        return None;
    }
    let scale = window
        .current_monitor()
        .map_or(1.0, |monitor| monitor_scale(&monitor));
    Some(WindowInfo {
        id: window.id().ok()?,
        frame: logical(window.x().ok()?, window.y().ok()?, width, height, scale),
        app: window.app_name().unwrap_or_default(),
        title: window.title().unwrap_or_default(),
        pid: i64::from(window.pid().ok()?),
    })
}

/// Das Fenster im Vordergrund.
pub fn frontmost_window() -> Result<WindowInfo, CaptureError> {
    let focused = Window::all()
        .map_err(system)?
        .iter()
        .find(|w| w.is_focused().unwrap_or(false))
        .and_then(window_info);
    match focused {
        Some(info) if info.pid != i64::from(std::process::id()) => Ok(info),
        _ => windows()?.into_iter().next().ok_or(CaptureError::NoWindow),
    }
}

/// Nimmt ein einzelnes Fenster auf.
pub fn window(id: u32, _shows_cursor: bool) -> Result<WindowShot, CaptureError> {
    let windows = Window::all().map_err(system)?;
    let window = windows
        .iter()
        .find(|w| w.id().is_ok_and(|other| other == id))
        .ok_or(CaptureError::NoWindow)?;
    let scale = window
        .current_monitor()
        .map_or(1.0, |monitor| monitor_scale(&monitor));
    Ok(WindowShot {
        scale,
        image: Arc::new(window.capture_image().map_err(system)?),
    })
}

fn physical_cursor() -> Result<(i32, i32), CaptureError> {
    let mut point = POINT::default();
    // SAFETY: `point` ist ein gültiger, beschreibbarer POINT.
    unsafe { GetCursorPos(&raw mut point) }.map_err(system)?;
    Ok((point.x, point.y))
}

/// Startet eine Videoaufnahme (H.264 in MP4) vom Bereich `area` des Bildschirms
/// `display_id`; `area` in Punkten relativ zum Bildschirm. Nutzt Windows
/// Graphics Capture und den Media-Foundation-Encoder.
pub fn start_recording(
    display_id: u32,
    area: Rect,
    path: &Path,
) -> Result<Recording, CaptureError> {
    let monitor = Monitor::all()
        .map_err(system)?
        .into_iter()
        .find(|m| m.id().is_ok_and(|id| id == display_id))
        .ok_or(CaptureError::NoDisplay)?;
    let crop = PixelCrop::new(
        area,
        monitor_scale(&monitor),
        monitor.width().map_err(system)?,
        monitor.height().map_err(system)?,
    );
    let source = recording::capture_monitor(display_id)?;
    let (started_sender, started) = mpsc::channel();
    let (stop, stop_requested) = mpsc::channel::<()>();
    let (finished_sender, finished) = mpsc::channel();
    let path = path.to_owned();
    // Die Aufnahme bleibt in diesem Thread, bis sie endet.
    std::thread::spawn(move || {
        let control = match recording::start(source, crop, path) {
            Ok(control) => {
                let _ = started_sender.send(Ok(()));
                control
            }
            Err(error) => {
                let _ = started_sender.send(Err(error));
                return;
            }
        };
        // Endet auch, wenn die Gegenseite verschwindet.
        let _ = stop_requested.recv();
        let _ = finished_sender.send(recording::finish(control));
    });
    started
        .recv_timeout(START_TIMEOUT)
        .map_err(|_| CaptureError::Timeout)??;
    Ok(Recording::new(stop, finished))
}

const START_TIMEOUT: Duration = Duration::from_secs(10);

/// Ausschnitt eines Bildschirms in physischen Pixeln; Breite und Höhe gerade,
/// wie es H.264 verlangt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelCrop {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl PixelCrop {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Werte sind auf den Bildschirm begrenzt"
    )]
    fn new(area: Rect, scale: f64, screen_width: u32, screen_height: u32) -> Self {
        let pixel = |points: f64| (points * scale).round().max(0.0) as u32;
        let x = pixel(area.x).min(screen_width.saturating_sub(2));
        let y = pixel(area.y).min(screen_height.saturating_sub(2));
        let even = |value: u32| (value / 2 * 2).max(2);
        Self {
            x,
            y,
            width: even(pixel(area.width).min(screen_width - x)),
            height: even(pixel(area.height).min(screen_height - y)),
        }
    }
}

/// Windows Graphics Capture: Bildschirm aufnehmen, Bereich ausschneiden, kodieren.
mod recording {
    use std::path::PathBuf;

    use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
    use windows_capture::encoder::{
        AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder,
    };
    use windows_capture::frame::Frame;
    use windows_capture::graphics_capture_api::{GraphicsCaptureApi, InternalCaptureControl};
    use windows_capture::monitor::Monitor as CaptureMonitor;
    use windows_capture::settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    };

    use super::{CaptureError, PixelCrop, system};

    const FRAMES_PER_SECOND: u32 = 30;

    pub struct Flags {
        crop: PixelCrop,
        path: PathBuf,
    }

    pub struct Recorder {
        crop: PixelCrop,
        encoder: Option<VideoEncoder>,
        /// Zwischenspeicher für die Zeilen ohne Auffüllung.
        rows: Vec<u8>,
        /// Bild von unten nach oben, so wie der Encoder es für Puffer erwartet.
        flipped: Vec<u8>,
    }

    impl GraphicsCaptureApiHandler for Recorder {
        type Flags = Flags;
        type Error = String;

        fn new(context: Context<Self::Flags>) -> Result<Self, Self::Error> {
            let Flags { crop, path } = context.flags;
            let encoder = VideoEncoder::new(
                VideoSettingsBuilder::new(crop.width, crop.height).frame_rate(FRAMES_PER_SECOND),
                AudioSettingsBuilder::default().disabled(true),
                ContainerSettingsBuilder::default(),
                path,
            )
            .map_err(|error| error.to_string())?;
            Ok(Self {
                crop,
                encoder: Some(encoder),
                rows: Vec::new(),
                flipped: Vec::new(),
            })
        }

        fn on_frame_arrived(
            &mut self,
            frame: &mut Frame,
            _control: InternalCaptureControl,
        ) -> Result<(), Self::Error> {
            let timestamp = frame
                .timestamp()
                .map_err(|error| error.to_string())?
                .Duration;
            let PixelCrop {
                x,
                y,
                width,
                height,
            } = self.crop;
            // Ändert sich die Bildschirmauflösung, passt der Ausschnitt nicht mehr.
            if x + width > frame.width() || y + height > frame.height() {
                return Ok(());
            }
            let buffer = frame
                .buffer_crop(x, y, x + width, y + height)
                .map_err(|error| error.to_string())?;
            let pixels = buffer.as_nopadding_buffer(&mut self.rows);
            flip_rows(pixels, width as usize * 4, &mut self.flipped);
            if let Some(encoder) = self.encoder.as_mut() {
                encoder
                    .send_frame_buffer(&self.flipped, timestamp)
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        }
    }

    /// Kehrt die Zeilenreihenfolge um (oben nach unten ↔ unten nach oben).
    pub fn flip_rows(pixels: &[u8], row_length: usize, target: &mut Vec<u8>) {
        target.clear();
        target.extend(pixels.chunks_exact(row_length).rev().flatten());
    }

    /// Bildschirm mit dem Handle `display_id` als Quelle der Aufnahme.
    pub fn capture_monitor(display_id: u32) -> Result<CaptureMonitor, CaptureError> {
        CaptureMonitor::enumerate()
            .map_err(system)?
            .into_iter()
            // Die Kennung ist das HMONITOR-Handle, gekürzt auf 32 Bit (wie bei `xcap`).
            .find(|monitor| hmonitor_id(monitor) == display_id)
            .ok_or(CaptureError::NoDisplay)
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "gleiche Kürzung wie die Bildschirm-Kennung von `xcap`"
    )]
    fn hmonitor_id(monitor: &CaptureMonitor) -> u32 {
        monitor.as_raw_hmonitor() as usize as u32
    }

    pub fn start(
        monitor: CaptureMonitor,
        crop: PixelCrop,
        path: PathBuf,
    ) -> Result<CaptureControl<Recorder, String>, CaptureError> {
        // Mauszeiger mitnehmen und den gelben Rand weglassen, wo Windows das erlaubt.
        let cursor = if GraphicsCaptureApi::is_cursor_settings_supported().unwrap_or(false) {
            CursorCaptureSettings::WithCursor
        } else {
            CursorCaptureSettings::Default
        };
        let border = if GraphicsCaptureApi::is_border_settings_supported().unwrap_or(false) {
            DrawBorderSettings::WithoutBorder
        } else {
            DrawBorderSettings::Default
        };
        let settings = Settings::new(
            monitor,
            cursor,
            border,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Bgra8,
            Flags { crop, path },
        );
        Recorder::start_free_threaded(settings).map_err(system)
    }

    /// Stoppt die Aufnahme und schreibt die Datei fertig.
    pub fn finish(control: CaptureControl<Recorder, String>) -> Result<(), CaptureError> {
        let recorder = control.callback();
        control.stop().map_err(system)?;
        let encoder = recorder.lock().encoder.take();
        encoder.map_or(Ok(()), |encoder| encoder.finish().map_err(system))
    }
}

/// Mausposition in logischen Punkten.
pub fn cursor_location() -> (f64, f64) {
    let Ok((x, y)) = physical_cursor() else {
        return (0.0, 0.0);
    };
    let scale = Monitor::from_point(x, y).map_or(1.0, |monitor| monitor_scale(&monitor));
    (f64::from(x) / scale, f64::from(y) / scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_is_even_and_inside_screen() {
        let area = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.5,
            height: 3000.0,
        };
        assert_eq!(
            PixelCrop::new(area, 1.5, 1920, 1080),
            PixelCrop {
                x: 15,
                y: 30,
                width: 150,
                height: 1050,
            }
        );
    }

    #[test]
    fn flips_rows_bottom_up() {
        let mut target = Vec::new();
        recording::flip_rows(&[1, 2, 3, 4, 5, 6], 2, &mut target);
        assert_eq!(target, vec![5, 6, 3, 4, 1, 2]);
    }

    #[test]
    fn converts_physical_to_logical() {
        assert_eq!(
            logical(3840, 0, 2880, 1800, 2.0),
            Rect {
                x: 1920.0,
                y: 0.0,
                width: 1440.0,
                height: 900.0
            }
        );
    }
}
