import type { UpdateProgress } from "../api";

const BYTES_PER_MEGABYTE = 1_000_000;

const megabytes = (bytes: number): string =>
  `${(bytes / BYTES_PER_MEGABYTE).toLocaleString("de-DE", {
    maximumFractionDigits: 1,
    minimumFractionDigits: 1,
  })} MB`;

/** Text zum Fortschritt des Downloads, z. B. „Lade herunter … 42 %“. */
export const progressLabel = ({
  downloaded,
  total,
}: UpdateProgress): string => {
  if (total === null || total === 0) {
    return `Lade herunter … ${megabytes(downloaded)}`;
  }
  if (downloaded >= total) {
    return "Installiere …";
  }
  const percent = Math.floor((downloaded / total) * 100);
  return `Lade herunter … ${percent} % von ${megabytes(total)}`;
};
