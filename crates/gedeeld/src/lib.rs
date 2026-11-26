//! De Stand, de Opdrachten en de berichten die server, dashboard en Renderers uitwisselen.
//!
//! De server is de enige bron van de Stand (ADR 0001). Elke Opdracht gaat naar de server;
//! die stuurt bij elke wijziging de hele Stand met een volgnummer naar elk scherm.
//! De TypeScript-typen van het dashboard komen uit deze crate (`cargo test -p gedeeld`
//! schrijft ze naar `dashboard/src/typen`).

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod camera;
pub mod camerarit;
pub mod cijfers;
pub mod coalitie;
mod graphic;
pub mod kamer;
pub mod persoonskaart;
pub mod presets;
pub mod sjabloon;
pub mod stoelbeeld;
mod tablet;
pub mod stemmingen;
pub mod uitslagindeling;
pub mod vaste_ritten;
pub mod zaalindeling;
pub mod zaalkaart;
pub mod zetelbalk;
#[cfg(feature = "volger")]
pub mod volger;

use crate::camera::{
    CameraOvergang, Camerabeeld, Camerakeuze, Camerastand, MAX_STANDPUNTNAAM, NaAankomst,
    OVERVLOEI_MAX_MS, OVERVLOEI_MIN_MS, OvergangSoort, Standpunt,
};
use crate::camerarit::{Bijschaving, Camerarit, RIT_MIN_S};
use crate::cijfers::{Aanvulling, Bewerking, CijferWeigering, Cijfers, Invoer, Soort};
pub use graphic::{
    Graphic, GraphicInBeeld, Indeling, KlaargezetteGraphic, MAX_KOP, MAX_REGEL, Overgang, VELD_FUNCTIE,
    VELD_GEGEVENS, VELD_NAMEN, VELD_VRIJE_REGEL, VELD_ZETELS,
};
use crate::persoonskaart::{EigenPersoon, Extra};
use crate::kamer::{Foto, Fractie, Kamer, KamerWeigering, Kamerlid};
use crate::presets::Preset;
use crate::stemmingen::{Lijstpunt, OpgehaaldeUitslag, Stemgedrag, StemmingWeigering, Stemmingen};
pub use crate::stoelbeeld::{Stoelbeeld, Stoelbron};
use crate::uitslagindeling::{Uitslagvolgorde, Volgorde, Zetelstand};
use crate::zaalindeling::{ZaalWeigering, Zaalindeling, Zaalmelding};

/// Versie van het systeem: de versie uit Cargo.toml plus de commit, zoals `0.1.0+4a46080` (zie
/// `build.rs`). De server weigert een Renderer met een andere versie.
pub const VERSIE: &str = env!("UITSLAGAVOND_VERSIE");

/// Alles wat op een moment vastligt over de uitzending.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[ts(export)]
pub struct Stand {
    /// De Graphic die klaarstaat, te zien in het Voorbeeld.
    #[serde(default)]
    pub klaar: Option<KlaargezetteGraphic>,
    /// De Graphic in de Ether.
    #[serde(default)]
    pub ether: Option<GraphicInBeeld>,
    /// De kleur van elke Stoel, in de volgorde van [`Stoel::alle`]. Leeg: elke Stoel houdt
    /// zijn eigen kleur (het blauwe leer).
    #[serde(default)]
    #[ts(type = "Array<string>")]
    pub stoelkleuren: Vec<Kleur>,
    /// Hoe de Stoelen naar `stoelkleuren` gaan: de Kleurovergang van het Sjabloon van de Graphic
    /// die ze kleurde, of die van de Stijl. De server zet hem bij elke nieuwe kleur, zodat Ether
    /// en Reserve dezelfde overgang tonen.
    #[serde(default)]
    pub kleurovergang: sjabloon::Kleurovergang,
    /// De partijlijst en de cijfersets.
    #[serde(default)]
    pub cijfers: Cijfers,
    /// De Kamer die zit: Kamerleden en Fracties uit de open data.
    #[serde(default)]
    pub kamer: Kamer,
    /// De Stijl van het programma. Zonder Stijl geldt de standaardstijl.
    #[serde(default)]
    pub stijl: sjabloon::Stijl,
    /// De Stemmingslijst van de dag en wat de Regie erbij invoerde.
    #[serde(default)]
    pub stemmingen: Stemmingen,
    /// De Standpunten, eerst de zes vaste.
    #[serde(default = "Standpunt::vaste")]
    pub standpunten: Vec<Standpunt>,
    /// De Cameraritten: eerst de zes vaste ([`vaste_ritten`]), dan de eigen op volgorde van
    /// opnemen.
    #[serde(default = "vaste_ritten::vaste")]
    pub cameraritten: Vec<Camerarit>,
    /// Waar de Regie de camera van de Ether heen stuurde. De server rekent de beweging uit.
    #[serde(default)]
    pub camera: Camerakeuze,
    /// De Presets, op nummer.
    #[serde(default)]
    pub presets: Vec<Preset>,
    /// De volgorde van de Partijen in de Uitslagindeling, vóór de uitzending ingesteld.
    #[serde(default)]
    pub uitslagvolgorde: Uitslagvolgorde,
    /// Waar elke Partij van de partijlijst nu staat: uitgerekend uit `uitslagvolgorde` en de
    /// partijlijst, na elke Opdracht. Het dashboard gebruikt deze volgorde overal.
    #[serde(default)]
    pub uitslagindeling: Volgorde,
    /// Welk Kamerlid op welke Stoel zit, uit het bestand van de Kamerperiode.
    #[serde(default)]
    pub zaalindeling: Zaalindeling,
    /// Wat er niet klopt aan de Zaalindeling en de voorstellen bij een wissel: uitgerekend na
    /// elke Opdracht.
    #[serde(default)]
    pub zaalmelding: Zaalmelding,
    /// De personen die de Regie zelf toevoegde voor de Persoonskaart.
    #[serde(default)]
    pub personen: Vec<EigenPersoon>,
    /// Welke extra gegevens op een nieuwe Persoonskaart standaard aan staan.
    #[serde(default)]
    pub persoonskaart_extra: Extra,
    /// De Sjablonen die de omroep bewerkte: aangepaste vaste Sjablonen en eigen Sjablonen. Een
    /// vast Sjabloon dat hier niet staat, is de standaard van deze versie van de software.
    #[serde(default)]
    pub sjablonen: Vec<sjabloon::BewerktSjabloon>,
    /// Een los klaargezet Stoelbeeld in het Voorbeeld (ADR 0005).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub klaar_stoelbeeld: Option<Stoelbeeld>,
    /// De uitgerekende 150 stoelkleuren van het klaargezette Stoelbeeld, voor de Preview-renderer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(type = "Array<string>")]
    pub klaar_stoelkleuren: Vec<Kleur>,
    /// Het actieve losse Stoelbeeld in de Ether, voor weergave op de Live-laagchip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ether_stoelbeeld: Option<Stoelbeeld>,
    /// Eventuele eigen Kleurkracht van het actieve Stoelbeeld in de Ether (0..=100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ether_kleurkracht: Option<u8>,
}

impl Default for Stand {
    fn default() -> Self {
        Stand {
            klaar: None,
            ether: None,
            stoelkleuren: Vec::new(),
            kleurovergang: sjabloon::Kleurovergang::default(),
            cijfers: Cijfers::default(),
            kamer: Kamer::default(),
            stijl: sjabloon::Stijl::default(),
            standpunten: Standpunt::vaste(),
            cameraritten: vaste_ritten::vaste(),
            camera: Camerakeuze::default(),
            stemmingen: Stemmingen::default(),
            presets: Vec::new(),
            uitslagvolgorde: Uitslagvolgorde::default(),
            uitslagindeling: Volgorde::default(),
            zaalindeling: Zaalindeling::default(),
            zaalmelding: Zaalmelding::default(),
            personen: Vec::new(),
            persoonskaart_extra: Extra::default(),
            sjablonen: Vec::new(),
            klaar_stoelbeeld: None,
            klaar_stoelkleuren: Vec::new(),
            ether_stoelbeeld: None,
            ether_kleurkracht: None,
        }
    }
}

/// Aantal Stoelen in de Zaal.
pub const AANTAL_STOELEN: usize = 150;

/// Hoe lang de rest van een overgeslagen Opbouw duurt, in milliseconden ([`Opdracht::SlaOpbouwOver`]).
pub const OPBOUW_OVERSLAAN_MS: u16 = 500;

/// Stoelen per rij in elk vak, van voor (rij 1) naar achter (rij 6).
pub const STOELEN_PER_RIJ: [u8; 6] = [2, 3, 4, 5, 5, 6];

/// Een Stoel in de Zaal, met zijn vaste plek. Vak 1 staat rechts in beeld vanaf de
/// Voorzitter (#7). Dezelfde nummering als het model (`Stoel_<vak>_<rij>_<nummer>`).
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[ts(export)]
pub struct Stoel {
    pub vak: u8,
    pub rij: u8,
    pub nummer: u8,
}

impl Stoel {
    /// Alle 150 Stoelen: vak na vak, in elk vak rij na rij, in elke rij op nummer.
    pub fn alle() -> impl Iterator<Item = Stoel> {
        (1..=6u8).flat_map(|vak| {
            STOELEN_PER_RIJ.iter().zip(1u8..).flat_map(move |(&n, rij)| {
                (1..=n).map(move |nummer| Stoel { vak, rij, nummer })
            })
        })
    }

    /// Plaats van deze Stoel in [`Stoel::alle`] en dus in [`Stand::stoelkleuren`].
    /// `None` als de Stoel niet in de Zaal bestaat.
    pub fn index(&self) -> Option<usize> {
        let per_vak: usize = STOELEN_PER_RIJ.iter().map(|&n| n as usize).sum();
        let rijen = STOELEN_PER_RIJ.get(usize::from(self.rij).checked_sub(1)?)?;
        if !(1..=6).contains(&self.vak) || self.nummer == 0 || self.nummer > *rijen {
            return None;
        }
        let voor: usize = STOELEN_PER_RIJ[..usize::from(self.rij) - 1]
            .iter()
            .map(|&n| n as usize)
            .sum();
        Some(usize::from(self.vak - 1) * per_vak + voor + usize::from(self.nummer) - 1)
    }
}

/// Een kleur in sRGB. Reist als hexcode, zoals `"#d4202c"`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct Kleur {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Een tekst is geen kleur van de vorm `#rrggbb`.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("geen kleur van de vorm #rrggbb: {0:?}")]
pub struct GeenKleur(String);

impl std::str::FromStr for Kleur {
    type Err = GeenKleur;
    fn from_str(s: &str) -> Result<Self, GeenKleur> {
        let fout = || GeenKleur(s.to_string());
        let hex = s.strip_prefix('#').filter(|h| h.len() == 6).ok_or_else(fout)?;
        let deel = |i: usize| u8::from_str_radix(hex.get(i..i + 2).ok_or_else(fout)?, 16).map_err(|_| fout());
        Ok(Kleur { r: deel(0)?, g: deel(2)?, b: deel(4)? })
    }
}

impl TryFrom<String> for Kleur {
    type Error = GeenKleur;
    fn try_from(s: String) -> Result<Self, GeenKleur> {
        s.parse()
    }
}

impl From<Kleur> for String {
    fn from(k: Kleur) -> String {
        format!("#{:02x}{:02x}{:02x}", k.r, k.g, k.b)
    }
}

