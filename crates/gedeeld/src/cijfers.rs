//! Partijen en cijfersets: de partijlijst, de sets met Zetels per Partij, en winst en verlies.
//!
//! De cijfers komen binnen via het invoerbestand ([`Invoer`]) of via losse Opdrachten.
//! Winst en verlies staan nergens opgeslagen: [`Cijfers::winst_en_verlies`] rekent ze uit
//! tegen de vergelijkingsset, met de som van de Voorgangers van een Partij.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Het aantal Zetels in de Tweede Kamer.
pub const ZETELS: u32 = 150;

/// Sleutel in `zetels` voor de Zetels van partijen die een set niet apart noemt.
pub const OVERIG: &str = "overig";

/// Alle partijen en cijfersets van de uitzending.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Cijfers {
    pub verkiezing: Option<Verkiezing>,
    /// De partijlijst: Partijen en Fracties zonder lijst, in de volgorde van het bestand.
    pub partijen: Vec<Partij>,
    pub sets: Vec<Cijferset>,
    /// De `id` van de set waartegen winst en verlies wordt berekend.
    pub vergelijk_met: Option<String>,
    /// Wat het invoerbestand de laatste keer bracht, voor de melding in het dashboard.
    pub laatste_invoer: Option<Invoermelding>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Verkiezing {
    pub naam: String,
    pub datum: String,
}

/// Een Partij of een Fractie zonder lijst in de partijlijst.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Partij {
    pub sleutel: String,
    pub naam: String,
    pub kort: String,
    /// Kleur als `#RRGGBB`: de aangeleverde kleur, of een tijdelijke kleur.
    pub kleur: String,
    /// De kleur komt uit de reeks tijdelijke kleuren, want er was geen geldige kleur.
    pub kleur_tijdelijk: bool,
    /// Pad naar het logo dat de omroep aanlevert. Zonder logo staat de korte naam op die plek.
    pub logo: Option<String>,
    pub fractie_zonder_lijst: bool,
    /// Sleutels van de Voorgangers. Een Voorganger hoort bij hooguit één Partij.
    pub voorgangers: Vec<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Soort {
    Peiling,
    Exitpoll,
    Prognose,
    Tussenstand,
    Uitslag,
}

/// Een set verkiezingscijfers: Zetels per Partij. Een Partij die ontbreekt heeft nul Zetels.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Cijferset {
    pub id: String,
    pub soort: Soort,
    pub label: String,
    pub tijd: Option<String>,
    pub bron: Option<String>,
    pub zetels: BTreeMap<String, u16>,
    #[ts(type = "Record<string, number> | null")]
    pub stemmen: Option<BTreeMap<String, u64>>,
    /// Vergrendeld: de server weigert elke wijziging, ook uit het invoerbestand.
    pub vergrendeld: bool,
    /// Stijgt bij elke wijziging van de cijfers. Een Graphic die met een lagere versie in
    /// beeld staat, krijgt het label "nieuwere cijfers".
    #[ts(type = "number")]
    pub versie: u64,
}

impl Cijferset {
    pub fn zetels_van(&self, partij: &str) -> u32 {
        self.zetels.get(partij).copied().map_or(0, u32::from)
    }

    pub fn totaal(&self) -> u32 {
        self.zetels.values().map(|&z| u32::from(z)).sum()
    }
}

/// Eén regel winst en verlies.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Regel {
    pub partij: String,
    pub zetels: u16,
    /// Zetels nu min de Zetels in de vergelijkingsset (van de Partij zelf plus haar Voorgangers).
    pub verschil: i32,
    /// Nieuw: geen Voorganger en geen Zetels in de vergelijkingsset.
    pub nieuw: bool,
}

impl Regel {
    /// De tekst in een Graphic over winst en verlies: "+5", "-3", "0" of "+26 nieuw".
    pub fn tekst(&self) -> String {
        let getal = match self.verschil {
            v if v > 0 => format!("+{v}"),
            v => v.to_string(),
        };
        if self.nieuw {
            format!("{getal} nieuw")
        } else {
            getal
        }
    }
}

