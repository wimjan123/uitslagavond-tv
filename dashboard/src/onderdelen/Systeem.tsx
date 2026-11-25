// Systeem: deze Werkplek, de status van Server, Ether, Reserve en Voorbeeld (met machine en versie,
// ook van een Renderer die de server weigert), een verwijzing naar wat het
// invoerbestand bracht (dat staat bij Cijfers) en wat de open data over de Kamer bracht.
import { Kamermelding } from "../Kamermelding";
import { useRegie } from "../regie/regie";
import { statusVan } from "../regie/systeem";
import { TweeKeerKnop } from "./TweeKeerKnop";
import { Werkbalk } from "./Werkbalk";
import type { WerkvlakProps } from "./index";

export function Systeem({ kiesOnderdeel }: WerkvlakProps) {
  const { werkplek, staat, rendererstatus, afmelden, stand } = useRegie();
  return (
    <>
      <Werkbalk naam="Systeem" />
      <div className="werkvlak-inhoud systeem">
        {/* Waarschuwingen over de gegevens bovenaan: die mag de hoofdredacteur niet missen. */}
        <Kamermelding stand={stand?.stand} />
        <section className="blok">
          <h3 className="label">Werkplek</h3>
          <p className="kop-regel">{werkplek}</p>
          <div>
            {/* Afmelden haalt deze Werkplek uit de uitzending: een tweede klik, net als wissen in Live. */}
            <TweeKeerKnop className="knop" tweeKeer vraag="Klik nog eens: afmelden" onClick={afmelden}>
              Andere naam of sleutel
            </TweeKeerKnop>
          </div>
        </section>
        <section className="blok blok-breed">
          <h3 className="label">Status</h3>
          <table className="tabel">
            <thead>
              <tr>
                <th>Onderdeel</th>
                <th>Wat</th>
                <th>Machine</th>
                <th>Versie</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {statusVan(staat, rendererstatus).map((r) => (
                <tr key={r.naam}>
                  <td className="titel-cel" title={r.rol ? `Start met ${r.rol}` : undefined}>{r.label}</td>
                  <td className="zacht">{r.wat}</td>
                  <td>
                    {r.machines.map((m) => (
                      <div key={`${m.machine}-${m.geweigerd}`}>{m.machine}</div>
                    ))}
                  </td>
                  <td>
                    {r.machines.map((m) => (
                      <div key={`${m.machine}-${m.geweigerd}`} className={m.geweigerd ? "tekst-let-op" : undefined}>
                        {m.versie}
                      </div>
                    ))}
                  </td>
                  <td className="systeem-status">
                    <span className={`punt punt-${r.toon}`} /> {r.status}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {rendererstatus.geweigerd.length > 0 && (
            <p className="zacht">
              De server weigert een Renderer met een andere versie. Installeer op die machine hetzelfde pakket als op de server
              ({rendererstatus.versie}); daarna verbindt hij vanzelf.
            </p>
          )}
        </section>
        <section className="blok">
          <h3 className="label">Invoerbestand</h3>
          <p className="zacht">
            {stand?.stand.cijfers.laatste_invoer ? `Binnen om ${stand.stand.cijfers.laatste_invoer.ontvangen}. ` : "Nog niets binnen. "}
            Wat het bestand bracht en wat is overgeslagen staat bij Cijfers.
          </p>
          <div>
            <button className="knop" onClick={() => kiesOnderdeel("cijfers")}>
              Naar Cijfers
            </button>
          </div>
        </section>
      </div>
    </>
  );
}