/// Eén handeling die de Stand verandert.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "soort")]
#[ts(export)]
pub enum Opdracht {
    /// Zet een nieuwe Graphic klaar, in plaats van wat er klaarstond. Zonder `standpunt` gaat de
    /// camera bij Toon naar het Standpunt van het Sjabloon.
    ZetKlaar {
        graphic: Graphic,
        #[serde(default)]
        #[ts(optional)]
        standpunt: Option<String>,
    },
    /// Wijzigt de klaargezette Graphic met dit id. Staat die al in de Ether, dan blijft de
    /// Ether gelijk tot Bijwerken.
    WijzigKlaar {
        #[ts(type = "number")]
        id: u64,
        graphic: Graphic,
        #[serde(default)]
        #[ts(optional)]
        standpunt: Option<String>,
    },
    /// Zet de klaargezette Graphic in de Ether. `versie` is de versie die het scherm liet zien;
    /// is die intussen veranderd, dan weigert de server. Zonder versie (een eenvoudige knop)
    /// toont hij wat klaarstaat.
    Toon {
        #[ts(optional, type = "number")]
        versie: Option<u64>,
    },
    /// Neemt de nieuwere versie van de Graphic in de Ether over, zonder wissel.
    Bijwerken,
    /// De Coalitiebouwer: de gekozen Partijen van de Coalitie (of Coalitiebalk) die klaarstaat en
    /// die in de Ether staat. Uitzondering op Bijwerken (besluit #6): de Ether werkt direct bij,
    /// en de Renderer schuift vloeiend naar de nieuwe coalitie. De cijfers blijven de kopie van
    /// de Graphic. Ook de Presentator kiest zo Partijen op de tablet.
    ZetCoalitie { partijen: Vec<String> },
    /// Haalt de Graphic uit de Ether.
    UitBeeld,
    /// Ongedaan maken van Uit beeld: zet precies deze Graphic terug in de Ether, met de cijfers
    /// die erin zitten. Wat klaarstaat blijft gelijk. Alleen als de Ether leeg is; de camera
    /// beweegt zoals bij Toon.
    TerugInBeeld {
        graphic: Graphic,
        #[serde(default)]
        #[ts(optional)]
        standpunt: Option<String>,
    },
    /// Kleur uit: de Stoelen in de Ether krijgen direct hun eigen leer terug, met de
    /// Kleurovergang van de Stijl. De Graphic blijft staan; de volgende Graphic die de Stoelen
    /// kleurt, zet de kleuren weer aan.
    KleurUit,
    /// Slaat de Opbouw over: de Partijen die nog niet aan de beurt waren, kleuren meteen mee
    /// (Vervagen, kort), en de labels gaan direct naar hun eindbeeld.
    SlaOpbouwOver,
    /// Stuurt de camera van de Ether naar een Standpunt of Camerarit: bewegen zoals het
    /// Standpunt zegt, of met `knip` meteen (Shift met de letter).
    KiesStandpunt {
        letter: String,
        #[serde(default)]
        knip: bool,
    },
    /// Zet de Vrije camera aan of uit (V op het dashboard, Start op de gamepad). Met de Vrije
    /// camera aan stuurt de gamepad de camera van de Ether; uit blijft de camera staan.
    ZetVrijeCamera { aan: bool },
    /// Zet Oefenen aan of uit (O): de gamepad stuurt dan alleen de camera van het Voorbeeld, de
    /// Ether blijft staan. Oefenen en de Vrije camera gaan niet samen.
    ZetOefenen { aan: bool },
    /// Start of stopt de opname van een Camerarit (R op het dashboard, B op de gamepad). Alleen
    /// tijdens Oefenen. Met `voor` neemt de Regie die Camerarit opnieuw op. Bij stoppen vult de
    /// server `opname` in met wat hij opnam; de nieuwe Camerarit speelt meteen af in het Voorbeeld.
    ZetOpnemen {
        aan: bool,
        #[serde(default)]
        #[ts(optional)]
        voor: Option<String>,
        #[serde(default)]
        #[ts(optional, type = "Array<[number, number, number, number, number, number]>")]
        opname: Option<Vec<[f32; 6]>>,
    },
    /// Speelt een Standpunt of Camerarit af in het Voorbeeld, vanaf waar de camera van de Ether
    /// nu staat ("Probeer in Voorbeeld", "Speel af in Voorbeeld"). De Ether blijft staan.
    ProbeerInVoorbeeld { letter: String },
    /// Stelt in hoe de camera naar een Standpunt gaat en wat hij daarna doet. Speelt af in het
    /// Voorbeeld.
    ZetStandpuntOvergang { letter: String, overgang: CameraOvergang, na_aankomst: NaAankomst },
    /// Zet de schuifjes van een Camerarit. Speelt af in het Voorbeeld.
    BijschaafCamerarit { letter: String, bijschaving: Bijschaving },
    /// Zet een vaste Camerarit terug naar zijn standaard: naam en schuifjes. Speelt af in het
    /// Voorbeeld.
    HerstelCamerarit { letter: String },
    /// Bewaart een nieuw eigen Standpunt. Zonder `stand` vult de server in waar de camera van de
    /// Ether nu staat.
    BewaarStandpunt {
        naam: String,
        #[serde(default)]
        #[ts(optional)]
        stand: Option<Camerastand>,
    },
    /// Zet een Standpunt (ook een vast) op een nieuwe plek; zonder `stand` waar de camera van de
    /// Ether nu staat.
    OverschrijfStandpunt {
        letter: String,
        #[serde(default)]
        #[ts(optional)]
        stand: Option<Camerastand>,
    },
    /// Geeft een eigen Standpunt of een Camerarit een andere naam.
    HernoemStandpunt { letter: String, naam: String },
    /// Geeft een eigen Standpunt of een Camerarit toets J, K of L, of geen. Had een ander die
    /// toets, dan raakt dat hem kwijt.
    ZetStandpuntToets {
        letter: String,
        #[serde(default)]
        #[ts(optional)]
        toets: Option<String>,
    },
    /// Wist een eigen Standpunt of een Camerarit. Kan niet zolang iets het gebruikt (zie
    /// [`Stand::gebruik_van_standpunt`]).
    WisStandpunt { letter: String },
    /// Geef alle 150 Stoelen tegelijk een kleur, in de volgorde van [`Stoel::alle`].
    /// Een lege lijst geeft elke Stoel zijn eigen kleur terug.
    ZetStoelkleuren {
        #[ts(type = "Array<string>")]
        kleuren: Vec<Kleur>,
    },

    // Partijen en cijfersets (zie de module `cijfers`).
    /// Leest het invoerbestand in. De server stuurt deze zelf als het bestand verandert.
    LeesInvoer {
        bestand: String,
        ontvangen: String,
        invoer: Invoer,
    },
    /// Zet de Zetels van één Partij in een set.
    ZetZetels { set: String, partij: String, zetels: u16 },
    /// Zet een set op slot: daarna weigert de server elke wijziging, ook uit het bestand.
    VergrendelSet { set: String },
    OntgrendelSet { set: String },
    /// Koppelt een Voorganger aan een Partij.
    KoppelVoorganger { partij: String, voorganger: String },
    OntkoppelVoorganger { partij: String, voorganger: String },
    /// Zet live een nieuwe Partij in de partijlijst. Zonder kleur krijgt ze een tijdelijke kleur.
    VoegPartijToe {
        naam: String,
        kort: String,
        #[serde(default)]
        #[ts(optional)]
        kleur: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        logo: Option<String>,
        #[serde(default)]
        fractie_zonder_lijst: bool,
    },
    /// Vult een Partij aan: alleen de velden die meegaan veranderen.
    VulPartijAan { partij: String, aanvulling: Aanvulling },
    /// Maakt een nieuwe, lege set van deze soort (peiling, exitpoll enz.).
    MaakSet { setsoort: Soort, label: String },
    /// Kiest de set waartegen winst en verlies wordt berekend. Zonder `set`: geen.
    KiesVergelijkingsset {
        #[serde(default)]
        #[ts(optional)]
        set: Option<String>,
    },

    // De Kamer die zit (zie de module `kamer`).
    /// Neemt de Kamer over zoals de server die uit de open data haalde. De server stuurt deze zelf.
    LeesKamer {
        opgehaald: String,
        peildatum: String,
        kamerleden: Vec<Kamerlid>,
        fracties: Vec<Fractie>,
    },
    /// Welke foto de server voor een Kamerlid heeft, zoals na het plaatsen van een eigen foto.
    ZetFoto { kamerlid: String, foto: Foto },

    // Stemmingen (zie de module `stemmingen`).
    /// Neemt de Stemmingslijst van een dag over. De server stuurt deze zelf.
    LeesStemmingslijst { dag: String, ingelezen: String, punten: Vec<Lijstpunt> },
    /// Een motie die niet in de lijst staat: de Regie typt de titel.
    VoegStemmingToe { titel: String },
    /// Vinkt een Stemming aan (of uit) voor de uitzending.
    ZetVoorUitzending { stemming: String, aan: bool },
    /// Het stemgedrag zoals de Regie het nu heeft ingevoerd. De server rekent de uitkomst uit.
    ZetStemgedrag { stemming: String, stemgedrag: Stemgedrag },
    VergrendelStemming { stemming: String },
    OntgrendelStemming { stemming: String },
    /// "Vastleggen en klaarzetten": zet de uitslag klaar als Graphic. Staat dezelfde Stemming al
    /// klaar, dan wijzigt die Graphic, zodat Bijwerken de Ether bijwerkt.
    LegStemmingVast { stemming: String },
    /// Waar een Stemming de Stoelen kleurt: in de Zaalindeling (standaard) of, tussen de
    /// beëdiging en een nieuwe Zaalindeling, in de Uitslagindeling met elke Fractie als één blok.
    ZetStemmingIndeling { indeling: Indeling },
    /// De officiële uitslagen die de server ophaalde (Controle na de uitzending). De server stuurt
    /// deze zelf. De invoer van de Regie verandert niet.
    LeesOfficieleUitslagen { opgehaald: String, uitslagen: Vec<OpgehaaldeUitslag> },
    /// "Neem officieel over": het officiële stemgedrag wordt de invoer van de Stemming. Een
    /// Graphic, ook in een Preset of de Ether, houdt de oude gegevens.
    NeemOfficieelOver { stemming: String },
    /// "Onze invoer klopt": de invoer blijft, het verschil met de officiële uitslag is afgehandeld.
    OnzeInvoerKlopt { stemming: String },

    // Presets (zie de module `presets`).
    /// Bewaart een Graphic als Preset, met het eerste vrije nummer. De Preset is een kopie: hij
    /// werkt zichzelf niet bij.
    BewaarPreset {
        graphic: Graphic,
        #[serde(default)]
        #[ts(optional)]
        standpunt: Option<String>,
    },
    WisPreset { nummer: u16 },
    /// Zet de Graphic van de Preset klaar, als nieuwe Graphic. Toon zet hem pas in de Ether.
    ZetPresetKlaar { nummer: u16 },
    /// Zet de Preset op de tablet van de Presentator, of haalt hem eraf.
    ZetPresetOpTablet { nummer: u16, aan: bool },

    // De Uitslagindeling (zie de module `uitslagindeling`).
    /// Zet de volgorde van de Partijen in de Uitslagindeling, vóór de uitzending.
    ZetUitslagvolgorde { volgorde: Uitslagvolgorde },

    // De Zaalindeling (zie de module `zaalindeling`).
    /// Leest het bestand met de Zaalindeling in. De server stuurt deze zelf als het bestand
    /// verandert.
    LeesZaalindeling { bestand: String, zaalindeling: Zaalindeling },
    /// Geeft een Kamerlid een Stoel, zoals bij het bevestigen van een voorstel na een wissel.
    GeefStoel { kamerlid: String, stoel: Stoel },

