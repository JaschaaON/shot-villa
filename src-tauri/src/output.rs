//! Ausgabe fertiger Bilder: Zwischenablage, Datei, Benachrichtigung.

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::imaging::{self, Format, Image, ImageError};
use crate::settings::{SaveFormat, Settings};
use crate::{clipboard, files, notify};

#[derive(Debug, thiserror::Error)]
pub enum OutputError {
    #[error(transparent)]
    Image(#[from] ImageError),
    #[error("Speichern fehlgeschlagen: {0}")]
    Io(#[from] std::io::Error),
}

/// Kopiert ein Bild in voller Auflösung in die Zwischenablage; die Pixeldichte
/// sagt anderen Apps, dass es ein Retina-Bild ist.
pub fn copy(image: &Image, scale: f64) -> Result<(), OutputError> {
    clipboard::write_image(&imaging::encode(image, Format::Png, scale)?);
    Ok(())
}

/// Speichert ein Bild im eingestellten Ordner, mit Format und Skalierung
/// aus den Einstellungen.
pub fn save(image: &Image, scale: f64, settings: &Settings) -> Result<PathBuf, OutputError> {
    let (bytes, format) = prepare(image, scale, settings)?;
    Ok(files::save_in_folder(
        &settings.save_folder(),
        &bytes,
        format.extension(),
    )?)
}

/// Speichert unter einem vom Nutzer gewählten Pfad. Die Endung bestimmt das Format.
pub fn save_as(
    image: &Image,
    scale: f64,
    settings: &Settings,
    path: &Path,
) -> Result<(), OutputError> {
    let format = match path.extension().and_then(|e| e.to_str()) {
        Some(e) if e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg") => Format::Jpeg,
        _ => Format::Png,
    };
    let (image, scale) = downscaled(image, scale, settings)?;
    files::write_screenshot(path, &imaging::encode(&image, format, scale)?)?;
    Ok(())
}

fn prepare(
    image: &Image,
    scale: f64,
    settings: &Settings,
) -> Result<(Vec<u8>, Format), OutputError> {
    let (image, scale) = downscaled(image, scale, settings)?;
    let format = match settings.save_format {
        SaveFormat::Png => Format::Png,
        SaveFormat::Auto => {
            if imaging::looks_photographic(&imaging::pixels(&image)?) {
                Format::Jpeg
            } else {
                Format::Png
            }
        }
    };
    Ok((imaging::encode(&image, format, scale)?, format))
}

/// Verkleinert Retina-Bilder auf 1x, wenn eingestellt; liefert die neue Skalierung.
fn downscaled(image: &Image, scale: f64, settings: &Settings) -> Result<(Image, f64), OutputError> {
    if settings.downscale_on_save && scale > 1.0 {
        let (width, height) = scaled_size(imaging::width(image), imaging::height(image), scale);
        Ok((imaging::resize(image, width, height)?, 1.0))
    } else {
        Ok((image.clone(), scale))
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn scaled_size(width: usize, height: usize, scale: f64) -> (usize, usize) {
    (
        ((width as f64 / scale).round() as usize).max(1),
        ((height as f64 / scale).round() as usize).max(1),
    )
}

pub fn notify(app: &AppHandle, title: &str, body: &str) {
    notify::show(app, title, body);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_size_halves_retina_images() {
        assert_eq!(scaled_size(3024, 1964, 2.0), (1512, 982));
        assert_eq!(scaled_size(1, 1, 2.0), (1, 1));
    }
}
