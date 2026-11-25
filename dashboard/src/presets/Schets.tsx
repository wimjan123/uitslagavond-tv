// Het kleine beeld van een Preset (DESIGN.md, Thumbnails): een schets van wat de Renderer tekent,
// uit de bewaarde gegevens. De Titelkaart is een naambalk linksonder met kop en regel; de
// Zetelverdeling en Partij uitgelicht een naambalk linksboven (bij Partij uitgelicht alleen haar
// Stoelen in kleur); de Zetelbalk en Twee sets een paneel onderin met balken; de Uitslag per Partij
// en Winst en verlies een paneel links; de Stemmingen een paneel met de titel en de
// voor-tegenbalk (onderin, of groot bij Fracties voor en tegen en de Hoofdelijke stemming).
// Binnen de schets gelden de kleuren van de uitzending (de Stijl, docs/vormgeving.md), niet die
// van het dashboard.
import type { Cijferset } from "../typen/Cijferset";
import type { Graphic } from "../typen/Graphic";
import type { Partij } from "../typen/Partij";
import type { Stemmingsuitslag } from "../typen/Stemmingsuitslag";
import type { Stijlkleuren } from "../typen/Stijlkleuren";
import { coalitieTekst } from "../coalitie/coalitie";

/** De stemkleuren, gelijk aan `STEM_VOOR` en `STEM_TEGEN` (geen Stijlkleur). */
const VOOR = "#29B894";
const TEGEN = "#C0283A";

function alsTitelkaart(g: Graphic): [string, string] {
  switch (g.sjabloon) {
    case "Titelkaart":
      return [g.kop, g.regel];
    case "UitslagStemming":
    case "HoofdelijkeStemming":
    case "FractiesVoorEnTegen":
      return [g.uitslag.titel, ""];
    case "Zetelverdeling":
    case "Zetelbalk":
    case "UitslagPerPartij":
    case "WinstEnVerlies":
      return [g.cijfers?.set.label ?? g.set, ""];
    case "Persoonskaart":
      return [g.gegevens?.naam ?? "", g.regel];
    case "PartijUitgelicht":
      return [g.cijfers?.partijen.find((p) => p.sleutel === g.partij)?.naam ?? g.partij, g.cijfers?.set.label ?? g.set];
    case "TweeSets":
      return [g.cijfers?.set.label ?? g.tweede, ""];
    case "Coalitie":
    case "Coalitiebalk":
      return [coalitieTekst(g), ""];
    case "Eigen": {
      const [kop = "", regel = ""] = Object.values(g.velden).filter((v): v is string => !!v?.trim());
      return [kop, regel];
    }
  }
}

/** Het paneel van een Stemming: onderin (Uitslag) of groot, met kicker, titel en balk. */
function Stemmingspaneel({ u, groot, kleuren: k }: { u: Stemmingsuitslag; groot: boolean; kleuren: Stijlkleuren }) {
  const y = groot ? 20 : 101;
  const voor = (272 * u.voor) / Math.max(u.voor + u.tegen, 1);
  return (
    <>
      <rect x="16" y={y} width="288" height={groot ? 148 : 66} fill={k.vlak} fillOpacity="0.95" />
      <rect x="24" y={y + 6} width="1.7" height="5" fill={k.accent} transform={`rotate(30 24.8 ${y + 8.5})`} />
      <text x="28" y={y + 11} fontSize="5" fontWeight="800" fill={k.hoofd}>
        {u.voorlopig ? "STEMMING · VOORLOPIG" : "STEMMING"}
      </text>
      <text x="24" y={y + 24} fontSize="10" fontWeight="700" fontFamily="serif" fill={k.hoofd}>
        {kort(u.titel, 40)}
      </text>
      <rect x="24" y={y + 31} width={voor} height="12" fill={VOOR} />
      <rect x={24 + voor} y={y + 31} width={272 - voor} height="12" fill={TEGEN} />
      <text x="27" y={y + 40} fontSize="7" fontWeight="800" fill={k.tekst}>
        {u.voor} voor
      </text>
      <text x="293" y={y + 40} fontSize="7" fontWeight="800" fill={k.tekst} textAnchor="end">
        {u.tegen} tegen
      </text>
    </>
  );
}

function kort(t: string, max: number): string {
  return t.length > max ? `${t.slice(0, max - 1).trimEnd()}…` : t;
}

