// Camera (DESIGN.md, Camera Werkbank): links de Standpunten en Cameraritten, in het midden Vrij,
// Oefenen, Opnemen en Bewaar boven een grote plattegrond, rechts het instelpaneel van wat gekozen is.
// Een klik op een rij of pin kiest het om te bewerken; de Ether krijgt het pas met de letter of met
// Kies nu. Elke wijziging in het paneel speelt meteen af in het Voorbeeld (de server doet dat).
import { useEffect, useRef, useState } from "react";
import type { KeyboardEvent, ReactNode } from "react";
import { standaardStandpunt } from "../graphic";
import { OefenToetsen, OpnameTijd, nieuweNaam } from "../multiviewer/Camera";
import { Plattegrond } from "../multiviewer/Plattegrond";
import {
  DUUR_FACTOR,
  RIT_MIN_S,
  kanHerhalen,
  komma,
  opnameS,
  ritAangepast,
  ritTiming,
  standpuntTiming,
  stukS,
} from "../regie/camerarit";
import { useCamera, useRegie } from "../regie/regie";
import { kiesStandpunt, voerUit } from "../regie/take";
import { toetsVan } from "../regie/toetsen";
import type { Bijschaving } from "../typen/Bijschaving";
import type { CameraOvergang } from "../typen/CameraOvergang";
import type { Camerarit } from "../typen/Camerarit";
import type { NaAankomst } from "../typen/NaAankomst";
import type { Opdracht } from "../typen/Opdracht";
import type { Stand } from "../typen/Stand";
import type { Standpunt } from "../typen/Standpunt";
import { Werkbalk } from "./Werkbalk";

/** Waar een weigering bij het gekozen Standpunt komt te staan. */
const CAMERAWERK = "camerawerk";
/** Gelijk aan EIGEN_TOETSEN, MAX_STANDPUNTNAAM, DUUR_MIN en DUUR_MAX in crates/gedeeld. */
const EIGEN_TOETSEN = ["J", "K", "L"];
const MAX_NAAM = 40;
const DUUR_MIN = 0.5;
const DUUR_MAX = 6;

const GAMEPAD = [
  ["Linker stick", "Vooruit, achteruit en opzij"],
  ["Rechter stick", "Rondkijken"],
  ["Rechter trigger", "Omhoog"],
  ["Linker trigger", "Omlaag"],
  ["Bumpers", "Inzoomen (rechts) en uitzoomen (links)"],
  ["A", "Langzaam of snel"],
  ["B", "Opname starten of stoppen, net als R (tijdens Oefenen)"],
  ["Start", "Vrije camera aan of uit, net als V"],
];

/** Waar een Standpunt of Camerarit gebruikt wordt (gelijk aan `Stand::gebruik_van_standpunt`). */
function gebruikVan(s: Stand, letter: string): string[] {
  const waar: string[] = [];
  if (s.klaar && (s.klaar.standpunt ?? standaardStandpunt(s.klaar.graphic)) === letter) waar.push("de klaargezette Graphic");
  if (s.ether && (s.ether.standpunt ?? standaardStandpunt(s.ether.graphic)) === letter) waar.push("de Graphic in de Ether");
  for (const p of s.presets) if ((p.standpunt ?? standaardStandpunt(p.graphic)) === letter) waar.push(`Preset ${p.nummer}`);
  if (!s.camera.vrij && s.camera.standpunt === letter) waar.push("de camera van de Ether");
  return waar;
}

type Item = { soort: "Standpunt"; p: Standpunt } | { soort: "Camerarit"; r: Camerarit };

