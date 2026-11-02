/**
 * Schreibt die `latest.json` für ein Release auf die Standardausgabe. Im Ordner
 * liegen die Pakete des Releases mit ihren `.sig`-Dateien.
 *
 * Aufruf: node --experimental-strip-types scripts/write-updater-manifest.ts <Besitzer/Repo> <Tag> <Ordner>
 */
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

import { assetUrl, buildManifest, versionFromTag } from "./updater-manifest.ts";
import type { PlatformFile } from "./updater-manifest.ts";

/** Sucht das Paket, dessen Name auf `suffix` endet, samt Signatur. */
const findPackage = (
  folder: string,
  repo: string,
  tag: string,
  suffix: string
): PlatformFile => {
  const signatures = readdirSync(folder).filter((name) =>
    name.endsWith(`${suffix}.sig`)
  );
  const [signatureFile, ...others] = signatures;
  if (signatureFile === undefined || others.length > 0) {
    throw new Error(
      `Erwartet genau eine Signatur auf ${suffix}, gefunden: ${signatures.length}`
    );
  }
  return {
    signature: readFileSync(path.join(folder, signatureFile), "utf-8").trim(),
    url: assetUrl(repo, tag, signatureFile.replace(/\.sig$/u, "")),
  };
};

const [repo, tag, folder] = process.argv.slice(2);
if (repo === undefined || tag === undefined || folder === undefined) {
  throw new Error("Aufruf: <Besitzer/Repo> <Tag> <Ordner>");
}

const manifest = buildManifest({
  macos: findPackage(folder, repo, tag, ".app.tar.gz"),
  publishedAt: new Date(),
  version: versionFromTag(tag),
  windows: findPackage(folder, repo, tag, "-setup.exe"),
});
process.stdout.write(`${JSON.stringify(manifest, null, 2)}\n`);
