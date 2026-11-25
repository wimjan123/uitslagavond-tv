// Maken > Persoonskaart: een Kamerlid of een eigen persoon kiezen (zoeken op naam, of klikken op
// zijn Stoel in de plattegrond bovenin), de vrije regel en de extra gegevens, en de eigen personen
// en eigen foto's beheren. De server vult de gegevens bij klaarzetten in.
import { CameraKeuze } from "./CameraKeuze";
import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { kandidaten, opStoel, standpuntVoor, waarschuwingen, zoek } from "../persoonskaart/persoonskaart";
import type { Kandidaat } from "../persoonskaart/persoonskaart";
import { zetStoelkiezer } from "../persoonskaart/stoelkeuze";
import { cameraKeuzes } from "../regie/camerarit";
import { useRegie } from "../regie/regie";
import type { EigenPersoon } from "../typen/EigenPersoon";
import type { Extra } from "../typen/Extra";
import type { Graphic } from "../typen/Graphic";
import type { Stoel } from "../typen/Stoel";
import type { Wie } from "../typen/Wie";

// Gelijk aan MAX_VRIJE_REGEL en MAX_NAAM in crates/gedeeld.
const MAX_VRIJE_REGEL = 80;
const MAX_NAAM = 60;
const MAKEN = "maken";
const PERSONEN = "personen";

const EXTRA: { sleutel: keyof Extra; naam: string }[] = [
  { sleutel: "leeftijd", naam: "Leeftijd" },
  { sleutel: "woonplaats", naam: "Woonplaats" },
  { sleutel: "geboorteplaats", naam: "Geboorteplaats" },
  { sleutel: "in_de_kamer_sinds", naam: "In de Kamer sinds" },
];

const stoelTekst = (s: Stoel) => `${s.vak}-${s.rij}-${s.nummer}`;
const zelfde = (a: Wie | null, b: Wie | null) => !!a && !!b && a.soort === b.soort && a.id === b.id;

