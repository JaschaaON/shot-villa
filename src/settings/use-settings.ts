import { useEffect, useState } from "react";

import { getSettings } from "../api";
import type { Settings } from "../api";

/** Einstellungen, aufgefrischt bei jedem Fokus des Fensters. */
export const useSettings = (): Settings | null => {
  const [settings, setSettings] = useState<Settings | null>(null);

  useEffect(() => {
    let active = true;
    const load = async () => {
      try {
        const loaded = await getSettings();
        if (active) {
          setSettings(loaded);
        }
      } catch (error) {
        console.error(error);
      }
    };
    const onFocus = () => {
      void load();
    };
    void load();
    window.addEventListener("focus", onFocus);
    return () => {
      active = false;
      window.removeEventListener("focus", onFocus);
    };
  }, []);

  return settings;
};
