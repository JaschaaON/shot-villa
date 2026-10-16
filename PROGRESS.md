# PROGRESS – Shot-VILLA

Legende: ☐ offen · ◐ implementiert, Tests grün, Praxistest offen · ☑ fertig (implementiert, getestet, im Betrieb geprüft)

## Infrastruktur

- ☑ Projekt-Setup: Tauri 2.12, React 19.3, Vite 8, TypeScript 7 strict, Oxlint + Ultracite (core, react, vitest, anti-slop, type-aware), Oxfmt, rustfmt, clippy pedantic (unwrap/expect/panic verboten), Vitest, Lefthook (pre-commit → `pnpm check`)
- ☑ Eigenes App-Icon und Menüleisten-Icon (prozedural erzeugt)
- ☑ Lokaler Release-Build, ad-hoc signiert; installiert unter `/Applications/Shot-VILLA.app`
- ☑ Einstellungen als `settings.json` (App-Config-Ordner), atomar geschrieben, fehlertolerant geladen

## Windows

- ◐ Windows-Version: Aufnahme (xcap), Texterkennung (Windows.Media.Ocr), QR (rqrr), Zwischenablage (arboard), Autostart, Strg-Kürzel im Editor, NSIS-Installer per GitHub Actions. Kompiliert und besteht clippy pedantic; auf einem echten Windows-Rechner noch nicht getestet.
- 0.1.1: Kein Konsolenfenster mehr unter Windows (Fenster-Subsystem); Schließen einer Konsole beendet Shot-VILLA damit auch nicht mehr.
- Bekannte Unterschiede: kein Fensterschatten, Mauszeiger wird nicht mit aufgenommen, eigene Fenster werden vor Vollbild-/Bereichsaufnahmen kurz ausgeblendet.

## Dokumentation und Website

- ☑ Handbuch mit Screenshots (`docs/README.md`), README, Betriebsanleitung (`docs/deploy.md`)
- ☑ Produktseite `website/` (statisch, Hell/Dunkel, responsiv), Download der Universal-DMG
- ☑ Release v0.1.0 mit `Shot-VILLA_0.1.0_universal.dmg`
- ◐ Image-Build per GitHub Actions nach GHCR, Compose-Stack – Deployment auf dem Server steht aus

## Muss-Features

