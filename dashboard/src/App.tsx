// uitslagavond-tv — DaVinci Deck Shell:
// - Twin 720p Preview & Live viewers bovenaan (ingeklapt in Studio-modus)
// - Horizontale OBS/DaVinci Transportbalk eronder
// - Actieve werkruimte in het midden (Regie, Zaal & Camera, Stemmingen, Coalitie, Cijfers, Studio)
// - DaVinci onderste paginabalk + Systeem slide-over lade vanuit de statuspil
// - Pop-out monitor-venster op /monitors
import { useEffect, useState } from "react";
import { BarChart3, Camera, Layers, Palette, PieChart, Vote } from "lucide-react";
import { RegieBron, useRegie } from "./regie/regie";
import type { Toegang } from "./regie/regie";
import { voerUit } from "./regie/take";
import { actieVoorToets, doelVan } from "./regie/toetsen";
import { Monitoren } from "./deck/Monitoren";
import { TransportBalk } from "./deck/TransportBalk";
import { RegieWerkruimte } from "./deck/RegieWerkruimte";
import { ZaalCameraWerkruimte } from "./deck/ZaalCameraWerkruimte";
import { StemmingenWerkruimte } from "./deck/StemmingenWerkruimte";
import { CoalitieWerkruimte } from "./deck/CoalitieWerkruimte";
import { CijfersWerkruimte } from "./deck/CijfersWerkruimte";
import { StudioWerkruimte } from "./deck/StudioWerkruimte";
import { SysteemDrawer } from "./deck/SysteemDrawer";

type Pagina = "regie" | "zaal" | "stemmingen" | "coalitie" | "cijfers" | "studio";

const OPSLAG = "uitslagavond.toegang";
const HOOGTE_KEY = "uitslagavond.monitorHoogte";

function leesToegang(): Toegang | null {
  try {
    const t = JSON.parse(localStorage.getItem(OPSLAG) ?? "null") as Toegang | null;
    if (t?.sleutel && t.werkplek) return t;
  } catch {
    // negeer ongeldige JSON
  }
  const sleutel = sessionStorage.getItem("uitslagavond.sleutel") || localStorage.getItem("uitslagavond.sleutel") || "";
  const werkplek = localStorage.getItem("uitslagavond.werkplek") || "";
  return sleutel && werkplek ? { sleutel, werkplek } : null;
}

export function App({ popoutMonitors = false }: { popoutMonitors?: boolean }) {
  const [toegang, setToegang] = useState<Toegang | null>(leesToegang);

  function meldAan(t: Toegang) {
    localStorage.setItem(OPSLAG, JSON.stringify(t));
    setToegang(t);
  }

  function afmelden() {
    localStorage.removeItem(OPSLAG);
    sessionStorage.removeItem("uitslagavond.sleutel");
    localStorage.removeItem("uitslagavond.sleutel");
    setToegang(null);
  }

  if (!toegang) {
    return <AanmeldScherm standaardWerkplek="Regie links" opAanmelden={meldAan} />;
  }

  return (
    <RegieBron toegang={toegang} afmelden={afmelden}>
      {popoutMonitors ? <PopoutMonitoren /> : <DeckShell />}
    </RegieBron>
  );
}

function PopoutMonitoren() {
  return (
    <div className="popout-shell">
      <Monitoren hoogte={600} opHoogte={() => {}} popout />
    </div>
  );
}

