//! Bildschirmgeometrie in globalen Punkten (Ursprung oben links).

use crate::capture::Rect;

/// Rahmen des Bildschirms, auf dem die Mitte des Rechtecks liegt.
pub fn frame_containing(rect: Rect) -> Rect {
    frame_at(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

/// Rahmen des Bildschirms, der den Punkt enthält; sonst der Hauptbildschirm.
#[cfg(target_os = "macos")]
pub fn frame_at(x: f64, y: f64) -> Rect {
    use objc2_core_foundation::CGPoint;
    use objc2_core_graphics::{CGDisplayBounds, CGGetDisplaysWithPoint, CGMainDisplayID};

    let mut display = 0;
    let mut count = 0;
    // SAFETY: Platz für genau einen Bildschirm, beide Zeiger sind gültig.
    let error =
        unsafe { CGGetDisplaysWithPoint(CGPoint::new(x, y), 1, &raw mut display, &raw mut count) };
    let display = if error.0 == 0 && count > 0 {
        display
    } else {
        CGMainDisplayID()
    };
    CGDisplayBounds(display).into()
}

/// Rahmen des Bildschirms, der den Punkt enthält; sonst der Hauptbildschirm.
#[cfg(windows)]
pub fn frame_at(x: f64, y: f64) -> Rect {
    const FALLBACK: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 1280.0,
        height: 800.0,
    };
    let frames: Vec<(Rect, bool)> = xcap::Monitor::all()
        .unwrap_or_default()
        .iter()
        .filter_map(|monitor| {
            let scale = f64::from(monitor.scale_factor().ok()?).max(1.0);
            let frame = Rect {
                x: f64::from(monitor.x().ok()?) / scale,
                y: f64::from(monitor.y().ok()?) / scale,
                width: f64::from(monitor.width().ok()?) / scale,
                height: f64::from(monitor.height().ok()?) / scale,
            };
            Some((frame, monitor.is_primary().unwrap_or(false)))
        })
        .collect();
    frames
        .iter()
        .find(|(frame, _)| frame.contains(x, y))
        .or_else(|| frames.iter().find(|(_, primary)| *primary))
        .map_or(FALLBACK, |(frame, _)| *frame)
}
