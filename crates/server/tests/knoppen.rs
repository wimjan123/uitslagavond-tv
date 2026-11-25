//! Knoppen via HTTP, zoals een Stream Deck met Bitfocus Companion ze stuurt (zie
//! `docs/stream-deck.md`): elke Opdracht van een knop met en zonder sleutel.

use std::path::Path;

use gedeeld::{BerichtVanServer, GenummerdeStand, Graphic, Opdracht};
use reqwest::StatusCode;
use serde_json::{Value, json};
use server::{Instellingen, Server};

const SLEUTEL: &str = "proefsleutel";

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
    .expect("server start")
}

fn url(server: &Server, pad: &str) -> String {
    format!("http://{}{pad}", server.adres())
}

async fn stand(server: &Server) -> GenummerdeStand {
    reqwest::Client::new()
        .get(url(server, "/api/stand"))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

/// Een POST zoals Companion hem stuurt: vaste body, de sleutel en de naam van de knop als kop.
async fn post(
    server: &Server,
    pad: &str,
    body: Option<&Value>,
    sleutel: Option<&str>,
) -> reqwest::Response {
    let mut v = reqwest::Client::new()
        .post(url(server, pad))
        .header("X-Afzender", "Stream Deck");
    if let Some(s) = sleutel {
        v = v.bearer_auth(s);
    }
    if let Some(b) = body {
        v = v.json(b);
    }
    v.send().await.unwrap()
}

/// Drukt een knop: eerst zonder en met een verkeerde sleutel (geweigerd, de Stand blijft gelijk),
/// dan met de sleutel. Geeft het antwoord van de server.
async fn druk(server: &Server, pad: &str, body: Option<Value>) -> (StatusCode, BerichtVanServer) {
    let voor = stand(server).await.volgnummer;
    for sleutel in [None, Some("fout")] {
        let r = post(server, pad, body.as_ref(), sleutel).await;
        assert_eq!(
            r.status(),
            StatusCode::UNAUTHORIZED,
            "{pad} met sleutel {sleutel:?}"
        );
    }
    assert_eq!(
        stand(server).await.volgnummer,
        voor,
        "{pad} zonder sleutel veranderde de Stand"
    );
    let r = post(server, pad, body.as_ref(), Some(SLEUTEL)).await;
    let status = r.status();
    (status, r.json().await.expect("antwoord als JSON"))
}

/// Een knop die moet lukken.
async fn druk_ok(server: &Server, pad: &str, body: Value) -> u64 {
    match druk(server, pad, Some(body)).await {
        (StatusCode::OK, BerichtVanServer::Uitgevoerd { volgnummer }) => volgnummer,
        anders => panic!("{pad} niet uitgevoerd: {anders:?}"),
    }
}

/// Zet via de gewone Opdracht een Opdracht klaar, voor de proeven die een begintoestand nodig hebben.
async fn voer_uit(server: &Server, opdracht: Opdracht) {
    let r = post(
        server,
        "/api/opdracht",
        Some(&serde_json::to_value(opdracht).unwrap()),
        Some(SLEUTEL),
    )
    .await;
    assert_eq!(r.status(), StatusCode::OK);
}

fn titelkaart(kop: &str) -> Graphic {
    Graphic::Titelkaart {
        kop: kop.into(),
        regel: "Voorbeeldgegevens".into(),
    }
}

#[tokio::test]
async fn knop_zet_preset_klaar_op_zijn_nummer() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    voer_uit(
        &server,
        Opdracht::BewaarPreset {
            graphic: titelkaart("Preset één"),
            standpunt: None,
        },
    )
    .await;
    voer_uit(
        &server,
        Opdracht::BewaarPreset {
            graphic: titelkaart("Preset twee"),
            standpunt: None,
        },
    )
    .await;

    druk_ok(
        &server,
        "/api/opdracht",
        json!({ "soort": "ZetPresetKlaar", "nummer": 2 }),
    )
    .await;

    let s = stand(&server).await.stand;
    let klaar = s.klaar.expect("Preset klaar");
    assert_eq!(klaar.graphic, titelkaart("Preset twee"));
    assert_eq!(klaar.klaargezet_door, "Stream Deck");
}

