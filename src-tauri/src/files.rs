//! Speichern von Bildern im Dateisystem.

use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

/// Binäre Property-List mit dem Wert `true`.
#[cfg(target_os = "macos")]
const BPLIST_TRUE: &[u8] = &[
    b'b', b'p', b'l', b'i', b's', b't', b'0', b'0', // Kopf
    0x09, // Objekt 0: true
    0x08, // Offset-Tabelle: Objekt 0 liegt bei Byte 8
    0, 0, 0, 0, 0, 0, // Trailer: unbenutzt
    1, 1, // Größe der Offsets und Objekt-Referenzen
    0, 0, 0, 0, 0, 0, 0, 1, // Anzahl Objekte
    0, 0, 0, 0, 0, 0, 0, 0, // oberstes Objekt
    0, 0, 0, 0, 0, 0, 0, 9, // Position der Offset-Tabelle
];

/// Dateiname aus Zeitstempel, z. B. `Shot-VILLA 2026-10-14 um 11.32.05.png`.
pub fn file_name(time: DateTime<Local>, extension: &str) -> String {
    format!(
        "Shot-VILLA {} um {}.{extension}",
        time.format("%Y-%m-%d"),
        time.format("%H.%M.%S")
    )
}

/// Erster freier Pfad: hängt bei Namensgleichheit „ 2“, „ 3“ … an.
pub fn unique_path(folder: &Path, name: &str) -> PathBuf {
    let candidate = folder.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let (stem, extension) = name.rsplit_once('.').unwrap_or((name, ""));
    (2..=u32::MAX)
        .map(|n| folder.join(format!("{stem} {n}.{extension}")))
        .find(|path| !path.exists())
        .unwrap_or(candidate)
}

/// Schreibt ein Bild in den Ordner und markiert es als Bildschirmfoto.
pub fn save_in_folder(folder: &Path, bytes: &[u8], extension: &str) -> io::Result<PathBuf> {
    std::fs::create_dir_all(folder)?;
    let path = unique_path(folder, &file_name(Local::now(), extension));
    write_screenshot(&path, bytes)?;
    Ok(path)
}

pub fn write_screenshot(path: &Path, bytes: &[u8]) -> io::Result<()> {
    std::fs::write(path, bytes)?;
    mark_as_screen_capture(path)
}

/// Setzt `kMDItemIsScreenCapture`, damit Spotlight und Finder die Datei als
/// Bildschirmfoto erkennen.
#[cfg(target_os = "macos")]
fn mark_as_screen_capture(path: &Path) -> io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path = CString::new(path.as_os_str().as_bytes()).map_err(io::Error::other)?;
    let name = c"com.apple.metadata:kMDItemIsScreenCapture";
    // SAFETY: Beide C-Strings sind gültig und nullterminiert, die Daten leben während des Aufrufs.
    let result = unsafe {
        libc::setxattr(
            path.as_ptr(),
            name.as_ptr(),
            BPLIST_TRUE.as_ptr().cast(),
            BPLIST_TRUE.len(),
            0,
            0,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// Windows kennt keine solche Markierung.
#[cfg(windows)]
#[expect(clippy::unnecessary_wraps, reason = "gleiche Signatur wie unter macOS")]
const fn mark_as_screen_capture(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn temp_folder(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("shot-villa-files-{}-{name}", std::process::id()))
    }

    #[test]
    fn file_name_contains_date_and_time() {
        let time = Local.with_ymd_and_hms(2026, 10, 14, 9, 5, 7).single();
        assert_eq!(
            time.map(|t| file_name(t, "png")),
            Some("Shot-VILLA 2026-10-14 um 09.05.07.png".to_owned())
        );
    }

    #[test]
    fn unique_path_adds_counter() -> io::Result<()> {
        let folder = temp_folder("unique");
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("a.png"), b"x")?;
        std::fs::write(folder.join("a 2.png"), b"x")?;
        assert_eq!(unique_path(&folder, "a.png"), folder.join("a 3.png"));
        assert_eq!(unique_path(&folder, "b.png"), folder.join("b.png"));
        Ok(())
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn saved_file_is_marked_as_screen_capture() -> io::Result<()> {
        let folder = temp_folder("xattr");
        let path = save_in_folder(&folder, b"data", "png")?;
        let output = std::process::Command::new("xattr")
            .arg("-l")
            .arg(&path)
            .output()?;
        assert!(String::from_utf8_lossy(&output.stdout).contains("kMDItemIsScreenCapture"));
        Ok(())
    }
}
