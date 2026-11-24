// De toestand van één Werkplek: de verbinding, de Stand, de Renderers en de kleine beelden.
// Elk onderdeel en elk vak van de multiviewer haalt die op met useRegie(). De camera komt 10 keer
// per seconde en staat apart (useCamera()), zodat niet het hele dashboard zo vaak opnieuw tekent.
import { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import type { BerichtVanServer } from "../typen/BerichtVanServer";
import type { Bewerking } from "../typen/Bewerking";
import type { Camerabeeld } from "../typen/Camerabeeld";
import type { GenummerdeStand } from "../typen/GenummerdeStand";
import type { Opdracht } from "../typen/Opdracht";
import type { Rol } from "../typen/Rol";
import { Verbinding } from "../verbinding";
import { GEEN_RENDERERS } from "./systeem";
import type { Renderermelding } from "./systeem";
import type { Antwoord, Monitor, Staat } from "../verbinding";

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

export interface Regie {
  werkplek: string;
  staat: Staat;
  stand: GenummerdeStand | null;
  /** De rollen van de verbonden Renderers. */
  renderers: Rol[];
  /** Welke Renderers verbonden zijn en welke de server weigerde, met machine en versie. */
  rendererstatus: Renderermelding;
  /** Welke set elke andere Werkplek open heeft in het onderdeel Cijfers. */
  bewerkingen: Bewerking[];
  beelden: Record<Monitor, string | null>;
  melding: Melding | null;
  /** Stuurt een Opdracht. Een weigering komt als melding bij `bij` te staan. */
  stuur(opdracht: Opdracht, bij: string): Promise<Antwoord>;
  /** Meldt de andere Werkplekken welke set deze Werkplek open heeft in Cijfers, of geen. */
  bewerk(set: string | null): void;
  /** Zet een eigen foto (JPEG of PNG) van een Kamerlid of eigen persoon, of haalt hem weg (`null`). */
  stuurFoto(id: string, foto: Blob | null, bij: string): Promise<Antwoord>;
  /** De foto van een Kamerlid of eigen persoon als object-URL, of `null` als er geen is. */
  haalFoto(id: string): Promise<string | null>;
  /** Plaatst een programmalogo of lettertype bij de server; geeft de naam die de server het gaf.
   * Een weigering komt als melding bij `bij` te staan. */
  plaatsBestand(bestand: File, bij: string): Promise<string | null>;
  /** Een bestand van de server (zoals het programmalogo) als object-URL, of `null` als het er niet is. */
  haalBestand(naam: string): Promise<string | null>;
  afmelden(): void;
}

const RegieContext = createContext<Regie | null>(null);
const CameraContext = createContext<Camerabeeld | null>(null);

/** Waar de camera's van de Ether en het Voorbeeld nu staan, of `null` zonder bericht van de server. */
export function useCamera(): Camerabeeld | null {
  return useContext(CameraContext);
}

export function useRegie(): Regie {
  const r = useContext(RegieContext);
  if (!r) throw new Error("useRegie buiten RegieBron");
  return r;
}

/** Hoe lang een melding blijft staan, in milliseconden. */
const MELDING_DUUR = 6000;

export function RegieBron({ toegang, afmelden, children }: { toegang: Toegang; afmelden: () => void; children: ReactNode }) {
  const [staat, setStaat] = useState<Staat>("verbinden");
  const [stand, setStand] = useState<GenummerdeStand | null>(null);
  const [rendererstatus, setRendererstatus] = useState<Renderermelding>(GEEN_RENDERERS);
  const [bewerkingen, setBewerkingen] = useState<Bewerking[]>([]);
  const [beelden, setBeelden] = useState<Record<Monitor, string | null>>({ Voorbeeld: null, Ether: null });
  const [melding, setMelding] = useState<Melding | null>(null);
  const [camera, setCamera] = useState<Camerabeeld | null>(null);
  const verbinding = useRef<Verbinding | null>(null);
  const meldingTijd = useRef<number | undefined>(undefined);

  useEffect(() => {
    const afzender = { soort: toegang.tablet ? "Tablet" : "Werkplek", naam: toegang.werkplek } as const;
    const v = new Verbinding(toegang.sleutel, afzender, {
      staat: (st) => {
        setStaat(st);
        // Zonder verbinding weet dit scherm niet wat de anderen open hebben, en niet welke Renderers er zijn.
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

  const renderers = useMemo(() => [...new Set(rendererstatus.verbonden.map((r) => r.rol))], [rendererstatus]);
  const regie = useMemo<Regie>(
    () => ({
      werkplek: toegang.werkplek,
      staat,
      stand,
      renderers,
      rendererstatus,
      bewerkingen,
      beelden,
      melding,
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
    [toegang, staat, stand, renderers, rendererstatus, bewerkingen, beelden, melding, afmelden],
  );
  return (
    <RegieContext.Provider value={regie}>
      <CameraContext.Provider value={camera}>{children}</CameraContext.Provider>
    </RegieContext.Provider>
  );
}

/** PUT of DELETE /api/foto/<id>, met het antwoord van de server zoals bij een Opdracht. */
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

/** POST /api/bestand: de naam die de server het bestand gaf, of de reden van de weigering. */
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
