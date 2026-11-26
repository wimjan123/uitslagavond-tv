//! Stemmingen: de Stemmingslijst van de dag en de invoer van de Regie.
//!
//! De server leest de Stemmingslijst uit de open data ([`Opdracht::LeesStemmingslijst`]). De Regie
//! vinkt aan welke Stemmingen in de uitzending komen en typt een motie die er niet in staat zelf in.
//! Tijdens de Stemming volgt de Regie de Voorzitter: Fracties aantikken in zijn volgorde, voor of
//! tegen, dan "Overige tegen" (of voor), of "Met algemene stemmen". Een Kamerlid dat anders stemt
//! dan zijn Fractie is een [`AfwijkendeStem`]. De server rekent na elke wijziging de [`Uitkomst`]
//! uit de Zetels van de Fracties uit.
//!
//! "Vastleggen en klaarzetten" ([`Opdracht::LegStemmingVast`]) zet een Graphic klaar met een kopie
//! van de Stemming als gegevens ([`Stemmingsuitslag`]).
//!
//! [`Opdracht::LeesStemmingslijst`]: crate::Opdracht::LeesStemmingslijst
//! [`Opdracht::LegStemmingVast`]: crate::Opdracht::LegStemmingVast

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::cijfers::Cijfers;
use crate::kamer::{Kamer, Kamerlid};
use crate::uitslagindeling::{Volgorde, stoelen_van_links_naar_rechts};
use crate::zaalindeling::Zaalindeling;
use crate::{AANTAL_STOELEN, Graphic, Indeling};

mod controle;
pub use controle::{
    Beoordeling, Bron, Controlestatus, Keuze, OfficieleUitslag, OpgehaaldeUitslag, Stemmingscontrole,
    Verschil,
};

/// Langste titel van een motie die de Regie zelf intypt, in tekens.
pub const MAX_TITEL: usize = 300;

/// De Stemmingslijst van één dag, met wat de Regie erbij invoerde.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Stemmingen {
    /// De dag van de lijst (`JJJJ-MM-DD`).
    pub dag: Option<String>,
    /// Wanneer de server de lijst het laatst uit de open data las.
    pub ingelezen: Option<String>,
    /// In de volgorde van de Stemmingslijst; moties die de Regie intypte staan achteraan.
    pub lijst: Vec<Stemming>,
    /// Waar een Stemming de Stoelen kleurt. Standaard de Zaalindeling; tussen de beëdiging en
    /// een nieuwe Zaalindeling kiest de Regie de Uitslagindeling (besluit #25).
    #[serde(default = "zaalindeling")]
    pub indeling: Indeling,
}

impl Default for Stemmingen {
    fn default() -> Self {
        Stemmingen { dag: None, ingelezen: None, lijst: Vec::new(), indeling: Indeling::Zaalindeling }
    }
}

fn zaalindeling() -> Indeling {
    Indeling::Zaalindeling
}

/// Waar Vastleggen de Stoelen van een Stemming vandaan haalt.
pub struct Stoelbron<'a> {
    pub zaal: &'a Zaalindeling,
    /// De Uitslagindeling van nu.
    pub volgorde: &'a Volgorde,
}

/// Eén punt op de Stemmingslijst zoals de open data het geeft (een `Zaak`).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Lijstpunt {
    /// `Zaak.Id` in de open data, of `hand-<n>` voor een motie die de Regie intypte.
    pub id: String,
    /// "Motie", "Amendement" of "Wetgeving".
    pub soort: Option<String>,
    /// Zoals "37020-XX, nr. 17".
    pub nummer: Option<String>,
    /// Zoals "Motie van het lid Nanninga over ...".
    pub titel: String,
    /// Zoals "Nanninga" of "Heera Dijk en Tijmstra".
    pub indieners: Option<String>,
    /// Het agendapunt, zoals "Moties ingediend bij het tweeminutendebat Kernenergie".
    pub agendapunt: Option<String>,
}

