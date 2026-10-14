import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import {
  useEffect,
  useEffectEvent,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";

import {
  closeWindow,
  copyText,
  exportImage,
  openImages,
  pinImage,
  recognizeText,
  runAction,
} from "../api";
import type { ActionId, ExportTarget, Settings } from "../api";
import { EditorCanvas } from "../editor/canvas";
import { formatColor, nextColorFormat } from "../editor/colors";
import type { ColorFormat, Rgb } from "../editor/colors";
import { cropPng, renderPng } from "../editor/export";
import { intersectRects, roundRect } from "../editor/geometry";
import { createDoc } from "../editor/model";
import type { Doc, Rect } from "../editor/model";
import { Options } from "../editor/options";
import { PixelReader } from "../editor/pixels";
import { editorReducer, initialState } from "../editor/state";
import { TextEditor } from "../editor/text-editor";
import { TOOLS, toolForKey } from "../editor/tools";
import { useElementSize } from "../editor/use-element-size";
import { useShot } from "../editor/use-shot";
import type { LoadedShot } from "../editor/use-shot";
import { fitZoom } from "../editor/zoom";
import { isMac, keys } from "../platform";
import { useSettings } from "../settings/use-settings";
import { fire } from "../ui/fire";
import { CopyIcon, CropIcon, OcrIcon, PinIcon, SaveIcon } from "../ui/icons";

const STAGE_PADDING = 32;
const ZOOM_STEP = 1.25;
const MIN_ZOOM = 0.05;
const MAX_ZOOM = 16;
const TOAST_MS = 1800;
const NUDGE_FAST = 10;

type ZoomMode = "fit" | number;

/** Strg-Kürzel unter Windows, Schlüssel: „⇧“ für Umschalt plus Taste. */
const CTRL_COMMANDS = new Map([
  ["z", "undo"],
  ["⇧z", "redo"],
  ["y", "redo"],
  ["s", "save"],
  ["⇧s", "save-as"],
  ["⇧c", "copy-image"],
  ["p", "pin"],
  ["⇧t", "ocr"],
  ["d", "duplicate"],
  ["=", "zoom-in"],
  ["+", "zoom-in"],
  ["-", "zoom-out"],
  ["0", "zoom-actual"],
  ["1", "zoom-fit"],
  ["o", "open-file"],
  ["⇧v", "load-clipboard"],
  [",", "settings"],
  ["w", "close"],
]);

/** Befehle, die als App-Aktion im Rust-Teil laufen. */
const APP_ACTIONS = new Map<string, ActionId>([
  ["open-file", "open-file"],
  ["load-clipboard", "load-clipboard"],
  ["settings", "settings"],
]);

const ctrlKey = (event: KeyboardEvent) =>
  `${event.shiftKey ? "⇧" : ""}${event.key.toLowerCase()}`;

const NUDGES = new Map<string, [number, number]>([
  ["ArrowDown", [0, 1]],
  ["ArrowLeft", [-1, 0]],
  ["ArrowRight", [1, 0]],
  ["ArrowUp", [0, -1]],
]);

const sizeText = (width: number, height: number, unit: string) =>
  `${Math.round(width)}×${Math.round(height)} ${unit}`;

/** Breite × Höhe der Auswahl oder des Bildes in Pixeln oder Punkten. */
const dimensionLabel = (doc: Doc, selection: Rect | null, pixels: boolean) => {
  const area = selection ?? doc.crop;
  const label = selection === null ? "Bildgröße" : "Auswahl";
  const value = pixels
    ? sizeText(area.width, area.height, "px")
    : sizeText(area.width / doc.scale, area.height / doc.scale, "pt");
  return { label, value };
};

const effectiveZoom = (mode: ZoomMode, fit: number, prefersActual: boolean) => {
  if (mode !== "fit") {
    return mode;
  }
  return prefersActual && fit >= 1 ? 1 : fit;
};

interface WorkspaceProps {
  shot: LoadedShot;
  settings: Settings | null;
}

/** Bearbeitung einer Aufnahme; wird für jede neue Aufnahme neu aufgebaut. */
const Workspace = ({ shot, settings }: WorkspaceProps) => {
  const [state, dispatch] = useReducer(editorReducer, shot.info, (info) =>
    initialState(createDoc(info.width, info.height, info.scale))
  );
  const [zoomMode, setZoomMode] = useState<ZoomMode>("fit");
  const [cursorColor, setCursorColor] = useState<Rgb | null>(null);
  const [formatOverride, setFormatOverride] = useState<ColorFormat | null>(
    null
  );
  const [showPixels, setShowPixels] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const textBounds = useRef(new Map<string, Rect>());
  const stageRef = useRef<HTMLElement>(null);
  const stage = useElementSize(stageRef);
  const pixels = useMemo(() => new PixelReader(shot.image), [shot]);

  const { doc } = state;
  const colorFormat = formatOverride ?? settings?.colorFormat ?? "hex";
  const fit = fitZoom(
    { height: doc.crop.height / doc.scale, width: doc.crop.width / doc.scale },
    {
      height: stage.height - STAGE_PADDING * 2,
      width: stage.width - STAGE_PADDING * 2,
    }
  );
  const zoom = effectiveZoom(zoomMode, fit, settings?.preferActualSize ?? true);
  const viewScale = zoom / doc.scale;

  useEffect(() => {
    const timer =
      toast === null
        ? undefined
        : setTimeout(() => {
            setToast(null);
          }, TOAST_MS);
    return () => {
      clearTimeout(timer);
    };
  }, [toast]);

  const changeZoom = (factor: number) => {
    setZoomMode(Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom * factor)));
  };

  /** Auswahl oder ganzer Zuschnitt in ganzen Pixeln. */
  const region = (): Rect =>
    roundRect(
      state.selection ? intersectRects(state.selection, doc.crop) : doc.crop
    );

  const output = async (target: ExportTarget, message: string) => {
    try {
      const path = await exportImage(
        await renderPng(shot.image, doc),
        doc.scale,
        target
      );
      setToast(
        path === null ? message : `${message}: ${path.split("/").pop() ?? path}`
      );
    } catch (error) {
      setToast(String(error));
    }
  };

  /** Kopiert das Bild und schließt den Editor – weiter geht's in der Ziel-App. */
  const copyImage = async () => {
    await exportImage(await renderPng(shot.image, doc), doc.scale, {
      kind: "copy",
    });
    await closeWindow();
  };

  const saveImage = async () => {
    await output({ kind: "save" }, "Gespeichert");
  };

  const saveImageAs = async () => {
    const path = await saveDialog({
      defaultPath: "Screeni.png",
      filters: [{ extensions: ["png", "jpg"], name: "Bild" }],
    });
    if (path !== null) {
      await output({ kind: "saveAs", path }, "Gespeichert");
    }
  };

  const pin = async () => {
    await pinImage(await renderPng(shot.image, doc), doc.scale);
  };

  const recognize = async () => {
    const area = region();
    if (area.width >= 1 && area.height >= 1) {
      await recognizeText(await cropPng(shot.image, area));
    }
  };

  const copyColor = async () => {
    if (cursorColor) {
      const text = formatColor(cursorColor, colorFormat);
      await copyText(text);
      setToast(`${text} kopiert`);
    }
  };

  /** Esc ohne Auswahl: je nach Einstellung kopieren/speichern, dann schließen. */
  const closeEditor = async () => {
    if (settings?.escCopies === true || settings?.escSaves === true) {
      const png = await renderPng(shot.image, doc);
      if (settings.escCopies) {
        await exportImage(png, doc.scale, { kind: "copy" });
      }
      if (settings.escSaves) {
        await exportImage(png, doc.scale, { kind: "save" });
      }
    }
    await closeWindow();
  };

  const commands = new Map<string, () => void>([
    ["close", fire(closeWindow)],
    ["copy-image", fire(copyImage)],
    [
      "crop",
      () => {
        dispatch({ type: "crop" });
      },
    ],
    [
      "duplicate",
      () => {
        dispatch({ offset: 12 * doc.scale, type: "duplicateSelected" });
      },
    ],
    ["ocr", fire(recognize)],
    ["pin", fire(pin)],
    [
      "redo",
      () => {
        dispatch({ type: "redo" });
      },
    ],
    [
      "reset-crop",
      () => {
        dispatch({ type: "resetCrop" });
      },
    ],
    ["save", fire(saveImage)],
    ["save-as", fire(saveImageAs)],
    [
      "undo",
      () => {
        dispatch({ type: "undo" });
      },
    ],
    [
      "zoom-actual",
      () => {
        setZoomMode(1);
      },
    ],
    [
      "zoom-fit",
      () => {
        setZoomMode("fit");
      },
    ],
    [
      "zoom-in",
      () => {
        changeZoom(ZOOM_STEP);
      },
    ],
    [
      "zoom-out",
      () => {
        changeZoom(1 / ZOOM_STEP);
      },
    ],
  ]);

  const runCommand = (command: string) => {
    const action = APP_ACTIONS.get(command);
    if (action === undefined) {
      commands.get(command)?.();
    } else {
      fire(async () => {
        await runAction(action);
      })();
    }
  };

  const onMenu = useEffectEvent((command: string) => {
    runCommand(command);
  });

  const onKey = useEffectEvent((event: KeyboardEvent) => {
    if (
      state.editingId !== null ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return;
    }
    // Unter Windows gibt es kein App-Menü; Strg-Kürzel laufen hier ein.
    const command =
      isMac || !event.ctrlKey ? undefined : CTRL_COMMANDS.get(ctrlKey(event));
    if (command !== undefined) {
      event.preventDefault();
      runCommand(command);
      return;
    }
    const nudge = NUDGES.get(event.key);
    if (nudge && state.selectedId !== null) {
      event.preventDefault();
      const step = (event.shiftKey ? NUDGE_FAST : 1) * doc.scale;
      dispatch({ dx: nudge[0] * step, dy: nudge[1] * step, type: "nudge" });
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      if (state.selectedId === null && state.selection === null) {
        fire(closeEditor)();
      } else {
        dispatch({ id: null, type: "select" });
      }
      return;
    }
    if (event.key === "Tab") {
      event.preventDefault();
      fire(copyColor)();
      return;
    }
    if (event.key === "Enter") {
      dispatch({ type: "crop" });
      return;
    }
    if (event.key === "Backspace" || event.key === "Delete") {
      dispatch({ type: "deleteSelected" });
      return;
    }
    const plain = !event.metaKey && !event.ctrlKey && !event.altKey;
    const tool = plain ? toolForKey(event.key) : undefined;
    if (tool) {
      dispatch({ tool, type: "setTool" });
    }
  });

  const onCopy = useEffectEvent((event: ClipboardEvent) => {
    if (
      state.editingId === null &&
      !(document.activeElement instanceof HTMLTextAreaElement)
    ) {
      event.preventDefault();
      fire(copyImage)();
    }
  });

  useEffect(() => {
    const unlisten = listen<string>("menu", (event) => {
      onMenu(event.payload);
    });
    window.addEventListener("keydown", onKey);
    document.addEventListener("copy", onCopy);
    return () => {
      void (async () => {
        (await unlisten)();
      })();
      window.removeEventListener("keydown", onKey);
      document.removeEventListener("copy", onCopy);
    };
  }, []);

  const editing = doc.marks.find((mark) => mark.id === state.editingId);
  const dimension = dimensionLabel(doc, state.selection, showPixels);
  const colorText = cursorColor ? formatColor(cursorColor, colorFormat) : "—";

  return (
    <div className="editor">
      <header className="toolbar" data-tauri-drag-region>
        <div className="group">
          <button
            onClick={fire(copyImage)}
            title={`Kopieren (${keys("⇧⌘C")}, Esc)`}
            type="button"
          >
            <CopyIcon />
          </button>
          <button
            onClick={fire(saveImage)}
            title={`Sichern (${keys("⌘S")})`}
            type="button"
          >
            <SaveIcon />
          </button>
          <button
            onClick={fire(pin)}
            title={`Anpinnen (${keys("⌘P")})`}
            type="button"
          >
            <PinIcon />
          </button>
          <button
            onClick={fire(recognize)}
            title={`Text erkennen (${keys("⇧⌘T")})`}
            type="button"
          >
            <OcrIcon />
          </button>
        </div>
        <div className="group tools">
          {TOOLS.map((info) => (
            <button
              aria-pressed={state.tool === info.tool}
              className={state.tool === info.tool ? "active" : undefined}
              key={info.tool}
              onClick={() => {
                dispatch({ tool: info.tool, type: "setTool" });
              }}
              title={`${info.label} (${info.key.toUpperCase()})`}
              type="button"
            >
              {info.icon}
            </button>
          ))}
          {state.selection ? (
            <button
              onClick={() => {
                dispatch({ type: "crop" });
              }}
              title="Zuschneiden (↩)"
              type="button"
            >
              <CropIcon />
            </button>
          ) : null}
        </div>
        <span className="toolbar-spacer" data-tauri-drag-region />
        <div className="info">
          <button
            aria-label={`Farbe ${colorText}`}
            className="info-item"
            onClick={() => {
              setFormatOverride(nextColorFormat(colorFormat));
            }}
            title="Klicken wechselt das Format, Tab kopiert"
            type="button"
          >
            <span
              className="swatch"
              style={{
                background: cursorColor
                  ? formatColor(cursorColor, "hex")
                  : "transparent",
              }}
            />
            <span>
              <strong>{colorText}</strong>
              <small>Tab kopiert</small>
            </span>
          </button>
          <button
            aria-label={`${dimension.label} ${dimension.value}`}
            className="info-item"
            onClick={() => {
              setShowPixels(!showPixels);
            }}
            title="Klicken wechselt zwischen Punkten und Pixeln"
            type="button"
          >
            <span>
              <strong>{dimension.value}</strong>
              <small>{dimension.label}</small>
            </span>
          </button>
          <button
            aria-label={`Zoom ${Math.round(zoom * 100)} %`}
            className="info-item"
            onClick={() => {
              setZoomMode(zoomMode === "fit" ? 1 : "fit");
            }}
            title="Klicken wechselt zwischen 100 % und Einpassen"
            type="button"
          >
            <span>
              <strong>{Math.round(zoom * 100)} %</strong>
              <small>Zoom</small>
            </span>
          </button>
        </div>
      </header>
      <Options dispatch={dispatch} state={state} />
      <main
        className="stage"
        onWheel={(event) => {
          if (event.ctrlKey || event.metaKey) {
            event.preventDefault();
            changeZoom(Math.exp(-event.deltaY * 0.01));
          }
        }}
        ref={stageRef}
      >
        <div className="canvas-wrap">
          <EditorCanvas
            dispatch={dispatch}
            image={shot.image}
            onCursorColor={setCursorColor}
            pixels={pixels}
            state={state}
            textBounds={textBounds}
            viewScale={viewScale}
          />
          {editing?.kind === "text" ? (
            <TextEditor
              crop={doc.crop}
              key={editing.id}
              mark={editing}
              onFinish={(text) => {
                dispatch({ text, type: "finishEditing" });
              }}
              scale={doc.scale}
              viewScale={viewScale}
            />
          ) : null}
        </div>
        {toast === null ? null : <div className="toast">{toast}</div>}
      </main>
    </div>
  );
};

const Empty = () => (
  <div className="editor">
    <header className="toolbar" data-tauri-drag-region />
    <main className="stage">
      <div className="empty">
        <p>Noch keine Aufnahme.</p>
        <div className="empty-actions">
          <button
            onClick={fire(async () => {
              await runAction("capture-area");
            })}
            type="button"
          >
            Bereich aufnehmen
          </button>
          <button
            onClick={fire(async () => {
              await runAction("capture-fullscreen");
            })}
            type="button"
          >
            Vollbild
          </button>
          <button
            onClick={fire(async () => {
              await runAction("open-file");
            })}
            type="button"
          >
            Bild öffnen …
          </button>
        </div>
      </div>
    </main>
  </div>
);

export const Editor = () => {
  const shot = useShot();
  const settings = useSettings();

  // Bilder per Drag & Drop öffnen.
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop" && event.payload.paths.length > 0) {
        const { paths } = event.payload;
        fire(async () => {
          await openImages(paths);
        })();
      }
    });
    return () => {
      void (async () => {
        (await unlisten)();
      })();
    };
  }, []);

  return shot ? (
    <Workspace key={shot.info.id} settings={settings} shot={shot} />
  ) : (
    <Empty />
  );
};
