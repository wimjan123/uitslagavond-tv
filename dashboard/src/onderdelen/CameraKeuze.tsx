// De camera bij Take, in elk formulier van Maken: de drie keuzes die er toe doen staan zichtbaar naast
// elkaar (van het Sjabloon, Camera blijft staan, Kies), en bij Kies alleen de bewegingen, Standpunten
// en Cameraritten die bij deze Graphic passen.
import { BLIJF } from "../regie/camerarit";
import type { Camerakeus } from "../regie/camerarit";

export interface CameraOptie {
  waarde: string;
  tekst: string;
  /** Voor de groepen in de lijst; zonder groep staat de optie los. */
  soort?: Camerakeus["soort"];
  toelichting?: string;
}

const GROEP_VOLGORDE: { soort: Camerakeus["soort"]; label: string }[] = [
  { soort: "Partijbeweging", label: "Bewegingen naar de Partij" },
  { soort: "Stoelbeweging", label: "Bewegingen naar de Stoel" },
  { soort: "Standpunt", label: "Standpunten" },
  { soort: "Camerarit", label: "Cameraritten" },
];

export function CameraKeuze({
  waarde,
  zet,
  standaard,
  opties,
}: {
  waarde: string;
  zet: (w: string) => void;
  /** Het Standpunt of de beweging van het Sjabloon, zoals "A · Totaal vanaf voorzitter" of "Aanschuiven naar de Partij · 8 s". */
  standaard: string | null;
  /** De passende bewegingen, Standpunten en Cameraritten, zonder "van het Sjabloon" en zonder "Camera blijft staan". */
  opties: CameraOptie[];
}) {
  const lijst = opties.filter((o) => o.waarde !== BLIJF && o.waarde !== "");
  const effectief = waarde === "" || waarde === BLIJF || lijst.some((o) => o.waarde === waarde) ? waarde : "";
  const kiest = effectief !== "" && effectief !== BLIJF;
  const groepen = GROEP_VOLGORDE.filter((g) => lijst.some((o) => o.soort === g.soort));
  const gekozenOptie = kiest ? lijst.find((o) => o.waarde === effectief) : undefined;
  return (
    <div className="veld camera-keuze">
      <span className="label">Camera bij Take</span>
      <div className="segment" role="group" aria-label="Camera bij Take">
        <button type="button" aria-pressed={effectief === ""} onClick={() => zet("")} title={standaard ?? undefined}>
          Van het Sjabloon
        </button>
        <button
          type="button"
          aria-pressed={effectief === BLIJF}
          onClick={() => zet(BLIJF)}
          title="De Live-camera blijft waar hij is; alleen de Graphic en de Stoelen veranderen"
        >
          Blijft staan
        </button>
        <button
          type="button"
          aria-pressed={kiest}
          disabled={lijst.length === 0}
          onClick={() => !kiest && lijst[0] && zet(lijst[0].waarde)}
        >
          Kies
        </button>
      </div>
      {effectief === "" && standaard && <span className="zacht een-regel">{standaard}</span>}
      {kiest && (
        <>
          <select value={effectief} onChange={(e) => zet(e.target.value)} aria-label="Camerabeweging of Standpunt">
            {groepen.length > 0
              ? groepen.map((g) => (
                  <optgroup key={g.soort} label={g.label}>
                    {lijst
                      .filter((o) => o.soort === g.soort)
                      .map((o) => (
                        <option key={o.waarde} value={o.waarde}>
                          {o.tekst}
                        </option>
                      ))}
                  </optgroup>
                ))
              : lijst.map((o) => (
                  <option key={o.waarde} value={o.waarde}>
                    {o.tekst}
                  </option>
                ))}
          </select>
          {gekozenOptie?.toelichting && <span className="zacht een-regel">{gekozenOptie.toelichting}</span>}
        </>
      )}
    </div>
  );
}
