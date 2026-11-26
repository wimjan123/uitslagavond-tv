//! Stoelbeeld: een losse kleuring van de 150 Stoelen in de Zaal zonder 2D-Graphic eroverheen
//! (ADR 0005), met eigen Kleurovergang en Kleurkracht.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::cijfers::CijferWeigering;
use crate::graphic::Indeling;
use crate::sjabloon::Kleurovergang;
use crate::uitslagindeling::{HUIDIGE_KAMER, Zetelstand};
use crate::{AANTAL_STOELEN, Kleur, Stand, Weigering};

/// Waar de stoelkleuren van een [`Stoelbeeld`] vandaan komen.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "soort")]
#[ts(export)]
pub enum Stoelbron {
    /// Alle Stoelen terug naar hun eigen blauwe leer.
    Uit,
    /// Een volledige uitslagset (in Uitslagindeling) of de huidige Kamer (in Zaalindeling).
    Set {
        set: String,
        #[serde(default)]
        indeling: Indeling,
    },
    /// Eén of meer Partijen uitgelicht in hun eigen kleur; alle andere Stoelen grijs.
    Partijen {
        set: String,
        partijen: Vec<String>,
    },
    /// Een coalitie als één blok links in de Zaal; de rest gedimd.
    Coalitie {
        set: String,
        partijen: Vec<String>,
    },
    /// Voor- en tegenkleuren van een Stemming op de Stoelen.
    Stemming {
        stemming: String,
    },
    /// Directe kleur per Stoel (150 kleuren, of leeg voor blauw leer), bijv. los aangeklikte Stoelen.
    Los {
        #[ts(type = "Array<string>")]
        kleuren: Vec<Kleur>,
    },
}

/// Een losse stoelkleuring voor Voorbeeld (Preview) of Ether (Live).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Stoelbeeld {
    pub bron: Stoelbron,
    /// Eigen Kleurovergang (Golf, Vervagen of Opbouw); `None` gebruikt die van de Stijl.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub overgang: Option<Kleurovergang>,
    /// Eigen Kleurkracht (0..=100); `None` gebruikt die van de Stijl.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kleurkracht: Option<u8>,
    /// Korte naam voor de laag-chip, zoals "VVD uitgelicht" of "Exitpoll 21:00".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
}

impl Stoelbeeld {
    /// Controleert de velden en berekent de 150 stoelkleuren (of leeg bij [`Stoelbron::Uit`]).
    pub fn bereken_kleuren(&self, stand: &Stand) -> Result<Vec<Kleur>, Weigering> {
        if let Some(k) = self.kleurkracht
            && k > 100
        {
            return Err(Weigering::KleurkrachtOngeldig(k));
        }
        let volgorde = stand.uitslagvolgorde.bepaal(&stand.cijfers);
        let onbekend = |set: &str| Weigering::from(CijferWeigering::OnbekendeSet(set.into()));
        let zetelstand_voor = |set: &str| -> Result<Zetelstand, Weigering> {
            if set == HUIDIGE_KAMER {
                Ok(Zetelstand::van_kamer(&volgorde, &stand.cijfers, &stand.kamer))
            } else {
                Zetelstand::van(&volgorde, &stand.cijfers, set).ok_or_else(|| onbekend(set))
            }
        };
        match &self.bron {
            Stoelbron::Uit => Ok(Vec::new()),
            Stoelbron::Set { set, indeling } => {
                let zs = match indeling {
                    Indeling::Zaalindeling => {
                        Zetelstand::van_zaal(&volgorde, &stand.cijfers, &stand.kamer, &stand.zaalindeling)
                    }
                    Indeling::Uitslagindeling => zetelstand_voor(set)?,
                };
                Ok(zs.stoelkleuren())
            }
            Stoelbron::Partijen { set, partijen } => {
                let zs = zetelstand_voor(set)?;
                Ok(zs.stoelkleuren_van_partijen(partijen))
            }
            Stoelbron::Coalitie { set, partijen } => {
                let zs = zetelstand_voor(set)?;
                Ok(zs.coalitiekleuren(partijen))
            }
            Stoelbron::Stemming { stemming } => {
                let stoelbron = crate::stemmingen::Stoelbron {
                    zaal: &stand.zaalindeling,
                    volgorde: &volgorde,
                };
                let s = stand
                    .stemmingen
                    .lijst
                    .iter()
                    .find(|x| x.punt.id == *stemming)
                    .ok_or_else(|| {
                        Weigering::from(crate::stemmingen::StemmingWeigering::OnbekendeStemming(
                            stemming.clone(),
                        ))
                    })?;
                let uitslag = crate::stemmingen::Stemmingsuitslag::van(
                    s,
                    &stand.kamer,
                    &stand.cijfers,
                    stand.stemmingen.indeling,
                    &stoelbron,
                )?;
                Ok(uitslag.stoelkleuren().unwrap_or_default())
            }
            Stoelbron::Los { kleuren } => {
                if !kleuren.is_empty() && kleuren.len() != AANTAL_STOELEN {
                    return Err(Weigering::StoelkleurenAantal(kleuren.len()));
                }
                Ok(kleuren.clone())
            }
        }
    }
}
