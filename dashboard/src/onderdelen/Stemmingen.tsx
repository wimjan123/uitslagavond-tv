// Stemmingen (DESIGN.md, Layout en Stemming Input): links de Stemmingslijst van de dag, rechts
// de invoer "volg de Voorzitter" met een vaste balk onderaan waarin "Vastleggen en klaarzetten"
// altijd te zien is. De server rekent de uitkomst uit; dit scherm stuurt alleen Opdrachten.
import { useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties, FormEvent } from "react";
import { STEMMINGSJABLONEN, alsStemmingsjabloon, plaatsInUitslagindeling, sjabloonNaam, stemmingVan, uurMinuut } from "../graphic";
import { Kruis, Microfoon, Plus, Slot, Ster, Zoek } from "../iconen";
import { useRegie } from "../regie/regie";
import { LEEG, afwijkend, algemeen, andere, kamerlidNaam, overige, stemVan, tik, volgendeOpen, voorzitterZin, wis, zelfdeStemgedrag } from "../stemmingen/invoer";
import type { Fractie } from "../typen/Fractie";
import type { Indeling } from "../typen/Indeling";
import type { Kamer } from "../typen/Kamer";
import type { Stand } from "../typen/Stand";
import type { Stem } from "../typen/Stem";
import type { Stemgedrag } from "../typen/Stemgedrag";
import type { Stemming } from "../typen/Stemming";
import { Controle } from "./Controle";
import { Werkbalk } from "./Werkbalk";
import "./stemmingen.css";

const BIJ = "stemmingen";

type Filter = "uitzending" | "alle" | "controle";

export function Stemmingen() {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const lijst = s?.stemmingen.lijst ?? [];
  const aantalUitzending = lijst.filter((x) => x.voor_uitzending).length;
  const [keuze, setKeuze] = useState<Filter | null>(null);
  const filter: Filter = keuze ?? (aantalUitzending > 0 ? "uitzending" : "alle");
  const zichtbaar = filter === "uitzending" ? lijst.filter((x) => x.voor_uitzending) : lijst;
  const teControleren = lijst.filter((x) => x.controle.status && x.controle.status !== "Gelijk" && !x.controle.afgehandeld).length;

  const [gekozen, setGekozen] = useState<string | null>(null);
  const [typen, setTypen] = useState(false);
  // Na "Voeg toe": de nieuwe motie kiezen zodra hij in de Stand staat.
  const [handVoor, setHandVoor] = useState<string[] | null>(null);
  useEffect(() => {
    if (!handVoor) return;
    const nieuw = lijst.find((x) => x.met_de_hand && !handVoor.includes(x.punt.id));
    if (nieuw) {
      setGekozen(nieuw.punt.id);
      setHandVoor(null);
    }
  }, [lijst, handVoor]);

  const actief = zichtbaar.find((x) => x.punt.id === gekozen) ?? zichtbaar.find((x) => !x.vastgelegd) ?? zichtbaar[0] ?? null;

  return (
    <>
      <Werkbalk naam="Stemmingen" hint={<Dag dag={s?.stemmingen.dag ?? null} ingelezen={s?.stemmingen.ingelezen ?? null} />}>
        <div className="filters" role="group" aria-label="Welke Stemmingen">
          <button className="filter" aria-pressed={filter === "uitzending"} onClick={() => setKeuze("uitzending")}>
            Voor de uitzending ({aantalUitzending})
          </button>
          <button className="filter" aria-pressed={filter === "alle"} onClick={() => setKeuze("alle")}>
            Alle {lijst.length}
          </button>
          <button className="filter" aria-pressed={filter === "controle"} onClick={() => setKeuze("controle")} title="Controle na de uitzending: onze invoer naast de officiële uitslag">
            Controle{teControleren > 0 && <span className="filter-telling">{teControleren}</span>}
          </button>
        </div>
        <div
          className="filters"
          role="group"
          aria-label="Stoelen"
          title="Waar de Stemmingen de Stoelen kleuren. Tussen de beëdiging en een nieuwe Zaalindeling: Uitslagindeling, elke Fractie één blok."
        >
          {(["Zaalindeling", "Uitslagindeling"] as Indeling[]).map((i) => (
            <button
              key={i}
              className="filter"
              aria-pressed={(s?.stemmingen.indeling ?? "Zaalindeling") === i}
              disabled={regie.staat !== "verbonden"}
              onClick={() => void regie.stuur({ soort: "ZetStemmingIndeling", indeling: i }, BIJ)}
            >
              {i}
            </button>
          ))}
        </div>
        <button className="filter" aria-pressed={typen} disabled={filter === "controle"} onClick={() => setTypen(!typen)}>
          <Plus /> Met de hand
        </button>
      </Werkbalk>
      {filter === "controle" ? (
        <Controle lijst={lijst} />
      ) : (
      <div className="werkvlak-inhoud stemmingen">
        <div className="lijst">
          {typen && (
            <MetDeHand
              klaar={(titel) => {
                setHandVoor(lijst.filter((x) => x.met_de_hand).map((x) => x.punt.id));
                setKeuze(null);
                void regie.stuur({ soort: "VoegStemmingToe", titel }, BIJ).then((a) => {
                  if (a.uitgevoerd) setTypen(false);
                  else setHandVoor(null);
                });
              }}
              stop={() => setTypen(false)}
            />
          )}
          <Lijst lijst={zichtbaar} stand={s} actief={actief?.punt.id ?? null} kies={setGekozen} leeg={lijst.length === 0} />
        </div>
        {actief && s ? (
          <Invoer
            key={actief.punt.id}
            stemming={actief}
            stand={s}
            volgende={() => setGekozen(volgendeOpen(zichtbaar, actief.punt.id) ?? actief.punt.id)}
          />
        ) : (
          <div className="invoer-leeg zacht">Kies links een Stemming, of typ er een met de hand.</div>
        )}
      </div>
      )}
    </>
  );
}