| ID | Feature | Status | Notizen |
|---|---|---|---|
| — | Berechtigungs-Flow Bildschirmaufnahme | ☑ | Preflight vor jeder Aufnahme; sonst System-Anfrage + eigenes Fenster (Einstellungen öffnen, Neu starten) |
| C1 | Vollbild | ☑ | Bildschirm unter dem Mauszeiger, native Auflösung, eigene Fenster ausgeschlossen |
| C2 | Bereich | ☑ | Alle Bildschirme eingefroren, Overlay je Bildschirm über der Menüleiste, Fadenkreuz, Lupe, Größenanzeige, Esc bricht ab |
| C4 | Bereich wiederholen | ☑ | Merkt Bildschirm + Rechteck; Hinweis, wenn noch kein Bereich |
| C5 | Aktives Fenster | ☑ | Vorderstes Fenster der aktiven App, mit Schatten, transparenter Hintergrund |
| C6 | Fenster wählen | ☑ | Overlay hebt Fenster unter dem Mauszeiger hervor (App + Titel), Klick nimmt auf |
| C11 | Mauszeiger | ◐ | Auto (nur Vollbild) / immer / nie |
| C12 | Aus Zwischenablage | ☑ | PNG oder TIFF; Menü + ⇧⌘V; Retina-Bilder behalten ihre Größe (Pixeldichte 144 dpi wird gelesen) |
| C13 | Datei öffnen / Drag & Drop / „Öffnen mit“ | ◐ | Dialog, Ziehen ins Editorfenster, Dateizuordnung des Systems; Absturz beim Kaltstart per „Öffnen mit“ behoben (Datei wird bis Ende des Starts zurückgestellt), Kaltstart geprüft |
| C14 | Instant-OCR/QR | ☑ | Bereich wählen → Text bzw. QR-Inhalt in die Zwischenablage + Mitteilung; Auswahl übersteht jetzt auch sehr schnelles Ziehen; Mitteilung laut Systemprotokoll zugestellt |
| C15 | Retina / 1×-Export | ◐ | Einstellung „Beim Speichern auf 1× verkleinern“; Kopien und Dateien tragen die Pixeldichte (144 dpi bei Retina), wie macOS-Bildschirmfotos |
| C16 | Bildschirmaufnahme (Video) | ☑ | macOS 15+: `SCStream` mit `SCRecordingOutput` (H.264/MP4, 30 fps, Retina, mit Mauszeiger, ohne Ton); Menü oder ⌃⌥⌘R, Bereich wie beim Bereichsfoto; danach Rahmen (silber) und mittig darunter Leiste mit **Start** und **×**/Esc – aufgenommen wird erst nach Start; dann Rahmen rot, Leiste mit Laufzeit und **Stopp** (gehören zur App, daher nicht im Video), Laufzeit in der Menüleiste; Stopp per Knopf, Kürzel oder Menü; Datei im Bildordner, als Bildschirmaufnahme gekennzeichnet, Finder zeigt sie. In der App geprüft (Menü und Kürzel, 11-s-Video 1512×756, Einzelbild ohne Rahmen/Leiste; Start erst per Klick, Stopp per Knopf, Abbrechen per Esc und ×, ohne Datei). Windows: Windows Graphics Capture über `windows-capture` (Bildschirm aufnehmen, Bereich ausschneiden, H.264/MP4 über Media Foundation, Mauszeiger an und gelber Rand aus, wo das System es erlaubt); Rahmen und Leiste per `WDA_EXCLUDEFROMCAPTURE` aus der Aufnahme genommen; besteht clippy pedantic, auf einem echten Windows-Rechner noch nicht getestet |
| A1 | Nach der Aufnahme | ◐ | Zeigen / Kopieren / Speichern kombinierbar |
| A2 | Mini-Vorschau | ☑ | Unten rechts; Kopieren, Speichern, Bearbeiten, Schließen; Schließen manuell oder bei Klick daneben |
| A4 | Esc-Verhalten | ☑ | Kopieren-Knopf schließt den Editor bzw. die Vorschau ebenfalls (Wunsch 2026-10-14). |
| A4 (Detail) | Esc-Verhalten | ◐ | Esc im Editor: optional kopieren/speichern, dann schließen |
| A5 | Mitteilungen | ◐ | macOS: eigenes UserNotifications-Framework (vorher kamen keine Mitteilungen an), Banner auch im Vordergrund; Windows: Toast. Editor: „Text erkennen“ zeigt jetzt „Text kopiert: …“ bzw. „Kein Text erkannt“ im Fenster (im Editor geprüft) |
| E1 | Editor-Fenster | ☑ | Werkzeugleiste in der Titelleiste, Infos rechts (Farbe, Größe, Zoom) |
| E2 | Zoom | ◐ | Einpassen, 100 %, ⌘+/⌘−/⌘0/⌘1, Pinch/⌘-Scrollrad; „100 % bevorzugen“ |
| E5 | px/pt | ◐ | Klick auf die Größenanzeige |
| E6 | Pipette + Farbformate | ☑ | HEX, RGB, CSS rgba, OKLCH; Klick wechselt Format, Tab kopiert |
| E10 | Undo/Redo | ☑ | ⌘Z / ⇧⌘Z, Ziehbewegungen als ein Schritt, 100 Schritte |
| T1–T10 | Werkzeuge | ☑ | Alle zehn in der App geprüft. Textwerkzeug war defekt (Klick nahm dem neuen Textfeld den Fokus, Text verschwand) – behoben |
| T1–T10 (Detail) | Werkzeuge | ◐ | Pfeil, Text, Rechteck, Oval, Linie, Freihand, Textmarker, Verpixeln/Weichzeichnen, Spotlight, Zähler; Einzeltasten A T R O L D H B S C, V = Auswahl; ⇧ = 45°/Quadrat |
| T16 | Zuschneiden | ☑ | Auswahl ziehen, ↩; Zuschnitt zurücksetzen im Menü |
| T20 | Markierungen bearbeiten | ◐ | Auswählen, Verschieben, Griffe, Pfeiltasten (⇧ = 10 pt), ⌘D, Entf |
| T21 | Farbwahl | ◐ | Eigene Palette + Systemfarbwähler |
| X1–X4 | Kopieren, Sichern, Sichern unter, Format | ◐ | ⇧⌘C/Esc, ⌘S, ⇧⌘S; PNG oder Auto (PNG/JPEG-Heuristik) |
| X5 | Drag-out | ☐ | Braucht natives Ziehen aus der Web-Ansicht – folgt |
| X7 | Pin | ☑ | Rahmenlos, immer vorne, Rechtsklick: 50/100/200 %, halbtransparent, Kopieren, Schließen; Rechtsklickmenü (50/100/200 %, halbtransparent, Schließen) geprüft |
| X10 | Metadaten | ☑ | xattr `kMDItemIsScreenCapture` (Test vorhanden) |
| O1–O4 | OCR/QR | ☑ | Vision lokal, Zeilen in Lesereihenfolge, Zeilenumbrüche entfernen, QR/Barcode |
| M1 | Menüleisten-Icon | ☑ | Alle Aufnahmen, OCR, Zwischenablage, Öffnen, Editor, Einstellungen, Beenden; zeigt Kürzel |
| M2 | Dock-Icon nur bei offenem Fenster | ◐ | |
| M3 | Autostart | ◐ | `SMAppService` bzw. Registry (Windows) |
| M4 | Begrüßung beim ersten Start | ☑ | „Installation erfolgreich“ mit Ort des Symbols und den eingestellten Kürzeln (0.1.1) |
| H1 | Globale Hotkeys | ☑ | 7 belegbare Kürzel mit Recorder; Vorgaben ⇧⌘1, ⇧⌘2, ⌃⌥⌘O; Kürzel während der Aufnahme pausiert; Recorder und globales Auslösen geprüft |

