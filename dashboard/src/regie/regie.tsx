// De toestand van één Werkplek: de verbinding, de Stand, de Renderers, de 720p beelden
// en het privé-Conceptbeeld per Werkplek (ADR 0001 / ADR 0005).
import { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import type { BerichtVanServer } from "../typen/BerichtVanServer";
import type { Bewerking } from "../typen/Bewerking";
import type { Camerabeeld } from "../typen/Camerabeeld";
import type { GenummerdeStand } from "../typen/GenummerdeStand";
import type { Graphic } from "../typen/Graphic";
import type { Opdracht } from "../typen/Opdracht";
import type { Rol } from "../typen/Rol";
import type { Sjabloon } from "../typen/Sjabloon";
import type { Stijl } from "../typen/Stijl";
import type { Stoelbeeld } from "../typen/Stoelbeeld";
import { Verbinding } from "../verbinding";
import { GEEN_RENDERERS } from "./systeem";
import type { Renderermelding } from "./systeem";
import type { Antwoord, Monitor, Staat } from "../verbinding";
import {
  bouwBreakUiStand,
  BREAK_UI_MODI,
  leesBreakUiModus,
  zetBreakUiModusInUrl,
} from "../deck/breakUiFixture";
import type { BreakUiModus } from "../deck/breakUiFixture";

export interface Toegang {
  sleutel: string;
  werkplek: string;
  /** De tablet van de Presentator: de server laat hem alleen klaarzetten. */
  tablet?: boolean;
}

/** Een weigering van de server, bij de knop die hem kreeg. */
export interface Melding {
  bij: string;
  tekst: string;
}

export interface ConceptVraag {
  graphic?: Graphic;
  stoelbeeld?: Stoelbeeld;
  sjabloon?: Sjabloon;
  stijl?: Stijl;
}

export interface ConceptAntwoord {
  blokken: unknown[];
  stoelkleuren: string[];
  fout?: string;
}

export interface Regie {
  sleutel: string;
  werkplek: string;
  staat: Staat;
  stand: GenummerdeStand | null;
  renderers: Rol[];
  rendererstatus: Renderermelding;
  bewerkingen: Bewerking[];
  beelden: Record<Monitor, string | null>;
  melding: Melding | null;
  stuur(opdracht: Opdracht, bij: string): Promise<Antwoord>;
  bewerk(set: string | null): void;
  stuurFoto(id: string, foto: Blob | null, bij: string): Promise<Antwoord>;
  haalFoto(id: string): Promise<string | null>;
  plaatsBestand(bestand: File, bij: string): Promise<string | null>;
  haalBestand(naam: string): Promise<string | null>;
  /** Haalt het laatste 720p 3D-Zaalbeeld op voor de Studio-editor en het Conceptbeeld. */
  haalZaalbeeld(monitor?: "ether" | "voorbeeld"): Promise<string | null>;
  /** Rekent een privé-Conceptbeeld uit op de server zonder de gedeelde Stand te raken. */
  haalConceptbeeld(vraag: ConceptVraag): Promise<ConceptAntwoord | null>;
  afmelden(): void;
}

const RegieContext = createContext<Regie | null>(null);
const CameraContext = createContext<Camerabeeld | null>(null);

export function useCamera(): Camerabeeld | null {
  return useContext(CameraContext);
}

export function useRegie(): Regie {
  const r = useContext(RegieContext);
  if (!r) throw new Error("useRegie buiten RegieBron");
  return r;
}

const MELDING_DUUR = 6000;

export function RegieBron({ toegang, afmelden, children }: { toegang: Toegang; afmelden: () => void; children: ReactNode }) {
  const [staat, setStaat] = useState<Staat>("verbinden");
  const [stand, setStand] = useState<GenummerdeStand | null>(null);
  const [rendererstatus, setRendererstatus] = useState<Renderermelding>(GEEN_RENDERERS);
  const [bewerkingen, setBewerkingen] = useState<Bewerking[]>([]);
  const [beelden, setBeelden] = useState<Record<Monitor, string | null>>({ Voorbeeld: null, Ether: null });
  const [melding, setMelding] = useState<Melding | null>(null);
  const [camera, setCamera] = useState<Camerabeeld | null>(null);
  const [breakModus, setBreakModus] = useState<BreakUiModus | null>(() => leesBreakUiModus());
  const verbinding = useRef<Verbinding | null>(null);
  const meldingTijd = useRef<number | undefined>(undefined);

  useEffect(() => {
    const afzender = { soort: toegang.tablet ? "Tablet" : "Werkplek", naam: toegang.werkplek } as const;
    const v = new Verbinding(toegang.sleutel, afzender, {
      staat: (st) => {
        setStaat(st);
        if (st !== "verbonden") {
          setBewerkingen([]);
          setRendererstatus(GEEN_RENDERERS);
        }
      },
      bericht: (b) => {
        if (b.soort === "Stand") setStand({ volgnummer: b.volgnummer, stand: b.stand });
        else if (b.soort === "Renderers") setRendererstatus({ versie: b.versie, verbonden: b.verbonden, geweigerd: b.geweigerd });
        else if (b.soort === "Bewerkingen") setBewerkingen(b.open);
        else if (b.soort === "Camera") {
          const { soort: _, ...beeld } = b;
          setCamera(beeld);
        }
      },
      beeld: (m, url) =>
        setBeelden((oud) => {
          if (oud[m]) URL.revokeObjectURL(oud[m]);
          return { ...oud, [m]: url };
        }),
    });
    verbinding.current = v;
    return () => v.stop();
  }, [toegang]);

  const effectief = useMemo(() => {
    if (!stand || !breakModus || breakModus === "live") {
      return { stand, melding };
    }
    return bouwBreakUiStand(stand, breakModus);
  }, [stand, melding, breakModus]);

  const renderers = useMemo(() => [...new Set(rendererstatus.verbonden.map((r) => r.rol))], [rendererstatus]);
  const regie = useMemo<Regie>(
    () => ({
      sleutel: toegang.sleutel,
      werkplek: toegang.werkplek,
      staat,
      stand: effectief.stand,
      renderers,
      rendererstatus,
      bewerkingen,
      beelden,
      melding: effectief.melding,
      afmelden,
      bewerk: (set) => verbinding.current?.bewerk(set),
      async stuurFoto(id, foto, bij) {
        const a = await fotoVerzoek(toegang, id, foto);
        window.clearTimeout(meldingTijd.current);
        if (a.uitgevoerd) setMelding(null);
        else {
          setMelding({ bij, tekst: hoofdletter(a.reden) });
          meldingTijd.current = window.setTimeout(() => setMelding(null), MELDING_DUUR);
        }
        return a;
      },
      async haalFoto(id) {
        const r = await fetch(`/api/foto/${encodeURIComponent(id)}`, { headers: { Authorization: `Bearer ${toegang.sleutel}` } }).catch(() => null);
        return r?.ok ? URL.createObjectURL(await r.blob()) : null;
      },
      async plaatsBestand(bestand, bij) {
        const a = await bestandVerzoek(toegang, bestand);
        window.clearTimeout(meldingTijd.current);
        if (typeof a === "string") {
          setMelding(null);
          return a;
        }
        setMelding({ bij, tekst: hoofdletter(a.reden) });
        meldingTijd.current = window.setTimeout(() => setMelding(null), MELDING_DUUR);
        return null;
      },
      async haalBestand(naam) {
        const pad = naam.split("/").map(encodeURIComponent).join("/");
        const r = await fetch(`/api/bestand/${pad}`, { headers: { Authorization: `Bearer ${toegang.sleutel}` } }).catch(() => null);
        return r?.ok ? URL.createObjectURL(await r.blob()) : null;
      },
      async haalZaalbeeld(monitor = "ether") {
        const r = await fetch(`/api/zaalbeeld?monitor=${monitor}`, {
          headers: { Authorization: `Bearer ${toegang.sleutel}` },
        }).catch(() => null);
        if (!r || r.status !== 200) return null;
        return URL.createObjectURL(await r.blob());
      },
      async haalConceptbeeld(vraag) {
        try {
          const r = await fetch("/api/conceptbeeld", {
            method: "POST",
            headers: {
              Authorization: `Bearer ${toegang.sleutel}`,
              "Content-Type": "application/json",
            },
            body: JSON.stringify(vraag),
          });
          const data = (await r.json().catch(() => null)) as ConceptAntwoord | null;
          if (!data) return null;
          return data;
        } catch {
          return null;
        }
      },
      async stuur(opdracht, bij) {
        const v = verbinding.current;
        const a = v ? await v.stuur(opdracht) : { uitgevoerd: false as const, reden: "Geen verbinding met de server" };
        window.clearTimeout(meldingTijd.current);
        if (a.uitgevoerd) {
          setMelding(null);
        } else {
          setMelding({ bij, tekst: hoofdletter(a.reden) });
          meldingTijd.current = window.setTimeout(() => setMelding(null), MELDING_DUUR);
        }
        return a;
      },
    }),
    [toegang, staat, effectief, renderers, rendererstatus, bewerkingen, beelden, afmelden],
  );
  return (
    <RegieContext.Provider value={regie}>
      <CameraContext.Provider value={camera}>
        {children}
        {breakModus !== null && (
          <div
            role="group"
            aria-label="Break-UI dataset schakelaar"
            style={{
              position: "fixed",
              bottom: 48,
              left: "50%",
              transform: "translateX(-50%)",
              zIndex: 9999,
              display: "inline-flex",
              alignItems: "center",
              gap: 2,
              padding: 3,
              borderRadius: 999,
              background: "#262626",
              border: "1px solid rgba(255,255,255,0.22)",
              boxShadow: "0 6px 20px rgba(0,0,0,0.45)",
              fontFamily: "system-ui, -apple-system, sans-serif",
              fontSize: 11.5,
            }}
          >
            {BREAK_UI_MODI.map((m) => (
              <button
                key={m.id}
                type="button"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  setBreakModus(m.id);
                  zetBreakUiModusInUrl(m.id);
                }}
                style={{
                  border: 0,
                  borderRadius: 999,
                  padding: "4px 10px",
                  fontSize: 11.5,
                  fontWeight: breakModus === m.id ? 600 : 400,
                  background: breakModus === m.id ? "#ffffff" : "transparent",
                  color: breakModus === m.id ? "#111111" : "#d4d4d4",
                  cursor: "pointer",
                  whiteSpace: "nowrap",
                }}
              >
                {m.label}
              </button>
            ))}
          </div>
        )}
      </CameraContext.Provider>
    </RegieContext.Provider>
  );
}

