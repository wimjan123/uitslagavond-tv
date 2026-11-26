// Toon, Bijwerken, Uit beeld, Kleur uit, Alles uit en Camera/Stoelbeeld-lagen: dezelfde handelingen
// voor knoppen en sneltoetsen (ADR 0005).
import { meerderheidSlaatOm } from "../coalitie/coalitie";
import { graphicTekst, nieuwereCijfers, sjabloonId, sjabloonNaam } from "../graphic";
import { biedHerstel, herstel } from "./herstel";
import { geldendSjabloon, gewijzigdSjabloon } from "../vormgeving/sjablonen";
import type { OvergangSoort } from "../typen/OvergangSoort";
import type { Stand } from "../typen/Stand";
import type { Stoelbeeld } from "../typen/Stoelbeeld";
import type { Regie } from "./regie";
import { standpuntVoorToets } from "./toetsen";
import type { Actie, TakeActie } from "./toetsen";

export const TAKE = "take";
export const CAMERA = "camera";
export const PRESETS = "presets";

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

function zelfdeSjabloon(s: Stand): boolean {
  const kopie = s.ether?.sjabloon;
  const nu = s.klaar && geldendSjabloon(s.sjablonen, sjabloonId(s.klaar.graphic));
  return !kopie || (!!nu && !gewijzigdSjabloon(kopie, nu));
}

export function kanBijwerken(s: Stand): boolean {
  return !!s.klaar && !!s.ether && s.klaar.id === s.ether.id && (s.klaar.versie !== s.ether.versie || nieuwereCijfers(s, s.klaar.graphic));
}

/** Of er minstens één laag (Graphic, Stoelbeeld of Camera) klaarstaat voor Take. */
export function heeftIetsKlaar(s: Stand | undefined): boolean {
  if (!s) return false;
  const graphicVerandert = !!s.klaar && !klaarStaatInBeeld(s);
  return graphicVerandert || !!s.klaar_stoelbeeld || !!s.camera.klaar_standpunt;
}

export function kan(actie: TakeActie, s: Stand | undefined): boolean {
  if (!s) return false;
  switch (actie) {
    case "Toon":
      return heeftIetsKlaar(s);
    case "Bijwerken":
      return kanBijwerken(s) && !meerderheidSlaatOm(s);
    case "UitBeeld":
      return !!s.ether;
    case "KleurUit":
      return s.stoelkleuren.length > 0;
    case "SlaOpbouwOver":
      return s.stoelkleuren.length > 0 && s.kleurovergang.vorm === "Opbouw";
    case "Ververs":
      return !!s.klaar && nieuwereCijfers(s, s.klaar.graphic);
  }
}

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
      if (s) void regie.stuur({ soort: "ZetOpnemen", aan: !s.camera.opname }, CAMERA);
      return;
    case "Toon":
      if (!s || !heeftIetsKlaar(s)) return;
      void regie.stuur({ soort: "Toon", versie: s.klaar?.versie }, TAKE);
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
      if (s?.klaar && kan("Ververs", s)) {
        void regie.stuur({ soort: "WijzigKlaar", id: s.klaar.id, graphic: s.klaar.graphic, standpunt: s.klaar.standpunt }, TAKE);
      }
      return;
  }
}

export function kiesStandpunt(regie: Regie, letter: string, knip = false): void {
  void regie.stuur({ soort: "KiesStandpunt", letter, knip }, CAMERA);
}

export function zetCameraKlaar(regie: Regie, letter: string | null): void {
  void regie.stuur({ soort: "ZetCameraKlaar", letter: letter ?? undefined }, CAMERA);
}

export function zetOvergang(regie: Regie, overgang: OvergangSoort | null, overvloei_ms?: number): void {
  void regie.stuur({ soort: "ZetOvergang", overgang: overgang ?? undefined, overvloei_ms }, TAKE);
}

export function zetStoelbeeldKlaar(regie: Regie, stoelbeeld: Stoelbeeld | null): void {
  void regie.stuur({ soort: "ZetStoelbeeldKlaar", stoelbeeld: stoelbeeld ?? undefined }, CAMERA);
}

export function zetStoelbeeldDirect(regie: Regie, stoelbeeld: Stoelbeeld): void {
  void regie.stuur({ soort: "ZetStoelbeeldDirect", stoelbeeld }, CAMERA);
}

export function wisKlaar(regie: Regie): void {
  void regie.stuur({ soort: "WisKlaar" }, TAKE);
}

export async function allesUit(regie: Regie): Promise<void> {
  const s = regie.stand?.stand;
  const ether = s?.ether;
  const a = await regie.stuur({ soort: "AllesUit" }, TAKE);
  if (!a.uitgevoerd || !ether || !s) return;
  biedHerstel({
    tekst: "Alles uit",
    wat: `${sjabloonNaam(ether.graphic, s.sjablonen)} · ${graphicTekst(ether.graphic)}`,
    knop: "Terug in beeld",
    geldig: (nu) => !nu.ether,
    doe: (r) => void r.stuur({ soort: "TerugInBeeld", graphic: ether.graphic, standpunt: ether.standpunt }, TAKE),
  });
}

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
