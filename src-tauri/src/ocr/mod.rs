//! Text- und QR-Code-Erkennung, lokal auf dem Rechner. Die Umsetzung steckt in
//! `mac.rs` (Vision) bzw. `win.rs` (`Windows.Media.Ocr` und `rqrr`).

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::recognize;

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::recognize;

#[derive(Debug, thiserror::Error)]
#[error("Texterkennung fehlgeschlagen: {0}")]
pub struct OcrError(pub String);

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Recognition {
    /// Erkannter Text, Zeilen in Lesereihenfolge.
    pub lines: Vec<String>,
    /// Inhalte erkannter QR- und Barcodes.
    pub codes: Vec<String>,
}

/// Fügt erkannte Zeilen zusammen. Ohne Zeilenumbrüche werden Zeilen mit
/// Leerzeichen verbunden; ein Trennstrich am Zeilenende verbindet direkt.
pub fn join_lines(lines: &[String], remove_line_breaks: bool) -> String {
    if !remove_line_breaks {
        return lines.join("\n");
    }
    lines.iter().fold(String::new(), |mut text, line| {
        if text.ends_with('-') {
            text.pop();
        } else if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(line.trim());
        text
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(items: &[&str]) -> Vec<String> {
        items.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn keeps_line_breaks_by_default() {
        assert_eq!(join_lines(&lines(&["eins", "zwei"]), false), "eins\nzwei");
    }

    #[test]
    fn removes_line_breaks_and_joins_hyphenation() {
        assert_eq!(
            join_lines(&lines(&["Bildschirm-", "aufnahme ist", " fertig"]), true),
            "Bildschirmaufnahme ist fertig"
        );
    }

    #[test]
    fn empty_input_gives_empty_text() {
        assert_eq!(join_lines(&[], true), "");
    }
}
