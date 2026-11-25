// Controle na de uitzending (DESIGN.md, Layout: Stemmingen, het deelscherm "Controle"): elke
// ingevoerde Stemming naast de officiële uitslag, de ernstigste bovenaan. De server haalt de
// officiële uitslag zelf op en overschrijft nooit iets; de Regie kiest per verschil.
import { Fragment } from "react";
import { uurMinuut } from "../graphic";
import { useRegie } from "../regie/regie";
import { STATUSSEN, STATUS_TEKST, controleLijst, minutenNaHamerslag, tellingen } from "../stemmingen/controle";
import type { Besluit } from "../typen/Besluit";
import type { Stemming } from "../typen/Stemming";
import type { Uitkomst } from "../typen/Uitkomst";

const BIJ = "controle";

export function Controle({ lijst }: { lijst: Stemming[] }) {
  const rijen = controleLijst(lijst);
  const t = tellingen(lijst);
  return (
    <div className="werkvlak-inhoud controle">
      <div className="controle-kop">
        <div className="controle-telling" aria-label="Per status">
          {STATUSSEN.map((s) => (
            <span key={s} className={`telling telling-${s}`}>
              <b>{t[s]}</b> {STATUS_TEKST[s]}
            </span>
          ))}
          {t.afgehandeld > 0 && (
            <span className="telling">
              <b>{t.afgehandeld}</b> afgehandeld
            </span>
          )}
        </div>
        <p className="zacht">
          De server haalt de officiële uitslag zelf op: Debat Direct ongeveer 2 uur na de hamerslag, de open data ongeveer 18 uur erna. Hij verandert de invoer nooit zelf.
        </p>
      </div>
      {rijen.length === 0 ? (
        <p className="lijst-leeg zacht">Nog geen ingevoerde Stemmingen om te controleren.</p>
      ) : (
        <table className="tabel controle-tabel">
          <thead>
            <tr>
              <th>Stemming</th>
              <th className="kol-ether">In de Ether</th>
              <th className="kol-uitslag">Onze invoer</th>
              <th className="kol-uitslag">Officieel</th>
              <th className="kol-controle">Status</th>
            </tr>
          </thead>
          <tbody>
            {rijen.map((s) => (
              <Rij key={s.punt.id} s={s} />
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function Rij({ s }: { s: Stemming }) {
  const regie = useRegie();
  const c = s.controle;
  const status = c.status!;
  const o = c.officieel;
  const verschil = status === "UitkomstAnders" || status === "VerschilPerFractie";
  const open = verschil && !c.afgehandeld;
  const melding = regie.melding?.bij === `${BIJ}-${s.punt.id}` ? regie.melding.tekst : null;
  const kies = (soort: "NeemOfficieelOver" | "OnzeInvoerKlopt") => void regie.stuur({ soort, stemming: s.punt.id }, `${BIJ}-${s.punt.id}`);
  const ether = s.in_ether_ms != null ? uurMinuut(s.in_ether_ms) : null;
  const vertraging = minutenNaHamerslag(c);
  const tijd = (t: string | null) => (t && Number.isFinite(Date.parse(t)) ? uurMinuut(Date.parse(t)) : "");
  const bron = o ? (o.bron === "OpenData" ? `open data ${tijd(c.open_data_gezien)}` : `Debat Direct ${tijd(c.debat_direct_gezien)}`) : null;
  return (
    <Fragment>
      <tr className={`${open ? "rij-verschil" : ""} ${verschil ? "rij-met-detail" : ""}`}>
        <td>
          <div className="motie-titel titel-cel">{s.punt.titel}</div>
          <div className="zacht">{s.met_de_hand ? "Met de hand" : (s.punt.nummer ?? s.punt.soort ?? "")}</div>
        </td>
        <td className="kol-ether">{ether ? <span className="tijd-ether">{ether}</span> : <span className="zacht">niet getoond</span>}</td>
        <td className="kol-uitslag">
          <Uitslag besluit={s.uitkomst.besluit} u={s.uitkomst} />
        </td>
        <td className="kol-uitslag">
          {o && c.officiele_uitkomst ? (
            <>
              <Uitslag besluit={o.besluit} u={c.officiele_uitkomst} />
              <div className="zacht">
                {bron}
                {vertraging !== null && ` · ${duur(vertraging)} na de hamerslag`}
              </div>
            </>
          ) : (
            <span className="zacht">{s.met_de_hand ? "met de hand: geen koppeling" : "nog niet binnen"}</span>
          )}
        </td>
        <td className="kol-controle">
          <span className={`controle-stempel controle-${status}`}>{STATUS_TEKST[status]}</span>
          {c.afgehandeld && (
            <div className="zacht">{c.afgehandeld === "OnzeInvoerKlopt" ? "Onze invoer klopt" : "Officieel overgenomen"}{c.beoordeling && ` · ${c.beoordeling.door}`}</div>
          )}
        </td>
      </tr>
      {verschil && (
        <tr className={`controle-detail ${open ? "rij-verschil" : ""}`}>
          <td colSpan={5}>
            <ul className="verschillen">
              {c.verschillen.map((v) => (
                <li key={v.kamerlid ?? v.wie}>
                  <span className="titel-cel">{v.wie}</span>
                  <span className="zacht">wij</span> <Stem stem={v.onze} leeg="open" />
                  <span className="zacht" aria-hidden>
                    →
                  </span>
                  <span className="zacht">officieel</span> <Stem stem={v.officieel} leeg="niet meegedaan" />
                </li>
              ))}
            </ul>
            {status === "UitkomstAnders" && ether && !c.afgehandeld && (
              <p className="controle-let-op">
                Deze Stemming was om {ether} in Live met een andere uitkomst dan officieel. Of je dat in de uitzending rechtzet, beslist de Regie.
              </p>
            )}
            {open && (
              <div className="controle-knoppen">
                <button className="knop knop-hoofd" disabled={regie.staat !== "verbonden"} onClick={() => kies("NeemOfficieelOver")}>
                  Neem officieel over
                </button>
                <button className="knop" disabled={regie.staat !== "verbonden"} onClick={() => kies("OnzeInvoerKlopt")}>
                  Onze invoer klopt
                </button>
                <span className="zacht">Een Preset en wat in Live stond houden hun eigen gegevens.</span>
              </div>
            )}
            {melding && (
              <p className="actiebalk-melding" role="alert">
                {melding}
              </p>
            )}
          </td>
        </tr>
      )}
    </Fragment>
  );
}

function Uitslag({ besluit, u }: { besluit: Besluit | null; u: Uitkomst }) {
  const klasse = besluit === "Aangenomen" ? "aangenomen" : besluit === "Verworpen" ? "verworpen" : "open";
  return (
    <>
      <div className={`controle-besluit ${klasse}`}>{besluit ?? "Open"}</div>
      <div className="zacht">
        {u.voor} voor, {u.tegen} tegen{u.open > 0 && `, ${u.open} open`}
      </div>
    </>
  );
}

function Stem({ stem, leeg }: { stem: "Voor" | "Tegen" | null; leeg: string }) {
  if (!stem) return <span className="zacht">{leeg}</span>;
  return <span className={stem === "Voor" ? "zin-voor" : "zin-tegen"}>{stem.toLowerCase()}</span>;
}

function duur(minuten: number): string {
  if (minuten < 60) return `${minuten} min`;
  const u = Math.floor(minuten / 60);
  const m = minuten % 60;
  return m ? `${u} u ${m} min` : `${u} u`;
}
