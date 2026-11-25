//! Controle na de uitzending (#57): de server haalt de officiële uitslag van elke Stemming op de
//! lijst zelf op en stuurt hem als [`Opdracht::LeesOfficieleUitslagen`].
//!
//! - [`debat_direct`]: elke minuut, tot elke Stemming een officiële uitslag heeft. De Kamer laat
//!   de uitslag daar ongeveer twee uur na de Stemming zien.
//! - [`open_data`]: de `Stemming`-regels, na de vaste tijd van de open data (10 minuten), tot
//!   elke Stemming ze heeft. Ze komen ongeveer 18 uur na de Stemming en gaan voor Debat Direct.
//!
//! De server kijkt alleen naar de dag van de Stemmingslijst, tot twee dagen erna. Wat hij vindt,
//! verandert de invoer van de Regie niet. De eerste keer dat Debat Direct een uitslag heeft,
//! schrijft hij in het logboek van de server hoeveel minuten na de hamerslag dat was: zo meet de
//! eerste echte Stemmingsdag de vertraging van Debat Direct. Dezelfde tijden staan per Stemming in
//! de Stand (`controle.debat_direct_gezien` en `officieel.hamerslag`).

pub mod debat_direct;
pub mod open_data;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use gedeeld::stemmingen::{Bron, OpgehaaldeUitslag, Stemmingen};
use gedeeld::{Afzender, Opdracht};
use jiff::civil::Date;

use crate::{Gedeeld, OpenData};

/// Hoe vaak de server bij Debat Direct kijkt. De API houdt antwoorden 60 seconden vast.
const DEBAT_DIRECT_ELKE: Duration = Duration::from_secs(60);
/// Hoe lang na de dag van de lijst de server nog kijkt.
const DAGEN_NA: i32 = 2;

/// Een tekst zonder hoofdletters, accenten en leestekens, met enkele spaties: "Christen-
/// Democratisch Appèl" wordt "christen democratisch appel", "37020-XV, nr. 8" wordt
/// "37020 xv nr 8".
pub(crate) fn gewoon(t: &str) -> String {
    let mut uit = String::with_capacity(t.len());
    for c in t.chars().flat_map(char::to_lowercase) {
        let c = match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ç' | 'ć' | 'č' => 'c',
            'ñ' | 'ń' => 'n',
            'š' | 'ş' | 'ś' => 's',
            'ž' | 'ź' | 'ż' => 'z',
            'ý' | 'ÿ' => 'y',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        };
        if c != ' ' || !uit.ends_with(' ') && !uit.is_empty() {
            uit.push(c);
        }
    }
    uit.trim_end().to_owned()
}

/// De tekst na het eerste " over " in een titel, zonder het nummer tussen haakjes aan het eind.
pub(crate) fn over_tekst(titel: &str) -> Option<String> {
    let (_, rest) = titel.split_once(" over ")?;
    let rest = match rest.trim_end().strip_suffix(')').and_then(|r| r.rfind(" (").map(|i| &r[..i])) {
        Some(zonder) => zonder,
        None => rest,
    };
    Some(gewoon(rest)).filter(|o| !o.is_empty())
}

/// Welke bronnen nog iets te brengen hebben voor de lijst van `dag`.
fn nodig(s: &Stemmingen, kamer_leeg: bool, vandaag: Date) -> Option<(Date, bool, bool)> {
    let dag: Date = s.dag.as_deref()?.parse().ok()?;
    if kamer_leeg {
        return None;
    }
    let dagen = vandaag.since(dag).ok()?.get_days();
    if !(0..=DAGEN_NA).contains(&dagen) || s.lijst.iter().all(|s| s.met_de_hand) {
        return None;
    }
    let van_lijst = || s.lijst.iter().filter(|s| !s.met_de_hand);
    let dd = van_lijst().any(|s| s.controle.officieel.is_none());
    let od = van_lijst().any(|s| s.controle.officieel.as_ref().is_none_or(|o| o.bron != Bron::OpenData));
    Some((dag, dd, od))
}

