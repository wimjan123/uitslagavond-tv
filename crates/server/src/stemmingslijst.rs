//! De Stemmingslijst van de dag uit de OData API van de Tweede Kamer.
//!
//! De Stemmingen van een dag staan als `Activiteit` met `Soort` "Stemmingen". Elk `Agendapunt`
//! daarvan heeft de moties, amendementen en wetsvoorstellen als `Zaak`. De lijst staat er al
//! dagen van tevoren (status "Gepland"), dus de server leest hem 's ochtends in en daarna steeds
//! na een vaste tijd, voor moties die erbij komen of van de lijst gaan. Wat de Regie al invoerde
//! blijft staan (zie [`gedeeld::stemmingen::Stemmingen::lees`]).

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use gedeeld::stemmingen::Lijstpunt;
use gedeeld::{Afzender, Opdracht};
use jiff::civil::Date;
use serde::Deserialize;

use crate::{Gedeeld, OpenData};

/// Na een mislukte poging eerder opnieuw proberen dan de vaste tijd.
const OPNIEUW: Duration = Duration::from_secs(60);

/// Het adres van de vraag naar de Stemmingen rond `dag`, met de parameters erbij. De API kent
/// alleen tijden met een zone; het venster is ruim en [`punten_op`] houdt alleen `dag` over.
pub fn vraag(basis: &str, dag: Date) -> (String, Vec<(&'static str, String)>) {
    let van = dag.yesterday().unwrap_or(dag);
    let tot = dag.tomorrow().unwrap_or(dag);
    let filter = format!(
        "Soort eq 'Stemmingen' and Verwijderd eq false and Datum ge {van}T12:00:00Z and Datum lt {tot}T12:00:00Z"
    );
    let expand = "Agendapunt($filter=Verwijderd eq false;$select=Id,Onderwerp,Volgorde;\
                  $expand=Zaak($filter=Verwijderd eq false;$select=Id,Nummer,Soort,Onderwerp,Titel,Volgnummer;\
                  $expand=Kamerstukdossier($select=Nummer,Toevoeging)))";
    (
        format!("{}/Activiteit", basis.trim_end_matches('/')),
        vec![
            ("$filter", filter),
            ("$select", "Id,Datum,Aanvangstijd,Onderwerp,Status".into()),
            ("$orderby", "Datum".into()),
            ("$expand", expand.into()),
        ],
    )
}

/// Eén pagina van het antwoord.
#[derive(Deserialize, Debug)]
pub struct Pagina {
    #[serde(rename = "value")]
    pub activiteiten: Vec<Activiteit>,
    #[serde(rename = "@odata.nextLink")]
    pub volgende: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Activiteit {
    pub id: String,
    /// Zoals "2026-11-24T15:10:00+01:00": de Nederlandse tijd.
    pub datum: Option<String>,
    #[serde(default)]
    pub agendapunt: Vec<Agendapunt>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Agendapunt {
    pub onderwerp: Option<String>,
    pub volgorde: Option<i64>,
    #[serde(default)]
    pub zaak: Vec<Zaak>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Zaak {
    pub id: String,
    pub soort: Option<String>,
    pub onderwerp: Option<String>,
    pub titel: Option<String>,
    pub volgnummer: Option<i64>,
    #[serde(default)]
    pub kamerstukdossier: Vec<Kamerstukdossier>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Kamerstukdossier {
    pub nummer: Option<i64>,
    pub toevoeging: Option<String>,
}

pub fn lees_pagina(json: &str) -> anyhow::Result<Pagina> {
    Ok(serde_json::from_str(json)?)
}

/// De punten van de Stemmingslijst van `dag`: de Activiteiten op volgorde van tijd, daarin de
/// agendapunten op hun volgorde, daarin de moties en amendementen op nummer en het wetsvoorstel
/// als laatste (de Kamer stemt eerst over de amendementen). Een Zaak die twee keer voorkomt,
/// telt één keer.
pub fn punten_op(activiteiten: &[Activiteit], dag: Date) -> Vec<Lijstpunt> {
    let dag = dag.to_string();
    let mut punten: Vec<Lijstpunt> = Vec::new();
    let mut van_dag: Vec<&Activiteit> = activiteiten
        .iter()
        .filter(|a| a.datum.as_deref().and_then(|d| d.get(..10)) == Some(dag.as_str()))
        .collect();
    van_dag.sort_by(|a, b| a.datum.cmp(&b.datum));
    for a in van_dag {
        let mut agendapunten: Vec<&Agendapunt> = a.agendapunt.iter().collect();
        agendapunten.sort_by_key(|p| p.volgorde.unwrap_or(i64::MAX));
        for p in agendapunten {
            let mut zaken: Vec<&Zaak> = p.zaak.iter().collect();
            zaken.sort_by_key(|z| {
                let dossier = z.kamerstukdossier.first();
                (
                    z.soort.as_deref() == Some("Wetgeving"),
                    dossier.and_then(|d| d.nummer).unwrap_or(i64::MAX),
                    dossier.and_then(|d| d.toevoeging.clone()),
                    z.volgnummer.unwrap_or(i64::MAX),
                )
            });
            for z in zaken {
                if punten.iter().any(|q| q.id == z.id) {
                    continue;
                }
                let Some(titel) = tekst(&z.onderwerp).or_else(|| tekst(&z.titel)) else {
                    continue;
                };
                punten.push(Lijstpunt {
                    id: z.id.clone(),
                    soort: tekst(&z.soort),
                    nummer: nummer(z),
                    indieners: indieners(&titel),
                    titel,
                    agendapunt: tekst(&p.onderwerp),
                });
            }
        }
    }
    punten
}

/// "37020-XX, nr. 17", of alleen "37022" voor het wetsvoorstel zelf.
fn nummer(z: &Zaak) -> Option<String> {
    let d = z.kamerstukdossier.first()?;
    let mut n = d.nummer?.to_string();
    if let Some(t) = d.toevoeging.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        n = format!("{n}-{t}");
    }
    match z.volgnummer {
        Some(v) if v > 0 => Some(format!("{n}, nr. {v}")),
        _ => Some(n),
    }
}

/// De indieners uit een titel als "Motie van de leden Heera Dijk en Tijmstra over ...":
/// "Heera Dijk en Tijmstra".
pub fn indieners(titel: &str) -> Option<String> {
    let (_, rest) = [" van het lid ", " van de leden "]
        .iter()
        .find_map(|v| titel.split_once(v))?;
    let eind = [" over ", " ter vervanging "]
        .iter()
        .filter_map(|e| rest.find(e))
        .min()
        .unwrap_or(rest.len());
    Some(rest[..eind].trim().to_owned()).filter(|n| !n.is_empty())
}

fn tekst(t: &Option<String>) -> Option<String> {
    t.as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
}

pub(crate) fn volg(g: Arc<Gedeeld>, inst: OpenData) {
    tokio::spawn(async move {
        let afzender = Afzender::OpenData { bron: inst.basis.clone() };
        let mut gestopt = g.gestopt.clone();
        let client = match reqwest::Client::builder()
            .user_agent(concat!("uitslagavond-tv/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("open data: geen HTTP-client: {e}");
                return;
            }
        };
        loop {
            let dag = inst.peildatum.unwrap_or_else(|| jiff::ZonedDateTime::now().date());
            let wacht = tokio::select! {
                r = ronde(&g, &afzender, &client, &inst.basis, dag) => match r {
                    Ok(()) => inst.elke,
                    Err(e) => {
                        tracing::warn!("stemmingslijst niet opgehaald: {e:#}");
                        OPNIEUW.min(inst.elke)
                    }
                },
                _ = gestopt.changed() => return,
            };
            tokio::select! {
                _ = tokio::time::sleep(wacht) => {}
                _ = gestopt.changed() => return,
            }
        }
    });
}

async fn ronde(
    g: &Gedeeld,
    afzender: &Afzender,
    client: &reqwest::Client,
    basis: &str,
    dag: Date,
) -> anyhow::Result<()> {
    let punten = haal_op(client, basis, dag).await?;
    let dag = dag.to_string();
    // Alleen een Opdracht als de lijst echt iets verandert.
    let ongewijzigd = {
        let s = g.uitzending.borrow();
        let nu = &s.stand.stemmingen;
        let mut proef = nu.clone();
        proef.lees(&dag, nu.ingelezen.as_deref().unwrap_or(""), &punten, &s.stand.kamer);
        proef == *nu
    };
    if ongewijzigd {
        tracing::debug!("open data: stemmingslijst ongewijzigd");
        return Ok(());
    }
    let n = punten.len();
    let opdracht = Opdracht::LeesStemmingslijst {
        dag,
        ingelezen: jiff::ZonedDateTime::now().strftime("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        punten,
    };
    g.voer_uit(afzender, &opdracht)
        .await
        .map_err(|e| anyhow::anyhow!("stemmingslijst niet ingelezen: {e}"))?;
    tracing::info!("open data: stemmingslijst ingelezen ({n} punten)");
    Ok(())
}

async fn haal_op(client: &reqwest::Client, basis: &str, dag: Date) -> anyhow::Result<Vec<Lijstpunt>> {
    let (adres, parameters) = vraag(basis, dag);
    let mut verzoek = client.get(adres).query(&parameters);
    let mut activiteiten = Vec::new();
    for _ in 0..20 {
        let tekst = verzoek
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .context("open data niet bereikbaar")?
            .error_for_status()?
            .text()
            .await?;
        let pagina = lees_pagina(&tekst).context("antwoord van de open data onleesbaar")?;
        activiteiten.extend(pagina.activiteiten);
        match pagina.volgende {
            Some(volgende) => verzoek = client.get(volgende),
            None => return Ok(punten_op(&activiteiten, dag)),
        }
    }
    anyhow::bail!("de open data geeft meer dan 20 pagina's")
}
