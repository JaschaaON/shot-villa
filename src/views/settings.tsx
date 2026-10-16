import { useEffect, useEffectEvent, useState } from "react";
import type { ReactNode } from "react";

import {
  chooseSaveFolder,
  getSettings,
  resumeHotkeys,
  setSettings as saveSettings,
  suspendHotkeys,
} from "../api";
import type { Hotkeys, Settings as SettingsData } from "../api";
import { isMac } from "../platform";
import { formatShortcut, shortcutFromEvent } from "../settings/shortcut";
import { fire } from "../ui/fire";

type Tab = "general" | "hotkeys" | "advanced";

const TABS: { id: Tab; label: string }[] = [
  { id: "general", label: "Allgemein" },
  { id: "hotkeys", label: "Tastenkürzel" },
  { id: "advanced", label: "Erweitert" },
];

const HOTKEY_LABELS: { key: keyof Hotkeys; label: string }[] = [
  { key: "fullscreen", label: "Vollbild aufnehmen" },
  { key: "area", label: "Bereich aufnehmen" },
  { key: "repeatArea", label: "Bereich wiederholen" },
  { key: "anyWindow", label: "Fenster wählen" },
  { key: "activeWindow", label: "Aktives Fenster" },
  { key: "ocr", label: "Text / QR-Code erkennen" },
  { key: "record", label: "Bildschirmaufnahme" },
  { key: "showApp", label: "Editor zeigen" },
];

const Row = ({ label, children }: { label: string; children: ReactNode }) => (
  <div className="row">
    <span className="row-label">{label}</span>
    <div className="row-control">{children}</div>
  </div>
);

const Check = ({
  checked,
  label,
  onChange,
}: {
  checked: boolean;
  label: string;
  onChange: (checked: boolean) => void;
}) => (
  <label className="check">
    <input
      checked={checked}
      onChange={(event) => {
        onChange(event.target.checked);
      }}
      type="checkbox"
    />
    {label}
  </label>
);

interface RecorderProps {
  value: string | null;
  onChange: (value: string | null) => void;
}

const recorderLabel = (recording: boolean, value: string | null) => {
  if (recording) {
    return "Kürzel drücken …";
  }
  return value === null ? "Aufnehmen" : formatShortcut(value);
};

const CURSOR_CHOICES = new Map<string, SettingsData["captureCursor"]>([
  ["auto", "auto"],
  ["exclude", "exclude"],
  ["include", "include"],
]);

const COLOR_CHOICES = new Map<string, SettingsData["colorFormat"]>([
  ["cssRgba", "cssRgba"],
  ["hex", "hex"],
  ["oklch", "oklch"],
  ["rgb", "rgb"],
]);

interface ListenerProps {
  onShortcut: (shortcut: string) => void;
  onCancel: () => void;
}

/**
 * Hört auf das nächste Tastenkürzel, solange es eingeblendet ist. Globale
 * Kürzel sind währenddessen abgeschaltet, damit sie nicht auslösen.
 */
const ShortcutListener = ({ onShortcut, onCancel }: ListenerProps) => {
  const onKey = useEffectEvent((event: KeyboardEvent) => {
    event.preventDefault();
    event.stopPropagation();
    if (event.code === "Escape") {
      onCancel();
      return;
    }
    const shortcut = shortcutFromEvent(event);
    if (shortcut !== null) {
      onShortcut(shortcut);
    }
  });

  useEffect(() => {
    fire(suspendHotkeys)();
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      fire(async () => {
        await resumeHotkeys();
      })();
    };
  }, []);

  return null;
};