async function fotoVerzoek(toegang: Toegang, id: string, foto: Blob | null): Promise<Antwoord> {
  try {
    const r = await fetch(`/api/foto/${encodeURIComponent(id)}`, {
      method: foto ? "PUT" : "DELETE",
      headers: { Authorization: `Bearer ${toegang.sleutel}`, "X-Afzender": toegang.werkplek },
      body: foto,
    });
    const b = (await r.json().catch(() => null)) as BerichtVanServer | null;
    if (b?.soort === "Uitgevoerd") return { uitgevoerd: true, volgnummer: b.volgnummer };
    if (b?.soort === "Geweigerd") return { uitgevoerd: false, reden: b.reden };
    return { uitgevoerd: false, reden: r.status === 404 ? "onbekende persoon" : `de server antwoordt ${r.status}` };
  } catch {
    return { uitgevoerd: false, reden: "Geen verbinding met de server" };
  }
}

async function bestandVerzoek(toegang: Toegang, bestand: File): Promise<string | { reden: string }> {
  try {
    const r = await fetch(`/api/bestand?naam=${encodeURIComponent(bestand.name)}`, {
      method: "POST",
      headers: { Authorization: `Bearer ${toegang.sleutel}`, "X-Afzender": toegang.werkplek },
      body: bestand,
    });
    const b = (await r.json().catch(() => null)) as { naam?: string; reden?: string } | null;
    if (r.ok && b?.naam) return b.naam;
    return { reden: b?.reden ?? (r.status === 413 ? "het bestand is te groot" : `de server antwoordt ${r.status}`) };
  } catch {
    return { reden: "Geen verbinding met de server" };
  }
}

function hoofdletter(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
