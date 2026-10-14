# Spezifikation – Screeni

Screeni nimmt Bildschirmfotos auf, markiert sie und gibt sie weiter – per Tastenkürzel, lokal und ohne Konten. Diese Datei beschreibt Funktionsumfang, Datenmodell, Dateiformate, Tastenkürzel und Plattformanforderungen. Den Umsetzungsstand je Funktion führt [PROGRESS.md](PROGRESS.md).

- **Plattformen:** macOS 14 oder neuer (Apple-Chip und Intel), Windows 10 und 11
- **Technik:** Tauri 2 mit Rust-Backend (Aufnahme, Kürzel, Menüleiste/Infobereich, Dateien, Texterkennung) und React/TypeScript-Frontend (Editor, Einstellungen, Overlay)
- **Grundsätze:** alles bleibt auf dem Gerät; keine Telemetrie, keine Cloud-Dienste, keine Lizenzprüfung, keine In-App-Käufe

Prioritäten: **Muss** (Version 1), **Später**, **Nicht**.

## 1. Funktionen

### 1.1 Aufnehmen

| ID | Funktion | Beschreibung | Prio |
|---|---|---|---|
| C1 | Vollbild | Bildschirm unter dem Mauszeiger in nativer Auflösung; eigene Fenster sind ausgeschlossen. | Muss |
| C2 | Bereich | Alle Bildschirme werden eingefroren; Auswahl mit Fadenkreuz, Lupe und Größenanzeige; Esc bricht ab. | Muss |
| C3 | Systemwerkzeug für Bereiche | Alternativ das interaktive Aufnahmewerkzeug des Betriebssystems nutzen. | Später |
| C4 | Bereich wiederholen | Zuletzt gewählten Bereich erneut aufnehmen. | Muss |
| C5 | Aktives Fenster | Vorderstes Fenster der aktiven App. | Muss |
| C6 | Fenster wählen | Fenster unter dem Mauszeiger hervorheben, per Klick aufnehmen. | Muss |
| C7 | Fensterhintergrund | Für Fensteraufnahmen: Schreibtischbild, transparent, Vollton oder ohne Schatten. | Später |
| C8 | Scrollende Aufnahme (automatisch) | Bereich wählen, Screeni scrollt selbst und setzt die Teilbilder zusammen. | Später |
| C9 | Scrollende Aufnahme (manuell) | Nutzer scrollt, Screeni setzt zusammen. | Später |
| C10 | Zeitverzögert | Vollbild nach einigen Sekunden. | Später |
| C11 | Mauszeiger | Nur bei Vollbild, immer oder nie mit aufnehmen. | Muss |
| C12 | Aus Zwischenablage | Bild aus der Zwischenablage im Editor öffnen. | Muss |
| C13 | Datei öffnen | PNG, JPEG, GIF, HEIC, TIFF, WebP; Dialog, Drag & Drop, „Öffnen mit“. | Muss |
| C14 | Text/QR sofort erkennen | Bereich wählen, Text bzw. QR-Inhalt landet in der Zwischenablage. | Muss |
| C15 | Retina | Native Auflösung; beim Speichern optional auf 1× verkleinern. | Muss |

### 1.2 Nach der Aufnahme

| ID | Funktion | Beschreibung | Prio |
|---|---|---|---|
| A1 | Verhalten | Zeigen, kopieren, speichern – frei kombinierbar. | Muss |
| A2 | Vorschau | Kleines Fenster unten rechts mit Kopieren, Speichern, Bearbeiten, Schließen. | Muss |
| A3 | Bereich öffnet | Bereichsaufnahmen im Editor oder in der Vorschau. | Später |
| A4 | Esc und Kopieren | Esc schließt den Editor (optional mit Kopieren/Speichern); Kopieren schließt Editor bzw. Vorschau. | Muss |
| A5 | Mitteilungen | Systemmitteilungen, z. B. „Gespeichert“ oder „Text kopiert“. | Muss |

### 1.3 Editor

| ID | Funktion | Beschreibung | Prio |
|---|---|---|---|
| E1 | Editorfenster | Werkzeugleiste, Optionsleiste, Zeichenfläche; rechts Farbe unter dem Mauszeiger, Größe, Zoom. | Muss |
| E2 | Zoom | Einpassen, 100 %, stufenlos per Pinch bzw. ⌘/Strg + Scrollrad. | Muss |
| E3 | Vollbild | Editor im Vollbildmodus. | Später |
| E4 | Immer im Vordergrund | Editor über anderen Fenstern. | Später |
| E5 | Punkte/Pixel | Maße in pt oder px. | Muss |
| E6 | Pipette | Farbe unter dem Mauszeiger, Formate HEX, RGB, CSS rgba(), OKLCH; Tab kopiert. | Muss |
| E7 | Kontrast | Kontrastverhältnis (WCAG, APCA) in einer Auswahl. | Später |
| E8 | Messen | Abstände in pt/px messen. | Später |
| E9 | Intelligente Auswahl | Auswahl rastet an Kanten des Inhalts ein. | Später |
| E10 | Widerrufen | Widerrufen und Wiederholen, Ziehbewegungen als ein Schritt. | Muss |