impl Cijfers {
    pub fn partij(&self, sleutel: &str) -> Option<&Partij> {
        self.partijen.iter().find(|p| p.sleutel == sleutel)
    }

    pub fn set(&self, id: &str) -> Option<&Cijferset> {
        self.sets.iter().find(|s| s.id == id)
    }

    /// De Partij die `voorganger` als Voorganger heeft, als die er is.
    pub fn opvolger_van(&self, voorganger: &str) -> Option<&Partij> {
        self.partijen
            .iter()
            .find(|p| p.voorgangers.iter().any(|v| v == voorganger))
    }

    /// Winst en verlies van set `id` tegen de vergelijkingsset, in de volgorde van de
    /// partijlijst. Een Partij die in de vergelijkingsset Zetels had en nu nul heeft, staat
    /// erin met "-N". Een Voorganger van een andere Partij krijgt geen eigen regel.
    /// `None` als de set of de vergelijkingsset er niet is.
    pub fn winst_en_verlies(&self, id: &str) -> Option<Vec<Regel>> {
        self.winst_en_verlies_tegen(id, self.vergelijk_met.as_deref()?)
    }

    /// Winst en verlies van set `id` tegen set `vorige`, zoals [`Cijfers::winst_en_verlies`].
    pub fn winst_en_verlies_tegen(&self, id: &str, vorige: &str) -> Option<Vec<Regel>> {
        let set = self.set(id)?;
        let vorige = self.set(vorige)?;
        let regels = self
            .partijen
            .iter()
            .filter_map(|p| {
                let nu = set.zetels_van(&p.sleutel);
                let opgegaan = self.opvolger_van(&p.sleutel).is_some();
                if opgegaan && nu == 0 {
                    return None;
                }
                let eerder = if opgegaan {
                    0
                } else {
                    vorige.zetels_van(&p.sleutel)
                        + p.voorgangers
                            .iter()
                            .map(|v| vorige.zetels_van(v))
                            .sum::<u32>()
                };
                if nu == 0 && eerder == 0 {
                    return None;
                }
                Some(Regel {
                    partij: p.sleutel.clone(),
                    zetels: nu as u16,
                    verschil: nu as i32 - eerder as i32,
                    nieuw: eerder == 0 && p.voorgangers.is_empty() && !opgegaan,
                })
            })
            .collect();
        Some(regels)
    }
}

/// Wat het invoerbestand de laatste keer bracht.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Invoermelding {
    pub bestand: String,
    pub ontvangen: String,
    /// Partijen die nieuw in de partijlijst kwamen.
    pub partijen_nieuw: Vec<String>,
    /// Sets die nieuw of gewijzigd binnenkwamen.
    pub sets_binnen: Vec<String>,
    pub sets_ongewijzigd: Vec<String>,
    pub overgeslagen: Vec<Overgeslagen>,
    /// Ingelezen, maar met iets om naar te kijken, zoals een set die niet op 150 Zetels uitkomt.
    pub waarschuwingen: Vec<String>,
}

/// Iets uit het invoerbestand dat niet is ingelezen, met de reden.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Overgeslagen {
    /// De `id` van de set of de `sleutel` van de Partij, of "bestand".
    pub wat: String,
    pub reden: String,
}

/// Het invoerbestand: een partijlijst en sets. Lees het met [`Invoer::uit_json`]: dat slaat
/// een onleesbare Partij of set over in plaats van het hele bestand te weigeren.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Invoer {
    #[serde(default)]
    pub verkiezing: Option<Verkiezing>,
    #[serde(default)]
    pub partijen: Vec<InvoerPartij>,
    #[serde(default)]
    pub sets: Vec<InvoerSet>,
    #[serde(default)]
    pub vergelijk_met: Option<String>,
    /// Wat bij het lezen al onleesbaar bleek.
    #[serde(default)]
    pub onleesbaar: Vec<Overgeslagen>,
}