export function Persoonskaart({ uitleg }: { uitleg: string }) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const [zoektekst, setZoektekst] = useState("");
  const [wie, setWie] = useState<Wie | null>(null);
  const [regel, setRegel] = useState("");
  const [eigenExtra, setExtra] = useState<Extra | null>(null);
  // leeg: het Standpunt van het Sjabloon
  const [standpunt, setStandpunt] = useState("");
  const verbonden = regie.staat === "verbonden";
  const extra = eigenExtra ?? s?.persoonskaart_extra ?? { leeftijd: true, woonplaats: true, geboorteplaats: false, in_de_kamer_sinds: false };

  // Zolang dit formulier open is, kiest een klik op een Stoel in de plattegrond het Kamerlid dat er zit.
  useEffect(() => {
    if (!s) return;
    zetStoelkiezer((plaats) => {
      const k = opStoel(s, plaats);
      if (k) {
        setWie(k.wie);
        setZoektekst("");
      }
    });
    return () => zetStoelkiezer(null);
  }, [s]);

  if (!s) return null;
  const gekozen = wie ? (kandidaten(s).find((k) => zelfde(k.wie, wie)) ?? null) : null;
  const resultaten = zoek(s, zoektekst);
  const klaar = s.klaar;
  const graphic: Graphic | null = wie ? { sjabloon: "Persoonskaart", wie, regel: regel.trim(), extra } : null;
  const klaarIsPersoonskaart = klaar?.graphic.sjabloon === "Persoonskaart";
  const keuzes = cameraKeuzes(s.standpunten, s.cameraritten);
  const vanSjabloon = wie ? standpuntVoor(s, wie) : "S";
  const sjabloonTekst = vanSjabloon.startsWith("stoel-")
    ? `shot op Stoel ${vanSjabloon.slice(6)}`
    : (keuzes.find((k) => k.letter === vanSjabloon)?.tekst ?? vanSjabloon);
  const melding = regie.melding?.bij === MAKEN ? regie.melding.tekst : null;
  const let_op = wie ? waarschuwingen(s, wie) : [];

  const zetKlaar = (e: FormEvent) => {
    e.preventDefault();
    if (graphic) void regie.stuur({ soort: "ZetKlaar", graphic, standpunt: standpunt || undefined }, MAKEN);
  };
  const wijzig = () => {
    if (klaar && graphic) void regie.stuur({ soort: "WijzigKlaar", id: klaar.id, graphic, standpunt: standpunt || undefined }, MAKEN);
  };
  const kies = (k: Kandidaat) => {
    setWie(k.wie);
    setZoektekst("");
    const eigen = k.wie.soort === "Eigen" ? s.personen.find((p) => p.id === k.wie.id) : undefined;
    if (eigen?.regel) setRegel(eigen.regel);
  };

  return (
    <div className="persoonskaart-maken">
      <form className="paneel-detail" onSubmit={zetKlaar}>
        <header>
          <h3 className="kop-regel">Persoonskaart</h3>
          <p className="zacht">{uitleg}</p>
        </header>
        <label className="veld">
          <span className="veld-kop">
            <span className="label">Persoon</span>
            <span className="zacht">Zoek op naam of Partij, of klik op een Stoel in de plattegrond bovenin</span>
          </span>
          <input type="search" value={zoektekst} onChange={(e) => setZoektekst(e.target.value)} placeholder="Naam of Partij" />
        </label>
        {resultaten.length > 0 && (
          <table className="tabel persoon-lijst">
            <tbody>
              {resultaten.map((k) => (
                <tr key={`${k.wie.soort}-${k.wie.id}`} onClick={() => kies(k)} aria-selected={zelfde(k.wie, wie)}>
                  <td className="titel-cel">{k.naam}</td>
                  <td className="zacht">{k.partij}</td>
                  <td className="zacht">{k.wie.soort === "Eigen" ? "eigen persoon" : k.stoel ? `Stoel ${stoelTekst(k.stoel)}` : "geen Stoel"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {gekozen ? (
          <Gekozen kandidaat={gekozen} />
        ) : (
          <p className="zacht">{wie ? "Deze persoon is er niet meer." : "Nog niemand gekozen."}</p>
        )}
        {let_op.map((t) => (
          <p key={t} className="melding" role="alert">
            {t}
          </p>
        ))}
        <label className="veld">
          <span className="veld-kop">
            <span className="label">Vrije regel</span>
            <span className="zacht">
              {regel.length}/{MAX_VRIJE_REGEL} · mag leeg blijven
            </span>
          </span>
          <input value={regel} maxLength={MAX_VRIJE_REGEL} onChange={(e) => setRegel(e.target.value)} placeholder="Eerste spreker in het debat" />
        </label>
        <fieldset className="veld">
          <span className="veld-kop">
            <span className="label">Extra gegevens</span>
            <span className="zacht">Uit de open data; wat onbekend is, blijft weg</span>
          </span>
          {EXTRA.map(({ sleutel, naam }) => (
            <label key={sleutel}>
              <input type="checkbox" checked={extra[sleutel]} onChange={(e) => setExtra({ ...extra, [sleutel]: e.target.checked })} /> {naam}
            </label>
          ))}
          <button
            className="knop knop-stil"
            type="button"
            disabled={!verbonden || EXTRA.every(({ sleutel }) => extra[sleutel] === s.persoonskaart_extra[sleutel])}
            title="Deze vinkjes staan voortaan aan bij een nieuwe Persoonskaart"
            onClick={() => void regie.stuur({ soort: "ZetStandaardExtra", extra }, MAKEN)}
          >
            Maak standaard
          </button>
        </fieldset>
        <CameraKeuze
          waarde={standpunt}
          zet={setStandpunt}
          standaard={sjabloonTekst}
          opties={keuzes.map((k) => ({ waarde: k.letter, tekst: k.tekst, soort: k.soort }))}
        />
        <div className="knoppen">
          <button className="knop knop-commit" type="submit" disabled={!verbonden || !graphic}>
            Zet klaar
          </button>
          <button className="knop" type="button" disabled={!verbonden || !klaarIsPersoonskaart || !graphic} onClick={wijzig}>
            Pas Preview aan
          </button>
        </div>
        {melding && (
          <p className="melding" role="alert">
            {melding}
          </p>
        )}
      </form>
      <EigenPersonen kies={(id) => setWie({ soort: "Eigen", id })} />
    </div>
  );
}

/** De gekozen persoon met de foto, en een eigen foto plaatsen of weghalen. */
function Gekozen({ kandidaat: k }: { kandidaat: Kandidaat }) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const [url, setUrl] = useState<string | null>(null);
  const kamerlid = k.wie.soort === "Kamerlid" ? s?.kamer.kamerleden.find((l) => l.id === k.wie.id) : undefined;
  const eigenFoto = k.wie.soort === "Eigen" ? k.foto : kamerlid?.foto === "Eigen";
  // een nieuwe Stand na een foto haalt hem opnieuw op
  const versie = `${k.wie.id}-${k.foto}-${kamerlid?.foto ?? ""}-${regie.stand?.volgnummer ?? 0}`;
  useEffect(() => {
    let weg = false;
    let gemaakt: string | null = null;
    void regie.haalFoto(k.wie.id).then((u) => {
      gemaakt = u;
      if (weg) {
        if (u) URL.revokeObjectURL(u);
      } else setUrl(u);
    });
    return () => {
      weg = true;
      if (gemaakt) URL.revokeObjectURL(gemaakt);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [versie]);
  const plaats = (bestand: File | undefined) => {
    if (bestand) void regie.stuurFoto(k.wie.id, bestand, MAKEN);
  };
  return (
    <div className="persoon-gekozen">
      {url && k.foto ? <img className="persoon-foto" src={url} alt="" /> : <span className="persoon-foto persoon-foto-leeg">geen foto</span>}
      <div className="persoon-gegevens">
        <strong>{k.naam}</strong>
        <span className="zacht">
          {[k.partij, k.wie.soort === "Eigen" ? "eigen persoon" : k.stoel ? `Stoel ${stoelTekst(k.stoel)}` : "geen Stoel"].filter(Boolean).join(" · ")}
        </span>
        <span className="knoppen">
          <label className="knop knop-stil bestand-knop">
            Eigen foto…
            <input type="file" accept="image/jpeg,image/png" onChange={(e) => plaats(e.target.files?.[0])} />
          </label>
          {eigenFoto && (
            <button className="knop knop-stil" type="button" onClick={() => void regie.stuurFoto(k.wie.id, null, MAKEN)}>
              {k.wie.soort === "Kamerlid" ? "Terug naar foto uit open data" : "Foto weghalen"}
            </button>
          )}
        </span>
      </div>
    </div>
  );
}

const LEEG: EigenPersoon = { id: "", naam: "", foto: false, regel: "" };

/** Personen die geen Kamerlid zijn: toevoegen, wijzigen en wissen. Ze blijven bewaard bij het programma. */
function EigenPersonen({ kies }: { kies: (id: string) => void }) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const [p, setP] = useState<EigenPersoon>(LEEG);
  const verbonden = regie.staat === "verbonden";
  const melding = regie.melding?.bij === PERSONEN ? regie.melding.tekst : null;
  if (!s) return null;
  const veld = (sleutel: "woonplaats" | "geboorteplaats" | "geboortedatum", waarde: string) =>
    setP({ ...p, [sleutel]: waarde.trim() ? waarde : undefined });
  const bewaar = async (e: FormEvent) => {
    e.preventDefault();
    const a = await regie.stuur({ soort: "BewaarPersoon", persoon: p }, PERSONEN);
    if (a.uitgevoerd) setP(LEEG);
  };
  return (
    <section className="paneel-detail eigen-personen" aria-label="Eigen personen">
      <header>
        <h3 className="kop-regel">Eigen personen</h3>
        <p className="zacht">Iemand die geen Kamerlid is, zoals een lijsttrekker. Blijft bewaard bij het programma, ook in een Preset.</p>
      </header>
      {s.personen.length > 0 && (
        <table className="tabel">
          <tbody>
            {s.personen.map((x) => (
              <tr key={x.id}>
                <td className="titel-cel">{x.naam}</td>
                <td className="zacht">{s.cijfers.partijen.find((q) => q.sleutel === x.partij)?.kort ?? ""}</td>
                <td className="zacht">{x.foto ? "foto" : "geen foto"}</td>
                <td>
                  <span className="knoppen">
                    <button className="knop knop-stil" type="button" onClick={() => kies(x.id)}>
                      Kies
                    </button>
                    <button className="knop knop-stil" type="button" onClick={() => setP(x)}>
                      Wijzig
                    </button>
                    <button className="knop knop-stil" type="button" disabled={!verbonden} onClick={() => void regie.stuur({ soort: "WisPersoon", persoon: x.id }, PERSONEN)}>
                      Wis
                    </button>
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <form className="eigen-persoon-form" onSubmit={(e) => void bewaar(e)}>
        <label className="veld">
          <span className="label">Naam</span>
          <input value={p.naam} maxLength={MAX_NAAM} onChange={(e) => setP({ ...p, naam: e.target.value })} />
        </label>
        <label className="veld">
          <span className="label">Partij</span>
          <select value={p.partij ?? ""} onChange={(e) => setP({ ...p, partij: e.target.value || undefined })}>
            <option value="">Geen Partij</option>
            {s.cijfers.partijen.map((q) => (
              <option key={q.sleutel} value={q.sleutel}>
                {q.naam}
              </option>
            ))}
          </select>
        </label>
        <label className="veld">
          <span className="label">Vrije regel</span>
          <input value={p.regel} maxLength={MAX_VRIJE_REGEL} onChange={(e) => setP({ ...p, regel: e.target.value })} placeholder="Lijsttrekker" />
        </label>
        <label className="veld">
          <span className="label">Geboortedatum</span>
          <input type="date" value={p.geboortedatum ?? ""} onChange={(e) => veld("geboortedatum", e.target.value)} />
        </label>
        <label className="veld">
          <span className="label">Woonplaats</span>
          <input value={p.woonplaats ?? ""} onChange={(e) => veld("woonplaats", e.target.value)} />
        </label>
        <label className="veld">
          <span className="label">Geboorteplaats</span>
          <input value={p.geboorteplaats ?? ""} onChange={(e) => veld("geboorteplaats", e.target.value)} />
        </label>
        <div className="knoppen">
          <button className="knop" type="submit" disabled={!verbonden || !p.naam.trim()}>
            {p.id ? "Bewaar wijziging" : "Voeg toe"}
          </button>
          {p.id && (
            <button className="knop knop-stil" type="button" onClick={() => setP(LEEG)}>
              Annuleer
            </button>
          )}
        </div>
        <p className="zacht">De foto plaats je na het toevoegen: kies de persoon en klik op Eigen foto.</p>
        {melding && (
          <p className="melding" role="alert">
            {melding}
          </p>
        )}
      </form>
    </section>
  );
}
