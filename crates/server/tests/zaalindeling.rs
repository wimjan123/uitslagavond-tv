//! Het bestand met de Zaalindeling: de server leest het bij elke wijziging in, en schrijft het
//! terug als de Regie een Stoel geeft. Alle Kamerleden zijn voorbeeldgegevens.

use std::path::Path;
use std::time::Duration;

use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use gedeeld::zaalindeling::Zaalindeling;
use gedeeld::{BerichtVanServer, GenummerdeStand, Opdracht, Stoel};
use server::{Instellingen, Server};

const SLEUTEL: &str = "proefsleutel";

async fn start(map: &Path, zaalindeling: &Path) -> Server {
    Server::start(Instellingen {
        adres: "127.0.0.1:0".parse().unwrap(),
        map: map.to_owned(),
        sleutel: SLEUTEL.into(),
        dashboard: None,
        invoer: None,
        zaalindeling: Some(zaalindeling.to_owned()),
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

async fn wacht_op(server: &Server, past: impl Fn(&GenummerdeStand) -> bool) -> GenummerdeStand {
    let tot = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let s = stand(server).await;
        if past(&s) {
            return s;
        }
        assert!(tokio::time::Instant::now() < tot, "Stand niet binnen 10 s: {s:?}");
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

fn lid(id: &str, achternaam: &str) -> Kamerlid {
    Kamerlid {
        id: id.into(),
        roepnaam: "Proef".into(),
        initialen: "P.".into(),
        tussenvoegsel: None,
        achternaam: achternaam.into(),
        fractie: "rood".into(),
        zetel: format!("z-{id}"),
        fractievoorzitter: false,
        van: "2025-11-12".into(),
        geboortedatum: None,
        geboorteplaats: None,
        woonplaats: None,
        foto: Foto::Geen,
        in_de_kamer_sinds: None,
        kieslijst: None,
    }
}

fn bestand(wie: &str) -> String {
    format!(
        r#"{{ "kamerperiode": "2025", "bron": "proef", "stoelen": [
            {{ "vak": 1, "rij": 1, "nummer": 1, "kamerlid": "{wie}", "naam": "" }} ] }}"#
    )
}

fn op_1_1_1(s: &GenummerdeStand) -> Option<&str> {
    s.stand.zaalindeling.op(Stoel { vak: 1, rij: 1, nummer: 1 })
}

#[tokio::test]
async fn de_server_leest_de_zaalindeling_in_en_schrijft_een_gegeven_stoel_terug() {
    let map = tempfile::tempdir().unwrap();
    let pad = map.path().join("zaalindeling.json");
    std::fs::write(&pad, bestand("a")).unwrap();
    let server = start(map.path(), &pad).await;
    wacht_op(&server, |s| op_1_1_1(s) == Some("a")).await;

    // de Regie past het bestand aan: de Stand volgt
    std::fs::write(&pad, bestand("b")).unwrap();
    wacht_op(&server, |s| op_1_1_1(s) == Some("b")).await;

    // een Stoel geven komt in het bestand, zodat een volgende aanpassing hem niet kwijtraakt
    let kamer = Opdracht::LeesKamer {
        opgehaald: "nu".into(),
        peildatum: "2026-11-22".into(),
        kamerleden: vec![lid("a", "Aal"), lid("b", "Bes")],
        fracties: vec![Fractie { id: "rood".into(), afkorting: "ROOD".into(), naam: "Rood".into(), zetels: 2 }],
    };
    assert!(matches!(opdracht(&server, &kamer).await, BerichtVanServer::Uitgevoerd { .. }));
    let geef = Opdracht::GeefStoel { kamerlid: "a".into(), stoel: Stoel { vak: 2, rij: 1, nummer: 1 } };
    let antwoord = opdracht(&server, &geef).await;
    assert!(matches!(antwoord, BerichtVanServer::Uitgevoerd { .. }), "{antwoord:?}");
    let terug = Zaalindeling::uit_json(&std::fs::read_to_string(&pad).unwrap()).unwrap();
    assert_eq!(terug.stoel_van("a"), Some(Stoel { vak: 2, rij: 1, nummer: 1 }));
    assert_eq!(terug.stoel_van("b"), Some(Stoel { vak: 1, rij: 1, nummer: 1 }));
    server.stop().await;
}
