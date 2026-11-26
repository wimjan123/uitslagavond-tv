// Horizontale OBS/DaVinci Transportbalk onder de monitoren:
// [ Beweeg | Fade (duur) | Knip ] · [ TAKE Spatie ] · [ Take out X ] · [ Kleur uit Z ] · [ ▾ Meer ] · [ Undo ] · [ Status ]
import { useEffect, useRef, useState } from "react";
import { ChevronDown, Eraser, Play, RefreshCw, RotateCcw, SkipForward, Sparkles, Square, Trash2, X } from "lucide-react";
import { allesUit, kan, klaarStaatInBeeld, TAKE, voerUit, wisKlaar, zetOvergang } from "../regie/take";
import { geldigHerstel, herstel, useHerstel } from "../regie/herstel";
import { useRegie } from "../regie/regie";
import type { OvergangSoort } from "../typen/OvergangSoort";

const FADE_DUREN = [300, 600, 1000, 1500, 2000];

export function TransportBalk({ opOpenSysteem }: { opOpenSysteem: () => void }) {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const ongedaan = geldigHerstel(useHerstel(), stand);
  const [meerOpen, setMeerOpen] = useState(false);
  const [klok, setKlok] = useState(() => tijdNu());
  const meerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const id = window.setInterval(() => setKlok(tijdNu()), 1000);
    return () => window.clearInterval(id);
  }, []);

  useEffect(() => {
    if (!meerOpen) return;
    const klik = (e: MouseEvent) => {
      if (meerRef.current && !meerRef.current.contains(e.target as Node)) setMeerOpen(false);
    };
    window.addEventListener("mousedown", klik);
    return () => window.removeEventListener("mousedown", klik);
  }, [meerOpen]);

  const overgang: OvergangSoort = stand?.camera.overgang ?? "Bewegen";
  const fadeMs = stand?.camera.overvloei_ms ?? 600;
  const kanTake = kan("Toon", stand);
  const kanUpdate = kan("Bijwerken", stand);
  const kanUit = kan("UitBeeld", stand);
  const kanKleurUit = kan("KleurUit", stand);
  const kanOpbouwOver = kan("SlaOpbouwOver", stand);
  const kanVervers = kan("Ververs", stand);
  const heeftKlaarIets = !!(stand?.klaar || stand?.klaar_stoelbeeld || stand?.camera.klaar_standpunt);

  // Korte samenvatting op de Take-knop zodat de Regie direct ziet welke lagen veranderen
  const lagen: string[] = [];
  if (stand?.klaar && !klaarStaatInBeeld(stand)) lagen.push("Graphic");
  if (stand?.klaar_stoelbeeld) lagen.push("Stoelen");
  if (stand?.camera.klaar_standpunt && stand.camera.klaar_standpunt !== "blijf") lagen.push("Camera");
  const takeSub = lagen.length > 0 ? lagen.join(" + ") : null;

  // Status pill indicators
  const verbonden = regie.staat === "verbonden";
  const heeftEther = regie.renderers.includes("Ether");
  const heeftReserve = regie.renderers.includes("Reserve");
  const geweigerd = regie.rendererstatus.geweigerd.length > 0;
  const dataWaarschuwing = !!(
    (stand?.cijfers.laatste_invoer?.waarschuwingen.length ?? 0) > 0 ||
    (stand?.kamer.laatste_ophaal?.fracties_nieuw.length ?? 0) > 0 ||
    (stand?.zaalmelding.zonder_stoel.length ?? 0) > 0
  );
  const pillKlas = !verbonden || geweigerd ? "err" : !heeftEther || dataWaarschuwing ? "warn" : "";

  return (
    <div className="transport-bar" role="toolbar" aria-label="Uitzendbesturing">
      {/* Links: Overgang (Beweeg / Fade / Knip) */}
      <div className="transport-group">
        <div className="seg" role="group" aria-label="Overgang bij Take">
          <button
            type="button"
            className={overgang === "Bewegen" ? "active amber" : ""}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => zetOvergang(regie, "Bewegen")}
            title="Camera beweegt vloeiend naar het nieuwe Standpunt"
          >
            Beweeg
          </button>
          <button
            type="button"
            className={overgang === "Overvloeien" ? "active amber" : ""}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => zetOvergang(regie, "Overvloeien", fadeMs)}
            title="Beeld vloeit over (Fade) naar het nieuwe Standpunt of de Camerarit"
          >
            Fade
          </button>
          <button
            type="button"
            className={overgang === "Knippen" ? "active amber" : ""}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => zetOvergang(regie, "Knippen")}
            title="Directe harde knip bij Take"
          >
            Knip
          </button>
        </div>

        {overgang === "Overvloeien" && (
          <select
            className="fade-ms-select"
            value={FADE_DUREN.includes(fadeMs) ? fadeMs : 600}
            onChange={(e) => zetOvergang(regie, "Overvloeien", Number(e.target.value))}
            aria-label="Duur van Fade"
          >
            {FADE_DUREN.map((ms) => (
              <option key={ms} value={ms}>
                {(ms / 1000).toFixed(1).replace(".", ",")} s
              </option>
            ))}
          </select>
        )}

        {regie.melding?.bij === TAKE && (
          <span className="chip live transport-err" title={regie.melding.tekst}>
            {regie.melding.tekst}
          </span>
        )}
      </div>

      {/* Midden: TAKE · Take out · Kleur uit · Meer */}
      <div className="transport-center">
        <button
          type="button"
          className={`btn take-btn${kanTake ? " ready" : ""}`}
          disabled={!kanTake}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => voerUit(regie, "Toon")}
          title="Zet wat in Preview klaarstaat op Live (Spatie)"
        >
          <Play size={14} fill="currentColor" />
          <span>TAKE{takeSub ? ` · ${takeSub}` : ""}</span>
          <kbd className="key">Spatie</kbd>
        </button>

        {kanUpdate && (
          <button
            type="button"
            className="btn amber"
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => voerUit(regie, "Bijwerken")}
            title="Werk de huidige live Graphic vloeiend bij (B)"
          >
            <RefreshCw size={13} />
            <span>Update</span>
            <kbd className="key">B</kbd>
          </button>
        )}

        <button
          type="button"
          className="btn"
          disabled={!kanUit}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => voerUit(regie, "UitBeeld")}
          title="Haalt alleen de 2D-Graphic uit Live; camera en stoelen blijven (X)"
        >
          <Square size={12} />
          <span>Take out</span>
          <kbd className="key">X</kbd>
        </button>

        <button
          type="button"
          className="btn"
          disabled={!kanKleurUit}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => voerUit(regie, "KleurUit")}
          title="Zet alleen de stoelen terug naar blauw leer (Z)"
        >
          <Sparkles size={13} />
          <span>Kleur uit</span>
          <kbd className="key">Z</kbd>
        </button>

        <div className="popover-anchor" ref={meerRef}>
          <button
            type="button"
            className="btn"
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => setMeerOpen((o) => !o)}
            aria-expanded={meerOpen}
            title="Extra uitzendacties"
          >
            <span>Meer</span>
            <ChevronDown size={13} />
          </button>

          {meerOpen && (
            <div className="popover-menu" role="menu">
              <button
                type="button"
                className="popover-item"
                disabled={!kanUpdate}
                onClick={() => {
                  voerUit(regie, "Bijwerken");
                  setMeerOpen(false);
                }}
              >
                <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                  <RefreshCw size={13} /> Update live Graphic
                </span>
                <kbd className="key">B</kbd>
              </button>
              <button
                type="button"
                className="popover-item"
                disabled={!kanOpbouwOver}
                onClick={() => {
                  voerUit(regie, "SlaOpbouwOver");
                  setMeerOpen(false);
                }}
              >
                <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                  <SkipForward size={13} /> Opbouw overslaan
                </span>
                <kbd className="key">C</kbd>
              </button>
              <button
                type="button"
                className="popover-item"
                disabled={!kanVervers}
                onClick={() => {
                  voerUit(regie, "Ververs");
                  setMeerOpen(false);
                }}
              >
                <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                  <RefreshCw size={13} /> Cijfers verversen
                </span>
                <kbd className="key">N</kbd>
              </button>
              <div className="popover-sep" />
              <button
                type="button"
                className="popover-item"
                disabled={!heeftKlaarIets}
                onClick={() => {
                  wisKlaar(regie);
                  setMeerOpen(false);
                }}
              >
                <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                  <Eraser size={13} /> Preview leegmaken
                </span>
              </button>
              <button
                type="button"
                className="popover-item danger"
                disabled={!kanUit && !kanKleurUit}
                onClick={() => {
                  void allesUit(regie);
                  setMeerOpen(false);
                }}
              >
                <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                  <Trash2 size={13} /> Alles uit (Graphic + Stoelen)
                </span>
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Rechts: 20s Undo + Systeem Status Pill */}
      <div className="transport-group">
        {ongedaan && (
          <button
            type="button"
            className="undo-pill"
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => herstel(regie)}
            title={`${ongedaan.tekst}: ${ongedaan.wat} (Ctrl+Z)`}
          >
            <RotateCcw size={13} />
            <span>{ongedaan.knop}</span>
            <kbd className="key">Ctrl+Z</kbd>
          </button>
        )}

        <button
          type="button"
          className={`status-pill${pillKlas ? ` ${pillKlas}` : ""}`}
          onMouseDown={(e) => e.preventDefault()}
          onClick={opOpenSysteem}
          title="Systeem, Renderers en Werkplek openen"
        >
          <span className={`dot ${!verbonden ? "live" : heeftEther ? "ok" : "warn"}`} />
          <span>{verbonden ? "Server" : regie.staat}</span>
          <span className="status-sep">·</span>
          <span>Live {heeftEther ? "ok" : "uit"}</span>
          {heeftReserve && (
            <>
              <span className="status-sep">·</span>
              <span>Res</span>
            </>
          )}
          <span className="status-sep">·</span>
          <span className="mono">{klok}</span>
        </button>
      </div>
    </div>
  );
}

function tijdNu(): string {
  const d = new Date();
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}:${String(d.getSeconds()).padStart(2, "0")}`;
}

export { X };
