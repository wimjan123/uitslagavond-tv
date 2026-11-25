//! De `Stemming`-regels uit de OData API van de Tweede Kamer: de gecontroleerde uitslag per
//! Fractie (of per Kamerlid bij een hoofdelijke stemming), ongeveer 18 uur na de Stemming.
//!
//! De vraag gaat net als de Stemmingslijst via de `Activiteit` "Stemmingen" van de dag, nu met
//! per agendapunt de `Besluit`en, hun `Zaak` (het id van de Stemming in de lijst) en de
//! `Stemming`-regels. Een regel zonder `Persoon_Id` is de stem van een Fractie; met `Persoon_Id`
//! de stem van één Kamerlid.
//!
//! Let op: de `Fractie_Id` in een `Stemming`-regel is niet altijd die van de Fractie in de Kamer.
//! Op 19 november 2026 had 50PLUS daar een ander id (`a34bf6c8-...`) dan in `FractieZetel`. Een
//! onbekend id zoekt de server daarom op naam (`ActorFractie`).

use gedeeld::kamer::Kamer;
use gedeeld::stemmingen::{
    AfwijkendeStem, Besluit, Bron, GetikteFractie, OfficieleUitslag, OpgehaaldeUitslag, Stem, Stemming,
};
use jiff::civil::Date;
use serde::Deserialize;

