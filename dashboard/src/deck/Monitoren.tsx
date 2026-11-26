// Live 720p Preview & Live monitors (DaVinci Deck):
// - Onmiskenbaar onderscheid: links PREVIEW (VOLGENDE, groen) vs. rechts ON AIR / LIVE (rood)
// - Duidelijke statuspil in Preview: "KLAAR VOOR TAKE" (groen) vs. "Gelijk aan Live" (gedimd)
// - Laagchips onder Preview markeren precies wat er verandert bij Take ("Camera loopt door" vs "→ Q")
// - 1-klik vergroten, pop-out (/monitors), ▶ Test rit en versleepbare hoogte
import { useEffect, useRef, useState } from "react";
import { Camera, ExternalLink, Layers, Maximize2, Minimize2, Play, Sparkles } from "lucide-react";
import { graphicTekst, sjabloonNaam, standaardStandpunt } from "../graphic";
import { CAMERA, heeftIetsKlaar, klaarStaatInBeeld } from "../regie/take";
import { useRegie } from "../regie/regie";
import type { Stand } from "../typen/Stand";
import { Schets } from "../presets/Schets";

const BLIJF = "blijf";

function cameraNaam(stand: Stand, letter: string | undefined): { naam: string; isRit: boolean; letter: string } {
  const l = letter || stand.camera.standpunt;
  if (l === BLIJF) return { naam: "Camera loopt door", isRit: false, letter: l };
  if (l === "aanschuiven" || l.startsWith("aanschuiven-")) return { naam: "Aanschuiven", isRit: false, letter: l };
  if (l.startsWith("stoel-")) return { naam: "Shot op Stoel", isRit: false, letter: l };
  const sp = stand.standpunten.find((p) => p.letter === l);
  if (sp) return { naam: `${sp.toets ?? sp.letter} · ${sp.naam}`, isRit: false, letter: l };
  const rit = stand.cameraritten.find((r) => r.letter === l);
  if (rit) return { naam: `${rit.toets ?? rit.letter} · ${rit.naam}`, isRit: true, letter: l };
  return { naam: l, isRit: false, letter: l };
}

function stoelenLabel(stand: Stand, soort: "preview" | "live"): string | null {
  if (soort === "preview") {
    if (stand.klaar_stoelbeeld) {
      if (stand.klaar_stoelbeeld.bron.soort === "Uit") return "Stoelen uit";
      return stand.klaar_stoelbeeld.label?.trim() || "Stoelbeeld";
    }
    if (stand.klaar && "cijfers" in stand.klaar.graphic && stand.klaar.graphic.cijfers) {
      return stand.klaar.graphic.cijfers.set.label;
    }
    if (stand.klaar && "uitslag" in stand.klaar.graphic) {
      return "Stemming";
    }
    return null;
  }
  if (stand.stoelkleuren.length === 0) return null;
  if (stand.ether_stoelbeeld?.label?.trim()) return stand.ether_stoelbeeld.label.trim();
  if (stand.ether && "cijfers" in stand.ether.graphic && stand.ether.graphic.cijfers) {
    return stand.ether.graphic.cijfers.set.label;
  }
  return "Gekleurd";
}

interface MonitorenProps {
  hoogte: number;
  opHoogte: (h: number) => void;
  popout?: boolean;
}

