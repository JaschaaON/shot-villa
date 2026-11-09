# Website betreiben

Die Produktseite liegt in `website/` (statisches HTML/CSS, keine externen Abhängigkeiten). GitHub Actions baut daraus ein Image mit nginx und legt die DMG aus dem neuesten Release dazu. Der Server baut nichts selbst.

```
Release mit DMG  ─┐
Push auf website/ ├─► GitHub Actions ─► ghcr.io/jaschaaon/shot-villa-web:latest ─► Container ─► Reverse Proxy
```

## Neue Version veröffentlichen

Alles baut GitHub Actions ([`release.yml`](../.github/workflows/release.yml)); ein Mac ist dafür nicht nötig.

1. Version in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (samt `Cargo.lock`) und `package.json` erhöhen und nach `main` pushen. Nach Änderungen an Abhängigkeiten vorher `pnpm notices` ausführen.
2. Tag setzen und pushen:
   ```bash
   git tag v0.3.3 && git push origin v0.3.3
   ```
3. Der Workflow legt das Release an, baut das universelle macOS-DMG und den Windows-Installer, hängt beides samt signierten Update-Paketen (`*.app.tar.gz`, `*-setup.exe` mit `.sig`) an, erzeugt `latest.json` und stößt den Website-Build an. Das Image `ghcr.io/jaschaaon/shot-villa-web:latest` enthält danach die neuen Downloads.
4. Auf dem Server das neue Image ziehen und den Container neu starten (z. B. „Update“ in der Container-Verwaltung).

## Updates in der App

Installierte Apps holen neue Versionen selbst. Sie rufen dazu `https://github.com/JaschaaON/shot-villa/releases/latest/download/latest.json` ab, eine statische Datei am neuesten Release. Das ist kein API-Aufruf und unterliegt nicht dem Abruflimit der GitHub-API (60 Abrufe pro Stunde ohne Anmeldung). Pakete prüft die App mit dem öffentlichen Schlüssel aus `tauri.conf.json` (`plugins.updater.pubkey`); `requireSignedVersion` verhindert, dass ein älteres, gültig signiertes Paket als neues ausgegeben wird.

### Umbenennung auf Pivolo

Version 0.3.8 ist als Migrationsupdate für vorhandene Installationen vorgesehen. `productName` wird zu `Pivolo`; Bundle-Kennung `de.villalife.shot-villa`, Signatur und Updater-Adresse bleiben stabil. Damit kann eine vorhandene Installation das signierte Update annehmen und macOS-Freigaben behalten. Unter Windows nutzt `src-tauri/windows/pivolo-migration.nsh` den vorhandenen Shot-VILLA-Installationsordner, ersetzt die dortigen Dateien und entfernt die alten Installations-Metadaten und Verknüpfungen. App-Daten unter der Bundle-Kennung werden nicht gelöscht. Vor der Veröffentlichung muss das Upgrade von einer installierten Shot-VILLA-Version auf macOS und Windows praktisch geprüft werden.

**Einmalige Einrichtung (Signaturschlüssel):**

1. Schlüsselpaar erzeugen: `pnpm tauri signer generate -w ~/.tauri/shot-villa-updater.key`. Der öffentliche Teil (`.key.pub`) steht bereits in `tauri.conf.json`.
2. Den privaten Schlüssel als GitHub-Secret `TAURI_SIGNING_PRIVATE_KEY` hinterlegen (Inhalt der Datei), bei gesetztem Passwort zusätzlich `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: `gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/shot-villa-updater.key`.
3. Den privaten Schlüssel zusätzlich im Passwortmanager sichern. **Geht er verloren, kann keine installierte App mehr aktualisiert werden**; dann hilft nur eine manuelle Neuinstallation mit neuem Schlüssel. Er gehört nie ins Repository.

**Ersteinführung:** Nur Versionen ab der ersten mit Updater gebauten Version suchen selbst nach Updates. Ältere Installationen (bis 0.3.3) werden einmal von Hand ersetzt.

**Lokaler Build:** Wer das DMG selbst baut, setzt `TAURI_SIGNING_PRIVATE_KEY` (Pfad zur Schlüsseldatei) und `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, sonst bricht `pnpm tauri build` ab, weil Update-Pakete signiert werden müssen.

**Anderer Update-Server:** Die Adresse steht in `tauri.conf.json` unter `plugins.updater.endpoints` (mehrere sind möglich, sie werden der Reihe nach versucht). `latest.json` und die Pakete müssen dort öffentlich per HTTPS erreichbar sein.

