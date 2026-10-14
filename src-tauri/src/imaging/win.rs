//! Bildoperationen mit der `image`-Crate.

use std::io::Cursor;
use std::sync::Arc;

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{ImageFormat, RgbaImage};

use super::{Format, ImageError, JPEG_QUALITY, PixelRect, Pixels};

/// Ein Bild als RGBA-Puffer; Kopien teilen sich die Pixel.
pub type Image = Arc<RgbaImage>;

pub fn width(image: &Image) -> usize {
    usize::try_from(image.width()).unwrap_or(usize::MAX)
}

pub fn height(image: &Image) -> usize {
    usize::try_from(image.height()).unwrap_or(usize::MAX)
}

#[expect(clippy::unnecessary_wraps, reason = "gleiche Signatur wie unter macOS")]
pub fn pixels(image: &Image) -> Result<Pixels, ImageError> {
    Ok(Pixels {
        width: width(image),
        height: height(image),
        data: image.as_raw().clone(),
    })
}

/// Schneidet einen Bereich aus; er wird auf das Bild begrenzt.
pub fn crop(image: &Image, rect: PixelRect) -> Result<Image, ImageError> {
    let rect = rect.clamped(width(image), height(image));
    if rect.width == 0 || rect.height == 0 {
        return Err(ImageError::Graphics);
    }
    Ok(Arc::new(
        image::imageops::crop_imm(&**image, rect.x, rect.y, rect.width, rect.height).to_image(),
    ))
}

/// Skaliert ein Bild auf die angegebene Pixelgröße.
pub fn resize(image: &Image, width: usize, height: usize) -> Result<Image, ImageError> {
    let width = u32::try_from(width).map_err(|_| ImageError::Graphics)?;
    let height = u32::try_from(height).map_err(|_| ImageError::Graphics)?;
    Ok(Arc::new(image::imageops::resize(
        &**image,
        width,
        height,
        FilterType::Lanczos3,
    )))
}

/// Kodiert ein Bild; die Pixeldichte schreibt Windows nicht mit.
pub fn encode(image: &Image, format: Format, _scale: f64) -> Result<Vec<u8>, ImageError> {
    let mut bytes = Cursor::new(Vec::new());
    match format {
        Format::Png => image
            .write_to(&mut bytes, ImageFormat::Png)
            .map_err(|_| ImageError::Encoding)?,
        Format::Jpeg => {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let quality = (JPEG_QUALITY * 100.0).round() as u8;
            // JPEG kennt keinen Alphakanal.
            let rgb = image::DynamicImage::ImageRgba8((**image).clone()).into_rgb8();
            JpegEncoder::new_with_quality(&mut bytes, quality)
                .encode_image(&rgb)
                .map_err(|_| ImageError::Encoding)?;
        }
    }
    Ok(bytes.into_inner())
}

/// Dekodiert ein Bild; unter Windows immer mit Skalierung 1.
pub fn decode(bytes: &[u8]) -> Result<(Image, f64), ImageError> {
    image::load_from_memory(bytes)
        .map(|decoded| (Arc::new(decoded.into_rgba8()), 1.0))
        .map_err(|_| ImageError::Decoding)
}

#[cfg(test)]
mod tests {
    use super::super::tests::{noise, solid};
    use super::*;

    fn from_pixels(pixels: &Pixels) -> Result<Image, ImageError> {
        let width = u32::try_from(pixels.width).map_err(|_| ImageError::Graphics)?;
        let height = u32::try_from(pixels.height).map_err(|_| ImageError::Graphics)?;
        RgbaImage::from_raw(width, height, pixels.data.clone())
            .map(Arc::new)
            .ok_or(ImageError::Graphics)
    }

    #[test]
    fn round_trips_pixels_through_png() -> Result<(), ImageError> {
        let image = from_pixels(&solid(4, 3, [10, 200, 30, 255]))?;
        let decoded = decode(&encode(&image, Format::Png, 1.0)?)?.0;
        assert_eq!((width(&decoded), height(&decoded)), (4, 3));
        assert_eq!(&decoded.as_raw()[..4], &[10, 200, 30, 255]);
        Ok(())
    }

    #[test]
    fn crops_and_resizes() -> Result<(), ImageError> {
        let image = from_pixels(&solid(40, 20, [0, 0, 0, 255]))?;
        let rect = PixelRect {
            x: 5,
            y: 5,
            width: 10,
            height: 8,
        };
        let cropped = crop(&image, rect)?;
        assert_eq!((width(&cropped), height(&cropped)), (10, 8));
        let resized = resize(&image, 20, 10)?;
        assert_eq!((width(&resized), height(&resized)), (20, 10));
        Ok(())
    }

    #[test]
    fn jpeg_encoding_works() -> Result<(), ImageError> {
        let image = from_pixels(&noise(16, 16))?;
        let bytes = encode(&image, Format::Jpeg, 1.0)?;
        assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
        Ok(())
    }
}
