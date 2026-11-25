//! Controle na de uitzending (#57).
//!
//! De server haalt de officiële uitslag zelf op: eerst Debat Direct (de Kamer laat die ongeveer
//! twee uur na de Stemming zien), daarna de `Stemming`-regels uit de open data (ongeveer 18 uur
//! later). Hij stuurt ze als [`Opdracht::LeesOfficieleUitslagen`]. Die zet de uitslag naast de
//! invoer van de Regie en rekent per ingevoerde Stemming een [`Controlestatus`] uit. De invoer
//! verandert pas als de Regie "Neem officieel over" kiest; "Onze invoer klopt" laat hem staan.
//! Een Graphic, ook in een Preset of in de Ether, houdt altijd zijn eigen kopie.
//!
//! Om de vertraging van Debat Direct te meten bewaart de Stand per Stemming wanneer de server de
//! uitslag daar voor het eerst zag, naast de hamerslag die Debat Direct zelf geeft.
//!
//! [`Opdracht::LeesOfficieleUitslagen`]: crate::Opdracht::LeesOfficieleUitslagen

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AfwijkendeStem, Besluit, GetikteFractie, Stem, Stemgedrag, Stemming, Uitkomst};
use crate::kamer::Kamer;

/// Waar de officiële uitslag vandaan komt.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Bron {
    /// Debat Direct van de Tweede Kamer, ongeveer twee uur na de Stemming.
    DebatDirect,
    /// De `Stemming`-regels uit de open data, ongeveer 18 uur na de Stemming. Gaat voor.
    OpenData,
}

/// De officiële uitslag van een Stemming, zoals een bron hem geeft.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct OfficieleUitslag {
    pub bron: Bron,
    pub besluit: Besluit,
    pub hoofdelijk: bool,
    /// De stem van elke Fractie die meedeed; bij een hoofdelijke stemming de stem van de meeste
    /// Kamerleden van de Fractie.
    pub fracties: Vec<GetikteFractie>,
    /// Kamerleden die anders stemden dan hun Fractie.
    pub afwijkend: Vec<AfwijkendeStem>,
    /// Het einde van de Stemming in de Zaal, als de bron het geeft (Debat Direct: `endedAt`).
    pub hamerslag: Option<String>,
}

impl OfficieleUitslag {
    /// Het officiële stemgedrag in de vorm van de invoer. Een Fractie die niet meedeed blijft open.
    pub fn als_stemgedrag(&self) -> Stemgedrag {
        Stemgedrag {
            getikt: self.fracties.clone(),
            overige: None,
            algemene_stemmen: false,
            afwijkend: self.afwijkend.clone(),
            hoofdelijk: self.hoofdelijk,
        }
    }

    /// Dezelfde uitslag, los van bron en hamerslag.
    fn zelfde_stemmen(&self, ander: &OfficieleUitslag) -> bool {
        let gesorteerd = |u: &OfficieleUitslag| {
            let mut f: Vec<_> = u
                .fracties
                .iter()
                .map(|g| (g.fractie.clone(), g.stem))
                .collect();
            let mut a: Vec<_> = u
                .afwijkend
                .iter()
                .map(|g| (g.kamerlid.clone(), g.stem))
                .collect();
            f.sort();
            a.sort();
            (u.besluit, f, a)
        };
        gesorteerd(self) == gesorteerd(ander)
    }
}

/// Wat de server bij één keer ophalen voor één Stemming vond.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct OpgehaaldeUitslag {
    /// Het id van de Stemming in de lijst.
    pub stemming: String,
    pub uitslag: OfficieleUitslag,
}

/// Hoe de invoer van de Regie zich verhoudt tot de officiële uitslag. Van ernstig naar gewoon.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[ts(export)]
pub enum Controlestatus {
    /// Aangenomen waar het verworpen was, of andersom.
    UitkomstAnders,
    /// Dezelfde uitkomst, maar een Fractie of Kamerlid stemde anders dan ingevoerd.
    VerschilPerFractie,
    /// Nog geen officiële uitslag.
    WachtOpOfficieel,
    Gelijk,
}

/// Wat de Regie koos bij een verschil.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub enum Keuze {
    OfficieelOvergenomen,
    OnzeInvoerKlopt,
}

/// Een keuze van de Regie, over deze officiële uitslag.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Beoordeling {
    pub keuze: Keuze,
    pub door: String,
    /// De officiële uitslag waarover de keuze ging. Komt er een andere, dan geldt de keuze niet meer.
    pub over: OfficieleUitslag,
}

/// Een Fractie of Kamerlid dat anders stemde dan de Regie invoerde.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Verschil {
    /// De afkorting van de Fractie, of de naam van het Kamerlid met de Fractie erachter.
    pub wie: String,
    /// Bij een Kamerlid zijn id.
    pub kamerlid: Option<String>,
    /// `None`: open in de invoer.
    pub onze: Option<Stem>,
    /// `None`: deed niet mee.
    pub officieel: Option<Stem>,
}

