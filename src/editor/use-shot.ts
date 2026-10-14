import { useEffect, useState } from "react";

import { currentShot, onShotReady, shotPng } from "../api";
import type { ShotInfo } from "../api";

export interface LoadedShot {
  info: ShotInfo;
  /** Dekodiertes Bild; `image.src` bleibt gültig, solange die Aufnahme angezeigt wird. */
  image: HTMLImageElement;
}

const loadImage = async (info: ShotInfo): Promise<LoadedShot> => {
  const image = new Image();
  image.src = URL.createObjectURL(await shotPng(info.id));
  await image.decode();
  return { image, info };
};

/**
 * Lädt das aktuelle Bild und folgt neuen Aufnahmen.
 * `initial` lädt stattdessen ein festes Bild (z. B. im Pin-Fenster).
 */
export const useShot = (
  initial?: () => Promise<ShotInfo | null>
): LoadedShot | null => {
  const [shot, setShot] = useState<LoadedShot | null>(null);

  useEffect(() => {
    let active = true;
    let shown: LoadedShot | null = null;
    let stopListening: (() => void) | null = null;

    const show = async (info: ShotInfo) => {
      const loaded = await loadImage(info);
      if (!active) {
        URL.revokeObjectURL(loaded.image.src);
        return;
      }
      if (shown) {
        URL.revokeObjectURL(shown.image.src);
      }
      shown = loaded;
      setShot(loaded);
    };

    const start = async () => {
      try {
        if (!initial) {
          const unlisten = await onShotReady((info) => {
            void show(info);
          });
          if (active) {
            stopListening = unlisten;
          } else {
            unlisten();
          }
        }
        const info = await (initial ?? currentShot)();
        if (info) {
          await show(info);
        }
      } catch (error) {
        console.error(error);
      }
    };
    void start();

    return () => {
      active = false;
      stopListening?.();
      if (shown) {
        URL.revokeObjectURL(shown.image.src);
      }
    };
  }, [initial]);

  return shot;
};
