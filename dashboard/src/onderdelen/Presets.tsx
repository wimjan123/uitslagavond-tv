// Presets, het startscherm (DESIGN.md, Preset Table): alle bewaarde Graphics als tabel met hun
// vaste nummer. Een klik op een rij of het cijfer zet de Preset klaar; tonen gaat alleen met Toon.
// Een Preset houdt de gegevens van het moment van bewaren.
import { telling } from "../coalitie/coalitie";
import { useState } from "react";
import type { MouseEvent } from "react";
import { graphicTekst, sjabloonNaam, standaardStandpunt, uurMinuut } from "../graphic";
import { cameraKeuzes } from "../regie/camerarit";
import { Plus, Tablet } from "../iconen";
import { Schets } from "../presets/Schets";
import { GROEPEN, bewaardAls, filterPresets, groepVan, statusVan } from "../presets/presets";
import type { Groep } from "../presets/presets";
import { useRegie } from "../regie/regie";
import { PRESETS } from "../regie/take";
import type { Graphic } from "../typen/Graphic";
import type { Preset } from "../typen/Preset";
import type { Stand } from "../typen/Stand";
import { TakeHint, Werkbalk } from "./Werkbalk";
import type { WerkvlakProps } from "./index";
import "./presets.css";

/** Een knop die geen focus pakt bij een klik, zodat de spatie daarna Toon blijft. */
const zonderFocus = (e: MouseEvent) => e.preventDefault();

export function Presets({ kiesOnderdeel }: WerkvlakProps) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const lijst = s?.presets ?? [];
  const [groep, setGroep] = useState<Groep | null>(null);
  const [opTablet, setOpTablet] = useState(false);
  const zichtbaar = filterPresets(lijst, groep, opTablet);
  const verbonden = regie.staat === "verbonden";
  const klaar = s?.klaar ?? null;
  const alBewaard = klaar ? bewaardAls(lijst, klaar.graphic, klaar.standpunt) : null;
  const melding = regie.melding?.bij === PRESETS ? regie.melding.tekst : null;

  return (
    <>
      <Werkbalk
        naam="Presets"
        hint={
          <>
            <kbd>1</kbd> tot <kbd>9</kbd> zet klaar · <TakeHint />
          </>
        }
      >
        <div className="filters" role="group" aria-label="Welke groep">
          <button className="filter" aria-pressed={groep === null} onClick={() => setGroep(null)}>
            Alle {lijst.length}
          </button>
          {GROEPEN.map((g) => {
            const n = lijst.filter((p) => groepVan(p.graphic) === g).length;
            return (
              <button key={g} className="filter" aria-pressed={groep === g} disabled={n === 0 && groep !== g} onClick={() => setGroep(g)}>
                {g} {n}
              </button>
            );
          })}
        </div>
        <button className="filter" aria-pressed={opTablet} onClick={() => setOpTablet(!opTablet)}>
          <Tablet /> Op de tablet {lijst.filter((p) => p.op_tablet).length}
        </button>
        <button
          className="knop knop-werkbalk"
          disabled={!verbonden || !klaar || alBewaard !== null}
          title={alBewaard !== null ? `Staat al als Preset ${alBewaard}` : klaar ? undefined : "Zet eerst een Graphic klaar"}
          onMouseDown={zonderFocus}
          onClick={() => klaar && void regie.stuur({ soort: "BewaarPreset", graphic: klaar.graphic, standpunt: klaar.standpunt }, PRESETS)}
        >
          <Plus /> Voorbeeld bewaren als Preset
        </button>
      </Werkbalk>
      {melding && (
        <p className="presets-melding" role="alert">
          {melding}
        </p>
      )}
      <div className="werkvlak-inhoud">
        <table className="tabel presets-tabel">
          <thead>
            <tr>
              <th className="kol-toets">Toets</th>
              <th className="kol-beeld">Beeld</th>
              <th>Graphic</th>
              <th>Gegevens</th>
              <th className="kol-camera">Camera</th>
              <th className="kol-tablet">Tablet</th>
              <th className="kol-status">Status</th>
            </tr>
          </thead>
          <tbody>
            {s &&
              zichtbaar.map((p) => (
                <Rij key={p.nummer} preset={p} stand={s} verbonden={verbonden} />
              ))}
            {lijst.length > 0 && zichtbaar.length === 0 && (
              <tr className="presets-leeg">
                <td colSpan={7} className="zacht">
                  {["Geen Presets", groep && `in de groep ${groep}`, opTablet && "op de tablet"].filter(Boolean).join(" ")}
                </td>
              </tr>
            )}
            {lijst.length === 0 && (
              <tr className="presets-leeg">
                <td colSpan={7} className="zacht">
                  Nog geen Presets. Zet een Graphic klaar en kies "Voorbeeld bewaren als Preset": de Preset houdt de gegevens van dat
                  moment.
                </td>
              </tr>
            )}
            <tr className="presets-nieuw">
              <td />
              <td colSpan={6}>
                <button className="nieuw-knop" onClick={() => kiesOnderdeel("maken")}>
                  <Plus /> Nieuwe Graphic maken
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </>
  );
}

