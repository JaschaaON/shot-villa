# PROGRESS – Screeni

Legende: ☐ offen · ◐ implementiert, Tests grün, Praxistest offen · ☑ fertig (implementiert, getestet, im Betrieb geprüft)

## Infrastruktur

- ☑ Projekt-Setup: Tauri 2.12, React 19.3, Vite 8, TypeScript 7 strict, Oxlint + Ultracite (core, react, vitest, anti-slop, type-aware), Oxfmt, rustfmt, clippy pedantic (unwrap/expect/panic verboten), Vitest, Lefthook (pre-commit → `pnpm check`)
- ☑ Eigenes App-Icon und Menüleisten-Icon (prozedural erzeugt)
- ☑ Lokaler Release-Build, ad-hoc signiert; installiert unter `/Applications/Screeni.app`
- ☑ Einstellungen als `settings.json` (App-Config-Ordner), atomar geschrieben, fehlertolerant geladen

## Windows

- ◐ Windows-Version: Aufnahme (xcap), Texterkennung (Windows.Media.Ocr), QR (rqrr), Zwischenablage (arboard), Autostart, Strg-Kürzel im Editor, NSIS-Installer per GitHub Actions. Kompiliert und besteht clippy pedantic; auf einem echten Windows-Rechner noch nicht getestet.
- 0.1.1: Kein Konsolenfenster mehr unter Windows (Fenster-Subsystem); Schließen einer Konsole beendet Screeni damit auch nicht mehr.
- Bekannte Unterschiede: kein Fensterschatten, Mauszeiger wird nicht mit aufgenommen, eigene Fenster werden vor Vollbild-/Bereichsaufnahmen kurz ausgeblendet.

## Dokumentation und Website

- ☑ Handbuch mit Screenshots (`docs/README.md`), README, Betriebsanleitung (`docs/deploy.md`)
- ☑ Produktseite `website/` (statisch, Hell/Dunkel, responsiv), Download der Universal-DMG
- ☑ Release v0.1.0 mit `Screeni_0.1.0_universal.dmg`
- ◐ Image-Build per GitHub Actions nach GHCR, Compose-Stack – Deployment auf dem Server steht aus

## Muss-Features

