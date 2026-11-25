//! De Kamer die zit: Kamerleden en Fracties uit de open data van de Tweede Kamer.
//!
//! De server haalt ze op en stuurt ze als [`Opdracht::LeesKamer`](crate::Opdracht::LeesKamer).
//! Een Fractie die niet in de partijlijst staat, komt erbij als Fractie zonder lijst, met een
//! tijdelijke kleur en de korte naam als tekst, en staat in de [`Kamermelding`].
//!
//! Een Graphic in beeld verandert hier niet door: een Graphic bewaart zijn eigen gegevens, dus
//! een wissel of een nieuwe Kamer komt pas in beeld als de Regie een nieuwe Graphic maakt.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::cijfers::Cijfers;

/// De Kamer die zit, zoals de open data die op de peildatum geeft.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Kamer {
    /// De dag waarop de server keek wie er in de Kamer zit (`JJJJ-MM-DD`).
    pub peildatum: Option<String>,
    /// Gesorteerd op achternaam.
    pub kamerleden: Vec<Kamerlid>,
    /// Gesorteerd op aantal Zetels, grootste eerst.
    pub fracties: Vec<Fractie>,
    /// Bij welke Partij uit de partijlijst een Fractie hoort: `Fractie.id` naar `Partij.sleutel`.
    pub partij_van: BTreeMap<String, String>,
    /// Wat het ophalen de laatste keer bracht, voor de melding in het dashboard.
    pub laatste_ophaal: Option<Kamermelding>,
    /// Kamerleden die er bij een eerder ophalen waren en nu niet meer, zoals ze het laatst
    /// waren. Voor het voorstel van een Stoel bij een wissel (zie [`crate::zaalindeling`]).
    #[serde(default)]
    pub vertrokken: Vec<Kamerlid>,
}

/// Een Kamerlid uit de open data (`Persoon` met de `FractieZetelPersoon` die op de peildatum geldt).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Kamerlid {
    /// `Persoon.Id` in de open data.
    pub id: String,
    pub roepnaam: String,
    /// Zoals "D.A.".
    pub initialen: String,
    pub tussenvoegsel: Option<String>,
    pub achternaam: String,
    /// `Fractie.id`.
    pub fractie: String,
    /// `FractieZetel.Id`: een opvolger komt meestal op dezelfde FractieZetel.
    pub zetel: String,
    pub fractievoorzitter: bool,
    /// Sinds wanneer dit Kamerlid op deze FractieZetel zit (`JJJJ-MM-DD`).
    pub van: String,
    pub geboortedatum: Option<String>,
    pub geboorteplaats: Option<String>,
    pub woonplaats: Option<String>,
    pub foto: Foto,
    /// De eerste dag in de Kamer (`JJJJ-MM-DD`): de vroegste `FractieZetelPersoon.Van` in de open
    /// data, ook uit een eerdere Kamerperiode.
    #[serde(default)]
    #[ts(optional)]
    pub in_de_kamer_sinds: Option<String>,
    /// De plaats op de lijst van de Fractie (`FractieZetel.Gewicht` in de open data): na een
    /// verkiezing de volgorde van de kieslijst. Vult de Stoelen voorlopig in de Uitslagindeling
    /// (zie [`crate::stemmingen`]).
    #[serde(default)]
    #[ts(optional)]
    pub kieslijst: Option<u16>,
}

/// Welke portretfoto de server voor een Kamerlid heeft.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Foto {
    /// Geen foto: de Persoonskaart toont een vlak in de Fractiekleur met de voorletters.
    Geen,
    /// De foto uit de open data.
    OpenData,
    /// Een eigen foto van de omroep, in plaats van die uit de open data.
    Eigen,
}

/// Een Fractie uit de open data.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Fractie {
    /// `Fractie.Id` in de open data.
    pub id: String,
    /// Zoals "VVD" of "Groep Markuszower".
    pub afkorting: String,
    pub naam: String,
    pub zetels: u16,
}

