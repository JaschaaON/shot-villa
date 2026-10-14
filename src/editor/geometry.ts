import type { Mark, Point, Rect } from "./model";
import { unreachable } from "./unreachable";

export const rectFromPoints = (a: Point, b: Point): Rect => ({
  height: Math.abs(b.y - a.y),
  width: Math.abs(b.x - a.x),
  x: Math.min(a.x, b.x),
  y: Math.min(a.y, b.y),
});

export const containsPoint = (
  rect: Rect,
  point: Point,
  tolerance = 0
): boolean =>
  point.x >= rect.x - tolerance &&
  point.x <= rect.x + rect.width + tolerance &&
  point.y >= rect.y - tolerance &&
  point.y <= rect.y + rect.height + tolerance;

export const intersectRects = (a: Rect, b: Rect): Rect => {
  const x = Math.max(a.x, b.x);
  const y = Math.max(a.y, b.y);
  const right = Math.min(a.x + a.width, b.x + b.width);
  const bottom = Math.min(a.y + a.height, b.y + b.height);
  return {
    height: Math.max(0, bottom - y),
    width: Math.max(0, right - x),
    x,
    y,
  };
};

/** Rundet auf ganze Pixel nach außen. */
export const roundRect = (rect: Rect): Rect => {
  const x = Math.floor(rect.x);
  const y = Math.floor(rect.y);
  return {
    height: Math.ceil(rect.y + rect.height) - y,
    width: Math.ceil(rect.x + rect.width) - x,
    x,
    y,
  };
};

const insetRect = (rect: Rect, inset: number): Rect => ({
  height: rect.height - inset * 2,
  width: rect.width - inset * 2,
  x: rect.x + inset,
  y: rect.y + inset,
});

export const distanceToSegment = (point: Point, a: Point, b: Point): number => {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const lengthSquared = dx * dx + dy * dy;
  const projection =
    lengthSquared === 0
      ? 0
      : ((point.x - a.x) * dx + (point.y - a.y) * dy) / lengthSquared;
  const t = Math.max(0, Math.min(1, projection));
  return Math.hypot(point.x - (a.x + t * dx), point.y - (a.y + t * dy));
};

/** Radius eines Zählers in Pixeln. */
export const counterRadius = (size: number, scale: number): number =>
  (size * 2.5 + 10) * scale;

const pointsBounds = (points: Point[]): Rect => {
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  return { height: Math.max(...ys) - y, width: Math.max(...xs) - x, x, y };
};

/** Gemessene Textgröße einer Markierung, falls schon gezeichnet. */
export type TextBounds = (mark: Mark) => Rect | undefined;

/** Begrenzungsrechteck einer Markierung. */
export const markBounds = (
  mark: Mark,
  textBounds: TextBounds,
  scale: number
): Rect => {
  switch (mark.kind) {
    case "arrow":
    case "line": {
      return rectFromPoints(mark.from, mark.to);
    }
    case "freehand": {
      return pointsBounds(mark.points);
    }
    case "counter": {
      const radius = counterRadius(mark.size, scale);
      return {
        height: radius * 2,
        width: radius * 2,
        x: mark.center.x - radius,
        y: mark.center.y - radius,
      };
    }
    case "text": {
      return textBounds(mark) ?? { height: 0, width: 0, ...mark.origin };
    }
    case "rect":
    case "oval":
    case "highlight":
    case "conceal":
    case "spotlight": {
      return mark.rect;
    }
    default: {
      return unreachable(mark);
    }
  }
};

const hitsPolyline = (points: Point[], point: Point, reach: number): boolean =>
  points.some((current, index) => {
    const previous = points[index - 1];
    return (
      previous !== undefined &&
      distanceToSegment(point, previous, current) <= reach
    );
  });

/** Liegt der Punkt auf der Markierung? `tolerance` in Pixeln. */
export const hitMark = (
  mark: Mark,
  point: Point,
  tolerance: number,
  textBounds: TextBounds,
  scale: number
): boolean => {
  switch (mark.kind) {
    case "arrow":
    case "line": {
      return (
        distanceToSegment(point, mark.from, mark.to) <=
        tolerance + mark.size * scale
      );
    }
    case "freehand": {
      return hitsPolyline(mark.points, point, tolerance + mark.size * scale);
    }
    case "rect":
    case "oval": {
      if (mark.fill === "filled") {
        return containsPoint(mark.rect, point, tolerance);
      }
      const reach = tolerance + mark.size * scale;
      return (
        containsPoint(mark.rect, point, reach) &&
        !containsPoint(insetRect(mark.rect, reach), point)
      );
    }
    case "counter":
    case "text":
    case "highlight":
    case "conceal":
    case "spotlight": {
      return containsPoint(
        markBounds(mark, textBounds, scale),
        point,
        tolerance
      );
    }
    default: {
      return unreachable(mark);
    }
  }
};

