// Dev/break-ui worst-case datasets for stress-testing DaVinci Deck at the RegieContext data boundary.
// Never loaded or rendered unless ?break-ui is in the URL (or import.meta.env.DEV).
import type { GenummerdeStand } from "../typen/GenummerdeStand";
import type { Graphic } from "../typen/Graphic";
import type { Preset } from "../typen/Preset";
import type { Stand } from "../typen/Stand";
import type { Stemming } from "../typen/Stemming";
import type { Melding } from "../regie/regie";

export type BreakUiModus = "live" | "worst" | "empty" | "one" | "huge";

export const BREAK_UI_MODI: { id: BreakUiModus; label: string }[] = [
  { id: "live", label: "Live server" },
  { id: "worst", label: "Worst case" },
  { id: "empty", label: "Leeg (0)" },
  { id: "one", label: "Één (1)" },
  { id: "huge", label: "120 Presets" },
];

export function leesBreakUiModus(): BreakUiModus | null {
  if (!import.meta.env.DEV) return null;
  const params = new URLSearchParams(location.search);
  if (!params.has("break-ui")) return "live";
  const v = params.get("break-ui") || "worst";
  if (v === "worst" || v === "empty" || v === "one" || v === "huge" || v === "live") return v;
  return "worst";
}

export function zetBreakUiModusInUrl(modus: BreakUiModus): void {
  const url = new URL(location.href);
  url.searchParams.set("break-ui", modus);
  window.history.replaceState(null, "", url.toString());
}

function maakPreset(nummer: number, graphic: Graphic, extra: Partial<Preset> = {}): Preset {
  return {
    nummer,
    graphic,
    zonder_graphic: false,
    op_tablet: false,
    bewaard_ms: Date.now() - nummer * 60_000,
    bewaard_door: "Aleksandra Wiśniewska-Kowalczyk",
    ...extra,
  };
}

