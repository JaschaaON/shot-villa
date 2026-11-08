import { describe, expect, it } from "vitest";

import { assetUrl, buildManifest, versionFromTag } from "./updater-manifest.ts";

describe("Update-Manifest", () => {
  it("nimmt die Version aus dem Tag", () => {
    expect(versionFromTag("v0.3.3")).toBe("0.3.3");
    expect(versionFromTag("0.3.3")).toBe("0.3.3");
  });

  it("verlinkt Dateien des Releases mit kodiertem Namen", () => {
    expect(assetUrl("JaschaaON/shot-villa", "v0.3.3", "Pivolo 1.exe")).toBe(
      "https://github.com/JaschaaON/shot-villa/releases/download/v0.3.3/Pivolo%201.exe"
    );
  });

  it("verwendet das universelle Mac-Paket für beide Prozessoren", () => {
    const macos = { signature: "mac", url: "https://example.test/mac.tar.gz" };
    const windows = { signature: "win", url: "https://example.test/win.exe" };
    const manifest = buildManifest({
      macos,
      publishedAt: new Date("2026-11-02T10:00:00Z"),
      version: "0.3.3",
      windows,
    });
    expect(manifest.version).toBe("0.3.3");
    expect(manifest.pub_date).toBe("2026-11-02T10:00:00.000Z");
    expect(manifest.platforms).toStrictEqual({
      "darwin-aarch64": macos,
      "darwin-x86_64": macos,
      "windows-x86_64": windows,
    });
  });
});
