//! De camera in de Zaal: Standpunten, de keuze van de Regie en de camerastand die de server
//! 100 keer per seconde naar de Renderers stuurt (ADR 0002).
//!
//! Alles in de assen van Bevy, in meter: y omhoog, de Voorzitter aan de kant van +z, vak 1 aan
//! de kant van +x. De camera kantelt nooit opzij, dus draai, helling en beeldhoek zijn genoeg.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Graphic, Stand, Weigering};

/// Hoe vaak per seconde de server de camera uitrekent.
pub const TIKKEN_PER_SECONDE: u32 = 100;
/// Kortste en langste overgang naar een Standpunt, in seconden.
pub const DUUR_MIN: f32 = 0.5;
pub const DUUR_MAX: f32 = 6.0;
/// Na zoveel seconden stopt een Standpunt met langzaam schuiven.
pub const SCHUIVEN_S: f32 = 20.0;

/// De onzichtbare ruimte waarin de camera mag vliegen, zoals `zaal/ruimte.py` hem in Blender
/// tekende. De server rekent ermee; het dashboard tekent de wanden op de plattegrond.
pub const RUIMTE_JSON: &str = include_str!("ruimte.json");

/// Waar een camera staat en waarheen hij kijkt.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[ts(export)]
pub struct Camerastand {
    /// Plek in meter.
    pub positie: [f32; 3],
    /// Draai om de verticale as in radialen. 0 kijkt langs -z (vanaf de Voorzitter de Zaal in),
    /// positief draait naar links.
    pub draai: f32,
    /// Helling in radialen, positief omhoog.
    pub helling: f32,
    /// Verticale beeldhoek in radialen, bij een beeld van 16:9.
    pub beeldhoek: f32,
}

impl Camerastand {
    /// Een stand uit graden, zoals de vaste Standpunten hieronder.
    pub fn uit_graden(positie: [f32; 3], draai: f32, helling: f32, beeldhoek: f32) -> Self {
        Camerastand {
            positie,
            draai: draai.to_radians(),
            helling: helling.to_radians(),
            beeldhoek: beeldhoek.to_radians(),
        }
    }

    /// De richting waarin de camera kijkt (lengte 1).
    pub fn vooruit(&self) -> [f32; 3] {
        let (sd, cd) = self.draai.sin_cos();
        let (sh, ch) = self.helling.sin_cos();
        [-sd * ch, sh, -cd * ch]
    }

    /// Waar een punt in beeld komt: x en y van -1 tot 1 (midden 0, rechts en boven positief) bij
    /// een beeld van 16:9. `None` als het punt achter de camera ligt.
    pub fn in_beeld(&self, punt: [f32; 3]) -> Option<[f32; 2]> {
        let v = self.vooruit();
        let (sd, cd) = self.draai.sin_cos();
        let rechts = [cd, 0.0, -sd];
        let boven = kruis(rechts, v);
        let d = [
            punt[0] - self.positie[0],
            punt[1] - self.positie[1],
            punt[2] - self.positie[2],
        ];
        let diepte = punt_maal(d, v);
        if diepte <= 0.01 {
            return None;
        }
        let th = (self.beeldhoek / 2.0).tan();
        let tw = th * 16.0 / 9.0;
        Some([
            punt_maal(d, rechts) / diepte / tw,
            punt_maal(d, boven) / diepte / th,
        ])
    }
}

