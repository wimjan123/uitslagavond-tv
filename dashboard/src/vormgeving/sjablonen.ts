// Sjablonen in de vormgevingstab: de lijst (vast, aangepast, eigen), de melding bij een nieuwe
// standaard, een nieuw eigen Sjabloon en wat de server niet opslaat. De vaste Sjablonen en hun
// vingerafdruk komen uit crates/gedeeld (typen/standaardSjablonen.ts).
import type { BewerktSjabloon } from "../typen/BewerktSjabloon";
import type { Blok } from "../typen/Blok";
import type { Bloksoort } from "../typen/Bloksoort";
import type { Erin } from "../typen/Erin";
import type { Eruit } from "../typen/Eruit";
import type { Sjabloon } from "../typen/Sjabloon";
import type { Stoelen } from "../typen/Stoelen";
import type { Tekstbron } from "../typen/Tekstbron";
import { EIGEN, GROEP_EIGEN, MAX_BLOKKEN, MAX_SJABLOONNAAM, STANDAARD_SJABLONEN, VINGERAFDRUKKEN } from "../typen/standaardSjablonen.ts";
import { MAX_SNELHEID, MIN_SNELHEID } from "../typen/standaardStijl.ts";
import { foutKleurovergang } from "./kleurovergang.ts";

export type Soort = "vast" | "aangepast" | "eigen";

export interface Rij {
  sjabloon: Sjabloon;
  soort: Soort;
  /** Een aangepast vast Sjabloon op een oudere standaard (gelijk aan `BewerktSjabloon::nieuwe_standaard`). */
  nieuweStandaard: boolean;
}

/** Het vaste Sjabloon met dit `id` zoals deze versie het levert. */
export function standaardVan(id: string): Sjabloon | undefined {
  return STANDAARD_SJABLONEN.find((s) => s.id === id);
}

/** Het Sjabloon met dit `id` zoals het nu geldt: bewerkt, anders de standaard (gelijk aan `Stand::sjabloon`). */
export function geldendSjabloon(bewerkt: readonly BewerktSjabloon[], id: string): Sjabloon | undefined {
  return bewerkt.find((b) => b.sjabloon.id === id)?.sjabloon ?? standaardVan(id);
}

/** Alle Sjablonen zoals ze nu gelden: eerst de vaste (aangepast of niet), dan de eigen. */
export function lijst(bewerkt: readonly BewerktSjabloon[]): Rij[] {
  const vast: Rij[] = STANDAARD_SJABLONEN.map((s) => {
    const b = bewerkt.find((x) => x.sjabloon.id === s.id);
    if (!b) return { sjabloon: s, soort: "vast", nieuweStandaard: false };
    const basis = b.standaard;
    return { sjabloon: b.sjabloon, soort: "aangepast", nieuweStandaard: !!basis && basis !== VINGERAFDRUKKEN[s.id] };
  });
  const eigen: Rij[] = bewerkt
    .filter((b) => !standaardVan(b.sjabloon.id))
    .map((b) => ({ sjabloon: b.sjabloon, soort: "eigen", nieuweStandaard: false }));
  return [...vast, ...eigen];
}

/** Een nieuw eigen Sjabloon met een `id` dat nog niet bestaat: een Vlak met een kop uit het invulveld "kop". */
export function nieuwSjabloon(bestaand: readonly string[]): Sjabloon {
  let n = 1;
  while (bestaand.includes(`${EIGEN}${n}`)) n++;
  const vlak = nieuwBlok("Vlak");
  const kop = nieuwBlok("Tekst");
  if (kop.soort.blok === "Tekst") kop.soort.bron = { Veld: "kop" };
  kop.plek = { x: 140, y: 790, breedte: 1000, hoogte: 100 };
  kop.animatie.start_na_ms = 200;
  return {
    id: `${EIGEN}${n}`,
    naam: `Eigen Sjabloon ${n}`,
    groep: GROEP_EIGEN,
    snelheid: 100,
    blokken: [vlak, kop],
    achtergrond: { soort: "Zaal" },
    stoelen: "Houden",
  };
}

/** De acht gewone Blokken waaruit de omroep een Sjabloon samenstelt (#11), met hun naam. */
export const GEWONE_BLOKKEN: { soort: Bloksoort["blok"]; naam: string }[] = [
  { soort: "Vlak", naam: "Vlak" },
  { soort: "Tekst", naam: "Tekst" },
  { soort: "Getal", naam: "Getal" },
  { soort: "Zetelbalk", naam: "Zetelbalk" },
  { soort: "Partijlogo", naam: "Partijlogo" },
  { soort: "FotoKamerlid", naam: "Foto Kamerlid" },
  { soort: "Lijn", naam: "Lijn" },
  { soort: "Programmalogo", naam: "Programmalogo" },
];

