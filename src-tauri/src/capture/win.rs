//! Bildschirm- und Fensteraufnahme unter Windows über `xcap`.
//!
//! `xcap` arbeitet mit physischen Pixeln. Nach außen gibt dieses Modul wie auf
//! macOS logische Koordinaten (physisch geteilt durch die Skalierung) und die
//! Skalierung als „Pixel pro Punkt“.

use std::sync::Arc;

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

/// Videoaufnahmen gibt es unter Windows noch nicht.
pub const fn start_recording(
    _display_id: u32,
    _area: Rect,
    _path: &std::path::Path,
) -> Result<Recording, CaptureError> {
    Err(CaptureError::Unsupported(
        "Bildschirmaufnahmen gibt es bisher nur unter macOS",
    ))
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
