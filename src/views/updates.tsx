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
  | { kind: "available"; version: string }
  | { kind: "installing"; progress: UpdateProgress | null }
  | { kind: "failed"; message: string };

const statusText = (state: State): string => {
  switch (state.kind) {
    case "checking": {
      return "Suche nach Updates …";
    }
    case "current": {
      return "Shot-VILLA ist auf dem neuesten Stand.";
    }
    case "available": {
      return `Version ${state.version} ist verfügbar.`;
    }
    case "installing": {
      return state.progress ? progressLabel(state.progress) : "Starte …";
    }
    case "failed": {
      return state.message;
    }
    default: {
      return "";
    }
  }
};

/** Fragt den Update-Server und übersetzt die Antwort in einen Zustand. */
const lookUp = async (): Promise<State> => {
  try {
    const found = await checkForUpdate();
    return found
      ? { kind: "available", version: found.version }
      : { kind: "current" };
  } catch (error) {
    return { kind: "failed", message: String(error) };
  }
};

interface Props {
  autoCheck: boolean;
  onAutoCheckChange: (autoCheck: boolean) => void;
}

/** Tab „Updates“: sucht beim Öffnen, installiert auf Knopfdruck. */
export const Updates = ({ autoCheck, onAutoCheckChange }: Props) => {
  const [version, setVersion] = useState("");
  const [state, setState] = useState<State>({ kind: "checking" });

  const search = async () => {
    setState({ kind: "checking" });
    setState(await lookUp());
  };

  const install = async () => {
    setState({ kind: "installing", progress: null });
    try {
      await installUpdate();
    } catch (error) {
      setState({ kind: "failed", message: String(error) });
    }
  };

  useEffect(() => {
    let active = true;
    const start = async () => {
      setVersion(await getVersion());
      const known = await availableUpdate();
      if (!active) {
        return;
      }
      const next: State = known
        ? { kind: "available", version: known.version }
        : await lookUp();
      if (active) {
        setState(next);
      }
    };
    fire(start)();
    const listening = onUpdateProgress((progress) => {
      setState({ kind: "installing", progress });
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
          {state.kind === "available" ? (
            <button className="primary" onClick={fire(install)} type="button">
              Installieren und neu starten
            </button>
          ) : null}
          <button disabled={busy} onClick={fire(search)} type="button">
            Erneut suchen
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
        Bei der Suche lädt Shot-VILLA nur eine kleine Versionsdatei von
        github.com. Aufnahmen und persönliche Daten werden nicht übertragen;
        GitHub sieht wie bei jedem Abruf deine IP-Adresse. Das Update wird vor
        der Installation anhand einer Signatur geprüft.
      </p>
    </>
  );
};