function DeckShell() {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [pagina, setPagina] = useState<Pagina>("regie");
  const [systeemOpen, setSysteemOpen] = useState(false);
  const [monitorHoogte, setMonitorHoogte] = useState<number>(() => {
    const opgeslagen = Number(localStorage.getItem(HOOGTE_KEY));
    if (Number.isFinite(opgeslagen) && opgeslagen >= 190 && opgeslagen <= 700) return opgeslagen;
    return Math.round(Math.min(360, Math.max(240, window.innerHeight * 0.36)));
  });

  function wijzigHoogte(h: number) {
    setMonitorHoogte(h);
    localStorage.setItem(HOOGTE_KEY, String(h));
  }

  // Globale sneltoetsen (Spatie = Take, X = Take out, Z = Kleur uit, 1-9 = Presets, A-H/Q-U/J-L = Camera, Ctrl+Z = Undo)
  // Enter triggert NOOIT Take.
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const actie = actieVoorToets({
        key: e.key,
        shiftKey: e.shiftKey,
        ctrlKey: e.ctrlKey,
        metaKey: e.metaKey,
        altKey: e.altKey,
        doel: doelVan(e.target),
      });
      if (!actie) return;
      e.preventDefault();
      voerUit(regie, actie);
    };
    const updateTypt = () => {
      document.body.classList.toggle("typt", doelVan(document.activeElement) === "tekstveld");
    };
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("focusin", updateTypt);
    window.addEventListener("focusout", updateTypt);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("focusin", updateTypt);
      window.removeEventListener("focusout", updateTypt);
    };
  }, [regie]);

  const inStudio = pagina === "studio";

  return (
    <div className="deck-shell">
      {!inStudio && (
        <>
          <Monitoren hoogte={monitorHoogte} opHoogte={wijzigHoogte} />
          <TransportBalk opOpenSysteem={() => setSysteemOpen(true)} />
        </>
      )}

      <main className="deck-main">
        {pagina === "regie" && <RegieWerkruimte opGaNaarZaal={() => setPagina("zaal")} />}
        {pagina === "zaal" && <ZaalCameraWerkruimte />}
        {pagina === "stemmingen" && <StemmingenWerkruimte />}
        {pagina === "coalitie" && <CoalitieWerkruimte />}
        {pagina === "cijfers" && <CijfersWerkruimte />}
        {pagina === "studio" && <StudioWerkruimte opTerug={() => setPagina("regie")} />}
      </main>

      {/* DaVinci onderste paginabalk */}
      <nav className="page-bar" aria-label="Werkruimtes">
        <button
          type="button"
          className={`page-tab${pagina === "regie" ? " active" : ""}`}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => setPagina("regie")}
        >
          <Layers size={14} />
          <span>Regie</span>
          {stand && stand.presets.length > 0 && <span className="page-tab-badge">{stand.presets.length}</span>}
        </button>
        <button
          type="button"
          className={`page-tab${pagina === "zaal" ? " active" : ""}`}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => setPagina("zaal")}
        >
          <Camera size={14} />
          <span>Zaal & Camera</span>
        </button>
        <button
          type="button"
          className={`page-tab${pagina === "stemmingen" ? " active" : ""}`}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => setPagina("stemmingen")}
        >
          <Vote size={14} />
          <span>Stemmingen</span>
        </button>
        <button
          type="button"
          className={`page-tab${pagina === "coalitie" ? " active" : ""}`}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => setPagina("coalitie")}
        >
          <PieChart size={14} />
          <span>Coalitie</span>
        </button>
        <button
          type="button"
          className={`page-tab${pagina === "cijfers" ? " active" : ""}`}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => setPagina("cijfers")}
        >
          <BarChart3 size={14} />
          <span>Cijfers</span>
        </button>
        <button
          type="button"
          className={`page-tab${pagina === "studio" ? " active" : ""}`}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => setPagina("studio")}
        >
          <Palette size={14} />
          <span>Studio</span>
        </button>
      </nav>

      {systeemOpen && <SysteemDrawer opSluit={() => setSysteemOpen(false)} />}
    </div>
  );
}

function AanmeldScherm({
  standaardWerkplek,
  opAanmelden,
}: {
  standaardWerkplek: string;
  opAanmelden: (t: Toegang) => void;
}) {
  const [werkplek, setWerkplek] = useState(standaardWerkplek);
  const [sleutel, setSleutel] = useState("");

  function verstuur(e: React.FormEvent) {
    e.preventDefault();
    if (!werkplek.trim() || !sleutel.trim()) return;
    opAanmelden({ werkplek: werkplek.trim(), sleutel: sleutel.trim() });
  }

  return (
    <div className="login-screen">
      <form className="login-card" onSubmit={verstuur}>
        <div>
          <h1 style={{ fontSize: 18, fontWeight: 700 }}>uitslagavond-tv</h1>
          <p style={{ fontSize: 12.5, color: "var(--text-2)", marginTop: 3 }}>Regie-werkplek aanmelden</p>
        </div>
        <div className="field">
          <label htmlFor="login-werkplek">Werkplek</label>
          <input
            id="login-werkplek"
            className="input"
            value={werkplek}
            onChange={(e) => setWerkplek(e.target.value)}
            placeholder="Bijv. Regie links"
            autoFocus
          />
        </div>
        <div className="field">
          <label htmlFor="login-sleutel">Gedeelde sleutel</label>
          <input
            id="login-sleutel"
            className="input"
            type="password"
            value={sleutel}
            onChange={(e) => setSleutel(e.target.value)}
            placeholder="Sleutel van de server"
          />
        </div>
        <button type="submit" className="btn lg amber" disabled={!werkplek.trim() || !sleutel.trim()}>
          Verbinden
        </button>
      </form>
    </div>
  );
}
