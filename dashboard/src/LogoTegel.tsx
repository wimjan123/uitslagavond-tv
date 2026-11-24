// Een partijlogo op een witte tegel (DESIGN.md, Logo Tile). Zonder logo, of als het logo niet laadt,
// staat de korte naam in wit op de partijkleur.
import { useState } from "react";
import type { Partij } from "./typen/Partij";

export function LogoTegel({ partij }: { partij: Partij }) {
  const [kapot, setKapot] = useState<string | null>(null);
  if (partij.logo && kapot !== partij.logo) {
    return (
      <span className="logo-tegel">
        <img src={partij.logo} alt="" onError={() => setKapot(partij.logo)} />
      </span>
    );
  }
  return (
    <span className="logo-tegel logo-tegel-tekst" style={{ background: partij.kleur }} aria-hidden="true">
      {partij.kort}
    </span>
  );
}
