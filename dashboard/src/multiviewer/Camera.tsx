// Het camerablok in de multiviewer-rij (DESIGN.md): het Standpunt of de Camerarit van de Ether, de
// plattegrond en de toetsen. Vrij zet de Vrije camera aan of uit (de gamepad aan de server stuurt hem);
// Oefenen laat de gamepad alleen het Voorbeeld sturen, en Opnemen neemt daarbij een Camerarit op.
// Met Vrij of Oefenen aan komt er "Bewaar als Standpunt" bij.
import { plaatsVan } from "../persoonskaart/persoonskaart";
import { useStoelkiezer } from "../persoonskaart/stoelkeuze";
import { AANSCHUIVEN, BOVEN_HET_PANEEL, BOVEN_HET_PANEEL_NAAM, opnameTijd } from "../regie/camerarit";
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
            : "Oefenen aan of uit: de gamepad stuurt alleen de camera van het Voorbeeld, de Ether blijft staan"
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
  return (
    <section className="camera" aria-label="Camera">
      <div className="camera-kop">
        <span className="monitor-label">Camera</span>
        <span className="camera-naam">{naam}</span>
      </div>
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
      <div className="standpunt-toetsen">
        {metToets.map((p) => (
          <button
            key={p.letter}
            className="standpunt-toets"
            aria-pressed={p.letter === huidig}
            disabled={!verbonden}
            title={`${p.naam}. Shift+${toetsVan(p)} knipt meteen.`}
            onClick={(e) => kies(p.letter, e.shiftKey)}
          >
            <kbd>{toetsVan(p)}</kbd>
            <span>{p.naam}</span>
          </button>
        ))}
        <button
          className="standpunt-toets"
          aria-pressed={vrij}
          disabled={!verbonden}
          title="Vrije camera aan of uit: de gamepad stuurt de camera van de Ether. Start op de gamepad doet hetzelfde."
          onClick={() => voerUit(regie, "VrijeCamera")}
        >
          <kbd>V</kbd>
          <span>Vrij</span>
        </button>
        <OefenToetsen klasse="standpunt-toets" />
        {(vrij || oefenen) && (
          <button
            className="standpunt-toets"
            disabled={!verbonden}
            title={
              oefenen
                ? "Bewaar waar de camera van het Voorbeeld nu staat als eigen Standpunt"
                : "Bewaar waar de camera nu staat als eigen Standpunt"
            }
            onClick={() => void regie.stuur({ soort: "BewaarStandpunt", naam: nieuweNaam(standpunten) }, CAMERA)}
          >
            <span>Bewaar als Standpunt</span>
          </button>
        )}
      </div>
      {melding && (
        <p className="take-melding" role="alert">
          {melding}
        </p>
      )}
    </section>
  );
}
