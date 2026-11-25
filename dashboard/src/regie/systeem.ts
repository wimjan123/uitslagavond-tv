// De status van Server, Ether, Reserve en Voorbeeld, voor het onderdeel Systeem en de statusstippen
// bovenin. Geen stip voor de Videowand: die valt buiten het bouwplan.
import type { RendererOpMachine } from "../typen/RendererOpMachine";
import type { Rol } from "../typen/Rol";
import type { Staat } from "../verbinding";

/** Wat de server laatst meldde over de Renderers, met zijn eigen versie. */
export interface Renderermelding {
  versie: string;
  verbonden: RendererOpMachine[];
  geweigerd: RendererOpMachine[];
}

export const GEEN_RENDERERS: Renderermelding = { versie: "", verbonden: [], geweigerd: [] };

export type Toon = "goed" | "let-op" | "fout" | "onbekend";

export interface Statusregel {
  naam: "Server" | Rol;
  /** De naam in de UI: Live en Preview, met "-renderer" erachter zodat Live hier niet "in de uitzending" lijkt. */
  label: string;
  /** Hoe je de Renderer start, voor de technicus: `--rol ether`. Leeg bij de Server. */
  rol: string;
  wat: string;
  toon: Toon;
  status: string;
  machines: { machine: string; versie: string; geweigerd: boolean }[];
}

const ROLLEN: { rol: Rol; label: string; wat: string }[] = [
  { rol: "Ether", label: "Live-renderer", wat: "Het beeld dat de uitzending in gaat" },
  { rol: "Reserve", label: "Reserve-renderer", wat: "Volgt Live; bij een storing schakel je op de mixer over" },
  { rol: "Voorbeeld", label: "Preview-renderer", wat: "Tekent de kleine beelden in de multiviewer" },
];

export function statusVan(staat: Staat, r: Renderermelding): Statusregel[] {
  const verbonden = staat === "verbonden";
  const server: Statusregel = {
    naam: "Server",
    label: "Server",
    rol: "",
    wat: "De enige bron van de Stand",
    toon: verbonden ? "goed" : staat === "verbinden" ? "let-op" : "fout",
    status: verbonden ? "Verbonden" : staat === "verbinden" ? "Verbinden" : staat === "geweigerd" ? "Weigert de sleutel" : "Geen verbinding",
    machines: verbonden && r.versie ? [{ machine: serveradres(), versie: r.versie, geweigerd: false }] : [],
  };
  const renderers = ROLLEN.map(({ rol, label, wat }): Statusregel => {
    if (!verbonden) return { naam: rol, label, rol: `--rol ${rol.toLowerCase()}`, wat, toon: "onbekend", status: "Onbekend zonder server", machines: [] };
    const aan = r.verbonden.filter((m) => m.rol === rol);
    const weg = r.geweigerd.filter((m) => m.rol === rol);
    const machines = [
      ...aan.map((m) => ({ machine: m.machine, versie: m.versie, geweigerd: false })),
      ...weg.map((m) => ({ machine: m.machine, versie: m.versie, geweigerd: true })),
    ];
    if (weg.length > 0) {
      const welke = weg.map((m) => m.machine).join(", ");
      return { naam: rol, label, rol: `--rol ${rol.toLowerCase()}`, wat, toon: "let-op", status: `Geweigerd: andere versie op ${welke}`, machines };
    }
    if (aan.length > 0) return { naam: rol, label, rol: `--rol ${rol.toLowerCase()}`, wat, toon: "goed", status: "Verbonden", machines };
    return { naam: rol, label, rol: `--rol ${rol.toLowerCase()}`, wat, toon: "let-op", status: "Niet verbonden", machines };
  });
  return [server, ...renderers];
}

/** Het adres van de server zoals deze browser hem bereikt. */
function serveradres(): string {
  return typeof location === "undefined" ? "server" : location.host;
}
