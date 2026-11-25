// De Coalitiebouwer: de teller naar 76 en welke coalitie er klaarstaat of in beeld is.
import type { Graphic } from "../typen/Graphic";
import type { Partij } from "../typen/Partij";
import type { Stand } from "../typen/Stand";

/** Gelijk aan `uitslagindeling::HUIDIGE_KAMER`: de Zetels van de Kamer uit de open data. */
export const HUIDIGE_KAMER = "huidige-kamer";
/** Gelijk aan `coalitie::MEERDERHEID`. */
export const MEERDERHEID = 76;

export type Coalitiegraphic = Extract<Graphic, { sjabloon: "Coalitie" | "Coalitiebalk" }>;

/** De Zetels per Partij in een set; bij de huidige Kamer de Zetels van elke Fractie bij haar Partij
 * (gelijk aan `Zetelstand::van_kamer`). */
export function zetelsVan(s: Stand, set: string): Record<string, number> {
  if (set !== HUIDIGE_KAMER) return { ...(s.cijfers.sets.find((x) => x.id === set)?.zetels ?? {}) };
  const uit: Record<string, number> = {};
  for (const f of s.kamer.fracties) {
    const p = s.kamer.partij_van[f.id];
    if (p) uit[p] = (uit[p] ?? 0) + f.zetels;
  }
  return uit;
}

/** De Zetels van de gekozen Partijen samen; dubbel telt één keer (gelijk aan `coalitie::Telling::van`). */
export function telling(zetels: Record<string, number>, partijen: string[]): number {
  return [...new Set(partijen)].reduce((som, p) => som + (zetels[p] ?? 0), 0);
}

/** De regel onder de teller in het dashboard (DESIGN.md, Coalitiebouwer Arc). */
export function regel(zetels: number): string {
  const nog = MEERDERHEID - zetels;
  return nog <= 0 ? "meerderheid" : `nog ${nog} ${nog === 1 ? "Zetel" : "Zetels"} tot een meerderheid`;
}

/** Een Partij aantikken: erbij als ze er niet in zat, anders eraf. */
export function wissel(partijen: string[], sleutel: string): string[] {
  return partijen.includes(sleutel) ? partijen.filter((p) => p !== sleutel) : [...partijen, sleutel];
}

function alsCoalitie(g: Graphic | undefined): Coalitiegraphic | null {
  return g && (g.sjabloon === "Coalitie" || g.sjabloon === "Coalitiebalk") ? g : null;
}

/** De coalitie die de Coalitiebouwer bijwerkt: die in de Ether, anders die klaarstaat. */
export function coalitieInBeeld(s: Stand): Coalitiegraphic | null {
  return alsCoalitie(s.ether?.graphic) ?? alsCoalitie(s.klaar?.graphic);
}

/** Wat de Coalitiebouwer toont, op het dashboard en op de tablet. */
export interface Bouwer {
  /** De coalitie die klaarstaat of in de Ether staat; elke aangetikte Partij werkt die direct bij. */
  inBeeld: Coalitiegraphic | null;
  /** De gekozen Partijen: die van de coalitie in beeld, anders de eigen keuze. */
  partijen: string[];
  zetels: Record<string, number>;
  /** De Partijen met Zetels, in de volgorde van de Uitslagindeling. */
  volgorde: Partij[];
  totaal: number;
}

/** De coalitie in beeld met de kopie van haar cijfers; zonder coalitie in beeld de eigen keuze met
 * de cijfers van `set`. */
export function bouwer(s: Stand, set: string, eigen: string[]): Bouwer {
  const inBeeld = coalitieInBeeld(s);
  const partijen = inBeeld?.partijen ?? eigen;
  const zetels = inBeeld?.cijfers?.set.zetels ?? zetelsVan(s, set);
  const volgorde = s.uitslagindeling.partijen
    .map((k) => s.cijfers.partijen.find((p) => p.sleutel === k))
    .filter((p): p is Partij => !!p && (zetels[p.sleutel] ?? 0) > 0);
  return { inBeeld, partijen, zetels, volgorde, totaal: telling(zetels, partijen) };
}

/** De coalitie in woorden: de korte namen op volgorde, of de set als er nog niemand in zit. */
export function coalitieTekst(g: Coalitiegraphic): string {
  const partijen = (g.cijfers?.partijen ?? []).filter((p) => g.partijen.includes(p.sleutel));
  const korte = partijen.length ? partijen.map((p) => p.kort) : g.partijen;
  if (korte.length === 0) return `${g.cijfers?.set.label ?? g.set}: nog geen Partijen`;
  return korte.length === 1 ? korte[0] : `${korte.slice(0, -1).join(", ")} en ${korte[korte.length - 1]}`;
}

/** Plekken op een halve cirkel met deze straal, van links naar rechts, rond (0, 0); y omhoog is min. */
export function boog(n: number, straal: number): { x: number; y: number }[] {
  return Array.from({ length: n }, (_, i) => {
    const hoek = n === 1 ? Math.PI / 2 : Math.PI - (Math.PI * i) / (n - 1);
    // geen -0 of 1e-14 aan de uiteinden
    const rond = (v: number) => (Math.abs(v) < 1e-9 ? 0 : v);
    return { x: rond(straal * Math.cos(hoek)), y: rond(-straal * Math.sin(hoek)) };
  });
}
