/**
 * Baut die `latest.json`, die der Updater der App abruft. Sie nennt die neue
 * Version und je Plattform das signierte Paket samt Signatur.
 */

export interface PlatformFile {
  url: string;
  signature: string;
}

export interface Manifest {
  version: string;
  notes: string;
  pub_date: string;
  platforms: Record<string, PlatformFile>;
}

interface ManifestInput {
  version: string;
  publishedAt: Date;
  /** Universelles Paket (Apple-Silicon und Intel). */
  macos: PlatformFile;
  windows: PlatformFile;
}

/** Version aus einem Tag wie `v0.3.3`. */
export const versionFromTag = (tag: string): string => tag.replace(/^v/u, "");

/** Download-Adresse einer Datei an einem GitHub-Release. */
export const assetUrl = (repo: string, tag: string, file: string): string =>
  `https://github.com/${repo}/releases/download/${tag}/${encodeURIComponent(file)}`;

export const buildManifest = ({
  version,
  publishedAt,
  macos,
  windows,
}: ManifestInput): Manifest => ({
  notes: `Pivolo ${version}`,
  platforms: {
    "darwin-aarch64": macos,
    "darwin-x86_64": macos,
    "windows-x86_64": windows,
  },
  pub_date: publishedAt.toISOString(),
  version,
});
