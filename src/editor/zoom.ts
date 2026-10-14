export interface Size {
  width: number;
  height: number;
}

/** Größe des Bildes in Punkten, also so, wie es auf dem Bildschirm aussah. */
export const sizeInPoints = (pixels: Size, scale: number): Size => ({
  height: pixels.height / scale,
  width: pixels.width / scale,
});

/**
 * Zoomstufe, bei der das Bild vollständig in den sichtbaren Bereich passt.
 * Vergrößert nie über 100 %.
 */
export const fitZoom = (image: Size, viewport: Size): number => {
  if (image.width <= 0 || image.height <= 0) {
    return 1;
  }
  return Math.min(
    1,
    viewport.width / image.width,
    viewport.height / image.height
  );
};
