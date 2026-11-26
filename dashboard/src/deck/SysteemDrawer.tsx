// Systeem-lade (Slide-over vanuit de klikbare statuspil in de Transportbalk):
// Verbonden Renderers & versie-controle, Werkplek wisselen, datameldingen (Kamer, Zaal, Invoer).
import { AlertTriangle, CheckCircle2, LogOut, Monitor, Server, X } from "lucide-react";
import { useRegie } from "../regie/regie";

export function SysteemDrawer({ opSluit }: { opSluit: () => void }) {
  const regie = useRegie();
  const stand = regie.stand?.stand;

  return (
    <div className="drawer-backdrop" onClick={opSluit}>
      <div className="drawer-panel" onClick={(e) => e.stopPropagation()} role="dialog" aria-label="Systeem en Renderers">
        <div className="pane-head">
          <div className="pane-title">
            <Server size={14} />
            <span>Systeem & Renderers</span>
          </div>
          <button type="button" className="btn icon sm ghost" onClick={opSluit} aria-label="Sluiten">
            <X size={15} />
          </button>
        </div>

        <div className="inspector-scroll">
          {/* Werkplek */}
          <div className="deck-card">
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 8 }}>
              <div>
                <div style={{ fontWeight: 600 }}>{regie.werkplek}</div>
                <div style={{ fontSize: 11.5, color: "var(--text-3)" }}>
                  Server {regie.staat} · versie <span className="mono">{regie.rendererstatus.versie || "—"}</span>
                </div>
              </div>
              <button type="button" className="btn sm" onClick={regie.afmelden}>
                <LogOut size={12} /> Wissel werkplek
              </button>
            </div>
          </div>

          {/* Datameldingen */}
          {((stand?.cijfers.laatste_invoer?.waarschuwingen.length ?? 0) > 0 ||
            (stand?.kamer.laatste_ophaal?.fracties_nieuw.length ?? 0) > 0 ||
            (stand?.zaalmelding.zonder_stoel.length ?? 0) > 0) && (
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              {stand?.cijfers.laatste_invoer?.waarschuwingen.map((w, i) => (
                <div key={i} className="melding-balk warn">
                  <AlertTriangle size={14} />
                  <span>Cijfers: {w}</span>
                </div>
              ))}
              {(stand?.kamer.laatste_ophaal?.fracties_nieuw.length ?? 0) > 0 && (
                <div className="melding-balk warn">
                  <AlertTriangle size={14} />
                  <span>Nieuwe fracties zonder kleur: {stand!.kamer.laatste_ophaal!.fracties_nieuw.join(", ")}</span>
                </div>
              )}
              {(stand?.zaalmelding.zonder_stoel.length ?? 0) > 0 && (
                <div className="melding-balk warn">
                  <AlertTriangle size={14} />
                  <span>
                    Zaalindeling:{" "}
                    {stand!.zaalmelding.zonder_stoel.length === 1
                      ? "1 Kamerlid zonder stoel"
                      : `${stand!.zaalmelding.zonder_stoel.length} Kamerleden zonder stoel`}
                  </span>
                </div>
              )}
            </div>
          )}

          {/* Verbonden Renderers */}
          <div className="field">
            <label>Verbonden Renderers ({regie.rendererstatus.verbonden.length})</label>
            <table className="deck-table">
              <thead>
                <tr>
                  <th>Rol</th>
                  <th>Machine</th>
                  <th>Versie</th>
                </tr>
              </thead>
              <tbody>
                {regie.rendererstatus.verbonden.map((r, i) => (
                  <tr key={`${r.rol}-${r.machine}-${i}`}>
                    <td>
                      <span className="chip preview">
                        <CheckCircle2 size={11} /> {r.rol}
                      </span>
                    </td>
                    <td>{r.machine}</td>
                    <td className="mono">{r.versie}</td>
                  </tr>
                ))}
                {regie.rendererstatus.verbonden.length === 0 && (
                  <tr>
                    <td colSpan={3} style={{ color: "var(--text-3)" }}>Geen Renderers verbonden.</td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>

          {/* Geweigerde Renderers */}
          {regie.rendererstatus.geweigerd.length > 0 && (
            <div className="field">
              <label>Geweigerd wegens afwijkende versie</label>
              <table className="deck-table">
                <thead>
                  <tr>
                    <th>Rol</th>
                    <th>Machine</th>
                    <th>Versie</th>
                  </tr>
                </thead>
                <tbody>
                  {regie.rendererstatus.geweigerd.map((r, i) => (
                    <tr key={`g-${r.rol}-${r.machine}-${i}`}>
                      <td><span className="chip live">{r.rol}</span></td>
                      <td>{r.machine}</td>
                      <td className="mono">{r.versie}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* Stand-info */}
          {regie.stand && (
            <div className="deck-card">
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", fontSize: 12 }}>
                <span style={{ color: "var(--text-2)", display: "inline-flex", alignItems: "center", gap: 6 }}>
                  <Monitor size={13} /> Volgnummer Stand
                </span>
                <span className="mono">#{regie.stand.volgnummer}</span>
              </div>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", fontSize: 12 }}>
                <span style={{ color: "var(--text-2)" }}>Preview resolutie</span>
                <span className="mono">1280 × 720 (720p)</span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
