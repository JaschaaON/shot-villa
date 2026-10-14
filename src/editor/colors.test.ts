import { describe, expect, it } from "vitest";

import { contrastText, formatColor, nextColorFormat, toOklch } from "./colors";

const amber = { b: 0, g: 194, r: 255 };

describe(formatColor, () => {
  it("formatiert HEX, RGB und CSS", () => {
    expect(formatColor(amber, "hex")).toBe("#FFC200");
    expect(formatColor(amber, "rgb")).toBe("255, 194, 0");
    expect(formatColor(amber, "cssRgba")).toBe("rgba(255, 194, 0, 1)");
  });

  it("formatiert OKLCH", () => {
    expect(formatColor({ b: 255, g: 255, r: 255 }, "oklch")).toBe(
      "oklch(100.0% 0.000 0.0)"
    );
    expect(formatColor({ b: 0, g: 0, r: 255 }, "oklch")).toBe(
      "oklch(62.8% 0.258 29.2)"
    );
  });
});

describe(toOklch, () => {
  it("liefert für Schwarz Helligkeit 0", () => {
    expect(toOklch({ b: 0, g: 0, r: 0 }).l).toBeCloseTo(0, 5);
  });
});

describe(nextColorFormat, () => {
  it("rotiert durch alle Formate", () => {
    expect(nextColorFormat("hex")).toBe("rgb");
    expect(nextColorFormat("oklch")).toBe("hex");
  });
});

describe(contrastText, () => {
  it("wählt dunkle Schrift auf hellem Grund", () => {
    expect(contrastText("#ffffff")).toBe("#111111");
    expect(contrastText("#111111")).toBe("#ffffff");
  });
});
