// Studio-werkruimte (DaVinci Deck):
// Klap de dubbele monitoren in tot een smalle 38px rode LIVE-strip bovenaan zodat de volledige
// hoogte beschikbaar is voor:
//   1. Sjablonen: intuïtieve drag-and-snap 16:9 canvas over het echte 3D-Zaalbeeld, Title-safe,
//      magnetisch uitlijnen, snelle anker-presets, ▶ Test animatie, Lagenlijst, Ctrl+Z/Ctrl+Shift+Z,
//      en een rustige 4-secties Inspector (Positie & Maat standaard ingeklapt).
//   2. Stijl: kleuren, snelheid, kleurovergang & Kleurkracht met live Conceptbeeld.
//   3. Camera-werkbank: Standpunten en Cameraritten fijnafstellen, hernoemen, toetsen toewijzen.
import { useEffect, useMemo, useRef, useState } from "react";
import {
  ArrowDown,
  ArrowLeft,
  ArrowUp,
  Camera,
  ChevronDown,
  Copy,
  Grid,
  Layers,
  Palette,
  Play,
  Plus,
  Redo2,
  RotateCcw,
  Save,
  Square,
  Trash2,
  Undo2,
} from "lucide-react";
import { graphicTekst, sjabloonNaam } from "../graphic";
import { useRegie } from "../regie/regie";
import { CAMERA, kan, voerUit } from "../regie/take";
import {
  ACHTERGRONDEN,
  BLOKNAMEN,
  blokTekst,
  ERIN,
  ERUIT,
  fout as foutSjabloon,
  GEWONE_BLOKKEN,
  gewijzigdSjabloon,
  lijst as sjablonenLijst,
  nieuwBlok,
  nieuwSjabloon,
  STOELEN as STOEL_OPTIES,
} from "../vormgeving/sjablonen";
import type { Achtergrond } from "../typen/Achtergrond";
import type { Blok } from "../typen/Blok";
import type { Bloksoort } from "../typen/Bloksoort";
import type { Erin } from "../typen/Erin";
import type { Eruit } from "../typen/Eruit";
import type { Kleurkeuze } from "../typen/Kleurkeuze";
import type { OvergangSoort } from "../typen/OvergangSoort";
import type { Sjabloon } from "../typen/Sjabloon";
import type { Stijl } from "../typen/Stijl";
import type { Stoelen } from "../typen/Stoelen";
import { ConceptCanvas } from "./ConceptCanvas";

const BIJ = "studio";
const W = 1920;
const H = 1080;
const SNAP = 18; // pixels in 1920x1080 ruimte

