// Een Stoel kiezen op de plattegrond bovenin: zolang het formulier van de Persoonskaart open is,
// kiest een klik op een Stoel het Kamerlid dat er zit. Anders doet een klik op een Stoel niets.
import { useSyncExternalStore } from "react";

type Kiezer = (plaats: number) => void;

let kiezer: Kiezer | null = null;
const luisteraars = new Set<() => void>();

/** Zet wat een klik op een Stoel doet, of `null` als hij niets doet. */
export function zetStoelkiezer(k: Kiezer | null): void {
  kiezer = k;
  luisteraars.forEach((l) => l());
}

/** Wat een klik op een Stoel nu doet, of `null`. */
export function useStoelkiezer(): Kiezer | null {
  return useSyncExternalStore(
    (l) => {
      luisteraars.add(l);
      return () => luisteraars.delete(l);
    },
    () => kiezer,
  );
}