#[tokio::test]
async fn knop_toon_zet_wat_klaarstaat_in_de_ether() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    voer_uit(
        &server,
        Opdracht::ZetKlaar {
            graphic: titelkaart("Goedenavond"),
            standpunt: None,
        },
    )
    .await;

    druk_ok(&server, "/api/opdracht", json!({ "soort": "Toon" })).await;

    let ether = stand(&server)
        .await
        .stand
        .ether
        .expect("Graphic in de Ether");
    assert_eq!(ether.graphic, titelkaart("Goedenavond"));
    assert_eq!(ether.getoond_door, "Stream Deck");
}

#[tokio::test]
async fn knop_toon_met_versie_weigert_als_de_graphic_intussen_veranderde() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    voer_uit(
        &server,
        Opdracht::ZetKlaar {
            graphic: titelkaart("Eerste kop"),
            standpunt: None,
        },
    )
    .await;
    let gezien = stand(&server).await.stand.klaar.unwrap();
    // intussen past een Werkplek de klaargezette Graphic aan (via HTTP met een eigen naam)
    let wijzig = Opdracht::WijzigKlaar {
        id: gezien.id,
        graphic: titelkaart("Nieuwe kop"),
        standpunt: None,
    };
    let r = reqwest::Client::new()
        .post(url(&server, "/api/opdracht"))
        .bearer_auth(SLEUTEL)
        .header("X-Afzender", "Tablet")
        .json(&wijzig)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let (status, antwoord) = druk(
        &server,
        "/api/opdracht",
        Some(json!({ "soort": "Toon", "versie": gezien.versie })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let BerichtVanServer::Geweigerd { reden } = antwoord else {
        panic!("niet geweigerd: {antwoord:?}")
    };
    assert!(reden.contains("Tablet"), "{reden}");
    assert!(stand(&server).await.stand.ether.is_none());

    // met de versie die nu klaarstaat lukt het
    let nu = stand(&server).await.stand.klaar.unwrap().versie;
    druk_ok(
        &server,
        "/api/opdracht",
        json!({ "soort": "Toon", "versie": nu }),
    )
    .await;
    assert_eq!(
        stand(&server).await.stand.ether.unwrap().graphic,
        titelkaart("Nieuwe kop")
    );
}

#[tokio::test]
async fn knop_bijwerken_neemt_de_nieuwe_versie_over_in_de_ether() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    voer_uit(
        &server,
        Opdracht::ZetKlaar {
            graphic: titelkaart("Oud"),
            standpunt: None,
        },
    )
    .await;
    voer_uit(&server, Opdracht::Toon { versie: None }).await;
    let id = stand(&server).await.stand.klaar.unwrap().id;
    voer_uit(
        &server,
        Opdracht::WijzigKlaar {
            id,
            graphic: titelkaart("Bijgewerkt"),
            standpunt: None,
        },
    )
    .await;

    druk_ok(&server, "/api/opdracht", json!({ "soort": "Bijwerken" })).await;

    assert_eq!(
        stand(&server).await.stand.ether.unwrap().graphic,
        titelkaart("Bijgewerkt")
    );
}

#[tokio::test]
async fn knop_uit_beeld_haalt_de_graphic_uit_de_ether() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    voer_uit(
        &server,
        Opdracht::ZetKlaar {
            graphic: titelkaart("Weg ermee"),
            standpunt: None,
        },
    )
    .await;
    voer_uit(&server, Opdracht::Toon { versie: None }).await;

    druk_ok(&server, "/api/opdracht", json!({ "soort": "UitBeeld" })).await;

    assert!(stand(&server).await.stand.ether.is_none());
}

#[tokio::test]
async fn knop_kiest_een_standpunt() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;

    druk_ok(
        &server,
        "/api/opdracht",
        json!({ "soort": "KiesStandpunt", "letter": "D" }),
    )
    .await;

    let camera = stand(&server).await.stand.camera;
    assert_eq!((camera.standpunt.as_str(), camera.knip), ("D", false));
}

#[tokio::test]
async fn knop_zet_de_vrije_camera_aan_en_uit() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;

    druk_ok(
        &server,
        "/api/opdracht",
        json!({ "soort": "ZetVrijeCamera", "aan": true }),
    )
    .await;
    assert!(stand(&server).await.stand.camera.vrij);
    druk_ok(
        &server,
        "/api/opdracht",
        json!({ "soort": "ZetVrijeCamera", "aan": false }),
    )
    .await;
    assert!(!stand(&server).await.stand.camera.vrij);
}