/** Stoelen als stippen in vier bogen, zoals de Zaal achter de Graphic. */
const STOELEN = [70, 88, 106, 124].flatMap((r, rij) => {
  const n = 14 + rij * 4;
  return Array.from({ length: n }, (_, i) => {
    const hoek = Math.PI * (0.08 + (0.84 * i) / (n - 1));
    return { x: 160 - r * Math.cos(hoek), y: 196 - r * Math.sin(hoek) * 0.62, hoek };
  });
});

/** De stippen van links naar rechts, zoals de Uitslagindeling de Stoelen vult. */
const LINKS_NAAR_RECHTS = STOELEN.map((_, i) => i).sort((a, b) => STOELEN[a].hoek - STOELEN[b].hoek);

/** De kleur van elke stip bij een Zetelverdeling: per Partij een taartpunt, grijs achteraan. */
function stipkleuren(g: Graphic): string[] {
  const kleuren = STOELEN.map(() => "#3a404c");
  if (!("cijfers" in g) || !g.cijfers) return kleuren;
  const zetels: string[] = [];
  // In de Coalitiebouwer staat de coalitie links als één blok en is de rest donker.
  const coalitie = "partijen" in g ? g.partijen : null;
  const partijen = coalitie
    ? [...g.cijfers.partijen.filter((p) => coalitie.includes(p.sleutel)), ...g.cijfers.partijen.filter((p) => !coalitie.includes(p.sleutel))]
    : g.cijfers.partijen;
  for (const p of partijen) {
    // Partij uitgelicht: alleen haar Stoelen in kleur
    const kleur =
      g.sjabloon === "PartijUitgelicht" && p.sleutel !== g.partij
        ? "#6b7280"
        : coalitie && !coalitie.includes(p.sleutel)
          ? `color-mix(in srgb, ${p.kleur} 20%, black)`
          : p.kleur;
    for (let i = 0; i < (g.cijfers.set.zetels[p.sleutel] ?? 0); i++) zetels.push(kleur);
  }
  LINKS_NAAR_RECHTS.forEach((stip, j) => {
    kleuren[stip] = zetels[Math.floor((j * 150) / STOELEN.length)] ?? "#6b7280";
  });
  return kleuren;
}

/** De Persoonskaart: een zijpaneel rechts met het fotovlak en de naam, de Zaal links. */
function PersoonskaartSchets({ graphic: g, kleuren: k, stippen }: { graphic: Extract<Graphic, { sjabloon: "Persoonskaart" }>; kleuren: Stijlkleuren; stippen: string[] }) {
  const naam = g.gegevens?.naam ?? "";
  return (
    <svg className="schets" viewBox="0 0 320 180" role="img" aria-label={`Klein beeld: Persoonskaart ${naam}`}>
      <rect width="320" height="180" fill="#0d1220" />
      {STOELEN.map((s, i) => (
        <circle key={i} cx={s.x} cy={s.y} r="2.6" fill={stippen[i]} />
      ))}
      <rect x="194" width="126" height="107" fill={k.achtergrond} fillOpacity="0.95" />
      <rect x="194" y="107" width="126" height="73" fill={k.vlak} fillOpacity="0.96" />
      <rect x="210" y="36" width="50" height="50" fill={g.gegevens?.foto ? "#5a6170" : k.hoofd} fillOpacity={g.gegevens?.foto ? 1 : 0.5} />
      <text x="202" y="136" fontSize="11" fontWeight="700" fontFamily="serif" fill={k.hoofd}>
        {kort(naam, 18)}
      </text>
      {g.regel && (
        <text x="202" y="155" fontSize="6" fill={k.tekst}>
          {kort(g.regel, 30)}
        </text>
      )}
    </svg>
  );
}

/** De Partijen van een set met hun Zetels, op volgorde, zoals de Zetelbalk ze toont (zonder overig). */
function delen(partijen: Partij[], set: Cijferset): { kleur: string; zetels: number }[] {
  return partijen.map((p) => ({ kleur: p.kleur, zetels: set.zetels[p.sleutel] ?? 0 })).filter((d) => d.zetels > 0);
}

