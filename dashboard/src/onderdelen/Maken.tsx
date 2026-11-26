// Maken: een Graphic invullen en klaarzetten: de Titelkaart, de Zetelverdeling in de Zaal, de
// Zetelbalk, Twee sets naast elkaar, de Uitslag per Partij, Winst en verlies, Partij uitgelicht of
// de Persoonskaart.
import { useState } from "react";
import type { FormEvent } from "react";
import { cameraProfielVanGraphic, cameraProfielVanSjabloon, passendeCameraKeuzes, standaardCameraTekst } from "../regie/camerarit";
import { useRegie } from "../regie/regie";
import type { Graphic } from "../typen/Graphic";
import type { Indeling } from "../typen/Indeling";
import type { Sjabloon } from "../typen/Sjabloon";
import { benodigd, standaardVan } from "../vormgeving/sjablonen";
import { Schets } from "../presets/Schets";
import { CameraKeuze } from "./CameraKeuze";
import { Persoonskaart } from "./Persoonskaart";
import { Werkbalk } from "./Werkbalk";

// Gelijk aan MAX_KOP en MAX_REGEL in crates/gedeeld.
const MAX_KOP = 80;
const MAX_REGEL = 120;
const MAKEN = "maken";

const SJABLONEN = [
  { naam: "Titelkaart", groep: "Algemeen", uitleg: "Een kop en een regel eronder. De Zaal houdt de kleuren van de vorige Graphic." },
  {
    naam: "Zetelverdeling in de Zaal",
    groep: "Uitslagavond",
    uitleg: "De Zetels van één set op de Stoelen: per Partij één taartpunt, van links naar rechts in de vaste volgorde. Zetels zonder Partij zijn grijs.",
  },
  {
    naam: "Zetelbalk",
    groep: "Uitslagavond",
    uitleg: "Eén set als balk onderin beeld, in de vaste volgorde, met de lijn bij 76. De Stoelen kleuren zoals bij de Zetelverdeling.",
  },
  {
    naam: "Twee sets naast elkaar",
    groep: "Uitslagavond",
    uitleg: "Twee Zetelbalken onder elkaar met dezelfde Partijen. De oudste set staat boven, de nieuwste onder met een pijltje voor het verschil; de Zaal toont de nieuwste.",
  },
  {
    naam: "Uitslag per Partij",
    groep: "Uitslagavond",
    uitleg: "Eén set als liggende balken van groot naar klein. De Stoelen kleuren zoals bij de Zetelverdeling.",
  },
  {
    naam: "Winst en verlies",
    groep: "Uitslagavond",
    uitleg:
      "Per Partij het verschil in Zetels met de vergelijkingsset, tegen de som van haar Voorgangers. Een nieuwe Partij krijgt \"nieuw\", een verdwenen Partij \"nu 0\". De Stoelen kleuren zoals bij de Zetelverdeling.",
  },
  {
    naam: "Partij uitgelicht",
    groep: "Uitslagavond",
    uitleg: "De Stoelen van één Partij in haar kleur, de rest grijs, met haar naam en Zetels linksboven.",
  },
  {
    naam: "Persoonskaart",
    groep: "Algemeen",
    uitleg: "Een kaart rechts met naam, foto en Partij van een Kamerlid of een eigen persoon. De Stoel van het Kamerlid licht op en de camera gaat er schuin van voren naartoe.",
  },
];

/** Hoeveel Partijen een Zetelbalk apart toont; de rest is samen "overig". */
const GROOTSTE = [3, 4, 5, 6, 7, 8, 9, 10, 12];

