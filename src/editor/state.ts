import { intersectRects, moveMark, roundRect } from "./geometry";
import { createDoc } from "./model";
import type {
  ArrowStyle,
  ConcealMode,
  Doc,
  FillMode,
  Mark,
  Rect,
  TextStyle,
  Tool,
} from "./model";
import { unreachable } from "./unreachable";

export interface ToolStyle {
  color: string;
  highlightColor: string;
  /** Strichstärke in Punkten. */
  size: number;
  /** Schriftgröße in Punkten. */
  fontSize: number;
  arrowStyle: ArrowStyle;
  fill: FillMode;
  concealMode: ConcealMode;
  textStyle: TextStyle;
  dashed: boolean;
}

export interface EditorState {
  doc: Doc;
  past: Doc[];
  future: Doc[];
  /** Stand vor einer laufenden Ziehbewegung. */
  dragBase: Doc | null;
  tool: Tool;
  style: ToolStyle;
  selectedId: string | null;
  /** Auswahlrahmen in Bildpixeln (für Zuschnitt, OCR, Kopieren). */
  selection: Rect | null;
  editingId: string | null;
}

export const DEFAULT_STYLE: ToolStyle = {
  arrowStyle: "solid",
  color: "#e5383b",
  concealMode: "pixelate",
  dashed: false,
  fill: "outline",
  fontSize: 20,
  highlightColor: "#ffe74c",
  size: 4,
  textStyle: "plain",
};

const HISTORY_LIMIT = 100;

export const initialState = (doc: Doc = createDoc(0, 0, 1)): EditorState => ({
  doc,
  dragBase: null,
  editingId: null,
  future: [],
  past: [],
  selectedId: null,
  selection: null,
  style: DEFAULT_STYLE,
  tool: "select",
});

export type EditorAction =
  | { type: "setTool"; tool: Tool }
  | { type: "setStyle"; style: Partial<ToolStyle> }
  | { type: "commit"; doc: Doc; selectedId?: string | null }
  | { type: "beginDrag" }
  | { type: "preview"; doc: Doc }
  | { type: "endDrag" }
  | { type: "undo" }
  | { type: "redo" }
  | { type: "select"; id: string | null }
  | { type: "setSelection"; rect: Rect | null }
  | { type: "crop" }
  | { type: "resetCrop" }
  | { type: "deleteSelected" }
  | { type: "duplicateSelected"; offset: number }
  | { type: "nudge"; dx: number; dy: number }
  | { type: "startEditing"; id: string }
  | { type: "finishEditing"; text: string };

/** Neuer Stand mit dem bisherigen Dokument in der Historie. */
const withHistory = (state: EditorState, doc: Doc): EditorState => ({
  ...state,
  doc,
  future: [],
  past: [...state.past, state.doc].slice(-HISTORY_LIMIT),
});

const replaceMark = (
  doc: Doc,
  id: string,
  update: (mark: Mark) => Mark
): Doc => ({
  ...doc,
  marks: doc.marks.map((mark) => (mark.id === id ? update(mark) : mark)),
});

/** Entfernt Texte ohne Inhalt, z. B. nach abgebrochener Eingabe. */
const withoutEmptyText = (doc: Doc): Doc =>
  doc.marks.some((mark) => mark.kind === "text" && mark.text === "")
    ? {
        ...doc,
        marks: doc.marks.filter(
          (mark) => mark.kind !== "text" || mark.text !== ""
        ),
      }
    : doc;

const pick = <T>(value: T | undefined, fallback: T): T => value ?? fallback;

/** Überträgt Stiländerungen auf eine Markierung. */
const applyStyle = (mark: Mark, style: Partial<ToolStyle>): Mark => {
  switch (mark.kind) {
    case "arrow": {
      return {
        ...mark,
        arrowStyle: pick(style.arrowStyle, mark.arrowStyle),
        color: pick(style.color, mark.color),
        size: pick(style.size, mark.size),
      };
    }
    case "line": {
      return {
        ...mark,
        color: pick(style.color, mark.color),
        dashed: pick(style.dashed, mark.dashed),
        size: pick(style.size, mark.size),
      };
    }
    case "rect":
    case "oval": {
      return {
        ...mark,
        color: pick(style.color, mark.color),
        fill: pick(style.fill, mark.fill),
        size: pick(style.size, mark.size),
      };
    }
    case "freehand":
    case "counter": {
      return {
        ...mark,
        color: pick(style.color, mark.color),
        size: pick(style.size, mark.size),
      };
    }
    case "highlight": {
      return { ...mark, color: pick(style.highlightColor, mark.color) };
    }
    case "conceal": {
      return { ...mark, mode: pick(style.concealMode, mark.mode) };
    }
    case "text": {
      return {
        ...mark,
        color: pick(style.color, mark.color),
        size: pick(style.fontSize, mark.size),
        textStyle: pick(style.textStyle, mark.textStyle),
      };
    }
    case "spotlight": {
      return mark;
    }
    default: {
      return unreachable(mark);
    }
  }
};

