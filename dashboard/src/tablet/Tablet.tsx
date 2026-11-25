// De tablet van de Presentator op /tablet (besluit #10, ontwerp T2): twee grote tabbladen onderin,
// Presets en Coalitiebouwer. Een tik zet een Preset klaar; de Regie drukt daarna op Toon. In de
// Coalitiebouwer werkt elke aangetikte Partij de Coalitie in beeld direct bij. De tablet meldt zich
// als Werkplek van de soort Tablet; de server weigert alles buiten klaarzetten en de Coalitiebouwer.
import { useCallback, useEffect, useState } from "react";
import { Aanmelden } from "../App";
import { bouwer, wissel } from "../coalitie/coalitie";
import { graphicTekst, nieuwereCijfers, sjabloonNaam, uurMinuut } from "../graphic";
import { CoalitieBoog } from "../onderdelen/Coalitie";
import { TweeKeerKnop } from "../onderdelen/TweeKeerKnop";
import type { Boogmaat } from "../onderdelen/Coalitie";
import { Schets } from "../presets/Schets";
import { filterPresets, statusVan } from "../presets/presets";
import { RegieBron, useRegie } from "../regie/regie";
import type { Toegang } from "../regie/regie";
import type { Graphic } from "../typen/Graphic";
import type { Preset } from "../typen/Preset";
import type { Stand } from "../typen/Stand";
import "./tablet.css";

const OPSLAG = "uitslagavond.tablet";
/** Waar een weigering op de tablet komt te staan. */
const TABLET = "tablet";

function leesToegang(): Toegang | null {
  try {
    const t = JSON.parse(localStorage.getItem(OPSLAG) ?? "null") as Toegang | null;
    return t?.sleutel && t.werkplek ? { ...t, tablet: true } : null;
  } catch {
    return null;
  }
}

export function TabletApp() {
  const [toegang, setToegang] = useState<Toegang | null>(leesToegang);
  const afmelden = useCallback(() => {
    localStorage.removeItem(OPSLAG);
    setToegang(null);
  }, []);
  if (!toegang) {
    return (
      <Aanmelden
        titel="Tablet van de Presentator"
        standaardNaam="Tablet"
        klaar={(t) => {
          const tablet = { ...t, tablet: true };
          localStorage.setItem(OPSLAG, JSON.stringify(tablet));
          setToegang(tablet);
        }}
      />
    );
  }
  return (
    <RegieBron toegang={toegang} afmelden={afmelden}>
      <Tablet />
    </RegieBron>
  );
}

type Tab = "presets" | "coalitie";

function Tablet() {
  const regie = useRegie();
  const [tab, setTab] = useState<Tab>("presets");
  const s = regie.stand?.stand;
  return (
    <div className="tablet">
      <header className="tablet-kop">
        <span className="titel-cel">{regie.werkplek}</span>
        <span className="status-punt">
          <span className={`punt punt-${regie.staat === "verbonden" ? "goed" : regie.staat === "verbinden" ? "let-op" : "fout"}`} />
          Server
        </span>
        <span className="tablet-ruimte" />
        <TweeKeerKnop className="knop knop-stil" tweeKeer vraag="Tik nog eens om af te melden" onClick={regie.afmelden}>
          Afmelden
        </TweeKeerKnop>
      </header>
      {regie.staat !== "verbonden" && (
        <div className="storing" role="alert">
          {regie.staat === "geweigerd"
            ? "De server weigert de verbinding: meld af en vul de sleutel opnieuw in"
            : regie.staat === "verbinden"
              ? "Verbinden met de server"
              : "Geen verbinding met de server. De tablet probeert het elke seconde opnieuw."}
        </div>
      )}
      {regie.melding?.bij === TABLET && (
        <p className="tablet-melding" role="alert">
          {regie.melding.tekst}
        </p>
      )}
      {s && <NuInBeeld stand={s} />}
      <main className="tablet-vlak">{s && (tab === "presets" ? <Presets stand={s} /> : <Coalitiebouwer stand={s} />)}</main>
      <nav className="tablet-tabs" role="tablist" aria-label="Tabbladen">
        <button role="tab" className="tablet-tab" aria-selected={tab === "presets"} onClick={() => setTab("presets")}>
          Presets
        </button>
        <button role="tab" className="tablet-tab" aria-selected={tab === "coalitie"} onClick={() => setTab("coalitie")}>
          Coalitiebouwer
        </button>
      </nav>
    </div>
  );
}

/** Wat nu in beeld is en wat klaarstaat, ook als de Regie het zelf klaarzette. */
function NuInBeeld({ stand: s }: { stand: Stand }) {
  const tekst = (g: Graphic) => `${sjabloonNaam(g, s.sjablonen)} · ${graphicTekst(g)}`;
  return (
    <div className="tablet-nu">
      <span className="tablet-nu-deel">
        <span className="stempel stempel-live">Live</span>
        <span className="een-regel">{s.ether ? tekst(s.ether.graphic) : <span className="zacht">niets</span>}</span>
      </span>
      <span className="tablet-nu-deel">
        <span className="stempel stempel-klaar">Klaar</span>
        <span className="een-regel">{s.klaar ? tekst(s.klaar.graphic) : <span className="zacht">niets</span>}</span>
      </span>
    </div>
  );
}

