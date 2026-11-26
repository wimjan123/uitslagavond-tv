// Standpunten en Cameraritten voor het dashboard: hoe je ze herkent, en wat de plattegrond en het
// instelpaneel van een Camerarit tonen. De server rekent de rit zelf uit (crates/gedeeld/src/camerarit.rs);
// hier staat alleen wat het dashboard nodig heeft om hem te tekenen en te beschrijven.
import type { BewerktSjabloon } from "../typen/BewerktSjabloon";
import type { CameraProfiel } from "../typen/CameraProfiel";
import type { Camerarit } from "../typen/Camerarit";
import type { Graphic } from "../typen/Graphic";
import type { Sjabloon } from "../typen/Sjabloon";
import type { Stand } from "../typen/Stand";
import type { Standpunt } from "../typen/Standpunt";
import type { Stoel } from "../typen/Stoel";
import { CAMERA_PROFIELEN, PARTIJBEWEGINGEN, STOELBEWEGINGEN } from "../typen/cameraBewegingen.ts";
import { RITSTAP, STANDAARD_RITTEN, VASTE_OPNAMES } from "../typen/vasteRitten.ts";
import { sjabloonId, standaardStandpunt } from "../graphic.ts";
import { benodigd, geldendSjabloon } from "../vormgeving/sjablonen.ts";

/** Gelijk aan TIKKEN_PER_SECONDE, HERHAAL_AFSTAND, HERHAAL_HOEK, RIT_MIN_S en DUUR_FACTOR in crates/gedeeld. */
export const TIKKEN = 100;
const HERHAAL_AFSTAND = 0.5;
const HERHAAL_HOEK = 10;
export const RIT_MIN_S = 0.5;
export const DUUR_FACTOR = 4;

/** Iets wat je kiest waar je een camera kiest: een gerichte beweging, een Standpunt of een Camerarit. */
export interface Camerakeus {
  letter: string;
  /** A tot H, Q tot U, J, K of L, of geen. */
  toets: string | null;
  naam: string;
  /** Zoals de Regie het in een keuzelijst ziet: "J · Close-up", "Boog om de Partij · 12 s", of alleen de naam. */
  tekst: string;
  soort: "Partijbeweging" | "Stoelbeweging" | "Standpunt" | "Camerarit";
  /** Korte toelichting onder de keuze, voor gerichte bewegingen. */
  toelichting?: string;
}

/** De toets van een Standpunt of Camerarit: de letter van een vast Standpunt of een vaste Camerarit, J, K of L, of geen. */
export function toetsVan(p: { letter: string; toets?: string; vast?: boolean }): string | null {
  return p.vast ? p.letter : (p.toets ?? null);
}

/** Aanschuiven: langzaam naar het blok van de gekozen Partij (gelijk aan `camera::AANSCHUIVEN`). */
export const AANSCHUIVEN = "aanschuiven";

/** Camera blijft staan: Toon verandert alleen de Graphic en de Stoelen, de Live-camera blijft
 * waar hij is, ook als hij vrij rijdt of een rit speelt (gelijk aan `camera::BLIJF`). */
export const BLIJF = "blijf";
export const BLIJF_NAAM = "Camera blijft staan";

/** Totaal boven het paneel: alle Stoelen boven een paneel onderin (gelijk aan `camera::BOVEN_HET_PANEEL`). */
export const BOVEN_HET_PANEEL = "boven-het-paneel";
export const BOVEN_HET_PANEEL_NAAM = "Totaal boven het paneel";

/** Camera blijft staan, alle Standpunten en Cameraritten, en Totaal boven het paneel; met `aanschuiven` ook Aanschuiven, voor een Graphic met een Partij. */
export function cameraKeuzes(standpunten: Standpunt[], ritten: Camerarit[], aanschuiven = false): Camerakeus[] {
  const keus = (letter: string, toets: string | null, naam: string, soort: Camerakeus["soort"]): Camerakeus => ({
    letter,
    toets,
    naam,
    tekst: toets ? `${toets} · ${naam}` : naam,
    soort,
  });
  return [
    keus(BLIJF, null, BLIJF_NAAM, "Standpunt"),
    ...standpunten.map((p) => keus(p.letter, toetsVan(p), p.naam, "Standpunt")),
    ...ritten.map((r) => keus(r.letter, toetsVan(r), r.naam, "Camerarit")),
    keus(BOVEN_HET_PANEEL, null, BOVEN_HET_PANEEL_NAAM, "Standpunt"),
    ...(aanschuiven ? [keus(AANSCHUIVEN, null, "Aanschuiven naar de gekozen Partij", "Standpunt")] : []),
  ];
}

