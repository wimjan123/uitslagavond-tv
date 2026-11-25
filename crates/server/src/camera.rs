//! De camerarekening: de server rekent de camera's 100 keer per seconde uit (ADR 0002).
//!
//! [`Cameraregie`] volgt de Stand en geeft per tik een [`Camerabeeld`] met de camera van de
//! Ether en die van het Voorbeeld. Een camera beweegt volgens een [`Beweging`]: een boog van de
//! vorige stand naar het doel, en daarna stil of langzaam schuiven. Met de Vrije camera aan
//! stuurt de gamepad hem ([`Stuur`], [`Cameraregie::stap`]). Alles hangt alleen af van de Stand,
//! de stick en het tiknummer, dus elke Renderer krijgt precies dezelfde reeks.
//!
//! Een Camerarit (#45) is een [`Beweging`] die na de overgang naar zijn beginpunt de opgenomen
//! [`Ritbaan`] rijdt. Tijdens Oefenen stuurt de gamepad alleen het [`Spoor`] van het Voorbeeld,
//! en neemt de regie desgevraagd de standen ervan op. Alles blijft binnen de [`Ruimte`].

use std::sync::{Arc, LazyLock};

use gedeeld::Stand;
use gedeeld::camera::{
    CameraOvergang, Camerabeeld, Camerakeuze, Camerapunt, Camerastand, NaAankomst, OvergangSoort,
    BLIJF, SCHUIVEN_S, Standpunt, TIKKEN_PER_SECONDE, aanschuif_overgang, aanschuif_stand, aanschuif_uit_sleutel,
    shot_op_stoel, stoel_uit_sleutel,
};
use gedeeld::camerarit::{Camerarit, OPNAME_MAX_S, RitBegin, Ritbaan, als_opnamepunt};
use serde::Deserialize;

/// De onzichtbare ruimte waarin de camera mag vliegen: vloer net boven de banken, wanden binnen
/// de muren, plafond onder de lampen. Getekend in Blender naast het model door `zaal/ruimte.py`
/// (zie `zaal/README.md`); de server leest de maten uit `crates/gedeeld/src/ruimte.json`.
///
/// - Wanden: een halve cirkel om het midden van de Zaal (aan de kant van -z), en voor de
///   Voorzitter een rechthoek tot de voorwand.
/// - Vloer: achter per afstand tot het midden de hoogte boven de banken, voor de Voorzitter één
///   hoogte boven balie en tafels. Over de eerste meter aan weerskanten van z = 0 loopt de ene
///   zacht over in de andere, zodat de vloer nergens een sprong maakt.
/// - Plafond: onder de hanglampen.
///
/// Losse voorwerpen (microfoons, schermen, de katheder) zitten er niet in: daar botst de camera
/// niet mee.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Ruimte {
    /// Straal van de halve cirkel om (0, 0) aan de kant van -z.
    pub straal: f32,
    /// Halve breedte (x) van het deel voor de Voorzitter, aan de kant van +z.
    pub voor_halve_breedte: f32,
    /// Hoe ver het deel voor de Voorzitter loopt (z).
    pub voor_diepte: f32,
    /// De vloer achter: `[afstand tot het midden, hoogte]`, op volgorde van afstand. Tussen twee
    /// punten loopt hij recht.
    pub vloer_rond: Vec<[f32; 2]>,
    /// De vloer voor de Voorzitter.
    pub vloer_voor: f32,
    pub plafond: f32,
}

static ZAAL: LazyLock<Ruimte> = LazyLock::new(|| {
    serde_json::from_str(gedeeld::camera::RUIMTE_JSON).expect("ruimte.json uit zaal/ruimte.py")
});

/// Over zoveel meter rond z = 0 gaan de vloer achter en voor in elkaar over.
const VLOER_OVERGANG: f32 = 1.0;

/// De studio boven en achter de Voorzitter: daar staat Totaal vanaf voorzitter (Standpunt A),
/// hoog boven het plafond en achter de voorwand. De Renderer laat het plafond en wat erboven
/// hangt weg zolang de camera daar is (`crates/renderer/src/schil.rs`), zoals in een virtuele
/// studio.
///
/// - Voor de Voorzitter (vanaf [`STUDIO_HELLING_VAN`]) loopt het plafond van de ruimte zacht op
///   tot [`STUDIO_HOOGTE`] bij de voorwand van de ruimte. Zo glijdt de camera van de studio de
///   Zaal in zonder sprong. Boven de halve cirkel, waar de hanglampen hangen, blijft het plafond
///   zoals het was.
/// - Achter de voorwand van de ruimte (tot [`STUDIO_DIEPTE`]) mag de camera alleen boven
///   [`STUDIO_VLOER`]. Daar ziet hij de voorwand en de nis niet (die horen bij de schil), dus
///   de vloer ligt zo laag dat een route van Standpunt A naar een Stoel vooraan er niet langs
///   schampt (ronde 4: met A op 12,5 m achter het midden zakt die route bij de voorwand tot
///   ongeveer 4,5 m).
const STUDIO_HELLING_VAN: f32 = 0.0;
const STUDIO_HOOGTE: f32 = 14.0;
const STUDIO_DIEPTE: f32 = 14.0;
const STUDIO_VLOER: f32 = 4.0;

impl Ruimte {
    /// De ruimte in de Zaal, zoals getekend in Blender.
    pub fn zaal() -> Ruimte {
        ZAAL.clone()
    }

    fn vloer_rond_op(&self, r: f32) -> f32 {
        let v = &self.vloer_rond;
        match v.iter().position(|p| p[0] >= r) {
            None => v.last().map_or(0.0, |p| p[1]),
            Some(0) => v[0][1],
            Some(i) => {
                let ([r0, y0], [r1, y1]) = (v[i - 1], v[i]);
                y0 + (y1 - y0) * (r - r0) / (r1 - r0).max(1e-6)
            }
        }
    }

    /// Hoogte van de vloer van de ruimte op deze plek.
    pub fn vloer_op(&self, x: f32, z: f32) -> f32 {
        let glad = |t: f32| {
            let t = t.clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let (laag, t) = if z > 0.0 {
            (self.vloer_rond_op(x.abs()), z)
        } else {
            (self.vloer_rond_op((x * x + z * z).sqrt()), -z)
        };
        let gemengd = laag + (self.vloer_voor - laag) * (1.0 - glad(t / VLOER_OVERGANG));
        if z > 0.0 {
            gemengd.max(self.vloer_voor)
        } else {
            gemengd.max(laag)
        }
    }

    /// Hoe hoog de camera op deze z mag: het plafond van de ruimte, voor de Voorzitter zacht
    /// oplopend naar de studio.
    pub fn plafond_op(&self, z: f32) -> f32 {
        let t = ((z - STUDIO_HELLING_VAN) / (self.voor_diepte - STUDIO_HELLING_VAN)).clamp(0.0, 1.0);
        // zacht begin en eind: wie langs het plafond glijdt, voelt geen knik
        self.plafond + (STUDIO_HOOGTE - self.plafond) * t * t * (3.0 - 2.0 * t)
    }

    /// Of een punt in de studio achter de voorwand van de ruimte ligt.
    fn in_studio_achter(&self, p: [f32; 3], marge: f32) -> bool {
        let [x, y, z] = p;
        z > self.voor_diepte - marge
            && z <= STUDIO_DIEPTE - marge
            && x.abs() <= self.voor_halve_breedte - marge
            && y >= STUDIO_VLOER + marge
            && y <= STUDIO_HOOGTE - marge
    }

    fn binnen_wanden(&self, x: f32, z: f32, marge: f32) -> bool {
        let in_cirkel = z <= 0.0 && x * x + z * z <= (self.straal - marge).powi(2);
        let voor = z >= -marge
            && z <= self.voor_diepte - marge
            && x.abs() <= self.voor_halve_breedte - marge;
        in_cirkel || voor
    }

    /// Of een punt in de ruimte ligt, met `marge` meter afstand tot elke rand.
    pub fn bevat(&self, p: [f32; 3], marge: f32) -> bool {
        let [x, y, z] = p;
        let binnen = self.binnen_wanden(x, z, marge)
            && y >= self.vloer_op(x, z) + marge
            && y <= self.plafond_op(z) - marge;
        binnen || self.in_studio_achter(p, marge)
    }

    /// Het dichtstbijzijnde punt in de ruimte, met `marge` meter afstand tot elke rand.
    fn klem_binnen(&self, p: [f32; 3], marge: f32) -> [f32; 3] {
        if self.in_studio_achter(p, marge) {
            return p;
        }
        let [mut x, y, mut z] = p;
        if !self.binnen_wanden(x, z, marge) {
            // naar de rechthoek voor de Voorzitter, of naar de halve cirkel; de dichtstbijzijnde
            let (w, d, straal) = (
                self.voor_halve_breedte - marge,
                self.voor_diepte - marge,
                self.straal - marge,
            );
            let rx = x.clamp(-w, w);
            let rz = z.clamp(0.0, d);
            let cz = z.min(0.0);
            let r = (x * x + cz * cz).sqrt();
            let (kx, kz) = if r > straal {
                (x * straal / r, cz * straal / r)
            } else {
                (x, cz)
            };
            let afstand = |a: f32, b: f32| (a - x).powi(2) + (b - z).powi(2);
            (x, z) = if afstand(rx, rz) <= afstand(kx, kz) {
                (rx, rz)
            } else {
                (kx, kz)
            };
        }
        let vloer = self.vloer_op(x, z) + marge;
        let y = y.clamp(vloer, (self.plafond_op(z) - marge).max(vloer));
        [x, y, z]
    }

    /// Het dichtstbijzijnde punt in de ruimte: zo glijdt de camera langs de rand.
    pub fn klem(&self, p: [f32; 3]) -> [f32; 3] {
        self.klem_binnen(p, 0.0)
    }
}

/// Wat de gamepad op dit moment doet. Sticks van -1 tot 1 (rechts en omhoog positief), triggers
/// van 0 tot 1. De server leest hem uit (`gamepad`); tests bootsen hem na.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stuur {
    /// Linker stick: opzij en vooruit.
    pub links: [f32; 2],
    /// Rechter stick: rondkijken (draaien en omhoog of omlaag kijken).
    pub rechts: [f32; 2],
    /// Rechter trigger.
    pub omhoog: f32,
    /// Linker trigger.
    pub omlaag: f32,
    /// Rechter bumper.
    pub inzoomen: bool,
    /// Linker bumper.
    pub uitzoomen: bool,
    /// Snel in plaats van langzaam (A wisselt).
    pub snel: bool,
}

