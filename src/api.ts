import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

import type { ColorFormat } from "./editor/colors";

export interface ShotInfo {
  id: number;
  width: number;
  height: number;
  /** Pixel pro Punkt (2 auf Retina-Displays). */
  scale: number;
}

export type ScreenAccess = "granted" | "denied";

export interface Hotkeys {
  fullscreen: string | null;
  area: string | null;
  repeatArea: string | null;
  anyWindow: string | null;
  activeWindow: string | null;
  ocr: string | null;
  showApp: string | null;
}

export interface Settings {
  hotkeys: Hotkeys;
  saveFolder: string | null;
  saveFormat: "png" | "auto";
  downscaleOnSave: boolean;
  afterCapture: { show: boolean; copy: boolean; save: boolean };
  areaOpens: "editor" | "thumbnail";
  thumbnailClosing: "manual" | "automatic";
  captureCursor: "auto" | "include" | "exclude";
  ocrRemoveLineBreaks: boolean;
  escCopies: boolean;
  escSaves: boolean;
  alwaysOnTop: boolean;
  preferActualSize: boolean;
  colorFormat: ColorFormat;
  launchAtLogin: boolean;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface WindowInfo {
  id: number;
  frame: Rect;
  app: string;
  title: string;
}

export interface OverlayInfo {
  mode: "area" | "window" | "ocr";
  display: { displayId: number; shotId: number; frame: Rect; scale: number };
  windows: WindowInfo[];
}

export type Selection =
  | { kind: "area"; rect: Rect }
  | { kind: "window"; id: number }
  | { kind: "cancel" };

export type ExportTarget =
  | { kind: "copy" }
  | { kind: "save" }
  | { kind: "saveAs"; path: string };

export type ActionId =
  | "capture-fullscreen"
  | "capture-area"
  | "capture-repeat-area"
  | "capture-any-window"
  | "capture-active-window"
  | "recognize-text"
  | "show-app"
  | "load-clipboard"
  | "open-file"
  | "settings";

/** Parameter der Bild-Befehle, die Bytes als Body übertragen. */
interface ImageParams {
  /** Pixel pro Punkt des Bildes. */
  scale: number;
  target?: ExportTarget;
}

/** Bytes als Body, Parameter als URI-kodiertes JSON im Header. */
const invokeRaw = async <T>(
  command: string,
  bytes: Uint8Array,
  params: ImageParams
): Promise<T> =>
  await invoke<T>(command, bytes, {
    headers: { "x-params": encodeURIComponent(JSON.stringify(params)) },
  });

export const currentShot = async () =>
  await invoke<ShotInfo | null>("current_shot");

export const shotInfo = async (id: number) =>
  await invoke<ShotInfo | null>("shot_info", { id });

export const shotPng = async (id: number): Promise<Blob> => {
  const bytes = await invoke<ArrayBuffer>("shot_png", { id });
  return new Blob([bytes], { type: "image/png" });
};

export const onShotReady = async (
  handler: (shot: ShotInfo) => void
): Promise<UnlistenFn> =>
  await listen<ShotInfo>("shot-ready", (event) => {
    handler(event.payload);
  });

export const runAction = async (id: ActionId) => {
  await invoke("run_action", { id });
};

export const getSettings = async () => await invoke<Settings>("get_settings");

/** Liefert Meldungen zu Kürzeln, die nicht registriert werden konnten. */
export const setSettings = async (settings: Settings) =>
  await invoke<string[]>("set_settings", { settings });

export const suspendHotkeys = async () => {
  await invoke("suspend_hotkeys");
};

export const resumeHotkeys = async () =>
  await invoke<string[]>("resume_hotkeys");

export const chooseSaveFolder = async () =>
  await invoke<string | null>("choose_save_folder");

export const overlayInfo = async () =>
  await invoke<OverlayInfo | null>("overlay_info");

export const overlayReady = async () => {
  await invoke("overlay_ready");
};

export const overlayFinish = async (selection: Selection) => {
  await invoke("overlay_finish", { selection });
};

/** Gibt ein gerendertes PNG aus; liefert beim Speichern den Pfad. */
export const exportImage = async (
  png: Uint8Array,
  scale: number,
  target: ExportTarget
) => await invokeRaw<string | null>("export_image", png, { scale, target });

export const exportShot = async (id: number, target: ExportTarget) =>
  await invoke<string | null>("export_shot", { id, target });

export const recognizeText = async (png: Uint8Array) => {
  await invokeRaw("recognize_text", png, { scale: 1 });
};

export const pinImage = async (png: Uint8Array, scale: number) => {
  await invokeRaw("pin_image", png, { scale });
};

export const pinShot = async () => await invoke<ShotInfo | null>("pin_shot");

export const editCurrent = async () => {
  await invoke("edit_current");
};

export const openImages = async (paths: string[]) => {
  await invoke("open_images", { paths });
};

export const closeWindow = async () => {
  await invoke("close_window");
};

export const copyText = async (text: string) => {
  await invoke("copy_text", { text });
};

export const screenAccess = async () =>
  await invoke<ScreenAccess>("screen_access");

export const openScreenAccessSettings = async () => {
  await invoke("open_screen_access_settings");
};

export const restartApp = async () => {
  await invoke("restart_app");
};