/// Een Stemming op de lijst, met de invoer van de Regie.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Stemming {
    pub punt: Lijstpunt,
    /// De Regie typte deze motie zelf in.
    pub met_de_hand: bool,
    /// Aangevinkt voor de uitzending.
    pub voor_uitzending: bool,
    pub stemgedrag: Stemgedrag,
    /// Uitgerekend door de server uit het stemgedrag en de Zetels van de Fracties.
    pub uitkomst: Uitkomst,
    /// "Vastleggen en klaarzetten" is gedaan.
    pub vastgelegd: bool,
    /// Op slot: de server weigert ander stemgedrag.
    pub vergrendeld: bool,
    /// Wanneer een Graphic van deze Stemming voor het eerst in de Ether kwam, in milliseconden
    /// sinds 1970 (UTC).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub in_ether_ms: Option<u64>,
    /// De officiële uitslag naast de invoer (Controle na de uitzending).
    #[serde(default)]
    pub controle: Stemmingscontrole,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[ts(export)]
pub enum Stem {
    Voor,
    Tegen,
}

impl Stem {
    pub fn andere(self) -> Stem {
        match self {
            Stem::Voor => Stem::Tegen,
            Stem::Tegen => Stem::Voor,
        }
    }
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[ts(export)]
pub enum Besluit {
    Aangenomen,
    Verworpen,
}

/// Hoe de Kamer stemde, zoals de Regie het invoert terwijl de Voorzitter het voorleest.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Stemgedrag {
    /// De Fracties die de Regie aantikte, in de volgorde van de Voorzitter.
    pub getikt: Vec<GetikteFractie>,
    /// "Overige tegen" of "Overige voor": de Fracties die niet getikt zijn.
    pub overige: Option<Stem>,
    /// "Met algemene stemmen": elke Fractie voor, zonder uitzondering.
    pub algemene_stemmen: bool,
    /// Kamerleden die anders stemmen dan hun Fractie.
    pub afwijkend: Vec<AfwijkendeStem>,
    /// Een hoofdelijke stemming: dezelfde invoer, een ander Sjabloon.
    pub hoofdelijk: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct GetikteFractie {
    /// `Fractie.id`.
    pub fractie: String,
    pub stem: Stem,
}

/// De stem van een Kamerlid dat anders stemt dan zijn Fractie.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct AfwijkendeStem {
    /// `Kamerlid.id`.
    pub kamerlid: String,
    pub stem: Stem,
}

/// Zetels voor, tegen en nog open, en het besluit zodra niets meer open is.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Uitkomst {
    pub voor: u16,
    pub tegen: u16,
    /// Zetels van Fracties die nog niet getikt zijn en niet onder "Overige" vallen.
    pub open: u16,
    /// Pas als niets meer open is. Bij staken der stemmen: verworpen.
    pub besluit: Option<Besluit>,
}

impl Default for Uitkomst {
    fn default() -> Self {
        Uitkomst { voor: 0, tegen: 0, open: 0, besluit: None }
    }
}

/// Waarom de server een Opdracht over Stemmingen niet uitvoert.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum StemmingWeigering {
    #[error("er is geen Stemming met id \"{0}\"")]
    OnbekendeStemming(String),
    #[error("deze Stemming is vergrendeld")]
    Vergrendeld,
    #[error("er is geen Fractie met id \"{0}\"")]
    OnbekendeFractie(String),
    #[error("er is geen Kamerlid met id \"{0}\"")]
    OnbekendKamerlid(String),
    #[error("{0} is twee keer getikt")]
    FractieTweeKeer(String),
    #[error("met algemene stemmen kan geen Fractie of Kamerlid anders stemmen")]
    AlgemeneStemmenMetUitzondering,
    #[error("er zijn nog {0} Zetels open: tik de Fracties of kies Overige")]
    FractiesOpen(u16),
    #[error("de titel is leeg")]
    TitelLeeg,
    #[error("de titel is langer dan {MAX_TITEL} tekens")]
    TitelTeLang,
    #[error("er is nog geen officiële uitslag van deze Stemming")]
    GeenOfficieleUitslag,
}

impl Stemgedrag {
    /// De stem van een Fractie: getikt, of via Overige of algemene stemmen. `None`: nog open.
    pub fn stem_van(&self, fractie: &str) -> Option<Stem> {
        if self.algemene_stemmen {
            return Some(Stem::Voor);
        }
        self.getikt
            .iter()
            .find(|g| g.fractie == fractie)
            .map(|g| g.stem)
            .or(self.overige)
    }