/** Feld, das das nächste Tastenkürzel aufnimmt. */
const ShortcutRecorder = ({ value, onChange }: RecorderProps) => {
  const [recording, setRecording] = useState(false);

  return (
    <div className="recorder">
      {recording ? (
        <ShortcutListener
          onCancel={() => {
            setRecording(false);
          }}
          onShortcut={(shortcut) => {
            onChange(shortcut);
            setRecording(false);
          }}
        />
      ) : null}
      <button
        className={recording ? "recording" : undefined}
        onBlur={() => {
          setRecording(false);
        }}
        onClick={() => {
          setRecording(!recording);
        }}
        type="button"
      >
        {recorderLabel(recording, value)}
      </button>
      {value !== null && !recording ? (
        <button
          aria-label="Kürzel entfernen"
          className="clear"
          onClick={() => {
            onChange(null);
          }}
          type="button"
        >
          ×
        </button>
      ) : null}
    </div>
  );
};

export const Settings = () => {
  const [tab, setTab] = useState<Tab>("general");
  const [settings, setSettings] = useState<SettingsData | null>(null);
  const [problems, setProblems] = useState<string[]>([]);

  useEffect(() => {
    const load = async () => {
      setSettings(await getSettings());
    };
    fire(load)();
  }, []);

  if (!settings) {
    return null;
  }

  const update = (change: Partial<SettingsData>) => {
    const next = { ...settings, ...change };
    setSettings(next);
    fire(async () => {
      try {
        setProblems(await saveSettings(next));
      } catch (error) {
        setProblems([String(error)]);
      }
    })();
  };

  const pickFolder = async () => {
    const folder = await chooseSaveFolder();
    if (folder !== null) {
      update({ saveFolder: folder });
    }
  };

  const setHotkey = (key: keyof Hotkeys) => (value: string | null) => {
    update({ hotkeys: { ...settings.hotkeys, [key]: value } });
  };

  return (
    <div className="settings">
      <nav className="tabs">
        {TABS.map(({ id, label }) => (
          <button
            aria-pressed={tab === id}
            className={tab === id ? "active" : undefined}
            key={id}
            onClick={() => {
              setTab(id);
            }}
            type="button"
          >
            {label}
          </button>
        ))}
      </nav>
      <section className="panel">
        {tab === "general" ? (
          <>
            <Row label="Ordner für Bilder">
              <button
                className="path"
                onClick={fire(pickFolder)}
                title={settings.saveFolder ?? "Schreibtisch"}
                type="button"
              >
                {settings.saveFolder ?? "Schreibtisch"}
              </button>
            </Row>
            <Row label="Format">
              <label className="check">
                <input
                  checked={settings.saveFormat === "png"}
                  name="format"
                  onChange={() => {
                    update({ saveFormat: "png" });
                  }}
                  type="radio"
                />
                PNG
              </label>
              <label className="check">
                <input
                  checked={settings.saveFormat === "auto"}
                  name="format"
                  onChange={() => {
                    update({ saveFormat: "auto" });
                  }}
                  type="radio"
                />
                Automatisch (PNG oder JPEG)
              </label>
            </Row>
            <Row label="Retina-Bilder">
              <Check
                checked={settings.downscaleOnSave}
                label="Beim Speichern auf 1× verkleinern"
                onChange={(downscaleOnSave) => {
                  update({ downscaleOnSave });
                }}
              />
            </Row>
            <hr />
            <Row label="Nach der Aufnahme">
              <Check
                checked={settings.afterCapture.show}
                label="Zeigen"
                onChange={(show) => {
                  update({ afterCapture: { ...settings.afterCapture, show } });
                }}
              />
              <Check
                checked={settings.afterCapture.copy}
                label="Kopieren"
                onChange={(copy) => {
                  update({ afterCapture: { ...settings.afterCapture, copy } });
                }}
              />
              <Check
                checked={settings.afterCapture.save}
                label="Speichern"
                onChange={(save) => {
                  update({ afterCapture: { ...settings.afterCapture, save } });
                }}
              />
            </Row>
            <Row label="Bereich öffnet">
              <select
                onChange={(event) => {
                  update({
                    areaOpens:
                      event.target.value === "thumbnail"
                        ? "thumbnail"
                        : "editor",
                  });
                }}
                value={settings.areaOpens}
              >
                <option value="editor">Editor</option>
                <option value="thumbnail">Vorschau</option>
              </select>
            </Row>
            <Row label="Vorschau schließen">
              <select
                onChange={(event) => {
                  update({
                    thumbnailClosing:
                      event.target.value === "automatic"
                        ? "automatic"
                        : "manual",
                  });
                }}
                value={settings.thumbnailClosing}
              >
                <option value="manual">Mit Schließen-Knopf</option>
                <option value="automatic">Automatisch (Klick daneben)</option>
              </select>
            </Row>
            <hr />
            <Row label="Autostart">
              <Check
                checked={settings.launchAtLogin}
                label="Beim Anmelden starten"
                onChange={(launchAtLogin) => {
                  update({ launchAtLogin });
                }}
              />
            </Row>
          </>
        ) : null}
        {tab === "hotkeys" ? (
          <>
            {HOTKEY_LABELS.map(({ key, label }) => (
              <Row key={key} label={label}>
                <ShortcutRecorder
                  onChange={setHotkey(key)}
                  value={settings.hotkeys[key]}
                />
              </Row>
            ))}
            {isMac ? (
              <p className="note">
                Um ⇧⌘3 / ⇧⌘4 zu verwenden, schalte die Bildschirmfoto-Kürzel
                unter Systemeinstellungen → Tastatur → Tastaturkurzbefehle →
                Bildschirmfotos aus.
              </p>
            ) : (
              <p className="note">
                Die Druck-Taste belegt Windows selbst. Soll Shot-VILLA sie
                nutzen, schalte unter Einstellungen → Barrierefreiheit →
                Tastatur „Bildschirmausschnitt mit der Druck-Taste öffnen“ aus.
              </p>
            )}
          </>
        ) : null}
        {tab === "advanced" ? (
          <>
            <Row label="Mauszeiger">
              <select
                onChange={(event) => {
                  const { value } = event.target;
                  update({
                    captureCursor: CURSOR_CHOICES.get(value) ?? "auto",
                  });
                }}
                value={settings.captureCursor}
              >
                <option value="auto">Nur bei Vollbild</option>
                <option value="include">Immer aufnehmen</option>
                <option value="exclude">Nie aufnehmen</option>
              </select>
            </Row>
            <Row label="Texterkennung">
              <Check
                checked={settings.ocrRemoveLineBreaks}
                label="Zeilenumbrüche entfernen"
                onChange={(ocrRemoveLineBreaks) => {
                  update({ ocrRemoveLineBreaks });
                }}
              />
            </Row>
            <Row label="Esc im Editor">
              <Check
                checked={settings.escCopies}
                label="Bild kopieren"
                onChange={(escCopies) => {
                  update({ escCopies });
                }}
              />
              <Check
                checked={settings.escSaves}
                label="Bild speichern"
                onChange={(escSaves) => {
                  update({ escSaves });
                }}
              />
            </Row>
            <Row label="Editor">
              <Check
                checked={settings.alwaysOnTop}
                label="Immer im Vordergrund"
                onChange={(alwaysOnTop) => {
                  update({ alwaysOnTop });
                }}
              />
              <Check
                checked={settings.preferActualSize}
                label="100 % bevorzugen, wenn das Bild passt"
                onChange={(preferActualSize) => {
                  update({ preferActualSize });
                }}
              />
            </Row>
            <Row label="Farbformat">
              <select
                onChange={(event) => {
                  const { value } = event.target;
                  update({ colorFormat: COLOR_CHOICES.get(value) ?? "hex" });
                }}
                value={settings.colorFormat}
              >
                <option value="hex">HEX (#RRGGBB)</option>
                <option value="rgb">RGB (r, g, b)</option>
                <option value="cssRgba">CSS rgba()</option>
                <option value="oklch">OKLCH</option>
              </select>
            </Row>
          </>
        ) : null}
        {problems.length > 0 ? (
          <div className="problems">
            {problems.map((problem) => (
              <p key={problem}>Kürzel nicht verfügbar: {problem}</p>
            ))}
          </div>
        ) : null}
      </section>
    </div>
  );
};
