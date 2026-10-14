//! Ablage aufgenommener und geöffneter Bilder.
//!
//! Jedes Bild bekommt eine Id. Das Frontend lädt die kodierten Bytes über
//! diese Id; der Rust-Teil behält das dekodierte Bild für Zuschnitt und OCR.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use crate::imaging::{self, Image};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShotInfo {
    pub id: u32,
    pub width: usize,
    pub height: usize,
    /// Pixel pro Punkt (2 auf Retina-Displays).
    pub scale: f64,
}

pub struct Entry {
    pub info: ShotInfo,
    pub image: Image,
    pub bytes: Vec<u8>,
    pub mime: &'static str,
}

#[derive(Default)]
struct Inner {
    next_id: u32,
    entries: HashMap<u32, Entry>,
    /// Das Bild im Editor bzw. in der Vorschau.
    current: Option<u32>,
}

#[derive(Default)]
pub struct Shots(Mutex<Inner>);

impl Shots {
    pub fn insert(&self, image: Image, scale: f64, bytes: Vec<u8>, mime: &'static str) -> ShotInfo {
        let mut inner = self.lock();
        inner.next_id += 1;
        let info = ShotInfo {
            id: inner.next_id,
            width: imaging::width(&image),
            height: imaging::height(&image),
            scale,
        };
        inner.entries.insert(
            info.id,
            Entry {
                info,
                image,
                bytes,
                mime,
            },
        );
        info
    }

    /// Macht ein Bild zum aktuellen und gibt das vorherige frei.
    pub fn make_current(&self, id: u32) {
        let mut inner = self.lock();
        if let Some(previous) = inner.current.replace(id)
            && previous != id
        {
            inner.entries.remove(&previous);
        }
    }

    pub fn current(&self) -> Option<ShotInfo> {
        let inner = self.lock();
        inner
            .current
            .and_then(|id| inner.entries.get(&id))
            .map(|entry| entry.info)
    }

    pub fn info(&self, id: u32) -> Option<ShotInfo> {
        self.lock().entries.get(&id).map(|entry| entry.info)
    }

    pub fn bytes(&self, id: u32) -> Option<(Vec<u8>, &'static str)> {
        self.lock()
            .entries
            .get(&id)
            .map(|entry| (entry.bytes.clone(), entry.mime))
    }

    pub fn image(&self, id: u32) -> Option<(Image, f64)> {
        self.lock()
            .entries
            .get(&id)
            .map(|entry| (entry.image.clone(), entry.info.scale))
    }

    pub fn remove(&self, id: u32) {
        let mut inner = self.lock();
        if inner.current != Some(id) {
            inner.entries.remove(&id);
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
