// Presentator-tablet (/tablet) in DaVinci Deck tokens:
// Behoudt de T2 2-tab indeling (Presets + Coalitiebouwer) met ondersteuning voor Stoelbeeld-presets.
// De tablet zet alleen klaar in Preview; de Regie drukt op Take.
import { useMemo, useState } from "react";
import { Check, LogOut, Sparkles } from "lucide-react";
import { RegieBron, useRegie } from "../regie/regie";
import type { Toegang } from "../regie/regie";
import { presetTitel, statusVan } from "../presets/presets";
import { Schets } from "../presets/Schets";

const SLEUTEL_KEY = "uitslagavond.tablet.sleutel";
const NAAM_KEY = "uitslagavond.tablet.naam";

export function TabletApp() {
  const [toegang, setToegang] = useState<Toegang | null>(() => {
    const sleutel = localStorage.getItem(SLEUTEL_KEY) || "";
    const werkplek = localStorage.getItem(NAAM_KEY) || "Tablet";
    return sleutel ? { sleutel, werkplek, tablet: true } : null;
  });

  if (!toegang) {
    return (
      <TabletLogin
        opAanmelden={(sleutel, naam) => {
          localStorage.setItem(SLEUTEL_KEY, sleutel);
          localStorage.setItem(NAAM_KEY, naam);
          setToegang({ sleutel, werkplek: naam, tablet: true });
        }}
      />
    );
  }

  return (
    <RegieBron
      toegang={toegang}
      afmelden={() => {
        localStorage.removeItem(SLEUTEL_KEY);
        setToegang(null);
      }}
    >
      <TabletScherm />
    </RegieBron>
  );
}