const setStyle = (
  state: EditorState,
  style: Partial<ToolStyle>
): EditorState => {
  const next = { ...state, style: { ...state.style, ...style } };
  const id = state.selectedId;
  return id === null
    ? next
    : withHistory(
        next,
        replaceMark(state.doc, id, (mark) => applyStyle(mark, style))
      );
};

const undo = (state: EditorState): EditorState => {
  const previous = state.past.at(-1);
  if (!previous) {
    return state;
  }
  return {
    ...state,
    doc: previous,
    editingId: null,
    future: [state.doc, ...state.future],
    past: state.past.slice(0, -1),
    selectedId: null,
  };
};

const redo = (state: EditorState): EditorState => {
  const [next, ...future] = state.future;
  if (!next) {
    return state;
  }
  return {
    ...state,
    doc: next,
    editingId: null,
    future,
    past: [...state.past, state.doc],
    selectedId: null,
  };
};

const endDrag = (state: EditorState): EditorState => {
  const base = state.dragBase;
  if (!base || base === state.doc) {
    return { ...state, dragBase: null };
  }
  return { ...withHistory({ ...state, doc: base }, state.doc), dragBase: null };
};

const crop = (state: EditorState): EditorState => {
  if (!state.selection) {
    return state;
  }
  const area = roundRect(intersectRects(state.selection, state.doc.crop));
  const cleared = { ...state, selection: null };
  return area.width < 1 || area.height < 1
    ? cleared
    : withHistory(cleared, { ...state.doc, crop: area });
};

const resetCrop = (state: EditorState): EditorState => {
  const { width, height } = state.doc;
  return withHistory(state, {
    ...state.doc,
    crop: { height, width, x: 0, y: 0 },
  });
};

const deleteSelected = (state: EditorState): EditorState => {
  const id = state.selectedId;
  if (id === null) {
    return state;
  }
  const doc = {
    ...state.doc,
    marks: state.doc.marks.filter((mark) => mark.id !== id),
  };
  return { ...withHistory(state, doc), editingId: null, selectedId: null };
};

const duplicateSelected = (state: EditorState, offset: number): EditorState => {
  const original = state.doc.marks.find((mark) => mark.id === state.selectedId);
  if (!original) {
    return state;
  }
  const copy = {
    ...moveMark(original, offset, offset),
    id: `${original.id}-${state.past.length}`,
  };
  return {
    ...withHistory(state, { ...state.doc, marks: [...state.doc.marks, copy] }),
    selectedId: copy.id,
  };
};

const nudge = (state: EditorState, dx: number, dy: number): EditorState => {
  const id = state.selectedId;
  return id === null
    ? state
    : withHistory(
        state,
        replaceMark(state.doc, id, (mark) => moveMark(mark, dx, dy))
      );
};

const finishEditing = (state: EditorState, rawText: string): EditorState => {
  const id = state.editingId;
  if (id === null || !state.doc.marks.some((mark) => mark.id === id)) {
    return { ...state, editingId: null };
  }
  const text = rawText.replace(/\s+$/u, "");
  const marks =
    text === ""
      ? state.doc.marks.filter((mark) => mark.id !== id)
      : state.doc.marks.map((mark) =>
          mark.id === id && mark.kind === "text" ? { ...mark, text } : mark
        );
  return {
    ...state,
    doc: { ...state.doc, marks },
    editingId: null,
    selectedId: text === "" ? null : id,
  };
};

export const editorReducer = (
  state: EditorState,
  action: EditorAction
): EditorState => {
  switch (action.type) {
    case "setTool": {
      return {
        ...state,
        doc: withoutEmptyText(state.doc),
        editingId: null,
        selectedId: null,
        selection: null,
        tool: action.tool,
      };
    }
    case "setStyle": {
      return setStyle(state, action.style);
    }
    case "commit": {
      return {
        ...withHistory(state, action.doc),
        selectedId:
          action.selectedId === undefined
            ? state.selectedId
            : action.selectedId,
      };
    }
    case "beginDrag": {
      return { ...state, dragBase: state.doc };
    }
    case "preview": {
      return { ...state, doc: action.doc };
    }
    case "endDrag": {
      return endDrag(state);
    }
    case "undo": {
      return undo(state);
    }
    case "redo": {
      return redo(state);
    }
    case "select": {
      return { ...state, selectedId: action.id, selection: null };
    }
    case "setSelection": {
      return { ...state, selectedId: null, selection: action.rect };
    }
    case "crop": {
      return crop(state);
    }
    case "resetCrop": {
      return resetCrop(state);
    }
    case "deleteSelected": {
      return deleteSelected(state);
    }
    case "duplicateSelected": {
      return duplicateSelected(state, action.offset);
    }
    case "nudge": {
      return nudge(state, action.dx, action.dy);
    }
    case "startEditing": {
      return { ...state, editingId: action.id, selectedId: action.id };
    }
    case "finishEditing": {
      return finishEditing(state, action.text);
    }
    default: {
      return unreachable(action);
    }
  }
};

/** Nächster freier Wert für einen Zähler. */
export const nextCounterValue = (doc: Doc): number => {
  let highest = 0;
  for (const mark of doc.marks) {
    if (mark.kind === "counter") {
      highest = Math.max(highest, mark.value);
    }
  }
  return highest + 1;
};