/** Alleen de Standpunten, Cameraritten en gerichte bewegingen die bij dit `CameraProfiel` passen
 * (gelijk aan `Stand::standpunt_past_bij`). */
export function passendeCameraKeuzes(
  standpunten: Standpunt[],
  ritten: Camerarit[],
  profiel: CameraProfiel,
): Camerakeus[] {
  const r = CAMERA_PROFIELEN[profiel];
  const keus = (
    letter: string,
    toets: string | null,
    naam: string,
    soort: Camerakeus["soort"],
    toelichting?: string,
    tekst?: string,
  ): Camerakeus => ({
    letter,
    toets,
    naam,
    tekst: tekst ?? (toets ? `${toets} · ${naam}` : naam),
    soort,
    toelichting,
  });
  const uit: Camerakeus[] = [keus(BLIJF, null, BLIJF_NAAM, "Standpunt")];
  if (r.partij) {
    for (const b of PARTIJBEWEGINGEN) {
      uit.push(keus(b.sleutel, null, b.naam, "Partijbeweging", b.toelichting, `${b.naam} · ${duur(b.duur_s)}`));
    }
  }
  if (r.stoel) {
    for (const b of STOELBEWEGINGEN) {
      uit.push(keus(b.sleutel, null, b.naam, "Stoelbeweging", b.toelichting, `${b.naam} · ${duur(b.duur_s)}`));
    }
  }
  if (r.bovenHetPaneel) {
    uit.push(keus(BOVEN_HET_PANEEL, null, BOVEN_HET_PANEEL_NAAM, "Standpunt"));
  }
  for (const p of standpunten) {
    const mag = p.vast ? r.vasteStandpunten.includes(p.letter) : r.eigen;
    if (mag) uit.push(keus(p.letter, toetsVan(p), p.naam, "Standpunt"));
  }
  for (const rit of ritten) {
    const mag = rit.vast ? r.vasteRitten.includes(rit.letter) : r.eigen;
    if (mag) {
      const t = toetsVan(rit);
      const basis = t ? `${t} · ${rit.naam}` : rit.naam;
      uit.push(keus(rit.letter, t, rit.naam, "Camerarit", undefined, `${basis} · ${duur(rit.bijschaving.duur_s)}`));
    }
  }
  return uit;
}

/** Het `CameraProfiel` van een Sjabloon (gelijk aan `Sjabloon::camera_profiel`). */
export function cameraProfielVanSjabloon(s: Sjabloon): CameraProfiel {
  switch (s.id) {
    case "partij-uitgelicht":
      return "Partij";
    case "persoonskaart":
      return "PersoonSjabloon";
    case "zetelverdeling":
      return "HeleZaal";
    case "coalitie":
      return "CoalitieZaal";
    case "zetelbalk":
    case "twee-sets":
    case "coalitiebalk":
    case "uitslag-stemming":
      return "BovenPaneel";
    case "uitslag-per-partij":
    case "winst-en-verlies":
    case "fracties-voor-en-tegen":
    case "hoofdelijke-stemming":
      return "GrootPaneel";
    case "titelkaart":
      return "Algemeen";
    default:
      if (s.stoelen === "PartijUitgelicht") return "Partij";
      if (benodigd(s).partij) return "AlgemeenMetPartij";
      if (s.achtergrond.soort === "ZaalDonker") return "GrootPaneel";
      return "Algemeen";
  }
}

/** Het `CameraProfiel` van een Graphic (gelijk aan `Stand::camera_profiel`). Met `heeftStoel`
 * voor een Persoonskaart in het formulier voordat de server `gegevens` invult. */
export function cameraProfielVanGraphic(
  g: Graphic,
  sjablonen: readonly BewerktSjabloon[] = [],
  heeftStoel?: boolean,
): CameraProfiel {
  switch (g.sjabloon) {
    case "PartijUitgelicht":
      return "Partij";
    case "Persoonskaart":
      return (heeftStoel ?? !!g.gegevens?.stoel) ? "PersoonMetStoel" : "PersoonZonderStoel";
    case "Zetelverdeling":
      return "HeleZaal";
    case "Coalitie":
      return "CoalitieZaal";
    case "Zetelbalk":
    case "TweeSets":
    case "Coalitiebalk":
    case "UitslagStemming":
      return "BovenPaneel";
    case "UitslagPerPartij":
    case "WinstEnVerlies":
    case "FractiesVoorEnTegen":
    case "HoofdelijkeStemming":
      return "GrootPaneel";
    case "Titelkaart":
      return "Algemeen";
    case "Eigen": {
      const sj = geldendSjabloon(sjablonen, g.id);
      if (sj) return cameraProfielVanSjabloon(sj);
      return g.partij ? "AlgemeenMetPartij" : "Algemeen";
    }
  }
}

