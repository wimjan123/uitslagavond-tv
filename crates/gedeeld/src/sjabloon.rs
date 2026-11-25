//! Sjablonen, Blokken en de Stijl (ADR 0003).
//!
//! Een Sjabloon is data: een lijst Blokken met elk een plek, een soort, een kleur en een
//! animatie. De Renderer tekent alleen Blokken. Een vast Sjabloon komt mee met de software
//! ([`vaste_sjablonen`]) en bestaat uit dezelfde Blokken als een eigen Sjabloon.
//!
//! Een Blok kiest zijn kleur en lettertype uit de [`Stijl`] ([`Kleurkeuze::Stijl`]), of krijgt
//! een eigen kleur of lettertype en volgt de Stijl dan niet meer. Vaste Sjablonen gebruiken
//! alleen de Stijl.
//!
//! [`Sjabloon::vul_in`] maakt van een Sjabloon, de [`Invulling`] van een Graphic, de cijfers
//! en de Stijl de [`IngevuldBlok`]ken die de Renderer tekent: elke bron is dan opgezocht.
//!
//! Een nieuw Blok: een variant in [`Bloksoort`] en in [`Inhoud`], en de tekening in de
//! Renderer (`crates/renderer/src/blokken.rs`). Een nieuw vast Sjabloon: een functie die het
//! Sjabloon bouwt, in [`vaste_sjablonen`], en een variant in [`crate::Graphic`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Kleur;
use crate::cijfers::{Cijfers, OVERIG, ZETELS};
use crate::stemmingen::{Besluit, Stem};
use crate::coalitie;
use crate::zetelbalk::{self, Balkdeel, Pijl};

/// Het ingebouwde vette lettertype. Elke Renderer heeft de lettertypes van [`INGEBOUWD`] zelf;
/// andere lettertypes haalt hij bij de server.
pub const LETTER_VET: &str = "DejaVuSans-Bold.ttf";
/// Het ingebouwde gewone lettertype.
pub const LETTER_GEWOON: &str = "DejaVuSans.ttf";
/// De tekstletter van de standaardstijl: Nunito Sans SemiBold.
pub const LETTER_TEKST: &str = "NunitoSans-SemiBold.ttf";
/// De cijferletter van de standaardstijl: Nunito Sans ExtraBold, met cijfers die even breed zijn.
pub const LETTER_CIJFERS: &str = "NunitoSans-ExtraBold.ttf";
/// De kopletter van de standaardstijl: dezelfde Nunito Sans ExtraBold.
pub const LETTER_KOP: &str = LETTER_CIJFERS;
/// De lettertypes die in elke Renderer zitten (SIL OFL en de DejaVu-licentie).
pub const INGEBOUWD: [&str; 4] = [LETTER_VET, LETTER_GEWOON, LETTER_TEKST, LETTER_CIJFERS];
/// De bestandsnaam van het programmalogo in de standaardstijl, in de map `bestanden` van de
/// server. Het logo zit niet in de software: de omroep legt het daar neer.
pub const PROGRAMMALOGO: &str = "programmalogo.png";

/// De kleuren, lettertypes, het programmalogo en de snelheid van het tv-programma. Er is één
/// Stijl.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Stijl {
    pub kleuren: Stijlkleuren,
    pub lettertypes: Stijlletters,
    pub programmalogo: Programmalogo,
    /// Snelheid van alle animaties, in procent: 100 is normaal, 70 langzaam, 140 snel. Geldt
    /// ook voor de kleurgolf in de Zaal.
    pub snelheid: u16,
    /// Tijden en maten van de animaties van de Blokken. Een Stand van vóór de Beweging krijgt
    /// de standaard.
    #[serde(default)]
    pub beweging: Beweging,
    /// Hoe de Stoelen van kleur wisselen. Een Sjabloon kan een eigen Kleurovergang hebben.
    #[serde(default)]
    pub kleurovergang: Kleurovergang,
    /// De Kleurkracht in procent: 0 toont de kleur zoals het leer hem in de Zaal geeft, 100 de
    /// volste kleur met de rest van de Zaal het donkerst. Een Stand van vóór de Kleurkracht
    /// krijgt de standaard.
    #[serde(default = "standaard_kleurkracht")]
    pub kleurkracht: u8,
}

/// De Kleurkracht van de standaardstijl, in procent.
pub const KLEURKRACHT: u8 = 50;

fn standaard_kleurkracht() -> u8 {
    KLEURKRACHT
}

/// Hoe de Stoelen van kleur wisselen (de Kleurovergang), bij snelheid 100. De snelheid van de
/// Stijl maakt hem korter of langer.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Kleurovergang {
    pub vorm: Overgangsvorm,
    /// Hoe lang de hele overgang duurt, in milliseconden; bij de Opbouw de tijd per Partij.
    pub duur_ms: u16,
}

impl Default for Kleurovergang {
    fn default() -> Self {
        Kleurovergang::standaard(Overgangsvorm::Golf)
    }
}

impl Kleurovergang {
    /// De standaard van een vorm: de Golf duurt een seconde, Vervagen anderhalve, de Opbouw
    /// anderhalve seconde per Partij.
    pub fn standaard(vorm: Overgangsvorm) -> Self {
        let duur_ms = match vorm {
            Overgangsvorm::Golf => 1000,
            Overgangsvorm::Vervagen | Overgangsvorm::Opbouw => 1500,
        };
        Kleurovergang { vorm, duur_ms }
    }

    /// De Kleurovergang bij Bijwerken: de Opbouw is voor Toon, bij nieuwe cijfers gaan de
    /// Stoelen die veranderen als Golf.
    pub fn bij_bijwerken(self) -> Self {
        if self.vorm == Overgangsvorm::Opbouw { Kleurovergang::default() } else { self }
    }

    /// Of de duur tussen [`MIN_KLEUROVERGANG_MS`] en [`MAX_KLEUROVERGANG_MS`] ligt.
    pub fn kan(&self) -> bool {
        (MIN_KLEUROVERGANG_MS..=MAX_KLEUROVERGANG_MS).contains(&self.duur_ms)
    }
}

/// De vorm van een Kleurovergang.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub enum Overgangsvorm {
    /// Van links naar rechts in beeld: elke Stoel vloeit over als de golf hem bereikt.
    #[default]
    Golf,
    /// Alle Stoelen tegelijk.
    Vervagen,
    /// Eén Partij tegelijk, de grootste eerst: de Stoelen worden eerst donker, dan licht elk
    /// blok op in zijn kleur ([`Bloksoort::Opbouwlabel`] toont intussen de Partij). Stoelen met
    /// dezelfde kleur lichten samen op. Alleen voor een Sjabloon, niet in de Stijl.
    Opbouw,
}

/// De kleur van de Stoelen die in een Opbouw nog niet aan de beurt zijn.
pub const OPBOUW_WACHTKLEUR: Kleur = Kleur { r: 0x30, g: 0x30, b: 0x30 };

/// De kortste en langste Kleurovergang, in milliseconden.
pub const MIN_KLEUROVERGANG_MS: u16 = 100;
pub const MAX_KLEUROVERGANG_MS: u16 = 10_000;

/// De tijden en maten van de animaties van de Blokken, bij snelheid 100. Welke animatie een
/// Blok krijgt, kiest het Sjabloon ([`Animatie`]); hoe die animatie loopt, staat hier.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Beweging {
    /// Hoe lang een Blok erover doet om in beeld te komen, in milliseconden. Weggaan duurt 70
    /// procent daarvan, Bijwerken en Optellen 140 procent.
    pub duur_ms: u16,
    /// Hoe ver een Blok invliegt of wegvliegt, in pixels van het 1080p-beeld.
    pub vlieg_px: u16,
    /// Bij invliegen is het Blok na dit deel van de beweging (in procent) helemaal zichtbaar;
    /// daarna schuift het nog.
    pub zichtbaar_na_pct: u8,
    /// Openen vanuit lijn en Sluiten naar lijn.
    pub lijn: Lijnbeweging,
}

/// Openen vanuit lijn: een lijn verschijnt, het vlak gaat open tot zijn volle hoogte en de
/// tekst vervaagt erin. Sluiten naar lijn gaat omgekeerd.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Lijnbeweging {
    /// Hoe lang de lijn erover doet om te verschijnen, in milliseconden.
    pub verschijnt_ms: u16,
    /// Hoe lang het vlak erover doet om open te gaan, in milliseconden.
    pub opent_ms: u16,
    /// Hoe lang de tekst erover doet om erin te vervagen, in milliseconden.
    pub tekst_ms: u16,
    /// Hoe dik de lijn is, in pixels van het 1080p-beeld.
    pub dikte_px: u8,
    /// Bij Sluiten naar lijn: tot dit deel van het weggaan (in procent) vervaagt de tekst.
    pub tekst_weg_pct: u8,
    /// Bij Sluiten naar lijn: tot dit deel van het weggaan (in procent) gaat het vlak dicht;
    /// daarna verdwijnt de lijn.
    pub vlak_dicht_pct: u8,
}

impl Default for Beweging {
    fn default() -> Self {
        Beweging {
            duur_ms: 500,
            vlieg_px: 120,
            zichtbaar_na_pct: 40,
            lijn: Lijnbeweging {
                verschijnt_ms: 80,
                opent_ms: 260,
                tekst_ms: 200,
                dikte_px: 2,
                tekst_weg_pct: 30,
                vlak_dicht_pct: 80,
            },
        }
    }
}

/// De kortste en langste duur van de Beweging, in milliseconden.
pub const MIN_DUUR_MS: u16 = 100;
pub const MAX_DUUR_MS: u16 = 5000;

/// De vijf kleuren van de Stijl.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Stijlkleuren {
    #[ts(type = "string")]
    pub hoofd: Kleur,
    #[ts(type = "string")]
    pub accent: Kleur,
    #[ts(type = "string")]
    pub tekst: Kleur,
    #[ts(type = "string")]
    pub vlak: Kleur,
    #[ts(type = "string")]
    pub achtergrond: Kleur,
}

/// De drie lettertypes van de Stijl, als bestandsnaam bij de server (of een ingebouwd
/// lettertype, zie [`LETTER_VET`]).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Stijlletters {
    pub kop: String,
    pub tekst: String,
    pub cijfers: String,
}

/// Het programmalogo: een beeld bij de server, of zolang dat er niet is een tekst.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Programmalogo {
    pub bestand: Option<String>,
    pub tekst: String,
}

impl Default for Stijl {
    /// De standaardstijl "Grafiet": geel en framboos op grafiet, zie `docs/vormgeving.md`.
    fn default() -> Self {
        let k = |hex: &str| hex.parse().expect("vaste kleur");
        Stijl {
            kleuren: Stijlkleuren {
                hoofd: k("#ffd166"),
                accent: k("#ef476f"),
                tekst: k("#ffffff"),
                vlak: k("#1c1f26"),
                achtergrond: k("#3a3f4b"),
            },
            lettertypes: Stijlletters {
                kop: LETTER_KOP.into(),
                tekst: LETTER_TEKST.into(),
                cijfers: LETTER_CIJFERS.into(),
            },
            programmalogo: Programmalogo {
                bestand: Some(PROGRAMMALOGO.into()),
                tekst: "Uitslagavond".into(),
            },
            snelheid: 100,
            beweging: Beweging::default(),
            kleurovergang: Kleurovergang::default(),
            kleurkracht: KLEURKRACHT,
        }
    }
}

/// De langzaamste en snelste snelheid van de Stijl, in procent.
pub const MIN_SNELHEID: u16 = 25;
pub const MAX_SNELHEID: u16 = 400;
/// Langste tekst van het programmalogo zonder beeld, in tekens.
pub const MAX_LOGOTEKST: usize = 40;

/// Waarom de server een Stijl niet opslaat.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum StijlWeigering {
    #[error("de snelheid is {0} procent; kies tussen {MIN_SNELHEID} en {MAX_SNELHEID}")]
    Snelheid(u16),
    #[error("geen geldige bestandsnaam: {0:?}")]
    Bestandsnaam(String),
    #[error("de tekst van het programmalogo is langer dan {MAX_LOGOTEKST} tekens")]
    LogotekstTeLang,
    #[error("de beweging klopt niet: {0}")]
    Beweging(&'static str),
    #[error("de Kleurovergang duurt {0} ms; kies tussen {MIN_KLEUROVERGANG_MS} en {MAX_KLEUROVERGANG_MS}")]
    Kleurovergang(u16),
    #[error("de Kleurkracht is {0} procent; kies tussen 0 en 100")]
    Kleurkracht(u8),
    #[error("de Opbouw kan alleen in een Sjabloon, niet in de Stijl")]
    OpbouwInStijl,
}

/// De eerste standaardstijl (tot 22 november 2026), zoals een oude stand.json hem bewaarde.
fn oude_standaard() -> Stijl {
    let k = |hex: &str| hex.parse().expect("vaste kleur");
    Stijl {
        kleuren: Stijlkleuren {
            hoofd: k("#12357a"),
            accent: k("#6f9fe0"),
            tekst: k("#ffffff"),
            vlak: k("#080f29"),
            achtergrond: k("#0d1220"),
        },
        lettertypes: Stijlletters {
            kop: LETTER_VET.into(),
            tekst: LETTER_GEWOON.into(),
            cijfers: LETTER_VET.into(),
        },
        programmalogo: Programmalogo { bestand: None, tekst: "UITSLAGAVOND".into() },
        snelheid: 100,
        beweging: Beweging::default(),
        kleurovergang: Kleurovergang::default(),
        kleurkracht: KLEURKRACHT,
    }
}

/// Een van de vijf kleuren van de Stijl.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[ts(export)]
pub enum Stijlkleur {
    Hoofd,
    Accent,
    Tekst,
    Vlak,
    Achtergrond,
}

/// Een van de drie lettertypes van de Stijl.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[ts(export)]
pub enum Stijlletter {
    Kop,
    Tekst,
    Cijfers,
}

impl Stijl {
    pub fn kleur(&self, k: Stijlkleur) -> Kleur {
        let c = &self.kleuren;
        match k {
            Stijlkleur::Hoofd => c.hoofd,
            Stijlkleur::Accent => c.accent,
            Stijlkleur::Tekst => c.tekst,
            Stijlkleur::Vlak => c.vlak,
            Stijlkleur::Achtergrond => c.achtergrond,
        }
    }

    pub fn letter(&self, l: Stijlletter) -> &str {
        let t = &self.lettertypes;
        match l {
            Stijlletter::Kop => &t.kop,
            Stijlletter::Tekst => &t.tekst,
            Stijlletter::Cijfers => &t.cijfers,
        }
    }

    /// Of deze Stijl kan: een snelheid tussen [`MIN_SNELHEID`] en [`MAX_SNELHEID`], geldige
    /// bestandsnamen en een tekst voor het programmalogo die niet te lang is.
    pub fn controleer(&self) -> Result<(), StijlWeigering> {
        if !(MIN_SNELHEID..=MAX_SNELHEID).contains(&self.snelheid) {
            return Err(StijlWeigering::Snelheid(self.snelheid));
        }
        if let Some(naam) = self.bestanden().find(|n| !geldige_bestandsnaam(n)) {
            return Err(StijlWeigering::Bestandsnaam(naam.into()));
        }
        if self.programmalogo.tekst.chars().count() > MAX_LOGOTEKST {
            return Err(StijlWeigering::LogotekstTeLang);
        }
        if !self.kleurovergang.kan() {
            return Err(StijlWeigering::Kleurovergang(self.kleurovergang.duur_ms));
        }
        if self.kleurovergang.vorm == Overgangsvorm::Opbouw {
            return Err(StijlWeigering::OpbouwInStijl);
        }
        if self.kleurkracht > 100 {
            return Err(StijlWeigering::Kleurkracht(self.kleurkracht));
        }
        self.beweging.controleer().map_err(StijlWeigering::Beweging)
    }

