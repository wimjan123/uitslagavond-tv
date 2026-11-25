//! De Zaalindeling: welk Kamerlid op welke Stoel zit, zoals de Kamer zelf de plekken toewijst
//! (besluiten #7, #14 en #25).
//!
//! Per Kamerperiode is er één bestand, zoals `zaalindeling/tk2025.json`: voor elke Stoel (vak,
//! rij, nummer) het id van het Kamerlid (`Persoon.Id` in de open data). De Fractie volgt uit het
//! Kamerlid, dus bij een overstap verandert alleen de kleur. De Regie verschuift Stoelen door het
//! bestand aan te passen; de server leest het bij elke wijziging in.
//!
//! Bij een wissel stelt het systeem voor dat het nieuwe Kamerlid de Stoel van het vertrokken
//! Kamerlid krijgt ([`Zaalindeling::voorstellen`]); tot de Regie dat bevestigt heeft het nieuwe
//! Kamerlid geen Stoel. De FractieZetel uit de open data is geen Stoel: die helpt alleen bij het
//! voorstel.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Stoel;
use crate::kamer::{Kamer, Kamerlid};

/// De Zaalindeling van één Kamerperiode.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Zaalindeling {
    /// Het jaar waarin de Kamer beëdigd werd, zoals "2025".
    pub kamerperiode: String,
    /// Waar de indeling vandaan komt.
    #[serde(default)]
    pub bron: String,
    pub stoelen: Vec<Bezetting>,
}

/// Wie er op één Stoel zit.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Bezetting {
    pub vak: u8,
    pub rij: u8,
    pub nummer: u8,
    /// `Persoon.Id` in de open data. Leeg: niemand.
    #[serde(default)]
    pub kamerlid: Option<String>,
    /// De naam, alleen om het bestand te kunnen lezen; het id telt.
    #[serde(default)]
    pub naam: String,
}

impl Bezetting {
    pub fn stoel(&self) -> Stoel {
        Stoel { vak: self.vak, rij: self.rij, nummer: self.nummer }
    }
}

/// Wat er niet klopt aan een Zaalindeling. Leeg: alles in orde.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Controle {
    /// Stoelen van de Zaal die niet in het bestand staan.
    pub ontbrekend: Vec<Stoel>,
    /// Stoelen die meer dan eens in het bestand staan.
    pub dubbele_stoelen: Vec<Stoel>,
    /// Stoelen die niet in de Zaal bestaan.
    pub onbekende_stoelen: Vec<Stoel>,
    /// Kamerleden op meer dan één Stoel.
    pub dubbele_kamerleden: Vec<String>,
}

impl Controle {
    pub fn in_orde(&self) -> bool {
        *self == Controle::default()
    }
}

/// Een voorstel bij een wissel: het nieuwe Kamerlid krijgt de Stoel van het vertrokken Kamerlid.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Stoelvoorstel {
    pub stoel: Stoel,
    /// Het id van het nieuwe Kamerlid.
    pub kamerlid: String,
    pub naam: String,
    /// De naam van het vertrokken Kamerlid.
    pub vertrokken: String,
}

/// Wat het dashboard over de Zaalindeling laat zien: uitgerekend na elke Opdracht.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Zaalmelding {
    pub controle: Controle,
    /// Kamerleden van de Kamer die zit zonder Stoel, als naam.
    pub zonder_stoel: Vec<String>,
    pub voorstellen: Vec<Stoelvoorstel>,
}

/// Waarom de server een Opdracht over de Zaalindeling niet uitvoert.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ZaalWeigering {
    #[error("er is geen Kamerlid met id \"{0}\"")]
    OnbekendKamerlid(String),
    #[error("Stoel {0} bestaat niet")]
    OnbekendeStoel(String),
    #[error("op Stoel {stoel} zit al {naam}")]
    StoelBezet { stoel: String, naam: String },
}