/** Winst en verlies per Partij, van winst naar verlies, tegen de som van haar Voorgangers (zoals `Cijfers::winst_en_verlies_tegen`, bij benadering). */
function verschillen(partijen: Partij[], set: Cijferset, vorige: Cijferset): { kleur: string; verschil: number }[] {
  const opgegaan = (s: string) => partijen.some((p) => p.voorgangers.includes(s));
  return partijen
    .filter((p) => !opgegaan(p.sleutel))
    .map((p) => {
      const eerder = [p.sleutel, ...p.voorgangers].reduce((som, s) => som + (vorige.zetels[s] ?? 0), 0);
      return { kleur: p.kleur, verschil: (set.zetels[p.sleutel] ?? 0) - eerder };
    })
    .sort((a, b) => b.verschil - a.verschil);
}

/** Een Zetelbalk in de schets: `breedte` pixels voor 150 Zetels. */
function Balk({ x, y, breedte, partijen, set }: { x: number; y: number; breedte: number; partijen: Partij[]; set: Cijferset }) {
  let links = x;
  return (
    <>
      <rect x={x} y={y} width={breedte} height="7" fill="#ffd166" fillOpacity="0.2" />
      {delen(partijen, set).map((d, i) => {
        const w = (d.zetels / 150) * breedte;
        const r = <rect key={i} x={links} y={y} width={Math.max(w - 0.6, 0.3)} height="7" fill={d.kleur} />;
        links += w;
        return r;
      })}
    </>
  );
}