impl Stuur {
    /// Niets ingedrukt, langzaam.
    pub const LOS: Stuur = Stuur {
        links: [0.0; 2],
        rechts: [0.0; 2],
        omhoog: 0.0,
        omlaag: 0.0,
        inzoomen: false,
        uitzoomen: false,
        snel: false,
    };

    /// Wat de camera wil: vooruit, opzij, omhoog, draaien, omhoog kijken en inzoomen van -1 tot
    /// 1, en snel van 0 tot 1. Een stick doet pas iets buiten de dode zone, en loopt daarna
    /// kwadratisch op: zo kan de Regie heel langzaam bewegen.
    fn wens(&self) -> [f32; 7] {
        let [opzij, vooruit] = stick(self.links);
        let [draai, kijk] = stick(self.rechts);
        let trigger = |t: f32| ((t - 0.05) / 0.95).clamp(0.0, 1.0);
        let zoom = f32::from(u8::from(self.inzoomen)) - f32::from(u8::from(self.uitzoomen));
        [
            vooruit,
            opzij,
            trigger(self.omhoog) - trigger(self.omlaag),
            draai,
            kijk,
            zoom,
            f32::from(u8::from(self.snel)),
        ]
    }
}

impl Stuur {
    /// Of de Regie iets beweegt: een stick buiten de dode zone, een trigger of een bumper.
    pub fn stuurt(&self) -> bool {
        self.wens()[..6].iter().any(|w| *w != 0.0)
    }
}

/// Een stick buiten de dode zone, kwadratisch, met lengte hooguit 1.
fn stick(v: [f32; 2]) -> [f32; 2] {
    const DODE_ZONE: f32 = 0.15;
    let v = v.map(|a| if a.is_finite() { a } else { 0.0 });
    let m = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if m <= DODE_ZONE {
        return [0.0; 2];
    }
    let k = ((m - DODE_ZONE) / (1.0 - DODE_ZONE)).min(1.0);
    [v[0] / m * k * k, v[1] / m * k * k]
}

/// Na loslaten staat de camera binnen zoveel seconden stil; even lang duurt optrekken.
pub const UITGLIJDEN_S: f32 = 0.5;
/// Snelheden van de Vrije camera, langzaam en snel.
const RIJDEN: [f32; 2] = [0.6, 2.0]; // meter per seconde
const DRAAIEN: [f32; 2] = [25.0, 70.0]; // graden per seconde
const ZOOMEN: [f32; 2] = [10.0, 30.0]; // graden beeldhoek per seconde
/// Verder omhoog of omlaag kijken kan niet: zo slaat de camera nooit over de kop.
const MAX_HELLING: f32 = 85.0;
const BEELDHOEK: [f32; 2] = [10.0, 100.0];
/// Binnen zoveel meter van de rand remt de camera af in de richting van de rand.
const ZACHTE_RAND: f32 = 0.5;

/// De camera die de gamepad stuurt.
#[derive(Clone, Debug, PartialEq)]
pub struct VrijeCamera {
    stand: Camerastand,
    /// [`Stuur::wens`] na gladmaken: elk deel loopt hooguit in [`UITGLIJDEN_S`] van vol naar nul.
    glad: [f32; 7],
    /// Uit: de stick doet niets meer, de camera glijdt uit en blijft staan.
    stuurbaar: bool,
}

impl VrijeCamera {
    fn nieuw(stand: Camerastand) -> Self {
        VrijeCamera {
            stand,
            glad: [0.0; 7],
            stuurbaar: true,
        }
    }

    /// Eén tik verder.
    fn stap(&mut self, stuur: &Stuur, ruimte: &Ruimte) {
        let dt = 1.0 / TIKKEN_PER_SECONDE as f32;
        let mut wens = if self.stuurbaar {
            stuur.wens()
        } else {
            Stuur::LOS.wens()
        };
        if !self.stuurbaar {
            wens[6] = self.glad[6];
        }
        let max = dt / UITGLIJDEN_S;
        for (g, w) in self.glad.iter_mut().zip(wens) {
            *g += (w - *g).clamp(-max, max);
        }
        let [vooruit, opzij, omhoog, draai, kijk, zoom, snel] = self.glad;
        let tempo = |[langzaam, vlug]: [f32; 2]| langzaam + (vlug - langzaam) * snel;
        let s = &mut self.stand;

        s.draai -= draai * tempo(DRAAIEN).to_radians() * dt;
        s.draai = (s.draai + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let grens = MAX_HELLING.to_radians();
        s.helling = (s.helling + kijk * tempo(DRAAIEN).to_radians() * dt).clamp(-grens, grens);
        s.beeldhoek = (s.beeldhoek - zoom * tempo(ZOOMEN).to_radians() * dt)
            .clamp(BEELDHOEK[0].to_radians(), BEELDHOEK[1].to_radians());

        // vooruit is horizontaal in de kijkrichting: omhoog en omlaag gaan met de triggers
        let (sd, cd) = s.draai.sin_cos();
        let mut v = scale(
            add(
                add(
                    scale([-sd, 0.0, -cd], vooruit),
                    scale([cd, 0.0, -sd], opzij),
                ),
                [0.0, omhoog, 0.0],
            ),
            tempo(RIJDEN),
        );
        // Bij de rand remt de camera af in de richting van de rand, en houdt hij zijn snelheid
        // langs de rand: zo glijdt hij er zacht langs.
        let binnen = ruimte.klem_binnen(s.positie, ZACHTE_RAND);
        let naar_rand = sub(s.positie, binnen);
        let diepte = len(naar_rand);
        if diepte > 1e-6 {
            let n = scale(naar_rand, 1.0 / diepte);
            let uit = dot(v, n);
            if uit > 0.0 {
                let rem = (diepte / ZACHTE_RAND).min(1.0);
                v = sub(v, scale(n, uit * rem));
            }
        }
        s.positie = ruimte.klem(add(s.positie, scale(v, dt)));
    }
}

/// Een boog van de ene plek naar de andere: een Bézierkromme met drie stukken.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Route {
    punten: [[f32; 3]; 4],
}

/// Midden van de open ruimte boven de vloer van de Zaal: de boog buigt die kant op.
const MIDDEN: [f32; 3] = [0.0, 3.4, -3.5];
/// Hoe ver de boog uitbuigt, als deel van de afstand.
const BOOG: f32 = 0.25;
/// Punten waarop een route gecontroleerd wordt.
const CONTROLEPUNTEN: usize = 64;

impl Route {
    /// De route van `van` naar `naar`: een boog naar het midden van de Zaal.
    /// Komt de boog buiten de ruimte, dan buigt hij minder, tot een rechte lijn.
    pub fn boog(van: [f32; 3], naar: [f32; 3], ruimte: &Ruimte) -> Route {
        let d = sub(naar, van);
        let lengte = len(d);
        let midden = lerp3(van, naar, 0.5);
        // horizontaal naar het midden van de Zaal, loodrecht op de koorde
        let koorde = norm([d[0], 0.0, d[2]]);
        let mut opzij = [MIDDEN[0] - midden[0], 0.0, MIDDEN[2] - midden[2]];
        let langs = dot(opzij, koorde);
        opzij = sub(opzij, scale(koorde, langs));
        if len(opzij) < 0.2 {
            opzij = [-koorde[2], 0.0, koorde[0]];
        }
        let opzij = norm(opzij);
        // en in de hoogte naar het midden: hoog onder de lampen door duikt hij omlaag
        let hoogte = (MIDDEN[1] - midden[1]).clamp(-0.5 * BOOG * lengte, 0.5 * BOOG * lengte);
        let bocht = add(scale(opzij, BOOG * lengte), [0.0, hoogte, 0.0]);
        // De grootste boog die in de ruimte blijft. Blijft geen enkele erin (van of naar de
        // studio boven het plafond), dan de boog die er het minst buiten komt: de camera glijdt
        // daar langs de rand (zie [`Ruimte::klem`]).
        let mut beste: Option<(f32, Route)> = None;
        for factor in [1.0, 0.5, 0.25, 0.0] {
            let b = scale(bocht, factor);
            let route = Route {
                punten: [
                    van,
                    add(lerp3(van, naar, 1.0 / 3.0), b),
                    add(lerp3(van, naar, 2.0 / 3.0), b),
                    naar,
                ],
            };
            let buiten: f32 = (0..=CONTROLEPUNTEN)
                .map(|i| {
                    let p = route.punt(i as f32 / CONTROLEPUNTEN as f32);
                    len(sub(ruimte.klem(p), p))
                })
                .sum();
            if buiten <= 0.0 {
                return route;
            }
            if beste.as_ref().is_none_or(|(b, _)| buiten < *b - 1e-3) {
                beste = Some((buiten, route));
            }
        }
        beste.expect("vier bogen geprobeerd").1
    }

    /// Het punt bij `s` van 0 (begin) tot 1 (eind).
    pub fn punt(&self, s: f32) -> [f32; 3] {
        let [a, b, c, d] = self.punten;
        let t = 1.0 - s;
        let w = [t * t * t, 3.0 * t * t * s, 3.0 * t * s * s, s * s * s];
        [0, 1, 2].map(|i| w[0] * a[i] + w[1] * b[i] + w[2] * c[i] + w[3] * d[i])
    }
}

/// Wat één camera doet vanaf een tik: een overgang naar het doel, daarna wat het Standpunt na
/// aankomst wil, of de Camerarit.
#[derive(Clone, Debug, PartialEq)]
pub struct Beweging {
    begin: u64,
    van: Camerastand,
    naar: Camerastand,
    route: Route,
    tikken: u64,
    overgang: CameraOvergang,
    na_aankomst: NaAankomst,
    /// Na de overgang rijdt de camera deze Camerarit, zonder pauze.
    rit: Option<Arc<Ritbaan>>,
}

impl Beweging {
    /// Meteen op `stand`, zonder overgang.
    fn knip(tik: u64, stand: Camerastand, na_aankomst: NaAankomst) -> Self {
        Beweging {
            begin: tik,
            van: stand,
            naar: stand,
            route: Route {
                punten: [stand.positie; 4],
            },
            tikken: 0,
            overgang: CameraOvergang::default(),
            na_aankomst,
            rit: None,
        }
    }

    /// Van `van` in een boog naar `naar`, volgens de overgang.
    fn overgang(tik: u64, van: Camerastand, naar: Camerastand, overgang: CameraOvergang, ruimte: &Ruimte) -> Self {
        Beweging {
            begin: tik,
            van,
            naar,
            route: Route::boog(van.positie, naar.positie, ruimte),
            tikken: (overgang.duur() * TIKKEN_PER_SECONDE as f32).round() as u64,
            overgang,
            na_aankomst: NaAankomst::Stil,
            rit: None,
        }
    }