function TabletScherm() {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [tab, setTab] = useState<"presets" | "coalitie">("presets");

  const tabletPresets = useMemo(
    () => (stand ? stand.presets.filter((p) => p.op_tablet) : []),
    [stand?.presets],
  );

  const coalitieSet = stand?.cijfers.sets[0];
  const gekozen = useMemo(() => {
    const g = stand?.klaar?.graphic;
    if (g && (g.sjabloon === "Coalitie" || g.sjabloon === "Coalitiebalk")) return g.partijen;
    return [] as string[];
  }, [stand?.klaar]);

  const totaal = gekozen.reduce((s, k) => s + (coalitieSet?.zetels[k] ?? 0), 0);

  return (
    <div className="tablet-shell">
      <header className="tablet-bar">
        <div className="tablet-tabs">
          <button type="button" className={`tablet-tab${tab === "presets" ? " active" : ""}`} onClick={() => setTab("presets")}>
            Presets ({tabletPresets.length})
          </button>
          <button type="button" className={`tablet-tab${tab === "coalitie" ? " active" : ""}`} onClick={() => setTab("coalitie")}>
            Coalitiebouwer
          </button>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <span className="chip">
            <span className={`dot ${regie.staat === "verbonden" ? "ok" : "live"}`} />
            {regie.staat === "verbonden" ? "Verbonden" : regie.staat}
          </span>
          <button type="button" className="btn sm ghost" onClick={regie.afmelden} title="Afmelden">
            <LogOut size={14} />
          </button>
        </div>
      </header>

      <main className="tablet-body">
        {!stand ? (
          <div style={{ color: "var(--text-3)", textAlign: "center", padding: 32 }}>Wachten op server…</div>
        ) : tab === "presets" ? (
          tabletPresets.length === 0 ? (
            <div style={{ color: "var(--text-3)", textAlign: "center", padding: 32 }}>
              Nog geen Presets voor de tablet klaargezet door de Regie.
            </div>
          ) : (
            <div className="tablet-grid">
              {tabletPresets.map((p) => {
                const st = statusVan(p, stand);
                return (
                  <button
                    key={p.nummer}
                    type="button"
                    className={`tablet-card${st.live ? " is-live" : st.klaar ? " is-preview" : ""}`}
                    onClick={() => void regie.stuur({ soort: "ZetPresetKlaar", nummer: p.nummer }, "tablet")}
                  >
                    <div className="preset-thumb">
                      {p.zonder_graphic ? (
                        <div className="monitor-placeholder" style={{ background: "#121826", color: "var(--text-2)" }}>
                          <Sparkles size={22} color="var(--amber)" />
                          <span>Stoelbeeld</span>
                        </div>
                      ) : (
                        <Schets graphic={p.graphic} kleuren={stand.stijl.kleuren} />
                      )}
                      <div className="preset-thumb-badges">
                        <span className="preset-num">{p.nummer}</span>
                        <div>
                          {st.live && <span className="preset-tally-pill live">LIVE</span>}
                          {st.klaar && <span className="preset-tally-pill preview">KLAAR</span>}
                        </div>
                      </div>
                    </div>
                    <div style={{ padding: "10px 12px", fontWeight: 600, fontSize: 14 }}>{presetTitel(p, stand)}</div>
                  </button>
                );
              })}
            </div>
          )
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 14, maxWidth: 920, margin: "0 auto" }}>
            <div className="deck-card">
              <div style={{ display: "flex", alignItems: "baseline", justifyContent: "space-between" }}>
                <div style={{ display: "flex", alignItems: "baseline", gap: 10 }}>
                  <span className="mono" style={{ fontSize: 36, fontWeight: 700, color: totaal >= 76 ? "var(--preview-bright)" : "var(--text)" }}>
                    {totaal}
                  </span>
                  <span style={{ color: "var(--text-2)" }}>/ 76 zetels</span>
                  {totaal >= 76 && (
                    <span className="chip preview">
                      <Check size={12} /> Meerderheid
                    </span>
                  )}
                </div>
                {gekozen.length > 0 && (
                  <button type="button" className="btn sm" onClick={() => void regie.stuur({ soort: "ZetCoalitie", partijen: [] }, "tablet")}>
                    Wis
                  </button>
                )}
              </div>
            </div>

            <div className="partij-grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(156px, 1fr))", gap: 10 }}>
              {stand.cijfers.partijen
                .filter((p) => (coalitieSet?.zetels[p.sleutel] ?? 0) > 0)
                .sort((a, b) => (coalitieSet?.zetels[b.sleutel] ?? 0) - (coalitieSet?.zetels[a.sleutel] ?? 0))
                .map((p) => {
                  const z = coalitieSet?.zetels[p.sleutel] ?? 0;
                  const aan = gekozen.includes(p.sleutel);
                  return (
                    <button
                      key={p.sleutel}
                      type="button"
                      className={`partij-btn${aan ? " active" : ""}`}
                      style={{ minHeight: 48, padding: "10px 12px", justifyContent: "space-between", fontSize: 14 }}
                      onClick={() => {
                        const nieuw = aan ? gekozen.filter((x) => x !== p.sleutel) : [...gekozen, p.sleutel];
                        void regie.stuur({ soort: "ZetCoalitie", partijen: nieuw }, "tablet");
                      }}
                    >
                      <span style={{ display: "inline-flex", alignItems: "center", gap: 8 }}>
                        <span className="partij-swatch" style={{ background: p.kleur, width: 14, height: 14 }} />
                        <span>{p.kort || p.naam}</span>
                      </span>
                      <span className="mono" style={{ fontWeight: 700 }}>{z}</span>
                    </button>
                  );
                })}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

function TabletLogin({ opAanmelden }: { opAanmelden: (sleutel: string, naam: string) => void }) {
  const [naam, setNaam] = useState("Presentator");
  const [sleutel, setSleutel] = useState("");
  return (
    <div className="login-screen">
      <form
        className="login-card"
        onSubmit={(e) => {
          e.preventDefault();
          if (sleutel.trim()) opAanmelden(sleutel.trim(), naam.trim() || "Tablet");
        }}
      >
        <h1 style={{ fontSize: 18, fontWeight: 700 }}>Presentator-tablet</h1>
        <div className="field">
          <label>Naam</label>
          <input className="input" value={naam} onChange={(e) => setNaam(e.target.value)} />
        </div>
        <div className="field">
          <label>Gedeelde sleutel</label>
          <input className="input" type="password" value={sleutel} onChange={(e) => setSleutel(e.target.value)} autoFocus />
        </div>
        <button type="submit" className="btn lg amber" disabled={!sleutel.trim()}>
          Verbinden
        </button>
      </form>
    </div>
  );
}
