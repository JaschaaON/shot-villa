//! Texterkennung mit `Windows.Media.Ocr`, QR-Codes mit `rqrr`.

use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;

use super::{OcrError, Recognition};
use crate::imaging::Image;

fn failed<E: std::fmt::Display>(error: E) -> OcrError {
    OcrError(error.to_string())
}

pub fn recognize(image: &Image) -> Result<Recognition, OcrError> {
    Ok(Recognition {
        lines: recognize_text(image)?,
        codes: recognize_codes(image),
    })
}

/// Zeilen in Lesereihenfolge; Sprache aus den Windows-Spracheinstellungen.
fn recognize_text(image: &Image) -> Result<Vec<String>, OcrError> {
    let width = i32::try_from(image.width()).map_err(failed)?;
    let height = i32::try_from(image.height()).map_err(failed)?;
    let writer = DataWriter::new().map_err(failed)?;
    writer.WriteBytes(image.as_raw()).map_err(failed)?;
    let buffer = writer.DetachBuffer().map_err(failed)?;
    let rgba =
        SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Rgba8, width, height)
            .map_err(failed)?;
    let bitmap = SoftwareBitmap::Convert(&rgba, BitmapPixelFormat::Bgra8).map_err(failed)?;

    let engine = OcrEngine::TryCreateFromUserProfileLanguages().map_err(failed)?;
    let result = engine
        .RecognizeAsync(&bitmap)
        .map_err(failed)?
        .join()
        .map_err(failed)?;
    result
        .Lines()
        .map_err(failed)?
        .into_iter()
        .map(|line| line.Text().map(|text| text.to_string()).map_err(failed))
        .collect()
}

/// Inhalte aller lesbaren QR-Codes im Bild.
fn recognize_codes(image: &Image) -> Vec<String> {
    let width = usize::try_from(image.width()).unwrap_or(0);
    let height = usize::try_from(image.height()).unwrap_or(0);
    let mut prepared = rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| {
        let pixel = image.get_pixel(u32::try_from(x).unwrap_or(0), u32::try_from(y).unwrap_or(0));
        luminance(pixel.0)
    });
    prepared
        .detect_grids()
        .into_iter()
        .filter_map(|grid| grid.decode().ok().map(|(_, content)| content))
        .collect()
}

/// Helligkeit nach ITU-R BT.601.
fn luminance([r, g, b, _]: [u8; 4]) -> u8 {
    let value = (299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b)) / 1000;
    u8::try_from(value).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luminance_spans_black_to_white() {
        assert_eq!(luminance([0, 0, 0, 255]), 0);
        assert_eq!(luminance([255, 255, 255, 255]), 255);
    }
}
