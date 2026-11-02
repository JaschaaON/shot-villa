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
3. Der Workflow legt das Release an, baut das universelle macOS-DMG und den Windows-Installer, hängt beides an und stößt den Website-Build an. Das Image `ghcr.io/jaschaaon/shot-villa-web:latest` enthält danach die neuen Downloads.
4. Auf dem Server das neue Image ziehen und den Container neu starten (z. B. „Update“ in der Container-Verwaltung).

**Signatur:** Auf dem Runner gibt es das lokale Zertifikat „Shot-VILLA Dev“ nicht, das DMG ist deshalb ad hoc signiert. macOS fragt nach jedem Update erneut nach den Freigaben (Bildschirmaufnahme, Bedienungshilfen). Wer das vermeiden will, hinterlegt ein Zertifikat als GitHub-Secret und trägt es in den Workflow ein. Ein lokaler Build mit `pnpm tauri build --target universal-apple-darwin --bundles app,dmg` und dem Zertifikat bleibt möglich.

## Einmalige Einrichtung

1. **Image-Zugriff:** Ist das Paket `shot-villa-web` in der GitHub Container Registry privat, `ghcr.io` mit einem GitHub-Token (Recht `read:packages`) in der Container-Verwaltung hinterlegen – oder unter *Packages → shot-villa-web → Package settings* die Sichtbarkeit auf *Public* stellen.
2. **Stack anlegen:** [`deploy/compose.yaml`](../deploy/compose.yaml) als Compose-Stack starten (z. B. über die GUI der Container-Verwaltung). Der Container lauscht nur auf `127.0.0.1:8098`.
3. **Route:** Einen Reverse Proxy (mit TLS) für die Domain auf Port `8098` des Servers zeigen lassen.

## Lokal ansehen

```bash
python3 -m http.server 4173 --directory website
```

Die DMG fürs lokale Ansehen nach `website/downloads/Shot-VILLA.dmg` kopieren (der Ordner ist von Git ausgenommen).
