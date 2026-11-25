//! Graphics: een ingevuld Sjabloon dat de Regie klaarzet, in het Voorbeeld bekijkt en toont.
//!
//! Er staat hooguit één Graphic klaar en één Graphic in de Ether. Een Graphic in de Ether is
//! een kopie: hij verandert nooit vanzelf. Wijzigt de Regie de klaargezette Graphic nadat die
//! getoond is, dan neemt de Ether de wijziging pas over na Bijwerken.

use std::borrow::Cow;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::cijfers::Cijfers;
use crate::persoonskaart::{Extra, MAX_VRIJE_REGEL, Persoonsgegevens, STANDPUNT_ZONDER_STOEL, Wie};
use crate::stemmingen::Stemmingsuitslag;
use crate::uitslagindeling::Zetelstand;

/// Langste kop van een Titelkaart, in tekens.
pub const MAX_KOP: usize = 80;
/// Langste regel onder de kop van een Titelkaart, in tekens.
pub const MAX_REGEL: usize = 120;

/// Een Sjabloon met zijn gegevens ingevuld.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "sjabloon")]
#[ts(export)]
pub enum Graphic {
    /// Een kop en een regel eronder. Kleurt geen Stoelen: de Zaal houdt de kleuren van de
    /// vorige Graphic.
    Titelkaart { kop: String, regel: String },
    /// De uitslag van een Stemming: titel, uitkomst, de voor-tegenbalk en de Fracties, en de
    /// Stoelen in de kleur van hun stem.
    UitslagStemming { uitslag: Stemmingsuitslag },
    /// Een hoofdelijke stemming, met dezelfde gegevens: de Namenlijst, en elke Stoel in de stem
    /// van het Kamerlid dat er zit.
    HoofdelijkeStemming { uitslag: Stemmingsuitslag },
    /// Dezelfde gegevens als de Fractielijst: twee kolommen partijlogo's, voor en tegen.
    FractiesVoorEnTegen { uitslag: Stemmingsuitslag },
    /// De Zetels van één set in de Zaal, in de Uitslagindeling, met labels voor naam en Zetels
    /// aan of uit. In de Zaalindeling toont hij de Kamer die zit: elke Stoel in de kleur van de
    /// Fractie van wie er zit, en `set` telt niet. `cijfers` vult de server bij klaarzetten in;
    /// daarna houdt de Graphic die vast.
    Zetelverdeling {
        set: String,
        namen: bool,
        zetels: bool,
        #[serde(default)]
        indeling: Indeling,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
    /// Eén persoon: een Kamerlid of een eigen persoon, met een vrije regel en de extra gegevens
    /// die aan staan. `gegevens` vult de server bij klaarzetten in; daarna houdt de Graphic die
    /// vast. Kleurt geen Stoelen, maar de Stoel van een Kamerlid licht op
    /// ([`Graphic::uitgelichte_stoel`]).
    Persoonskaart {
        wie: Wie,
        #[serde(default)]
        regel: String,
        #[serde(default)]
        extra: Extra,
        #[serde(default)]
        #[ts(optional)]
        gegevens: Option<Persoonsgegevens>,
    },
    /// Een set als Zetelbalk: alle Partijen of de `grootste` N plus "overig", in de volgorde van
    /// de Uitslagindeling. Kleurt de Stoelen zoals de Zetelverdeling. `cijfers` vult de server
    /// bij klaarzetten in.
    Zetelbalk {
        set: String,
        #[serde(default)]
        #[ts(optional)]
        grootste: Option<u16>,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
    /// Twee sets naast elkaar: twee Zetelbalken onder elkaar met dezelfde Partijen in dezelfde
    /// volgorde, de oudste set boven en de nieuwste onder met een pijltje voor het verschil. De
    /// grootste N komen uit de nieuwste set; de Zaal toont de nieuwste set. Welke set de
    /// nieuwste is, volgt uit de cijfers (de set die later in de lijst staat).
    TweeSets {
        set: String,
        tweede: String,
        #[serde(default)]
        #[ts(optional)]
        grootste: Option<u16>,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
    /// Uitslag per Partij: een set als liggende balken van groot naar klein, alle Partijen of de
    /// grootste N plus "overig". Kleurt de Stoelen zoals de Zetelverdeling.
    UitslagPerPartij {
        set: String,
        #[serde(default)]
        #[ts(optional)]
        grootste: Option<u16>,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
    /// Winst en verlies: per Partij het verschil in Zetels tussen een set en de vergelijkingsset
    /// (tegen de som van haar Voorgangers), van winst naar verlies. `vergelijk_met`: `None` is de
    /// vergelijkingsset van de cijfers (standaard de vorige verkiezing). Kleurt de Stoelen zoals
    /// de Zetelverdeling. `cijfers` vult de server bij klaarzetten in, met de vergelijkingsset
    /// als `vorige`.
    WinstEnVerlies {
        set: String,
        #[serde(default)]
        #[ts(optional)]
        vergelijk_met: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
    /// Coalitie in de Zaal: de gekozen Partijen schuiven in de Uitslagindeling samen naar links
    /// tot één blok, de rest wordt donker; een naambalk met de Partijen en een teller naar 76.
    /// Uitzondering op Bijwerken: elke Partij erbij of eraf werkt direct bij, ook in de Ether
    /// ([`crate::Opdracht::ZetCoalitie`]). `set` mag ook [`crate::uitslagindeling::HUIDIGE_KAMER`]
    /// zijn. `cijfers` vult de server bij klaarzetten in.
    Coalitie {
        set: String,
        /// Sleutels van de gekozen Partijen.
        partijen: Vec<String>,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
    /// Partij uitgelicht: een set en een Partij. Haar Stoelen staan in haar kleur, de andere
    /// Stoelen worden grijs. `cijfers` vult de server bij klaarzetten in.
    PartijUitgelicht {
        set: String,
        partij: String,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
    /// Een Graphic van een eigen Sjabloon (`id`, zie [`crate::sjabloon::EIGEN`]): de invulvelden
    /// die het Sjabloon vraagt, en een set en een Partij als zijn Blokken die nodig hebben.
    /// Kleurt de Stoelen zoals het Sjabloon zegt. `cijfers` vult de server bij klaarzetten in als
    /// er een set is; daarna houdt de Graphic die vast.
    Eigen {
        id: String,
        #[serde(default)]
        velden: BTreeMap<String, String>,
        #[serde(default)]
        #[ts(optional)]
        set: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        partij: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
    /// De Coalitiebalk: een gestapelde balk met de coalitie vooraan en een streep bij 76, de
    /// teller en het tekort. De Stoelen kleuren zoals bij Coalitie in de Zaal.
    Coalitiebalk {
        set: String,
        partijen: Vec<String>,
        #[serde(default)]
        #[ts(optional)]
        cijfers: Option<Zetelstand>,
    },
}

/// Hoe een Zetelverdeling de Stoelen verdeelt.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub enum Indeling {
    /// De Zetels van een set, per Partij één blok van voor tot achter.
    #[default]
    Uitslagindeling,
    /// De Kamer die zit: elk Kamerlid op zijn eigen Stoel.
    Zaalindeling,
}

/// Velden van de Zetelverdeling in de [`crate::sjabloon::Invulling`]: "ja" of leeg.
pub const VELD_NAMEN: &str = "namen";
pub const VELD_ZETELS: &str = "zetels";
/// Velden van de Persoonskaart in de [`crate::sjabloon::Invulling`]: de functie (alleen bij een
/// fractievoorzitter), de regel met extra gegevens en de vrije regel.
pub const VELD_FUNCTIE: &str = "functie";
pub const VELD_GEGEVENS: &str = "gegevens";
pub const VELD_VRIJE_REGEL: &str = "regel";

impl Graphic {
    /// Het vaste Sjabloon van deze Graphic en wat de Graphic erin invult.
    pub fn invulling(&self) -> crate::sjabloon::Invulling {
        let (kop, regel) = match self {
            Graphic::Titelkaart { kop, regel } => (kop.clone(), regel.clone()),
            Graphic::UitslagStemming { uitslag }
            | Graphic::HoofdelijkeStemming { uitslag }
            | Graphic::FractiesVoorEnTegen { uitslag } => {
                let sjabloon = match self {
                    Graphic::HoofdelijkeStemming { .. } => crate::sjabloon::HOOFDELIJKE_STEMMING,
                    Graphic::FractiesVoorEnTegen { .. } => crate::sjabloon::FRACTIES_VOOR_EN_TEGEN,
                    _ => crate::sjabloon::UITSLAG_STEMMING,
                };
                return crate::sjabloon::Invulling {
                    sjabloon: sjabloon.into(),
                    velden: [(crate::sjabloon::VELD_TITEL.into(), uitslag.kop())].into(),
                    stemming: Some(uitslag.clone()),
                    ..Default::default()
                };
            }
            Graphic::Zetelverdeling {
                set,
                namen,
                zetels,
                indeling,
                ..
            } => {
                let set = match indeling {
                    Indeling::Uitslagindeling => set.clone(),
                    Indeling::Zaalindeling => crate::uitslagindeling::ZAALINDELING.into(),
                };
                let ja = |aan: bool| if aan { "ja" } else { "" }.to_string();
                return crate::sjabloon::Invulling {
                    sjabloon: crate::sjabloon::ZETELVERDELING.into(),
                    velden: [
                        (VELD_NAMEN.into(), ja(*namen)),
                        (VELD_ZETELS.into(), ja(*zetels)),
                    ]
                    .into(),
                    set: Some(set),
                    ..Default::default()
                };
            }
            Graphic::Persoonskaart {
                regel,
                extra,
                gegevens,
                ..
            } => {
                let g = gegevens.as_ref();
                return crate::sjabloon::Invulling {
                    sjabloon: "persoonskaart".into(),
                    velden: [
                        (
                            VELD_FUNCTIE.into(),
                            g.and_then(|g| g.functie.clone()).unwrap_or_default(),
                        ),
                        (
                            VELD_GEGEVENS.into(),
                            g.map(|g| g.regel(extra)).unwrap_or_default(),
                        ),
                        (VELD_VRIJE_REGEL.into(), regel.trim().to_owned()),
                    ]
                    .into(),
                    persoon: g.map(|g| crate::sjabloon::Persoon {
                        naam: g.naam.clone(),
                        foto: g.foto.clone(),
                        partij: g.partij.clone(),
                        voorletters: g.voorletters.clone(),
                    }),
                    ..Default::default()
                };
            }
            Graphic::Zetelbalk { set, grootste, .. }
            | Graphic::UitslagPerPartij { set, grootste, .. } => {
                let sjabloon = match self {
                    Graphic::Zetelbalk { .. } => crate::sjabloon::ZETELBALK,
                    _ => crate::sjabloon::UITSLAG_PER_PARTIJ,
                };
                return crate::sjabloon::Invulling {
                    sjabloon: sjabloon.into(),
                    set: Some(set.clone()),
                    grootste: *grootste,
                    ..Default::default()
                };
            }
            Graphic::TweeSets {
                set,
                tweede,
                grootste,
                cijfers,
            } => {
                // de nieuwste set is de set van de kopie; zonder kopie de volgorde van de Regie
                let (set, vorige) = match cijfers {
                    Some(Zetelstand {
                        set: nieuw,
                        vorige: Some(oud),
                        ..
                    }) => (nieuw.id.clone(), oud.id.clone()),
                    _ => (tweede.clone(), set.clone()),
                };
                return crate::sjabloon::Invulling {
                    sjabloon: crate::sjabloon::TWEE_SETS.into(),
                    set: Some(set),
                    vorige_set: Some(vorige),
                    grootste: *grootste,
                    ..Default::default()
                };
            }
            Graphic::WinstEnVerlies {
                set,
                vergelijk_met,
                cijfers,
            } => {
                let vorige = match cijfers {
                    Some(Zetelstand {
                        vorige: Some(v), ..
                    }) => Some(v.id.clone()),
                    _ => vergelijk_met.clone(),
                };
                return crate::sjabloon::Invulling {
                    sjabloon: crate::sjabloon::WINST_EN_VERLIES.into(),
                    set: Some(set.clone()),
                    vorige_set: vorige,
                    ..Default::default()
                };
            }
            Graphic::PartijUitgelicht { set, partij, .. } => {
                return crate::sjabloon::Invulling {
                    sjabloon: crate::sjabloon::PARTIJ_UITGELICHT.into(),
                    set: Some(set.clone()),
                    partij: Some(partij.clone()),
                    ..Default::default()
                };
            }
            Graphic::Eigen {
                id,
                velden,
                set,
                partij,
                cijfers,
            } => {
                return crate::sjabloon::Invulling {
                    sjabloon: id.clone(),
                    velden: velden.clone(),
                    set: cijfers
                        .as_ref()
                        .map(|c| c.set.id.clone())
                        .or_else(|| set.clone()),
                    vorige_set: cijfers
                        .as_ref()
                        .and_then(|c| c.vorige.as_ref())
                        .map(|v| v.id.clone()),
                    partij: partij.clone(),
                    ..Default::default()
                };
            }
            Graphic::Coalitie {
                set,
                partijen,
                cijfers,
            }
            | Graphic::Coalitiebalk {
                set,
                partijen,
                cijfers,
            } => {
                let sjabloon = match self {
                    Graphic::Coalitie { .. } => crate::sjabloon::COALITIE,
                    _ => crate::sjabloon::COALITIEBALK,
                };
                return crate::sjabloon::Invulling {
                    sjabloon: sjabloon.into(),
                    set: Some(
                        cijfers
                            .as_ref()
                            .map_or_else(|| set.clone(), |c| c.set.id.clone()),
                    ),
                    coalitie: partijen.clone(),
                    ..Default::default()
                };
            }
        };
        crate::sjabloon::Invulling {
            sjabloon: "titelkaart".into(),
            velden: [("kop".into(), kop), ("regel".into(), regel)].into(),
            ..Default::default()
        }
    }

    /// De cijfers die deze Graphic vasthoudt, als ze die heeft. De Renderer zoekt er de bronnen
    /// van het Sjabloon in op, in plaats van in de cijfers van de Stand.
    pub fn cijfers(&self) -> Option<Cijfers> {
        self.zetelstand().map(Zetelstand::als_cijfers)
    }

    /// De kopie van de cijfers die deze Graphic vasthoudt.
    pub fn zetelstand(&self) -> Option<&Zetelstand> {
        match self {
            Graphic::Zetelverdeling { cijfers, .. }
            | Graphic::Zetelbalk { cijfers, .. }
            | Graphic::TweeSets { cijfers, .. }
            | Graphic::UitslagPerPartij { cijfers, .. }
            | Graphic::WinstEnVerlies { cijfers, .. }
            | Graphic::PartijUitgelicht { cijfers, .. }
            | Graphic::Coalitie { cijfers, .. }
            | Graphic::Coalitiebalk { cijfers, .. }
            | Graphic::Eigen { cijfers, .. } => cijfers.as_ref(),
            Graphic::Titelkaart { .. }
            | Graphic::UitslagStemming { .. }
            | Graphic::HoofdelijkeStemming { .. }
            | Graphic::FractiesVoorEnTegen { .. }
            | Graphic::Persoonskaart { .. } => None,
        }
    }

    /// De set en de versie waarmee deze Graphic is ingevuld (bij Twee sets: de nieuwste).
    pub fn cijferset(&self) -> Option<(&str, u64)> {
        self.zetelstand().map(|c| (c.set.id.as_str(), c.set.versie))
    }

    /// Alle sets met hun versie waarmee deze Graphic is ingevuld.
    pub fn cijfersets(&self) -> Vec<(&str, u64)> {
        self.zetelstand()
            .into_iter()
            .flat_map(|c| c.vorige.iter().chain([&c.set]))
            .map(|s| (s.id.as_str(), s.versie))
            .collect()
    }

    /// De kleuren van de Stoelen bij deze Graphic. `None`: de Zaal houdt de kleuren van de
    /// vorige Graphic.
    pub fn stoelkleuren(&self) -> Option<Vec<crate::Kleur>> {
        match self {
            Graphic::UitslagStemming { uitslag }
            | Graphic::HoofdelijkeStemming { uitslag }
            | Graphic::FractiesVoorEnTegen { uitslag } => uitslag.stoelkleuren(),
            Graphic::PartijUitgelicht {
                partij, cijfers, ..
            } => cijfers.as_ref().map(|c| c.stoelkleuren_van(partij)),
            Graphic::Coalitie {
                partijen, cijfers, ..
            }
            | Graphic::Coalitiebalk {
                partijen, cijfers, ..
            } => cijfers.as_ref().map(|c| c.coalitiekleuren(partijen)),
            _ => self.zetelstand().map(Zetelstand::stoelkleuren),
        }
    }

    /// De Stoel die oplicht: die van het Kamerlid op een Persoonskaart.
    pub fn uitgelichte_stoel(&self) -> Option<crate::Stoel> {
        match self {
            Graphic::Persoonskaart {
                gegevens: Some(g), ..
            } => g.stoel,
            _ => None,
        }
    }

    /// Het id van de Stemming waar deze Graphic over gaat.
    pub fn stemming(&self) -> Option<&str> {
        match self {
            Graphic::Titelkaart { .. }
            | Graphic::Zetelverdeling { .. }
            | Graphic::Zetelbalk { .. }
            | Graphic::TweeSets { .. }
            | Graphic::UitslagPerPartij { .. }
            | Graphic::Persoonskaart { .. }
            | Graphic::WinstEnVerlies { .. }
            | Graphic::PartijUitgelicht { .. }
            | Graphic::Coalitie { .. }
            | Graphic::Coalitiebalk { .. }
            | Graphic::Eigen { .. } => None,
            Graphic::UitslagStemming { uitslag }
            | Graphic::HoofdelijkeStemming { uitslag }
            | Graphic::FractiesVoorEnTegen { uitslag } => Some(&uitslag.stemming),
        }
    }

    /// Het `id` van het Sjabloon van deze Graphic.
    pub fn sjabloon_id(&self) -> Cow<'_, str> {
        use crate::sjabloon as s;
        Cow::Borrowed(match self {
            Graphic::Titelkaart { .. } => "titelkaart",
            Graphic::UitslagStemming { .. } => s::UITSLAG_STEMMING,
            Graphic::HoofdelijkeStemming { .. } => s::HOOFDELIJKE_STEMMING,
            Graphic::FractiesVoorEnTegen { .. } => s::FRACTIES_VOOR_EN_TEGEN,
            Graphic::Zetelverdeling { .. } => s::ZETELVERDELING,
            Graphic::Persoonskaart { .. } => "persoonskaart",
            Graphic::Zetelbalk { .. } => s::ZETELBALK,
            Graphic::TweeSets { .. } => s::TWEE_SETS,
            Graphic::UitslagPerPartij { .. } => s::UITSLAG_PER_PARTIJ,
            Graphic::WinstEnVerlies { .. } => s::WINST_EN_VERLIES,
            Graphic::PartijUitgelicht { .. } => s::PARTIJ_UITGELICHT,
            Graphic::Coalitie { .. } => s::COALITIE,
            Graphic::Coalitiebalk { .. } => s::COALITIEBALK,
            Graphic::Eigen { id, .. } => return Cow::Borrowed(id),
        })
    }

    /// De gekozen Partij van deze Graphic, als ze er een heeft.
    pub fn partij(&self) -> Option<&str> {
        match self {
            Graphic::PartijUitgelicht { partij, .. } => Some(partij),
            Graphic::Eigen { partij, .. } => partij.as_deref(),
            _ => None,
        }
    }

    /// De kleuren van de Stoelen bij deze Graphic in een Sjabloon dat zegt wat de Stoelen doen.
    /// `None`: de Zaal houdt de kleuren van de vorige Graphic.
    pub fn stoelkleuren_in(&self, stoelen: crate::sjabloon::Stoelen) -> Option<Vec<crate::Kleur>> {
        use crate::sjabloon::Stoelen;
        match stoelen {
            Stoelen::Gegevens => self.stoelkleuren(),
            Stoelen::Houden => None,
            Stoelen::Uit => Some(Vec::new()),
            Stoelen::PartijUitgelicht => {
                let (partij, cijfers) = (self.partij()?, self.zetelstand()?);
                Some(cijfers.stoelkleuren_van(partij))
            }
            Stoelen::StoelUitgelicht => Some(vec![crate::sjabloon::GRIJS; crate::AANTAL_STOELEN]),
        }
    }

    /// De sleutel van Aanschuiven naar het blok van de gekozen Partij, als de Graphic een Partij
    /// en Zetels heeft ([`crate::camera::aanschuifsleutel`]).
    pub fn aanschuifsleutel(&self) -> Option<String> {
        let (van, tot) = self.zetelstand()?.plaatsen_van(self.partij()?)?;
        Some(crate::camera::aanschuifsleutel(van, tot))
    }

    /// Het Standpunt bij een keuze: Aanschuiven wordt het blok van de gekozen Partij, of zonder
    /// Partij het Standpunt van de soort Graphic. Elke andere keuze blijft wat hij is.
    pub fn standpunt_bij<'a>(&self, keuze: Option<&'a str>) -> Cow<'a, str> {
        match keuze {
            Some(crate::camera::AANSCHUIVEN) => match self.aanschuifsleutel() {
                Some(s) => Cow::Owned(s),
                None => Cow::Owned(self.standaard_standpunt().into_owned()),
            },
            Some(l) => Cow::Borrowed(l),
            None => Cow::Owned(self.standaard_standpunt().into_owned()),
        }
    }

    /// De letter van het standaard Standpunt van het Sjabloon. Een Graphic kan een ander kiezen
    /// (zie [`KlaargezetteGraphic::standpunt`]). Een Persoonskaart van een Kamerlid met een Stoel
    /// kiest het shot van die Stoel ([`crate::camera::stoelsleutel`]).
    pub fn standaard_standpunt(&self) -> Cow<'static, str> {
        match self {
            Graphic::Titelkaart { .. }
            | Graphic::UitslagStemming { .. }
            | Graphic::Zetelverdeling { .. }
            | Graphic::Zetelbalk { .. }
            | Graphic::TweeSets { .. }
            | Graphic::UitslagPerPartij { .. }
            | Graphic::WinstEnVerlies { .. }
            | Graphic::Coalitie { .. }
            | Graphic::Coalitiebalk { .. }
            | Graphic::Eigen { .. } => Cow::Borrowed("A"),
            // Bovenaf: alle Stoelen met hun eigen kleur, en het paneel dekt het beeld bijna.
            Graphic::HoofdelijkeStemming { .. } | Graphic::FractiesVoorEnTegen { .. } => {
                Cow::Borrowed("G")
            }
            // langzaam naar het blok van de gekozen Partij
            Graphic::PartijUitgelicht { .. } => match self.aanschuifsleutel() {
                Some(s) => Cow::Owned(s),
                None => Cow::Borrowed("A"),
            },
            Graphic::Persoonskaart { .. } => match self.uitgelichte_stoel() {
                Some(stoel) => Cow::Owned(crate::camera::stoelsleutel(stoel)),
                None => Cow::Borrowed(STANDPUNT_ZONDER_STOEL),
            },
        }
    }

    /// Weigert een Graphic die de Renderer niet goed kan tonen.
    pub fn controleer(&self) -> Result<(), crate::Weigering> {
        use crate::Weigering;
        match self {
            Graphic::Titelkaart { kop, regel } => {
                if kop.trim().is_empty() {
                    return Err(Weigering::KopLeeg);
                }
                if kop.chars().count() > MAX_KOP {
                    return Err(Weigering::KopTeLang);
                }
                if regel.chars().count() > MAX_REGEL {
                    return Err(Weigering::RegelTeLang);
                }
            }
            Graphic::UitslagStemming { uitslag }
            | Graphic::HoofdelijkeStemming { uitslag }
            | Graphic::FractiesVoorEnTegen { uitslag } => {
                if uitslag.titel.trim().is_empty() {
                    return Err(Weigering::KopLeeg);
                }
            }
            // de set controleert de Stand bij het invullen van de cijfers
            Graphic::Zetelverdeling { .. }
            | Graphic::Zetelbalk { .. }
            | Graphic::UitslagPerPartij { .. }
            | Graphic::WinstEnVerlies {
                vergelijk_met: None,
                ..
            }
            | Graphic::PartijUitgelicht { .. }
            | Graphic::Coalitie { .. }
            | Graphic::Coalitiebalk { .. } => {}
            // de persoon controleert de Stand bij het invullen van de gegevens
            Graphic::Persoonskaart { regel, .. } => {
                if regel.chars().count() > MAX_VRIJE_REGEL {
                    return Err(Weigering::RegelTeLang);
                }
            }
            Graphic::Eigen { id, velden, .. } => {
                if !crate::sjabloon::is_eigen_id(id) {
                    return Err(Weigering::OnbekendSjabloon(id.clone()));
                }
                if velden.values().any(|v| v.chars().count() > MAX_REGEL) {
                    return Err(Weigering::RegelTeLang);
                }
            }
            Graphic::WinstEnVerlies {
                set,
                vergelijk_met: Some(tweede),
                ..
            }
            | Graphic::TweeSets { set, tweede, .. } => {
                if set == tweede {
                    return Err(Weigering::TweeKeerDezelfdeSet);
                }
            }
        }
        Ok(())
    }
}

/// De Graphic die klaarstaat en in het Voorbeeld te zien is.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct KlaargezetteGraphic {
    /// Blijft gelijk zolang de Regie deze Graphic wijzigt; een nieuwe Graphic krijgt een nieuw id.
    #[ts(type = "number")]
    pub id: u64,
    /// Verandert bij elke wijziging. Toon stuurt mee welke versie het scherm liet zien.
    #[ts(type = "number")]
    pub versie: u64,
    pub graphic: Graphic,
    /// Een ander Standpunt dan dat van het Sjabloon, als letter.
    #[serde(default)]
    #[ts(optional)]
    pub standpunt: Option<String>,
    /// Het standaard Standpunt van een bewerkt Sjabloon ([`crate::sjabloon::Sjabloon::standpunt`]).
    /// De Stand vult dit in bij klaarzetten en als het Sjabloon verandert.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub standpunt_van_sjabloon: Option<String>,
    /// De Werkplek die deze Graphic klaarzette.
    pub klaargezet_door: String,
    /// De Werkplek die de laatste versie maakte.
    pub gewijzigd_door: String,
}

/// De Graphic in de Ether: een kopie van de klaargezette Graphic op het moment van tonen.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct GraphicInBeeld {
    /// Het id van de klaargezette Graphic waar deze kopie van komt.
    #[ts(type = "number")]
    pub id: u64,
    /// De versie van die Graphic in deze kopie.
    #[ts(type = "number")]
    pub versie: u64,
    pub graphic: Graphic,
    /// Een ander Standpunt dan dat van het Sjabloon, als letter.
    #[serde(default)]
    #[ts(optional)]
    pub standpunt: Option<String>,
    /// Het volgnummer van de Toon. Een nieuwe Toon is altijd een wissel, ook van dezelfde Graphic.
    #[ts(type = "number")]
    pub getoond: u64,
    /// Sinds wanneer, in milliseconden sinds 1970 (UTC).
    #[ts(type = "number")]
    pub sinds_ms: u64,
    pub getoond_door: String,
    /// De Stijl op het moment van Toon. De Graphic houdt die vorm tot de volgende Toon, ook bij
    /// Bijwerken; een nieuwe Stijl geldt alleen voor wat klaarstaat.
    #[serde(default)]
    pub stijl: crate::sjabloon::Stijl,
    /// Het Sjabloon op het moment van Toon, net als de Stijl: de Graphic houdt die vorm tot de
    /// volgende Toon. Zonder (een Stand van vóór #62): het vaste Sjabloon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub sjabloon: Option<crate::sjabloon::Sjabloon>,
}

impl KlaargezetteGraphic {
    /// De letter van het Standpunt: de keuze voor deze Graphic, anders die van het Sjabloon.
    pub fn standpunt(&self) -> Cow<'_, str> {
        self.graphic.standpunt_bij(
            self.standpunt
                .as_deref()
                .or(self.standpunt_van_sjabloon.as_deref()),
        )
    }
}