    /// De Stijl na het laden van een bewaarde Stand. Een Stand van vóór 22 november 2026 had de
    /// eerste standaardstijl: die wordt de huidige standaard.
    /// Een Stijl die de Regie zelf aanpaste, blijft zoals hij is.
    pub fn na_laden(self) -> Stijl {
        if self == oude_standaard() { Stijl::default() } else { self }
    }

    /// Alle bestanden die deze Stijl bij de server nodig heeft.
    pub fn bestanden(&self) -> impl Iterator<Item = &str> {
        let t = &self.lettertypes;
        [t.kop.as_str(), t.tekst.as_str(), t.cijfers.as_str()]
            .into_iter()
            .chain(self.programmalogo.bestand.as_deref())
    }
}

impl Beweging {
    /// Of deze Beweging kan: een duur tussen [`MIN_DUUR_MS`] en [`MAX_DUUR_MS`], een lijn die
    /// opent, en bij Sluiten naar lijn eerst de tekst weg, dan het vlak dicht, dan de lijn weg.
    pub fn controleer(&self) -> Result<(), &'static str> {
        let l = &self.lijn;
        if !(MIN_DUUR_MS..=MAX_DUUR_MS).contains(&self.duur_ms) {
            return Err("de duur moet tussen 100 en 5000 milliseconden liggen");
        }
        if !(1..=100).contains(&self.zichtbaar_na_pct) {
            return Err("zichtbaar na moet tussen 1 en 100 procent liggen");
        }
        if l.verschijnt_ms == 0 || l.opent_ms == 0 {
            return Err("de lijn moet verschijnen en het vlak moet opengaan");
        }
        if !(l.tekst_weg_pct < l.vlak_dicht_pct && l.vlak_dicht_pct < 100) {
            return Err("bij sluiten naar lijn: eerst de tekst weg, dan het vlak dicht, dan de lijn weg");
        }
        Ok(())
    }
}

/// De kleur van een Blok: uit de Stijl, of een eigen kleur die de Stijl niet meer volgt.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Kleurkeuze {
    Stijl(Stijlkleur),
    Eigen(#[ts(type = "string")] Kleur),
}

/// Het lettertype van een Blok: uit de Stijl, of een eigen lettertype (bestandsnaam).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Letterkeuze {
    Stijl(Stijlletter),
    Eigen(String),
}

impl Kleurkeuze {
    pub fn in_stijl(&self, stijl: &Stijl) -> Kleur {
        match self {
            Kleurkeuze::Stijl(k) => stijl.kleur(*k),
            Kleurkeuze::Eigen(k) => *k,
        }
    }
}

impl Letterkeuze {
    pub fn in_stijl(&self, stijl: &Stijl) -> String {
        match self {
            Letterkeuze::Stijl(l) => stijl.letter(*l).to_string(),
            Letterkeuze::Eigen(b) => b.clone(),
        }
    }
}

/// Plek en grootte van een Blok in pixels van het 1080p-beeld (1920 bij 1080), vanaf linksboven.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Plek {
    pub x: i32,
    pub y: i32,
    pub breedte: u32,
    pub hoogte: u32,
}

/// Een onderdeel van een Sjabloon met een eigen plek, grootte, bron en animatie.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Blok {
    pub plek: Plek,
    pub soort: Bloksoort,
    /// De kleur van het vlak, de lijn of de letters. Een Partijlogo gebruikt de kleur van de
    /// Partij; deze kleur is dan die van de korte naam.
    pub kleur: Kleurkeuze,
    pub animatie: Animatie,
}

fn honderd() -> u8 {
    100
}

fn letter_vet() -> Letterkeuze {
    Letterkeuze::Eigen(LETTER_VET.into())
}

fn tekstletter() -> Letterkeuze {
    Letterkeuze::Stijl(Stijlletter::Tekst)
}

/// De acht gewone Blokken. De grote Blokken (zoals de Winst-en-verliesgrafiek) komen met hun
/// Sjabloon.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "blok")]
#[ts(export)]
pub enum Bloksoort {
    /// Een gekleurd vlak. `hoek` is de ronding in pixels, `dekking` die van de kleur van het
    /// Blok in procent.
    Vlak {
        hoek: u16,
        dekking: u8,
        /// Een kleurverloop van de kleur van het Blok naar een tweede kleur.
        #[serde(default)]
        #[ts(optional)]
        verloop: Option<Verloop>,
        /// Draaiing om het midden, in graden met de klok mee.
        #[serde(default)]
        draaiing: i16,
        /// Een rand binnen het vlak.
        #[serde(default)]
        #[ts(optional)]
        rand: Option<Rand>,
        /// Het vlak is zo breed als de tekst van deze Blokken (een naambalk).
        #[serde(default)]
        #[ts(optional)]
        om_tekst: Option<OmTekst>,
        /// Alleen te zien als de tekst van dit Blok (zijn plaats in het Sjabloon) niet leeg is,
        /// zoals het streepje voor de vrije regel van de Persoonskaart.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        bij_tekst: Option<u16>,
    },
    /// Vaste of ingevulde tekst. De tekst loopt door op de volgende regel als hij niet past,
    /// behalve met `een_regel`: dan wordt de letter kleiner tot hij past.
    Tekst {
        bron: Tekstbron,
        letter: Letterkeuze,
        grootte: u16,
        uitlijning: Uitlijning,
        #[serde(default)]
        een_regel: bool,
    },
    /// Een getal dat bij Bijwerken naar de nieuwe stand telt.
    Getal {
        bron: Getalbron,
        letter: Letterkeuze,
        grootte: u16,
        uitlijning: Uitlijning,
    },
    /// Alle partijen van de gekozen set in één balk, of de grootste N plus "overig" (zie
    /// [`crate::zetelbalk`]), in de volgorde van de cijfers. De kleur van het Blok is die van de
    /// lijn bij 76; Zetels zonder Partij staan in Tekst op [`RESTDEKKING`] procent, overig in Hoofd op
    /// 45 procent.
    Zetelbalk {
        letter: Letterkeuze,
        lijn_bij_76: bool,
        /// De korte namen onder de balk, in Hoofd.
        #[serde(default = "tekstletter")]
        naamletter: Letterkeuze,
        /// Onder elk deel de Zetels (in `letter` en Tekst) en de korte naam, als ze passen.
        #[serde(default)]
        labels: bool,
        /// Bij Twee sets naast elkaar: de vorige set in plaats van de gekozen set.
        #[serde(default)]
        vorige_set: bool,
        /// Bij elk label een pijltje voor het verschil met de vorige set.
        #[serde(default)]
        pijlen: bool,
        /// Coalitiebalk: de Partijen van de Coalitiebouwer vooraan in hun partijkleur, de rest
        /// erachter in Tekst op [`RESTDEKKING`] procent zonder label.
        #[serde(default)]
        coalitie: bool,
    },
    /// Het logo van een partij; zonder logo de korte naam in de partijkleur.
    Partijlogo { bron: Partijbron, hoek: u16 },
    /// De foto van het gekozen Kamerlid (of een andere persoon op de Persoonskaart). Zonder foto
    /// een vlak in de partijkleur met de voorletters in `letter` en de kleur van het Blok.
    FotoKamerlid {
        hoek: u16,
        #[serde(default = "letter_vet")]
        letter: Letterkeuze,
    },
    /// Het Partijlogo, of zonder logo de korte naam in `letter` op een vlak in de partijkleur dat
    /// om de naam past, met een tekst erachter in `tekstletter` en de kleur van het Blok (de
    /// functie op de Persoonskaart). Zonder Partij alleen de tekst.
    Partijregel {
        bron: Partijbron,
        letter: Letterkeuze,
        grootte: u16,
        tekst: Tekstbron,
        tekstletter: Letterkeuze,
        tekstgrootte: u16,
    },
    /// Een streep, zoals de lijn bij 76. `dekking` in procent.
    Lijn {
        #[serde(default = "honderd")]
        dekking: u8,
    },
    /// Het programmalogo uit de Stijl, met `dekking` in procent. Zonder beeld: de tekst van het
    /// programmalogo in `letter`.
    Programmalogo {
        letter: Letterkeuze,
        #[serde(default = "honderd")]
        dekking: u8,
    },
    /// Een afbeelding bij de server (PNG of JPEG), met `dekking` in procent. Alleen voor de
    /// achtergrond van een Sjabloon ([`Achtergrond::Afbeelding`]), niet in de lijst van Blokken.
    Afbeelding {
        bestand: String,
        #[serde(default = "honderd")]
        dekking: u8,
    },
    /// Groot Blok van de Zetelverdeling: per Partij van de gekozen set een label met de korte
    /// naam en de Zetels, in de volgorde van de cijfers (bij een Zetelverdeling de
    /// Uitslagindeling). De invulvelden [`crate::VELD_NAMEN`] en [`crate::VELD_ZETELS`] zetten
    /// naam en Zetels aan ("ja") of uit; beide uit: geen labels. Het aantal Zetels staat in
    /// `letter` en de kleur Tekst, de korte naam in `naamletter` en de kleur Hoofd.
    /// Bij een Opbouw in de Zaal verschijnen ze in de Ether pas als de laatste Partij gekleurd is.
    Partijlabels {
        letter: Letterkeuze,
        naamletter: Letterkeuze,
        grootte: u16,
        /// Elke Partij een eigen kaartje, even breed, met de Zetels boven de naam en een streep in
        /// de partijkleur eronder. Anders één balk met de labels erin.
        #[serde(default)]
        kaartjes: bool,
    },
    /// Groot Blok van de Opbouw ([`Overgangsvorm::Opbouw`]): één groot label midden in zijn
    /// Plek voor de Partij die in de Zaal oplicht, met een streep in de partijkleur, de Zetels in
    /// `letter` en Tekst, de korte naam in `naamletter` en Hoofd, en "zetels" eronder. Na de
    /// Opbouw, en in het Voorbeeld, is het weg. Naam en Zetels volgen de invulvelden zoals bij de
    /// Partijlabels.
    Opbouwlabel {
        letter: Letterkeuze,
        naamletter: Letterkeuze,
        grootte: u16,
    },
    /// Groot Blok van de Stemmingen: de voor-tegenbalk. Voor in groen met "101 voor", tegen in
    /// rood met "49 tegen" (het getal in `letter`, het woord in `tekstletter`), en de lijn bij
    /// 76 in de kleur van het Blok.
    Stemmingsbalk {
        letter: Letterkeuze,
        tekstletter: Letterkeuze,
    },
    /// De uitkomst van een Stemming ("AANGENOMEN") in kapitalen op een vlak in de stemkleur,
    /// rechts in zijn Plek. De letters in de kleur van het Blok.
    Uitkomst { letter: Letterkeuze, grootte: u16 },
    /// Groot Blok Fractielijst: de Fracties voor en tegen. `onder_balk`: labels in de partijkleur
    /// onder de Stemmingsbalk, voor links en tegen rechts van waar de balk wisselt. Anders twee
    /// kolommen met een kop in de stemkleur en daaronder de partijlogo's; zonder logo de korte
    /// naam op de partijkleur.
    Fractielijst {
        letter: Letterkeuze,
        tekstletter: Letterkeuze,
        grootte: u16,
        onder_balk: bool,
    },
    /// Groot Blok Namenlijst (Hoofdelijke stemming): eerst voor, dan tegen, met een kop in de
    /// stemkleur; per Fractie haar label en de achternamen van haar Kamerleden met die stem, in
    /// `naamletter` en de kleur van het Blok. Past het niet, dan wordt de letter kleiner.
    Namenlijst {
        letter: Letterkeuze,
        naamletter: Letterkeuze,
        grootte: u16,
    },
    /// Groot Blok van de Uitslag per Partij: per Partij van de gekozen set een liggende balk in
    /// de partijkleur, van groot naar klein, met de korte naam (in `naamletter` en Tekst) voor
    /// de as en de Zetels (in `letter` en Tekst) erachter, en rasterlijnen in Hoofd. Alle
    /// Partijen of de grootste N plus "overig" onderaan.
    Uitslaggrafiek {
        letter: Letterkeuze,
        naamletter: Letterkeuze,
    },
    /// Groot Blok van Winst en verlies: de Uitslaggrafiek met de as in het midden. Per Partij
    /// het verschil met de vergelijkingsset (de som van haar Voorgangers), winst naar rechts en
    /// verlies naar links, van de grootste winst naar het grootste verlies. Een nieuwe Partij
    /// krijgt "nieuw" achter het getal; een Partij die verdwijnt een balk in Hoofd op 60
    /// procent en "nu 0".
    WinstEnVerliesgrafiek {
        letter: Letterkeuze,
        naamletter: Letterkeuze,
    },
}

/// Een kleurverloop op een Vlak: van de kleur van het Blok (met de dekking van het Vlak) naar
/// `naar`, in een rechte lijn zoals `linear-gradient` in CSS.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Verloop {
    pub naar: Kleurkeuze,
    /// Dekking van `naar` in procent. Dezelfde kleur met dekking 0 is een verloop naar
    /// doorzichtig (een waas over het beeld).
    pub naar_dekking: u8,
    /// De richting in graden, zoals in CSS: 90 is van links naar rechts, 180 van boven naar
    /// onder.
    pub richting: u16,
    /// Waar het verloop begint en eindigt, in procent van de lengte langs de richting.
    /// Daarvoor en daarna is de kleur effen.
    pub van: u8,
    pub tot: u8,
}

/// Een rand binnen een Vlak.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Rand {
    pub kleur: Kleurkeuze,
    /// In procent.
    pub dekking: u8,
    /// In pixels.
    pub dikte: u16,
}

/// Een Vlak dat zo breed is als de tekst erin: de rechterrand ligt `marge` pixels achter de
/// langste tekst van deze Blokken (hun plaats in het Sjabloon). De Plek van het Vlak is de
/// grootste breedte. Een Tekst in zo'n Vlak blijft op één regel en krijgt een kleinere letter
/// als hij niet in zijn Plek past.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct OmTekst {
    pub blokken: Vec<u16>,
    pub marge: u16,
}

impl OmTekst {
    /// De breedte van het Vlak op `vlak`, bij teksten met deze Plek, uitlijning en echte
    /// breedte (in pixels). Een tekst is nooit breder dan zijn Plek.
    pub fn breedte(&self, vlak: Plek, teksten: impl IntoIterator<Item = (Plek, Uitlijning, f32)>) -> f32 {
        let rechts = teksten
            .into_iter()
            .map(|(p, u, b)| {
                let b = b.min(p.breedte as f32);
                let x = p.x as f32;
                match u {
                    Uitlijning::Links => x + b,
                    Uitlijning::Midden => x + (p.breedte as f32 + b) / 2.0,
                    Uitlijning::Rechts => x + p.breedte as f32,
                }
            })
            .fold(vlak.x as f32, f32::max);
        (rechts - vlak.x as f32 + f32::from(self.marge)).min(vlak.breedte as f32)
    }
}

