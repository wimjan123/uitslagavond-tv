// Coalitiebouwer-werkruimte (DaVinci Deck):
// Interactieve partijtegels + 150-zetels meerderheidsboog + live-omslagwaarschuwing + direct klaarzetten
// als Coalitie in de Zaal, Coalitiebalk of alleen Stoelen (Stoelbeeld).
import { useMemo, useState } from "react";
import { AlertTriangle, Check, Save, Send, Sparkles } from "lucide-react";
import { meerderheidSlaatOm } from "../coalitie/coalitie";
import { BOVEN_HET_PANEEL, passendeCameraKeuzes } from "../regie/camerarit";
import { useRegie } from "../regie/regie";
import { CAMERA, zetStoelbeeldKlaar } from "../regie/take";
import type { Graphic } from "../typen/Graphic";

const BIJ = "coalitie";

export function CoalitieWerkruimte() {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [setId, setSetId] = useState<string>("");
  const [vorm, setVorm] = useState<"Coalitie" | "Coalitiebalk">("Coalitie");
  const [cameraBlijft, setCameraBlijft] = useState<boolean>(false);
  const [cameraKeuze, setCameraKeuze] = useState<string>("coalitie-grens");

  const gekozenSet = useMemo(() => {
    if (!stand) return undefined;
    return stand.cijfers.sets.find((s) => s.id === setId) ?? stand.cijfers.sets[0];
  }, [stand?.cijfers.sets, setId]);

  const gekozen = useMemo(() => {
    const g = stand?.klaar?.graphic;
    if (g && (g.sjabloon === "Coalitie" || g.sjabloon === "Coalitiebalk")) return g.partijen;
    return [] as string[];
  }, [stand?.klaar]);

  const [lokaal, setLokaal] = useState<string[]>([]);
  const partijen = lokaal.length > 0 ? lokaal : gekozen;

  const cameraProfiel = vorm === "Coalitie" ? "CoalitieZaal" : "BovenPaneel";
  const passendeOpties = useMemo(() => {
    if (!stand) return [];
    return passendeCameraKeuzes(stand.standpunten, stand.cameraritten, cameraProfiel).filter((k) => k.letter !== "blijf");
  }, [stand?.standpunten, stand?.cameraritten, cameraProfiel]);
  const standaardCamera = vorm === "Coalitie" ? "coalitie-grens" : BOVEN_HET_PANEEL;
  const geldigeCamera = passendeOpties.some((o) => o.letter === cameraKeuze) ? cameraKeuze : standaardCamera;
  const effectiefStandpunt = cameraBlijft ? "blijf" : geldigeCamera;

  if (!stand) return null;

  const totaal = partijen.reduce((s, k) => s + (gekozenSet?.zetels[k] ?? 0), 0);
  const meerderheid = totaal >= 76;
  const omslag = meerderheidSlaatOm(stand);

  function wissel(sleutel: string) {
    const nieuw = partijen.includes(sleutel) ? partijen.filter((p) => p !== sleutel) : [...partijen, sleutel];
    setLokaal(nieuw);
    // Als er al een Coalitie klaarstaat, werkt ZetCoalitie direct bij
    if (stand?.klaar && (stand.klaar.graphic.sjabloon === "Coalitie" || stand.klaar.graphic.sjabloon === "Coalitiebalk")) {
      void regie.stuur({ soort: "ZetCoalitie", partijen: nieuw }, BIJ);
    }
  }

  function maakGraphic(): Graphic | null {
    if (!gekozenSet || partijen.length === 0) return null;
    return { sjabloon: vorm, set: gekozenSet.id, partijen };
  }

  return (
    <div className="work-scroll" style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 10, flexWrap: "wrap" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <select
            className="select sm"
            style={{ width: 220 }}
            value={gekozenSet?.id ?? ""}
            onChange={(e) => setSetId(e.target.value)}
          >
            {stand.cijfers.sets.map((s) => (
              <option key={s.id} value={s.id}>
                {s.label}
              </option>
            ))}
          </select>
          <div className="seg">
            <button type="button" className={vorm === "Coalitie" ? "active amber" : ""} onClick={() => setVorm("Coalitie")}>
              Coalitie in de Zaal
            </button>
            <button type="button" className={vorm === "Coalitiebalk" ? "active amber" : ""} onClick={() => setVorm("Coalitiebalk")}>
              Coalitiebalk
            </button>
          </div>
          <div className="seg" title="Bepaalt of een lopende camerabeweging blijft lopen bij Take">
            <button
              type="button"
              className={cameraBlijft ? "active preview" : ""}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => setCameraBlijft(true)}
            >
              Camera blijft lopen
            </button>
            <button
              type="button"
              className={!cameraBlijft ? "active amber" : ""}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => setCameraBlijft(false)}
            >
              {vorm === "Coalitie" ? "Coalitie & 76" : "Boven paneel"}
            </button>
          </div>
          {!cameraBlijft && (
            <select
              className="select sm"
              style={{ width: 250 }}
              value={geldigeCamera}
              onChange={(e) => setCameraKeuze(e.target.value)}
              aria-label="Passende camerabeweging voor coalitie"
            >
              {passendeOpties.map((o) => (
                <option key={o.letter} value={o.letter}>
                  {o.tekst}
                </option>
              ))}
            </select>
          )}
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <button
            type="button"
            className="btn sm"
            disabled={!gekozenSet || partijen.length === 0}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => {
              if (!gekozenSet) return;
              zetStoelbeeldKlaar(regie, {
                bron: { soort: "Coalitie", set: gekozenSet.id, partijen },
                label: `Coalitie (${totaal})`,
              });
            }}
          >
            <Sparkles size={13} /> Alleen stoelen
          </button>
          <button
            type="button"
            className="btn sm"
            disabled={!gekozenSet || partijen.length === 0}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => {
              const g = maakGraphic();
              if (g) void regie.stuur({ soort: "BewaarPreset", graphic: g, standpunt: effectiefStandpunt }, BIJ);
            }}
          >
            <Save size={13} /> Bewaar Preset
          </button>
          <button
            type="button"
            className="btn sm preview"
            disabled={!gekozenSet || partijen.length === 0}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => {
              const g = maakGraphic();
              if (g) void regie.stuur({ soort: "ZetKlaar", graphic: g, standpunt: effectiefStandpunt }, BIJ);
            }}
          >
            <Send size={13} /> Zet in Preview
          </button>
        </div>
      </div>

      {!gekozenSet && (
        <div className="deck-card" style={{ color: "var(--text-3)", textAlign: "center", padding: 24 }}>
          Geen cijfersets beschikbaar. Maak eerst een set aan bij Cijfers.
        </div>
      )}

      {omslag && (
        <div className="melding-balk warn">
          <AlertTriangle size={14} />
          <span>Let op: met de nieuwste cijfers slaat de meerderheid om. Zet opnieuw klaar in Preview voor Take.</span>
        </div>
      )}

      {regie.melding?.bij === BIJ && <div className="melding-balk">{regie.melding.tekst}</div>}

      {/* Meerderheidsbalk + grote teller */}
      <div className="deck-card">
        <div style={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 12 }}>
          <div style={{ display: "flex", alignItems: "baseline", gap: 10 }}>
            <span className="mono" style={{ fontSize: 32, fontWeight: 700, color: meerderheid ? "var(--preview-bright)" : "var(--text)" }}>
              {totaal}
            </span>
            <span style={{ fontSize: 13, color: "var(--text-2)" }}>/ 76 voor meerderheid</span>
            {meerderheid && (
              <span className="chip preview">
                <Check size={12} /> Meerderheid (+{totaal - 75})
              </span>
            )}
          </div>
          {partijen.length > 0 && (
            <button type="button" className="btn sm ghost" onClick={() => setLokaal([])}>
              Wis selectie
            </button>
          )}
        </div>

        {/* Gekleurde zetelbalk met 76-lijn (schaalt veilig mee als totaal > 150) */}
        <div style={{ position: "relative", height: 18, borderRadius: 4, background: "var(--bg-input)", overflow: "hidden", display: "flex" }}>
          {partijen.map((sleutel) => {
            const p = stand.cijfers.partijen.find((x) => x.sleutel === sleutel);
            const z = gekozenSet?.zetels[sleutel] ?? 0;
            if (!p || z <= 0) return null;
            const schaal = Math.max(150, totaal);
            return (
              <div
                key={sleutel}
                style={{ width: `${(z / schaal) * 100}%`, background: p.kleur, height: "100%" }}
                title={`${p.naam}: ${z} zetels`}
              />
            );
          })}
          <div style={{ position: "absolute", left: `${(76 / Math.max(150, totaal)) * 100}%`, top: 0, bottom: 0, width: 2, background: "#fff", opacity: 0.85 }} />
        </div>
      </div>

      {/* Partijtegels */}
      <div className="partij-grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(150px, 1fr))" }}>
        {stand.cijfers.partijen
          .filter((p) => (gekozenSet?.zetels[p.sleutel] ?? 0) > 0)
          .sort((a, b) => (gekozenSet?.zetels[b.sleutel] ?? 0) - (gekozenSet?.zetels[a.sleutel] ?? 0))
          .map((p) => {
            const z = gekozenSet?.zetels[p.sleutel] ?? 0;
            const aan = partijen.includes(p.sleutel);
            return (
              <button
                key={p.sleutel}
                type="button"
                className={`partij-btn${aan ? " active" : ""}`}
                style={{ justifyContent: "space-between", padding: "9px 10px" }}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => wissel(p.sleutel)}
              >
                <span style={{ display: "inline-flex", alignItems: "center", gap: 8, minWidth: 0 }}>
                  <span className="partij-swatch" style={{ background: p.kleur, width: 12, height: 12 }} />
                  <span style={{ fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {p.kort || p.naam}
                  </span>
                </span>
                <span className="mono" style={{ fontWeight: 700, fontSize: 13 }}>{z}</span>
              </button>
            );
          })}
      </div>
    </div>
  );
}

export { CAMERA };
