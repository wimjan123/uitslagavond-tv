// Wat het ophalen van de Kamer uit de open data bracht: wissels, Fracties die niet in de
// partijlijst stonden en Kamerleden zonder foto (de Persoonskaart toont dan voorletters).
import { useRegie } from "./regie/regie";
import type { Stand } from "./typen/Stand";
import type { Stoel } from "./typen/Stoel";
import "./invoermelding.css";

const ZAAL = "zaalindeling";

const stoelTekst = (s: Stoel) => `${s.vak}-${s.rij}-${s.nummer}`;

export function Kamermelding({ stand }: { stand: Stand | undefined }) {
  const kamer = stand?.kamer;
  const m = kamer?.laatste_ophaal;
  const naam = (k: { roepnaam: string; tussenvoegsel: string | null; achternaam: string }) =>
    [k.roepnaam, k.tussenvoegsel, k.achternaam].filter(Boolean).join(" ");
  const zonderFoto = kamer?.kamerleden.filter((k) => k.foto === "Geen").map(naam) ?? [];
  const sleutels = new Set(Object.values(kamer?.partij_van ?? {}));
  const tijdelijk =
    stand?.cijfers.partijen
      .filter((p) => sleutels.has(p.sleutel) && p.fractie_zonder_lijst && p.kleur_tijdelijk)
      .map((p) => p.kort) ?? [];
  return (
    <section className="paneel invoermelding" aria-live="polite">
      <h2 className="label">Kamer uit de open data</h2>
      {!kamer || !m ? (
        <p className="stil">Nog niets opgehaald</p>
      ) : (
        <>
          <p>
            <strong>{kamer.kamerleden.length}</strong> Kamerleden in {kamer.fracties.length} Fracties op{" "}
            <strong>{m.peildatum}</strong>
            <span className="stil"> · gewijzigd {m.opgehaald.slice(0, 16).replace("T", " ")}</span>
          </p>
          {m.opvolgers.length > 0 && (
            <ul>
              {m.opvolgers.map((o) => (
                <li key={o.zetel}>
                  <strong>{o.opvolger}</strong> volgt {o.vertrokken} op
                </li>
              ))}
            </ul>
          )}
          {(m.nieuw.length > 0 || m.vertrokken.length > 0) && (
            <p>
              {m.nieuw.length > 0 && <>Nieuw: {m.nieuw.join(", ")}</>}
              {m.nieuw.length > 0 && m.vertrokken.length > 0 && " · "}
              {m.vertrokken.length > 0 && <>Vertrokken: {m.vertrokken.join(", ")}</>}
            </p>
          )}
          {m.andere_fractie.length > 0 && <p>Andere Fractie: {m.andere_fractie.join("; ")}</p>}
        </>
      )}
      {tijdelijk.length > 0 && (
        <p className="let-op">Niet in de partijlijst, tijdelijke kleur en korte naam: {tijdelijk.join(", ")}</p>
      )}
      {zonderFoto.length > 0 && <p className="let-op">Zonder foto, voorletters in beeld: {zonderFoto.join(", ")}</p>}
      <Zaalmelding stand={stand} />
    </section>
  );
}

/** De Zaalindeling: wat de controle vond, en bij een wissel het voorstel van een Stoel, dat de
 * Regie met één klik bevestigt. Tot dan heeft het nieuwe Kamerlid geen Stoel. */
function Zaalmelding({ stand }: { stand: Stand | undefined }) {
  const regie = useRegie();
  const z = stand?.zaalindeling;
  const m = stand?.zaalmelding;
  if (!z || !m || z.stoelen.length === 0) return null;
  const c = m.controle;
  const fouten = [
    c.ontbrekend.length > 0 && `ontbreekt: ${c.ontbrekend.map(stoelTekst).join(", ")}`,
    c.dubbele_stoelen.length > 0 && `dubbel: ${c.dubbele_stoelen.map(stoelTekst).join(", ")}`,
    c.onbekende_stoelen.length > 0 && `bestaat niet: ${c.onbekende_stoelen.map(stoelTekst).join(", ")}`,
    c.dubbele_kamerleden.length > 0 && `${c.dubbele_kamerleden.length} Kamerleden op twee Stoelen`,
  ].filter(Boolean);
  const voorgesteld = new Set(m.voorstellen.map((v) => v.naam));
  const zonderVoorstel = m.zonder_stoel.filter((n) => !voorgesteld.has(n));
  const melding = regie.melding?.bij === ZAAL ? regie.melding.tekst : null;
  return (
    <>
      <h3 className="label">Zaalindeling {z.kamerperiode}</h3>
      {fouten.length > 0 && <p className="let-op">Stoelen in het bestand: {fouten.join("; ")}</p>}
      {m.voorstellen.length > 0 && (
        <ul>
          {m.voorstellen.map((v) => (
            <li key={v.kamerlid}>
              <strong>{v.naam}</strong> op de Stoel van {v.vertrokken} ({stoelTekst(v.stoel)}){" "}
              <button
                className="knop"
                type="button"
                disabled={regie.staat !== "verbonden"}
                onClick={() => void regie.stuur({ soort: "GeefStoel", kamerlid: v.kamerlid, stoel: v.stoel }, ZAAL)}
              >
                Geef Stoel
              </button>
            </li>
          ))}
        </ul>
      )}
      {zonderVoorstel.length > 0 && (
        <p className="let-op">Zonder Stoel (zet ze in het bestand): {zonderVoorstel.join(", ")}</p>
      )}
      {fouten.length === 0 && m.zonder_stoel.length === 0 && <p className="stil">Elk Kamerlid heeft een Stoel</p>}
      {melding && (
        <p className="melding" role="alert">
          {melding}
        </p>
      )}
    </>
  );
}
