// Vormgeving: de Stijl van het programma (vijf kleuren, drie lettertypes, het programmalogo en de
// snelheid) en de Sjablonen (Sjablonen.tsx). Een wijziging geldt pas na Opslaan: het Voorbeeld toont hem
// meteen, de Graphic in de Ether houdt haar vorm tot de volgende Toon.
import { useEffect, useRef, useState } from "react";
import { useRegie } from "../regie/regie";
import { MAX_LOGOTEKST, MAX_SNELHEID, MIN_SNELHEID, STANDAARD_STIJL } from "../typen/standaardStijl";
import type { Stijl } from "../typen/Stijl";
import { KLEUREN, LETTERS, fout, gewijzigd, letterKeuzes, letterNaam } from "../vormgeving/stijl";
import { KleurovergangKeuze } from "./KleurovergangKeuze";
import { SjablonenDeel } from "./Sjablonen";
import { Werkbalk } from "./Werkbalk";
import "./vormgeving.css";

const VORMGEVING = "vormgeving";

type Deel = "stijl" | "sjablonen";

export function Vormgeving() {
  const [deel, setDeel] = useState<Deel>("stijl");
  return (
    <>
      <Werkbalk naam="Vormgeving" hint="Een wijziging geldt na Opslaan, in Live pas bij de volgende Take">
        <div className="filters">
          <button className="filter" aria-pressed={deel === "stijl"} onClick={() => setDeel("stijl")}>
            Stijl
          </button>
          <button className="filter" aria-pressed={deel === "sjablonen"} onClick={() => setDeel("sjablonen")}>
            Sjablonen
          </button>
        </div>
      </Werkbalk>
      {deel === "stijl" ? <StijlDeel /> : <SjablonenDeel />}
    </>
  );
}