fn punt_maal(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn kruis(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Bewegen of knippen naar een Standpunt.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[ts(export)]
pub enum OvergangSoort {
    #[default]
    Bewegen,
    Knippen,
}

/// Hoe de camera naar een Standpunt gaat.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[ts(export)]
pub struct CameraOvergang {
    pub soort: OvergangSoort,
    /// Duur in seconden, van [`DUUR_MIN`] tot [`DUUR_MAX`].
    pub duur_s: f32,
    /// 0 begint meteen op volle snelheid, 1 begint zo zacht mogelijk.
    pub zacht_starten: f32,
    /// 0 stopt abrupt, 1 komt zo zacht mogelijk tot stilstand.
    pub zacht_stoppen: f32,
}

impl Default for CameraOvergang {
    fn default() -> Self {
        CameraOvergang {
            soort: OvergangSoort::Bewegen,
            duur_s: 2.0,
            zacht_starten: 1.0,
            zacht_stoppen: 1.0,
        }
    }
}

impl CameraOvergang {
    /// De duur, binnen de grenzen.
    pub fn duur(&self) -> f32 {
        if self.duur_s.is_finite() {
            self.duur_s.clamp(DUUR_MIN, DUUR_MAX)
        } else {
            2.0
        }
    }

    /// Hoe ver de camera is (0 tot 1) na een deel `t` (0 tot 1) van de duur. Een kromme van
    /// Hermite: met zacht starten en stoppen op 1 is de snelheid aan begin en eind nul, op 0 is
    /// de beweging daar even snel als gemiddeld.
    pub fn voortgang(&self, t: f32) -> f32 {
        zachte_voortgang(t, self.zacht_starten, self.zacht_stoppen)
    }
}

/// Hoe ver een beweging is (0 tot 1) na een deel `t` (0 tot 1) van de duur, met zacht starten en
/// stoppen van 0 tot 1 (zie [`CameraOvergang::voortgang`]). Ook voor een Camerarit.
pub fn zachte_voortgang(t: f32, zacht_starten: f32, zacht_stoppen: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let m0 = 1.0 - zacht_starten.clamp(0.0, 1.0);
    let m1 = 1.0 - zacht_stoppen.clamp(0.0, 1.0);
    let (t2, t3) = (t * t, t * t * t);
    (t3 - 2.0 * t2 + t) * m0 + (-2.0 * t3 + 3.0 * t2) + (t3 - t2) * m1
}

/// Wat de camera doet als hij bij het Standpunt is.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Default)]
#[serde(tag = "soort")]
#[ts(export)]
pub enum NaAankomst {
    /// Blijft stil staan.
    #[default]
    Stil,
    /// Schuift langzaam in één richting en stopt na [`SCHUIVEN_S`] seconden.
    Schuiven {
        /// De richting; de lengte telt niet.
        richting: [f32; 3],
        /// Snelheid in centimeter per seconde.
        cm_per_s: f32,
    },
}

/// De toetsen voor eigen Standpunten, in de volgorde waarin een nieuw Standpunt ze krijgt.
pub const EIGEN_TOETSEN: [&str; 3] = ["J", "K", "L"];
/// Langste naam van een Standpunt, in tekens.
pub const MAX_STANDPUNTNAAM: usize = 40;

/// Een vooraf vastgelegde camerapositie in de Zaal, met een letter op het toetsenbord.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[ts(export)]
pub struct Standpunt {
    /// De sleutel waarmee een Sjabloon of Graphic het Standpunt kiest: "A" tot "H" voor de vaste,
    /// "E1", "E2" en verder voor eigen Standpunten. Verandert nooit.
    pub letter: String,
    pub naam: String,
    /// Een vast Standpunt kun je niet wissen of hernoemen, wel opnieuw opslaan.
    pub vast: bool,
    /// De toets van een eigen Standpunt (J, K of L), of geen: dan kies je het in de lijst of
    /// op de plattegrond. Een vast Standpunt heeft zijn letter als toets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub toets: Option<String>,
    pub stand: Camerastand,
    #[serde(default)]
    pub overgang: CameraOvergang,
    #[serde(default)]
    pub na_aankomst: NaAankomst,
}

impl Standpunt {
    /// De zes vaste Standpunten (A, S, D, F, G, H), gekozen met beelden uit de Zaal van
    /// `zaal/maak.sh`. Ze staan binnen de onzichtbare ruimte van de server.
    pub fn vaste() -> Vec<Standpunt> {
        let vast = |letter: &str, naam: &str, stand: Camerastand| Standpunt {
            letter: letter.into(),
            naam: naam.into(),
            vast: true,
            toets: None,
            stand,
            overgang: CameraOvergang::default(),
            na_aankomst: NaAankomst::Stil,
        };
        vec![
            // Hoog tegen de wand achter de Voorzitter, met een heel wijde lens: zo staan alle
            // 150 Stoelen in beeld, ook de voorste aan de zijkanten.
            vast(
                "A",
                "Totaal vanaf voorzitter",
                Camerastand::uit_graden([0.0, 4.5, 4.45], 0.0, -23.0, 99.0),
            ),
            vast(
                "S",
                "Laag over de banken",
                Camerastand::uit_graden([-3.5, 1.8, -1.0], -25.0, -6.0, 45.0),
            ),
            vast(
                "D",
                "Linkerflank",
                Camerastand::uit_graden([-9.8, 2.8, 0.8], -30.0, -12.0, 55.0),
            ),
            vast(
                "F",
                "Rechterflank",
                Camerastand::uit_graden([9.8, 2.8, 0.8], 30.0, -12.0, 55.0),
            ),
            vast(
                "G",
                "Bovenaf",
                Camerastand::uit_graden([0.0, 4.5, -3.0], 0.0, -68.0, 80.0),
            ),
            // Recht voor het spreekgestoelte (x 3,19, z 2,21, 34,9 graden gedraaid), op ooghoogte.
            vast(
                "H",
                "Katheder",
                Camerastand::uit_graden([1.21, 1.7, -0.68], -145.5, -2.0, 30.0),
            ),
        ]
    }
}

/// Hoe ver de camera van een Stoel staat bij het shot van de Persoonskaart, in meter langs de
/// vloer, en hoeveel graden opzij van recht voor de Stoel.
const SHOT_AFSTAND: f32 = 3.2;
const SHOT_OPZIJ: f32 = 28.0;
/// Hoe hoog boven de rugleuning de camera staat, en waar hij naar kijkt (het hoofd van wie er zit).
const SHOT_HOOGTE: f32 = 0.95;
const SHOT_HOOFD: f32 = 0.1;
/// Verticale beeldhoek van het shot, in graden.
const SHOT_BEELDHOEK: f32 = 34.0;
/// Waar de Stoel in beeld komt: links van het midden, want de Persoonskaart staat rechts
/// (x -1 tot 1 over de breedte; de kaart begint bij 0,21).
const SHOT_PLEK_X: f32 = -0.42;

/// Aanschuiven: de keuze in een Sjabloon, Graphic of Preset. Bij het tonen wordt dat de sleutel
/// van het blok van de gekozen Partij ([`aanschuifsleutel`]); zonder Partij het Standpunt van de
/// soort Graphic.
pub const AANSCHUIVEN: &str = "aanschuiven";

/// Zo lang duurt Aanschuiven, in seconden bij snelheid 100.
pub const AANSCHUIVEN_S: f32 = 8.0;

/// De overgang van Aanschuiven: langzaam en zacht.
pub fn aanschuif_overgang() -> CameraOvergang {
    CameraOvergang { duur_s: AANSCHUIVEN_S, ..CameraOvergang::default() }
}

/// De sleutel van Aanschuiven naar de Stoelen op plaats `van` tot en met `tot` van links naar
/// rechts ([`crate::uitslagindeling::stoelen_van_links_naar_rechts`]), zoals "aanschuiven-40-65".
pub fn aanschuifsleutel(van: usize, tot: usize) -> String {
    format!("{AANSCHUIVEN}-{van}-{tot}")
}

/// De plaatsen van links naar rechts bij een sleutel van [`aanschuifsleutel`].
pub fn aanschuif_uit_sleutel(sleutel: &str) -> Option<(usize, usize)> {
    let (van, tot) = sleutel.strip_prefix(AANSCHUIVEN)?.strip_prefix('-')?.split_once('-')?;
    let (van, tot) = (van.parse().ok()?, tot.parse().ok()?);
    (van <= tot && tot < crate::AANTAL_STOELEN).then_some((van, tot))
}

/// Waar Aanschuiven eindigt: schuin van voren op het blok, vanaf de kant van de Voorzitter, zo ver
/// dat het hele blok in beeld past, en hoog genoeg om over de banken ervoor te kijken.
pub fn aanschuif_stand(van: usize, tot: usize) -> Option<Camerastand> {
    use crate::zaalkaart::STOELPLEKKEN;
    let volgorde = crate::uitslagindeling::stoelen_van_links_naar_rechts();
    let plekken: Vec<[f32; 3]> = volgorde.get(van..=tot)?.iter().map(|&i| STOELPLEKKEN[i]).collect();
    let n = plekken.len() as f32;
    let midden: [f32; 3] = std::array::from_fn(|k| plekken.iter().map(|p| p[k]).sum::<f32>() / n);
    let breed = plekken
        .iter()
        .map(|p| (p[0] - midden[0]).hypot(p[2] - midden[2]))
        .fold(0.0_f32, f32::max)
        .max(0.8);
    // van het blok naar de Voorzitter, en zo ver dat het blok de helft van de breedte vult
    let naar = [AANSCHUIF_VANAF[0] - midden[0], AANSCHUIF_VANAF[1] - midden[2]];
    let lengte = naar[0].hypot(naar[1]).max(1e-3);
    let beeldhoek = AANSCHUIF_BEELDHOEK.to_radians();
    let horizontaal = 2.0 * ((beeldhoek / 2.0).tan() * 16.0 / 9.0).atan();
    let afstand = (breed / (horizontaal / 2.0 * AANSCHUIF_VULLING).tan()).clamp(2.5, lengte * 0.85);
    let positie = [
        midden[0] + naar[0] / lengte * afstand,
        (midden[1] + 1.2 + 0.18 * afstand).clamp(2.0, 4.2),
        midden[2] + naar[1] / lengte * afstand,
    ];
    let d = [midden[0] - positie[0], midden[1] + 0.2 - positie[1], midden[2] - positie[2]];
    Some(Camerastand {
        positie,
        draai: (-d[0]).atan2(-d[2]),
        helling: d[1].atan2(d[0].hypot(d[2])),
        beeldhoek,
    })
}

/// Vanaf dit punt (x, z) kijkt Aanschuiven naar een blok: voor de Voorzitter, in het midden.
const AANSCHUIF_VANAF: [f32; 2] = [0.0, 1.5];
/// De verticale beeldhoek van Aanschuiven, in graden.
const AANSCHUIF_BEELDHOEK: f32 = 40.0;
/// Welk deel van de breedte van het beeld het blok vult.
const AANSCHUIF_VULLING: f32 = 0.6;

/// De sleutel waarmee een Graphic de camera naar het shot van een Stoel stuurt, zoals
/// "stoel-3-2-1". Die kies je overal waar je een Standpunt kiest; de overgang is die van een
/// Standpunt ([`CameraOvergang::default`]).
pub fn stoelsleutel(stoel: crate::Stoel) -> String {
    format!("stoel-{}-{}-{}", stoel.vak, stoel.rij, stoel.nummer)
}

/// De Stoel bij een sleutel van [`stoelsleutel`].
pub fn stoel_uit_sleutel(sleutel: &str) -> Option<crate::Stoel> {
    let mut d = sleutel.strip_prefix("stoel-")?.split('-').map(str::parse::<u8>);
    let stoel = crate::Stoel { vak: d.next()?.ok()?, rij: d.next()?.ok()?, nummer: d.next()?.ok()? };
    (d.next().is_none() && stoel.index().is_some()).then_some(stoel)
}

/// Het shot van de Persoonskaart op een Stoel: schuin van voren, van de kant van het midden van
/// de Zaal waar het Kamerlid naartoe kijkt, met de Stoel links in beeld (de kaart staat rechts).
/// Het is een stand als die van een Standpunt; de server rekent de overgang erheen uit.
pub fn shot_op_stoel(stoel: crate::Stoel) -> Option<Camerastand> {
    let p = crate::zaalkaart::STOELPLEKKEN[stoel.index()?];
    // recht voor de Stoel: naar het midden van de Zaal
    let r = (p[0] * p[0] + p[2] * p[2]).sqrt().max(1e-3);
    let voor = [-p[0] / r, -p[2] / r];
    // schuin: opzij naar de middenlijn van de Zaal (x = 0), zodat de camera binnen blijft
    let kant = if p[0] >= 0.0 { -1.0 } else { 1.0 };
    let (s, c) = (kant * SHOT_OPZIJ).to_radians().sin_cos();
    let richting = [voor[0] * c - voor[1] * s, voor[0] * s + voor[1] * c];
    let positie = [
        p[0] + richting[0] * SHOT_AFSTAND,
        p[1] + SHOT_HOOGTE,
        p[2] + richting[1] * SHOT_AFSTAND,
    ];
    let doel = [p[0], p[1] + SHOT_HOOFD, p[2]];
    let d = [doel[0] - positie[0], doel[1] - positie[1], doel[2] - positie[2]];
    let vlak = (d[0] * d[0] + d[2] * d[2]).sqrt();
    let beeldhoek = SHOT_BEELDHOEK.to_radians();
    // de camera draait zoveel naar rechts dat de Stoel op SHOT_PLEK_X komt
    let opzij = (-SHOT_PLEK_X * (beeldhoek / 2.0).tan() * 16.0 / 9.0).atan();
    Some(Camerastand {
        positie,
        draai: (-d[0]).atan2(-d[2]) - opzij,
        helling: d[1].atan2(vlak),
        beeldhoek,
    })
}

impl Standpunt {
    /// De toets op het toetsenbord: de letter van een vast Standpunt, J, K of L, of geen.
    pub fn toets(&self) -> Option<&str> {
        if self.vast { Some(&self.letter) } else { self.toets.as_deref() }
    }

    /// Hoe de Regie het Standpunt herkent, zoals "Standpunt J (Close-up)".
    fn kenmerk(&self) -> String {
        format!("Standpunt {} ({})", self.toets().unwrap_or(&self.letter), self.naam)
    }
}

/// Waar een Standpunt gebruikt wordt; zolang er iets is, kan het niet gewist worden.
/// Sjablonen (#42) komen hier bij als ze een eigen Standpunt kunnen kiezen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StandpuntGebruik {
    KlaargezetteGraphic { naam: String },
    GraphicInEther { naam: String },
    Preset { nummer: u16, naam: String },
    /// De camera van de Ether staat erop of gaat erheen (niet als de Vrije camera aan staat).
    CameraVanDeEther,
}

impl std::fmt::Display for StandpuntGebruik {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StandpuntGebruik::KlaargezetteGraphic { naam } => write!(f, "de klaargezette Graphic {naam}"),
            StandpuntGebruik::GraphicInEther { naam } => write!(f, "de Graphic in de Ether {naam}"),
            StandpuntGebruik::Preset { nummer, naam } => write!(f, "Preset {nummer} {naam}"),
            StandpuntGebruik::CameraVanDeEther => write!(f, "de camera van de Ether"),
        }
    }
}

