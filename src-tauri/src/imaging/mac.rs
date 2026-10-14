//! Bildoperationen mit Core Graphics und `ImageIO`.

use std::ffi::c_void;

use objc2_core_foundation::{
    CFData, CFDictionary, CFMutableData, CFNumber, CFRetained, CFString, CFType, CGPoint, CGRect,
    CGSize,
};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGBitmapContextCreateImage, CGColorSpace, CGContext, CGImage,
    CGImageAlphaInfo, CGInterpolationQuality,
};
use objc2_image_io::{
    CGImageDestination, CGImageSource, kCGImageDestinationLossyCompressionQuality,
    kCGImagePropertyDPIHeight, kCGImagePropertyDPIWidth,
};

use super::{Format, ImageError, JPEG_QUALITY, PixelRect, Pixels};

/// Pixeldichte eines Bildes ohne Retina-Skalierung.
const BASE_DPI: f64 = 72.0;

/// Leitet die Retina-Skalierung aus der Pixeldichte ab. Nur 144 und 216 dpi
/// (wie bei Bildschirmfotos) zählen; Fotos mit 300 dpi bleiben in voller Größe.
fn scale_from_dpi(dpi: Option<f64>) -> f64 {
    let ratio = dpi.unwrap_or(BASE_DPI) / BASE_DPI;
    [2.0, 3.0]
        .into_iter()
        .find(|scale| (ratio - scale).abs() < 0.05)
        .unwrap_or(1.0)
}

/// Ein Bild; Kopien teilen sich die Pixel.
pub type Image = CFRetained<CGImage>;

const fn uti(format: Format) -> &'static str {
    match format {
        Format::Png => "public.png",
        Format::Jpeg => "public.jpeg",
    }
}

pub fn width(image: &Image) -> usize {
    CGImage::width(Some(image))
}

pub fn height(image: &Image) -> usize {
    CGImage::height(Some(image))
}

pub fn pixels(image: &Image) -> Result<Pixels, ImageError> {
    let width = width(image);
    let height = height(image);
    let mut data = vec![0_u8; width * height * 4];
    let context = bitmap_context(width, height, data.as_mut_ptr().cast())?;
    CGContext::draw_image(Some(&context), full_rect(width, height), Some(image));
    drop(context);
    Ok(Pixels {
        width,
        height,
        data,
    })
}

/// Erzeugt ein Bild aus RGBA-Pixeln.
#[cfg(test)]
pub fn from_pixels(pixels: &Pixels) -> Result<Image, ImageError> {
    use objc2_core_graphics::CGBitmapContextGetData;
    let context = bitmap_context(pixels.width, pixels.height, std::ptr::null_mut())?;
    let target = CGBitmapContextGetData(Some(&context)).cast::<u8>();
    if target.is_null() || pixels.data.len() != pixels.width * pixels.height * 4 {
        return Err(ImageError::Graphics);
    }
    // SAFETY: Der Kontext besitzt einen Puffer von width * height * 4 Bytes
    // (Zeilenlänge width * 4), die Quelle hat genau diese Länge.
    unsafe { std::ptr::copy_nonoverlapping(pixels.data.as_ptr(), target, pixels.data.len()) };
    CGBitmapContextCreateImage(Some(&context)).ok_or(ImageError::Graphics)
}

/// Schneidet einen Bereich aus; er wird auf das Bild begrenzt.
pub fn crop(image: &Image, rect: PixelRect) -> Result<Image, ImageError> {
    let rect = rect.clamped(width(image), height(image));
    let rect = CGRect::new(
        CGPoint::new(f64::from(rect.x), f64::from(rect.y)),
        CGSize::new(f64::from(rect.width), f64::from(rect.height)),
    );
    CGImage::with_image_in_rect(Some(image), rect).ok_or(ImageError::Graphics)
}

/// Skaliert ein Bild auf die angegebene Pixelgröße.
pub fn resize(image: &Image, width: usize, height: usize) -> Result<Image, ImageError> {
    let context = bitmap_context(width, height, std::ptr::null_mut())?;
    CGContext::set_interpolation_quality(Some(&context), CGInterpolationQuality::High);
    CGContext::draw_image(Some(&context), full_rect(width, height), Some(image));
    CGBitmapContextCreateImage(Some(&context)).ok_or(ImageError::Graphics)
}