impl GraphicInBeeld {
    /// De letter van het Standpunt: de keuze voor deze Graphic, anders die van het Sjabloon.
    pub fn standpunt(&self) -> Cow<'_, str> {
        let van_sjabloon = self.sjabloon.as_ref().and_then(|s| s.standpunt.as_deref());
        self.graphic
            .standpunt_bij(self.standpunt.as_deref().or(van_sjabloon))
    }

    /// De kleuren van de Stoelen zoals het Sjabloon van deze Graphic zegt. `None`: de Zaal houdt
    /// de kleuren van de vorige Graphic.
    pub fn stoelkleuren(&self) -> Option<Vec<crate::Kleur>> {
        self.graphic.stoelkleuren_in(
            self.sjabloon
                .as_ref()
                .map(|s| s.stoelen)
                .unwrap_or_default(),
        )
    }

    /// De Kleurovergang van het Sjabloon van deze Graphic, of anders die van haar Stijl.
    pub fn kleurovergang(&self) -> crate::sjabloon::Kleurovergang {
        self.sjabloon
            .as_ref()
            .and_then(|s| s.kleurovergang)
            .unwrap_or(self.stijl.kleurovergang)
    }
}

/// Hoe de Ether van de ene Graphic naar de volgende gaat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overgang {
    /// Niets veranderd.
    Geen,
    /// Er was niets; de Graphic komt in beeld.
    Erin,
    /// De oude Graphic animeert weg, daarna komt de nieuwe in beeld.
    Wissel,
    /// Dezelfde Graphic met nieuwere gegevens: alles beweegt vloeiend naar de nieuwe stand.
    Bijwerken,
    /// De Graphic gaat uit beeld.
    Eruit,
}

impl Overgang {
    /// De overgang tussen de Graphic in de vorige en in de huidige Stand.
    pub fn tussen(vorige: Option<&GraphicInBeeld>, huidig: Option<&GraphicInBeeld>) -> Overgang {
        match (vorige, huidig) {
            (None, None) => Overgang::Geen,
            (None, Some(_)) => Overgang::Erin,
            (Some(_), None) => Overgang::Eruit,
            (Some(v), Some(h)) if v.getoond != h.getoond => Overgang::Wissel,
            (Some(v), Some(h)) if v.versie != h.versie || v.graphic != h.graphic => {
                Overgang::Bijwerken
            }
            (Some(_), Some(_)) => Overgang::Geen,
        }
    }
}
