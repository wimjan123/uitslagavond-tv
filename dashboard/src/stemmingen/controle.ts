// Controle na de uitzending: welke Stemmingen in de tabel staan en in welke volgorde. De server
// rekent de status en de verschillen uit; hier alleen sorteren en tellen.
import type { Controlestatus } from "../typen/Controlestatus";
import type { Stemming } from "../typen/Stemming";
import type { Stemmingscontrole } from "../typen/Stemmingscontrole";

export const STATUSSEN: Controlestatus[] = ["UitkomstAnders", "VerschilPerFractie", "WachtOpOfficieel", "Gelijk"];

export const STATUS_TEKST: Record<Controlestatus, string> = {
  UitkomstAnders: "Uitkomst anders",
  VerschilPerFractie: "Verschil per Fractie",
  WachtOpOfficieel: "Wacht op officieel",
  Gelijk: "Gelijk",
};

/**
 * De ingevoerde Stemmingen, de ernstigste bovenaan: eerst een omgedraaide uitkomst (die in de Ether
 * zijn geweest voorop), dan een verschil per Fractie, dan wat nog wacht, dan wat gelijk is. Een
 * verschil waarover de Regie al koos, staat onderaan.
 */
export function controleLijst(lijst: Stemming[]): Stemming[] {
  const rang = (s: Stemming) => [
    s.controle.afgehandeld ? 1 : 0,
    STATUSSEN.indexOf(s.controle.status!),
    s.in_ether_ms != null ? 0 : 1,
  ];
  return lijst
    .filter((s) => s.controle.status !== null)
    .map((s, i) => ({ s, i, r: rang(s) }))
    .sort((a, b) => a.r[0] - b.r[0] || a.r[1] - b.r[1] || a.r[2] - b.r[2] || a.i - b.i)
    .map((x) => x.s);
}

export type Tellingen = Record<Controlestatus | "afgehandeld", number>;

/** Hoeveel Stemmingen elke status hebben; een verschil waarover de Regie koos telt als afgehandeld. */
export function tellingen(lijst: Stemming[]): Tellingen {
  const t: Tellingen = { UitkomstAnders: 0, VerschilPerFractie: 0, WachtOpOfficieel: 0, Gelijk: 0, afgehandeld: 0 };
  for (const s of lijst) {
    const st = s.controle.status;
    if (!st) continue;
    if (s.controle.afgehandeld && st !== "Gelijk") t.afgehandeld++;
    else t[st]++;
  }
  return t;
}

/** Hoeveel minuten na de hamerslag de server de uitslag voor het eerst bij Debat Direct zag. */
export function minutenNaHamerslag(c: Stemmingscontrole): number | null {
  const gezien = Date.parse(c.debat_direct_gezien ?? "");
  const hamer = Date.parse(c.officieel?.hamerslag ?? "");
  if (!Number.isFinite(gezien) || !Number.isFinite(hamer)) return null;
  return Math.floor((gezien - hamer) / 60000);
}
