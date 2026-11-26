// Gemaakt door `cargo test -p gedeeld` uit crates/gedeeld/src/camera.rs; niet met de hand wijzigen.
import type { CameraProfiel } from "./CameraProfiel";

export interface GerichteBeweging {
sleutel: string;
naam: string;
toelichting: string;
duur_s: number;
}

export interface ProfielRegels {
vasteStandpunten: readonly string[];
vasteRitten: readonly string[];
bovenHetPaneel: boolean;
partij: boolean;
stoel: boolean;
eigen: boolean;
}

/** De gerichte camerabewegingen naar het blok van de gekozen Partij. */
export const PARTIJBEWEGINGEN: readonly GerichteBeweging[] = [
  { sleutel: "aanschuiven", naam: "Aanschuiven naar de Partij", toelichting: "Schuift langzaam naar schuin voor het blok van de gekozen Partij", duur_s: 8 },
  { sleutel: "partij-boog", naam: "Boog om de Partij", toelichting: "Draait in een rustige boog voor het blok van de Partij langs", duur_s: 12 },
  { sleutel: "partij-indalen", naam: "Indalen op de Partij", toelichting: "Begint hoog boven de Zaal en daalt in naar het blok van de Partij", duur_s: 10 },
  { sleutel: "partij-langs", naam: "Langs de banken van de Partij", toelichting: "Glijdt laag langs de voorste banken van het blok", duur_s: 10 },
  { sleutel: "partij-uitzoomen", naam: "Dichtbij en uitzoomen", toelichting: "Begint dicht op de Stoelen en trekt terug tot het hele blok", duur_s: 10 },
  { sleutel: "partij-vast", naam: "Vast shot op de Partij", toelichting: "Gaat vlot naar een stilstaand shot op het blok van de Partij", duur_s: 2 },
];

/** De gerichte camerabewegingen naar de Stoel van het Kamerlid. */
export const STOELBEWEGINGEN: readonly GerichteBeweging[] = [
  { sleutel: "stoel-shot", naam: "Shot op de eigen Stoel", toelichting: "Gaat in 2 s naar het vaste shot op de Stoel, links van de kaart", duur_s: 2 },
  { sleutel: "stoel-aanschuiven", naam: "Aanschuiven naar de Stoel", toelichting: "Schuift in 8 s rustig dichterbij richting de Stoel", duur_s: 8 },
  { sleutel: "stoel-langs", naam: "Langs de Stoel glijden", toelichting: "Glijdt in 10 s zijdelings langs de Stoel met diepte in de banken", duur_s: 10 },
  { sleutel: "stoel-indalen", naam: "Indalen naar de Stoel", toelichting: "Begint hoger boven de banken en daalt in 8 s naar de Stoel", duur_s: 8 },
  { sleutel: "stoel-uitzoomen", naam: "Uitzoomen vanaf de Stoel", toelichting: "Begint dicht op de Stoel en trekt in 8 s iets terug", duur_s: 8 },
];

/** Welke vaste Standpunten, vaste Cameraritten en bewegingen bij elk CameraProfiel passen. */
export const CAMERA_PROFIELEN: Readonly<Record<CameraProfiel, ProfielRegels>> = {
  "Partij": { vasteStandpunten: ["A","G"], vasteRitten: [], bovenHetPaneel: false, partij: true, stoel: false, eigen: false },
  "PersoonMetStoel": { vasteStandpunten: [], vasteRitten: [], bovenHetPaneel: false, partij: false, stoel: true, eigen: false },
  "PersoonZonderStoel": { vasteStandpunten: ["S","D","F","A"], vasteRitten: ["W","T","E"], bovenHetPaneel: false, partij: false, stoel: false, eigen: true },
  "PersoonSjabloon": { vasteStandpunten: ["S","D","F","A"], vasteRitten: ["W","T","E"], bovenHetPaneel: false, partij: false, stoel: true, eigen: true },
  "HeleZaal": { vasteStandpunten: ["A","G"], vasteRitten: ["Q","Y","E"], bovenHetPaneel: false, partij: false, stoel: false, eigen: true },
  "CoalitieZaal": { vasteStandpunten: ["A","D","G"], vasteRitten: ["Q","Y","E"], bovenHetPaneel: false, partij: false, stoel: false, eigen: true },
  "BovenPaneel": { vasteStandpunten: ["G"], vasteRitten: ["Y","E"], bovenHetPaneel: true, partij: false, stoel: false, eigen: true },
  "GrootPaneel": { vasteStandpunten: ["G","A"], vasteRitten: ["E","W","T"], bovenHetPaneel: false, partij: false, stoel: false, eigen: true },
  "Algemeen": { vasteStandpunten: ["A","S","D","F","G","H"], vasteRitten: ["Q","W","E","T","Y","U"], bovenHetPaneel: false, partij: false, stoel: false, eigen: true },
  "AlgemeenMetPartij": { vasteStandpunten: ["A","S","D","F","G","H"], vasteRitten: ["Q","W","E","T","Y","U"], bovenHetPaneel: false, partij: true, stoel: false, eigen: true },
};
