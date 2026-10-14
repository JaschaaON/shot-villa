import { useEffect, useRef, useState } from "react";

import { contrastText } from "./colors";
import type { Rect, TextMark } from "./model";

interface Props {
  mark: TextMark;
  crop: Rect;
  /** Pixel pro Punkt des Bildes. */
  scale: number;
  viewScale: number;
  onFinish: (text: string) => void;
}

/** Textfeld über der Zeichenfläche, solange ein Text bearbeitet wird. */
export const TextEditor = ({
  mark,
  crop,
  scale,
  viewScale,
  onFinish,
}: Props) => {
  const [text, setText] = useState(mark.text);
  const ref = useRef<HTMLTextAreaElement>(null);
  const fontSize = mark.size * scale * viewScale;
  const isLabel = mark.textStyle === "label";

  useEffect(() => {
    ref.current?.focus();
    ref.current?.select();
  }, []);

  const lines = text.split("\n");
  const columns = Math.max(2, ...lines.map((line) => line.length + 1));

  return (
    <textarea
      className="text-editor"
      cols={columns}
      onBlur={() => {
        onFinish(text);
      }}
      onChange={(event) => {
        setText(event.target.value);
      }}
      onKeyDown={(event) => {
        event.stopPropagation();
        if (
          event.key === "Escape" ||
          (event.key === "Enter" && event.metaKey)
        ) {
          event.preventDefault();
          onFinish(text);
        }
      }}
      ref={ref}
      rows={lines.length}
      spellCheck={false}
      style={{
        background: isLabel ? mark.color : "transparent",
        color: isLabel ? contrastText(mark.color) : mark.color,
        fontSize,
        left: (mark.origin.x - crop.x) * viewScale,
        padding: isLabel ? fontSize * 0.35 : 0,
        top: (mark.origin.y - crop.y) * viewScale,
      }}
      value={text}
    />
  );
};