/// Een naam voor de Regie, zoals `Titelkaart "Uitslag"`.
fn graphic_naam(g: &Graphic) -> String {
    let sjabloon = match g {
        Graphic::Titelkaart { .. } => "Titelkaart",
        Graphic::UitslagStemming { .. } => "Uitslag van een Stemming",
        Graphic::HoofdelijkeStemming { .. } => "Hoofdelijke stemming",
        Graphic::FractiesVoorEnTegen { .. } => "Fracties voor en tegen",
        Graphic::Zetelverdeling { .. } => "Zetelverdeling",
        Graphic::Persoonskaart { .. } => "Persoonskaart",
        Graphic::Zetelbalk { .. } => "Zetelbalk",
        Graphic::TweeSets { .. } => "Twee sets naast elkaar",
        Graphic::UitslagPerPartij { .. } => "Uitslag per Partij",
        Graphic::WinstEnVerlies { .. } => "Winst en verlies",
        Graphic::PartijUitgelicht { .. } => "Partij uitgelicht",
        Graphic::Coalitie { .. } => "Coalitie in de Zaal",
        Graphic::Coalitiebalk { .. } => "Coalitiebalk",
        Graphic::Eigen { id, .. } => id,
    };
    let kop = match g {
        Graphic::Titelkaart { kop, .. } => kop.clone(),
        Graphic::UitslagStemming { uitslag }
        | Graphic::HoofdelijkeStemming { uitslag }
        | Graphic::FractiesVoorEnTegen { uitslag } => uitslag.samenvatting(),
        Graphic::Zetelverdeling { set, cijfers, .. }
        | Graphic::Zetelbalk { set, cijfers, .. }
        | Graphic::TweeSets { set, cijfers, .. }
        | Graphic::UitslagPerPartij { set, cijfers, .. }
        | Graphic::WinstEnVerlies { set, cijfers, .. }
        | Graphic::PartijUitgelicht { set, cijfers, .. }
        | Graphic::Coalitie { set, cijfers, .. }
        | Graphic::Coalitiebalk { set, cijfers, .. } => {
            cijfers.as_ref().map_or(set.clone(), |c| c.set.label.clone())
        }
        Graphic::Persoonskaart { wie, gegevens, .. } => gegevens.as_ref().map_or_else(
            || match wie {
                crate::persoonskaart::Wie::Kamerlid { id } | crate::persoonskaart::Wie::Eigen { id } => id.clone(),
            },
            |g| g.naam.clone(),
        ),
        Graphic::Eigen { velden, .. } => velden.values().find(|v| !v.trim().is_empty()).cloned().unwrap_or_default(),
    };
    format!("{sjabloon} \"{kop}\"")
}

