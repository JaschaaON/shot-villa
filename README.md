# Screeni

Bildschirmfotos für macOS und Windows: aufnehmen, markieren, weitergeben – per Tastenkürzel und ohne Umwege.

![Screeni-Editor](docs/images/editor.jpg)

- **Aufnahmen:** Vollbild, Bereich (eingefrorener Bildschirm mit Lupe), Bereich wiederholen, Fenster wählen, aktives Fenster
- **Editor:** Pfeil, Text, Rechteck, Oval, Linie, Freihand, Textmarker, Verpixeln/Weichzeichnen, Spotlight, Zähler, Zuschneiden, Pipette mit HEX/RGB/CSS/OKLCH
- **Ausgabe:** Kopieren, Sichern (PNG oder automatisch PNG/JPEG), Anpinnen als schwebendes Fenster, kleine Vorschau nach der Aufnahme
- **Texterkennung** und QR-Codes lokal per Apple Vision
- Frei belegbare globale Kürzel, Menüleisten-App, Start bei der Anmeldung

**Handbuch mit Screenshots:** [docs/README.md](docs/README.md)

## Installation

`Screeni.dmg` aus den Releases bzw. von der Website laden, Screeni in „Programme“ ziehen, beim Öffnen der DMG und beim ersten Start jeweils unter Systemeinstellungen → Datenschutz & Sicherheit „Dennoch öffnen“ wählen (Screeni ist nicht notarisiert) und die Bildschirmaufnahme erlauben. Details im [Handbuch](docs/README.md#installation). Voraussetzung: macOS 14 oder neuer. Für Windows 10/11 gibt es `Screeni-Setup.exe` ([Hinweise](docs/README.md#windows)).

## Entwicklung

Stack: [Tauri 2](https://tauri.app) mit Rust-Backend und React/TypeScript-Frontend. Plattform-Teile: macOS über ScreenCaptureKit, Vision und ImageIO (`objc2`), Windows über `xcap`, `Windows.Media.Ocr`, `rqrr` und `arboard` – jeweils in `src-tauri/src/{capture,imaging,ocr}/{mac,win}.rs`. Der Windows-Installer entsteht per GitHub Actions (`.github/workflows/windows.yml`); `pnpm check` prüft die Windows-Seite mit clippy (braucht `brew install llvm` für `llvm-rc`).

```bash
pnpm install
pnpm tauri dev
```

| Befehl | Zweck |
|---|---|
| `pnpm check` | Lint (Oxlint + Ultracite inkl. Anti-Slop), Formatierung, Typen, rustfmt, clippy pedantic, Vitest, Rust-Tests – läuft auch als Pre-commit-Hook |
| `pnpm fix` | Lint-Autofix und Formatierung |
| `pnpm tauri build` | Release-Build (`.app` und `.dmg`) |

### Signieren

Die Bildschirmaufnahme-Freigabe von macOS hängt an der Signatur. Mit Ad-hoc-Signatur verfällt sie bei jedem Build. Deshalb signiert der Build mit einem lokalen, selbst erstellten Zertifikat **„Screeni Dev“** (`signingIdentity` in `src-tauri/tauri.conf.json`). Wer das Repo auf einem anderen Mac baut, legt ein eigenes Code-Signing-Zertifikat mit diesem Namen an oder setzt `signingIdentity` auf `"-"`.

### Aufbau

```
src/                  Frontend (React)
  editor/             Modell, Geometrie, Renderer, Zustand, Werkzeuge
  views/              Fenster: Editor, Overlay, Vorschau, Pin, Einstellungen, Berechtigung
src-tauri/src/        Backend (Rust)
  capture.rs          ScreenCaptureKit: Bildschirme, Fenster
  actions.rs          Aufnahme-Abläufe, Overlay-Sitzung, „Nach der Aufnahme“
  imaging.rs          Zuschnitt, Skalierung, PNG/JPEG
  ocr.rs              Texterkennung und QR (Vision)
  settings.rs         Einstellungen (settings.json)
  hotkeys.rs, tray.rs, menu.rs, windows.rs, …
website/              Produktseite mit Download und Anleitung
deploy/               Dockerfile, nginx, Compose-Stack
docs/                 Handbuch
```

Website betreiben und Versionen veröffentlichen: [docs/deploy.md](docs/deploy.md)

Analyse und Feature-Liste: [SPEC.md](SPEC.md) · Fortschritt: [PROGRESS.md](PROGRESS.md)

## Lizenz

Screeni © 2026 Jascha Coenen, veröffentlicht unter der [MIT-Lizenz](LICENSE). Die Lizenzen der enthaltenen Bibliotheken stehen in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) (neu erzeugen mit `pnpm notices`, z. B. nach Abhängigkeits-Updates); beide Dateien liegen auch in jedem App-Paket und auf der Website.
