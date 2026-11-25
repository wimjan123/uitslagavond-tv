//! Eigen Standpunten via de server (#44): bewaren waar de camera staat, en wissen dat geweigerd
//! wordt zolang een Graphic het Standpunt gebruikt.

use std::path::Path;

use gedeeld::{BerichtVanServer, GenummerdeStand, Graphic, Opdracht};
use server::{Instellingen, Server};

const SLEUTEL: &str = "proefsleutel";

/// Start de server zoals op een pc zonder gamepad: hij leest er een, maar er zit er geen in.
async fn start(map: &Path) -> Server {
    Server::start(Instellingen {
        adres: "127.0.0.1:0".parse().unwrap(),
        map: map.to_owned(),
        sleutel: SLEUTEL.into(),
        dashboard: None,
        invoer: None,
        zaalindeling: None,
        open_data: None,
    })
    .await
    .expect("server start zonder gamepad")
}

/// Stuurt een Opdracht als knop via HTTP en geeft het antwoord terug.
async fn opdracht(server: &Server, o: &Opdracht) -> BerichtVanServer {
    reqwest::Client::new()
        .post(format!("http://{}/api/opdracht", server.adres()))
        .bearer_auth(SLEUTEL)
        .header("X-Afzender", "Proef")
        .json(o)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
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

#[tokio::test]
async fn bewaar_als_standpunt_neemt_de_camera_van_nu_en_wissen_wacht_tot_niets_het_gebruikt() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;

    // zonder plek: de server neemt de camera van de Ether, die nu op A staat
    let bewaar = Opdracht::BewaarStandpunt {
        naam: "Close-up".into(),
        stand: None,
    };
    assert!(matches!(
        opdracht(&server, &bewaar).await,
        BerichtVanServer::Uitgevoerd { .. }
    ));
    let s = stand(&server).await.stand;
    let nieuw = s
        .standpunten
        .iter()
        .find(|p| p.naam == "Close-up")
        .expect("bewaard");
    assert_eq!(nieuw.stand, s.standpunt("A").unwrap().stand);
    assert_eq!(nieuw.toets(), Some("J"));
    let letter = nieuw.letter.clone();

    let graphic = Graphic::Titelkaart {
        kop: "Voorbeeldgegevens".into(),
        regel: String::new(),
    };
    let klaar = Opdracht::ZetKlaar {
        graphic,
        standpunt: Some(letter.clone()),
    };
    assert!(matches!(
        opdracht(&server, &klaar).await,
        BerichtVanServer::Uitgevoerd { .. }
    ));

    let wis = Opdracht::WisStandpunt {
        letter: letter.clone(),
    };
    let BerichtVanServer::Geweigerd { reden } = opdracht(&server, &wis).await else {
        panic!("wissen had geweigerd moeten worden");
    };
    assert_eq!(
        reden,
        "Standpunt J (Close-up) wordt gebruikt door de klaargezette Graphic Titelkaart \"Voorbeeldgegevens\""
    );
    assert!(stand(&server).await.stand.standpunt(&letter).is_some());

    // een andere Graphic klaarzetten: dan gebruikt niets het meer
    let ander = Opdracht::ZetKlaar {
        graphic: Graphic::Titelkaart {
            kop: "Andere kop".into(),
            regel: String::new(),
        },
        standpunt: None,
    };
    assert!(matches!(
        opdracht(&server, &ander).await,
        BerichtVanServer::Uitgevoerd { .. }
    ));
    assert!(matches!(
        opdracht(&server, &wis).await,
        BerichtVanServer::Uitgevoerd { .. }
    ));
    assert!(stand(&server).await.stand.standpunt(&letter).is_none());
    server.stop().await;
}

/// Voert een Opdracht uit en eist dat de server hem uitvoert.
async fn uitgevoerd(server: &Server, o: Opdracht) {
    let antwoord = opdracht(server, &o).await;
    assert!(
        matches!(antwoord, BerichtVanServer::Uitgevoerd { .. }),
        "{o:?}: {antwoord:?}"
    );
}

#[tokio::test]
async fn de_server_neemt_tijdens_oefenen_100_standen_per_seconde_op_en_bewaart_de_camerarit() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    uitgevoerd(&server, Opdracht::ZetOefenen { aan: true }).await;
    uitgevoerd(
        &server,
        Opdracht::ZetOpnemen {
            aan: true,
            voor: None,
            opname: None,
        },
    )
    .await;
    let begin = std::time::Instant::now();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    // stoppen zonder opname: de server vult in wat hij opnam
    uitgevoerd(
        &server,
        Opdracht::ZetOpnemen {
            aan: false,
            voor: None,
            opname: None,
        },
    )
    .await;
    let ms = begin.elapsed().as_millis() as usize;
    let s = stand(&server).await.stand;
    let rit = s
        .cameraritten
        .iter()
        .find(|r| !r.vast)
        .expect("een eigen Camerarit");
    // 100 per seconde: in ruim 300 ms ongeveer 30, met een paar tikken speling
    assert!(
        (25..=ms / 10 + 5).contains(&rit.opname.len()),
        "{} standen in {ms} ms",
        rit.opname.len()
    );
    assert_eq!(rit.toets.as_deref(), Some("J"));
    assert_eq!(s.camera.opname, None);
    assert_eq!(
        s.camera.proef.as_ref().map(|p| p.letter.as_str()),
        Some(rit.letter.as_str())
    );
    // zonder gamepad stond het Voorbeeld stil, op A
    let a = s.standpunt("A").unwrap().stand;
    assert!(
        rit.opname
            .iter()
            .all(|p| (p[0] - a.positie[0]).abs() < 1e-3 && (p[1] - a.positie[1]).abs() < 1e-3)
    );
    server.stop().await;
}
