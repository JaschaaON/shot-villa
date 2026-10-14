/** Alle Koordinaten sind Pixel des Originalbildes (Ursprung oben links). */
export interface Point {
  x: number;
  y: number;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type Tool =
  | "select"
  | "arrow"
  | "text"
  | "rect"
  | "oval"
  | "line"
  | "freehand"
  | "highlight"
  | "conceal"
  | "spotlight"
  | "counter";

export type ArrowStyle = "solid" | "open";
export type FillMode = "outline" | "filled";
export type ConcealMode = "blur" | "pixelate";
export type TextStyle = "plain" | "label";

/** Gemeinsame Gestaltung; `size` ist die Strichstärke in Punkten. */
export interface Style {
  color: string;
  size: number;
}

interface Base {
  id: string;
}

export interface ArrowMark extends Base, Style {
  kind: "arrow";
  from: Point;
  to: Point;
  arrowStyle: ArrowStyle;
}

export interface LineMark extends Base, Style {
  kind: "line";
  from: Point;
  to: Point;
  dashed: boolean;
}

export interface BoxMark extends Base, Style {
  kind: "rect" | "oval";
  rect: Rect;
  fill: FillMode;
}

export interface FreehandMark extends Base, Style {
  kind: "freehand";
  points: Point[];
}

export interface HighlightMark extends Base, Style {
  kind: "highlight";
  rect: Rect;
}

export interface ConcealMark extends Base {
  kind: "conceal";
  rect: Rect;
  mode: ConcealMode;
}

export interface SpotlightMark extends Base {
  kind: "spotlight";
  rect: Rect;
}

export interface CounterMark extends Base, Style {
  kind: "counter";
  center: Point;
  value: number;
}

export interface TextMark extends Base, Style {
  kind: "text";
  origin: Point;
  text: string;
  textStyle: TextStyle;
}

export type Mark =
  | ArrowMark
  | LineMark
  | BoxMark
  | FreehandMark
  | HighlightMark
  | ConcealMark
  | SpotlightMark
  | CounterMark
  | TextMark;

export interface Doc {
  /** Pixelgröße des Originalbildes. */
  width: number;
  height: number;
  /** Pixel pro Punkt. */
  scale: number;
  /** Sichtbarer Ausschnitt; ohne Zuschnitt das ganze Bild. */
  crop: Rect;
  marks: Mark[];
}

export const createDoc = (
  width: number,
  height: number,
  scale: number
): Doc => ({
  crop: { height, width, x: 0, y: 0 },
  height,
  marks: [],
  scale,
  width,
});

let counter = 0;

/** Eindeutige Id für neue Formen. */
export const newId = (): string => {
  counter += 1;
  return `s${Date.now().toString(36)}${counter.toString(36)}`;
};