/** De naam van elke soort Blok, ook de grote kant-en-klare. */
export const BLOKNAMEN: Record<Bloksoort["blok"], string> = {
  Vlak: "Vlak",
  Tekst: "Tekst",
  Getal: "Getal",
  Zetelbalk: "Zetelbalk",
  Partijlogo: "Partijlogo",
  FotoKamerlid: "Foto Kamerlid",
  Partijregel: "Partijregel",
  Lijn: "Lijn",
  Programmalogo: "Programmalogo",
  Afbeelding: "Afbeelding",
  Partijlabels: "Partijlabels",
  Opbouwlabel: "Opbouwlabel",
  Stemmingsbalk: "Stemmingsbalk",
  Uitkomst: "Uitkomst",
  Fractielijst: "Fractielijst",
  Namenlijst: "Namenlijst",
  Uitslaggrafiek: "Uitslaggrafiek",
  WinstEnVerliesgrafiek: "Winst-en-verliesgrafiek",
};

const vervagen = (start_na_ms = 0) => ({ erin: "Vervagen" as Erin, eruit: "Vervagen" as Eruit, start_na_ms });

/** Een nieuw Blok van deze soort, op een plek waar het te zien is, in de kleuren van de Stijl. */
export function nieuwBlok(soort: Bloksoort["blok"]): Blok {
  const kop = { Stijl: "Kop" } as const;
  const cijfers = { Stijl: "Cijfers" } as const;
  const tekst = { Stijl: "Tekst" } as const;
  switch (soort) {
    case "Vlak":
      return { plek: { x: 96, y: 760, breedte: 1100, hoogte: 200 }, soort: { blok: "Vlak", hoek: 0, dekking: 93, draaiing: 0 }, kleur: { Stijl: "Vlak" }, animatie: { erin: "OpenenVanuitLijn", eruit: "SluitenNaarLijn", start_na_ms: 0 } };
    case "Tekst":
      return { plek: { x: 140, y: 800, breedte: 1000, hoogte: 70 }, soort: { blok: "Tekst", bron: { Vast: "Tekst" }, letter: tekst, grootte: 48, uitlijning: "Links", een_regel: false }, kleur: { Stijl: "Tekst" }, animatie: vervagen(200) };
    case "Getal":
      return { plek: { x: 140, y: 780, breedte: 300, hoogte: 140 }, soort: { blok: "Getal", bron: { Veld: "getal" }, letter: cijfers, grootte: 120, uitlijning: "Links" }, kleur: { Stijl: "Tekst" }, animatie: { erin: "Optellen", eruit: "Vervagen", start_na_ms: 200 } };
    case "Zetelbalk":
      return { plek: { x: 144, y: 872, breedte: 1632, hoogte: 52 }, soort: { blok: "Zetelbalk", letter: cijfers, lijn_bij_76: true, naamletter: tekst, labels: true, vorige_set: false, pijlen: false, coalitie: false }, kleur: { Stijl: "Accent" }, animatie: { erin: "Groeien", eruit: "Vervagen", start_na_ms: 200 } };
    case "Partijlogo":
      return { plek: { x: 140, y: 780, breedte: 160, hoogte: 100 }, soort: { blok: "Partijlogo", bron: "Partij", hoek: 4 }, kleur: { Stijl: "Tekst" }, animatie: vervagen(200) };
    case "FotoKamerlid":
      return { plek: { x: 1500, y: 300, breedte: 300, hoogte: 360 }, soort: { blok: "FotoKamerlid", hoek: 0, letter: kop }, kleur: { Stijl: "Tekst" }, animatie: vervagen(200) };
    case "Lijn":
      return { plek: { x: 140, y: 880, breedte: 1000, hoogte: 3 }, soort: { blok: "Lijn", dekking: 100 }, kleur: { Stijl: "Tekst" }, animatie: { erin: "Opbouwen", eruit: "Vervagen", start_na_ms: 200 } };
    case "Programmalogo":
      return { plek: { x: 1448, y: 46, breedte: 400, hoogte: 131 }, soort: { blok: "Programmalogo", letter: kop, dekking: 90 }, kleur: { Stijl: "Tekst" }, animatie: vervagen() };
    default:
      // de grote Blokken komen met hun vaste Sjabloon; een kopie van zo'n Blok kan wel
      throw new Error(`${soort} is geen gewoon Blok`);
  }
}

/** Wat elke soort achtergrond heet. */
export const ACHTERGRONDEN = [
  { soort: "Zaal", naam: "Zaal" },
  { soort: "ZaalDonker", naam: "Zaal, donker en onscherp" },
  { soort: "Kleur", naam: "Eigen kleur" },
  { soort: "Afbeelding", naam: "Eigen afbeelding" },
] as const;

