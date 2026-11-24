// De onderdelen van het dashboard, in de volgorde van de tabbladen (DESIGN.md, Tab strip).
// Een onderdeel zonder Werkvlak is nog niet gebouwd: het tabblad staat er gedimd.
import type { ComponentType } from "react";
import { Camerawerk } from "./Camerawerk";
import { Cijfers } from "./Cijfers";
import { Coalitie } from "./Coalitie";
import { Maken } from "./Maken";
import { Presets } from "./Presets";
import { Stemmingen } from "./Stemmingen";
import { Systeem } from "./Systeem";
import { Vormgeving } from "./Vormgeving";

/** Wat elk Werkvlak meekrijgt: een ander onderdeel openen, zoals Maken vanuit Presets. */
export interface WerkvlakProps {
  kiesOnderdeel(id: string): void;
}

export interface Onderdeel {
  id: string;
  naam: string;
  Werkvlak?: ComponentType<WerkvlakProps>;
}

export const ONDERDELEN: Onderdeel[] = [
  { id: "presets", naam: "Presets", Werkvlak: Presets },
  { id: "maken", naam: "Maken", Werkvlak: Maken },
  { id: "cijfers", naam: "Cijfers", Werkvlak: Cijfers },
  { id: "stemmingen", naam: "Stemmingen", Werkvlak: Stemmingen },
  { id: "coalitie", naam: "Coalitie", Werkvlak: Coalitie },
  { id: "camera", naam: "Camera", Werkvlak: Camerawerk },
  { id: "vormgeving", naam: "Vormgeving", Werkvlak: Vormgeving },
  { id: "systeem", naam: "Systeem", Werkvlak: Systeem },
];
