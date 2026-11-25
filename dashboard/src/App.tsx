import { useCallback, useEffect, useState } from "react";
import type { FormEvent } from "react";
import { Multiviewer } from "./multiviewer/vakken";
import { ONDERDELEN } from "./onderdelen";
import { RegieBron, useRegie } from "./regie/regie";
import { statusVan } from "./regie/systeem";
import type { Toegang } from "./regie/regie";
import { voerUit } from "./regie/take";
import { actieVoorToets, doelVan } from "./regie/toetsen";

const OPSLAG = "uitslagavond.toegang";

function leesToegang(): Toegang | null {
  try {
    const t = JSON.parse(localStorage.getItem(OPSLAG) ?? "null") as Toegang | null;
    return t?.sleutel && t.werkplek ? t : null;
  } catch {
    return null;
  }
}

export function App() {
  const [toegang, setToegang] = useState<Toegang | null>(leesToegang);
  const afmelden = useCallback(() => {
    localStorage.removeItem(OPSLAG);
    setToegang(null);
  }, []);
  if (!toegang) {
    return (
      <Aanmelden
        klaar={(t) => {
          localStorage.setItem(OPSLAG, JSON.stringify(t));
          setToegang(t);
        }}
      />
    );
  }
  return (
    <RegieBron toegang={toegang} afmelden={afmelden}>
      <Schil />
    </RegieBron>
  );
}

/** De eerste keer openen: een naam voor deze Werkplek kiezen, zonder inlog, en de sleutel van de server. */
export function Aanmelden({ klaar, titel = "Regie", standaardNaam = "" }: { klaar: (t: Toegang) => void; titel?: string; standaardNaam?: string }) {
  const [werkplek, setWerkplek] = useState(standaardNaam);
  const [sleutel, setSleutel] = useState("");
  const verstuur = (e: FormEvent) => {
    e.preventDefault();
    if (werkplek.trim() && sleutel) klaar({ werkplek: werkplek.trim(), sleutel });
  };
  return (
    <main className="aanmelden">
      <form className="aanmelden-vak" onSubmit={verstuur}>
        <h1 className="kop-regel">{titel}</h1>
        <label className="veld">
          <span className="label">Naam van deze Werkplek</span>
          <input value={werkplek} onChange={(e) => setWerkplek(e.target.value)} placeholder="Regie links" autoFocus={!standaardNaam} />
          <span className="zacht">Staat bij alles wat je klaarzet, zoals "klaargezet door Regie links".</span>
        </label>
        <label className="veld">
          <span className="label">Sleutel van de server</span>
          <input type="password" value={sleutel} onChange={(e) => setSleutel(e.target.value)} autoFocus={!!standaardNaam} />
        </label>
        <button className="knop knop-hoofd" type="submit" disabled={!werkplek.trim() || !sleutel}>
          Verbinden
        </button>
      </form>
    </main>
  );
}

