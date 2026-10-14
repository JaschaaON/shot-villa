import type { Doc, Rect } from "./model";
import { renderDoc } from "./render";

const surface = (width: number, height: number) => {
  const canvas = new OffscreenCanvas(
    Math.max(1, Math.round(width)),
    Math.max(1, Math.round(height))
  );
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Zeichenfläche nicht verfügbar");
  }
  return { canvas, ctx };
};

const toPng = async (canvas: OffscreenCanvas): Promise<Uint8Array> => {
  const blob = await canvas.convertToBlob({ type: "image/png" });
  return new Uint8Array(await blob.arrayBuffer());
};

/** Rendert das Dokument (Zuschnitt samt Markierungen) als PNG in voller Auflösung. */
export const renderPng = async (
  image: CanvasImageSource,
  doc: Doc
): Promise<Uint8Array> => {
  const { canvas, ctx } = surface(doc.crop.width, doc.crop.height);
  renderDoc(ctx, { doc, image, textBounds: new Map(), ui: null, viewScale: 1 });
  return await toPng(canvas);
};

/** Ausschnitt des unbearbeiteten Bildes als PNG, z. B. für die Texterkennung. */
export const cropPng = async (
  image: CanvasImageSource,
  rect: Rect
): Promise<Uint8Array> => {
  const { canvas, ctx } = surface(rect.width, rect.height);
  ctx.drawImage(
    image,
    rect.x,
    rect.y,
    rect.width,
    rect.height,
    0,
    0,
    rect.width,
    rect.height
  );
  return await toPng(canvas);
};