export function Schets({ graphic, kleuren: k }: { graphic: Graphic; kleuren: Stijlkleuren }) {
  const stemming = "uitslag" in graphic ? graphic.uitslag : null;
  const [kop, regel] = alsTitelkaart(graphic);
  const stippen = stipkleuren(graphic);
  // Een lange kop krimpt, zoals in de Renderer, tot hij op één regel past.
  const kopMaat = Math.min(14, 260 / Math.max(kop.length * 0.62, 1));
  const zetelverdeling =
    graphic.sjabloon === "Zetelverdeling" || graphic.sjabloon === "PartijUitgelicht" || graphic.sjabloon === "Coalitie";
  if (graphic.sjabloon === "Persoonskaart") return <PersoonskaartSchets graphic={graphic} kleuren={k} stippen={stippen} />;
  const c = "cijfers" in graphic ? graphic.cijfers : undefined;
  const paneel = (x: number, y: number, w: number, h: number) => (
    <>
      <rect x={x} y={y} width={w} height={h} fill={k.achtergrond} fillOpacity="0.95" />
      <text x={x + 8} y={y + 11} fontSize="5" fontWeight="800" fill={k.hoofd}>
        {graphic.sjabloon === "UitslagPerPartij"
          ? "ZETELS PER PARTIJ"
          : graphic.sjabloon === "WinstEnVerlies"
            ? "WINST EN VERLIES"
            : graphic.sjabloon === "TweeSets"
              ? "VERGELIJKING"
              : graphic.sjabloon === "Coalitiebalk"
                ? "COALITIE"
                : "ZETELS"}
      </text>
      <text x={x + 8} y={y + 23} fontSize="10" fontWeight="700" fontFamily="serif" fill={k.hoofd}>
        {kort(kop, 26)}
      </text>
    </>
  );
  if (graphic.sjabloon === "WinstEnVerlies") {
    const rijen = c?.vorige ? verschillen(c.partijen, c.set, c.vorige).slice(0, 12) : [];
    const grootste = Math.max(1, ...rijen.map((r) => Math.abs(r.verschil)));
    return (
      <svg className="schets" viewBox="0 0 320 180" role="img" aria-label={`Klein beeld: ${kop}`}>
        <rect width="320" height="180" fill="#0d1220" />
        {STOELEN.map((s, i) => (
          <circle key={i} cx={s.x} cy={s.y} r="2.6" fill={stippen[i]} />
        ))}
        {paneel(16, 20, 222, 148)}
        <rect x={150} y={52} width="0.6" height="110" fill="#ffffff" fillOpacity="0.7" />
        {rijen.map((r, i) => {
          const w = (Math.abs(r.verschil) / grootste) * 64;
          return <rect key={i} x={r.verschil < 0 ? 150 - w : 150} y={54 + i * 9} width={w} height="5.5" fill={r.kleur} />;
        })}
      </svg>
    );
  }
  if (
    graphic.sjabloon === "Zetelbalk" ||
    graphic.sjabloon === "TweeSets" ||
    graphic.sjabloon === "UitslagPerPartij" ||
    graphic.sjabloon === "Coalitiebalk"
  ) {
    const rijen = c ? delen(c.partijen, c.set).sort((a, b) => b.zetels - a.zetels).slice(0, 10) : [];
    const grootste = Math.max(1, ...rijen.map((r) => r.zetels));
    return (
      <svg className="schets" viewBox="0 0 320 180" role="img" aria-label={`Klein beeld: ${kop}`}>
        <rect width="320" height="180" fill="#0d1220" />
        {STOELEN.map((s, i) => (
          <circle key={i} cx={s.x} cy={s.y} r="2.6" fill={stippen[i]} />
        ))}
        {graphic.sjabloon === "Zetelbalk" && (
          <>
            {paneel(16, 113, 288, 55)}
            {c && <Balk x={24} y={145} breedte={272} partijen={c.partijen} set={c.set} />}
          </>
        )}
        {graphic.sjabloon === "TweeSets" && (
          <>
            {paneel(16, 69, 288, 99)}
            {c?.vorige && <Balk x={24} y={107} breedte={272} partijen={c.partijen} set={c.vorige} />}
            {c && <Balk x={24} y={139} breedte={272} partijen={c.partijen} set={c.set} />}
          </>
        )}
        {graphic.sjabloon === "Coalitiebalk" && (
          <>
            {paneel(16, 113, 288, 55)}
            {c && (
              <Balk
                x={24}
                y={145}
                breedte={272}
                partijen={c.partijen.filter((p) => graphic.partijen.includes(p.sleutel))}
                set={c.set}
              />
            )}
          </>
        )}
        {graphic.sjabloon === "UitslagPerPartij" && (
          <>
            {paneel(16, 20, 222, 148)}
            {rijen.map((r, i) => (
              <rect key={i} x={66} y={56 + i * 10.5} width={(r.zetels / grootste) * 150} height="6" fill={r.kleur} />
            ))}
          </>
        )}
      </svg>
    );
  }
  // De naambalk is zo breed als de tekst, zoals in de Renderer (bij benadering).
  const balk = zetelverdeling
    ? Math.min(170, 22 + kort(kop, 22).length * 6)
    : Math.min(290, 22 + Math.max(kop.length * kopMaat * 0.6, kort(regel, 52).length * 3.6));
  return (
    <svg className="schets" viewBox="0 0 320 180" role="img" aria-label={`Klein beeld: ${kop}`}>
      <rect width="320" height="180" fill="#0d1220" />
      {STOELEN.map((s, i) => (
        <circle key={i} cx={s.x} cy={s.y} r="2.6" fill={stippen[i]} />
      ))}
      {stemming ? (
        <Stemmingspaneel u={stemming} groot={graphic.sjabloon !== "UitslagStemming"} kleuren={k} />
      ) : zetelverdeling ? (
        <>
          <rect x="16" y="9" width={balk} height="28" fill={k.vlak} fillOpacity="0.93" />
          <rect x="24" y="14" width="1.7" height="5" fill={k.accent} transform="rotate(30 24.8 16.5)" />
          <text x="28" y="19" fontSize="5" fontWeight="800" fill={k.hoofd}>
            {graphic.sjabloon === "PartijUitgelicht"
              ? kort(regel, 30)
              : graphic.sjabloon === "Coalitie"
                ? "COALITIE"
                : "ZETELVERDELING"}
          </text>
          <text x="23" y="31" fontSize="10" fontWeight="700" fontFamily="serif" fill={k.hoofd}>
            {kort(kop, 22)}
          </text>
        </>
      ) : (
        <>
          <rect width="217" height="180" fill={`url(#waas-${k.achtergrond.slice(1)})`} />
          <defs>
            <linearGradient id={`waas-${k.achtergrond.slice(1)}`}>
              <stop offset="0.3" stopColor={k.achtergrond} stopOpacity="0.97" />
              <stop offset="0.92" stopColor={k.achtergrond} stopOpacity="0" />
            </linearGradient>
          </defs>
          <rect x="16" y="124" width={balk} height="35" fill={k.vlak} fillOpacity="0.93" />
          <text x="23" y={138 + kopMaat * 0.2} fontSize={kopMaat} fontWeight="700" fontFamily="serif" fill={k.hoofd}>
            {kop}
          </text>
          {regel && (
            <text x="23" y="153" fontSize="6.5" fill={k.tekst}>
              {kort(regel, 52)}
            </text>
          )}
        </>
      )}
    </svg>
  );
}