/// Een Partij zoals ze in het invoerbestand staat.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct InvoerPartij {
    pub sleutel: String,
    pub naam: String,
    #[serde(default)]
    pub kort: Option<String>,
    #[serde(default)]
    pub kleur: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub fractie_zonder_lijst: bool,
    #[serde(default)]
    pub voorgangers: Vec<String>,
}

/// Een set zoals ze in het invoerbestand staat.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct InvoerSet {
    pub id: String,
    pub soort: Soort,
    pub label: String,
    #[serde(default)]
    pub tijd: Option<String>,
    #[serde(default)]
    pub bron: Option<String>,
    pub zetels: BTreeMap<String, u16>,
    #[serde(default)]
    #[ts(type = "Record<string, number> | null")]
    pub stemmen: Option<BTreeMap<String, u64>>,
}

impl Invoer {
    /// Leest het invoerbestand. Een Partij of set die niet klopt komt in `onleesbaar`; is het
    /// hele bestand geen JSON, dan staat alleen dat in `onleesbaar`.
    pub fn uit_json(tekst: &str) -> Invoer {
        let mut invoer = Invoer::default();
        let waarde: serde_json::Value = match serde_json::from_str(tekst) {
            Ok(w) => w,
            Err(e) => {
                invoer.onleesbaar.push(Overgeslagen {
                    wat: "bestand".into(),
                    reden: format!("geen geldige JSON: {e}"),
                });
                return invoer;
            }
        };
        let Some(veld) = waarde.as_object() else {
            invoer.onleesbaar.push(Overgeslagen {
                wat: "bestand".into(),
                reden: "het bestand is geen JSON-object".into(),
            });
            return invoer;
        };
        invoer.verkiezing = veld
            .get("verkiezing")
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        invoer.vergelijk_met = veld
            .get("vergelijk_met")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        for p in lijst(veld.get("partijen")) {
            match serde_json::from_value::<InvoerPartij>(p.clone()) {
                Ok(p) => invoer.partijen.push(p),
                Err(e) => invoer.onleesbaar.push(Overgeslagen {
                    wat: naam_van(p, "sleutel", "partij"),
                    reden: format!("Partij onleesbaar: {e}"),
                }),
            }
        }
        for s in lijst(veld.get("sets")) {
            match serde_json::from_value::<InvoerSet>(s.clone()) {
                Ok(s) => invoer.sets.push(s),
                Err(e) => invoer.onleesbaar.push(Overgeslagen {
                    wat: naam_van(s, "id", "set"),
                    reden: format!("set onleesbaar: {e}"),
                }),
            }
        }
        invoer
    }
}

fn lijst(w: Option<&serde_json::Value>) -> &[serde_json::Value] {
    w.and_then(|w| w.as_array()).map_or(&[], Vec::as_slice)
}

fn naam_van(w: &serde_json::Value, veld: &str, anders: &str) -> String {
    w.get(veld)
        .and_then(|v| v.as_str())
        .unwrap_or(anders)
        .to_owned()
}

/// Kleuren voor een Partij zonder kleur: de 20 kleuren met groot onderling contrast van
/// Kenneth Kelly (1965), zonder wit en zwart. Een nieuwe Partij krijgt de kleur die het verst
/// afligt van alle kleuren die al in gebruik zijn.
pub const TIJDELIJKE_KLEUREN: [&str; 20] = [
    "#F3C300", "#875692", "#F38400", "#A1CAF1", "#BE0032", "#C2B280", "#848482", "#008856",
    "#E68FAC", "#0067A5", "#F99379", "#604E97", "#F6A600", "#B3446C", "#DCD300", "#882D17",
    "#8DB600", "#654522", "#E25822", "#2B3D26",
];

