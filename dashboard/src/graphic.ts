// Namen en tijden van Graphics, zoals het dashboard ze toont.
import type { BewerktSjabloon } from "./typen/BewerktSjabloon";
import type { Graphic } from "./typen/Graphic";
import type { Stand } from "./typen/Stand";
import type { Stemmingsuitslag } from "./typen/Stemmingsuitslag";
import { coalitieTekst } from "./coalitie/coalitie";

/** De naam van het Sjabloon. Een eigen Sjabloon heet zoals in `sjablonen` (de bewerkte Sjablonen van de Stand). */
export function sjabloonNaam(g: Graphic, sjablonen: readonly BewerktSjabloon[] = []): string {
  switch (g.sjabloon) {
    case "Eigen":
      return sjablonen.find((b) => b.sjabloon.id === g.id)?.sjabloon.naam ?? "Eigen Sjabloon";
    case "Titelkaart":
      return "Titelkaart";
    case "UitslagStemming":
      return "Uitslag van een Stemming";
    case "HoofdelijkeStemming":
      return "Hoofdelijke stemming";
    case "FractiesVoorEnTegen":
      return "Fracties voor en tegen";
    case "Zetelverdeling":
      return "Zetelverdeling in de Zaal";
    case "Persoonskaart":
      return "Persoonskaart";
    case "Zetelbalk":
      return "Zetelbalk";
    case "TweeSets":
      return "Twee sets naast elkaar";
    case "UitslagPerPartij":
      return "Uitslag per Partij";
    case "WinstEnVerlies":
      return "Winst en verlies";
    case "PartijUitgelicht":
      return "Partij uitgelicht";
    case "Coalitie":
      return "Coalitie in de Zaal";
    case "Coalitiebalk":
      return "Coalitiebalk";
  }
}

/** Het `id` van het Sjabloon van de Graphic (gelijk aan `Graphic::sjabloon_id`). */
export function sjabloonId(g: Graphic): string {
  switch (g.sjabloon) {
    case "Eigen":
      return g.id;
    case "UitslagStemming":
      return "uitslag-stemming";
    case "HoofdelijkeStemming":
      return "hoofdelijke-stemming";
    case "FractiesVoorEnTegen":
      return "fracties-voor-en-tegen";
    case "TweeSets":
      return "twee-sets";
    case "UitslagPerPartij":
      return "uitslag-per-partij";
    case "WinstEnVerlies":
      return "winst-en-verlies";
    case "PartijUitgelicht":
      return "partij-uitgelicht";
    default:
      return g.sjabloon.toLowerCase();
  }
}

/** De drie Sjablonen voor Stemmingen: dezelfde uitslag, een andere vorm. */
export const STEMMINGSJABLONEN = ["UitslagStemming", "FractiesVoorEnTegen", "HoofdelijkeStemming"] as const;
export type Stemmingsjabloon = (typeof STEMMINGSJABLONEN)[number];

/** Dezelfde uitslag in een ander Sjabloon voor Stemmingen. */
export function alsStemmingsjabloon(u: Stemmingsuitslag, sjabloon: Stemmingsjabloon): Graphic {
  return { sjabloon, uitslag: u };
}

/** Het id van de Stemming waar de Graphic over gaat (gelijk aan `Graphic::stemming`). */
export function stemmingVan(g: Graphic | undefined): string | null {
  return g && "uitslag" in g ? g.uitslag.stemming : null;
}

/** De letter van het standaard Standpunt van het Sjabloon (gelijk aan `Graphic::standaard_standpunt`). */
export function standaardStandpunt(g: Graphic): string {
  switch (g.sjabloon) {
    case "Titelkaart":
    case "UitslagStemming":
    case "Zetelverdeling":
    case "Zetelbalk":
    case "TweeSets":
    case "UitslagPerPartij":
    case "WinstEnVerlies":
    case "PartijUitgelicht":
    case "Coalitie":
    case "Coalitiebalk":
    case "Eigen":
      return "A";
    case "Persoonskaart": {
      const stoel = g.gegevens?.stoel;
      return stoel ? `stoel-${stoel.vak}-${stoel.rij}-${stoel.nummer}` : "S";
    }
    case "HoofdelijkeStemming":
    case "FractiesVoorEnTegen":
      return "G";
  }
}

/** Wat de Graphic toont, kort, voor naast de naam van het Sjabloon. */
export function graphicTekst(g: Graphic): string {
  switch (g.sjabloon) {
    case "Titelkaart":
      return g.kop;
    case "UitslagStemming":
    case "HoofdelijkeStemming":
    case "FractiesVoorEnTegen":
      // Voorlopig: de Kamerleden staan op een Stoel die het systeem op kieslijstvolgorde koos.
      return g.uitslag.voorlopig ? `${g.uitslag.titel} (voorlopig)` : g.uitslag.titel;
    case "Zetelverdeling":
    case "Zetelbalk":
    case "UitslagPerPartij":
      return g.cijfers?.set.label ?? g.set;
    case "Persoonskaart":
      return g.gegevens?.naam ?? g.wie.id;
    case "TweeSets":
      return g.cijfers ? `${g.cijfers.vorige?.label ?? g.set} en ${g.cijfers.set.label}` : `${g.set} en ${g.tweede}`;
    case "WinstEnVerlies":
      return g.cijfers ? `${g.cijfers.set.label} tegenover ${g.cijfers.vorige?.label ?? ""}` : g.set;
    case "PartijUitgelicht": {
      const p = g.cijfers?.partijen.find((q) => q.sleutel === g.partij);
      return `${p?.kort ?? g.partij} in ${g.cijfers?.set.label ?? g.set}`;
    }
    case "Coalitie":
    case "Coalitiebalk":
      return coalitieTekst(g);
    case "Eigen":
      return Object.values(g.velden).find((v) => v?.trim()) ?? "";
  }
}

/** Of er nieuwere cijfers zijn dan die waarmee de Graphic is ingevuld (gelijk aan `Stand::nieuwere_cijfers`). */
export function nieuwereCijfers(s: Stand, g: Graphic): boolean {
  const c = "cijfers" in g ? g.cijfers : undefined;
  if (!c) return false;
  const sets = c.vorige ? [c.vorige, c.set] : [c.set];
  return sets.some((set) => s.cijfers.sets.some((x) => x.id === set.id && x.versie > set.versie));
}

/** De plaats van een Partij van links naar rechts in de Uitslagindeling (gelijk aan `Volgorde::plaats`). */
export function plaatsInUitslagindeling(s: Stand, sleutel: string | undefined): number {
  const i = sleutel === undefined ? -1 : s.uitslagindeling.partijen.indexOf(sleutel);
  return i < 0 ? s.uitslagindeling.partijen.length : i;
}

/** Een tijd als "20.58", zoals de Regie hem leest. */
export function uurMinuut(ms: number): string {
  const d = new Date(ms);
  return `${String(d.getHours()).padStart(2, "0")}.${String(d.getMinutes()).padStart(2, "0")}`;
}
