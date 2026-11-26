// Zaal & Camera werkruimte (DaVinci Deck):
// - Links: Interactieve Zaal-plattegrond (klikbare Stoelen bij Losse stoelen) + Preview vs Direct Live
//          schakelaar + Standpunten (A–H, J–L) & Cameraritten (Q–U) met Test-knop + Gamepad-bediening.
// - Rechts: Los Stoelbeeld (Alleen stoelen laten oplichten zonder 2D-Graphic): 5 bronnen
//          (Set, 1+ Partijen, Coalitie, Stemming, Losse stoelen), Kleurovergang, Kleurkracht,
//          Zet klaar / Direct live / Bewaar als Preset.
import { useMemo, useState } from "react";
import { Camera, Circle, Gamepad2, Play, Radio, Save, Send, Sparkles, Square } from "lucide-react";
import { STOELPLEKKEN } from "../typen/zaalkaart";
import { useCamera, useRegie } from "../regie/regie";
import { CAMERA, kiesStandpunt, zetCameraKlaar, zetStoelbeeldDirect, zetStoelbeeldKlaar } from "../regie/take";
import { toetsVan } from "../regie/camerarit";
import type { Indeling } from "../typen/Indeling";
import type { Overgangsvorm } from "../typen/Overgangsvorm";
import type { Stoelbeeld } from "../typen/Stoelbeeld";
import type { Stoelbron } from "../typen/Stoelbron";

const GRIJS = "#4C4F55";

