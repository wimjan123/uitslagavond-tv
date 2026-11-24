// Presets in het dashboard: groep, stempels en filters (DESIGN.md, Preset Table). Een Preset is een
// kopie van een Graphic met de gegevens van het moment van bewaren; de server houdt hem bij.
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

export interface PresetStatus {
  /** De Graphic die klaarstaat komt uit deze Preset. */
  klaar: boolean;
  /** De Graphic in de Ether komt uit deze Preset. */
  live: boolean;
  /** Wanneer een Graphic uit deze Preset het laatst getoond werd. */
  wasLiveOm: number | null;
}

/** Of deze Graphic de ongewijzigde uit de Preset is: id en versie gelijk aan `als`. */
function ongewijzigd(g: { id: number; versie: number } | null | undefined, als: number | undefined): boolean {
  return als !== undefined && !!g && g.id === als && g.versie === als;
}

/** Gelijk aan `Stand::preset_staat_klaar` en `Stand::preset_in_beeld`: een gewijzigde Graphic is de Preset niet meer. */
export function statusVan(p: Preset, s: Stand): PresetStatus {
  return {
    klaar: ongewijzigd(s.klaar, p.klaargezet_als),
    live: ongewijzigd(s.ether, p.getoond_als),
    wasLiveOm: p.laatst_live_ms ?? null,
  };
}

/** De Presets van één groep (of alle, bij `null`), eventueel alleen die op de tablet. */
export function filterPresets(lijst: Preset[], groep: Groep | null, opTablet: boolean): Preset[] {
  return lijst.filter((p) => (groep === null || groepVan(p.graphic) === groep) && (!opTablet || p.op_tablet));
}

/** Het nummer van de Preset die deze Graphic met dit Standpunt al bewaart, of `null`. */
export function bewaardAls(lijst: Preset[], graphic: Graphic, standpunt: string | undefined): number | null {
  const g = JSON.stringify(graphic);
  return lijst.find((p) => JSON.stringify(p.graphic) === g && (p.standpunt ?? null) === (standpunt ?? null))?.nummer ?? null;
}
