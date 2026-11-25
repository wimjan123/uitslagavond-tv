// Wat het invoerbestand bracht: welke sets binnenkwamen, welke zijn overgeslagen en waarom,
// en welke Partijen nog een tijdelijke kleur hebben. Staat in het onderdeel Cijfers, onder de sets.
import type { Cijfers } from "./typen/Cijfers";
import "./invoermelding.css";

export function Invoermelding({ cijfers }: { cijfers: Cijfers | undefined }) {
  const m = cijfers?.laatste_invoer;
  const zonderKleur = cijfers?.partijen.filter((p) => p.kleur_tijdelijk).map((p) => p.kort) ?? [];
  const zonderLogo = cijfers?.partijen.filter((p) => !p.logo).length ?? 0;
  return (
    <section className="paneel invoermelding" aria-live="polite">
      <h2 className="label">Invoerbestand</h2>
      {!m ? (
        <p className="stil">Nog niets binnen</p>
      ) : (
        <>
          <p>
            Binnen om <strong>{m.ontvangen}</strong> uit{" "}
            <span className="bestand" title={m.bestand}>
              {m.bestand.split(/[\\/]/).pop()}
            </span>
          </p>
          <p>
            {m.sets_binnen.length > 0 ? <>Nieuw of gewijzigd: {m.sets_binnen.join(", ")}</> : "Geen nieuwe cijfers"}
            {m.sets_ongewijzigd.length > 0 && <span className="stil"> · {m.sets_ongewijzigd.length} ongewijzigd</span>}
            {m.partijen_nieuw.length > 0 && <span className="stil"> · {m.partijen_nieuw.length} Partijen nieuw</span>}
          </p>
          {m.overgeslagen.length > 0 && (
            <div className="let-op">
              <h3 className="label">Overgeslagen</h3>
              <ul>
                {m.overgeslagen.map((o, i) => (
                  <li key={i}>
                    <strong>{o.wat}</strong>: {o.reden}
                  </li>
                ))}
              </ul>
            </div>
          )}
          {m.waarschuwingen.length > 0 && (
            <ul className="let-op">
              {m.waarschuwingen.map((w, i) => (
                <li key={i}>{w}</li>
              ))}
            </ul>
          )}
        </>
      )}
      {zonderKleur.length > 0 && (
        <details className="tijdelijk">
          <summary>
            {zonderKleur.length} Partijen met een tijdelijke kleur
          </summary>
          <p>{zonderKleur.join(", ")}</p>
        </details>
      )}
      {zonderLogo > 0 && <p className="stil">{zonderLogo} Partijen zonder logo: de korte naam staat op die plek</p>}
    </section>
  );
}
