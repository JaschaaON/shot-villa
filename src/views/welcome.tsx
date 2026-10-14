import { closeWindow, runAction } from "../api";
import type { ActionId, Hotkeys } from "../api";
import { isMac } from "../platform";
import { formatShortcut } from "../settings/shortcut";
import { useSettings } from "../settings/use-settings";
import { fire } from "../ui/fire";

const SHORTCUTS: { key: keyof Hotkeys; label: string }[] = [
  { key: "area", label: "Bereich aufnehmen" },
  { key: "fullscreen", label: "Vollbild aufnehmen" },
  { key: "ocr", label: "Text oder QR-Code erkennen" },
];

/** Schließt die Begrüßung und startet eine Aktion. */
const closeAndRun = (action: ActionId) =>
  fire(async () => {
    await runAction(action);
    await closeWindow();
  });

/** Begrüßung nach der Installation. */
export const Welcome = () => {
  const settings = useSettings();
  const place = isMac
    ? "in der Menüleiste oben rechts"
    : "im Infobereich der Taskleiste unten rechts";

  return (
    <main className="welcome">
      <img
        alt=""
        className="welcome-icon"
        height="72"
        src="/icon.png"
        width="72"
      />
      <h1>Installation erfolgreich</h1>
      <p className="welcome-lead">
        Screeni läuft jetzt {place} und wartet auf dein erstes Bildschirmfoto.
      </p>

      <dl className="welcome-keys">
        {SHORTCUTS.map(({ key, label }) => {
          const shortcut = settings?.hotkeys[key] ?? null;
          return (
            <div key={key}>
              <dt>{shortcut === null ? "–" : formatShortcut(shortcut)}</dt>
              <dd>{label}</dd>
            </div>
          );
        })}
      </dl>

      <p className="welcome-note">
        {isMac
          ? "Beim ersten Aufnehmen fragt macOS nach der Berechtigung „Bildschirmaufnahme“. Erlaube sie und starte Screeni danach einmal neu."
          : "Alle Kürzel kannst du in den Einstellungen ändern. Screeni startet auf Wunsch mit Windows."}
      </p>

      <div className="welcome-actions">
        <button onClick={closeAndRun("settings")} type="button">
          Einstellungen
        </button>
        <button
          className="primary"
          onClick={closeAndRun("capture-area")}
          type="button"
        >
          Bereich aufnehmen
        </button>
        <button onClick={fire(closeWindow)} type="button">
          Fertig
        </button>
      </div>
    </main>
  );
};