/// Kodiert ein Bild; `scale` landet als Pixeldichte in der Datei (2 = 144 dpi).
pub fn encode(image: &Image, format: Format, scale: f64) -> Result<Vec<u8>, ImageError> {
    let data = CFMutableData::new(None, 0).ok_or(ImageError::Encoding)?;
    let uti = CFString::from_static_str(uti(format));
    let destination = unsafe { CGImageDestination::with_data(&data, &uti, 1, None) }
        .ok_or(ImageError::Encoding)?;
    let dpi = CFNumber::new_f64(BASE_DPI * scale);
    let quality = CFNumber::new_f64(JPEG_QUALITY);
    // SAFETY: Statische Konstanten aus ImageIO.
    let (dpi_width, dpi_height, quality_key) = unsafe {
        (
            kCGImagePropertyDPIWidth,
            kCGImagePropertyDPIHeight,
            kCGImageDestinationLossyCompressionQuality,
        )
    };
    let options = if format == Format::Jpeg {
        CFDictionary::<CFString, CFNumber>::from_slices(
            &[dpi_width, dpi_height, quality_key],
            &[&dpi, &dpi, &quality],
        )
    } else {
        CFDictionary::<CFString, CFNumber>::from_slices(&[dpi_width, dpi_height], &[&dpi, &dpi])
    };
    unsafe { destination.add_image(image, Some(options.as_ref())) };
    if unsafe { destination.finalize() } {
        Ok(data.to_vec())
    } else {
        Err(ImageError::Encoding)
    }
}

/// Dekodiert ein Bild und liest seine Pixeldichte als Skalierung (144 dpi = 2).
pub fn decode(bytes: &[u8]) -> Result<(Image, f64), ImageError> {
    let data = CFData::from_bytes(bytes);
    let source = unsafe { CGImageSource::with_data(&data, None) }.ok_or(ImageError::Decoding)?;
    let image = unsafe { source.image_at_index(0, None) }.ok_or(ImageError::Decoding)?;
    // SAFETY: Bildeigenschaften sind ein Wörterbuch mit Zeichenketten als Schlüsseln.
    let properties = unsafe { source.properties_at_index(0, None) }
        .map(|dict| unsafe { CFRetained::cast_unchecked::<CFDictionary<CFString, CFType>>(dict) });
    // SAFETY: Statische Konstante aus ImageIO.
    let key = unsafe { kCGImagePropertyDPIWidth };
    let dpi = properties
        .and_then(|dict| dict.get(key))
        .and_then(|value| value.downcast::<CFNumber>().ok())
        .and_then(|number| number.as_f64());
    Ok((image, scale_from_dpi(dpi)))
}

fn bitmap_context(
    width: usize,
    height: usize,
    data: *mut c_void,
) -> Result<CFRetained<CGContext>, ImageError> {
    let space = CGColorSpace::new_device_rgb().ok_or(ImageError::Graphics)?;
    // SAFETY: `data` ist null (CoreGraphics verwaltet den Speicher) oder zeigt auf
    // width * height * 4 Bytes, die länger leben als der Kontext.
    unsafe {
        CGBitmapContextCreate(
            data,
            width,
            height,
            8,
            width * 4,
            Some(&space),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }
    .ok_or(ImageError::Graphics)
}

#[allow(clippy::cast_precision_loss)]
fn full_rect(width: usize, height: usize) -> CGRect {
    CGRect::new(CGPoint::ZERO, CGSize::new(width as f64, height as f64))
}

#[cfg(test)]
mod tests {
    use super::super::tests::{noise, solid};
    use super::*;

    #[test]
    fn round_trips_pixels_through_png() -> Result<(), ImageError> {
        let image = from_pixels(&solid(4, 3, [10, 200, 30, 255]))?;
        let decoded = decode(&encode(&image, Format::Png, 1.0)?)?.0;
        let pixels = pixels(&decoded)?;
        assert_eq!((pixels.width, pixels.height), (4, 3));
        assert_eq!(&pixels.data[..4], &[10, 200, 30, 255]);
        Ok(())
    }

    #[test]
    fn retina_dpi_becomes_scale() {
        assert!((scale_from_dpi(Some(144.0)) - 2.0).abs() < f64::EPSILON);
        assert!((scale_from_dpi(Some(216.0)) - 3.0).abs() < f64::EPSILON);
        assert!((scale_from_dpi(Some(72.0)) - 1.0).abs() < f64::EPSILON);
        assert!((scale_from_dpi(Some(300.0)) - 1.0).abs() < f64::EPSILON);
        assert!((scale_from_dpi(None) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn keeps_retina_scale_as_dpi() -> Result<(), ImageError> {
        let image = from_pixels(&solid(4, 4, [0, 0, 0, 255]))?;
        for format in [Format::Png, Format::Jpeg] {
            let (_, scale) = decode(&encode(&image, format, 2.0)?)?;
            assert!((scale - 2.0).abs() < f64::EPSILON);
        }
        Ok(())
    }

    #[test]
    fn crops_and_resizes() -> Result<(), ImageError> {
        let image = from_pixels(&solid(40, 20, [0, 0, 0, 255]))?;
        let cropped = crop(
            &image,
            PixelRect {
                x: 5,
                y: 5,
                width: 10,
                height: 8,
            },
        )?;
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