impl Bloksoort {
    /// Het lettertype van dit Blok, als het letters heeft.
    pub fn letter(&self) -> Option<&Letterkeuze> {
        match self {
            Bloksoort::Tekst { letter, .. }
            | Bloksoort::Getal { letter, .. }
            | Bloksoort::Zetelbalk { letter, .. }
            | Bloksoort::Programmalogo { letter, .. }
            | Bloksoort::Partijlabels { letter, .. }
            | Bloksoort::Opbouwlabel { letter, .. }
            | Bloksoort::Stemmingsbalk { letter, .. }
            | Bloksoort::Uitkomst { letter, .. }
            | Bloksoort::Fractielijst { letter, .. }
            | Bloksoort::Namenlijst { letter, .. }
            | Bloksoort::Uitslaggrafiek { letter, .. }
            | Bloksoort::FotoKamerlid { letter, .. }
            | Bloksoort::Partijregel { letter, .. }
            | Bloksoort::WinstEnVerliesgrafiek { letter, .. } => Some(letter),
            Bloksoort::Vlak { .. }
            | Bloksoort::Partijlogo { .. }
            | Bloksoort::Lijn { .. }
            | Bloksoort::Afbeelding { .. } => None,
        }
    }
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Tekstbron {
    Vast(String),
    /// Een invulveld dat de Regie typt, met deze naam.
    Veld(String),
    Partijnaam,
    Setnaam,
    /// De naam van de vorige set bij Twee sets naast elkaar.
    VorigeSetnaam,
    /// "zetels tegenover" en de naam van de vergelijkingsset (Winst en verlies).
    ZetelsTegenover,
    /// De korte namen van de Partijen in de Coalitiebouwer: "D66, CDA, VVD en JA21".
    Coalitiepartijen,
    /// "1 tekort voor 76" of "meerderheid".
    Coalitieregel,
    Persoonsnaam,
    PartijVanPersoon,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Getalbron {
    /// Zetels van de gekozen partij in de gekozen set.
    Zetels,
    /// Winst of verlies van de gekozen partij tegen de vergelijkingsset.
    Winst,
    /// Een invulveld dat de Regie typt.
    Veld(String),
    /// De Zetels van de Partijen in de Coalitiebouwer samen: de teller naar 76.
    Coalitie,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Partijbron {
    Partij,
    PartijVanPersoon,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Uitlijning {
    Links,
    Midden,
    Rechts,
}

/// Hoe een Blok in beeld komt en weggaat. Bijwerken is altijd vloeiend. Geen tijdlijn.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Animatie {
    pub erin: Erin,
    pub eruit: Eruit,
    /// Hoe lang het Blok wacht voordat het in beeld komt, in milliseconden.
    pub start_na_ms: u32,
}

/// De zeven manieren om in beeld te komen, en Geen.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[ts(export)]
pub enum Erin {
    Geen,
    Vervagen,
    VanLinks,
    VanRechts,
    VanOnder,
    /// Het Blok wordt van links naar rechts zichtbaar.
    Opbouwen,
    Groeien,
    /// Een Getal telt op vanaf nul; elk ander Blok vervaagt.
    Optellen,
    /// Een vlak verschijnt als dunne horizontale lijn en gaat open tot zijn volle hoogte; daarna vervaagt de tekst erin.
    OpenenVanuitLijn,
}

impl Erin {
    pub const ALLE: [Erin; 8] = [
        Erin::Vervagen,
        Erin::VanLinks,
        Erin::VanRechts,
        Erin::VanOnder,
        Erin::Opbouwen,
        Erin::Groeien,
        Erin::Optellen,
        Erin::OpenenVanuitLijn,
    ];
}

/// De vijf manieren om weg te gaan.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[ts(export)]
pub enum Eruit {
    Vervagen,
    NaarLinks,
    NaarRechts,
    NaarOnder,
    /// Het Blok klapt van boven en onder dicht naar het midden.
    Inklappen,
    /// Omgekeerd van Openen vanuit lijn: de tekst vervaagt, dan gaat het vlak dicht tot een
    /// lijn, en de lijn verdwijnt.
    SluitenNaarLijn,
}

impl Eruit {
    pub const ALLE: [Eruit; 6] = [
        Eruit::Vervagen,
        Eruit::NaarLinks,
        Eruit::NaarRechts,
        Eruit::NaarOnder,
        Eruit::Inklappen,
        Eruit::SluitenNaarLijn,
    ];
}

/// Een vorm voor een graphic, opgebouwd uit Blokken. Het eerste Blok ligt achteraan.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Sjabloon {
    pub id: String,
    pub naam: String,
    pub groep: String,
    /// Snelheid van dit Sjabloon in procent, bovenop die van de Stijl.
    pub snelheid: u16,
    pub blokken: Vec<Blok>,
    /// Wat er achter de Blokken te zien is. De Regie kan dat niet per Graphic veranderen.
    #[serde(default)]
    pub achtergrond: Achtergrond,
    /// Wat de Stoelen doen bij een Graphic van dit Sjabloon.
    #[serde(default)]
    pub stoelen: Stoelen,
    /// Het standaard Standpunt of de standaard Camerarit, als letter. Zonder: het Standpunt dat
    /// bij de soort Graphic hoort ([`crate::Graphic::standaard_standpunt`]). De Regie kan per
    /// Graphic een ander kiezen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub standpunt: Option<String>,
    /// Een eigen Kleurovergang voor de Stoelen. Zonder: die van de Stijl.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kleurovergang: Option<Kleurovergang>,
}

/// Wat er achter de Blokken van een Sjabloon te zien is.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[serde(tag = "soort")]
#[ts(export)]
pub enum Achtergrond {
    /// De Zaal zoals de camera hem ziet.
    #[default]
    Zaal,
    /// De Zaal, donker en onscherp, zodat een groot paneel rustig leest.
    ZaalDonker,
    /// Een effen kleur over het hele beeld.
    Kleur { kleur: Kleurkeuze },
    /// Een eigen afbeelding (PNG of JPEG) bij de server over het hele beeld.
    Afbeelding { bestand: String },
}

/// De dekking in procent van het zwarte vlak over de Zaal achter een Sjabloon met
/// [`Achtergrond::ZaalDonker`]. De Renderer mengt lineair, dus 88 procent zwart laat de Zaal er
/// ongeveer een derde zo licht uitzien. Met 80 bleven felle Stoelen achter het paneel te
/// opvallend (council-ronde 4).
pub const DONKER: u8 = 88;

/// Wat de Stoelen doen bij een Graphic van een Sjabloon.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub enum Stoelen {
    /// Ze kleuren volgens de gegevens van de Graphic: de set, de Stemming of de coalitie. Zonder
    /// gegevens houden ze hun kleur.
    #[default]
    Gegevens,
    /// De Stoelen van de gekozen Partij in haar kleur, de rest grijs.
    PartijUitgelicht,
    /// Alle Stoelen grijs (in de Zaal donker neutraal), zodat alleen de Stoel van het Kamerlid
    /// op de Persoonskaart opvalt: die licht op in de kleur van zijn Partij.
    StoelUitgelicht,
    /// Ze houden de kleuren van de vorige Graphic.
    Houden,
    /// Ze krijgen hun eigen leer terug, net als bij Kleur uit.
    Uit,
}

/// Wat een Graphic invult: de waarden van de invulvelden en de keuzes van de Regie.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Invulling {
    /// Het `id` van het Sjabloon.
    pub sjabloon: String,
    pub velden: BTreeMap<String, String>,
    /// De sleutel van de gekozen Partij.
    pub partij: Option<String>,
    /// Het `id` van de gekozen set; bij Twee sets naast elkaar de nieuwste.
    pub set: Option<String>,
    /// Bij Twee sets naast elkaar: het `id` van de oudste set.
    pub vorige_set: Option<String>,
    /// Een Zetelbalk of Uitslaggrafiek toont de grootste zoveel Partijen van de gekozen set plus
    /// "overig"; `None`: alle.
    pub grootste: Option<u16>,
    pub persoon: Option<Persoon>,
    /// De uitslag van een Stemming, voor de Blokken van de Sjablonen voor Stemmingen.
    pub stemming: Option<crate::stemmingen::Stemmingsuitslag>,
    /// De sleutels van de Partijen in de Coalitiebouwer.
    pub coalitie: Vec<String>,
}

/// De persoon op een Persoonskaart: een Kamerlid of iemand die de Regie toevoegt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Persoon {
    pub naam: String,
    /// Bestandsnaam van de foto bij de server.
    pub foto: Option<String>,
    /// Sleutel van de Partij.
    pub partij: Option<String>,
    /// Op het vlak als er geen foto is, zoals "SV".
    pub voorletters: String,
}

/// Een Blok met alle bronnen opgezocht: klaar om te tekenen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngevuldBlok {
    pub plek: Plek,
    pub kleur: Kleur,
    pub animatie: Animatie,
    pub inhoud: Inhoud,
}

/// Wat een Blok toont. Lettertypes, logo's en foto's zijn bestandsnamen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inhoud {
    Vlak {
        hoek: u16,
        dekking: u8,
        verloop: Option<IngevuldVerloop>,
        draaiing: i16,
        /// Kleur, dekking in procent en dikte.
        rand: Option<(Kleur, u8, u16)>,
        om_tekst: Option<OmTekst>,
    },
    Tekst {
        tekst: String,
        letter: String,
        grootte: u16,
        uitlijning: Uitlijning,
        /// In een Vlak om Tekst: op één regel, met een kleinere letter als het niet past.
        een_regel: bool,
    },
    Getal {
        waarde: Getalwaarde,
        letter: String,
        grootte: u16,
        uitlijning: Uitlijning,
        /// De teller van de Coalitiebouwer staat op 76 of meer: de Renderer laat hem pulseren.
        meerderheid: bool,
    },
    Zetelbalk {
        delen: Vec<Zeteldeel>,
        letter: String,
        lijn_bij_76: bool,
        naamletter: String,
        labels: bool,
        /// Een losse Zetelbalk: een label staat in zijn deel als het past, anders eronder op een
        /// van twee rijen met een tikje naar zijn deel. Niet bij Twee sets en de Coalitiebalk:
        /// daar is onder de balk geen ruimte voor twee rijen.
        twee_labelrijen: bool,
        /// De kleur van de Zetels in de labels (Tekst uit de Stijl).
        tekstkleur: Kleur,
        /// De kleur van de korte naam en het pijltje (Hoofd uit de Stijl).
        naamkleur: Kleur,
    },
    /// Leeg: niets te tonen. Met de as in het midden: Winst en verlies, en elke rij heeft
    /// [`Zeteldeel::winst`].
    Uitslaggrafiek {
        rijen: Vec<Zeteldeel>,
        as_in_het_midden: bool,
        /// De as loopt van 0 tot `schaal` Zetels, met om de `stap` Zetels een rasterlijn.
        schaal: u32,
        stap: u32,
        letter: String,
        naamletter: String,
        tekstkleur: Kleur,
        rasterkleur: Kleur,
    },
    Partijlogo {
        logo: Option<String>,
        kort: String,
        kleur: Kleur,
        hoek: u16,
    },
    /// De kleur van het Blok is die van de voorletters.
    Foto {
        foto: Option<String>,
        naam: String,
        hoek: u16,
        voorletters: String,
        /// Zonder foto: het vlak in de partijkleur (grijs zonder Partij).
        vlakkleur: Kleur,
        letter: String,
    },
    /// De kleur van het Blok is die van de tekst. Zonder Partij: `kort` leeg en geen logo.
    Partijregel {
        logo: Option<String>,
        kort: String,
        partijkleur: Kleur,
        letter: String,
        grootte: u16,
        tekst: String,
        tekstletter: String,
        tekstgrootte: u16,
    },
    Lijn {
        dekking: u8,
    },
    Programmalogo {
        logo: Option<String>,
        tekst: String,
        letter: String,
        dekking: u8,
    },
    /// Een afbeelding die de hele Plek vult.
    Afbeelding {
        bestand: String,
        dekking: u8,
    },
    /// Leeg: niets te tonen, ook geen vlak.
    Partijlabels {
        labels: Vec<Partijlabel>,
        letter: String,
        naamletter: String,
        grootte: u16,
        /// De kleur van de Zetels (Tekst uit de Stijl); het Blok zelf kleurt het vlak erachter.
        tekstkleur: Kleur,
        /// De kleur van de korte naam (Hoofd uit de Stijl).
        naamkleur: Kleur,
        kaartjes: bool,
    },
    /// Het grote label van de Opbouw: de Renderer toont het label van de Partij met de kleur die
    /// in de Zaal oplicht.
    Opbouwlabel {
        labels: Vec<Partijlabel>,
        letter: String,
        naamletter: String,
        grootte: u16,
        tekstkleur: Kleur,
        naamkleur: Kleur,
    },
    Stemmingsbalk {
        voor: u16,
        tegen: u16,
        algemene_stemmen: bool,
        letter: String,
        tekstletter: String,
        /// De kleur van getal en woord (Tekst uit de Stijl); de kleur van het Blok is de lijn.
        tekstkleur: Kleur,
    },
    Uitkomst {
        tekst: String,
        /// De stemkleur van het vlak.
        achter: Kleur,
        letter: String,
        grootte: u16,
    },
    /// Leeg zonder Stemming.
    Fractielijst {
        voor: Vec<Fractielabel>,
        tegen: Vec<Fractielabel>,
        /// Zetels voor en tegen: waar de balk wisselt, en de koppen van de kolommen.
        aantal_voor: u16,
        aantal_tegen: u16,
        letter: String,
        tekstletter: String,
        grootte: u16,
        onder_balk: bool,
        /// De kleur van de koppen (Tekst uit de Stijl).
        tekstkleur: Kleur,
    },
    /// Leeg zonder Stemming.
    Namenlijst {
        voor: Vec<Naamgroep>,
        tegen: Vec<Naamgroep>,
        aantal_voor: u16,
        aantal_tegen: u16,
        letter: String,
        naamletter: String,
        grootte: u16,
    },
}

/// Een Fractie in een Fractielijst of Namenlijst.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fractielabel {
    pub kort: String,
    pub kleur: Kleur,
    /// Licht of donker: leesbaar op `kleur` ([`tekst_op`]).
    pub tekstkleur: Kleur,
    pub logo: Option<String>,
    /// Bij een verdeelde Fractie: hoeveel Kamerleden aan deze kant stemden. Leeg als de hele
    /// Fractie hetzelfde stemde.
    pub aantal: Option<u16>,
    /// Hoeveel Kamerleden van de Fractie aan deze kant stemden (0 buiten een Stemming).
    pub stemmen: u16,
    /// Hoeveel Zetels de Fractie heeft (0 buiten een Stemming): bij een verdeelde Fractie staat
    /// "25 van 26", zodat het aantal niet leest als haar Zetels (council ronde 6).
    pub zetels: u16,
}

/// De namen van één Fractie aan één kant van een Hoofdelijke stemming.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Naamgroep {
    pub fractie: Fractielabel,
    pub namen: Vec<String>,
}