export type Handle =
  | "start"
  | "end"
  | "nw"
  | "n"
  | "ne"
  | "e"
  | "se"
  | "s"
  | "sw"
  | "w";

const rectHandles = ({ x, y, width, height }: Rect): [Handle, Point][] => [
  ["nw", { x, y }],
  ["n", { x: x + width / 2, y }],
  ["ne", { x: x + width, y }],
  ["e", { x: x + width, y: y + height / 2 }],
  ["se", { x: x + width, y: y + height }],
  ["s", { x: x + width / 2, y: y + height }],
  ["sw", { x, y: y + height }],
  ["w", { x, y: y + height / 2 }],
];

export const handlePositions = (mark: Mark): [Handle, Point][] => {
  switch (mark.kind) {
    case "arrow":
    case "line": {
      return [
        ["start", mark.from],
        ["end", mark.to],
      ];
    }
    case "rect":
    case "oval":
    case "highlight":
    case "conceal":
    case "spotlight": {
      return rectHandles(mark.rect);
    }
    case "counter":
    case "freehand":
    case "text": {
      return [];
    }
    default: {
      return unreachable(mark);
    }
  }
};

/** Verschiebt ein Rechteck an einem Griff. */
export const resizeRect = (rect: Rect, handle: Handle, point: Point): Rect => {
  const left = handle.includes("w") ? point.x : rect.x;
  const right = handle.includes("e") ? point.x : rect.x + rect.width;
  const top = handle.startsWith("n") ? point.y : rect.y;
  const bottom = handle.startsWith("s") ? point.y : rect.y + rect.height;
  return rectFromPoints({ x: left, y: top }, { x: right, y: bottom });
};

export const resizeMark = (mark: Mark, handle: Handle, point: Point): Mark => {
  switch (mark.kind) {
    case "arrow":
    case "line": {
      return handle === "start"
        ? { ...mark, from: point }
        : { ...mark, to: point };
    }
    case "rect":
    case "oval":
    case "highlight":
    case "conceal":
    case "spotlight": {
      return { ...mark, rect: resizeRect(mark.rect, handle, point) };
    }
    case "counter":
    case "freehand":
    case "text": {
      return mark;
    }
    default: {
      return unreachable(mark);
    }
  }
};

const movePoint = (point: Point, dx: number, dy: number): Point => ({
  x: point.x + dx,
  y: point.y + dy,
});

export const moveMark = (mark: Mark, dx: number, dy: number): Mark => {
  switch (mark.kind) {
    case "arrow":
    case "line": {
      return {
        ...mark,
        from: movePoint(mark.from, dx, dy),
        to: movePoint(mark.to, dx, dy),
      };
    }
    case "freehand": {
      return { ...mark, points: mark.points.map((p) => movePoint(p, dx, dy)) };
    }
    case "counter": {
      return { ...mark, center: movePoint(mark.center, dx, dy) };
    }
    case "text": {
      return { ...mark, origin: movePoint(mark.origin, dx, dy) };
    }
    case "rect":
    case "oval":
    case "highlight":
    case "conceal":
    case "spotlight": {
      return {
        ...mark,
        rect: { ...mark.rect, x: mark.rect.x + dx, y: mark.rect.y + dy },
      };
    }
    default: {
      return unreachable(mark);
    }
  }
};

/** Mit gedrückter Umschalttaste: Linien auf 45°-Schritte. */
export const constrainLine = (from: Point, to: Point): Point => {
  const angle = Math.atan2(to.y - from.y, to.x - from.x);
  const step = Math.PI / 4;
  const snapped = Math.round(angle / step) * step;
  const length = Math.hypot(to.x - from.x, to.y - from.y);
  return {
    x: from.x + Math.cos(snapped) * length,
    y: from.y + Math.sin(snapped) * length,
  };
};

/** Mit gedrückter Umschalttaste: Rechtecke und Ovale quadratisch. */
export const constrainSquare = (from: Point, to: Point): Point => {
  const size = Math.max(Math.abs(to.x - from.x), Math.abs(to.y - from.y));
  return {
    x: from.x + Math.sign(to.x - from.x || 1) * size,
    y: from.y + Math.sign(to.y - from.y || 1) * size,
  };
};
