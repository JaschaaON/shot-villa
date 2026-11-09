import type { Dispatch, ReactNode } from "react";

import { HIGHLIGHT_PALETTE, PALETTE } from "./colors";
import type { Tool } from "./model";
import type { EditorAction, EditorState, ToolStyle } from "./state";
import { TOOLS } from "./tools";

const SIZES = [2, 4, 6, 10] as const;
const FONT_SIZES = [14, 20, 28, 40] as const;
const SIZE_LABELS = ["S", "M", "L", "XL"] as const;

interface Props {
  state: EditorState;
  dispatch: Dispatch<EditorAction>;
}

interface ChoiceProps<T extends string | number | boolean> {
  label: string;
  value: T;
  options: readonly { value: T; label: ReactNode; title: string }[];
  onChange: (value: T) => void;
}

const Field = ({ label, children }: { label: string; children: ReactNode }) => (
  <div className="option">
    <span className="option-label">{label}</span>
    {children}
  </div>
);

const Choice = <T extends string | number | boolean>({
  label,
  value,
  options,
  onChange,
}: ChoiceProps<T>) => (
  <Field label={label}>
    <div className="segmented">
      {options.map((option) => (
        <button
          aria-pressed={option.value === value}
          className={option.value === value ? "active" : undefined}
          key={String(option.value)}
          onClick={() => {
            onChange(option.value);
          }}
          title={option.title}
          type="button"
        >
          {option.label}
        </button>
      ))}
    </div>
  </Field>
);

/** Werkzeug, dessen Optionen gezeigt werden: das der Auswahl oder das aktive. */
const activeKind = (state: EditorState): Tool => {
  const selected = state.doc.marks.find((mark) => mark.id === state.selectedId);
  return selected ? selected.kind : state.tool;
};

export const Options = ({ state, dispatch }: Props) => {
  const { style } = state;
  const kind = activeKind(state);
  const set = (change: Partial<ToolStyle>) => {
    dispatch({ style: change, type: "setStyle" });
  };

  const usesColor = !["select", "conceal", "spotlight", "highlight"].includes(
    kind
  );
  const usesSize = [
    "arrow",
    "line",
    "rect",
    "oval",
    "freehand",
    "counter",
  ].includes(kind);

  const info = TOOLS.find((entry) => entry.tool === kind);

  return (
    <div className="options">
      <div className="options-head">
        {info?.icon}
        <span>{info?.label ?? "Auswahl"}</span>
        {info ? <kbd>{info.key.toUpperCase()}</kbd> : null}
      </div>
      {kind === "arrow" ? (
        <Choice
          label="Spitze"
          onChange={(arrowStyle) => {
            set({ arrowStyle });
          }}
          options={[
            { label: "Gefüllt", title: "Gefüllte Spitze", value: "solid" },
            { label: "Offen", title: "Offene Spitze", value: "open" },
          ]}
          value={style.arrowStyle}
        />
      ) : null}
      {kind === "rect" || kind === "oval" ? (
        <Choice
          label="Füllung"
          onChange={(fill) => {
            set({ fill });
          }}
          options={[
            { label: "Rahmen", title: "Nur Umriss", value: "outline" },
            { label: "Gefüllt", title: "Gefüllte Fläche", value: "filled" },
          ]}
          value={style.fill}
        />
      ) : null}
      {kind === "line" ? (
        <Choice
          label="Linie"
          onChange={(dashed) => {
            set({ dashed });
          }}
          options={[
            {
              label: "Durchgezogen",
              title: "Durchgezogene Linie",
              value: false,
            },
            { label: "Gestrichelt", title: "Gestrichelte Linie", value: true },
          ]}
          value={style.dashed}
        />
      ) : null}
      {kind === "conceal" ? (
        <Choice
          label="Art"
          onChange={(concealMode) => {
            set({ concealMode });
          }}
          options={[
            { label: "Verpixeln", title: "Grobe Pixel", value: "pixelate" },
            { label: "Weichzeichnen", title: "Unscharf", value: "blur" },
          ]}
          value={style.concealMode}
        />
      ) : null}
      {kind === "text" ? (
        <>
          <Choice
            label="Schrift"
            onChange={(fontSize) => {
              set({ fontSize });
            }}
            options={FONT_SIZES.map((size, index) => ({
              label: SIZE_LABELS[index],
              title: `Schrift ${size} pt`,
              value: size,
            }))}
            value={style.fontSize}
          />
          <Choice
            label="Stil"
            onChange={(textStyle) => {
              set({ textStyle });
            }}
            options={[
              {
                label: "Text",
                title: "Farbiger Text mit Kontur",
                value: "plain",
              },
              {
                label: "Etikett",
                title: "Text auf farbigem Grund",
                value: "label",
              },
            ]}
            value={style.textStyle}
          />
        </>
      ) : null}
      {usesSize ? (
        <Choice
          label="Stärke"
          onChange={(size) => {
            set({ size });
          }}
          options={SIZES.map((size, index) => ({
            label: SIZE_LABELS[index],
            title: `Stärke ${size} pt`,
            value: size,
          }))}
          value={style.size}
        />
      ) : null}
      {usesColor ? (
        <Field label="Farbe">
          <div className="palette">
            {PALETTE.map((color) => (
              <button
                aria-label={color}
                aria-pressed={style.color === color}
                className={style.color === color ? "color active" : "color"}
                key={color}
                onClick={() => {
                  set({ color });
                }}
                style={{ background: color }}
                type="button"
              />
            ))}
            <label className="custom" title="Eigene Farbe">
              <span className="visually-hidden">Eigene Farbe</span>
              <input
                onChange={(event) => {
                  set({ color: event.target.value });
                }}
                type="color"
                value={style.color}
              />
            </label>
          </div>
        </Field>
      ) : null}
      {kind === "highlight" ? (
        <Field label="Markerfarbe">
          <div className="palette">
            {HIGHLIGHT_PALETTE.map((color) => (
              <button
                aria-label={color}
                className={
                  style.highlightColor === color ? "color active" : "color"
                }
                key={color}
                onClick={() => {
                  set({ highlightColor: color });
                }}
                style={{ background: color }}
                type="button"
              />
            ))}
          </div>
        </Field>
      ) : null}
      {kind === "select" ? (
        <span className="hint">
          Ziehen wählt einen Bereich.
          <br />↩ schneidet zu, Tab kopiert die Farbe.
        </span>
      ) : null}
    </div>
  );
};
