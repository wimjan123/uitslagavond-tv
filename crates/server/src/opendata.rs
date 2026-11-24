//! De OData API van de Tweede Kamer: de vraag naar Kamerleden en het lezen van het antwoord.
//!
//! Documentatie: <https://opendata.tweedekamer.nl/documentatie/odata-api>. De data is CC0.
//! De server vraagt alle personen met een `FractieZetelPersoon` die op of na de peildatum nog
//! geldt, en kiest daaruit zelf wie op de peildatum in de Kamer zit. Zo telt een nieuw Kamerlid
//! mee vanaf de dag van de beëdiging, ook als de Kamer het al eerder invoerde. Van elk Kamerlid
//! komt de hele geschiedenis van `FractieZetelPersoon` mee, voor de eerste dag in de Kamer.

use std::collections::BTreeMap;

use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use jiff::civil::Date;
use serde::Deserialize;

/// Het adres van de API.
pub const BASIS: &str = "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0";

/// Het adres van de vraag naar Kamerleden op `peildatum`, met de parameters erbij.
pub fn vraag(basis: &str, peildatum: Date) -> (String, Vec<(&'static str, String)>) {
    let filter = format!(
        "Verwijderd eq false and FractieZetelPersoon/any(a:a/Verwijderd eq false and \
         (a/TotEnMet eq null or a/TotEnMet ge {peildatum}))"
    );
    let expand = format!(
        "FractieZetelPersoon($filter=Verwijderd eq false;$select=FractieZetel_Id,Functie,Van,TotEnMet;\
         $expand=FractieZetel($select=Id,Gewicht;$expand=Fractie($select=Id,Afkorting,NaamNL,AantalZetels)))"
    );
    let select = "Id,Initialen,Roepnaam,Voornamen,Tussenvoegsel,Achternaam,Geboortedatum,\
                  Geboorteplaats,Woonplaats,ContentType,ContentLength";
    (
        format!("{}/Persoon", basis.trim_end_matches('/')),
        vec![
            ("$filter", filter),
            ("$select", select.into()),
            ("$expand", expand),
        ],
    )
}

/// Het adres van de portretfoto van een persoon.
pub fn foto(basis: &str, persoon: &str) -> String {
    format!(
        "{}/Persoon({persoon})/resource",
        basis.trim_end_matches('/')
    )
}

/// Eén pagina van het antwoord. De API geeft hoogstens 250 personen per pagina.
#[derive(Deserialize, Debug)]
pub struct Pagina {
    #[serde(rename = "value")]
    pub personen: Vec<Persoon>,
    /// Het adres van de volgende pagina, als die er is.
    #[serde(rename = "@odata.nextLink")]
    pub volgende: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Persoon {
    pub id: String,
    pub initialen: Option<String>,
    pub roepnaam: Option<String>,
    pub voornamen: Option<String>,
    pub tussenvoegsel: Option<String>,
    pub achternaam: Option<String>,
    pub geboortedatum: Option<String>,
    pub geboorteplaats: Option<String>,
    pub woonplaats: Option<String>,
    /// Leeg als er geen portretfoto is.
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    #[serde(default)]
    pub fractie_zetel_persoon: Vec<ZetelPersoon>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ZetelPersoon {
    #[serde(rename = "FractieZetel_Id")]
    pub fractie_zetel_id: String,
    pub functie: Option<String>,
    pub van: Option<String>,
    pub tot_en_met: Option<String>,
    pub fractie_zetel: Option<FractieZetel>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FractieZetel {
    pub fractie: Option<OdataFractie>,
    /// De plaats van de FractieZetel in de lijst van de Fractie: na een verkiezing de
    /// volgorde van de kieslijst.
    #[serde(default)]
    pub gewicht: Option<u16>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct OdataFractie {
    pub id: String,
    pub afkorting: Option<String>,
    #[serde(rename = "NaamNL")]
    pub naam_nl: Option<String>,
    pub aantal_zetels: Option<u16>,
}

pub fn lees_pagina(json: &str) -> anyhow::Result<Pagina> {
    Ok(serde_json::from_str(json)?)
}

/// De Kamer op de peildatum, uit de personen van alle pagina's.
#[derive(Debug, Clone, Default)]
pub struct OpgehaaldeKamer {
    pub kamerleden: Vec<Kamerlid>,
    pub fracties: Vec<Fractie>,
    /// De grootte van de foto in de open data, per Kamerlid met een foto.
    pub fotogrootte: BTreeMap<String, u64>,
}

/// Wie op `peildatum` in de Kamer zit: per persoon de `FractieZetelPersoon` met `Van` op of vóór
/// de peildatum en `TotEnMet` leeg of op of na de peildatum. De eerste dag in de Kamer is de
/// vroegste `Van` van die persoon. `foto` is [`Foto::OpenData`] als de
/// open data een foto heeft; de server past dat aan naar wat er op schijf staat.
pub fn kamer_op(personen: &[Persoon], peildatum: Date) -> OpgehaaldeKamer {
    let mut k = OpgehaaldeKamer::default();
    for p in personen {
        let Some((z, f)) = p.fractie_zetel_persoon.iter().find_map(|z| {
            let f = z.fractie_zetel.as_ref()?.fractie.as_ref()?;
            let van = dag(z.van.as_deref())?;
            let geldt =
                van <= peildatum && dag(z.tot_en_met.as_deref()).is_none_or(|t| peildatum <= t);
            geldt.then_some((z, f))
        }) else {
            continue;
        };
        if !k.fracties.iter().any(|g| g.id == f.id) {
            let afkorting = tekst(&f.afkorting)
                .or_else(|| tekst(&f.naam_nl))
                .unwrap_or_default();
            k.fracties.push(Fractie {
                id: f.id.clone(),
                naam: tekst(&f.naam_nl).unwrap_or_else(|| afkorting.clone()),
                afkorting,
                zetels: f.aantal_zetels.unwrap_or(0),
            });
        }
        let heeft_foto = p
            .content_type
            .as_deref()
            .is_some_and(|t| t.starts_with("image/"));
        if heeft_foto {
            k.fotogrootte
                .insert(p.id.clone(), p.content_length.unwrap_or(0));
        }
        let roepnaam = tekst(&p.roepnaam)
            .or_else(|| {
                tekst(&p.voornamen).map(|v| v.split(' ').next().unwrap_or_default().to_owned())
            })
            .unwrap_or_default();
        k.kamerleden.push(Kamerlid {
            id: p.id.clone(),
            roepnaam,
            initialen: tekst(&p.initialen).unwrap_or_default(),
            tussenvoegsel: tekst(&p.tussenvoegsel),
            achternaam: tekst(&p.achternaam).unwrap_or_default(),
            fractie: f.id.clone(),
            zetel: z.fractie_zetel_id.clone(),
            fractievoorzitter: z.functie.as_deref() == Some("Fractievoorzitter"),
            van: dag(z.van.as_deref())
                .map(|d| d.to_string())
                .unwrap_or_default(),
            geboortedatum: tekst(&p.geboortedatum),
            geboorteplaats: tekst(&p.geboorteplaats),
            woonplaats: tekst(&p.woonplaats),
            kieslijst: z.fractie_zetel.as_ref().and_then(|fz| fz.gewicht),
            foto: if heeft_foto {
                Foto::OpenData
            } else {
                Foto::Geen
            },
            in_de_kamer_sinds: p
                .fractie_zetel_persoon
                .iter()
                .filter_map(|z| dag(z.van.as_deref()))
                .min()
                .map(|d| d.to_string()),
        });
    }
    k
}

/// De datum uit "2026-11-16T00:00:00+01:00". De API zet daar de Nederlandse dag in.
fn dag(t: Option<&str>) -> Option<Date> {
    t?.get(..10)?.parse().ok()
}

fn tekst(t: &Option<String>) -> Option<String> {
    t.as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
}
