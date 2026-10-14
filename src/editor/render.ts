import { contrastText } from "./colors";
import { counterRadius, handlePositions, markBounds } from "./geometry";
import type {
  ArrowMark,
  BoxMark,
  ConcealMark,
  CounterMark,
  Doc,
  FreehandMark,
  LineMark,
  Rect,
  Mark,
  TextMark,
} from "./model";
import { unreachable } from "./unreachable";

const FONT_STACK =
  '-apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif';
const LINE_HEIGHT = 1.25;
const SPOTLIGHT_DIM = "rgba(0, 0, 0, 0.55)";

/** Anzeige und Export zeichnen mit derselben Logik. */
type Ctx = CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D;

export interface RenderOptions {
  image: CanvasImageSource;
  doc: Doc;
  /** Bildschirmpixel je Bildpixel; bestimmt die Größe der Griffe. */
  viewScale: number;
  /** Auswahlrahmen und Griffe zeichnen (nur in der Anzeige, nie im Export). */
  ui: {
    selection: Rect | null;
    selectedId: string | null;
    editingId: string | null;
  } | null;
  /** Wird mit den gemessenen Textgrößen gefüllt. */
  textBounds: Map<string, Rect>;
}

export const textFont = (mark: TextMark, scale: number): string =>
  `600 ${Math.round(mark.size * scale)}px ${FONT_STACK}`;

const withShadow = (ctx: Ctx, scale: number, draw: () => void) => {
  ctx.save();
  ctx.shadowColor = "rgba(0, 0, 0, 0.3)";
  ctx.shadowBlur = 3 * scale;
  ctx.shadowOffsetY = 1 * scale;
  draw();
  ctx.restore();
};

const drawArrow = (ctx: Ctx, mark: ArrowMark, scale: number) => {
  const dx = mark.to.x - mark.from.x;
  const dy = mark.to.y - mark.from.y;
  const length = Math.hypot(dx, dy);
  if (length < 1) {
    return;
  }
  const stroke = mark.size * scale;
  const head = Math.min(length * 0.6, stroke * 4 + 8 * scale);
  withShadow(ctx, scale, () => {
    ctx.translate(mark.from.x, mark.from.y);
    ctx.rotate(Math.atan2(dy, dx));
    ctx.fillStyle = mark.color;
    ctx.strokeStyle = mark.color;
    if (mark.arrowStyle === "solid") {
      const tail = stroke * 0.5;
      const shaft = stroke * 1.2;
      const wing = head * 0.5;
      ctx.beginPath();
      ctx.moveTo(0, -tail / 2);
      ctx.lineTo(length - head, -shaft / 2);
      ctx.lineTo(length - head, -wing);
      ctx.lineTo(length, 0);
      ctx.lineTo(length - head, wing);
      ctx.lineTo(length - head, shaft / 2);
      ctx.lineTo(0, tail / 2);
      ctx.closePath();
      ctx.fill();
    } else {
      ctx.lineWidth = stroke;
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(length, 0);
      ctx.moveTo(length - head * 0.8, -head * 0.5);
      ctx.lineTo(length, 0);
      ctx.lineTo(length - head * 0.8, head * 0.5);
      ctx.stroke();
    }
  });
};

const drawLine = (ctx: Ctx, mark: LineMark, scale: number) => {
  const stroke = mark.size * scale;
  withShadow(ctx, scale, () => {
    ctx.strokeStyle = mark.color;
    ctx.lineWidth = stroke;
    ctx.lineCap = "round";
    if (mark.dashed) {
      ctx.setLineDash([stroke * 2.5, stroke * 2]);
    }
    ctx.beginPath();
    ctx.moveTo(mark.from.x, mark.from.y);
    ctx.lineTo(mark.to.x, mark.to.y);
    ctx.stroke();
  });
};

const drawBox = (ctx: Ctx, mark: BoxMark, scale: number) => {
  const { x, y, width, height } = mark.rect;
  const stroke = mark.size * scale;
  withShadow(ctx, scale, () => {
    ctx.beginPath();
    if (mark.kind === "oval") {
      ctx.ellipse(
        x + width / 2,
        y + height / 2,
        width / 2,
        height / 2,
        0,
        0,
        Math.PI * 2
      );
    } else {
      ctx.roundRect(
        x,
        y,
        width,
        height,
        Math.min(3 * scale, width / 2, height / 2)
      );
    }
    if (mark.fill === "filled") {
      ctx.fillStyle = mark.color;
      ctx.fill();
    } else {
      ctx.strokeStyle = mark.color;
      ctx.lineWidth = stroke;
      ctx.lineJoin = "round";
      ctx.stroke();
    }
  });
};

