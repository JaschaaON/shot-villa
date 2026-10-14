import { useEffect, useLayoutEffect, useRef } from "react";
import type {
  Dispatch,
  PointerEvent as ReactPointerEvent,
  RefObject,
} from "react";

import type { Rgb } from "./colors";
import {
  constrainLine,
  constrainSquare,
  handlePositions,
  hitMark,
  moveMark,
  rectFromPoints,
  resizeMark,
} from "./geometry";
import type { Handle } from "./geometry";
import { newId } from "./model";
import type { Mark, Point, Rect, Tool } from "./model";
import type { PixelReader } from "./pixels";
import { renderDoc } from "./render";
import { nextCounterValue } from "./state";
import type { EditorAction, EditorState } from "./state";
import { unreachable } from "./unreachable";

type Drag =
  | { kind: "create"; id: string; start: Point }
  | { kind: "move"; id: string; last: Point }
  | { kind: "resize"; id: string; handle: Handle }
  | { kind: "marquee"; start: Point };

const MIN_SIZE = 3;
const CURSORS = new Map<Tool, string>([
  ["select", "default"],
  ["text", "text"],
]);
const HIT_TOLERANCE_CSS = 6;
const HANDLE_CSS = 7;

interface Props {
  image: HTMLImageElement;
  state: EditorState;
  dispatch: Dispatch<EditorAction>;
  /** CSS-Pixel je Bildpixel. */
  viewScale: number;
  pixels: PixelReader;
  onCursorColor: (color: Rgb | null) => void;
  textBounds: RefObject<Map<string, Rect>>;
}

/** Neue Form des aktuellen Werkzeugs am Punkt `at`. */
const createMark = (state: EditorState, at: Point): Mark | null => {
  const { style, doc } = state;
  const id = newId();
  const rect = { height: 0, width: 0, ...at };
  switch (state.tool) {
    case "arrow": {
      return {
        arrowStyle: style.arrowStyle,
        color: style.color,
        from: at,
        id,
        kind: "arrow",
        size: style.size,
        to: at,
      };
    }
    case "line": {
      return {
        color: style.color,
        dashed: style.dashed,
        from: at,
        id,
        kind: "line",
        size: style.size,
        to: at,
      };
    }
    case "rect":
    case "oval": {
      return {
        color: style.color,
        fill: style.fill,
        id,
        kind: state.tool,
        rect,
        size: style.size,
      };
    }
    case "freehand": {
      return {
        color: style.color,
        id,
        kind: "freehand",
        points: [at],
        size: style.size,
      };
    }
    case "highlight": {
      return {
        color: style.highlightColor,
        id,
        kind: "highlight",
        rect,
        size: style.size,
      };
    }
    case "conceal": {
      return { id, kind: "conceal", mode: style.concealMode, rect };
    }
    case "spotlight": {
      return { id, kind: "spotlight", rect };
    }
    case "counter": {
      return {
        center: at,
        color: style.color,
        id,
        kind: "counter",
        size: style.size,
        value: nextCounterValue(doc),
      };
    }
    case "text": {
      return {
        color: style.color,
        id,
        kind: "text",
        origin: at,
        size: style.fontSize,
        text: "",
        textStyle: style.textStyle,
      };
    }
    case "select": {
      return null;
    }
    default: {
      return unreachable(state.tool);
    }
  }
};

/** Zieht eine neu entstehende Form zum Punkt `to`. */
const extendMark = (
  mark: Mark,
  start: Point,
  to: Point,
  constrained: boolean
): Mark => {
  switch (mark.kind) {
    case "arrow":
    case "line": {
      return { ...mark, to: constrained ? constrainLine(start, to) : to };
    }
    case "freehand": {
      return { ...mark, points: [...mark.points, to] };
    }
    case "rect":
    case "oval":
    case "highlight":
    case "conceal":
    case "spotlight": {
      return {
        ...mark,
        rect: rectFromPoints(
          start,
          constrained ? constrainSquare(start, to) : to
        ),
      };
    }
    case "counter":
    case "text": {
      return mark;
    }
    default: {
      return unreachable(mark);
    }
  }
};

const isTooSmall = (mark: Mark): boolean => {
  switch (mark.kind) {
    case "arrow":
    case "line": {
      return (
        Math.hypot(mark.to.x - mark.from.x, mark.to.y - mark.from.y) < MIN_SIZE
      );
    }
    case "freehand": {
      return mark.points.length < 2;
    }
    case "rect":
    case "oval":
    case "highlight":
    case "conceal":
    case "spotlight": {
      return mark.rect.width < MIN_SIZE || mark.rect.height < MIN_SIZE;
    }
    case "counter":
    case "text": {
      return false;
    }
    default: {
      return unreachable(mark);
    }
  }
};

const withMark = (
  state: EditorState,
  id: string,
  update: (mark: Mark) => Mark
) => ({
  ...state.doc,
  marks: state.doc.marks.map((mark) => (mark.id === id ? update(mark) : mark)),
});

