// De vakken van de multiviewer-rij: Voorbeeld, de knoppen om te tonen, de Ether en de camera.
import type { ComponentType, ReactNode } from "react";
import { graphicTekst, nieuwereCijfers, sjabloonNaam, uurMinuut } from "../graphic";
import { useRegie } from "../regie/regie";
import { TAKE, kan, kanBijwerken, klaarStaatInBeeld, voerUit } from "../regie/take";
import type { BewerktSjabloon } from "../typen/BewerktSjabloon";
import type { Graphic } from "../typen/Graphic";
import { Camera, OpnameTijd, naamVan } from "./Camera";
import { Monitor } from "./Monitor";

export interface Vak {
  id: string;
  /** Kolombreedte in het raster van de rij, zoals "1fr" of "170px". */
  breedte: string;
  Inhoud: ComponentType;
}

export const VAKKEN: Vak[] = [
  { id: "voorbeeld", breedte: "1fr", Inhoud: Voorbeeld },
  { id: "take", breedte: "170px", Inhoud: Take },
  { id: "ether", breedte: "1fr", Inhoud: Ether },
  { id: "camera", breedte: "330px", Inhoud: Camera },
];

function naam(g: Graphic, sjablonen: readonly BewerktSjabloon[] = []): string {
  return `${sjabloonNaam(g, sjablonen)} · ${graphicTekst(g)}`;
}

function useZonderBeeld(): string {
  const { renderers, staat } = useRegie();
  if (staat !== "verbonden") return "Geen verbinding met de server";
  return renderers.includes("Voorbeeld") ? "Wacht op beeld" : "Geen beeld: de Renderer met rol Voorbeeld is niet verbonden";
}

function Voorbeeld() {
  const { stand, beelden } = useRegie();
  const zonderBeeld = useZonderBeeld();
  const s = stand?.stand;
  const klaar = s?.klaar ?? null;
  const inBeeld = !!s && klaarStaatInBeeld(s);
  const oefenen = s?.camera.oefenen ?? false;
  const proef = s?.camera.proef;
  let rechts: ReactNode = null;
  if (proef) {
    const rit = s?.cameraritten.some((r) => r.letter === proef.letter);
    rechts = `${rit ? "speelt Camerarit" : "probeert"} ${naamVan(s, proef.letter)}${rit ? " af" : ""}`;
  } else if (klaar && s && nieuwereCijfers(s, klaar.graphic)) rechts = <span className="let-op">nieuwere cijfers</span>;
  else if (klaar && inBeeld) rechts = "staat in beeld";
  else if (klaar && klaar.gewijzigd_door !== klaar.klaargezet_door) rechts = `gewijzigd door ${klaar.gewijzigd_door}`;
  else if (klaar) rechts = `klaargezet door ${klaar.klaargezet_door}`;
  // Tijdens Oefenen staat er wat de gamepad doet; tijdens een opname een rode stip met de tijd.
  const kop = !oefenen ? undefined : s?.camera.opname ? (
    <span className="monitor-opname">
      <span className="punt punt-ether" aria-hidden />
      <OpnameTijd />
    </span>
  ) : (
    "Oefenen"
  );
  return (
    <Monitor
      label="Voorbeeld"
      kop={kop}
      toestand={oefenen ? "oefenen" : klaar && !inBeeld ? "klaar" : "rust"}
      gestippeld={!!proef}
      naam={oefenen && !proef ? "De gamepad stuurt alleen dit beeld, de Ether blijft staan" : klaar ? naam(klaar.graphic, stand?.stand.sjablonen) : null}
      rechts={rechts}
      beeld={beelden.Voorbeeld}
      zonderBeeld={zonderBeeld}
    />
  );
}

function Ether() {
  const { stand, beelden } = useRegie();
  const zonderBeeld = useZonderBeeld();
  const s = stand?.stand;
  const ether = s?.ether ?? null;
  const rechts = !ether ? null : s && nieuwereCijfers(s, ether.graphic) ? (
    <span className="let-op">nieuwere cijfers</span>
  ) : s && kanBijwerken(s) ? (
    <span className="let-op">Nieuwere versie</span>
  ) : (
    `sinds ${uurMinuut(ether.sinds_ms)}`
  );
  return (
    <Monitor
      label="Ether"
      toestand={ether ? "ether" : "rust"}
      naam={ether ? naam(ether.graphic, s?.sjablonen) : null}
      rechts={rechts}
      beeld={beelden.Ether}
      zonderBeeld={zonderBeeld}
    />
  );
}

function Take() {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const verbonden = regie.staat === "verbonden";
  const melding = regie.melding?.bij === TAKE ? regie.melding.tekst : null;
  return (
    <div className="take">
      <button className="toon" disabled={!verbonden || !kan("Toon", s)} onClick={() => voerUit(regie, "Toon")}>
        <span>Toon</span>
        <kbd>Spatie</kbd>
      </button>
      <button className="knop knop-take" disabled={!verbonden || !kan("Bijwerken", s)} onClick={() => voerUit(regie, "Bijwerken")}>
        Bijwerken <kbd>B</kbd>
      </button>
      <button className="knop knop-take" disabled={!verbonden || !kan("UitBeeld", s)} onClick={() => voerUit(regie, "UitBeeld")}>
        Uit beeld <kbd>X</kbd>
      </button>
      <button
        className="knop knop-take"
        disabled={!verbonden || !kan("KleurUit", s)}
        onClick={() => voerUit(regie, "KleurUit")}
        title="De Stoelen in de Ether krijgen hun eigen leer terug; de Graphic blijft staan"
      >
        Kleur uit <kbd>Z</kbd>
      </button>
      <button
        className="knop knop-take"
        disabled={!verbonden || !kan("SlaOpbouwOver", s)}
        onClick={() => voerUit(regie, "SlaOpbouwOver")}
        title="De Partijen die nog niet aan de beurt waren, kleuren meteen mee; de kaartjes komen direct"
      >
        Opbouw overslaan <kbd>C</kbd>
      </button>
      {melding && (
        <p className="take-melding" role="alert">
          {melding}
        </p>
      )}
    </div>
  );
}

export function Multiviewer() {
  return (
    <section className="multiviewer" aria-label="Multiviewer" style={{ gridTemplateColumns: VAKKEN.map((v) => v.breedte).join(" ") }}>
      {VAKKEN.map(({ id, Inhoud }) => (
        <Inhoud key={id} />
      ))}
    </section>
  );
}
