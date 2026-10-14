import { describe, expect, it } from "vitest";

import {
  constrainLine,
  distanceToSegment,
  hitMark,
  intersectRects,
  moveMark,
  rectFromPoints,
  resizeRect,
  roundRect,
} from "./geometry";
import type { BoxMark, Mark, Rect } from "./model";

const noText = (_mark: Mark): Rect | undefined => undefined;

const box: BoxMark = {
  color: "#000000",
  fill: "outline",
  id: "b",
  kind: "rect",
  rect: { height: 100, width: 100, x: 0, y: 0 },
  size: 2,
};

describe(rectFromPoints, () => {
  it("normalisiert beliebige Ziehrichtungen", () => {
    expect(rectFromPoints({ x: 10, y: 10 }, { x: 0, y: 5 })).toStrictEqual({
      height: 5,
      width: 10,
      x: 0,
      y: 5,
    });
  });
});

describe(intersectRects, () => {
  it("begrenzt auf die Schnittmenge", () => {
    expect(
      intersectRects(
        { height: 10, width: 10, x: 5, y: 5 },
        { height: 10, width: 10, x: 0, y: 0 }
      )
    ).toStrictEqual({ height: 5, width: 5, x: 5, y: 5 });
  });

  it("liefert ein leeres Rechteck ohne Überlappung", () => {
    expect(
      intersectRects(
        { height: 1, width: 1, x: 5, y: 5 },
        { height: 1, width: 1, x: 0, y: 0 }
      ).width
    ).toBe(0);
  });
});

describe(roundRect, () => {
  it("rundet nach außen", () => {
    expect(roundRect({ height: 1, width: 1.2, x: 0.5, y: 0.5 })).toStrictEqual({
      height: 2,
      width: 2,
      x: 0,
      y: 0,
    });
  });
});

describe(distanceToSegment, () => {
  it("misst den Abstand zur Strecke", () => {
    expect(
      distanceToSegment({ x: 5, y: 3 }, { x: 0, y: 0 }, { x: 10, y: 0 })
    ).toBe(3);
    expect(
      distanceToSegment({ x: 13, y: 4 }, { x: 0, y: 0 }, { x: 10, y: 0 })
    ).toBe(5);
  });
});

describe(hitMark, () => {
  it("trifft den Rand eines Rahmens, aber nicht die Mitte", () => {
    expect(hitMark(box, { x: 1, y: 50 }, 2, noText, 1)).toBeTruthy();
    expect(hitMark(box, { x: 50, y: 50 }, 2, noText, 1)).toBeFalsy();
  });

  it("trifft gefüllte Formen überall", () => {
    expect(
      hitMark({ ...box, fill: "filled" }, { x: 50, y: 50 }, 2, noText, 1)
    ).toBeTruthy();
  });
});

describe(resizeRect, () => {
  it("zieht die Ecke unten rechts", () => {
    expect(
      resizeRect({ height: 10, width: 10, x: 0, y: 0 }, "se", { x: 20, y: 30 })
    ).toStrictEqual({
      height: 30,
      width: 20,
      x: 0,
      y: 0,
    });
  });

  it("verschiebt nur eine Kante", () => {
    expect(
      resizeRect({ height: 10, width: 10, x: 0, y: 0 }, "w", { x: 4, y: 99 })
    ).toStrictEqual({
      height: 10,
      width: 6,
      x: 4,
      y: 0,
    });
  });
});

describe(moveMark, () => {
  it("verschiebt Rechtecke", () => {
    const moved = moveMark(box, 5, -5);
    expect(moved.kind === "rect" ? moved.rect : null).toStrictEqual({
      height: 100,
      width: 100,
      x: 5,
      y: -5,
    });
  });
});

describe(constrainLine, () => {
  it("rastet auf 45°-Schritte ein", () => {
    const end = constrainLine({ x: 0, y: 0 }, { x: 10, y: 1 });
    expect(end.y).toBeCloseTo(0, 5);
    expect(end.x).toBeCloseTo(Math.hypot(10, 1), 5);
  });
});