| ID | Feature | Status | Notizen |
|---|---|---|---|
| — | Berechtigungs-Flow Bildschirmaufnahme | ◐ | Preflight vor jeder Aufnahme; sonst System-Anfrage + eigenes Fenster (Einstellungen öffnen, Neu starten) |
| C1 | Vollbild | ◐ | Bildschirm unter dem Mauszeiger, native Auflösung, eigene Fenster ausgeschlossen |
| C2 | Bereich | ◐ | Alle Bildschirme eingefroren, Overlay je Bildschirm über der Menüleiste, Fadenkreuz, Lupe, Größenanzeige, Esc bricht ab |
| C4 | Bereich wiederholen | ◐ | Merkt Bildschirm + Rechteck; Hinweis, wenn noch kein Bereich |
| C5 | Aktives Fenster | ◐ | Vorderstes Fenster der aktiven App, mit Schatten, transparenter Hintergrund |
| C6 | Fenster wählen | ◐ | Overlay hebt Fenster unter dem Mauszeiger hervor (App + Titel), Klick nimmt auf |
| C11 | Mauszeiger | ◐ | Auto (nur Vollbild) / immer / nie |
| C12 | Aus Zwischenablage | ◐ | PNG oder TIFF; Menü + ⇧⌘V |
| C13 | Datei öffnen / Drag & Drop / „Öffnen mit“ | ◐ | Dialog, Ziehen ins Editorfenster, Dateizuordnung des Systems; Absturz beim Kaltstart per „Öffnen mit“ behoben (Datei wird bis Ende des Starts zurückgestellt), Kaltstart geprüft |
| C14 | Instant-OCR/QR | ◐ | Bereich wählen → Text bzw. QR-Inhalt in die Zwischenablage + Mitteilung; Auswahl übersteht jetzt auch sehr schnelles Ziehen |
| C15 | Retina / 1×-Export | ◐ | Einstellung „Beim Speichern auf 1× verkleinern“ |
| A1 | Nach der Aufnahme | ◐ | Zeigen / Kopieren / Speichern kombinierbar |
| A2 | Mini-Vorschau | ◐ | Unten rechts; Kopieren, Speichern, Bearbeiten, Schließen; Schließen manuell oder bei Klick daneben |
| A4 | Esc-Verhalten | ◐ | Kopieren-Knopf schließt den Editor bzw. die Vorschau ebenfalls (Wunsch 2026-10-14). |
| A4 (Detail) | Esc-Verhalten | ◐ | Esc im Editor: optional kopieren/speichern, dann schließen |
| A5 | Mitteilungen | ◐ | macOS: eigenes UserNotifications-Framework (vorher kamen keine Mitteilungen an), Banner auch im Vordergrund; Windows: Toast. Editor: „Text erkennen“ zeigt jetzt „Text kopiert: …“ bzw. „Kein Text erkannt“ im Fenster (im Editor geprüft) |
| E1 | Editor-Fenster | ◐ | Werkzeugleiste in der Titelleiste, Infos rechts (Farbe, Größe, Zoom) |
| E2 | Zoom | ◐ | Einpassen, 100 %, ⌘+/⌘−/⌘0/⌘1, Pinch/⌘-Scrollrad; „100 % bevorzugen“ |
| E5 | px/pt | ◐ | Klick auf die Größenanzeige |
| E6 | Pipette + Farbformate | ◐ | HEX, RGB, CSS rgba, OKLCH; Klick wechselt Format, Tab kopiert |
| E10 | Undo/Redo | ◐ | ⌘Z / ⇧⌘Z, Ziehbewegungen als ein Schritt, 100 Schritte |
| T1–T10 | Werkzeuge | ◐ | Texteingabe im Hintergrundtest nicht prüfbar (Fokus) – bitte von Hand testen. |
| T1–T10 (Detail) | Werkzeuge | ◐ | Pfeil, Text, Rechteck, Oval, Linie, Freihand, Textmarker, Verpixeln/Weichzeichnen, Spotlight, Zähler; Einzeltasten A T R O L D H B S C, V = Auswahl; ⇧ = 45°/Quadrat |
| T16 | Zuschneiden | ◐ | Auswahl ziehen, ↩; Zuschnitt zurücksetzen im Menü |
| T20 | Markierungen bearbeiten | ◐ | Auswählen, Verschieben, Griffe, Pfeiltasten (⇧ = 10 pt), ⌘D, Entf |
| T21 | Farbwahl | ◐ | Eigene Palette + Systemfarbwähler |
| X1–X4 | Kopieren, Sichern, Sichern unter, Format | ◐ | ⇧⌘C/Esc, ⌘S, ⇧⌘S; PNG oder Auto (PNG/JPEG-Heuristik) |
| X5 | Drag-out | ☐ | Braucht natives Ziehen aus der Web-Ansicht – folgt |
| X7 | Pin | ◐ | Rahmenlos, immer vorne, Rechtsklick: 50/100/200 %, halbtransparent, Kopieren, Schließen |
| X10 | Metadaten | ◐ | xattr `kMDItemIsScreenCapture` (Test vorhanden) |
| O1–O4 | OCR/QR | ◐ | Vision lokal, Zeilen in Lesereihenfolge, Zeilenumbrüche entfernen, QR/Barcode |
| M1 | Menüleisten-Icon | ◐ | Alle Aufnahmen, OCR, Zwischenablage, Öffnen, Editor, Einstellungen, Beenden; zeigt Kürzel |
| M2 | Dock-Icon nur bei offenem Fenster | ◐ | |
| M3 | Autostart | ◐ | `SMAppService` bzw. Registry (Windows) |
| M4 | Begrüßung beim ersten Start | ☑ | „Installation erfolgreich“ mit Ort des Symbols und den eingestellten Kürzeln (0.1.1) |
| H1 | Globale Hotkeys | ◐ | 7 belegbare Kürzel mit Recorder; Vorgaben ⇧⌘1, ⇧⌘2, ⌃⌥⌘O; Kürzel während der Aufnahme pausiert |

## Entscheidungen / Hinweise

- 2026-10-14: `clippy::needless_pass_by_value` wird an Tauri-Commands per `#[expect(..., reason)]` gezielt ausgenommen (vom Nutzer freigegeben).
- Markdown-Dateien sind von `oxfmt` ausgenommen, damit `CLAUDE.md` unverändert bleibt.
- Anti-Slop verbietet „shape“ in Bezeichnern: das Editor-Modell heißt deshalb „Mark“ (Markierung).
- Bilddaten zwischen Frontend und Rust laufen als Raw-Body (keine JSON-Zahlenlisten).
- 2026-10-14: Builds werden mit dem selbstsignierten Zertifikat „Screeni Dev“ (Anmelde-Schlüsselbund) signiert. Die Freigabe für die Bildschirmaufnahme hängt damit am Zertifikat und bleibt über Builds hinweg erhalten (bei ad-hoc-Signatur hing sie am Code-Hash und verfiel bei jedem Build).
- Die Berechtigungsprüfung nutzt zusätzlich zum Preflight den Abruf der aufnehmbaren Inhalte, weil der Preflight-Wert nach einer Freigabe bis zum Neustart veraltet ist.