### 1.4 Werkzeuge

| ID | Werkzeug | Taste | Optionen | Prio |
|---|---|---|---|---|
| T1 | Pfeil | A | gefüllte oder offene Spitze, Stärke, Farbe; ⇧ rastet auf 45° | Muss |
| T2 | Text | T | Text mit Kontur oder Etikett, Schriftgröße, Farbe | Muss |
| T3 | Rechteck | R | Rahmen oder gefüllt; ⇧ quadratisch | Muss |
| T4 | Oval | O | Rahmen oder gefüllt; ⇧ kreisrund | Muss |
| T5 | Linie | L | durchgezogen oder gestrichelt | Muss |
| T6 | Freihand | D | geglättet | Muss |
| T7 | Textmarker | H | eigene Markerfarben | Muss |
| T8 | Unkenntlich machen | B | verpixeln oder weichzeichnen | Muss |
| T9 | Spotlight | S | dunkelt alles außer dem Bereich ab | Muss |
| T10 | Zähler | C | nummeriert automatisch weiter | Muss |
| T11 | Lupe | – | vergrößerter Ausschnitt als Markierung | Später |
| T12 | Bemaßung | – | Maßlinie als Markierung | Später |
| T13 | Bild einfügen | – | Bild aus der Zwischenablage als Markierung | Später |
| T14 | Aufnahme anhängen | – | weitere Aufnahme an das Bild anfügen | Später |
| T15 | Hintergrund | – | siehe 1.5 | Später |
| T16 | Zuschneiden | ↩ | auf Auswahl; zurücksetzen | Muss |
| T17 | Bildgröße | – | Breite/Höhe ändern | Später |
| T18 | Einbrennen | – | Markierungen ins Bild übernehmen | Später |
| T19 | Ausrichten | – | an anderen Markierungen einrasten | Später |
| T20 | Markierungen bearbeiten | V | auswählen, verschieben, Griffe, Pfeiltasten, duplizieren, löschen | Muss |
| T21 | Farbwahl | – | eigene Palette plus Systemfarbwähler | Muss |
| T22 | Vorher/Nachher-GIF | – | animiertes GIF aus zwei Bildern | Später |
| T23 | Freistellen | – | Motiv vom Hintergrund lösen | Nicht |

### 1.5 Hintergrund (Präsentation)

| ID | Funktion | Beschreibung | Prio |
|---|---|---|---|
| B1 | Verlauf | linear, radial, Vorlagen, eigene Farben | Später |
| B2 | Vollton | Vorlagen und eigene Farbe | Später |
| B3 | Bild | Schreibtischbild oder eigene Datei | Später |
| B4 | Gestaltung | Abstand, Ecken, Schatten, Seitenverhältnis | Später |
| B5 | Struktur | leichtes Rauschen für Verläufe | Später |

### 1.6 Ausgabe

| ID | Funktion | Beschreibung | Prio |
|---|---|---|---|
| X1 | Kopieren | Bild in die Zwischenablage | Muss |
| X2 | Sichern | in einen Standardordner, Dateiname mit Datum und Uhrzeit | Muss |
| X3 | Sichern unter | Dateidialog, PNG oder JPEG | Muss |
| X4 | Format | PNG oder automatisch (Fotos als JPEG, Oberflächen als PNG) | Muss |
| X5 | Herausziehen | Bild als Datei aus Editor oder Vorschau ziehen | Muss |
| X6 | Drucken | | Später |
| X7 | Anpinnen | rahmenloses Fenster über allen anderen; 50/100/200 %, halbtransparent | Muss |
| X8 | Upload in eigenen S3-Speicher | S3-kompatibler Speicher, Link in die Zwischenablage | Später |
| X10 | Als Bildschirmfoto markieren | Datei-Metadaten, damit das System Screenshots erkennt (macOS) | Muss |

### 1.7 Texterkennung

| ID | Funktion | Beschreibung | Prio |
|---|---|---|---|
| O1 | Text erkennen | im Editor (Auswahl oder ganzes Bild) und direkt vom Bildschirm | Muss |
| O2 | Sprachen | Sprachen des Systems | Muss |
| O3 | Zeilenumbrüche entfernen | Option; Silbentrennung wird zusammengeführt | Muss |
| O4 | QR- und Barcodes | Inhalt in die Zwischenablage | Muss |

### 1.8 Verlauf

| ID | Funktion | Beschreibung | Prio |
|---|---|---|---|
| L1–L7 | Bibliothek | durchsuchbare Galerie gespeicherter Aufnahmen mit Filtern und Texterkennung | Später |

### 1.9 App

| ID | Funktion | Beschreibung | Prio |
|---|---|---|---|
| M1 | Menüleiste / Infobereich | alle Aktionen mit Kürzeln | Muss |
| M2 | Dock | Dock-Symbol nur bei offenem Fenster (macOS) | Muss |
| M3 | Autostart | Start bei der Anmeldung | Muss |
| M4 | Begrüßung | nach der Installation: Ort des Symbols und wichtigste Kürzel | Muss |
| M5 | Erneut öffnen | Editor bei erneutem Start zeigen | Später |
| H1 | Globale Kürzel | sieben frei belegbare Kürzel mit Aufnahmefeld | Muss |
| H2 | Konflikte | belegte Kürzel melden | Muss |
| H3 | Systemkürzel | Hinweis, wie die Systemkürzel freigegeben werden | Muss |
| U1 | URL-Schema | Aufnahmen per Link `screeni://…` auslösen | Später |