/// Wat het ophalen de laatste keer bracht. Namen staan erin zoals [`Kamerlid::naam`].
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Kamermelding {
    pub opgehaald: String,
    pub peildatum: String,
    /// Kamerleden die er de vorige keer niet waren. Leeg bij de eerste keer ophalen.
    pub nieuw: Vec<String>,
    pub vertrokken: Vec<String>,
    /// Een nieuw Kamerlid op de FractieZetel van een vertrokken Kamerlid.
    pub opvolgers: Vec<Opvolging>,
    /// Kamerleden die nu bij een andere Fractie horen, zoals "Naam: PVV naar Groep Markuszower".
    pub andere_fractie: Vec<String>,
    /// Fracties die niet in de partijlijst stonden en er nu bij staan, met een tijdelijke kleur.
    pub fracties_nieuw: Vec<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Opvolging {
    pub zetel: String,
    pub opvolger: String,
    pub vertrokken: String,
}

/// Waarom de server een Opdracht over de Kamer niet uitvoert.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum KamerWeigering {
    #[error("er is geen Kamerlid met id \"{0}\"")]
    OnbekendKamerlid(String),
}

impl Kamerlid {
    /// "Van Groningen": de achternaam met tussenvoegsel, zoals in een lijst namen.
    pub fn achternaam_met_tussenvoegsel(&self) -> String {
        match self
            .tussenvoegsel
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            Some(t) => {
                let mut c = t.chars();
                let eerste = c
                    .next()
                    .map(|e| e.to_uppercase().collect::<String>())
                    .unwrap_or_default();
                format!("{eerste}{} {}", c.as_str(), self.achternaam)
            }
            None => self.achternaam.clone(),
        }
    }

    /// "Dieke van Groningen".
    pub fn naam(&self) -> String {
        let voor = if self.roepnaam.trim().is_empty() {
            self.initialen.trim()
        } else {
            self.roepnaam.trim()
        };
        [
            voor,
            self.tussenvoegsel.as_deref().unwrap_or("").trim(),
            self.achternaam.trim(),
        ]
        .into_iter()
        .filter(|d| !d.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
    }

    /// De voorletters op de Persoonskaart als er geen foto is: de eerste letter van de roepnaam
    /// en van de achternaam, zonder tussenvoegsel ("Dieke van Groningen" wordt "DG").
    pub fn voorletters(&self) -> String {
        let eerste = |t: &str| t.chars().find(|c| c.is_alphabetic());
        let voor = eerste(&self.roepnaam).or_else(|| eerste(&self.initialen));
        voor.into_iter()
            .chain(eerste(&self.achternaam))
            .flat_map(char::to_uppercase)
            .collect()
    }
}

impl Kamer {
    pub fn kamerlid(&self, id: &str) -> Option<&Kamerlid> {
        self.kamerleden.iter().find(|k| k.id == id)
    }

    pub fn fractie(&self, id: &str) -> Option<&Fractie> {
        self.fracties.iter().find(|f| f.id == id)
    }

    /// De sleutel van de Partij uit de partijlijst waar dit Kamerlid bij hoort.
    pub fn partij_van(&self, k: &Kamerlid) -> Option<&str> {
        self.partij_van.get(&k.fractie).map(String::as_str)
    }

    /// Kamerleden zonder foto: het dashboard waarschuwt, de Persoonskaart toont voorletters.
    pub fn zonder_foto(&self) -> impl Iterator<Item = &Kamerlid> {
        self.kamerleden.iter().filter(|k| k.foto == Foto::Geen)
    }

    pub fn zet_foto(&mut self, id: &str, foto: Foto) -> Result<(), KamerWeigering> {
        let k = self
            .kamerleden
            .iter_mut()
            .find(|k| k.id == id)
            .ok_or_else(|| KamerWeigering::OnbekendKamerlid(id.into()))?;
        k.foto = foto;
        Ok(())
    }