    /// De stem van een Kamerlid: zijn afwijkende stem, anders die van zijn Fractie.
    pub fn stem_van_kamerlid(&self, k: &Kamerlid) -> Option<Stem> {
        self.afwijkend
            .iter()
            .find(|a| a.kamerlid == k.id)
            .map(|a| a.stem)
            .or_else(|| self.stem_van(&k.fractie))
    }

    /// De stem per Stoel in de Uitslagindeling: elke Fractie één blok zo groot als haar Zetels,
    /// van links naar rechts in de volgorde van de Uitslagindeling. Bij een hoofdelijke stemming
    /// krijgen de Kamerleden binnen hun blok op volgorde van de kieslijst een Stoel, met hun eigen
    /// stem; anders heeft niemand een Stoel en kleurt een afwijkende stem niets.
    fn stoelen_in_uitslagindeling(&self, kamer: &Kamer, volgorde: &Volgorde) -> Vec<Option<Stem>> {
        let mut uit = vec![None; AANTAL_STOELEN];
        let mut fracties: Vec<_> = kamer.fracties.iter().collect();
        fracties.sort_by_key(|f| {
            kamer.partij_van.get(&f.id).map_or(usize::MAX, |p| volgorde.plaats(p))
        });
        let mut stoelen = stoelen_van_links_naar_rechts().into_iter();
        for f in fracties {
            let leden = if self.hoofdelijk { op_kieslijst(kamer, &f.id) } else { Vec::new() };
            let fractie = self.stem_van(&f.id);
            for (j, stoel) in stoelen.by_ref().take(usize::from(f.zetels)).enumerate() {
                uit[stoel] = leden.get(j).map_or(fractie, |k| self.stem_van_kamerlid(k));
            }
        }
        uit
    }

    /// Weigert stemgedrag met een Fractie of Kamerlid dat niet in de Kamer zit.
    pub fn controleer(&self, kamer: &Kamer) -> Result<(), StemmingWeigering> {
        if self.algemene_stemmen
            && (!self.getikt.is_empty() || self.overige.is_some() || !self.afwijkend.is_empty())
        {
            return Err(StemmingWeigering::AlgemeneStemmenMetUitzondering);
        }
        for (i, g) in self.getikt.iter().enumerate() {
            let f = kamer
                .fractie(&g.fractie)
                .ok_or_else(|| StemmingWeigering::OnbekendeFractie(g.fractie.clone()))?;
            if self.getikt[..i].iter().any(|o| o.fractie == g.fractie) {
                return Err(StemmingWeigering::FractieTweeKeer(f.afkorting.clone()));
            }
        }
        for a in &self.afwijkend {
            if kamer.kamerlid(&a.kamerlid).is_none() {
                return Err(StemmingWeigering::OnbekendKamerlid(a.kamerlid.clone()));
            }
        }
        Ok(())
    }

    /// Telt per Zetel: elke Fractie met haar stem, behalve haar Kamerleden met een afwijkende
    /// stem, die elk één Zetel aan hun eigen kant tellen.
    pub fn uitkomst(&self, kamer: &Kamer) -> Uitkomst {
        let mut u = Uitkomst::default();
        let mut tel = |stem: Option<Stem>, n: u16| match stem {
            Some(Stem::Voor) => u.voor += n,
            Some(Stem::Tegen) => u.tegen += n,
            None => u.open += n,
        };
        for f in &kamer.fracties {
            let afwijkers = self
                .afwijkend
                .iter()
                .filter(|a| kamer.kamerlid(&a.kamerlid).is_some_and(|k| k.fractie == f.id))
                .count();
            let afwijkers = u16::try_from(afwijkers).unwrap_or(u16::MAX);
            tel(self.stem_van(&f.id), f.zetels.saturating_sub(afwijkers));
        }
        for a in &self.afwijkend {
            if kamer.kamerlid(&a.kamerlid).is_some() {
                tel(Some(a.stem), 1);
            }
        }
        if u.open == 0 && u.voor + u.tegen > 0 {
            u.besluit = Some(if u.voor > u.tegen { Besluit::Aangenomen } else { Besluit::Verworpen });
        }
        u
    }
}