    // De Persoonskaart (zie de module `persoonskaart`).
    /// Bewaart een eigen persoon: zonder id een nieuwe, met id een wijziging.
    BewaarPersoon { persoon: EigenPersoon },
    WisPersoon { persoon: String },
    /// Of de server een foto van deze eigen persoon heeft, zoals na het plaatsen (`PUT
    /// /api/foto/<id>`). De server stuurt deze zelf.
    ZetFotoPersoon { persoon: String, foto: bool },
    /// Welke extra gegevens op een nieuwe Persoonskaart standaard aan staan (vormgevingstab).
    ZetStandaardExtra { extra: Extra },

    // De Stijl (zie de module `sjabloon`).
    /// Opslaan in de vormgevingstab: de nieuwe Stijl. De Graphic in de Ether houdt haar vorm tot
    /// de volgende Toon; de klaargezette Graphic krijgt de nieuwe vorm en telt als net veranderd.
    ZetStijl { stijl: sjabloon::Stijl },

    // Sjablonen bewerken in de vormgevingstab.
    /// Opslaan van een vast of eigen Sjabloon. Net als bij de Stijl houdt de Graphic in de Ether
    /// haar vorm tot de volgende Toon; een klaargezette Graphic van dit Sjabloon krijgt de nieuwe
    /// vorm en telt als net veranderd. Gelijk aan de standaard: het vaste Sjabloon is niet meer
    /// aangepast.
    ZetSjabloon { sjabloon: sjabloon::Sjabloon },
    /// Een vast Sjabloon terug naar de standaard (ook: de nieuwe standaard overnemen), of een
    /// eigen Sjabloon weg. Een eigen Sjabloon dat klaarstaat of in een Preset zit, mag niet weg.
    WisSjabloon { id: String },
    /// Een aangepast vast Sjabloon houden zoals het is, ook al is er een nieuwe standaard: de
    /// melding verdwijnt.
    HoudSjabloon { id: String },

    // Drie lagen, Overvloeien (Fade) en uitgebreid Preset-beheer (ADR 0005).
    /// Zet een los Standpunt of een losse Camerarit klaar in het Voorbeeld (`None` haalt hem weg).
    ZetCameraKlaar {
        #[serde(default)]
        #[ts(optional)]
        letter: Option<String>,
    },
    /// Kiest de overgang (Bewegen, Overvloeien/Fade of Knippen) en optioneel de duur van Fade in ms.
    ZetOvergang {
        #[serde(default)]
        #[ts(optional)]
        overgang: Option<OvergangSoort>,
        #[serde(default)]
        #[ts(optional)]
        overvloei_ms: Option<u16>,
    },
    /// Zet een los Stoelbeeld klaar in het Voorbeeld (`None` haalt het weg).
    ZetStoelbeeldKlaar {
        #[serde(default)]
        #[ts(optional)]
        stoelbeeld: Option<Stoelbeeld>,
    },
    /// Past een Stoelbeeld direct toe in de Ether (Live), zonder 2D-Graphic aan te raken.
    ZetStoelbeeldDirect { stoelbeeld: Stoelbeeld },
    /// Maakt het Voorbeeld leeg (Graphic, los Stoelbeeld en los klaargezette camera).
    WisKlaar,
    /// Haalt zowel de 2D-Graphic uit de Ether als alle stoelkleuren weg.
    AllesUit,
    /// Bewaart een uitgebreide Preset: Graphic, Stoelbeeld, camera, overgang en/of eigen naam.
    BewaarPresetUitgebreid {
        #[serde(default)]
        #[ts(optional)]
        naam: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        graphic: Option<Graphic>,
        #[serde(default)]
        #[ts(optional)]
        standpunt: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        stoelbeeld: Option<Stoelbeeld>,
        #[serde(default)]
        #[ts(optional)]
        overgang: Option<OvergangSoort>,
        #[serde(default)]
        #[ts(optional)]
        overvloei_ms: Option<u16>,
    },
    /// Werkt een bestaande Preset bij (naam, lagen en overgang) zonder zijn nummer te verliezen.
    BewerkPreset {
        nummer: u16,
        #[serde(default)]
        #[ts(optional)]
        naam: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        graphic: Option<Graphic>,
        #[serde(default)]
        zonder_graphic: bool,
        #[serde(default)]
        #[ts(optional)]
        standpunt: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        stoelbeeld: Option<Stoelbeeld>,
        #[serde(default)]
        #[ts(optional)]
        overgang: Option<OvergangSoort>,
        #[serde(default)]
        #[ts(optional)]
        overvloei_ms: Option<u16>,
    },
    /// Verplaatst Preset `nummer` naar positie `naar` (1-gebaseerd) en hernummert 1..=N voor sneltoetsen 1–9.
    VerplaatsPreset { nummer: u16, naar: u16 },
}

