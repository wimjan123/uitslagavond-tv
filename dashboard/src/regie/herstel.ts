// Herstel na een handeling die iets weghaalt (Uit beeld, stemmen wissen, coalitie wissen): een tijd
// lang staat er in de multiviewer een knop die het terugzet (ook met Ctrl+Z). Liever dan een vraag
// vooraf, want onder tijdsdruk is één klik terug sneller dan elke keer bevestigen.
//
// Een herstel geldt alleen zolang de Stand nog is zoals vlak na de handeling: heeft iemand (ook een
// andere Werkplek) intussen iets anders getoond, klaargezet of ingevoerd, dan verdwijnt de knop, zodat
// herstel nooit over een latere keuze heen schrijft.
import { useSyncExternalStore } from "react";
import type { Stand } from "../typen/Stand";
import type { Regie } from "./regie";

/** Hoe lang de herstelknop hoogstens blijft staan, in milliseconden. */
export const HERSTEL_DUUR = 20_000;

export interface Herstel {
  /** Wat er net gebeurde, zoals "Uit beeld". */
  tekst: string;
  /** Wat er terugkomt, zoals "Zetelverdeling · exitpoll". */
  wat: string;
  /** De tekst op de knop, zoals "Terug in beeld". */
  knop: string;
  /** Of het herstel nog past bij deze Stand. */
  geldig(s: Stand): boolean;
  /** Zet terug wat er net gebeurde, met de Regie van nu. */
  doe(regie: Regie): void;
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
export function biedHerstel(h: Omit<Herstel, "tot">, nu = Date.now()): void {
  globalThis.clearTimeout(klok);
  huidig = { ...h, tot: nu + HERSTEL_DUUR };
  klok = globalThis.setTimeout(vergeetHerstel, HERSTEL_DUUR);
  meld();
}

export function vergeetHerstel(): void {
  globalThis.clearTimeout(klok);
  huidig = null;
  meld();
}

/** Het herstel dat nu geldt voor deze Stand, of `null`. */
export function geldigHerstel(h: Herstel | null, s: Stand | undefined): Herstel | null {
  return h && s && h.geldig(s) ? h : null;
}

/** Voert het herstel uit als het nog geldt, en haalt de knop weg. Geeft terug of er iets gebeurde. */
export function herstel(regie: Regie): boolean {
  const h = geldigHerstel(huidig, regie.stand?.stand);
  vergeetHerstel();
  if (!h) return false;
  h.doe(regie);
  return true;
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
