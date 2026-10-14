//! Bildoperationen mit Core Graphics und `ImageIO`.

use std::ffi::c_void;

use objc2_core_foundation::{
    CFData, CFDictionary, CFMutableData, CFNumber, CFRetained, CFString, CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGBitmapContextCreateImage, CGColorSpace, CGContext, CGImage,
    CGImageAlphaInfo, CGInterpolationQuality,
};
use objc2_image_io::{
    CGImageDestination, CGImageSource, kCGImageDestinationLossyCompressionQuality,
};

use super::{Format, ImageError, JPEG_QUALITY, PixelRect, Pixels};

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

pub fn encode(image: &Image, format: Format) -> Result<Vec<u8>, ImageError> {
    let data = CFMutableData::new(None, 0).ok_or(ImageError::Encoding)?;
    let uti = CFString::from_static_str(uti(format));
    let destination = unsafe { CGImageDestination::with_data(&data, &uti, 1, None) }
        .ok_or(ImageError::Encoding)?;
    let quality = CFNumber::new_f64(JPEG_QUALITY);
    // SAFETY: Statische Konstante aus ImageIO.
    let key = unsafe { kCGImageDestinationLossyCompressionQuality };
    let options = CFDictionary::<CFString, CFNumber>::from_slices(&[key], &[&quality]);
    let options = (format == Format::Jpeg).then_some(options.as_ref());
    unsafe { destination.add_image(image, options) };
    if unsafe { destination.finalize() } {
        Ok(data.to_vec())
    } else {
        Err(ImageError::Encoding)
    }
}

pub fn decode(bytes: &[u8]) -> Result<Image, ImageError> {
    let data = CFData::from_bytes(bytes);
    let source = unsafe { CGImageSource::with_data(&data, None) }.ok_or(ImageError::Decoding)?;
    unsafe { source.image_at_index(0, None) }.ok_or(ImageError::Decoding)
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
        let decoded = decode(&encode(&image, Format::Png)?)?;
        let pixels = pixels(&decoded)?;
        assert_eq!((pixels.width, pixels.height), (4, 3));
        assert_eq!(&pixels.data[..4], &[10, 200, 30, 255]);
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
        let bytes = encode(&image, Format::Jpeg)?;
        assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
        Ok(())
    }
}
