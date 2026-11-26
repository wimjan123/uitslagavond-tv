// Stemmingen-werkruimte (DaVinci Deck):
// Behoudt de "volg de Voorzitter"-flow (grote Fractie-tegels, kleefbare uitkomstbalk, Ctrl+Enter
// vastleggen & klaarzetten, Controle-weergave) in een rustige, overzichtelijke deck-stijl.
import { useEffect, useMemo, useState } from "react";
import { Check, CheckCircle2, Lock, Plus, Send, Unlock, X } from "lucide-react";
import { stemVan } from "../stemmingen/invoer";
import { controleLijst, STATUS_TEKST } from "../stemmingen/controle";
import { useRegie } from "../regie/regie";
import type { Fractie } from "../typen/Fractie";
import type { Stem } from "../typen/Stem";
import type { Stemming } from "../typen/Stemming";
import type { Stemgedrag } from "../typen/Stemgedrag";

const BIJ = "stemmingen";

function berekenUitkomst(sg: Stemgedrag, fracties: Fractie[]): { voor: number; tegen: number; open: number; besluit: "Aangenomen" | "Verworpen" | null } {
  let voor = 0;
  let tegen = 0;
  let open = 0;
  for (const f of fracties) {
    const s = stemVan(sg, f.id)?.stem;
    if (s === "Voor") voor += f.zetels;
    else if (s === "Tegen") tegen += f.zetels;
    else open++;
  }
  const besluit = open === 0 && (voor > 0 || tegen > 0) ? (voor > tegen ? "Aangenomen" : "Verworpen") : null;
  return { voor, tegen, open, besluit };
}