/// Eigen Standpunten: bewaren, hernoemen, overschrijven, een toets geven en wissen (#44).
impl Stand {
    /// Alles wat dit Standpunt gebruikt. Leeg: het mag weg.
    pub fn gebruik_van_standpunt(&self, letter: &str) -> Vec<StandpuntGebruik> {
        let mut gebruik = Vec::new();
        if let Some(k) = self.klaar.as_ref().filter(|k| k.standpunt() == letter) {
            gebruik.push(StandpuntGebruik::KlaargezetteGraphic { naam: graphic_naam(&k.graphic) });
        }
        if let Some(e) = self.ether.as_ref().filter(|e| e.standpunt() == letter) {
            gebruik.push(StandpuntGebruik::GraphicInEther { naam: graphic_naam(&e.graphic) });
        }
        for p in self.presets.iter().filter(|p| p.standpunt() == letter) {
            gebruik.push(StandpuntGebruik::Preset { nummer: p.nummer, naam: graphic_naam(&p.graphic) });
        }
        if !self.camera.vrij && self.camera.standpunt == letter {
            gebruik.push(StandpuntGebruik::CameraVanDeEther);
        }
        gebruik
    }

    pub(crate) fn bewaar_standpunt(&mut self, naam: &str, stand: Option<Camerastand>) -> Result<(), Weigering> {
        let naam = geldige_naam(naam)?;
        let stand = geldige_stand(stand)?;
        let bezet = |l: &str| self.standpunten.iter().any(|s| s.letter == l);
        let letter = (1..).map(|n| format!("E{n}")).find(|l| !bezet(l)).expect("oneindig");
        let toets = self.eerste_vrije_toets();
        self.standpunten.push(Standpunt {
            letter,
            naam,
            vast: false,
            toets,
            stand,
            overgang: CameraOvergang::default(),
            na_aankomst: NaAankomst::Stil,
        });
        Ok(())
    }