fn geldige_kleur(k: &str) -> bool {
    k.len() == 7 && k.starts_with('#') && k[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn rgb(k: &str) -> (i32, i32, i32) {
    let deel = |i: usize| i32::from_str_radix(&k[i..i + 2], 16).unwrap_or(0);
    (deel(1), deel(3), deel(5))
}

/// Afstand tussen twee kleuren zoals het oog ze ziet (benadering "redmean").
fn afstand(a: &str, b: &str) -> f64 {
    let ((r1, g1, b1), (r2, g2, b2)) = (rgb(a), rgb(b));
    let rm = f64::from(r1 + r2) / 2.0;
    let (dr, dg, db) = (f64::from(r1 - r2), f64::from(g1 - g2), f64::from(b1 - b2));
    ((2.0 + rm / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rm) / 256.0) * db * db).sqrt()
}

/// De tijdelijke kleur die het verst afligt van alle kleuren die al in gebruik zijn.
pub fn tijdelijke_kleur<'a>(in_gebruik: impl Iterator<Item = &'a str> + Clone) -> &'static str {
    TIJDELIJKE_KLEUREN
        .iter()
        .copied()
        .map(|k| {
            let dichtst = in_gebruik
                .clone()
                .map(|g| afstand(k, g))
                .fold(f64::INFINITY, f64::min);
            (k, dichtst)
        })
        .fold(("", f64::NEG_INFINITY), |beste, (k, d)| {
            if d > beste.1 { (k, d) } else { beste }
        })
        .0
}

/// Waarom de server een Opdracht over partijen of cijfersets niet uitvoert.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum CijferWeigering {
    #[error("er is geen Partij \"{0}\"")]
    OnbekendePartij(String),
    #[error("er is geen set \"{0}\"")]
    OnbekendeSet(String),
    #[error("de set \"{0}\" is vergrendeld: ontgrendel hem eerst")]
    Vergrendeld(String),
    #[error(
        "\"{voorganger}\" is al Voorganger van {partij}: een Voorganger hoort bij hooguit één Partij"
    )]
    VoorgangerAlGekoppeld { voorganger: String, partij: String },
    #[error("een Partij kan niet haar eigen Voorganger zijn")]
    EigenVoorganger,
    #[error("meer dan {ZETELS} Zetels kan niet")]
    TeveelZetels,
    #[error("{0} staat al in de partijlijst")]
    PartijBestaatAl(String),
    #[error("een Partij heeft een naam nodig")]
    NaamLeeg,
    #[error("kleur \"{0}\" is geen #RRGGBB")]
    OngeldigeKleur(String),
    #[error("een set heeft een naam nodig")]
    LabelLeeg,
}

/// Wat de Regie bij een Partij aanvult. Alleen de velden die erin staan veranderen, zodat twee
/// Werkplekken elk een ander veld kunnen bijwerken. Een leeg logo haalt het logo weg.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Aanvulling {
    #[serde(default)]
    #[ts(optional)]
    pub naam: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub kort: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub kleur: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub logo: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub fractie_zonder_lijst: Option<bool>,
}

/// Welke set een andere Werkplek nu open heeft in het onderdeel Cijfers. Openen zet niets op slot:
/// het is alleen een melding, zoals "Werkplek 2 bewerkt: Exitpoll". De server houdt dit bij per
/// verbinding, buiten de Stand.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Bewerking {
    pub werkplek: String,
    pub set: String,
}

/// Een sleutel uit `basis` die nog niet in `bezet` staat: "eps", anders "eps-2", "eps-3".
fn vrije_sleutel(basis: &str, bezet: impl Fn(&str) -> bool) -> String {
    let basis = if basis.is_empty() { "nieuw" } else { basis };
    (1..)
        .map(|i| if i == 1 { basis.to_owned() } else { format!("{basis}-{i}") })
        .find(|s| s != OVERIG && !bezet(s))
        .expect("oneindig veel kandidaten")
}

fn nette_kleur(k: &str) -> Result<String, CijferWeigering> {
    let k = k.trim();
    if geldige_kleur(k) {
        Ok(k.to_uppercase())
    } else {
        Err(CijferWeigering::OngeldigeKleur(k.into()))
    }
}