/// De controle van één Stemming.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Stemmingscontrole {
    pub officieel: Option<OfficieleUitslag>,
    /// Wanneer de server de uitslag voor het eerst bij Debat Direct zag.
    pub debat_direct_gezien: Option<String>,
    /// Wanneer de server de uitslag voor het eerst in de open data zag.
    pub open_data_gezien: Option<String>,
    pub beoordeling: Option<Beoordeling>,
    /// Uitgerekend na elke Opdracht. `None`: de Regie voerde niets in.
    pub status: Option<Controlestatus>,
    /// Uitgerekend: wat anders is, Fracties in de volgorde van de Kamer, dan Kamerleden.
    pub verschillen: Vec<Verschil>,
    /// Uitgerekend: de uitkomst uit de officiële stemmen en de Zetels, met het officiële besluit.
    pub officiele_uitkomst: Option<Uitkomst>,
    /// Uitgerekend: de keuze van de Regie, zolang die over de officiële uitslag van nu gaat.
    pub afgehandeld: Option<Keuze>,
}

impl Stemmingscontrole {
    /// Neemt een opgehaalde uitslag over: de open data gaat voor Debat Direct.
    pub(super) fn lees(&mut self, opgehaald: &str, u: &OfficieleUitslag) {
        let gezien = match u.bron {
            Bron::DebatDirect => &mut self.debat_direct_gezien,
            Bron::OpenData => &mut self.open_data_gezien,
        };
        gezien.get_or_insert_with(|| opgehaald.to_owned());
        let vervang = match &self.officieel {
            None => true,
            Some(o) => o.bron == Bron::DebatDirect || u.bron == Bron::OpenData,
        };
        if vervang {
            let mut nieuw = u.clone();
            if nieuw.hamerslag.is_none() {
                nieuw.hamerslag = self.officieel.as_ref().and_then(|o| o.hamerslag.clone());
            }
            self.officieel = Some(nieuw);
        }
    }
}

impl Stemming {
    /// Of de Regie iets invoerde voor deze Stemming.
    pub fn ingevoerd(&self) -> bool {
        self.vastgelegd
            || self.stemgedrag
                != Stemgedrag {
                    hoofdelijk: self.stemgedrag.hoofdelijk,
                    ..Stemgedrag::default()
                }
    }

    /// Rekent status, verschillen en de officiële uitkomst opnieuw uit.
    pub(super) fn reken_controle(&mut self, kamer: &Kamer) {
        let ingevoerd = self.ingevoerd();
        let c = &mut self.controle;
        c.verschillen.clear();
        c.officiele_uitkomst = None;
        c.afgehandeld = None;
        c.status = None;
        let Some(o) = &c.officieel else {
            if ingevoerd {
                c.status = Some(Controlestatus::WachtOpOfficieel);
            }
            return;
        };
        let officieel = o.als_stemgedrag();
        c.officiele_uitkomst = Some(Uitkomst {
            besluit: Some(o.besluit),
            ..officieel.uitkomst(kamer)
        });
        c.afgehandeld = c
            .beoordeling
            .as_ref()
            .filter(|b| b.over.zelfde_stemmen(o))
            .map(|b| b.keuze);
        if !ingevoerd {
            return;
        }
        let onze = &self.stemgedrag;
        for f in &kamer.fracties {
            let (a, b) = (onze.stem_van(&f.id), officieel.stem_van(&f.id));
            if a != b {
                c.verschillen.push(Verschil {
                    wie: f.afkorting.clone(),
                    kamerlid: None,
                    onze: a,
                    officieel: b,
                });
            }
        }
        // Een Kamerlid telt mee als het in een van beide anders stemt dan zijn Fractie.
        let leden = onze
            .afwijkend
            .iter()
            .chain(&officieel.afwijkend)
            .map(|a| a.kamerlid.as_str());
        let mut gezien = Vec::new();
        for id in leden {
            if gezien.contains(&id) {
                continue;
            }
            gezien.push(id);
            let Some(k) = kamer.kamerlid(id) else {
                continue;
            };
            let stem = |sg: &Stemgedrag| {
                sg.afwijkend
                    .iter()
                    .find(|a| a.kamerlid == id)
                    .map(|a| a.stem)
                    .or_else(|| sg.stem_van(&k.fractie))
            };
            let (a, b) = (stem(onze), stem(&officieel));
            if a != b {
                let fractie = kamer
                    .fractie(&k.fractie)
                    .map_or(k.fractie.as_str(), |f| f.afkorting.as_str());
                c.verschillen.push(Verschil {
                    wie: format!("{} ({fractie})", k.naam()),
                    kamerlid: Some(id.to_owned()),
                    onze: a,
                    officieel: b,
                });
            }
        }
        c.status = Some(if self.uitkomst.besluit.is_some_and(|b| b != o.besluit) {
            Controlestatus::UitkomstAnders
        } else if !c.verschillen.is_empty() {
            Controlestatus::VerschilPerFractie
        } else {
            Controlestatus::Gelijk
        });
    }
}
