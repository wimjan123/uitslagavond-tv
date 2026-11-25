// Cijfers (DESIGN.md, Layout: Cijfers): links de sets als lijst, rechts de Partijen van de gekozen
// set in twee kolommen met plus en min, Nu, de vergelijkingsset en het verschil. Twee Werkplekken
// kunnen dezelfde set bewerken: per vakje telt de laatste wijziging, en elk scherm ziet welke set
// de ander open heeft. Openen zet niets op slot; alleen Vergrendel doet dat.
import { useEffect, useRef, useState } from "react";
import type { FormEvent, KeyboardEvent } from "react";
import { inUitslagindeling, kopVergelijking, metNu, rijen, ruimOp, vakje, verdeel, verschilTekst, zichtbaar } from "../cijfers/cijfers";
import type { Concept, Rij } from "../cijfers/cijfers";
import { Invoermelding } from "../Invoermelding";
import { LogoTegel } from "../LogoTegel";
import { Min, Plus, Slot } from "../iconen";
import { useRegie } from "../regie/regie";
import type { Aanvulling } from "../typen/Aanvulling";
import type { Bewerking } from "../typen/Bewerking";
import type { Cijfers as CijferStand } from "../typen/Cijfers";
import type { Cijferset } from "../typen/Cijferset";
import type { Partij } from "../typen/Partij";
import type { Soort } from "../typen/Soort";
import { Slotknop } from "./Slotknop";
import { Werkbalk } from "./Werkbalk";
import "./cijfers.css";

const BIJ = "cijfers";
const ZETELS = 150;
const SOORTEN: { soort: Soort; naam: string }[] = [
  { soort: "peiling", naam: "Peiling" },
  { soort: "exitpoll", naam: "Exitpoll" },
  { soort: "prognose", naam: "Prognose" },
  { soort: "tussenstand", naam: "Tussenstand" },
  { soort: "uitslag", naam: "Uitslag" },
];
const soortNaam = (s: Soort) => SOORTEN.find((x) => x.soort === s)?.naam ?? s;
const totaal = (s: Cijferset) => Object.values(s.zetels).reduce((a, b) => a + b, 0);

type Weergave = "sets" | "partijen";

export function Cijfers() {
  const regie = useRegie();
  const cijfers = regie.stand?.stand.cijfers;
  const volgnummer = regie.stand?.volgnummer ?? 0;
  const [weergave, setWeergave] = useState<Weergave>("sets");
  const [gekozen, setGekozen] = useState<string | null>(null);
  const sets = cijfers?.sets ?? [];
  const actief = sets.find((s) => s.id === gekozen) ?? sets.find((s) => s.id !== cijfers?.vergelijk_met) ?? sets[0] ?? null;
  const [nieuwePartij, setNieuwePartij] = useState(false);

  // Meld de andere Werkplekken welke set hier open staat; weg als je Cijfers verlaat.
  const bewerk = useRef(regie.bewerk);
  bewerk.current = regie.bewerk;
  const open = weergave === "sets" ? (actief?.id ?? null) : null;
  useEffect(() => bewerk.current(open), [open]);
  useEffect(() => () => bewerk.current(null), []);

  const melding = regie.melding?.bij === BIJ ? regie.melding.tekst : null;
  const letOp = cijfers?.partijen.filter((p) => p.kleur_tijdelijk).length ?? 0;

  return (
    <>
      <Werkbalk naam="Cijfers" hint={<AnderenBewerken bewerkingen={regie.bewerkingen} sets={sets} />}>
        <div className="filters" role="group" aria-label="Wat bewerken">
          <button className="filter" aria-pressed={weergave === "sets"} onClick={() => setWeergave("sets")}>
            Sets {sets.length}
          </button>
          <button className="filter" aria-pressed={weergave === "partijen"} onClick={() => setWeergave("partijen")}>
            Partijen {cijfers?.partijen.length ?? 0}
            {letOp > 0 && <span className="telling-let-op" title="Partijen met een tijdelijke kleur">{letOp}</span>}
          </button>
        </div>
        <label className="werkbalk-keuze">
          <span className="zacht">Vergelijk met</span>
          <select
            value={cijfers?.vergelijk_met ?? ""}
            disabled={regie.staat !== "verbonden"}
            onChange={(e) => void regie.stuur({ soort: "KiesVergelijkingsset", set: e.target.value || undefined }, BIJ)}
          >
            <option value="">Geen</option>
            {sets.map((s) => (
              <option key={s.id} value={s.id}>
                {s.label}
              </option>
            ))}
          </select>
        </label>
      </Werkbalk>
      {melding && (
        <div className="cijfers-melding" role="alert">
          {melding}
        </div>
      )}
      {cijfers && weergave === "sets" && (
        <div className="werkvlak-inhoud twee-delen cijfers">
          <div className="lijst cijfers-lijst">
            <SetLijst cijfers={cijfers} actief={actief?.id ?? null} kies={setGekozen} bewerkingen={regie.bewerkingen} />
            <div className="cijfers-invoermelding">
              <Invoermelding cijfers={cijfers} />
            </div>
          </div>
          {actief ? (
            <SetInvoer
              key={actief.id}
              cijfers={cijfers}
              set={actief}
              volgnummer={volgnummer}
              anderen={regie.bewerkingen.filter((b) => b.set === actief.id).map((b) => b.werkplek)}
              nieuwePartij={() => {
                setNieuwePartij(true);
                setWeergave("partijen");
              }}
            />
          ) : (
            <p className="cijfers-leeg zacht">Nog geen sets. Maak links een nieuwe set, of laat het invoerbestand ze brengen.</p>
          )}
        </div>
      )}
      {cijfers && weergave === "partijen" && <Partijen cijfers={cijfers} nieuw={nieuwePartij} setNieuw={setNieuwePartij} />}
    </>
  );
}

