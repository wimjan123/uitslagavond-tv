// De slotknop van een cijferset of een Stemming.
import { Slot } from "../iconen";

/** Of iets op slot zit, als vaste stempel, en daarnaast een knop met wat hij doet. Zo verschilt de
 * toestand niet alleen in één letter ("Vergrendel" of "Vergrendeld"). */
export function Slotknop({ op, disabled, wissel }: { op: boolean; disabled?: boolean; wissel(): void }) {
  return (
    <span className="slotknop">
      <span className={`stempel ${op ? "stempel-slot" : ""}`} title={op ? "Op slot: niemand kan dit wijzigen" : "Open: dit kan nog wijzigen"}>
        <Slot open={!op} /> {op ? "Op slot" : "Open"}
      </span>
      <button className="knop" disabled={disabled} onClick={wissel}>
        {op ? "Ontgrendel" : "Zet op slot"}
      </button>
    </span>
  );
}