/// De tekstkleur op een vlak in kleur `achter`: `donker` op een lichte kleur, anders `licht`.
/// Kijkt naar de helderheid (relatieve luminantie, WCAG): wit zolang het minstens 3,5 keer zo
/// licht is als de kleur ([`LICHT_VANAF`]; de tekst is altijd vet en 30 px of groter), anders
/// donker. Zo blijft wit op middelblauw, rood en donkergroen, en wordt het donker op het groen
/// van D66, het oranje van VVD, cyaan, geel en lichtblauw (council ronde 4: wit op oranje en fel
/// groen verloor zijn contrast).
pub fn tekst_op(achter: Kleur, licht: Kleur, donker: Kleur) -> Kleur {
    if luminantie(achter) > LICHT_VANAF { donker } else { licht }
}

/// Vanaf deze helderheid (relatieve luminantie) staat er donkere tekst op een kleur: daar haalt
/// wit nog maar 3,5 keer zoveel licht als de kleur ((1 + 0,05) / (0,25 + 0,05)).
pub const LICHT_VANAF: f32 = 0.25;

/// De relatieve luminantie van een kleur (WCAG), van 0 (zwart) tot 1 (wit).
pub fn luminantie(k: Kleur) -> f32 {
    let lin = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(k.r) + 0.7152 * lin(k.g) + 0.0722 * lin(k.b)
}

/// Een kleurverloop met de kleuren opgezocht.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IngevuldVerloop {
    pub begin: Kleur,
    /// In procent.
    pub begin_dekking: u8,
    pub eind: Kleur,
    pub eind_dekking: u8,
    pub richting: u16,
    pub van: u8,
    pub tot: u8,
}

impl IngevuldVerloop {
    /// De kleur op deel `t` (0 tot 1) van de lijn van het verloop, als rood, groen, blauw en
    /// dekking van 0 tot 1. Tussen `van` en `tot` loopt de kleur gelijkmatig in sRGB, zoals in CSS.
    pub fn op(&self, t: f32) -> [f32; 4] {
        let (van, tot) = (f32::from(self.van) / 100.0, f32::from(self.tot) / 100.0);
        let f = if tot <= van {
            if t < van { 0.0 } else { 1.0 }
        } else {
            ((t - van) / (tot - van)).clamp(0.0, 1.0)
        };
        let k = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * f) / 255.0;
        let (a, b) = (self.begin, self.eind);
        [
            k(a.r, b.r),
            k(a.g, b.g),
            k(a.b, b.b),
            k(self.begin_dekking, self.eind_dekking) * 255.0 / 100.0,
        ]
    }

    /// Waar punt (`x`, `y`) van een vlak van `breedte` bij `hoogte` op de lijn van het verloop
    /// ligt (0 tot 1). Zoals in CSS is de lijn zo lang dat twee hoeken precies op 0 en 1 liggen.
    pub fn positie(&self, x: f32, y: f32, breedte: f32, hoogte: f32) -> f32 {
        let hoek = f32::from(self.richting).to_radians();
        let (sin, cos) = hoek.sin_cos();
        let lengte = (breedte * sin).abs() + (hoogte * cos).abs();
        let (dx, dy) = (x - breedte / 2.0, y - hoogte / 2.0);
        // richting 0 wijst omhoog: de y-as van het beeld loopt omlaag
        (dx * sin - dy * cos) / lengte + 0.5
    }
}

/// Eén label van de Zetelverdeling: de korte naam en de Zetels, elk als ze aan staan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Partijlabel {
    pub partij: String,
    pub kort: Option<String>,
    pub zetels: Option<u16>,
    pub kleur: Kleur,
}

/// Een getal, of een tekst als een invulveld geen getal is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Getalwaarde {
    /// `plus`: een positief getal krijgt een plusteken en een negatief een echt minteken (winst
    /// en verlies). `achter` komt achter het getal.
    Getal {
        waarde: i64,
        plus: bool,
        achter: String,
    },
    Tekst(String),
}

impl Getalwaarde {
    pub fn tekst(&self) -> String {
        match self {
            Getalwaarde::Getal { waarde, .. } => self.tekst_bij(*waarde),
            Getalwaarde::Tekst(t) => t.clone(),
        }
    }

    /// De tekst met `waarde` in plaats van het eigen getal: voor optellen en bijwerken.
    pub fn tekst_bij(&self, waarde: i64) -> String {
        match self {
            // winst en verlies: "+5" en "−5" met een echt minteken, zoals in de vormgeving
            Getalwaarde::Getal { plus: true, achter, .. } if waarde < 0 => {
                format!("\u{2212}{}{achter}", waarde.unsigned_abs())
            }
            Getalwaarde::Getal { plus, achter, .. } => {
                let teken = if *plus && waarde > 0 { "+" } else { "" };
                format!("{teken}{waarde}{achter}")
            }
            Getalwaarde::Tekst(t) => t.clone(),
        }
    }

    pub fn getal(&self) -> Option<i64> {
        match self {
            Getalwaarde::Getal { waarde, .. } => Some(*waarde),
            Getalwaarde::Tekst(_) => None,
        }
    }
}

/// Een deel van de Zetelbalk of een rij van de Uitslaggrafiek. Zonder partij: Zetels die nog
/// bij niemand horen. Overig heeft als partij [`OVERIG`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Zeteldeel {
    pub partij: Option<String>,
    pub kort: String,
    pub kleur: Kleur,
    /// De dekking van de kleur in procent.
    pub dekking: u8,
    pub zetels: u16,
    /// Het verschil met de vorige set (Twee sets naast elkaar).
    pub pijl: Option<Pijl>,
    /// Winst of verlies tegen de vergelijkingsset (Winst en verlies).
    pub winst: Option<Winst>,
}

/// Winst of verlies van een rij in de Winst-en-verliesgrafiek. Een Partij die verdwijnt heeft
/// nul Zetels en een negatief verschil.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Winst {
    pub verschil: i32,
    /// Een nieuwe Partij: geen Voorganger en geen Zetels in de vergelijkingsset.
    pub nieuw: bool,
}

/// Neutraal grijs voor Zetels zonder partij (#7).
pub const GRIJS: Kleur = Kleur {
    r: 0x6b,
    g: 0x72,
    b: 0x80,
};

/// De stemkleur voor: geen Stijlkleur, gelijk in Sjabloon en Zaal (`docs/vormgeving.md`). Voor is
/// licht blauwgroen en tegen donkerrood: ze verschillen ook in helderheid (0,37 tegen 0,13), zodat
/// wie rood en groen slecht onderscheidt ze toch uit elkaar houdt, en voor botst minder met het
/// groen van D66 (council ronde 4). Op voor staat donkere tekst, op tegen witte ([`tekst_op`]).
pub const STEM_VOOR: Kleur = Kleur { r: 0x29, g: 0xb8, b: 0x94 };
/// De stemkleur tegen.
pub const STEM_TEGEN: Kleur = Kleur { r: 0xc0, g: 0x28, b: 0x3a };

/// De `id` van de vaste Sjablonen voor Stemmingen.
pub const UITSLAG_STEMMING: &str = "uitslag-stemming";
pub const FRACTIES_VOOR_EN_TEGEN: &str = "fracties-voor-en-tegen";
pub const HOOFDELIJKE_STEMMING: &str = "hoofdelijke-stemming";
/// Het invulveld met de titel van een Stemming als kop.
pub const VELD_TITEL: &str = "titel";

impl Sjabloon {
    /// Zoekt elke bron op en kiest kleuren en lettertypes uit de Stijl.
    pub fn vul_in(&self, i: &Invulling, cijfers: &Cijfers, stijl: &Stijl) -> Vec<IngevuldBlok> {
        let in_naambalk: Vec<usize> = self
            .blokken
            .iter()
            .filter_map(|b| match &b.soort {
                Bloksoort::Vlak {
                    om_tekst: Some(o), ..
                } => Some(o.blokken.iter().map(|&n| usize::from(n))),
                _ => None,
            })
            .flatten()
            .collect();
        let mut blokken: Vec<IngevuldBlok> = self
            .blokken
            .iter()
            .enumerate()
            .map(|(n, b)| IngevuldBlok {
                plek: b.plek,
                kleur: b.kleur.in_stijl(stijl),
                animatie: b.animatie,
                inhoud: inhoud(b, i, cijfers, stijl, in_naambalk.contains(&n)),
            })
            .collect();
        // een Vlak bij een lege tekst is niet te zien
        let leeg: Vec<bool> = blokken
            .iter()
            .map(|b| matches!(&b.inhoud, Inhoud::Tekst { tekst, .. } if tekst.trim().is_empty()))
            .collect();
        for (b, ingevuld) in self.blokken.iter().zip(&mut blokken) {
            if let Bloksoort::Vlak { bij_tekst: Some(n), .. } = &b.soort
                && leeg.get(usize::from(*n)).copied().unwrap_or(true)
                && let Inhoud::Vlak { dekking, verloop, rand, .. } = &mut ingevuld.inhoud
            {
                *dekking = 0;
                *verloop = None;
                *rand = None;
            }
        }
        blokken
    }

    /// Alle bestanden die dit Sjabloon bij de server nodig heeft, met de lettertypes uit de
    /// Stijl; zonder logo's en foto's (die hangen af van de invulling).
    pub fn lettertypes<'a>(&'a self, stijl: &'a Stijl) -> impl Iterator<Item = String> + 'a {
        self.blokken.iter().flat_map(move |b| {
            let tweede = match &b.soort {
                Bloksoort::Partijlabels { naamletter: l, .. }
                | Bloksoort::Opbouwlabel { naamletter: l, .. }
                | Bloksoort::Uitslaggrafiek { naamletter: l, .. }
                | Bloksoort::Namenlijst { naamletter: l, .. }
                | Bloksoort::Stemmingsbalk { tekstletter: l, .. }
                | Bloksoort::Fractielijst { tekstletter: l, .. } => Some(l),
                _ => None,
            };
            b.soort.letter().into_iter().chain(tweede).map(|l| l.in_stijl(stijl))
        })
    }
}

/// De labels van de Zetelverdeling: per Partij met Zetels in de set, in de volgorde van de
/// cijfers. De invulvelden [`crate::VELD_NAMEN`] en [`crate::VELD_ZETELS`] zetten naam en Zetels
/// aan; beide uit: geen labels.
fn partijlabels(i: &Invulling, set: Option<&crate::cijfers::Cijferset>, cijfers: &Cijfers) -> Vec<Partijlabel> {
    let aan = |veld: &str| i.velden.get(veld).is_some_and(|v| v == "ja");
    let (namen, zetels) = (aan(crate::VELD_NAMEN), aan(crate::VELD_ZETELS));
    let Some(s) = set.filter(|_| namen || zetels) else {
        return Vec::new();
    };
    cijfers
        .partijen
        .iter()
        .filter_map(|p| {
            let z = s.zetels_van(&p.sleutel);
            (z > 0).then(|| Partijlabel {
                partij: p.sleutel.clone(),
                kort: namen.then(|| p.kort.clone()),
                zetels: zetels.then_some(z as u16),
                kleur: p.kleur.parse().unwrap_or(GRIJS),
            })
        })
        .collect()
}