/// Waarom de server een Opdracht niet uitvoert.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Weigering {
    #[error("de kop is leeg")]
    KopLeeg,
    #[error("de kop is langer dan {MAX_KOP} tekens")]
    KopTeLang,
    #[error("de regel is langer dan {MAX_REGEL} tekens")]
    RegelTeLang,
    #[error("er staat geen Graphic klaar")]
    NietsKlaar,
    #[error("gewijzigd door {0}")]
    GewijzigdDoor(String),
    #[error("er staat intussen een andere Graphic klaar, van {0}")]
    AndereGraphicKlaar(String),
    #[error("deze Graphic staat al zo in beeld")]
    AlInBeeld,
    #[error("er is niets bij te werken")]
    NietsBijTeWerken,
    #[error("er staat geen Graphic in beeld")]
    NietsInBeeld,
    #[error("er staat al een Graphic in beeld: haal die eerst uit beeld")]
    EtherNietLeeg,
    #[error("de Stoelen hebben al hun eigen kleur")]
    KleurAlUit,
    #[error("er loopt geen Opbouw")]
    GeenOpbouw,
    #[error("er is geen Standpunt {0}")]
    OnbekendStandpunt(String),
    #[error("{standpunt} past niet bij deze Graphic: {reden}")]
    StandpuntPastNiet { standpunt: String, reden: String },
    #[error("Standpunt {0} is vast")]
    VastStandpunt(String),
    #[error("Camerarit {0} is vast: bijschaven en hernoemen kan, wissen of opnieuw opnemen niet")]
    VasteCamerarit(String),
    #[error("{standpunt} wordt gebruikt door {waar}")]
    StandpuntInGebruik { standpunt: String, waar: String },
    #[error("de naam van een Standpunt is leeg")]
    StandpuntZonderNaam,
    #[error("de naam van een Standpunt is langer dan {MAX_STANDPUNTNAAM} tekens")]
    StandpuntnaamTeLang,
    #[error("een eigen Standpunt krijgt J, K, L of geen toets, niet {0}")]
    OnbekendeToets(String),
    #[error("de server weet niet waar de camera staat")]
    GeenCamerastand,
    #[error("opnemen kan alleen tijdens Oefenen")]
    OpnemenZonderOefenen,
    #[error("er loopt een opname: stop die eerst")]
    OpnameLoopt,
    #[error("er loopt geen opname")]
    GeenOpname,
    #[error("de opname is te kort")]
    OpnameTeKort,
    #[error("herhalen kan niet: {0}")]
    HerhalenKanNiet(String),
    #[error("er moet minstens {RIT_MIN_S} s van de Camerarit overblijven")]
    RitTeKort,
    #[error("er zijn {AANTAL_STOELEN} stoelkleuren nodig (of geen), niet {0}")]
    StoelkleurenAantal(usize),
    #[error(transparent)]
    Cijfers(#[from] CijferWeigering),
    #[error(transparent)]
    Kamer(#[from] KamerWeigering),
    #[error(transparent)]
    Stemming(#[from] StemmingWeigering),
    #[error(transparent)]
    Zaal(#[from] ZaalWeigering),
    #[error("er is geen Preset {0}")]
    OnbekendePreset(u16),
    #[error("er is geen Kamerlid of eigen persoon met id \"{0}\"")]
    OnbekendePersoon(String),
    #[error("de naam is leeg")]
    NaamLeeg,
    #[error("de naam is langer dan {} tekens", persoonskaart::MAX_NAAM)]
    NaamTeLang,
    #[error("kies twee verschillende sets")]
    TweeKeerDezelfdeSet,
    #[error("er is geen Sjabloon {0:?}")]
    OnbekendSjabloon(String),
    #[error("het Sjabloon {0} wordt nog gebruikt (klaargezet of in een Preset)")]
    SjabloonInGebruik(String),
    #[error("het Sjabloon {0} is niet aangepast")]
    NietAangepast(String),
    #[error(transparent)]
    Sjabloon(#[from] sjabloon::SjabloonWeigering),
    #[error(transparent)]
    Stijl(#[from] sjabloon::StijlWeigering),
    #[error("kies een vergelijkingsset: er is geen standaard (zie Systeem)")]
    GeenVergelijkingsset,
    #[error("er staat geen Coalitie klaar of in beeld")]
    GeenCoalitie,
    #[error("de tablet zet alleen klaar: de Regie zet het in de Ether")]
    NietVanafTablet,
    #[error("Preset {0} staat niet op de tablet")]
    NietOpTablet(u16),
    #[error("kleurkracht is {0}; kies tussen 0 en 100")]
    KleurkrachtOngeldig(u8),
    #[error("een Preset heeft minstens een Graphic, Stoelbeeld of Standpunt nodig")]
    PresetLeeg,
}

/// Wie een Opdracht gaf, en wanneer. De server vult dit in.
#[derive(Clone, Copy, Debug)]
pub struct Herkomst<'a> {
    /// De naam van de afzender, zoals de Werkplek.
    pub door: &'a str,
    /// Het volgnummer dat de Stand krijgt als de Opdracht slaagt.
    pub volgnummer: u64,
    /// Milliseconden sinds 1970 (UTC).
    pub tijd_ms: u64,
}

impl Stand {
    /// Voert een Opdracht uit op deze Stand. Bij een weigering blijft de Stand gelijk.
    pub fn voer_uit(&mut self, opdracht: &Opdracht, h: &Herkomst) -> Result<(), Weigering> {
        match opdracht {
            Opdracht::ZetKlaar { graphic, standpunt } => {
                graphic.controleer()?;
                self.controleer_sjabloon_van(graphic)?;
                self.controleer_standpunt(standpunt.as_deref())?;
                let graphic = self.vul_cijfers_in(graphic, h.tijd_ms)?;
                self.controleer_standpunt_voor(self.camera_profiel(&graphic), standpunt.as_deref())?;
                self.zet_klaar(graphic, standpunt.clone(), h);
            }
            Opdracht::WijzigKlaar { id, graphic, standpunt } => {
                graphic.controleer()?;
                self.controleer_sjabloon_van(graphic)?;
                self.controleer_standpunt(standpunt.as_deref())?;
                let graphic = &self.vul_cijfers_in(graphic, h.tijd_ms)?;
                self.controleer_standpunt_voor(self.camera_profiel(graphic), standpunt.as_deref())?;
                let klaar = self.klaar.as_mut().ok_or(Weigering::NietsKlaar)?;
                if klaar.id != *id {
                    return Err(Weigering::AndereGraphicKlaar(klaar.klaargezet_door.clone()));
                }
                klaar.versie = h.volgnummer;
                klaar.graphic = graphic.clone();
                klaar.standpunt = standpunt.clone();
                klaar.gewijzigd_door = h.door.into();
                self.camera.proef = None;
                self.zet_standpunt_van_sjabloon();
            }
            Opdracht::Toon { versie } => {
                let heeft_graphic = self.klaar.is_some();
                let heeft_stoelen = self.klaar_stoelbeeld.is_some();
                let heeft_camera = self.camera.klaar_standpunt.is_some();
                if !heeft_graphic && !heeft_stoelen && !heeft_camera {
                    return Err(Weigering::NietsKlaar);
                }
                let graphic_verandert = if let Some(klaar) = self.klaar.as_ref() {
                    if versie.is_some_and(|v| v != klaar.versie) {
                        return Err(Weigering::GewijzigdDoor(klaar.gewijzigd_door.clone()));
                    }
                    !self.staat_klaar_in_beeld()
                } else {
                    false
                };
                if !graphic_verandert && !heeft_stoelen && !heeft_camera {
                    return Err(Weigering::AlInBeeld);
                }
                let overgang = self.camera.overgang;
                let knip = matches!(overgang, Some(OvergangSoort::Knippen | OvergangSoort::Overvloeien));
                let fade = matches!(overgang, Some(OvergangSoort::Overvloeien));
                if fade {
                    self.camera.overvloei_sinds = h.volgnummer;
                }
                if graphic_verandert {
                    let klaar = self.klaar.as_ref().expect("graphic_verandert");
                    let ether = self.in_beeld(klaar.id, klaar.versie, &klaar.graphic, &klaar.standpunt, h);
                    self.presets.iter_mut().for_each(|p| p.getoond(&ether));
                    self.zet_in_ether_met_knip(ether, knip, !heeft_camera, !heeft_stoelen, h);
                }
                if let Some(sb) = self.klaar_stoelbeeld.take() {
                    let kleuren = if !self.klaar_stoelkleuren.is_empty() || matches!(sb.bron, Stoelbron::Uit) {
                        std::mem::take(&mut self.klaar_stoelkleuren)
                    } else {
                        sb.bereken_kleuren(self)?
                    };
                    self.stoelkleuren = kleuren;
                    self.kleurovergang = sb.overgang.unwrap_or_else(|| self.stijl_van_ether().kleurovergang);
                    self.ether_kleurkracht = sb.kleurkracht;
                    self.ether_stoelbeeld = if matches!(sb.bron, Stoelbron::Uit) { None } else { Some(sb) };
                }
                if let Some(letter) = self.camera.klaar_standpunt.take() {
                    if letter != camera::BLIJF {
                        self.kies_standpunt(&letter, knip, h);
                    }
                }
            }
            Opdracht::TerugInBeeld { graphic, standpunt } => {
                if self.ether.is_some() {
                    return Err(Weigering::EtherNietLeeg);
                }
                graphic.controleer()?;
                self.controleer_sjabloon_van(graphic)?;
                self.controleer_standpunt_voor(self.camera_profiel(graphic), standpunt.as_deref())?;
                // Een nieuw id: wat klaarstaat is niet deze Graphic, dus Bijwerken of de stempels
                // van een Preset horen er niet bij.
                let ether = self.in_beeld(h.volgnummer, h.volgnummer, graphic, standpunt, h);
                self.zet_in_ether(ether, h);
            }
            Opdracht::Bijwerken => {
                self.ververs_cijfers_van_klaar(h);
                if !self.kan_bijwerken() {
                    return Err(match self.ether {
                        None => Weigering::NietsInBeeld,
                        Some(_) => Weigering::NietsBijTeWerken,
                    });
                }
                let (Some(ether), Some(klaar)) = (self.ether.as_mut(), self.klaar.as_ref()) else {
                    unreachable!("kan_bijwerken vraagt beide")
                };
                let ander_standpunt = ether.standpunt() != klaar.standpunt();
                ether.versie = klaar.versie;
                ether.graphic = klaar.graphic.clone();
                ether.standpunt = klaar.standpunt.clone();
                if let Some(k) = ether.stoelkleuren() {
                    self.stoelkleuren = k;
                    self.kleurovergang = ether.kleurovergang().bij_bijwerken();
                }
                if ander_standpunt && klaar.standpunt() != camera::BLIJF {
                    let letter = klaar.standpunt().into_owned();
                    self.kies_standpunt(&letter, false, h);
                }
            }
            Opdracht::ZetCoalitie { partijen } => self.zet_coalitie(partijen, h)?,
            Opdracht::UitBeeld => {
                if self.ether.take().is_none() {
                    return Err(Weigering::NietsInBeeld);
                }
            }
            Opdracht::KleurUit => {
                if self.stoelkleuren.is_empty() {
                    return Err(Weigering::KleurAlUit);
                }
                self.stoelkleuren.clear();
                self.ether_stoelbeeld = None;
                self.ether_kleurkracht = None;
                self.kleurovergang = self.stijl_van_ether().kleurovergang;
            }
            Opdracht::SlaOpbouwOver => {
                if self.kleurovergang.vorm != sjabloon::Overgangsvorm::Opbouw || self.stoelkleuren.is_empty() {
                    return Err(Weigering::GeenOpbouw);
                }
                self.kleurovergang =
                    sjabloon::Kleurovergang { vorm: sjabloon::Overgangsvorm::Vervagen, duur_ms: OPBOUW_OVERSLAAN_MS };
            }
            Opdracht::KiesStandpunt { letter, knip } => {
                // "Camera blijft staan" is een keuze voor een Graphic, geen plek om heen te gaan
                if letter == camera::BLIJF {
                    return Err(Weigering::OnbekendStandpunt(letter.clone()));
                }
                self.controleer_standpunt(Some(letter))?;
                // Gerichte partij- en stoelbewegingen gaan naar de Graphic in de Ether
                let letter = if let Some(b) = camera::Partijbeweging::uit_sleutel(letter) {
                    let e = self.ether.as_ref().ok_or(Weigering::NietsInBeeld)?;
                    if e.graphic.partij().is_none() {
                        return Err(Weigering::StandpuntPastNiet {
                            standpunt: b.naam().into(),
                            reden: "de Graphic in de Ether heeft geen gekozen Partij".into(),
                        });
                    }
                    e.graphic.standpunt_bij(Some(letter)).into_owned()
                } else if let Some(b) = camera::Stoelbeweging::uit_sleutel(letter) {
                    let e = self.ether.as_ref().ok_or(Weigering::NietsInBeeld)?;
                    if e.graphic.uitgelichte_stoel().is_none() {
                        return Err(Weigering::StandpuntPastNiet {
                            standpunt: b.naam().into(),
                            reden: "de Graphic in de Ether heeft geen uitgelichte Stoel".into(),
                        });
                    }
                    e.graphic.standpunt_bij(Some(letter)).into_owned()
                } else {
                    letter.clone()
                };
                self.kies_standpunt(&letter, *knip, h);
            }
            Opdracht::ZetVrijeCamera { aan } => self.zet_vrije_camera(*aan)?,
            Opdracht::ZetOefenen { aan } => self.zet_oefenen(*aan)?,
            Opdracht::ZetOpnemen { aan, voor, opname } => {
                self.zet_opnemen(*aan, voor.as_deref(), opname.as_deref(), h)?
            }
            Opdracht::ProbeerInVoorbeeld { letter } => self.probeer(letter, h)?,
            Opdracht::ZetStandpuntOvergang { letter, overgang, na_aankomst } => {
                self.zet_standpunt_overgang(letter, *overgang, *na_aankomst, h)?
            }
            Opdracht::HerstelCamerarit { letter } => self.herstel_camerarit(letter, h)?,
            Opdracht::BijschaafCamerarit { letter, bijschaving } => {
                self.bijschaaf_camerarit(letter, *bijschaving, h)?
            }
            Opdracht::BewaarStandpunt { naam, stand } => self.bewaar_standpunt(naam, *stand)?,
            Opdracht::OverschrijfStandpunt { letter, stand } => {
                self.overschrijf_standpunt(letter, *stand)?
            }
            Opdracht::HernoemStandpunt { letter, naam } => self.hernoem_standpunt(letter, naam)?,
            Opdracht::ZetStandpuntToets { letter, toets } => {
                self.zet_standpunt_toets(letter, toets.as_deref())?
            }
            Opdracht::WisStandpunt { letter } => self.wis_standpunt(letter)?,
            Opdracht::ZetStoelkleuren { kleuren } => {
                if !kleuren.is_empty() && kleuren.len() != AANTAL_STOELEN {
                    return Err(Weigering::StoelkleurenAantal(kleuren.len()));
                }
                self.stoelkleuren = kleuren.clone();
                self.kleurovergang = self.stijl_van_ether().kleurovergang;
            }
            Opdracht::LeesInvoer { bestand, ontvangen, invoer } => {
                self.cijfers.lees_invoer(bestand, ontvangen, invoer)
            }
            Opdracht::ZetZetels { set, partij, zetels } => {
                self.cijfers.zet_zetels(set, partij, *zetels)?
            }
            Opdracht::VergrendelSet { set } => self.cijfers.vergrendel(set, true)?,
            Opdracht::OntgrendelSet { set } => self.cijfers.vergrendel(set, false)?,
            Opdracht::KoppelVoorganger { partij, voorganger } => {
                self.cijfers.koppel_voorganger(partij, voorganger)?
            }
            Opdracht::OntkoppelVoorganger { partij, voorganger } => {
                self.cijfers.ontkoppel_voorganger(partij, voorganger)?
            }
            Opdracht::VoegPartijToe { naam, kort, kleur, logo, fractie_zonder_lijst } => {
                self.cijfers.voeg_partij_toe(
                    naam,
                    kort,
                    kleur.as_deref(),
                    logo.as_deref(),
                    *fractie_zonder_lijst,
                )?;
            }
            Opdracht::VulPartijAan { partij, aanvulling } => {
                self.cijfers.vul_partij_aan(partij, aanvulling)?
            }
            Opdracht::MaakSet { setsoort, label } => {
                self.cijfers.maak_set(*setsoort, label)?;
            }
            Opdracht::KiesVergelijkingsset { set } => {
                self.cijfers.kies_vergelijkingsset(set.as_deref())?
            }
            Opdracht::LeesKamer { opgehaald, peildatum, kamerleden, fracties } => {
                self.kamer.lees(&mut self.cijfers, opgehaald, peildatum, kamerleden, fracties);
                self.stemmingen.reken_opnieuw(&self.kamer);
            }
            Opdracht::ZetFoto { kamerlid, foto } => self.kamer.zet_foto(kamerlid, *foto)?,
            Opdracht::LeesStemmingslijst { dag, ingelezen, punten } => {
                self.stemmingen.lees(dag, ingelezen, punten, &self.kamer)
            }
            Opdracht::VoegStemmingToe { titel } => {
                self.stemmingen.voeg_toe(titel, h.volgnummer, &self.kamer)?
            }
            Opdracht::ZetVoorUitzending { stemming, aan } => {
                self.stemmingen.zet_voor_uitzending(stemming, *aan)?
            }
            Opdracht::ZetStemgedrag { stemming, stemgedrag } => {
                self.stemmingen.zet_stemgedrag(stemming, stemgedrag, &self.kamer)?
            }
            Opdracht::VergrendelStemming { stemming } => self.stemmingen.vergrendel(stemming, true)?,
            Opdracht::OntgrendelStemming { stemming } => self.stemmingen.vergrendel(stemming, false)?,
            Opdracht::ZetStemmingIndeling { indeling } => self.stemmingen.indeling = *indeling,
            Opdracht::LegStemmingVast { stemming } => {
                let stoelen = stemmingen::Stoelbron {
                    zaal: &self.zaalindeling,
                    volgorde: &self.uitslagvolgorde.bepaal(&self.cijfers),
                };
                let graphic = self.stemmingen.leg_vast(stemming, &self.kamer, &self.cijfers, &stoelen)?;
                let zelfde = self.klaar.as_ref().filter(|k| k.graphic.stemming() == Some(stemming.as_str()));
                match zelfde {
                    Some(k) => {
                        let (id, standpunt) = (k.id, k.standpunt.clone());
                        self.voer_uit(&Opdracht::WijzigKlaar { id, graphic, standpunt }, h)?;
                    }
                    None => self.zet_klaar(graphic, None, h),
                }
            }
            Opdracht::LeesOfficieleUitslagen { opgehaald, uitslagen } => {
                self.stemmingen.lees_officieel(opgehaald, uitslagen)
            }
            Opdracht::NeemOfficieelOver { stemming } => {
                self.stemmingen.neem_officieel_over(stemming, h.door, &self.kamer)?
            }
            Opdracht::OnzeInvoerKlopt { stemming } => self.stemmingen.onze_invoer_klopt(stemming, h.door)?,
            Opdracht::BewaarPreset { graphic, standpunt } => {
                graphic.controleer()?;
                self.controleer_sjabloon_van(graphic)?;
                self.controleer_standpunt(standpunt.as_deref())?;
                let graphic = &self.vul_cijfers_in(graphic, h.tijd_ms)?;
                self.controleer_standpunt_voor(self.camera_profiel(graphic), standpunt.as_deref())?;
                let nummer = presets::eerste_vrije_nummer(&self.presets);
                // Bewaar je wat klaarstaat of in beeld staat, dan krijgt de Preset meteen de
                // stempel Klaar of Live, alsof hij uit deze Preset kwam.
                let klaar = self.klaar.as_ref().filter(|k| {
                    k.id == k.versie && k.graphic == *graphic && k.standpunt == *standpunt
                });
                let ether = self.ether.as_ref().filter(|e| {
                    e.id == e.versie && e.graphic == *graphic && e.standpunt == *standpunt
                });
                self.presets.push(Preset {
                    nummer,
                    naam: None,
                    graphic: graphic.clone(),
                    zonder_graphic: false,
                    standpunt: standpunt.clone(),
                    stoelbeeld: None,
                    overgang: None,
                    overvloei_ms: None,
                    op_tablet: false,
                    bewaard_ms: h.tijd_ms,
                    bewaard_door: h.door.into(),
                    klaargezet_als: klaar.map(|k| k.id),
                    getoond_als: ether.map(|e| e.id),
                    laatst_live_ms: ether.map(|e| e.sinds_ms),
                });
                self.presets.sort_by_key(|p| p.nummer);
            }
            Opdracht::WisPreset { nummer } => {
                let voor = self.presets.len();
                self.presets.retain(|p| p.nummer != *nummer);
                if self.presets.len() == voor {
                    return Err(Weigering::OnbekendePreset(*nummer));
                }
            }
            Opdracht::ZetPresetKlaar { nummer } => {
                let p = self.preset(*nummer)?.clone();
                self.controleer_standpunt(p.standpunt.as_deref())?;
                if let Some(ov) = p.overgang {
                    self.camera.overgang = Some(ov);
                }
                if let Some(ms) = p.overvloei_ms {
                    self.camera.overvloei_ms = ms.clamp(OVERVLOEI_MIN_MS, OVERVLOEI_MAX_MS);
                }
                if let Some(sb) = p.stoelbeeld.clone() {
                    let kleuren = sb.bereken_kleuren(self)?;
                    self.klaar_stoelbeeld = Some(sb);
                    self.klaar_stoelkleuren = kleuren;
                } else if p.zonder_graphic {
                    self.klaar_stoelbeeld = None;
                    self.klaar_stoelkleuren.clear();
                }
                if p.zonder_graphic {
                    self.klaar = None;
                    self.camera.klaar_standpunt = p.standpunt.clone();
                    self.camera.proef = None;
                } else {
                    self.camera.klaar_standpunt = None;
                    if p.stoelbeeld.is_none() {
                        self.klaar_stoelbeeld = None;
                        self.klaar_stoelkleuren.clear();
                    }
                    self.zet_klaar(p.graphic.clone(), p.standpunt.clone(), h);
                    self.preset_mut(*nummer)?.klaargezet_als = Some(h.volgnummer);
                }
            }
            Opdracht::ZetPresetOpTablet { nummer, aan } => self.preset_mut(*nummer)?.op_tablet = *aan,
            Opdracht::ZetUitslagvolgorde { volgorde } => self.uitslagvolgorde = volgorde.clone(),
            Opdracht::LeesZaalindeling { zaalindeling, .. } => self.zaalindeling = zaalindeling.clone(),
            Opdracht::GeefStoel { kamerlid, stoel } => {
                self.zaalindeling.geef_stoel(&self.kamer, kamerlid, *stoel)?
            }
            Opdracht::BewaarPersoon { persoon } => self.bewaar_persoon(persoon, h.volgnummer)?,
            Opdracht::WisPersoon { persoon } => self.wis_persoon(persoon)?,
            Opdracht::ZetFotoPersoon { persoon, foto } => self.zet_foto_persoon(persoon, *foto)?,
            Opdracht::ZetStandaardExtra { extra } => self.persoonskaart_extra = *extra,
            Opdracht::ZetStijl { stijl } => self.zet_stijl(stijl, h)?,
            Opdracht::ZetSjabloon { sjabloon } => self.zet_sjabloon(sjabloon, h)?,
            Opdracht::WisSjabloon { id } => self.wis_sjabloon(id, h)?,
            Opdracht::HoudSjabloon { id } => {
                let b = self.sjablonen.iter_mut().find(|b| b.sjabloon.id == *id && b.standaard.is_some());
                let b = b.ok_or_else(|| Weigering::NietAangepast(id.clone()))?;
                let vast = sjabloon::vast_sjabloon(id).ok_or_else(|| Weigering::OnbekendSjabloon(id.clone()))?;
                b.standaard = Some(sjabloon::vingerafdruk(&vast));
            }
            Opdracht::ZetCameraKlaar { letter } => {
                self.controleer_standpunt(letter.as_deref())?;
                self.camera.klaar_standpunt = letter.clone();
                self.camera.proef = None;
            }
            Opdracht::ZetOvergang { overgang, overvloei_ms } => {
                self.camera.overgang = *overgang;
                if let Some(ms) = overvloei_ms {
                    self.camera.overvloei_ms = (*ms).clamp(OVERVLOEI_MIN_MS, OVERVLOEI_MAX_MS);
                }
            }
            Opdracht::ZetStoelbeeldKlaar { stoelbeeld } => match stoelbeeld {
                Some(sb) => {
                    let kleuren = sb.bereken_kleuren(self)?;
                    self.klaar_stoelbeeld = Some(sb.clone());
                    self.klaar_stoelkleuren = kleuren;
                }
                None => {
                    self.klaar_stoelbeeld = None;
                    self.klaar_stoelkleuren.clear();
                }
            },
            Opdracht::ZetStoelbeeldDirect { stoelbeeld } => {
                let kleuren = stoelbeeld.bereken_kleuren(self)?;
                self.stoelkleuren = kleuren;
                self.kleurovergang = stoelbeeld
                    .overgang
                    .unwrap_or_else(|| self.stijl_van_ether().kleurovergang);
                self.ether_kleurkracht = stoelbeeld.kleurkracht;
                self.ether_stoelbeeld = if matches!(stoelbeeld.bron, Stoelbron::Uit) {
                    None
                } else {
                    Some(stoelbeeld.clone())
                };
            }
            Opdracht::WisKlaar => {
                self.klaar = None;
                self.klaar_stoelbeeld = None;
                self.klaar_stoelkleuren.clear();
                self.camera.klaar_standpunt = None;
                self.camera.proef = None;
            }
            Opdracht::AllesUit => {
                if self.ether.is_none() && self.stoelkleuren.is_empty() {
                    return Err(Weigering::NietsInBeeld);
                }
                self.ether = None;
                self.stoelkleuren.clear();
                self.ether_stoelbeeld = None;
                self.ether_kleurkracht = None;
                self.kleurovergang = self.stijl_van_ether().kleurovergang;
            }
            Opdracht::BewaarPresetUitgebreid {
                naam,
                graphic,
                standpunt,
                stoelbeeld,
                overgang,
                overvloei_ms,
            } => {
                if graphic.is_none() && stoelbeeld.is_none() && standpunt.is_none() {
                    return Err(Weigering::PresetLeeg);
                }
                self.controleer_standpunt(standpunt.as_deref())?;
                if let Some(sb) = stoelbeeld {
                    sb.bereken_kleuren(self)?;
                }
                let (zonder_graphic, g) = match graphic {
                    Some(g) => {
                        g.controleer()?;
                        self.controleer_sjabloon_van(g)?;
                        let ingevuld = self.vul_cijfers_in(g, h.tijd_ms)?;
                        self.controleer_standpunt_voor(self.camera_profiel(&ingevuld), standpunt.as_deref())?;
                        (false, ingevuld)
                    }
                    None => (
                        true,
                        Graphic::Titelkaart {
                            kop: naam.clone().unwrap_or_else(|| "Stoelbeeld".into()),
                            regel: String::new(),
                        },
                    ),
                };
                let nummer = presets::eerste_vrije_nummer(&self.presets);
                let schoon_naam = naam
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.chars().take(80).collect());
                let schoon_sb = stoelbeeld.clone().map(|mut sb| {
                    sb.label = sb
                        .label
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.chars().take(80).collect());
                    sb
                });
                self.presets.push(Preset {
                    nummer,
                    naam: schoon_naam,
                    graphic: g,
                    zonder_graphic,
                    standpunt: standpunt.clone(),
                    stoelbeeld: schoon_sb,
                    overgang: *overgang,
                    overvloei_ms: overvloei_ms.map(|ms| ms.clamp(OVERVLOEI_MIN_MS, OVERVLOEI_MAX_MS)),
                    op_tablet: false,
                    bewaard_ms: h.tijd_ms,
                    bewaard_door: h.door.into(),
                    klaargezet_als: None,
                    getoond_als: None,
                    laatst_live_ms: None,
                });
                self.presets.sort_by_key(|p| p.nummer);
            }
            Opdracht::BewerkPreset {
                nummer,
                naam,
                graphic,
                zonder_graphic,
                standpunt,
                stoelbeeld,
                overgang,
                overvloei_ms,
            } => {
                self.controleer_standpunt(standpunt.as_deref())?;
                if let Some(sb) = stoelbeeld {
                    sb.bereken_kleuren(self)?;
                }
                let ingevuld = match graphic {
                    Some(g) if !zonder_graphic => {
                        g.controleer()?;
                        self.controleer_sjabloon_van(g)?;
                        let g = self.vul_cijfers_in(g, h.tijd_ms)?;
                        self.controleer_standpunt_voor(self.camera_profiel(&g), standpunt.as_deref())?;
                        Some(g)
                    }
                    _ => None,
                };
                let p = self.preset_mut(*nummer)?;
                p.naam = naam
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.chars().take(80).collect());
                p.zonder_graphic = *zonder_graphic;
                if let Some(g) = ingevuld {
                    p.graphic = g;
                }
                p.standpunt = standpunt.clone();
                p.stoelbeeld = stoelbeeld.clone().map(|mut sb| {
                    sb.label = sb
                        .label
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.chars().take(80).collect());
                    sb
                });
                p.overgang = *overgang;
                p.overvloei_ms = overvloei_ms.map(|ms| ms.clamp(OVERVLOEI_MIN_MS, OVERVLOEI_MAX_MS));
                p.bewaard_ms = h.tijd_ms;
                p.bewaard_door = h.door.into();
            }
            Opdracht::VerplaatsPreset { nummer, naar } => {
                let Some(van_i) = self.presets.iter().position(|p| p.nummer == *nummer) else {
                    return Err(Weigering::OnbekendePreset(*nummer));
                };
                let p = self.presets.remove(van_i);
                let doel = usize::from(naar.saturating_sub(1)).min(self.presets.len());
                self.presets.insert(doel, p);
                for (i, preset) in self.presets.iter_mut().enumerate() {
                    preset.nummer = (i + 1) as u16;
                }
            }
        }
        self.uitslagindeling = self.uitslagvolgorde.bepaal(&self.cijfers);
        self.zaalmelding = self.zaalindeling.melding(&self.kamer);
        self.stemmingen.reken_controle(&self.kamer);
        Ok(())
    }

    /// De Graphic met de cijfers van nu erin, voor een Sjabloon dat zijn cijfers vasthoudt, of
    /// met de gegevens van de persoon op de dag van `tijd_ms` (Persoonskaart).
    fn vul_cijfers_in(&self, graphic: &Graphic, tijd_ms: u64) -> Result<Graphic, Weigering> {
        let mut g = graphic.clone();
        if let Graphic::Persoonskaart { wie, gegevens, .. } = &mut g {
            *gegevens = Some(self.persoonsgegevens(wie, tijd_ms)?);
        }
        let volgorde = self.uitslagvolgorde.bepaal(&self.cijfers);
        if let Graphic::Zetelverdeling { set, indeling, cijfers, .. } = &mut g {
            *cijfers = Some(match indeling {
                Indeling::Uitslagindeling => Zetelstand::van(&volgorde, &self.cijfers, set)
                    .ok_or_else(|| CijferWeigering::OnbekendeSet(set.clone()))?,
                Indeling::Zaalindeling => {
                    Zetelstand::van_zaal(&volgorde, &self.cijfers, &self.kamer, &self.zaalindeling)
                }
            });
        }
        let onbekend = |set: &str| Weigering::from(CijferWeigering::OnbekendeSet(set.into()));
        match &mut g {
            Graphic::Zetelbalk { set, cijfers, .. } | Graphic::UitslagPerPartij { set, cijfers, .. } => {
                *cijfers = Some(Zetelstand::van(&volgorde, &self.cijfers, set).ok_or_else(|| onbekend(set))?);
            }
            Graphic::PartijUitgelicht { set, partij, cijfers } => {
                let p = self.cijfers.partij(partij).ok_or_else(|| CijferWeigering::OnbekendePartij(partij.clone()))?;
                let mut stand = Zetelstand::van(&volgorde, &self.cijfers, set).ok_or_else(|| onbekend(set))?;
                // ook zonder Zetels in deze set: haar naam staat in beeld
                if !stand.partijen.iter().any(|q| q.sleutel == p.sleutel) {
                    stand.partijen.push(p.clone());
                }
                *cijfers = Some(stand);
            }
            Graphic::WinstEnVerlies { set, vergelijk_met, cijfers } => {
                let tegen = vergelijk_met
                    .clone()
                    .or_else(|| self.cijfers.vergelijk_met.clone())
                    .ok_or(Weigering::GeenVergelijkingsset)?;
                if tegen == *set {
                    return Err(Weigering::TweeKeerDezelfdeSet);
                }
                for s in [&*set, &tegen] {
                    self.cijfers.set(s).ok_or_else(|| onbekend(s))?;
                }
                *cijfers = Zetelstand::tegen(&volgorde, &self.cijfers, set, &tegen);
            }
            Graphic::Coalitie { set, cijfers, .. } | Graphic::Coalitiebalk { set, cijfers, .. } => {
                *cijfers = Some(if set == crate::uitslagindeling::HUIDIGE_KAMER {
                    Zetelstand::van_kamer(&volgorde, &self.cijfers, &self.kamer)
                } else {
                    Zetelstand::van(&volgorde, &self.cijfers, set).ok_or_else(|| onbekend(set))?
                });
            }
            Graphic::TweeSets { set, tweede, cijfers, .. } => {
                for s in [&*set, &*tweede] {
                    self.cijfers.set(s).ok_or_else(|| onbekend(s))?;
                }
                *cijfers = Zetelstand::van_twee(&volgorde, &self.cijfers, set, tweede);
            }
            Graphic::Eigen { set: Some(set), partij, cijfers, .. } => {
                // met de vergelijkingsset, zodat een Getal met winst of verlies iets toont
                let mut stand = match self.cijfers.vergelijk_met.as_deref().filter(|v| v != set) {
                    Some(tegen) => Zetelstand::tegen(&volgorde, &self.cijfers, set, tegen),
                    None => Zetelstand::van(&volgorde, &self.cijfers, set),
                }
                .ok_or_else(|| onbekend(set))?;
                if let Some(p) = partij.as_deref() {
                    let p = self.cijfers.partij(p).ok_or_else(|| CijferWeigering::OnbekendePartij(p.into()))?;
                    if !stand.partijen.iter().any(|q| q.sleutel == p.sleutel) {
                        stand.partijen.push(p.clone());
                    }
                }
                *cijfers = Some(stand);
            }
            Graphic::Eigen { set: None, partij: Some(p), .. } => {
                self.cijfers.partij(p).ok_or_else(|| CijferWeigering::OnbekendePartij(p.clone()))?;
            }
            _ => {}
        }
        Ok(g)
    }

    /// Het Sjabloon met dit `id` zoals het nu geldt: bewerkt door de omroep, anders het vaste.
    pub fn sjabloon(&self, id: &str) -> Option<Cow<'_, sjabloon::Sjabloon>> {
        match self.bewerkt(id) {
            Some(b) => Some(Cow::Borrowed(&b.sjabloon)),
            None => sjabloon::vast_sjabloon(id).map(Cow::Owned),
        }
    }

    /// Het bewerkte Sjabloon met dit `id`, als de omroep het aanpaste of maakte.
    pub fn bewerkt(&self, id: &str) -> Option<&sjabloon::BewerktSjabloon> {
        self.sjablonen.iter().find(|b| b.sjabloon.id == id)
    }

    fn controleer_sjabloon_van(&self, graphic: &Graphic) -> Result<(), Weigering> {
        let id = graphic.sjabloon_id();
        match self.sjabloon(&id) {
            Some(_) => Ok(()),
            None => Err(Weigering::OnbekendSjabloon(id.into_owned())),
        }
    }

    /// Zet het standaard Standpunt van het Sjabloon bij de klaargezette Graphic.
    fn zet_standpunt_van_sjabloon(&mut self) {
        if let Some(k) = &self.klaar {
            let standpunt = self.sjabloon(&k.graphic.sjabloon_id()).and_then(|s| s.standpunt.clone());
            if let Some(k) = self.klaar.as_mut() {
                k.standpunt_van_sjabloon = standpunt;
            }
        }
    }

    /// De klaargezette Graphic van dit Sjabloon krijgt de nieuwe vorm als nieuwe versie, zodat
    /// Toon weigert op een scherm dat nog de oude vorm liet zien (#21).
    fn nieuwe_vorm_voor_klaar(&mut self, id: &str, h: &Herkomst) {
        if let Some(k) = self.klaar.as_mut().filter(|k| k.graphic.sjabloon_id() == id) {
            k.versie = h.volgnummer;
            k.gewijzigd_door = h.door.into();
        }
        self.zet_standpunt_van_sjabloon();
    }

    /// Opslaan van een Sjabloon uit de vormgevingstab ([`Opdracht::ZetSjabloon`]).
    fn zet_sjabloon(&mut self, nieuw: &sjabloon::Sjabloon, h: &Herkomst) -> Result<(), Weigering> {
        nieuw.controleer()?;
        self.controleer_standpunt_voor(nieuw.camera_profiel(), nieuw.standpunt.as_deref())?;
        if self.sjabloon(&nieuw.id).is_some_and(|s| *s == *nieuw) {
            return Ok(());
        }
        let vast = sjabloon::vast_sjabloon(&nieuw.id);
        let plek = self.sjablonen.iter().position(|b| b.sjabloon.id == nieuw.id);
        match (&vast, plek) {
            // gelijk aan de standaard: niet meer aangepast
            (Some(v), Some(i)) if v == nieuw => {
                self.sjablonen.remove(i);
            }
            (_, Some(i)) => self.sjablonen[i].sjabloon = nieuw.clone(),
            (_, None) => self.sjablonen.push(sjabloon::BewerktSjabloon {
                sjabloon: nieuw.clone(),
                standaard: vast.as_ref().map(sjabloon::vingerafdruk),
            }),
        }
        self.nieuwe_vorm_voor_klaar(&nieuw.id, h);
        Ok(())
    }

    /// Een vast Sjabloon terug naar de standaard, of een eigen Sjabloon weg
    /// ([`Opdracht::WisSjabloon`]).
    fn wis_sjabloon(&mut self, id: &str, h: &Herkomst) -> Result<(), Weigering> {
        let i = self.sjablonen.iter().position(|b| b.sjabloon.id == id);
        let i = i.ok_or_else(|| match sjabloon::is_vast(id) {
            true => Weigering::NietAangepast(id.into()),
            false => Weigering::OnbekendSjabloon(id.into()),
        })?;
        if !sjabloon::is_vast(id) {
            let klaar = self.klaar.iter().any(|k| k.graphic.sjabloon_id() == id);
            if klaar || self.presets.iter().any(|p| p.graphic.sjabloon_id() == id) {
                return Err(Weigering::SjabloonInGebruik(id.into()));
            }
        }
        self.sjablonen.remove(i);
        self.nieuwe_vorm_voor_klaar(id, h);
        Ok(())
    }

    /// Of er nieuwere cijfers zijn dan die waarmee deze Graphic is ingevuld (het label
    /// "nieuwere cijfers" in het dashboard). De Ether verandert pas na Bijwerken.
    pub fn nieuwere_cijfers(&self, graphic: &Graphic) -> bool {
        graphic
            .cijfersets()
            .into_iter()
            .any(|(set, versie)| self.cijfers.nieuwere_cijfers(set, versie))
    }

    /// Bij Bijwerken: staat de klaargezette Graphic ook in de Ether en zijn er nieuwere cijfers,
    /// dan krijgt hij die, als nieuwe versie.
    fn ververs_cijfers_van_klaar(&mut self, h: &Herkomst) {
        let Some(klaar) = &self.klaar else { return };
        let zelfde = self.ether.as_ref().is_some_and(|e| e.id == klaar.id);
        if !zelfde || !self.nieuwere_cijfers(&klaar.graphic) {
            return;
        }
        let Ok(graphic) = self.vul_cijfers_in(&klaar.graphic, h.tijd_ms) else { return };
        let klaar = self.klaar.as_mut().expect("net gezien");
        klaar.graphic = graphic;
        klaar.versie = h.volgnummer;
        klaar.gewijzigd_door = h.door.into();
    }

    /// Zet de gekozen Partijen in de Coalitie die klaarstaat en in die in de Ether, met een nieuwe
    /// versie. Zo werkt de Ether direct bij, zonder Bijwerken.
    fn zet_coalitie(&mut self, gekozen: &[String], h: &Herkomst) -> Result<(), Weigering> {
        fn zet(g: &mut Graphic, gekozen: &[String]) -> bool {
            match g {
                Graphic::Coalitie { partijen, .. } | Graphic::Coalitiebalk { partijen, .. } => {
                    *partijen = gekozen.to_vec();
                    true
                }
                _ => false,
            }
        }
        let mut gedaan = false;
        if let Some(k) = self.klaar.as_mut()
            && zet(&mut k.graphic, gekozen)
        {
            k.versie = h.volgnummer;
            k.gewijzigd_door = h.door.into();
            gedaan = true;
        }
        if let Some(e) = self.ether.as_mut()
            && zet(&mut e.graphic, gekozen)
        {
            e.versie = h.volgnummer;
            if let Some(k) = e.stoelkleuren() {
                self.stoelkleuren = k;
                self.kleurovergang = e.kleurovergang().bij_bijwerken();
            }
            gedaan = true;
        }
        if gedaan { Ok(()) } else { Err(Weigering::GeenCoalitie) }
    }

    fn preset(&self, nummer: u16) -> Result<&Preset, Weigering> {
        self.presets.iter().find(|p| p.nummer == nummer).ok_or(Weigering::OnbekendePreset(nummer))
    }

    fn preset_mut(&mut self, nummer: u16) -> Result<&mut Preset, Weigering> {
        self.presets.iter_mut().find(|p| p.nummer == nummer).ok_or(Weigering::OnbekendePreset(nummer))
    }

    /// De Presets die dit Standpunt gebruiken, ook als Standpunt van hun Sjabloon. Zolang er een
    /// is, mag het Standpunt niet weg.
    pub fn presets_met_standpunt<'a>(&'a self, letter: &'a str) -> impl Iterator<Item = &'a Preset> {
        self.presets.iter().filter(move |p| p.standpunt() == letter)
    }

    /// Of de klaargezette Graphic uit deze Preset komt (stempel Klaar).
    pub fn preset_staat_klaar(&self, p: &Preset) -> bool {
        self.klaar.as_ref().is_some_and(|k| p.is_bron_van_klaar(k))
    }

    /// Of de Graphic in de Ether uit deze Preset komt (stempel Live).
    pub fn preset_in_beeld(&self, p: &Preset) -> bool {
        self.ether.as_ref().is_some_and(|e| p.is_bron_van_ether(e))
    }

    /// Een kopie voor de Ether van deze Graphic, in de Stijl en het Sjabloon van nu.
    fn in_beeld(&self, id: u64, versie: u64, graphic: &Graphic, standpunt: &Option<String>, h: &Herkomst) -> GraphicInBeeld {
        GraphicInBeeld {
            id,
            versie,
            graphic: graphic.clone(),
            standpunt: standpunt.clone(),
            getoond: h.volgnummer,
            sinds_ms: h.tijd_ms,
            getoond_door: h.door.into(),
            stijl: self.stijl.clone(),
            sjabloon: self.sjabloon(&graphic.sjabloon_id()).map(Cow::into_owned),
        }
    }

    /// Zet een Graphic in de Ether, als wissel: de camera gaat naar haar Standpunt (behalve als
    /// de camera blijft staan) en de Stoelen krijgen haar kleuren.
    fn zet_in_ether(&mut self, ether: GraphicInBeeld, h: &Herkomst) {
        self.zet_in_ether_met_knip(ether, false, true, true, h);
    }

    fn zet_in_ether_met_knip(
        &mut self,
        ether: GraphicInBeeld,
        knip: bool,
        met_camera: bool,
        met_stoelen: bool,
        h: &Herkomst,
    ) {
        if met_camera {
            let letter = ether.standpunt();
            if letter != camera::BLIJF {
                self.kies_standpunt(&letter, knip, h);
            }
        }
        if met_stoelen && let Some(k) = ether.stoelkleuren() {
            self.stoelkleuren = k;
            self.kleurovergang = ether.kleurovergang();
            self.ether_stoelbeeld = None;
            self.ether_kleurkracht = None;
        }
        if let Some(id) = ether.graphic.stemming() {
            self.stemmingen.in_ether(id, ether.sinds_ms);
        }
        self.ether = Some(ether);
    }

    fn zet_klaar(&mut self, graphic: Graphic, standpunt: Option<String>, h: &Herkomst) {
        self.klaar = Some(KlaargezetteGraphic {
            id: h.volgnummer,
            versie: h.volgnummer,
            graphic,
            standpunt,
            standpunt_van_sjabloon: None,
            klaargezet_door: h.door.into(),
            gewijzigd_door: h.door.into(),
        });
        self.camera.proef = None;
        self.zet_standpunt_van_sjabloon();
    }

    /// Het Standpunt met deze letter.
    pub fn standpunt(&self, letter: &str) -> Option<&Standpunt> {
        self.standpunten.iter().find(|s| s.letter == letter)
    }

    /// De Camerarit met deze sleutel.
    pub fn camerarit(&self, letter: &str) -> Option<&Camerarit> {
        self.cameraritten.iter().find(|r| r.letter == letter)
    }

    /// Waar de camera staat bij dit Standpunt, aan het begin van deze Camerarit, of bij een
    /// gerichte beweging naar een Partij of Stoel. Bij [`camera::BLIJF`]: die van het Standpunt
    /// van de Ether.
    pub fn camerastand_van(&self, letter: &str) -> Option<Camerastand> {
        let letter = if letter == camera::BLIJF { self.camera.standpunt.as_str() } else { letter };
        self.standpunt(letter)
            .map(|s| s.stand)
            .or_else(|| self.camerarit(letter).and_then(Camerarit::beginstand))
            .or_else(|| camera::Stoelbeweging::uit_doelsleutel(letter).and_then(|(b, s)| b.stand_op(s, 0.0)))
            .or_else(|| camera::Partijbeweging::uit_doelsleutel(letter).and_then(|(b, van, tot)| b.stand_op(van, tot, 0.0)))
            .or_else(|| camera::Zaalbeweging::uit_sleutel(letter).map(|b| b.stand_op(0.0)))
            .or_else(|| camera::Coalitiebeweging::uit_sleutel(letter).map(|b| b.stand_op(76, 0.0)))
            .or_else(|| camera::Paneelbeweging::uit_sleutel(letter).map(|b| b.stand_op(0.0)))
            .or_else(|| (letter == camera::BOVEN_HET_PANEEL).then(camera::boven_het_paneel))
    }

    /// Het [`camera::CameraProfiel`] van een Graphic: welke Standpunten en bewegingen erbij passen.
    pub fn camera_profiel(&self, graphic: &Graphic) -> camera::CameraProfiel {
        use camera::CameraProfiel;
        match graphic {
            Graphic::PartijUitgelicht { .. } => CameraProfiel::Partij,
            Graphic::Persoonskaart { .. } => {
                if graphic.uitgelichte_stoel().is_some() {
                    CameraProfiel::PersoonMetStoel
                } else {
                    CameraProfiel::PersoonZonderStoel
                }
            }
            Graphic::Zetelverdeling { .. } => CameraProfiel::HeleZaal,
            Graphic::Coalitie { .. } => CameraProfiel::CoalitieZaal,
            Graphic::Zetelbalk { .. }
            | Graphic::TweeSets { .. }
            | Graphic::Coalitiebalk { .. }
            | Graphic::UitslagStemming { .. } => CameraProfiel::BovenPaneel,
            Graphic::UitslagPerPartij { .. }
            | Graphic::WinstEnVerlies { .. }
            | Graphic::FractiesVoorEnTegen { .. }
            | Graphic::HoofdelijkeStemming { .. } => CameraProfiel::GrootPaneel,
            Graphic::Titelkaart { .. } => CameraProfiel::Algemeen,
            Graphic::Eigen { id, partij, .. } => match self.sjabloon(id) {
                Some(s) => s.camera_profiel(),
                None if partij.is_some() => CameraProfiel::AlgemeenMetPartij,
                None => CameraProfiel::Algemeen,
            },
        }
    }

    /// Of een Standpunt, Camerarit of gerichte beweging bij dit [`camera::CameraProfiel`] past.
    pub fn standpunt_past_bij(&self, profiel: camera::CameraProfiel, letter: &str) -> bool {
        if letter == camera::BLIJF {
            return true;
        }
        if letter == camera::BOVEN_HET_PANEEL {
            return profiel.mag_boven_het_paneel();
        }
        if camera::Partijbeweging::uit_sleutel(letter).is_some()
            || camera::Partijbeweging::uit_doelsleutel(letter).is_some()
        {
            return profiel.mag_partijbeweging();
        }
        if camera::Stoelbeweging::uit_sleutel(letter).is_some()
            || camera::Stoelbeweging::uit_doelsleutel(letter).is_some()
        {
            return profiel.mag_stoelbeweging();
        }
        if camera::Zaalbeweging::uit_sleutel(letter).is_some() {
            return profiel.mag_zaalbeweging();
        }
        if camera::Coalitiebeweging::uit_sleutel(letter).is_some() {
            return profiel.mag_coalitiebeweging();
        }
        if camera::Paneelbeweging::uit_sleutel(letter).is_some() {
            return profiel.mag_paneelbeweging();
        }
        if let Some(s) = self.standpunt(letter) {
            return if s.vast { profiel.vaste_standpunten().contains(&s.letter.as_str()) } else { profiel.mag_eigen() };
        }
        if let Some(r) = self.camerarit(letter) {
            return if r.vast { profiel.vaste_ritten().contains(&r.letter.as_str()) } else { profiel.mag_eigen() };
        }
        false
    }

    /// Een leesbare naam voor een Standpunt-sleutel (voor in een weigering).
    fn standpunt_label(&self, letter: &str) -> String {
        if letter == camera::BOVEN_HET_PANEEL {
            return "Totaal boven het paneel".into();
        }
        if let Some(b) = camera::Partijbeweging::uit_sleutel(letter)
            .or_else(|| camera::Partijbeweging::uit_doelsleutel(letter).map(|(b, _, _)| b))
        {
            return b.naam().into();
        }
        if let Some(b) = camera::Stoelbeweging::uit_sleutel(letter)
            .or_else(|| camera::Stoelbeweging::uit_doelsleutel(letter).map(|(b, _)| b))
        {
            return b.naam().into();
        }
        if let Some(b) = camera::Zaalbeweging::uit_sleutel(letter) {
            return b.naam().into();
        }
        if let Some(b) = camera::Coalitiebeweging::uit_sleutel(letter) {
            return b.naam().into();
        }
        if let Some(b) = camera::Paneelbeweging::uit_sleutel(letter) {
            return b.naam().into();
        }
        if let Some(s) = self.standpunt(letter) {
            return s.naam.clone();
        }
        if let Some(r) = self.camerarit(letter) {
            return r.naam.clone();
        }
        letter.into()
    }

    /// Controleert of het Standpunt bestaat én bij het profiel van de Graphic of het Sjabloon past.
    fn controleer_standpunt_voor(&self, profiel: camera::CameraProfiel, letter: Option<&str>) -> Result<(), Weigering> {
        self.controleer_standpunt(letter)?;
        if let Some(l) = letter
            && !self.standpunt_past_bij(profiel, l)
        {
            return Err(Weigering::StandpuntPastNiet {
                standpunt: self.standpunt_label(l),
                reden: profiel.uitleg().into(),
            });
        }
        Ok(())
    }

    /// Of er een Standpunt, Camerarit of gerichte beweging met deze sleutel is, of Camera blijft
    /// staan (geen sleutel mag ook).
    fn controleer_standpunt(&self, letter: Option<&str>) -> Result<(), Weigering> {
        match letter {
            Some(l)
                if self.standpunt(l).is_none()
                    && self.camerarit(l).is_none()
                    && l != camera::BLIJF
                    && l != camera::BOVEN_HET_PANEEL
                    && camera::Partijbeweging::uit_sleutel(l).is_none()
                    && camera::Partijbeweging::uit_doelsleutel(l).is_none()
                    && camera::Stoelbeweging::uit_sleutel(l).is_none()
                    && camera::Stoelbeweging::uit_doelsleutel(l).is_none()
                    && camera::Zaalbeweging::uit_sleutel(l).is_none()
                    && camera::Coalitiebeweging::uit_sleutel(l).is_none()
                    && camera::Paneelbeweging::uit_sleutel(l).is_none() =>
            {
                Err(Weigering::OnbekendStandpunt(l.into()))
            }
            _ => Ok(()),
        }
    }

    fn kies_standpunt(&mut self, letter: &str, knip: bool, h: &Herkomst) {
        // een Standpunt kiezen zet de Vrije camera uit en stopt het afspelen in het Voorbeeld;
        // Oefenen en een opname lopen door
        self.camera.standpunt = letter.into();
        self.camera.knip = knip;
        self.camera.sinds = h.volgnummer;
        self.camera.vrij = false;
        self.camera.proef = None;
    }

    /// Opslaan van een nieuwe Stijl. De Graphic in de Ether houdt haar eigen kopie
    /// ([`GraphicInBeeld::stijl`]). De klaargezette Graphic krijgt de nieuwe vorm als nieuwe
    /// versie, zodat Toon weigert op een scherm dat nog de oude vorm liet zien (#21).
    fn zet_stijl(&mut self, stijl: &sjabloon::Stijl, h: &Herkomst) -> Result<(), Weigering> {
        stijl.controleer()?;
        if *stijl == self.stijl {
            return Ok(());
        }
        self.stijl = stijl.clone();
        if let Some(klaar) = self.klaar.as_mut() {
            klaar.versie = h.volgnummer;
            klaar.gewijzigd_door = h.door.into();
        }
        Ok(())
    }

    /// De Stijl van wat de Ether toont: die van de Graphic in de Ether, anders de huidige. Zo
    /// loopt ook de kleurgolf in de Zaal op de snelheid van de Graphic die hem begon.
    pub fn stijl_van_ether(&self) -> &sjabloon::Stijl {
        self.ether.as_ref().map_or(&self.stijl, |e| &e.stijl)
    }

    /// Of de klaargezette Graphic een nieuwere versie is van de Graphic in de Ether.
    pub fn kan_bijwerken(&self) -> bool {
        match (&self.ether, &self.klaar) {
            (Some(e), Some(k)) => {
                e.id == k.id && (e.versie != k.versie || self.nieuwere_cijfers(&k.graphic))
            }
            _ => false,
        }
    }

    /// Of precies wat klaarstaat ook in de Ether staat, in dezelfde Stijl en hetzelfde Sjabloon.
    /// Na een nieuwe Stijl of een bewerkt Sjabloon mag Toon dus weer, ook als de Graphic in de
    /// Ether daarna is bijgewerkt (#51, #61, #62).
    pub fn staat_klaar_in_beeld(&self) -> bool {
        match (&self.ether, &self.klaar) {
            (Some(e), Some(k)) => {
                let sjabloon = self.sjabloon(&k.graphic.sjabloon_id());
                e.id == k.id
                    && e.versie == k.versie
                    && e.stijl == self.stijl
                    && (e.sjabloon.is_none() || e.sjabloon.as_ref() == sjabloon.as_deref())
            }
            _ => false,
        }
    }
}

