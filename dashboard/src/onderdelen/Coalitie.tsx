// De Coalitiebouwer (DESIGN.md, Coalitiebouwer Arc): de Partijen op een halve cirkel, de teller naar
// 76 in het midden, en rechts de keuze van Sjabloon en set. Staat er een Coalitie klaar of in de
// Ether, dan werkt elke aangetikte Partij die direct bij, ook in de Ether (besluit #6).
import { useState } from "react";
import type { CSSProperties } from "react";
import { HUIDIGE_KAMER, MEERDERHEID, boog, bouwer, coalitieInBeeld, regel, telling, wissel, zetelsVan } from "../coalitie/coalitie";
import type { Bouwer } from "../coalitie/coalitie";
import { LogoTegel } from "../LogoTegel";
import { HerstelKnop } from "../multiviewer/vakken";
import { Vink } from "../iconen";
import { biedHerstel } from "../regie/herstel";
import { useRegie } from "../regie/regie";
import type { Graphic } from "../typen/Graphic";
import type { Stand } from "../typen/Stand";
import { TweeKeerKnop } from "./TweeKeerKnop";
import { Werkbalk } from "./Werkbalk";
import "./coalitie.css";

const COALITIE = "coalitie";
const ZETELS = 150;
/** De boog: breedte en hoogte van het vlak, de straal, en hoe plat hij is (een halve ellips, zodat
 * vijftien Partijen naast elkaar passen zonder te overlappen). */
export const BOOG = { breedte: 860, hoogte: 370, straal: 390, plat: 0.8 };
export type Boogmaat = typeof BOOG;
const LEEG: Bouwer = { inBeeld: null, partijen: [], zetels: {}, volgorde: [], totaal: 0 };

type Sjabloon = "Coalitie" | "Coalitiebalk";
const SJABLONEN: { id: Sjabloon; naam: string; uitleg: string }[] = [
  { id: "Coalitie", naam: "Coalitie in de Zaal", uitleg: "De coalitie als één blok links in de Zaal, de rest donker, met de lijn bij 76 en de teller." },
  { id: "Coalitiebalk", naam: "Coalitiebalk", uitleg: "Een gestapelde balk onderin met de coalitie vooraan, de streep bij 76 en de teller." },
];

const zelfde = (a: readonly string[], b: readonly string[]) => a.length === b.length && a.every((x, i) => x === b[i]);

/** Herstel na een wijziging van een live coalitie (dashboard en tablet). Alleen terug zolang dezelfde
 * Graphic (zelfde id) nog in Live staat met precies de nieuwe Partijen: toont een andere Werkplek
 * intussen iets anders, ook een coalitie met dezelfde Partijen, dan vervalt het. */
export function biedCoalitieHerstel(s: Stand, voor: string[], nieuw: string[], tekst: string, bij: string): void {
  const etherId = s.ether?.id;
  if (etherId === undefined) return;
  biedHerstel({
    tekst,
    wat: "Coalitie live",
    knop: "Zet terug",
    geldig: (nu) => {
      const g = nu.ether?.id === etherId ? nu.ether.graphic : undefined;
      return !!g && (g.sjabloon === "Coalitie" || g.sjabloon === "Coalitiebalk") && zelfde(g.partijen, nieuw);
    },
    doe: (r) => void r.stuur({ soort: "ZetCoalitie", partijen: voor }, bij),
  });
}

