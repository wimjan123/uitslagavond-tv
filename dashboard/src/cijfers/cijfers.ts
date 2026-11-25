// Het onderdeel Cijfers (DESIGN.md): de rijen per Partij met Nu, de vergelijkingsset en het
// verschil, en de eigen wijzigingen die nog onderweg zijn naar de server. De server rekent winst
// en verlies voor de Graphics uit (`Cijfers::winst_en_verlies`); dit is dezelfde regel voor het scherm.
import type { Cijfers } from "../typen/Cijfers";
import type { Cijferset } from "../typen/Cijferset";
import type { Partij } from "../typen/Partij";
import type { Volgorde } from "../typen/Volgorde";

/**
 * De Partijen in de volgorde van de Uitslagindeling, van links naar rechts: zoals de server die in
 * `Stand.uitslagindeling` zet (`Uitslagvolgorde::bepaal`). Een Partij die daar (nog) niet in staat,
 * komt achteraan in de volgorde van de partijlijst; zonder volgorde geldt de partijlijst.
 */
export function inUitslagindeling(cijfers: Cijfers, volgorde?: Volgorde): Partij[] {
  if (!volgorde) return cijfers.partijen;
  const plaats = (p: Partij) => {
    const i = volgorde.partijen.indexOf(p.sleutel);
    return i < 0 ? volgorde.partijen.length : i;
  };
  return [...cijfers.partijen].sort((a, b) => plaats(a) - plaats(b));
}

/** Eén Partij in het onderdeel Cijfers. */
export interface Rij {
  partij: Partij;
  nu: number;
  /** Zetels in de vergelijkingsset, met die van de Voorgangers erbij. `null` zonder vergelijkingsset. */
  eerder: number | null;
  verschil: number | null;
  /** Nieuw: geen Voorganger en geen Zetels in de vergelijkingsset. */
  nieuw: boolean;
  /** Telt als nieuw zodra de Partij Zetels heeft. */
  kanNieuw: boolean;
}

function zetels(set: Cijferset | undefined, partij: string): number {
  return set?.zetels[partij] ?? 0;
}

/**
 * De rijen van set `id`. Een Voorganger van een andere Partij en een Fractie zonder lijst staan er
 * alleen als ze in deze set Zetels hebben: verkiezingscijfers gaan per Partij.
 */
export function rijen(cijfers: Cijfers, id: string, volgorde?: Volgorde): Rij[] {
  const set = cijfers.sets.find((s) => s.id === id);
  const vorige = cijfers.vergelijk_met ? cijfers.sets.find((s) => s.id === cijfers.vergelijk_met) : undefined;
  const opgegaan = new Set(cijfers.partijen.flatMap((p) => p.voorgangers));
  return inUitslagindeling(cijfers, volgorde).flatMap((p) => {
    const nu = zetels(set, p.sleutel);
    const weg = opgegaan.has(p.sleutel);
    if (nu === 0 && (weg || p.fractie_zonder_lijst)) return [];
    const eerder = !vorige ? null : weg ? 0 : zetels(vorige, p.sleutel) + p.voorgangers.reduce((n, v) => n + zetels(vorige, v), 0);
    return [metNu({ partij: p, nu: 0, eerder, verschil: null, nieuw: false, kanNieuw: eerder === 0 && p.voorgangers.length === 0 && !weg }, nu)];
  });
}

/** Dezelfde rij met een ander aantal Zetels nu, zoals een eigen wijziging die nog onderweg is. */
export function metNu(r: Rij, nu: number): Rij {
  return { ...r, nu, verschil: r.eerder === null ? null : nu - r.eerder, nieuw: r.kanNieuw && nu > 0 };
}

/** De kop van de vergelijkingskolom: het jaartal uit de naam van de vergelijkingsset, anders de naam. */
export function kopVergelijking(cijfers: Cijfers): string | null {
  const set = cijfers.sets.find((s) => s.id === cijfers.vergelijk_met);
  if (!set) return null;
  return set.label.match(/\b(19|20)\d\d\b/)?.[0] ?? set.label;
}

/** Twee kolommen: de eerste helft links, de rest rechts. */
export function verdeel<T>(lijst: T[], kolommen = 2): T[][] {
  const per = Math.ceil(lijst.length / kolommen);
  return Array.from({ length: kolommen }, (_, i) => lijst.slice(i * per, (i + 1) * per));
}

/**
 * Eigen wijzigingen per vakje (`set/partij`) die nog onderweg zijn. `tot` is het volgnummer van de
 * Stand waarin de server de wijziging uitvoerde, of `null` zolang het antwoord er niet is.
 */
export type Concept = Record<string, { zetels: number; tot: number | null }>;

export const vakje = (set: string, partij: string) => `${set}/${partij}`;

/** Wat het vakje toont: de eigen waarde die nog onderweg is, anders de Stand. */
export function zichtbaar(concept: Concept, set: string, partij: string, inStand: number): number {
  return concept[vakje(set, partij)]?.zetels ?? inStand;
}

/** Haalt de wijzigingen weg die in deze Stand al zijn verwerkt: daarna telt de Stand, ook als een andere Werkplek het vakje intussen veranderde. */
export function ruimOp(concept: Concept, volgnummer: number): Concept {
  const over = Object.entries(concept).filter(([, c]) => c.tot === null || c.tot > volgnummer);
  return over.length === Object.keys(concept).length ? concept : Object.fromEntries(over);
}

/** "+5", "-3" of "0". */
export function verschilTekst(v: number): string {
  return v > 0 ? `+${v}` : String(v);
}
