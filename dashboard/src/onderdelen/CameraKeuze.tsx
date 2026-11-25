// De camera bij Take, in elk formulier van Maken: de drie keuzes die er toe doen staan zichtbaar naast
// elkaar (van het Sjabloon, Camera blijft staan, zelf kiezen), en pas bij "Kies" de lijst.
import { BLIJF } from "../regie/camerarit";

export interface CameraOptie {
  waarde: string;
  tekst: string;
  /** Voor de groepen in de lijst; zonder groep staat de optie los. */
  soort?: "Standpunt" | "Camerarit";
}

export function CameraKeuze({
  waarde,
  zet,
  standaard,
  opties,
}: {
  waarde: string;
  zet: (w: string) => void;
  /** Het Standpunt van het Sjabloon, zoals "A · Totaal vanaf voorzitter". */
  standaard: string | null;
  /** Alle Standpunten en Cameraritten, zonder "van het Sjabloon" en zonder "Camera blijft staan". */
  opties: CameraOptie[];
}) {
  const lijst = opties.filter((o) => o.waarde !== BLIJF && o.waarde !== "");
  const kiest = waarde !== "" && waarde !== BLIJF;
  const groepen = (["Standpunt", "Camerarit"] as const).filter((g) => lijst.some((o) => o.soort === g));
  return (
    <div className="veld camera-keuze">
      <span className="label">Camera bij Take</span>
      <div className="segment" role="group" aria-label="Camera bij Take">
        <button type="button" aria-pressed={waarde === ""} onClick={() => zet("")} title={standaard ?? undefined}>
          Sjabloon
        </button>
        <button type="button" aria-pressed={waarde === BLIJF} onClick={() => zet(BLIJF)} title="De Live-camera blijft waar hij is; alleen de Graphic en de Stoelen veranderen">
          Blijft staan
        </button>
        <button type="button" aria-pressed={kiest} onClick={() => !kiest && lijst[0] && zet(lijst[0].waarde)}>
          Kies
        </button>
      </div>
      {waarde === "" && standaard && <span className="zacht een-regel">{standaard}</span>}
      {kiest && (
        <select value={waarde} onChange={(e) => zet(e.target.value)} aria-label="Standpunt of Camerarit">
          {groepen.length > 0
            ? groepen.map((g) => (
                <optgroup key={g} label={g === "Standpunt" ? "Standpunten" : "Cameraritten"}>
                  {lijst
                    .filter((o) => o.soort === g)
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
      )}
    </div>
  );
}
