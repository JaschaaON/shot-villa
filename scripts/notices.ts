/**
 * Sammelt die Lizenztexte aller Bibliotheken, die in Shot-VILLA ausgeliefert
 * werden (Rust-Crates und npm-Pakete des Frontends), und schreibt sie nach
 * THIRD-PARTY-NOTICES.md und website/third-party-notices.txt.
 *
 * Aufruf: pnpm notices
 */
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

interface Component {
  name: string;
  version: string;
  license: string;
  directory: string;
}

interface CargoPackage {
  name: string;
  version: string;
  license: string | null;
  manifest_path: string;
}

interface NpmPackage {
  name: string;
  versions: string[];
  paths: string[];
}

const LICENSE_FILE = /^(?:licen[cs]e|copying|notice)/iu;

const isObject = (value: unknown): value is object =>
  typeof value === "object" && value !== null;

const isString = (value: unknown): value is string => typeof value === "string";

const isStringArray = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every(isString);

const isCargoPackage = (value: unknown): value is CargoPackage =>
  isObject(value) &&
  "name" in value &&
  isString(value.name) &&
  "version" in value &&
  isString(value.version) &&
  "manifest_path" in value &&
  isString(value.manifest_path) &&
  "license" in value &&
  (isString(value.license) || value.license === null);

const isNpmPackage = (value: unknown): value is NpmPackage =>
  isObject(value) &&
  "name" in value &&
  isString(value.name) &&
  "versions" in value &&
  isStringArray(value.versions) &&
  "paths" in value &&
  isStringArray(value.paths);

/** Liest `cargo metadata` und gibt alle Crates außer Shot-VILLA selbst zurück. */
const cargoComponents = (): Component[] => {
  const output = execFileSync(
    "cargo",
    [
      "metadata",
      "--format-version",
      "1",
      "--manifest-path",
      "src-tauri/Cargo.toml",
    ],
    { encoding: "utf-8", maxBuffer: 256 * 1024 * 1024 }
  );
  const metadata: unknown = JSON.parse(output);
  const packages =
    isObject(metadata) &&
    "packages" in metadata &&
    Array.isArray(metadata.packages)
      ? metadata.packages.filter(isCargoPackage)
      : [];
  return packages
    .filter((pkg) => pkg.name !== "shot-villa")
    .map((pkg) => ({
      directory: path.dirname(pkg.manifest_path),
      license: pkg.license ?? "siehe Lizenztext",
      name: pkg.name,
      version: pkg.version,
    }));
};

/** Liest die Produktionsabhängigkeiten des Frontends über pnpm. */
const npmComponents = (): Component[] => {
  const output = execFileSync(
    "pnpm",
    ["licenses", "list", "--prod", "--json"],
    {
      encoding: "utf-8",
    }
  );
  const byLicense: unknown = JSON.parse(output);
  if (!isObject(byLicense)) {
    return [];
  }
  return Object.entries(byLicense).flatMap(([license, entries]) =>
    (Array.isArray(entries) ? entries.filter(isNpmPackage) : []).map((pkg) => ({
      directory: pkg.paths[0] ?? "",
      license,
      name: pkg.name,
      version: pkg.versions[0] ?? "",
    }))
  );
};

/** Inhalt aller Lizenzdateien eines Pakets. */
const licenseText = (directory: string): string =>
  readdirSync(directory)
    .filter((file) => LICENSE_FILE.test(file))
    .toSorted()
    .map((file) => readFileSync(path.join(directory, file), "utf-8").trim())
    .join("\n\n");

const build = (): string => {
  const components = [...npmComponents(), ...cargoComponents()].toSorted(
    (a, b) => a.name.localeCompare(b.name)
  );
  // Gleiche Lizenztexte nur einmal abdrucken.
  const groups = new Map<string, { text: string; members: Component[] }>();
  for (const component of components) {
    const text =
      licenseText(component.directory) || `Lizenz: ${component.license}`;
    const key = createHash("sha256").update(text).digest("hex");
    const group = groups.get(key) ?? { members: [], text };
    group.members.push(component);
    groups.set(key, group);
  }

  const header = [
    "# Lizenzen der Drittsoftware",
    "",
    "Shot-VILLA © 2026 Jascha Coenen steht unter der MIT-Lizenz (siehe LICENSE).",
    `Shot-VILLA enthält ${components.length} Bibliotheken anderer Autorinnen und Autoren.`,
    "Ihre Lizenzen folgen hier; Pakete mit gleichem Lizenztext stehen zusammen.",
    "Diese Datei wird mit `pnpm notices` erzeugt.",
    "",
  ];
  const sections = [...groups.values()].map(({ members, text }) =>
    [
      "---",
      "",
      ...members.map(
        (member) => `- ${member.name} ${member.version} (${member.license})`
      ),
      "",
      "```",
      text,
      "```",
      "",
    ].join("\n")
  );
  return [...header, ...sections].join("\n");
};

const notices = build();
writeFileSync("THIRD-PARTY-NOTICES.md", notices);
writeFileSync("website/third-party-notices.txt", notices);
console.log(
  `Lizenzen geschrieben: ${notices.length.toLocaleString("de-DE")} Zeichen`
);
