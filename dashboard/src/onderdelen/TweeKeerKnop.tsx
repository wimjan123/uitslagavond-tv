// Een knop die bij gevaar een tweede klik vraagt (binnen vier seconden), zoals Wis coalitie in de Ether.
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
}: {
  tweeKeer: boolean;
  /** De tekst na de eerste klik. */
  vraag: string;
  onClick(): void;
  children: ReactNode;
  className?: string;
  disabled?: boolean;
  title?: string;
}) {
  const [gevraagd, setGevraagd] = useState(false);
  const klok = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(klok.current), []);
  const klik = () => {
    clearTimeout(klok.current);
    if (!tweeKeer || gevraagd) {
      setGevraagd(false);
      onClick();
      return;
    }
    setGevraagd(true);
    klok.current = setTimeout(() => setGevraagd(false), 4000);
  };
  return (
    <button className={`${className ?? ""}${gevraagd ? " knop-vraag" : ""}`} disabled={disabled} title={title} onClick={klik}>
      {gevraagd ? vraag : children}
    </button>
  );
}
