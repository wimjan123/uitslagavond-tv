// Golf, Vervagen of Opbouw en hoe lang: voor de Stijl en voor een Sjabloon met een eigen Kleurovergang.
import type { Kleurovergang } from "../typen/Kleurovergang";
import { metVorm, vormenVoor } from "../vormgeving/kleurovergang";

/** De langste duur op de schuif, in milliseconden; via de Opdracht kan tot 10 seconden. */
const SCHUIF_MAX_MS = 5000;

/** `sjabloon`: de keuze in een Sjabloon, met de Opbouw erbij. */
export function KleurovergangKeuze({ waarde, zet, sjabloon = false }: { waarde: Kleurovergang; zet(k: Kleurovergang): void; sjabloon?: boolean }) {
  const vormen = vormenVoor(sjabloon);
  const uitleg = vormen.find((v) => v.vorm === waarde.vorm)?.uitleg;
  return (
    <div className="kleurovergang">
      <div className="segment segment-klein" role="group" aria-label="Vorm van de Kleurovergang">
        {vormen.map((v) => (
          <button key={v.vorm} type="button" aria-pressed={v.vorm === waarde.vorm} title={v.uitleg} onClick={() => v.vorm !== waarde.vorm && zet(metVorm(v.vorm))}>
            {v.naam}
          </button>
        ))}
      </div>
      <div className="stijl-snelheid">
        <input
          type="range"
          aria-label="Duur van de Kleurovergang"
          min={200}
          max={Math.max(SCHUIF_MAX_MS, waarde.duur_ms)}
          step={100}
          value={waarde.duur_ms}
          onChange={(e) => zet({ ...waarde, duur_ms: Number(e.target.value) })}
        />
        <span className="stijl-procent">
          {(waarde.duur_ms / 1000).toLocaleString("nl-NL", { minimumFractionDigits: 1 })} s{waarde.vorm === "Opbouw" ? " per Partij" : ""}
        </span>
      </div>
      {uitleg && <p className="zacht">{uitleg}</p>}
    </div>
  );
}