export function Coalitie() {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const [sjabloon, setSjabloon] = useState<Sjabloon>("Coalitie");
  const [setKeuze, setSetKeuze] = useState("");
  const [eigen, setEigen] = useState<string[]>([]);
  const verbonden = regie.staat === "verbonden";
  const melding = regie.melding?.bij === COALITIE ? regie.melding.tekst : null;

  const kamer = (s?.kamer.fracties.length ?? 0) > 0;
  const sets = [...(s?.cijfers.sets ?? []).map((x) => ({ id: x.id, label: x.label })), ...(kamer ? [{ id: HUIDIGE_KAMER, label: "Huidige Kamer (open data)" }] : [])];
  const inBeeldNu = s ? coalitieInBeeld(s) : null;
  // Standaard de set van de coalitie in beeld, anders die van de Graphic in Live, anders de nieuwste.
  const etherSet = s?.ether && "set" in s.ether.graphic ? s.ether.graphic.set : undefined;
  const set = setKeuze || inBeeldNu?.set || (etherSet && sets.some((x) => x.id === etherSet) ? etherSet : "") || sets[sets.length - 1]?.id || "";
  const setNaam = sets.find((x) => x.id === set)?.label ?? set;
  // Wat de Coalitiebouwer toont: de coalitie die in beeld of klaar is, met haar eigen cijfers.
  const b = s ? bouwer(s, set, eigen) : LEEG;
  const { inBeeld, partijen, zetels } = b;
  const inEther = !!inBeeld && s?.ether?.graphic === inBeeld;

  /** Zet de Partijen van de coalitie in beeld; in de Ether met herstel, want dat is meteen live. */
  const zetInBeeld = (nieuw: string[], tekst: string) => {
    void regie.stuur({ soort: "ZetCoalitie", partijen: nieuw }, COALITIE);
    if (inEther && s) biedCoalitieHerstel(s, partijen, nieuw, tekst, COALITIE);
  };
  const tik = (sleutel: string) => {
    const nieuw = wissel(partijen, sleutel);
    const kort = b.volgorde.find((p) => p.sleutel === sleutel)?.kort ?? sleutel;
    if (inBeeld) zetInBeeld(nieuw, `${kort} ${nieuw.includes(sleutel) ? "erbij" : "eraf"}`);
    else setEigen(nieuw);
  };
  const wis = () => {
    if (inBeeld) {
      zetInBeeld([], "Coalitie gewist");
      return;
    }
    const voor = partijen;
    setEigen([]);
    biedHerstel({ tekst: "Coalitie gewist", wat: "Coalitiebouwer", knop: "Zet coalitie terug", geldig: (nu) => !coalitieInBeeld(nu), doe: () => setEigen(voor) });
  };
  const zetKlaar = () => {
    const graphic: Graphic = { sjabloon, set, partijen };
    void regie.stuur({ soort: "ZetKlaar", graphic }, COALITIE);
  };

  const gekozenDelen = b.volgorde.filter((p) => partijen.includes(p.sleutel));
  return (
    <>
      <Werkbalk naam="Coalitie" />
      <div className="werkvlak-inhoud coalitie">
        <CoalitieBoog
          bouwer={b}
          uit={!verbonden && !!inBeeld}
          tik={tik}
          live={inEther}
          liveSet={inBeeld?.cijfers?.set.label ?? inBeeld?.set}
          nu={inEther && s ? telling(zetelsVan(s, set), partijen) : undefined}
          leeg="Geen Zetels in deze set: kies een andere set of lees eerst cijfers in."
        />
        <aside className="paneel-detail coalitie-zijkant">
          <header className="coalitie-kop">
            <h3 className="kop-regel">Coalitiebouwer</h3>
            {inBeeld && <span className={inEther ? "stempel stempel-live" : "stempel stempel-klaar"}>{inEther ? "Live" : "Klaar"}</span>}
          </header>
          <div className="veld">
            <span className="label">Sjabloon</span>
            <div className="filters" role="group" aria-label="Sjabloon">
              {SJABLONEN.map((x) => (
                <button key={x.id} className="filter" aria-pressed={sjabloon === x.id} onClick={() => setSjabloon(x.id)} title={x.uitleg}>
                  {x.naam}
                </button>
              ))}
            </div>
          </div>
          <label className="veld">
            <span className="veld-kop">
              <span className="label">Set</span>
              <span className="zacht">De Graphic houdt deze cijfers vast</span>
            </span>
            <select value={set} onChange={(e) => setSetKeuze(e.target.value)}>
              {sets.map((x) => (
                <option key={x.id} value={x.id}>
                  {x.label}
                </option>
              ))}
            </select>
          </label>
          <ol className="coalitie-lijst">
            {gekozenDelen.map((p) => (
              <li key={p.sleutel}>
                <LogoTegel partij={p} />
                <span className="titel-cel">{p.kort}</span>
                <span className="zacht">{zetels[p.sleutel]}</span>
              </li>
            ))}
          </ol>
          <div className="knoppen">
            <button
              className={inEther ? "knop" : "knop knop-commit"}
              disabled={!verbonden || !set || partijen.length === 0}
              title={inEther ? "Deze coalitie staat al in Live en werkt bij elke klik bij. Dit zet een kopie klaar." : undefined}
              onClick={zetKlaar}
            >
              {inEther
                ? `Kopie met cijfers van nu · ${s ? telling(zetelsVan(s, set), partijen) : 0} Zetels`
                : `Zet klaar · ${setNaam} · ${s ? telling(zetelsVan(s, set), partijen) : 0} Zetels`}
            </button>
            {partijen.length === 0 && <span className="knop-reden">Kies eerst Partijen</span>}
            <TweeKeerKnop
              className="knop knop-gevaar"
              disabled={partijen.length === 0 || (!verbonden && !!inBeeld)}
              tweeKeer={inEther}
              title={inEther ? "Haalt alle Partijen uit de coalitie in Live. Klik twee keer; daarna kun je ze nog terugzetten." : "Haalt alle Partijen uit de coalitie"}
              vraag="Klik nog eens: wis uit Live"
              onClick={wis}
            >
              {inEther ? "Wis coalitie live" : "Wis coalitie"}
            </TweeKeerKnop>
          </div>
          {melding && (
            <p className="melding" role="alert">
              {melding}
            </p>
          )}
          {/* Een misklik op de live boog: het terugzetten staat ook hier, naast de klik, niet alleen bij Take. */}
          {inEther && <HerstelKnop />}
        </aside>
      </div>
    </>
  );
}

/** De Partijen op de boog met de teller naar 76 in het midden, en de Zetelbalk eronder. Ook op de
 * tablet van de Presentator, daar groter. */
