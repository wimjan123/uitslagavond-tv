// De vakken van de multiviewer-rij: Voorbeeld, de knoppen om te tonen, de Ether en de camera.
import { useEffect, useState } from "react";
import type { ComponentType, ReactNode } from "react";
import { graphicTekst, nieuwereCijfers, sjabloonNaam, uurMinuut } from "../graphic";
import { useRegie } from "../regie/regie";
import { HERSTEL_DUUR, geldigHerstel, herstel, useHerstel } from "../regie/herstel";
import { TAKE, kan, kanBijwerken, klaarStaatInBeeld, voerUit } from "../regie/take";
import { doelVan } from "../regie/toetsen";
import type { BewerktSjabloon } from "../typen/BewerktSjabloon";
import type { Graphic } from "../typen/Graphic";
import type { Stand } from "../typen/Stand";
import { Druppel, Overslaan, UitBeeld, Ververs } from "../iconen";
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
  } else if (klaar && inBeeld) {
    // Wat klaarstaat, staat al in beeld: de Ether zegt wat er te doen is (B werkt bij).
    rechts = "staat in beeld";
  } else if (klaar && s && nieuwereCijfers(s, klaar.graphic)) {
    // Een Preset houdt de cijfers van het moment van bewaren; Ververs zet de cijfers van nu erin.
    rechts = (
      <button
        className="strook-knop"
        title="Er zijn nieuwere cijfers dan in deze Graphic. Ververs (N) zet de cijfers van nu erin; daarna Toon."
        onClick={() => voerUit(regie, "Ververs")}
      >
        oude cijfers · Ververs <kbd>N</kbd>
      </button>
    );
  }
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
      label="Preview"
      kop={kop}
      toestand={oefenen ? "oefenen" : klaar ? "klaar" : "rust"}
      gestippeld={!!proef}
      {...(oefenen && !proef
        ? { naam: "Oefenen", detail: "De gamepad stuurt alleen dit beeld, Live blijft staan" }
        : regels(klaar?.graphic, s?.sjablonen))}
      rechts={rechts}
      beeld={beelden.Voorbeeld}
      zonderBeeld={zonderBeeld}
    />
  );
}

