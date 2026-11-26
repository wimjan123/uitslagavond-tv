// Regie-werkruimte (DaVinci Deck):
// Links: Media Pool (Presets) — visuele 16:9 kaarten + compacte lijst, directe zoekbalk,
//        drag-and-drop 1-9, en een Portal-contextmenu (nooit afgesneden!).
// Rechts: Directe Inspector — direct gekoppeld aan Preview (geen verwarrend 3e minischermpje):
//        standaard "Camera blijft lopen" zodat je een Graphic direct over een lopend pannetje legt.
import { memo, useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import {
  Camera,
  ChevronDown,
  Copy,
  Edit3,
  LayoutGrid,
  List,
  MoreHorizontal,
  Pencil,
  Plus,
  Save,
  Search,
  Send,
  Sparkles,
  Tablet,
  Trash2,
} from "lucide-react";
import { graphicTekst, sjabloonId, sjabloonNaam, standaardStandpunt } from "../graphic";
import { stoelVan } from "../persoonskaart/persoonskaart";
import { filterPresets, GROEPEN, presetTitel, statusVan } from "../presets/presets";
import type { Groep } from "../presets/presets";
import { Schets } from "../presets/Schets";
import {
  BOVEN_HET_PANEEL,
  BOVEN_HET_PANEEL_NAAM,
  cameraProfielVanGraphic,
  gerichteBewegingNaam,
  passendeCameraKeuzes,
} from "../regie/camerarit";
import type { Camerakeus } from "../regie/camerarit";
import { biedHerstel } from "../regie/herstel";
import { useRegie } from "../regie/regie";
import { PRESETS } from "../regie/take";
import { benodigd, geldendSjabloon, lijst as sjablonenLijst } from "../vormgeving/sjablonen";
import type { Graphic } from "../typen/Graphic";
import type { Indeling } from "../typen/Indeling";
import type { OvergangSoort } from "../typen/OvergangSoort";
import type { Preset } from "../typen/Preset";
import type { Stand } from "../typen/Stand";
import type { Stoelbeeld } from "../typen/Stoelbeeld";

const BLIJF = "blijf";
const KLAAR_BIJ = "inspector";

const CAMERA_GROEPEN: { soort: Camerakeus["soort"]; label: string }[] = [
  { soort: "Partijbeweging", label: "Bewegingen naar de Partij" },
  { soort: "Stoelbeweging", label: "Bewegingen naar de Stoel" },
  { soort: "Zaalbeweging", label: "Zetelverdeling-bewegingen" },
  { soort: "Coalitiebeweging", label: "Coalitie & 76-grens" },
  { soort: "Paneelbeweging", label: "Bewegingen boven het paneel" },
  { soort: "Standpunt", label: "Standpunten" },
  { soort: "Camerarit", label: "Cameraritten" },
];

function cameraLabelKort(stand: Stand, letter: string | undefined): string {
  if (!letter) return "";
  if (letter === BLIJF) return "Loopt door";
  if (letter === BOVEN_HET_PANEEL) return BOVEN_HET_PANEEL_NAAM;
  const gericht = gerichteBewegingNaam(letter);
  if (gericht) return gericht;
  const sp = stand.standpunten.find((p) => p.letter === letter);
  if (sp) return sp.toets ?? sp.letter;
  const rit = stand.cameraritten.find((r) => r.letter === letter);
  if (rit) return rit.toets ?? rit.letter;
  return letter;
}

type TemplateKeuze =
  | "Titelkaart"
  | "Zetelverdeling"
  | "Zetelbalk"
  | "UitslagPerPartij"
  | "WinstEnVerlies"
  | "TweeSets"
  | "PartijUitgelicht"
  | "Persoonskaart"
  | `eigen:${string}`;

const VASTE_TEMPLATES: { id: TemplateKeuze; naam: string; groep: string }[] = [
  { id: "Titelkaart", naam: "Titelkaart", groep: "Naambalk" },
  { id: "Zetelverdeling", naam: "Zetelverdeling", groep: "Zaal + kop" },
  { id: "Zetelbalk", naam: "Zetelbalk", groep: "Balk onderin" },
  { id: "UitslagPerPartij", naam: "Uitslag per Partij", groep: "Paneel links" },
  { id: "WinstEnVerlies", naam: "Winst en verlies", groep: "Verschil" },
  { id: "TweeSets", naam: "Twee sets", groep: "Vergelijking" },
  { id: "PartijUitgelicht", naam: "Partij uitgelicht", groep: "Focus partij" },
  { id: "Persoonskaart", naam: "Persoonskaart", groep: "Kamerlid" },
];

interface MenuPositie {
  preset: Preset;
  titel: string;
  top: number;
  left: number;
}

export function RegieWerkruimte({ opGaNaarZaal }: { opGaNaarZaal: () => void }) {
  const regie = useRegie();
  const stand = regie.stand?.stand;

  // Media Pool UI state
  const [weergave, setWeergave] = useState<"kaarten" | "lijst">(() =>
    localStorage.getItem("deck-pool-view") === "lijst" ? "lijst" : "kaarten",
  );
  const [groep, setGroep] = useState<Groep | null>(null);
  const [alleenTablet, setAlleenTablet] = useState(false);
  const [zoek, setZoek] = useState("");
  const [menuPos, setMenuPos] = useState<MenuPositie | null>(null);
  const [hernoemNr, setHernoemNr] = useState<number | null>(null);
  const [hernoemTekst, setHernoemTekst] = useState("");
  const [sleepNr, setSleepNr] = useState<number | null>(null);
  const [doelNr, setDoelNr] = useState<number | null>(null);

  // Direct Inspector state (default: Camera blijft lopen = BLIJF!)
  const [bewerkPresetNr, setBewerkPresetNr] = useState<number | null>(null);
  const [presetNaam, setPresetNaam] = useState("");
  const [sjabloonKeuze, setSjabloonKeuze] = useState<TemplateKeuze>("Titelkaart");
  const [kop, setKop] = useState("");
  const [regel, setRegel] = useState("");
  const [setId, setSetId] = useState("");
  const [tweedeSetId, setTweedeSetId] = useState("");
  const [partijSleutel, setPartijSleutel] = useState("");
  const [indeling, setIndeling] = useState<Indeling>("Uitslagindeling");
  const [wieId, setWieId] = useState("");
  const [wieSoort, setWieSoort] = useState<"Kamerlid" | "Eigen">("Kamerlid");
  const [eigenVelden, setEigenVelden] = useState<Record<string, string>>({});
  const [cameraModus, setCameraModus] = useState<"blijf" | "kies">("blijf");
  const [standpuntKeuze, setStandpuntKeuze] = useState<string>("A");
  const [overgangKeuze, setOvergangKeuze] = useState<OvergangSoort | "">("");
  const [fadeMs, setFadeMs] = useState<number>(600);
  const [zonderGraphic, setZonderGraphic] = useState(false);
  const [stoelbeeldDraft, setStoelbeeldDraft] = useState<Stoelbeeld | undefined>(undefined);

  const portalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    localStorage.setItem("deck-pool-view", weergave);
  }, [weergave]);

  useEffect(() => {
    if (!menuPos) return;
    const sluit = (e: MouseEvent) => {
      if (portalRef.current && !portalRef.current.contains(e.target as Node)) setMenuPos(null);
    };
    const sluitScroll = () => setMenuPos(null);
    window.addEventListener("mousedown", sluit);
    window.addEventListener("scroll", sluitScroll, true);
    window.addEventListener("resize", sluitScroll);
    return () => {
      window.removeEventListener("mousedown", sluit);
      window.removeEventListener("scroll", sluitScroll, true);
      window.removeEventListener("resize", sluitScroll);
    };
  }, [menuPos]);

  useEffect(() => {
    if (!stand) return;
    if (!setId && stand.cijfers.sets[0]) setSetId(stand.cijfers.sets[0].id);
    if (!tweedeSetId && stand.cijfers.sets[1]) setTweedeSetId(stand.cijfers.sets[1].id);
    if (!partijSleutel && stand.cijfers.partijen[0]) setPartijSleutel(stand.cijfers.partijen[0].sleutel);
    if (!wieId && stand.kamer.kamerleden[0]) setWieId(stand.kamer.kamerleden[0].id);
  }, [stand?.cijfers.sets.length, stand?.cijfers.partijen.length, stand?.kamer.kamerleden.length]);

  const eigenSjablonen = useMemo(() => {
    if (!stand) return [];
    return sjablonenLijst(stand.sjablonen).filter((r) => r.soort === "eigen").map((r) => r.sjabloon);
  }, [stand?.sjablonen]);

  const zichtbarePresets = useMemo(() => {
    if (!stand) return [];
    const basis = filterPresets(stand.presets, groep, alleenTablet);
    const q = zoek.trim().toLowerCase();
    if (!q) return basis;
    return basis.filter(
      (p) =>
        presetTitel(p, stand).toLowerCase().includes(q) ||
        String(p.nummer) === q ||
        (!p.zonder_graphic && sjabloonNaam(p.graphic, stand.sjablonen).toLowerCase().includes(q)),
    );
  }, [stand?.presets, stand?.sjablonen, groep, alleenTablet, zoek]);

  const conceptGraphic: Graphic | undefined = useMemo(() => {
    if (zonderGraphic) return undefined;
    const gekozenSet = setId || stand?.cijfers.sets[0]?.id || "";
    const gekozenTweede = tweedeSetId || stand?.cijfers.sets[1]?.id || gekozenSet;
    const gekozenPartij = partijSleutel || stand?.cijfers.partijen[0]?.sleutel || "";
    if (sjabloonKeuze.startsWith("eigen:")) {
      const id = sjabloonKeuze.slice("eigen:".length);
      return {
        sjabloon: "Eigen",
        id,
        velden: eigenVelden,
        set: gekozenSet || undefined,
        partij: gekozenPartij || undefined,
      };
    }
    switch (sjabloonKeuze) {
      case "Titelkaart":
        return { sjabloon: "Titelkaart", kop: kop.trim() || "Titelkaart", regel };
      case "Zetelverdeling":
        return { sjabloon: "Zetelverdeling", set: gekozenSet, indeling, namen: true, zetels: true };
      case "Zetelbalk":
        return { sjabloon: "Zetelbalk", set: gekozenSet };
      case "UitslagPerPartij":
        return { sjabloon: "UitslagPerPartij", set: gekozenSet };
      case "WinstEnVerlies":
        return { sjabloon: "WinstEnVerlies", set: gekozenSet, vergelijk_met: tweedeSetId || undefined };
      case "TweeSets":
        return { sjabloon: "TweeSets", set: gekozenSet, tweede: gekozenTweede };
      case "PartijUitgelicht":
        return { sjabloon: "PartijUitgelicht", set: gekozenSet, partij: gekozenPartij };
      case "Persoonskaart":
        return {
          sjabloon: "Persoonskaart",
          wie: wieSoort === "Eigen" ? { soort: "Eigen", id: wieId } : { soort: "Kamerlid", id: wieId },
          regel,
          extra: stand?.persoonskaart_extra ?? { leeftijd: false, woonplaats: false, geboorteplaats: false, in_de_kamer_sinds: false },
        };
    }
  }, [zonderGraphic, sjabloonKeuze, kop, regel, setId, tweedeSetId, partijSleutel, indeling, wieId, wieSoort, eigenVelden, stand?.cijfers, stand?.persoonskaart_extra]);

  const persoonHeeftStoel = useMemo(() => {
    if (!stand || sjabloonKeuze !== "Persoonskaart" || wieSoort !== "Kamerlid" || !wieId) return false;
    return !!stoelVan(stand, wieId);
  }, [stand, sjabloonKeuze, wieSoort, wieId]);

  const cameraProfiel = useMemo(() => {
    if (!conceptGraphic) return "Algemeen" as const;
    return cameraProfielVanGraphic(conceptGraphic, stand?.sjablonen ?? [], persoonHeeftStoel);
  }, [conceptGraphic, stand?.sjablonen, persoonHeeftStoel]);

  const passendeOpties = useMemo(() => {
    if (!stand) return [];
    return passendeCameraKeuzes(stand.standpunten, stand.cameraritten, cameraProfiel).filter((k) => k.letter !== BLIJF);
  }, [stand?.standpunten, stand?.cameraritten, cameraProfiel]);

  const standaardVoorConcept = useMemo(() => {
    if (!stand || !conceptGraphic) return passendeOpties[0]?.letter ?? "A";
    const sj = geldendSjabloon(stand.sjablonen, sjabloonId(conceptGraphic))?.standpunt;
    if (sj && passendeOpties.some((o) => o.letter === sj)) return sj;
    const basis = standaardStandpunt(conceptGraphic);
    if (basis.startsWith("stoel-")) return "stoel-shot";
    if (passendeOpties.some((o) => o.letter === basis)) return basis;
    return passendeOpties[0]?.letter ?? "A";
  }, [stand, conceptGraphic, passendeOpties]);

  const geldigeStandpuntKeuze = passendeOpties.some((o) => o.letter === standpuntKeuze)
    ? standpuntKeuze
    : standaardVoorConcept;

  const effectiefStandpunt = cameraModus === "blijf" ? BLIJF : geldigeStandpuntKeuze;

  function kiesSjabloon(id: TemplateKeuze) {
    setSjabloonKeuze(id);
    // Bij gerichte Graphics schakelt de camera standaard meteen mee naar een passende beweging.
    if (id === "PartijUitgelicht") {
      setCameraModus("kies");
      setStandpuntKeuze("aanschuiven");
    } else if (id === "Persoonskaart") {
      setCameraModus("kies");
      setStandpuntKeuze("stoel-shot");
    } else if (id === "Zetelverdeling") {
      setCameraModus("kies");
      setStandpuntKeuze("zaal-opbouw");
    } else if (id === "Zetelbalk" || id === "TweeSets") {
      setCameraModus("kies");
      setStandpuntKeuze(BOVEN_HET_PANEEL);
    }
  }

  function openMenuVoor(e: React.MouseEvent<HTMLButtonElement>, p: Preset, titel: string) {
    e.stopPropagation();
    const rect = e.currentTarget.getBoundingClientRect();
    const menuHoogte = 200;
    const menuBreedte = 216;
    const pastOnder = rect.bottom + menuHoogte < window.innerHeight - 12;
    const top = pastOnder ? rect.bottom + 4 : Math.max(8, rect.top - menuHoogte - 4);
    const left = Math.max(8, Math.min(window.innerWidth - menuBreedte - 8, rect.right - menuBreedte));
    setMenuPos({ preset: p, titel, top, left });
  }

  function laadPresetInInspector(p: Preset) {
    setBewerkPresetNr(p.nummer);
    setPresetNaam(p.naam ?? "");
    if (!p.standpunt || p.standpunt === BLIJF) {
      setCameraModus("blijf");
    } else {
      setCameraModus("kies");
      setStandpuntKeuze(p.standpunt);
    }
    setOvergangKeuze(p.overgang ?? "");
    setFadeMs(p.overvloei_ms ?? 600);
    setZonderGraphic(!!p.zonder_graphic);
    setStoelbeeldDraft(p.stoelbeeld);
    if (!p.zonder_graphic) {
      laadGraphicInVelden(p.graphic);
    }
  }

  function laadGraphicInVelden(g: Graphic) {
    setZonderGraphic(false);
    switch (g.sjabloon) {
      case "Titelkaart":
        setSjabloonKeuze("Titelkaart");
        setKop(g.kop);
        setRegel(g.regel);
        break;
      case "Zetelverdeling":
        setSjabloonKeuze("Zetelverdeling");
        setSetId(g.set);
        setIndeling(g.indeling);
        break;
      case "Zetelbalk":
        setSjabloonKeuze("Zetelbalk");
        setSetId(g.set);
        break;
      case "UitslagPerPartij":
        setSjabloonKeuze("UitslagPerPartij");
        setSetId(g.set);
        break;
      case "WinstEnVerlies":
        setSjabloonKeuze("WinstEnVerlies");
        setSetId(g.set);
        setTweedeSetId(g.vergelijk_met ?? "");
        break;
      case "TweeSets":
        setSjabloonKeuze("TweeSets");
        setSetId(g.set);
        setTweedeSetId(g.tweede);
        break;
      case "PartijUitgelicht":
        setSjabloonKeuze("PartijUitgelicht");
        setSetId(g.set);
        setPartijSleutel(g.partij);
        break;
      case "Persoonskaart":
        setSjabloonKeuze("Persoonskaart");
        setWieSoort(g.wie.soort);
        setWieId(g.wie.id);
        setRegel(g.regel);
        break;
      case "Eigen": {
        setSjabloonKeuze(`eigen:${g.id}`);
        const schoon: Record<string, string> = {};
        for (const [k, v] of Object.entries(g.velden)) if (v !== undefined) schoon[k] = v;
        setEigenVelden(schoon);
        if (g.set) setSetId(g.set);
        if (g.partij) setPartijSleutel(g.partij);
        break;
      }
      default:
        setSjabloonKeuze("Titelkaart");
        setKop(graphicTekst(g));
        setRegel("");
    }
  }

  function startNieuw() {
    setBewerkPresetNr(null);
    setPresetNaam("");
    setZonderGraphic(false);
    setStoelbeeldDraft(undefined);
    setSjabloonKeuze("Titelkaart");
    setKop("");
    setRegel("");
    setCameraModus("blijf");
    setOvergangKeuze("");
  }

  async function klikPreset(p: Preset) {
    if (hernoemNr === p.nummer) return;
    laadPresetInInspector(p);
    await regie.stuur({ soort: "ZetPresetKlaar", nummer: p.nummer }, PRESETS);
  }

  async function zetKlaarUitInspector() {
    if (!stand) return;
    if (overgangKeuze) {
      await regie.stuur(
        {
          soort: "ZetOvergang",
          overgang: overgangKeuze,
          overvloei_ms: overgangKeuze === "Overvloeien" ? fadeMs : undefined,
        },
        KLAAR_BIJ,
      );
    }
    if (stoelbeeldDraft) {
      await regie.stuur({ soort: "ZetStoelbeeldKlaar", stoelbeeld: stoelbeeldDraft }, KLAAR_BIJ);
    }
    if (conceptGraphic) {
      await regie.stuur(
        {
          soort: "ZetKlaar",
          graphic: conceptGraphic,
          standpunt: effectiefStandpunt,
        },
        KLAAR_BIJ,
      );
    } else {
      await regie.stuur({ soort: "ZetCameraKlaar", letter: effectiefStandpunt }, KLAAR_BIJ);
    }
  }

  async function bewaarAlsNieuwePreset() {
    if (!conceptGraphic && !stoelbeeldDraft && cameraModus === "blijf") return;
    await regie.stuur(
      {
        soort: "BewaarPresetUitgebreid",
        naam: presetNaam.trim() || undefined,
        graphic: conceptGraphic,
        standpunt: effectiefStandpunt,
        stoelbeeld: stoelbeeldDraft,
        overgang: overgangKeuze || undefined,
        overvloei_ms: overgangKeuze === "Overvloeien" ? fadeMs : undefined,
      },
      KLAAR_BIJ,
    );
  }

  async function bewaarWijzigingPreset() {
    if (bewerkPresetNr === null) return;
    await regie.stuur(
      {
        soort: "BewerkPreset",
        nummer: bewerkPresetNr,
        naam: presetNaam.trim() || undefined,
        graphic: conceptGraphic,
        zonder_graphic: zonderGraphic,
        standpunt: effectiefStandpunt,
        stoelbeeld: stoelbeeldDraft,
        overgang: overgangKeuze || undefined,
        overvloei_ms: overgangKeuze === "Overvloeien" ? fadeMs : undefined,
      },
      KLAAR_BIJ,
    );
  }

  async function wisPresetMetUndo(p: Preset) {
    const a = await regie.stuur({ soort: "WisPreset", nummer: p.nummer }, PRESETS);
    if (!a.uitgevoerd) return;
    if (bewerkPresetNr === p.nummer) setBewerkPresetNr(null);
    biedHerstel({
      tekst: "Preset gewist",
      wat: presetTitel(p, stand),
      knop: "Terugzetten",
      geldig: () => true,
      doe: (r) =>
        void r.stuur(
          {
            soort: "BewaarPresetUitgebreid",
            naam: p.naam,
            graphic: p.zonder_graphic ? undefined : p.graphic,
            standpunt: p.standpunt,
            stoelbeeld: p.stoelbeeld,
            overgang: p.overgang,
            overvloei_ms: p.overvloei_ms,
          },
          PRESETS,
        ),
    });
  }

  async function dupliceerPreset(p: Preset) {
    await regie.stuur(
      {
        soort: "BewaarPresetUitgebreid",
        naam: `${presetTitel(p, stand)} (kopie)`,
        graphic: p.zonder_graphic ? undefined : p.graphic,
        standpunt: p.standpunt,
        stoelbeeld: p.stoelbeeld,
        overgang: p.overgang,
        overvloei_ms: p.overvloei_ms,
      },
      PRESETS,
    );
  }

  async function rondHernoemenAf(p: Preset) {
    const nieuw = hernoemTekst.trim();
    setHernoemNr(null);
    await regie.stuur(
      {
        soort: "BewerkPreset",
        nummer: p.nummer,
        naam: nieuw || undefined,
        graphic: p.zonder_graphic ? undefined : p.graphic,
        zonder_graphic: !!p.zonder_graphic,
        standpunt: p.standpunt,
        stoelbeeld: p.stoelbeeld,
        overgang: p.overgang,
        overvloei_ms: p.overvloei_ms,
      },
      PRESETS,
    );
  }

  function verplaats(vanNummer: number, naarNummer: number) {
    if (vanNummer === naarNummer) return;
    void regie.stuur({ soort: "VerplaatsPreset", nummer: vanNummer, naar: naarNummer }, PRESETS);
  }

  if (!stand) {
    return (
      <div className="work-scroll" style={{ display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-3)" }}>
        Wachten op server…
      </div>
    );
  }

  const actiefEigenSjabloon = sjabloonKeuze.startsWith("eigen:")
    ? eigenSjablonen.find((s) => s.id === sjabloonKeuze.slice("eigen:".length))
    : undefined;
  const eigenNodig = actiefEigenSjabloon ? benodigd(actiefEigenSjabloon) : null;

  return (
    <div className="regie-split">
      {/* LINKS: Media Pool (Presets) */}
      <section className="media-pool" aria-label="Media Pool Presets">
        <div className="pane-head">
          <div className="pane-title">
            <span>Presets</span>
            <span className="mono" style={{ fontSize: 11.5, color: "var(--text-3)" }}>
              {zichtbarePresets.length}/{stand.presets.length}
            </span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
              <Search size={12} style={{ position: "absolute", left: 7, color: "var(--text-3)", pointerEvents: "none" }} />
              <input
                className="input sm"
                style={{ width: 136, paddingLeft: 24 }}
                value={zoek}
                onChange={(e) => setZoek(e.target.value)}
                placeholder="Zoek…"
                aria-label="Zoek in Presets"
              />
            </div>
            <div className="seg" role="group" aria-label="Weergave Presets">
              <button
                type="button"
                className={weergave === "kaarten" ? "active" : ""}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => setWeergave("kaarten")}
                title="Visuele 16:9 kaarten"
              >
                <LayoutGrid size={13} />
              </button>
              <button
                type="button"
                className={weergave === "lijst" ? "active" : ""}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => setWeergave("lijst")}
                title="Compacte lijst"
              >
                <List size={13} />
              </button>
            </div>
            <button
              type="button"
              className="btn sm amber"
              onMouseDown={(e) => e.preventDefault()}
              onClick={startNieuw}
            >
              <Plus size={13} />
              Nieuw
            </button>
          </div>
        </div>

        <div className="pool-toolbar">
          <div className="filter-pills">
            <button
              type="button"
              className={`filter-pill${groep === null ? " active" : ""}`}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => setGroep(null)}
            >
              Alle
            </button>
            {GROEPEN.map((g) => (
              <button
                key={g}
                type="button"
                className={`filter-pill${groep === g ? " active" : ""}`}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => setGroep(g)}
              >
                {g}
              </button>
            ))}
          </div>
          <button
            type="button"
            className={`btn sm${alleenTablet ? " active" : " ghost"}`}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => setAlleenTablet((v) => !v)}
            title="Alleen Presets voor de Presentator-tablet tonen"
          >
            <Tablet size={12} />
            Tablet
          </button>
        </div>

        {regie.melding?.bij === PRESETS && (
          <div style={{ padding: "8px 12px 0" }}>
            <div className="melding-balk">{regie.melding.tekst}</div>
          </div>
        )}

        <div className="pool-scroll">
          {zichtbarePresets.length === 0 ? (
            <div style={{ padding: "36px 16px", textAlign: "center", color: "var(--text-3)", display: "flex", flexDirection: "column", alignItems: "center", gap: 10 }}>
              <span>{zoek ? "Geen Presets gevonden voor deze zoekterm." : "Geen Presets in deze selectie."}</span>
              <button type="button" className="btn sm" onClick={startNieuw}>
                <Plus size={13} /> Maak eerste Graphic
              </button>
            </div>
          ) : weergave === "kaarten" ? (
            <div className="preset-grid">
              {zichtbarePresets.map((p) => {
                const st = statusVan(p, stand);
                const titel = presetTitel(p, stand);
                return (
                  <PresetKaart
                    key={p.nummer}
                    preset={p}
                    titel={titel}
                    stand={stand}
                    live={st.live}
                    klaar={st.klaar}
                    geselecteerd={bewerkPresetNr === p.nummer}
                    hernoemt={hernoemNr === p.nummer}
                    hernoemTekst={hernoemTekst}
                    sleept={sleepNr === p.nummer}
                    doel={doelNr === p.nummer && sleepNr !== p.nummer}
                    opSleepStart={() => setSleepNr(p.nummer)}
                    opSleepOver={() => setDoelNr(p.nummer)}
                    opSleepLeave={() => setDoelNr((d) => (d === p.nummer ? null : d))}
                    opDrop={() => {
                      if (sleepNr !== null) verplaats(sleepNr, p.nummer);
                      setSleepNr(null);
                      setDoelNr(null);
                    }}
                    opSleepEind={() => {
                      setSleepNr(null);
                      setDoelNr(null);
                    }}
                    opKlik={() => void klikPreset(p)}
                    opMenu={(e) => openMenuVoor(e, p, titel)}
                    opHernoemTekst={setHernoemTekst}
                    opHernoemKlaar={() => void rondHernoemenAf(p)}
                    opHernoemAnnuleer={() => setHernoemNr(null)}
                  />
                );
              })}
            </div>
          ) : (
            <div className="preset-list">
              {zichtbarePresets.map((p) => {
                const st = statusVan(p, stand);
                const titel = presetTitel(p, stand);
                return (
                  <div
                    key={p.nummer}
                    draggable
                    onDragStart={() => setSleepNr(p.nummer)}
                    onDragOver={(e) => {
                      e.preventDefault();
                      setDoelNr(p.nummer);
                    }}
                    onDrop={(e) => {
                      e.preventDefault();
                      if (sleepNr !== null) verplaats(sleepNr, p.nummer);
                      setSleepNr(null);
                      setDoelNr(null);
                    }}
                    onClick={() => void klikPreset(p)}
                    className={`preset-row${bewerkPresetNr === p.nummer ? " selected" : ""}${st.live ? " is-live" : st.klaar ? " is-preview" : ""}`}
                  >
                    <kbd className="key">{p.nummer}</kbd>
                    <div className="preset-row-thumb">
                      {!p.zonder_graphic && <Schets graphic={p.graphic} kleuren={stand.stijl.kleuren} />}
                    </div>
                    <div style={{ minWidth: 0 }}>
                      <div className="preset-title" title={titel}>{titel}</div>
                      <div className="preset-meta" title={cameraLabelKort(stand, p.standpunt)}>
                        {p.zonder_graphic ? "Stoelbeeld / Camera" : sjabloonNaam(p.graphic, stand.sjablonen)}
                        {p.standpunt ? ` · ${p.standpunt === BLIJF ? "Camera loopt door" : cameraLabelKort(stand, p.standpunt)}` : ""}
                      </div>
                    </div>
                    <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                      {st.live && <span className="preset-tally-pill live">LIVE</span>}
                      {st.klaar && <span className="preset-tally-pill preview">PREVIEW</span>}
                    </div>
                    <button
                      type="button"
                      className="btn icon sm ghost preset-menu-btn"
                      onMouseDown={(e) => e.preventDefault()}
                      onClick={(e) => openMenuVoor(e, p, titel)}
                      aria-label="Acties voor Preset"
                    >
                      <MoreHorizontal size={14} />
                    </button>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </section>

      {/* RECHTS: Directe Inspector — direct gekoppeld aan de grote Preview-monitor linksboven */}
      <aside className="inspector-pane" aria-label="Directe Inspector">
        <div className="pane-head">
          <div className="pane-title">
            <span>{bewerkPresetNr !== null ? `Preset #${bewerkPresetNr}` : "Nieuwe Graphic"}</span>
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            <button
              type="button"
              className="btn sm ghost"
              onMouseDown={(e) => e.preventDefault()}
              onClick={opGaNaarZaal}
              title="Alleen stoelen of camera instellen"
            >
              <Sparkles size={12} />
              Alleen stoelen
            </button>
            {bewerkPresetNr !== null && (
              <button
                type="button"
                className="btn sm ghost"
                onMouseDown={(e) => e.preventDefault()}
                onClick={startNieuw}
              >
                Nieuw
              </button>
            )}
          </div>
        </div>

        <div className="inspector-scroll">
          {regie.melding?.bij === KLAAR_BIJ && <div className="melding-balk">{regie.melding.tekst}</div>}

          {/* Zichtbare 1-klik schakelaar: Camera blijft lopen vs. passende beweging/Standpunt */}
          <div className="field">
            <label>Camera bij deze Graphic</label>
            <div className="seg" style={{ width: "100%" }}>
              <button
                type="button"
                style={{ flex: 1 }}
                className={cameraModus === "blijf" ? "active preview" : ""}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => setCameraModus("blijf")}
                title="Legt de Graphic over het lopende beeld/pannetje heen zonder de camera te stoppen"
              >
                Camera blijft lopen
              </button>
              <button
                type="button"
                style={{ flex: 1 }}
                className={cameraModus === "kies" ? "active amber" : ""}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  setCameraModus("kies");
                  if (!passendeOpties.some((o) => o.letter === standpuntKeuze)) {
                    setStandpuntKeuze(standaardVoorConcept);
                  }
                }}
                title="Schakel bij Take mee naar een passende beweging, Standpunt of Camerarit"
              >
                {cameraProfiel === "Partij"
                  ? "Naar Partij"
                  : cameraProfiel === "PersoonMetStoel"
                    ? "Naar Stoel"
                    : cameraProfiel === "HeleZaal"
                      ? "Zetelverdeling"
                      : cameraProfiel === "CoalitieZaal"
                        ? "Coalitie & 76"
                        : cameraProfiel === "BovenPaneel"
                          ? "Boven paneel"
                          : "Wissel camera"}
              </button>
            </div>
            {cameraModus === "kies" && (
              <>
                <select
                  className="select sm"
                  style={{ marginTop: 4 }}
                  value={geldigeStandpuntKeuze}
                  onChange={(e) => setStandpuntKeuze(e.target.value)}
                  aria-label="Kies passende camerabeweging of Standpunt"
                >
                  {CAMERA_GROEPEN.filter((g) => passendeOpties.some((o) => o.soort === g.soort)).map((g) => (
                    <optgroup key={g.soort} label={g.label}>
                      {passendeOpties
                        .filter((o) => o.soort === g.soort)
                        .map((o) => (
                          <option key={o.letter} value={o.letter}>
                            {o.tekst}
                          </option>
                        ))}
                    </optgroup>
                  ))}
                </select>
                {passendeOpties.find((o) => o.letter === geldigeStandpuntKeuze)?.toelichting && (
                  <span style={{ fontSize: 11.5, color: "var(--text-3)", marginTop: 2 }}>
                    {passendeOpties.find((o) => o.letter === geldigeStandpuntKeuze)?.toelichting}
                  </span>
                )}
              </>
            )}
          </div>

          {/* Sjabloonkiezer */}
          {!zonderGraphic && (
            <div className="field">
              <label>Sjabloon</label>
              <div className="tpl-grid">
                {VASTE_TEMPLATES.map((t) => (
                  <button
                    key={t.id}
                    type="button"
                    className={`tpl-tile${sjabloonKeuze === t.id ? " active" : ""}`}
                    onMouseDown={(e) => e.preventDefault()}
                    onClick={() => kiesSjabloon(t.id)}
                  >
                    <span className="tpl-tile-name">{t.naam}</span>
                    <span className="tpl-tile-sub">{t.groep}</span>
                  </button>
                ))}
                {eigenSjablonen.map((s) => {
                  const id: TemplateKeuze = `eigen:${s.id}`;
                  return (
                    <button
                      key={id}
                      type="button"
                      className={`tpl-tile${sjabloonKeuze === id ? " active" : ""}`}
                      onMouseDown={(e) => e.preventDefault()}
                      onClick={() => kiesSjabloon(id)}
                    >
                      <span className="tpl-tile-name">{s.naam}</span>
                      <span className="tpl-tile-sub">Eigen sjabloon</span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {/* Dynamische invulvelden met backend-maxLength */}
          {!zonderGraphic && sjabloonKeuze === "Titelkaart" && (
            <>
              <div className="field">
                <label htmlFor="insp-kop">
                  <span>Kop</span>
                  <span className="mono" style={{ fontSize: 10.5, color: "var(--text-3)" }}>{kop.length}/80</span>
                </label>
                <input
                  id="insp-kop"
                  className="input"
                  maxLength={80}
                  value={kop}
                  onChange={(e) => setKop(e.target.value)}
                  placeholder="Bijv. Voorlopige uitslag"
                />
              </div>
              <div className="field">
                <label htmlFor="insp-regel">
                  <span>Tweede regel</span>
                  <span className="mono" style={{ fontSize: 10.5, color: "var(--text-3)" }}>{regel.length}/120</span>
                </label>
                <input
                  id="insp-regel"
                  className="input"
                  maxLength={120}
                  value={regel}
                  onChange={(e) => setRegel(e.target.value)}
                  placeholder="Optioneel"
                />
              </div>
            </>
          )}

          {!zonderGraphic &&
            (sjabloonKeuze === "Zetelverdeling" ||
              sjabloonKeuze === "Zetelbalk" ||
              sjabloonKeuze === "UitslagPerPartij" ||
              sjabloonKeuze === "WinstEnVerlies" ||
              sjabloonKeuze === "TweeSets" ||
              sjabloonKeuze === "PartijUitgelicht" ||
              eigenNodig?.set) && (
              <div className="field">
                <label htmlFor="insp-set">Cijferset</label>
                <select id="insp-set" className="select" value={setId} onChange={(e) => setSetId(e.target.value)}>
                  {stand.cijfers.sets.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.label}
                    </option>
                  ))}
                  {sjabloonKeuze === "Zetelverdeling" && <option value="huidige-kamer">Huidige Kamer</option>}
                </select>
              </div>
            )}

          {!zonderGraphic && sjabloonKeuze === "Zetelverdeling" && (
            <div className="field">
              <label>Zaalindeling</label>
              <div className="seg">
                <button
                  type="button"
                  className={indeling === "Uitslagindeling" ? "active amber" : ""}
                  onClick={() => setIndeling("Uitslagindeling")}
                >
                  Uitslagindeling
                </button>
                <button
                  type="button"
                  className={indeling === "Zaalindeling" ? "active amber" : ""}
                  onClick={() => setIndeling("Zaalindeling")}
                >
                  Echte stoelen
                </button>
              </div>
            </div>
          )}

          {!zonderGraphic && (sjabloonKeuze === "TweeSets" || sjabloonKeuze === "WinstEnVerlies") && (
            <div className="field">
              <label htmlFor="insp-set2">{sjabloonKeuze === "TweeSets" ? "Tweede set" : "Vergelijk met"}</label>
              <select id="insp-set2" className="select" value={tweedeSetId} onChange={(e) => setTweedeSetId(e.target.value)}>
                {sjabloonKeuze === "WinstEnVerlies" && <option value="">Standaard vergelijkingsset</option>}
                {stand.cijfers.sets.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.label}
                  </option>
                ))}
              </select>
            </div>
          )}

          {!zonderGraphic && (sjabloonKeuze === "PartijUitgelicht" || eigenNodig?.partij) && (
            <div className="field">
              <label htmlFor="insp-partij">Partij</label>
              <select id="insp-partij" className="select" value={partijSleutel} onChange={(e) => setPartijSleutel(e.target.value)}>
                {stand.cijfers.partijen.map((p) => (
                  <option key={p.sleutel} value={p.sleutel}>
                    {p.kort || p.naam} — {p.naam}
                  </option>
                ))}
              </select>
            </div>
          )}

          {!zonderGraphic && sjabloonKeuze === "Persoonskaart" && (
            <>
              <div className="field">
                <label>Persoon</label>
                <div className="seg" style={{ marginBottom: 4 }}>
                  <button
                    type="button"
                    className={wieSoort === "Kamerlid" ? "active amber" : ""}
                    onClick={() => {
                      setWieSoort("Kamerlid");
                      if (stand.kamer.kamerleden[0]) setWieId(stand.kamer.kamerleden[0].id);
                    }}
                  >
                    Kamerlid
                  </button>
                  <button
                    type="button"
                    className={wieSoort === "Eigen" ? "active amber" : ""}
                    onClick={() => {
                      setWieSoort("Eigen");
                      if (stand.personen[0]) setWieId(stand.personen[0].id);
                    }}
                  >
                    Eigen persoon
                  </button>
                </div>
                <select className="select" value={wieId} onChange={(e) => setWieId(e.target.value)}>
                  {wieSoort === "Kamerlid"
                    ? stand.kamer.kamerleden.map((k) => (
                        <option key={k.id} value={k.id}>
                          {[k.roepnaam, k.tussenvoegsel, k.achternaam].filter(Boolean).join(" ")}
                        </option>
                      ))
                    : stand.personen.map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.naam}
                        </option>
                      ))}
                </select>
              </div>
              <div className="field">
                <label htmlFor="insp-p-regel">Ondertekst</label>
                <input id="insp-p-regel" className="input" maxLength={80} value={regel} onChange={(e) => setRegel(e.target.value)} placeholder="Bijv. Fractievoorzitter" />
              </div>
            </>
          )}

          {!zonderGraphic && eigenNodig && eigenNodig.velden.length > 0 && (
            <>
              {eigenNodig.velden.map((v) => (
                <div className="field" key={v}>
                  <label>{v}</label>
                  <input
                    className="input"
                    maxLength={120}
                    value={eigenVelden[v] ?? ""}
                    onChange={(e) => setEigenVelden((oud) => ({ ...oud, [v]: e.target.value }))}
                  />
                </div>
              ))}
            </>
          )}

          {/* 1-laags uitklapbare Overgang & Preset-naam */}
          <details className="deck-fold">
            <summary>
              <span style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
                <Camera size={13} />
                Overgang & Preset-naam
              </span>
              <ChevronDown size={13} />
            </summary>
            <div className="deck-fold-body">
              <div className="field">
                <label htmlFor="insp-pnaam">Naam in Media Pool</label>
                <input
                  id="insp-pnaam"
                  className="input sm"
                  maxLength={80}
                  value={presetNaam}
                  onChange={(e) => setPresetNaam(e.target.value)}
                  placeholder="Optioneel (anders titel van Graphic)"
                />
              </div>

              <div className="field">
                <label>Overgang bij Take</label>
                <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                  <select
                    className="select sm"
                    value={overgangKeuze}
                    onChange={(e) => setOvergangKeuze(e.target.value as OvergangSoort | "")}
                  >
                    <option value="">Volg transportbalk</option>
                    <option value="Bewegen">Beweeg</option>
                    <option value="Overvloeien">Fade (Overvloeien)</option>
                    <option value="Knippen">Knip</option>
                  </select>
                  {overgangKeuze === "Overvloeien" && (
                    <input
                      type="number"
                      className="input sm mono"
                      style={{ width: 84 }}
                      min={150}
                      max={3000}
                      step={100}
                      value={fadeMs}
                      onChange={(e) => setFadeMs(Number(e.target.value) || 600)}
                    />
                  )}
                </div>
              </div>
            </div>
          </details>
        </div>

        <div className="inspector-footer">
          <div style={{ display: "flex", gap: 6 }}>
            {bewerkPresetNr !== null ? (
              <button
                type="button"
                className="btn sm"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => void bewaarWijzigingPreset()}
              >
                <Save size={13} />
                Bewaar #{bewerkPresetNr}
              </button>
            ) : (
              <button
                type="button"
                className="btn sm"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => void bewaarAlsNieuwePreset()}
              >
                <Save size={13} />
                Bewaar Preset
              </button>
            )}
          </div>
          <button
            type="button"
            className="btn sm preview"
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => void zetKlaarUitInspector()}
          >
            <Send size={13} />
            Zet in Preview
          </button>
        </div>
      </aside>

      {/* Portal contextmenu: position: fixed in document.body zodat kaarten of scrollcontainers hem nooit afknippen! */}
      {menuPos &&
        createPortal(
          <div
            ref={portalRef}
            className="popover-portal"
            role="menu"
            style={{ top: menuPos.top, left: menuPos.left }}
          >
            <button
              type="button"
              className="popover-item"
              onClick={() => {
                laadPresetInInspector(menuPos.preset);
                setMenuPos(null);
              }}
            >
              <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                <Edit3 size={13} /> Bewerken in Inspector
              </span>
            </button>
            <button
              type="button"
              className="popover-item"
              onClick={() => {
                setHernoemNr(menuPos.preset.nummer);
                setHernoemTekst(menuPos.titel);
                setMenuPos(null);
              }}
            >
              <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                <Pencil size={13} /> Hernoemen
              </span>
            </button>
            <button
              type="button"
              className="popover-item"
              onClick={() => {
                void dupliceerPreset(menuPos.preset);
                setMenuPos(null);
              }}
            >
              <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                <Copy size={13} /> Dupliceren
              </span>
            </button>
            <button
              type="button"
              className="popover-item"
              onClick={() => {
                void regie.stuur(
                  { soort: "ZetPresetOpTablet", nummer: menuPos.preset.nummer, aan: !menuPos.preset.op_tablet },
                  PRESETS,
                );
                setMenuPos(null);
              }}
            >
              <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                <Tablet size={13} /> {menuPos.preset.op_tablet ? "Van tablet halen" : "Toon op tablet"}
              </span>
            </button>
            <div className="popover-sep" />
            <button
              type="button"
              className="popover-item danger"
              onClick={() => {
                void wisPresetMetUndo(menuPos.preset);
                setMenuPos(null);
              }}
            >
              <span style={{ display: "inline-flex", alignItems: "center", gap: 7 }}>
                <Trash2 size={13} /> Wissen
              </span>
            </button>
          </div>,
          document.body,
        )}
    </div>
  );
}

