import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";

import {
  availableUpdate,
  checkForUpdate,
  installUpdate,
  onUpdateProgress,
} from "../api";
import type { UpdateProgress } from "../api";
import { progressLabel } from "../settings/update-status";
import { fire } from "../ui/fire";

type State =
  | { kind: "checking" }
  | { kind: "current" }
  | { kind: "installing"; version: string; progress: UpdateProgress | null }
  | { kind: "failed"; message: string };

type SetState = (state: State) => void;

const statusText = (state: State): string => {
  switch (state.kind) {
    case "checking": {
      return "Suche nach Updates …";
    }
    case "current": {
      return "Pivolo ist auf dem neuesten Stand.";
    }
    case "installing": {
      const detail = state.progress
        ? progressLabel(state.progress)
        : "Starte …";
      return `Version ${state.version}: ${detail}`;
    }
    case "failed": {
      return state.message;
    }
    default: {
      return "";
    }
  }
};

/** Lädt die Version, installiert sie und startet die App neu. */
const install = async (version: string, setState: SetState) => {
  setState({ kind: "installing", progress: null, version });
  try {
    await installUpdate();
  } catch (error) {
    setState({ kind: "failed", message: String(error) });
  }
};

/** Sucht nach einer neueren Version und installiert sie gleich, wenn es eine gibt. */
const searchAndInstall = async (setState: SetState) => {
  setState({ kind: "checking" });
  try {
    const found = await checkForUpdate();
    if (found) {
      await install(found.version, setState);
    } else {
      setState({ kind: "current" });
    }
  } catch (error) {
    setState({ kind: "failed", message: String(error) });
  }
};

interface Props {
  autoCheck: boolean;
  onAutoCheckChange: (autoCheck: boolean) => void;
}

/**
 * Tab „Updates“: Beim Öffnen und auf Knopfdruck wird gesucht; eine gefundene
 * Version wird ohne weitere Rückfrage installiert, danach startet die App neu.
 */
export const Updates = ({ autoCheck, onAutoCheckChange }: Props) => {
  const [version, setVersion] = useState("");
  const [state, setState] = useState<State>({ kind: "checking" });

  useEffect(() => {
    let active = true;
    const start = async () => {
      setVersion(await getVersion());
      // Hat die Suche im Hintergrund schon etwas gefunden, gleich installieren.
      const known = await availableUpdate();
      if (!active) {
        return;
      }
      await (known
        ? install(known.version, setState)
        : searchAndInstall(setState));
    };
    fire(start)();
    const listening = onUpdateProgress((progress) => {
      setState((current) =>
        current.kind === "installing" ? { ...current, progress } : current
      );
    });
    return () => {
      active = false;
      fire(async () => {
        (await listening)();
      })();
    };
  }, []);

  const busy = state.kind === "checking" || state.kind === "installing";

  return (
    <>
      <div className="row">
        <span className="row-label">Installierte Version</span>
        <div className="row-control">{version}</div>
      </div>
      <div className="row">
        <span className="row-label">Status</span>
        <output className="row-control">{statusText(state)}</output>
      </div>
      <div className="row">
        <span className="row-label" />
        <div className="row-control">
          <button
            disabled={busy}
            onClick={fire(async () => {
              await searchAndInstall(setState);
            })}
            type="button"
          >
            Nach Updates suchen
          </button>
        </div>
      </div>
      <hr />
      <div className="row">
        <span className="row-label">Automatisch</span>
        <div className="row-control">
          <label className="check">
            <input
              checked={autoCheck}
              onChange={(event) => {
                onAutoCheckChange(event.target.checked);
              }}
              type="checkbox"
            />
            Beim Start und täglich nach Updates suchen
          </label>
        </div>
      </div>
      <p className="note">
        Eine neue Version wird heruntergeladen, geprüft und installiert; danach
        startet Pivolo von selbst neu. Bei der Suche lädt die App nur eine
        kleine Versionsdatei von github.com. Aufnahmen und persönliche Daten
        werden nicht übertragen; GitHub sieht wie bei jedem Abruf deine
        IP-Adresse. Das Update wird vor der Installation anhand einer Signatur
        geprüft.
      </p>
    </>
  );
};