export function CoalitieBoog({
  bouwer: b,
  uit,
  tik,
  leeg,
  live = false,
  liveSet,
  nu,
  gebaar = "klik",
  maat = BOOG,
}: {
  bouwer: Bouwer;
  uit: boolean;
  tik(sleutel: string): void;
  leeg: string;
  /** De coalitie staat in de Ether: elke tik is meteen live. */
  live?: boolean;
  /** De set waarvan de cijfers in de live coalitie staan. */
  liveSet?: string;
  /** De Zetels van deze Partijen met de cijfers van nu; wijkt dat af van wat live staat, dan zie je het. */
  nu?: number;
  /** "klik" op het dashboard, "tik" op de tablet. */
  gebaar?: "klik" | "tik";
  maat?: Boogmaat;
}) {
  const { volgorde, partijen, zetels, totaal } = b;
  const plekken = boog(volgorde.length, maat.straal);
  const raster = maat.breedte < 560;
  const gekozenDelen = volgorde.filter((p) => partijen.includes(p.sleutel));
  return (
    <section className={`coalitie-boog${live ? " coalitie-live" : ""}`} aria-label="Coalitiebouwer">
      {live && (
        <p className="coalitie-live-regel">
          <span className="stempel stempel-live">Live</span> Elke {gebaar} wijzigt de live coalitie{liveSet && <span className="zacht"> · {liveSet}</span>}
        </p>
      )}
      {live && nu !== undefined && nu !== totaal && (
        <p className={(nu >= MEERDERHEID) !== (totaal >= MEERDERHEID) ? "coalitie-nu coalitie-omslag" : "coalitie-nu"} role="status">
          Met de cijfers van nu: <strong>{nu} Zetels</strong>
          {(nu >= MEERDERHEID) !== (totaal >= MEERDERHEID) && (nu >= MEERDERHEID ? ", wel een meerderheid" : ", geen meerderheid meer")}
        </p>
      )}
      {volgorde.length === 0 ? (
        <p className="melding">{leeg}</p>
      ) : (
        // Smal (een telefoon): een raster met een eigen tikvlak per Partij, want op een boog zo klein
        // overlappen de tegels en raakt een tik de buurpartij.
        <div className={raster ? "boog boog-raster" : "boog"} style={raster ? undefined : { width: maat.breedte, height: maat.hoogte }}>
          {raster && <Teller totaal={totaal} oud={live && nu !== undefined && nu !== totaal} />}
          {volgorde.map((p, i) => {
            const gekozen = partijen.includes(p.sleutel);
            return (
              <button
                key={p.sleutel}
                className="boog-partij"
                aria-pressed={gekozen}
                disabled={uit}
                title={`${p.naam}: ${zetels[p.sleutel]} Zetels`}
                aria-label={`${p.naam}, ${zetels[p.sleutel]} Zetels`}
                style={raster ? { "--partij": p.kleur } as CSSProperties : { left: maat.breedte / 2 + plekken[i].x, top: maat.hoogte - 34 + plekken[i].y * maat.plat, "--partij": p.kleur } as CSSProperties}
                onClick={() => tik(p.sleutel)}
              >
                {gekozen && (
                  <span className="boog-vink">
                    <Vink />
                  </span>
                )}
                {/* De korte naam leesbaar in wit, de partijkleur als streep bovenaan: geen minitekst op kleur. */}
                <span className="boog-naam">{p.kort}</span>
                <span className="boog-zetels">{zetels[p.sleutel]}</span>
              </button>
            );
          })}
          {!raster && <Teller totaal={totaal} oud={live && nu !== undefined && nu !== totaal} />}
        </div>
      )}
      <div className="coalitie-zetelbalk" style={{ width: `min(${maat.breedte}px, 100%)` }} role="img" aria-label={`Zetelbalk: ${totaal} van ${ZETELS} Zetels in de coalitie`}>
        {gekozenDelen.map((p) => (
          <span key={p.sleutel} style={{ width: `${((zetels[p.sleutel] ?? 0) / ZETELS) * 100}%`, background: p.kleur }} title={`${p.kort} ${zetels[p.sleutel]}`} />
        ))}
        <span className="zetelbalk-meerderheid" style={{ left: `${(75.5 / ZETELS) * 100}%` }} />
      </div>
    </section>
  );
}

/** De teller naar 76. Met `oud` staat er wat Live toont met oudere cijfers: dan in amber en zonder de
 * conclusie over een meerderheid, want die kan met de cijfers van nu anders zijn (zie de regel erboven). */
function Teller({ totaal, oud = false }: { totaal: number; oud?: boolean }) {
  return (
    <div className={oud ? "boog-midden boog-oud" : "boog-midden"}>
      <span className={totaal >= MEERDERHEID && !oud ? "boog-totaal meerderheid" : "boog-totaal"}>{totaal}</span>
      <span className="zacht">{oud ? "in Live, met oudere cijfers" : regel(totaal)}</span>
    </div>
  );
}