function Ether() {
  const regie = useRegie();
  const { stand, beelden, renderers, staat } = regie;
  const zonderBeeld = useZonderBeeld();
  const s = stand?.stand;
  const ether = s?.ether ?? null;
  // Valt de Renderer van de Ether weg, dan moet dat hier staan: overschakelen naar Reserve begint met zien.
  const geenEther = staat === "verbonden" && !renderers.includes("Ether");
  const rechts = geenEther ? (
    <span className="strook-fout" title="De Live-renderer is niet verbonden. Schakel op de mixer over naar Reserve.">
      Live-Renderer weg
    </span>
  ) : !ether ? null : s && kanBijwerken(s) ? (
    // Eén handeling op deze plek: de chip is zelf de knop Update.
    <button
      className="strook-knop"
      title="Wat klaarstaat is een nieuwere versie van deze Graphic, of heeft nieuwere cijfers: Update zet die in Live"
      onClick={() => voerUit(regie, "Bijwerken")}
    >
      {nieuwereCijfers(s, ether.graphic) ? "oude cijfers" : "nieuwere versie"} · Update <kbd>B</kbd>
    </button>
  ) : s && nieuwereCijfers(s, ether.graphic) ? (
    s.klaar ? (
      // Iets anders staat klaar: hier valt niets te doen, dus gewone tekst, geen knopvorm.
      <span className="let-op" title="Deze Graphic in Live heeft niet de cijfers van nu. Zet haar opnieuw klaar (cijfers van nu) en Take.">oude cijfers</span>
    ) : (
      <button
        className="strook-knop"
        title="Zet deze Graphic opnieuw klaar met de cijfers van nu. Daarna zet Take ze Live."
        onClick={() => void regie.stuur({ soort: "ZetKlaar", graphic: ether.graphic, standpunt: ether.standpunt }, TAKE)}
      >
        oude cijfers · zet opnieuw klaar
      </button>
    )
  ) : (
    `sinds ${uurMinuut(ether.sinds_ms)}`
  );
  return (
    // Live is altijd de uitzending, ook zonder Graphic (dan de Zaal): daarom altijd de rode tally.
    <Monitor
      label="Live"
      leeg="Geen Graphic"
      toestand={geenEther ? "fout" : "ether"}
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
  const oud = kanToon && kan("Ververs", s);
  const typen = useTypt();
  return (
    <div className={typen ? "take take-typt" : "take"}>
      <span className="take-uitleg" title={kanToon ? (typen ? "Klik zet Preview Live. Zolang je in een veld typt, werken de sneltoetsen niet: klik buiten het veld of druk Tab." : "Take: zet wat in Preview staat Live (Spatie). Enter doet dit nooit.") : waaromGeenToon(s)}>
        <button
          className={`toon${oud ? " toon-oud" : ""}`}
          disabled={!kanToon}
          // Enter toont nooit iets, ook niet als deze knop de focus heeft (DESIGN.md, Keyboard).
          onKeyDown={(e) => e.key === "Enter" && e.preventDefault()}
          onClick={() => voerUit(regie, "Toon")}
        >
          <span className="toon-regel">
            <span>Take</span>
            {!typen && <kbd>Spatie</kbd>}
          </span>
          {oud && <span className="toon-let-op" title="Wat klaarstaat heeft niet de cijfers van nu. N zet de cijfers van nu erin.">{typen ? "oude cijfers" : "oude cijfers (N)"}</span>}
          {!kanToon && verbonden && <span className="toon-reden">{s?.klaar ? "staat al live" : "niets klaar"}</span>}
          {typen && kanToon && <span className="toon-typen">sneltoetsen uit tijdens typen</span>}
        </button>
      </span>
      <span
        className="take-uitleg"
        title={
          kan("Bijwerken", s)
            ? "Zet de nieuwere versie of de nieuwere cijfers in Live, zonder opnieuw op te bouwen"
            : "Kan als wat klaarstaat een nieuwere versie is van de Graphic in Live"
        }
      >
        <button className="knop knop-take" disabled={!verbonden || !kan("Bijwerken", s)} onClick={() => voerUit(regie, "Bijwerken")}>
          <Ververs />
          <span className="knop-woord">Update</span>
          <kbd>B</kbd>
        </button>
      </span>
      <span className="take-uitleg" title={s?.ether ? "Haalt de Graphic uit Live. Daarna kun je hem nog terugzetten (Ctrl+Z)." : "Er staat niets in Live"}>
        <button className="knop knop-take knop-uit" disabled={!verbonden || !kan("UitBeeld", s)} onClick={() => voerUit(regie, "UitBeeld")}>
          <UitBeeld />
          <span className="knop-woord">Take out</span>
          <kbd>X</kbd>
        </button>
      </span>
      <span
        className="take-uitleg"
        title={kan("KleurUit", s) ? "De Stoelen in Live krijgen hun eigen leer terug; de Graphic blijft staan" : "De Stoelen in Live hebben geen kleur"}
      >
        <button className="knop knop-take" disabled={!verbonden || !kan("KleurUit", s)} onClick={() => voerUit(regie, "KleurUit")}>
          <Druppel />
          <span className="knop-woord">Kleur uit</span>
          <kbd>Z</kbd>
        </button>
      </span>
      <span
        className="take-uitleg"
        title={
          kan("SlaOpbouwOver", s)
            ? "De Partijen die nog niet aan de beurt waren, kleuren meteen mee; de kaartjes komen direct"
            : "Er loopt geen Opbouw in Live"
        }
      >
        <button className="knop knop-take" disabled={!verbonden || !kan("SlaOpbouwOver", s)} onClick={() => voerUit(regie, "SlaOpbouwOver")}>
          <Overslaan />
          <span className="knop-woord">Opbouw over</span>
          <kbd>C</kbd>
        </button>
      </span>
      {melding && (
        <p className="take-melding" role="alert">
          {melding}
        </p>
      )}
      <HerstelKnop />
    </div>
  );
}

/** Of de focus in een tekstveld staat: dan typen toetsen en werken de sneltoetsen niet. */
function useTypt(): boolean {
  const [typt, setTypt] = useState(false);
  // Tijdens typen gaan letters naar het veld: dan verdwijnen alle toetskappen in de multiviewer (CSS).
  useEffect(() => {
    document.documentElement.toggleAttribute("data-typt", typt);
  }, [typt]);
  useEffect(() => {
    const kijk = () => setTypt(doelVan(document.activeElement) === "tekstveld");
    // Na focusout staat de focus nog even op het oude veld; daarna kijken.
    const later = () => window.setTimeout(kijk);
    window.addEventListener("focusin", kijk);
    window.addEventListener("focusout", later);
    return () => {
      window.removeEventListener("focusin", kijk);
      window.removeEventListener("focusout", later);
    };
  }, []);
  return typt;
}

/** Na Uit beeld of een wisactie: wat er gebeurde, wat terugkomt, en een knop (Ctrl+Z) om het terug te
 * zetten. De knop verdwijnt zodra er intussen iets anders gebeurde (zie herstel.ts). */
export function HerstelKnop({ toets = true }: { toets?: boolean }) {
  const regie = useRegie();
  const h = geldigHerstel(useHerstel(), regie.stand?.stand);
  const [, tik] = useState(0);
  useEffect(() => {
    if (!h) return;
    const t = window.setInterval(() => tik((n) => n + 1), 1000);
    return () => window.clearInterval(t);
  }, [h]);
  if (!h) return null;
  const over = Math.max(0, Math.ceil((h.tot - Date.now()) / 1000));
  return (
    <div className="herstel" role="status">
      <span className="herstel-wat" title={`${h.tekst}: ${h.wat}`}>
        <span className="herstel-tekst">{h.tekst}:</span> {h.wat}
      </span>
      <span className="herstel-sec" aria-hidden>{over} s</span>
      <button className="knop herstel-knop" onClick={() => herstel(regie)}>
        {h.knop} {toets && <kbd>Ctrl+Z</kbd>}
      </button>
      <span className="herstel-tijd" style={{ width: `${(over / (HERSTEL_DUUR / 1000)) * 100}%` }} aria-label={`nog ${over} seconden`} />
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