/// De Stand met het volgnummer van de laatste wijziging. Het volgnummer stijgt bij elke
/// uitgevoerde Opdracht en blijft bewaard na een herstart van de server.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq)]
#[ts(export)]
pub struct GenummerdeStand {
    #[ts(type = "number")]
    pub volgnummer: u64,
    pub stand: Stand,
}

/// Het beeld dat een Renderer tekent. De rol kies je bij het opstarten.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[ts(export)]
pub enum Rol {
    Ether,
    Reserve,
    Voorbeeld,
    /// Bestaat in de code, maar wordt nog niet gebouwd.
    Videowand,
}

/// Een Renderer zoals de Regie hem in het onderdeel Systeem ziet: rol, machine en versie.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[ts(export)]
pub struct RendererOpMachine {
    pub rol: Rol,
    pub machine: String,
    pub versie: String,
}

/// Wie een Opdracht stuurt of de Stand volgt. Komt in het logboek.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "soort")]
#[ts(export)]
pub enum Afzender {
    /// Een laptop van de Regie, met een eigen naam.
    Werkplek { naam: String },
    /// De tablet van de Presentator: ook een Werkplek met een eigen naam, maar hij zet alleen
    /// Presets klaar en tikt Partijen aan in de Coalitiebouwer ([`Stand::controleer_afzender`]).
    Tablet { naam: String },
    /// Een Renderer met zijn rol, versie en de naam van de machine waarop hij draait.
    Renderer {
        rol: Rol,
        versie: String,
        #[serde(default)]
        machine: String,
    },
    /// Een knop via HTTP, zoals een Stream Deck.
    Knop { naam: String },
    /// Het invoerbestand met partijlijst en sets, dat de server zelf inleest.
    Invoerbestand { pad: String },
    /// De open data van de Tweede Kamer, die de server zelf ophaalt.
    OpenData { bron: String },
    /// Het bestand met de Zaalindeling, dat de server zelf inleest.
    Zaalindeling { pad: String },
    /// De gamepad aan de server, met de naam die de controller zelf opgeeft.
    Gamepad { naam: String },
}