export const EditorCanvas = ({
  image,
  state,
  dispatch,
  viewScale,
  pixels,
  onCursorColor,
  textBounds,
}: Props) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const stateRef = useRef(state);
  const dragRef = useRef<Drag | null>(null);
  const { doc } = state;

  useLayoutEffect(() => {
    stateRef.current = state;
  });

  useEffect(() => {
    const ctx = canvasRef.current?.getContext("2d");
    if (!ctx) {
      return;
    }
    renderDoc(ctx, {
      doc,
      image,
      textBounds: textBounds.current,
      ui: {
        editingId: state.editingId,
        selectedId: state.selectedId,
        selection: state.selection,
      },
      viewScale,
    });
  }, [
    doc,
    image,
    state.editingId,
    state.selectedId,
    state.selection,
    textBounds,
    viewScale,
  ]);

  const toImage = (event: { clientX: number; clientY: number }): Point => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return { x: 0, y: 0 };
    }
    const bounds = canvas.getBoundingClientRect();
    return {
      x:
        doc.crop.x +
        ((event.clientX - bounds.left) * canvas.width) / bounds.width,
      y:
        doc.crop.y +
        ((event.clientY - bounds.top) * canvas.height) / bounds.height,
    };
  };

  const textSize = (mark: Mark) => textBounds.current.get(mark.id);

  const onPointerDown = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    if (event.button !== 0) {
      return;
    }
    const { current } = stateRef;
    const point = toImage(event);
    const tolerance = HIT_TOLERANCE_CSS / viewScale;
    event.currentTarget.setPointerCapture(event.pointerId);

    const selected = current.doc.marks.find(
      (mark) => mark.id === current.selectedId
    );
    const handle = selected
      ? handlePositions(selected).find(
          ([, position]) =>
            Math.hypot(position.x - point.x, position.y - point.y) <=
            HANDLE_CSS / viewScale
        )
      : undefined;
    if (selected && handle) {
      dispatch({ type: "beginDrag" });
      dragRef.current = { handle: handle[0], id: selected.id, kind: "resize" };
      return;
    }

    const hit = current.doc.marks.findLast((mark) =>
      hitMark(mark, point, tolerance, textSize, current.doc.scale)
    );
    if (hit) {
      if (
        hit.kind === "text" &&
        (event.detail >= 2 || current.selectedId === hit.id)
      ) {
        dispatch({ id: hit.id, type: "startEditing" });
        return;
      }
      dispatch({ id: hit.id, type: "select" });
      dispatch({ type: "beginDrag" });
      dragRef.current = { id: hit.id, kind: "move", last: point };
      return;
    }

    if (current.tool === "select") {
      dragRef.current = { kind: "marquee", start: point };
      dispatch({ rect: null, type: "setSelection" });
      return;
    }
    const mark = createMark(current, point);
    if (!mark) {
      return;
    }
    if (mark.kind === "counter") {
      dispatch({
        doc: { ...current.doc, marks: [...current.doc.marks, mark] },
        selectedId: null,
        type: "commit",
      });
      return;
    }
    if (mark.kind === "text") {
      dispatch({
        doc: { ...current.doc, marks: [...current.doc.marks, mark] },
        selectedId: mark.id,
        type: "commit",
      });
      dispatch({ id: mark.id, type: "startEditing" });
      return;
    }
    dispatch({ id: null, type: "select" });
    dispatch({ type: "beginDrag" });
    dispatch({
      doc: { ...current.doc, marks: [...current.doc.marks, mark] },
      type: "preview",
    });
    dragRef.current = { id: mark.id, kind: "create", start: point };
  };

  const onPointerMove = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    const point = toImage(event);
    onCursorColor(pixels.at(point.x, point.y));
    const drag = dragRef.current;
    if (!drag) {
      return;
    }
    const { current } = stateRef;
    switch (drag.kind) {
      case "marquee": {
        dispatch({
          rect: rectFromPoints(drag.start, point),
          type: "setSelection",
        });
        break;
      }
      case "create": {
        dispatch({
          doc: withMark(current, drag.id, (mark) =>
            extendMark(mark, drag.start, point, event.shiftKey)
          ),
          type: "preview",
        });
        break;
      }
      case "move": {
        const dx = point.x - drag.last.x;
        const dy = point.y - drag.last.y;
        dragRef.current = { ...drag, last: point };
        dispatch({
          doc: withMark(current, drag.id, (mark) => moveMark(mark, dx, dy)),
          type: "preview",
        });
        break;
      }
      case "resize": {
        dispatch({
          doc: withMark(current, drag.id, (mark) =>
            resizeMark(mark, drag.handle, point)
          ),
          type: "preview",
        });
        break;
      }
      default: {
        break;
      }
    }
  };

  const onPointerUp = () => {
    const drag = dragRef.current;
    dragRef.current = null;
    if (!drag || drag.kind === "marquee") {
      const { selection } = stateRef.current;
      if (
        selection &&
        (selection.width < MIN_SIZE || selection.height < MIN_SIZE)
      ) {
        dispatch({ rect: null, type: "setSelection" });
      }
      return;
    }
    const { current } = stateRef;
    if (drag.kind === "create") {
      const created = current.doc.marks.find((mark) => mark.id === drag.id);
      if (!created || isTooSmall(created)) {
        dispatch({
          doc: {
            ...current.doc,
            marks: current.doc.marks.filter((mark) => mark.id !== drag.id),
          },
          type: "preview",
        });
      } else {
        dispatch({ id: drag.id, type: "select" });
      }
    }
    dispatch({ type: "endDrag" });
  };

  const cursor = CURSORS.get(state.tool) ?? "crosshair";

  return (
    <canvas
      className="canvas"
      height={doc.crop.height}
      onPointerCancel={onPointerUp}
      onPointerDown={onPointerDown}
      onPointerLeave={() => {
        onCursorColor(null);
      }}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      ref={canvasRef}
      style={{
        cursor,
        height: doc.crop.height * viewScale,
        width: doc.crop.width * viewScale,
      }}
      width={doc.crop.width}
    />
  );
};