/** Leesbare naam van een gerichte partij- of stoelbeweging (zowel abstracte keuzesleutel zoals
 * `"partij-boog"` als uitgevulde doelsleutel zoals `"partij-boog-40-65"` of `"stoel-langs-2-3-1"`). */
export function gerichteBewegingNaam(letter: string): string | null {
  for (const b of PARTIJBEWEGINGEN) {
    if (letter === b.sleutel || new RegExp(`^${b.sleutel}-\\d+-\\d+$`).test(letter)) {
      return b.naam;
    }
  }
  const losseStoel = /^stoel-(\d+-\d+-\d+)$/.exec(letter);
  if (losseStoel) return `Shot op Stoel ${losseStoel[1]}`;
  for (const b of STOELBEWEGINGEN) {
    if (letter === b.sleutel) return b.naam;
    const m = new RegExp(`^${b.sleutel}-(\\d+-\\d+-\\d+)$`).exec(letter);
    if (m) return `${b.naam} · Stoel ${m[1]}`;
  }
  return null;
}

/** Of deze sleutel een geanimeerde gerichte beweging is (zodat "Test rit" hem kan proefdraaien). */
export function isGerichteRit(letter: string): boolean {
  const ritten = [
    "aanschuiven",
    "partij-boog",
    "partij-indalen",
    "partij-langs",
    "partij-uitzoomen",
    "stoel-aanschuiven",
    "stoel-langs",
    "stoel-indalen",
    "stoel-uitzoomen",
  ];
  return ritten.some((r) => letter === r || letter.startsWith(`${r}-`));
}

/** De effectieve standaard camera-omschrijving voor een Graphic (van het bewerkte of vaste
 * Sjabloon, of de standaard van de soort Graphic). */
export function standaardCameraTekst(s: Stand, g: Graphic, stoelOverride?: Stoel | null): string | null {
  const sj = geldendSjabloon(s.sjablonen, sjabloonId(g));
  const stoel = stoelOverride !== undefined ? stoelOverride : (g.sjabloon === "Persoonskaart" ? (g.gegevens?.stoel ?? null) : null);
  const letter = sj?.standpunt ?? standaardStandpunt(g);
  if (g.sjabloon === "Persoonskaart") {
    if (stoel) {
      const sb = STOELBEWEGINGEN.find((b) => b.sleutel === letter);
      const plek = `${stoel.vak}-${stoel.rij}-${stoel.nummer}`;
      if (sb) return `${sb.naam} · Stoel ${plek} · ${duur(sb.duur_s)}`;
      if (letter.startsWith("stoel-")) return `Shot op Stoel ${plek} · 2 s`;
    } else if (STOELBEWEGINGEN.some((b) => b.sleutel === letter) || letter.startsWith("stoel-")) {
      const fallback = s.standpunten.find((p) => p.letter === "S");
      return fallback ? `${fallback.letter} · ${fallback.naam}` : "S";
    }
  }
  const pb = PARTIJBEWEGINGEN.find((b) => b.sleutel === letter);
  if (pb) return `${pb.naam} · ${duur(pb.duur_s)}`;
  if (letter === BOVEN_HET_PANEEL) return BOVEN_HET_PANEEL_NAAM;
  if (letter === BLIJF) return BLIJF_NAAM;
  const sp = s.standpunten.find((p) => p.letter === letter);
  if (sp) {
    const t = toetsVan(sp);
    return t ? `${t} · ${sp.naam}` : sp.naam;
  }
  const rit = s.cameraritten.find((r) => r.letter === letter);
  if (rit) {
    const t = toetsVan(rit);
    const basis = t ? `${t} · ${rit.naam}` : rit.naam;
    return `${basis} · ${duur(rit.bijschaving.duur_s)}`;
  }
  return gerichteBewegingNaam(letter) ?? letter;
}

type Punt = [number, number, number, number, number, number];
const uitgerekend = new Map<string, Punt[]>();

/** De opname (100 standen per seconde): bij een vaste Camerarit uit de software, met de standen
 * tussen twee bewaarde recht aangevuld (gelijk aan `Camerarit::punten`, op een millimeter na). */
export function punten(r: Camerarit): Punt[] {
  const dun = r.vast ? VASTE_OPNAMES[r.letter] : undefined;
  if (!dun) return r.opname;
  let vol = uitgerekend.get(r.letter);
  if (!vol) {
    vol = [];
    for (let i = 0; i < dun.length - 1; i++)
      for (let k = 0; k < RITSTAP; k++) vol.push(dun[i].map((v, j) => v + ((dun[i + 1][j] - v) * k) / RITSTAP) as Punt);
    vol.push(dun[dun.length - 1]);
    uitgerekend.set(r.letter, vol);
  }
  return vol;
}