impl Afzender {
    /// De naam die de Regie te zien krijgt, zoals bij "klaargezet door".
    pub fn naam(&self) -> String {
        match self {
            Afzender::Werkplek { naam } | Afzender::Tablet { naam } | Afzender::Knop { naam } => naam.clone(),
            Afzender::Renderer { rol, .. } => format!("Renderer {rol:?}"),
            Afzender::Invoerbestand { .. } => "Invoerbestand".into(),
            Afzender::OpenData { .. } => "Open data".into(),
            Afzender::Zaalindeling { .. } => "Zaalindeling".into(),
            Afzender::Gamepad { .. } => "Gamepad".into(),
        }
    }
}

/// Berichten van een scherm (dashboard of Renderer) naar de server, via de WebSocket.
/// Het eerste bericht is altijd [`BerichtAanServer::Hallo`].
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "soort")]
#[ts(export)]
pub enum BerichtAanServer {
    Hallo { afzender: Afzender },
    Opdracht { opdracht: Opdracht },
    /// Welke set deze Werkplek nu open heeft in het onderdeel Cijfers, of geen. Verandert de
    /// Stand niet en zet niets op slot: de andere Werkplekken zien het alleen.
    Bewerk {
        #[serde(default)]
        #[ts(optional)]
        set: Option<String>,
    },
}