impl Stemmingen {
    pub fn stemming(&self, id: &str) -> Option<&Stemming> {
        self.lijst.iter().find(|s| s.punt.id == id)
    }

    fn stemming_mut(&mut self, id: &str) -> Result<&mut Stemming, StemmingWeigering> {
        self.lijst
            .iter_mut()
            .find(|s| s.punt.id == id)
            .ok_or_else(|| StemmingWeigering::OnbekendeStemming(id.into()))
    }

    /// Neemt de Stemmingslijst van `dag` over. Op dezelfde dag blijft wat de Regie invoerde; een
    /// punt dat van de lijst verdween blijft staan als de Regie er iets mee deed. Een nieuwe dag
    /// vervangt de lijst pas als er die dag Stemmingen zijn, zodat een uitzending na middernacht
    /// haar lijst houdt.
    pub fn lees(&mut self, dag: &str, ingelezen: &str, punten: &[Lijstpunt], kamer: &Kamer) {
        if self.dag.as_deref() != Some(dag) {
            if punten.is_empty() && self.dag.is_some() {
                return;
            }
            self.dag = Some(dag.into());
            self.lijst.clear();
        }
        let mut oud = std::mem::take(&mut self.lijst);
        let mut lijst: Vec<Stemming> = punten
            .iter()
            .map(|p| match oud.iter().position(|s| s.punt.id == p.id) {
                Some(i) => {
                    let mut s = oud.remove(i);
                    s.punt = p.clone();
                    s
                }
                None => Stemming::nieuw(p.clone(), kamer),
            })
            .collect();
        lijst.extend(oud.into_iter().filter(|s| {
            s.met_de_hand || s.vastgelegd || s.voor_uitzending || s.stemgedrag != Stemgedrag::default()
        }));
        self.lijst = lijst;
        self.ingelezen = Some(ingelezen.into());
    }

    /// Een motie die niet in de lijst staat: de Regie typt de titel.
    pub fn voeg_toe(&mut self, titel: &str, volgnummer: u64, kamer: &Kamer) -> Result<(), StemmingWeigering> {
        let titel = titel.trim();
        if titel.is_empty() {
            return Err(StemmingWeigering::TitelLeeg);
        }
        if titel.chars().count() > MAX_TITEL {
            return Err(StemmingWeigering::TitelTeLang);
        }
        let mut s = Stemming::nieuw(
            Lijstpunt {
                id: format!("hand-{volgnummer}"),
                soort: None,
                nummer: None,
                titel: titel.into(),
                indieners: None,
                agendapunt: None,
            },
            kamer,
        );
        s.met_de_hand = true;
        s.voor_uitzending = true;
        self.lijst.push(s);
        Ok(())
    }

    pub fn zet_voor_uitzending(&mut self, id: &str, aan: bool) -> Result<(), StemmingWeigering> {
        self.stemming_mut(id)?.voor_uitzending = aan;
        Ok(())
    }

    pub fn zet_stemgedrag(&mut self, id: &str, stemgedrag: &Stemgedrag, kamer: &Kamer) -> Result<(), StemmingWeigering> {
        let s = self.stemming_mut(id)?;
        if s.vergrendeld {
            return Err(StemmingWeigering::Vergrendeld);
        }
        stemgedrag.controleer(kamer)?;
        s.stemgedrag = stemgedrag.clone();
        s.uitkomst = stemgedrag.uitkomst(kamer);
        Ok(())
    }

    pub fn vergrendel(&mut self, id: &str, aan: bool) -> Result<(), StemmingWeigering> {
        self.stemming_mut(id)?.vergrendeld = aan;
        Ok(())
    }

    /// De Graphic voor een Stemming zonder open Fracties: de Uitslag van een Stemming, of bij een
    /// hoofdelijke stemming de Hoofdelijke stemming. Markeert de Stemming als vastgelegd.
    pub fn leg_vast(
        &mut self,
        id: &str,
        kamer: &Kamer,
        cijfers: &Cijfers,
        stoelen: &Stoelbron,
    ) -> Result<Graphic, StemmingWeigering> {
        let indeling = self.indeling;
        let s = self.stemming_mut(id)?;
        let uitslag = Stemmingsuitslag::van(s, kamer, cijfers, indeling, stoelen)?;
        s.vastgelegd = true;
        Ok(if s.stemgedrag.hoofdelijk {
            Graphic::HoofdelijkeStemming { uitslag }
        } else {
            Graphic::UitslagStemming { uitslag }
        })
    }

