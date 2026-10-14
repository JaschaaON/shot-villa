/**
 * Macht aus einer asynchronen Aktion einen Ereignis-Handler. Fehler landen in
 * der Konsole, damit kein Promise unbehandelt bleibt.
 */
export const fire =
  (task: () => Promise<void>): (() => void) =>
  () => {
    void (async () => {
      try {
        await task();
      } catch (error) {
        console.error(error);
      }
    })();
  };