function Dag({ dag, ingelezen }: { dag: string | null; ingelezen: string | null }) {
  if (!dag) return <>Nog geen stemmingslijst uit de open data</>;
  const d = new Date(`${dag}T12:00:00`).toLocaleDateString("nl-NL", { weekday: "long", day: "numeric", month: "long" });
  const tijd = ingelezen ? Date.parse(ingelezen) : NaN;
  return (
    <>
      {d.charAt(0).toUpperCase() + d.slice(1)}
      {Number.isFinite(tijd) && ` · lijst ingelezen ${uurMinuut(tijd)}`}
    </>
  );
}

function MetDeHand({ klaar, stop }: { klaar: (titel: string) => void; stop: () => void }) {
  const [titel, setTitel] = useState("");
  const verstuur = (e: FormEvent) => {
    e.preventDefault();
    if (titel.trim()) klaar(titel.trim());
  };
  return (
    <form className="met-de-hand" onSubmit={verstuur}>
      <input
        value={titel}
        onChange={(e) => setTitel(e.target.value)}
        placeholder="Titel van de motie"
        aria-label="Titel van de motie"
        maxLength={300}
        autoFocus
        onKeyDown={(e) => e.key === "Escape" && stop()}
      />
      <button className="knop knop-hoofd" type="submit" disabled={!titel.trim()}>
        Voeg toe
      </button>
    </form>
  );
}