export function Maken() {
  const regie = useRegie();
  // een getal: een vast Sjabloon uit SJABLONEN; een tekst: het id van een eigen Sjabloon
  const [keuze, setGekozen] = useState<number | string>(0);
  const eigen = (regie.stand?.stand.sjablonen ?? []).filter((b) => !standaardVan(b.sjabloon.id)).map((b) => b.sjabloon);
  const eigenGekozen = typeof keuze === "string" ? eigen.find((e) => e.id === keuze) : undefined;
  const gekozen = typeof keuze === "number" ? keuze : eigenGekozen ? -1 : 0;
  return (
    <>
      <Werkbalk naam="Maken" />
      <div className="werkvlak-inhoud twee-delen">
        <div className="lijst">
          <table className="tabel">
            <thead>
              <tr>
                <th>Sjabloon</th>
                <th>Groep</th>
              </tr>
            </thead>
            <tbody>
              {SJABLONEN.map((s, i) => (
                <tr key={s.naam} className={i === gekozen ? "gekozen" : undefined} aria-selected={i === gekozen} onClick={() => setGekozen(i)}>
                  <td className="titel-cel">{s.naam}</td>
                  <td className="zacht">{s.groep}</td>
                </tr>
              ))}
              {eigen.map((e) => (
                <tr key={e.id} className={e.id === eigenGekozen?.id ? "gekozen" : undefined} aria-selected={e.id === eigenGekozen?.id} onClick={() => setGekozen(e.id)}>
                  <td className="titel-cel">{e.naam}</td>
                  <td className="zacht">Eigen Sjablonen</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {eigenGekozen ? (
          <EigenGraphic key={eigenGekozen.id} sjabloon={eigenGekozen} />
        ) : gekozen === 0 ? (
          <Titelkaart />
        ) : gekozen === 1 ? (
          <Zetelverdeling />
        ) : gekozen === 5 ? (
          <WinstEnVerlies />
        ) : gekozen === 6 ? (
          <PartijUitgelicht />
        ) : gekozen === 7 ? (
          <Persoonskaart uitleg={SJABLONEN[7].uitleg} />
        ) : (
          <Zetelbalk key={gekozen} soort={(["Zetelbalk", "TweeSets", "UitslagPerPartij"] as const)[gekozen - 2]} sjabloon={SJABLONEN[gekozen]} />
        )}
      </div>
    </>
  );
}

function Titelkaart() {
  const regie = useRegie();
  const [kop, setKop] = useState("");
  const [regel, setRegel] = useState("");
  // leeg: het Standpunt van het Sjabloon
  const [standpunt, setStandpunt] = useState("");
  const klaar = regie.stand?.stand.klaar ?? null;
  const verbonden = regie.staat === "verbonden";
  const s = regie.stand?.stand;
  const graphic: Graphic = { sjabloon: "Titelkaart", kop: kop.trim(), regel: regel.trim() };
  const klaarIsTitelkaart = klaar?.graphic.sjabloon === "Titelkaart";
  const keuzes = passendeCameraKeuzes(s?.standpunten ?? [], s?.cameraritten ?? [], "Algemeen");
  const standaard = s ? standaardCameraTekst(s, graphic) : null;
  const gekozen = standpunt || undefined;
  const zelfde =
    klaar?.graphic.sjabloon === "Titelkaart" &&
    klaar.graphic.kop === graphic.kop &&
    klaar.graphic.regel === graphic.regel &&
    (klaar.standpunt ?? "") === standpunt;
  const melding = regie.melding?.bij === MAKEN ? regie.melding.tekst : null;

  const zetKlaar = (e: FormEvent) => {
    e.preventDefault();
    if (graphic.kop) void regie.stuur({ soort: "ZetKlaar", graphic, standpunt: gekozen }, MAKEN);
  };
  const wijzig = () => {
    if (klaar) void regie.stuur({ soort: "WijzigKlaar", id: klaar.id, graphic, standpunt: gekozen }, MAKEN);
  };
  const neemOver = () => {
    if (klaar?.graphic.sjabloon === "Titelkaart") {
      setKop(klaar.graphic.kop);
      setRegel(klaar.graphic.regel);
      setStandpunt(klaar.standpunt ?? "");
    }
  };

  return (
    <div className="maken-met-schets">
    <form className="paneel-detail" onSubmit={zetKlaar}>
      <header>
        <h3 className="kop-regel" title={SJABLONEN[0].uitleg}>Titelkaart</h3>
      </header>
      <label className="veld">
        <span className="veld-kop">
          <span className="label">Kop</span>
          <span className="zacht">
            {kop.length}/{MAX_KOP}
          </span>
        </span>
        <input value={kop} maxLength={MAX_KOP} onChange={(e) => setKop(e.target.value)} placeholder="bijv. De Uitslagavond" />
      </label>
      <label className="veld">
        <span className="veld-kop">
          <span className="label">Regel</span>
          <span className="zacht">
            {regel.length}/{MAX_REGEL}
          </span>
        </span>
        <input value={regel} maxLength={MAX_REGEL} onChange={(e) => setRegel(e.target.value)} placeholder="bijv. Live vanuit Den Haag" />
      </label>
      <CameraKeuze
        waarde={standpunt}
        zet={setStandpunt}
        standaard={standaard}
        opties={keuzes.map((k) => ({ waarde: k.letter, tekst: k.tekst, soort: k.soort, toelichting: k.toelichting }))}
      />
      <div className="knoppen">
        <button className="knop knop-commit" type="submit" disabled={!verbonden || !graphic.kop}>
          Zet klaar
        </button>
        <button
          className="knop"
          type="button"
          disabled={!verbonden || !klaarIsTitelkaart || !graphic.kop || zelfde}
          title={klaarIsTitelkaart ? "Past de Titelkaart in Preview aan; staat die al in Live, dan pas na Update" : "Kan als er een Titelkaart klaarstaat"}
          onClick={wijzig}
        >
          Pas Preview aan
        </button>
        <button
          className="knop knop-stil"
          type="button"
          disabled={!klaarIsTitelkaart || zelfde}
          title={klaarIsTitelkaart ? "Haalt kop en regel uit de Titelkaart die klaarstaat" : "Kan als er een Titelkaart klaarstaat"}
          onClick={neemOver}
        >
          Laad uit Preview
        </button>
        {!graphic.kop && <span className="knop-reden">Vul eerst een kop in</span>}
      </div>
      {melding && (
        <p className="melding" role="alert">
          {melding}
        </p>
      )}
    </form>
    <figure className="maken-schets">
      <figcaption className="label">Schets</figcaption>
      {regie.stand && (
        <Schets graphic={{ sjabloon: "Titelkaart", kop: kop.trim() || "Kop", regel: regel.trim() }} kleuren={regie.stand.stand.stijl.kleuren} />
      )}
    </figure>
    </div>
  );
}

function Zetelverdeling() {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const sets = s?.cijfers.sets ?? [];
  const [set, setSet] = useState("");
  const [indeling, setIndeling] = useState<Indeling>("Uitslagindeling");
  const [namen, setNamen] = useState(true);
  const [zetels, setZetels] = useState(true);
  const [standpunt, setStandpunt] = useState("");
  const gekozenSet = sets.find((x) => x.id === set) ?? sets[0];
  const klaar = s?.klaar ?? null;
  const verbonden = regie.staat === "verbonden";
  const inZaal = indeling === "Zaalindeling";
  const graphic: Graphic | null = inZaal
    ? { sjabloon: "Zetelverdeling", set: "", namen, zetels, indeling }
    : gekozenSet
      ? { sjabloon: "Zetelverdeling", set: gekozenSet.id, namen, zetels, indeling }
      : null;
  const zaal = s?.zaalmelding;
  const zaalLeeg = (s?.zaalindeling.stoelen.length ?? 0) === 0;
  const keuzes = passendeCameraKeuzes(s?.standpunten ?? [], s?.cameraritten ?? [], "HeleZaal");
  const standaard = s && graphic ? standaardCameraTekst(s, graphic) : null;
  const gekozen = standpunt || undefined;
  const klaarIsZetelverdeling = klaar?.graphic.sjabloon === "Zetelverdeling";
  const melding = regie.melding?.bij === MAKEN ? regie.melding.tekst : null;
  const kort = (sleutel: string) => s?.cijfers.partijen.find((p) => p.sleutel === sleutel)?.kort ?? sleutel;
  const nietOpLijst = s?.uitslagindeling.niet_op_lijst ?? [];

  const zetKlaar = (e: FormEvent) => {
    e.preventDefault();
    if (graphic) void regie.stuur({ soort: "ZetKlaar", graphic, standpunt: gekozen }, MAKEN);
  };
  const wijzig = () => {
    if (klaar && graphic) void regie.stuur({ soort: "WijzigKlaar", id: klaar.id, graphic, standpunt: gekozen }, MAKEN);
  };

  return (
    <form className="paneel-detail" onSubmit={zetKlaar}>
      <header>
        <h3 className="kop-regel" title={SJABLONEN[1].uitleg}>{SJABLONEN[1].naam}</h3>
      </header>
      <fieldset className="veld">
        <span className="veld-kop">
          <span className="label">Indeling</span>
          <span className="zacht">{inZaal ? "De Kamer die zit, elk Kamerlid op zijn eigen Stoel" : "De Zetels van een set, per Partij één blok"}</span>
        </span>
        <label>
          <input type="radio" name="indeling" checked={!inZaal} onChange={() => setIndeling("Uitslagindeling")} /> Uitslagindeling
        </label>
        <label>
          <input type="radio" name="indeling" checked={inZaal} onChange={() => setIndeling("Zaalindeling")} /> Zaalindeling
        </label>
      </fieldset>
      {inZaal ? (
        zaalLeeg && <p className="melding">Nog geen Zaalindeling: start de server met het bestand van de Kamerperiode.</p>
      ) : sets.length === 0 ? (
        <p className="melding">Nog geen sets: lees eerst het invoerbestand in of maak een set (zie Systeem).</p>
      ) : (
        <label className="veld">
          <span className="veld-kop">
            <span className="label">Set</span>
            <span className="zacht">De Graphic houdt deze cijfers vast tot Update</span>
          </span>
          <select value={gekozenSet?.id ?? ""} onChange={(e) => setSet(e.target.value)}>
            {sets.map((x) => (
              <option key={x.id} value={x.id}>
                {x.label}
              </option>
            ))}
          </select>
        </label>
      )}
      <fieldset className="veld">
        <span className="veld-kop">
          <span className="label">Labels</span>
          <span className="zacht">Onderin beeld, per Partij</span>
        </span>
        <label>
          <input type="checkbox" checked={namen} onChange={(e) => setNamen(e.target.checked)} /> Naam
        </label>
        <label>
          <input type="checkbox" checked={zetels} onChange={(e) => setZetels(e.target.checked)} /> Zetels
        </label>
      </fieldset>
      <CameraKeuze
        waarde={standpunt}
        zet={setStandpunt}
        standaard={standaard}
        opties={keuzes.map((k) => ({ waarde: k.letter, tekst: k.tekst, soort: k.soort, toelichting: k.toelichting }))}
      />
      <div className="knoppen">
        <button className="knop knop-commit" type="submit" disabled={!verbonden || !graphic}>
          Zet klaar
        </button>
        <button className="knop" type="button" disabled={!verbonden || !klaarIsZetelverdeling || !graphic} onClick={wijzig}>
          Pas Preview aan
        </button>
      </div>
      {inZaal ? (
        (zaal?.zonder_stoel.length ?? 0) > 0 && (
          <p className="melding" role="alert">
            Zonder Stoel, niet in beeld: {zaal?.zonder_stoel.join(", ")}. Zie Systeem.
          </p>
        )
      ) : (
      <p className="zacht">
        Volgorde in de Zaal, van links naar rechts:{" "}
        {(s?.uitslagindeling.partijen ?? [])
          .filter((p) => (gekozenSet?.zetels[p] ?? 0) > 0)
          .map(kort)
          .join(" · ") || "nog geen Zetels in deze set"}
      </p>
      )}
      {nietOpLijst.length > 0 && (
        <p className="melding" role="alert">
          Niet in de vaste volgorde van het programma: {nietOpLijst.map(kort).join(", ")}. {nietOpLijst.length === 1 ? "Deze Partij staat" : "Deze Partijen staan"} rechts
          achteraan.
        </p>
      )}
      {melding && (
        <p className="melding" role="alert">
          {melding}
        </p>
      )}
    </form>
  );
}

type Balksoort = "Zetelbalk" | "TweeSets" | "UitslagPerPartij";

/** Het formulier van de Zetelbalk, Twee sets naast elkaar en de Uitslag per Partij: set, tweede set en N. */
function Zetelbalk({ soort, sjabloon }: { soort: Balksoort; sjabloon: (typeof SJABLONEN)[number] }) {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const sets = s?.cijfers.sets ?? [];
  const [set, setSet] = useState("");
  const [tweede, setTweede] = useState("");
  const [grootste, setGrootste] = useState("");
  const [standpunt, setStandpunt] = useState("");
  const eerste = sets.find((x) => x.id === set) ?? sets[0];
  const ander = sets.find((x) => x.id === tweede) ?? sets.find((x) => x.id !== eerste?.id);
  const n = grootste ? Number(grootste) : undefined;
  const twee = soort === "TweeSets";
  const klaar = s?.klaar ?? null;
  const verbonden = regie.staat === "verbonden";
  const graphic: Graphic | null = !eerste
    ? null
    : twee
      ? ander && ander.id !== eerste.id
        ? { sjabloon: "TweeSets", set: eerste.id, tweede: ander.id, grootste: n }
        : null
      : { sjabloon: soort, set: eerste.id, grootste: n };
  // de set die later in de lijst staat is de nieuwste (gelijk aan `Zetelstand::van_twee`)
  const [boven, onder] = eerste && ander && sets.indexOf(ander) < sets.indexOf(eerste) ? [ander, eerste] : [eerste, ander];
  const profiel = soort === "UitslagPerPartij" ? "GrootPaneel" : "BovenPaneel";
  const keuzes = passendeCameraKeuzes(s?.standpunten ?? [], s?.cameraritten ?? [], profiel);
  const standaard = s && graphic ? standaardCameraTekst(s, graphic) : null;
  const gekozen = standpunt || undefined;
  const melding = regie.melding?.bij === MAKEN ? regie.melding.tekst : null;

  const zetKlaar = (e: FormEvent) => {
    e.preventDefault();
    if (graphic) void regie.stuur({ soort: "ZetKlaar", graphic, standpunt: gekozen }, MAKEN);
  };
  const wijzig = () => {
    if (klaar && graphic) void regie.stuur({ soort: "WijzigKlaar", id: klaar.id, graphic, standpunt: gekozen }, MAKEN);
  };
  const setKeuze = (waarde: string, zet: (id: string) => void, label: string, uitleg: string) => (
    <label className="veld">
      <span className="veld-kop">
        <span className="label">{label}</span>
        <span className="zacht">{uitleg}</span>
      </span>
      <select value={waarde} onChange={(e) => zet(e.target.value)}>
        {sets.map((x) => (
          <option key={x.id} value={x.id}>
            {x.label}
          </option>
        ))}
      </select>
    </label>
  );

  return (
    <form className="paneel-detail" onSubmit={zetKlaar}>
      <header>
        <h3 className="kop-regel" title={sjabloon.uitleg}>{sjabloon.naam}</h3>
      </header>
      {sets.length < (twee ? 2 : 1) ? (
        <p className="melding">{twee ? "Er zijn twee sets nodig" : "Nog geen sets"}: lees eerst het invoerbestand in of maak een set (zie Systeem).</p>
      ) : (
        <>
          {setKeuze(eerste?.id ?? "", setSet, twee ? "Eerste set" : "Set", "De Graphic houdt deze cijfers vast tot Update")}
          {twee && setKeuze(ander?.id ?? "", setTweede, "Tweede set", "Kies een andere set dan de eerste")}
          {twee && boven && onder && boven.id !== onder.id && (
            <p className="zacht">
              Boven: {boven.label}. Onder, met pijltjes en in de Zaal: {onder.label}.
            </p>
          )}
        </>
      )}
      <label className="veld">
        <span className="veld-kop">
          <span className="label">Partijen</span>
          <span className="zacht">{twee ? "De grootste uit de nieuwste set; de rest samen als overig" : "De rest samen als overig"}</span>
        </span>
        <select value={grootste} onChange={(e) => setGrootste(e.target.value)}>
          <option value="">Alle Partijen</option>
          {GROOTSTE.map((g) => (
            <option key={g} value={g}>
              Grootste {g} en overig
            </option>
          ))}
        </select>
      </label>
      <CameraKeuze
        waarde={standpunt}
        zet={setStandpunt}
        standaard={standaard}
        opties={keuzes.map((k) => ({ waarde: k.letter, tekst: k.tekst, soort: k.soort, toelichting: k.toelichting }))}
      />
      <div className="knoppen">
        <button className="knop knop-commit" type="submit" disabled={!verbonden || !graphic}>
          Zet klaar
        </button>
        <button className="knop" type="button" disabled={!verbonden || klaar?.graphic.sjabloon !== soort || !graphic} onClick={wijzig}>
          Pas Preview aan
        </button>
      </div>
      {melding && (
        <p className="melding" role="alert">
          {melding}
        </p>
      )}
    </form>
  );
}

/** Kiezen uit een lijst, met een korte uitleg naast het label. */
function Keuze({
  label,
  uitleg,
  waarde,
  zet,
  opties,
}: {
  label: string;
  uitleg: string;
  waarde: string;
  zet: (w: string) => void;
  opties: { waarde: string; tekst: string }[];
}) {
  return (
    <label className="veld">
      <span className="veld-kop">
        <span className="label">{label}</span>
        <span className="zacht">{uitleg}</span>
      </span>
      <select value={waarde} onChange={(e) => zet(e.target.value)}>
        {opties.map((o) => (
          <option key={o.waarde} value={o.waarde}>
            {o.tekst}
          </option>
        ))}
      </select>
    </label>
  );
}

/** Set, Standpunt en de knoppen: wat de formulieren van Winst en verlies en Partij uitgelicht delen. */
function useCijferformulier() {
  const regie = useRegie();
  const s = regie.stand?.stand;
  const sets = s?.cijfers.sets ?? [];
  const [set, setSet] = useState("");
  const [standpunt, setStandpunt] = useState("");
  const gekozenSet = sets.find((x) => x.id === set) ?? sets[sets.length - 1];
  const stuur = (graphic: Graphic | null, wijzig: boolean) => {
    const klaar = s?.klaar;
    if (!graphic) return;
    const gekozen = standpunt || undefined;
    if (!wijzig) void regie.stuur({ soort: "ZetKlaar", graphic, standpunt: gekozen }, MAKEN);
    else if (klaar) void regie.stuur({ soort: "WijzigKlaar", id: klaar.id, graphic, standpunt: gekozen }, MAKEN);
  };
  const setVeld = (
    <Keuze
      label="Set"
      uitleg="De Graphic houdt deze cijfers vast tot Update"
      waarde={gekozenSet?.id ?? ""}
      zet={setSet}
      opties={sets.map((x) => ({ waarde: x.id, tekst: x.label }))}
    />
  );
  const onderkant = (graphic: Graphic | null, sjabloon: Graphic["sjabloon"], profielFallback?: import("../typen/CameraProfiel").CameraProfiel) => {
    const profiel = graphic
      ? cameraProfielVanGraphic(graphic, s?.sjablonen ?? [])
      : (profielFallback ?? (sjabloon === "PartijUitgelicht" ? "Partij" : sjabloon === "WinstEnVerlies" ? "GrootPaneel" : "Algemeen"));
    const keuzes = passendeCameraKeuzes(s?.standpunten ?? [], s?.cameraritten ?? [], profiel);
    const standaard = s && graphic ? standaardCameraTekst(s, graphic) : null;
    const melding = regie.melding?.bij === MAKEN ? regie.melding.tekst : null;
    const verbonden = regie.staat === "verbonden";
    return (
      <>
        <CameraKeuze
          waarde={standpunt}
          zet={setStandpunt}
          standaard={standaard}
          opties={keuzes.map((k) => ({ waarde: k.letter, tekst: k.tekst, soort: k.soort, toelichting: k.toelichting }))}
        />
        <div className="knoppen">
          <button className="knop knop-commit" type="submit" disabled={!verbonden || !graphic}>
            Zet klaar
          </button>
          <button className="knop" type="button" disabled={!verbonden || s?.klaar?.graphic.sjabloon !== sjabloon || !graphic} onClick={() => stuur(graphic, true)}>
            Pas Preview aan
          </button>
        </div>
        {melding && (
          <p className="melding" role="alert">
            {melding}
          </p>
        )}
      </>
    );
  };
  const zetKlaar = (graphic: Graphic | null) => (e: FormEvent) => {
    e.preventDefault();
    stuur(graphic, false);
  };
  return { s, sets, gekozenSet, setVeld, onderkant, zetKlaar };
}

function WinstEnVerlies() {
  const { s, sets, gekozenSet, setVeld, onderkant, zetKlaar } = useCijferformulier();
  const [tegen, setTegen] = useState("");
  const standaard = sets.find((x) => x.id === s?.cijfers.vergelijk_met);
  const anders = sets.filter((x) => x.id !== gekozenSet?.id);
  const vergelijk = anders.find((x) => x.id === tegen);
  const tegenover = vergelijk ?? standaard;
  const graphic: Graphic | null =
    gekozenSet && tegenover && tegenover.id !== gekozenSet.id
      ? { sjabloon: "WinstEnVerlies", set: gekozenSet.id, vergelijk_met: vergelijk?.id }
      : null;
  const sjabloon = SJABLONEN[5];
  return (
    <form className="paneel-detail" onSubmit={zetKlaar(graphic)}>
      <header>
        <h3 className="kop-regel" title={sjabloon.uitleg}>{sjabloon.naam}</h3>
      </header>
      {sets.length < 2 ? (
        <p className="melding">Er zijn twee sets nodig: lees eerst het invoerbestand in of maak een set (zie Systeem).</p>
      ) : (
        <>
          {setVeld}
          <Keuze
            label="Vergelijkingsset"
            uitleg="Winst en verlies tegenover deze set"
            waarde={vergelijk?.id ?? ""}
            zet={setTegen}
            opties={[
              { waarde: "", tekst: standaard ? `Standaard: ${standaard.label}` : "Geen standaard, kies een set" },
              ...anders.map((x) => ({ waarde: x.id, tekst: x.label })),
            ]}
          />
          {!graphic && (
            <p className="melding">{tegenover ? "Kies een andere set dan de set zelf." : "Kies een vergelijkingsset: er is geen standaard (zie Systeem)."}</p>
          )}
        </>
      )}
      {onderkant(graphic, "WinstEnVerlies")}
    </form>
  );
}

function PartijUitgelicht() {
  const { s, sets, gekozenSet, setVeld, onderkant, zetKlaar } = useCijferformulier();
  const [partij, setPartij] = useState("");
  // de Partijen met Zetels in de set eerst, van links naar rechts zoals in de Zaal
  const zetels = (p: string) => gekozenSet?.zetels[p] ?? 0;
  const volgorde = (s?.uitslagindeling.partijen ?? []).filter((p) => s?.cijfers.partijen.some((q) => q.sleutel === p));
  const opVolgorde = [...volgorde.filter((p) => zetels(p) > 0), ...volgorde.filter((p) => zetels(p) === 0)];
  const gekozen = opVolgorde.includes(partij) ? partij : opVolgorde[0];
  const graphic: Graphic | null = gekozenSet && gekozen ? { sjabloon: "PartijUitgelicht", set: gekozenSet.id, partij: gekozen } : null;
  const kort = (p: string) => s?.cijfers.partijen.find((q) => q.sleutel === p)?.kort ?? p;
  const sjabloon = SJABLONEN[6];
  return (
    <form className="paneel-detail" onSubmit={zetKlaar(graphic)}>
      <header>
        <h3 className="kop-regel" title={sjabloon.uitleg}>{sjabloon.naam}</h3>
      </header>
      {sets.length === 0 ? (
        <p className="melding">Nog geen sets: lees eerst het invoerbestand in of maak een set (zie Systeem).</p>
      ) : (
        <>
          {setVeld}
          <Keuze
            label="Partij"
            uitleg="Haar Stoelen lichten op"
            waarde={gekozen ?? ""}
            zet={setPartij}
            opties={opVolgorde.map((p) => ({ waarde: p, tekst: `${kort(p)} · ${zetels(p)} ${zetels(p) === 1 ? "zetel" : "zetels"}` }))}
          />
        </>
      )}
      {onderkant(graphic, "PartijUitgelicht")}
    </form>
  );
}

/** Een Graphic van een eigen Sjabloon: de invulvelden die het Sjabloon vraagt, en een set en een
 * Partij als zijn Blokken die nodig hebben. */
function EigenGraphic({ sjabloon }: { sjabloon: Sjabloon }) {
  const { s, sets, gekozenSet, setVeld, onderkant, zetKlaar } = useCijferformulier();
  const nodig = benodigd(sjabloon);
  const [velden, setVelden] = useState<Record<string, string>>({});
  const [partij, setPartij] = useState("");
  const partijen = (s?.uitslagindeling.partijen ?? []).filter((p) => s?.cijfers.partijen.some((q) => q.sleutel === p));
  const gekozenPartij = partijen.includes(partij) ? partij : partijen[0];
  const kort = (p: string) => s?.cijfers.partijen.find((q) => q.sleutel === p)?.kort ?? p;
  const ingevuld = Object.fromEntries(nodig.velden.map((v) => [v, (velden[v] ?? "").trim()]));
  const compleet = (!nodig.set || !!gekozenSet) && (!nodig.partij || !!gekozenPartij);
  const graphic: Graphic | null = compleet
    ? {
        sjabloon: "Eigen",
        id: sjabloon.id,
        velden: ingevuld,
        set: nodig.set ? gekozenSet?.id : undefined,
        partij: nodig.partij ? gekozenPartij : undefined,
      }
    : null;
  return (
    <form className="paneel-detail" onSubmit={zetKlaar(graphic)}>
      <header>
        <h3 className="kop-regel">{sjabloon.naam}</h3>
        <p className="zacht">Eigen Sjabloon uit Vormgeving. Vul in wat het Sjabloon vraagt.</p>
      </header>
      {nodig.velden.map((v) => (
        <label className="veld" key={v}>
          <span className="veld-kop">
            <span className="label">{v}</span>
            <span className="zacht">
              {(velden[v] ?? "").length}/{MAX_REGEL}
            </span>
          </span>
          <input value={velden[v] ?? ""} maxLength={MAX_REGEL} onChange={(e) => setVelden({ ...velden, [v]: e.target.value })} />
        </label>
      ))}
      {nodig.set && (sets.length === 0 ? <p className="melding">Nog geen sets: lees eerst het invoerbestand in of maak een set (zie Systeem).</p> : setVeld)}
      {nodig.partij && (
        <Keuze label="Partij" uitleg="Voor de Blokken met een Partij" waarde={gekozenPartij ?? ""} zet={setPartij} opties={partijen.map((p) => ({ waarde: p, tekst: kort(p) }))} />
      )}
      {onderkant(graphic, "Eigen", cameraProfielVanSjabloon(sjabloon))}
    </form>
  );
}