fn inhoud(blok: &Blok, i: &Invulling, cijfers: &Cijfers, stijl: &Stijl, een_regel: bool) -> Inhoud {
    let (soort, kleur) = (&blok.soort, &blok.kleur);
    let partij_van = |bron: Partijbron| {
        let sleutel = match bron {
            Partijbron::Partij => i.partij.as_deref(),
            Partijbron::PartijVanPersoon => i.persoon.as_ref().and_then(|p| p.partij.as_deref()),
        };
        sleutel.and_then(|s| cijfers.partij(s))
    };
    let set = || i.set.as_deref().and_then(|s| cijfers.set(s));
    let vorige_set = || i.vorige_set.as_deref().and_then(|s| cijfers.set(s));
    let volgorde: Vec<String> = cijfers.partijen.iter().map(|p| p.sleutel.clone()).collect();
    // De Partijen met een eigen deel: uit de gekozen (nieuwste) set, ook voor de vorige set.
    let gekozen = || set().map(|s| zetelbalk::grootste(s, &volgorde, i.grootste)).unwrap_or_default();
    let telling = || set().map_or(coalitie::Telling { zetels: 0 }, |s| coalitie::Telling::van(s, &i.coalitie));
    let deel = |d: &Balkdeel, pijl: Option<Pijl>| {
        let zetels = d.zetels.min(ZETELS) as u16;
        match d.partij.as_deref().and_then(|p| cijfers.partij(p)) {
            Some(p) => Zeteldeel {
                partij: Some(p.sleutel.clone()),
                kort: p.kort.clone(),
                kleur: p.kleur.parse().unwrap_or(GRIJS),
                dekking: 100,
                zetels,
                pijl,
                winst: None,
            },
            None => Zeteldeel {
                partij: Some(OVERIG.into()),
                kort: OVERIG.into(),
                kleur: stijl.kleur(Stijlkleur::Hoofd),
                dekking: 45,
                zetels,
                pijl,
                winst: None,
            },
        }
    };
    let tekst_van = |bron: &Tekstbron| match bron {
        Tekstbron::Vast(t) => t.clone(),
        Tekstbron::Veld(v) => i.velden.get(v).cloned().unwrap_or_default(),
        Tekstbron::Partijnaam => partij_van(Partijbron::Partij)
            .map(|p| p.naam.clone())
            .unwrap_or_default(),
        Tekstbron::Setnaam => set().map(|s| s.label.clone()).unwrap_or_default(),
        Tekstbron::VorigeSetnaam => vorige_set().map(|s| s.label.clone()).unwrap_or_default(),
        Tekstbron::ZetelsTegenover => vorige_set()
            .map(|s| format!("zetels tegenover {}", s.label))
            .unwrap_or_default(),
        Tekstbron::Coalitiepartijen => {
            let korte: Vec<&str> = coalitie::op_volgorde(&volgorde, &i.coalitie)
                .iter()
                .filter_map(|p| cijfers.partij(p))
                .map(|p| p.kort.as_str())
                .collect();
            coalitie::namen(&korte)
        }
        Tekstbron::Coalitieregel => telling().regel(),
        Tekstbron::Persoonsnaam => i
            .persoon
            .as_ref()
            .map(|p| p.naam.clone())
            .unwrap_or_default(),
        Tekstbron::PartijVanPersoon => partij_van(Partijbron::PartijVanPersoon)
            .map(|p| p.naam.clone())
            .unwrap_or_default(),
    };
    match soort {
        Bloksoort::Vlak {
            hoek,
            dekking,
            verloop,
            draaiing,
            rand,
            om_tekst,
            ..
        } => Inhoud::Vlak {
            hoek: *hoek,
            dekking: *dekking,
            verloop: verloop.as_ref().map(|v| IngevuldVerloop {
                begin: kleur.in_stijl(stijl),
                begin_dekking: *dekking,
                eind: v.naar.in_stijl(stijl),
                eind_dekking: v.naar_dekking,
                richting: v.richting,
                van: v.van,
                tot: v.tot,
            }),
            draaiing: *draaiing,
            rand: rand
                .as_ref()
                .map(|r| (r.kleur.in_stijl(stijl), r.dekking, r.dikte)),
            om_tekst: om_tekst.clone(),
        },
        Bloksoort::Tekst {
            bron,
            letter,
            grootte,
            uitlijning,
            een_regel: eigen,
        } => Inhoud::Tekst {
            tekst: tekst_van(bron),
            letter: letter.in_stijl(stijl),
            grootte: *grootte,
            uitlijning: *uitlijning,
            een_regel: een_regel || *eigen,
        },
        Bloksoort::Getal {
            bron,
            letter,
            grootte,
            uitlijning,
        } => {
            let getal = |waarde: i64, plus: bool, achter: &str| Getalwaarde::Getal {
                waarde,
                plus,
                achter: achter.into(),
            };
            let waarde = match bron {
                Getalbron::Zetels => {
                    let z = match (set(), i.partij.as_deref()) {
                        (Some(s), Some(p)) => s.zetels_van(p),
                        _ => 0,
                    };
                    getal(i64::from(z), false, "")
                }
                Getalbron::Winst => {
                    let regel = i
                        .set
                        .as_deref()
                        .zip(i.partij.as_deref())
                        .and_then(|(s, p)| {
                            cijfers
                                .winst_en_verlies(s)?
                                .into_iter()
                                .find(|r| r.partij == p)
                        });
                    match regel {
                        Some(r) => getal(
                            i64::from(r.verschil),
                            true,
                            if r.nieuw { " nieuw" } else { "" },
                        ),
                        None => getal(0, true, ""),
                    }
                }
                Getalbron::Coalitie => getal(i64::from(telling().zetels), false, ""),
                Getalbron::Veld(v) => {
                    let t = i.velden.get(v).cloned().unwrap_or_default();
                    match t.trim().parse::<i64>() {
                        Ok(w) => getal(w, t.trim().starts_with('+'), ""),
                        Err(_) => Getalwaarde::Tekst(t),
                    }
                }
            };
            Inhoud::Getal {
                waarde,
                letter: letter.in_stijl(stijl),
                grootte: *grootte,
                uitlijning: *uitlijning,
                meerderheid: *bron == Getalbron::Coalitie && telling().meerderheid(),
            }
        }
        Bloksoort::Zetelbalk {
            letter,
            lijn_bij_76,
            naamletter,
            labels,
            vorige_set: vorige,
            pijlen,
            coalitie: met_coalitie,
        } => {
            let gekozen = gekozen();
            let mut delen = Vec::new();
            let deze = if *vorige { vorige_set() } else { set() };
            if let (true, Some(s)) = (*met_coalitie, deze) {
                for (p, erin) in coalitie::blokken(&volgorde, &i.coalitie) {
                    let zetels = s.zetels_van(&p);
                    if zetels == 0 {
                        continue;
                    }
                    let mut d = deel(&Balkdeel { partij: Some(p.clone()), zetels }, None);
                    if !erin {
                        // een eigen sleutel: komt de Partij erbij, dan krimpt dit deel en groeit
                        // haar deel in de coalitie, zodat ze vloeiend naar links schuift
                        d.partij = Some(format!("{RESTDEEL}{p}"));
                        d.kort = String::new();
                        d.kleur = stijl.kleur(Stijlkleur::Tekst);
                        d.dekking = RESTDEKKING;
                    }
                    delen.push(d);
                }
            } else if let Some(s) = deze {
                let eerder = vorige_set()
                    .filter(|_| *pijlen)
                    .map(|v| zetelbalk::delen(v, &volgorde, &gekozen));
                for d in zetelbalk::delen(s, &volgorde, &gekozen) {
                    let pijl = eerder.as_ref().map(|e| zetelbalk::pijl(e, &d));
                    delen.push(deel(&d, pijl));
                }
            }
            let totaal: u32 = delen.iter().map(|d| u32::from(d.zetels)).sum();
            if totaal < ZETELS {
                delen.push(Zeteldeel {
                    partij: None,
                    kort: String::new(),
                    kleur: stijl.kleur(Stijlkleur::Tekst),
                    dekking: RESTDEKKING,
                    zetels: (ZETELS - totaal) as u16,
                    pijl: None,
                    winst: None,
                });
            }
            Inhoud::Zetelbalk {
                delen,
                letter: letter.in_stijl(stijl),
                lijn_bij_76: *lijn_bij_76,
                naamletter: naamletter.in_stijl(stijl),
                labels: *labels,
                twee_labelrijen: *labels && !*vorige && !*pijlen && !*met_coalitie,
                tekstkleur: stijl.kleur(Stijlkleur::Tekst),
                naamkleur: stijl.kleur(Stijlkleur::Hoofd),
            }
        }
        Bloksoort::Uitslaggrafiek { letter, naamletter } => {
            let rijen: Vec<Zeteldeel> = set()
                .map(|s| zetelbalk::delen(s, &volgorde, &gekozen()))
                .map(zetelbalk::van_groot_naar_klein)
                .unwrap_or_default()
                .iter()
                .map(|d| deel(d, None))
                .collect();
            let grootste = rijen.iter().map(|r| u32::from(r.zetels)).max().unwrap_or(0);
            let (schaal, stap) = zetelbalk::as_van(grootste);
            Inhoud::Uitslaggrafiek {
                rijen,
                as_in_het_midden: false,
                schaal,
                stap,
                letter: letter.in_stijl(stijl),
                naamletter: naamletter.in_stijl(stijl),
                tekstkleur: stijl.kleur(Stijlkleur::Tekst),
                rasterkleur: stijl.kleur(Stijlkleur::Hoofd),
            }
        }
        Bloksoort::WinstEnVerliesgrafiek { letter, naamletter } => {
            let regels = i
                .set
                .as_deref()
                .zip(i.vorige_set.as_deref())
                .and_then(|(s, v)| cijfers.winst_en_verlies_tegen(s, v))
                .unwrap_or_default();
            let rijen: Vec<Zeteldeel> = zetelbalk::van_winst_naar_verlies(regels, &volgorde)
                .into_iter()
                .filter_map(|r| {
                    let p = cijfers.partij(&r.partij)?;
                    let weg = r.zetels == 0;
                    Some(Zeteldeel {
                        partij: Some(p.sleutel.clone()),
                        kort: p.kort.clone(),
                        kleur: if weg { stijl.kleur(Stijlkleur::Hoofd) } else { p.kleur.parse().unwrap_or(GRIJS) },
                        dekking: if weg { 60 } else { 100 },
                        zetels: r.zetels,
                        pijl: None,
                        winst: Some(Winst { verschil: r.verschil, nieuw: r.nieuw }),
                    })
                })
                .collect();
            let grootste = rijen.iter().filter_map(|r| r.winst).map(|w| w.verschil.unsigned_abs()).max().unwrap_or(0);
            let (schaal, stap) = zetelbalk::as_van(grootste);
            Inhoud::Uitslaggrafiek {
                rijen,
                as_in_het_midden: true,
                schaal,
                stap,
                letter: letter.in_stijl(stijl),
                naamletter: naamletter.in_stijl(stijl),
                tekstkleur: stijl.kleur(Stijlkleur::Tekst),
                rasterkleur: stijl.kleur(Stijlkleur::Hoofd),
            }
        }
        Bloksoort::Partijlogo { bron, hoek } => match partij_van(*bron) {
            Some(p) => Inhoud::Partijlogo {
                logo: p.logo.clone(),
                kort: p.kort.clone(),
                kleur: p.kleur.parse().unwrap_or(GRIJS),
                hoek: *hoek,
            },
            None => Inhoud::Partijlogo {
                logo: None,
                kort: String::new(),
                kleur: GRIJS,
                hoek: *hoek,
            },
        },
        Bloksoort::FotoKamerlid { hoek, letter } => Inhoud::Foto {
            foto: i.persoon.as_ref().and_then(|p| p.foto.clone()),
            naam: i
                .persoon
                .as_ref()
                .map(|p| p.naam.clone())
                .unwrap_or_default(),
            hoek: *hoek,
            voorletters: i
                .persoon
                .as_ref()
                .map(|p| p.voorletters.clone())
                .unwrap_or_default(),
            vlakkleur: partij_van(Partijbron::PartijVanPersoon)
                .and_then(|p| p.kleur.parse().ok())
                .unwrap_or(GRIJS),
            letter: letter.in_stijl(stijl),
        },
        Bloksoort::Partijregel {
            bron,
            letter,
            grootte,
            tekst,
            tekstletter,
            tekstgrootte,
        } => {
            let p = partij_van(*bron);
            let tekst = tekst_van(tekst);
            Inhoud::Partijregel {
                logo: p.and_then(|p| p.logo.clone()),
                kort: p.map(|p| p.kort.clone()).unwrap_or_default(),
                partijkleur: p.and_then(|p| p.kleur.parse().ok()).unwrap_or(GRIJS),
                letter: letter.in_stijl(stijl),
                grootte: *grootte,
                tekst,
                tekstletter: tekstletter.in_stijl(stijl),
                tekstgrootte: *tekstgrootte,
            }
        }
        Bloksoort::Lijn { dekking } => Inhoud::Lijn { dekking: *dekking },
        Bloksoort::Partijlabels { letter, naamletter, grootte, kaartjes } => Inhoud::Partijlabels {
            labels: partijlabels(i, set(), cijfers),
            letter: letter.in_stijl(stijl),
            naamletter: naamletter.in_stijl(stijl),
            grootte: *grootte,
            tekstkleur: stijl.kleur(Stijlkleur::Tekst),
            naamkleur: stijl.kleur(Stijlkleur::Hoofd),
            kaartjes: *kaartjes,
        },
        Bloksoort::Opbouwlabel { letter, naamletter, grootte } => Inhoud::Opbouwlabel {
            labels: partijlabels(i, set(), cijfers),
            letter: letter.in_stijl(stijl),
            naamletter: naamletter.in_stijl(stijl),
            grootte: *grootte,
            tekstkleur: stijl.kleur(Stijlkleur::Tekst),
            naamkleur: stijl.kleur(Stijlkleur::Hoofd),
        },
        Bloksoort::Stemmingsbalk { letter, tekstletter } => {
            let (voor, tegen, algemeen) = i
                .stemming
                .as_ref()
                .map_or((0, 0, false), |u| (u.voor, u.tegen, u.algemene_stemmen));
            Inhoud::Stemmingsbalk {
                voor,
                tegen,
                algemene_stemmen: algemeen,
                letter: letter.in_stijl(stijl),
                tekstletter: tekstletter.in_stijl(stijl),
                tekstkleur: stijl.kleur(Stijlkleur::Tekst),
            }
        }
        Bloksoort::Uitkomst { letter, grootte } => {
            let (tekst, achter) = match i.stemming.as_ref().map(|u| u.besluit) {
                Some(Besluit::Aangenomen) => ("AANGENOMEN", STEM_VOOR),
                Some(Besluit::Verworpen) => ("VERWORPEN", STEM_TEGEN),
                None => ("", GRIJS),
            };
            Inhoud::Uitkomst {
                tekst: tekst.into(),
                achter,
                letter: letter.in_stijl(stijl),
                grootte: *grootte,
            }
        }
        Bloksoort::Fractielijst {
            letter,
            tekstletter,
            grootte,
            onder_balk,
        } => {
            let kant = |stem: Stem| i.stemming.as_ref().map_or_else(Vec::new, |u| fracties_aan_kant(u, stem, stijl));
            let (aantal_voor, aantal_tegen) = i.stemming.as_ref().map_or((0, 0), |u| (u.voor, u.tegen));
            Inhoud::Fractielijst {
                voor: kant(Stem::Voor),
                tegen: kant(Stem::Tegen),
                aantal_voor,
                aantal_tegen,
                letter: letter.in_stijl(stijl),
                tekstletter: tekstletter.in_stijl(stijl),
                grootte: *grootte,
                onder_balk: *onder_balk,
                tekstkleur: stijl.kleur(Stijlkleur::Tekst),
            }
        }
        Bloksoort::Namenlijst {
            letter,
            naamletter,
            grootte,
        } => {
            // Per kant elke Fractie met haar aantal; namen alleen van wie anders stemde dan de
            // Fractie, want dat is het nieuws van een hoofdelijke stemming. 150 namen zijn op tv
            // niet te lezen.
            let kant = |stem: Stem| -> Vec<Naamgroep> {
                let Some(u) = &i.stemming else { return Vec::new() };
                let labels = fracties_aan_kant(u, stem, stijl);
                u.fracties
                    .iter()
                    .filter_map(|f| {
                        let label = labels.iter().find(|l| l.kort == f.kort.as_deref().unwrap_or(&f.afkorting))?;
                        let namen = if f.stem == stem {
                            Vec::new()
                        } else {
                            u.afwijkend.iter().filter(|a| a.fractie == f.afkorting && a.stem == stem).map(|a| achternaam(u, a)).collect()
                        };
                        Some(Naamgroep { fractie: label.clone(), namen })
                    })
                    .collect()
            };
            let (aantal_voor, aantal_tegen) = i.stemming.as_ref().map_or((0, 0), |u| (u.voor, u.tegen));
            Inhoud::Namenlijst {
                voor: kant(Stem::Voor),
                tegen: kant(Stem::Tegen),
                aantal_voor,
                aantal_tegen,
                letter: letter.in_stijl(stijl),
                naamletter: naamletter.in_stijl(stijl),
                grootte: *grootte,
            }
        }
        Bloksoort::Programmalogo { letter, dekking } => Inhoud::Programmalogo {
            logo: stijl.programmalogo.bestand.clone(),
            tekst: stijl.programmalogo.tekst.clone(),
            letter: letter.in_stijl(stijl),
            dekking: *dekking,
        },
        Bloksoort::Afbeelding { bestand, dekking } => Inhoud::Afbeelding {
            bestand: bestand.clone(),
            dekking: *dekking,
        },
    }
}

fn fractielabel(kort: &str, kleur: Option<&str>, logo: Option<String>, stijl: &Stijl) -> Fractielabel {
    let kleur = kleur.and_then(|k| k.parse().ok()).unwrap_or(GRIJS);
    Fractielabel {
        kort: kort.into(),
        kleur,
        tekstkleur: tekst_op(kleur, stijl.kleur(Stijlkleur::Tekst), stijl.kleur(Stijlkleur::Vlak)),
        logo,
        aantal: None,
        stemmen: 0,
        zetels: 0,
    }
}

/// De naam van een Kamerlid met een afwijkende stem zoals in de Namenlijst ("Van der Werf"),
/// anders zijn volledige naam.
fn achternaam(u: &crate::stemmingen::Stemmingsuitslag, a: &crate::stemmingen::AfwijkendInUitslag) -> String {
    u.kamerleden.iter().find(|k| k.kamerlid == a.kamerlid).map_or_else(|| a.naam.clone(), |k| k.naam.clone())
}

/// De Fracties aan één kant van een Stemming, in de volgorde van de Kamer. Een verdeelde Fractie
/// staat aan beide kanten, elk met het aantal Kamerleden dat zo stemde: zo ziet de kijker dat
/// niet de hele Fractie voor of tegen was.
fn fracties_aan_kant(u: &crate::stemmingen::Stemmingsuitslag, stem: Stem, stijl: &Stijl) -> Vec<Fractielabel> {
    u.fracties
        .iter()
        .filter_map(|f| {
            let afwijkers = |s: Stem| u.afwijkend.iter().filter(|a| a.fractie == f.afkorting && a.stem == s).count() as u16;
            let anders = afwijkers(f.stem.andere());
            let hier = if f.stem == stem { f.zetels.saturating_sub(anders) } else { anders };
            (hier > 0).then(|| Fractielabel {
                aantal: (anders > 0).then_some(hier),
                stemmen: hier,
                zetels: f.zetels,
                ..fractielabel(f.kort.as_deref().unwrap_or(&f.afkorting), f.kleur.as_deref(), f.logo.clone(), stijl)
            })
        })
        .collect()
}