    /// De stand op deze tik.
    pub fn stand_op(&self, tik: u64, ruimte: &Ruimte) -> Camerastand {
        let verstreken = tik.saturating_sub(self.begin);
        if verstreken < self.tikken {
            let t = verstreken as f32 / self.tikken as f32;
            let s = self.overgang.voortgang(t);
            let hoek = |a: f32, b: f32| a + kortste_hoek(b - a) * s;
            return Camerastand {
                positie: ruimte.klem(self.route.punt(s)),
                draai: hoek(self.van.draai, self.naar.draai),
                helling: hoek(self.van.helling, self.naar.helling),
                beeldhoek: self.van.beeldhoek + (self.naar.beeldhoek - self.van.beeldhoek) * s,
            };
        }
        if let Some(i) = self.ritplek(tik) {
            let mut s = self.rit.as_ref().expect("ritplek vraagt een rit").standen[i];
            s.positie = ruimte.klem(s.positie);
            return s;
        }
        let mut stand = self.naar;
        if let NaAankomst::Schuiven { richting, cm_per_s } = self.na_aankomst {
            let sec = (verstreken - self.tikken) as f32 / TIKKEN_PER_SECONDE as f32;
            let afstand = cm_per_s / 100.0 * geschoven(sec);
            stand.positie = ruimte.klem(add(stand.positie, scale(norm(richting), afstand)));
        }
        stand
    }

    /// Welke stand van de Camerarit de camera op deze tik rijdt, of geen (zonder rit, of nog in
    /// de overgang erheen).
    fn ritplek(&self, tik: u64) -> Option<usize> {
        let rit = self.rit.as_ref()?;
        let verstreken = tik.saturating_sub(self.begin).checked_sub(self.tikken)? as usize;
        let laatste = rit.standen.len() - 1;
        Some(if rit.herhalen && laatste > 0 {
            verstreken % laatste
        } else {
            verstreken.min(laatste)
        })
    }

    /// Waar in de opname van de Camerarit de camera op deze tik is, in seconden.
    fn opname_s(&self, tik: u64) -> Option<f32> {
        let i = self.ritplek(tik)?;
        self.rit.as_ref().map(|r| r.opname_s(i))
    }

    /// De tik vanaf waar de camera stilstaat: aangekomen en uitgeschoven, of de rit uit.
    pub fn stil_na(&self) -> u64 {
        let na = match (&self.rit, self.na_aankomst) {
            (Some(r), _) if r.herhalen => return u64::MAX,
            (Some(r), _) => r.standen.len() as u64 - 1,
            (None, NaAankomst::Stil) => 0,
            (None, NaAankomst::Schuiven { .. }) => (SCHUIVEN_S * TIKKEN_PER_SECONDE as f32) as u64,
        };
        self.begin + self.tikken + na
    }
}

/// Hoeveel seconden "op volle snelheid" er geschoven is na `sec` seconden: één seconde zacht
/// optrekken, één seconde zacht afremmen, stop na [`SCHUIVEN_S`].
fn geschoven(sec: f32) -> f32 {
    let t = sec.clamp(0.0, SCHUIVEN_S);
    // snelheid loopt lineair op van 0 naar 1 in de eerste seconde en terug in de laatste
    let op = |t: f32| if t < 1.0 { t * t / 2.0 } else { t - 0.5 };
    op(t) - op((t - (SCHUIVEN_S - 1.0)).max(0.0))
}

/// Waar een camera heen gaat: een Standpunt of een Camerarit.
#[derive(Clone, Debug, PartialEq)]
enum Doel {
    Standpunt(Standpunt),
    Rit(Camerarit),
}

impl Doel {
    /// Het Standpunt of de Camerarit met deze sleutel, of het shot van een Stoel (Persoonskaart):
    /// dat gaat als een Standpunt met de gewone overgang, en staat daarna stil.
    fn van(stand: &Stand, letter: &str) -> Option<Doel> {
        stand
            .standpunt(letter)
            .map(|s| Doel::Standpunt(s.clone()))
            .or_else(|| stand.camerarit(letter).map(|r| Doel::Rit(r.clone())))
            .or_else(|| {
                let stoel = stoel_uit_sleutel(letter)?;
                Some(Doel::Standpunt(Standpunt {
                    letter: letter.into(),
                    naam: format!("Stoel {}-{}-{}", stoel.vak, stoel.rij, stoel.nummer),
                    vast: true,
                    toets: None,
                    stand: shot_op_stoel(stoel)?,
                    overgang: CameraOvergang::default(),
                    na_aankomst: NaAankomst::Stil,
                }))
            })
            .or_else(|| {
                // Aanschuiven: langzaam naar het blok van een Partij, en daarna stil
                let (van, tot) = aanschuif_uit_sleutel(letter)?;
                Some(Doel::Standpunt(Standpunt {
                    letter: letter.into(),
                    naam: "Aanschuiven".into(),
                    vast: true,
                    toets: None,
                    stand: aanschuif_stand(van, tot)?,
                    overgang: aanschuif_overgang(),
                    na_aankomst: NaAankomst::Stil,
                }))
            })
            .or_else(|| {
                (letter == gedeeld::camera::BOVEN_HET_PANEEL).then(|| {
                    Doel::Standpunt(Standpunt {
                        letter: letter.into(),
                        naam: "Totaal boven het paneel".into(),
                        vast: true,
                        toets: None,
                        stand: gedeeld::camera::boven_het_paneel(),
                        overgang: CameraOvergang::default(),
                        na_aankomst: NaAankomst::Stil,
                    })
                })
            })
    }

    /// Het Doel met deze sleutel, of anders Standpunt A (dat is er altijd, tenzij de Stand stuk is).
    fn of_a(stand: &Stand, letter: &str) -> Doel {
        Doel::van(stand, letter).unwrap_or_else(|| Doel::Standpunt(standpunt_of_a(stand, "A")))
    }

    /// De beweging erheen vanaf `van`, en of dat een knip is.
    fn beweging(&self, tik: u64, van: Camerastand, knip: bool, ruimte: &Ruimte) -> (Beweging, bool) {
        match self {
            Doel::Standpunt(s) => {
                if knip || s.overgang.soort == OvergangSoort::Knippen {
                    (Beweging::knip(tik, s.stand, s.na_aankomst), true)
                } else {
                    let mut b = Beweging::overgang(tik, van, s.stand, s.overgang, ruimte);
                    b.na_aankomst = s.na_aankomst;
                    (b, false)
                }
            }
            Doel::Rit(r) => {
                let baan = Arc::new(r.baan());
                let begin = baan.standen[0];
                let knip = knip || r.bijschaving.begin == RitBegin::Knippen;
                let mut b = if knip {
                    Beweging::knip(tik, begin, NaAankomst::Stil)
                } else {
                    Beweging::overgang(tik, van, begin, CameraOvergang::default(), ruimte)
                };
                b.rit = Some(baan);
                (b, knip)
            }
        }
    }
}

/// Eén camera: wat hij doet en hoe vaak hij al knipte.
#[derive(Clone, Debug, PartialEq)]
pub struct Spoor {
    pub beweging: Beweging,
    /// Met de Vrije camera gaat de beweging niet op: dan staat de camera waar deze hem zet.
    pub vrij: Option<VrijeCamera>,
    pub snede: u32,
}

impl Spoor {
    fn stand(&self, tik: u64, ruimte: &Ruimte) -> Camerastand {
        match &self.vrij {
            Some(v) => v.stand,
            None => self.beweging.stand_op(tik, ruimte),
        }
    }

    fn punt(&self, tik: u64, ruimte: &Ruimte) -> Camerapunt {
        Camerapunt {
            stand: self.stand(tik, ruimte),
            snede: self.snede,
        }
    }

    /// Naar een Standpunt of Camerarit, vanaf `van`.
    fn ga_naar(&mut self, tik: u64, van: Camerastand, doel: &Doel, knip: bool, ruimte: &Ruimte) {
        let (beweging, knipt) = doel.beweging(tik, van, knip, ruimte);
        self.beweging = beweging;
        self.vrij = None;
        if knipt {
            self.snede = self.snede.wrapping_add(1);
        }
    }

    /// Vanaf hier met de gamepad.
    fn maak_vrij(&mut self, tik: u64, ruimte: &Ruimte) {
        let hier = self.stand(tik, ruimte);
        self.vrij = Some(VrijeCamera::nieuw(hier));
    }
}

/// Waar het Voorbeeld zijn camera vandaan haalt.
#[derive(Clone, Debug, PartialEq)]
enum Voorbeeldbron {
    /// De Vrije camera staat aan, of de camera blijft staan bij de klaargezette Graphic
    /// ([`BLIJF`]): het Voorbeeld toont de camera van de Ether.
    Ether,
    /// Het Standpunt of de Camerarit van de klaargezette Graphic (of zonder Graphic dat van de
    /// Ether).
    Doel(Doel),
    /// Wat de Regie liet afspelen ("Probeer in Voorbeeld", of na een opname).
    Proef,
    /// Oefenen: de gamepad stuurt het Voorbeeld.
    Oefenen,
}

/// Het langste dat de server opneemt, in tikken.
const OPNAME_MAX: usize = (OPNAME_MAX_S * TIKKEN_PER_SECONDE as f32) as usize;

/// Rekent de camera's van de Ether en het Voorbeeld uit, tik na tik.
#[derive(Clone, Debug)]
pub struct Cameraregie {
    ruimte: Ruimte,
    ether: Spoor,
    voorbeeld: Spoor,
    /// De keuze waar het spoor van de Ether nu bij hoort.
    keuze: Camerakeuze,
    bron: Voorbeeldbron,
    /// Het volgnummer van het laatste verzoek om af te spelen dat het Voorbeeld begon.
    laatste_proef: Option<u64>,
    /// De opname die loopt: wanneer hij begon, en de standen van het Voorbeeld tot nu.
    opname: Option<(u64, Vec<[f32; 6]>)>,
}

impl Cameraregie {
    /// Begint met de camera van de Ether op het gekozen Standpunt, zonder overgang.
    pub fn nieuw(stand: &Stand, tik: u64, ruimte: Ruimte) -> Self {
        let mut ether = Spoor {
            beweging: Beweging::knip(tik, standpunt_of_a(stand, "A").stand, NaAankomst::Stil),
            vrij: None,
            snede: 0,
        };
        let doel = Doel::of_a(stand, &stand.camera.standpunt);
        let van = ether.stand(tik, &ruimte);
        ether.ga_naar(tik, van, &doel, true, &ruimte);
        ether.snede = 0;
        if stand.camera.vrij {
            ether.maak_vrij(tik, &ruimte);
        }
        let mut regie = Cameraregie {
            ruimte,
            voorbeeld: ether.clone(),
            ether,
            keuze: stand.camera.clone(),
            bron: Voorbeeldbron::Ether,
            // wat de Regie voor de herstart liet afspelen, speelt niet opnieuw
            laatste_proef: stand.camera.proef.as_ref().map(|p| p.sinds),
            opname: None,
        };
        regie.volg_voorbeeld(stand, tik);
        regie
    }

