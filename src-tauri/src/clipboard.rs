//! Zugriff auf die allgemeine Zwischenablage.

#[cfg(target_os = "macos")]
pub use mac::{read_image, write_image, write_text};
#[cfg(windows)]
pub use win::{read_image, write_image, write_text};

#[cfg(target_os = "macos")]
mod mac {
    use objc2_app_kit::{
        NSBitmapImageRep, NSPasteboard, NSPasteboardTypePNG, NSPasteboardTypeString,
        NSPasteboardTypeTIFF,
    };
    use objc2_foundation::{NSData, NSString};

    /// Legt ein Bild als PNG und TIFF ab; manche Apps lesen nur TIFF.
    pub fn write_image(png: &[u8]) {
        let data = NSData::with_bytes(png);
        let tiff =
            NSBitmapImageRep::imageRepWithData(&data).and_then(|rep| rep.TIFFRepresentation());
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        // SAFETY: Statische Typkonstanten aus AppKit.
        unsafe {
            pasteboard.setData_forType(Some(&data), NSPasteboardTypePNG);
            if let Some(tiff) = tiff {
                pasteboard.setData_forType(Some(&tiff), NSPasteboardTypeTIFF);
            }
        }
    }

    pub fn write_text(text: &str) {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        // SAFETY: Statische Typkonstante aus AppKit.
        unsafe { pasteboard.setString_forType(&NSString::from_str(text), NSPasteboardTypeString) };
    }

    /// Bilddaten aus der Zwischenablage (PNG bevorzugt, sonst TIFF).
    pub fn read_image() -> Option<Vec<u8>> {
        let pasteboard = NSPasteboard::generalPasteboard();
        // SAFETY: Statische Typkonstanten aus AppKit.
        let data = unsafe {
            pasteboard
                .dataForType(NSPasteboardTypePNG)
                .or_else(|| pasteboard.dataForType(NSPasteboardTypeTIFF))
        }?;
        Some(data.to_vec())
    }
}

#[cfg(windows)]
mod win {
    use std::borrow::Cow;

    use arboard::{Clipboard, ImageData};

    use crate::imaging::{self, Format};

    fn report(result: Result<(), arboard::Error>) {
        if let Err(error) = result {
            log::warn!("Zwischenablage: {error}");
        }
    }

    pub fn write_image(png: &[u8]) {
        let Ok(image) = imaging::decode(png) else {
            log::warn!("Zwischenablage: Bild nicht lesbar");
            return;
        };
        let data = ImageData {
            width: imaging::width(&image),
            height: imaging::height(&image),
            bytes: Cow::Borrowed(image.as_raw()),
        };
        report(Clipboard::new().and_then(|mut clipboard| clipboard.set_image(data)));
    }

    pub fn write_text(text: &str) {
        report(Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text)));
    }

    /// Bild aus der Zwischenablage als PNG.
    pub fn read_image() -> Option<Vec<u8>> {
        let data = Clipboard::new().ok()?.get_image().ok()?;
        let width = u32::try_from(data.width).ok()?;
        let height = u32::try_from(data.height).ok()?;
        let image = image::RgbaImage::from_raw(width, height, data.bytes.into_owned())?;
        imaging::encode(&std::sync::Arc::new(image), Format::Png).ok()
    }
}
