//! De Stemmingslijst uit de open data van de Tweede Kamer, met vastgelegde antwoorden van de API
//! (`tests/opendata/stemmingslijst-*.json`, zie de README daar). Geen netwerk: een nep-API op
//! 127.0.0.1 geeft die antwoorden terug.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::{RawQuery, State};
use axum::http::header;
use axum::routing::get;
use gedeeld::GenummerdeStand;
use jiff::civil::{Date, date};
use server::stemmingslijst::{indieners, lees_pagina, punten_op, vraag};
use server::{Instellingen, OpenData, Server};

const OP_24_NOV: &str = include_str!("opendata/stemmingslijst-2026-11-24.json");
const OP_12_NOV: &str = include_str!("opendata/stemmingslijst-2026-11-12.json");
const OP_22_NOV: &str = include_str!("opendata/stemmingslijst-2026-11-22.json");
const SLEUTEL: &str = "proefsleutel";

fn punten(antwoord: &str, dag: Date) -> Vec<gedeeld::stemmingen::Lijstpunt> {
    let pagina = lees_pagina(antwoord).expect("vastgelegd antwoord is leesbaar");
    assert!(pagina.volgende.is_none());
    punten_op(&pagina.activiteiten, dag)
}

#[test]
fn de_lijst_van_dinsdag_24_november_telt_129_moties_op_volgorde_van_de_agenda() {
    let p = punten(OP_24_NOV, date(2026, 11, 24));
    assert_eq!(p.len(), 129);
    assert!(p.iter().all(|p| p.soort.as_deref() == Some("Motie")));

    let eerste = &p[0];
    assert_eq!(eerste.id, "69f8c004-910e-4015-8f1d-a791b170340c");
    assert_eq!(eerste.nummer.as_deref(), Some("37020-XV, nr. 8"));
    assert_eq!(
        eerste.titel,
        "Motie van het lid Bamenga over het opbouwen van een netwerk voor nieuwkomers stimuleren"
    );
    assert_eq!(eerste.indieners.as_deref(), Some("Bamenga"));
    assert_eq!(
        eerste.agendapunt.as_deref(),
        Some("Moties ingediend bij het wetgevingsoverleg over het begrotingsonderdeel Inburgering en Integratie")
    );

    // Een nader gewijzigde motie heeft geen Kamerstukdossier en dus geen nummer.
    let ceder = p.iter().find(|p| p.id == "54562db0-fa01-4caf-ab01-53de16c2a903").unwrap();
    assert_eq!(ceder.nummer, None);
    assert_eq!(ceder.indieners.as_deref(), Some("Ceder"));

    let leden = p.iter().find(|p| p.id == "d49223a3-ea8c-4c8d-a4e0-4d106c147796").unwrap();
    assert_eq!(leden.indieners.as_deref(), Some("Keijzer en Nanninga"));
}

#[test]
fn een_dag_met_wetsvoorstellen_zet_de_amendementen_voor_het_wetsvoorstel() {
    let p = punten(OP_12_NOV, date(2026, 11, 12));
    // Twee Activiteiten die dag: 30+23+3+3+1+3+7+7 en 4 wetsvoorstellen.
    assert_eq!(p.len(), 81);
    let soort = |i: usize| p[i].soort.as_deref().unwrap();
    assert_eq!(soort(0), "Amendement");
    assert_eq!(p[0].nummer.as_deref(), Some("37022, nr. 14"));
    assert_eq!(p[0].indieners.as_deref(), Some("Grinwis"));
    // De tweede Activiteit (21.05 uur) met alleen de wetsvoorstellen komt als laatste.
    let laatste: Vec<(&str, Option<&str>)> =
        p[77..].iter().map(|p| (p.soort.as_deref().unwrap(), p.nummer.as_deref())).collect();
    assert_eq!(
        laatste,
        [("Wetgeving", Some("37022")), ("Wetgeving", Some("37023")), ("Wetgeving", Some("37024")), ("Wetgeving", Some("37053"))]
    );
    // Een amendement "ter vervanging van": de indieners stoppen daar.
    let vervanging = p
        .iter()
        .find(|p| p.titel.starts_with("Nader gewijzigd amendement van de leden Grinwis en Van Brenk ter vervanging van nr. 27"))
        .unwrap();
    assert_eq!(vervanging.indieners.as_deref(), Some("Grinwis en Van Brenk"));
}

