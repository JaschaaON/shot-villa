//! Bildoperationen, plattformunabhängig nach außen: Pixelzugriff, Zuschnitt,
//! Skalierung, Kodierung und Dekodierung. Die Umsetzung steckt in `mac.rs`
//! (Core Graphics, `ImageIO`) bzw. `win.rs` (`image`-Crate).

use std::collections::HashSet;

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::{Image, crop, decode, encode, height, pixels, resize, width};

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::{Image, crop, decode, encode, height, pixels, resize, width};

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Bild konnte nicht verarbeitet werden")]
    Graphics,
    #[error("Bild konnte nicht kodiert werden")]
    Encoding,
    #[error("Datei ist kein lesbares Bild")]
    Decoding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Png,
    Jpeg,
}

impl Format {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }

    pub const fn mime(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }
}

/// JPEG-Qualität von 0 bis 1.
pub const JPEG_QUALITY: f64 = 0.9;

/// Rechteck in ganzen Bildpixeln.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    /// Begrenzt das Rechteck auf ein Bild der angegebenen Größe.
    pub fn clamped(self, width: usize, height: usize) -> Self {
        let max_x = u32::try_from(width).unwrap_or(u32::MAX);
        let max_y = u32::try_from(height).unwrap_or(u32::MAX);
        let x = self.x.min(max_x);
        let y = self.y.min(max_y);
        Self {
            x,
            y,
            width: self.width.min(max_x - x),
            height: self.height.min(max_y - y),
        }
    }
}

/// RGBA-Pixel (8 Bit, sRGB) eines Bildes.
pub struct Pixels {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

/// Entscheidet für das Format „Auto“: Fotos und Verläufe als JPEG, flache
/// Oberflächen und Text als PNG. Maß ist der Anteil verschiedener Farben in
/// einer Stichprobe.
pub fn looks_photographic(pixels: &Pixels) -> bool {
    const MAX_SAMPLES: usize = 160_000;
    const DISTINCT_RATIO: f64 = 0.2;

    let total = pixels.width * pixels.height;
    if total == 0 {
        return false;
    }
    let step = total.div_ceil(MAX_SAMPLES).max(1);
    let mut colors = HashSet::new();
    let mut samples = 0_u32;
    for pixel in pixels.data.as_chunks::<4>().0.iter().step_by(step) {
        colors.insert(u32::from_le_bytes(*pixel));
        samples += 1;
    }
    let distinct = u32::try_from(colors.len()).unwrap_or(u32::MAX);
    f64::from(distinct) / f64::from(samples) > DISTINCT_RATIO
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn solid(width: usize, height: usize, rgba: [u8; 4]) -> Pixels {
        Pixels {
            width,
            height,
            data: rgba.repeat(width * height),
        }
    }

    pub fn noise(width: usize, height: usize) -> Pixels {
        let mut state: u32 = 0x1234_5678;
        let data = (0..width * height * 4)
            .map(|i| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                if i % 4 == 3 {
                    255
                } else {
                    state.to_le_bytes()[0]
                }
            })
            .collect();
        Pixels {
            width,
            height,
            data,
        }
    }

    #[test]
    fn flat_images_are_not_photographic() {
        assert!(!looks_photographic(&solid(200, 100, [255, 255, 255, 255])));
    }

    #[test]
    fn noisy_images_are_photographic() {
        assert!(looks_photographic(&noise(200, 100)));
    }

    #[test]
    fn clamps_rect_to_image() {
        let rect = PixelRect {
            x: 90,
            y: 5,
            width: 50,
            height: 10,
        };
        assert_eq!(
            rect.clamped(100, 8),
            PixelRect {
                x: 90,
                y: 5,
                width: 10,
                height: 3
            }
        );
    }
}
