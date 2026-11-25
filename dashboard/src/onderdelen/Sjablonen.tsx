// Vormgeving > Sjablonen: één blokkeneditor voor vaste en eigen Sjablonen (ADR 0003). Links de
// lijst, in het midden een schets van het beeld met de Blokken (klik kiest, slepen verplaatst,
// het hoekje rechtsonder maakt groter) en de lijst Blokken, rechts de instellingen van het Blok of
// het hele Sjabloon. Geen tijdlijn: elk Blok heeft een animatie in, uit en "start na". Een
// wijziging geldt na Opslaan, in de Ether pas bij de volgende Toon.
import { useRef, useState } from "react";
import type { CSSProperties, PointerEvent as ReactPointerEvent } from "react";
import { cameraKeuzes } from "../regie/camerarit";
import { useRegie } from "../regie/regie";
import type { Blok } from "../typen/Blok";
import type { Bloksoort } from "../typen/Bloksoort";
import type { Kleurkeuze } from "../typen/Kleurkeuze";
import type { Letterkeuze } from "../typen/Letterkeuze";
import type { Sjabloon } from "../typen/Sjabloon";
import type { Stijl } from "../typen/Stijl";
import type { Stijlkleur } from "../typen/Stijlkleur";
import type { Stijlletter } from "../typen/Stijlletter";
import type { Tekstbron } from "../typen/Tekstbron";
import type { Uitlijning } from "../typen/Uitlijning";
import { MAX_SNELHEID, MIN_SNELHEID, STANDAARD_STIJL } from "../typen/standaardStijl";
import {
  ACHTERGRONDEN,
  BLOKNAMEN,
  ERIN,
  ERUIT,
  GEWONE_BLOKKEN,
  STOELEN,
  TEKSTBRONNEN,
  blokTekst,
  fout,
  gewijzigdSjabloon,
  lijst,
  nieuwBlok,
  nieuwSjabloon,
  standaardVan,
  tekstbronNaam,
  verplaatsBlok,
  wisBlok,
} from "../vormgeving/sjablonen";
import { letterKeuzes, letterNaam } from "../vormgeving/stijl";
import { KleurovergangKeuze } from "./KleurovergangKeuze";
import { Kiezer, useBestand } from "./Vormgeving";

const VORMGEVING = "vormgeving";
const STIJLKLEUREN: { kleur: Stijlkleur; naam: string }[] = [
  { kleur: "Hoofd", naam: "Hoofd" },
  { kleur: "Accent", naam: "Accent" },
  { kleur: "Tekst", naam: "Tekst" },
  { kleur: "Vlak", naam: "Vlak" },
  { kleur: "Achtergrond", naam: "Achtergrond" },
];
const STIJLLETTERS: Stijlletter[] = ["Kop", "Tekst", "Cijfers"];