export function ZaalCameraWerkruimte() {
  const regie = useRegie();
  const cameraBeeld = useCamera();
  const stand = regie.stand?.stand;

  // Veilige default: Preview (groen); pas op aanvraag Direct live (rood)
  const [doelModus, setDoelModus] = useState<"preview" | "live">("preview");

  // Stoelbeeld-composer state
  const [bronSoort, setBronSoort] = useState<"Set" | "Partijen" | "Coalitie" | "Stemming" | "Los">("Partijen");
  const [setId, setSetId] = useState<string>("");
  const [indeling, setIndeling] = useState<Indeling>("Uitslagindeling");
  const [gekozenPartijen, setGekozenPartijen] = useState<string[]>([]);
  const [stemmingId, setStemmingId] = useState<string>("");
  const [losseStoelen, setLosseStoelen] = useState<Record<number, string>>({});
  const [penseelKleur, setPenseelKleur] = useState<string>("#F0B44C");
  const [overgangVorm, setOvergangVorm] = useState<Overgangsvorm>("Golf");
  const [duurMs, setDuurMs] = useState<number>(1000);
  const [kleurkracht, setKleurkracht] = useState<number>(60);
  const [presetNaam, setPresetNaam] = useState<string>("");

  const effectieveSet = setId || stand?.cijfers.sets[0]?.id || "";
  const effectieveStemming = stemmingId || stand?.stemmingen.lijst[0]?.punt.id || "";

  const stoelbeeld: Stoelbeeld | null = useMemo(() => {
    if (!stand) return null;
    let bron: Stoelbron;
    let autoLabel = "";
    switch (bronSoort) {
      case "Set": {
        const s = stand.cijfers.sets.find((x) => x.id === effectieveSet);
        bron = { soort: "Set", set: effectieveSet, indeling };
        autoLabel = s?.label ?? effectieveSet;
        break;
      }
      case "Partijen": {
        if (gekozenPartijen.length === 0) return null;
        bron = { soort: "Partijen", set: effectieveSet, partijen: gekozenPartijen };
        autoLabel = gekozenPartijen
          .map((k) => stand.cijfers.partijen.find((p) => p.sleutel === k)?.kort ?? k)
          .join(" + ");
        break;
      }
      case "Coalitie": {
        if (gekozenPartijen.length === 0) return null;
        bron = { soort: "Coalitie", set: effectieveSet, partijen: gekozenPartijen };
        autoLabel = `Coalitie ${gekozenPartijen
          .map((k) => stand.cijfers.partijen.find((p) => p.sleutel === k)?.kort ?? k)
          .join(" + ")}`;
        break;
      }
      case "Stemming": {
        if (!effectieveStemming) return null;
        bron = { soort: "Stemming", stemming: effectieveStemming };
        const st = stand.stemmingen.lijst.find((x) => x.punt.id === effectieveStemming);
        autoLabel = st ? `Stemming · ${st.punt.titel.slice(0, 24)}` : "Stemming";
        break;
      }
      case "Los": {
        const kleuren = Array.from({ length: 150 }, (_, i) => losseStoelen[i] ?? GRIJS);
        bron = { soort: "Los", kleuren };
        const n = Object.keys(losseStoelen).length;
        autoLabel = n === 1 ? "1 losse stoel" : `${n} losse stoelen`;
        break;
      }
    }
    return {
      bron,
      overgang: { vorm: overgangVorm, duur_ms: duurMs },
      kleurkracht,
      label: presetNaam.trim() || autoLabel,
    };
  }, [stand, bronSoort, effectieveSet, indeling, gekozenPartijen, effectieveStemming, losseStoelen, overgangVorm, duurMs, kleurkracht, presetNaam]);

  if (!stand) return null;

  function klikCamera(letter: string, knip: boolean) {
    if (doelModus === "preview") {
      zetCameraKlaar(regie, letter);
    } else {
      kiesStandpunt(regie, letter, knip);
    }
  }

  function wisselPartij(sleutel: string) {
    setGekozenPartijen((oud) => (oud.includes(sleutel) ? oud.filter((x) => x !== sleutel) : [...oud, sleutel]));
  }

  function klikStoel(index: number) {
    if (bronSoort !== "Los") return;
    setLosseStoelen((oud) => {
      const kopie = { ...oud };
      if (kopie[index]) delete kopie[index];
      else kopie[index] = penseelKleur;
      return kopie;
    });
  }

  // Welke kleuren op de plattegrond tonen: bij Losse stoelen de eigen tekening, anders Preview of Live
  const kaartKleuren: string[] =
    bronSoort === "Los"
      ? Array.from({ length: 150 }, (_, i) => losseStoelen[i] ?? "#2c3442")
      : (stand.klaar_stoelkleuren?.length ?? 0) === 150
        ? (stand.klaar_stoelkleuren ?? [])
        : stand.stoelkleuren;

  return (
    <div className="zaal-grid">
      {/* LINKS: Plattegrond + Camera-besturing */}
      <div className="zaal-col">
        <div className="pane-head">
          <div className="pane-title">
            <Camera size={14} />
            <span>Camera & Zaalplattegrond</span>
          </div>
          <div className="seg" role="group" aria-label="Camera bestemming">
            <button
              type="button"
              className={doelModus === "preview" ? "active preview" : ""}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => setDoelModus("preview")}
              title="Klaarzetten in Preview; beweegt pas bij Take"
            >
              Preview (veilig)
            </button>
            <button
              type="button"
              className={doelModus === "live" ? "active live" : ""}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => setDoelModus("live")}
              title="Stuurt de Live-camera direct aan"
            >
              <Radio size={12} />
              Direct live
            </button>
          </div>
        </div>

        <div className="zaal-scroll">
          {regie.melding?.bij === CAMERA && <div className="melding-balk">{regie.melding.tekst}</div>}

          {/* Interactieve 150-stoelen Zaalplattegrond */}
          <div className="plattegrond-wrap">
            <svg viewBox="-12 -14 24 22" role="img" aria-label="Plattegrond van de Zaal">
              {/* Halve cirkel achtergrond */}
              <path d="M -11 0 A 11 11 0 0 1 11 0 L 11 6 L -11 6 Z" fill="#1b1f29" stroke="rgba(255,255,255,0.08)" strokeWidth="0.12" />
              {/* Voorzitter */}
              <rect x="-2.4" y="3.4" width="4.8" height="1.1" rx="0.25" fill="#2b303c" />
              <text x="0" y="4.2" fontSize="0.65" fill="#838998" textAnchor="middle">
                Voorzitter
              </text>

              {/* 150 Stoelen */}
              {STOELPLEKKEN.map((s, i) => {
                const kleur = kaartKleuren[i] || "#3a455a";
                return (
                  <circle
                    key={i}
                    cx={s[0]}
                    cy={s[1]}
                    r={bronSoort === "Los" ? 0.34 : 0.28}
                    fill={kleur}
                    stroke={bronSoort === "Los" && losseStoelen[i] ? "#ffffff" : "rgba(0,0,0,0.35)"}
                    strokeWidth={bronSoort === "Los" && losseStoelen[i] ? 0.1 : 0.05}
                    style={bronSoort === "Los" ? { cursor: "pointer" } : undefined}
                    onClick={() => klikStoel(i)}
                  />
                );
              })}

              {/* Preview camera positie (groen) */}
              {cameraBeeld && (
                <g transform={`translate(${cameraBeeld.voorbeeld.stand.positie[0]}, ${cameraBeeld.voorbeeld.stand.positie[2]})`}>
                  <circle r="0.52" fill="#2ea66d" stroke="#0d1016" strokeWidth="0.14" />
                </g>
              )}
              {/* Live camera positie (rood) */}
              {cameraBeeld && (
                <g transform={`translate(${cameraBeeld.ether.stand.positie[0]}, ${cameraBeeld.ether.stand.positie[2]})`}>
                  <circle r="0.58" fill="#e5484d" stroke="#ffffff" strokeWidth="0.14" />
                </g>
              )}
            </svg>
          </div>

          {/* Standpunten */}
          <div className="field">
            <label>
              <span>Standpunten ({doelModus === "preview" ? "klik zet klaar in Preview" : "klik stuurt direct Live"})</span>
              {stand.camera.klaar_standpunt && (
                <button
                  type="button"
                  className="btn sm ghost"
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => zetCameraKlaar(regie, null)}
                >
                  Wis klaargezette camera
                </button>
              )}
            </label>
            <div className="cam-grid">
              {stand.standpunten.map((sp) => {
                const toets = toetsVan(sp);
                const isLive = !stand.camera.vrij && stand.camera.standpunt === sp.letter;
                const isPrev = stand.camera.klaar_standpunt === sp.letter;
                return (
                  <button
                    key={sp.letter}
                    type="button"
                    className={`cam-tile${isLive ? " is-live" : isPrev ? " is-preview" : ""}`}
                    onMouseDown={(e) => e.preventDefault()}
                    onClick={(e) => klikCamera(sp.letter, e.shiftKey)}
                    title={doelModus === "preview" ? `Zet ${sp.naam} klaar in Preview` : `Direct Live naar ${sp.naam} (Shift = knip)`}
                  >
                    <span className="cam-tile-name">{sp.naam}</span>
                    {toets && <kbd className="key">{toets}</kbd>}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Cameraritten */}
          <div className="field">
            <label>Cameraritten (wacht op beginpunt in Preview · start pas bij Take)</label>
            <div className="cam-grid">
              {stand.cameraritten.map((r) => {
                const toets = toetsVan(r);
                const isLive = !stand.camera.vrij && stand.camera.standpunt === r.letter;
                const isPrev = stand.camera.klaar_standpunt === r.letter;
                return (
                  <div
                    key={r.letter}
                    className={`cam-tile${isLive ? " is-live" : isPrev ? " is-preview" : ""}`}
                    style={{ cursor: "pointer" }}
                    onClick={(e) => klikCamera(r.letter, e.shiftKey)}
                  >
                    <span className="cam-tile-name">{r.naam}</span>
                    <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
                      <button
                        type="button"
                        className="btn icon sm ghost"
                        onMouseDown={(e) => e.preventDefault()}
                        onClick={(e) => {
                          e.stopPropagation();
                          void regie.stuur({ soort: "ProbeerInVoorbeeld", letter: r.letter }, CAMERA);
                        }}
                        title="Proefdraaien in Preview"
                      >
                        <Play size={11} />
                      </button>
                      {toets && <kbd className="key">{toets}</kbd>}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Gamepad / Vrije camera */}
          <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap", paddingTop: 4 }}>
            <button
              type="button"
              className={`btn sm${stand.camera.vrij ? " live" : ""}`}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => void regie.stuur({ soort: "ZetVrijeCamera", aan: !stand.camera.vrij }, CAMERA)}
            >
              <Gamepad2 size={13} />
              Vrije camera Live
              <kbd className="key">V</kbd>
            </button>
            <button
              type="button"
              className={`btn sm${stand.camera.oefenen ? " preview" : ""}`}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => void regie.stuur({ soort: "ZetOefenen", aan: !stand.camera.oefenen }, CAMERA)}
            >
              Oefenen in Preview
              <kbd className="key">O</kbd>
            </button>
            <button
              type="button"
              className={`btn sm${stand.camera.opname ? " live" : ""}`}
              disabled={!stand.camera.oefenen && !stand.camera.opname}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => void regie.stuur({ soort: "ZetOpnemen", aan: !stand.camera.opname }, CAMERA)}
            >
              {stand.camera.opname ? <Square size={11} /> : <Circle size={11} />}
              {stand.camera.opname ? "Stop opname" : "Opnemen"}
              <kbd className="key">R</kbd>
            </button>
          </div>
        </div>
      </div>

      {/* RECHTS: Alleen Stoelen (Stoelbeeld) */}
      <div className="zaal-col" style={{ background: "var(--bg-surface)" }}>
        <div className="pane-head">
          <div className="pane-title">
            <Sparkles size={14} color="var(--amber)" />
            <span>Stoelbeeld (alleen stoelen)</span>
          </div>
          <button
            type="button"
            className="btn sm ghost"
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => zetStoelbeeldKlaar(regie, { bron: { soort: "Uit" }, label: "Stoelen uit" })}
            title="Zet 'Stoelen uit' klaar in Preview"
          >
            Stoelen uit klaarzetten
          </button>
        </div>

        <div className="zaal-scroll">
          {/* Bron kiezen */}
          <div className="field">
            <label>Welke stoelen oplichten</label>
            <div className="seg" style={{ flexWrap: "wrap" }}>
              {(
                [
                  ["Partijen", "Partij(en)"],
                  ["Set", "Hele set"],
                  ["Coalitie", "Coalitie"],
                  ["Stemming", "Stemming"],
                  ["Los", "Losse stoelen"],
                ] as const
              ).map(([id, label]) => (
                <button
                  key={id}
                  type="button"
                  className={bronSoort === id ? "active amber" : ""}
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => setBronSoort(id)}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>

          {/* Set-selector voor Set, Partijen, Coalitie */}
          {(bronSoort === "Set" || bronSoort === "Partijen" || bronSoort === "Coalitie") && (
            <div className="field">
              <label>Cijferset</label>
              <select className="select" value={effectieveSet} onChange={(e) => setSetId(e.target.value)}>
                {stand.cijfers.sets.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.label}
                  </option>
                ))}
                <option value="huidige-kamer">Huidige Kamer</option>
              </select>
            </div>
          )}

          {bronSoort === "Set" && (
            <div className="field">
              <label>Indeling</label>
              <div className="seg">
                <button
                  type="button"
                  className={indeling === "Uitslagindeling" ? "active amber" : ""}
                  onClick={() => setIndeling("Uitslagindeling")}
                >
                  Uitslagindeling
                </button>
                <button
                  type="button"
                  className={indeling === "Zaalindeling" ? "active amber" : ""}
                  onClick={() => setIndeling("Zaalindeling")}
                >
                  Echte stoelen
                </button>
              </div>
            </div>
          )}

          {/* Partijen kiezen (1 of meer!) */}
          {(bronSoort === "Partijen" || bronSoort === "Coalitie") && (
            <div className="field">
              <label>
                <span>Kies 1 of meer Partijen</span>
                {gekozenPartijen.length > 0 && (
                  <button type="button" className="btn sm ghost" onClick={() => setGekozenPartijen([])}>
                    Wis ({gekozenPartijen.length})
                  </button>
                )}
              </label>
              <div className="partij-grid">
                {stand.cijfers.partijen.map((p) => {
                  const aan = gekozenPartijen.includes(p.sleutel);
                  return (
                    <button
                      key={p.sleutel}
                      type="button"
                      className={`partij-btn${aan ? " active" : ""}`}
                      onMouseDown={(e) => e.preventDefault()}
                      onClick={() => wisselPartij(p.sleutel)}
                    >
                      <span className="partij-swatch" style={{ background: p.kleur }} />
                      <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.kort || p.naam}</span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {bronSoort === "Stemming" && (
            <div className="field">
              <label>Stemming</label>
              <select className="select" value={effectieveStemming} onChange={(e) => setStemmingId(e.target.value)}>
                {stand.stemmingen.lijst.map((s) => (
                  <option key={s.punt.id} value={s.punt.id}>
                    {s.punt.titel}
                  </option>
                ))}
              </select>
            </div>
          )}

          {bronSoort === "Los" && (
            <div className="deck-card">
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 8 }}>
                <span style={{ fontSize: 12, color: "var(--text-2)" }}>
                  Klik op stoelen op de plattegrond links ({Object.keys(losseStoelen).length} gekozen)
                </span>
                <button type="button" className="btn sm ghost" onClick={() => setLosseStoelen({})}>
                  Alles wissen
                </button>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <input
                  type="color"
                  value={penseelKleur}
                  onChange={(e) => setPenseelKleur(e.target.value)}
                  style={{ width: 32, height: 28, border: 0, background: "transparent", cursor: "pointer" }}
                />
                {["#F0B44C", "#29B894", "#E5484D", "#3B82F6", "#FFFFFF"].map((k) => (
                  <button
                    key={k}
                    type="button"
                    className="btn sm"
                    style={{
                      width: 24,
                      padding: 0,
                      background: k,
                      borderColor: penseelKleur === k ? "#fff" : "transparent",
                    }}
                    onClick={() => setPenseelKleur(k)}
                    aria-label={`Kleur ${k}`}
                  />
                ))}
              </div>
            </div>
          )}

          {/* Kleurovergang & Kleurkracht */}
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
            <div className="field">
              <label>Kleurovergang</label>
              <div className="seg">
                {(["Golf", "Vervagen", "Opbouw"] as const).map((v) => (
                  <button
                    key={v}
                    type="button"
                    className={overgangVorm === v ? "active amber" : ""}
                    onClick={() => setOvergangVorm(v)}
                  >
                    {v}
                  </button>
                ))}
              </div>
            </div>
            <div className="field">
              <label>
                <span>Duur</span>
                <span className="mono">{(duurMs / 1000).toFixed(1)} s</span>
              </label>
              <input
                type="range"
                min={300}
                max={3000}
                step={100}
                value={duurMs}
                onChange={(e) => setDuurMs(Number(e.target.value))}
              />
            </div>
          </div>

          <div className="field">
            <label>
              <span>Kleurkracht</span>
              <span className="mono">{kleurkracht}%</span>
            </label>
            <input
              type="range"
              min={0}
              max={100}
              step={5}
              value={kleurkracht}
              onChange={(e) => setKleurkracht(Number(e.target.value))}
            />
          </div>

          <div className="field">
            <label>Label / Preset-naam (optioneel)</label>
            <input
              className="input sm"
              maxLength={80}
              value={presetNaam}
              onChange={(e) => setPresetNaam(e.target.value)}
              placeholder="Bijv. VVD + CDA uitgelicht"
            />
          </div>
        </div>

        <div className="inspector-footer">
          <button
            type="button"
            className="btn sm"
            disabled={!stoelbeeld}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => {
              if (!stoelbeeld) return;
              void regie.stuur(
                {
                  soort: "BewaarPresetUitgebreid",
                  naam: stoelbeeld.label,
                  stoelbeeld,
                  standpunt: stand.camera.klaar_standpunt,
                },
                CAMERA,
              );
            }}
          >
            <Save size={13} />
            Bewaar Preset
          </button>
          <div style={{ display: "flex", gap: 6 }}>
            <button
              type="button"
              className="btn sm danger-outline"
              disabled={!stoelbeeld}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => stoelbeeld && zetStoelbeeldDirect(regie, stoelbeeld)}
              title="Kleurt de stoelen direct op Live zonder Take"
            >
              Direct live
            </button>
            <button
              type="button"
              className="btn sm preview"
              disabled={!stoelbeeld}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => stoelbeeld && zetStoelbeeldKlaar(regie, stoelbeeld)}
            >
              <Send size={13} />
              Zet klaar
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