    /// Neemt een nieuwe Stand over vanaf deze tik.
    pub fn volg(&mut self, stand: &Stand, tik: u64) {
        let (oud, nieuw) = (&self.keuze, &stand.camera);
        let ander_standpunt =
            (&oud.standpunt, oud.knip, oud.sinds) != (&nieuw.standpunt, nieuw.knip, nieuw.sinds);
        let vrij_anders = oud.vrij != nieuw.vrij;
        self.keuze = nieuw.clone();
        if ander_standpunt && let Some(doel) = Doel::van(stand, &stand.camera.standpunt) {
            let van = self.ether.stand(tik, &self.ruimte);
            self.ether.ga_naar(tik, van, &doel, stand.camera.knip, &self.ruimte);
        }
        if vrij_anders || ander_standpunt {
            if stand.camera.vrij {
                let van = self.ether.stand(tik, &self.ruimte);
                self.ether
                    .vrij
                    .get_or_insert_with(|| VrijeCamera::nieuw(van))
                    .stuurbaar = true;
            } else if let Some(v) = &mut self.ether.vrij {
                v.stuurbaar = false;
            }
        }
        self.volg_voorbeeld(stand, tik);
    }

    /// Het Voorbeeld toont de klaargezette Graphic vanaf haar eigen Standpunt of Camerarit;
    /// zonder klaargezette Graphic dat van de Ether. Het knipt er meteen heen. Met de Vrije
    /// camera aan toont het die camera, tijdens Oefenen wat de gamepad doet, en speelt de Regie
    /// iets af, dan dat (vanaf waar de Ether staat).
    fn volg_voorbeeld(&mut self, stand: &Stand, tik: u64) {
        let c = &stand.camera;
        match (&c.opname, &self.opname) {
            (Some(o), Some((sinds, _))) if *sinds == o.sinds => {}
            (Some(o), _) => self.opname = Some((o.sinds, Vec::new())),
            (None, _) => self.opname = None,
        }
        if c.vrij {
            if self.bron != Voorbeeldbron::Ether {
                self.voorbeeld.snede = self.voorbeeld.snede.wrapping_add(1);
                self.bron = Voorbeeldbron::Ether;
            }
            return;
        }
        if self.bron == Voorbeeldbron::Ether {
            // vanaf hier rekent het Voorbeeld zelf, vanaf waar de Ether nu is
            self.voorbeeld.beweging = self.ether.beweging.clone();
            self.voorbeeld.vrij = self.ether.vrij.clone();
        }
        if let Some(p) = c.proef.as_ref().filter(|p| self.laatste_proef != Some(p.sinds)) {
            self.laatste_proef = Some(p.sinds);
            if let Some(doel) = Doel::van(stand, &p.letter) {
                let van = self.ether.stand(tik, &self.ruimte);
                // het Voorbeeld springt naar waar de Ether staat
                self.voorbeeld.snede = self.voorbeeld.snede.wrapping_add(1);
                self.voorbeeld.ga_naar(tik, van, &doel, false, &self.ruimte);
                self.bron = Voorbeeldbron::Proef;
            }
        }
        let speelt_af = self.bron == Voorbeeldbron::Proef && c.proef.is_some();
        if c.opname.is_some() || (c.oefenen && !speelt_af) {
            if self.bron != Voorbeeldbron::Oefenen {
                self.voorbeeld.maak_vrij(tik, &self.ruimte);
                self.bron = Voorbeeldbron::Oefenen;
            }
            return;
        }
        if speelt_af {
            return;
        }
        // blijft de camera staan bij de klaargezette Graphic, dan toont het Voorbeeld de Ether
        if stand.klaar.as_ref().is_some_and(|k| k.standpunt() == BLIJF) {
            if self.bron != Voorbeeldbron::Ether {
                self.voorbeeld.snede = self.voorbeeld.snede.wrapping_add(1);
                self.bron = Voorbeeldbron::Ether;
            }
            return;
        }
        let letter = stand
            .klaar
            .as_ref()
            .map_or(std::borrow::Cow::Borrowed(c.standpunt.as_str()), |k| k.standpunt());
        let doel = Doel::of_a(stand, &letter);
        if self.bron != Voorbeeldbron::Doel(doel.clone()) {
            let van = self.voorbeeld.stand(tik, &self.ruimte);
            self.voorbeeld.ga_naar(tik, van, &doel, true, &self.ruimte);
            self.bron = Voorbeeldbron::Doel(doel);
        }
    }

    /// Rekent de Vrije camera (of tijdens Oefenen de camera van het Voorbeeld) één tik verder
    /// met wat de gamepad nu doet, en neemt op. Roep dit elke tik aan, vóór
    /// [`Cameraregie::beeld`] van dezelfde tik.
    pub fn stap(&mut self, tik: u64, stuur: &Stuur) {
        if let Some(v) = &mut self.ether.vrij {
            v.stap(stuur, &self.ruimte);
        }
        if self.keuze.oefenen {
            // speelt het Voorbeeld iets af, dan neemt de stick het over vanaf waar het is
            if self.bron == Voorbeeldbron::Proef && stuur.stuurt() {
                self.voorbeeld.maak_vrij(tik, &self.ruimte);
                self.bron = Voorbeeldbron::Oefenen;
            }
            if let Some(v) = &mut self.voorbeeld.vrij {
                v.stap(stuur, &self.ruimte);
            }
        }
        if let Some((_, punten)) = &mut self.opname
            && punten.len() < OPNAME_MAX
        {
            punten.push(als_opnamepunt(&self.voorbeeld.stand(tik, &self.ruimte)));
        }
    }

    /// De camera's op deze tik.
    pub fn beeld(&self, tik: u64) -> Camerabeeld {
        let ether = self.ether.punt(tik, &self.ruimte);
        let (voorbeeld, voorbeeld_rit) = match self.bron {
            Voorbeeldbron::Ether => (
                Camerapunt {
                    stand: ether.stand,
                    snede: self.voorbeeld.snede,
                },
                None,
            ),
            _ => (
                self.voorbeeld.punt(tik, &self.ruimte),
                self.voorbeeld.vrij.is_none().then(|| self.voorbeeld.beweging.opname_s(tik)).flatten(),
            ),
        };
        Camerabeeld {
            tik,
            ether,
            voorbeeld,
            opname: self.opname.as_ref().map(|(_, p)| p.len() as u32),
            voorbeeld_rit,
        }
    }

    pub fn ether(&self) -> &Spoor {
        &self.ether
    }

    /// Of de gamepad nu het Voorbeeld stuurt: dan is dat "de camera van nu".
    pub fn oefent(&self) -> bool {
        self.keuze.oefenen
    }

    /// Wat er tot nu toe is opgenomen, of niets als er geen opname loopt.
    pub fn opname(&self) -> Option<&[[f32; 6]]> {
        self.opname.as_ref().map(|(_, p)| p.as_slice())
    }
}

/// Het Standpunt met deze letter, of anders A (dat is er altijd, tenzij de Stand stuk is).
fn standpunt_of_a(stand: &Stand, letter: &str) -> Standpunt {
    stand
        .standpunt(letter)
        .or_else(|| stand.standpunt("A"))
        .cloned()
        .unwrap_or_else(|| Standpunt::vaste().swap_remove(0))
}