/** Rechts in de werkbalk: welke set elke andere Werkplek open heeft. */
function AnderenBewerken({ bewerkingen, sets }: { bewerkingen: Bewerking[]; sets: Cijferset[] }) {
  if (bewerkingen.length === 0) return <>Geen andere Werkplek in Cijfers</>;
  return (
    <span className="anderen">
      {bewerkingen.map((b, i) => (
        <span key={i}>
          {b.werkplek} bewerkt: <strong>{sets.find((s) => s.id === b.set)?.label ?? b.set}</strong>
        </span>
      ))}
    </span>
  );
}

function SetLijst({ cijfers, actief, kies, bewerkingen }: { cijfers: CijferStand; actief: string | null; kies: (id: string) => void; bewerkingen: Bewerking[] }) {
  const regie = useRegie();
  const [maken, setMaken] = useState(false);
  // Na "Maak set": de nieuwe set kiezen zodra hij in de Stand staat.
  const [voor, setVoor] = useState<string[] | null>(null);
  useEffect(() => {
    if (!voor) return;
    const nieuw = cijfers.sets.find((s) => !voor.includes(s.id));
    if (nieuw) {
      kies(nieuw.id);
      setVoor(null);
    }
  }, [cijfers.sets, voor, kies]);

  return (
    <table className="tabel setlijst">
      <thead>
        <tr>
          <th>Set</th>
          <th className="kol-getal">Zetels</th>
        </tr>
      </thead>
      <tbody>
        {cijfers.sets.map((s) => {
          const t = totaal(s);
          const wie = bewerkingen.filter((b) => b.set === s.id).map((b) => b.werkplek);
          const regel = [soortNaam(s.soort), s.tijd, s.id === cijfers.vergelijk_met && "vergelijkingsset"].filter(Boolean).join(" · ");
          return (
            <tr key={s.id} className={s.id === actief ? "gekozen" : ""} aria-selected={s.id === actief} onClick={() => kies(s.id)}>
              <td>
                <div className="set-naam titel-cel">
                  <span className="een-regel">{s.label}</span>
                  {s.vergrendeld && (
                    <span className="zacht" title="Vergrendeld">
                      <Slot />
                    </span>
                  )}
                </div>
                <div className="zacht">{regel}</div>
                {wie.length > 0 && <div className="set-bewerkt">{wie.join(", ")} bewerkt</div>}
              </td>
              <td className={`kol-getal ${t > 0 && t !== ZETELS ? "let-op" : t === 0 ? "zacht-getal" : ""}`} title={t > 0 && t !== ZETELS ? `Telt op tot ${t}, niet ${ZETELS}` : undefined}>
                {t}
              </td>
            </tr>
          );
        })}
        <tr className="rij-nieuw">
          <td colSpan={2}>
            {maken ? (
              <NieuweSet
                stop={() => setMaken(false)}
                klaar={(setsoort, label) => {
                  setVoor(cijfers.sets.map((s) => s.id));
                  void regie.stuur({ soort: "MaakSet", setsoort, label }, BIJ).then((a) => {
                    if (a.uitgevoerd) setMaken(false);
                    else setVoor(null);
                  });
                }}
              />
            ) : (
              <button className="knop knop-stil" onClick={() => setMaken(true)} disabled={regie.staat !== "verbonden"}>
                <Plus /> Nieuwe set
              </button>
            )}
          </td>
        </tr>
      </tbody>
    </table>
  );
}