function Rij({ preset: p, stand: s, verbonden }: { preset: Preset; stand: Stand; verbonden: boolean }) {
  const regie = useRegie();
  const status = statusVan(p, s);
  const letter = p.standpunt ?? standaardStandpunt(p.graphic);
  const standpunt = cameraKeuzes(s.standpunten, s.cameraritten).find((x) => x.letter === letter);
  const rij = status.live ? "rij-live" : status.klaar ? "rij-klaar" : "";
  const zetKlaar = () => verbonden && void regie.stuur({ soort: "ZetPresetKlaar", nummer: p.nummer }, PRESETS);
  return (
    <tr className={rij} onClick={zetKlaar} title={`Zet Preset ${p.nummer} klaar`}>
      <td className="kol-toets">
        <span className="toets-kap">{p.nummer}</span>
      </td>
      <td className="kol-beeld">
        <Schets graphic={p.graphic} kleuren={s.stijl.kleuren} />
      </td>
      <td className="cel-tekst">
        <div className="titel-cel">{sjabloonNaam(p.graphic, s.sjablonen)}</div>
        <div className="zacht een-regel">{p.graphic.sjabloon === "Titelkaart" ? p.graphic.kop : groepVan(p.graphic)}</div>
      </td>
      <td className="cel-tekst">
        <Gegevens graphic={p.graphic} />
        <div className="zacht een-regel">
          bewaard {uurMinuut(p.bewaard_ms)} door {p.bewaard_door}
        </div>
      </td>
      <td className="kol-camera">
        <span className="camera-cel">
          {standpunt?.toets && <kbd>{standpunt.toets}</kbd>}
          {standpunt ? (
            <span className="zacht een-regel">
              {standpunt.soort === "Camerarit" ? `Camerarit ${standpunt.naam}` : standpunt.naam}
            </span>
          ) : (
            <span className="let-op">Standpunt bestaat niet meer</span>
          )}
        </span>
      </td>
      <td className="kol-tablet">
        <button
          className="tablet-knop"
          aria-pressed={p.op_tablet}
          aria-label={p.op_tablet ? `Preset ${p.nummer} van de tablet halen` : `Preset ${p.nummer} op de tablet zetten`}
          title={p.op_tablet ? "Staat op de tablet. Klik om eraf te halen" : "Op de tablet zetten"}
          disabled={!verbonden}
          onMouseDown={zonderFocus}
          onClick={(e) => {
            e.stopPropagation();
            void regie.stuur({ soort: "ZetPresetOpTablet", nummer: p.nummer, aan: !p.op_tablet }, PRESETS);
          }}
        >
          <Tablet />
        </button>
      </td>
      <td className="kol-status">
        <span className="status-cel">
          {status.live && <span className="stempel stempel-live">Live</span>}
          {status.klaar && <span className="stempel stempel-klaar">Klaar</span>}
          {!status.live && !status.klaar && status.wasLiveOm !== null && <span className="zacht">was live {uurMinuut(status.wasLiveOm)}</span>}
        </span>
      </td>
    </tr>
  );
}

/** Wat de Graphic toont buiten de kop: de regel, of de uitkomst van de Stemming. */
function Gegevens({ graphic: g }: { graphic: Graphic }) {
  if (g.sjabloon === "Titelkaart") {
    return <div className="een-regel">{g.regel || <span className="zacht">Alleen een kop</span>}</div>;
  }
  if (g.sjabloon === "Persoonskaart") {
    return (
      <div className="een-regel">
        {graphicTekst(g)}
        {g.regel && <span className="zacht"> · {g.regel}</span>}
      </div>
    );
  }
  if (g.sjabloon === "Zetelverdeling") {
    const labels = [g.namen && "namen", g.zetels && "Zetels"].filter(Boolean).join(" en ");
    return (
      <div className="een-regel">
        {graphicTekst(g)}
        <span className="zacht"> · {labels ? `labels: ${labels}` : "zonder labels"}</span>
      </div>
    );
  }
  if (g.sjabloon === "Zetelbalk" || g.sjabloon === "TweeSets" || g.sjabloon === "UitslagPerPartij") {
    return (
      <div className="een-regel">
        {graphicTekst(g)}
        <span className="zacht"> · {g.grootste ? `grootste ${g.grootste} en overig` : "alle Partijen"}</span>
      </div>
    );
  }
  if (g.sjabloon === "WinstEnVerlies" || g.sjabloon === "PartijUitgelicht") {
    return <div className="een-regel">{graphicTekst(g)}</div>;
  }
  if (g.sjabloon === "Coalitie" || g.sjabloon === "Coalitiebalk") {
    const zetels = g.cijfers ? telling(g.cijfers.set.zetels, g.partijen) : null;
    return (
      <div className="een-regel">
        {g.cijfers?.set.label ?? g.set}
        {zetels !== null && <span className="zacht"> · {zetels} Zetels samen</span>}
      </div>
    );
  }
  if (g.sjabloon === "Eigen") return <div className="een-regel zacht">{graphicTekst(g)}</div>;
  const u = g.uitslag;
  return (
    <div className="een-regel">
      <span className="besluit-woord">{u.besluit}</span>{" "}
      {u.algemene_stemmen ? "met algemene stemmen" : `${u.voor} voor · ${u.tegen} tegen`}
      <span className="zacht"> · {graphicTekst(g)}</span>
    </div>
  );
}
