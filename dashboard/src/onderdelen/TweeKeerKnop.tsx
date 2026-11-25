// Een knop die bij gevaar een tweede klik vraagt (binnen vier seconden), zoals Wis coalitie in Live.
import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";

export function TweeKeerKnop({
  tweeKeer,
  vraag,
  onClick,
  children,
  className,
  disabled,
  title,
  geldigVoor = "",
}: {
  tweeKeer: boolean;
  /** De tekst na de eerste klik. */
  vraag: string;
  onClick(): void;
  children: ReactNode;
  className?: string;
  disabled?: boolean;
  title?: string;
  /** Waar de vraag over gaat, zoals de Partijen in Live. Verandert dit na de eerste klik, dan vervalt de
   * vraag: de tweede klik zou anders iets wissen wat je bij de eerste klik niet zag. */
  geldigVoor?: string;
}) {
  // De vraag geldt alleen voor wat er stond bij de eerste klik.
  const [gevraagdVoor, setGevraagdVoor] = useState<string | null>(null);
  const gevraagd = gevraagdVoor === geldigVoor;
  const klok = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(klok.current), []);
  const klik = () => {
    clearTimeout(klok.current);
    if (!tweeKeer || gevraagd) {
      setGevraagdVoor(null);
      onClick();
      return;
    }
    setGevraagdVoor(geldigVoor);
    klok.current = setTimeout(() => setGevraagdVoor(null), 4000);
  };
  return (
    <button className={`${className ?? ""}${gevraagd ? " knop-vraag" : ""}`} disabled={disabled} title={title} onClick={klik}>
      {gevraagd ? vraag : children}
    </button>
  );
}