/** De standaard van een vaste Camerarit, of `null`. */
export function standaardRit(letter: string): Camerarit | null {
  return STANDAARD_RITTEN.find((s) => s.letter === letter) ?? null;
}

/** Of een vaste Camerarit anders is dan zijn standaard (naam of schuifjes). */
export function ritAangepast(r: Camerarit): boolean {
  const s = standaardRit(r.letter);
  return !!s && (s.naam !== r.naam || JSON.stringify(s.bijschaving) !== JSON.stringify(r.bijschaving));
}

/** Het begin en eind (indexen, tot en met) van wat er na afknippen overblijft (gelijk aan `Camerarit::stuk`). */
function stuk(r: Camerarit): [number, number] {
  const n = punten(r).length;
  if (n < 2) return [0, Math.max(n - 1, 0)];
  const tik = (s: number) => Math.round(Math.max(s, 0) * TIKKEN);
  const min = Math.min(Math.floor(RIT_MIN_S * TIKKEN), n - 1);
  const b = Math.min(tik(r.bijschaving.begin_eraf_s), n - 1);
  const e = Math.max(n - 1 - tik(r.bijschaving.eind_eraf_s), b);
  if (e - b < min) {
    const b2 = Math.min(b, n - 1 - min);
    return [b2, b2 + min];
  }
  return [b, e];
}

/** Hoe lang de opname is, in seconden. */
export function opnameS(r: Camerarit): number {
  return Math.max(punten(r).length - 1, 0) / TIKKEN;
}

/** Hoe lang het stuk is dat na afknippen overblijft, in seconden. */
export function stukS(r: Camerarit): number {
  const [b, e] = stuk(r);
  return (e - b) / TIKKEN;
}

const kortsteHoek = (d: number) => ((((d + Math.PI) % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI)) - Math.PI;

/** Waarom de rit niet kan herhalen, of `null` als het kan (gelijk aan `Camerarit::kan_herhalen`). */
export function kanHerhalen(r: Camerarit): string | null {
  const [b, e] = stuk(r);
  const p = punten(r);
  const a = p[b];
  const z = p[e];
  if (!a || !z) return "er is geen opname";
  const afstand = Math.hypot(a[0] - z[0], a[1] - z[1], a[2] - z[2]);
  const hoek =
    (Math.max(Math.abs(kortsteHoek(z[3] - a[3])), Math.abs(kortsteHoek(z[4] - a[4])), Math.abs(z[5] - a[5])) * 180) / Math.PI;
  const redenen: string[] = [];
  if (afstand > HERHAAL_AFSTAND) redenen.push(`liggen ${komma(afstand)} m uit elkaar`);
  if (hoek > HERHAAL_HOEK) redenen.push(`kijken ${Math.round(hoek)}° anders`);
  return redenen.length ? `begin en eind ${redenen.join(" en ")}` : null;
}

/** De route van bovenaf als [x, z], van begin eraf tot eind eraf, met hooguit `aantal` + 1 punten. */
export function routeVan(r: Camerarit, aantal = 60): [number, number][] {
  const [b, e] = stuk(r);
  const stap = Math.max(1, Math.ceil((e - b) / aantal));
  const route: [number, number][] = [];
  const p = punten(r);
  for (let i = b; i < e; i += stap) route.push([p[i][0], p[i][2]]);
  if (p[e]) route.push([p[e][0], p[e][2]]);
  return route;
}

/** Een getal met één cijfer achter de komma, zoals "2,3". */
export function komma(v: number): string {
  return v.toFixed(1).replace(".", ",");
}

/** Een duur zoals op de knoppen in de multiviewer: hele seconden zonder ",0" ("16 s"), anders "2,5 s". */
export function duur(v: number): string {
  return Math.abs(v - Math.round(v)) < 0.05 ? `${Math.round(v)} s` : `${komma(v)} s`;
}

/** De tijdregel in de lijst, zoals "16 s · herhaalt". */
export function ritTiming(r: Camerarit): string {
  const s = duur(r.bijschaving.duur_s);
  return r.bijschaving.eind === "Herhalen" ? `${s} · herhaalt` : s;
}

/** De tijdregel van een Standpunt, zoals "2 s · schuift" of "knipt". */
export function standpuntTiming(p: Standpunt): string {
  if (p.overgang.soort === "Knippen") return "knipt";
  const s = duur(p.overgang.duur_s);
  return p.na_aankomst.soort === "Schuiven" ? `${s} · schuift` : s;
}

/** Hoe lang er al is opgenomen, zoals "12,3 s". */
export function opnameTijd(tikken: number): string {
  return `${komma(tikken / TIKKEN)} s`;
}