    /// Neemt de opgehaalde Kamer over en schrijft in [`Kamer::laatste_ophaal`] wat er veranderde.
    /// Een Fractie die niet in de partijlijst staat, komt daar bij met een tijdelijke kleur.
    pub fn lees(
        &mut self,
        cijfers: &mut Cijfers,
        opgehaald: &str,
        peildatum: &str,
        kamerleden: &[Kamerlid],
        fracties: &[Fractie],
    ) {
        let mut m = Kamermelding {
            opgehaald: opgehaald.into(),
            peildatum: peildatum.into(),
            ..Default::default()
        };

        let mut partij_van = BTreeMap::new();
        for f in fracties {
            let sleutel = match cijfers.partij_voor_fractie(&f.afkorting, &f.naam) {
                Some(p) => p.sleutel.clone(),
                None => {
                    m.fracties_nieuw.push(f.afkorting.clone());
                    cijfers
                        .voeg_fractie_zonder_lijst_toe(
                            &sleutel_van(&f.afkorting),
                            &f.naam,
                            &f.afkorting,
                        )
                        .sleutel
                        .clone()
                }
            };
            partij_van.insert(f.id.clone(), sleutel);
        }

        let afkorting = |lijst: &[Fractie], id: &str| {
            lijst
                .iter()
                .find(|f| f.id == id)
                .map_or_else(|| id.to_owned(), |f| f.afkorting.clone())
        };
        let eerste_keer = self.peildatum.is_none() && self.kamerleden.is_empty();
        if !eerste_keer {
            let vertrokken: Vec<&Kamerlid> = self
                .kamerleden
                .iter()
                .filter(|oud| !kamerleden.iter().any(|k| k.id == oud.id))
                .collect();
            for k in kamerleden {
                match self.kamerlid(&k.id) {
                    None => {
                        m.nieuw.push(k.naam());
                        if let Some(v) = vertrokken.iter().find(|v| v.zetel == k.zetel) {
                            m.opvolgers.push(Opvolging {
                                zetel: k.zetel.clone(),
                                opvolger: k.naam(),
                                vertrokken: v.naam(),
                            });
                        }
                    }
                    Some(oud) if oud.fractie != k.fractie => m.andere_fractie.push(format!(
                        "{}: {} naar {}",
                        k.naam(),
                        afkorting(&self.fracties, &oud.fractie),
                        afkorting(fracties, &k.fractie)
                    )),
                    Some(_) => {}
                }
            }
            m.vertrokken = vertrokken.iter().map(|k| k.naam()).collect();
            let weg: Vec<Kamerlid> = vertrokken.into_iter().cloned().collect();
            self.vertrokken
                .retain(|v| !weg.iter().any(|w| w.id == v.id));
            self.vertrokken.extend(weg);
        }
        // wie terugkomt, is niet meer vertrokken
        self.vertrokken
            .retain(|v| !kamerleden.iter().any(|k| k.id == v.id));

        let mut kamerleden = kamerleden.to_vec();
        kamerleden.sort_by(|a, b| {
            (a.achternaam.to_lowercase(), &a.roepnaam)
                .cmp(&(b.achternaam.to_lowercase(), &b.roepnaam))
        });
        let mut fracties = fracties.to_vec();
        fracties.sort_by(|a, b| {
            b.zetels
                .cmp(&a.zetels)
                .then_with(|| a.afkorting.cmp(&b.afkorting))
        });

        self.peildatum = Some(peildatum.into());
        self.kamerleden = kamerleden;
        self.fracties = fracties;
        self.partij_van = partij_van;
        self.laatste_ophaal = Some(m);
    }
}

/// Sleutel voor een Fractie die er nog niet is: "Groep Markuszower" wordt "groep-markuszower".
fn sleutel_van(afkorting: &str) -> String {
    afkorting
        .split(|c: char| !c.is_alphanumeric())
        .filter(|d| !d.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}
