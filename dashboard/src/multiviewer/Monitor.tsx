// Een monitor zoals op een multiviewer: een 16:9-beeld op zwart en een labelstrook eronder.
import type { ReactNode } from "react";

/** De kleur van de lijn boven de labelstrook: groen klaar, rood in de Ether, amber tijdens Oefenen, anders grijs. */
export type Toestand = "klaar" | "ether" | "oefenen" | "rust" | "fout";

export function Monitor({
  label,
  kop,
  toestand,
  gestippeld = false,
  naam,
  detail,
  rechts,
  beeld,
  zonderBeeld,
  leeg = "Leeg",
}: {
  label: string;
  /** Wat er als naam staat zonder Graphic. */
  leeg?: string;
  /** Wat er in de labelstrook staat, als dat iets anders is dan het label. */
  kop?: ReactNode;
  toestand: Toestand;
  /** De lijn boven de strook is gestippeld: het beeld speelt iets af op verzoek van de Regie. */
  gestippeld?: boolean;
  naam: string | null;
  /** De tweede regel: wat de Graphic toont, zoals "exitpoll tegenover Uitslag 2025". */
  detail?: string | null;
  rechts?: ReactNode;
  beeld: string | null;
  /** Uitleg in het zwart als er geen klein beeld is. */
  zonderBeeld: string;
}) {
  return (
    <figure className={`monitor monitor-${toestand}${gestippeld ? " monitor-gestippeld" : ""}`}>
      <div className="monitor-beeld">
        {beeld ? <img src={beeld} alt={`${label}: ${naam ?? leeg}`} /> : <span className="monitor-zonder">{zonderBeeld}</span>}
      </div>
      <figcaption className="monitor-strook">
        <span className="monitor-regel">
          <span className="monitor-label">{kop ?? label}</span>
          <span className="monitor-naam" title={naam ?? undefined}>
            {naam ?? leeg}
          </span>
          {rechts && <span className="monitor-rechts">{rechts}</span>}
        </span>
        <span className="monitor-detail" title={detail ?? undefined}>
          {detail || "\u00a0"}
        </span>
      </figcaption>
    </figure>
  );
}
