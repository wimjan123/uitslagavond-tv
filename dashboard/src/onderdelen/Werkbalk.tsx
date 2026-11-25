// De werkbalk boven elk onderdeel: de naam als label, filters, en rechts een hint.
import type { ReactNode } from "react";

export function Werkbalk({ naam, children, hint }: { naam: string; children?: ReactNode; hint?: ReactNode }) {
  // Zonder filters of hint geen lege band onder de multiviewer: dan alleen de kop voor schermlezers.
  if (!children && !hint) return <h2 className="alleen-voorlezen">{naam}</h2>;
  return (
    <div className="werkbalk">
      {/* De naam staat al op het tabblad; hier alleen voor schermlezers. */}
      <h2 className="alleen-voorlezen">{naam}</h2>
      {children}
      {hint && <span className="werkbalk-hint">{hint}</span>}
    </div>
  );
}

