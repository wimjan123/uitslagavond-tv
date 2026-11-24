// De WebSocket naar de server. Valt de verbinding weg, dan probeert hij het elke seconde opnieuw.
import type { Afzender } from "./typen/Afzender";
import type { BerichtAanServer } from "./typen/BerichtAanServer";
import type { BerichtVanServer } from "./typen/BerichtVanServer";
import type { Opdracht } from "./typen/Opdracht";

export type Staat = "verbinden" | "verbonden" | "geen verbinding" | "geweigerd";

/** De monitors van de multiviewer waarvoor de Renderer met rol Voorbeeld kleine beelden stuurt. */
export type Monitor = "Voorbeeld" | "Ether";
// Gelijk aan `Monitor::byte` in crates/gedeeld.
const MONITOR_BYTE: Monitor[] = ["Voorbeeld", "Ether"];

/** Het antwoord van de server op één Opdracht; bij uitvoeren met het volgnummer van de nieuwe Stand. */
export type Antwoord = { uitgevoerd: true; volgnummer: number } | { uitgevoerd: false; reden: string };

export interface Luisteraar {
  staat(s: Staat): void;
  bericht(b: BerichtVanServer): void;
  /** Een nieuw klein beeld (een object-URL), of `null` als er geen beeld is. */
  beeld(m: Monitor, url: string | null): void;
}

export class Verbinding {
  private ws: WebSocket | null = null;
  private gestopt = false;
  private poging: number | undefined;
  // De server beantwoordt Opdrachten één voor één, in volgorde.
  private wachtend: ((a: Antwoord) => void)[] = [];
  // De set die deze Werkplek open heeft in Cijfers; na een nieuwe verbinding meldt hij die opnieuw.
  private bewerkt: string | null = null;

  constructor(
    private sleutel: string,
    /** Een Werkplek van de Regie, of de tablet van de Presentator. */
    private afzender: Afzender,
    private luisteraar: Luisteraar,
  ) {
    this.verbind();
  }

  /** Stuurt een Opdracht en geeft het antwoord van de server. */
  stuur(opdracht: Opdracht): Promise<Antwoord> {
    if (this.ws?.readyState !== WebSocket.OPEN) {
      return Promise.resolve({ uitgevoerd: false, reden: "Geen verbinding met de server" });
    }
    this.zend({ soort: "Opdracht", opdracht });
    return new Promise((klaar) => this.wachtend.push(klaar));
  }

  /** Meldt welke set deze Werkplek open heeft in het onderdeel Cijfers, of geen. */
  bewerk(set: string | null): void {
    if (set === this.bewerkt) return;
    this.bewerkt = set;
    if (this.ws?.readyState === WebSocket.OPEN) this.zend({ soort: "Bewerk", set: set ?? undefined });
  }

  stop(): void {
    this.gestopt = true;
    window.clearTimeout(this.poging);
    this.ws?.close();
  }

  private zend(b: BerichtAanServer): void {
    this.ws?.send(JSON.stringify(b));
  }

  private beantwoord(a: Antwoord): void {
    this.wachtend.shift()?.(a);
  }

  private ontvang(data: unknown): void {
    if (data instanceof ArrayBuffer) {
      const bytes = new Uint8Array(data);
      const monitor = MONITOR_BYTE[bytes[0]];
      if (!monitor) return;
      const url = bytes.length > 1 ? URL.createObjectURL(new Blob([bytes.subarray(1)], { type: "image/jpeg" })) : null;
      this.luisteraar.beeld(monitor, url);
      return;
    }
    const b = JSON.parse(data as string) as BerichtVanServer;
    if (b.soort === "Uitgevoerd") this.beantwoord({ uitgevoerd: true, volgnummer: b.volgnummer });
    else if (b.soort === "Geweigerd") this.beantwoord({ uitgevoerd: false, reden: b.reden });
    this.luisteraar.bericht(b);
  }

  private verbind(): void {
    const protocol = location.protocol === "https:" ? "wss" : "ws";
    const url = `${protocol}://${location.host}/ws?sleutel=${encodeURIComponent(this.sleutel)}`;
    this.luisteraar.staat("verbinden");
    const ws = new WebSocket(url);
    ws.binaryType = "arraybuffer";
    this.ws = ws;
    let geopend = false;
    ws.onopen = () => {
      geopend = true;
      this.zend({ soort: "Hallo", afzender: this.afzender });
      if (this.bewerkt) this.zend({ soort: "Bewerk", set: this.bewerkt });
      this.luisteraar.staat("verbonden");
    };
    ws.onmessage = (e) => this.ontvang(e.data);
    ws.onclose = () => {
      for (const w of this.wachtend.splice(0)) w({ uitgevoerd: false, reden: "De verbinding viel weg" });
      for (const m of MONITOR_BYTE) this.luisteraar.beeld(m, null);
      if (this.gestopt) return;
      // Een verbinding die nooit openging, is meestal een verkeerde sleutel (401).
      this.luisteraar.staat(geopend ? "geen verbinding" : "geweigerd");
      this.poging = window.setTimeout(() => this.verbind(), 1000);
    };
  }
}
