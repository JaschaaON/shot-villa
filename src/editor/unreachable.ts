/** Für vollständige `switch`-Anweisungen: der Compiler prüft, dass kein Fall fehlt. */
export const unreachable = (value: never): never => {
  throw new Error(`Unerwarteter Fall: ${JSON.stringify(value)}`);
};
