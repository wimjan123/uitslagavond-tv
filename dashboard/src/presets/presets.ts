// Presets in het dashboard: groep, stempels, filters en titels (ook voor Stoelbeeld-/camera-presets).
import { graphicTekst, sjabloonNaam } from "../graphic.ts";
import type { Graphic } from "../typen/Graphic";
import type { Preset } from "../typen/Preset";
import type { Stand } from "../typen/Stand";

/** De vier groepen van de vaste Sjablonen (issue #6), in de volgorde van de filters. */
export const GROEPEN = ["Uitslagavond", "Coalitiebouwer", "Stemmingen", "Algemeen"] as const;
export type Groep = (typeof GROEPEN)[number];

export function groepVan(g: Graphic): Groep {
  switch (g.sjabloon) {
    case "Titelkaart":
    case "Persoonskaart":
    case "Eigen":
      return "Algemeen";
    case "Zetelverdeling":
    case "Zetelbalk":
    case "TweeSets":
    case "UitslagPerPartij":
    case "WinstEnVerlies":
    case "PartijUitgelicht":
      return "Uitslagavond";
    case "Coalitie":
    case "Coalitiebalk":
      return "Coalitiebouwer";
    case "UitslagStemming":
    case "HoofdelijkeStemming":
    case "FractiesVoorEnTegen":
      return "Stemmingen";
  }
}

export function groepVanPreset(p: Preset): Groep {
  if (p.zonder_graphic) {
    if (p.stoelbeeld?.bron.soort === "Coalitie") return "Coalitiebouwer";
    if (p.stoelbeeld?.bron.soort === "Stemming") return "Stemmingen";
    if (p.stoelbeeld) return "Uitslagavond";
    return "Algemeen";
  }
  return groepVan(p.graphic);
}

export function presetTitel(p: Preset, stand?: Stand): string {
  if (p.naam?.trim()) return p.naam.trim();
  if (p.zonder_graphic) {
    if (p.stoelbeeld?.label?.trim()) return p.stoelbeeld.label.trim();
    if (p.stoelbeeld) return "Stoelbeeld";
    return p.standpunt ? `Camera ${p.standpunt}` : `Preset ${p.nummer}`;
  }
  const tekst = graphicTekst(p.graphic).trim();
  return tekst || sjabloonNaam(p.graphic, stand?.sjablonen);
}

export interface PresetStatus {
  klaar: boolean;
  live: boolean;
  wasLiveOm: number | null;
}

function ongewijzigd(g: { id: number; versie: number } | null | undefined, als: number | undefined): boolean {
  return als !== undefined && !!g && g.id === als && g.versie === als;
}

export function statusVan(p: Preset, s: Stand): PresetStatus {
  if (p.zonder_graphic) {
    const zelfdeStoel = !!p.stoelbeeld && JSON.stringify(s.klaar_stoelbeeld) === JSON.stringify(p.stoelbeeld);
    const zelfdeCam = !!p.standpunt && s.camera.klaar_standpunt === p.standpunt;
    const liveStoel = !!p.stoelbeeld && JSON.stringify(s.ether_stoelbeeld) === JSON.stringify(p.stoelbeeld);
    return {
      klaar: !s.klaar && (zelfdeStoel || zelfdeCam),
      live: liveStoel,
      wasLiveOm: p.laatst_live_ms ?? null,
    };
  }
  return {
    klaar: ongewijzigd(s.klaar, p.klaargezet_als),
    live: ongewijzigd(s.ether, p.getoond_als),
    wasLiveOm: p.laatst_live_ms ?? null,
  };
}

export function filterPresets(lijst: Preset[], groep: Groep | null, opTablet: boolean): Preset[] {
  return lijst.filter((p) => (groep === null || groepVanPreset(p) === groep) && (!opTablet || p.op_tablet));
}

export function bewaardAls(lijst: Preset[], graphic: Graphic, standpunt: string | undefined): number | null {
  const g = JSON.stringify(graphic);
  return lijst.find((p) => !p.zonder_graphic && JSON.stringify(p.graphic) === g && (p.standpunt ?? null) === (standpunt ?? null))?.nummer ?? null;
}
