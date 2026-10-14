/** Läuft die App auf macOS? Sonst Windows. */
export const isMac = /Mac/u.test(navigator.userAgent);

const MODIFIERS = [
  { label: "Strg", symbols: ["⌃", "⌘"] },
  { label: "Alt", symbols: ["⌥"] },
  { label: "Umschalt", symbols: ["⇧"] },
] as const;

/**
 * Tastenkürzel in der Schreibweise der Plattform. Eingabe ist die
 * Mac-Schreibweise, z. B. `⇧⌘C`; unter Windows wird daraus `Strg+Umschalt+C`.
 */
export const keys = (macLabel: string, mac: boolean = isMac): string => {
  if (mac) {
    return macLabel;
  }
  const key = macLabel.replaceAll(/[⌃⌘⌥⇧]/gu, "");
  const modifiers = MODIFIERS.filter(({ symbols }) =>
    symbols.some((symbol) => macLabel.includes(symbol))
  ).map(({ label }) => label);
  return [...modifiers, key].join("+");
};