    /// Neemt de officiële uitslagen over die de server ophaalde. De invoer blijft staan.
    pub fn lees_officieel(&mut self, opgehaald: &str, uitslagen: &[OpgehaaldeUitslag]) {
        for u in uitslagen {
            if let Some(s) = self.lijst.iter_mut().find(|s| s.punt.id == u.stemming) {
                s.controle.lees(opgehaald, &u.uitslag);
            }
        }
    }

    /// "Neem officieel over": de officiële stemmen worden de invoer. Ook bij een vergrendelde
    /// Stemming, want het is een bewuste keuze. Een Graphic houdt zijn eigen kopie.
    pub fn neem_officieel_over(&mut self, id: &str, door: &str, kamer: &Kamer) -> Result<(), StemmingWeigering> {
        let s = self.stemming_mut(id)?;
        let o = s.controle.officieel.clone().ok_or(StemmingWeigering::GeenOfficieleUitslag)?;
        s.stemgedrag = o.als_stemgedrag();
        s.uitkomst = s.stemgedrag.uitkomst(kamer);
        s.controle.beoordeling = Some(Beoordeling { keuze: Keuze::OfficieelOvergenomen, door: door.into(), over: o });
        Ok(())
    }

    /// "Onze invoer klopt": de invoer blijft, het verschil telt als afgehandeld.
    pub fn onze_invoer_klopt(&mut self, id: &str, door: &str) -> Result<(), StemmingWeigering> {
        let s = self.stemming_mut(id)?;
        let o = s.controle.officieel.clone().ok_or(StemmingWeigering::GeenOfficieleUitslag)?;
        s.controle.beoordeling = Some(Beoordeling { keuze: Keuze::OnzeInvoerKlopt, door: door.into(), over: o });
        Ok(())
    }

    /// Noteert dat een Graphic van deze Stemming in de Ether kwam; de eerste keer telt.
    pub fn in_ether(&mut self, id: &str, sinds_ms: u64) {
        if let Some(s) = self.lijst.iter_mut().find(|s| s.punt.id == id) {
            s.in_ether_ms.get_or_insert(sinds_ms);
        }
    }

    /// Rekent de controle van elke Stemming opnieuw uit.
    pub fn reken_controle(&mut self, kamer: &Kamer) {
        self.lijst.iter_mut().for_each(|s| s.reken_controle(kamer));
    }

    /// Rekent de uitkomst opnieuw uit na een nieuwe Kamer. Een vastgelegde of vergrendelde
    /// Stemming houdt de uitkomst van toen.
    pub fn reken_opnieuw(&mut self, kamer: &Kamer) {
        for s in self.lijst.iter_mut().filter(|s| !s.vastgelegd && !s.vergrendeld) {
            s.uitkomst = s.stemgedrag.uitkomst(kamer);
        }
    }
}

impl Stemming {
    fn nieuw(punt: Lijstpunt, kamer: &Kamer) -> Stemming {
        let stemgedrag = Stemgedrag::default();
        Stemming {
            punt,
            met_de_hand: false,
            voor_uitzending: false,
            uitkomst: stemgedrag.uitkomst(kamer),
            stemgedrag,
            vastgelegd: false,
            vergrendeld: false,
            in_ether_ms: None,
            controle: Stemmingscontrole::default(),
        }
    }
}