export function SjablonenDeel() {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [keuze, setGekozenId] = useState("titelkaart");
  // `concepten`: wat de omroep wijzigde en nog niet opsloeg, op id; ook een nieuw eigen Sjabloon
  const [concepten, setConcepten] = useState<Record<string, Sjabloon>>({});
  const [blokIndex, setBlokIndex] = useState<number | null>(null);
  const [bezig, setBezig] = useState(false);
  if (!stand) return null;

  const rijen = lijst(stand.sjablonen);
  // een eigen Sjabloon dat een ander net wiste: terug naar de Titelkaart
  const gekozenId = concepten[keuze] || rijen.some((r) => r.sjabloon.id === keuze) ? keuze : "titelkaart";
  const nieuwe = Object.values(concepten).filter((c) => !rijen.some((r) => r.sjabloon.id === c.id));
  const rij = rijen.find((r) => r.sjabloon.id === gekozenId);
  const opgeslagen = rij?.sjabloon ?? null;
  const concept = concepten[gekozenId] ?? opgeslagen;
  if (!concept) return null;
  const vast = standaardVan(concept.id);
  const anders = !opgeslagen || gewijzigdSjabloon(concept, opgeslagen);
  const reden = fout(concept);
  const verbonden = regie.staat === "verbonden";
  const melding = regie.melding?.bij === VORMGEVING ? regie.melding.tekst : null;
  const blok = blokIndex !== null ? concept.blokken[blokIndex] : undefined;

  const kies = (id: string) => {
    setGekozenId(id);
    setBlokIndex(null);
  };
  const zet = (nieuw: Sjabloon) => setConcepten((c) => ({ ...c, [nieuw.id]: nieuw }));
  const zetBlok = (i: number, wijzig: (b: Blok) => void) => {
    const s = structuredClone(concept);
    wijzig(s.blokken[i]);
    zet(s);
  };
  const vergeet = (id: string) =>
    setConcepten((c) => {
      const { [id]: _weg, ...rest } = c;
      return rest;
    });
  const opslaan = async () => {
    setBezig(true);
    const a = await regie.stuur({ soort: "ZetSjabloon", sjabloon: concept }, VORMGEVING);
    setBezig(false);
    if (a.uitgevoerd) vergeet(concept.id);
  };
  const wis = async () => {
    if (!opgeslagen) {
      vergeet(concept.id);
      kies("titelkaart");
      return;
    }
    const a = await regie.stuur({ soort: "WisSjabloon", id: concept.id }, VORMGEVING);
    if (a.uitgevoerd) {
      vergeet(concept.id);
      if (!vast) kies("titelkaart");
    }
  };
  const nieuwEigen = () => {
    const s = nieuwSjabloon([...rijen.map((r) => r.sjabloon.id), ...Object.keys(concepten)]);
    zet(s);
    kies(s.id);
  };
  const voegToe = (soort: Bloksoort["blok"]) => {
    const s = structuredClone(concept);
    s.blokken.push(nieuwBlok(soort));
    zet(s);
    setBlokIndex(s.blokken.length - 1);
  };
  const verplaats = (naar: number) => {
    if (blokIndex === null) return;
    zet(verplaatsBlok(concept, blokIndex, naar));
    setBlokIndex(Math.max(0, Math.min(concept.blokken.length - 1, naar)));
  };

  const vasteRijen = rijen.filter((r) => r.soort !== "eigen");
  const eigenRijen = [...rijen.filter((r) => r.soort === "eigen").map((r) => r.sjabloon), ...nieuwe];

  return (
    <div className="werkvlak-inhoud sjablonen-werk">
      <div className="sjablonen-lijst">
        <table className="tabel">
          <tbody>
            <tr className="groep-rij">
              <th>Vaste Sjablonen</th>
            </tr>
            {vasteRijen.map((r) => (
              <tr key={r.sjabloon.id} tabIndex={0} className={r.sjabloon.id === gekozenId ? "gekozen" : undefined} onClick={() => kies(r.sjabloon.id)}>
                <td>
                  <div className="titel-cel">
                    {r.sjabloon.naam}
                    {concepten[r.sjabloon.id] && <span className="tekst-let-op"> *</span>}
                  </div>
                  <div className="zacht">
                    {r.sjabloon.groep}
                    {r.soort === "aangepast" && <span className="stempel">Aangepast</span>}
                    {r.nieuweStandaard && <span className="stempel stempel-let-op">Nieuwe standaard</span>}
                  </div>
                </td>
              </tr>
            ))}
            <tr className="groep-rij">
              <th>Eigen Sjablonen</th>
            </tr>
            {eigenRijen.map((s) => (
              <tr key={s.id} tabIndex={0} className={s.id === gekozenId ? "gekozen" : undefined} onClick={() => kies(s.id)}>
                <td>
                  <div className="titel-cel">
                    {(concepten[s.id] ?? s).naam}
                    {concepten[s.id] && <span className="tekst-let-op"> *</span>}
                  </div>
                  <div className="zacht">{nieuwe.includes(s) ? "Nog niet opgeslagen" : `${s.blokken.length} Blokken`}</div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="sjablonen-nieuw">
          <button className="knop" onClick={nieuwEigen}>
            Nieuw Sjabloon
          </button>
        </div>
      </div>

      <div className="sjablonen-midden">
        {rij?.nieuweStandaard && (
          <div className="sjablonen-melding" role="status">
            <span>
              <b>Er is een nieuwe standaard</b> voor {rij.sjabloon.naam}. Jouw aanpassing blijft staan tot je kiest.
            </span>
            <button className="knop" disabled={!verbonden} onClick={() => void wis()}>
              Nieuwe standaard overnemen
            </button>
            <button className="knop" disabled={!verbonden} onClick={() => void regie.stuur({ soort: "HoudSjabloon", id: concept.id }, VORMGEVING)}>
              Mijn versie houden
            </button>
          </div>
        )}
        <Schets sjabloon={concept} stijl={stand.stijl} gekozen={blokIndex} kies={setBlokIndex} zetBlok={zetBlok} />
        <div className="sjablonen-blokbalk">
          <span className="label">Blokken</span>
          <select aria-label="Blok toevoegen" value="" onChange={(e) => e.target.value && voegToe(e.target.value as Bloksoort["blok"])}>
            <option value="">Blok toevoegen…</option>
            {GEWONE_BLOKKEN.map((b) => (
              <option key={b.soort} value={b.soort}>
                {b.naam}
              </option>
            ))}
          </select>
          <button className="knop" disabled={blokIndex === null || blokIndex === concept.blokken.length - 1} onClick={() => verplaats((blokIndex ?? 0) + 1)} title="Een plaats naar voren">
            Naar voren
          </button>
          <button className="knop" disabled={!blokIndex} onClick={() => verplaats((blokIndex ?? 0) - 1)} title="Een plaats naar achteren">
            Naar achteren
          </button>
          <button
            className="knop"
            disabled={blokIndex === null}
            onClick={() => {
              if (blokIndex === null) return;
              const s = structuredClone(concept);
              s.blokken.push(structuredClone(s.blokken[blokIndex]));
              zet(s);
              setBlokIndex(s.blokken.length - 1);
            }}
          >
            Kopie
          </button>
          <button
            className="knop"
            disabled={blokIndex === null}
            onClick={() => {
              if (blokIndex === null) return;
              zet(wisBlok(concept, blokIndex));
              setBlokIndex(null);
            }}
          >
            Wis Blok
          </button>
        </div>
        <table className="tabel sjablonen-blokken">
          <thead>
            <tr>
              <th>#</th>
              <th>Blok</th>
              <th>Inhoud</th>
              <th>In beeld</th>
              <th>Start na</th>
            </tr>
          </thead>
          <tbody>
            {concept.blokken.map((b, i) => (
              <tr key={i} tabIndex={0} className={i === blokIndex ? "gekozen" : undefined} onClick={() => setBlokIndex(i === blokIndex ? null : i)}>
                <td className="zacht">{i + 1}</td>
                <td className="titel-cel">{BLOKNAMEN[b.soort.blok]}</td>
                <td className="zacht sjablonen-inhoud">{blokTekst(b)}</td>
                <td className="zacht">{ERIN.find((e) => e.erin === b.animatie.erin)?.naam}</td>
                <td className="zacht">{(b.animatie.start_na_ms / 1000).toLocaleString("nl-NL")} s</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="sjablonen-paneel">
        <div className="segment" role="group" aria-label="Instellingen">
          <button aria-pressed={blokIndex === null} onClick={() => setBlokIndex(null)}>
            Heel het Sjabloon
          </button>
          <button aria-pressed={blokIndex !== null} disabled={blokIndex === null}>
            {blokIndex === null ? "Blok" : `Blok ${blokIndex + 1}`}
          </button>
        </div>
        {blok && blokIndex !== null ? (
          <BlokVelden key={blokIndex} blok={blok} stijl={stand.stijl} zet={(w) => zetBlok(blokIndex, w)} />
        ) : (
          <SjabloonVelden sjabloon={concept} vast={!!vast} zet={zet} />
        )}
      </div>

      <div className="actiebalk sjablonen-actiebalk">
        {vast ? (
          <button
            className="knop"
            disabled={rij?.soort !== "aangepast" || !verbonden}
            onClick={() => void wis()}
            title="Gooit de aanpassing weg: het Sjabloon wordt weer zoals het met de software meekomt."
          >
            Terug naar standaard
          </button>
        ) : (
          <button className="knop" disabled={!verbonden && !!opgeslagen} onClick={() => void wis()}>
            Wis Sjabloon
          </button>
        )}
        <button className="knop" disabled={!concepten[concept.id] || !opgeslagen} onClick={() => vergeet(concept.id)}>
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

/** De instellingen van het hele Sjabloon: naam, achtergrond, Stoelen en hun Kleurovergang, Standpunt en snelheid. */
function SjabloonVelden({ sjabloon: s, vast, zet }: { sjabloon: Sjabloon; vast: boolean; zet(s: Sjabloon): void }) {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const keuzes = cameraKeuzes(stand?.standpunten ?? [], stand?.cameraritten ?? []);
  const stijlOvergang = (stand?.stijl ?? STANDAARD_STIJL).kleurovergang;
  const wijzig = (w: (s: Sjabloon) => void) => {
    const n = structuredClone(s);
    w(n);
    zet(n);
  };
  const a = s.achtergrond;
  return (
    <div className="sjablonen-velden">
      <label className="veld">
        <span className="label">Naam</span>
        <input value={s.naam} readOnly={vast} onChange={(e) => wijzig((n) => (n.naam = e.target.value))} />
      </label>
      <label className="veld">
        <span className="label">Achtergrond</span>
        <select
          value={a.soort}
          onChange={(e) =>
            wijzig((n) => {
              const soort = e.target.value as (typeof ACHTERGRONDEN)[number]["soort"];
              n.achtergrond = soort === "Kleur" ? { soort, kleur: { Stijl: "Achtergrond" } } : soort === "Afbeelding" ? { soort, bestand: "" } : { soort };
            })
          }
        >
          {ACHTERGRONDEN.map((x) => (
            <option key={x.soort} value={x.soort}>
              {x.naam}
            </option>
          ))}
        </select>
      </label>
      {a.soort === "Kleur" && <Kleurkiezer kleur={a.kleur} zet={(k) => wijzig((n) => (n.achtergrond = { soort: "Kleur", kleur: k }))} />}
      {a.soort === "Afbeelding" && (
        <div className="knoppen">
          <Kiezer
            tekst="Afbeelding plaatsen…"
            accept="image/png,image/jpeg"
            uit={regie.staat !== "verbonden"}
            kies={(f) => {
              if (!f) return;
              void regie.plaatsBestand(f, VORMGEVING).then((naam) => naam && wijzig((n) => (n.achtergrond = { soort: "Afbeelding", bestand: naam })));
            }}
          />
          <span className="zacht">{a.bestand || "Nog geen afbeelding"}</span>
        </div>
      )}
      {a.soort === "ZaalDonker" && <p className="zacht">De Zaal blijft in beeld, ongeveer half zo licht en onscherp.</p>}
      <label className="veld">
        <span className="label">Stoelen</span>
        <select value={s.stoelen} onChange={(e) => wijzig((n) => (n.stoelen = e.target.value as Sjabloon["stoelen"]))}>
          {STOELEN.map((x) => (
            <option key={x.stoelen} value={x.stoelen}>
              {x.naam}
            </option>
          ))}
        </select>
      </label>
      <div className="veld">
        <label className="veld">
          <span className="label">Kleurovergang</span>
          <select
            value={s.kleurovergang ? "eigen" : "stijl"}
            onChange={(e) => wijzig((n) => (n.kleurovergang = e.target.value === "eigen" ? structuredClone(stijlOvergang) : undefined))}
          >
            <option value="stijl">Zoals de Stijl ({stijlOvergang.vorm === "Golf" ? "Golf" : "Vervagen"})</option>
            <option value="eigen">Eigen Kleurovergang</option>
          </select>
        </label>
        {s.kleurovergang && <KleurovergangKeuze waarde={s.kleurovergang} zet={(k) => wijzig((n) => (n.kleurovergang = k))} />}
      </div>
      <label className="veld">
        <span className="veld-kop">
          <span className="label">Standpunt</span>
          <span className="zacht">De Regie kan per Graphic een ander kiezen</span>
        </span>
        <select value={s.standpunt ?? ""} onChange={(e) => wijzig((n) => (n.standpunt = e.target.value || undefined))}>
          <option value="">Zoals de soort Graphic</option>
          {(["Standpunt", "Camerarit"] as const).map((soort) => (
            <optgroup key={soort} label={soort === "Standpunt" ? "Standpunten" : "Cameraritten"}>
              {keuzes
                .filter((k) => k.soort === soort)
                .map((k) => (
                  <option key={k.letter} value={k.letter}>
                    {k.tekst}
                  </option>
                ))}
            </optgroup>
          ))}
        </select>
      </label>
      <div className="veld">
        <span className="veld-kop">
          <span className="label">Snelheid</span>
          <span className="zacht">bovenop die van de Stijl</span>
        </span>
        <div className="stijl-snelheid">
          <input
            type="range"
            aria-label="Snelheid van dit Sjabloon"
            min={MIN_SNELHEID}
            max={MAX_SNELHEID}
            step={5}
            value={s.snelheid}
            onChange={(e) => wijzig((n) => (n.snelheid = Number(e.target.value)))}
          />
          <span className="stijl-procent">{s.snelheid}%</span>
        </div>
      </div>
    </div>
  );
}

/** Een Stijlkleur of een eigen kleur. */
function Kleurkiezer({ kleur, zet }: { kleur: Kleurkeuze; zet(k: Kleurkeuze): void }) {
  const eigen = "Eigen" in kleur ? kleur.Eigen : null;
  return (
    <div className="sjablonen-rij">
      <select aria-label="Kleur" value={"Stijl" in kleur ? kleur.Stijl : "eigen"} onChange={(e) => zet(e.target.value === "eigen" ? { Eigen: "#ffffff" } : { Stijl: e.target.value as Stijlkleur })}>
        {STIJLKLEUREN.map((k) => (
          <option key={k.kleur} value={k.kleur}>
            {k.naam} (Stijl)
          </option>
        ))}
        <option value="eigen">Eigen kleur</option>
      </select>
      {eigen !== null && <input type="color" aria-label="Eigen kleur" value={eigen} onChange={(e) => zet({ Eigen: e.target.value })} />}
    </div>
  );
}

/** Een Stijlletter of een eigen lettertype. */
function Letterkiezer({ letter, stijl, zet }: { letter: Letterkeuze; stijl: Stijl; zet(l: Letterkeuze): void }) {
  const waarde = "Stijl" in letter ? `stijl:${letter.Stijl}` : letter.Eigen;
  const eigen = letterKeuzes(stijl);
  if ("Eigen" in letter && !eigen.includes(letter.Eigen)) eigen.push(letter.Eigen);
  return (
    <select aria-label="Lettertype" value={waarde} onChange={(e) => zet(e.target.value.startsWith("stijl:") ? { Stijl: e.target.value.slice(6) as Stijlletter } : { Eigen: e.target.value })}>
      <optgroup label="Uit de Stijl">
        {STIJLLETTERS.map((l) => (
          <option key={l} value={`stijl:${l}`}>
            {l} ({letterNaam(stijl.lettertypes[l === "Kop" ? "kop" : l === "Tekst" ? "tekst" : "cijfers"])})
          </option>
        ))}
      </optgroup>
      <optgroup label="Eigen lettertype">
        {eigen.map((b) => (
          <option key={b} value={b}>
            {letterNaam(b)}
          </option>
        ))}
      </optgroup>
    </select>
  );
}

/** Een getal in een invoerveld. */
function Getal({ naam, waarde, zet, min, max, stap = 1 }: { naam: string; waarde: number; zet(n: number): void; min?: number; max?: number; stap?: number }) {
  return (
    <label className="veld sjablonen-getal">
      <span className="label">{naam}</span>
      <input
        type="number"
        value={waarde}
        min={min}
        max={max}
        step={stap}
        onChange={(e) => {
          const n = Number(e.target.value);
          if (Number.isFinite(n)) zet(n);
        }}
      />
    </label>
  );
}

/** De instellingen van één Blok: plek, grootte, kleur, lettertype, inhoud en animatie. */
function BlokVelden({ blok: b, stijl, zet }: { blok: Blok; stijl: Stijl; zet(w: (b: Blok) => void): void }) {
  const s = b.soort;
  const letter = "letter" in s ? s.letter : null;
  const grootte = "grootte" in s ? s.grootte : null;
  return (
    <div className="sjablonen-velden">
      <div className="veld">
        <span className="label">{BLOKNAMEN[s.blok]}: plek en grootte in pixels van het beeld (1920 bij 1080)</span>
        <div className="sjablonen-vier">
          <Getal naam="X" waarde={b.plek.x} zet={(n) => zet((x) => (x.plek.x = Math.round(n)))} />
          <Getal naam="Y" waarde={b.plek.y} zet={(n) => zet((x) => (x.plek.y = Math.round(n)))} />
          <Getal naam="Breedte" waarde={b.plek.breedte} min={0} zet={(n) => zet((x) => (x.plek.breedte = Math.max(0, Math.round(n))))} />
          <Getal naam="Hoogte" waarde={b.plek.hoogte} min={0} zet={(n) => zet((x) => (x.plek.hoogte = Math.max(0, Math.round(n))))} />
        </div>
      </div>
      <div className="veld">
        <span className="label">Kleur</span>
        <Kleurkiezer kleur={b.kleur} zet={(k) => zet((x) => (x.kleur = k))} />
      </div>
      {letter && (
        <div className="veld">
          <span className="label">Lettertype</span>
          <div className="sjablonen-rij">
            <Letterkiezer letter={letter} stijl={stijl} zet={(l) => zet((x) => "letter" in x.soort && (x.soort.letter = l))} />
            {grootte !== null && <Getal naam="Grootte" waarde={grootte} min={6} max={400} zet={(n) => zet((x) => "grootte" in x.soort && (x.soort.grootte = Math.round(n)))} />}
          </div>
        </div>
      )}
      {s.blok === "Tekst" && <Tekstbronveld bron={s.bron} zet={(bron) => zet((x) => x.soort.blok === "Tekst" && (x.soort.bron = bron))} />}
      {s.blok === "Getal" && (
        <label className="veld">
          <span className="label">Getal</span>
          <div className="sjablonen-rij">
            <select
              value={typeof s.bron === "object" ? "Veld" : s.bron}
              onChange={(e) => zet((x) => x.soort.blok === "Getal" && (x.soort.bron = e.target.value === "Veld" ? { Veld: "getal" } : (e.target.value as "Zetels" | "Winst" | "Coalitie")))}
            >
              <option value="Veld">Invulveld</option>
              <option value="Zetels">Zetels van de gekozen Partij</option>
              <option value="Winst">Winst of verlies van de gekozen Partij</option>
              <option value="Coalitie">Teller van de coalitie</option>
            </select>
            {typeof s.bron === "object" && (
              <input aria-label="Naam van het invulveld" value={s.bron.Veld} onChange={(e) => zet((x) => x.soort.blok === "Getal" && (x.soort.bron = { Veld: e.target.value }))} />
            )}
          </div>
        </label>
      )}
      {(s.blok === "Tekst" || s.blok === "Getal") && (
        <label className="veld">
          <span className="label">Uitlijning</span>
          <select value={s.uitlijning} onChange={(e) => zet((x) => "uitlijning" in x.soort && (x.soort.uitlijning = e.target.value as Uitlijning))}>
            <option value="Links">Links</option>
            <option value="Midden">Midden</option>
            <option value="Rechts">Rechts</option>
          </select>
        </label>
      )}
      {s.blok === "Tekst" && (
        <label className="sjablonen-vink">
          <input type="checkbox" checked={s.een_regel} onChange={(e) => zet((x) => x.soort.blok === "Tekst" && (x.soort.een_regel = e.target.checked))} />
          Op één regel, kleiner als het niet past
        </label>
      )}
      {s.blok === "Vlak" && (
        <div className="sjablonen-vier">
          <Getal naam="Dekking %" waarde={s.dekking} min={0} max={100} zet={(n) => zet((x) => x.soort.blok === "Vlak" && (x.soort.dekking = Math.min(100, Math.max(0, Math.round(n)))))} />
          <Getal naam="Ronding" waarde={s.hoek} min={0} zet={(n) => zet((x) => x.soort.blok === "Vlak" && (x.soort.hoek = Math.max(0, Math.round(n))))} />
          <Getal naam="Draaiing °" waarde={s.draaiing} min={-180} max={180} zet={(n) => zet((x) => x.soort.blok === "Vlak" && (x.soort.draaiing = Math.round(n)))} />
        </div>
      )}
      {(s.blok === "Lijn" || s.blok === "Programmalogo") && (
        <Getal naam="Dekking %" waarde={s.dekking} min={0} max={100} zet={(n) => zet((x) => (x.soort.blok === "Lijn" || x.soort.blok === "Programmalogo") && (x.soort.dekking = Math.min(100, Math.max(0, Math.round(n)))))} />
      )}
      {s.blok === "Zetelbalk" && (
        <>
          <label className="sjablonen-vink">
            <input type="checkbox" checked={s.lijn_bij_76} onChange={(e) => zet((x) => x.soort.blok === "Zetelbalk" && (x.soort.lijn_bij_76 = e.target.checked))} />
            Lijn bij 76
          </label>
          <label className="sjablonen-vink">
            <input type="checkbox" checked={s.labels} onChange={(e) => zet((x) => x.soort.blok === "Zetelbalk" && (x.soort.labels = e.target.checked))} />
            Zetels en korte naam onder elk deel
          </label>
        </>
      )}
      <div className="veld">
        <span className="label">Animatie</span>
        <div className="sjablonen-rij">
          <label className="veld">
            <span className="zacht">In beeld</span>
            <select value={b.animatie.erin} onChange={(e) => zet((x) => (x.animatie.erin = e.target.value as Blok["animatie"]["erin"]))}>
              {ERIN.map((x) => (
                <option key={x.erin} value={x.erin}>
                  {x.naam}
                </option>
              ))}
            </select>
          </label>
          <label className="veld">
            <span className="zacht">Uit beeld</span>
            <select value={b.animatie.eruit} onChange={(e) => zet((x) => (x.animatie.eruit = e.target.value as Blok["animatie"]["eruit"]))}>
              {ERUIT.map((x) => (
                <option key={x.eruit} value={x.eruit}>
                  {x.naam}
                </option>
              ))}
            </select>
          </label>
          <Getal naam="Start na (s)" waarde={b.animatie.start_na_ms / 1000} min={0} max={30} stap={0.1} zet={(n) => zet((x) => (x.animatie.start_na_ms = Math.max(0, Math.round(n * 1000))))} />
        </div>
      </div>
    </div>
  );
}

/** Waar de tekst van een Tekst-Blok vandaan komt: vaste tekst, een invulveld of de gegevens. */
function Tekstbronveld({ bron, zet }: { bron: Tekstbron; zet(b: Tekstbron): void }) {
  const soort = typeof bron === "object" ? ("Vast" in bron ? "Vast" : "Veld") : bron;
  return (
    <label className="veld">
      <span className="label">Tekst</span>
      <select
        value={soort}
        onChange={(e) => {
          const v = e.target.value;
          zet(v === "Vast" ? { Vast: "Tekst" } : v === "Veld" ? { Veld: "kop" } : (v as Exclude<Tekstbron, object>));
        }}
      >
        <option value="Vast">Vaste tekst</option>
        <option value="Veld">Invulveld (de Regie typt het bij Maken)</option>
        {TEKSTBRONNEN.map((t) => (
          <option key={t.bron} value={t.bron}>
            {t.naam}
          </option>
        ))}
      </select>
      {typeof bron === "object" && "Vast" in bron && <input aria-label="Vaste tekst" value={bron.Vast} onChange={(e) => zet({ Vast: e.target.value })} />}
      {typeof bron === "object" && "Veld" in bron && <input aria-label="Naam van het invulveld" value={bron.Veld} onChange={(e) => zet({ Veld: e.target.value })} />}
    </label>
  );
}

/** De kleur van een keuze in de Stijl, als CSS. */
function css(k: Kleurkeuze, stijl: Stijl, dekking = 100): string {
  const hex = "Eigen" in k ? k.Eigen : stijl.kleuren[k.Stijl.toLowerCase() as keyof Stijl["kleuren"]];
  const a = Math.round((Math.max(0, Math.min(100, dekking)) / 100) * 255);
  return `${hex}${a.toString(16).padStart(2, "0")}`;
}

/** De lettersoort van een keuze, als CSS: een schatting, het dashboard heeft de lettertypes van de Renderer niet. */
function lettercss(l: Letterkeuze | null): string {
  const s = l && "Stijl" in l ? l.Stijl : null;
  return s === "Kop" ? 'Georgia, "Times New Roman", serif' : "Inter, system-ui, sans-serif";
}

/** Wat een Blok in de schets toont. */
function schetstekst(b: Blok): string {
  const s = b.soort;
  if (s.blok === "Tekst") return typeof s.bron === "object" ? ("Vast" in s.bron ? s.bron.Vast : `‹${s.bron.Veld}›`) : `‹${tekstbronNaam(s.bron)}›`;
  if (s.blok === "Getal") return "123";
  if (s.blok === "Vlak" || s.blok === "Lijn" || s.blok === "Afbeelding") return "";
  return BLOKNAMEN[s.blok];
}

/** Een schets van het beeld (1920 bij 1080) met de Blokken. Klik kiest een Blok, slepen verplaatst
 * het, het hoekje rechtsonder van het gekozen Blok maakt het groter of kleiner. Alleen een schets:
 * het Voorbeeld toont de echte Graphic zodra die klaarstaat. */
function Schets({
  sjabloon: s,
  stijl,
  gekozen,
  kies,
  zetBlok,
}: {
  sjabloon: Sjabloon;
  stijl: Stijl;
  gekozen: number | null;
  kies(i: number | null): void;
  zetBlok(i: number, w: (b: Blok) => void): void;
}) {
  const vlak = useRef<HTMLDivElement>(null);
  const sleep = useRef<{ i: number; x: number; y: number; plek: Blok["plek"]; maat: boolean } | null>(null);
  const afbeelding = useBestand(s.achtergrond.soort === "Afbeelding" ? s.achtergrond.bestand || null : null);
  const begin = (e: ReactPointerEvent, i: number, maat: boolean) => {
    e.stopPropagation();
    kies(i);
    sleep.current = { i, x: e.clientX, y: e.clientY, plek: { ...s.blokken[i].plek }, maat };
    (e.target as Element).setPointerCapture(e.pointerId);
  };
  const beweeg = (e: ReactPointerEvent) => {
    const z = sleep.current;
    const r = vlak.current?.getBoundingClientRect();
    if (!z || !r) return;
    const schaal = 1920 / r.width;
    const dx = Math.round((e.clientX - z.x) * schaal);
    const dy = Math.round((e.clientY - z.y) * schaal);
    if (dx === 0 && dy === 0) return;
    zetBlok(z.i, (b) => {
      if (z.maat) {
        b.plek.breedte = Math.max(2, z.plek.breedte + dx);
        b.plek.hoogte = Math.max(2, z.plek.hoogte + dy);
      } else {
        b.plek.x = z.plek.x + dx;
        b.plek.y = z.plek.y + dy;
      }
    });
  };
  const a = s.achtergrond;
  const achter =
    a.soort === "Kleur" ? { background: css(a.kleur, stijl) } : a.soort === "Afbeelding" && afbeelding.url ? { backgroundImage: `url(${afbeelding.url})`, backgroundSize: "cover" } : undefined;
  return (
    <div
      ref={vlak}
      className={`sjabloon-schets${a.soort === "ZaalDonker" ? " schets-donker" : ""}`}
      style={achter}
      onPointerDown={() => kies(null)}
      onPointerMove={beweeg}
      onPointerUp={() => (sleep.current = null)}
      aria-label={`Schets van ${s.naam}`}
    >
      {a.soort !== "Kleur" && a.soort !== "Afbeelding" && <span className="schets-zaal">Zaal</span>}
      {s.blokken.map((b, i) => {
        const p = b.plek;
        const soort = b.soort;
        const pos = { left: `${(p.x / 1920) * 100}%`, top: `${(p.y / 1080) * 100}%`, width: `${(p.breedte / 1920) * 100}%`, height: `${(p.hoogte / 1080) * 100}%` };
        let stijlBlok: CSSProperties = {};
        if (soort.blok === "Vlak") {
          const van = css(b.kleur, stijl, soort.dekking);
          stijlBlok = {
            background: soort.verloop
              ? `linear-gradient(${soort.verloop.richting}deg, ${van} ${soort.verloop.van}%, ${css(soort.verloop.naar, stijl, soort.verloop.naar_dekking)} ${soort.verloop.tot}%)`
              : van,
            borderRadius: `${(soort.hoek / 1920) * 100}cqw`,
            transform: soort.draaiing ? `rotate(${soort.draaiing}deg)` : undefined,
            boxShadow: soort.rand ? `inset 0 0 0 ${Math.max(1, soort.rand.dikte / 3)}px ${css(soort.rand.kleur, stijl, soort.rand.dekking)}` : undefined,
          };
        } else if (soort.blok === "Lijn") {
          stijlBlok = { background: css(b.kleur, stijl, soort.dekking) };
        } else if (soort.blok === "Tekst" || soort.blok === "Getal") {
          stijlBlok = {
            color: css(b.kleur, stijl),
            fontSize: `${(soort.grootte / 1920) * 100}cqw`,
            fontFamily: lettercss(soort.letter),
            fontWeight: soort.blok === "Getal" || ("Stijl" in soort.letter && soort.letter.Stijl === "Cijfers") ? 800 : 600,
            textAlign: soort.uitlijning === "Links" ? "left" : soort.uitlijning === "Midden" ? "center" : "right",
            whiteSpace: soort.blok === "Getal" || soort.een_regel ? "nowrap" : undefined,
          };
        } else {
          stijlBlok = { color: css(b.kleur, stijl), fontSize: `${(Math.min(p.hoogte * 0.4, 28) / 1920) * 100}cqw` };
        }
        const groot = !["Vlak", "Lijn", "Tekst", "Getal"].includes(soort.blok);
        return (
          <div
            key={i}
            className={`schets-blok${groot ? " schets-groot" : ""}${i === gekozen ? " schets-gekozen" : ""}`}
            style={{ ...pos, ...stijlBlok }}
            onPointerDown={(e) => begin(e, i, false)}
            title={`${i + 1}. ${BLOKNAMEN[soort.blok]}`}
          >
            {schetstekst(b)}
            {i === gekozen && <span className="schets-maat" onPointerDown={(e) => begin(e, i, true)} />}
          </div>
        );
      })}
    </div>
  );
}