**Signatur (macOS):** Alle Builds, auch die von GitHub Actions, tragen dieselbe Signatur: das selbstsignierte Zertifikat „Shot-VILLA Signing“. macOS knüpft die Freigaben (Bildschirmaufnahme, Bedienungshilfen) an diese Identität, sie bleiben deshalb über alle Updates erhalten. Bei ad-hoc-Signatur (jeder Build ein neuer Code-Hash) würde macOS nach jedem Update erneut fragen.

- **Secrets im Repo:** `APPLE_CERTIFICATE` (p12 als Base64), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` (`Shot-VILLA Signing`). `release.yml` richtet damit ein temporäres Schlüsselbund ein und prüft die Signatur im Lauf.
- **Sicherung:** `~/.tauri/shot-villa-signing.p12` und `signing.password` im Passwortmanager ablegen. Das Zertifikat gilt bis 2046. **Geht es verloren, ändert sich die Identität, und macOS fragt einmal neu nach den Freigaben.** Dafür setzt die App ihre Freigaben beim ersten Start einmal selbst zurück (`PERMISSION_EPOCH` in `permission.rs`); bei jedem künftigen Identitätswechsel die Zahl erhöhen.
- **Lokaler Build:** Das Zertifikat einmal importieren: `security import ~/.tauri/shot-villa-signing.p12 -k ~/Library/Keychains/login.keychain-db -P "$(cat ~/.tauri/signing.password)" -T /usr/bin/codesign`. `tauri.conf.json` verweist auf „Shot-VILLA Signing“.
- **Neues Zertifikat erzeugen:** `openssl req -x509 -newkey rsa:2048 -nodes -days 7300 -subj "/CN=Shot-VILLA Signing" -addext "keyUsage=critical,digitalSignature" -addext "extendedKeyUsage=critical,codeSigning" -addext "basicConstraints=critical,CA:false" -keyout key.pem -out cert.pem`, dann `openssl pkcs12 -export -legacy -inkey key.pem -in cert.pem -name "Shot-VILLA Signing" -out shot-villa-signing.p12` (ohne `-legacy` kann `security import` die Datei nicht lesen).

## Einmalige Einrichtung

1. **Image-Zugriff:** Ist das Paket `shot-villa-web` in der GitHub Container Registry privat, `ghcr.io` mit einem GitHub-Token (Recht `read:packages`) in der Container-Verwaltung hinterlegen – oder unter *Packages → shot-villa-web → Package settings* die Sichtbarkeit auf *Public* stellen.
2. **Stack anlegen:** [`deploy/compose.yaml`](../deploy/compose.yaml) als Compose-Stack starten (z. B. über die GUI der Container-Verwaltung). Der Container lauscht nur auf `127.0.0.1:8098`.
3. **Route:** Einen Reverse Proxy (mit TLS) für die Domain auf Port `8098` des Servers zeigen lassen.

## Lokal ansehen

```bash
python3 -m http.server 4173 --directory website
```

Die DMG fürs lokale Ansehen nach `website/downloads/Pivolo.dmg` kopieren (der Ordner ist von Git ausgenommen).

## Produktbilder neu erzeugen

Die Bilder in `docs/images/` und `website/images/` entstehen ohne die App: `tools/screenshots/capture.mjs` lässt die gebaute Oberfläche in einem Headless-Chromium laufen, ersetzt die Tauri-Schnittstelle durch eine Attrappe (`tauri-mock.js`) und zeichnet die Markierungen im Editor per Maus und Tastatur auf ein erfundenes Beispielbild (`sample.html`).

```bash
pnpm build
node tools/screenshots/capture.mjs /tmp/bilder            # alle Bilder
node tools/screenshots/capture.mjs /tmp/bilder editor     # nur eines
```

Danach `editor`, `settings-general`, `settings-shortcuts` und `settings-advanced` nach `docs/images/` und `website/images/` kopieren; die übrigen Bilder (Begrüßung, Vorschau, Aufnahmeleiste …) dienen der Sichtprüfung. Auf Linux braucht das Skript Schriften für ⌘ ⇧ ⌥ ⌃ (Umgebungsvariable `SYMBOLS_FONT`, Einzelheiten im Kopf der Datei). Chromium wird unter `~/.cache/ms-playwright` gesucht oder über `CHROME` angegeben.