export function StudioWerkruimte({ opTerug }: { opTerug: () => void }) {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [sub, setSub] = useState<"sjablonen" | "stijl" | "camera">("sjablonen");

  if (!stand) return null;

  const liveTekst = stand.ether
    ? `${sjabloonNaam(stand.ether.graphic, stand.sjablonen)} · ${graphicTekst(stand.ether.graphic)}`
    : stand.stoelkleuren.length > 0
      ? "Zaal gekleurd (geen Graphic)"
      : "Zaal vrij";

  return (
    <div className="studio-shell">
      {/* Smalle 38px rode LIVE-strip: noodrem blijft altijd onder handbereik */}
      <header className="studio-live-strip">
        <div className="studio-live-left">
          <button type="button" className="btn sm ghost" onClick={opTerug}>
            <ArrowLeft size={13} /> Terug naar Regie
          </button>
          <span className="studio-live-tag">
            <span className="dot" style={{ background: "#fff" }} /> LIVE
          </span>
          <span style={{ fontSize: 12.5, fontWeight: 500, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {liveTekst}
          </span>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <button
            type="button"
            className="btn sm danger-outline"
            disabled={!kan("UitBeeld", stand)}
            onClick={() => voerUit(regie, "UitBeeld")}
          >
            <Square size={11} /> Take out <kbd className="key">X</kbd>
          </button>
        </div>
      </header>

      {/* Subnavigatie: Sjablonen / Stijl / Camera-werkbank */}
      <div className="studio-subbar">
        <div className="seg">
          <button type="button" className={sub === "sjablonen" ? "active amber" : ""} onClick={() => setSub("sjablonen")}>
            <Layers size={13} /> Sjablonen
          </button>
          <button type="button" className={sub === "stijl" ? "active amber" : ""} onClick={() => setSub("stijl")}>
            <Palette size={13} /> Stijl
          </button>
          <button type="button" className={sub === "camera" ? "active amber" : ""} onClick={() => setSub("camera")}>
            <Camera size={13} /> Camera-werkbank
          </button>
        </div>
        {regie.melding?.bij === BIJ && <span className="chip live">{regie.melding.tekst}</span>}
      </div>

      {sub === "sjablonen" && <SjabloonEditor />}
      {sub === "stijl" && <StijlEditor />}
      {sub === "camera" && <CameraWerkbank />}
    </div>
  );
}

/* ==========================================================================
   1. Eenvoudige Drag-and-Snap Sjabloon Editor (over het echte 3D-Zaalbeeld)
   ========================================================================== */

function SjabloonEditor() {
  const regie = useRegie();
  const stand = regie.stand!.stand;
  const rijen = useMemo(() => sjablonenLijst(stand.sjablonen), [stand.sjablonen]);

  const [gekozenId, setGekozenId] = useState<string>(rijen[0]?.sjabloon.id ?? "titelkaart");
  const opgeslagen = useMemo(() => rijen.find((r) => r.sjabloon.id === gekozenId) ?? rijen[0], [rijen, gekozenId]);

  const [concept, setConcept] = useState<Sjabloon>(() => structuredClone(opgeslagen.sjabloon));
  const [geschiedenis, setGeschiedenis] = useState<Sjabloon[]>([]);
  const [toekomst, setToekomst] = useState<Sjabloon[]>([]);
  const [gekozenBlok, setGekozenBlok] = useState<number>(0);
  const [toonSafe, setToonSafe] = useState<boolean>(true);
  const [testAnimatie, setTestAnimatie] = useState<boolean>(false);
  const [voegToeOpen, setVoegToeOpen] = useState<boolean>(false);
  const [zaalUrl, setZaalUrl] = useState<string | null>(null);
  const [snapLijnen, setSnapLijnen] = useState<{ x: number | null; y: number | null }>({ x: null, y: null });

  const canvasRef = useRef<HTMLDivElement>(null);

  // Wissel van Sjabloon -> laad opgeslagen versie
  useEffect(() => {
    setConcept(structuredClone(opgeslagen.sjabloon));
    setGeschiedenis([]);
    setToekomst([]);
    setGekozenBlok(0);
  }, [opgeslagen.sjabloon.id]);

  // Haal echt 720p 3D-Zaalbeeld op als canvas-achtergrond
  useEffect(() => {
    let actief = true;
    let gemaakt: string | null = null;
    void regie.haalZaalbeeld("ether").then((u) => {
      if (!actief) {
        if (u) URL.revokeObjectURL(u);
        return;
      }
      gemaakt = u;
      setZaalUrl(u);
    });
    return () => {
      actief = false;
      if (gemaakt) URL.revokeObjectURL(gemaakt);
    };
  }, [regie.beelden.Ether !== null]);

  // Ctrl+Z / Ctrl+Shift+Z binnen de Sjabloon-editor
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement)?.matches("input, textarea, select")) return;
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
        e.preventDefault();
        e.stopPropagation();
        if (e.shiftKey) herhaal();
        else ongedaan();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [geschiedenis, toekomst, concept]);

  function pasAan(fn: (s: Sjabloon) => Sjabloon) {
    setGeschiedenis((g) => [...g.slice(-29), structuredClone(concept)]);
    setToekomst([]);
    setConcept((oud) => fn(structuredClone(oud)));
  }

  function ongedaan() {
    if (geschiedenis.length === 0) return;
    const vorig = geschiedenis[geschiedenis.length - 1];
    setToekomst((t) => [structuredClone(concept), ...t]);
    setGeschiedenis((g) => g.slice(0, -1));
    setConcept(vorig);
  }

  function herhaal() {
    if (toekomst.length === 0) return;
    const volgend = toekomst[0];
    setGeschiedenis((g) => [...g, structuredClone(concept)]);
    setToekomst((t) => t.slice(1));
    setConcept(volgend);
  }

  function wijzigBlok(index: number, fn: (b: Blok) => void) {
    pasAan((s) => {
      if (s.blokken[index]) fn(s.blokken[index]);
      return s;
    });
  }

  function maakNieuw() {
    const bestaand = rijen.map((r) => r.sjabloon.id);
    const vers = nieuwSjabloon(bestaand);
    setConcept(vers);
    setGekozenId(vers.id);
    setGekozenBlok(0);
    setGeschiedenis([]);
    setToekomst([]);
  }

  function dupliceerHuidig() {
    const bestaand = rijen.map((r) => r.sjabloon.id);
    const basis = nieuwSjabloon(bestaand);
    const kopie: Sjabloon = {
      ...structuredClone(concept),
      id: basis.id,
      naam: `${concept.naam} (kopie)`,
      groep: basis.groep,
    };
    setConcept(kopie);
    setGekozenId(kopie.id);
    setGekozenBlok(0);
    setGeschiedenis([]);
    setToekomst([]);
  }

  function pasAnkerToe(anker: "linksonder" | "onderin" | "rechts" | "midden") {
    const b = concept.blokken[gekozenBlok];
    if (!b) return;
    wijzigBlok(gekozenBlok, (blok) => {
      if (anker === "linksonder") {
        blok.plek = { x: 96, y: Math.max(54, 1026 - blok.plek.hoogte), breedte: Math.min(blok.plek.breedte, 1100), hoogte: blok.plek.hoogte };
      } else if (anker === "onderin") {
        blok.plek = { x: 96, y: Math.max(54, 1026 - blok.plek.hoogte), breedte: 1728, hoogte: blok.plek.hoogte };
      } else if (anker === "rechts") {
        blok.plek = { x: 1260, y: 96, breedte: 564, hoogte: 888 };
      } else {
        blok.plek = {
          x: Math.round((W - blok.plek.breedte) / 2),
          y: Math.round((H - blok.plek.hoogte) / 2),
          breedte: blok.plek.breedte,
          hoogte: blok.plek.hoogte,
        };
      }
    });
  }

  function startSlepen(e: React.PointerEvent, index: number, modus: "move" | "se" | "sw" | "ne" | "nw") {
    e.stopPropagation();
    e.preventDefault();
    setGekozenBlok(index);
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    const schaalX = W / rect.width;
    const schaalY = H / rect.height;
    const startX = e.clientX;
    const startY = e.clientY;
    const begin = { ...concept.blokken[index].plek };
    const snapSnapshot = structuredClone(concept);
    let gewijzigd = false;

    const doelX = [96, 960, 1824];
    const doelY = [54, 540, 1026];
    concept.blokken.forEach((b, i) => {
      if (i !== index) {
        doelX.push(b.plek.x, b.plek.x + b.plek.breedte);
        doelY.push(b.plek.y, b.plek.y + b.plek.hoogte);
      }
    });

    const snap = (val: number, lijst: number[]): [number, number | null] => {
      for (const d of lijst) {
        if (Math.abs(val - d) <= SNAP) return [d, d];
      }
      return [Math.round(val), null];
    };

    const beweeg = (ev: PointerEvent) => {
      const dx = (ev.clientX - startX) * schaalX;
      const dy = (ev.clientY - startY) * schaalY;
      gewijzigd = true;
      let nx = begin.x;
      let ny = begin.y;
      let nw = begin.breedte;
      let nh = begin.hoogte;
      let sx: number | null = null;
      let sy: number | null = null;

      if (modus === "move") {
        const [x1, lx] = snap(begin.x + dx, doelX);
        const [y1, ly] = snap(begin.y + dy, doelY);
        nx = Math.max(0, Math.min(W - nw, x1));
        ny = Math.max(0, Math.min(H - nh, y1));
        sx = lx;
        sy = ly;
      } else {
        if (modus.includes("e")) nw = Math.max(20, Math.min(W - nx, Math.round(begin.breedte + dx)));
        if (modus.includes("s")) nh = Math.max(10, Math.min(H - ny, Math.round(begin.hoogte + dy)));
        if (modus.includes("w")) {
          const nieuwX = Math.max(0, Math.min(begin.x + begin.breedte - 20, Math.round(begin.x + dx)));
          nw = begin.breedte + (begin.x - nieuwX);
          nx = nieuwX;
        }
        if (modus.includes("n")) {
          const nieuwY = Math.max(0, Math.min(begin.y + begin.hoogte - 10, Math.round(begin.y + dy)));
          nh = begin.hoogte + (begin.y - nieuwY);
          ny = nieuwY;
        }
      }
      setSnapLijnen({ x: sx, y: sy });
      setConcept((oud) => {
        const kopie = structuredClone(oud);
        if (kopie.blokken[index]) kopie.blokken[index].plek = { x: nx, y: ny, breedte: nw, hoogte: nh };
        return kopie;
      });
    };

    const klaar = () => {
      setSnapLijnen({ x: null, y: null });
      if (gewijzigd) {
        setGeschiedenis((g) => [...g.slice(-29), snapSnapshot]);
        setToekomst([]);
      }
      window.removeEventListener("pointermove", beweeg);
      window.removeEventListener("pointerup", klaar);
    };

    window.addEventListener("pointermove", beweeg);
    window.addEventListener("pointerup", klaar);
  }

  function speelAnimatieProef() {
    setTestAnimatie(false);
    window.setTimeout(() => setTestAnimatie(true), 20);
    window.setTimeout(() => setTestAnimatie(false), 1200);
  }

  const validatieFout = foutSjabloon(concept);
  const isNieuwEigen = !rijen.some((r) => r.sjabloon.id === concept.id);
  const gewijzigd = isNieuwEigen || gewijzigdSjabloon(concept, opgeslagen.sjabloon);
  const actiefBlok = concept.blokken[gekozenBlok];
  const bg = zaalUrl ?? regie.beelden.Ether ?? regie.beelden.Voorbeeld;

  return (
    <div className="studio-editor-grid">
      {/* LINKS: Sjablonenkiezer + Lagenlijst */}
      <aside className="studio-left">
        <div className="pane-head">
          <span className="pane-title">Sjabloon</span>
          <div style={{ display: "flex", gap: 4 }}>
            <button type="button" className="btn icon sm" onClick={dupliceerHuidig} title="Dupliceer huidig Sjabloon">
              <Copy size={12} />
            </button>
            <button type="button" className="btn sm amber" onClick={maakNieuw}>
              <Plus size={12} /> Nieuw
            </button>
          </div>
        </div>

        <div style={{ padding: "8px 8px 6px", borderBottom: "1px solid var(--hairline)" }}>
          <select
            className="select sm"
            value={isNieuwEigen ? "__nieuw__" : opgeslagen.sjabloon.id}
            onChange={(e) => {
              if (e.target.value !== "__nieuw__") setGekozenId(e.target.value);
            }}
          >
            {isNieuwEigen && <option value="__nieuw__">{concept.naam} (nieuw)</option>}
            {rijen.map((r) => (
              <option key={r.sjabloon.id} value={r.sjabloon.id}>
                {r.sjabloon.naam} {r.soort === "aangepast" ? "· aangepast" : r.soort === "eigen" ? "· eigen" : ""}
              </option>
            ))}
          </select>
        </div>

        <div className="pane-head" style={{ background: "transparent" }}>
          <span className="pane-title" style={{ fontSize: 12 }}>Lagen ({concept.blokken.length})</span>
          <div className="popover-anchor">
            <button type="button" className="btn sm" onClick={() => setVoegToeOpen((o) => !o)}>
              <Plus size={12} /> Blok
            </button>
            {voegToeOpen && (
              <div className="popover-menu left">
                {GEWONE_BLOKKEN.map((g) => (
                  <button
                    key={g.soort}
                    type="button"
                    className="popover-item"
                    onClick={() => {
                      const nieuw = nieuwBlok(g.soort);
                      pasAan((s) => {
                        s.blokken.push(nieuw);
                        return s;
                      });
                      setGekozenBlok(concept.blokken.length);
                      setVoegToeOpen(false);
                    }}
                  >
                    {g.naam}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="work-list">
          {concept.blokken.map((b, i) => (
            <div
              key={i}
              onClick={() => setGekozenBlok(i)}
              className={`work-list-item${gekozenBlok === i ? " active" : ""}`}
              style={{ cursor: "pointer", flexDirection: "row", alignItems: "center", justifyContent: "space-between", padding: "6px 8px" }}
            >
              <div style={{ minWidth: 0 }}>
                <div style={{ fontSize: 12, fontWeight: 600 }}>{BLOKNAMEN[b.soort.blok]}</div>
                <div style={{ fontSize: 11, color: "var(--text-3)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {blokTekst(b) || `${b.plek.breedte}×${b.plek.hoogte}`}
                </div>
              </div>
              <div style={{ display: "flex", gap: 2, flexShrink: 0 }} onClick={(e) => e.stopPropagation()}>
                <button
                  type="button"
                  className="btn icon sm ghost"
                  disabled={i === 0}
                  onClick={() => {
                    pasAan((s) => {
                      [s.blokken[i - 1], s.blokken[i]] = [s.blokken[i], s.blokken[i - 1]];
                      return s;
                    });
                    setGekozenBlok(i - 1);
                  }}
                >
                  <ArrowUp size={11} />
                </button>
                <button
                  type="button"
                  className="btn icon sm ghost"
                  disabled={i === concept.blokken.length - 1}
                  onClick={() => {
                    pasAan((s) => {
                      [s.blokken[i + 1], s.blokken[i]] = [s.blokken[i], s.blokken[i + 1]];
                      return s;
                    });
                    setGekozenBlok(i + 1);
                  }}
                >
                  <ArrowDown size={11} />
                </button>
                <button
                  type="button"
                  className="btn icon sm ghost"
                  onClick={() => {
                    pasAan((s) => {
                      s.blokken.splice(i, 1);
                      return s;
                    });
                    setGekozenBlok(Math.max(0, i - 1));
                  }}
                  title="Verwijder blok"
                >
                  <Trash2 size={11} />
                </button>
              </div>
            </div>
          ))}
        </div>
      </aside>

      {/* MIDDEN: 16:9 Drag-and-Snap Canvas over het echte 3D-Zaalbeeld */}
      <section className="studio-center">
        <div className="studio-canvas-toolbar">
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <button type="button" className="btn icon sm" disabled={geschiedenis.length === 0} onClick={ongedaan} title="Ongedaan maken (Ctrl+Z)">
              <Undo2 size={13} />
            </button>
            <button type="button" className="btn icon sm" disabled={toekomst.length === 0} onClick={herhaal} title="Opnieuw (Ctrl+Shift+Z)">
              <Redo2 size={13} />
            </button>
            <button type="button" className={`btn sm${toonSafe ? " active" : ""}`} onClick={() => setToonSafe((v) => !v)}>
              <Grid size={12} /> Title-safe
            </button>
            <button type="button" className="btn sm" onClick={speelAnimatieProef}>
              <Play size={12} /> Test animatie
            </button>
          </div>

          {/* Snelle anker-presets voor geselecteerd blok */}
          <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <span style={{ fontSize: 11, color: "var(--text-3)", marginRight: 2 }}>Plaats:</span>
            <button type="button" className="btn sm ghost" disabled={!actiefBlok} onClick={() => pasAnkerToe("linksonder")}>Linksonder</button>
            <button type="button" className="btn sm ghost" disabled={!actiefBlok} onClick={() => pasAnkerToe("onderin")}>Onderin breed</button>
            <button type="button" className="btn sm ghost" disabled={!actiefBlok} onClick={() => pasAnkerToe("rechts")}>Zijpaneel</button>
            <button type="button" className="btn sm ghost" disabled={!actiefBlok} onClick={() => pasAnkerToe("midden")}>Midden</button>
          </div>
        </div>

        <div className="studio-canvas-stage">
          <div
            ref={canvasRef}
            className="studio-canvas-frame"
            style={{
              background:
                concept.achtergrond.soort === "Kleur"
                  ? kleurHex(concept.achtergrond.kleur, stand.stijl)
                  : "#0e131e",
            }}
          >
            {/* Echt 3D-Zaalbeeld op achtergrond (geen nep-gradiënt!) */}
            {(concept.achtergrond.soort === "Zaal" || concept.achtergrond.soort === "ZaalDonker") && bg && (
              <img
                src={bg}
                alt=""
                draggable={false}
                style={{
                  position: "absolute",
                  inset: 0,
                  width: "100%",
                  height: "100%",
                  objectFit: "cover",
                  filter: concept.achtergrond.soort === "ZaalDonker" ? "blur(6px) brightness(0.45) saturate(0.25)" : undefined,
                  pointerEvents: "none",
                }}
              />
            )}

            {toonSafe && <div className="studio-safe-box" />}
            {snapLijnen.x !== null && <div className="studio-snap-line-x" style={{ left: `${(snapLijnen.x / W) * 100}%` }} />}
            {snapLijnen.y !== null && <div className="studio-snap-line-y" style={{ top: `${(snapLijnen.y / H) * 100}%` }} />}

            {concept.blokken.map((b, i) => {
              const sel = i === gekozenBlok;
              const k = kleurHex(b.kleur, stand.stijl);
              const isVlak = b.soort.blok === "Vlak";
              const isTekst = b.soort.blok === "Tekst" || b.soort.blok === "Getal";
              const isLijn = b.soort.blok === "Lijn";
              return (
                <div
                  key={i}
                  className={`studio-block${sel ? " selected" : ""}`}
                  onPointerDown={(e) => startSlepen(e, i, "move")}
                  style={{
                    left: `${(b.plek.x / W) * 100}%`,
                    top: `${(b.plek.y / H) * 100}%`,
                    width: `${(b.plek.breedte / W) * 100}%`,
                    height: `${(b.plek.hoogte / H) * 100}%`,
                    background: isVlak ? k : isLijn ? k : "rgba(255,255,255,0.06)",
                    opacity: testAnimatie ? 0.25 : b.soort.blok === "Vlak" ? b.soort.dekking / 100 : 1,
                    borderRadius: b.soort.blok === "Vlak" ? `${(b.soort.hoek / 1080) * 100}%` : 2,
                    color: isTekst ? k : "#ebedf2",
                    fontSize: isTekst ? "clamp(10px, 1.5vw, 22px)" : 11,
                    fontWeight: 700,
                    transition: testAnimatie ? "opacity 350ms ease, transform 350ms ease" : undefined,
                    transform: testAnimatie ? "translateY(10px)" : undefined,
                  }}
                >
                  <div className="studio-block-inner" style={{ padding: isTekst ? "0 6px" : 0 }}>
                    {!isVlak && !isLijn && (
                      <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {blokTekst(b) || BLOKNAMEN[b.soort.blok]}
                      </span>
                    )}
                  </div>

                  {sel && (
                    <>
                      <div className="studio-handle" style={{ left: -5, top: -5, cursor: "nwse-resize" }} onPointerDown={(e) => startSlepen(e, i, "nw")} />
                      <div className="studio-handle" style={{ right: -5, top: -5, cursor: "nesw-resize" }} onPointerDown={(e) => startSlepen(e, i, "ne")} />
                      <div className="studio-handle" style={{ left: -5, bottom: -5, cursor: "nesw-resize" }} onPointerDown={(e) => startSlepen(e, i, "sw")} />
                      <div className="studio-handle" style={{ right: -5, bottom: -5, cursor: "nwse-resize" }} onPointerDown={(e) => startSlepen(e, i, "se")} />
                    </>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </section>

      {/* RECHTS: 4-secties DaVinci Inspector (Positie & Maat standaard ingeklapt) */}
      <aside className="studio-right">
        <div className="pane-head">
          <span className="pane-title">{actiefBlok ? BLOKNAMEN[actiefBlok.soort.blok] : "Sjabloon-instellingen"}</span>
          {opgeslagen.soort !== "vast" && !isNieuwEigen && (
            <button
              type="button"
              className="btn sm ghost"
              onClick={() => void regie.stuur({ soort: "WisSjabloon", id: opgeslagen.sjabloon.id }, BIJ)}
            >
              <RotateCcw size={12} /> {opgeslagen.soort === "aangepast" ? "Herstel" : "Wis"}
            </button>
          )}
        </div>

        <div className="inspector-scroll">
          {/* Sjabloon basis (naam, achtergrond, stoelen) */}
          <div className="field">
            <label>Naam Sjabloon</label>
            <input
              className="input sm"
              maxLength={40}
              value={concept.naam}
              onChange={(e) => pasAan((s) => ({ ...s, naam: e.target.value }))}
            />
          </div>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
            <div className="field">
              <label>Achtergrond</label>
              <select
                className="select sm"
                value={concept.achtergrond.soort}
                onChange={(e) => {
                  const v = e.target.value as Achtergrond["soort"];
                  pasAan((s) => ({
                    ...s,
                    achtergrond:
                      v === "Kleur"
                        ? { soort: "Kleur", kleur: { Stijl: "Achtergrond" } }
                        : v === "Afbeelding"
                          ? { soort: "Afbeelding", bestand: "" }
                          : { soort: v },
                  }));
                }}
              >
                {ACHTERGRONDEN.map((a) => (
                  <option key={a.soort} value={a.soort}>
                    {a.naam}
                  </option>
                ))}
              </select>
            </div>
            <div className="field">
              <label>Stoelen</label>
              <select
                className="select sm"
                value={concept.stoelen}
                onChange={(e) => pasAan((s) => ({ ...s, stoelen: e.target.value as Stoelen }))}
              >
                {STOEL_OPTIES.map((o) => (
                  <option key={o.stoelen} value={o.stoelen}>
                    {o.naam}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {actiefBlok && (
            <>
              {/* Sectie 1: Inhoud */}
              <details className="deck-fold" open>
                <summary>
                  <span>1. Inhoud</span>
                  <ChevronDown size={13} />
                </summary>
                <div className="deck-fold-body">
                  {actiefBlok.soort.blok === "Tekst" && (
                    <>
                      <div className="field">
                        <label>Tekstbron</label>
                        <select
                          className="select sm"
                          value={
                            typeof actiefBlok.soort.bron === "object"
                              ? "Vast" in actiefBlok.soort.bron
                                ? "Vast"
                                : "Veld"
                              : actiefBlok.soort.bron
                          }
                          onChange={(e) => {
                            const v = e.target.value;
                            wijzigBlok(gekozenBlok, (b) => {
                              if (b.soort.blok !== "Tekst") return;
                              b.soort.bron = v === "Vast" ? { Vast: "Tekst" } : v === "Veld" ? { Veld: "kop" } : (v as Exclude<typeof b.soort.bron, object>);
                            });
                          }}
                        >
                          <option value="Veld">Invulveld (in Regie)</option>
                          <option value="Vast">Vaste tekst</option>
                          <option value="Partijnaam">Naam van gekozen Partij</option>
                          <option value="Setnaam">Naam van gekozen set</option>
                        </select>
                      </div>
                      {typeof actiefBlok.soort.bron === "object" && "Vast" in actiefBlok.soort.bron && (
                        <div className="field">
                          <label>Vaste tekst</label>
                          <input
                            className="input sm"
                            value={actiefBlok.soort.bron.Vast}
                            onChange={(e) =>
                              wijzigBlok(gekozenBlok, (b) => {
                                if (b.soort.blok === "Tekst") b.soort.bron = { Vast: e.target.value };
                              })
                            }
                          />
                        </div>
                      )}
                      {typeof actiefBlok.soort.bron === "object" && "Veld" in actiefBlok.soort.bron && (
                        <div className="field">
                          <label>Veldnaam</label>
                          <input
                            className="input sm"
                            value={actiefBlok.soort.bron.Veld}
                            onChange={(e) =>
                              wijzigBlok(gekozenBlok, (b) => {
                                if (b.soort.blok === "Tekst") b.soort.bron = { Veld: e.target.value };
                              })
                            }
                          />
                        </div>
                      )}
                    </>
                  )}
                  {actiefBlok.soort.blok !== "Tekst" && (
                    <div style={{ fontSize: 12, color: "var(--text-2)" }}>
                      Bloktype: <strong>{BLOKNAMEN[actiefBlok.soort.blok]}</strong>
                    </div>
                  )}
                </div>
              </details>

              {/* Sectie 2: Vorm & Kleur */}
              <details className="deck-fold" open>
                <summary>
                  <span>2. Vorm & Kleur</span>
                  <ChevronDown size={13} />
                </summary>
                <div className="deck-fold-body">
                  <div className="field">
                    <label>Kleur</label>
                    <select
                      className="select sm"
                      value={"Stijl" in actiefBlok.kleur ? `Stijl:${actiefBlok.kleur.Stijl}` : "Eigen"}
                      onChange={(e) => {
                        const v = e.target.value;
                        wijzigBlok(gekozenBlok, (b) => {
                          if (v.startsWith("Stijl:")) b.kleur = { Stijl: v.slice(6) as Extract<Kleurkeuze, { Stijl: unknown }>["Stijl"] };
                          else b.kleur = { Eigen: "#F0B44C" };
                        });
                      }}
                    >
                      <option value="Stijl:Vlak">Stijl: Vlak</option>
                      <option value="Stijl:Achtergrond">Stijl: Achtergrond</option>
                      <option value="Stijl:Hoofd">Stijl: Hoofd</option>
                      <option value="Stijl:Accent">Stijl: Accent</option>
                      <option value="Stijl:Tekst">Stijl: Tekst</option>
                      <option value="Eigen">Eigen kleur</option>
                    </select>
                  </div>
                  {"Eigen" in actiefBlok.kleur && (
                    <input
                      type="color"
                      value={actiefBlok.kleur.Eigen}
                      onChange={(e) =>
                        wijzigBlok(gekozenBlok, (b) => {
                          b.kleur = { Eigen: e.target.value };
                        })
                      }
                      style={{ width: 42, height: 26, border: 0, background: "transparent" }}
                    />
                  )}
                  {actiefBlok.soort.blok === "Vlak" && (
                    <div className="field">
                      <label>
                        <span>Dekking</span>
                        <span className="mono">{actiefBlok.soort.dekking}%</span>
                      </label>
                      <input
                        type="range"
                        min={10}
                        max={100}
                        value={actiefBlok.soort.dekking}
                        onChange={(e) =>
                          wijzigBlok(gekozenBlok, (b) => {
                            if (b.soort.blok === "Vlak") b.soort.dekking = Number(e.target.value);
                          })
                        }
                      />
                    </div>
                  )}
                </div>
              </details>

              {/* Sectie 3: Animatie */}
              <details className="deck-fold">
                <summary>
                  <span>3. Animatie</span>
                  <ChevronDown size={13} />
                </summary>
                <div className="deck-fold-body">
                  <div className="field">
                    <label>Erin</label>
                    <select
                      className="select sm"
                      value={actiefBlok.animatie.erin}
                      onChange={(e) =>
                        wijzigBlok(gekozenBlok, (b) => {
                          b.animatie.erin = e.target.value as Erin;
                        })
                      }
                    >
                      {ERIN.map((x) => (
                        <option key={x.erin} value={x.erin}>{x.naam}</option>
                      ))}
                    </select>
                  </div>
                  <div className="field">
                    <label>Eruit</label>
                    <select
                      className="select sm"
                      value={actiefBlok.animatie.eruit}
                      onChange={(e) =>
                        wijzigBlok(gekozenBlok, (b) => {
                          b.animatie.eruit = e.target.value as Eruit;
                        })
                      }
                    >
                      {ERUIT.map((x) => (
                        <option key={x.eruit} value={x.eruit}>{x.naam}</option>
                      ))}
                    </select>
                  </div>
                  <div className="field">
                    <label>
                      <span>Start na</span>
                      <span className="mono">{actiefBlok.animatie.start_na_ms} ms</span>
                    </label>
                    <input
                      type="range"
                      min={0}
                      max={1000}
                      step={50}
                      value={actiefBlok.animatie.start_na_ms}
                      onChange={(e) =>
                        wijzigBlok(gekozenBlok, (b) => {
                          b.animatie.start_na_ms = Number(e.target.value);
                        })
                      }
                    />
                  </div>
                </div>
              </details>

              {/* Sectie 4: Positie & Maat (standaard ingeklapt — geen pixel-gevecht!) */}
              <details className="deck-fold">
                <summary>
                  <span>4. Positie & Maat (exacte pixels)</span>
                  <ChevronDown size={13} />
                </summary>
                <div className="deck-fold-body">
                  <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
                    {(["x", "y", "breedte", "hoogte"] as const).map((k) => (
                      <div className="field" key={k}>
                        <label>{k.toUpperCase()}</label>
                        <input
                          type="number"
                          className="input sm mono"
                          value={actiefBlok.plek[k]}
                          onChange={(e) =>
                            wijzigBlok(gekozenBlok, (b) => {
                              b.plek[k] = Math.max(0, Number(e.target.value) || 0);
                            })
                          }
                        />
                      </div>
                    ))}
                  </div>
                </div>
              </details>
            </>
          )}

          {validatieFout && <div className="melding-balk">{validatieFout}</div>}
        </div>

        <div className="inspector-footer">
          <span style={{ fontSize: 11.5, color: "var(--text-3)" }}>
            {gewijzigd ? "Niet opgeslagen" : "Opgeslagen"}
          </span>
          <button
            type="button"
            className="btn sm amber"
            disabled={!gewijzigd || !!validatieFout}
            onClick={() => void regie.stuur({ soort: "ZetSjabloon", sjabloon: concept }, BIJ)}
          >
            <Save size={13} /> Bewaar Sjabloon
          </button>
        </div>
      </aside>
    </div>
  );
}

/* ==========================================================================
   2. Stijl-editor (kleuren, snelheid, kleurovergang & Kleurkracht)
   ========================================================================== */

function StijlEditor() {
  const regie = useRegie();
  const stand = regie.stand!.stand;
  const [stijl, setStijl] = useState<Stijl>(() => structuredClone(stand.stijl));

  useEffect(() => {
    setStijl(structuredClone(stand.stijl));
  }, [JSON.stringify(stand.stijl)]);

  const gewijzigd = JSON.stringify(stijl) !== JSON.stringify(stand.stijl);

  return (
    <div className="regie-split">
      <div className="work-scroll" style={{ display: "flex", flexDirection: "column", gap: 14 }}>
        <div className="deck-card">
          <div style={{ fontWeight: 600 }}>Uitzendkleuren</div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(150px, 1fr))", gap: 10 }}>
            {(
              [
                ["achtergrond", "Achtergrond"],
                ["vlak", "Vlak"],
                ["hoofd", "Hoofd"],
                ["accent", "Accent"],
                ["tekst", "Tekst"],
              ] as const
            ).map(([k, label]) => (
              <div className="field" key={k}>
                <label>{label}</label>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <input
                    type="color"
                    value={stijl.kleuren[k]}
                    onChange={(e) => setStijl((s) => ({ ...s, kleuren: { ...s.kleuren, [k]: e.target.value } }))}
                    style={{ width: 32, height: 28, border: 0, background: "transparent", cursor: "pointer" }}
                  />
                  <span className="mono" style={{ fontSize: 11.5, color: "var(--text-2)" }}>{stijl.kleuren[k]}</span>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="deck-card">
          <div style={{ fontWeight: 600 }}>Animatie & Stoelen</div>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
            <div className="field">
              <label>
                <span>Animatiesnelheid</span>
                <span className="mono">{stijl.snelheid}%</span>
              </label>
              <input
                type="range"
                min={50}
                max={200}
                step={5}
                value={stijl.snelheid}
                onChange={(e) => setStijl((s) => ({ ...s, snelheid: Number(e.target.value) }))}
              />
            </div>
            <div className="field">
              <label>
                <span>Kleurkracht in de Zaal</span>
                <span className="mono">{stijl.kleurkracht}%</span>
              </label>
              <input
                type="range"
                min={0}
                max={100}
                step={5}
                value={stijl.kleurkracht}
                onChange={(e) => setStijl((s) => ({ ...s, kleurkracht: Number(e.target.value) }))}
              />
            </div>
          </div>
        </div>
      </div>

      <aside className="inspector-pane">
        <div className="pane-head">
          <span className="pane-title">Live Stijl-voorbeeld</span>
        </div>
        <div className="inspector-scroll">
          <ConceptCanvas
            graphic={{ sjabloon: "Titelkaart", kop: "Uitslagavond", regel: "Tweede Kamerverkiezingen" }}
            stijl={stijl}
            label="Stijl-concept"
          />
        </div>
        <div className="inspector-footer">
          <button type="button" className="btn sm ghost" disabled={!gewijzigd} onClick={() => setStijl(structuredClone(stand.stijl))}>
            Herstel
          </button>
          <button
            type="button"
            className="btn sm amber"
            disabled={!gewijzigd}
            onClick={() => void regie.stuur({ soort: "ZetStijl", stijl }, BIJ)}
          >
            <Save size={13} /> Bewaar Stijl
          </button>
        </div>
      </aside>
    </div>
  );
}

/* ==========================================================================
   3. Camera-werkbank (Standpunten & Cameraritten fijnafstellen)
   ========================================================================== */

function CameraWerkbank() {
  const regie = useRegie();
  const stand = regie.stand!.stand;
  const [nieuwNaam, setNieuwNaam] = useState("");

  return (
    <div className="work-scroll" style={{ display: "flex", flexDirection: "column", gap: 14 }}>
      <div className="deck-card" style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: 8 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, flex: 1, minWidth: 240 }}>
          <input
            className="input sm"
            style={{ maxWidth: 260 }}
            maxLength={40}
            value={nieuwNaam}
            onChange={(e) => setNieuwNaam(e.target.value)}
            placeholder="Naam voor nieuw eigen Standpunt…"
          />
          <button
            type="button"
            className="btn sm amber"
            disabled={!nieuwNaam.trim()}
            onClick={() => {
              void regie.stuur({ soort: "BewaarStandpunt", naam: nieuwNaam.trim() }, CAMERA);
              setNieuwNaam("");
            }}
          >
            <Plus size={12} /> Bewaar huidige camera
          </button>
        </div>
      </div>

      <table className="deck-table">
        <thead>
          <tr>
            <th>Toets</th>
            <th>Standpunt</th>
            <th>Overgang</th>
            <th>Duur</th>
            <th>Acties</th>
          </tr>
        </thead>
        <tbody>
          {stand.standpunten.map((sp) => (
            <tr key={sp.letter}>
              <td><kbd className="key">{sp.toets ?? sp.letter}</kbd></td>
              <td style={{ fontWeight: 500 }}>{sp.naam}</td>
              <td>
                <select
                  className="select sm"
                  style={{ width: 130 }}
                  value={sp.overgang.soort}
                  onChange={(e) =>
                    void regie.stuur(
                      {
                        soort: "ZetStandpuntOvergang",
                        letter: sp.letter,
                        overgang: { ...sp.overgang, soort: e.target.value as OvergangSoort },
                        na_aankomst: sp.na_aankomst,
                      },
                      CAMERA,
                    )
                  }
                >
                  <option value="Bewegen">Beweeg</option>
                  <option value="Overvloeien">Fade</option>
                  <option value="Knippen">Knip</option>
                </select>
              </td>
              <td className="mono">{sp.overgang.duur_s.toFixed(1)} s</td>
              <td>
                <div style={{ display: "flex", gap: 6 }}>
                  <button
                    type="button"
                    className="btn sm"
                    onClick={() => void regie.stuur({ soort: "ProbeerInVoorbeeld", letter: sp.letter }, CAMERA)}
                  >
                    <Play size={11} /> Test
                  </button>
                  {!sp.vast && (
                    <button
                      type="button"
                      className="btn sm danger-outline"
                      onClick={() => void regie.stuur({ soort: "WisStandpunt", letter: sp.letter }, CAMERA)}
                    >
                      <Trash2 size={11} />
                    </button>
                  )}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function kleurHex(k: Kleurkeuze, stijl: Stijl): string {
  if ("Eigen" in k) return k.Eigen;
  switch (k.Stijl) {
    case "Achtergrond": return stijl.kleuren.achtergrond;
    case "Vlak": return stijl.kleuren.vlak;
    case "Hoofd": return stijl.kleuren.hoofd;
    case "Accent": return stijl.kleuren.accent;
    case "Tekst": return stijl.kleuren.tekst;
  }
}

export type { Bloksoort };
