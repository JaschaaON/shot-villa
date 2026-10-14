import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";

import { closeWindow, exportImage, pinShot } from "../api";
import { useShot } from "../editor/use-shot";
import type { LoadedShot } from "../editor/use-shot";
import { fire } from "../ui/fire";

const ZOOMS = [0.5, 1, 2] as const;

interface MenuProps {
  shot: LoadedShot;
  at: { x: number; y: number };
  zoom: number;
  translucent: boolean;
  onZoom: (zoom: number) => void;
  onTranslucent: () => void;
  onClose: () => void;
}

const PinMenu = ({
  shot,
  at,
  zoom,
  translucent,
  onZoom,
  onTranslucent,
  onClose,
}: MenuProps) => {
  const copy = async () => {
    onClose();
    const response = await fetch(shot.image.src);
    await exportImage(
      new Uint8Array(await response.arrayBuffer()),
      shot.info.scale,
      { kind: "copy" }
    );
  };

  return (
    <div className="context-menu" role="menu" style={{ left: at.x, top: at.y }}>
      {ZOOMS.map((value) => (
        <button
          className={value === zoom ? "active" : undefined}
          key={value}
          onClick={() => {
            onZoom(value);
          }}
          role="menuitem"
          type="button"
        >
          {value * 100} %
        </button>
      ))}
      <hr />
      <button onClick={onTranslucent} role="menuitem" type="button">
        {translucent ? "Deckend" : "Halbtransparent"}
      </button>
      <button onClick={fire(copy)} role="menuitem" type="button">
        Kopieren
      </button>
      <button onClick={fire(closeWindow)} role="menuitem" type="button">
        Schließen
      </button>
    </div>
  );
};

/** Angepinntes Bild: rahmenlos, verschiebbar, immer im Vordergrund. */
export const Pin = () => {
  const shot = useShot(pinShot);
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);
  const [zoom, setZoom] = useState(1);
  const [translucent, setTranslucent] = useState(false);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        fire(closeWindow)();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  if (!shot) {
    return null;
  }
  const { width, height, scale } = shot.info;

  const resize = async (next: number) => {
    setZoom(next);
    setMenu(null);
    await getCurrentWindow().setSize(
      new LogicalSize((width / scale) * next, (height / scale) * next)
    );
  };

  return (
    <div className="pin" style={{ opacity: translucent ? 0.5 : 1 }}>
      <button
        className="pin-surface"
        onContextMenu={(event) => {
          event.preventDefault();
          setMenu({ x: event.clientX, y: event.clientY });
        }}
        onDoubleClick={fire(closeWindow)}
        onMouseDown={(event) => {
          if (event.button === 0) {
            setMenu(null);
            fire(async () => {
              await getCurrentWindow().startDragging();
            })();
          }
        }}
        title="Ziehen verschiebt, Rechtsklick zeigt Optionen, Doppelklick schließt"
        type="button"
      >
        <img alt="Angepinntes Bild" draggable={false} src={shot.image.src} />
      </button>
      {menu ? (
        <PinMenu
          at={menu}
          onClose={() => {
            setMenu(null);
          }}
          onTranslucent={() => {
            setTranslucent(!translucent);
            setMenu(null);
          }}
          onZoom={(value) => {
            fire(async () => {
              await resize(value);
            })();
          }}
          shot={shot}
          translucent={translucent}
          zoom={zoom}
        />
      ) : null}
    </div>
  );
};