/// Eén veld van de knopstatus als platte tekst, zoals Companion het in een variabele zet.
async fn veld(server: &Server, naam: &str) -> String {
    let r = reqwest::Client::new()
        .get(url(server, &format!("/api/knoppen/{naam}")))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK, "veld {naam}");
    r.text().await.unwrap()
}

#[tokio::test]
async fn knopstatus_zegt_per_veld_wat_klaarstaat_en_wat_in_beeld_is() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    for (naam, verwacht) in [
        ("klaar_versie", ""),
        ("kan_tonen", "nee"),
        ("in_beeld", "nee"),
        ("kan_bijwerken", "nee"),
        ("standpunt", "A"),
        ("vrije_camera", "uit"),
        ("preset_klaar", ""),
        ("preset_live", ""),
    ] {
        assert_eq!(veld(&server, naam).await, verwacht, "{naam} bij het begin");
    }

    voer_uit(
        &server,
        Opdracht::BewaarPreset {
            graphic: titelkaart("Preset"),
            standpunt: None,
        },
    )
    .await;
    voer_uit(&server, Opdracht::ZetPresetKlaar { nummer: 1 }).await;
    let klaar = stand(&server).await.stand.klaar.unwrap();
    assert_eq!(
        veld(&server, "klaar_versie").await,
        klaar.versie.to_string()
    );
    assert_eq!(veld(&server, "klaar_door").await, "Stream Deck");
    assert_eq!(veld(&server, "kan_tonen").await, "ja");
    assert_eq!(veld(&server, "preset_klaar").await, "1");

    voer_uit(&server, Opdracht::Toon { versie: None }).await;
    voer_uit(
        &server,
        Opdracht::KiesStandpunt {
            letter: "G".into(),
            knip: true,
        },
    )
    .await;
    voer_uit(&server, Opdracht::ZetVrijeCamera { aan: true }).await;
    assert_eq!(veld(&server, "in_beeld").await, "ja");
    assert_eq!(veld(&server, "kan_tonen").await, "nee");
    assert_eq!(veld(&server, "preset_live").await, "1");
    assert_eq!(veld(&server, "standpunt").await, "G");
    assert_eq!(veld(&server, "vrije_camera").await, "aan");

    // alles tegelijk als JSON, en een onbekend veld
    let r = reqwest::Client::new()
        .get(url(&server, "/api/knoppen"))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap();
    let alles: Value = r.json().await.unwrap();
    assert_eq!(alles["in_beeld"], "ja");
    let r = reqwest::Client::new()
        .get(url(&server, "/api/knoppen/onbekend"))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn knopstatus_vraagt_de_sleutel() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    for pad in ["/api/knoppen", "/api/knoppen/in_beeld"] {
        let r = reqwest::Client::new()
            .get(url(&server, pad))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED, "{pad}");
    }
}

#[tokio::test]
async fn wisselknop_zet_de_vrije_camera_om_zoals_v_op_het_dashboard() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;

    let (status, _) = druk(&server, "/api/vrije-camera/wissel", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(stand(&server).await.stand.camera.vrij);
    let (status, _) = druk(&server, "/api/vrije-camera/wissel", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!stand(&server).await.stand.camera.vrij);
}

#[tokio::test]
async fn knop_kleur_uit_geeft_de_stoelen_hun_eigen_leer_terug() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    assert_eq!(veld(&server, "kleur").await, "uit");

    let rood = "#d4202c".parse().unwrap();
    voer_uit(
        &server,
        Opdracht::ZetStoelkleuren {
            kleuren: vec![rood; 150],
        },
    )
    .await;
    assert_eq!(veld(&server, "kleur").await, "aan");

    druk_ok(&server, "/api/opdracht", json!({"soort": "KleurUit"})).await;
    assert_eq!(veld(&server, "kleur").await, "uit");
    assert!(stand(&server).await.stand.stoelkleuren.is_empty());

    // nog eens: er is niets meer uit te zetten
    let (status, _) = druk(&server, "/api/opdracht", Some(json!({"soort": "KleurUit"}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
