// De plattegrond van de Zaal van bovenaf (DESIGN.md, Floor Plan): de Stoelen in de kleuren van nu,
// een ronde pin per Standpunt, een vierkante pin met routelijn per Camerarit, een rode markering
// waar de Live-camera staat en kijkt, en tijdens Oefenen of afspelen een groene voor het
// Voorbeeld. Tijdens een opname tekent een groen spoor de route. Met de Vrije camera of Oefenen aan
// toont een gestippelde lijn de onzichtbare ruimte waarin de camera mag vliegen. De Stoel van een
// Persoonskaart heeft een ring; met `kiesStoel` (formulier van de Persoonskaart open) kies je een
// Kamerlid door op zijn Stoel te klikken.
// Assen in meter zoals in de server: x naar rechts, z naar de Voorzitter (onderaan).
import { useEffect, useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import { routeVan } from "../regie/camerarit";
import { useCamera } from "../regie/regie";
import type { Camerarit } from "../typen/Camerarit";
import type { Camerastand } from "../typen/Camerastand";
import type { Standpunt } from "../typen/Standpunt";
import { toetsVan } from "../regie/toetsen";
import { HANGLAMPEN, RUIMTE, STOELPLEKKEN } from "../typen/zaalkaart";

/** Van -14 tot 14 meter breed, van de tribune tot net achter de wand achter de Voorzitter. Staat een
 * Standpunt verder naar voren (zoals A, hoog achter de Voorzitter), dan groeit de
 * kaart naar onderen mee, zodat elke pin te zien en aan te klikken is. Niet naar de vrije camera: dan
 * zou de kaart tijdens het vliegen verspringen. */
function viewBox(zs: number[]): string {
  const onder = Math.min(12, Math.max(5.8, ...zs.map((z) => z + 1.4)));
  return `-14 -13.6 28 ${(13.6 + onder).toFixed(1)}`;
}
/** De vloer: de halve cirkel tot de balustrade van de tribune en de rechthoek voor de Voorzitter. */
const VLOER = "M -13 0 A 13 13 0 0 1 13 0 L 13 0 L 11.5 0 L 11.5 4.6 L -11.5 4.6 L -11.5 0 Z";
/** De balie van de Voorzitter en vak K, als lijn. */
const BALIE = "-10,1.2 -5.6,1.2 -1.6,3.7 2.4,3.7 5.2,1.5 10,1.2";
/** De wanden van de onzichtbare ruimte: de rechthoek voor de Voorzitter en de halve cirkel. */
const { straal: R, voorHalveBreedte: W, voorDiepte: D } = RUIMTE;
const RAND = `M ${-W} ${D} L ${-W} 0 L ${-R} 0 A ${R} ${R} 0 0 1 ${R} 0 L ${W} 0 L ${W} ${D} Z`;
/** Kleur van een Stoel zonder kleur in de Stand: het blauwe leer. */
const LEER = "#3d4f73";
/** Lengte van de kijkkegel van een pin, in meter. */
const KEGEL = 4.5;

function horizontaleHoek(s: Camerastand): number {
  return 2 * Math.atan(Math.tan(s.beeldhoek / 2) * (16 / 9));
}

/** Een kijkkegel als driehoek vanaf de camera, of een cirkel als de camera bijna recht omlaag kijkt. */
function Kegel({ stand, className, lengte = KEGEL }: { stand: Camerastand; className: string; lengte?: number }) {
  const [x, , z] = stand.positie;
  if (stand.helling < -1.05) return <circle className={className} cx={x} cy={z} r={lengte * 0.55} />;
  const half = Math.min(horizontaleHoek(stand) / 2, 1.4);
  const punt = (a: number) => `${x - Math.sin(stand.draai + a) * lengte},${z - Math.cos(stand.draai + a) * lengte}`;
  return <polygon className={className} points={`${x},${z} ${punt(half)} ${punt(-half)}`} />;
}

/** Het groene spoor van de opname die loopt: de plekken van het Voorbeeld, 10 per seconde. */
function useSpoor(): [number, number][] {
  const camera = useCamera();
  const [spoor, setSpoor] = useState<[number, number][]>([]);
  const liep = useRef(false);
  useEffect(() => {
    const loopt = camera?.opname !== undefined;
    if (!loopt) {
      if (liep.current) setSpoor([]);
      liep.current = false;
      return;
    }
    const [x, , z] = camera!.voorbeeld.stand.positie;
    setSpoor((oud) => (liep.current ? [...oud, [x, z]] : [[x, z]]));
    liep.current = true;
  }, [camera]);
  return spoor;
}

export function Plattegrond({
  standpunten,
  cameraritten = [],
  huidig,
  vrij = false,
  voorbeeld = false,
  gekozen = null,
  groot = false,
  stoelkleuren,
  uitgelicht = null,
  kiesStoel = null,
  kies,
}: {
  standpunten: Standpunt[];
  cameraritten?: Camerarit[];
  /** Het Standpunt of de Camerarit van de Ether. */
  huidig: string | null;
  /** De Vrije camera of Oefenen staat aan: toon de onzichtbare ruimte. */
  vrij?: boolean;
  /** Toon ook de camera van Preview (tijdens Oefenen of als het iets afspeelt). */
  voorbeeld?: boolean;
  /** Het Standpunt dat de Regie bewerkt (onder Camera). */
  gekozen?: string | null;
  groot?: boolean;
  stoelkleuren: string[];
  /** De plaats van de Stoel die oplicht (Persoonskaart in de Ether), of `null`. */
  uitgelicht?: number | null;
  /** Kiest het Kamerlid op een Stoel; zonder doet een klik op een Stoel niets. */
  kiesStoel?: ((plaats: number) => void) | null;
  kies: (letter: string) => void;
}) {
  const camera = useCamera();
  const spoor = useSpoor();
  const druk = (e: KeyboardEvent, letter: string) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      e.stopPropagation();
      kies(letter);
    }
  };
  const actief = standpunten.find((s) => s.letter === huidig);
  // een vaste Camerarit heeft een toets; zijn route en pin staan alleen in beeld als hij gekozen is
  const zichtbareRitten = cameraritten.filter((r) => !r.vast || r.letter === huidig || r.letter === gekozen);
  return (
    <svg className={`plattegrond${groot ? " plattegrond-groot" : ""}`} viewBox={viewBox(standpunten.map((p) => p.stand.positie[2]))} role="group" aria-label="Plattegrond van de Zaal">
      <path className="plattegrond-vloer" d={VLOER} />
      <polyline className="plattegrond-balie" points={BALIE} />
      {HANGLAMPEN.map(([x, z], i) => (
        <circle key={i} className="plattegrond-lamp" cx={x} cy={z} r={0.18} />
      ))}
      {STOELPLEKKEN.map(([x, z], i) =>
        kiesStoel ? (
          <circle
            key={i}
            className="stoel-kiesbaar"
            cx={x}
            cy={z}
            r={0.36}
            fill={stoelkleuren[i] ?? LEER}
            role="button"
            aria-label={`Stoel ${i + 1}`}
            onClick={() => kiesStoel(i)}
          />
        ) : (
          <circle key={i} cx={x} cy={z} r={0.3} fill={stoelkleuren[i] ?? LEER} />
        ),
      )}
      {uitgelicht !== null && STOELPLEKKEN[uitgelicht] && (
        <circle className="stoel-uitgelicht" cx={STOELPLEKKEN[uitgelicht][0]} cy={STOELPLEKKEN[uitgelicht][1]} r={0.62} aria-label="Stoel van de Persoonskaart" />
      )}
      {vrij && <path className="plattegrond-ruimte" d={RAND} aria-label="Onzichtbare ruimte van de camera" />}
      {actief && <Kegel stand={actief.stand} className="pin-kegel" />}
      {zichtbareRitten.map((r) => {
        const route = routeVan(r);
        const aan = r.letter === huidig || r.letter === gekozen;
        return (
          <polyline
            key={`route-${r.letter}`}
            className={`rit-route${aan ? " rit-route-aan" : ""}`}
            points={route.map(([x, z]) => `${x},${z}`).join(" ")}
          />
        );
      })}
      {spoor.length > 1 && <polyline className="plattegrond-spoor" points={spoor.map(([x, z]) => `${x},${z}`).join(" ")} />}
      {/* onder de pins, zodat de letter leesbaar blijft als de camera op een Standpunt staat */}
      {camera && voorbeeld && (
        <g className="camera-voorbeeld" aria-label="Camera van Preview">
          <Kegel stand={camera.voorbeeld.stand} className="camera-voorbeeld-kegel" lengte={2.2} />
          <circle cx={camera.voorbeeld.stand.positie[0]} cy={camera.voorbeeld.stand.positie[2]} r={1.1} />
        </g>
      )}
      {camera && (
        <g className="camera-ether" aria-label="Live-camera">
          <Kegel stand={camera.ether.stand} className="camera-ether-kegel" lengte={2.2} />
          <circle cx={camera.ether.stand.positie[0]} cy={camera.ether.stand.positie[2]} r={1.45} />
        </g>
      )}
      {standpunten.map((s) => {
        const [x, , z] = s.stand.positie;
        const aan = s.letter === huidig;
        const toets = toetsVan(s);
        return (
          <g
            key={s.letter}
            className={`pin${aan ? " pin-aan" : ""}${s.letter === gekozen ? " pin-gekozen" : ""}`}
            role="button"
            tabIndex={0}
            aria-label={toets ? `${toets}: ${s.naam}` : s.naam}
            aria-pressed={aan}
            onClick={() => kies(s.letter)}
            onKeyDown={(e) => druk(e, s.letter)}
          >
            <title>{toets ? `${s.naam} (${toets})` : s.naam}</title>
            <circle cx={x} cy={z} r={toets ? 1.05 : 0.7} />
            {toets && (
              <text x={x} y={z} dy="0.36">
                {toets}
              </text>
            )}
          </g>
        );
      })}
      {zichtbareRitten.map((r) => {
        const route = routeVan(r);
        const [x, z] = route[0] ?? [0, 0];
        const [ex, ez] = route[route.length - 1] ?? [0, 0];
        const aan = r.letter === huidig;
        const toets = toetsVan(r);
        const maat = toets ? 1.9 : 1.3;
        return (
          <g
            key={r.letter}
            className={`pin pin-rit${aan ? " pin-aan" : ""}${r.letter === gekozen ? " pin-gekozen" : ""}`}
            role="button"
            tabIndex={0}
            aria-label={toets ? `${toets}: Camerarit ${r.naam}` : `Camerarit ${r.naam}`}
            aria-pressed={aan}
            onClick={() => kies(r.letter)}
            onKeyDown={(e) => druk(e, r.letter)}
          >
            <title>{toets ? `Camerarit ${r.naam} (${toets})` : `Camerarit ${r.naam}`}</title>
            <circle className="rit-eind" cx={ex} cy={ez} r={0.35} />
            <rect x={x - maat / 2} y={z - maat / 2} width={maat} height={maat} rx={0.15} />
            {toets && (
              <text x={x} y={z} dy="0.36">
                {toets}
              </text>
            )}
          </g>
        );
      })}
    </svg>
  );
}
