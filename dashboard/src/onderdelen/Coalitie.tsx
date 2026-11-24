// De Coalitiebouwer (DESIGN.md, Coalitiebouwer Arc): de Partijen op een halve cirkel, de teller naar
// 76 in het midden, en rechts de keuze van Sjabloon en set. Staat er een Coalitie klaar of in de
// Ether, dan werkt elke aangetikte Partij die direct bij, ook in de Ether (besluit #6).
import { useState } from "react";
import { HUIDIGE_KAMER, MEERDERHEID, boog, bouwer, coalitieInBeeld, regel, wissel } from "../coalitie/coalitie";
import type { Bouwer } from "../coalitie/coalitie";
import { LogoTegel } from "../LogoTegel";
import { useRegie } from "../regie/regie";
import type { Graphic } from "../typen/Graphic";
import { TakeHint, Werkbalk } from "./Werkbalk";
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
  const set = setKeuze || inBeeldNu?.set || sets[0]?.id || "";
  // Wat de Coalitiebouwer toont: de coalitie die in beeld of klaar is, met haar eigen cijfers.
  const b = s ? bouwer(s, set, eigen) : LEEG;
  const { inBeeld, partijen, zetels } = b;
  const inEther = !!inBeeld && s?.ether?.graphic === inBeeld;

  const tik = (sleutel: string) => {
    const nieuw = wissel(partijen, sleutel);
    if (inBeeld) void regie.stuur({ soort: "ZetCoalitie", partijen: nieuw }, COALITIE);
    else setEigen(nieuw);
  };
  const wis = () => {
    if (inBeeld) void regie.stuur({ soort: "ZetCoalitie", partijen: [] }, COALITIE);
    else setEigen([]);
  };
  const zetKlaar = () => {
    const graphic: Graphic = { sjabloon, set, partijen };
    void regie.stuur({ soort: "ZetKlaar", graphic }, COALITIE);
  };

  const gekozenDelen = b.volgorde.filter((p) => partijen.includes(p.sleutel));
  return (
    <>
      <Werkbalk naam="Coalitie" hint={<TakeHint />} />
      <div className="werkvlak-inhoud coalitie">
        <CoalitieBoog
          bouwer={b}
          uit={!verbonden && !!inBeeld}
          tik={tik}
          leeg="Geen Zetels in deze set: kies een andere set of lees eerst cijfers in."
        />
        <aside className="paneel-detail coalitie-zijkant">
          <header>
            <h3 className="kop-regel">Coalitiebouwer</h3>
            <p className="zacht">
              {inBeeld
                ? inEther
                  ? "Elke Partij die je aantikt, werkt direct bij in de Ether."
                  : "Elke Partij die je aantikt, werkt direct bij in de klaargezette Graphic."
                : "Tik Partijen aan en zet de Coalitie klaar. Daarna werkt elke Partij direct bij, ook in de Ether."}
            </p>
          </header>
          {inBeeld && <span className={inEther ? "stempel stempel-live" : "stempel stempel-klaar"}>{inEther ? "LIVE" : "KLAAR"}</span>}
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
            <button className="knop knop-hoofd" disabled={!verbonden || !set} onClick={zetKlaar}>
              Zet klaar als {SJABLONEN.find((x) => x.id === sjabloon)?.naam}
            </button>
            <button className="knop" disabled={partijen.length === 0 || (!verbonden && !!inBeeld)} onClick={wis}>
              Wis coalitie
            </button>
          </div>
          {melding && (
            <p className="melding" role="alert">
              {melding}
            </p>
          )}
        </aside>
      </div>
    </>
  );
}

/** De Partijen op de boog met de teller naar 76 in het midden, en de Zetelbalk eronder. Ook op de
 * tablet van de Presentator, daar groter. */
export function CoalitieBoog({ bouwer: b, uit, tik, leeg, maat = BOOG }: { bouwer: Bouwer; uit: boolean; tik(sleutel: string): void; leeg: string; maat?: Boogmaat }) {
  const { volgorde, partijen, zetels, totaal } = b;
  const plekken = boog(volgorde.length, maat.straal);
  const gekozenDelen = volgorde.filter((p) => partijen.includes(p.sleutel));
  return (
    <section className="coalitie-boog" aria-label="Coalitiebouwer">
      {volgorde.length === 0 ? (
        <p className="melding">{leeg}</p>
      ) : (
        <div className="boog" style={{ width: maat.breedte, height: maat.hoogte }}>
          {volgorde.map((p, i) => {
            const gekozen = partijen.includes(p.sleutel);
            return (
              <button
                key={p.sleutel}
                className="boog-partij"
                aria-pressed={gekozen}
                disabled={uit}
                title={`${p.naam}: ${zetels[p.sleutel]} Zetels`}
                style={{ left: maat.breedte / 2 + plekken[i].x, top: maat.hoogte - 34 + plekken[i].y * maat.plat }}
                onClick={() => tik(p.sleutel)}
              >
                {gekozen && <span className="boog-vink" aria-hidden="true">✓</span>}
                <LogoTegel partij={p} />
                <span className="boog-zetels">{zetels[p.sleutel]}</span>
              </button>
            );
          })}
          <div className="boog-midden">
            <span className={totaal >= MEERDERHEID ? "boog-totaal meerderheid" : "boog-totaal"}>{totaal}</span>
            <span className="zacht">{regel(totaal)}</span>
          </div>
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