/** Wat de Stoelen kunnen doen. */
export const STOELEN: { stoelen: Stoelen; naam: string }[] = [
  { stoelen: "Gegevens", naam: "Kleuren volgens de gegevens" },
  { stoelen: "PartijUitgelicht", naam: "Gekozen Partij licht op, de rest grijs" },
  { stoelen: "Houden", naam: "Houden hun kleur" },
  { stoelen: "Uit", naam: "Uit: hun eigen blauwe leer" },
];

export const ERIN: { erin: Erin; naam: string }[] = [
  { erin: "Geen", naam: "Meteen" },
  { erin: "Vervagen", naam: "Vervagen" },
  { erin: "VanLinks", naam: "Invliegen van links" },
  { erin: "VanRechts", naam: "Invliegen van rechts" },
  { erin: "VanOnder", naam: "Invliegen van onder" },
  { erin: "Opbouwen", naam: "Opbouwen" },
  { erin: "Groeien", naam: "Groeien" },
  { erin: "Optellen", naam: "Optellen" },
  { erin: "OpenenVanuitLijn", naam: "Openen vanuit lijn" },
];

export const ERUIT: { eruit: Eruit; naam: string }[] = [
  { eruit: "Vervagen", naam: "Vervagen" },
  { eruit: "NaarLinks", naam: "Wegvliegen naar links" },
  { eruit: "NaarRechts", naam: "Wegvliegen naar rechts" },
  { eruit: "NaarOnder", naam: "Wegvliegen naar onder" },
  { eruit: "Inklappen", naam: "Inklappen" },
  { eruit: "SluitenNaarLijn", naam: "Sluiten naar lijn" },
];

/** De bron van een Tekst als korte tekst voor de editor. */
export function tekstbronNaam(b: Tekstbron): string {
  if (typeof b === "object") return "Vast" in b ? b.Vast : `invulveld "${b.Veld}"`;
  return TEKSTBRONNEN.find((t) => t.bron === b)?.naam ?? b;
}

/** De bronnen van een Tekst die geen vaste tekst of invulveld zijn. */
export const TEKSTBRONNEN: { bron: Exclude<Tekstbron, object>; naam: string }[] = [
  { bron: "Partijnaam", naam: "Naam van de gekozen Partij" },
  { bron: "Setnaam", naam: "Naam van de gekozen set" },
  { bron: "VorigeSetnaam", naam: "Naam van de vorige set" },
  { bron: "ZetelsTegenover", naam: "Zetels tegenover de vergelijkingsset" },
  { bron: "Coalitiepartijen", naam: "Partijen van de coalitie" },
  { bron: "Coalitieregel", naam: "Tekort voor 76" },
  { bron: "Persoonsnaam", naam: "Naam van de persoon" },
  { bron: "PartijVanPersoon", naam: "Partij van de persoon" },
];

/** Een korte beschrijving van een Blok voor de lijst in de editor. */
export function blokTekst(b: Blok): string {
  const s = b.soort;
  switch (s.blok) {
    case "Tekst":
      return tekstbronNaam(s.bron);
    case "Getal":
      return typeof s.bron === "object" ? `invulveld "${s.bron.Veld}"` : s.bron === "Zetels" ? "Zetels van de Partij" : s.bron === "Winst" ? "Winst of verlies" : "Teller van de coalitie";
    case "Vlak":
      return `${s.dekking}%${s.om_tekst ? ", om de tekst" : ""}${s.draaiing ? `, ${s.draaiing}°` : ""}`;
    default:
      return "";
  }
}

/** Een JSON-waarde zonder lege velden: `null` en `undefined` tellen als hetzelfde. */
function zonderLeeg(w: unknown): unknown {
  if (Array.isArray(w)) return w.map(zonderLeeg);
  if (w && typeof w === "object")
    return Object.fromEntries(
      Object.entries(w)
        .filter(([, v]) => v !== null && v !== undefined)
        .sort(([a], [b]) => (a < b ? -1 : 1))
        .map(([k, v]) => [k, zonderLeeg(v)]),
    );
  return w;
}

/** Of `concept` iets anders is dan `opgeslagen`. */
export function gewijzigdSjabloon(concept: Sjabloon, opgeslagen: Sjabloon): boolean {
  return JSON.stringify(zonderLeeg(concept)) !== JSON.stringify(zonderLeeg(opgeslagen));
}

