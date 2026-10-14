//! Systemmitteilungen, z. B. „Text kopiert“ nach der Erkennung per Kürzel.

#[cfg(target_os = "macos")]
mod mac;
#[cfg(windows)]
mod win;

#[cfg(target_os = "macos")]
pub use mac::{init, show};
#[cfg(windows)]
pub use win::{init, show};