function NieuweSet({ klaar, stop }: { klaar: (soort: Soort, label: string) => void; stop: () => void }) {
  const [soort, setSoort] = useState<Soort>("exitpoll");
  const [label, setLabel] = useState("");
  const verstuur = (e: FormEvent) => {
    e.preventDefault();
    if (label.trim()) klaar(soort, label.trim());
  };
  return (
    <form className="nieuwe-set" onSubmit={verstuur} onKeyDown={(e) => e.key === "Escape" && stop()}>
      <select value={soort} onChange={(e) => setSoort(e.target.value as Soort)} aria-label="Soort set">
        {SOORTEN.map((s) => (
          <option key={s.soort} value={s.soort}>
            {s.naam}
          </option>
        ))}
      </select>
      <input value={label} onChange={(e) => setLabel(e.target.value)} placeholder="Exitpoll 21.00" aria-label="Naam van de set" autoFocus maxLength={80} />
      <div className="knoppen">
        <button className="knop knop-hoofd" type="submit" disabled={!label.trim()}>
          Maak set
        </button>
        <button className="knop knop-stil" type="button" onClick={stop}>
          Annuleer
        </button>
      </div>
    </form>
  );
}

function SetInvoer({
  cijfers,
  set,
  volgnummer,
  anderen,
  nieuwePartij,
}: {
  cijfers: CijferStand;
  set: Cijferset;
  volgnummer: number;
  anderen: string[];
  nieuwePartij: () => void;
}) {
  const regie = useRegie();
  const [concept, setConcept] = useState<Concept>({});
  const laatste = useRef(volgnummer);
  laatste.current = volgnummer;
  useEffect(() => setConcept((c) => ruimOp(c, volgnummer)), [volgnummer]);

  const dicht = set.vergrendeld || regie.staat !== "verbonden";
  // Met de eigen wijzigingen die nog onderweg zijn, zodat snel klikken optelt.
  const alle = rijen(cijfers, set.id, regie.stand?.stand.uitslagindeling).map((r) => metNu(r, zichtbaar(concept, set.id, r.partij.sleutel, r.nu)));
  // Op een laptop of groter drie kolommen, zodat alle Partijen zonder scrollen in beeld staan.
  const kolommen = useKolommen();
  const delen = verdeel(alle, kolommen);
  const kop = kopVergelijking(cijfers);
  const t = alle.reduce((n, r) => n + r.nu, 0) + (set.zetels.overig ?? 0);

  const zet = (partij: string, zetels: number) => {
    const k = vakje(set.id, partij);
    setConcept((c) => ({ ...c, [k]: { zetels, tot: null } }));
    void regie.stuur({ soort: "ZetZetels", set: set.id, partij, zetels }, BIJ).then((a) =>
      setConcept((c) => {
        if (c[k]?.zetels !== zetels || c[k].tot !== null) return c;
        if (!a.uitgevoerd) {
          const { [k]: _, ...rest } = c;
          return rest;
        }
        return ruimOp({ ...c, [k]: { zetels, tot: a.volgnummer } }, laatste.current);
      }),
    );
  };

  const meta = [soortNaam(set.soort), set.tijd, set.bron].filter(Boolean).join(" · ");
  return (
    <div className="set-invoer">
      <div className="invoer-kop">
        <div className="invoer-titel">
          <span className="zacht">
            {meta}
            {anderen.length > 0 && <span className="set-bewerkt"> · ook open bij {anderen.join(", ")}</span>}
          </span>
          <h3 className="kop-regel">{set.label}</h3>
        </div>
        <div className="invoer-kop-knoppen">
          <span className={`set-totaal ${t !== ZETELS ? "let-op" : ""}`}>
            {t} van {ZETELS} Zetels
          </span>
          <Slotknop
            op={set.vergrendeld}
            disabled={regie.staat !== "verbonden"}
            wissel={() => void regie.stuur({ soort: set.vergrendeld ? "OntgrendelSet" : "VergrendelSet", set: set.id }, BIJ)}
          />
        </div>
      </div>
      <Zetelbalk rijen={alle} />
      <div className="partijkolommen" style={{ gridTemplateColumns: `repeat(${kolommen}, minmax(0, 1fr))` }}>
        {delen.map((kolom, k) => (
          <table key={k} className="tabel partijtabel">
            <thead>
              <tr>
                <th>Partij</th>
                <th className="kol-nu">Nu</th>
                <th className="kol-getal">{kop ?? "Vorige"}</th>
                <th className="kol-getal kol-verschil">Verschil</th>
              </tr>
            </thead>
            <tbody>
              {kolom.map((r, i) => (
                <PartijRij
                  key={r.partij.sleutel}
                  rij={r}
                  volgorde={k * delen[0].length + i}
                  dicht={dicht}
                  zet={(n) => zet(r.partij.sleutel, n)}
                />
              ))}
            </tbody>
          </table>
        ))}
      </div>
      <div className="set-voet">
        <button className="knop knop-stil" onClick={nieuwePartij}>
          <Plus /> Nieuwe Partij
        </button>
        <span className="zacht">
          {set.vergrendeld
            ? "Op slot: kies Ontgrendel rechtsboven om cijfers te wijzigen."
            : kop
              ? `Verschil tegen ${cijfers.sets.find((s) => s.id === cijfers.vergelijk_met)?.label}, met de Voorgangers opgeteld.`
              : "Kies bovenin een set om mee te vergelijken."}
        </span>
      </div>
    </div>
  );
}

