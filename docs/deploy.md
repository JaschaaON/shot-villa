# Website betreiben

Die Produktseite liegt in `website/` (statisches HTML/CSS, keine externen Abhängigkeiten). GitHub Actions baut daraus ein Image mit nginx und legt die DMG aus dem neuesten Release dazu. Der Server baut nichts selbst.

```
Release mit DMG  ─┐
Push auf website/ ├─► GitHub Actions ─► ghcr.io/jaschaaon/screeni-web:latest ─► Container ─► Reverse Proxy
```

## Neue Version veröffentlichen

1. Version in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` und `package.json` erhöhen, Versionsangabe in `website/index.html` anpassen. Nach Änderungen an Abhängigkeiten `pnpm notices` ausführen.
2. Universal-Build (Apple-Chip und Intel) erzeugen:
   ```bash
   pnpm tauri build --target universal-apple-darwin --bundles app,dmg
   ```
3. Release mit der DMG anlegen – das stößt den Website-Build an:
   ```bash
   gh release create v0.2.0 "src-tauri/target/universal-apple-darwin/release/bundle/dmg/Screeni_0.2.0_universal.dmg" --title "Screeni 0.2.0" --notes "…"
   ```
4. Auf dem Server das neue Image ziehen und den Container neu starten (z. B. „Update“ in der Container-Verwaltung).

## Einmalige Einrichtung

1. **Image-Zugriff:** Ist das Paket `screeni-web` in der GitHub Container Registry privat, `ghcr.io` mit einem GitHub-Token (Recht `read:packages`) in der Container-Verwaltung hinterlegen – oder unter *Packages → screeni-web → Package settings* die Sichtbarkeit auf *Public* stellen.
2. **Stack anlegen:** [`deploy/compose.yaml`](../deploy/compose.yaml) als Compose-Stack starten (z. B. über die GUI der Container-Verwaltung). Der Container lauscht nur auf `127.0.0.1:8098`.
3. **Route:** Einen Reverse Proxy (mit TLS) für die Domain auf Port `8098` des Servers zeigen lassen.

## Lokal ansehen

```bash
python3 -m http.server 4173 --directory website
```

Die DMG fürs lokale Ansehen nach `website/downloads/Screeni.dmg` kopieren (der Ordner ist von Git ausgenommen).