fn tekst(s: Stoel) -> String {
    format!("{}-{}-{}", s.vak, s.rij, s.nummer)
}

impl Zaalindeling {
    pub fn uit_json(json: &str) -> Result<Zaalindeling, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    /// Het bestand zoals de server het terugschrijft: één regel per Stoel, zodat de Regie het
    /// met de hand kan aanpassen.
    pub fn als_json(&self) -> String {
        let regel = |b: &Bezetting| {
            let kamerlid = b
                .kamerlid
                .as_ref()
                .map_or_else(|| "null".to_owned(), |k| serde_json::to_string(k).unwrap());
            format!(
                "    {{ \"vak\": {}, \"rij\": {}, \"nummer\": {}, \"kamerlid\": {kamerlid}, \"naam\": {} }}",
                b.vak,
                b.rij,
                b.nummer,
                serde_json::to_string(&b.naam).unwrap()
            )
        };
        let stoelen: Vec<String> = self.stoelen.iter().map(regel).collect();
        format!(
            "{{\n  \"kamerperiode\": {},\n  \"bron\": {},\n  \"stoelen\": [\n{}\n  ]\n}}\n",
            serde_json::to_string(&self.kamerperiode).unwrap(),
            serde_json::to_string(&self.bron).unwrap(),
            stoelen.join(",\n")
        )
    }

    /// Meldt ontbrekende, dubbele en onbekende Stoelen en Kamerleden op meer dan één Stoel.
    pub fn controleer(&self) -> Controle {
        let mut c = Controle::default();
        let mut gezien = BTreeSet::new();
        let mut plek_van: BTreeMap<&str, BTreeSet<(u8, u8, u8)>> = BTreeMap::new();
        for b in &self.stoelen {
            let s = b.stoel();
            if s.index().is_none() {
                c.onbekende_stoelen.push(s);
            } else if !gezien.insert((s.vak, s.rij, s.nummer)) && !c.dubbele_stoelen.contains(&s) {
                c.dubbele_stoelen.push(s);
            }
            if let Some(k) = &b.kamerlid {
                plek_van.entry(k).or_default().insert((s.vak, s.rij, s.nummer));
            }
        }
        c.ontbrekend = Stoel::alle()
            .filter(|s| !gezien.contains(&(s.vak, s.rij, s.nummer)))
            .collect();
        c.dubbele_kamerleden = plek_van
            .into_iter()
            .filter(|(_, plekken)| plekken.len() > 1)
            .map(|(k, _)| k.to_owned())
            .collect();
        c
    }

    /// Het id van het Kamerlid op deze Stoel, ook als dat Kamerlid vertrokken is.
    pub fn op(&self, stoel: Stoel) -> Option<&str> {
        self.stoelen
            .iter()
            .find(|b| b.stoel() == stoel)
            .and_then(|b| b.kamerlid.as_deref())
    }

    /// De Stoel van een Kamerlid.
    pub fn stoel_van(&self, kamerlid: &str) -> Option<Stoel> {
        self.stoelen
            .iter()
            .find(|b| b.kamerlid.as_deref() == Some(kamerlid))
            .map(Bezetting::stoel)
    }

