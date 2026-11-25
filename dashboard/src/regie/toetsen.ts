// Sneltoetsen van het dashboard (DESIGN.md, Keyboard). Ze werken overal buiten een tekstveld.
// Een nieuwe toets komt in TOETSEN. De cijfers 1 tot 9 zetten een Preset klaar.
import { toetsVan } from "./camerarit.ts";

export { toetsVan };

/** Kleur uit geeft de Stoelen in de Ether direct hun eigen leer terug, los van de Graphic. */
export type TakeActie = "Toon" | "Bijwerken" | "UitBeeld" | "KleurUit";

/** Stuur de camera van de Ether naar het Standpunt met deze toets; met Shift knipt hij meteen. */
export interface StandpuntActie {
  soort: "Standpunt";
  toets: string;
  knip: boolean;
}

/**
 * Zet de Vrije camera aan of uit (de gamepad stuurt dan de camera van de Ether), Oefenen aan of uit
 * (de gamepad stuurt alleen het Voorbeeld), of start of stopt een opname (alleen tijdens Oefenen).
 */
export type CameraActie = "VrijeCamera" | "Oefenen" | "Opnemen";

/** Zet de Preset met dit nummer klaar. Hij komt pas in de Ether met Toon. */
export interface PresetActie {
  soort: "Preset";
  nummer: number;
}

export type Actie = TakeActie | CameraActie | StandpuntActie | PresetActie;

export interface Toets {
  key: string;
  shiftKey?: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  /** Waar de focus staat: in een tekstveld typen toetsen, op een knop drukken spatie en Enter die knop. */
  doel: "tekstveld" | "knop" | "elders";
}

/** Enter staat hier bewust niet in: Enter toont nooit iets. B blijft Bijwerken; op de gamepad neemt B op. */
const TOETSEN: Record<string, TakeActie | CameraActie> = {
  " ": "Toon",
  b: "Bijwerken",
  x: "UitBeeld",
  z: "KleurUit",
  v: "VrijeCamera",
  o: "Oefenen",
  r: "Opnemen",
};

/**
 * De zes vaste Standpunten (Totaal vanaf voorzitter, Laag over de banken, Linkerflank, Rechterflank,
 * Bovenaf, Katheder), dan J, K en L voor eigen Standpunten (gelijk aan `EIGEN_TOETSEN` in crates/gedeeld).
 */
export const STANDPUNT_TOETSEN = ["a", "s", "d", "f", "g", "h", "j", "k", "l"];

/** Het Standpunt of de Camerarit die deze toets nu heeft, of `null`. */
export function standpuntVoorToets<T extends { letter: string; toets?: string; vast?: boolean }>(lijst: T[], toets: string): T | null {
  return lijst.find((p) => toetsVan(p) === toets) ?? null;
}

export function actieVoorToets(t: Toets): Actie | null {
  if (t.ctrlKey || t.metaKey || t.altKey || t.doel === "tekstveld") return null;
  if (t.key === " " && t.doel === "knop") return null;
  const k = t.key.toLowerCase();
  if (STANDPUNT_TOETSEN.includes(k)) return { soort: "Standpunt", toets: k.toUpperCase(), knip: !!t.shiftKey };
  if (/^[1-9]$/.test(k)) return { soort: "Preset", nummer: Number(k) };
  return TOETSEN[k] ?? null;
}

/** Bepaalt waar de focus staat, voor {@link Toets.doel}. */
export function doelVan(el: EventTarget | null): Toets["doel"] {
  if (!(el instanceof Element)) return "elders";
  if ((el instanceof HTMLElement && el.isContentEditable) || el.matches("input, textarea, select")) return "tekstveld";
  if (el.matches("button, a[href], [role=button], [role=tab]")) return "knop";
  return "elders";
}
