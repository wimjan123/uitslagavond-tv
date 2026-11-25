// Herstel na een handeling die iets weghaalt (Uit beeld, stemmen wissen, coalitie wissen): een paar
// seconden lang staat er in de multiviewer een knop die het terugzet. Liever dan een vraag vooraf,
// want onder tijdsdruk is één klik terug sneller dan elke keer bevestigen.
import { useSyncExternalStore } from "react";

/** Hoe lang de herstelknop blijft staan, in milliseconden. */
export const HERSTEL_DUUR = 10_000;

export interface Herstel {
  /** Wat er net gebeurde, zoals "Uit beeld: Zetelverdeling". */
  tekst: string;
  /** De tekst op de knop, zoals "Terug in beeld". */
  knop: string;
  /** Zet terug wat er net gebeurde. */
  doe(): void;
  /** Tot wanneer de knop blijft staan (Date.now()). */
  tot: number;
}

let huidig: Herstel | null = null;
let klok: ReturnType<typeof setTimeout> | undefined;
const luisteraars = new Set<() => void>();

function meld() {
  for (const l of luisteraars) l();
}

/** Biedt herstel aan voor de handeling die net gebeurde; een nieuw aanbod vervangt het vorige. */
export function biedHerstel(tekst: string, knop: string, doe: () => void, nu = Date.now()): void {
  globalThis.clearTimeout(klok);
  huidig = { tekst, knop, doe, tot: nu + HERSTEL_DUUR };
  klok = globalThis.setTimeout(vergeetHerstel, HERSTEL_DUUR);
  meld();
}

export function vergeetHerstel(): void {
  globalThis.clearTimeout(klok);
  huidig = null;
  meld();
}

/** Voert het herstel uit en haalt de knop weg. */
export function herstel(): void {
  const h = huidig;
  vergeetHerstel();
  h?.doe();
}

export function useHerstel(): Herstel | null {
  return useSyncExternalStore(
    (l) => {
      luisteraars.add(l);
      return () => luisteraars.delete(l);
    },
    () => huidig,
  );
}
