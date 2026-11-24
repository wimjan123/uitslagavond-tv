// De invoer "volg de Voorzitter" (DESIGN.md, Stemming Input): elke knop maakt een nieuw
// Stemgedrag, dat het dashboard naar de server stuurt. De server rekent de uitkomst uit.
import type { Fractie } from "../typen/Fractie";
import type { Kamerlid } from "../typen/Kamerlid";
import type { Stem } from "../typen/Stem";
import type { Stemgedrag } from "../typen/Stemgedrag";
import type { Stemming } from "../typen/Stemming";

export const LEEG: Stemgedrag = { getikt: [], overige: null, algemene_stemmen: false, afwijkend: [], hoofdelijk: false };

export function andere(s: Stem): Stem {
  return s === "Voor" ? "Tegen" : "Voor";
}

/** Tikt een Fractie aan aan de kant `kant`. Staat hij daar al, dan gaat hij eraf; staat hij aan de andere kant, dan wisselt hij. */
export function tik(sg: Stemgedrag, fractie: string, kant: Stem): Stemgedrag {
  const basis = sg.algemene_stemmen ? { ...LEEG, hoofdelijk: sg.hoofdelijk } : sg;
  const i = basis.getikt.findIndex((g) => g.fractie === fractie);
  if (i < 0) return { ...basis, getikt: [...basis.getikt, { fractie, stem: kant }] };
  if (basis.getikt[i].stem === kant) return { ...basis, getikt: basis.getikt.filter((_, j) => j !== i) };
  return { ...basis, getikt: basis.getikt.map((g, j) => (j === i ? { fractie, stem: kant } : g)) };
}

/** "Overige tegen" als je voor tikt, "Overige voor" als je tegen tikt. Nog eens: weer weg. */
export function overige(sg: Stemgedrag, kant: Stem): Stemgedrag {
  const rest = andere(kant);
  return { ...sg, algemene_stemmen: false, overige: sg.overige === rest ? null : rest };
}

/** "Met algemene stemmen", of weer uit. Wist wat er getikt was. */
export function algemeen(sg: Stemgedrag): Stemgedrag {
  return { ...LEEG, hoofdelijk: sg.hoofdelijk, algemene_stemmen: !sg.algemene_stemmen };
}

export function wis(sg: Stemgedrag): Stemgedrag {
  return { ...LEEG, hoofdelijk: sg.hoofdelijk };
}

/** Zet de Afwijkende stem van een Kamerlid, of haalt hem weg met `null`. */
export function afwijkend(sg: Stemgedrag, kamerlid: string, stem: Stem | null): Stemgedrag {
  const rest = sg.afwijkend.filter((a) => a.kamerlid !== kamerlid);
  return { ...sg, algemene_stemmen: false, afwijkend: stem ? [...rest, { kamerlid, stem }] : rest };
}

/** De stem van een Fractie en of die van Overige of algemene stemmen komt (gelijk aan `Stemgedrag::stem_van`). */
export function stemVan(sg: Stemgedrag, fractie: string): { stem: Stem; getikt: boolean } | null {
  if (sg.algemene_stemmen) return { stem: "Voor", getikt: false };
  const g = sg.getikt.find((x) => x.fractie === fractie);
  if (g) return { stem: g.stem, getikt: true };
  return sg.overige ? { stem: sg.overige, getikt: false } : null;
}

export function zelfdeStemgedrag(a: Stemgedrag, b: Stemgedrag): boolean {
  const lijst = (x: Stemgedrag) =>
    JSON.stringify([x.getikt.map((g) => [g.fractie, g.stem]), x.overige, x.algemene_stemmen, x.afwijkend.map((a) => [a.kamerlid, a.stem]), x.hoofdelijk]);
  return lijst(a) === lijst(b);
}

/** Een stuk van de Voorzitter-zin. Met `stem` kleurt het dashboard het in de kleur van die kant. */
export interface Deel {
  tekst: string;
  stem?: Stem;
}

function opsomming(namen: Deel[]): Deel[] {
  return namen.flatMap((n, i) => (i === 0 ? [n] : [{ tekst: i === namen.length - 1 ? " en " : ", " }, n]));
}

/**
 * De invoer in de woorden van de Voorzitter, zoals "de leden van de fracties van SP en PvdD voor
 * en de overige fracties ertegen". `namen` geeft per Kamerlid de naam met Fractie.
 */
export function voorzitterZin(sg: Stemgedrag, fracties: Fractie[], namen: Record<string, string>): Deel[] {
  if (sg.algemene_stemmen) return [{ tekst: "met algemene stemmen", stem: "Voor" }, { tekst: " aangenomen" }];
  const afk = (id: string) => fracties.find((f) => f.id === id)?.afkorting ?? id;
  const woord = (s: Stem): Deel => ({ tekst: s === "Voor" ? "voor" : "ertegen", stem: s });
  const zinnen: Deel[][] = [];
  for (const kant of [...new Set(sg.getikt.map((g) => g.stem))]) {
    const van = sg.getikt.filter((g) => g.stem === kant).map((g): Deel => ({ tekst: afk(g.fractie), stem: kant }));
    zinnen.push([{ tekst: van.length === 1 ? "de leden van de fractie van " : "de leden van de fracties van " }, ...opsomming(van), { tekst: " " }, woord(kant)]);
  }
  if (sg.overige) {
    const rest: Deel = { tekst: sg.overige === "Voor" ? "ervoor" : "ertegen", stem: sg.overige };
    zinnen.push([{ tekst: zinnen.length ? "de overige fracties " : "alle fracties " }, rest]);
  }
  const delen = zinnen.flatMap((z, i) => (i === 0 ? z : [{ tekst: " en " }, ...z]));
  sg.afwijkend.forEach((a, i) => {
    delen.push({ tekst: i === 0 ? (delen.length ? ", met uitzondering van " : "") + "het lid " : " en het lid " });
    delen.push({ tekst: namen[a.kamerlid] ?? a.kamerlid });
    delen.push({ tekst: ", dat " }, { tekst: a.stem === "Voor" ? "voor" : "tegen", stem: a.stem }, { tekst: " stemde" });
  });
  return delen;
}

/** "Dieke van Groningen", zoals `Kamerlid::naam`. */
export function kamerlidNaam(k: Kamerlid): string {
  const voor = k.roepnaam.trim() || k.initialen.trim();
  return [voor, k.tussenvoegsel?.trim() ?? "", k.achternaam.trim()].filter(Boolean).join(" ");
}

/** De volgende Stemming na `id` die nog niet is vastgelegd; vooraan verder als er na `id` geen is. */
export function volgendeOpen(lijst: Stemming[], id: string): string | null {
  const i = lijst.findIndex((s) => s.punt.id === id);
  const rij = [...lijst.slice(i + 1), ...lijst.slice(0, Math.max(i, 0))];
  return rij.find((s) => !s.vastgelegd)?.punt.id ?? null;
}