## 2. Datenmodell

Alle Koordinaten im Editor sind Pixel des aufgenommenen Bildes, Ursprung oben links. Bildschirm- und Fensterrahmen werden in Punkten (logischen Pixeln) geführt; `scale` rechnet um.

```ts
type Doc = {
  width: number;        // Bildgröße in Pixeln
  height: number;
  scale: number;        // Pixel pro Punkt (2 auf Retina-Displays)
  crop: Rect;           // sichtbarer Ausschnitt
  marks: Mark[];        // Zeichenreihenfolge
};

type Mark =
  | { kind: "arrow"; from: Point; to: Point; arrowStyle: "solid" | "open"; color; size }
  | { kind: "line"; from: Point; to: Point; dashed: boolean; color; size }
  | { kind: "rect" | "oval"; rect: Rect; fill: "outline" | "filled"; color; size }
  | { kind: "freehand"; points: Point[]; color; size }
  | { kind: "highlight"; rect: Rect; color }
  | { kind: "conceal"; rect: Rect; mode: "blur" | "pixelate" }
  | { kind: "spotlight"; rect: Rect }
  | { kind: "counter"; center: Point; value: number; color; size }
  | { kind: "text"; origin: Point; text: string; textStyle: "plain" | "label"; color; size };
```

Der Editorzustand (Dokument, Historie, Werkzeug, Stil, Auswahl) liegt in einem reinen Reducer (`src/editor/state.ts`); gezeichnet wird für Anzeige und Export mit derselben Funktion (`src/editor/render.ts`). Markierungen sind nach dem Speichern Teil des Bildes; ein eigenes Projektformat gibt es nicht.

## 3. Dateien

| Datei | Ort | Inhalt |
|---|---|---|
| Einstellungen | `settings.json` im App-Konfigurationsordner (macOS: `~/Library/Application Support/de.villalife.screeni/`, Windows: `%APPDATA%\de.villalife.screeni\`) | JSON, fehlende Felder bekommen Vorgaben; wird atomar geschrieben |
| Aufnahmen | eingestellter Ordner, Vorgabe Schreibtisch | `Screeni JJJJ-MM-TT um HH.MM.SS.png` bzw. `.jpg`; bei gleichem Namen „ 2“, „ 3“ … |
| Metadaten | macOS: erweitertes Attribut `com.apple.metadata:kMDItemIsScreenCapture` | kennzeichnet die Datei als Bildschirmfoto |

## 4. Tastenkürzel

Global (frei belegbar, Vorgaben): **⇧⌘1** Vollbild · **⇧⌘2** Bereich · **⌃⌥⌘O** Text/QR erkennen. Unter Windows entsprechend mit Strg.

Im Editor: Werkzeuge V A T R O L D H B S C · Esc kopieren und schließen · ⇧⌘C kopieren · ⌘S sichern · ⇧⌘S sichern unter · ⌘P anpinnen · ⇧⌘T Text erkennen · ⌘Z/⇧⌘Z widerrufen/wiederholen · ↩ zuschneiden · ⌘D duplizieren · ⌫ löschen · Pfeiltasten verschieben · ⌘+ ⌘− ⌘0 ⌘1 Zoom · Tab Farbe kopieren. Unter Windows Strg statt ⌘.

## 5. Plattformanforderungen

| | macOS | Windows |
|---|---|---|
| Bildschirmaufnahme | ScreenCaptureKit; Berechtigung „Bildschirmaufnahme“ erforderlich, wird vor jeder Aufnahme geprüft, sonst erscheint eine Anleitung | Windows Graphics Capture über `xcap`; keine Berechtigung nötig |
| Texterkennung | Vision | Windows.Media.Ocr, QR über `rqrr` |
| Zwischenablage | NSPasteboard (PNG und TIFF) | `arboard` |
| Autostart | `SMAppService` | Registry-Eintrag über das Autostart-Plugin |
| Signierung | eigenes Zertifikat, damit die Aufnahmeberechtigung über Builds erhalten bleibt | unsigniert (SmartScreen-Hinweis beim ersten Start) |
| Sandbox | keine | – |

## 6. Entscheidungen

- Keine Sandbox, um Aufnahme und globale Kürzel ohne Einschränkungen nutzen zu können.
- Bilddaten zwischen Frontend und Rust laufen als Binärdaten, nicht als JSON.
- Die Plattformteile stecken hinter gemeinsamen Schnittstellen (`src-tauri/src/{capture,imaging,ocr}/{mac,win}.rs`); die Abläufe in `actions.rs` sind plattformneutral.
- Unter Windows gibt es kein App-Menü (es hinge an jedem Fenster); Editor-Kürzel verarbeitet dort das Frontend.
