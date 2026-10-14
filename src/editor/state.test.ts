import { describe, expect, it } from "vitest";

import { createDoc } from "./model";
import type { BoxMark, CounterMark, Doc, TextMark } from "./model";
import { editorReducer, initialState, nextCounterValue } from "./state";
import type { EditorState } from "./state";

const box: BoxMark = {
  color: "#000000",
  fill: "outline",
  id: "box",
  kind: "rect",
  rect: { height: 10, width: 10, x: 0, y: 0 },
  size: 2,
};

const withMarks = (doc: Doc, marks: Doc["marks"]): Doc => ({ ...doc, marks });

const loaded = (): EditorState => initialState(createDoc(200, 100, 2));

describe(editorReducer, () => {
  it("macht Änderungen rückgängig und wiederholt sie", () => {
    const start = loaded();
    const changed = editorReducer(start, {
      doc: withMarks(start.doc, [box]),
      type: "commit",
    });
    const undone = editorReducer(changed, { type: "undo" });
    expect(undone.doc.marks).toHaveLength(0);
    const redone = editorReducer(undone, { type: "redo" });
    expect(redone.doc.marks).toStrictEqual([box]);
  });

  it("legt eine Ziehbewegung als einen Schritt in die Historie", () => {
    let state = editorReducer(loaded(), { type: "beginDrag" });
    for (const x of [1, 2, 3]) {
      state = editorReducer(state, {
        doc: withMarks(state.doc, [{ ...box, rect: { ...box.rect, x } }]),
        type: "preview",
      });
    }
    state = editorReducer(state, { type: "endDrag" });
    expect(state.past).toHaveLength(1);
    expect(editorReducer(state, { type: "undo" }).doc.marks).toHaveLength(0);
  });

  it("schneidet auf die Auswahl zu und rundet auf ganze Pixel", () => {
    let state = editorReducer(loaded(), {
      rect: { height: 20.4, width: 50.5, x: 10.2, y: -5 },
      type: "setSelection",
    });
    state = editorReducer(state, { type: "crop" });
    expect(state.doc.crop).toStrictEqual({
      height: 16,
      width: 51,
      x: 10,
      y: 0,
    });
    expect(state.selection).toBeNull();
    expect(editorReducer(state, { type: "resetCrop" }).doc.crop.width).toBe(
      200
    );
  });

  it("überträgt Stiländerungen auf die ausgewählte Form", () => {
    let state = editorReducer(loaded(), {
      doc: withMarks(loaded().doc, [box]),
      selectedId: "box",
      type: "commit",
    });
    state = editorReducer(state, {
      style: { color: "#ff0000", size: 8 },
      type: "setStyle",
    });
    const [mark] = state.doc.marks;
    expect(
      mark?.kind === "rect" ? [mark.color, mark.size] : null
    ).toStrictEqual(["#ff0000", 8]);
  });

  it("entfernt leeren Text nach dem Bearbeiten", () => {
    const text: TextMark = {
      color: "#000000",
      id: "t",
      kind: "text",
      origin: { x: 0, y: 0 },
      size: 20,
      text: "",
      textStyle: "plain",
    };
    let state = editorReducer(loaded(), {
      doc: withMarks(loaded().doc, [text]),
      type: "commit",
    });
    state = editorReducer(state, { id: "t", type: "startEditing" });
    state = editorReducer(state, { text: "  ", type: "finishEditing" });
    expect(state.doc.marks).toHaveLength(0);
    expect(state.editingId).toBeNull();
  });

  it("verwirft leere Texte beim Werkzeugwechsel", () => {
    const empty: TextMark = {
      color: "#000000",
      id: "leer",
      kind: "text",
      origin: { x: 0, y: 0 },
      size: 20,
      text: "",
      textStyle: "plain",
    };
    let state = editorReducer(loaded(), {
      doc: withMarks(loaded().doc, [empty, box]),
      type: "commit",
    });
    state = editorReducer(state, { tool: "arrow", type: "setTool" });
    expect(state.doc.marks).toStrictEqual([box]);
  });

  it("löscht und dupliziert die Auswahl", () => {
    let state = editorReducer(loaded(), {
      doc: withMarks(loaded().doc, [box]),
      selectedId: "box",
      type: "commit",
    });
    state = editorReducer(state, { offset: 10, type: "duplicateSelected" });
    expect(state.doc.marks).toHaveLength(2);
    state = editorReducer(state, { type: "deleteSelected" });
    expect(state.doc.marks).toStrictEqual([box]);
  });
});

const counter = (value: number): CounterMark => ({
  center: { x: 0, y: 0 },
  color: "#000000",
  id: `c${value}`,
  kind: "counter",
  size: 4,
  value,
});

describe(nextCounterValue, () => {
  it("zählt ab der höchsten Zahl weiter", () => {
    expect(nextCounterValue(createDoc(1, 1, 1))).toBe(1);
    expect(
      nextCounterValue(withMarks(createDoc(1, 1, 1), [counter(1), counter(4)]))
    ).toBe(5);
  });
});
