import { describe, expect, it } from "vitest";

import { recognitionMessage } from "./recognition";

describe(recognitionMessage, () => {
  it("zeigt den Anfang des erkannten Textes in einer Zeile", () => {
    expect(recognitionMessage({ found: "text", text: "Hallo\nWelt" })).toBe(
      "Text kopiert: Hallo Welt"
    );
  });

  it("kürzt lange Texte", () => {
    const message = recognitionMessage({ found: "code", text: "x".repeat(80) });
    expect(message).toBe(`QR-Code kopiert: ${"x".repeat(60)}…`);
  });

  it("meldet, wenn nichts gefunden wurde", () => {
    expect(recognitionMessage({ found: "nothing", text: "" })).toBe(
      "Kein Text erkannt"
    );
  });
});