/** Waarom dit Sjabloon niet opgeslagen kan worden, of `null` (een deel van `Sjabloon::controleer`). */
export function fout(s: Sjabloon): string | null {
  if (!s.naam.trim()) return "Het Sjabloon heeft geen naam";
  if ([...s.naam].length > MAX_SJABLOONNAAM) return `De naam is langer dan ${MAX_SJABLOONNAAM} tekens`;
  if (!Number.isInteger(s.snelheid) || s.snelheid < MIN_SNELHEID || s.snelheid > MAX_SNELHEID)
    return `Kies een snelheid tussen ${MIN_SNELHEID} en ${MAX_SNELHEID} procent`;
  if (s.blokken.length > MAX_BLOKKEN) return `Meer dan ${MAX_BLOKKEN} Blokken`;
  const overgang = s.kleurovergang && foutKleurovergang(s.kleurovergang);
  if (overgang) return overgang;
  if (s.achtergrond.soort === "Afbeelding" && !s.achtergrond.bestand) return "Kies een afbeelding voor de achtergrond";
  const leeg = s.blokken.findIndex((b) => veldnamen(b).some((v) => !v.trim()));
  if (leeg >= 0) return `Blok ${leeg + 1} heeft een invulveld zonder naam`;
  const maat = s.blokken.findIndex((b) => !(b.plek.breedte >= 0 && b.plek.hoogte >= 0));
  if (maat >= 0) return `Blok ${maat + 1} heeft geen geldige grootte`;
  return null;
}

/** De namen van de invulvelden die dit Blok vraagt. */
function veldnamen(b: Blok): string[] {
  const s = b.soort;
  const uit: string[] = [];
  if ((s.blok === "Tekst" || s.blok === "Getal") && typeof s.bron === "object" && "Veld" in s.bron) uit.push(s.bron.Veld);
  if (s.blok === "Partijregel" && typeof s.tekst === "object" && "Veld" in s.tekst) uit.push(s.tekst.Veld);
  return uit;
}

/** Wat de Regie invult bij een Graphic van dit Sjabloon: de invulvelden (op volgorde, zonder
 * dubbele), en of het een set en een Partij nodig heeft. */
export function benodigd(s: Sjabloon): { velden: string[]; set: boolean; partij: boolean } {
  const velden = [...new Set(s.blokken.flatMap(veldnamen))];
  let set = false;
  let partij = false;
  for (const { soort: b } of s.blokken) {
    const tekstbron = b.blok === "Tekst" ? b.bron : b.blok === "Partijregel" ? b.tekst : null;
    if (b.blok === "Partijlogo" || b.blok === "Partijregel" || tekstbron === "Partijnaam") partij = true;
    if (b.blok === "Getal" && (b.bron === "Zetels" || b.bron === "Winst")) set = partij = true;
    if (b.blok === "Zetelbalk" || b.blok === "Uitslaggrafiek" || b.blok === "WinstEnVerliesgrafiek" || b.blok === "Partijlabels" || b.blok === "Opbouwlabel") set = true;
    if (tekstbron === "Setnaam" || tekstbron === "VorigeSetnaam" || tekstbron === "ZetelsTegenover") set = true;
  }
  return { velden, set, partij };
}

/** De verwijzingen van een Vlak naar andere Blokken (om de tekst, bij een tekst) omgezet met `nieuw`;
 * `null` uit `nieuw`: de verwijzing valt weg. */
function verwijs(s: Sjabloon, nieuw: (i: number) => number | null): Sjabloon {
  const blokken = s.blokken.map((b) => {
    if (b.soort.blok !== "Vlak") return b;
    const soort = { ...b.soort };
    if (soort.om_tekst) {
      const blokken = soort.om_tekst.blokken.map(nieuw).filter((i): i is number => i !== null);
      soort.om_tekst = blokken.length ? { ...soort.om_tekst, blokken } : undefined;
    }
    if (soort.bij_tekst != null) soort.bij_tekst = nieuw(soort.bij_tekst) ?? undefined;
    return { ...b, soort };
  });
  return { ...s, blokken };
}

/** Het Sjabloon zonder Blok `i`. Een naambalk om dat Blok past daarna om de rest. */
export function wisBlok(s: Sjabloon, i: number): Sjabloon {
  const zonder = { ...s, blokken: s.blokken.filter((_, n) => n !== i) };
  return verwijs(zonder, (n) => (n === i ? null : n > i ? n - 1 : n));
}

/** Het Sjabloon met Blok `van` op plaats `naar` (0 ligt achteraan). Verwijzingen schuiven mee. */
export function verplaatsBlok(s: Sjabloon, van: number, naar: number): Sjabloon {
  if (naar < 0 || naar >= s.blokken.length || van === naar) return s;
  const blokken = [...s.blokken];
  const [blok] = blokken.splice(van, 1);
  blokken.splice(naar, 0, blok);
  return verwijs({ ...s, blokken }, (n) => {
    if (n === van) return naar;
    if (van < naar && n > van && n <= naar) return n - 1;
    if (naar < van && n >= naar && n < van) return n + 1;
    return n;
  });
}