const drawFreehand = (ctx: Ctx, mark: FreehandMark, scale: number) => {
  const [first, ...rest] = mark.points;
  if (!first) {
    return;
  }
  withShadow(ctx, scale, () => {
    ctx.strokeStyle = mark.color;
    ctx.lineWidth = mark.size * scale;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.beginPath();
    ctx.moveTo(first.x, first.y);
    let previous = first;
    for (const point of rest) {
      // Glättung: Kurve zum Mittelpunkt, Stützpunkt ist der letzte Punkt.
      ctx.quadraticCurveTo(
        previous.x,
        previous.y,
        (previous.x + point.x) / 2,
        (previous.y + point.y) / 2
      );
      previous = point;
    }
    ctx.lineTo(previous.x, previous.y);
    ctx.stroke();
  });
};

const drawCounter = (ctx: Ctx, mark: CounterMark, scale: number) => {
  const radius = counterRadius(mark.size, scale);
  withShadow(ctx, scale, () => {
    ctx.beginPath();
    ctx.arc(mark.center.x, mark.center.y, radius, 0, Math.PI * 2);
    ctx.fillStyle = mark.color;
    ctx.fill();
  });
  ctx.save();
  ctx.lineWidth = Math.max(1, radius * 0.12);
  ctx.strokeStyle = "#ffffff";
  ctx.beginPath();
  ctx.arc(mark.center.x, mark.center.y, radius, 0, Math.PI * 2);
  ctx.stroke();
  ctx.fillStyle = contrastText(mark.color);
  ctx.font = `700 ${Math.round(radius * 1.1)}px ${FONT_STACK}`;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    String(mark.value),
    mark.center.x,
    mark.center.y + radius * 0.05
  );
  ctx.restore();
};

/** Zeichnet Text und liefert seine Begrenzung. */
const drawText = (ctx: Ctx, mark: TextMark, scale: number): Rect => {
  const lines = mark.text.split("\n");
  const fontSize = Math.round(mark.size * scale);
  const lineHeight = fontSize * LINE_HEIGHT;
  ctx.save();
  ctx.font = textFont(mark, scale);
  ctx.textBaseline = "top";
  const width = Math.max(
    ...lines.map((line) => ctx.measureText(line).width),
    fontSize * 0.5
  );
  const height = lines.length * lineHeight;
  const padding = mark.textStyle === "label" ? fontSize * 0.35 : 0;
  const bounds = {
    height: height + padding * 2,
    width: width + padding * 2,
    x: mark.origin.x,
    y: mark.origin.y,
  };
  if (mark.textStyle === "label") {
    withShadow(ctx, scale, () => {
      ctx.fillStyle = mark.color;
      ctx.beginPath();
      ctx.roundRect(
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        fontSize * 0.25
      );
      ctx.fill();
    });
    ctx.fillStyle = contrastText(mark.color);
  } else {
    ctx.lineJoin = "round";
    ctx.lineWidth = Math.max(2, fontSize * 0.16);
    ctx.strokeStyle =
      contrastText(mark.color) === "#111111" ? "rgba(0,0,0,0.55)" : "#ffffff";
    for (const [index, line] of lines.entries()) {
      ctx.strokeText(line, bounds.x, bounds.y + index * lineHeight);
    }
    ctx.fillStyle = mark.color;
  }
  for (const [index, line] of lines.entries()) {
    ctx.fillText(
      line,
      bounds.x + padding,
      bounds.y + padding + index * lineHeight
    );
  }
  ctx.restore();
  return bounds;
};

let scratch: OffscreenCanvas | null = null;

const drawConceal = (
  ctx: Ctx,
  mark: ConcealMark,
  image: CanvasImageSource,
  scale: number
) => {
  const { x, y, width, height } = mark.rect;
  if (width < 1 || height < 1) {
    return;
  }
  scratch ??= new OffscreenCanvas(1, 1);
  const block = (mark.mode === "pixelate" ? 8 : 6) * scale;
  const smallWidth = Math.max(1, Math.round(width / block));
  const smallHeight = Math.max(1, Math.round(height / block));
  scratch.width = smallWidth;
  scratch.height = smallHeight;
  const small = scratch.getContext("2d");
  if (!small) {
    return;
  }
  small.imageSmoothingEnabled = true;
  small.drawImage(image, x, y, width, height, 0, 0, smallWidth, smallHeight);
  ctx.save();
  ctx.beginPath();
  ctx.rect(x, y, width, height);
  ctx.clip();
  ctx.imageSmoothingEnabled = mark.mode === "blur";
  ctx.imageSmoothingQuality = "high";
  ctx.drawImage(scratch, 0, 0, smallWidth, smallHeight, x, y, width, height);
  ctx.restore();
};

