// De vakken van de multiviewer-rij: Voorbeeld, de knoppen om te tonen, de Ether en de camera.
import type { ComponentType, ReactNode } from "react";
import { graphicTekst, nieuwereCijfers, sjabloonNaam, uurMinuut } from "../graphic";
import { useRegie } from "../regie/regie";
import { herstel, useHerstel } from "../regie/herstel";
import { TAKE, kan, kanBijwerken, klaarStaatInBeeld, voerUit } from "../regie/take";
import type { BewerktSjabloon } from "../typen/BewerktSjabloon";
import type { Graphic } from "../typen/Graphic";
import type { Stand } from "../typen/Stand";
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
  { id: "take", breedte: "156px", Inhoud: Take },
  { id: "ether", breedte: "1fr", Inhoud: Ether },
  { id: "camera", breedte: "minmax(340px, 25%)", Inhoud: Camera },
];

/** De twee regels onder een monitor: het Sjabloon en wat de Graphic toont. */
function regels(g: Graphic | undefined, sjablonen: readonly BewerktSjabloon[] = []): { naam: string | null; detail: string | null } {
  return g ? { naam: sjabloonNaam(g, sjablonen), detail: graphicTekst(g) } : { naam: null, detail: null };
}

function useZonderBeeld(): string {
  const { renderers, staat } = useRegie();
  if (staat !== "verbonden") return "Geen verbinding met de server";
  return renderers.includes("Voorbeeld") ? "Wacht op beeld" : "Geen beeld: de Renderer met rol Voorbeeld is niet verbonden";
}

function Voorbeeld() {
  const regie = useRegie();
  const { stand, beelden } = regie;
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
  } else if (klaar && s && nieuwereCijfers(s, klaar.graphic)) {
    // Een Preset houdt de cijfers van het moment van bewaren; Ververs zet de cijfers van nu erin.
    rechts = (
      <button
        className="strook-knop"
        title="Er zijn nieuwere cijfers dan in deze Graphic. Ververs zet de cijfers van nu erin; daarna Toon."
        onClick={() => void regie.stuur({ soort: "WijzigKlaar", id: klaar.id, graphic: klaar.graphic, standpunt: klaar.standpunt }, TAKE)}
      >
        oude cijfers · Ververs
      </button>
    );
  }
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
      {...(oefenen && !proef
        ? { naam: "Oefenen", detail: "De gamepad stuurt alleen dit beeld, de Ether blijft staan" }
        : regels(klaar?.graphic, s?.sjablonen))}
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
  const rechts = !ether ? null : s && kanBijwerken(s) ? (
    <span className="let-op" title="Wat klaarstaat is een nieuwere versie van deze Graphic, of heeft nieuwere cijfers">
      {nieuwereCijfers(s, ether.graphic) ? "oude cijfers" : "nieuwere versie klaar"} · <kbd>B</kbd> werkt bij
    </span>
  ) : s && nieuwereCijfers(s, ether.graphic) ? (
    <span className="let-op" title="Zet deze Graphic opnieuw klaar met de cijfers van nu, en toon of werk bij">
      oude cijfers
    </span>
  ) : (
    `sinds ${uurMinuut(ether.sinds_ms)}`
  );
  return (
    <Monitor
      label="Ether"
      toestand={ether ? "ether" : "rust"}
      {...regels(ether?.graphic, s?.sjablonen)}
      rechts={rechts}
      beeld={beelden.Ether}
      zonderBeeld={zonderBeeld}
    />
  );
}

/** Waarom Toon niet kan, voor de tooltip. */
function waaromGeenToon(s: Stand | undefined): string {
  if (!s?.klaar) return "Er staat niets klaar: zet een Preset of Graphic klaar";
  return "Wat klaarstaat, staat al zo in beeld";
}

function Take() {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const verbonden = regie.staat === "verbonden";
  const melding = regie.melding?.bij === TAKE ? regie.melding.tekst : null;
  const kanToon = verbonden && kan("Toon", s);
  return (
    <div className="take">
      <button
        className="toon"
        disabled={!kanToon}
        title={kanToon ? "Zet wat klaarstaat in de Ether (Spatie). Enter toont nooit." : waaromGeenToon(s)}
        // Enter toont nooit iets, ook niet als deze knop de focus heeft (DESIGN.md, Keyboard).
        onKeyDown={(e) => e.key === "Enter" && e.preventDefault()}
        onClick={() => voerUit(regie, "Toon")}
      >
        <span>Toon</span>
        <kbd>Spatie</kbd>
      </button>
      <button
        className="knop knop-take"
        disabled={!verbonden || !kan("Bijwerken", s)}
        title={
          kan("Bijwerken", s)
            ? "Zet de nieuwere versie of de nieuwere cijfers in de Ether, zonder opnieuw op te bouwen"
            : "Kan als wat klaarstaat een nieuwere versie is van de Graphic in de Ether"
        }
        onClick={() => voerUit(regie, "Bijwerken")}
      >
        Bijwerken <kbd>B</kbd>
      </button>
      <button
        className="knop knop-take knop-uit"
        disabled={!verbonden || !kan("UitBeeld", s)}
        title={s?.ether ? "Haalt de Graphic uit de Ether. Daarna kun je hem 10 seconden terugzetten." : "Er staat niets in de Ether"}
        onClick={() => voerUit(regie, "UitBeeld")}
      >
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
      <HerstelKnop />
    </div>
  );
}

/** Na Uit beeld of een wisactie: wat er gebeurde en een knop om het terug te zetten. */
function HerstelKnop() {
  const h = useHerstel();
  if (!h) return null;
  return (
    <div className="herstel" role="status">
      <span className="herstel-tekst">{h.tekst}</span>
      <button className="knop herstel-knop" onClick={herstel}>
        {h.knop}
      </button>
      <span className="herstel-tijd" style={{ animationDuration: `${Math.max(0, h.tot - Date.now())}ms` }} aria-hidden />
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
