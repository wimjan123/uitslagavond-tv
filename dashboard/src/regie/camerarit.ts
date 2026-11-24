// Standpunten en Cameraritten voor het dashboard: hoe je ze herkent, en wat de plattegrond en het
// instelpaneel van een Camerarit tonen. De server rekent de rit zelf uit (crates/gedeeld/src/camerarit.rs);
// hier staat alleen wat het dashboard nodig heeft om hem te tekenen en te beschrijven.
import type { Camerarit } from "../typen/Camerarit";
import type { Standpunt } from "../typen/Standpunt";

/** Gelijk aan TIKKEN_PER_SECONDE, HERHAAL_AFSTAND, HERHAAL_HOEK, RIT_MIN_S en DUUR_FACTOR in crates/gedeeld. */
export const TIKKEN = 100;
const HERHAAL_AFSTAND = 0.5;
const HERHAAL_HOEK = 10;
export const RIT_MIN_S = 0.5;
export const DUUR_FACTOR = 4;

/** Iets wat je kiest waar je een camera kiest: een Standpunt of een Camerarit. */
export interface Camerakeus {
  letter: string;
  /** A tot H, J, K of L, of geen. */
  toets: string | null;
  naam: string;
  /** Zoals de Regie het in een keuzelijst ziet: "J · Close-up", of alleen de naam zonder toets. */
  tekst: string;
  soort: "Standpunt" | "Camerarit";
}

/** De toets van een Standpunt of Camerarit: de letter van een vast Standpunt, J, K of L, of geen. */
export function toetsVan(p: { letter: string; toets?: string; vast?: boolean }): string | null {
  return p.vast ? p.letter : (p.toets ?? null);
}

export function cameraKeuzes(standpunten: Standpunt[], ritten: Camerarit[]): Camerakeus[] {
  const keus = (letter: string, toets: string | null, naam: string, soort: Camerakeus["soort"]): Camerakeus => ({
    letter,
    toets,
    naam,
    tekst: toets ? `${toets} · ${naam}` : naam,
    soort,
  });
  return [
    ...standpunten.map((p) => keus(p.letter, toetsVan(p), p.naam, "Standpunt")),
    ...ritten.map((r) => keus(r.letter, toetsVan(r), r.naam, "Camerarit")),
  ];
}

/** Het begin en eind (indexen, tot en met) van wat er na afknippen overblijft (gelijk aan `Camerarit::stuk`). */
function stuk(r: Camerarit): [number, number] {
  const n = r.opname.length;
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
  return Math.max(r.opname.length - 1, 0) / TIKKEN;
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
  const a = r.opname[b];
  const z = r.opname[e];
  if (!a || !z) return "er is geen opname";
  const afstand = Math.hypot(a[0] - z[0], a[1] - z[1], a[2] - z[2]);
  const hoek =
    (Math.max(Math.abs(kortsteHoek(z[3] - a[3])), Math.abs(kortsteHoek(z[4] - a[4])), Math.abs(z[5] - a[5])) * 180) / Math.PI;
  const redenen: string[] = [];
  if (afstand > HERHAAL_AFSTAND) redenen.push(`liggen ${komma(afstand)} m uit elkaar`);
  if (hoek > HERHAAL_HOEK) redenen.push(`kijken ${Math.round(hoek)}° anders`);
  return redenen.length ? `begin en eind ${redenen.join(" en ")}` : null;
}

/** De route van bovenaf als [x, z], van begin eraf tot eind eraf, met hooguit `punten` + 1 punten. */
export function routeVan(r: Camerarit, punten = 60): [number, number][] {
  const [b, e] = stuk(r);
  const stap = Math.max(1, Math.ceil((e - b) / punten));
  const route: [number, number][] = [];
  for (let i = b; i < e; i += stap) route.push([r.opname[i][0], r.opname[i][2]]);
  if (r.opname[e]) route.push([r.opname[e][0], r.opname[e][2]]);
  return route;
}

/** Een getal met één cijfer achter de komma, zoals "2,3". */
export function komma(v: number): string {
  return v.toFixed(1).replace(".", ",");
}

/** De tijdregel in de lijst, zoals "16,0 s · herhaalt". */
export function ritTiming(r: Camerarit): string {
  const s = `${komma(r.bijschaving.duur_s)} s`;
  return r.bijschaving.eind === "Herhalen" ? `${s} · herhaalt` : s;
}

/** De tijdregel van een Standpunt, zoals "2,0 s · schuift" of "knipt". */
export function standpuntTiming(p: Standpunt): string {
  if (p.overgang.soort === "Knippen") return "knipt";
  const s = `${komma(p.overgang.duur_s)} s`;
  return p.na_aankomst.soort === "Schuiven" ? `${s} · schuift` : s;
}

/** Hoe lang er al is opgenomen, zoals "12,3 s". */
export function opnameTijd(tikken: number): string {
  return `${komma(tikken / TIKKEN)} s`;
}