    pub(crate) fn overschrijf_standpunt(&mut self, letter: &str, stand: Option<Camerastand>) -> Result<(), Weigering> {
        let stand = geldige_stand(stand)?;
        self.standpunt_mut(letter)?.stand = stand;
        Ok(())
    }

    /// De eerste van J, K en L die geen eigen Standpunt en geen Camerarit heeft.
    pub(crate) fn eerste_vrije_toets(&self) -> Option<String> {
        let bezet = |t: &str| {
            self.standpunten.iter().any(|s| s.toets() == Some(t))
                || self.cameraritten.iter().any(|r| r.toets.as_deref() == Some(t))
        };
        EIGEN_TOETSEN.iter().find(|t| !bezet(t)).map(|t| t.to_string())
    }

    /// Hernoemt een eigen Standpunt of een Camerarit.
    pub(crate) fn hernoem_standpunt(&mut self, letter: &str, naam: &str) -> Result<(), Weigering> {
        let naam = geldige_naam(naam)?;
        if let Some(r) = self.cameraritten.iter_mut().find(|r| r.letter == letter) {
            r.naam = naam;
            return Ok(());
        }
        self.eigen_standpunt_mut(letter)?.naam = naam;
        Ok(())
    }

    /// Geeft een eigen Standpunt of een Camerarit een toets; wie hem had, raakt hem kwijt.
    pub(crate) fn zet_standpunt_toets(&mut self, letter: &str, toets: Option<&str>) -> Result<(), Weigering> {
        match self.camerarit(letter) {
            Some(r) if r.vast => return Err(Weigering::VasteCamerarit(letter.into())),
            Some(_) => {}
            None => {
                self.eigen_standpunt_mut(letter)?;
            }
        }
        if let Some(t) = toets.filter(|t| !EIGEN_TOETSEN.contains(t)) {
            return Err(Weigering::OnbekendeToets(t.into()));
        }
        let eigen = self.standpunten.iter_mut().filter(|s| !s.vast).map(|s| (&s.letter, &mut s.toets));
        let ritten = self.cameraritten.iter_mut().filter(|r| !r.vast).map(|r| (&r.letter, &mut r.toets));
        for (l, t) in eigen.chain(ritten) {
            if l == letter {
                *t = toets.map(Into::into);
            } else if toets.is_some() && t.as_deref() == toets {
                *t = None;
            }
        }
        Ok(())
    }