/** Alleen de Presets die de Regie op de tablet zette. */
function Presets({ stand: s }: { stand: Stand }) {
  const lijst = filterPresets(s.presets, null, true);
  return (
    <section className="tablet-presets" aria-label="Presets">
      {lijst.length === 0 && <p className="tablet-leeg">De Regie heeft nog geen Presets op de tablet gezet.</p>}
      {lijst.map((p) => (
        <Rij key={p.nummer} preset={p} stand={s} />
      ))}
    </section>
  );
}

function Rij({ preset: p, stand: s }: { preset: Preset; stand: Stand }) {
  const regie = useRegie();
  const status = statusVan(p, s);
  const rij = status.live ? "tablet-rij rij-live" : status.klaar ? "tablet-rij rij-klaar" : "tablet-rij";
  return (
    <article className={rij}>
      <Schets graphic={p.graphic} kleuren={s.stijl.kleuren} />
      <div className="tablet-rij-tekst">
        <div className="tablet-rij-naam">
          <span className="toets-kap">{p.nummer}</span> {sjabloonNaam(p.graphic)}
          {status.live && <span className="stempel stempel-live">Live</span>}
          {status.klaar && <span className="stempel stempel-klaar">Klaar</span>}
        </div>
        <div className="zacht een-regel">{graphicTekst(p.graphic)}</div>
        {nieuwereCijfers(s, p.graphic) && (
          <div className="tablet-oud" title="Er zijn nieuwere cijfers; de Regie ververst ze voor Toon">
            cijfers van {uurMinuut(p.bewaard_ms)}, er zijn nieuwere
          </div>
        )}
      </div>
      <button
        className={`knop tablet-klaarzetten${status.live && !status.klaar ? " tablet-opnieuw" : ""}`}
        // Staat hij al klaar, dan zou een tweede tik de Regie alleen een nieuwe versie geven.
        disabled={regie.staat !== "verbonden" || status.klaar}
        onClick={() => void regie.stuur({ soort: "ZetPresetKlaar", nummer: p.nummer }, TABLET)}
      >
        {status.klaar ? "Staat klaar" : status.live ? "Opnieuw klaarzetten" : "Klaarzetten"}
      </button>
    </article>
  );
}

/** De boog zo groot als de tablet toelaat, liggend en staand. */
function useBoogmaat(): Boogmaat {
  const [breedte, setBreedte] = useState(() => window.innerWidth);
  useEffect(() => {
    const meet = () => setBreedte(window.innerWidth);
    window.addEventListener("resize", meet);
    return () => window.removeEventListener("resize", meet);
  }, []);
  const b = Math.min(1040, breedte - 48);
  const straal = b / 2 - 46;
  // Staand is er ruimte in de hoogte: een hogere boog, zodat de tegels aan de uiteinden niet overlappen.
  const plat = breedte < 900 ? 1.15 : 0.8;
  return { breedte: b, hoogte: Math.round(straal * plat + 96), straal, plat };
}

/** De Coalitie die klaarstaat of in beeld is: elke tik werkt haar direct bij. Een nieuwe Coalitie
 * klaarzetten doet de Regie, of de Presentator met een Preset. */
function Coalitiebouwer({ stand: s }: { stand: Stand }) {
  const regie = useRegie();
  const maat = useBoogmaat();
  const b = bouwer(s, "", []);
  const { inBeeld } = b;
  if (!inBeeld) {
    const coalities = filterPresets(s.presets, "Coalitiebouwer", true);
    return (
      <section className="tablet-presets">
        <p className="tablet-uitleg zacht">
          Er staat geen Coalitie klaar of in beeld.{" "}
          {coalities.length > 0 ? "Zet er een klaar; daarna tik je hier Partijen aan en af." : "Vraag de Regie een Coalitie op de tablet te zetten."}
        </p>
        {coalities.map((p) => (
          <Rij key={p.nummer} preset={p} stand={s} />
        ))}
      </section>
    );
  }
  const inEther = s.ether?.graphic === inBeeld;
  const stuur = (partijen: string[]) => void regie.stuur({ soort: "ZetCoalitie", partijen }, TABLET);
  return (
    <section className="tablet-coalitie">
      {!inEther && (
        <p className="tablet-uitleg">
          <span className="stempel stempel-klaar">Klaar</span>
          <span className="zacht">Elke tik werkt bij in de klaargezette Coalitie</span>
        </p>
      )}
      <CoalitieBoog
        bouwer={b}
        uit={regie.staat !== "verbonden"}
        tik={(sleutel) => stuur(wissel(b.partijen, sleutel))}
        leeg="Geen Zetels in de set van deze Coalitie."
        live={inEther}
        maat={maat}
      />
      <TweeKeerKnop
        className="knop tablet-wis"
        tweeKeer={inEther}
        vraag="Tik nog eens: wis alle Partijen uit de Ether"
        disabled={regie.staat !== "verbonden" || b.partijen.length === 0}
        onClick={() => stuur([])}
      >
        Wis coalitie
      </TweeKeerKnop>
    </section>
  );
}

