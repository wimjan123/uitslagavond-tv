// Cijfers-werkruimte (DaVinci Deck):
// 3-koloms snelle zetelinvoer (Enter bevestigt, Esc herstelt), vergrendelen, nieuwe set aanmaken,
// multi-Werkplek bewerkingsmelding en Partijen-beheer.
import { useEffect, useMemo, useState } from "react";
import { Lock, Plus, Unlock, Users } from "lucide-react";
import { useRegie } from "../regie/regie";
import type { Cijferset } from "../typen/Cijferset";
import type { Soort } from "../typen/Soort";

const BIJ = "cijfers";

function som(set: Cijferset): number {
  return Object.values(set.zetels).reduce((a, b) => a + (b ?? 0), 0);
}

export function CijfersWerkruimte() {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [sub, setSub] = useState<"sets" | "partijen">("sets");
  const [setId, setSetId] = useState<string>("");
  const [nieuwOpen, setNieuwOpen] = useState(false);
  const [nieuwLabel, setNieuwLabel] = useState("");
  const [nieuwSoort, setNieuwSoort] = useState<Soort>("prognose");
  const [nieuwePartijNaam, setNieuwePartijNaam] = useState("");
  const [nieuwePartijKort, setNieuwePartijKort] = useState("");
  const [nieuwePartijKleur, setNieuwePartijKleur] = useState("#3B82F6");

  const actief: Cijferset | undefined = useMemo(() => {
    if (!stand) return undefined;
    return stand.cijfers.sets.find((s) => s.id === setId) ?? stand.cijfers.sets[0];
  }, [stand?.cijfers.sets, setId]);

  useEffect(() => {
    regie.bewerk(sub === "sets" ? actief?.id ?? null : null);
    return () => regie.bewerk(null);
  }, [sub, actief?.id]);

  if (!stand) return null;

  const totaal = actief ? som(actief) : 0;
  const anderen = regie.bewerkingen.filter((b) => b.set === actief?.id);

  async function maakSet() {
    const label = nieuwLabel.trim();
    if (!label) return;
    const a = await regie.stuur({ soort: "MaakSet", setsoort: nieuwSoort, label }, BIJ);
    if (a.uitgevoerd) {
      setNieuwLabel("");
      setNieuwOpen(false);
    }
  }

  async function voegPartijToe() {
    const naam = nieuwePartijNaam.trim();
    if (!naam) return;
    const a = await regie.stuur(
      {
        soort: "VoegPartijToe",
        naam,
        kort: nieuwePartijKort.trim() || naam.slice(0, 6).toUpperCase(),
        kleur: nieuwePartijKleur,
        fractie_zonder_lijst: false,
      },
      BIJ,
    );
    if (a.uitgevoerd) {
      setNieuwePartijNaam("");
      setNieuwePartijKort("");
    }
  }

  return (
    <div style={{ flex: 1, minHeight: 0, display: "flex", flexDirection: "column" }}>
      <div className="studio-subbar">
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <div className="seg">
            <button type="button" className={sub === "sets" ? "active amber" : ""} onClick={() => setSub("sets")}>
              Zetelsets ({stand.cijfers.sets.length})
            </button>
            <button type="button" className={sub === "partijen" ? "active amber" : ""} onClick={() => setSub("partijen")}>
              Partijen ({stand.cijfers.partijen.length})
            </button>
          </div>
          {sub === "sets" && actief && (
            <select
              className="select sm"
              style={{ width: 220 }}
              value={actief.id}
              onChange={(e) => setSetId(e.target.value)}
            >
              {stand.cijfers.sets.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.label} ({som(s)})
                </option>
              ))}
            </select>
          )}
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          {sub === "sets" && actief && (
            <>
              <span className={`chip ${totaal === 150 ? "preview" : "warn"} mono`}>
                {totaal} / 150
              </span>
              <button
                type="button"
                className="btn sm"
                onClick={() =>
                  void regie.stuur(
                    { soort: actief.vergrendeld ? "OntgrendelSet" : "VergrendelSet", set: actief.id },
                    BIJ,
                  )
                }
              >
                {actief.vergrendeld ? <Lock size={12} /> : <Unlock size={12} />}
                {actief.vergrendeld ? "Vergrendeld" : "Vergrendel"}
              </button>
              <button type="button" className="btn sm amber" onClick={() => setNieuwOpen((o) => !o)}>
                <Plus size={13} /> Nieuwe set
              </button>
            </>
          )}
        </div>
      </div>

      {nieuwOpen && sub === "sets" && (
        <div style={{ display: "flex", gap: 8, padding: "8px 12px", background: "var(--bg-well)", borderBottom: "1px solid var(--hairline)" }}>
          <input
            className="input sm"
            style={{ maxWidth: 260 }}
            value={nieuwLabel}
            onChange={(e) => setNieuwLabel(e.target.value)}
            placeholder="Bijv. Prognose 22:00"
            onKeyDown={(e) => e.key === "Enter" && void maakSet()}
            autoFocus
          />
          <select className="select sm" style={{ width: 140 }} value={nieuwSoort} onChange={(e) => setNieuwSoort(e.target.value as Soort)}>
            <option value="peiling">Peiling</option>
            <option value="exitpoll">Exitpoll</option>
            <option value="prognose">Prognose</option>
            <option value="uitslag">Uitslag</option>
          </select>
          <button type="button" className="btn sm amber" onClick={() => void maakSet()}>
            Maak set
          </button>
        </div>
      )}

      {regie.melding?.bij === BIJ && (
        <div style={{ padding: "8px 12px 0" }}>
          <div className="melding-balk">{regie.melding.tekst}</div>
        </div>
      )}

      {sub === "sets" ? (
        <div className="work-scroll" style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          {anderen.length > 0 && (
            <div className="melding-balk warn">
              <Users size={14} />
              <span>Ook open op: {anderen.map((a) => a.werkplek).join(", ")}</span>
            </div>
          )}

          {actief ? (
            <div className="cijfers-cols">
              {stand.cijfers.partijen.map((p) => (
                <ZetelRij
                  key={`${actief.id}-${p.sleutel}`}
                  naam={p.kort || p.naam}
                  volledig={p.naam}
                  kleur={p.kleur}
                  waarde={actief.zetels[p.sleutel] ?? 0}
                  vergrendeld={actief.vergrendeld}
                  opZet={(n) => void regie.stuur({ soort: "ZetZetels", set: actief.id, partij: p.sleutel, zetels: n }, BIJ)}
                />
              ))}
            </div>
          ) : (
            <div style={{ padding: 24, color: "var(--text-3)" }}>Geen cijfersets beschikbaar.</div>
          )}
        </div>
      ) : (
        <div className="work-scroll" style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <div className="deck-card" style={{ flexDirection: "row", alignItems: "flex-end", flexWrap: "wrap", gap: 8 }}>
            <div className="field" style={{ flex: 1, minWidth: 160 }}>
              <label>Partijnaam</label>
              <input className="input sm" value={nieuwePartijNaam} onChange={(e) => setNieuwePartijNaam(e.target.value)} placeholder="Bijv. Nieuw Sociaal Contract" />
            </div>
            <div className="field" style={{ width: 100 }}>
              <label>Kort</label>
              <input className="input sm" value={nieuwePartijKort} onChange={(e) => setNieuwePartijKort(e.target.value)} placeholder="NSC" />
            </div>
            <div className="field">
              <label>Kleur</label>
              <input type="color" value={nieuwePartijKleur} onChange={(e) => setNieuwePartijKleur(e.target.value)} style={{ width: 34, height: 26, border: 0, background: "transparent" }} />
            </div>
            <button type="button" className="btn sm amber" onClick={() => void voegPartijToe()}>
              <Plus size={13} /> Partij toevoegen
            </button>
          </div>

          <table className="deck-table">
            <thead>
              <tr>
                <th>Kleur</th>
                <th>Kort</th>
                <th>Naam</th>
                <th>Sleutel</th>
              </tr>
            </thead>
            <tbody>
              {stand.cijfers.partijen.map((p) => (
                <tr key={p.sleutel}>
                  <td>
                    <input
                      type="color"
                      value={p.kleur}
                      onChange={(e) =>
                        void regie.stuur(
                          { soort: "VulPartijAan", partij: p.sleutel, aanvulling: { kleur: e.target.value } },
                          BIJ,
                        )
                      }
                      style={{ width: 26, height: 22, border: 0, background: "transparent", cursor: "pointer" }}
                    />
                  </td>
                  <td style={{ fontWeight: 600 }}>{p.kort}</td>
                  <td>{p.naam}</td>
                  <td className="mono" style={{ color: "var(--text-3)" }}>{p.sleutel}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function ZetelRij({
  naam,
  volledig,
  kleur,
  waarde,
  vergrendeld,
  opZet,
}: {
  naam: string;
  volledig: string;
  kleur: string;
  waarde: number;
  vergrendeld: boolean;
  opZet: (n: number) => void;
}) {
  const [tekst, setTekst] = useState(String(waarde));
  useEffect(() => setTekst(String(waarde)), [waarde]);

  function bewaar() {
    const n = Number.parseInt(tekst, 10);
    if (Number.isFinite(n) && n >= 0 && n <= 150 && n !== waarde) opZet(n);
    else setTekst(String(waarde));
  }

  return (
    <div className="cijfers-row" title={volledig}>
      <span className="partij-swatch" style={{ background: kleur }} />
      <span style={{ fontWeight: 500, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{naam}</span>
      <input
        className="input sm cijfers-input"
        type="number"
        min={0}
        max={150}
        disabled={vergrendeld}
        value={tekst}
        onChange={(e) => setTekst(e.target.value)}
        onBlur={bewaar}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            bewaar();
          } else if (e.key === "Escape") {
            setTekst(String(waarde));
            (e.target as HTMLInputElement).blur();
          }
        }}
      />
      <span className="mono" style={{ fontSize: 11, color: "var(--text-3)", textAlign: "right" }}>zet.</span>
    </div>
  );
}
