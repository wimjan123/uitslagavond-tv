// Gemaakt door `cargo test -p gedeeld` (crates/gedeeld/tests/standaard_stijl.rs). Niet met de hand wijzigen.
import type { Stijl } from "./Stijl";

/** De standaard-Stijl (docs/vormgeving.md). */
export const STANDAARD_STIJL: Stijl = {
  "kleuren": {
    "hoofd": "#ffd166",
    "accent": "#ef476f",
    "tekst": "#ffffff",
    "vlak": "#1c1f26",
    "achtergrond": "#3a3f4b"
  },
  "lettertypes": {
    "kop": "NunitoSans-ExtraBold.ttf",
    "tekst": "NunitoSans-SemiBold.ttf",
    "cijfers": "NunitoSans-ExtraBold.ttf"
  },
  "programmalogo": {
    "bestand": "programmalogo.png",
    "tekst": "Uitslagavond"
  },
  "snelheid": 100,
  "beweging": {
    "duur_ms": 500,
    "vlieg_px": 120,
    "zichtbaar_na_pct": 40,
    "lijn": {
      "verschijnt_ms": 80,
      "opent_ms": 260,
      "tekst_ms": 200,
      "dikte_px": 2,
      "tekst_weg_pct": 30,
      "vlak_dicht_pct": 80
    }
  },
  "kleurovergang": {
    "vorm": "Golf",
    "duur_ms": 1000
  },
  "kleurkracht": 50
};

/** De lettertypes die in elke Renderer zitten. */
export const INGEBOUWDE_LETTERS: readonly string[] = ["DejaVuSans-Bold.ttf","DejaVuSans.ttf","NunitoSans-SemiBold.ttf","NunitoSans-ExtraBold.ttf"];

/** De langzaamste en snelste snelheid van de Stijl, in procent. */
export const MIN_SNELHEID = 25;
export const MAX_SNELHEID = 400;

/** Langste tekst van het programmalogo zonder beeld, in tekens. */
export const MAX_LOGOTEKST = 40;

/** De kortste en langste Kleurovergang, en de standaardduur per vorm, in milliseconden (bij de Opbouw per Partij). */
export const MIN_KLEUROVERGANG_MS = 100;
export const MAX_KLEUROVERGANG_MS = 10000;
export const STANDAARD_KLEUROVERGANG_MS = { Golf: 1000, Vervagen: 1500, Opbouw: 1500 } as const;