/// De gegevens van een Graphic over een Stemming: een kopie op het moment van vastleggen.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Stemmingsuitslag {
    /// Het id van de Stemming in de lijst.
    pub stemming: String,
    pub titel: String,
    pub nummer: Option<String>,
    pub indieners: Option<String>,
    pub voor: u16,
    pub tegen: u16,
    pub besluit: Besluit,
    pub algemene_stemmen: bool,
    /// Elke Fractie met haar stem, in de volgorde van de Kamer (grootste eerst).
    pub fracties: Vec<FractieInUitslag>,
    pub afwijkend: Vec<AfwijkendInUitslag>,
    /// Waar de Stoelen gekleurd zijn.
    #[serde(default = "zaalindeling")]
    pub indeling: Indeling,
    /// Kamerleden staan op een Stoel die het systeem koos: een hoofdelijke stemming in de
    /// Uitslagindeling, met de Kamerleden binnen het blok van hun Fractie op volgorde van de
    /// kieslijst. Het dashboard markeert dat als "voorlopig".
    #[serde(default)]
    pub voorlopig: bool,
    /// Per Stoel, in de volgorde van [`crate::Stoel::alle`], de stem die er te zien is. Leeg:
    /// niemand (grijs). Een lege lijst (een uitslag van vóór #56): de Stoelen houden hun kleur.
    #[serde(default)]
    pub stoelen: Vec<Option<Stem>>,
    /// Elk Kamerlid met zijn eigen stem, per Fractie in de volgorde van `fracties` en daarin op
    /// volgorde van de kieslijst: de Namenlijst van de Hoofdelijke stemming.
    #[serde(default)]
    pub kamerleden: Vec<KamerlidInUitslag>,
}