/// Het adres van de vraag naar de Besluiten en Stemmingen rond `dag`, met de parameters erbij.
pub fn vraag(basis: &str, dag: Date) -> (String, Vec<(&'static str, String)>) {
    let (adres, mut p) = crate::stemmingslijst::vraag(basis, dag);
    let expand = "Agendapunt($filter=Verwijderd eq false;$select=Id;\
                  $expand=Besluit($filter=Verwijderd eq false;$select=Id,BesluitSoort,StemmingsSoort;\
                  $expand=Zaak($select=Id),Stemming($filter=Verwijderd eq false;\
                  $select=Soort,FractieGrootte,ActorNaam,ActorFractie,Fractie_Id,Persoon_Id)))";
    for (k, v) in &mut p {
        match *k {
            "$select" => *v = "Id,Datum".into(),
            "$expand" => *v = expand.into(),
            _ => {}
        }
    }
    (adres, p)
}

#[derive(Deserialize, Debug)]
pub struct Pagina {
    #[serde(rename = "value")]
    pub activiteiten: Vec<Activiteit>,
    #[serde(rename = "@odata.nextLink")]
    pub volgende: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Activiteit {
    #[serde(default)]
    pub agendapunt: Vec<Agendapunt>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Agendapunt {
    #[serde(default)]
    pub besluit: Vec<OdataBesluit>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct OdataBesluit {
    /// Zoals "Stemmen - aangenomen" of "Stemmen - aangehouden (al tijdens debat)".
    pub besluit_soort: Option<String>,
    /// "Met handopsteken" of "Hoofdelijk".
    pub stemmings_soort: Option<String>,
    #[serde(default)]
    pub zaak: Vec<ZaakId>,
    #[serde(default)]
    pub stemming: Vec<Regel>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ZaakId {
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct Regel {
    /// "Voor", "Tegen" of "Niet deelgenomen".
    #[serde(rename = "Soort")]
    pub soort: Option<String>,
    #[serde(rename = "Fractie_Id")]
    pub fractie: Option<String>,
    #[serde(rename = "Persoon_Id")]
    pub persoon: Option<String>,
    /// De naam van de Fractie, zoals "50PLUS" of "Keijzer".
    #[serde(rename = "ActorFractie")]
    pub actor_fractie: Option<String>,
}

pub fn lees_pagina(json: &str) -> anyhow::Result<Pagina> {
    Ok(serde_json::from_str(json)?)
}

/// De uitslagen van de Besluiten met Stemmingen die bij een Stemming op de lijst horen.
pub fn uitslagen(activiteiten: &[Activiteit], lijst: &[Stemming], kamer: &Kamer) -> Vec<OpgehaaldeUitslag> {
    let besluiten = activiteiten.iter().flat_map(|a| &a.agendapunt).flat_map(|p| &p.besluit);
    let mut uit: Vec<OpgehaaldeUitslag> = Vec::new();
    for b in besluiten {
        let soort = b.besluit_soort.as_deref().unwrap_or("").to_lowercase();
        let besluit = if soort.contains("aangenomen") {
            Besluit::Aangenomen
        } else if soort.contains("verworpen") {
            Besluit::Verworpen
        } else {
            continue;
        };
        if b.stemming.is_empty() {
            continue;
        }
        for z in &b.zaak {
            let Some(s) = lijst.iter().find(|s| s.punt.id.eq_ignore_ascii_case(&z.id)) else { continue };
            let (fracties, afwijkend) = stemmen(&b.stemming, kamer);
            let u = OpgehaaldeUitslag {
                stemming: s.punt.id.clone(),
                uitslag: OfficieleUitslag {
                    bron: Bron::OpenData,
                    besluit,
                    hoofdelijk: b.stemmings_soort.as_deref() == Some("Hoofdelijk"),
                    fracties,
                    afwijkend,
                    hamerslag: None,
                },
            };
            // Een later Besluit over dezelfde Zaak (een nieuwe Stemming) gaat voor.
            uit.retain(|o| o.stemming != u.stemming);
            uit.push(u);
        }
    }
    uit
}

fn stem(s: Option<&str>) -> Option<Stem> {
    match s? {
        "Voor" => Some(Stem::Voor),
        "Tegen" => Some(Stem::Tegen),
        _ => None,
    }
}

/// De stem per Fractie en wie anders stemde dan zijn Fractie. Stemden de Kamerleden van een
/// Fractie apart (hoofdelijk, of een verdeelde Fractie), dan geldt de stem van de meesten.
fn stemmen<'a>(regels: &'a [Regel], kamer: &'a Kamer) -> (Vec<GetikteFractie>, Vec<AfwijkendeStem>) {
    let mut fracties: Vec<GetikteFractie> = regels
        .iter()
        .filter(|r| r.persoon.is_none())
        .filter_map(|r| Some(GetikteFractie { fractie: fractie_van(r, kamer)?.to_owned(), stem: stem(r.soort.as_deref())? }))
        .collect();
    let leden: Vec<(&str, &str, Stem)> = regels
        .iter()
        .filter_map(|r| Some((r.persoon.as_deref()?, fractie_van(r, kamer)?, stem(r.soort.as_deref())?)))
        .collect();
    let mut afwijkend = Vec::new();
    let mut gedaan: Vec<&str> = Vec::new();
    for (_, f, _) in &leden {
        if gedaan.contains(f) {
            continue;
        }
        gedaan.push(f);
        let van_f: Vec<_> = leden.iter().filter(|(_, g, _)| g == f).collect();
        let meeste = match fracties.iter().find(|g| g.fractie == *f) {
            Some(g) => g.stem,
            None => {
                let voor = van_f.iter().filter(|(.., s)| *s == Stem::Voor).count();
                let m = if voor * 2 >= van_f.len() { Stem::Voor } else { Stem::Tegen };
                fracties.push(GetikteFractie { fractie: (*f).to_owned(), stem: m });
                m
            }
        };
        afwijkend.extend(
            van_f.iter().filter(|(.., s)| *s != meeste).map(|(p, _, s)| AfwijkendeStem { kamerlid: (*p).to_owned(), stem: *s }),
        );
    }
    (fracties, afwijkend)
}

/// Het id van de Fractie in de Kamer bij een regel: op id, anders op naam.
fn fractie_van<'a>(r: &'a Regel, kamer: &'a Kamer) -> Option<&'a str> {
    let id = r.fractie.as_deref();
    if let Some(f) = kamer.fracties.iter().find(|f| Some(f.id.as_str()) == id) {
        return Some(&f.id);
    }
    let naam = super::gewoon(r.actor_fractie.as_deref()?);
    let bij_naam = kamer
        .fracties
        .iter()
        .find(|f| super::gewoon(&f.afkorting) == naam || super::gewoon(&f.naam) == naam);
    match bij_naam {
        Some(f) => Some(&f.id),
        None => {
            tracing::warn!("open data: Fractie {naam} ({}) niet in de Kamer", id.unwrap_or("zonder id"));
            id
        }
    }
}
