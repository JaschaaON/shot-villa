import { describe, expect, it } from "vitest";

import { fitZoom, sizeInPoints } from "./zoom";

describe(sizeInPoints, () => {
  it("rechnet Retina-Pixel in Punkte um", () => {
    expect(sizeInPoints({ height: 1964, width: 3024 }, 2)).toStrictEqual({
      height: 982,
      width: 1512,
    });
  });
});

describe(fitZoom, () => {
  it("verkleinert große Bilder auf die Fenstergröße", () => {
    expect(
      fitZoom({ height: 1000, width: 2000 }, { height: 500, width: 1000 })
    ).toBe(0.5);
  });

  it("nimmt die engere Achse", () => {
    expect(
      fitZoom({ height: 1000, width: 1000 }, { height: 250, width: 800 })
    ).toBe(0.25);
  });

  it("vergrößert kleine Bilder nicht", () => {
    expect(
      fitZoom({ height: 10, width: 10 }, { height: 500, width: 500 })
    ).toBe(1);
  });

  it("behandelt leere Bilder robust", () => {
    expect(fitZoom({ height: 0, width: 0 }, { height: 500, width: 500 })).toBe(
      1
    );
  });
});
