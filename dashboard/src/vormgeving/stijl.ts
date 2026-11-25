// De Stijl in de vormgevingstab: wat de Regie mag opslaan en hoe een lettertype heet. De grenzen
// en de standaard-Stijl komen uit crates/gedeeld (typen/standaardStijl.ts).
import { INGEBOUWDE_LETTERS, MAX_LOGOTEKST, MAX_SNELHEID, MIN_SNELHEID } from "../typen/standaardStijl.ts";
import type { Stijl } from "../typen/Stijl";
import { foutKleurovergang } from "./kleurovergang.ts";
import type { Stijlkleuren } from "../typen/Stijlkleuren";

/** De vijf kleuren, in de volgorde van docs/vormgeving.md, met waarvoor ze zijn. */
export const KLEUREN: { sleutel: keyof Stijlkleuren; naam: string; waarvoor: string }[] = [
  { sleutel: "hoofd", naam: "Hoofd", waarvoor: "Koppen en namen, korte namen van Partijen" },
  { sleutel: "accent", naam: "Accent", waarvoor: "Alleen als markering: het streepje voor een kicker, de lijn bij 76" },
  { sleutel: "tekst", naam: "Tekst", waarvoor: "Regels, getallen, partijnamen in grafieken" },
  { sleutel: "vlak", naam: "Vlak", waarvoor: "De naambalk en de onderkant van elk paneel" },
  { sleutel: "achtergrond", naam: "Achtergrond", waarvoor: "De bovenkant van het verloop in panelen, de waas over het beeld" },
];

/** De drie lettertypes, met waarvoor ze zijn. */
export const LETTERS: { sleutel: keyof Stijl["lettertypes"]; naam: string; waarvoor: string }[] = [
  { sleutel: "kop", naam: "Kop", waarvoor: "Koppen en namen" },
  { sleutel: "tekst", naam: "Tekst", waarvoor: "Partijnamen en regels" },
  { sleutel: "cijfers", naam: "Cijfers", waarvoor: "Getallen en de kicker" },
];

const HEX = /^#[0-9a-f]{6}$/i;

/** Waarom deze Stijl niet opgeslagen kan worden, of `null` (gelijk aan `Stijl::controleer`). */
export function fout(s: Stijl): string | null {
  const kleur = KLEUREN.find((k) => !HEX.test(s.kleuren[k.sleutel]));
  if (kleur) return `${kleur.naam} is geen kleur van de vorm #rrggbb`;
  if (!Number.isInteger(s.snelheid) || s.snelheid < MIN_SNELHEID || s.snelheid > MAX_SNELHEID)
    return `Kies een snelheid tussen ${MIN_SNELHEID} en ${MAX_SNELHEID} procent`;
  if ([...s.programmalogo.tekst].length > MAX_LOGOTEKST) return `De tekst van het programmalogo is langer dan ${MAX_LOGOTEKST} tekens`;
  if (!Number.isInteger(s.kleurkracht) || s.kleurkracht < 0 || s.kleurkracht > 100) return "Kies een Kleurkracht tussen 0 en 100 procent";
  return foutKleurovergang(s.kleurovergang, false);
}

/** Of `concept` iets anders is dan de opgeslagen Stijl. */
export function gewijzigd(concept: Stijl, opgeslagen: Stijl): boolean {
  return JSON.stringify(gewoon(concept)) !== JSON.stringify(gewoon(opgeslagen));
}

function gewoon(s: Stijl): Stijl {
  const kleuren = Object.fromEntries(KLEUREN.map((k) => [k.sleutel, s.kleuren[k.sleutel].toLowerCase()])) as Stijlkleuren;
  return {
    kleuren,
    lettertypes: { ...s.lettertypes },
    programmalogo: { ...s.programmalogo },
    snelheid: s.snelheid,
    beweging: { ...s.beweging, lijn: { ...s.beweging.lijn } },
    kleurovergang: { ...s.kleurovergang },
    kleurkracht: s.kleurkracht,
  };
}

/** De lettertypes om uit te kiezen: de ingebouwde, en daarna elk eigen lettertype dat de Stijl noemt. */
export function letterKeuzes(...stijlen: Stijl[]): string[] {
  const eigen = stijlen.flatMap((s) => LETTERS.map((l) => s.lettertypes[l.sleutel]));
  return [...new Set([...INGEBOUWDE_LETTERS, ...eigen])];
}

/** "PlayfairDisplay-Black.ttf" wordt "Playfair Display Black"; de hash die de server achter een
 * eigen bestand zet, valt weg. */
export function letterNaam(bestand: string): string {
  return bestand
    .replace(/\.[a-z0-9]+$/i, "")
    .replace(/-[0-9a-f]{16}$/, "")
    .replace(/[-_]+/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .trim();
}