/// Eén naam in de Namenlijst.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct KamerlidInUitslag {
    pub kamerlid: String,
    /// De achternaam met tussenvoegsel, zoals "Van der Werf".
    pub naam: String,
    /// De afkorting van de Fractie.
    pub fractie: String,
    pub stem: Stem,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct FractieInUitslag {
    pub fractie: String,
    pub afkorting: String,
    /// De kleur van de Partij, als `#rrggbb`.
    pub kleur: Option<String>,
    pub zetels: u16,
    pub stem: Stem,
    /// Het logo van de Partij: een bestandsnaam bij de server.
    #[serde(default)]
    #[ts(optional)]
    pub logo: Option<String>,
    /// De korte naam van de Partij, zoals "CU" voor de Fractie "ChristenUnie".
    #[serde(default)]
    #[ts(optional)]
    pub kort: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct AfwijkendInUitslag {
    pub kamerlid: String,
    pub naam: String,
    /// De afkorting van de Fractie.
    pub fractie: String,
    pub stem: Stem,
}

impl Stemmingsuitslag {
    pub(crate) fn van(
        s: &Stemming,
        kamer: &Kamer,
        cijfers: &Cijfers,
        indeling: Indeling,
        bron: &Stoelbron,
    ) -> Result<Stemmingsuitslag, StemmingWeigering> {
        let u = s.stemgedrag.uitkomst(kamer);
        let besluit = match u.besluit {
            Some(b) => b,
            None if u.open > 0 => return Err(StemmingWeigering::FractiesOpen(u.open)),
            None => return Err(StemmingWeigering::FractiesOpen(0)),
        };
        let fracties = kamer
            .fracties
            .iter()
            .filter_map(|f| {
                let partij = kamer.partij_van.get(&f.id).and_then(|p| cijfers.partij(p));
                Some(FractieInUitslag {
                    fractie: f.id.clone(),
                    afkorting: f.afkorting.clone(),
                    kleur: partij.map(|p| p.kleur.clone()),
                    zetels: f.zetels,
                    stem: s.stemgedrag.stem_van(&f.id)?,
                    logo: partij.and_then(|p| p.logo.clone()),
                    kort: partij.map(|p| p.kort.clone()).filter(|k| !k.trim().is_empty()),
                })
            })
            .collect();
        let afwijkend = s
            .stemgedrag
            .afwijkend
            .iter()
            .filter_map(|a| {
                let k = kamer.kamerlid(&a.kamerlid)?;
                Some(AfwijkendInUitslag {
                    kamerlid: k.id.clone(),
                    naam: k.naam(),
                    fractie: kamer.fractie(&k.fractie).map_or_else(|| k.fractie.clone(), |f| f.afkorting.clone()),
                    stem: a.stem,
                })
            })
            .collect();
        let sg = &s.stemgedrag;
        let voorlopig = indeling == Indeling::Uitslagindeling && sg.hoofdelijk;
        let stoelen = match indeling {
            Indeling::Zaalindeling => bron
                .zaal
                .kamerleden(kamer)
                .into_iter()
                .map(|k| k.and_then(|k| sg.stem_van_kamerlid(k)))
                .collect(),
            Indeling::Uitslagindeling => sg.stoelen_in_uitslagindeling(kamer, bron.volgorde),
        };
        let kamerleden = kamer
            .fracties
            .iter()
            .flat_map(|f| op_kieslijst(kamer, &f.id).into_iter().map(move |k| (f, k)))
            .filter_map(|(f, k)| {
                Some(KamerlidInUitslag {
                    kamerlid: k.id.clone(),
                    naam: k.achternaam_met_tussenvoegsel(),
                    fractie: f.afkorting.clone(),
                    stem: sg.stem_van_kamerlid(k)?,
                })
            })
            .collect();
        Ok(Stemmingsuitslag {
            stemming: s.punt.id.clone(),
            titel: s.punt.titel.clone(),
            nummer: s.punt.nummer.clone(),
            indieners: s.punt.indieners.clone(),
            voor: u.voor,
            tegen: u.tegen,
            besluit,
            algemene_stemmen: s.stemgedrag.algemene_stemmen,
            fracties,
            afwijkend,
            indeling,
            voorlopig,
            stoelen,
            kamerleden,
        })
    }

    /// De uitkomst in één regel, zoals "Aangenomen: 94 voor, 56 tegen" of "Met algemene stemmen
    /// aangenomen".
    pub fn samenvatting(&self) -> String {
        let woord = match self.besluit {
            Besluit::Aangenomen => "aangenomen",
            Besluit::Verworpen => "verworpen",
        };
        if self.algemene_stemmen {
            format!("Met algemene stemmen {woord}")
        } else {
            let mut w = woord.to_string();
            w[..1].make_ascii_uppercase();
            format!("{w}: {} voor, {} tegen", self.voor, self.tegen)
        }
    }

    /// De titel als kop van het Sjabloon: zonder de indieners ("Motie van het lid Een over X"
    /// wordt "Motie over X") en hooguit [`crate::MAX_KOP`] tekens.
    pub fn kop(&self) -> String {
        let t = self.titel.trim();
        let korter = [" van het lid ", " van de leden "].iter().find_map(|van| {
            let (soort, rest) = t.split_once(van)?;
            let (_, onderwerp) = rest.split_once(" over ")?;
            (!soort.contains(' ')).then(|| format!("{soort} over {onderwerp}"))
        });
        inkorten(korter.as_deref().unwrap_or(t), crate::MAX_KOP)
    }

    /// De kleur per Stoel, in de volgorde van [`crate::Stoel::alle`]: de stemkleur, of grijs
    /// zonder stem. `None` voor een uitslag zonder Stoelen (van vóór #56).
    pub fn stoelkleuren(&self) -> Option<Vec<crate::Kleur>> {
        use crate::sjabloon::{GRIJS, STEM_TEGEN, STEM_VOOR};
        (self.stoelen.len() == AANTAL_STOELEN).then(|| {
            self.stoelen
                .iter()
                .map(|s| match s {
                    Some(Stem::Voor) => STEM_VOOR,
                    Some(Stem::Tegen) => STEM_TEGEN,
                    None => GRIJS,
                })
                .collect()
        })
    }
}

/// De Kamerleden van een Fractie op volgorde van de kieslijst; wie geen plek op de lijst heeft
/// achteraan, op achternaam.
fn op_kieslijst<'a>(kamer: &'a Kamer, fractie: &str) -> Vec<&'a Kamerlid> {
    let mut leden: Vec<&Kamerlid> = kamer.kamerleden.iter().filter(|k| k.fractie == fractie).collect();
    leden.sort_by_key(|k| (k.kieslijst.is_none(), k.kieslijst, k.achternaam.to_lowercase()));
    leden
}

/// Kort een tekst in tot `max` tekens, met "…" aan het eind als er iets af moest.
fn inkorten(t: &str, max: usize) -> String {
    if t.chars().count() <= max {
        return t.into();
    }
    let mut s: String = t.chars().take(max - 1).collect();
    s.truncate(s.trim_end().len());
    s.push('…');
    s
}
