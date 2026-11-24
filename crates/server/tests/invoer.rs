//! Het invoerbestand: de server let erop en leest het bij elke wijziging in, behalve in een
//! vergrendelde set. Het slot blijft na een herstart. Alle cijfers zijn voorbeeldgegevens.

use std::path::Path;
use std::time::Duration;

use gedeeld::{BerichtVanServer, GenummerdeStand, Opdracht};
use server::{Instellingen, Server};

const SLEUTEL: &str = "proefsleutel";

async fn start(map: &Path, invoer: &Path) -> Server {
    Server::start(Instellingen {
        adres: "127.0.0.1:0".parse().unwrap(),
        map: map.to_owned(),
        sleutel: SLEUTEL.into(),
        dashboard: None,
        invoer: Some(invoer.to_owned()),
        zaalindeling: None,
        open_data: None,
    })
    .await
    .expect("server start")
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

/// Wacht tot de Stand op de server aan `past` voldoet.
async fn wacht_op(server: &Server, past: impl Fn(&GenummerdeStand) -> bool) -> GenummerdeStand {
    let tot = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let s = stand(server).await;
        if past(&s) {
            return s;
        }
        assert!(
            tokio::time::Instant::now() < tot,
            "Stand niet binnen 10 s: {s:?}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn opdracht(server: &Server, o: &Opdracht) -> BerichtVanServer {
    reqwest::Client::new()
        .post(format!("http://{}/api/opdracht", server.adres()))
        .bearer_auth(SLEUTEL)
        .json(o)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn bestand(alfa: u16, beta: u16) -> String {
    format!(
        r#"{{ "partijen": [ {{ "sleutel": "alfa", "naam": "Alfa" }}, {{ "sleutel": "beta", "naam": "Beta" }} ],
            "sets": [ {{ "id": "exit", "soort": "exitpoll", "label": "Exitpoll", "zetels": {{ "alfa": {alfa}, "beta": {beta} }} }} ] }}"#
    )
}

fn alfa(s: &GenummerdeStand) -> Option<u32> {
    s.stand.cijfers.set("exit").map(|e| e.zetels_van("alfa"))
}

#[tokio::test]
async fn een_gewijzigd_invoerbestand_komt_binnen_en_een_vergrendelde_set_blijft_gelijk_ook_na_een_herstart()
 {
    let map = tempfile::tempdir().unwrap();
    let invoer = map.path().join("cijfers.json");
    std::fs::write(&invoer, bestand(80, 70)).unwrap();
    let server = start(&map.path().join("server"), &invoer).await;

    wacht_op(&server, |s| alfa(s) == Some(80)).await;
    std::fs::write(&invoer, bestand(81, 69)).unwrap();
    let s = wacht_op(&server, |s| alfa(s) == Some(81)).await;
    let m = s.stand.cijfers.laatste_invoer.as_ref().unwrap();
    assert_eq!(m.bestand, invoer.display().to_string());
    assert_eq!(m.sets_binnen, ["exit"]);

    let o = Opdracht::VergrendelSet { set: "exit".into() };
    assert!(matches!(
        opdracht(&server, &o).await,
        BerichtVanServer::Uitgevoerd { .. }
    ));
    std::fs::write(&invoer, bestand(90, 60)).unwrap();
    let s = wacht_op(&server, |s| {
        s.stand
            .cijfers
            .laatste_invoer
            .as_ref()
            .is_some_and(|m| !m.overgeslagen.is_empty())
    })
    .await;
    assert_eq!(alfa(&s), Some(81));
    assert_eq!(
        s.stand.cijfers.laatste_invoer.unwrap().overgeslagen[0].reden,
        "de set is vergrendeld"
    );

    // Na een herstart staat het slot er nog, en het gewijzigde bestand komt er nog steeds niet in.
    let voor_herstart = s.volgnummer;
    server.stop().await;
    let server = start(&map.path().join("server"), &invoer).await;
    let s = wacht_op(&server, |s| s.volgnummer > voor_herstart).await;
    assert_eq!(
        s.stand
            .cijfers
            .laatste_invoer
            .as_ref()
            .unwrap()
            .overgeslagen[0]
            .reden,
        "de set is vergrendeld"
    );
    assert!(s.stand.cijfers.set("exit").unwrap().vergrendeld);
    assert_eq!(alfa(&s), Some(81));

    let o = Opdracht::OntgrendelSet { set: "exit".into() };
    assert!(matches!(
        opdracht(&server, &o).await,
        BerichtVanServer::Uitgevoerd { .. }
    ));
    std::fs::write(&invoer, bestand(91, 59)).unwrap();
    wacht_op(&server, |s| alfa(s) == Some(91)).await;
    server.stop().await;
}

#[tokio::test]
async fn een_tweede_koppeling_van_een_voorganger_weigert_de_server_met_een_reden() {
    let map = tempfile::tempdir().unwrap();
    let invoer = map.path().join("cijfers.json");
    std::fs::write(
        &invoer,
        r#"{ "partijen": [ { "sleutel": "oud", "naam": "Oud" }, { "sleutel": "nieuw", "naam": "Nieuw", "voorgangers": ["oud"] },
                           { "sleutel": "ander", "naam": "Ander" } ] }"#,
    )
    .unwrap();
    let server = start(&map.path().join("server"), &invoer).await;
    wacht_op(&server, |s| s.stand.cijfers.partijen.len() == 3).await;
    let o = Opdracht::KoppelVoorganger {
        partij: "ander".into(),
        voorganger: "oud".into(),
    };
    let BerichtVanServer::Geweigerd { reden } = opdracht(&server, &o).await else {
        panic!("tweede koppeling niet geweigerd");
    };
    assert_eq!(
        reden,
        "\"oud\" is al Voorganger van Nieuw: een Voorganger hoort bij hooguit één Partij"
    );
    server.stop().await;
}