/// Berichten van de server naar een scherm, via de WebSocket.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "soort")]
#[ts(export)]
pub enum BerichtVanServer {
    /// De hele Stand, direct na Hallo en daarna bij elke wijziging.
    Stand(GenummerdeStand),
    /// De Opdracht is uitgevoerd; de Stand met dit volgnummer volgt.
    Uitgevoerd {
        #[ts(type = "number")]
        volgnummer: u64,
    },
    /// De server voert de Opdracht niet uit, of weigert de verbinding.
    Geweigerd { reden: String },
    /// De Renderers die nu verbonden zijn, en die de server net weigerde omdat hun versie
    /// anders is, met de versie van de server. Alleen voor Werkplekken: na Hallo en bij elke
    /// verandering.
    Renderers {
        versie: String,
        verbonden: Vec<RendererOpMachine>,
        geweigerd: Vec<RendererOpMachine>,
    },
    /// Waar de camera's nu staan, 100 keer per seconde (ADR 0002). Een Werkplek krijgt er 10
    /// per seconde, voor de plattegrond.
    Camera(Camerabeeld),
    /// Welke set elke andere Werkplek open heeft in het onderdeel Cijfers. Alleen voor
    /// Werkplekken: na Hallo en bij elke verandering. De eigen verbinding staat er niet in.
    Bewerkingen { open: Vec<Bewerking> },
}

