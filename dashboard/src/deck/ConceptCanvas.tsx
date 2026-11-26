// Privé-Conceptbeeld (16:9) over het echte 3D-Zaalbeeld uit de server (/api/zaalbeeld) of de
// live Preview-stream. Gebruikt de echte Rust-blokberekening (/api/conceptbeeld) zonder ooit de
// gedeelde Stand aan te raken (ADR 0001 / ADR 0005).
import { useEffect, useState } from "react";
import type { Graphic } from "../typen/Graphic";
import type { Sjabloon } from "../typen/Sjabloon";
import type { Stijl } from "../typen/Stijl";
import type { Stoelbeeld } from "../typen/Stoelbeeld";
import { useRegie } from "../regie/regie";
import { Schets } from "../presets/Schets";

interface ConceptCanvasProps {
  graphic?: Graphic;
  stoelbeeld?: Stoelbeeld;
  sjabloon?: Sjabloon;
  stijl?: Stijl;
  label?: string;
}

export function ConceptCanvas({ graphic, stoelbeeld, sjabloon, stijl, label = "Concept" }: ConceptCanvasProps) {
  const regie = useRegie();
  const stand = regie.stand?.stand;
  const [zaalUrl, setZaalUrl] = useState<string | null>(null);
  const [stoelkleuren, setStoelkleuren] = useState<string[]>([]);
  const [fout, setFout] = useState<string | null>(null);

  // Haal het echte 720p 3D-Zaalbeeld op als achtergrond (geen nep-blauwe gradiënt)
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

  // Privé server-validatie + stoelkleuren voor Stoelbeeld
  useEffect(() => {
    if (!graphic && !stoelbeeld && !sjabloon) {
      setStoelkleuren([]);
      setFout(null);
      return;
    }
    let actief = true;
    const t = window.setTimeout(() => {
      void regie.haalConceptbeeld({ graphic, stoelbeeld, sjabloon, stijl }).then((res) => {
        if (!actief || !res) return;
        setFout(res.fout ?? null);
        setStoelkleuren(res.stoelkleuren ?? []);
      });
    }, 120);
    return () => {
      actief = false;
      window.clearTimeout(t);
    };
  }, [JSON.stringify(graphic), JSON.stringify(stoelbeeld), sjabloon?.id, JSON.stringify(stijl)]);

  const bg = zaalUrl ?? regie.beelden.Ether ?? regie.beelden.Voorbeeld;
  const kleuren = stijl?.kleuren ?? stand?.stijl.kleuren;

  return (
    <div className="concept-box">
      <span className="concept-tag">{label}</span>
      {bg && <img className="concept-bg-img" src={bg} alt="" draggable={false} />}

      {/* Als alleen een Stoelbeeld actief is, tonen we de stoelkleuren als halve cirkel over de Zaal */}
      {!graphic && stoelkleuren.length > 0 && (
        <svg viewBox="0 0 320 180" style={{ position: "absolute", inset: 0, width: "100%", height: "100%", zIndex: 2 }}>
          <rect width="320" height="180" fill={bg ? "rgba(13,18,32,0.45)" : "#0d1220"} />
          {stoelStippen(stoelkleuren).map((s, i) => (
            <circle key={i} cx={s.x} cy={s.y} r="3" fill={s.kleur} />
          ))}
        </svg>
      )}

      {/* Graphic-preview (met echte Stijl-kleuren) over de Zaal */}
      {graphic && kleuren && (
        <div style={{ position: "absolute", inset: 0, zIndex: 2, opacity: bg ? 0.94 : 1 }}>
          <Schets graphic={graphic} kleuren={kleuren} />
        </div>
      )}

      {!graphic && stoelkleuren.length === 0 && !bg && (
        <div className="monitor-placeholder">
          <span>3D-Zaalbeeld volgt zodra Voorbeeld draait</span>
        </div>
      )}

      {fout && (
        <div
          style={{
            position: "absolute",
            left: 6,
            right: 6,
            bottom: 6,
            zIndex: 5,
            padding: "4px 8px",
            borderRadius: 4,
            background: "rgba(201, 55, 61, 0.9)",
            color: "#fff",
            fontSize: 11,
          }}
        >
          {fout}
        </div>
      )}
    </div>
  );
}

function stoelStippen(kleuren: string[]): { x: number; y: number; kleur: string }[] {
  const stippen: { x: number; y: number; hoek: number }[] = [];
  for (const [rij, r] of [66, 84, 102, 120].entries()) {
    const n = 14 + rij * 4;
    for (let i = 0; i < n; i++) {
      const hoek = Math.PI * (0.08 + (0.84 * i) / (n - 1));
      stippen.push({ x: 160 - r * Math.cos(hoek), y: 186 - r * Math.sin(hoek) * 0.62, hoek });
    }
  }
  const volgorde = stippen.map((_, i) => i).sort((a, b) => stippen[a].hoek - stippen[b].hoek);
  return volgorde.map((idx, j) => ({
    x: stippen[idx].x,
    y: stippen[idx].y,
    kleur: kleuren[Math.floor((j * kleuren.length) / stippen.length)] ?? "#4c4f55",
  }));
}