/** De vaste opbouw: tabbladen met status, de multiviewer-rij, en het werkvlak dat scrolt. */
function Schil() {
  const regie = useRegie();
  const [actief, setActief] = useState(() => ONDERDELEN.find((o) => o.Werkvlak)?.id ?? "");
  const onderdeel = ONDERDELEN.find((o) => o.id === actief);
  const Werkvlak = onderdeel?.Werkvlak;

  // Een klik op een knop geeft hem geen focus: zo blijft de spatie daarna altijd Toon, en drukt hij
  // niet nog eens de knop die je net met de muis gebruikte (zoals Wissen of Uit beeld).
  useEffect(() => {
    const geenFocus = (e: MouseEvent) => {
      const el = e.target instanceof Element ? e.target.closest("button, [role=tab]") : null;
      if (!el || (el as HTMLButtonElement).disabled) return;
      e.preventDefault();
      // Stond je nog in een veld (zoals een Zetelaantal), dan rondt dat veld eerst af: zo komt de laatste
      // waarde vóór de klik binnen (Opdrachten gaan op volgorde), en is Spatie daarna weer Toon.
      const actief = document.activeElement;
      if (actief instanceof HTMLElement && doelVan(actief) === "tekstveld") actief.blur();
    };
    window.addEventListener("mousedown", geenFocus);
    return () => window.removeEventListener("mousedown", geenFocus);
  }, []);

  useEffect(() => {
    const toets = (e: KeyboardEvent) => {
      if (e.key === "Escape" && doelVan(e.target) === "tekstveld") {
        (e.target as HTMLElement).blur();
        return;
      }
      // Te smal: het dashboard staat niet in beeld, dus de sneltoetsen ook niet.
      if (window.matchMedia("(max-width: 899px)").matches) return;
      const actie = actieVoorToets({ key: e.key, shiftKey: e.shiftKey, ctrlKey: e.ctrlKey, metaKey: e.metaKey, altKey: e.altKey, doel: doelVan(e.target) });
      if (!actie || e.repeat) return;
      e.preventDefault();
      voerUit(regie, actie);
    };
    window.addEventListener("keydown", toets);
    return () => window.removeEventListener("keydown", toets);
  }, [regie]);

  return (
    <div className="schil">
      <header className="tabstrook">
        <nav className="tabs" role="tablist" aria-label="Onderdelen">
          {ONDERDELEN.map((o) => (
            <button
              key={o.id}
              role="tab"
              className="tab"
              aria-selected={o.id === actief}
              disabled={!o.Werkvlak}
              title={o.Werkvlak ? undefined : "Komt later"}
              onClick={() => setActief(o.id)}
            >
              {o.naam}
              <TabTelling id={o.id} />
            </button>
          ))}
        </nav>
        <Status />
      </header>
      {regie.staat !== "verbonden" && (
        <div className="storing" role="alert">
          {regie.staat === "geweigerd"
            ? "De server weigert de verbinding: controleer de sleutel onder Systeem"
            : regie.staat === "verbinden"
              ? "Verbinden met de server"
              : "Geen verbinding met de server. Het dashboard probeert het elke seconde opnieuw."}
        </div>
      )}
      <Multiviewer />
      <main className="werkvlak">{Werkvlak && <Werkvlak kiesOnderdeel={setActief} />}</main>
      <div className="te-smal" role="alert">
        <h1 className="kop-regel">De Regie werkt op een laptop</h1>
        <p>Dit scherm is te smal voor de multiviewer en de lijsten. Maak het venster breder, of open op een tablet of telefoon de tablet van de Presentator.</p>
        <a className="knop knop-hoofd" href="/tablet">
          Naar de tablet van de Presentator
        </a>
      </div>
    </div>
  );
}

/** Open werk achter een tabblad: Stemmingen voor de uitzending die nog niet vastliggen, en in amber
 * de Stemmingen waarvan de controle na de uitzending een verschil vond. */
function TabTelling({ id }: { id: string }) {
  const { stand } = useRegie();
  if (id !== "stemmingen" || !stand) return null;
  const lijst = stand.stand.stemmingen.lijst;
  const open = lijst.filter((x) => x.voor_uitzending && !x.vastgelegd).length;
  const controle = lijst.filter((x) => x.controle.status && x.controle.status !== "Gelijk" && !x.controle.afgehandeld).length;
  return (
    <>
      {open > 0 && (
        <span className="tab-telling" title={`${open} Stemmingen voor de uitzending liggen nog niet vast`}>
          {open} open
        </span>
      )}
      {controle > 0 && (
        <span className="tab-telling tab-telling-let-op" title={`${controle} Stemmingen wijken af van de officiële uitslag`}>
          {controle} verschil
        </span>
      )}
    </>
  );
}

function Status() {
  const { staat, rendererstatus } = useRegie();
  const [nu, setNu] = useState(() => new Date());
  useEffect(() => {
    const t = window.setInterval(() => setNu(new Date()), 1000);
    return () => window.clearInterval(t);
  }, []);
  // Bovenin alleen Server, Ether en Reserve (DESIGN.md); het Voorbeeld staat onder Systeem.
  const punten = statusVan(staat, rendererstatus).filter((r) => r.naam !== "Voorbeeld");
  const twee = (n: number) => String(n).padStart(2, "0");
  return (
    <div className="status">
      {punten.map((p) => (
        <span key={p.naam} className={`status-punt status-${p.toon}`} title={p.status}>
          <span className={`punt punt-${p.toon}`} />
          {p.naam === "Ether" ? "Live" : p.naam}
          {p.toon !== "goed" && <span className="status-tekst">{p.status.toLowerCase()}</span>}
        </span>
      ))}
      <time className="klok">
        {twee(nu.getHours())}.{twee(nu.getMinutes())}.{twee(nu.getSeconds())}
      </time>
    </div>
  );
}