/** De 150 Zetels in partijkleuren, in de volgorde van de Uitslagindeling, met de streep bij 76. */
function Zetelbalk({ rijen }: { rijen: Rij[] }) {
  return (
    <div className="cijfers-zetelbalk" role="img" aria-label="Zetelbalk">
      {rijen
        .filter((r) => r.nu > 0)
        .map((r) => (
          <span key={r.partij.sleutel} style={{ width: `${(Math.min(r.nu, ZETELS) / ZETELS) * 100}%`, background: r.partij.kleur }} title={`${r.partij.kort} ${r.nu}`} />
        ))}
      <span className="zetelbalk-meerderheid" style={{ left: `${(75.5 / ZETELS) * 100}%` }} />
    </div>
  );
}

function PartijRij({ rij, volgorde, dicht, zet }: { rij: Rij; volgorde: number; dicht: boolean; zet: (n: number) => void }) {
  const { partij: p, nu: waarde, verschil, nieuw } = rij;
  return (
    <tr>
      <td>
        <span className="partij-cel">
          <LogoTegel partij={p} />
          <span className="een-regel titel-cel" title={p.naam}>
            {p.kort}
          </span>
        </span>
      </td>
      <td className="kol-nu">
        {dicht ? (
          // Op slot: alleen het getal, zonder knoppen die lijken te werken.
          <span className="stapper stapper-dicht" title="Op slot: ontgrendel de set om te wijzigen">
            {waarde}
          </span>
        ) : (
          <span className="stapper">
            <button className="stap" aria-label={`${p.kort} min 1`} disabled={waarde <= 0} onClick={() => zet(waarde - 1)}>
              <Min />
            </button>
            <ZetelVak partij={p} waarde={waarde} volgorde={volgorde} dicht={dicht} zet={zet} />
            <button className="stap" aria-label={`${p.kort} plus 1`} disabled={waarde >= ZETELS} onClick={() => zet(waarde + 1)}>
              <Plus />
            </button>
          </span>
        )}
      </td>
      <td className="kol-getal zacht-getal">{rij.eerder ?? ""}</td>
      <td className="kol-getal kol-verschil">
        {verschil !== null && (nieuw ? <span className="nieuw">{verschilTekst(verschil)} nieuw</span> : verschilTekst(verschil))}
      </td>
    </tr>
  );
}

/**
 * Het getal van één Partij. Tijdens het typen houdt het vakje je eigen tekst; Enter of verlaten
 * stuurt hem en Enter springt naar de volgende Partij. Pijl omhoog en omlaag tellen er één bij of af.
 */
