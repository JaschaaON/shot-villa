//! MP4-Datei (H.264, ohne Ton) über den Sink Writer von Media Foundation.
//!
//! Der Sink Writer lebt in einem eigenen Thread; Bilder kommen über einen
//! Kanal. Er nutzt den Software-Encoder von Windows und läuft damit auch auf
//! Rechnern ohne Grafikkarten-Encoder, etwa in virtuellen Maschinen.

use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::JoinHandle;

use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Media::MediaFoundation::{
    IMFMediaType, IMFSinkWriter, MF_MT_AVG_BITRATE, MF_MT_DEFAULT_STRIDE, MF_MT_FRAME_RATE,
    MF_MT_FRAME_SIZE, MF_MT_INTERLACE_MODE, MF_MT_MAJOR_TYPE, MF_MT_PIXEL_ASPECT_RATIO,
    MF_MT_SUBTYPE, MF_VERSION, MFCreateMediaType, MFCreateMemoryBuffer, MFCreateSample,
    MFCreateSinkWriterFromURL, MFMediaType_Video, MFSTARTUP_FULL, MFShutdown, MFStartup,
    MFVideoFormat_H264, MFVideoFormat_RGB32, MFVideoInterlace_Progressive,
};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
use windows::core::{GUID, HSTRING};

/// Zeiteinheit von Media Foundation: 100 Nanosekunden.
const UNITS_PER_SECOND: i64 = 10_000_000;
/// Bilder, die auf den Encoder warten dürfen; weitere werden verworfen.
const QUEUE_LENGTH: usize = 8;
/// Bits pro Bildpunkt und Bild – ruhige Bildschirminhalte brauchen wenig.
const BITS_PER_PIXEL: f64 = 0.15;

/// Ein Bild in BGRA, Zeilen von unten nach oben, Zeitstempel in 100 ns.
struct Frame {
    pixels: Vec<u8>,
    time: i64,
}

pub struct Mp4Writer {
    frames: SyncSender<Frame>,
    thread: JoinHandle<Result<(), String>>,
}

impl Mp4Writer {
    /// Legt die Datei an; kehrt erst zurück, wenn der Encoder bereit ist.
    pub fn create(path: &Path, width: u32, height: u32, fps: u32) -> Result<Self, String> {
        let (frames, received) = mpsc::sync_channel(QUEUE_LENGTH);
        let (ready_sender, ready) = mpsc::channel();
        let path = path.to_owned();
        let thread = std::thread::spawn(move || {
            let _runtime = Runtime::start().map_err(|error| error.to_string())?;
            let output = match Output::open(&path, width, height, fps) {
                Ok(output) => {
                    let _ = ready_sender.send(Ok(()));
                    output
                }
                Err(error) => {
                    let _ = ready_sender.send(Err(error.to_string()));
                    return Ok(());
                }
            };
            output.encode(&received).map_err(|error| error.to_string())
        });
        ready
            .recv()
            .map_err(|_| "Encoder nicht gestartet".to_owned())??;
        Ok(Self { frames, thread })
    }

    /// Reicht ein Bild weiter; ist der Encoder im Rückstand, entfällt es.
    pub fn send(&self, pixels: Vec<u8>, time: i64) -> Result<(), String> {
        match self.frames.try_send(Frame { pixels, time }) {
            Ok(()) | Err(TrySendError::Full(_)) => Ok(()),
            Err(TrySendError::Disconnected(_)) => Err("Encoder beendet".to_owned()),
        }
    }

    /// Schreibt die restlichen Bilder und schließt die Datei.
    pub fn finish(self) -> Result<(), String> {
        drop(self.frames);
        self.thread
            .join()
            .map_err(|_| "Encoder abgestürzt".to_owned())?
    }
}

/// COM und Media Foundation für die Dauer des Encoder-Threads.
struct Runtime {
    com: bool,
}

impl Runtime {
    fn start() -> windows::core::Result<Self> {
        // SAFETY: Initialisierung des eigenen Threads; Gegenstück in `drop`.
        let com = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        // SAFETY: Version aus denselben Headern; Gegenstück in `drop`.
        if let Err(error) = unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) } {
            if com {
                // SAFETY: passt zum erfolgreichen `CoInitializeEx` oben.
                unsafe { CoUninitialize() };
            }
            return Err(error);
        }
        Ok(Self { com })
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // SAFETY: `MFStartup` war erfolgreich, sonst gäbe es kein `Runtime`.
        let _ = unsafe { MFShutdown() };
        if self.com {
            // SAFETY: passt zum erfolgreichen `CoInitializeEx` in `start`.
            unsafe { CoUninitialize() };
        }
    }
}

struct Output {
    writer: IMFSinkWriter,
    stream: u32,
    frame_duration: i64,
}

