import { describe, expect, it } from "vitest";

import { formatShortcut, shortcutFromEvent } from "./shortcut";

const press = (
  code: string,
  modifiers: Partial<
    Record<"metaKey" | "shiftKey" | "altKey" | "ctrlKey", boolean>
  > = {}
) => ({
  altKey: false,
  code,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
  ...modifiers,
});

describe(shortcutFromEvent, () => {
  it("baut Kürzel mit Modifiern", () => {
    expect(
      shortcutFromEvent(
        press("Digit1", { metaKey: true, shiftKey: true }),
        true
      )
    ).toBe("Shift+CmdOrCtrl+Digit1");
    expect(
      shortcutFromEvent(
        press("KeyO", { altKey: true, ctrlKey: true, metaKey: true }),
        true
      )
    ).toBe("Control+Alt+CmdOrCtrl+KeyO");
  });

  it("nutzt unter Windows Strg und die Windows-Taste", () => {
    expect(
      shortcutFromEvent(
        press("Digit1", { ctrlKey: true, shiftKey: true }),
        false
      )
    ).toBe("Control+Shift+Digit1");
    expect(shortcutFromEvent(press("KeyS", { metaKey: true }), false)).toBe(
      "Super+KeyS"
    );
  });

  it("ignoriert reine Modifier und Tasten ohne Modifier", () => {
    expect(
      shortcutFromEvent(press("ShiftLeft", { shiftKey: true }))
    ).toBeNull();
    expect(shortcutFromEvent(press("KeyA"))).toBeNull();
  });

  it("erlaubt F-Tasten ohne Modifier", () => {
    expect(shortcutFromEvent(press("F5"))).toBe("F5");
  });
});

describe(formatShortcut, () => {
  it("zeigt Mac-Symbole in Systemreihenfolge", () => {
    expect(formatShortcut("CmdOrCtrl+Shift+1", true)).toBe("⇧⌘1");
    expect(formatShortcut("Shift+CmdOrCtrl+Digit2", true)).toBe("⇧⌘2");
    expect(formatShortcut("Control+Alt+CmdOrCtrl+O", true)).toBe("⌃⌥⌘O");
    expect(formatShortcut("Control+KeyA", true)).toBe("⌃A");
  });
});