/// Een klein beeld voor de multiviewer van het dashboard. De Renderer met rol Voorbeeld stuurt
/// het als binair WebSocket-bericht: één byte met [`Monitor::byte`], dan een JPEG. Alleen de
/// byte, zonder JPEG, betekent: geen beeld. De server stuurt het ongewijzigd door naar elke
/// Werkplek.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Monitor {
    /// De klaargezette Graphic.
    Voorbeeld,
    /// De Graphic in de Ether, zoals de Ether hem tekent.
    Ether,
}

impl Monitor {
    pub const ALLE: [Monitor; 2] = [Monitor::Voorbeeld, Monitor::Ether];

    pub fn byte(self) -> u8 {
        match self {
            Monitor::Voorbeeld => 0,
            Monitor::Ether => 1,
        }
    }

    pub fn uit_byte(b: u8) -> Option<Monitor> {
        Monitor::ALLE.into_iter().find(|m| m.byte() == b)
    }
}

/// Grootste klein beeld dat de server doorstuurt, in bytes (ruim voor 720p JPEG).
pub const MAX_BEELD: usize = 4 << 20;

#[cfg(test)]
mod tests {
    use super::*;

    const H: Herkomst = Herkomst { door: "Proef", volgnummer: 1, tijd_ms: 0 };

    #[test]
    fn de_zaal_telt_150_stoelen_in_vaste_volgorde() {
        let alle: Vec<Stoel> = Stoel::alle().collect();
        assert_eq!(alle.len(), 150);
        assert_eq!(alle[0], Stoel { vak: 1, rij: 1, nummer: 1 });
        // vak 1 heeft rijen van 2, 3, 4, 5, 5 en 6 Stoelen: de eerste van vak 2 is nummer 25
        assert_eq!(alle[25], Stoel { vak: 2, rij: 1, nummer: 1 });
        assert_eq!(alle[149], Stoel { vak: 6, rij: 6, nummer: 6 });
        for (i, s) in alle.iter().enumerate() {
            assert_eq!(s.index(), Some(i));
        }
        assert_eq!(Stoel { vak: 1, rij: 1, nummer: 3 }.index(), None);
        assert_eq!(Stoel { vak: 7, rij: 1, nummer: 1 }.index(), None);
    }

    #[test]
    fn alle_stoelen_krijgen_in_een_opdracht_een_nieuwe_kleur() {
        let mut stand = Stand::default();
        assert!(stand.stoelkleuren.is_empty(), "zonder kleuren houden de Stoelen hun eigen kleur");
        let rood: Kleur = "#d4202c".parse().unwrap();
        stand
            .voer_uit(&Opdracht::ZetStoelkleuren { kleuren: vec![rood; 150] }, &H)
            .unwrap();
        assert_eq!(stand.stoelkleuren.len(), 150);
        assert!(stand.stoelkleuren.iter().all(|k| *k == Kleur { r: 0xd4, g: 0x20, b: 0x2c }));
    }

    #[test]
    fn een_onvolledige_lijst_stoelkleuren_wordt_geweigerd() {
        let mut stand = Stand::default();
        let fout = stand.voer_uit(
            &Opdracht::ZetStoelkleuren { kleuren: vec![Kleur { r: 1, g: 2, b: 3 }; 149] },
            &H,
        );
        assert_eq!(fout, Err(Weigering::StoelkleurenAantal(149)));
        assert!(stand.stoelkleuren.is_empty());
        // leeg mag: dan krijgen de Stoelen hun eigen kleur terug
        stand.voer_uit(&Opdracht::ZetStoelkleuren { kleuren: vec![] }, &H).unwrap();
    }

    #[test]
    fn een_kleur_reist_als_hexcode() {
        let json = r##"{"soort":"ZetStoelkleuren","kleuren":["#00AE41","#0e2c5a"]}"##;
        let o: Opdracht = serde_json::from_str(json).unwrap();
        assert_eq!(
            o,
            Opdracht::ZetStoelkleuren {
                kleuren: vec![Kleur { r: 0x00, g: 0xae, b: 0x41 }, Kleur { r: 0x0e, g: 0x2c, b: 0x5a }]
            }
        );
        assert_eq!(serde_json::to_string(&Kleur { r: 0, g: 0xae, b: 0x41 }).unwrap(), "\"#00ae41\"");
        assert!(serde_json::from_str::<Kleur>("\"rood\"").is_err());
        assert!(serde_json::from_str::<Kleur>("\"#12345\"").is_err());
    }

    #[test]
    fn een_oude_stand_zonder_stoelkleuren_laadt_nog() {
        let s: Stand = serde_json::from_str(r#"{"tekst":"hallo"}"#).unwrap();
        assert!(s.stoelkleuren.is_empty());
    }
}
