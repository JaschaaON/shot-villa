import { describe, expect, it } from "vitest";

import { progressLabel } from "./update-status";

describe("Fortschritt des Updates", () => {
  it("zeigt Prozent, wenn die Größe bekannt ist", () => {
    expect(progressLabel({ downloaded: 2_100_000, total: 8_400_000 })).toBe(
      "Lade herunter … 25 % von 8,4 MB"
    );
  });

  it("zeigt nur die Menge, wenn der Server keine Größe nennt", () => {
    expect(progressLabel({ downloaded: 1_500_000, total: null })).toBe(
      "Lade herunter … 1,5 MB"
    );
    expect(progressLabel({ downloaded: 1_500_000, total: 0 })).toBe(
      "Lade herunter … 1,5 MB"
    );
  });

  it("wechselt nach dem Download zum Installieren", () => {
    expect(progressLabel({ downloaded: 8_400_000, total: 8_400_000 })).toBe(
      "Installiere …"
    );
  });
});