/// De vaste Sjablonen die met de software meekomen.
pub fn vaste_sjablonen() -> Vec<Sjabloon> {
    vec![
        titelkaart(),
        zetelverdeling(),
        zetelbalk_sjabloon(),
        twee_sets(),
        uitslag_per_partij(),
        winst_en_verlies(),
        partij_uitgelicht(),
        coalitie_in_de_zaal(),
        coalitiebalk(),
        uitslag_stemming(),
        fracties_voor_en_tegen(),
        hoofdelijke_stemming(),
        persoonskaart(),
    ]
}

/// De `id`'s van de vaste Sjablonen met cijfers.
pub const ZETELVERDELING: &str = "zetelverdeling";
pub const ZETELBALK: &str = "zetelbalk";
pub const TWEE_SETS: &str = "twee-sets";
pub const UITSLAG_PER_PARTIJ: &str = "uitslag-per-partij";
pub const WINST_EN_VERLIES: &str = "winst-en-verlies";
pub const PARTIJ_UITGELICHT: &str = "partij-uitgelicht";
/// De `id`'s van de Sjablonen van de Coalitiebouwer.
pub const COALITIE: &str = "coalitie";
pub const COALITIEBALK: &str = "coalitiebalk";
/// Voor de sleutel van een Partij buiten de coalitie in de Coalitiebalk: een eigen deel, apart
/// van haar deel in de coalitie.
pub const RESTDEEL: &str = "rest:";
/// De dekking van de rest van een Zetelbalk (in Tekst): neutraal grijs, zodat de Partijen ervoor
/// eruit springen.
pub const RESTDEKKING: u8 = 14;

/// Het vaste Sjabloon met dit `id`.
pub fn vast_sjabloon(id: &str) -> Option<Sjabloon> {
    vaste_sjablonen().into_iter().find(|s| s.id == id)
}

// Bouwstenen van de vaste Sjablonen (`docs/vormgeving.md`).

pub fn plek(x: i32, y: i32, breedte: u32, hoogte: u32) -> Plek {
    Plek {
        x,
        y,
        breedte,
        hoogte,
    }
}

pub fn animatie(erin: Erin, eruit: Eruit, start_na_ms: u32) -> Animatie {
    Animatie {
        erin,
        eruit,
        start_na_ms,
    }
}

/// Een effen Vlak zonder ronding.
pub fn vlak(dekking: u8) -> Bloksoort {
    vlak_met(dekking, None, 0, None, None)
}

/// Een Vlak zonder ronding.
pub fn vlak_met(
    dekking: u8,
    verloop: Option<Verloop>,
    draaiing: i16,
    rand: Option<Rand>,
    om_tekst: Option<OmTekst>,
) -> Bloksoort {
    Bloksoort::Vlak {
        hoek: 0,
        dekking,
        verloop,
        draaiing,
        rand,
        om_tekst,
        bij_tekst: None,
    }
}

/// Een tekst uit de Stijl, links uitgelijnd.
pub fn tekst(bron: Tekstbron, letter: Stijlletter, grootte: u16) -> Bloksoort {
    Bloksoort::Tekst {
        bron,
        letter: Letterkeuze::Stijl(letter),
        grootte,
        uitlijning: Uitlijning::Links,
        een_regel: false,
    }
}

/// De naambalk: een Vlak in de kleur Vlak op 93 procent dat zo breed is als de tekst van
/// `blokken` plus 48 pixels. Opent vanuit een lijn.
pub fn naambalk(p: Plek, blokken: Vec<u16>) -> Blok {
    Blok {
        plek: p,
        soort: vlak_met(93, None, 0, None, Some(OmTekst { blokken, marge: 48 })),
        kleur: Kleurkeuze::Stijl(Stijlkleur::Vlak),
        animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
    }
}

/// Een paneel voor een grote graphic: een verloop van Achtergrond naar Vlak (beide op 95
/// procent), schuin onder 120 graden, tot 62 procent. Opent vanuit een lijn.
pub fn paneel(p: Plek) -> Blok {
    Blok {
        plek: p,
        soort: vlak_met(
            97,
            Some(Verloop {
                naar: Kleurkeuze::Stijl(Stijlkleur::Vlak),
                naar_dekking: 97,
                richting: 120,
                van: 0,
                tot: 62,
            }),
            0,
            None,
            None,
        ),
        kleur: Kleurkeuze::Stijl(Stijlkleur::Achtergrond),
        animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
    }
}

/// De kicker boven een kop: een streepje in Accent (10 bij 30) en 30 pixels tekst in Hoofd en
/// de cijferletter, linksboven op (`x`, `y`). Twee Blokken.
pub fn kicker(x: i32, y: i32, breedte: u32, bron: Tekstbron) -> [Blok; 2] {
    [
        Blok {
            plek: plek(x, y, 10, 30),
            soort: vlak_met(100, None, 0, None, None),
            kleur: Kleurkeuze::Stijl(Stijlkleur::Accent),
            animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
        },
        Blok {
            plek: plek(x + 24, y - 6, breedte, 42),
            soort: tekst(bron, Stijlletter::Cijfers, 30),
            kleur: Kleurkeuze::Stijl(Stijlkleur::Hoofd),
            animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
        },
    ]
}

/// Het programmalogo als bug rechtsboven, zoals on-air: 196 pixels breed op 90 procent. Zonder logo: de tekst ervan in wit.
pub fn programmabug() -> Blok {
    Blok {
        plek: plek(1424, 46, 400, 131),
        soort: Bloksoort::Programmalogo {
            letter: Letterkeuze::Stijl(Stijlletter::Kop),
            dekking: 90,
        },
        kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
        animatie: animatie(Erin::Vervagen, Eruit::Vervagen, 0),
    }
}

/// De Titelkaart: een kop en een regel in een naambalk linksonder, over een waas in de kleur
/// Vlak die naar boven doorzichtig wordt, en het programmalogo rechtsboven.
fn titelkaart() -> Sjabloon {
    let veld = |v: &str| Tekstbron::Veld(v.into());
    let naambalk_tekst = |p, bron, letter, grootte, kleur| Blok {
        plek: p,
        soort: tekst(bron, letter, grootte),
        kleur: Kleurkeuze::Stijl(kleur),
        animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
    };
    Sjabloon {
        id: "titelkaart".into(),
        naam: "Titelkaart".into(),
        groep: "Algemeen".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::Gegevens,
        standpunt: None,
        blokken: vec![
            // 0: een waas onderin, van Vlak naar doorzichtig, zodat de naambalk rustig staat en
            // de Zaal erboven helder blijft
            Blok {
                plek: plek(0, 560, 1920, 520),
                soort: vlak_met(
                    85,
                    Some(Verloop {
                        naar: Kleurkeuze::Stijl(Stijlkleur::Vlak),
                        naar_dekking: 0,
                        richting: 0,
                        van: 0,
                        tot: 100,
                    }),
                    0,
                    None,
                    None,
                ),
                kleur: Kleurkeuze::Stijl(Stijlkleur::Vlak),
                animatie: animatie(Erin::Vervagen, Eruit::Vervagen, 0),
            },
            // 1 tot 3: de naambalk met kop en regel
            naambalk(plek(96, 744, 1728, 210), vec![2, 3]),
            naambalk_tekst(
                plek(140, 768, 1640, 104),
                veld("kop"),
                Stijlletter::Kop,
                92,
                Stijlkleur::Hoofd,
            ),
            naambalk_tekst(
                plek(140, 876, 1640, 54),
                veld("regel"),
                Stijlletter::Tekst,
                40,
                Stijlkleur::Tekst,
            ),
            // 4: het programmalogo
            programmabug(),
        ],
        kleurovergang: None,
    }
}

/// De Zetelverdeling in de Zaal: de Stoelen kleuren in de Uitslagindeling (dat doet de Zaal, niet
/// een Blok), als Opbouw: één Partij tegelijk, de grootste eerst, met haar label groot op de
/// vloer. Daarna staan alle Partijen als kaartjes op één rij in het plafond, in dezelfde volgorde
/// als hun Stoelen van links naar rechts. Linksboven een naambalk met de kicker "ZETELVERDELING"
/// en de naam van de set; rechtsboven het programmalogo. De camera staat stil op Standpunt A, met
/// de hele halve cirkel in beeld: zo zie je bij elke stap het blok bij het label. De plekken passen
/// daarbij en bij het eindbeeld van de Onthulling.
fn zetelverdeling() -> Sjabloon {
    let [streepje, kicker_tekst] = kicker(140, 84, 1000, Tekstbron::Vast("ZETELVERDELING".into()));
    Sjabloon {
        id: ZETELVERDELING.into(),
        naam: "Zetelverdeling in de Zaal".into(),
        groep: "Uitslagavond".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::Gegevens,
        // Standpunt A, zoals elke zetelgraphic: geen beweging tijdens de Opbouw
        standpunt: None,
        blokken: vec![
            naambalk(plek(96, 54, 1100, 166), vec![2, 3]),
            streepje,
            kicker_tekst,
            Blok {
                plek: plek(140, 122, 1010, 76),
                soort: tekst(Tekstbron::Setnaam, Stijlletter::Kop, 64),
                kleur: Kleurkeuze::Stijl(Stijlkleur::Hoofd),
                animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
            },
            programmabug(),
            Blok {
                plek: plek(96, 250, 1728, 136),
                soort: Bloksoort::Partijlabels {
                    letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                    naamletter: Letterkeuze::Stijl(Stijlletter::Tekst),
                    grootte: 34,
                    kaartjes: true,
                },
                kleur: Kleurkeuze::Stijl(Stijlkleur::Vlak),
                animatie: animatie(Erin::Vervagen, Eruit::Vervagen, 0),
            },
            Blok {
                plek: plek(96, 850, 1728, 150),
                soort: Bloksoort::Opbouwlabel {
                    letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                    naamletter: Letterkeuze::Stijl(Stijlletter::Kop),
                    grootte: 52,
                },
                kleur: Kleurkeuze::Stijl(Stijlkleur::Vlak),
                animatie: animatie(Erin::Vervagen, Eruit::Vervagen, 0),
            },
        ],
        kleurovergang: Some(Kleurovergang::standaard(Overgangsvorm::Opbouw)),
    }
}

/// De Persoonskaart: een zijpaneel rechts tot de rand van het beeld, met de foto, daaronder de
/// Partij met de functie, de naam, een regel met extra gegevens en de vrije regel met een
/// streepje in Accent ervoor. De Zaal
/// blijft links in beeld, met de Stoel van het Kamerlid (zie [`crate::camera::shot_op_stoel`]):
/// alleen die Stoel in de kleur van zijn Partij, de rest donker ([`Stoelen::StoelUitgelicht`]).
fn persoonskaart() -> Sjabloon {
    let open = |start| animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, start);
    let tekstblok = |p, bron, letter, grootte, kleur, een_regel| Blok {
        plek: p,
        soort: Bloksoort::Tekst {
            bron,
            letter: Letterkeuze::Stijl(letter),
            grootte,
            uitlijning: Uitlijning::Links,
            een_regel,
        },
        kleur: Kleurkeuze::Stijl(kleur),
        animatie: open(0),
    };
    let veld = |v: &str| Tekstbron::Veld(v.into());
    let mut blokken = vec![
        // 0: het zijpaneel, effen (de Zaal schijnt er niet doorheen), een verloop van Achtergrond
        // naar Vlak dat precies op de bovenkant van het onderste vlak (y 640) Vlak is: geen naad
        // (council ronde 6)
        Blok {
            plek: plek(1166, 0, 754, 1080),
            soort: vlak_met(
                100,
                Some(Verloop {
                    naar: Kleurkeuze::Stijl(Stijlkleur::Vlak),
                    naar_dekking: 100,
                    richting: 180,
                    van: 0,
                    tot: 59,
                }),
                0,
                None,
                None,
            ),
            kleur: Kleurkeuze::Stijl(Stijlkleur::Achtergrond),
            animatie: open(0),
        },
        // 1: onderaan Vlak
        Blok {
            plek: plek(1166, 640, 754, 440),
            soort: vlak(100),
            kleur: Kleurkeuze::Stijl(Stijlkleur::Vlak),
            animatie: open(0),
        },
    ];
    blokken.extend([
        // 2: de foto, of de voorletters in de kopletter op de partijkleur; zo breed als de tekst.
        // De Renderer maakt een portret vrijstaand: het paneel loopt achter het hoofd door, en de
        // foto staat met een rechte onderkant op het onderste vlak (y 640), zoals achter een
        // desk, in plaats van weg te vloeien (council ronde 5).
        Blok {
            plek: plek(1214, 130, 610, 510),
            soort: Bloksoort::FotoKamerlid {
                hoek: 0,
                letter: Letterkeuze::Stijl(Stijlletter::Kop),
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Vlak),
            animatie: animatie(Erin::Vervagen, Eruit::Vervagen, 260),
        },
        // 3: de Partij op de partijkleur, met de functie erachter
        Blok {
            plek: plek(1214, 674, 610, 47),
            soort: Bloksoort::Partijregel {
                bron: Partijbron::PartijVanPersoon,
                letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                grootte: 30,
                tekst: veld(crate::VELD_FUNCTIE),
                tekstletter: Letterkeuze::Stijl(Stijlletter::Tekst),
                tekstgrootte: 34,
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
            animatie: open(0),
        },
        // 4 en 5: de naam en de regel met extra gegevens, elk op één regel
        tekstblok(
            plek(1214, 736, 610, 96),
            Tekstbron::Persoonsnaam,
            Stijlletter::Kop,
            76,
            Stijlkleur::Hoofd,
            true,
        ),
        tekstblok(
            plek(1214, 838, 610, 46),
            veld(crate::VELD_GEGEVENS),
            Stijlletter::Tekst,
            34,
            Stijlkleur::Hoofd,
            true,
        ),
        // 6 en 7: de vrije regel met een streepje in Accent ervoor; zonder regel geen streepje
        Blok {
            plek: plek(1214, 931, 10, 30),
            soort: Bloksoort::Vlak {
                hoek: 0,
                dekking: 100,
                verloop: None,
                draaiing: 0,
                rand: None,
                om_tekst: None,
                bij_tekst: Some(7),
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Accent),
            animatie: open(0),
        },
        tekstblok(
            plek(1238, 900, 586, 96),
            veld(crate::VELD_VRIJE_REGEL),
            Stijlletter::Tekst,
            36,
            Stijlkleur::Tekst,
            false,
        ),
        // de onderkant van het portret zakt in de laatste 36 px zacht in het onderste vlak, zodat
        // hij er niet op geplakt staat en ook niet als een geest wegvloeit (council ronde 5 en 6)
        Blok {
            plek: plek(1166, 604, 754, 36),
            soort: vlak_met(
                0,
                Some(Verloop {
                    naar: Kleurkeuze::Stijl(Stijlkleur::Vlak),
                    naar_dekking: 100,
                    richting: 180,
                    van: 0,
                    tot: 100,
                }),
                0,
                None,
                None,
            ),
            kleur: Kleurkeuze::Stijl(Stijlkleur::Vlak),
            animatie: animatie(Erin::Vervagen, Eruit::Vervagen, 260),
        },
        // het programmalogo
        programmabug(),
    ]);
    Sjabloon {
        id: "persoonskaart".into(),
        naam: "Persoonskaart".into(),
        groep: "Algemeen".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::StoelUitgelicht,
        standpunt: None,
        blokken,
        kleurovergang: None,
    }
}