function ZetelVak({ partij, waarde, volgorde, dicht, zet }: { partij: Partij; waarde: number; volgorde: number; dicht: boolean; zet: (n: number) => void }) {
  const [tekst, setTekst] = useState<string | null>(null);
  const annuleer = useRef(false);
  const stuur = (t: string) => {
    const n = Number(t.trim());
    if (t.trim() !== "" && Number.isInteger(n) && n >= 0 && n <= ZETELS && n !== waarde) zet(n);
  };
  const toets = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      const volgende = document.querySelector<HTMLInputElement>(`[data-vakje="${volgorde + 1}"]`);
      if (volgende) volgende.focus();
      else e.currentTarget.blur();
    } else if (e.key === "Escape") {
      annuleer.current = true;
    } else if (e.key === "ArrowUp" || e.key === "ArrowDown") {
      e.preventDefault();
      const n = Math.min(ZETELS, Math.max(0, waarde + (e.key === "ArrowUp" ? 1 : -1)));
      setTekst(null);
      if (n !== waarde) zet(n);
    }
  };
  const gewijzigd = tekst !== null && tekst !== String(waarde);
  return (
    <span className="zetelvak-plek">
      <input
        className="zetelvak"
        inputMode="numeric"
        aria-label={`Zetels ${partij.kort}`}
        data-vakje={volgorde}
        disabled={dicht}
        value={tekst ?? String(waarde)}
        onFocus={(e) => e.currentTarget.select()}
        onChange={(e) => setTekst(e.target.value.replace(/\D/g, "").slice(0, 3))}
        onKeyDown={toets}
        onBlur={() => {
          if (!annuleer.current && tekst !== null) stuur(tekst);
          annuleer.current = false;
          setTekst(null);
        }}
      />
      {/* Een gewijzigd getal is nog niet bewaard: zeg hier wat Enter en Esc doen (Esc gooit het weg). */}
      {gewijzigd && (
        <span className="zetelvak-hint" role="status">
          <kbd>Enter</kbd> bewaart · <kbd>Esc</kbd> annuleert
        </span>
      )}
    </span>
  );
}

/** De partijlijst: links alle Partijen, rechts de gekozen Partij om aan te vullen, of een nieuwe. */
function Partijen({ cijfers, nieuw, setNieuw }: { cijfers: CijferStand; nieuw: boolean; setNieuw: (n: boolean) => void }) {
  const regie = useRegie();
  const lijst = inUitslagindeling(cijfers, regie.stand?.stand.uitslagindeling);
  const [gekozen, setGekozen] = useState<string | null>(null);
  const [voor, setVoor] = useState<string[] | null>(null);
  useEffect(() => {
    if (!voor) return;
    const n = cijfers.partijen.find((p) => !voor.includes(p.sleutel));
    if (n) {
      setGekozen(n.sleutel);
      setVoor(null);
    }
  }, [cijfers.partijen, voor]);
  const actief = nieuw ? null : (lijst.find((p) => p.sleutel === gekozen) ?? lijst[0] ?? null);
  const opvolger = (s: string) => cijfers.partijen.find((p) => p.voorgangers.includes(s));

  return (
    <div className="werkvlak-inhoud twee-delen cijfers">
      <div className="lijst">
        <table className="tabel partijlijst">
          <thead>
            <tr>
              <th className="kol-logo">Logo</th>
              <th>Partij</th>
            </tr>
          </thead>
          <tbody>
            {lijst.map((p) => {
              const op = opvolger(p.sleutel);
              const regel = [p.fractie_zonder_lijst && "Fractie zonder lijst", op && `Voorganger van ${op.kort}`, p.voorgangers.length > 0 && `na ${p.voorgangers.map((v) => cijfers.partijen.find((q) => q.sleutel === v)?.kort ?? v).join(", ")}`]
                .filter(Boolean)
                .join(" · ");
              return (
                <tr
                  key={p.sleutel}
                  className={p.sleutel === actief?.sleutel ? "gekozen" : ""}
                  aria-selected={p.sleutel === actief?.sleutel}
                  onClick={() => {
                    setNieuw(false);
                    setGekozen(p.sleutel);
                  }}
                >
                  <td className="kol-logo">
                    <LogoTegel partij={p} />
                  </td>
                  <td>
                    <div className="titel-cel een-regel">
                      {p.kort} <span className="zacht">{p.naam !== p.kort ? p.naam : ""}</span>
                    </div>
                    {(regel || p.kleur_tijdelijk) && (
                      <div className="zacht een-regel">
                        {p.kleur_tijdelijk && <span className="let-op">tijdelijke kleur</span>}
                        {p.kleur_tijdelijk && regel && " · "}
                        {regel}
                      </div>
                    )}
                  </td>
                </tr>
              );
            })}
            <tr className="rij-nieuw">
              <td colSpan={2}>
                <button className="knop knop-stil" onClick={() => setNieuw(true)} disabled={regie.staat !== "verbonden"}>
                  <Plus /> Nieuwe Partij
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      {nieuw ? (
        <NieuwePartij
          stop={() => setNieuw(false)}
          klaar={(naam, kort, kleur, fractie_zonder_lijst) => {
            setVoor(cijfers.partijen.map((p) => p.sleutel));
            void regie.stuur({ soort: "VoegPartijToe", naam, kort, kleur, fractie_zonder_lijst }, BIJ).then((a) => {
              if (a.uitgevoerd) setNieuw(false);
              else setVoor(null);
            });
          }}
        />
      ) : actief ? (
        <PartijAanvullen key={actief.sleutel} cijfers={cijfers} partij={actief} />
      ) : (
        <p className="cijfers-leeg zacht">Nog geen Partijen. Voeg er een toe, of laat het invoerbestand de partijlijst brengen.</p>
      )}
    </div>
  );
}