export function Camerawerk() {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const standpunten = s?.standpunten ?? [];
  const ritten = s?.cameraritten ?? [];
  const vrij = s?.camera.vrij ?? false;
  const oefenen = s?.camera.oefenen ?? false;
  const neemtOp = !!s?.camera.opname;
  const inEther = vrij ? null : (s?.camera.standpunt ?? null);
  const verbonden = regie.staat === "verbonden";
  const [gekozen, setGekozen] = useState<string | null>(null);
  const bekend = useRef<Set<string> | null>(null);

  // Een net bewaard Standpunt of een net opgenomen Camerarit wordt meteen gekozen.
  useEffect(() => {
    const alle = [...standpunten.map((p) => p.letter), ...ritten.map((r) => r.letter)];
    const nieuw = bekend.current ? alle.find((l) => !bekend.current!.has(l)) : undefined;
    if (nieuw) setGekozen(nieuw);
    if (regie.stand) bekend.current = new Set(alle);
  }, [standpunten, ritten, regie.stand]);

  const groepen: { naam: string; leeg?: string; rijen: { letter: string; toets: string | null; naam: string; tijd: string }[] }[] = [
    {
      naam: "Vaste Standpunten",
      rijen: standpunten.filter((p) => p.vast).map((p) => ({ letter: p.letter, toets: toetsVan(p), naam: p.naam, tijd: standpuntTiming(p) })),
    },
    {
      naam: "Eigen Standpunten",
      leeg: "Zet Vrij of Oefenen aan en kies Bewaar als Standpunt.",
      rijen: standpunten.filter((p) => !p.vast).map((p) => ({ letter: p.letter, toets: toetsVan(p), naam: p.naam, tijd: standpuntTiming(p) })),
    },
    {
      naam: "Vaste Cameraritten",
      rijen: ritten.filter((r) => r.vast).map((r) => ({ letter: r.letter, toets: toetsVan(r), naam: r.naam, tijd: ritTiming(r) })),
    },
    {
      naam: "Eigen Cameraritten",
      leeg: "Zet Oefenen aan en neem een rit op met R.",
      rijen: ritten.filter((r) => !r.vast).map((r) => ({ letter: r.letter, toets: toetsVan(r), naam: r.naam, tijd: ritTiming(r) })),
    },
  ];
  const p = standpunten.find((x) => x.letter === gekozen);
  const r = ritten.find((x) => x.letter === gekozen);
  const item: Item | null = p ? { soort: "Standpunt", p } : r ? { soort: "Camerarit", r } : null;
  const druk = (e: KeyboardEvent, letter: string) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      e.stopPropagation();
      setGekozen(letter);
    }
  };

  return (
    <>
      <Werkbalk
        naam="Camera"
        hint={
          <>
            <kbd>V</kbd> Vrij · <kbd>O</kbd> Oefenen · <kbd>R</kbd> Opnemen · <kbd>A</kbd> tot <kbd>L</kbd> kiest voor de Ether
          </>
        }
      />
      <div className="werkvlak-inhoud camerawerk">
        <div className="lijst">
          <table className="tabel">
            <tbody>
              {groepen.map((g) => [
                <tr key={g.naam} className="groep-rij">
                  <th colSpan={3}>{g.naam}</th>
                </tr>,
                ...g.rijen.map((x) => (
                  <tr
                    key={x.letter}
                    className={x.letter === gekozen ? "gekozen" : undefined}
                    aria-selected={x.letter === gekozen}
                    tabIndex={0}
                    onClick={() => setGekozen(x.letter)}
                    onKeyDown={(e) => druk(e, x.letter)}
                  >
                    <td className="toets-cel">{x.toets && <kbd>{x.toets}</kbd>}</td>
                    <td>
                      <div className="titel-cel">{x.naam}</div>
                      <div className="zacht">{x.tijd}</div>
                    </td>
                    <td className="ether-cel">{x.letter === inEther && <span className="punt punt-ether" title="In de Ether" />}</td>
                  </tr>
                )),
                g.rijen.length === 0 && g.leeg && (
                  <tr key={`${g.naam}-leeg`}>
                    <td colSpan={3} className="zacht">
                      {g.leeg}
                    </td>
                  </tr>
                ),
              ])}
            </tbody>
          </table>
        </div>
        <div className="camerawerk-midden">
          <div className="knoppen">
            <button className="knop" aria-pressed={vrij} disabled={!verbonden || neemtOp} onClick={() => voerUit(regie, "VrijeCamera")}>
              <kbd>V</kbd> Vrij
            </button>
            <OefenToetsen klasse="knop" />
            <button
              className="knop"
              disabled={!verbonden || !(vrij || oefenen)}
              title={
                oefenen
                  ? "Bewaar waar de camera van het Voorbeeld nu staat"
                  : vrij
                    ? "Bewaar waar de camera nu staat"
                    : "Zet eerst Vrij of Oefenen aan"
              }
              onClick={() => void regie.stuur({ soort: "BewaarStandpunt", naam: nieuweNaam(standpunten) }, CAMERAWERK)}
            >
              Bewaar als Standpunt
            </button>
          </div>
          <Plattegrond
            groot
            standpunten={standpunten}
            cameraritten={ritten}
            huidig={inEther}
            vrij={vrij || oefenen}
            voorbeeld={oefenen || !!s?.camera.proef}
            gekozen={gekozen}
            stoelkleuren={s?.stoelkleuren ?? []}
            kies={setGekozen}
          />
        </div>
        <div className="camerawerk-paneel">
          {neemtOp ? (
            <Opname />
          ) : item?.soort === "Standpunt" ? (
            <StandpuntDetail key={item.p.letter} p={item.p} />
          ) : item?.soort === "Camerarit" ? (
            <RitDetail key={item.r.letter} r={item.r} />
          ) : (
            <Uitleg />
          )}
          <section className="blok">
            <h3 className="label">Gamepad</h3>
            <table className="tabel">
              <tbody>
                {GAMEPAD.map(([knop, wat]) => (
                  <tr key={knop}>
                    <td className="titel-cel">{knop}</td>
                    <td className="zacht">{wat}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            <p className="zacht">
              De gamepad zit aan de server. Met Vrij stuurt hij de Ether, met Oefenen alleen het Voorbeeld. Na loslaten glijdt de
              camera in een halve seconde tot stilstand.
            </p>
          </section>
        </div>
      </div>
    </>
  );
}

/** Het paneel zonder keuze: zo neem je een Camerarit op. */
function Uitleg() {
  return (
    <section className="paneel-detail">
      <h3 className="kop-regel">Een Camerarit opnemen</h3>
      <ol className="stappen">
        <li>
          Zet <kbd>O</kbd> Oefenen aan. De gamepad stuurt nu alleen het Voorbeeld; de Ether blijft staan.
        </li>
        <li>Rijd met de gamepad naar het beginpunt.</li>
        <li>
          Druk <kbd>R</kbd> (of B op de gamepad), rijd de route, en druk nog een keer om te stoppen.
        </li>
        <li>De rit staat meteen op de eerste vrije letter van J, K en L en speelt af in het Voorbeeld. Schaaf hem bij met de schuifjes.</li>
      </ol>
      <p className="zacht">Kies links of op de plattegrond een Standpunt of Camerarit om het in te stellen.</p>
    </section>
  );
}

/** Het paneel tijdens een opname. */
function Opname() {
  const regie = useRegie();
  const voor = regie.stand?.stand.camera.opname?.voor;
  const naam = regie.stand?.stand.cameraritten.find((r) => r.letter === voor)?.naam;
  return (
    <section className="paneel-detail">
      <h3 className="kop-regel opname-kop">
        <span className="punt punt-ether" aria-hidden />
        {naam ? `${naam} opnieuw opnemen` : "Opname loopt"}
      </h3>
      <p className="opname-tijd">
        <OpnameTijd />
      </p>
      <p className="zacht">Stuur het Voorbeeld met de gamepad. De server bewaart 100 posities per seconde, tot 60 seconden.</p>
      <div className="knoppen">
        <button className="knop knop-hoofd" disabled={regie.staat !== "verbonden"} onClick={() => voerUit(regie, "Opnemen")}>
          Stop <kbd>R</kbd>
        </button>
      </div>
    </section>
  );
}

/** Een schuifje dat pas bij loslaten naar de server stuurt; tijdens het slepen toont het de waarde. */
function Schuif({
  label,
  waarde,
  min,
  max,
  stap,
  tekst,
  uit = false,
  zet,
}: {
  label: string;
  waarde: number;
  min: number;
  max: number;
  stap: number;
  tekst: (v: number) => string;
  uit?: boolean;
  zet: (v: number) => void;
}) {
  const [nu, setNu] = useState<number | null>(null);
  const v = nu ?? waarde;
  const klaar = () => {
    if (nu !== null && nu !== waarde) zet(nu);
    setNu(null);
  };
  return (
    <label className={`schuif${uit ? " schuif-uit" : ""}`}>
      <span className="schuif-kop">
        <span>{label}</span>
        <span className="zacht">{tekst(v)}</span>
      </span>
      <input
        type="range"
        min={min}
        max={max}
        step={stap}
        value={v}
        disabled={uit}
        onChange={(e) => setNu(Number(e.target.value))}
        onPointerUp={klaar}
        onKeyUp={klaar}
        onBlur={klaar}
      />
    </label>
  );
}

function Segment<T extends string>({
  label,
  keuzes,
  waarde,
  uit,
  zet,
}: {
  label: string;
  keuzes: { waarde: T; tekst: string; uit?: string }[];
  waarde: T;
  uit?: boolean;
  zet: (v: T) => void;
}) {
  return (
    <div className="segment segment-klein" role="group" aria-label={label}>
      {keuzes.map((k) => (
        <button
          key={k.waarde}
          type="button"
          aria-pressed={k.waarde === waarde}
          disabled={uit || !!k.uit}
          title={k.uit}
          onClick={() => k.waarde !== waarde && zet(k.waarde)}
        >
          {k.tekst}
        </button>
      ))}
    </div>
  );
}

const procent = (v: number) => `${Math.round(v * 100)}%`;
const seconden = (v: number) => `${komma(v)} s`;

/** Een klein krommetje van hoe ver de camera is tegen de tijd (gelijk aan `zachte_voortgang`). */
function Kromme({ start, stop }: { start: number; stop: number }) {
  const m0 = 1 - start;
  const m1 = 1 - stop;
  const punten = Array.from({ length: 25 }, (_, i) => {
    const t = i / 24;
    const y = (t ** 3 - 2 * t ** 2 + t) * m0 + (-2 * t ** 3 + 3 * t ** 2) + (t ** 3 - t ** 2) * m1;
    return `${(t * 60).toFixed(1)},${(30 - y * 28).toFixed(1)}`;
  });
  return (
    <svg className="kromme" viewBox="0 -1 60 32" aria-label="Hoe de camera optrekt en afremt">
      <path className="kromme-as" d="M 0 30 H 60 M 0 2 H 60" />
      <polyline points={punten.join(" ")} />
    </svg>
  );
}

/** De richtingen voor langzaam schuiven, ten opzichte van waar het Standpunt kijkt. */
const RICHTINGEN = ["Vooruit", "Achteruit", "Naar links", "Naar rechts", "Omhoog", "Omlaag"] as const;
type Richting = (typeof RICHTINGEN)[number];

function richtingVector(p: Standpunt, r: Richting): [number, number, number] {
  const [sd, cd] = [Math.sin(p.stand.draai), Math.cos(p.stand.draai)];
  const v: Record<Richting, [number, number, number]> = {
    Vooruit: [-sd, 0, -cd],
    Achteruit: [sd, 0, cd],
    "Naar links": [-cd, 0, sd],
    "Naar rechts": [cd, 0, -sd],
    Omhoog: [0, 1, 0],
    Omlaag: [0, -1, 0],
  };
  return v[r].map((x) => Math.round(x * 1000) / 1000) as [number, number, number];
}

function richtingVan(p: Standpunt, v: [number, number, number]): Richting {
  const lengte = Math.hypot(...v) || 1;
  let beste: Richting = "Vooruit";
  let score = -2;
  for (const r of RICHTINGEN) {
    const w = richtingVector(p, r);
    const d = (w[0] * v[0] + w[1] * v[1] + w[2] * v[2]) / lengte;
    if (d > score) [beste, score] = [r, d];
  }
  return beste;
}

/** Naam, toets, Wis en de regel die zegt wat het item gebruikt: gelijk voor Standpunt en Camerarit. */
function Beheer({ letter, naam, toets, vast, children }: { letter: string; naam: string; toets: string | null; vast: boolean; children?: ReactNode }) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const verbonden = regie.staat === "verbonden";
  const gebruik = s && !vast ? gebruikVan(s, letter) : [];
  const stuur = (o: Opdracht) => void regie.stuur(o, CAMERAWERK);
  if (vast) return <>{children}</>;
  return (
    <>
      {children}
      <label className="veld">
        <span className="veld-kop">
          <span className="label">Toets</span>
          <span className="zacht">Zonder toets kies je het hier of op de plattegrond</span>
        </span>
        <select
          value={toets ?? ""}
          disabled={!verbonden}
          onChange={(e) => stuur({ soort: "ZetStandpuntToets", letter, toets: e.target.value || undefined })}
        >
          <option value="">Geen</option>
          {EIGEN_TOETSEN.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>
      </label>
      <div className="knoppen wis-regel">
        <button
          className="knop"
          disabled={!verbonden || gebruik.length > 0}
          title={gebruik.length ? undefined : `${naam} wissen`}
          onClick={() => stuur({ soort: "WisStandpunt", letter })}
        >
          Wis
        </button>
        {gebruik.length > 0 && <span className="zacht">Gebruikt door {gebruik.join(", ")}</span>}
      </div>
    </>
  );
}

function Naam({ letter, naam, vast, soort }: { letter: string; naam: string; vast: boolean; soort: string }) {
  const regie = useRegie();
  const [tekst, setTekst] = useState(naam);
  const hernoem = () => {
    if (tekst.trim() && tekst.trim() !== naam) void regie.stuur({ soort: "HernoemStandpunt", letter, naam: tekst }, CAMERAWERK);
  };
  if (vast)
    return (
      <h3 className="kop-regel">
        {naam} <span className="stempel">Vast</span>
      </h3>
    );
  return (
    <label className="veld">
      <span className="label">{soort}</span>
      <input
        value={tekst}
        maxLength={MAX_NAAM}
        onChange={(e) => setTekst(e.target.value)}
        onBlur={hernoem}
        onKeyDown={(e) => e.key === "Enter" && hernoem()}
      />
    </label>
  );
}

function Melding() {
  const regie = useRegie();
  const melding = regie.melding?.bij === CAMERAWERK ? regie.melding.tekst : null;
  return melding ? (
    <p className="melding" role="alert">
      {melding}
    </p>
  ) : null;
}

/** Waarom afspelen in het Voorbeeld nu niet kan, of `null`. */
function nietAfspelen(s: Stand | undefined): string | null {
  if (s?.camera.vrij) return "Met de Vrije camera aan toont het Voorbeeld de Ether. Zet Vrij eerst uit.";
  return null;
}

function StandpuntDetail({ p }: { p: Standpunt }) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const verbonden = regie.staat === "verbonden";
  const vrij = s?.camera.vrij ?? false;
  const oefenen = s?.camera.oefenen ?? false;
  const stuur = (o: Opdracht) => void regie.stuur(o, CAMERAWERK);
  const zet = (overgang: CameraOvergang, na_aankomst: NaAankomst = p.na_aankomst) =>
    stuur({ soort: "ZetStandpuntOvergang", letter: p.letter, overgang, na_aankomst });
  const o = p.overgang;
  const knipt = o.soort === "Knippen";
  const na = p.na_aankomst;
  const blok = nietAfspelen(s);
  return (
    <section className="paneel-detail">
      <header>
        <Naam letter={p.letter} naam={p.naam} vast={p.vast} soort="Standpunt" />
      </header>
      <div className="knoppen">
        <button className="knop knop-hoofd" disabled={!verbonden} onClick={() => kiesStandpunt(regie, p.letter)}>
          Kies nu {toetsVan(p) && <kbd>{toetsVan(p)}</kbd>}
        </button>
        <button
          className="knop"
          disabled={!verbonden || !!blok}
          title={blok ?? "Speel de overgang vanaf de Ether af in het Voorbeeld"}
          onClick={() => stuur({ soort: "ProbeerInVoorbeeld", letter: p.letter })}
        >
          Probeer in Voorbeeld
        </button>
      </div>
      <div className="instelling">
        <div className="instelling-kop">
          <h4 className="label">Overgang hierheen</h4>
          <Segment
            label="Soort overgang"
            keuzes={[
              { waarde: "Bewegen", tekst: "Bewegen" },
              { waarde: "Knippen", tekst: "Knippen" },
            ]}
            waarde={o.soort}
            uit={!verbonden}
            zet={(soort) => zet({ ...o, soort })}
          />
        </div>
        <Schuif label="Duur" waarde={o.duur_s} min={DUUR_MIN} max={DUUR_MAX} stap={0.1} tekst={seconden} uit={knipt || !verbonden} zet={(duur_s) => zet({ ...o, duur_s })} />
        <div className="schuif-met-kromme">
          <div>
            <Schuif label="Zacht starten" waarde={o.zacht_starten} min={0} max={1} stap={0.05} tekst={procent} uit={knipt || !verbonden} zet={(zacht_starten) => zet({ ...o, zacht_starten })} />
            <Schuif label="Zacht stoppen" waarde={o.zacht_stoppen} min={0} max={1} stap={0.05} tekst={procent} uit={knipt || !verbonden} zet={(zacht_stoppen) => zet({ ...o, zacht_stoppen })} />
          </div>
          <Kromme start={knipt ? 0 : o.zacht_starten} stop={knipt ? 0 : o.zacht_stoppen} />
        </div>
      </div>
      <div className="instelling">
        <div className="instelling-kop">
          <h4 className="label">Na aankomst</h4>
          <Segment
            label="Na aankomst"
            keuzes={[
              { waarde: "Stil", tekst: "Stil" },
              { waarde: "Schuiven", tekst: "Langzaam schuiven" },
            ]}
            waarde={na.soort}
            uit={!verbonden}
            zet={(soort) =>
              zet(o, soort === "Stil" ? { soort: "Stil" } : { soort: "Schuiven", richting: richtingVector(p, "Naar rechts"), cm_per_s: 3 })
            }
          />
        </div>
        {na.soort === "Schuiven" && (
          <>
            <label className="veld">
              <span className="label">Richting</span>
              <select
                value={richtingVan(p, na.richting)}
                disabled={!verbonden}
                onChange={(e) => zet(o, { ...na, richting: richtingVector(p, e.target.value as Richting) })}
              >
                {RICHTINGEN.map((r) => (
                  <option key={r}>{r}</option>
                ))}
              </select>
            </label>
            <Schuif
              label="Snelheid"
              waarde={na.cm_per_s}
              min={1}
              max={10}
              stap={0.5}
              tekst={(v) => `${komma(v)} cm/s, stopt na 20 s`}
              uit={!verbonden}
              zet={(cm_per_s) => zet(o, { ...na, cm_per_s })}
            />
          </>
        )}
      </div>
      <div className="instelling">
        <h4 className="label">Positie</h4>
        <div className="knoppen">
          <button
            className="knop"
            disabled={!verbonden || !(vrij || oefenen)}
            title={
              oefenen
                ? "Zet dit Standpunt waar de camera van het Voorbeeld nu staat"
                : vrij
                  ? "Zet dit Standpunt waar de camera nu staat"
                  : "Zet eerst Vrij of Oefenen aan"
            }
            onClick={() => stuur({ soort: "OverschrijfStandpunt", letter: p.letter })}
          >
            Vervang door huidige camera
          </button>
        </div>
      </div>
      <Beheer letter={p.letter} naam={p.naam} toets={toetsVan(p)} vast={p.vast} />
      <Melding />
    </section>
  );
}

/** De balk van de opname: afgeknipte stukken gearceerd, een groene streep waar het Voorbeeld is. */
function Ritbalk({ r }: { r: Camerarit }) {
  const camera = useCamera();
  const s = useRegie().stand?.stand;
  const lengte = Math.max(opnameS(r), 0.01);
  const b = r.bijschaving;
  const speelt = s?.camera.proef?.letter === r.letter || s?.klaar?.standpunt === r.letter;
  const plek = speelt ? camera?.voorbeeld_rit : undefined;
  const deel = (v: number) => `${Math.min(Math.max((v / lengte) * 100, 0), 100)}%`;
  return (
    <div className="ritbalk" aria-label={`Opname van ${komma(lengte)} s, ${komma(stukS(r))} s over`}>
      <div className="ritbalk-baan">
        <span className="ritbalk-eraf" style={{ left: 0, width: deel(b.begin_eraf_s) }} />
        <span className="ritbalk-eraf" style={{ right: 0, width: deel(b.eind_eraf_s) }} />
        {plek !== undefined && <span className="ritbalk-kop" style={{ left: deel(plek) }} />}
      </div>
      <div className="ritbalk-schaal zacht">
        <span>0 s</span>
        <span>opname {seconden(lengte)}</span>
      </div>
    </div>
  );
}

function RitDetail({ r }: { r: Camerarit }) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const verbonden = regie.staat === "verbonden";
  const b = r.bijschaving;
  const stuur = (o: Opdracht) => void regie.stuur(o, CAMERAWERK);
  const zet = (wijzig: Partial<Bijschaving>) => stuur({ soort: "BijschaafCamerarit", letter: r.letter, bijschaving: { ...b, ...wijzig } });
  const lengte = opnameS(r);
  const stuk = stukS(r);
  const herhalen = kanHerhalen(r);
  const blok = nietAfspelen(s);
  const opnieuw = async () => {
    if (!s?.camera.oefenen) {
      const a = await regie.stuur({ soort: "ZetOefenen", aan: true }, CAMERAWERK);
      if (!a.uitgevoerd) return;
    }
    stuur({ soort: "ZetOpnemen", aan: true, voor: r.letter });
  };
  return (
    <section className="paneel-detail">
      <header>
        <Naam letter={r.letter} naam={r.naam} vast={r.vast} soort="Camerarit" />
      </header>
      <div className="knoppen">
        <button className="knop knop-hoofd" disabled={!verbonden} onClick={() => kiesStandpunt(regie, r.letter)}>
          Kies nu {toetsVan(r) && <kbd>{toetsVan(r)}</kbd>}
        </button>
        <button
          className="knop"
          disabled={!verbonden || !!blok}
          title={blok ?? "Speel de rit vanaf de Ether af in het Voorbeeld"}
          onClick={() => stuur({ soort: "ProbeerInVoorbeeld", letter: r.letter })}
        >
          Speel af in Voorbeeld
        </button>
      </div>
      <Ritbalk r={r} />
      <div className="instelling">
        <Schuif label="Gladheid" waarde={b.gladheid} min={0} max={1} stap={0.05} tekst={procent} uit={!verbonden} zet={(gladheid) => zet({ gladheid })} />
        <Schuif
          label="Duur"
          waarde={b.duur_s}
          min={Math.max(stuk / DUUR_FACTOR, 0.1)}
          max={stuk * DUUR_FACTOR}
          stap={0.1}
          tekst={(v) => `${seconden(v)} (opgenomen ${seconden(stuk)})`}
          uit={!verbonden}
          zet={(duur_s) => zet({ duur_s })}
        />
        <Schuif
          label="Begin eraf"
          waarde={b.begin_eraf_s}
          min={0}
          max={Math.max(lengte - b.eind_eraf_s - RIT_MIN_S, 0)}
          stap={0.1}
          tekst={seconden}
          uit={!verbonden}
          zet={(begin_eraf_s) => zet({ begin_eraf_s })}
        />
        <Schuif
          label="Eind eraf"
          waarde={b.eind_eraf_s}
          min={0}
          max={Math.max(lengte - b.begin_eraf_s - RIT_MIN_S, 0)}
          stap={0.1}
          tekst={seconden}
          uit={!verbonden}
          zet={(eind_eraf_s) => zet({ eind_eraf_s })}
        />
        <div className="schuif-met-kromme">
          <div>
            <Schuif label="Zacht starten" waarde={b.zacht_starten} min={0} max={1} stap={0.05} tekst={procent} uit={!verbonden} zet={(zacht_starten) => zet({ zacht_starten })} />
            <Schuif label="Zacht stoppen" waarde={b.zacht_stoppen} min={0} max={1} stap={0.05} tekst={procent} uit={!verbonden} zet={(zacht_stoppen) => zet({ zacht_stoppen })} />
          </div>
          <Kromme start={b.zacht_starten} stop={b.zacht_stoppen} />
        </div>
      </div>
      <div className="instelling">
        <div className="instelling-kop">
          <h4 className="label">Begin</h4>
          <Segment
            label="Begin"
            keuzes={[
              { waarde: "Overgang", tekst: "Eerst overgang" },
              { waarde: "Knippen", tekst: "Knippen" },
            ]}
            waarde={b.begin}
            uit={!verbonden}
            zet={(begin) => zet({ begin })}
          />
        </div>
        <div className="instelling-kop">
          <h4 className="label">Eind</h4>
          <Segment
            label="Eind"
            keuzes={[
              { waarde: "Stoppen", tekst: "Stoppen" },
              { waarde: "Herhalen", tekst: "Herhalen", uit: herhalen ? `Herhalen kan niet: ${herhalen}` : undefined },
            ]}
            waarde={b.eind}
            uit={!verbonden}
            zet={(eind) => zet({ eind })}
          />
        </div>
        {herhalen && <p className="zacht">Herhalen kan niet: {herhalen}.</p>}
      </div>
      <div className="knoppen">
        {r.vast ? (
          <button
            className="knop"
            disabled={!verbonden || !ritAangepast(r)}
            title={ritAangepast(r) ? "Zet de naam en de schuifjes terug zoals de rit meekwam" : "Deze rit staat zoals hij meekwam"}
            onClick={() => stuur({ soort: "HerstelCamerarit", letter: r.letter })}
          >
            Terug naar standaard
          </button>
        ) : (
          <button className="knop" disabled={!verbonden || !!s?.camera.vrij} title={s?.camera.vrij ? "Zet eerst Vrij uit" : "Zet Oefenen aan en neem deze rit opnieuw op"} onClick={() => void opnieuw()}>
            Opnieuw opnemen
          </button>
        )}
      </div>
      <Beheer letter={r.letter} naam={r.naam} toets={toetsVan(r)} vast={r.vast} />
      <Melding />
    </section>
  );
}