impl Cijfers {
    /// Zet een nieuwe Partij achteraan de partijlijst, met een sleutel uit de korte naam. Zonder
    /// kleur krijgt ze een tijdelijke kleur; zonder logo staat de korte naam op die plek. Weigert
    /// als er al een Partij met dezelfde naam of korte naam is.
    pub fn voeg_partij_toe(
        &mut self,
        naam: &str,
        kort: &str,
        kleur: Option<&str>,
        logo: Option<&str>,
        fractie_zonder_lijst: bool,
    ) -> Result<&Partij, CijferWeigering> {
        let naam = naam.trim();
        if naam.is_empty() {
            return Err(CijferWeigering::NaamLeeg);
        }
        let kort = Some(kort.trim()).filter(|k| !k.is_empty()).unwrap_or(naam);
        let (a, b) = (kaal(naam), kaal(kort));
        if let Some(p) = self.partijen.iter().find(|p| {
            [kaal(&p.naam), kaal(&p.kort)].iter().any(|w| *w == a || *w == b)
        }) {
            return Err(CijferWeigering::PartijBestaatAl(p.kort.clone()));
        }
        let kleur = kleur.map(str::trim).filter(|k| !k.is_empty()).map(nette_kleur).transpose()?;
        let sleutel = vrije_sleutel(&b, |s| self.partij(s).is_some());
        let tijdelijk = kleur.is_none();
        let kleur = kleur.unwrap_or_else(|| {
            tijdelijke_kleur(self.partijen.iter().map(|p| p.kleur.as_str())).into()
        });
        self.partijen.push(Partij {
            sleutel,
            naam: naam.into(),
            kort: kort.into(),
            kleur,
            kleur_tijdelijk: tijdelijk,
            logo: logo.map(str::trim).filter(|l| !l.is_empty()).map(str::to_owned),
            fractie_zonder_lijst,
            voorgangers: Vec::new(),
        });
        Ok(self.partijen.last().expect("net toegevoegd"))
    }

    /// Vult een Partij aan: alleen de velden in `a` veranderen. Een eigen kleur vervangt de
    /// tijdelijke kleur.
    pub fn vul_partij_aan(&mut self, sleutel: &str, a: &Aanvulling) -> Result<(), CijferWeigering> {
        let naam = a.naam.as_deref().map(str::trim);
        if naam == Some("") {
            return Err(CijferWeigering::NaamLeeg);
        }
        let kleur = a.kleur.as_deref().map(nette_kleur).transpose()?;
        let p = self.partij_mut(sleutel)?;
        if let Some(n) = naam {
            p.naam = n.into();
        }
        if let Some(k) = a.kort.as_deref().map(str::trim) {
            p.kort = if k.is_empty() { p.naam.clone() } else { k.into() };
        }
        if let Some(k) = kleur {
            p.kleur = k;
            p.kleur_tijdelijk = false;
        }
        if let Some(l) = a.logo.as_deref().map(str::trim) {
            p.logo = Some(l).filter(|l| !l.is_empty()).map(str::to_owned);
        }
        if let Some(f) = a.fractie_zonder_lijst {
            p.fractie_zonder_lijst = f;
        }
        Ok(())
    }

    /// Maakt een lege set, met een `id` uit de naam.
    pub fn maak_set(&mut self, soort: Soort, label: &str) -> Result<&Cijferset, CijferWeigering> {
        let label = label.trim();
        if label.is_empty() {
            return Err(CijferWeigering::LabelLeeg);
        }
        let id = vrije_sleutel(&kaal(label), |s| self.set(s).is_some());
        self.sets.push(Cijferset {
            id,
            soort,
            label: label.into(),
            tijd: None,
            bron: None,
            zetels: BTreeMap::new(),
            stemmen: None,
            vergrendeld: false,
            versie: 1,
        });
        Ok(self.sets.last().expect("net gemaakt"))
    }

    /// Kiest de set waartegen winst en verlies wordt berekend, of geen.
    pub fn kies_vergelijkingsset(&mut self, set: Option<&str>) -> Result<(), CijferWeigering> {
        if let Some(id) = set {
            self.set(id).ok_or_else(|| CijferWeigering::OnbekendeSet(id.into()))?;
        }
        self.vergelijk_met = set.map(str::to_owned);
        Ok(())
    }
}