    /// Per Stoel, in de volgorde van [`Stoel::alle`], het Kamerlid van de Kamer die zit. Een
    /// Stoel van een vertrokken Kamerlid is leeg.
    pub fn kamerleden<'a>(&self, kamer: &'a Kamer) -> Vec<Option<&'a Kamerlid>> {
        Stoel::alle()
            .map(|s| self.op(s).and_then(|id| kamer.kamerlid(id)))
            .collect()
    }

    /// Bij een wissel: per nieuw Kamerlid zonder Stoel de Stoel van een vertrokken Kamerlid.
    /// Eerst het vertrokken Kamerlid op dezelfde FractieZetel, anders een uit dezelfde Fractie.
    pub fn voorstellen(&self, kamer: &Kamer) -> Vec<Stoelvoorstel> {
        // Stoelen met een Kamerlid dat niet meer in de Kamer zit, met wat we van hem weten
        let mut vrij: Vec<(Stoel, &Kamerlid)> = self
            .stoelen
            .iter()
            .filter_map(|b| {
                let id = b.kamerlid.as_deref()?;
                if kamer.kamerlid(id).is_some() {
                    return None;
                }
                Some((b.stoel(), kamer.vertrokken.iter().find(|v| v.id == id)?))
            })
            .collect();
        let nieuw: Vec<&Kamerlid> = kamer
            .kamerleden
            .iter()
            .filter(|k| self.stoel_van(&k.id).is_none())
            .collect();
        let mut uit = Vec::new();
        for pas in [
            (|n: &Kamerlid, v: &Kamerlid| n.zetel == v.zetel) as fn(&Kamerlid, &Kamerlid) -> bool,
            |n, v| n.fractie == v.fractie,
        ] {
            for n in &nieuw {
                if uit.iter().any(|u: &Stoelvoorstel| u.kamerlid == n.id) {
                    continue;
                }
                if let Some(i) = vrij.iter().position(|(_, v)| pas(n, v)) {
                    let (stoel, v) = vrij.remove(i);
                    uit.push(Stoelvoorstel {
                        stoel,
                        kamerlid: n.id.clone(),
                        naam: n.naam(),
                        vertrokken: v.naam(),
                    });
                }
            }
        }
        uit.sort_by_key(|u| u.stoel.index());
        uit
    }

    /// Wat het dashboard meldt over deze Zaalindeling en de Kamer die zit.
    pub fn melding(&self, kamer: &Kamer) -> Zaalmelding {
        if self.stoelen.is_empty() {
            return Zaalmelding::default();
        }
        Zaalmelding {
            controle: self.controleer(),
            zonder_stoel: kamer
                .kamerleden
                .iter()
                .filter(|k| self.stoel_van(&k.id).is_none())
                .map(Kamerlid::naam)
                .collect(),
            voorstellen: self.voorstellen(kamer),
        }
    }

    /// Geeft een Kamerlid van de Kamer die zit deze Stoel, zoals na een voorstel bij een wissel.
    /// Had het Kamerlid al een Stoel, dan is die daarna leeg. Een Stoel waar een ander Kamerlid
    /// van de Kamer zit, geeft de server niet weg.
    pub fn geef_stoel(&mut self, kamer: &Kamer, kamerlid: &str, stoel: Stoel) -> Result<(), ZaalWeigering> {
        let k = kamer
            .kamerlid(kamerlid)
            .ok_or_else(|| ZaalWeigering::OnbekendKamerlid(kamerlid.into()))?;
        if stoel.index().is_none() {
            return Err(ZaalWeigering::OnbekendeStoel(tekst(stoel)));
        }
        if let Some(ander) = self.op(stoel).filter(|id| *id != kamerlid).and_then(|id| kamer.kamerlid(id)) {
            return Err(ZaalWeigering::StoelBezet { stoel: tekst(stoel), naam: ander.naam() });
        }
        for b in self.stoelen.iter_mut().filter(|b| b.kamerlid.as_deref() == Some(kamerlid)) {
            b.kamerlid = None;
            b.naam.clear();
        }
        let naam = k.naam();
        match self.stoelen.iter_mut().find(|b| b.stoel() == stoel) {
            Some(b) => {
                b.kamerlid = Some(kamerlid.into());
                b.naam = naam;
            }
            None => {
                self.stoelen.push(Bezetting {
                    vak: stoel.vak,
                    rij: stoel.rij,
                    nummer: stoel.nummer,
                    kamerlid: Some(kamerlid.into()),
                    naam,
                });
                self.stoelen.sort_by_key(|b| b.stoel().index());
            }
        }
        Ok(())
    }
}