    /// Wist een eigen Standpunt of een Camerarit, als niets het gebruikt.
    pub(crate) fn wis_standpunt(&mut self, letter: &str) -> Result<(), Weigering> {
        let kenmerk = match self.camerarit(letter) {
            Some(r) if r.vast => return Err(Weigering::VasteCamerarit(letter.into())),
            Some(r) => r.kenmerk(),
            None => self.eigen_standpunt_mut(letter)?.kenmerk(),
        };
        let gebruik = self.gebruik_van_standpunt(letter);
        if !gebruik.is_empty() {
            let waar: Vec<String> = gebruik.iter().map(ToString::to_string).collect();
            return Err(Weigering::StandpuntInGebruik { standpunt: kenmerk, waar: waar.join(" en ") });
        }
        self.standpunten.retain(|s| s.letter != letter);
        self.cameraritten.retain(|r| r.letter != letter);
        if self.camera.proef.as_ref().is_some_and(|p| p.letter == letter) {
            self.camera.proef = None;
        }
        Ok(())
    }

    pub(crate) fn standpunt_mut(&mut self, letter: &str) -> Result<&mut Standpunt, Weigering> {
        self.standpunten
            .iter_mut()
            .find(|s| s.letter == letter)
            .ok_or_else(|| Weigering::OnbekendStandpunt(letter.into()))
    }

    fn eigen_standpunt_mut(&mut self, letter: &str) -> Result<&mut Standpunt, Weigering> {
        let s = self.standpunt_mut(letter)?;
        if s.vast {
            return Err(Weigering::VastStandpunt(letter.into()));
        }
        Ok(s)
    }
}