function StijlDeel() {
  const regie = useRegie();
  const opgeslagen = regie.stand?.stand.stijl;
  // `null`: niets gewijzigd, het formulier volgt de opgeslagen Stijl (ook als een ander opslaat)
  const [concept, setConcept] = useState<Stijl | null>(null);
  const [bezig, setBezig] = useState(false);
  const logo = useBestand((concept ?? opgeslagen)?.programmalogo.bestand ?? null);
  if (!opgeslagen) return null;
  const stijl = concept ?? opgeslagen;
  const anders = gewijzigd(stijl, opgeslagen);
  const reden = fout(stijl);
  const verbonden = regie.staat === "verbonden";
  const melding = regie.melding?.bij === VORMGEVING ? regie.melding.tekst : null;
  const zet = (wijzig: (s: Stijl) => void) => {
    const s: Stijl = structuredClone(stijl);
    wijzig(s);
    setConcept(s);
  };
  const opslaan = async () => {
    setBezig(true);
    const a = await regie.stuur({ soort: "ZetStijl", stijl }, VORMGEVING);
    setBezig(false);
    if (a.uitgevoerd) setConcept(null);
  };
  const plaats = async (bestand: File | undefined, wijzig: (s: Stijl, naam: string) => void) => {
    if (!bestand) return;
    const naam = await regie.plaatsBestand(bestand, VORMGEVING);
    if (naam) zet((s) => wijzig(s, naam));
  };

  return (
    <div className="werkvlak-inhoud vormgeving-stijl">
      <div className="stijl-velden">
        <section className="stijl-deel">
          <h3 className="label">Kleuren</h3>
          <table className="tabel stijl-kleuren">
            <tbody>
              {KLEUREN.map((k) => (
                <tr key={k.sleutel}>
                  <td className="titel-cel">{k.naam}</td>
                  <td>
                    <input
                      type="color"
                      aria-label={`${k.naam}, kies een kleur`}
                      value={/^#[0-9a-f]{6}$/i.test(stijl.kleuren[k.sleutel]) ? stijl.kleuren[k.sleutel] : "#000000"}
                      onChange={(e) => zet((s) => (s.kleuren[k.sleutel] = e.target.value))}
                    />
                  </td>
                  <td>
                    <input
                      className="stijl-hex"
                      aria-label={`${k.naam} als hexcode`}
                      value={stijl.kleuren[k.sleutel]}
                      maxLength={7}
                      spellCheck={false}
                      onChange={(e) => zet((s) => (s.kleuren[k.sleutel] = e.target.value.trim()))}
                    />
                  </td>
                  <td className="zacht">{k.waarvoor}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>

        <section className="stijl-deel">
          <h3 className="label">Lettertypes</h3>
          <table className="tabel stijl-letters">
            <tbody>
              {LETTERS.map((l) => (
                <tr key={l.sleutel}>
                  <td className="titel-cel">{l.naam}</td>
                  <td>
                    <select
                      aria-label={`Lettertype ${l.naam}`}
                      value={stijl.lettertypes[l.sleutel]}
                      onChange={(e) => zet((s) => (s.lettertypes[l.sleutel] = e.target.value))}
                    >
                      {letterKeuzes(stijl, opgeslagen).map((b) => (
                        <option key={b} value={b}>
                          {letterNaam(b)}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td>
                    <Kiezer
                      tekst="Eigen lettertype…"
                      accept=".ttf,.otf,font/ttf,font/otf"
                      uit={!verbonden}
                      kies={(f) => void plaats(f, (s, naam) => (s.lettertypes[l.sleutel] = naam))}
                    />
                  </td>
                  <td className="zacht">{l.waarvoor}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>

        <section className="stijl-deel">
          <h3 className="label">Programmalogo</h3>
          <div className="stijl-logo">
            <Logobeeld naam={stijl.programmalogo.bestand ?? null} logo={logo} tekst={stijl.programmalogo.tekst} />
            <div className="stijl-logo-keuzes">
              <div className="knoppen">
                <Kiezer
                  tekst="Beeld plaatsen…"
                  accept="image/png,image/jpeg"
                  uit={!verbonden}
                  kies={(f) => void plaats(f, (s, naam) => (s.programmalogo.bestand = naam))}
                />
                <button className="knop" disabled={!stijl.programmalogo.bestand} onClick={() => zet((s) => (s.programmalogo.bestand = null))}>
                  Zonder beeld
                </button>
              </div>
              <label className="veld">
                <span className="veld-kop">
                  <span className="label">Tekst zonder beeld</span>
                  <span className="zacht">
                    {[...stijl.programmalogo.tekst].length}/{MAX_LOGOTEKST}
                  </span>
                </span>
                <input
                  value={stijl.programmalogo.tekst}
                  maxLength={MAX_LOGOTEKST}
                  onChange={(e) => zet((s) => (s.programmalogo.tekst = e.target.value))}
                />
              </label>
            </div>
          </div>
        </section>

        <section className="stijl-deel">
          <h3 className="label">Snelheid</h3>
          <div className="stijl-snelheid">
            <span className="zacht">Langzaam</span>
            <input
              type="range"
              aria-label="Snelheid van alle animaties"
              min={MIN_SNELHEID}
              max={MAX_SNELHEID}
              step={5}
              value={stijl.snelheid}
              onChange={(e) => zet((s) => (s.snelheid = Number(e.target.value)))}
            />
            <span className="zacht">Snel</span>
            <span className="stijl-procent">{stijl.snelheid}%</span>
          </div>
          <p className="zacht">Alle animaties en de Kleurovergang in de Zaal. 100% is normaal; een Sjabloon kan daar nog een eigen snelheid bovenop hebben.</p>
        </section>

        <section className="stijl-deel">
          <h3 className="label">Stoelen in de Zaal</h3>
          <div className="veld">
            <span className="veld-kop">
              <span className="label">Kleurovergang</span>
              <span className="zacht">een Sjabloon kan een eigen hebben</span>
            </span>
            <KleurovergangKeuze waarde={stijl.kleurovergang} zet={(k) => zet((s) => (s.kleurovergang = k))} />
          </div>
          <div className="veld">
            <span className="label">Kleurkracht</span>
            <div className="stijl-snelheid">
              <span className="zacht">Zacht</span>
              <input
                type="range"
                aria-label="Kleurkracht van de Stoelen"
                min={0}
                max={100}
                step={5}
                value={stijl.kleurkracht}
                onChange={(e) => zet((s) => (s.kleurkracht = Number(e.target.value)))}
              />
              <span className="zacht">Krachtig</span>
              <span className="stijl-procent">{stijl.kleurkracht}%</span>
            </div>
          </div>
          <p className="zacht">Hoe hoger, hoe voller de kleur van de Stoelen en hoe donkerder de rest van de Zaal zolang ze gekleurd zijn.</p>
        </section>
      </div>

      <aside className="stijl-schets-kolom">
        <h3 className="label">Schets</h3>
        <Schets stijl={stijl} logo={logo.url} />
        <p className="zacht">
          Alleen de kleuren en het logo. Het Voorbeeld bovenin toont de echte Graphic in de nieuwe Stijl, zodra je opslaat. De Ether houdt haar vorm tot
          de volgende Take.
        </p>
      </aside>

      <div className="actiebalk stijl-actiebalk">
        <button
          className="knop"
          disabled={!gewijzigd(stijl, STANDAARD_STIJL)}
          onClick={() => setConcept(structuredClone(STANDAARD_STIJL))}
          title="Zet de standaard-Stijl terug in het formulier. Daarna Opslaan."
        >
          Standaard-Stijl
        </button>
        <button className="knop" disabled={!anders} onClick={() => setConcept(null)}>
          Wijzigingen weggooien
        </button>
        <div className="actiebalk-rechts">
          {(reden ?? melding) && (
            <span className="melding" role="alert">
              {reden ?? melding}
            </span>
          )}
          {!reden && !melding && <span className={anders ? "tekst-let-op" : "zacht"}>{anders ? "Niet opgeslagen" : "Opgeslagen"}</span>}
          <button className="knop knop-hoofd" disabled={!anders || !!reden || !verbonden || bezig} onClick={() => void opslaan()}>
            Opslaan
          </button>
        </div>
      </div>
    </div>
  );
}

/** Een knop die een bestand laat kiezen. */
export function Kiezer({ tekst, accept, uit, kies }: { tekst: string; accept: string; uit: boolean; kies(f: File | undefined): void }) {
  const invoer = useRef<HTMLInputElement>(null);
  return (
    <>
      <button className="knop" disabled={uit} onClick={() => invoer.current?.click()}>
        {tekst}
      </button>
      <input
        ref={invoer}
        type="file"
        accept={accept}
        hidden
        onChange={(e) => {
          kies(e.target.files?.[0]);
          e.target.value = "";
        }}
      />
    </>
  );
}

/** Een bestand van de server als object-URL. `ontbreekt`: de server heeft het niet. */
export function useBestand(naam: string | null): { url: string | null; ontbreekt: boolean } {
  const regie = useRegie();
  // de Regie verandert bij elke Stand; het bestand alleen opnieuw halen als de naam verandert
  const haal = useRef(regie.haalBestand);
  haal.current = regie.haalBestand;
  const [binnen, setBinnen] = useState<{ naam: string; url: string | null } | null>(null);
  useEffect(() => {
    if (!naam) return;
    let weg = false;
    let gemaakt: string | null = null;
    void haal.current(naam).then((url) => {
      gemaakt = url;
      if (weg) {
        if (url) URL.revokeObjectURL(url);
      } else setBinnen({ naam, url });
    });
    return () => {
      weg = true;
      if (gemaakt) URL.revokeObjectURL(gemaakt);
    };
  }, [naam]);
  const nu = naam && binnen?.naam === naam ? binnen : null;
  return { url: nu?.url ?? null, ontbreekt: !!nu && !nu.url };
}

/** Het programmalogo zoals de server het heeft, of de tekst die de Renderer zonder beeld toont. */
function Logobeeld({ naam, logo, tekst }: { naam: string | null; logo: { url: string | null; ontbreekt: boolean }; tekst: string }) {
  return (
    <div className="stijl-logo-vak">
      <div className="stijl-logo-beeld">{logo.url ? <img src={logo.url} alt="Programmalogo" /> : <span>{tekst}</span>}</div>
      <span className="zacht">
        {!naam ? "Zonder beeld: de Renderer toont de tekst." : logo.ontbreekt ? `${naam} staat niet op de server: de Renderer toont de tekst.` : naam}
      </span>
    </div>
  );
}

/** Een schets van een naambalk over een paneel, in de kleuren van de Stijl (geen dashboardkleuren). */
function Schets({ stijl, logo }: { stijl: Stijl; logo: string | null }) {
  const k = stijl.kleuren;
  return (
    <div className="stijl-schets" style={{ background: "#0d1220" }} aria-label="Schets van de Stijl">
      <div className="schets-waas" style={{ background: `linear-gradient(90deg, ${k.achtergrond}f2 0%, ${k.achtergrond}00 70%)` }} />
      <div className="schets-paneel" style={{ background: `linear-gradient(160deg, ${k.achtergrond}f2, ${k.vlak}f2 62%)` }}>
        <div className="schets-kicker" style={{ color: k.hoofd }}>
          <span className="schets-streepje" style={{ background: k.accent }} />
          UITSLAG
        </div>
        {[48, 36, 22].map((w, i) => (
          <div key={i} className="schets-balk">
            <span style={{ width: `${w}%`, background: k.hoofd }} />
            <b style={{ color: k.tekst }}>{w}</b>
          </div>
        ))}
      </div>
      <div className="schets-naambalk" style={{ background: `${k.vlak}ed` }}>
        <div className="schets-kop" style={{ color: k.hoofd }}>
          De Uitslagavond
        </div>
        <div className="schets-regel" style={{ color: k.tekst }}>
          Voorbeeldgegevens
        </div>
      </div>
      <div className="schets-bug" style={{ color: k.tekst }}>
        {logo ? <img src={logo} alt="" /> : stijl.programmalogo.tekst}
      </div>
    </div>
  );
}