interface PresetKaartProps {
  preset: Preset;
  titel: string;
  stand: Stand;
  live: boolean;
  klaar: boolean;
  geselecteerd: boolean;
  hernoemt: boolean;
  hernoemTekst: string;
  sleept: boolean;
  doel: boolean;
  opSleepStart: () => void;
  opSleepOver: () => void;
  opSleepLeave: () => void;
  opDrop: () => void;
  opSleepEind: () => void;
  opKlik: () => void;
  opMenu: (e: React.MouseEvent<HTMLButtonElement>) => void;
  opHernoemTekst: (t: string) => void;
  opHernoemKlaar: () => void;
  opHernoemAnnuleer: () => void;
}

const PresetKaart = memo(function PresetKaart({
  preset: p,
  titel,
  stand,
  live,
  klaar,
  geselecteerd,
  hernoemt,
  hernoemTekst,
  sleept,
  doel,
  opSleepStart,
  opSleepOver,
  opSleepLeave,
  opDrop,
  opSleepEind,
  opKlik,
  opMenu,
  opHernoemTekst,
  opHernoemKlaar,
  opHernoemAnnuleer,
}: PresetKaartProps) {
  return (
    <div
      draggable={!hernoemt}
      onDragStart={opSleepStart}
      onDragOver={(e) => {
        e.preventDefault();
        opSleepOver();
      }}
      onDragLeave={opSleepLeave}
      onDrop={(e) => {
        e.preventDefault();
        opDrop();
      }}
      onDragEnd={opSleepEind}
      onClick={opKlik}
      className={`preset-card${geselecteerd ? " selected" : ""}${live ? " is-live" : klaar ? " is-preview" : ""}${sleept ? " dragging" : ""}${doel ? " drag-over" : ""}`}
      title="Klik om in Preview te zetten en in de Inspector te openen · Sleep om 1–9 te herordenen"
    >
      <div className="preset-thumb">
        {p.zonder_graphic ? (
          <div className="monitor-placeholder" style={{ background: "#121826", color: "var(--text-2)" }}>
            <Sparkles size={18} color="var(--amber)" />
            <span style={{ fontSize: 11 }}>{p.stoelbeeld ? "Stoelbeeld" : `Camera ${p.standpunt ?? ""}`}</span>
          </div>
        ) : (
          <Schets graphic={p.graphic} kleuren={stand.stijl.kleuren} />
        )}
        <div className="preset-thumb-badges">
          <span className="preset-num">{p.nummer <= 9 ? p.nummer : `#${p.nummer}`}</span>
          <div style={{ display: "flex", gap: 4 }}>
            {live && <span className="preset-tally-pill live">LIVE</span>}
            {klaar && <span className="preset-tally-pill preview">PREVIEW</span>}
          </div>
        </div>
      </div>

      <div className="preset-card-body">
        <div className="preset-card-row">
          {hernoemt ? (
            <input
              className="input sm"
              autoFocus
              maxLength={80}
              value={hernoemTekst}
              onClick={(e) => e.stopPropagation()}
              onChange={(e) => opHernoemTekst(e.target.value)}
              onBlur={opHernoemKlaar}
              onKeyDown={(e) => {
                if (e.key === "Enter") opHernoemKlaar();
                if (e.key === "Escape") opHernoemAnnuleer();
              }}
            />
          ) : (
            <span className="preset-title" title={titel}>{titel}</span>
          )}

          <button
            type="button"
            className="btn icon sm ghost preset-menu-btn"
            onMouseDown={(e) => e.preventDefault()}
            onClick={opMenu}
            aria-label="Acties voor Preset"
          >
            <MoreHorizontal size={14} />
          </button>
        </div>

        <div className="preset-meta" title={p.standpunt ? `Camera: ${cameraLabelKort(stand, p.standpunt)}` : undefined}>
          <span>{p.zonder_graphic ? "Zaal" : sjabloonNaam(p.graphic, stand.sjablonen)}</span>
          {p.standpunt && (
            <>
              <span>·</span>
              <span className="mono">{cameraLabelKort(stand, p.standpunt)}</span>
            </>
          )}
          {p.overgang === "Overvloeien" && (
            <>
              <span>·</span>
              <span>Fade</span>
            </>
          )}
          {p.op_tablet && <Tablet size={11} style={{ marginLeft: "auto", color: "var(--amber)", flexShrink: 0 }} />}
        </div>
      </div>
    </div>
  );
});
