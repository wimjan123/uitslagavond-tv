// Een monitor zoals op een multiviewer: een 16:9-beeld op zwart en een labelstrook eronder.
import type { ReactNode } from "react";

/** De kleur van de lijn boven de labelstrook: groen klaar, rood in de Ether, amber tijdens Oefenen, anders grijs. */
export type Toestand = "klaar" | "ether" | "oefenen" | "rust";

export function Monitor({
  label,
  kop,
  toestand,
  gestippeld = false,
  naam,
  rechts,
  beeld,
  zonderBeeld,
}: {
  label: string;
  /** Wat er in de labelstrook staat, als dat iets anders is dan het label. */
  kop?: ReactNode;
  toestand: Toestand;
  /** De lijn boven de strook is gestippeld: het beeld speelt iets af op verzoek van de Regie. */
  gestippeld?: boolean;
  naam: string | null;
  rechts?: ReactNode;
  beeld: string | null;
  /** Uitleg in het zwart als er geen klein beeld is. */
  zonderBeeld: string;
}) {
  return (
    <figure className={`monitor monitor-${toestand}${gestippeld ? " monitor-gestippeld" : ""}`}>
      <div className="monitor-beeld">
        {beeld ? <img src={beeld} alt={`${label}: ${naam ?? "leeg"}`} /> : <span className="monitor-zonder">{zonderBeeld}</span>}
      </div>
      <figcaption className="monitor-strook">
        <span className="monitor-label">{kop ?? label}</span>
        <span className="monitor-naam">{naam ?? "Leeg"}</span>
        {rechts && <span className="monitor-rechts">{rechts}</span>}
      </figcaption>
    </figure>
  );
}