impl Output {
    fn open(path: &Path, width: u32, height: u32, fps: u32) -> windows::core::Result<Self> {
        let pixels = f64::from(width) * f64::from(height) * f64::from(fps);
        let bitrate = bitrate(pixels);
        // SAFETY: Media Foundation läuft (`Runtime`); alle Zeiger sind gültige Referenzen.
        unsafe {
            let writer = MFCreateSinkWriterFromURL(&HSTRING::from(path), None, None)?;
            let encoded = video_type(&MFVideoFormat_H264, width, height, fps)?;
            encoded.SetUINT32(&MF_MT_AVG_BITRATE, bitrate)?;
            let stream = writer.AddStream(&encoded)?;
            let raw = video_type(&MFVideoFormat_RGB32, width, height, fps)?;
            // Negative Zeilenlänge: Zeilen von unten nach oben. Als u32 im
            // Zweierkomplement abgelegt, wie Media Foundation es erwartet.
            raw.SetUINT32(&MF_MT_DEFAULT_STRIDE, (width * 4).wrapping_neg())?;
            writer.SetInputMediaType(stream, &raw, None)?;
            writer.BeginWriting()?;
            Ok(Self {
                writer,
                stream,
                frame_duration: UNITS_PER_SECOND / i64::from(fps.max(1)),
            })
        }
    }

    /// Kodiert, bis der Kanal geschlossen wird, und schließt dann die Datei.
    fn encode(&self, frames: &Receiver<Frame>) -> windows::core::Result<()> {
        let mut first = None;
        for frame in frames {
            let time = frame.time - *first.get_or_insert(frame.time);
            self.write(&frame.pixels, time)?;
        }
        // SAFETY: `BeginWriting` war erfolgreich.
        unsafe { self.writer.Finalize() }
    }

    fn write(&self, pixels: &[u8], time: i64) -> windows::core::Result<()> {
        let length = u32::try_from(pixels.len()).map_err(|_| E_INVALIDARG)?;
        // SAFETY: Der Puffer fasst `length` Bytes; geschrieben wird nur
        // zwischen `Lock` und `Unlock`.
        unsafe {
            let buffer = MFCreateMemoryBuffer(length)?;
            let mut data = std::ptr::null_mut();
            buffer.Lock(&raw mut data, None, None)?;
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), data, pixels.len());
            buffer.Unlock()?;
            buffer.SetCurrentLength(length)?;
            let sample = MFCreateSample()?;
            sample.AddBuffer(&buffer)?;
            sample.SetSampleTime(time)?;
            sample.SetSampleDuration(self.frame_duration)?;
            self.writer.WriteSample(self.stream, &sample)
        }
    }
}

/// Gemeinsame Angaben für das Roh- und das kodierte Video.
fn video_type(
    subtype: &GUID,
    width: u32,
    height: u32,
    fps: u32,
) -> windows::core::Result<IMFMediaType> {
    // SAFETY: Media Foundation läuft (`Runtime`); alle Zeiger sind gültige Referenzen.
    unsafe {
        let media_type = MFCreateMediaType()?;
        media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        media_type.SetGUID(&MF_MT_SUBTYPE, subtype)?;
        media_type.SetUINT32(
            &MF_MT_INTERLACE_MODE,
            MFVideoInterlace_Progressive.0.cast_unsigned(),
        )?;
        media_type.SetUINT64(&MF_MT_FRAME_SIZE, pair(width, height))?;
        media_type.SetUINT64(&MF_MT_FRAME_RATE, pair(fps, 1))?;
        media_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pair(1, 1))?;
        Ok(media_type)
    }
}

/// Zwei Werte in einem 64-Bit-Attribut, der erste in der oberen Hälfte.
fn pair(high: u32, low: u32) -> u64 {
    (u64::from(high) << 32) | u64::from(low)
}

/// Bitrate für so viele Bildpunkte pro Sekunde, mindestens 1 Mbit/s.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "auf den gültigen u32-Bereich begrenzt"
)]
fn bitrate(pixels_per_second: f64) -> u32 {
    (pixels_per_second * BITS_PER_PIXEL).clamp(1_000_000.0, f64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_pairs_high_first() {
        assert_eq!(pair(1920, 1080), 0x0000_0780_0000_0438);
        assert_eq!(pair(30, 1), 0x0000_001E_0000_0001);
    }

    #[test]
    fn bitrate_scales_with_area() {
        assert_eq!(bitrate(1920.0 * 1080.0 * 30.0), 9_331_200);
        assert_eq!(bitrate(100.0 * 100.0 * 30.0), 1_000_000);
    }

    #[test]
    fn stride_is_negative_in_twos_complement() {
        assert_eq!((400_u32 * 4).wrapping_neg().cast_signed(), -1600);
    }
}
