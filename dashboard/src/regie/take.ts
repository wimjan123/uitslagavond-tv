// Toon, Bijwerken, Uit beeld, Kleur uit en Standpunten: dezelfde handelingen voor de knoppen en de sneltoetsen.
import { graphicTekst, nieuwereCijfers, sjabloonId, sjabloonNaam } from "../graphic";
import { biedHerstel, herstel } from "./herstel";
import { geldendSjabloon, gewijzigdSjabloon } from "../vormgeving/sjablonen";
import type { Stand } from "../typen/Stand";
import type { Regie } from "./regie";
import { standpuntVoorToets } from "./toetsen";
import type { Actie, TakeActie } from "./toetsen";

export const TAKE = "take";
/** Waar een weigering bij een Standpunt komt te staan: in het camerablok. */
export const CAMERA = "camera";
/** Waar een weigering bij een Preset komt te staan: in het onderdeel Presets. */
export const PRESETS = "presets";

/** Of precies wat klaarstaat ook in de Ether staat, in dezelfde Stijl en hetzelfde Sjabloon (gelijk
 * aan `Stand::staat_klaar_in_beeld`). Beide Stijlen komen uit dezelfde serde-uitvoer, dus de JSON is gelijk
 * als de Stijlen gelijk zijn. */
export function klaarStaatInBeeld(s: Stand): boolean {
  return (
    !!s.klaar &&
    !!s.ether &&
    s.klaar.id === s.ether.id &&
    s.klaar.versie === s.ether.versie &&
    JSON.stringify(s.ether.stijl) === JSON.stringify(s.stijl) &&
    zelfdeSjabloon(s)
  );
}

/** Of de Graphic in Live het Sjabloon heeft dat nu geldt voor wat klaarstaat. */
function zelfdeSjabloon(s: Stand): boolean {
  const kopie = s.ether?.sjabloon;
  const nu = s.klaar && geldendSjabloon(s.sjablonen, sjabloonId(s.klaar.graphic));
  return !kopie || (!!nu && !gewijzigdSjabloon(kopie, nu));
}

/** Of de klaargezette Graphic een nieuwere versie is van die in de Ether, of nieuwere cijfers heeft
 * (gelijk aan `Stand::kan_bijwerken`). */
export function kanBijwerken(s: Stand): boolean {
  return !!s.klaar && !!s.ether && s.klaar.id === s.ether.id && (s.klaar.versie !== s.ether.versie || nieuwereCijfers(s, s.klaar.graphic));
}

export function kan(actie: TakeActie, s: Stand | undefined): boolean {
  if (!s) return false;
  switch (actie) {
    case "Toon":
      return !!s.klaar && !klaarStaatInBeeld(s);
    case "Bijwerken":
      return kanBijwerken(s);
    case "UitBeeld":
      return !!s.ether;
    case "KleurUit":
      return s.stoelkleuren.length > 0;
    case "SlaOpbouwOver":
      // gelijk aan de Weigering GeenOpbouw; of de Opbouw al klaar is, weet alleen de Renderer
      return s.stoelkleuren.length > 0 && s.kleurovergang.vorm === "Opbouw";
    case "Ververs":
      return !!s.klaar && nieuwereCijfers(s, s.klaar.graphic);
  }
}

/** Voert de handeling uit. Toon stuurt mee welke versie dit scherm liet zien. */
export function voerUit(regie: Regie, actie: Actie): void {
  const s = regie.stand?.stand;
  if (typeof actie === "object") {
    if (actie.soort === "Preset") {
      void regie.stuur({ soort: "ZetPresetKlaar", nummer: actie.nummer }, PRESETS);
      return;
    }
    const p = standpuntVoorToets([...(s?.standpunten ?? []), ...(s?.cameraritten ?? [])], actie.toets);
    if (p) kiesStandpunt(regie, p.letter, actie.knip);
    return;
  }
  switch (actie) {
    case "Herstel":
      herstel(regie);
      return;
    case "VrijeCamera":
      if (s) void regie.stuur({ soort: "ZetVrijeCamera", aan: !s.camera.vrij }, CAMERA);
      return;
    case "Oefenen":
      if (s) void regie.stuur({ soort: "ZetOefenen", aan: !s.camera.oefenen }, CAMERA);
      return;
    case "Opnemen":
      // stoppen: de server vult zelf in wat hij opnam
      if (s) void regie.stuur({ soort: "ZetOpnemen", aan: !s.camera.opname }, CAMERA);
      return;
    case "Toon":
      if (!s?.klaar) return;
      void regie.stuur({ soort: "Toon", versie: s.klaar.versie }, TAKE);
      return;
    case "Bijwerken":
      void regie.stuur({ soort: "Bijwerken" }, TAKE);
      return;
    case "UitBeeld":
      void uitBeeld(regie);
      return;
    case "KleurUit":
      void regie.stuur({ soort: "KleurUit" }, TAKE);
      return;
    case "SlaOpbouwOver":
      void regie.stuur({ soort: "SlaOpbouwOver" }, TAKE);
      return;
    case "Ververs":
      // Een Preset houdt de cijfers van het moment van bewaren; WijzigKlaar zet de cijfers van nu erin.
      if (s?.klaar && kan("Ververs", s)) {
        void regie.stuur({ soort: "WijzigKlaar", id: s.klaar.id, graphic: s.klaar.graphic, standpunt: s.klaar.standpunt }, TAKE);
      }
      return;
  }
}

/** Stuur de Live-camera naar het Standpunt of de Camerarit met deze sleutel; zet de Vrije camera uit. */
export function kiesStandpunt(regie: Regie, letter: string, knip = false): void {
  void regie.stuur({ soort: "KiesStandpunt", letter, knip }, CAMERA);
}

/** Haalt de Graphic uit Live en biedt "Terug in beeld" aan. Dat zet precies dezelfde Graphic terug,
 * met de cijfers die erin zaten, en laat wat klaarstaat ongemoeid (Opdracht TerugInBeeld). Het geldt
 * zolang Live leeg is; toont iemand intussen iets, dan vervalt het. */
async function uitBeeld(regie: Regie): Promise<void> {
  const s = regie.stand?.stand;
  const ether = s?.ether;
  const a = await regie.stuur({ soort: "UitBeeld" }, TAKE);
  if (!a.uitgevoerd || !ether || !s) return;
  biedHerstel({
    tekst: "Take out",
    wat: `${sjabloonNaam(ether.graphic, s.sjablonen)} · ${graphicTekst(ether.graphic)}`,
    knop: "Terug in beeld",
    geldig: (nu) => !nu.ether,
    doe: (r) => void r.stuur({ soort: "TerugInBeeld", graphic: ether.graphic, standpunt: ether.standpunt }, TAKE),
  });
}