fn geldige_naam(naam: &str) -> Result<String, Weigering> {
    let naam = naam.trim();
    if naam.is_empty() {
        return Err(Weigering::StandpuntZonderNaam);
    }
    if naam.chars().count() > MAX_STANDPUNTNAAM {
        return Err(Weigering::StandpuntnaamTeLang);
    }
    Ok(naam.into())
}

fn geldige_stand(stand: Option<Camerastand>) -> Result<Camerastand, Weigering> {
    stand
        .filter(|s| s.positie.iter().chain([&s.draai, &s.helling, &s.beeldhoek]).all(|v| v.is_finite()))
        .ok_or(Weigering::GeenCamerastand)
}

/// Het Standpunt waar de Regie de camera van de Ether heen stuurde.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Camerakeuze {
    /// De letter van het Standpunt.
    pub standpunt: String,
    /// Meteen knippen, zonder beweging (Shift met de letter).
    pub knip: bool,
    /// Het volgnummer van de Stand waarin de keuze viel. Een nieuwe keuze voor hetzelfde
    /// Standpunt telt ook: de camera gaat er dan opnieuw heen.
    #[ts(type = "number")]
    pub sinds: u64,
    /// De Vrije camera staat aan: de gamepad stuurt de camera van de Ether. Een Standpunt kiezen
    /// zet hem uit; aan- en uitzetten verandert `standpunt`, `knip` en `sinds` niet.
    #[serde(default)]
    pub vrij: bool,
    /// Oefenen staat aan: de gamepad stuurt alleen de camera van het Voorbeeld.
    #[serde(default)]
    pub oefenen: bool,
    /// Er loopt een opname van een Camerarit (alleen tijdens Oefenen).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub opname: Option<Opname>,
    /// Wat het Voorbeeld nu afspeelt op verzoek van de Regie. Een nieuwe klaargezette Graphic,
    /// een keuze voor de Ether of Oefenen aan of uit maakt dit leeg.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub proef: Option<Proef>,
}

/// Een opname die loopt.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Opname {
    /// De Camerarit die opnieuw wordt opgenomen; zonder komt er een nieuwe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub voor: Option<String>,
    /// Het volgnummer van de Stand waarin de opname begon.
    #[ts(type = "number")]
    pub sinds: u64,
}

/// Een Standpunt of Camerarit dat het Voorbeeld afspeelt, vanaf waar de Ether toen stond.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Proef {
    pub letter: String,
    /// Het volgnummer van de Stand met het verzoek: een nieuw verzoek speelt opnieuw af.
    #[ts(type = "number")]
    pub sinds: u64,
}

impl Default for Camerakeuze {
    fn default() -> Self {
        Camerakeuze {
            standpunt: "A".into(),
            knip: true,
            sinds: 0,
            vrij: false,
            oefenen: false,
            opname: None,
            proef: None,
        }
    }
}

/// Eén camera op één tik: de stand, en een teller die verspringt bij elke knip. Tussen twee tikken
/// met een andere `snede` vult een Renderer niets in.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[ts(export)]
pub struct Camerapunt {
    pub stand: Camerastand,
    pub snede: u32,
}

/// Wat de server 100 keer per seconde naar de Renderers stuurt.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[ts(export)]
pub struct Camerabeeld {
    /// Tikken sinds de server startte, 100 per seconde.
    #[ts(type = "number")]
    pub tik: u64,
    /// De camera van de Ether (en de Reserve).
    pub ether: Camerapunt,
    /// De camera van het Voorbeeld: het Standpunt van de klaargezette Graphic.
    pub voorbeeld: Camerapunt,
    /// Tijdens een opname: hoeveel tikken er al zijn opgenomen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub opname: Option<u32>,
    /// Speelt het Voorbeeld een Camerarit af: waar in de opname het nu is, in seconden.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub voorbeeld_rit: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zaalkaart::STOELPLEKKEN;

