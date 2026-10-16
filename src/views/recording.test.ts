import { describe, expect, it } from "vitest";

import { formatElapsed } from "./recording";

describe(formatElapsed, () => {
  it("zeigt Minuten und zweistellige Sekunden", () => {
    expect(formatElapsed(5)).toBe("0:05");
    expect(formatElapsed(125)).toBe("2:05");
  });
});
