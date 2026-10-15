import { useEffect, useState } from "react";

import { openScreenAccessSettings, restartApp, screenAccess } from "../api";
import type { ScreenAccess } from "../api";

const POLL_INTERVAL_MS = 1500;

export const Permission = () => {
  const [access, setAccess] = useState<ScreenAccess>("denied");

  useEffect(() => {
    const check = async () => {
      try {
        setAccess(await screenAccess());
      } catch (error) {
        console.error(error);
      }
    };
    void check();
    const timer = setInterval(() => {
      void check();
    }, POLL_INTERVAL_MS);
    return () => {
      clearInterval(timer);
    };
  }, []);

  return (
    <main className="permission">
      <h1>Bildschirmaufnahme erlauben</h1>
      <p>
        Shot-VILLA braucht die Berechtigung „Bildschirm- und
        Systemaudioaufnahme“, sonst enthalten Aufnahmen nur den
        Schreibtischhintergrund.
      </p>
      <ol>
        <li>Systemeinstellungen öffnen.</li>
        <li>Shot-VILLA in der Liste einschalten.</li>
        <li>
          Shot-VILLA neu starten – macOS übernimmt die Freigabe erst danach.
        </li>
      </ol>
      <div className="actions">
        <button
          onClick={() => {
            void openScreenAccessSettings();
          }}
          type="button"
        >
          Systemeinstellungen öffnen
        </button>
        <button
          className="primary"
          onClick={() => {
            void restartApp();
          }}
          type="button"
        >
          Neu starten
        </button>
      </div>
      {access === "granted" ? (
        <p className="status">Freigabe erkannt. Bitte jetzt neu starten.</p>
      ) : null}
    </main>
  );
};
