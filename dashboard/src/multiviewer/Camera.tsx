// Het camerablok in de multiviewer-rij (DESIGN.md): het Standpunt of de Camerarit van de Ether, de
// plattegrond en de toetsen. Vrij zet de Vrije camera aan of uit (de gamepad aan de server stuurt hem);
// Oefenen laat de gamepad alleen het Voorbeeld sturen, en Opnemen neemt daarbij een Camerarit op.
// Met Vrij of Oefenen aan komt er "Bewaar als Standpunt" bij.
import { plaatsVan } from "../persoonskaart/persoonskaart";
import { useStoelkiezer } from "../persoonskaart/stoelkeuze";
import { AANSCHUIVEN, BLIJF, BLIJF_NAAM, BOVEN_HET_PANEEL, BOVEN_HET_PANEEL_NAAM, opnameTijd } from "../regie/camerarit";
import { useCamera, useRegie } from "../regie/regie";
import { CAMERA, kiesStandpunt, voerUit } from "../regie/take";
import { toetsVan } from "../regie/toetsen";
import type { Stand } from "../typen/Stand";
import type { Standpunt } from "../typen/Standpunt";
import { Plattegrond } from "./Plattegrond";
import "../onderdelen/camera.css";

/** Een naam voor een nieuw eigen Standpunt; de Regie geeft het later een betere onder Camera. */
export function nieuweNaam(standpunten: Standpunt[]): string {
  return `Eigen ${standpunten.filter((p) => !p.vast).length + 1}`;
}

/** De naam van het Standpunt of de Camerarit met deze sleutel, of van het shot op een Stoel (Persoonskaart). */
export function naamVan(s: Stand | undefined, letter: string | null): string {
  if (!s || !letter) return "";
  const stoel = /^stoel-(\d+-\d+-\d+)$/.exec(letter);
  if (stoel) return `Stoel ${stoel[1]}`;
  if (letter === AANSCHUIVEN || letter.startsWith(`${AANSCHUIVEN}-`)) return "Aanschuiven";
  if (letter === BLIJF) return BLIJF_NAAM;
  if (letter === BOVEN_HET_PANEEL) return BOVEN_HET_PANEEL_NAAM;
  return s.standpunten.find((p) => p.letter === letter)?.naam ?? s.cameraritten.find((r) => r.letter === letter)?.naam ?? "";
}

/** Hoe lang de opname die loopt al duurt; tekent alleen zichzelf 10 keer per seconde opnieuw. */
export function OpnameTijd() {
  const camera = useCamera();
  return <>{opnameTijd(camera?.opname ?? 0)}</>;
}

/** Oefenen en Opnemen als toetsen, zoals in het camerablok en in het onderdeel Camera. */
export function OefenToetsen({ klasse }: { klasse: string }) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const verbonden = regie.staat === "verbonden";
  const oefenen = s?.camera.oefenen ?? false;
  const neemtOp = !!s?.camera.opname;
  return (
    <>
      <button
        className={`${klasse} toets-oefenen`}
        aria-pressed={oefenen}
        disabled={!verbonden || neemtOp}
        title={
          neemtOp
            ? "Stop eerst de opname"
            : "Oefenen aan of uit: de gamepad stuurt alleen de camera van Preview, Live blijft staan"
        }
        onClick={() => voerUit(regie, "Oefenen")}
      >
        <kbd>O</kbd>
        <span>Oefenen</span>
      </button>
      <button
        className={`${klasse} toets-opnemen`}
        aria-pressed={neemtOp}
        disabled={!verbonden || !oefenen}
        title={oefenen ? "Start of stop de opname van een Camerarit. B op de gamepad doet hetzelfde." : "Opnemen kan alleen tijdens Oefenen"}
        onClick={() => voerUit(regie, "Opnemen")}
      >
        {neemtOp ? <span className="punt punt-ether" aria-hidden /> : <kbd>R</kbd>}
        <span>{neemtOp ? <>Stop <OpnameTijd /></> : "Opnemen"}</span>
      </button>
    </>
  );
}