export function bouwBreakUiStand(basis: GenummerdeStand, modus: Exclude<BreakUiModus, "live">): {
  stand: GenummerdeStand;
  melding: Melding | null;
} {
  const s: Stand = structuredClone(basis.stand);

  if (modus === "empty") {
    s.klaar = null;
    s.ether = null;
    s.klaar_stoelbeeld = undefined;
    s.klaar_stoelkleuren = [];
    s.ether_stoelbeeld = undefined;
    s.stoelkleuren = [];
    s.camera.klaar_standpunt = undefined;
    s.presets = [];
    s.cijfers.sets = [];
    s.cijfers.partijen = [];
    s.stemmingen.lijst = [];
    s.kamer.fracties = [];
    s.kamer.kamerleden = [];
    s.personen = [];
    return { stand: { volgnummer: basis.volgnummer + 9000, stand: s }, melding: null };
  }

  if (modus === "one") {
    const enePartij = s.cijfers.partijen[0] ?? {
      sleutel: "a",
      naam: "J",
      kort: "J",
      kleur: "#3B82F6",
      kleur_tijdelijk: false,
      logo: null,
      voorgangers: [],
      fractie_zonder_lijst: false,
    };
    s.cijfers.partijen = [enePartij];
    s.cijfers.sets = [
      {
        id: "set-1",
        soort: "prognose",
        label: "1",
        tijd: null,
        bron: null,
        stemmen: null,
        versie: 1,
        vergrendeld: false,
        zetels: { [enePartij.sleutel]: 1 },
      },
    ];
    s.presets = [
      maakPreset(1, { sjabloon: "Titelkaart", kop: "J", regel: "" }, { naam: "J", op_tablet: true, klaargezet_als: 1 }),
    ];
    s.klaar = {
      id: 1,
      versie: 1,
      graphic: s.presets[0].graphic,
      klaargezet_door: "Jo",
      gewijzigd_door: "Jo",
    };
    s.ether = null;
    s.kamer.fracties = [{ id: "f1", afkorting: "J", naam: "J", zetels: 1 }];
    s.stemmingen.lijst = [
      {
        punt: { id: "m-one", soort: "Motie", nummer: "1", titel: "Motie 1", indieners: "J", agendapunt: null },
        met_de_hand: false,
        voor_uitzending: true,
        vergrendeld: false,
        vastgelegd: false,
        uitkomst: { voor: 0, tegen: 0, open: 1, besluit: null },
        stemgedrag: { getikt: [], overige: null, algemene_stemmen: false, afwijkend: [], hoofdelijk: false },
        controle: { status: "WachtOpOfficieel", officieel: null, afgehandeld: null, debat_direct_gezien: null, open_data_gezien: null, beoordeling: null, officiele_uitkomst: null, verschillen: [] },
      },
    ];
    s.zaalmelding = { ...s.zaalmelding, zonder_stoel: ["Jo"] };
    return { stand: { volgnummer: basis.volgnummer + 9001, stand: s }, melding: null };
  }

  if (modus === "huge") {
    const lijst: Preset[] = [];
    for (let i = 1; i <= 120; i++) {
      lijst.push(
        maakPreset(
          i,
          {
            sjabloon: "Titelkaart",
            kop: i % 5 === 0
              ? `Uitslagavond Tweede Kamer — Tussenstand #${i} met volledige herverdeling`
              : `Preset ${i} · Uitslag`,
            regel: i % 3 === 0 ? "Tweede Kamerverkiezingen 2026" : "",
          },
          {
            op_tablet: i <= 24,
            standpunt: i % 4 === 0 ? "Q" : "A",
            overgang: i % 3 === 0 ? "Overvloeien" : undefined,
          },
        ),
      );
    }
    s.presets = lijst;
    return { stand: { volgnummer: basis.volgnummer + 9002, stand: s }, melding: null };
  }

  // modus === "worst": alle realistische randgevallen tegelijk verspreid over rijen en schermen
  const eersteSet = s.cijfers.sets[0]?.id ?? "exitpoll";
  s.camera.overgang = "Overvloeien";
  s.camera.overvloei_ms = 1500;
  s.camera.klaar_standpunt = "Q";

  s.klaar = {
    id: 700,
    versie: 701,
    graphic: {
      sjabloon: "Titelkaart",
      kop: "Verkiezingsuitslag Tweede Kamer — Voorlopige prognose met volledige herverdeling",
      regel: "Aleksandra Wiśniewska-Kowalczyk & Đặng Thị Ngọc Hân — live vanuit Den Haag",
    },
    standpunt: "Q",
    klaargezet_door: "Aleksandra Wiśniewska-Kowalczyk",
    gewijzigd_door: "Konstantin Oberhauser-Wettstein",
  };
  s.ether = {
    id: 700,
    versie: 700,
    graphic: {
      sjabloon: "Titelkaart",
      kop: "TweedeKamerverkiezingenExitpollPrognoseNoordHollandEnZuidHolland2026Definitief",
      regel: "نور الهدى عبد الرحمن · 王秀英 · 🦊 Fox",
    },
    standpunt: "A",
    getoond: 700,
    sinds_ms: Date.now() - 120_000,
    getoond_door: "Regie links",
    stijl: s.stijl,
  };
  s.klaar_stoelbeeld = {
    bron: { soort: "Uit" },
    label: "VVD + D66 + GroenLinks-PvdA + CDA + ChristenUnie + Partij voor de Dieren (uitgelicht)",
  };
  s.ether_stoelbeeld = {
    bron: { soort: "Uit" },
    label: "TweedeKamerverkiezingenStoelverdelingNoordHollandZuidHolland",
  };
  s.stoelkleuren = Array.from({ length: 150 }, (_, i) => (i % 2 === 0 ? "#29B894" : "#E5484D"));

  s.presets = [
    maakPreset(
      1,
      {
        sjabloon: "Titelkaart",
        kop: "TweedeKamerverkiezingenExitpollPrognoseNoordHollandEnZuidHolland2026Definitief",
        regel: "bartholomew.fitzgerald@northwind-industries-holdings.example.com",
      },
      {
        naam: "TweedeKamerverkiezingenExitpollPrognoseNoordHolland2026",
        standpunt: "Q",
        overgang: "Overvloeien",
        overvloei_ms: 1500,
        op_tablet: true,
        getoond_als: 700,
      },
    ),
    maakPreset(
      2,
      {
        sjabloon: "Titelkaart",
        kop: "Aleksandra Wiśniewska-Kowalczyk & Đặng Thị Ngọc Hân",
        regel: "Seán O'Brien-Ó Súilleabháin · María José de la Cruz y Fernández",
      },
      {
        naam: "Đặng Thị Ngọc Hân — Voorlopige uitslag (100% geteld)",
        standpunt: "blijf",
        op_tablet: true,
        klaargezet_als: 700,
      },
    ),
    maakPreset(
      3,
      {
        sjabloon: "Titelkaart",
        kop: "J",
        regel: "",
      },
      { naam: "J" },
    ),
    maakPreset(
      4,
      {
        sjabloon: "Titelkaart",
        kop: "<b>&amp;</b> **bold** <script>alert(1)</script>",
        regel: "نور الهدى عبد الرحمن · 王秀英 · 👩🏽‍💻 Priya",
      },
      { naam: "<b>&amp;</b> 🦊 Fox · نور الهدى عبد الرحمن" },
    ),
    maakPreset(
      5,
      { sjabloon: "Zetelverdeling", set: eersteSet, namen: true, zetels: true, indeling: "Uitslagindeling" },
      {
        zonder_graphic: true,
        naam: "Alleen stoelen · VVD + D66 + GroenLinks-PvdA + CDA + ChristenUnie",
        standpunt: "D",
        stoelbeeld: { bron: { soort: "Uit" }, label: "Stoelbeeld met heel lange beschrijving voor test" },
        overgang: "Overvloeien",
        op_tablet: true,
      },
    ),
    maakPreset(
      6,
      { sjabloon: "Titelkaart", kop: "   ", regel: "" },
      { naam: undefined },
    ),
  ];

  // Worst-case lange motie (260 tekens) + 1 open fractie + lange namen in Stemmingen
  const eersteFracties = s.kamer.fracties.slice(0, Math.max(0, s.kamer.fracties.length - 1));
  const worstMotie: Stemming = {
    punt: {
      id: "m-worst-1",
      soort: "Motie",
      nummer: "36410-VIII-42",
      titel:
        "Gewijzigde motie van de leden Wiśniewska-Kowalczyk, Đặng Thị Ngọc Hân en Van der Berg-O'Brien over het per direct instellen van een onafhankelijke staatscommissie voor de langetermijnbegroting van de publieke omroep, regionale infrastructuur en digitale weerbaarheid (t.v.v. 36410-VIII-19)",
      indieners: "Aleksandra Wiśniewska-Kowalczyk, Christopher Alexander Montgomery III, Seán O'Brien-Ó Súilleabháin",
      agendapunt: "Begroting Onderwijs, Cultuur en Wetenschap",
    },
    met_de_hand: false,
    voor_uitzending: true,
    vergrendeld: false,
    vastgelegd: false,
    uitkomst: { voor: 75, tegen: 70, open: 1, besluit: null },
    stemgedrag: {
      getikt: eersteFracties.map((f, idx) => ({ fractie: f.id, stem: idx % 2 === 0 ? "Voor" : "Tegen" })),
      overige: null,
      algemene_stemmen: false,
      afwijkend: [],
      hoofdelijk: false,
    },
    in_ether_ms: Date.now() - 60_000,
    controle: {
      status: "UitkomstAnders",
      officieel: {
        bron: "DebatDirect",
        besluit: "Verworpen",
        hoofdelijk: false,
        fracties: [],
        afwijkend: [],
        hamerslag: "2026-11-26T00:15:00Z",
      },
      afgehandeld: null,
      debat_direct_gezien: "2026-11-26T00:19:00Z",
      open_data_gezien: null,
      beoordeling: null,
      officiele_uitkomst: null,
      verschillen: [],
    },
  };
  s.stemmingen.lijst = [worstMotie, ...s.stemmingen.lijst];

  // Worst-case partij + set over 150 zetels (162) + 1 Kamerlid zonder stoel + waarschuwing
  s.cijfers.partijen = [
    {
      sleutel: "worst-long",
      naam: "Samenwerkingsverband voor Democratische Vernieuwing en Regionale Infrastructuur",
      kort: "Benachrichtigungseinstellungen",
      kleur: "#F0B44C",
      kleur_tijdelijk: false,
      logo: null,
      voorgangers: [],
      fractie_zonder_lijst: false,
    },
    ...s.cijfers.partijen,
  ];
  if (s.cijfers.sets[0]) {
    s.cijfers.sets[0].zetels["worst-long"] = 24;
  }
  s.zaalmelding = { ...s.zaalmelding, zonder_stoel: ["Aleksandra Wiśniewska-Kowalczyk"] };
  s.cijfers.laatste_invoer = {
    bestand: "Q3-uitslag-DEFINITIEF-(herzien)-v12-[goedgekeurd].json",
    ontvangen: "23.59",
    partijen_nieuw: [],
    sets_binnen: [],
    sets_ongewijzigd: [],
    overgeslagen: [],
    waarschuwingen: ["Set Exitpoll komt uit op 174 Zetels in plaats van 150 Zetels"],
  };

  return {
    stand: { volgnummer: basis.volgnummer + 9003, stand: s },
    melding: {
      bij: "take",
      tekst: "Gewijzigd door Aleksandra Wiśniewska-Kowalczyk op Werkplek Regie-Noord-Oost-2",
    },
  };
}