## Praxistest 2026-10-14 (macOS 27, installierte App)

Geprüft und in Ordnung: Vollbild, Bereich, Bereich wiederholen, Fenster wählen, Aktives Fenster, Aus Zwischenablage, Texterkennung (Editor und Bereich), alle Editor-Werkzeuge, QR-Code, Pin-Kontextmenü, Reiter „Erweitert“, Zuschneiden, Undo/Redo, Pipette, Sichern (Retina, Bildschirmfoto-Kennzeichnung), Kopieren (schließt Editor), Anpinnen (Verschieben, Esc), Mini-Vorschau (Sichern, Bearbeiten), Kürzel-Recorder, globale Kürzel, „Öffnen mit“ beim Kaltstart.

Dabei gefunden und behoben:
- Textwerkzeug: Eingabefeld verlor sofort den Fokus.
- „Text erkennen“ im Editor gab keine Rückmeldung; Mitteilungen kamen auf macOS gar nicht an.
- Absturz beim Start per „Öffnen mit“.
- Bilder aus der Zwischenablage/Dateien erschienen doppelt so groß (Pixeldichte fehlte).

Noch nicht in der App geprüft: Sichern unter, Drag & Drop ins Editorfenster, „Öffnen …“-Dialog, Sichtbarkeit der Mitteilungs-Banner (Zustellung laut Systemprotokoll ok), Windows-Version.

## 0.2.0 – Umbenennung und neues Design (2026-10-15)

- Name aus markenrechtlichen Gründen: **Shot-VILLA** (Bundle-Kennung `de.villalife.shot-villa`, Programmdatei `shot-villa`, Repo `JaschaaON/shot-villa`, Website shot-villa.villalife.de, Container-Image `shot-villa-web`).
- Einstellungen der früheren Installation werden beim ersten Start übernommen (Test vorhanden), ein eingeschalteter Autostart wird für die neue App neu angemeldet.
- Design Silber-Grau: kühle Neutraltöne, Grafit als Akzent, metallische Leisten und Knöpfe, Glas-Meldungen, Steel-Grau für Auswahlrahmen; Hell- und Dunkelmodus. Neues Icon (gebürstetes Silber, Grafit-Sucher) als SVG in `src-tauri/icons/icon.svg`. Website und Screenshots angepasst.
- Signatur jetzt mit dem selbstsignierten Zertifikat „Shot-VILLA Dev“; die Bildschirmaufnahme muss für die umbenannte App einmal neu erlaubt werden.

## 0.2.1 – Neues Icon (2026-10-15)

- Icon „Grafit“ (Auswahl aus fünf Entwürfen): dunkles Grafit-Squircle in macOS-Form (Superellipse im Apple-Raster) mit silbernen Sucherecken und Silberpunkt.
- Ursache des weißen Rands beim alten Icon: `qlmanage` hatte das SVG auf deckend Weiß gerastert. Jetzt Rastern per Browser-Canvas mit echter Transparenz (Ecken 0 % deckend), im Dock geprüft.

## Entscheidungen / Hinweise

- 2026-10-14: `clippy::needless_pass_by_value` wird an Tauri-Commands per `#[expect(..., reason)]` gezielt ausgenommen (vom Nutzer freigegeben).
- Markdown-Dateien sind von `oxfmt` ausgenommen, damit `CLAUDE.md` unverändert bleibt.
- Anti-Slop verbietet „shape“ in Bezeichnern: das Editor-Modell heißt deshalb „Mark“ (Markierung).
- Bilddaten zwischen Frontend und Rust laufen als Raw-Body (keine JSON-Zahlenlisten).
- 2026-10-14: Builds werden mit dem selbstsignierten Zertifikat „Shot-VILLA Dev“ (Anmelde-Schlüsselbund) signiert. Die Freigabe für die Bildschirmaufnahme hängt damit am Zertifikat und bleibt über Builds hinweg erhalten (bei ad-hoc-Signatur hing sie am Code-Hash und verfiel bei jedem Build).
- Die Berechtigungsprüfung nutzt zusätzlich zum Preflight den Abruf der aufnehmbaren Inhalte, weil der Preflight-Wert nach einer Freigabe bis zum Neustart veraltet ist.
