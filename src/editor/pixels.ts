import type { Rgb } from "./colors";

/** Liest Pixelfarben eines Bildes; die Daten werden einmal erzeugt. */
export class PixelReader {
  readonly #data: ImageData | null;

  constructor(image: HTMLImageElement) {
    const canvas = document.createElement("canvas");
    canvas.width = image.naturalWidth;
    canvas.height = image.naturalHeight;
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    ctx?.drawImage(image, 0, 0);
    this.#data = ctx?.getImageData(0, 0, canvas.width, canvas.height) ?? null;
  }

  at(x: number, y: number): Rgb | null {
    const data = this.#data;
    if (!data) {
      return null;
    }
    const px = Math.floor(x);
    const py = Math.floor(y);
    if (px < 0 || py < 0 || px >= data.width || py >= data.height) {
      return null;
    }
    const index = (py * data.width + px) * 4;
    return {
      b: data.data[index + 2] ?? 0,
      g: data.data[index + 1] ?? 0,
      r: data.data[index] ?? 0,
    };
  }
}
