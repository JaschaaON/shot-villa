import type { Recognized } from "../api";
import { unreachable } from "./unreachable";

const PREVIEW_CHARS = 60;

/** Kurze Rückmeldung nach der Texterkennung, mit Anfang des Textes. */
export const recognitionMessage = ({ found, text }: Recognized): string => {
  const line = text.replaceAll(/\s+/gu, " ").trim();
  const preview =
    line.length > PREVIEW_CHARS ? `${line.slice(0, PREVIEW_CHARS)}…` : line;
  switch (found) {
    case "text": {
      return `Text kopiert: ${preview}`;
    }
    case "code": {
      return `QR-Code kopiert: ${preview}`;
    }
    case "nothing": {
      return "Kein Text erkannt";
    }
    default: {
      return unreachable(found);
    }
  }
};
