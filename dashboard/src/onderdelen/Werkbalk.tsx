// De werkbalk boven elk onderdeel: de naam als label, filters, en rechts een hint.
import type { ReactNode } from "react";

export function Werkbalk({ naam, children, hint }: { naam: string; children?: ReactNode; hint?: ReactNode }) {
  return (
    <div className="werkbalk">
      <h2 className="label">{naam}</h2>
      {children}
      {hint && <span className="werkbalk-hint">{hint}</span>}
    </div>
  );
}

/** De sneltoetsen om te tonen, voor rechts in de werkbalk. */
export function TakeHint() {
  return (
    <>
      <kbd>Spatie</kbd> toont · <kbd>B</kbd> werkt bij · <kbd>X</kbd> haalt uit beeld · <kbd>Z</kbd> kleur uit · <kbd>C</kbd> slaat Opbouw over
    </>
  );
}