/// De kicker, de titel van de Stemming als kop (op één regel) en de uitkomst rechtsboven, in
/// een paneel met linksboven (`x`, `y`): de bovenkant van de drie Sjablonen voor Stemmingen,
/// Vijf Blokken.
fn stemmingskop(x: i32, y: i32, kicker_tekst: &str) -> Vec<Blok> {
    let erin = animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0);
    let [streepje, kicker_blok] = kicker(x + 48, y + 38, 1000, Tekstbron::Vast(kicker_tekst.into()));
    vec![
        streepje,
        kicker_blok,
        Blok {
            plek: plek(x + 48, y + 76, 1200, 80),
            soort: Bloksoort::Tekst {
                bron: Tekstbron::Veld(VELD_TITEL.into()),
                letter: Letterkeuze::Stijl(Stijlletter::Kop),
                grootte: 64,
                uitlijning: Uitlijning::Links,
                een_regel: true,
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Hoofd),
            animatie: erin,
        },
        Blok {
            plek: plek(x + 1728 - 48 - 560, y + 44, 560, 78),
            soort: Bloksoort::Uitkomst {
                letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                grootte: 44,
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
            animatie: erin,
        },
    ]
}

/// De Uitslag van een Stemming: een paneel onderin
/// met kicker, titel, uitkomst, de voor-tegenbalk en de Fracties eronder. De Zaal kleurt de
/// Stoelen in hun stem.
fn uitslag_stemming() -> Sjabloon {
    // onder de balk is ruimte voor drie rijen labels van 30 px
    let (x, y) = (96, 512);
    let erin = animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0);
    let mut blokken = vec![
        paneel(plek(x, y, 1728, 494)),
    ];
    blokken.extend(stemmingskop(x, y, "STEMMING"));
    blokken.extend([
        Blok {
            // boven de balk staat het drempelgetal bij de streep
            plek: plek(x + 48, y + 206, 1632, 72),
            soort: Bloksoort::Stemmingsbalk {
                letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                tekstletter: Letterkeuze::Stijl(Stijlletter::Tekst),
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Accent),
            animatie: erin,
        },
        Blok {
            plek: plek(x + 48, y + 302, 1632, 170),
            soort: Bloksoort::Fractielijst {
                letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                tekstletter: Letterkeuze::Stijl(Stijlletter::Tekst),
                grootte: 30,
                onder_balk: true,
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
            animatie: erin,
        },
        programmabug(),
    ]);
    Sjabloon {
        id: UITSLAG_STEMMING.into(),
        naam: "Uitslag van een Stemming".into(),
        groep: "Stemmingen".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::Gegevens,
        // de Stoelen boven het paneel (council-ronde 3)
        standpunt: Some(crate::camera::BOVEN_HET_PANEEL.into()),
        blokken,
        kleurovergang: None,
    }
}

/// Een groot paneel voor de Fracties voor en tegen en de Hoofdelijke stemming, met de kop van
/// de Stemming en daaronder het grote Blok `groot`. Het paneel is zo breed als het beeld binnen
/// title safe en begint daarom onder het programmalogo.
fn groot_stemmingspaneel(id: &str, naam: &str, kicker_tekst: &str, groot: Bloksoort) -> Sjabloon {
    let (x, y, hoog) = (96, 196, 814);
    let mut blokken = vec![
        paneel(plek(x, y, 1728, hoog)),
    ];
    blokken.extend(stemmingskop(x, y, kicker_tekst));
    blokken.extend([
        Blok {
            plek: plek(x + 48, y + 186, 1632, hoog - 186 - 40),
            soort: groot,
            kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
            animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
        },
        programmabug(),
    ]);
    Sjabloon {
        id: id.into(),
        naam: naam.into(),
        groep: "Stemmingen".into(),
        snelheid: 100,
        // het paneel vult bijna het hele beeld: de Zaal erachter donker en onscherp
        achtergrond: Achtergrond::ZaalDonker,
        stoelen: Stoelen::Gegevens,
        standpunt: None,
        blokken,
        kleurovergang: None,
    }
}

/// Fracties voor en tegen: het grote Blok Fractielijst met twee kolommen partijlogo's.
fn fracties_voor_en_tegen() -> Sjabloon {
    groot_stemmingspaneel(
        FRACTIES_VOOR_EN_TEGEN,
        "Fracties voor en tegen",
        "STEMMING",
        Bloksoort::Fractielijst {
            letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
            tekstletter: Letterkeuze::Stijl(Stijlletter::Tekst),
            grootte: 40,
            onder_balk: false,
        },
    )
}

/// Hoofdelijke stemming: het grote Blok Namenlijst, met per kant elke Fractie en haar aantal en
/// alleen de namen van wie anders stemde dan de eigen Fractie. De Zaal kleurt elke Stoel in de
/// stem van het Kamerlid dat er zit.
fn hoofdelijke_stemming() -> Sjabloon {
    groot_stemmingspaneel(
        HOOFDELIJKE_STEMMING,
        "Hoofdelijke stemming",
        "HOOFDELIJKE STEMMING",
        Bloksoort::Namenlijst {
            letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
            naamletter: Letterkeuze::Stijl(Stijlletter::Tekst),
            grootte: 40,
        },
    )
}

/// Een Tekst in een paneel: vervaagt erin als het paneel open is.
fn paneeltekst(p: Plek, bron: Tekstbron, letter: Stijlletter, grootte: u16, kleur: Stijlkleur, uitlijning: Uitlijning) -> Blok {
    Blok {
        plek: p,
        soort: Bloksoort::Tekst {
            bron,
            letter: Letterkeuze::Stijl(letter),
            grootte,
            uitlijning,
            een_regel: false,
        },
        kleur: Kleurkeuze::Stijl(kleur),
        animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
    }
}

/// De regel rechts in een paneel met cijfers: "150 zetels · meerderheid 76" in Hoofd.
fn meerderheid(p: Plek) -> Blok {
    paneeltekst(
        p,
        Tekstbron::Vast("150 zetels · meerderheid 76".into()),
        Stijlletter::Tekst,
        30,
        Stijlkleur::Hoofd,
        Uitlijning::Rechts,
    )
}

/// Een Zetelbalk van 52 pixels hoog: de lijn bij 76 in Accent met
/// "76" erboven, en onder elk deel de Zetels en de korte naam. Groeit vanaf de as als het
/// paneel open is.
pub fn zetelbalk_blok(p: Plek, vorige_set: bool, pijlen: bool) -> Blok {
    Blok {
        plek: p,
        soort: Bloksoort::Zetelbalk {
            letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
            lijn_bij_76: true,
            naamletter: Letterkeuze::Stijl(Stijlletter::Tekst),
            labels: true,
            vorige_set,
            pijlen,
            coalitie: false,
        },
        kleur: Kleurkeuze::Stijl(Stijlkleur::Accent),
        animatie: animatie(Erin::Opbouwen, Eruit::SluitenNaarLijn, 200),
    }
}

/// De Zetelbalk: een paneel onderin met de kicker
/// "ZETELS", de naam van de set als kop en de Zetelbalk met labels. De Stoelen kleuren zoals
/// bij de Zetelverdeling (dat doet de Zaal).
fn zetelbalk_sjabloon() -> Sjabloon {
    let [streepje, kicker_tekst] = kicker(144, 686, 1000, Tekstbron::Vast("ZETELS".into()));
    Sjabloon {
        id: ZETELBALK.into(),
        naam: "Zetelbalk".into(),
        groep: "Uitslagavond".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::Gegevens,
        // de Stoelen boven het paneel (council-ronde 3)
        standpunt: Some(crate::camera::BOVEN_HET_PANEEL.into()),
        blokken: vec![
            // het paneel is hoger dan bij de Coalitiebalk: onder de balk is ruimte voor twee
            // rijen labels
            paneel(plek(96, 646, 1728, 360)),
            streepje,
            kicker_tekst,
            paneeltekst(plek(144, 722, 1100, 80), Tekstbron::Setnaam, Stijlletter::Kop, 64, Stijlkleur::Hoofd, Uitlijning::Links),
            meerderheid(plek(1176, 682, 600, 42)),
            zetelbalk_blok(plek(144, 830, 1632, 52), false, false),
            programmabug(),
        ],
        kleurovergang: None,
    }
}

/// Twee sets naast elkaar: twee Zetelbalken onder elkaar in één paneel, de oudste set boven en
/// de nieuwste onder, met boven elke balk de naam van de set en onder de nieuwste een pijltje
/// voor het verschil. De kop is de naam van de nieuwste set; die kleurt de Stoelen.
fn twee_sets() -> Sjabloon {
    let [streepje, kicker_tekst] = kicker(144, 452, 1000, Tekstbron::Vast("VERGELIJKING".into()));
    let setnaam = |y, bron| paneeltekst(plek(144, y, 760, 42), bron, Stijlletter::Tekst, 30, Stijlkleur::Hoofd, Uitlijning::Links);
    Sjabloon {
        id: TWEE_SETS.into(),
        naam: "Twee sets naast elkaar".into(),
        groep: "Uitslagavond".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::Gegevens,
        standpunt: None,
        blokken: vec![
            paneel(plek(96, 412, 1728, 594)),
            streepje,
            kicker_tekst,
            paneeltekst(plek(144, 488, 1100, 80), Tekstbron::Setnaam, Stijlletter::Kop, 64, Stijlkleur::Hoofd, Uitlijning::Links),
            meerderheid(plek(1176, 448, 600, 42)),
            setnaam(590, Tekstbron::VorigeSetnaam),
            zetelbalk_blok(plek(144, 644, 1632, 52), true, false),
            setnaam(784, Tekstbron::Setnaam),
            zetelbalk_blok(plek(144, 832, 1632, 52), false, true),
            programmabug(),
        ],
        kleurovergang: None,
    }
}

/// Uitslag per Partij: een groot paneel links met de
/// kicker "ZETELS PER PARTIJ", de naam van de set als kop, een witte lijn en de Uitslaggrafiek.
/// De Stoelen kleuren zoals bij de Zetelverdeling.
fn uitslag_per_partij() -> Sjabloon {
    let [streepje, kicker_tekst] = kicker(148, 190, 900, Tekstbron::Vast("ZETELS PER PARTIJ".into()));
    Sjabloon {
        id: UITSLAG_PER_PARTIJ.into(),
        naam: "Uitslag per Partij".into(),
        groep: "Uitslagavond".into(),
        snelheid: 100,
        // het paneel beslaat het grootste deel van het beeld: de Zaal ernaast donker en onscherp
        achtergrond: Achtergrond::ZaalDonker,
        stoelen: Stoelen::Gegevens,
        // Bovenaf, zoals de Stemmingen achter hun grote paneel: vanaf Totaal (A) stonden naast het
        // paneel alleen de donkere studio en tribune, en die werden achter Zaal donker zwart
        // (council-ronde 7)
        standpunt: Some("G".into()),
        blokken: vec![
            // 30 px onder het programmalogo, dat er vrij boven staat (council ronde 6)
            paneel(plek(96, 150, 1728, 860)),
            streepje,
            kicker_tekst,
            paneeltekst(plek(148, 226, 820, 108), Tekstbron::Setnaam, Stijlletter::Kop, 92, Stijlkleur::Hoofd, Uitlijning::Links),
            meerderheid(plek(1172, 284, 600, 42)),
            Blok {
                plek: plek(148, 344, 1624, 2),
                soort: Bloksoort::Lijn { dekking: 55 },
                kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
                animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
            },
            Blok {
                plek: plek(96, 360, 1676, 642),
                soort: Bloksoort::Uitslaggrafiek {
                    letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                    naamletter: Letterkeuze::Stijl(Stijlletter::Tekst),
                },
                kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
                animatie: animatie(Erin::Opbouwen, Eruit::SluitenNaarLijn, 200),
            },
            programmabug(),
        ],
        kleurovergang: None,
    }
}

/// Winst en verlies: het paneel van de Uitslag
/// per Partij met de kicker "WINST EN VERLIES", de naam van de set als kop, rechts "zetels
/// tegenover" en de vergelijkingsset, een witte lijn en de Winst-en-verliesgrafiek. De Stoelen
/// kleuren zoals bij de Zetelverdeling.
fn winst_en_verlies() -> Sjabloon {
    let [streepje, kicker_tekst] = kicker(148, 190, 900, Tekstbron::Vast("WINST EN VERLIES".into()));
    Sjabloon {
        id: WINST_EN_VERLIES.into(),
        naam: "Winst en verlies".into(),
        groep: "Uitslagavond".into(),
        snelheid: 100,
        // het paneel beslaat het grootste deel van het beeld: de Zaal ernaast donker en onscherp
        achtergrond: Achtergrond::ZaalDonker,
        stoelen: Stoelen::Gegevens,
        // Bovenaf, zoals Uitslag per Partij en de Stemmingen (council-ronde 7)
        standpunt: Some("G".into()),
        blokken: vec![
            // 30 px onder het programmalogo, dat er vrij boven staat (council ronde 6)
            paneel(plek(96, 150, 1728, 860)),
            streepje,
            kicker_tekst,
            paneeltekst(plek(148, 226, 820, 108), Tekstbron::Setnaam, Stijlletter::Kop, 92, Stijlkleur::Hoofd, Uitlijning::Links),
            paneeltekst(plek(1172, 284, 600, 42), Tekstbron::ZetelsTegenover, Stijlletter::Tekst, 30, Stijlkleur::Hoofd, Uitlijning::Rechts),
            Blok {
                plek: plek(148, 344, 1624, 2),
                soort: Bloksoort::Lijn { dekking: 55 },
                kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
                animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
            },
            Blok {
                plek: plek(148, 360, 1624, 642),
                soort: Bloksoort::WinstEnVerliesgrafiek {
                    letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                    naamletter: Letterkeuze::Stijl(Stijlletter::Tekst),
                },
                kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
                animatie: animatie(Erin::Opbouwen, Eruit::SluitenNaarLijn, 200),
            },
            programmabug(),
        ],
        kleurovergang: None,
    }
}

/// Partij uitgelicht: haar Stoelen in haar kleur, de rest grijs (dat doet de Zaal). Linksboven
/// een naambalk met links groot haar Zetels en "zetels", rechts daarvan de naam van de set als
/// kicker en de naam van de Partij als kop; rechtsboven het programmalogo. Met dezelfde
/// bouwstenen als de Zetelverdeling, het grote getal zoals bij de Coalitie.
fn partij_uitgelicht() -> Sjabloon {
    let [streepje, kicker_tekst] = kicker(380, 90, 900, Tekstbron::Setnaam);
    let open = animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0);
    Sjabloon {
        id: PARTIJ_UITGELICHT.into(),
        naam: "Partij uitgelicht".into(),
        groep: "Uitslagavond".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::Gegevens,
        standpunt: None,
        blokken: vec![
            // 0: de naambalk past om de kicker en de naam
            naambalk(plek(96, 54, 1300, 212), vec![4, 5]),
            // 1 en 2: de Zetels groot, rechts uitgelijnd, met "zetels" eronder
            Blok {
                plek: plek(140, 74, 200, 124),
                soort: Bloksoort::Getal {
                    bron: Getalbron::Zetels,
                    letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                    grootte: 112,
                    uitlijning: Uitlijning::Rechts,
                },
                kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
                animatie: open,
            },
            paneeltekst(plek(140, 192, 200, 42), Tekstbron::Vast("zetels".into()), Stijlletter::Tekst, 30, Stijlkleur::Hoofd, Uitlijning::Rechts),
            // 3 en 4: de naam van de set als kicker
            streepje,
            kicker_tekst,
            // 5: de naam van de Partij
            paneeltekst(plek(380, 134, 1000, 92), Tekstbron::Partijnaam, Stijlletter::Kop, 76, Stijlkleur::Hoofd, Uitlijning::Links),
            programmabug(),
        ],
        kleurovergang: None,
    }
}