export function Monitoren({ hoogte, opHoogte, popout = false }: MonitorenProps) {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [focus, setFocus] = useState<"geen" | "Voorbeeld" | "Ether">("geen");
  const [sleept, setSleept] = useState(false);
  const [fadeKey, setFadeKey] = useState(0);
  const laatsteFade = useRef<number>(0);

  useEffect(() => {
    const sinds = stand?.camera.overvloei_sinds ?? 0;
    if (sinds > 0 && laatsteFade.current > 0 && sinds !== laatsteFade.current) {
      setFadeKey((k) => k + 1);
    }
    if (sinds > 0) laatsteFade.current = sinds;
  }, [stand?.camera.overvloei_sinds]);

  function startSleep(e: React.PointerEvent) {
    e.preventDefault();
    const startY = e.clientY;
    const startH = hoogte;
    setSleept(true);
    const beweeg = (ev: PointerEvent) => {
      const maxH = Math.round(window.innerHeight * 0.68);
      const nieuw = Math.max(190, Math.min(maxH, startH + (ev.clientY - startY)));
      opHoogte(nieuw);
    };
    const klaar = () => {
      setSleept(false);
      window.removeEventListener("pointermove", beweeg);
      window.removeEventListener("pointerup", klaar);
    };
    window.addEventListener("pointermove", beweeg);
    window.addEventListener("pointerup", klaar);
  }

  function openPopout() {
    window.open("/monitors", "uitslagavond-monitors", "width=1360,height=460,menubar=no,toolbar=no,location=no");
  }

  const klaarVoorTake = heeftIetsKlaar(stand);
  const graphicVerandert = !!(stand?.klaar && !klaarStaatInBeeld(stand));
  const stoelenVeranderen = !!stand?.klaar_stoelbeeld;

  const pCamLetter =
    stand?.camera.klaar_standpunt ??
    stand?.klaar?.standpunt ??
    (stand?.klaar ? standaardStandpunt(stand.klaar.graphic) : BLIJF);
  const cameraVerandert =
    !!stand &&
    ((!!stand.camera.klaar_standpunt && stand.camera.klaar_standpunt !== BLIJF) ||
      (graphicVerandert && pCamLetter !== BLIJF && pCamLetter !== stand.camera.standpunt));

  const pCam = stand ? cameraNaam(stand, pCamLetter) : null;
  const lCam = stand ? (stand.camera.vrij ? { naam: "Vrije camera", isRit: false, letter: "V" } : cameraNaam(stand, stand.camera.standpunt)) : null;

  const pStoelen = stand ? stoelenLabel(stand, "preview") : null;
  const lStoelen = stand ? stoelenLabel(stand, "live") : null;

  const pGraphic = stand?.klaar ? `${sjabloonNaam(stand.klaar.graphic, stand.sjablonen)} · ${graphicTekst(stand.klaar.graphic)}` : null;
  const lGraphic = stand?.ether ? `${sjabloonNaam(stand.ether.graphic, stand.sjablonen)} · ${graphicTekst(stand.ether.graphic)}` : null;
  const ietsOpZender = !!(stand?.ether || (stand?.stoelkleuren.length ?? 0) > 0);

  const toonPreview = focus === "geen" || focus === "Voorbeeld";
  const toonLive = focus === "geen" || focus === "Ether";
  const fadeMs = stand?.camera.overvloei_ms ?? 600;

  return (
    <>
      <section
        className="monitoren-well"
        style={popout ? { flex: 1, height: "100%" } : { height: `${hoogte}px` }}
        aria-label="Preview en Live monitoren"
      >
        <div className={`monitoren-grid${focus !== "geen" ? " focus-one" : ""}`}>
          {toonPreview && (
            <div className={`monitor-card preview${klaarVoorTake ? "" : " synced"}`}>
              <div className="monitor-head">
                <div className="monitor-tally preview">
                  <span className={`dot ${klaarVoorTake ? "ok" : ""}`} />
                  <span>PREVIEW · VOLGENDE</span>
                  <span className={`chip ${klaarVoorTake ? "preview" : ""}`} style={{ height: 19, fontSize: 10.5, textTransform: "none", letterSpacing: 0 }}>
                    {klaarVoorTake ? "Klaar voor Take" : "Gelijk aan Live"}
                  </span>
                  {stand?.camera.oefenen && <span className="chip amber" style={{ height: 19, fontSize: 10.5 }}>Oefenen</span>}
                  {stand?.camera.opname && <span className="chip live" style={{ height: 19, fontSize: 10.5 }}>Opname</span>}
                </div>
                <div className="monitor-actions">
                  {pCam?.isRit && (
                    <button
                      type="button"
                      className="btn sm preview"
                      onMouseDown={(e) => e.preventDefault()}
                      onClick={() => void regie.stuur({ soort: "ProbeerInVoorbeeld", letter: pCam.letter }, CAMERA)}
                      title="Camerarit proefdraaien in Preview (zonder Live te raken)"
                    >
                      <Play size={12} />
                      Test rit
                    </button>
                  )}
                  <button
                    type="button"
                    className="btn icon sm ghost"
                    onMouseDown={(e) => e.preventDefault()}
                    onClick={() => setFocus((f) => (f === "Voorbeeld" ? "geen" : "Voorbeeld"))}
                    title={focus === "Voorbeeld" ? "Beide monitoren tonen" : "Preview vergroten"}
                  >
                    {focus === "Voorbeeld" ? <Minimize2 size={13} /> : <Maximize2 size={13} />}
                  </button>
                  {!popout && (
                    <button
                      type="button"
                      className="btn icon sm ghost"
                      onMouseDown={(e) => e.preventDefault()}
                      onClick={openPopout}
                      title="Monitoren openen in los venster"
                    >
                      <ExternalLink size={13} />
                    </button>
                  )}
                </div>
              </div>

              <div className="monitor-frame">
                {regie.beelden.Voorbeeld ? (
                  <img className="monitor-img" src={regie.beelden.Voorbeeld} alt="Preview 720p" draggable={false} />
                ) : stand?.klaar ? (
                  <div className="monitor-schets-wrap">
                    <Schets graphic={stand.klaar.graphic} kleuren={stand.stijl.kleuren} />
                  </div>
                ) : (
                  <div className="monitor-placeholder">
                    <span>Geen Preview-renderer verbonden</span>
                  </div>
                )}
              </div>

              <div className="monitor-layers">
                <span className={`layer-chip${cameraVerandert ? " staged" : " empty"}`} title={pCam?.naam}>
                  <Camera size={12} />
                  <span className="layer-val">{cameraVerandert ? `→ ${pCam?.naam}` : "Camera loopt door"}</span>
                </span>
                <span className={`layer-chip${stoelenVeranderen ? " staged" : pStoelen ? "" : " empty"}`} title={pStoelen ?? "Stoelen blijven gelijk"}>
                  <Sparkles size={12} />
                  <span className="layer-val">{pStoelen ?? "Stoelen blijven"}</span>
                </span>
                <span className={`layer-chip${graphicVerandert ? " staged" : " empty"}`} title={pGraphic ?? "Geen nieuwe Graphic"}>
                  <Layers size={12} />
                  <span className="layer-val">
                    {graphicVerandert ? pGraphic : pGraphic ? "Zelfde als Live" : "Geen Graphic"}
                  </span>
                </span>
              </div>
            </div>
          )}

          {toonLive && (
            <div className={`monitor-card live${ietsOpZender ? " onair" : ""}`}>
              <div className="monitor-head">
                <div className="monitor-tally live">
                  <span className="dot live" />
                  <span>LIVE · OP ZENDER</span>
                  <span className={`chip ${ietsOpZender ? "live" : ""}`} style={{ height: 19, fontSize: 10.5, textTransform: "none", letterSpacing: 0 }}>
                    {stand?.ether ? "Graphic actief" : (stand?.stoelkleuren.length ?? 0) > 0 ? "Stoelen actief" : "Alleen Zaal"}
                  </span>
                </div>
                <div className="monitor-actions">
                  <span className="mono" style={{ fontSize: 10.5, color: "var(--text-3)", marginRight: 2 }}>
                    {regie.beelden.Ether ? "720p" : "Schets"}
                  </span>
                  <button
                    type="button"
                    className="btn icon sm ghost"
                    onMouseDown={(e) => e.preventDefault()}
                    onClick={() => setFocus((f) => (f === "Ether" ? "geen" : "Ether"))}
                    title={focus === "Ether" ? "Beide monitoren tonen" : "Live vergroten"}
                  >
                    {focus === "Ether" ? <Minimize2 size={13} /> : <Maximize2 size={13} />}
                  </button>
                </div>
              </div>

              <div className="monitor-frame">
                {regie.beelden.Ether ? (
                  <img className="monitor-img" src={regie.beelden.Ether} alt="Live 720p" draggable={false} />
                ) : stand?.ether ? (
                  <div className="monitor-schets-wrap">
                    <Schets graphic={stand.ether.graphic} kleuren={stand.ether.stijl.kleuren} />
                  </div>
                ) : (
                  <div className="monitor-placeholder">
                    <span>Zaal live (geen Graphic)</span>
                  </div>
                )}
                {fadeKey > 0 && (
                  <div
                    key={fadeKey}
                    className="monitor-fade-flash active"
                    style={{ "--fade-ms": `${fadeMs}ms` } as React.CSSProperties}
                  />
                )}
              </div>

              <div className="monitor-layers">
                <span className="layer-chip onair" title={lCam?.naam}>
                  <Camera size={12} />
                  <span className="layer-val">{lCam?.naam ?? "—"}</span>
                </span>
                <span className={`layer-chip${lStoelen ? " onair" : " empty"}`} title={lStoelen ?? "Stoelen uit"}>
                  <Sparkles size={12} />
                  <span className="layer-val">{lStoelen ?? "Stoelen uit"}</span>
                </span>
                <span className={`layer-chip${lGraphic ? " onair" : " empty"}`} title={lGraphic ?? "Geen Graphic live"}>
                  <Layers size={12} />
                  <span className="layer-val">{lGraphic ?? "Geen Graphic"}</span>
                </span>
              </div>
            </div>
          )}
        </div>
      </section>

      {!popout && (
        <div
          className={`deck-splitter${sleept ? " dragging" : ""}`}
          onPointerDown={startSleep}
          role="separator"
          aria-orientation="horizontal"
          aria-label="Hoogte monitoren aanpassen"
          title="Sleep om de grootte van Preview en Live aan te passen"
        />
      )}
    </>
  );
}