#[test]
fn alleen_stemmingen_van_de_gevraagde_dag_tellen() {
    // De vraag heeft een ruim venster; een Activiteit van een andere dag valt eruit.
    assert!(punten(OP_24_NOV, date(2026, 11, 25)).is_empty());
    assert!(punten(OP_22_NOV, date(2026, 11, 22)).is_empty());
}

#[test]
fn indieners_uit_de_titel() {
    assert_eq!(indieners("Motie van het lid Nanninga c.s. over iets").as_deref(), Some("Nanninga c.s."));
    assert_eq!(indieners("Wijziging van de Wet minimumbelasting 2024"), None);
}

#[test]
fn de_vraag_zoekt_de_stemmingen_rond_de_dag() {
    let (adres, p) = vraag("https://voorbeeld/OData/v4/2.0/", date(2026, 11, 24));
    assert_eq!(adres, "https://voorbeeld/OData/v4/2.0/Activiteit");
    let filter = &p.iter().find(|(k, _)| *k == "$filter").unwrap().1;
    assert!(filter.contains("Soort eq 'Stemmingen'"));
    assert!(filter.contains("Datum ge 2026-11-23T12:00:00Z and Datum lt 2026-11-25T12:00:00Z"), "{filter}");
}

/// Een nep-API: `/Activiteit` geeft het antwoord dat nu in `antwoord` staat; `/Persoon` niets.
async fn nep_api(antwoord: Arc<Mutex<&'static str>>) -> String {
    let app = Router::new()
        .route(
            "/Activiteit",
            get(|State(a): State<Arc<Mutex<&'static str>>>, RawQuery(q): RawQuery| async move {
                assert!(q.unwrap_or_default().contains("Stemmingen"));
                ([(header::CONTENT_TYPE, "application/json")], *a.lock().unwrap())
            }),
        )
        .route("/Persoon", get(|| async { ([(header::CONTENT_TYPE, "application/json")], r#"{"value":[]}"#) }))
        .with_state(antwoord);
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adres = l.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    format!("http://{adres}")
}

async fn stand(server: &Server) -> GenummerdeStand {
    reqwest::Client::new()
        .get(format!("http://{}/api/stand", server.adres()))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn wacht_op(server: &Server, past: impl Fn(&GenummerdeStand) -> bool) -> GenummerdeStand {
    let tot = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let s = stand(server).await;
        if past(&s) {
            return s;
        }
        assert!(tokio::time::Instant::now() < tot, "Stand niet binnen 10 s");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn de_server_leest_de_stemmingslijst_in_en_houdt_hem_bij() {
    let map = tempfile::tempdir().unwrap();
    let antwoord = Arc::new(Mutex::new(OP_24_NOV));
    let basis = nep_api(antwoord.clone()).await;
    let server = Server::start(Instellingen {
        adres: "127.0.0.1:0".parse().unwrap(),
        map: map.path().to_owned(),
        sleutel: SLEUTEL.into(),
        dashboard: None,
        invoer: None,
        zaalindeling: None,
        open_data: Some(OpenData {
            basis,
            elke: Duration::from_millis(100),
            peildatum: Some(date(2026, 11, 24)),
            debat_direct: None,
        }),
    })
    .await
    .expect("server start");

    let s = wacht_op(&server, |s| s.stand.stemmingen.lijst.len() == 129).await;
    assert_eq!(s.stand.stemmingen.dag.as_deref(), Some("2026-11-24"));
    assert!(s.stand.stemmingen.ingelezen.is_some());

    // Ongewijzigd: de server stuurt geen nieuwe Opdracht, dus het volgnummer blijft gelijk.
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(stand(&server).await.volgnummer, s.volgnummer);

    // Een lege lijst voor dezelfde dag haalt de moties zonder invoer weg.
    *antwoord.lock().unwrap() = OP_22_NOV;
    wacht_op(&server, |s| s.stand.stemmingen.lijst.is_empty()).await;
    server.stop().await;
}