impl Cijfers {
    fn set_mut(&mut self, id: &str) -> Result<&mut Cijferset, CijferWeigering> {
        self.sets
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| CijferWeigering::OnbekendeSet(id.into()))
    }

    fn partij_mut(&mut self, sleutel: &str) -> Result<&mut Partij, CijferWeigering> {
        self.partijen
            .iter_mut()
            .find(|p| p.sleutel == sleutel)
            .ok_or_else(|| CijferWeigering::OnbekendePartij(sleutel.into()))
    }

    /// Zet de Zetels van één Partij in een set.
    pub fn zet_zetels(
        &mut self,
        set: &str,
        partij: &str,
        zetels: u16,
    ) -> Result<(), CijferWeigering> {
        if self.partij(partij).is_none() && partij != OVERIG {
            return Err(CijferWeigering::OnbekendePartij(partij.into()));
        }
        if u32::from(zetels) > ZETELS {
            return Err(CijferWeigering::TeveelZetels);
        }
        let s = self.set_mut(set)?;
        if s.vergrendeld {
            return Err(CijferWeigering::Vergrendeld(set.into()));
        }
        if s.zetels_van(partij) != u32::from(zetels) {
            if zetels == 0 {
                s.zetels.remove(partij);
            } else {
                s.zetels.insert(partij.into(), zetels);
            }
            s.versie += 1;
        }
        Ok(())
    }

    pub fn vergrendel(&mut self, set: &str, dicht: bool) -> Result<(), CijferWeigering> {
        self.set_mut(set)?.vergrendeld = dicht;
        Ok(())
    }

    /// Koppelt een Voorganger aan een Partij. Weigert als de Voorganger al bij een andere
    /// Partij hoort.
    pub fn koppel_voorganger(
        &mut self,
        partij: &str,
        voorganger: &str,
    ) -> Result<(), CijferWeigering> {
        if partij == voorganger {
            return Err(CijferWeigering::EigenVoorganger);
        }
        if self.partij(voorganger).is_none() {
            return Err(CijferWeigering::OnbekendePartij(voorganger.into()));
        }
        if let Some(ander) = self.opvolger_van(voorganger) {
            if ander.sleutel == partij {
                return Ok(());
            }
            return Err(CijferWeigering::VoorgangerAlGekoppeld {
                voorganger: voorganger.into(),
                partij: ander.naam.clone(),
            });
        }
        self.partij_mut(partij)?.voorgangers.push(voorganger.into());
        Ok(())
    }

    pub fn ontkoppel_voorganger(
        &mut self,
        partij: &str,
        voorganger: &str,
    ) -> Result<(), CijferWeigering> {
        self.partij_mut(partij)?
            .voorgangers
            .retain(|v| v != voorganger);
        Ok(())
    }

    /// Leest het invoerbestand in. Partijen en sets met een bekende sleutel of `id` worden
    /// bijgewerkt, nieuwe komen erbij; wat niet in het bestand staat blijft staan. Een
    /// vergrendelde set blijft gelijk. Wat binnenkwam en wat is overgeslagen staat daarna in
    /// [`Cijfers::laatste_invoer`].
    pub fn lees_invoer(&mut self, bestand: &str, ontvangen: &str, invoer: &Invoer) {
        let mut m = Invoermelding {
            bestand: bestand.into(),
            ontvangen: ontvangen.into(),
            overgeslagen: invoer.onleesbaar.clone(),
            ..Default::default()
        };
        if let Some(v) = &invoer.verkiezing {
            self.verkiezing = Some(v.clone());
        }
        self.lees_partijen(&invoer.partijen, &mut m);
        self.lees_sets(&invoer.sets, &mut m);
        if let Some(id) = &invoer.vergelijk_met {
            if self.set(id).is_some() {
                self.vergelijk_met = Some(id.clone());
            } else {
                m.waarschuwingen.push(format!(
                    "vergelijkingsset \"{id}\" bestaat niet: niet gekozen"
                ));
            }
        }
        self.laatste_invoer = Some(m);
    }

    fn lees_partijen(&mut self, partijen: &[InvoerPartij], m: &mut Invoermelding) {
        let mut gezien: Vec<&str> = Vec::new();
        for p in partijen {
            if p.sleutel.trim().is_empty() || p.sleutel == OVERIG {
                m.overgeslagen.push(Overgeslagen {
                    wat: p.naam.clone(),
                    reden: format!("ongeldige sleutel \"{}\"", p.sleutel),
                });
                continue;
            }
            if gezien.contains(&p.sleutel.as_str()) {
                m.overgeslagen.push(Overgeslagen {
                    wat: p.sleutel.clone(),
                    reden: "staat twee keer in de partijlijst".into(),
                });
                continue;
            }
            gezien.push(&p.sleutel);
            let kleur = p.kleur.as_deref().map(str::trim).filter(|k| !k.is_empty());
            if let Some(k) = kleur.filter(|k| !geldige_kleur(k)) {
                m.waarschuwingen.push(format!(
                    "{}: kleur \"{k}\" is geen #RRGGBB, tijdelijke kleur gekozen",
                    p.sleutel
                ));
            }
            let kleur = kleur.filter(|k| geldige_kleur(k)).map(str::to_uppercase);
            let logo = p.logo.clone().filter(|l| !l.trim().is_empty());
            let kort = p
                .kort
                .clone()
                .filter(|k| !k.trim().is_empty())
                .unwrap_or_else(|| p.naam.clone());
            match self.partijen.iter_mut().find(|q| q.sleutel == p.sleutel) {
                Some(q) => {
                    q.naam = p.naam.clone();
                    q.kort = kort;
                    q.logo = logo;
                    q.fractie_zonder_lijst = p.fractie_zonder_lijst;
                    if let Some(k) = kleur {
                        q.kleur = k;
                        q.kleur_tijdelijk = false;
                    }
                }
                None => {
                    m.partijen_nieuw.push(p.sleutel.clone());
                    let tijdelijk = kleur.is_none();
                    let kleur = kleur.unwrap_or_else(|| {
                        tijdelijke_kleur(self.partijen.iter().map(|q| q.kleur.as_str())).into()
                    });
                    self.partijen.push(Partij {
                        sleutel: p.sleutel.clone(),
                        naam: p.naam.clone(),
                        kort,
                        kleur,
                        kleur_tijdelijk: tijdelijk,
                        logo,
                        fractie_zonder_lijst: p.fractie_zonder_lijst,
                        voorgangers: Vec::new(),
                    });
                }
            }
        }
        // Voorgangers pas als alle Partijen er zijn, zodat de volgorde in het bestand niet uitmaakt.
        for p in partijen
            .iter()
            .filter(|p| gezien.contains(&p.sleutel.as_str()))
        {
            for v in &p.voorgangers {
                if let Err(w) = self.koppel_voorganger(&p.sleutel, v) {
                    m.waarschuwingen.push(format!(
                        "{}: Voorganger \"{v}\" niet gekoppeld, want {w}",
                        p.sleutel
                    ));
                }
            }
        }
    }

    fn lees_sets(&mut self, sets: &[InvoerSet], m: &mut Invoermelding) {
        let mut gezien: Vec<&str> = Vec::new();
        for s in sets {
            let mut overslaan = |reden: String| {
                m.overgeslagen.push(Overgeslagen {
                    wat: s.id.clone(),
                    reden,
                });
            };
            if s.id.trim().is_empty() {
                overslaan("set zonder id".into());
                continue;
            }
            if gezien.contains(&s.id.as_str()) {
                overslaan("staat twee keer in het bestand".into());
                continue;
            }
            gezien.push(&s.id);
            if let Some(p) = s
                .zetels
                .keys()
                .find(|p| *p != OVERIG && self.partij(p).is_none())
            {
                overslaan(format!("Partij \"{p}\" staat niet in de partijlijst"));
                continue;
            }
            let zetels: BTreeMap<String, u16> = s
                .zetels
                .iter()
                .filter(|(_, z)| **z > 0)
                .map(|(p, z)| (p.clone(), *z))
                .collect();
            let nieuw = Cijferset {
                id: s.id.clone(),
                soort: s.soort,
                label: s.label.clone(),
                tijd: s.tijd.clone(),
                bron: s.bron.clone(),
                zetels,
                stemmen: s.stemmen.clone(),
                vergrendeld: false,
                versie: 1,
            };
            match self.sets.iter_mut().find(|o| o.id == s.id) {
                Some(oud) if zelfde_cijfers(oud, &nieuw) => {
                    m.sets_ongewijzigd.push(s.id.clone());
                    continue;
                }
                Some(oud) if oud.vergrendeld => {
                    overslaan("de set is vergrendeld".into());
                    continue;
                }
                Some(oud) => {
                    *oud = Cijferset {
                        versie: oud.versie + 1,
                        ..nieuw
                    };
                }
                None => self.sets.push(nieuw),
            }
            m.sets_binnen.push(s.id.clone());
            let totaal = s.zetels.values().map(|&z| u32::from(z)).sum::<u32>();
            if totaal != ZETELS {
                m.waarschuwingen.push(format!(
                    "{}: telt op tot {totaal} Zetels, niet {ZETELS}",
                    s.id
                ));
            }
        }
    }

    /// De Partij uit de partijlijst die bij een Fractie uit de open data hoort: dezelfde
    /// sleutel, korte naam of naam, zonder te letten op hoofdletters, spaties en streepjes.
    /// Een Partij met een eigen kleur gaat voor op een Fractie zonder lijst met een tijdelijke
    /// kleur, zodat een Fractie na het inlezen van de partijlijst bij de echte Partij hoort.
    pub fn partij_voor_fractie(&self, afkorting: &str, naam: &str) -> Option<&Partij> {
        let (a, n) = (kaal(afkorting), kaal(naam));
        let past = |p: &&Partij| {
            [&p.sleutel, &p.kort, &p.naam]
                .iter()
                .any(|w| kaal(w) == a || kaal(w) == n)
        };
        let tijdelijk = |p: &&Partij| p.fractie_zonder_lijst && p.kleur_tijdelijk;
        let mut passend = self.partijen.iter().filter(past);
        let eerste = passend.next()?;
        if !tijdelijk(&eerste) {
            return Some(eerste);
        }
        passend.find(|p| !tijdelijk(p)).or(Some(eerste))
    }

    /// Zet een Fractie die niet in de partijlijst staat erbij, als Fractie zonder lijst met een
    /// tijdelijke kleur en zonder logo, zodat de korte naam als tekst op de plek van het logo staat.
    pub fn voeg_fractie_zonder_lijst_toe(
        &mut self,
        sleutel: &str,
        naam: &str,
        kort: &str,
    ) -> &Partij {
        if let Some(i) = self.partijen.iter().position(|p| p.sleutel == sleutel) {
            return &self.partijen[i];
        }
        let kleur = tijdelijke_kleur(self.partijen.iter().map(|p| p.kleur.as_str())).into();
        self.partijen.push(Partij {
            sleutel: sleutel.into(),
            naam: naam.into(),
            kort: kort.into(),
            kleur,
            kleur_tijdelijk: true,
            logo: None,
            fractie_zonder_lijst: true,
            voorgangers: Vec::new(),
        });
        self.partijen.last().expect("net toegevoegd")
    }

    /// Of een Graphic die set `id` met `versie` in beeld heeft, het label "nieuwere cijfers" krijgt.
    pub fn nieuwere_cijfers(&self, id: &str, versie: u64) -> bool {
        self.set(id).is_some_and(|s| s.versie > versie)
    }
}

/// Alles behalve het slot en de versie.
fn zelfde_cijfers(a: &Cijferset, b: &Cijferset) -> bool {
    (&a.soort, &a.label, &a.tijd, &a.bron, &a.zetels, &a.stemmen)
        == (&b.soort, &b.label, &b.tijd, &b.bron, &b.zetels, &b.stemmen)
}

/// Alleen letters en cijfers, in kleine letters: "GL-PvdA" wordt "glpvda".
fn kaal(t: &str) -> String {
    t.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