/// Het verschil tussen twee hoeken, tussen -pi en pi.
fn kortste_hoek(d: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    (d + PI).rem_euclid(TAU) - PI
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f32; 3], f: f32) -> [f32; 3] {
    [a[0] * f, a[1] * f, a[2] * f]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 1e-6 {
        scale(a, 1.0 / l)
    } else {
        [0.0; 3]
    }
}
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    add(a, scale(sub(b, a), t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gedeeld::zaalkaart::{HANGLAMPEN, STOELPLEKKEN};
    use gedeeld::{Graphic, Herkomst, Opdracht};

    const T: u64 = TIKKEN_PER_SECONDE as u64;

    fn ruimte() -> Ruimte {
        Ruimte::zaal()
    }

    /// Voert een Opdracht uit met het volgende volgnummer.
    fn doe(stand: &mut Stand, volgnummer: &mut u64, o: Opdracht) {
        *volgnummer += 1;
        let h = Herkomst {
            door: "Proef",
            volgnummer: *volgnummer,
            tijd_ms: 0,
        };
        stand.voer_uit(&o, &h).unwrap();
    }

    fn kies(letter: &str, knip: bool) -> Opdracht {
        Opdracht::KiesStandpunt {
            letter: letter.into(),
            knip,
        }
    }

    fn standpunt(stand: &Stand, letter: &str) -> Camerastand {
        stand.standpunt(letter).unwrap().stand
    }

    fn ongeveer(a: Camerastand, b: Camerastand) -> bool {
        let d = (0..3)
            .map(|i| (a.positie[i] - b.positie[i]).abs())
            .fold(0.0, f32::max);
        d < 1e-4
            && kortste_hoek(a.draai - b.draai).abs() < 1e-4
            && (a.helling - b.helling).abs() < 1e-4
            && (a.beeldhoek - b.beeldhoek).abs() < 1e-4
    }

    /// Ligt de camera ergens onder of naast een hanglamp, dan blijft hij er minstens 40 cm onder.
    fn onder_de_lampen(p: [f32; 3]) -> bool {
        HANGLAMPEN.iter().all(|l| {
            let naast = ((p[0] - l.x).powi(2) + (p[2] - l.z).powi(2)).sqrt() > l.straal + 0.5;
            naast || p[1] < l.onderkant - 0.4
        })
    }

    #[test]
    fn het_plafond_van_de_ruimte_ligt_onder_alle_hanglampen() {
        assert_eq!(HANGLAMPEN.len(), 18);
        for l in HANGLAMPEN {
            assert!(ruimte().plafond <= l.onderkant - 0.4, "{l:?}");
        }
    }

    #[test]
    fn de_vaste_standpunten_staan_in_de_ruimte() {
        for s in Standpunt::vaste() {
            assert!(
                ruimte().bevat(s.stand.positie, 0.0),
                "{} {:?}",
                s.letter,
                s.stand.positie
            );
        }
    }

    #[test]
    fn klem_houdt_de_camera_binnen_wanden_vloer_en_plafond() {
        for p in [
            [0.0, 9.0, -5.0],
            [20.0, 3.0, -2.0],
            [11.5, 3.0, 2.0],
            [0.0, 0.2, -11.0],
            [0.0, 3.0, 9.0],
        ] {
            let k = ruimte().klem(p);
            assert!(ruimte().bevat(k, -1e-4), "{p:?} -> {k:?}");
        }
        let binnen = [1.0, 3.0, -4.0];
        assert_eq!(ruimte().klem(binnen), binnen);
    }

    #[test]
    fn elke_route_tussen_vaste_standpunten_blijft_in_de_ruimte_en_onder_de_lampen() {
        let vaste = Standpunt::vaste();
        for a in &vaste {
            for b in &vaste {
                let route = Route::boog(a.stand.positie, b.stand.positie, &ruimte());
                for i in 0..=200 {
                    let p = ruimte().klem(route.punt(i as f32 / 200.0));
                    assert!(
                        ruimte().bevat(p, -1e-4),
                        "{} naar {}: {p:?}",
                        a.letter,
                        b.letter
                    );
                    assert!(
                        onder_de_lampen(p),
                        "{} naar {}: {p:?} tegen een lamp",
                        a.letter,
                        b.letter
                    );
                }
            }
        }
    }

    #[test]
    fn het_shot_van_elke_stoel_blijft_in_de_ruimte_met_de_stoel_links_van_de_kaart() {
        use gedeeld::Stoel;
        use gedeeld::camera::shot_op_stoel;
        // de Persoonskaart begint op x 1166 van 1920: rechts daarvan zit de Stoel achter de kaart
        let kaart = 1166.0 / 960.0 - 1.0;
        let a = Standpunt::vaste()[0].stand.positie;
        for (i, stoel) in Stoel::alle().enumerate() {
            let s = shot_op_stoel(stoel).expect("elke Stoel heeft een shot");
            assert!(ruimte().bevat(s.positie, 0.05), "Stoel {stoel:?}: {:?}", s.positie);
            assert!(onder_de_lampen(s.positie), "Stoel {stoel:?} tegen een lamp");
            let [x, y] = s.in_beeld(STOELPLEKKEN[i]).expect("de Stoel ligt voor de camera");
            assert!((-0.85..kaart - 0.15).contains(&x), "Stoel {stoel:?} niet links in beeld: x {x}");
            assert!((-0.6..0.6).contains(&y), "Stoel {stoel:?} te hoog of te laag in beeld: y {y}");
            // schuin van voren: de camera staat aan de kant van het midden van de Zaal, waar het
            // Kamerlid naartoe kijkt
            let p = STOELPLEKKEN[i];
            let voor = (s.positie[0] - p[0]) * -p[0] + (s.positie[2] - p[2]) * -p[2];
            assert!(voor > 0.0, "Stoel {stoel:?}: camera achter de Stoel");
            // de weg ernaartoe vanaf Totaal vanaf voorzitter blijft in de ruimte
            let route = Route::boog(a, s.positie, &ruimte());
            for t in 0..=50 {
                let p = ruimte().klem(route.punt(t as f32 / 50.0));
                assert!(onder_de_lampen(p), "Stoel {stoel:?}: route tegen een lamp bij {p:?}");
            }
        }
        assert!(shot_op_stoel(Stoel { vak: 7, rij: 1, nummer: 1 }).is_none());
    }

    #[test]
    fn een_persoonskaart_beweegt_de_camera_in_twee_seconden_naar_het_shot_van_de_stoel() {
        use gedeeld::Stoel;
        use gedeeld::camera::shot_op_stoel;
        use gedeeld::kamer::{Foto, Fractie, Kamerlid};
        use gedeeld::persoonskaart::{Extra, Wie};
        use gedeeld::zaalindeling::{Bezetting, Zaalindeling};
        let mut stand = Stand::default();
        let mut v = 0;
        // VOORBEELDGEGEVENS: een verzonnen Kamerlid op Stoel 4-5-2
        let lid = Kamerlid {
            id: "proef".into(),
            roepnaam: "Proef".into(),
            initialen: "P.".into(),
            tussenvoegsel: None,
            achternaam: "Persoon".into(),
            fractie: "f".into(),
            zetel: "z".into(),
            fractievoorzitter: false,
            van: "2025-11-12".into(),
            geboortedatum: None,
            geboorteplaats: None,
            woonplaats: None,
            foto: Foto::Geen,
            in_de_kamer_sinds: None,
            kieslijst: None,
        };
        let fractie = Fractie { id: "f".into(), afkorting: "F".into(), naam: "F".into(), zetels: 1 };
        doe(&mut stand, &mut v, Opdracht::LeesKamer {
            opgehaald: "nu".into(),
            peildatum: "2026-11-22".into(),
            kamerleden: vec![lid],
            fracties: vec![fractie],
        });
        let stoel = Stoel { vak: 4, rij: 5, nummer: 2 };
        let zaalindeling = Zaalindeling {
            kamerperiode: "2025".into(),
            bron: String::new(),
            stoelen: vec![Bezetting { vak: 4, rij: 5, nummer: 2, kamerlid: Some("proef".into()), naam: String::new() }],
        };
        doe(&mut stand, &mut v, Opdracht::LeesZaalindeling { bestand: "proef".into(), zaalindeling });
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        let graphic = Graphic::Persoonskaart {
            wie: Wie::Kamerlid { id: "proef".into() },
            regel: String::new(),
            extra: Extra::default(),
            gegevens: None,
        };
        doe(&mut stand, &mut v, Opdracht::ZetKlaar { graphic, standpunt: None });
        regie.volg(&stand, 10);
        let shot = shot_op_stoel(stoel).unwrap();
        // het Voorbeeld knipt naar het shot, de Ether blijft tot Toon
        assert!(ongeveer(regie.beeld(10).voorbeeld.stand, shot));
        doe(&mut stand, &mut v, Opdracht::Toon { versie: None });
        regie.volg(&stand, 20);
        let halverwege = regie.beeld(20 + T).ether.stand;
        assert!(!ongeveer(halverwege, shot) && !ongeveer(halverwege, standpunt(&stand, "A")));
        assert!(ongeveer(regie.beeld(20 + 2 * T).ether.stand, shot));
        assert!(ongeveer(regie.beeld(20 + 30 * T).ether.stand, shot), "daarna staat hij stil");
    }

    #[test]
    fn van_de_studio_glijdt_de_camera_zonder_sprong_de_zaal_in() {
        use gedeeld::Stoel;
        let a = Standpunt::vaste()[0].stand.positie;
        assert!(a[1] > 5.2 && a[2] > ruimte().voor_diepte, "A staat in de studio: {a:?}");
        let mut doelen: Vec<[f32; 3]> = Standpunt::vaste().iter().map(|s| s.stand.positie).collect();
        doelen.extend(Stoel::alle().filter_map(|s| shot_op_stoel(s).map(|s| s.positie)));
        // Totaal boven het paneel staat ook in de studio
        let paneel = gedeeld::camera::boven_het_paneel().positie;
        assert!(ruimte().bevat(paneel, 0.0), "Totaal boven het paneel in de ruimte: {paneel:?}");
        doelen.push(paneel);
        for doel in doelen {
            for (van, naar) in [(a, doel), (doel, a)] {
                let route = Route::boog(van, naar, &ruimte());
                let n = 400;
                let punten: Vec<[f32; 3]> = (0..=n).map(|i| ruimte().klem(route.punt(i as f32 / n as f32))).collect();
                // stappen van een vierhonderdste van de route, nooit een sprong
                let stap = len(sub(naar, van)) / n as f32;
                for w in punten.windows(2) {
                    let d = len(sub(w[1], w[0]));
                    assert!(d < 4.0 * stap + 0.01, "{van:?} naar {naar:?}: sprong van {d} m bij {:?}", w[0]);
                }
                assert!(punten.iter().all(|p| onder_de_lampen(*p)), "{van:?} naar {naar:?} tegen een lamp");
            }
        }
    }

    #[test]
    fn de_route_van_a_naar_g_is_een_boog() {
        let vaste = Standpunt::vaste();
        let (a, g) = (vaste[0].stand.positie, vaste[4].stand.positie);
        let route = Route::boog(a, g, &ruimte());
        let midden = route.punt(0.5);
        let recht = lerp3(a, g, 0.5);
        assert!(
            len(sub(midden, recht)) > 0.3,
            "geen boog: {midden:?} tegen {recht:?}"
        );
    }

    #[test]
    fn kies_standpunt_beweegt_in_twee_seconden_met_zachte_start_en_stop() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        let a = standpunt(&stand, "A");
        assert!(ongeveer(regie.beeld(0).ether.stand, a));

        doe(&mut stand, &mut v, kies("G", false));
        regie.volg(&stand, 100);
        let g = standpunt(&stand, "G");
        let snede = regie.beeld(100).ether.snede;
        assert!(
            ongeveer(regie.beeld(100).ether.stand, a),
            "begint waar hij was"
        );
        // zacht starten: na 1 tik nog bijna niets bewogen
        let p1 = regie.beeld(101).ether.stand.positie;
        assert!(len(sub(p1, a.positie)) < 0.01);
        let halverwege = regie.beeld(100 + T).ether.stand;
        assert!(!ongeveer(halverwege, a) && !ongeveer(halverwege, g));
        assert!(
            ongeveer(regie.beeld(100 + 2 * T).ether.stand, g),
            "na 2 s aangekomen"
        );
        assert!(
            ongeveer(regie.beeld(100 + 10 * T).ether.stand, g),
            "daarna stil"
        );
        assert_eq!(
            regie.beeld(100 + 2 * T).ether.snede,
            snede,
            "bewegen is geen knip"
        );
    }

    #[test]
    fn shift_met_de_letter_knipt_meteen() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        let snede = regie.beeld(0).ether.snede;
        doe(&mut stand, &mut v, kies("D", true));
        regie.volg(&stand, 50);
        assert!(ongeveer(
            regie.beeld(50).ether.stand,
            standpunt(&stand, "D")
        ));
        assert_eq!(regie.beeld(50).ether.snede, snede + 1);
    }

    #[test]
    fn een_standpunt_dat_knippen_heeft_knipt_ook_zonder_shift() {
        let mut stand = Stand::default();
        stand.standpunten[5].overgang.soort = OvergangSoort::Knippen;
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, kies("H", false));
        regie.volg(&stand, 10);
        assert!(ongeveer(
            regie.beeld(10).ether.stand,
            standpunt(&stand, "H")
        ));
    }

    #[test]
    fn de_duur_van_het_standpunt_telt() {
        let mut stand = Stand::default();
        stand.standpunten[1].overgang.duur_s = 0.5;
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, kies("S", false));
        regie.volg(&stand, 0);
        assert!(!ongeveer(
            regie.beeld(T / 2 - 1).ether.stand,
            standpunt(&stand, "S")
        ));
        assert!(ongeveer(
            regie.beeld(T / 2).ether.stand,
            standpunt(&stand, "S")
        ));
    }

    #[test]
    fn langzaam_schuiven_stopt_na_twintig_seconden() {
        let mut stand = Stand::default();
        stand.standpunten[4].na_aankomst = NaAankomst::Schuiven {
            richting: [1.0, 0.0, 0.0],
            cm_per_s: 4.0,
        };
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, kies("G", true));
        regie.volg(&stand, 0);
        let x = |tik: u64| regie.beeld(tik).ether.stand.positie[0];
        let begin = x(0);
        // ongeveer 4 cm per seconde, in één richting
        let per_s = x(11 * T) - x(10 * T);
        assert!((per_s - 0.04).abs() < 1e-3, "{per_s}");
        assert!(x(5 * T) > begin && x(15 * T) > x(5 * T));
        // na 20 s stil: 19 s op volle snelheid
        assert!(
            (x(20 * T) - begin - 0.76).abs() < 1e-3,
            "{}",
            x(20 * T) - begin
        );
        assert_eq!(x(20 * T), x(60 * T));
    }

    #[test]
    fn bij_blijf_volgt_het_voorbeeld_de_ether_en_toon_beweegt_niets() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        // de Ether rijdt naar D terwijl de Graphic klaarstaat
        doe(&mut stand, &mut v, kies("D", false));
        regie.volg(&stand, 0);
        let graphic = Graphic::Titelkaart {
            kop: "Voorbeeldgegevens".into(),
            regel: String::new(),
        };
        doe(
            &mut stand,
            &mut v,
            Opdracht::ZetKlaar {
                graphic,
                standpunt: Some(BLIJF.into()),
            },
        );
        regie.volg(&stand, T / 2);
        for tik in [T / 2, T, 3 * T] {
            let beeld = regie.beeld(tik);
            assert!(ongeveer(beeld.voorbeeld.stand, beeld.ether.stand), "tik {tik}");
        }
        assert!(!ongeveer(regie.beeld(T / 2).ether.stand, standpunt(&stand, "D")));

        doe(&mut stand, &mut v, Opdracht::Toon { versie: None });
        regie.volg(&stand, T);
        assert_eq!(stand.camera.standpunt, "D");
        // de rit naar D loopt gewoon door, zonder snede
        let snede = regie.beeld(T).ether.snede;
        assert!(ongeveer(regie.beeld(3 * T).ether.stand, standpunt(&stand, "D")));
        assert_eq!(regie.beeld(3 * T).ether.snede, snede);
    }

    #[test]
    fn een_nieuwe_graphic_tonen_beweegt_de_camera_naar_haar_standpunt() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        let graphic = Graphic::Titelkaart {
            kop: "Voorbeeldgegevens".into(),
            regel: String::new(),
        };
        doe(
            &mut stand,
            &mut v,
            Opdracht::ZetKlaar {
                graphic,
                standpunt: Some("F".into()),
            },
        );
        regie.volg(&stand, 10);
        // het Voorbeeld knipt naar het Standpunt van de klaargezette Graphic, de Ether blijft
        assert!(ongeveer(
            regie.beeld(10).voorbeeld.stand,
            standpunt(&stand, "F")
        ));
        assert!(ongeveer(
            regie.beeld(10).ether.stand,
            standpunt(&stand, "A")
        ));

        doe(&mut stand, &mut v, Opdracht::Toon { versie: None });
        regie.volg(&stand, 20);
        assert_eq!(stand.camera.standpunt, "F");
        assert!(ongeveer(
            regie.beeld(20 + 2 * T).ether.stand,
            standpunt(&stand, "F")
        ));
    }

    #[test]
    fn bijwerken_met_een_ander_standpunt_beweegt_de_camera_mee() {
        let mut stand = Stand::default();
        let mut v = 0;
        let graphic = Graphic::Titelkaart {
            kop: "Voorbeeldgegevens".into(),
            regel: String::new(),
        };
        doe(
            &mut stand,
            &mut v,
            Opdracht::ZetKlaar {
                graphic: graphic.clone(),
                standpunt: Some("S".into()),
            },
        );
        doe(&mut stand, &mut v, Opdracht::Toon { versie: None });
        let id = stand.klaar.as_ref().unwrap().id;
        // dezelfde tekst, ander Standpunt: Bijwerken stuurt de camera naar H
        doe(
            &mut stand,
            &mut v,
            Opdracht::WijzigKlaar {
                id,
                graphic: graphic.clone(),
                standpunt: Some("H".into()),
            },
        );
        assert_eq!(stand.camera.standpunt, "S", "de Ether wacht op Bijwerken");
        doe(&mut stand, &mut v, Opdracht::Bijwerken);
        assert_eq!(stand.camera.standpunt, "H");
        // Bijwerken zonder ander Standpunt laat de camera waar de Regie hem zette
        doe(&mut stand, &mut v, kies("G", false));
        let kop = Graphic::Titelkaart {
            kop: "Andere kop".into(),
            regel: String::new(),
        };
        doe(
            &mut stand,
            &mut v,
            Opdracht::WijzigKlaar {
                id,
                graphic: kop,
                standpunt: Some("H".into()),
            },
        );
        doe(&mut stand, &mut v, Opdracht::Bijwerken);
        assert_eq!(stand.camera.standpunt, "G");
    }

    #[test]
    fn zonder_eigen_keuze_gebruikt_een_graphic_het_standpunt_van_het_sjabloon() {
        let mut stand = Stand::default();
        let mut v = 0;
        doe(&mut stand, &mut v, kies("G", true));
        let graphic = Graphic::Titelkaart {
            kop: "Voorbeeldgegevens".into(),
            regel: String::new(),
        };
        doe(
            &mut stand,
            &mut v,
            Opdracht::ZetKlaar {
                graphic,
                standpunt: None,
            },
        );
        doe(&mut stand, &mut v, Opdracht::Toon { versie: None });
        assert_eq!(stand.camera.standpunt, "A");
        assert!(!stand.camera.knip);
    }

    #[test]
    fn een_onbekend_standpunt_wordt_geweigerd() {
        let mut stand = Stand::default();
        let h = Herkomst {
            door: "Proef",
            volgnummer: 1,
            tijd_ms: 0,
        };
        assert!(stand.voer_uit(&kies("P", false), &h).is_err());
        let graphic = Graphic::Titelkaart {
            kop: "Voorbeeldgegevens".into(),
            regel: String::new(),
        };
        assert!(
            stand
                .voer_uit(
                    &Opdracht::ZetKlaar {
                        graphic,
                        standpunt: Some("P".into())
                    },
                    &h
                )
                .is_err()
        );
        assert_eq!(stand.camera, Camerakeuze::default());
    }

    #[test]
    fn een_nieuwe_keuze_halverwege_begint_waar_de_camera_is() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, kies("G", false));
        regie.volg(&stand, 0);
        let onderweg = regie.beeld(T).ether.stand;
        doe(&mut stand, &mut v, kies("D", false));
        regie.volg(&stand, T);
        assert!(ongeveer(regie.beeld(T).ether.stand, onderweg));
        assert!(ongeveer(
            regie.beeld(3 * T).ether.stand,
            standpunt(&stand, "D")
        ));
    }

    #[test]
    fn de_vloer_van_de_ruimte_ligt_boven_alle_stoelen() {
        let r = ruimte();
        for (i, p) in STOELPLEKKEN.iter().enumerate() {
            // de plek is 15 cm onder de bovenrand van de rugleuning
            let rugleuning = p[1] + 0.15;
            assert!(r.vloer_op(p[0], p[2]) > rugleuning, "Stoel {i} {p:?}");
        }
    }

    /// Zet de Vrije camera aan en geeft de regie terug, met de Stand en het volgnummer.
    fn vrij_vanaf(letter: &str) -> (Stand, u64, Cameraregie) {
        let mut stand = Stand::default();
        let mut v = 0;
        doe(&mut stand, &mut v, kies(letter, true));
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: true });
        regie.volg(&stand, 0);
        (stand, v, regie)
    }

    /// Stuurt `tikken` tikken lang met deze stick, vanaf `tik`; geeft de standen terug.
    fn stuur(regie: &mut Cameraregie, tik: &mut u64, tikken: u64, s: Stuur) -> Vec<Camerastand> {
        (0..tikken)
            .map(|_| {
                *tik += 1;
                regie.stap(*tik, &s);
                regie.beeld(*tik).ether.stand
            })
            .collect()
    }

    const VOORUIT: Stuur = Stuur {
        links: [0.0, 1.0],
        ..Stuur::LOS
    };

    #[test]
    fn met_de_vrije_camera_uit_doet_de_stick_niets() {
        let stand = Stand::default();
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        let mut tik = 0;
        let standen = stuur(&mut regie, &mut tik, 2 * T, VOORUIT);
        assert!(standen.iter().all(|s| ongeveer(*s, standpunt(&stand, "A"))));
    }

    #[test]
    fn de_stick_beweegt_de_vrije_camera_en_na_loslaten_glijdt_hij_in_een_halve_seconde_stil() {
        let (stand, _, mut regie) = vrij_vanaf("G");
        let g = standpunt(&stand, "G");
        let mut tik = 0;
        let rijden = stuur(&mut regie, &mut tik, T, VOORUIT);
        // vooruit is waar de camera heen kijkt, horizontaal: G kijkt langs -z
        let eind = rijden.last().unwrap().positie;
        assert!(eind[2] < g.positie[2] - 0.2, "{eind:?}");
        assert!(
            (eind[1] - g.positie[1]).abs() < 1e-4,
            "blijft op dezelfde hoogte"
        );

        let los = stuur(&mut regie, &mut tik, T, Stuur::LOS);
        let stap = |i: usize| len(sub(los[i + 1].positie, los[i].positie));
        assert!(stap(0) > 0.001, "glijdt nog na het loslaten");
        assert!(stap(40) > 0.0, "na 0,4 s nog niet stil");
        assert!(stap(0) > stap(20) && stap(20) > stap(40), "remt af");
        let stil = los[T as usize / 2].positie;
        assert!(
            los[T as usize / 2..].iter().all(|s| s.positie == stil),
            "na 0,5 s stil"
        );
        assert!(len(sub(stil, los[0].positie)) > 0.05, "geen harde stop");
    }

    #[test]
    fn snel_rijdt_verder_dan_langzaam() {
        let afstand = |snel: bool| {
            let (_, _, mut regie) = vrij_vanaf("G");
            let mut tik = 0;
            let begin = regie.beeld(0).ether.stand.positie;
            let s = stuur(&mut regie, &mut tik, T, Stuur { snel, ..VOORUIT });
            len(sub(s.last().unwrap().positie, begin))
        };
        assert!(afstand(true) > 2.0 * afstand(false));
    }

    /// Een vaste reeks wilde stickbewegingen, als van een Regie die alle kanten op stuurt.
    fn wild(i: u64) -> Stuur {
        let mut x = i / 37 * 2654435761 % 4294967296;
        let mut r = || {
            x = (x * 1103515245 + 12345) % 2147483648;
            x as f32 / 1073741824.0 - 1.0
        };
        Stuur {
            links: [r(), r()],
            rechts: [r(), r()],
            omhoog: r().max(0.0),
            omlaag: r().max(0.0),
            inzoomen: r() > 0.5,
            uitzoomen: r() > 0.5,
            snel: r() > 0.0,
        }
    }

    #[test]
    fn de_vrije_camera_kantelt_nooit_opzij() {
        let (_, _, mut regie) = vrij_vanaf("A");
        let mut tik = 0;
        let mut standen = Vec::new();
        // eerst lang de rechter stick omhoog en omlaag, dan wild alle kanten op
        for rechts in [[0.3, 1.0], [-0.2, -1.0]] {
            standen.extend(stuur(
                &mut regie,
                &mut tik,
                5 * T,
                Stuur {
                    rechts,
                    ..Stuur::LOS
                },
            ));
        }
        for i in 0..60 * T {
            standen.extend(stuur(&mut regie, &mut tik, 1, wild(i)));
        }
        let r = ruimte();
        for s in &standen {
            assert!(r.bevat(s.positie, -1e-4), "{s:?}");
            assert!(s.helling.abs() < 85f32.to_radians() + 1e-4, "{s:?}");
            assert!(
                (10f32.to_radians()..=100f32.to_radians()).contains(&s.beeldhoek),
                "{s:?}"
            );
            // de horizon blijft recht: twee punten links en rechts op ooghoogte, even ver voor
            // de camera, staan even hoog in beeld
            let (sd, cd) = s.draai.sin_cos();
            let voor = add(s.positie, [-sd * 5.0, 0.0, -cd * 5.0]);
            let [links, rechts] = [-1.0, 1.0].map(|k| {
                let p = add(voor, [cd * k, 0.0, -sd * k]);
                s.in_beeld(p).unwrap()
            });
            assert!(
                (links[1] - rechts[1]).abs() < 1e-4,
                "{s:?}: {links:?} {rechts:?}"
            );
            assert!(links[0] < rechts[0]);
        }
    }

    #[test]
    fn aan_de_rand_glijdt_de_camera_zacht_langs() {
        let r = ruimte();
        let (_, _, mut regie) = vrij_vanaf("D");
        let mut tik = 0;
        // schuin naar achteren tegen de wand van de halve cirkel, en omhoog tegen het plafond
        let tegen = Stuur {
            links: [-0.7, 0.7],
            rechts: [-0.3, 0.0],
            omhoog: 1.0,
            ..Stuur::LOS
        };
        let standen = stuur(&mut regie, &mut tik, 12 * T, tegen);
        let p: Vec<[f32; 3]> = standen.iter().map(|s| s.positie).collect();
        for w in p.windows(3) {
            assert!(r.bevat(w[2], -1e-4), "{:?}", w[2]);
            // geen harde stop: de snelheid verandert per tik hooguit zoveel als bij gewoon
            // optrekken en afremmen (1,2 m/s in een halve seconde, langzaam)
            let versnelling = len(sub(sub(w[2], w[1]), sub(w[1], w[0])));
            assert!(versnelling < 3.0 / (T * T) as f32, "{w:?}: {versnelling}");
        }
        let eind = *p.last().unwrap();
        // de zachte rand houdt hem er net van af
        assert!(!r.bevat(eind, 0.25), "staat aan de rand: {eind:?}");
        // en schuift nog langs de rand
        let laatste = len(sub(eind, p[p.len() - 1 - T as usize]));
        assert!(laatste > 0.1, "{laatste}");
    }

    #[test]
    fn de_vrije_camera_komt_niet_door_vloer_of_plafond() {
        let r = ruimte();
        for (omhoog, omlaag) in [(1.0, 0.0), (0.0, 1.0)] {
            let (_, _, mut regie) = vrij_vanaf("S");
            let mut tik = 0;
            let s = Stuur {
                omhoog,
                omlaag,
                snel: true,
                ..VOORUIT
            };
            for st in stuur(&mut regie, &mut tik, 8 * T, s) {
                assert!(r.bevat(st.positie, -1e-4), "{st:?}");
            }
        }
    }

    #[test]
    fn een_standpunt_kiezen_met_de_vrije_camera_aan_beweegt_vanaf_waar_hij_is() {
        let (mut stand, mut v, mut regie) = vrij_vanaf("G");
        let mut tik = 0;
        let hier = *stuur(&mut regie, &mut tik, T, VOORUIT).last().unwrap();
        doe(&mut stand, &mut v, kies("D", false));
        regie.volg(&stand, tik);
        assert!(!stand.camera.vrij);
        assert!(ongeveer(regie.beeld(tik).ether.stand, hier), "geen sprong");
        let daarna = stuur(&mut regie, &mut tik, 3 * T, VOORUIT);
        assert!(
            ongeveer(*daarna.last().unwrap(), standpunt(&stand, "D")),
            "de stick doet niets meer"
        );
    }

    #[test]
    fn vrije_camera_uit_laat_de_camera_uitglijden_en_staan() {
        let (mut stand, mut v, mut regie) = vrij_vanaf("G");
        let mut tik = 0;
        stuur(&mut regie, &mut tik, T, VOORUIT);
        doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: false });
        regie.volg(&stand, tik);
        let daarna = stuur(&mut regie, &mut tik, 3 * T, VOORUIT);
        let stil = daarna[T as usize / 2];
        assert!(daarna[T as usize / 2..].iter().all(|s| *s == stil));
        assert!(
            len(sub(stil.positie, daarna[0].positie)) > 0.01,
            "glijdt uit"
        );
    }

    #[test]
    fn het_voorbeeld_toont_de_vrije_camera() {
        let (_, _, mut regie) = vrij_vanaf("G");
        let mut tik = 0;
        stuur(&mut regie, &mut tik, T, VOORUIT);
        let b = regie.beeld(tik);
        assert_eq!(b.voorbeeld.stand, b.ether.stand);
    }

    // Oefenen en de Camerarit (#45)

    fn oefen(stand: &mut Stand, v: &mut u64, regie: &mut Cameraregie, tik: u64, aan: bool) {
        doe(stand, v, Opdracht::ZetOefenen { aan });
        regie.volg(stand, tik);
    }

    /// Stuurt en geeft per tik de camera's van Ether en Voorbeeld.
    fn stuur_beide(
        regie: &mut Cameraregie,
        tik: &mut u64,
        tikken: u64,
        s: Stuur,
    ) -> Vec<(Camerastand, Camerastand)> {
        (0..tikken)
            .map(|_| {
                *tik += 1;
                regie.stap(*tik, &s);
                let b = regie.beeld(*tik);
                (b.ether.stand, b.voorbeeld.stand)
            })
            .collect()
    }

    #[test]
    fn tijdens_oefenen_verandert_de_camera_van_de_ether_niet() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        let a = standpunt(&stand, "A");
        oefen(&mut stand, &mut v, &mut regie, 0, true);
        let mut tik = 0;
        let standen = stuur_beide(&mut regie, &mut tik, 3 * T, Stuur { snel: true, ..VOORUIT });
        assert!(standen.iter().all(|(e, _)| *e == a), "de Ether blijft op A");
        let (_, voorbeeld) = standen.last().unwrap();
        assert!(len(sub(voorbeeld.positie, a.positie)) > 1.0, "de gamepad stuurt het Voorbeeld");

        // een Standpunt kiezen voor de Ether werkt nog, en de Ether beweegt precies zoals zonder
        // gamepad
        doe(&mut stand, &mut v, kies("G", false));
        regie.volg(&stand, tik);
        let mut zonder = regie.clone();
        let mut tik2 = tik;
        let met = stuur_beide(&mut regie, &mut tik, 3 * T, wild(7));
        let los = stuur_beide(&mut zonder, &mut tik2, 3 * T, Stuur::LOS);
        assert!(met.iter().zip(&los).all(|(a, b)| a.0 == b.0), "de stick raakt de Ether niet");
        assert!(ongeveer(met.last().unwrap().0, standpunt(&stand, "G")));
        assert!(stand.camera.oefenen);
    }

    #[test]
    fn oefenen_uit_zet_het_voorbeeld_terug_op_het_standpunt() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        oefen(&mut stand, &mut v, &mut regie, 0, true);
        let mut tik = 0;
        stuur_beide(&mut regie, &mut tik, T, VOORUIT);
        oefen(&mut stand, &mut v, &mut regie, tik, false);
        assert!(ongeveer(regie.beeld(tik).voorbeeld.stand, standpunt(&stand, "A")));
        let na = stuur_beide(&mut regie, &mut tik, T, VOORUIT);
        assert!(na.iter().all(|(_, vb)| ongeveer(*vb, standpunt(&stand, "A"))), "de stick doet niets meer");
    }

    /// Een rit van vier seconden langs x, vanaf (-2, 3, -4), en zijn sleutel.
    fn met_rit(stand: &mut Stand, v: &mut u64) -> String {
        let opname: Vec<[f32; 6]> = (0..=400)
            .map(|i| [-2.0 + i as f32 / 100.0, 3.0, -4.0, 0.0, -0.3, 0.9])
            .collect();
        doe(stand, v, Opdracht::ZetOefenen { aan: true });
        doe(stand, v, Opdracht::ZetOpnemen { aan: true, voor: None, opname: None });
        doe(stand, v, Opdracht::ZetOpnemen { aan: false, voor: None, opname: Some(opname) });
        doe(stand, v, Opdracht::ZetOefenen { aan: false });
        let r = stand.cameraritten.last().unwrap().letter.clone();
        let mut b = stand.camerarit(&r).unwrap().bijschaving;
        b.gladheid = 0.0;
        doe(stand, v, Opdracht::BijschaafCamerarit { letter: r.clone(), bijschaving: b });
        r
    }

    #[test]
    fn een_camerarit_in_de_ether_gaat_eerst_naar_het_begin_en_rijdt_dan_zonder_pauze() {
        let mut stand = Stand::default();
        let mut v = 0;
        let r = met_rit(&mut stand, &mut v);
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, kies(&r, false));
        regie.volg(&stand, 0);
        let x = |tik: u64| regie.beeld(tik).ether.stand.positie;
        // eerst de gewone overgang van 2 s naar het beginpunt
        assert!(ongeveer(regie.beeld(0).ether.stand, standpunt(&stand, "A")));
        assert!((x(2 * T)[0] + 2.0).abs() < 1e-3 && (x(2 * T)[2] + 4.0).abs() < 1e-3);
        // dan meteen de rit: een meter per seconde
        assert!((x(3 * T)[0] + 1.0).abs() < 1e-3, "{:?}", x(3 * T));
        assert!((x(6 * T)[0] - 2.0).abs() < 1e-3);
        // en hij stopt aan het eind
        assert_eq!(x(6 * T), x(20 * T));
        assert_eq!(regie.beeld(20 * T).ether.snede, regie.beeld(0).ether.snede, "geen knip");
    }

    #[test]
    fn met_shift_of_begin_knippen_start_de_rit_meteen_op_het_beginpunt() {
        let mut stand = Stand::default();
        let mut v = 0;
        let r = met_rit(&mut stand, &mut v);
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        let snede = regie.beeld(0).ether.snede;
        doe(&mut stand, &mut v, kies(&r, true));
        regie.volg(&stand, 10);
        assert!((regie.beeld(10).ether.stand.positie[0] + 2.0).abs() < 1e-3);
        assert!((regie.beeld(10 + T).ether.stand.positie[0] + 1.0).abs() < 1e-3);
        assert_eq!(regie.beeld(10).ether.snede, snede + 1);

        let mut b = stand.camerarit(&r).unwrap().bijschaving;
        b.begin = gedeeld::camerarit::RitBegin::Knippen;
        doe(&mut stand, &mut v, Opdracht::BijschaafCamerarit { letter: r.clone(), bijschaving: b });
        doe(&mut stand, &mut v, kies("A", true));
        regie.volg(&stand, 20 * T);
        doe(&mut stand, &mut v, kies(&r, false));
        regie.volg(&stand, 21 * T);
        assert!((regie.beeld(21 * T).ether.stand.positie[0] + 2.0).abs() < 1e-3);
    }

    #[test]
    fn aanschuiven_eindigt_voor_elk_blok_in_de_ruimte_en_kijkt_naar_het_blok() {
        let r = ruimte();
        let volgorde = gedeeld::uitslagindeling::stoelen_van_links_naar_rechts();
        for van in 0..150 {
            for tot in van..150 {
                let s = aanschuif_stand(van, tot).unwrap();
                assert!(r.bevat(s.positie, 0.1), "{van}-{tot}: {:?} buiten de ruimte", s.positie);
                // het midden van het blok staat in beeld
                let n = (tot - van + 1) as f32;
                let m: [f32; 3] = std::array::from_fn(|k| volgorde[van..=tot].iter().map(|&i| STOELPLEKKEN[i][k]).sum::<f32>() / n);
                let d = sub(m, s.positie);
                let draai = (-d[0]).atan2(-d[2]);
                let verschil = (draai - s.draai + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                assert!(verschil.abs() < 0.05, "{van}-{tot}: kijkt {verschil} rad naast het blok");
            }
        }
    }

    #[test]
    fn aanschuiven_duurt_acht_seconden() {
        let mut stand = Stand::default();
        let mut v = 0;
        let sleutel = gedeeld::camera::aanschuifsleutel(40, 65);
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, kies(&sleutel, false));
        regie.volg(&stand, 0);
        let doel = aanschuif_stand(40, 65).unwrap();
        assert!(len(sub(regie.beeld(4 * T).ether.stand.positie, doel.positie)) > 0.1, "na 4 s nog onderweg");
        assert!(len(sub(regie.beeld(8 * T + 2).ether.stand.positie, doel.positie)) < 1e-3, "na 8 s aangekomen");
    }

    #[test]
    fn elke_vaste_camerarit_blijft_in_de_ruimte() {
        let r = ruimte();
        for rit in gedeeld::vaste_ritten::vaste() {
            let buiten: Vec<usize> = rit.punten().iter().enumerate().filter(|(_, p)| !r.bevat([p[0], p[1], p[2]], 0.1)).map(|(i, _)| i).collect();
            assert!(buiten.is_empty(), "{}: {} standen buiten de ruimte, de eerste op tik {:?}", rit.naam, buiten.len(), buiten.first());
        }
    }

    #[test]
    fn herhalen_begint_opnieuw_zonder_knip() {
        let mut stand = Stand::default();
        let mut v = 0;
        // een rondje van vier seconden
        let rond: Vec<[f32; 6]> = (0..=400)
            .map(|i| {
                let a = i as f32 / 400.0 * std::f32::consts::TAU;
                [1.5 * a.sin(), 3.0, -4.0 + 1.5 * (1.0 - a.cos()), 0.0, -0.3, 0.9]
            })
            .collect();
        doe(&mut stand, &mut v, Opdracht::ZetOefenen { aan: true });
        doe(&mut stand, &mut v, Opdracht::ZetOpnemen { aan: true, voor: None, opname: None });
        doe(&mut stand, &mut v, Opdracht::ZetOpnemen { aan: false, voor: None, opname: Some(rond) });
        let eigen = stand.cameraritten.iter().find(|r| !r.vast).unwrap();
        let r = eigen.letter.clone();
        let mut b = eigen.bijschaving;
        b.eind = gedeeld::camerarit::RitEind::Herhalen;
        b.begin = gedeeld::camerarit::RitBegin::Knippen;
        doe(&mut stand, &mut v, Opdracht::BijschaafCamerarit { letter: r.clone(), bijschaving: b });
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, kies(&r, false));
        regie.volg(&stand, 0);
        let p = |tik: u64| regie.beeld(tik).ether.stand.positie;
        let snede = regie.beeld(0).ether.snede;
        for ronde in 1..4 {
            assert!(len(sub(p(ronde * 4 * T), p(0))) < 1e-3, "ronde {ronde}");
            assert!(len(sub(p(ronde * 4 * T + T), p(T))) < 1e-3, "ronde {ronde}");
        }
        assert_eq!(regie.beeld(13 * T).ether.snede, snede, "herhalen is geen knip");
    }

    #[test]
    fn na_een_opname_speelt_de_camerarit_meteen_af_in_het_voorbeeld() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        oefen(&mut stand, &mut v, &mut regie, 0, true);
        let mut tik = 0;
        doe(&mut stand, &mut v, Opdracht::ZetOpnemen { aan: true, voor: None, opname: None });
        regie.volg(&stand, tik);
        let rijden = stuur_beide(&mut regie, &mut tik, 2 * T, VOORUIT);
        assert_eq!(regie.beeld(tik).opname, Some(2 * T as u32), "100 standen per seconde");
        let opname = regie.opname().unwrap().to_vec();
        assert_eq!(opname.len(), 2 * T as usize);
        let eerste = rijden[0].1;
        assert!((opname[0][2] - eerste.positie[2]).abs() < 1e-3, "de stand van het Voorbeeld");
        assert!(rijden.iter().all(|(e, _)| *e == standpunt(&stand, "A")), "de Ether blijft staan");

        doe(&mut stand, &mut v, Opdracht::ZetOpnemen { aan: false, voor: None, opname: Some(opname.clone()) });
        regie.volg(&stand, tik);
        assert_eq!(regie.beeld(tik).opname, None);
        // het Voorbeeld gaat vanaf de Ether (A) naar het begin van de rit en rijdt hem
        let b = regie.beeld(tik);
        assert!(ongeveer(b.voorbeeld.stand, standpunt(&stand, "A")));
        let eind = gedeeld::camerarit::als_stand(opname.last().unwrap());
        let r = stand.cameraritten.iter().find(|r| !r.vast).unwrap();
        let klaar = tik + 2 * T + (r.duur_s() * T as f32) as u64;
        assert!(len(sub(regie.beeld(klaar + 1).voorbeeld.stand.positie, eind.positie)) < 0.02);
        assert!(regie.beeld(tik + 3 * T).voorbeeld_rit.is_some(), "de groene streep op de balk");
        // de Ether stond al die tijd stil
        assert_eq!(regie.beeld(klaar).ether.stand, standpunt(&stand, "A"));
    }

    #[test]
    fn de_stick_neemt_het_voorbeeld_over_als_het_tijdens_oefenen_afspeelt() {
        let mut stand = Stand::default();
        let mut v = 0;
        let r = met_rit(&mut stand, &mut v);
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        oefen(&mut stand, &mut v, &mut regie, 0, true);
        doe(&mut stand, &mut v, Opdracht::ProbeerInVoorbeeld { letter: r });
        regie.volg(&stand, 0);
        let mut tik = 0;
        let los = stuur_beide(&mut regie, &mut tik, T, Stuur::LOS);
        let hier = los.last().unwrap().1;
        assert!(!ongeveer(hier, standpunt(&stand, "A")), "speelt af");
        let na = stuur_beide(&mut regie, &mut tik, 2, VOORUIT);
        // de camera was midden in de overgang (een paar cm per tik): hooguit één tik verder
        let tik_eerder = len(sub(hier.positie, los[los.len() - 2].1.positie));
        assert!(len(sub(na[0].1.positie, hier.positie)) < 1.5 * tik_eerder, "geen sprong");
        // nog een Stand erbij: het afspelen begint niet opnieuw
        doe(&mut stand, &mut v, kies("D", false));
        regie.volg(&stand, tik);
        assert!(len(sub(regie.beeld(tik).voorbeeld.stand.positie, na[1].1.positie)) < 1e-4);
    }

    #[test]
    fn probeer_in_voorbeeld_beweegt_vanaf_de_ether_en_de_ether_blijft() {
        let mut stand = Stand::default();
        let mut v = 0;
        let mut regie = Cameraregie::nieuw(&stand, 0, ruimte());
        doe(&mut stand, &mut v, kies("D", true));
        regie.volg(&stand, 0);
        doe(&mut stand, &mut v, Opdracht::ProbeerInVoorbeeld { letter: "F".into() });
        regie.volg(&stand, 10);
        assert!(ongeveer(regie.beeld(10).voorbeeld.stand, standpunt(&stand, "D")));
        let half = regie.beeld(10 + T).voorbeeld.stand;
        assert!(!ongeveer(half, standpunt(&stand, "D")) && !ongeveer(half, standpunt(&stand, "F")));
        assert!(ongeveer(regie.beeld(10 + 2 * T).voorbeeld.stand, standpunt(&stand, "F")));
        assert!(ongeveer(regie.beeld(10 + 2 * T).ether.stand, standpunt(&stand, "D")));
        // een nieuwe klaargezette Graphic stopt het afspelen
        let graphic = Graphic::Titelkaart { kop: "Voorbeeldgegevens".into(), regel: String::new() };
        doe(&mut stand, &mut v, Opdracht::ZetKlaar { graphic, standpunt: Some("G".into()) });
        regie.volg(&stand, 3 * T);
        assert!(ongeveer(regie.beeld(3 * T).voorbeeld.stand, standpunt(&stand, "G")));
    }
}
