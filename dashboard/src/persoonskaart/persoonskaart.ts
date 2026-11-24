// De Persoonskaart in het dashboard: zoeken op naam, een Kamerlid kiezen door op zijn Stoel te
// klikken, en de waarschuwingen (geen foto, geen Stoel). De server vult de gegevens bij klaarzetten
// in (`Stand::persoonsgegevens`); hier staat alleen wat het formulier vooraf moet weten.
import type { Kamerlid } from "../typen/Kamerlid";
import type { Stand } from "../typen/Stand";
import type { Stoel } from "../typen/Stoel";
import type { Wie } from "../typen/Wie";

/** Stoelen per rij in elk vak, van voor naar achter (gelijk aan `STOELEN_PER_RIJ`). */
const STOELEN_PER_RIJ = [2, 3, 4, 5, 5, 6];
const PER_VAK = STOELEN_PER_RIJ.reduce((a, b) => a + b, 0);

/** De Stoel op plaats `i` in de volgorde van de Stand (gelijk aan `Stoel::alle`). */
export function stoelOp(i: number): Stoel | null {
  if (!Number.isInteger(i) || i < 0 || i >= 6 * PER_VAK) return null;
  let rest = i % PER_VAK;
  for (let rij = 0; rij < STOELEN_PER_RIJ.length; rij++) {
    if (rest < STOELEN_PER_RIJ[rij]) return { vak: Math.floor(i / PER_VAK) + 1, rij: rij + 1, nummer: rest + 1 };
    rest -= STOELEN_PER_RIJ[rij];
  }
  return null;
}

/** De plaats van een Stoel in de volgorde van de Stand (gelijk aan `Stoel::index`). */
export function plaatsVan(s: Stoel): number {
  return (s.vak - 1) * PER_VAK + STOELEN_PER_RIJ.slice(0, s.rij - 1).reduce((a, b) => a + b, 0) + s.nummer - 1;
}

/** De sleutel van het shot op een Stoel (gelijk aan `camera::stoelsleutel`). */
export function stoelsleutel(s: Stoel): string {
  return `stoel-${s.vak}-${s.rij}-${s.nummer}`;
}

/** "Dieke van Groningen" (gelijk aan `Kamerlid::naam`). */
export function naamVan(k: Pick<Kamerlid, "roepnaam" | "initialen" | "tussenvoegsel" | "achternaam">): string {
  return [k.roepnaam.trim() || k.initialen.trim(), k.tussenvoegsel?.trim() ?? "", k.achternaam.trim()].filter(Boolean).join(" ");
}

/** Iemand die de Regie op de kaart kan zetten. */
export interface Kandidaat {
  wie: Wie;
  naam: string;
  /** De korte naam van de Partij, of leeg. */
  partij: string;
  stoel: Stoel | null;
  foto: boolean;
}

/** De Stoel van een Kamerlid in de Zaalindeling (gelijk aan `Zaalindeling::stoel_van`). */
export function stoelVan(s: Stand, kamerlid: string): Stoel | null {
  const b = s.zaalindeling.stoelen.find((x) => x.kamerlid === kamerlid);
  return b ? { vak: b.vak, rij: b.rij, nummer: b.nummer } : null;
}

function kortVan(s: Stand, sleutel: string | undefined): string {
  return (sleutel && s.cijfers.partijen.find((p) => p.sleutel === sleutel)?.kort) || "";
}

/** Alle Kamerleden van de Kamer die zit en daarna de eigen personen. */
export function kandidaten(s: Stand): Kandidaat[] {
  const leden: Kandidaat[] = s.kamer.kamerleden.map((k) => ({
    wie: { soort: "Kamerlid", id: k.id },
    naam: naamVan(k),
    partij: kortVan(s, s.kamer.partij_van[k.fractie]),
    stoel: stoelVan(s, k.id),
    foto: k.foto !== "Geen",
  }));
  const eigen: Kandidaat[] = s.personen.map((p) => ({
    wie: { soort: "Eigen", id: p.id },
    naam: p.naam,
    partij: kortVan(s, p.partij),
    stoel: null,
    foto: p.foto,
  }));
  return [...leden, ...eigen];
}

/** Zonder accenten en hoofdletters, zodat "dijk" ook "Dijk" vindt en "e" ook "ë". */
function plat(t: string): string {
  return t.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
}

/** Zoeken op naam of Partij: elk woord van de zoektekst moet ergens in naam of Partij staan. */
export function zoek(s: Stand, tekst: string, max = 12): Kandidaat[] {
  const woorden = plat(tekst).split(/\s+/).filter(Boolean);
  if (woorden.length === 0) return [];
  return kandidaten(s)
    .filter((k) => {
      const hooiberg = plat(`${k.naam} ${k.partij}`);
      return woorden.every((w) => hooiberg.includes(w));
    })
    .slice(0, max);
}

/** Het Kamerlid op de Stoel op plaats `i`, als dat bij de Kamer die zit hoort. */
export function opStoel(s: Stand, i: number): Kandidaat | null {
  const stoel = stoelOp(i);
  if (!stoel) return null;
  const b = s.zaalindeling.stoelen.find((x) => x.vak === stoel.vak && x.rij === stoel.rij && x.nummer === stoel.nummer);
  const id = b?.kamerlid;
  if (!id) return null;
  return kandidaten(s).find((k) => k.wie.soort === "Kamerlid" && k.wie.id === id) ?? null;
}

/** Wat de Regie moet weten voordat de kaart in beeld gaat. */
export function waarschuwingen(s: Stand, wie: Wie): string[] {
  const k = kandidaten(s).find((x) => x.wie.soort === wie.soort && x.wie.id === wie.id);
  if (!k) return [wie.soort === "Kamerlid" ? "Dit Kamerlid zit niet meer in de Kamer" : "Deze persoon is gewist"];
  const uit: string[] = [];
  if (!k.foto) uit.push("Geen foto: de kaart toont de voorletters op de partijkleur");
  if (wie.soort === "Kamerlid" && !k.stoel) uit.push("Geen Stoel in de Zaalindeling: er licht niets op en de camera gaat naar het Standpunt van het Sjabloon");
  return uit;
}

/** Het Standpunt van het Sjabloon voor deze persoon (gelijk aan `Graphic::standaard_standpunt`). */
export function standpuntVoor(s: Stand, wie: Wie): string {
  const stoel = wie.soort === "Kamerlid" ? stoelVan(s, wie.id) : null;
  return stoel ? stoelsleutel(stoel) : "S";
}