/// De teller van de Coalitiebouwer, rechts in een paneel:
/// "zetels samen", daaronder het tekort voor 76 in Accent, en rechts het totaal in de
/// cijferletter. `rechts` is de rechterrand van het totaal, `y` de bovenkant van "zetels samen".
fn coalitieteller(rechts: i32, y: i32) -> [Blok; 3] {
    let regels_rechts = rechts - 160 - 22;
    [
        paneeltekst(plek(regels_rechts - 600, y, 600, 46), Tekstbron::Vast("zetels samen".into()), Stijlletter::Tekst, 34, Stijlkleur::Tekst, Uitlijning::Rechts),
        paneeltekst(plek(regels_rechts - 600, y + 46, 600, 44), Tekstbron::Coalitieregel, Stijlletter::Cijfers, 32, Stijlkleur::Accent, Uitlijning::Rechts),
        Blok {
            plek: plek(rechts - 160, y - 34, 160, 160),
            soort: Bloksoort::Getal {
                bron: Getalbron::Coalitie,
                letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                grootte: 132,
                uitlijning: Uitlijning::Rechts,
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
            animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
        },
    ]
}

/// De Partijen van de coalitie als kop in de kopletter, op één regel.
fn coalitiekop(p: Plek) -> Blok {
    Blok {
        plek: p,
        soort: Bloksoort::Tekst {
            bron: Tekstbron::Coalitiepartijen,
            letter: Letterkeuze::Stijl(Stijlletter::Kop),
            grootte: 64,
            uitlijning: Uitlijning::Links,
            een_regel: true,
        },
        kleur: Kleurkeuze::Stijl(Stijlkleur::Hoofd),
        animatie: animatie(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0),
    }
}

/// Coalitie in de Zaal: de Stoelen tonen de coalitie als één blok links met de rest donker (dat
/// doet de Zaal, met de lijn bij 76). Linksboven een naambalk met de kicker "COALITIE" en de
/// Partijen, rechtsonder een paneel met de teller.
fn coalitie_in_de_zaal() -> Sjabloon {
    let [streepje, kicker_tekst] = kicker(140, 84, 1000, Tekstbron::Vast("COALITIE".into()));
    let mut blokken = vec![
        naambalk(plek(96, 54, 1100, 166), vec![2, 3]),
        streepje,
        kicker_tekst,
        coalitiekop(plek(140, 122, 1010, 76)),
        programmabug(),
        paneel(plek(1224, 846, 600, 160)),
    ];
    blokken.extend(coalitieteller(1776, 880));
    Sjabloon {
        id: COALITIE.into(),
        naam: "Coalitie in de Zaal".into(),
        groep: "Coalitiebouwer".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::Gegevens,
        standpunt: None,
        blokken,
        kleurovergang: None,
    }
}

/// De Coalitiebalk: het paneel van de Zetelbalk met
/// de kicker "COALITIE", de Partijen als kop, rechts de teller, en de Zetelbalk met de coalitie
/// vooraan en de lijn bij 76.
fn coalitiebalk() -> Sjabloon {
    let [streepje, kicker_tekst] = kicker(144, 716, 1000, Tekstbron::Vast("COALITIE".into()));
    let mut balk = zetelbalk_blok(plek(144, 872, 1632, 52), false, false);
    if let Bloksoort::Zetelbalk { coalitie, .. } = &mut balk.soort {
        *coalitie = true;
    }
    let mut blokken = vec![
        paneel(plek(96, 676, 1728, 330)),
        streepje,
        kicker_tekst,
        coalitiekop(plek(144, 752, 1150, 80)),
    ];
    blokken.extend(coalitieteller(1776, 734));
    blokken.extend([balk, programmabug()]);
    Sjabloon {
        id: COALITIEBALK.into(),
        naam: "Coalitiebalk".into(),
        groep: "Coalitiebouwer".into(),
        snelheid: 100,
        achtergrond: Achtergrond::Zaal,
        stoelen: Stoelen::Gegevens,
        // de Stoelen boven het paneel (council-ronde 3)
        standpunt: Some(crate::camera::BOVEN_HET_PANEEL.into()),
        blokken,
        kleurovergang: None,
    }
}

/// Of een bestandsnaam bij de server veilig is: delen van letters, cijfers, `.`, `-` en `_`,
/// gescheiden door `/`, en geen deel dat met een punt begint. Zo blijft elk bestand in de map
/// van de server en in de kopie van de Renderer.
pub fn geldige_bestandsnaam(naam: &str) -> bool {
    !naam.is_empty()
        && naam.len() <= 200
        && naam.split('/').all(|deel| {
            !deel.is_empty()
                && !deel.starts_with('.')
                && deel
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        })
}

/// Het begin van het `id` van een eigen Sjabloon, zoals `eigen-opkomst`.
pub const EIGEN: &str = "eigen-";
/// Langste naam van een Sjabloon, in tekens.
pub const MAX_SJABLOONNAAM: usize = 40;
/// Het grootste aantal Blokken in een Sjabloon.
pub const MAX_BLOKKEN: usize = 200;
/// De groep van een eigen Sjabloon in de lijsten van het dashboard.
pub const GROEP_EIGEN: &str = "Eigen";

/// Een Sjabloon dat de omroep in de vormgevingstab bewerkte: een aangepast vast Sjabloon (met
/// hetzelfde `id` als het vaste) of een eigen Sjabloon (`id` begint met [`EIGEN`]).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct BewerktSjabloon {
    pub sjabloon: Sjabloon,
    /// Bij een aangepast vast Sjabloon: de [`vingerafdruk`] van de standaard waar het op
    /// gebaseerd is. Zonder: een eigen Sjabloon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub standaard: Option<String>,
}

impl BewerktSjabloon {
    /// Of de software sinds het aanpassen met een andere standaard kwam. De aanpassing blijft;
    /// de vormgevingstab meldt het, met een knop om de nieuwe standaard over te nemen.
    pub fn nieuwe_standaard(&self) -> bool {
        match (&self.standaard, vast_sjabloon(&self.sjabloon.id)) {
            (Some(basis), Some(vast)) => *basis != vingerafdruk(&vast),
            _ => false,
        }
    }
}

/// Een vingerafdruk van een Sjabloon: 16 tekens, gelijk zolang het Sjabloon gelijk is. Een
/// FNV-1a-hash van de JSON, dus vast, ook na een nieuwe versie van Rust.
pub fn vingerafdruk(s: &Sjabloon) -> String {
    let json = serde_json::to_vec(s).expect("een Sjabloon is altijd JSON");
    let hash = json
        .iter()
        .fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3));
    format!("{hash:016x}")
}

/// Of dit het `id` van een vast Sjabloon is.
pub fn is_vast(id: &str) -> bool {
    vaste_sjablonen().iter().any(|s| s.id == id)
}

/// Of dit een geldig `id` voor een eigen Sjabloon is: [`EIGEN`] met daarachter kleine letters,
/// cijfers en `-`, samen hooguit 60 tekens.
pub fn is_eigen_id(id: &str) -> bool {
    id.strip_prefix(EIGEN).is_some_and(|rest| {
        !rest.is_empty()
            && id.len() <= 60
            && rest.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    })
}

/// Waarom de server een Sjabloon niet opslaat.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum SjabloonWeigering {
    #[error("{0:?} is geen vast Sjabloon en geen eigen Sjabloon")]
    Id(String),
    #[error("het Sjabloon heeft geen naam")]
    NaamLeeg,
    #[error("de naam van het Sjabloon is langer dan {MAX_SJABLOONNAAM} tekens")]
    NaamTeLang,
    #[error("de snelheid is {0} procent; kies tussen {MIN_SNELHEID} en {MAX_SNELHEID}")]
    Snelheid(u16),
    #[error("de Kleurovergang duurt {0} ms; kies tussen {MIN_KLEUROVERGANG_MS} en {MAX_KLEUROVERGANG_MS}")]
    Kleurovergang(u16),
    #[error("meer dan {MAX_BLOKKEN} Blokken")]
    TeVeelBlokken,
    #[error("geen geldige bestandsnaam: {0:?}")]
    Bestandsnaam(String),
    #[error("Blok {0} verwijst naar een Blok dat er niet is")]
    Verwijzing(usize),
    #[error("Blok {0} heeft een invulveld zonder naam")]
    VeldZonderNaam(usize),
}

impl Sjabloon {
    /// Of dit Sjabloon kan: een vast of eigen `id`, een naam, een snelheid tussen
    /// [`MIN_SNELHEID`] en [`MAX_SNELHEID`], geldige bestandsnamen, invulvelden met een naam en
    /// verwijzingen naar Blokken die er zijn. Het Standpunt controleert de Stand.
    pub fn controleer(&self) -> Result<(), SjabloonWeigering> {
        if !is_vast(&self.id) && !is_eigen_id(&self.id) {
            return Err(SjabloonWeigering::Id(self.id.clone()));
        }
        if self.naam.trim().is_empty() {
            return Err(SjabloonWeigering::NaamLeeg);
        }
        if self.naam.chars().count() > MAX_SJABLOONNAAM {
            return Err(SjabloonWeigering::NaamTeLang);
        }
        if !(MIN_SNELHEID..=MAX_SNELHEID).contains(&self.snelheid) {
            return Err(SjabloonWeigering::Snelheid(self.snelheid));
        }
        if let Some(k) = self.kleurovergang.filter(|k| !k.kan()) {
            return Err(SjabloonWeigering::Kleurovergang(k.duur_ms));
        }
        if self.blokken.len() > MAX_BLOKKEN {
            return Err(SjabloonWeigering::TeVeelBlokken);
        }
        let mut bestanden: Vec<&str> = Vec::new();
        if let Achtergrond::Afbeelding { bestand } = &self.achtergrond {
            bestanden.push(bestand);
        }
        for (n, b) in self.blokken.iter().enumerate() {
            let letters = [b.soort.letter(), tweede_letter(&b.soort)];
            bestanden.extend(letters.into_iter().flatten().filter_map(|l| match l {
                Letterkeuze::Eigen(naam) => Some(naam.as_str()),
                Letterkeuze::Stijl(_) => None,
            }));
            let verwijst = match &b.soort {
                Bloksoort::Vlak { om_tekst, bij_tekst, .. } => {
                    om_tekst.iter().flat_map(|o| o.blokken.iter()).chain(bij_tekst).copied().collect()
                }
                _ => Vec::new(),
            };
            if verwijst.iter().any(|&i| usize::from(i) >= self.blokken.len()) {
                return Err(SjabloonWeigering::Verwijzing(n));
            }
            match &b.soort {
                Bloksoort::Afbeelding { bestand, .. } => bestanden.push(bestand),
                Bloksoort::Tekst { bron: Tekstbron::Veld(v), .. }
                | Bloksoort::Partijregel { tekst: Tekstbron::Veld(v), .. }
                | Bloksoort::Getal { bron: Getalbron::Veld(v), .. }
                    if v.trim().is_empty() =>
                {
                    return Err(SjabloonWeigering::VeldZonderNaam(n));
                }
                _ => {}
            }
        }
        if let Some(naam) = bestanden.into_iter().find(|n| !geldige_bestandsnaam(n)) {
            return Err(SjabloonWeigering::Bestandsnaam(naam.into()));
        }
        Ok(())
    }

    /// Het Sjabloon met zijn achtergrond als eerste Blok, zoals de Renderer het tekent: een
    /// donker Vlak, een Vlak in de kleur of een Afbeelding over het hele beeld, dat vervaagt.
    /// Verwijzingen naar Blokken schuiven mee. Bij de Zaal: het Sjabloon zelf.
    pub fn met_achtergrond(&self) -> Sjabloon {
        let soort_en_kleur = match &self.achtergrond {
            Achtergrond::Zaal => return self.clone(),
            Achtergrond::ZaalDonker => (vlak(DONKER), Kleurkeuze::Eigen(Kleur { r: 0, g: 0, b: 0 })),
            Achtergrond::Kleur { kleur } => (vlak(100), *kleur),
            Achtergrond::Afbeelding { bestand } => (
                Bloksoort::Afbeelding { bestand: bestand.clone(), dekking: 100 },
                Kleurkeuze::Stijl(Stijlkleur::Achtergrond),
            ),
        };
        let mut s = self.clone();
        for b in &mut s.blokken {
            if let Bloksoort::Vlak { om_tekst, bij_tekst, .. } = &mut b.soort {
                om_tekst.iter_mut().flat_map(|o| o.blokken.iter_mut()).for_each(|i| *i += 1);
                if let Some(i) = bij_tekst {
                    *i += 1;
                }
            }
        }
        let (soort, kleur) = soort_en_kleur;
        s.blokken.insert(
            0,
            Blok {
                plek: plek(0, 0, 1920, 1080),
                soort,
                kleur,
                animatie: animatie(Erin::Vervagen, Eruit::Vervagen, 0),
            },
        );
        s
    }

    /// De bestanden die dit Sjabloon zelf noemt: eigen lettertypes en afbeeldingen. Lettertypes
    /// uit de Stijl staan in [`Sjabloon::lettertypes`].
    pub fn eigen_bestanden(&self) -> Vec<String> {
        let mut uit = Vec::new();
        if let Achtergrond::Afbeelding { bestand } = &self.achtergrond {
            uit.push(bestand.clone());
        }
        for b in &self.blokken {
            if let Bloksoort::Afbeelding { bestand, .. } = &b.soort {
                uit.push(bestand.clone());
            }
            for l in [b.soort.letter(), tweede_letter(&b.soort)].into_iter().flatten() {
                if let Letterkeuze::Eigen(naam) = l {
                    uit.push(naam.clone());
                }
            }
        }
        uit
    }
}

/// Het tweede lettertype van een Blok, als het er een heeft.
fn tweede_letter(soort: &Bloksoort) -> Option<&Letterkeuze> {
    match soort {
        Bloksoort::Partijlabels { naamletter: l, .. }
        | Bloksoort::Opbouwlabel { naamletter: l, .. }
        | Bloksoort::Uitslaggrafiek { naamletter: l, .. }
        | Bloksoort::WinstEnVerliesgrafiek { naamletter: l, .. }
        | Bloksoort::Zetelbalk { naamletter: l, .. }
        | Bloksoort::Namenlijst { naamletter: l, .. }
        | Bloksoort::Stemmingsbalk { tekstletter: l, .. }
        | Bloksoort::Fractielijst { tekstletter: l, .. }
        | Bloksoort::Partijregel { tekstletter: l, .. } => Some(l),
        _ => None,
    }
}
