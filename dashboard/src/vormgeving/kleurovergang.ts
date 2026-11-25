// De Kleurovergang van de Stoelen: Golf of Vervagen, met een duur (crates/gedeeld/src/sjabloon.rs).
import type { Kleurovergang } from "../typen/Kleurovergang";
import type { Overgangsvorm } from "../typen/Overgangsvorm";
import { MAX_KLEUROVERGANG_MS, MIN_KLEUROVERGANG_MS, STANDAARD_KLEUROVERGANG_MS } from "../typen/standaardStijl.ts";

export const VORMEN: { vorm: Overgangsvorm; naam: string; uitleg: string }[] = [
  { vorm: "Golf", naam: "Golf", uitleg: "Van links naar rechts in beeld" },
  { vorm: "Vervagen", naam: "Vervagen", uitleg: "Alle Stoelen tegelijk" },
];

/** Een andere vorm krijgt de standaardduur van die vorm. */
export function metVorm(vorm: Overgangsvorm): Kleurovergang {
  return { vorm, duur_ms: STANDAARD_KLEUROVERGANG_MS[vorm] };
}

/** Waarom deze Kleurovergang niet kan, of `null` (gelijk aan `Kleurovergang::kan`). */
export function foutKleurovergang(k: Kleurovergang): string | null {
  if (!Number.isInteger(k.duur_ms) || k.duur_ms < MIN_KLEUROVERGANG_MS || k.duur_ms > MAX_KLEUROVERGANG_MS)
    return `Kies een Kleurovergang tussen ${MIN_KLEUROVERGANG_MS / 1000} en ${MAX_KLEUROVERGANG_MS / 1000} seconden`;
  return null;
}
