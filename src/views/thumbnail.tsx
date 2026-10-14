import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useEffectEvent, useState } from "react";

import { closeWindow, editCurrent, exportShot } from "../api";
import type { ExportTarget } from "../api";
import type { LoadedShot } from "../editor/use-shot";
import { useShot } from "../editor/use-shot";
import { useSettings } from "../settings/use-settings";
import { fire } from "../ui/fire";
import { CloseIcon, CopyIcon, EditIcon, SaveIcon } from "../ui/icons";

const Preview = ({ shot }: { shot: LoadedShot }) => {
  const [status, setStatus] = useState<string | null>(null);

  const output = async (target: ExportTarget, message: string) => {
    try {
      await exportShot(shot.info.id, target);
      setStatus(message);
    } catch (error) {
      setStatus(String(error));
    }
  };

  return (
    <div className="thumbnail">
      <button
        className="thumbnail-image"
        onClick={fire(editCurrent)}
        title="Im Editor öffnen"
        type="button"
      >
        <img alt="Aufnahme" draggable={false} src={shot.image.src} />
      </button>
      <div className="thumbnail-bar" data-tauri-drag-region>
        <button
          onClick={fire(async () => {
            await exportShot(shot.info.id, { kind: "copy" });
            await closeWindow();
          })}
          title="Kopieren"
          type="button"
        >
          <CopyIcon />
        </button>
        <button
          onClick={fire(async () => {
            await output({ kind: "save" }, "Gespeichert");
          })}
          title="Speichern"
          type="button"
        >
          <SaveIcon />
        </button>
        <button onClick={fire(editCurrent)} title="Bearbeiten" type="button">
          <EditIcon />
        </button>
        <span className="thumbnail-status" data-tauri-drag-region>
          {status}
        </span>
        <button onClick={fire(closeWindow)} title="Schließen" type="button">
          <CloseIcon />
        </button>
      </div>
    </div>
  );
};

/** Kleine Vorschau nach einer Aufnahme. */
export const Thumbnail = () => {
  const shot = useShot();
  const settings = useSettings();

  const onFocusChange = useEffectEvent((focused: boolean) => {
    if (!focused && settings?.thumbnailClosing === "automatic") {
      fire(closeWindow)();
    }
  });

  useEffect(() => {
    const unlisten = getCurrentWindow().onFocusChanged(({ payload }) => {
      onFocusChange(payload);
    });
    return () => {
      void (async () => {
        (await unlisten)();
      })();
    };
  }, []);

  return shot ? <Preview key={shot.info.id} shot={shot} /> : null;
};