export function StemmingenWerkruimte() {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [sub, setSub] = useState<"invoer" | "controle">("invoer");
  const [gekozenId, setGekozenId] = useState<string>("");
  const [alleenUitzending, setAlleenUitzending] = useState(false);
  const [nieuweTitel, setNieuweTitel] = useState("");
  const [nieuwOpen, setNieuwOpen] = useState(false);
  const [titelOpen, setTitelOpen] = useState(false);

  const lijst = useMemo(() => {
    if (!stand) return [];
    return stand.stemmingen.lijst.filter((s) => !alleenUitzending || s.voor_uitzending);
  }, [stand?.stemmingen.lijst, alleenUitzending]);

  const actief: Stemming | undefined = useMemo(
    () => lijst.find((s) => s.punt.id === gekozenId) ?? lijst[0],
    [lijst, gekozenId],
  );

  useEffect(() => {
    if (actief && actief.punt.id !== gekozenId) setGekozenId(actief.punt.id);
  }, [actief?.punt.id]);

  // Ctrl+Enter = Vastleggen en klaarzetten
  useEffect(() => {
    if (!actief || sub !== "invoer") return;
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        void regie.stuur({ soort: "LegStemmingVast", stemming: actief.punt.id }, BIJ);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [actief?.punt.id, sub, regie]);

  if (!stand) return null;

  const fracties = stand.kamer.fracties;
  const resultaat = actief ? berekenUitkomst(actief.stemgedrag, fracties) : null;

  function kleurVanFractie(fractieId: string): string {
    const partijSleutel = stand?.kamer.partij_van[fractieId];
    return stand?.cijfers.partijen.find((p) => p.sleutel === partijSleutel)?.kleur ?? "#64748b";
  }

  function zetFractieStem(s: Stemming, fractieId: string, stem: Stem | null) {
    if (s.vergrendeld) return;
    const zonder = s.stemgedrag.getikt.filter((g) => g.fractie !== fractieId);
    const getikt = stem ? [...zonder, { fractie: fractieId, stem }] : zonder;
    const nieuw: Stemgedrag = { ...s.stemgedrag, getikt };
    void regie.stuur({ soort: "ZetStemgedrag", stemming: s.punt.id, stemgedrag: nieuw }, BIJ);
  }

  function zetOverige(s: Stemming, stem: Stem | null) {
    if (s.vergrendeld) return;
    const nieuw: Stemgedrag = { ...s.stemgedrag, overige: stem, algemene_stemmen: false };
    void regie.stuur({ soort: "ZetStemgedrag", stemming: s.punt.id, stemgedrag: nieuw }, BIJ);
  }

  function zetAlgemeneStemmen(s: Stemming) {
    if (s.vergrendeld) return;
    const nieuw: Stemgedrag = { ...s.stemgedrag, getikt: [], overige: null, afwijkend: [], algemene_stemmen: !s.stemgedrag.algemene_stemmen };
    void regie.stuur({ soort: "ZetStemgedrag", stemming: s.punt.id, stemgedrag: nieuw }, BIJ);
  }

  async function voegToe() {
    const t = nieuweTitel.trim();
    if (!t) return;
    const a = await regie.stuur({ soort: "VoegStemmingToe", titel: t }, BIJ);
    if (a.uitgevoerd) {
      setNieuweTitel("");
      setNieuwOpen(false);
    }
  }

  return (
    <div style={{ flex: 1, minHeight: 0, display: "flex", flexDirection: "column" }}>
      <div className="studio-subbar">
        <div className="seg">
          <button type="button" className={sub === "invoer" ? "active amber" : ""} onClick={() => setSub("invoer")}>
            Stemmingen invoeren
          </button>
          <button type="button" className={sub === "controle" ? "active amber" : ""} onClick={() => setSub("controle")}>
            Controle na afloop
          </button>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <div className="seg" title="Zaalindeling voor stemmingsgraphics">
            <button
              type="button"
              className={stand.stemmingen.indeling === "Zaalindeling" ? "active amber" : ""}
              onClick={() => void regie.stuur({ soort: "ZetStemmingIndeling", indeling: "Zaalindeling" }, BIJ)}
            >
              Echte stoelen
            </button>
            <button
              type="button"
              className={stand.stemmingen.indeling === "Uitslagindeling" ? "active amber" : ""}
              onClick={() => void regie.stuur({ soort: "ZetStemmingIndeling", indeling: "Uitslagindeling" }, BIJ)}
            >
              Uitslagindeling
            </button>
          </div>
          <button
            type="button"
            className="btn sm"
            onClick={() => setNieuwOpen((o) => !o)}
          >
            <Plus size={13} /> Eigen stemming
          </button>
        </div>
      </div>

      {nieuwOpen && (
        <div style={{ display: "flex", gap: 8, padding: "8px 12px", background: "var(--bg-well)", borderBottom: "1px solid var(--hairline)" }}>
          <input
            className="input sm"
            value={nieuweTitel}
            onChange={(e) => setNieuweTitel(e.target.value)}
            placeholder="Titel van de motie of stemming…"
            onKeyDown={(e) => e.key === "Enter" && void voegToe()}
            autoFocus
          />
          <button type="button" className="btn sm amber" onClick={() => void voegToe()}>
            Toevoegen
          </button>
          <button type="button" className="btn sm ghost" onClick={() => setNieuwOpen(false)}>
            <X size={13} />
          </button>
        </div>
      )}

      {regie.melding?.bij === BIJ && (
        <div style={{ padding: "8px 12px 0" }}>
          <div className="melding-balk">{regie.melding.tekst}</div>
        </div>
      )}

      {sub === "invoer" ? (
        <div className="work-split">
          <aside className="work-sidebar">
            <div className="pane-head">
              <span className="pane-title">Stemmingslijst ({lijst.length})</span>
              <button
                type="button"
                className={`btn sm${alleenUitzending ? " active" : " ghost"}`}
                onClick={() => setAlleenUitzending((v) => !v)}
              >
                Uitzending
              </button>
            </div>
            <div className="work-list">
              {lijst.map((s) => {
                const u = berekenUitkomst(s.stemgedrag, fracties);
                const isActief = actief?.punt.id === s.punt.id;
                return (
                  <button
                    key={s.punt.id}
                    type="button"
                    className={`work-list-item${isActief ? " active" : ""}`}
                    onClick={() => setGekozenId(s.punt.id)}
                  >
                    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 6 }}>
                      <span style={{ fontSize: 12, fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {s.punt.titel}
                      </span>
                      {s.vergrendeld && <Lock size={11} color="var(--text-3)" />}
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 11, color: "var(--text-3)" }}>
                      <span className="mono">{u.voor} voor · {u.tegen} tegen</span>
                      {u.besluit && (
                        <span className={`chip ${u.besluit === "Aangenomen" ? "preview" : "live"}`} style={{ height: 18, fontSize: 10 }}>
                          {u.besluit}
                        </span>
                      )}
                    </div>
                  </button>
                );
              })}
              {lijst.length === 0 && (
                <div style={{ padding: 16, color: "var(--text-3)", fontSize: 12 }}>Geen stemmingen geladen.</div>
              )}
            </div>
          </aside>

          <div className="work-scroll" style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {actief && resultaat ? (
              <>
                {/* Sticky uitkomst-header */}
                <div className="deck-card" style={{ position: "sticky", top: 0, zIndex: 5 }}>
                  <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 12 }}>
                    <div style={{ minWidth: 0, flex: 1 }}>
                      <div
                        className={`motie-titel-clamp${titelOpen ? " open" : ""}`}
                        title={actief.punt.titel}
                        onClick={() => setTitelOpen((v) => !v)}
                        style={{ cursor: "pointer" }}
                      >
                        {actief.punt.titel}
                      </div>
                      {actief.punt.indieners && (
                        <div style={{ fontSize: 11.5, color: "var(--text-3)", marginTop: 2, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={actief.punt.indieners}>
                          {actief.punt.indieners}
                        </div>
                      )}
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: 6, flexShrink: 0 }}>
                      <button
                        type="button"
                        className={`btn sm${actief.voor_uitzending ? " active" : ""}`}
                        onClick={() => void regie.stuur({ soort: "ZetVoorUitzending", stemming: actief.punt.id, aan: !actief.voor_uitzending }, BIJ)}
                      >
                        {actief.voor_uitzending ? <Check size={12} /> : null} Uitzending
                      </button>
                      <button
                        type="button"
                        className="btn sm"
                        onClick={() =>
                          void regie.stuur(
                            { soort: actief.vergrendeld ? "OntgrendelStemming" : "VergrendelStemming", stemming: actief.punt.id },
                            BIJ,
                          )
                        }
                      >
                        {actief.vergrendeld ? <Lock size={12} /> : <Unlock size={12} />}
                        {actief.vergrendeld ? "Ontgrendel" : "Slot"}
                      </button>
                      <button
                        type="button"
                        className="btn sm preview"
                        disabled={!resultaat.besluit}
                        onClick={() => void regie.stuur({ soort: "LegStemmingVast", stemming: actief.punt.id }, BIJ)}
                      >
                        <Send size={12} />
                        Vastleggen & klaarzetten
                        <kbd className="key">Ctrl+↵</kbd>
                      </button>
                    </div>
                  </div>

                  {/* Voortgangsbalk Voor / Tegen */}
                  <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
                    <div style={{ display: "flex", justifyContent: "space-between", fontSize: 12 }}>
                      <span className="mono" style={{ color: "#29b894", fontWeight: 700 }}>{resultaat.voor} voor</span>
                      <span style={{ fontWeight: 600 }}>
                        {resultaat.besluit ??
                          (resultaat.open === 1 ? "1 fractie open" : `${resultaat.open} fracties open`)}
                      </span>
                      <span className="mono" style={{ color: "#e5484d", fontWeight: 700 }}>{resultaat.tegen} tegen</span>
                    </div>
                    <div className="stem-bar">
                      <div className="stem-bar-voor" style={{ width: `${(resultaat.voor / 150) * 100}%` }} />
                      <div className="stem-bar-tegen" style={{ width: `${(resultaat.tegen / 150) * 100}%`, marginLeft: "auto" }} />
                      <div className="stem-bar-mid" />
                    </div>
                  </div>

                  {/* Snelle "Overige" acties */}
                  <div style={{ display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
                    <button
                      type="button"
                      className={`btn sm${actief.stemgedrag.overige === "Voor" ? " preview" : ""}`}
                      disabled={actief.vergrendeld}
                      onClick={() => zetOverige(actief, actief.stemgedrag.overige === "Voor" ? null : "Voor")}
                    >
                      Rest voor
                    </button>
                    <button
                      type="button"
                      className={`btn sm${actief.stemgedrag.overige === "Tegen" ? " live" : ""}`}
                      disabled={actief.vergrendeld}
                      onClick={() => zetOverige(actief, actief.stemgedrag.overige === "Tegen" ? null : "Tegen")}
                    >
                      Rest tegen
                    </button>
                    <button
                      type="button"
                      className={`btn sm${actief.stemgedrag.algemene_stemmen ? " preview" : ""}`}
                      disabled={actief.vergrendeld}
                      onClick={() => zetAlgemeneStemmen(actief)}
                    >
                      Algemene stemmen
                    </button>
                  </div>
                </div>

                {/* Fractietegels: links klik = Voor, rechts of knop = Tegen */}
                <div className="stem-tiles">
                  {fracties.map((f) => {
                    const stem = stemVan(actief.stemgedrag, f.id)?.stem;
                    const direct = actief.stemgedrag.getikt.find((g) => g.fractie === f.id)?.stem;
                    return (
                      <div key={f.id} className={`stem-tile${stem === "Voor" ? " voor" : stem === "Tegen" ? " tegen" : ""}`}>
                        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 6 }}>
                          <span style={{ display: "inline-flex", alignItems: "center", gap: 6, fontWeight: 600, fontSize: 12.5 }}>
                            <span className="partij-swatch" style={{ background: kleurVanFractie(f.id) }} />
                            {f.afkorting}
                          </span>
                          <span className="mono" style={{ fontSize: 11.5, color: "var(--text-2)" }}>{f.zetels}</span>
                        </div>
                        <div style={{ display: "flex", gap: 4 }}>
                          <button
                            type="button"
                            className={`btn sm${direct === "Voor" ? " preview" : ""}`}
                            style={{ flex: 1, height: 24, fontSize: 11 }}
                            disabled={actief.vergrendeld}
                            onClick={() => zetFractieStem(actief, f.id, direct === "Voor" ? null : "Voor")}
                          >
                            Voor
                          </button>
                          <button
                            type="button"
                            className={`btn sm${direct === "Tegen" ? " live" : ""}`}
                            style={{ flex: 1, height: 24, fontSize: 11 }}
                            disabled={actief.vergrendeld}
                            onClick={() => zetFractieStem(actief, f.id, direct === "Tegen" ? null : "Tegen")}
                          >
                            Tegen
                          </button>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </>
            ) : (
              <div style={{ padding: 24, color: "var(--text-3)" }}>Kies links een stemming om in te voeren.</div>
            )}
          </div>
        </div>
      ) : (
        <div className="work-scroll">
          <table className="deck-table" style={{ tableLayout: "fixed" }}>
            <thead>
              <tr>
                <th style={{ width: "44%" }}>Stemming</th>
                <th style={{ width: 96 }}>Onze invoer</th>
                <th style={{ width: 110 }}>Officieel</th>
                <th style={{ width: 150 }}>Status</th>
                <th>Actie</th>
              </tr>
            </thead>
            <tbody>
              {controleLijst(stand.stemmingen.lijst).map((s) => {
                const st = s.controle.status;
                const u = berekenUitkomst(s.stemgedrag, fracties);
                const off = s.controle.officieel;
                const heeftVerschil = st === "UitkomstAnders" || st === "VerschilPerFractie";
                return (
                  <tr key={s.punt.id}>
                    <td style={{ fontWeight: 500, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={s.punt.titel}>
                      {s.punt.titel}
                    </td>
                    <td className="mono" style={{ whiteSpace: "nowrap" }}>{`${u.voor} / ${u.tegen}`}</td>
                    <td className="mono" style={{ whiteSpace: "nowrap" }}>{off ? off.besluit : "—"}</td>
                    <td>
                      <span className={`chip ${st === "Gelijk" ? "preview" : heeftVerschil ? "live" : ""}`}>
                        {st === "Gelijk" && <CheckCircle2 size={11} />}
                        {st ? STATUS_TEKST[st] : "—"}
                      </span>
                    </td>
                    <td>
                      {heeftVerschil && !s.controle.afgehandeld && (
                        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                          <button
                            type="button"
                            className="btn sm"
                            onClick={() => void regie.stuur({ soort: "NeemOfficieelOver", stemming: s.punt.id }, BIJ)}
                          >
                            Neem officieel over
                          </button>
                          <button
                            type="button"
                            className="btn sm ghost"
                            onClick={() => void regie.stuur({ soort: "OnzeInvoerKlopt", stemming: s.punt.id }, BIJ)}
                          >
                            Onze invoer klopt
                          </button>
                        </div>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