const drawSpotlights = (ctx: Ctx, doc: Doc) => {
  const spots = doc.marks.filter((mark) => mark.kind === "spotlight");
  if (spots.length === 0) {
    return;
  }
  ctx.save();
  ctx.beginPath();
  ctx.rect(0, 0, doc.width, doc.height);
  for (const spot of spots) {
    const { x, y, width, height } = spot.rect;
    ctx.roundRect(x, y, width, height, 4 * doc.scale);
  }
  ctx.fillStyle = SPOTLIGHT_DIM;
  ctx.fill("evenodd");
  ctx.restore();
};

const drawUi = (
  ctx: Ctx,
  options: RenderOptions,
  ui: NonNullable<RenderOptions["ui"]>
) => {
  const { doc, viewScale } = options;
  const px = 1 / viewScale;
  if (ui.selection && ui.selection.width > 0 && ui.selection.height > 0) {
    const { x, y, width, height } = ui.selection;
    ctx.save();
    ctx.beginPath();
    ctx.rect(doc.crop.x, doc.crop.y, doc.crop.width, doc.crop.height);
    ctx.rect(x, y, width, height);
    ctx.fillStyle = "rgba(0, 0, 0, 0.35)";
    ctx.fill("evenodd");
    ctx.lineWidth = px;
    ctx.strokeStyle = "#ffffff";
    ctx.setLineDash([5 * px, 4 * px]);
    ctx.strokeRect(x, y, width, height);
    ctx.restore();
  }
  const selected = doc.marks.find((mark) => mark.id === ui.selectedId);
  if (!selected) {
    return;
  }
  const bounds = markBounds(
    selected,
    (mark) => options.textBounds.get(mark.id),
    doc.scale
  );
  const handles = handlePositions(selected);
  ctx.save();
  ctx.lineWidth = px;
  if (handles.length === 0 || selected.kind === "highlight") {
    ctx.strokeStyle = "#3a86ff";
    ctx.setLineDash([4 * px, 3 * px]);
    const pad = 4 * px;
    ctx.strokeRect(
      bounds.x - pad,
      bounds.y - pad,
      bounds.width + pad * 2,
      bounds.height + pad * 2
    );
    ctx.setLineDash([]);
  }
  const size = 8 * px;
  for (const [, point] of handles) {
    ctx.fillStyle = "#ffffff";
    ctx.strokeStyle = "#3a86ff";
    ctx.beginPath();
    ctx.rect(point.x - size / 2, point.y - size / 2, size, size);
    ctx.fill();
    ctx.stroke();
  }
  ctx.restore();
};

const drawMark = (
  ctx: Ctx,
  mark: Mark,
  scale: number,
  textBounds: Map<string, Rect>
) => {
  switch (mark.kind) {
    case "arrow": {
      drawArrow(ctx, mark, scale);
      break;
    }
    case "line": {
      drawLine(ctx, mark, scale);
      break;
    }
    case "rect":
    case "oval": {
      drawBox(ctx, mark, scale);
      break;
    }
    case "freehand": {
      drawFreehand(ctx, mark, scale);
      break;
    }
    case "counter": {
      drawCounter(ctx, mark, scale);
      break;
    }
    case "text": {
      textBounds.set(mark.id, drawText(ctx, mark, scale));
      break;
    }
    case "highlight":
    case "conceal":
    case "spotlight": {
      // Werden vorab in eigenen Ebenen gezeichnet.
      break;
    }
    default: {
      unreachable(mark);
    }
  }
};

/** Zeichnet das Dokument, beschnitten auf den Zuschnitt, in den Kontext. */
export const renderDoc = (ctx: Ctx, options: RenderOptions): void => {
  const { doc, image } = options;
  const { crop, scale } = doc;
  ctx.save();
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  ctx.translate(-crop.x, -crop.y);
  ctx.drawImage(image, 0, 0);

  for (const mark of doc.marks) {
    if (mark.kind === "conceal") {
      drawConceal(ctx, mark, image, scale);
    }
  }
  for (const mark of doc.marks) {
    if (mark.kind === "highlight") {
      ctx.save();
      ctx.globalCompositeOperation = "multiply";
      ctx.fillStyle = mark.color;
      ctx.fillRect(mark.rect.x, mark.rect.y, mark.rect.width, mark.rect.height);
      ctx.restore();
    }
  }
  drawSpotlights(ctx, doc);
  for (const mark of doc.marks) {
    if (mark.id !== options.ui?.editingId) {
      drawMark(ctx, mark, scale, options.textBounds);
    }
  }
  if (options.ui) {
    drawUi(ctx, options, options.ui);
  }
  ctx.restore();
};