    #[test]
    fn zacht_starten_en_stoppen_begint_en_eindigt_stil() {
        let o = CameraOvergang::default();
        assert_eq!(o.voortgang(0.0), 0.0);
        assert!((o.voortgang(1.0) - 1.0).abs() < 1e-6);
        assert!((o.voortgang(0.5) - 0.5).abs() < 1e-6);
        // snelheid aan het begin en eind bijna nul, in het midden hoger dan gemiddeld
        assert!(o.voortgang(0.01) < 0.001);
        assert!(1.0 - o.voortgang(0.99) < 0.001);
        assert!(o.voortgang(0.51) - o.voortgang(0.49) > 0.02);
        // zonder zachtheid: rechte lijn
        let hard = CameraOvergang {
            zacht_starten: 0.0,
            zacht_stoppen: 0.0,
            ..o
        };
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!((hard.voortgang(t) - t).abs() < 1e-6);
        }
        // altijd vooruit, nooit voorbij het doel
        let half = CameraOvergang {
            zacht_starten: 0.3,
            zacht_stoppen: 0.8,
            ..o
        };
        let mut vorige = 0.0;
        for i in 1..=100 {
            let v = half.voortgang(i as f32 / 100.0);
            assert!(v >= vorige && v <= 1.0 + 1e-6);
            vorige = v;
        }
    }

    #[test]
    fn de_duur_blijft_tussen_een_halve_en_zes_seconden() {
        let o = |duur_s| CameraOvergang {
            duur_s,
            ..Default::default()
        };
        assert_eq!(o(0.1).duur(), 0.5);
        assert_eq!(o(9.0).duur(), 6.0);
        assert_eq!(o(2.5).duur(), 2.5);
        assert_eq!(o(f32::NAN).duur(), 2.0);
    }

    #[test]
    fn de_zes_vaste_standpunten_hebben_hun_letter() {
        let letters: Vec<String> = Standpunt::vaste().into_iter().map(|s| s.letter).collect();
        assert_eq!(letters, ["A", "S", "D", "F", "G", "H"]);
        assert!(
            Standpunt::vaste()
                .iter()
                .all(|s| s.vast && s.overgang.duur() == 2.0)
        );
    }

    #[test]
    fn totaal_vanaf_voorzitter_toont_alle_150_stoelen() {
        let a = &Standpunt::vaste()[0];
        let buiten: Vec<usize> = STOELPLEKKEN
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                // de hele rugleuning, niet alleen het midden: 45 cm naar beide kanten
                [-0.45f32, 0.0, 0.45].iter().any(|dx| {
                    a.stand
                        .in_beeld([p[0] + dx, p[1], p[2]])
                        .is_none_or(|[x, y]| x.abs() > 0.97 || y.abs() > 0.97)
                })
            })
            .map(|(i, _)| i)
            .collect();
        assert!(
            buiten.is_empty(),
            "Stoelen buiten beeld vanaf A: {buiten:?}"
        );
    }

    /// Schrijft de plattegrond voor het dashboard naast de typen van ts-rs (zie `.cargo/config.toml`).
    #[test]
    fn schrijf_de_zaalkaart_voor_het_dashboard() {
        use crate::zaalkaart::HANGLAMPEN;
        let Some(map) = std::env::var_os("TS_RS_EXPORT_DIR") else {
            return;
        };
        let paar = |x: f32, z: f32| format!("[{x:.2}, {z:.2}]");
        let stoelen: Vec<String> = STOELPLEKKEN.iter().map(|p| paar(p[0], p[2])).collect();
        let lampen: Vec<String> = HANGLAMPEN.iter().map(|l| paar(l.x, l.z)).collect();
        let ruimte: serde_json::Value = serde_json::from_str(RUIMTE_JSON).unwrap();
        let maat = |k: &str| ruimte[k].as_f64().unwrap();
        let tekst = format!(
            "// Gemaakt door `cargo test -p gedeeld` uit crates/gedeeld/src/zaalkaart.rs; niet met de hand wijzigen.\n\
             // Plekken in meter van bovenaf: [x, z], de Voorzitter aan de kant van +z, vak 1 aan de kant van +x.\n\n\
             /** Per Stoel, in de volgorde van de Stand (vak, rij, nummer). */\n\
             export const STOELPLEKKEN: [number, number][] = [\n  {}\n];\n\n\
             /** De hanglampen onder het daklicht. */\n\
             export const HANGLAMPEN: [number, number][] = [\n  {}\n];\n\n\
             /** De wanden van de onzichtbare ruimte van de camera (crates/gedeeld/src/ruimte.json). */\n\
             export const RUIMTE = {{ straal: {:.2}, voorHalveBreedte: {:.2}, voorDiepte: {:.2} }};\n",
            stoelen.join(",\n  "),
            lampen.join(",\n  "),
            maat("straal"),
            maat("voor_halve_breedte"),
            maat("voor_diepte"),
        );
        let pad = std::path::Path::new(&map).join("zaalkaart.ts");
        if std::fs::read_to_string(&pad).ok().as_deref() != Some(tekst.as_str()) {
            std::fs::write(&pad, tekst).unwrap();
        }
    }

    #[test]
    fn vak_1_staat_rechts_in_beeld_vanaf_de_voorzitter() {
        let a = Standpunt::vaste()[0].stand;
        let [x, _] = a.in_beeld(STOELPLEKKEN[0]).unwrap();
        assert!(x > 0.0);
        let [x, _] = a.in_beeld(STOELPLEKKEN[149]).unwrap();
        assert!(x < 0.0);
    }
}
