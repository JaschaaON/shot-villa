/** Farbpalette des Editors. */
export const PALETTE = [
  "#e5383b",
  "#f77f00",
  "#fcbf49",
  "#2a9d8f",
  "#3a86ff",
  "#8338ec",
  "#111111",
  "#ffffff",
] as const;

export const HIGHLIGHT_PALETTE = [
  "#ffe74c",
  "#a7f3a1",
  "#9ad7ff",
  "#ffb3d9",
] as const;

export type ColorFormat = "hex" | "rgb" | "cssRgba" | "oklch";

export interface Rgb {
  r: number;
  g: number;
  b: number;
}

const hexByte = (value: number) =>
  value.toString(16).padStart(2, "0").toUpperCase();

const toLinear = (channel: number) => {
  const c = channel / 255;
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
};

export interface Oklch {
  /** Helligkeit 0–1. */
  l: number;
  /** Buntheit. */
  c: number;
  /** Farbton in Grad. */
  h: number;
}

/** sRGB → OKLCH (Björn Ottosson, OKLab). */
export const toOklch = ({ r, g, b }: Rgb): Oklch => {
  const lr = toLinear(r);
  const lg = toLinear(g);
  const lb = toLinear(b);
  const l = Math.cbrt(
    0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb
  );
  const m = Math.cbrt(
    0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb
  );
  const s = Math.cbrt(
    0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb
  );
  const lightness = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const bb = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  const chroma = Math.hypot(a, bb);
  const hue = (Math.atan2(bb, a) * 180) / Math.PI;
  return { c: chroma, h: hue < 0 ? hue + 360 : hue, l: lightness };
};

export const formatColor = (color: Rgb, format: ColorFormat): string => {
  const { r, g, b } = color;
  switch (format) {
    case "hex": {
      return `#${hexByte(r)}${hexByte(g)}${hexByte(b)}`;
    }
    case "rgb": {
      return `${r}, ${g}, ${b}`;
    }
    case "cssRgba": {
      return `rgba(${r}, ${g}, ${b}, 1)`;
    }
    case "oklch": {
      const { l, c, h } = toOklch(color);
      const hue = c < 0.0001 ? 0 : h;
      return `oklch(${(l * 100).toFixed(1)}% ${c.toFixed(3)} ${hue.toFixed(1)})`;
    }
    default: {
      return `#${hexByte(r)}${hexByte(g)}${hexByte(b)}`;
    }
  }
};

export const COLOR_FORMATS: readonly ColorFormat[] = [
  "hex",
  "rgb",
  "cssRgba",
  "oklch",
];

export const nextColorFormat = (format: ColorFormat): ColorFormat =>
  COLOR_FORMATS[(COLOR_FORMATS.indexOf(format) + 1) % COLOR_FORMATS.length] ??
  "hex";

/** Gut lesbare Schriftfarbe auf dem Hintergrund `hex`. */
export const contrastText = (hex: string): string => {
  const value = Number.parseInt(hex.slice(1), 16);
  const r = Math.trunc(value / 65_536) % 256;
  const g = Math.trunc(value / 256) % 256;
  const b = value % 256;
  const luminance =
    0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
  return luminance > 0.4 ? "#111111" : "#ffffff";
};
