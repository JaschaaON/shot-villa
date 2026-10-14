import { isMac } from "../platform";

/** Kürzel im Format der Tauri-Accelerators, z. B. `CmdOrCtrl+Shift+Digit1`. */

interface KeyInput {
  code: string;
  metaKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  ctrlKey: boolean;
}

const MODIFIER_CODES = new Set([
  "MetaLeft",
  "MetaRight",
  "ShiftLeft",
  "ShiftRight",
  "AltLeft",
  "AltRight",
  "ControlLeft",
  "ControlRight",
  "CapsLock",
  "Fn",
]);

const isFunctionKey = (code: string) => /^F\d{1,2}$/u.test(code);

/**
 * Kürzel aus einem Tastendruck. Liefert `null` für reine Modifier und für
 * Tasten ohne Modifier (außer F-Tasten), weil die sonst überall auslösen.
 */
export const shortcutFromEvent = (
  event: KeyInput,
  mac: boolean = isMac
): string | null => {
  if (MODIFIER_CODES.has(event.code) || event.code === "") {
    return null;
  }
  // Unter Windows ist die Meta-Taste die Windows-Taste, Strg heißt Control.
  const meta = mac ? "CmdOrCtrl" : "Super";
  const modifiers = [
    event.ctrlKey ? "Control" : null,
    event.altKey ? "Alt" : null,
    event.shiftKey ? "Shift" : null,
    event.metaKey ? meta : null,
  ].filter((part) => part !== null);
  if (modifiers.length === 0 && !isFunctionKey(event.code)) {
    return null;
  }
  return [...modifiers, event.code].join("+");
};

const SYMBOLS = new Map(
  Object.entries({
    Alt: "⌥",
    Cmd: "⌘",
    CmdOrCtrl: "⌘",
    Command: "⌘",
    Control: "⌃",
    Ctrl: "⌃",
    Option: "⌥",
    Shift: "⇧",
    Super: "⌘",
  })
);

const MODIFIER_ORDER = ["⌃", "⌥", "⇧", "⌘"];

const KEY_LABELS = new Map(
  Object.entries({
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
    ArrowUp: "↑",
    Backquote: "`",
    Backslash: "\\",
    BracketLeft: "[",
    BracketRight: "]",
    Comma: ",",
    Enter: "↩",
    Equal: "=",
    Escape: "⎋",
    Minus: "-",
    Period: ".",
    Quote: "'",
    Semicolon: ";",
    Slash: "/",
    Space: "Leertaste",
    Tab: "⇥",
  })
);

const keyLabel = (key: string): string =>
  KEY_LABELS.get(key) ??
  key
    .replace(/^Key/u, "")
    .replace(/^Digit/u, "")
    .replace(/^Numpad/u, "Num ")
    .toUpperCase();

const WINDOWS_MODIFIERS = new Map(
  Object.entries({
    Alt: "Alt",
    Cmd: "Win",
    CmdOrCtrl: "Strg",
    Command: "Win",
    Control: "Strg",
    Ctrl: "Strg",
    Option: "Alt",
    Shift: "Umschalt",
    Super: "Win",
  })
);

const WINDOWS_ORDER = ["Strg", "Alt", "Umschalt", "Win"];

/** Lesbare Darstellung: `⇧⌘1` auf dem Mac, `Strg+Umschalt+1` unter Windows. */
export const formatShortcut = (
  shortcut: string,
  mac: boolean = isMac
): string => {
  const parts = shortcut.split("+");
  const key = keyLabel(parts.pop() ?? "");
  if (!mac) {
    const modifiers = [
      ...new Set(parts.map((part) => WINDOWS_MODIFIERS.get(part) ?? part)),
    ].toSorted((a, b) => WINDOWS_ORDER.indexOf(a) - WINDOWS_ORDER.indexOf(b));
    return [...modifiers, key].join("+");
  }
  const modifiers = parts
    .map((part) => SYMBOLS.get(part) ?? part)
    .toSorted((a, b) => MODIFIER_ORDER.indexOf(a) - MODIFIER_ORDER.indexOf(b));
  return `${modifiers.join("")}${key}`;
};