function Lijst({ lijst, stand, actief, kies, leeg }: { lijst: Stemming[]; stand: Stand | undefined; actief: string | null; kies: (id: string) => void; leeg: boolean }) {
  const regie = useRegie();
  const klaar = stemmingVan(stand?.klaar?.graphic);
  const ether = stand?.ether;
  if (lijst.length === 0) {
    return (
      <p className="lijst-leeg zacht">
        {leeg ? "Geen Stemmingen op de lijst van deze dag. Typ een motie met de hand." : "Nog niets aangevinkt voor de uitzending. Kies Alle en vink moties aan met de ster."}
      </p>
    );
  }
  return (
    <table className="tabel motielijst">
      <thead>
        <tr>
          <th className="kol-ster">
            <span className="verborgen">Voor de uitzending</span>
          </th>
          <th>Stemming</th>
          <th className="kol-status">Status</th>
        </tr>
      </thead>
      <tbody>
        {lijst.map((x) => {
          const id = x.punt.id;
          const live = stemmingVan(ether?.graphic) === id;
          const isKlaar = !live && klaar === id;
          const rij = [id === actief && "gekozen", live && "rij-live", isKlaar && "rij-klaar"].filter(Boolean).join(" ");
          return (
            <tr key={id} className={rij} aria-selected={id === actief} onClick={() => kies(id)}>
              <td className="kol-ster">
                <button
                  className={`ster ${x.voor_uitzending ? "ster-aan" : ""}`}
                  aria-pressed={x.voor_uitzending}
                  title={x.voor_uitzending ? "Voor de uitzending: klik om uit te vinken" : "Vink aan voor de uitzending"}
                  onClick={(e) => {
                    e.stopPropagation();
                    void regie.stuur({ soort: "ZetVoorUitzending", stemming: id, aan: !x.voor_uitzending }, BIJ);
                  }}
                >
                  <Ster gevuld={x.voor_uitzending} />
                </button>
              </td>
              <td>
                <div className="motie-titel titel-cel">{x.punt.titel}</div>
                <div className="zacht">{x.met_de_hand ? "Met de hand" : (x.punt.nummer ?? x.punt.soort ?? "")}</div>
              </td>
              <td className="kol-status">
                <Status stemming={x} live={live} klaar={isKlaar} sinds={live && ether ? ether.sinds_ms : null} />
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

function Status({ stemming, live, klaar, sinds }: { stemming: Stemming; live: boolean; klaar: boolean; sinds: number | null }) {
  const b = stemming.uitkomst.besluit;
  return (
    <span className="status-cel">
      {live ? (
        <span className="stempel stempel-live">Live {sinds !== null && uurMinuut(sinds)}</span>
      ) : klaar ? (
        <span className="stempel stempel-klaar">Klaar</span>
      ) : (
        <span className="zacht">{stemming.vastgelegd && b ? b.toLowerCase() : b ? "ingevoerd" : "open"}</span>
      )}
      {stemming.vergrendeld && (
        <span className="zacht" title="Vergrendeld">
          <Slot />
        </span>
      )}
    </span>
  );
}

/** Kleur van de Partij bij een Fractie, voor het streepje op de toets. */
/** De Fracties in de volgorde van de Uitslagindeling, zoals ze in de Zaal zitten; bij gelijke plaats de grootste eerst. */
function inUitslagindeling(stand: Stand, fracties: Fractie[]): Fractie[] {
  const plaats = (f: Fractie) => plaatsInUitslagindeling(stand, stand.kamer.partij_van[f.id]);
  return [...fracties].sort((a, b) => plaats(a) - plaats(b) || b.zetels - a.zetels);
}

function kleurVan(stand: Stand, fractie: string): string | undefined {
  const sleutel = stand.kamer.partij_van[fractie];
  return stand.cijfers.partijen.find((p) => p.sleutel === sleutel)?.kleur;
}

function namenMetFractie(kamer: Kamer): Record<string, string> {
  const uit: Record<string, string> = {};
  for (const k of kamer.kamerleden) {
    const f = kamer.fracties.find((x) => x.id === k.fractie)?.afkorting;
    uit[k.id] = f ? `${kamerlidNaam(k)} (${f})` : kamerlidNaam(k);
  }
  return uit;
}

function Invoer({ stemming, stand, volgende }: { stemming: Stemming; stand: Stand; volgende: () => void }) {
  const regie = useRegie();
  const id = stemming.punt.id;
  const kamer = stand.kamer;
  const [kant, setKant] = useState<Stem>("Voor");
  const [zoeken, setZoeken] = useState(false);
  const inhoud = useRef<HTMLDivElement>(null);
  // Het stemgedrag dat dit scherm net stuurde, tot de Stand het terugmeldt: zo gaat een snelle
  // tweede tik nooit verloren.
  const [wacht, setWacht] = useState<Stemgedrag | null>(null);
  useEffect(() => {
    if (wacht && zelfdeStemgedrag(wacht, stemming.stemgedrag)) setWacht(null);
  }, [wacht, stemming.stemgedrag]);
  const sg = wacht ?? stemming.stemgedrag;
  const namen = useMemo(() => namenMetFractie(kamer), [kamer]);

  const verbonden = regie.staat === "verbonden";
  const dicht = stemming.vergrendeld || !verbonden;
  const u = stemming.uitkomst;
  const zetel = (nieuw: Stemgedrag) => {
    setWacht(nieuw);
    void regie.stuur({ soort: "ZetStemgedrag", stemming: id, stemgedrag: nieuw }, BIJ).then((a) => {
      if (!a.uitgevoerd) setWacht(null);
    });
  };
  const legVast = () => {
    void regie.stuur({ soort: "LegStemmingVast", stemming: id }, BIJ).then((a) => {
      if (a.uitgevoerd) volgende();
    });
  };
  const melding = regie.melding?.bij === BIJ ? regie.melding.tekst : null;
  const zin = voorzitterZin(sg, kamer.fracties, namen);
  const totaal = u.voor + u.tegen + u.open || 150;
  const kop = [stemming.punt.nummer, stemming.punt.indieners].filter(Boolean).join(" · ");
  const woordKlasse = u.besluit === "Aangenomen" ? "aangenomen" : u.besluit === "Verworpen" ? "verworpen" : "open";
  // De klaargezette Graphic van deze Stemming: de Regie kiest een ander Sjabloon met dezelfde uitslag.
  const klaar = stand.klaar && stemmingVan(stand.klaar.graphic) === id ? stand.klaar : null;
  const klaarUitslag = klaar && "uitslag" in klaar.graphic ? klaar.graphic.uitslag : null;
  const voorlopig = klaarUitslag ? klaarUitslag.voorlopig : sg.hoofdelijk && stand.stemmingen.indeling === "Uitslagindeling";

  return (
    <section className="invoer" aria-label="Invoer van de Stemming">
      <div className="invoer-inhoud" ref={inhoud}>
        <header className="invoer-kop">
          <div className="invoer-titel">
            <p className="zacht">{stemming.met_de_hand ? "Met de hand ingetypt" : kop || stemming.punt.soort}</p>
            <h3 className="kop-regel">{stemming.punt.titel}</h3>
          </div>
          <div className="invoer-kop-knoppen">
            <button
              className="filter"
              aria-pressed={sg.hoofdelijk}
              disabled={dicht}
              onClick={() => zetel({ ...sg, hoofdelijk: !sg.hoofdelijk })}
              title="Een hoofdelijke stemming gaat in hetzelfde scherm: tik per Fractie, uitzonderingen met Afwijkende stem"
            >
              Hoofdelijk
            </button>
            <button
              className={`knop ${stemming.vergrendeld ? "knop-hoofd" : ""}`}
              disabled={!verbonden}
              onClick={() => void regie.stuur({ soort: stemming.vergrendeld ? "OntgrendelStemming" : "VergrendelStemming", stemming: id }, BIJ)}
            >
              <Slot open={!stemming.vergrendeld} /> {stemming.vergrendeld ? "Vergrendeld" : "Vergrendel"}
            </button>
          </div>
        </header>

        <div className="uitkomst-blok">
          <div className="uitkomst">
            <span className={`uitkomst-woord ${woordKlasse}`}>
              {u.besluit ?? "Open"}
            </span>
            <span className="uitkomst-getal">
              {u.voor} voor · {u.tegen} tegen{u.open > 0 && ` · ${u.open} open`}
            </span>
            <span className="zacht">uit de Zetels van {kamer.fracties.length} Fracties</span>
          </div>
          <div className="zetelbalk" role="img" aria-label={`${u.voor} voor, ${u.tegen} tegen, ${u.open} open`}>
            <span className="zetelbalk-voor" style={{ width: `${(u.voor / totaal) * 100}%` }} />
            <span className="zetelbalk-open" style={{ width: `${(u.open / totaal) * 100}%` }} />
            <span className="zetelbalk-tegen" style={{ width: `${(u.tegen / totaal) * 100}%` }} />
            <span className="zetelbalk-meerderheid" style={{ left: `${(75.5 / totaal) * 100}%` }} />
          </div>
        </div>

        {voorlopig && (
          <p className="melding">
            Voorlopig: de Stoelen staan in de Uitslagindeling, met de Kamerleden in het blok van hun Fractie op volgorde van de kieslijst.
          </p>
        )}
        {klaar && klaarUitslag && (
          <div className="kant">
            <span className="zacht">Klaar als</span>
            <div className="filters" role="group" aria-label="Sjabloon">
              {STEMMINGSJABLONEN.map((sj) => (
                <button
                  key={sj}
                  className="filter"
                  aria-pressed={klaar.graphic.sjabloon === sj}
                  disabled={!verbonden}
                  onClick={() =>
                    void regie.stuur({ soort: "WijzigKlaar", id: klaar.id, graphic: alsStemmingsjabloon(klaarUitslag, sj), standpunt: klaar.standpunt }, BIJ)
                  }
                >
                  {sjabloonNaam(alsStemmingsjabloon(klaarUitslag, sj))}
                </button>
              ))}
            </div>
          </div>
        )}

        <p className="voorzitter" aria-live="polite">
          <Microfoon />
          <span>
            {zin.length === 0 ? (
              <span className="zacht">Tik de Fracties in de volgorde waarin de Voorzitter ze noemt.</span>
            ) : (
              zin.map((d, i) =>
                d.stem ? (
                  <span key={i} className={`zin-${d.stem === "Voor" ? "voor" : "tegen"}`}>
                    {d.tekst}
                  </span>
                ) : (
                  <span key={i}>{d.tekst}</span>
                ),
              )
            )}
          </span>
        </p>

        <div className="kant">
          <span className="zacht">Ik tik nu</span>
          <div className="segment" role="group" aria-label="Kant">
            {(["Voor", "Tegen"] as const).map((k) => (
              <button key={k} className={`segment-${k.toLowerCase()}`} aria-pressed={kant === k} onClick={() => setKant(k)}>
                {k}
              </button>
            ))}
          </div>
          <span className="zacht">Noemt de Voorzitter beide kanten? Tik de ene kant, wissel, tik de andere.</span>
        </div>

        {kamer.fracties.length === 0 ? (
          <p className="melding">Nog geen Fracties: de server heeft de Kamer nog niet uit de open data gehaald (zie Systeem).</p>
        ) : (
          <div className="fractie-toetsen">
            {inUitslagindeling(stand, kamer.fracties).map((f) => {
              const st = stemVan(sg, f.id);
              const volg = sg.getikt.findIndex((g) => g.fractie === f.id);
              const cls = ["fractie-toets", st && `fractie-${st.stem === "Voor" ? "voor" : "tegen"}`, st && !st.getikt && "fractie-overige"].filter(Boolean).join(" ");
              return (
                <button
                  key={f.id}
                  className={cls}
                  style={{ "--partij": kleurVan(stand, f.id) ?? "var(--graphite-line)" } as CSSProperties}
                  disabled={dicht}
                  onClick={() => zetel(tik(sg, f.id, kant))}
                  title={`${f.naam}: ${f.zetels} Zetels`}
                >
                  <span className="toets-volg">{volg >= 0 ? volg + 1 : ""}</span>
                  <span className="toets-stem">{st ? st.stem : ""}</span>
                  <span className="toets-naam">{f.afkorting}</span>
                  <span className="toets-zetels">{f.zetels}</span>
                </button>
              );
            })}
          </div>
        )}

        {(zoeken || sg.afwijkend.length > 0) && (
          <Afwijkend sg={sg} kamer={kamer} namen={namen} kant={kant} open={zoeken} dicht={dicht} zet={zetel} sluit={() => {
              setZoeken(false);
              inhoud.current?.scrollTo({ top: 0 });
            }} />
        )}
      </div>

      <div className="actiebalk">
        {melding && (
          <p className="actiebalk-melding" role="alert">
            {melding}
          </p>
        )}
        <button className="knop" aria-pressed={sg.overige === andere(kant)} disabled={dicht} onClick={() => zetel(overige(sg, kant))}>
          Overige {kant === "Voor" ? "tegen" : "voor"}
        </button>
        <button className="knop" aria-pressed={sg.algemene_stemmen} disabled={dicht} onClick={() => zetel(algemeen(sg))}>
          Met algemene stemmen
        </button>
        <button className="knop" aria-pressed={zoeken} disabled={dicht || kamer.kamerleden.length === 0} onClick={() => setZoeken(!zoeken)}>
          Afwijkende stem
        </button>
        <button className="knop" disabled={dicht || zelfdeStemgedrag(sg, { ...LEEG, hoofdelijk: sg.hoofdelijk })} onClick={() => zetel(wis(sg))}>
          Wissen
        </button>
        <span className="actiebalk-rechts">
          <button className="knop" onClick={volgende}>
            Volgende
          </button>
          <button
            className="knop knop-commit"
            disabled={!verbonden || !u.besluit || wacht !== null}
            title={u.besluit ? undefined : `Nog ${u.open} Zetels open: tik de Fracties of kies Overige`}
            onClick={legVast}
          >
            Vastleggen en klaarzetten
          </button>
        </span>
      </div>
    </section>
  );
}

function Afwijkend({
  sg,
  kamer,
  namen,
  kant,
  open,
  dicht,
  zet,
  sluit,
}: {
  sg: Stemgedrag;
  kamer: Kamer;
  namen: Record<string, string>;
  kant: Stem;
  open: boolean;
  dicht: boolean;
  zet: (sg: Stemgedrag) => void;
  sluit: () => void;
}) {
  const [zoek, setZoek] = useState("");
  const z = zoek.trim().toLowerCase();
  const treffers = z
    ? kamer.kamerleden.filter((k) => namen[k.id]?.toLowerCase().includes(z) && !sg.afwijkend.some((a) => a.kamerlid === k.id)).slice(0, 8)
    : [];
  const kies = (kamerlid: string, fractie: string) => {
    // Anders dan de Fractie; is die nog open, dan de kant waarop je nu tikt.
    const fs = stemVan(sg, fractie);
    zet(afwijkend(sg, kamerlid, fs ? andere(fs.stem) : kant));
    setZoek("");
  };
  return (
    <div className="afwijkend">
      <h4 className="label">Afwijkende stem</h4>
      {sg.afwijkend.map((a) => (
        <div key={a.kamerlid} className="afwijkend-rij">
          <span className="titel-cel">{namen[a.kamerlid] ?? a.kamerlid}</span>
          <div className="segment segment-klein" role="group" aria-label="Stem">
            {(["Voor", "Tegen"] as const).map((k) => (
              <button key={k} className={`segment-${k.toLowerCase()}`} aria-pressed={a.stem === k} disabled={dicht} onClick={() => zet(afwijkend(sg, a.kamerlid, k))}>
                {k}
              </button>
            ))}
          </div>
          <button className="knop knop-stil" disabled={dicht} aria-label={`Haal ${namen[a.kamerlid]} weg`} onClick={() => zet(afwijkend(sg, a.kamerlid, null))}>
            <Kruis />
          </button>
        </div>
      ))}
      {open && (
        <div className="afwijkend-zoek">
          <label className="zoekveld">
            <Zoek />
            <input value={zoek} onChange={(e) => setZoek(e.target.value)} placeholder="Zoek een Kamerlid" aria-label="Zoek een Kamerlid" autoFocus />
          </label>
          {treffers.map((k) => (
            <button key={k.id} className="treffer" onClick={() => kies(k.id, k.fractie)}>
              {namen[k.id]}
            </button>
          ))}
          {z && treffers.length === 0 && <p className="zacht">Geen Kamerlid met "{zoek.trim()}".</p>}
          <div>
            <button className="knop knop-stil" onClick={sluit}>
              Klaar met zoeken
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