function NieuwePartij({ klaar, stop }: { klaar: (naam: string, kort: string, kleur: string | undefined, fractieZonderLijst: boolean) => void; stop: () => void }) {
  const [naam, setNaam] = useState("");
  const [kort, setKort] = useState("");
  const [kleur, setKleur] = useState<string | null>(null);
  const [zonderLijst, setZonderLijst] = useState(false);
  const verstuur = (e: FormEvent) => {
    e.preventDefault();
    if (naam.trim()) klaar(naam.trim(), kort.trim(), kleur ?? undefined, zonderLijst);
  };
  return (
    <form className="paneel-detail partij-paneel" onSubmit={verstuur}>
      <header>
        <h3 className="kop-regel">Nieuwe Partij</h3>
        <p className="zacht">Zonder kleur krijgt de Partij een tijdelijke kleur die goed afwijkt. Zonder logo staat de korte naam op die plek.</p>
      </header>
      <label className="veld">
        <span className="label">Naam</span>
        <input value={naam} onChange={(e) => setNaam(e.target.value)} placeholder="Nieuwe Beweging" autoFocus maxLength={80} />
      </label>
      <label className="veld">
        <span className="label">Korte naam</span>
        <input value={kort} onChange={(e) => setKort(e.target.value)} placeholder="NB" maxLength={16} />
      </label>
      <div className="veld">
        <span className="label">Kleur</span>
        <span className="kleur-keuze">
          <input type="color" value={kleur ?? "#808080"} onChange={(e) => setKleur(e.target.value)} aria-label="Kleur" />
          <span className="zacht">{kleur ? kleur.toUpperCase() : "Tijdelijke kleur"}</span>
          {kleur && (
            <button type="button" className="knop knop-stil" onClick={() => setKleur(null)}>
              Geen eigen kleur
            </button>
          )}
        </span>
      </div>
      <label className="vinkje">
        <input type="checkbox" checked={zonderLijst} onChange={(e) => setZonderLijst(e.target.checked)} />
        Fractie zonder lijst
      </label>
      <div className="knoppen">
        <button className="knop knop-hoofd" type="submit" disabled={!naam.trim()}>
          Voeg toe
        </button>
        <button className="knop knop-stil" type="button" onClick={stop}>
          Annuleer
        </button>
      </div>
    </form>
  );
}

