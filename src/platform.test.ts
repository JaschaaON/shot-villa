import { describe, expect, it } from "vitest";

import { keys } from "./platform";

describe("Tastenkürzel je Plattform", () => {
  it("lässt Mac-Kürzel unverändert", () => {
    expect(keys("⇧⌘C", true)).toBe("⇧⌘C");
  });

  it("schreibt Windows-Kürzel aus", () => {
    expect(keys("⇧⌘C", false)).toBe("Strg+Umschalt+C");
    expect(keys("⌃⌥⌘O", false)).toBe("Strg+Alt+O");
    expect(keys("Esc", false)).toBe("Esc");
  });
});