pub(crate) fn volg(g: Arc<Gedeeld>, inst: OpenData) {
    tokio::spawn(async move {
        let afzender = Afzender::OpenData { bron: inst.basis.clone() };
        let mut gestopt = g.gestopt.clone();
        let Ok(client) = reqwest::Client::builder()
            .user_agent(concat!("uitslagavond-tv/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .build()
        else {
            tracing::error!("controle: geen HTTP-client");
            return;
        };
        let tik = DEBAT_DIRECT_ELKE.min(inst.elke);
        let mut open_data_om = tokio::time::Instant::now();
        loop {
            let vandaag = inst.peildatum.unwrap_or_else(|| jiff::ZonedDateTime::now().date());
            let nu_nodig = {
                let s = g.uitzending.borrow();
                nodig(&s.stand.stemmingen, s.stand.kamer.fracties.is_empty(), vandaag)
            };
            if let Some((dag, dd, od)) = nu_nodig {
                let ronde = async {
                    if let (true, Some(basis)) = (dd, &inst.debat_direct) {
                        if let Err(e) = ronde_debat_direct(&g, &afzender, &client, basis, dag).await {
                            tracing::warn!("Debat Direct niet opgehaald: {e:#}");
                        }
                    }
                    if od && tokio::time::Instant::now() >= open_data_om {
                        open_data_om = tokio::time::Instant::now() + inst.elke;
                        if let Err(e) = ronde_open_data(&g, &afzender, &client, &inst.basis, dag).await {
                            tracing::warn!("Stemmingen uit de open data niet opgehaald: {e:#}");
                        }
                    }
                };
                tokio::select! {
                    _ = ronde => {}
                    _ = gestopt.changed() => return,
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(tik) => {}
                // Een nieuwe lijst of nieuwe invoer: meteen kijken of er iets te halen is.
                _ = wacht_op_lijst(&g, nu_nodig.is_none()) => {}
                _ = gestopt.changed() => return,
            }
        }
    });
}

/// Wacht tot de Stemmingslijst verandert, als er nu niets te halen is; anders nooit.
async fn wacht_op_lijst(g: &Gedeeld, niets: bool) {
    if !niets {
        return std::future::pending().await;
    }
    let mut standen = g.uitzending.subscribe();
    let (dag, lengte, kamer_leeg) = {
        let s = &g.uitzending.borrow().stand;
        (s.stemmingen.dag.clone(), s.stemmingen.lijst.len(), s.kamer.fracties.is_empty())
    };
    loop {
        if standen.changed().await.is_err() {
            return std::future::pending().await;
        }
        let s = &standen.borrow_and_update().stand;
        if s.stemmingen.dag != dag || s.stemmingen.lijst.len() != lengte || !s.kamer.fracties.is_empty() && kamer_leeg {
            return;
        }
    }
}

async fn haal(client: &reqwest::Client, adres: &str) -> anyhow::Result<String> {
    Ok(client
        .get(adres)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .with_context(|| format!("{adres} niet bereikbaar"))?
        .error_for_status()?
        .text()
        .await?)
}

async fn ronde_debat_direct(
    g: &Gedeeld,
    afzender: &Afzender,
    client: &reqwest::Client,
    basis: &str,
    dag: Date,
) -> anyhow::Result<()> {
    let agenda = debat_direct::lees_agenda(&haal(client, &debat_direct::agenda(basis, dag)).await?)
        .context("agenda van Debat Direct onleesbaar")?;
    let mut debatten = Vec::new();
    for id in agenda.stemmingen() {
        let json = haal(client, &debat_direct::debat(basis, dag, id)).await?;
        debatten.push(debat_direct::lees_debat(&json).context("debat van Debat Direct onleesbaar")?);
    }
    if debatten.iter().all(|d| d.votings.is_empty()) {
        return Ok(());
    }
    let actors = debat_direct::lees_actors(&haal(client, &debat_direct::actors(basis)).await?)
        .context("partijen van Debat Direct onleesbaar")?;
    let uitslagen = {
        let s = g.uitzending.borrow();
        debat_direct::uitslagen(&debatten, &actors, &s.stand.stemmingen.lijst, &s.stand.kamer)
    };
    stuur(g, afzender, uitslagen).await
}

async fn ronde_open_data(
    g: &Gedeeld,
    afzender: &Afzender,
    client: &reqwest::Client,
    basis: &str,
    dag: Date,
) -> anyhow::Result<()> {
    let (adres, parameters) = open_data::vraag(basis, dag);
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
        let pagina = open_data::lees_pagina(&tekst).context("antwoord van de open data onleesbaar")?;
        activiteiten.extend(pagina.activiteiten);
        match pagina.volgende {
            Some(volgende) => verzoek = client.get(volgende),
            None => {
                let uitslagen = {
                    let s = g.uitzending.borrow();
                    open_data::uitslagen(&activiteiten, &s.stand.stemmingen.lijst, &s.stand.kamer)
                };
                return stuur(g, afzender, uitslagen).await;
            }
        }
    }
    anyhow::bail!("de open data geeft meer dan 20 pagina's")
}

/// Stuurt de uitslagen als Opdracht, alleen als de Stand er echt door verandert, en schrijft de
/// vertraging van Debat Direct in het logboek.
async fn stuur(g: &Gedeeld, afzender: &Afzender, uitslagen: Vec<OpgehaaldeUitslag>) -> anyhow::Result<()> {
    let nu = jiff::ZonedDateTime::now();
    let opgehaald = nu.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string();
    let (voor, na) = {
        let s = g.uitzending.borrow();
        let voor = s.stand.stemmingen.clone();
        let mut na = voor.clone();
        na.lees_officieel(&opgehaald, &uitslagen);
        (voor, na)
    };
    if na == voor {
        return Ok(());
    }
    for (o, n) in voor.lijst.iter().zip(&na.lijst) {
        if o.controle.debat_direct_gezien.is_none() && n.controle.debat_direct_gezien.is_some() {
            let hamerslag = n.controle.officieel.as_ref().and_then(|o| o.hamerslag.as_deref());
            let minuten = hamerslag
                .and_then(|h| h.parse::<jiff::Timestamp>().ok())
                .map(|h| (nu.timestamp().as_second() - h.as_second()) / 60);
            let wat = n.punt.nummer.as_deref().unwrap_or(&n.punt.titel);
            match minuten {
                Some(m) => tracing::info!(
                    "Debat Direct heeft de uitslag van {wat}: {m} minuten na de hamerslag ({})",
                    hamerslag.unwrap_or_default()
                ),
                None => tracing::info!("Debat Direct heeft de uitslag van {wat}"),
            }
        }
    }
    let n = uitslagen.len();
    g.voer_uit(afzender, &Opdracht::LeesOfficieleUitslagen { opgehaald, uitslagen })
        .await
        .map_err(|e| anyhow::anyhow!("officiële uitslagen niet ingelezen: {e}"))?;
    tracing::info!("controle: {n} officiële uitslagen ingelezen");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gewoon_haalt_hoofdletters_accenten_en_leestekens_weg() {
        assert_eq!(gewoon("Christen-Democratisch Appèl"), "christen democratisch appel");
        assert_eq!(gewoon("37020-XV, nr. 8"), "37020 xv nr 8");
        assert_eq!(gewoon(" Kostić c.s. "), "kostic c s");
    }

    #[test]
    fn over_tekst_zonder_het_nummer_aan_het_eind() {
        assert_eq!(
            over_tekst("de gewijzigde motie-Goudzwaard over overlastmelding (23645, nr. ??, was nr. 912)").as_deref(),
            Some("overlastmelding")
        );
        assert_eq!(over_tekst("Gewijzigde motie van het lid Goudzwaard over overlastmelding").as_deref(), Some("overlastmelding"));
        assert_eq!(over_tekst("Wijziging van de Wet minimumbelasting 2024"), None);
    }
}