/** Een Partij aanvullen. Elk veld gaat apart naar de server, zodat twee Werkplekken elk een ander veld kunnen bijwerken. */
function PartijAanvullen({ cijfers, partij }: { cijfers: CijferStand; partij: Partij }) {
  const regie = useRegie();
  const dicht = regie.staat !== "verbonden";
  const vul = (aanvulling: Aanvulling) => void regie.stuur({ soort: "VulPartijAan", partij: partij.sleutel, aanvulling }, BIJ);
  const opvolger = cijfers.partijen.find((p) => p.voorgangers.includes(partij.sleutel));
  const kandidaten = cijfers.partijen.filter((p) => p.sleutel !== partij.sleutel && !partij.voorgangers.includes(p.sleutel));
  return (
    <div className="paneel-detail partij-paneel">
      <header className="partij-kop">
        <LogoTegel partij={partij} />
        <div>
          <h3 className="kop-regel">{partij.naam}</h3>
          <p className="zacht">
            {partij.fractie_zonder_lijst ? "Fractie zonder lijst" : "Partij"}
            {opvolger && ` · Voorganger van ${opvolger.naam}`}
          </p>
        </div>
      </header>
      <Tekstveld label="Naam" waarde={partij.naam} dicht={dicht} bewaar={(naam) => vul({ naam })} />
      <Tekstveld label="Korte naam" waarde={partij.kort} dicht={dicht} bewaar={(kort) => vul({ kort })} />
      <div className="veld">
        <span className="label">Kleur</span>
        <span className="kleur-keuze">
          <input type="color" value={partij.kleur.toLowerCase()} disabled={dicht} aria-label="Kleur" onChange={(e) => vul({ kleur: e.target.value })} />
          <span className="zacht">{partij.kleur}</span>
          {partij.kleur_tijdelijk && <span className="let-op">tijdelijke kleur: kies een eigen kleur</span>}
        </span>
      </div>
      <Tekstveld
        label="Logo"
        waarde={partij.logo ?? ""}
        dicht={dicht}
        hint="Adres of pad van het logo dat de omroep aanlevert. Leeg: de korte naam staat op die plek."
        bewaar={(logo) => vul({ logo })}
      />
      <label className="vinkje">
        <input type="checkbox" checked={partij.fractie_zonder_lijst} disabled={dicht} onChange={(e) => vul({ fractie_zonder_lijst: e.target.checked })} />
        Fractie zonder lijst
      </label>
      <div className="veld">
        <span className="label">Voorgangers</span>
        <p className="zacht">Winst en verlies rekent tegen de som van de Zetels van de Partij en haar Voorgangers. Een Voorganger hoort bij hooguit één Partij.</p>
        {partij.voorgangers.length > 0 && (
          <table className="tabel voorgangers">
            <tbody>
              {partij.voorgangers.map((v) => {
                const vp = cijfers.partijen.find((p) => p.sleutel === v);
                return (
                  <tr key={v}>
                    <td className="titel-cel">{vp?.naam ?? v}</td>
                    <td className="kol-actie">
                      <button className="knop knop-stil" disabled={dicht} onClick={() => void regie.stuur({ soort: "OntkoppelVoorganger", partij: partij.sleutel, voorganger: v }, BIJ)}>
                        Ontkoppel
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
        <select
          value=""
          disabled={dicht}
          aria-label="Voorganger toevoegen"
          onChange={(e) => e.target.value && void regie.stuur({ soort: "KoppelVoorganger", partij: partij.sleutel, voorganger: e.target.value }, BIJ)}
        >
          <option value="">Voorganger toevoegen</option>
          {kandidaten.map((p) => {
            const al = cijfers.partijen.find((q) => q.voorgangers.includes(p.sleutel));
            return (
              <option key={p.sleutel} value={p.sleutel} disabled={!!al}>
                {p.kort}
                {al ? ` (al Voorganger van ${al.kort})` : ""}
              </option>
            );
          })}
        </select>
      </div>
    </div>
  );
}

/** Een tekstveld dat de Stand toont en pas bij Enter of verlaten bewaart. */
function Tekstveld({ label, waarde, dicht, hint, bewaar }: { label: string; waarde: string; dicht: boolean; hint?: string; bewaar: (t: string) => void }) {
  const [tekst, setTekst] = useState<string | null>(null);
  const annuleer = useRef(false);
  return (
    <label className="veld">
      <span className="label">{label}</span>
      <input
        value={tekst ?? waarde}
        disabled={dicht}
        maxLength={200}
        onChange={(e) => setTekst(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") e.currentTarget.blur();
          if (e.key === "Escape") annuleer.current = true;
        }}
        onBlur={() => {
          if (!annuleer.current && tekst !== null && tekst !== waarde) bewaar(tekst);
          annuleer.current = false;
          setTekst(null);
        }}
      />
      {hint && <span className="zacht">{hint}</span>}
    </label>
  );
}

/** Drie kolommen Partijen vanaf 1360 px breed, anders twee. */
function useKolommen(): number {
  const meet = () => (window.innerWidth >= 1360 ? 3 : 2);
  const [n, setN] = useState(meet);
  useEffect(() => {
    const kijk = () => setN(meet());
    window.addEventListener("resize", kijk);
    return () => window.removeEventListener("resize", kijk);
  }, []);
  return n;
}