export function Camera() {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const standpunten = s?.standpunten ?? [];
  const ritten = s?.cameraritten ?? [];
  const vrij = s?.camera.vrij ?? false;
  const oefenen = s?.camera.oefenen ?? false;
  const huidig = vrij ? null : (s?.camera.standpunt ?? null);
  const naam = vrij ? "Vrije camera" : naamVan(s, huidig);
  const verbonden = regie.staat === "verbonden";
  const melding = regie.melding?.bij === CAMERA ? regie.melding.tekst : null;
  const kies = (letter: string, knip = false) => verbonden && kiesStandpunt(regie, letter, knip);
  const metToets = [...standpunten, ...ritten].filter((p) => toetsVan(p));
  const kiesStoel = useStoelkiezer();
  const ether = s?.ether?.graphic;
  const stoel = ether?.sjabloon === "Persoonskaart" ? ether.gegevens?.stoel : undefined;
  const vastePunten = metToets.filter((p) => standpunten.includes(p as Standpunt));
  const ritToetsen = metToets.filter((p) => !standpunten.includes(p as Standpunt));
  const toets = (p: (typeof metToets)[number], rit: boolean) => (
    <button
      key={p.letter}
      className={`standpunt-toets${rit ? " standpunt-rit" : ""}`}
      aria-pressed={p.letter === huidig}
      disabled={!verbonden}
      title={`${rit ? "Camerarit" : "Standpunt"} ${p.naam}: de Live-camera gaat erheen. Shift+${toetsVan(p)} knipt meteen.`}
      onClick={(e) => kies(p.letter, e.shiftKey)}
    >
      <kbd>{toetsVan(p)}</kbd>
      <span className="standpunt-naam">{p.naam}</span>
      {rit && "bijschaving" in p && <span className="standpunt-duur">{Math.round(p.bijschaving.duur_s)} s</span>}
    </button>
  );
  return (
    <section className="camera" aria-label="Camera">
      <div className="camera-kop">
        <span className="monitor-label" title="Deze toetsen sturen de Live-camera">
          Camera <span className="camera-live">Live</span>
        </span>
        <span className="camera-naam" title={naam}>
          {naam}
        </span>
      </div>
      <div className="camera-kaart">
        <Plattegrond
          standpunten={standpunten}
          cameraritten={ritten}
          huidig={huidig}
          vrij={vrij || oefenen}
          voorbeeld={oefenen || !!s?.camera.proef}
          stoelkleuren={s?.stoelkleuren ?? []}
          uitgelicht={stoel ? plaatsVan(stoel) : null}
          kiesStoel={kiesStoel}
          kies={(l) => kies(l)}
        />
        <span className="camera-schakels">
          <button
            className="camera-schakel"
            aria-pressed={vrij}
            disabled={!verbonden}
            title="Vrije camera aan of uit: de gamepad stuurt de Live-camera. Start op de gamepad doet hetzelfde."
            onClick={() => voerUit(regie, "VrijeCamera")}
          >
            <kbd>V</kbd>
            <span>Vrij</span>
          </button>
          <OefenToetsen klasse="camera-schakel" />
        </span>
      </div>
      <div className="standpunt-toetsen" role="group" aria-label="Standpunten">
        {vastePunten.map((p) => toets(p, false))}
      </div>
      {ritToetsen.length > 0 && <span className="rit-kop">Cameraritten · direct live</span>}
      {ritToetsen.length > 0 && (
        <div className="standpunt-toetsen rit-toetsen" role="group" aria-label="Cameraritten">
          {ritToetsen.map((p) => toets(p, true))}
        </div>
      )}
      {(vrij || oefenen) && (
        <button
          className="standpunt-toets standpunt-bewaar"
          disabled={!verbonden}
          title={
            oefenen
              ? "Bewaar waar de camera van Preview nu staat als eigen Standpunt"
              : "Bewaar waar de camera nu staat als eigen Standpunt"
          }
          onClick={() => void regie.stuur({ soort: "BewaarStandpunt", naam: nieuweNaam(standpunten) }, CAMERA)}
        >
          <span>Bewaar als Standpunt</span>
        </button>
      )}
      {melding && (
        <p className="take-melding" role="alert">
          {melding}
        </p>
      )}
    </section>
  );
}
