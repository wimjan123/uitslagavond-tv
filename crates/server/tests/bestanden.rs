//! Lettertypes, logo's en foto's: de server levert ze uit de map `bestanden` aan de Renderers.

use server::{Instellingen, Server};

const SLEUTEL: &str = "proefsleutel";

async fn start(map: &std::path::Path) -> Server {
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

async fn haal(server: &Server, naam: &str, sleutel: &str) -> (u16, Vec<u8>) {
    let a = reqwest::Client::new()
        .get(format!("http://{}/api/bestand/{naam}", server.adres()))
        .bearer_auth(sleutel)
        .send()
        .await
        .unwrap();
    (a.status().as_u16(), a.bytes().await.unwrap().to_vec())
}

#[tokio::test]
async fn de_server_levert_een_bestand_uit_zijn_map_alleen_met_de_sleutel() {
    let map = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(map.path().join("bestanden/logos")).unwrap();
    std::fs::write(map.path().join("bestanden/Letter.ttf"), b"letters").unwrap();
    std::fs::write(map.path().join("bestanden/logos/a.png"), b"logo").unwrap();
    std::fs::write(map.path().join("geheim.txt"), b"geheim").unwrap();
    let server = start(map.path()).await;

    assert_eq!(
        haal(&server, "Letter.ttf", SLEUTEL).await,
        (200, b"letters".to_vec())
    );
    assert_eq!(
        haal(&server, "logos/a.png", SLEUTEL).await,
        (200, b"logo".to_vec())
    );
    assert_eq!(haal(&server, "Letter.ttf", "fout").await.0, 401);
    assert_eq!(haal(&server, "Bestaat-niet.ttf", SLEUTEL).await.0, 404);
    // buiten de map: geweigerd
    assert_eq!(haal(&server, "..%2Fgeheim.txt", SLEUTEL).await.0, 400);
    assert_eq!(haal(&server, ".verborgen", SLEUTEL).await.0, 400);
    server.stop().await;
}

async fn stand(server: &Server) -> gedeeld::GenummerdeStand {
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

/// De Persoonskaart noemt een foto als `fotos/<soort>/<id>`: de Renderer haalt hem zo op, ook al
/// staat hij naast de Stand en niet in de map `bestanden`.
#[tokio::test]
async fn de_renderer_haalt_een_foto_van_de_persoonskaart_op_en_de_regie_plaatst_een_foto_van_een_eigen_persoon() {
    use gedeeld::Opdracht;
    use gedeeld::persoonskaart::EigenPersoon;
    let map = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(map.path().join("fotos/open-data")).unwrap();
    std::fs::create_dir_all(map.path().join("fotos/eigen")).unwrap();
    // VOORBEELDGEGEVENS: het begin van een JPEG en een PNG
    std::fs::write(map.path().join("fotos/open-data/abc-1.jpg"), b"\xFF\xD8\xFF uit de open data").unwrap();
    std::fs::write(map.path().join("fotos/eigen/abc-1.png"), b"\x89PNG\r\n\x1a\n eigen").unwrap();
    let server = start(map.path()).await;
    assert_eq!(haal(&server, "fotos/open-data/abc-1", SLEUTEL).await, (200, b"\xFF\xD8\xFF uit de open data".to_vec()));
    assert_eq!(haal(&server, "fotos/eigen/abc-1", SLEUTEL).await, (200, b"\x89PNG\r\n\x1a\n eigen".to_vec()));
    assert_eq!(haal(&server, "fotos/eigen/bestaat-niet", SLEUTEL).await.0, 404);
    assert_eq!(haal(&server, "fotos/anders/abc-1", SLEUTEL).await.0, 404);

    // een eigen persoon: eerst bewaren, dan de foto plaatsen
    let client = reqwest::Client::new();
    let persoon = EigenPersoon {
        id: String::new(),
        naam: "Mila Verzonnen".into(),
        foto: false,
        partij: None,
        regel: String::new(),
        geboortedatum: None,
        woonplaats: None,
        geboorteplaats: None,
    };
    let r = client
        .post(format!("http://{}/api/opdracht", server.adres()))
        .bearer_auth(SLEUTEL)
        .json(&Opdracht::BewaarPersoon { persoon })
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let id = stand(&server).await.stand.personen[0].id.clone();
    let foto = b"\xFF\xD8\xFF eigen persoon".to_vec();
    let r = client
        .put(format!("http://{}/api/foto/{id}", server.adres()))
        .bearer_auth(SLEUTEL)
        .body(foto.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert!(stand(&server).await.stand.personen[0].foto);
    assert_eq!(haal(&server, &format!("fotos/personen/{id}"), SLEUTEL).await, (200, foto));
    let r = client
        .delete(format!("http://{}/api/foto/{id}", server.adres()))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert!(!stand(&server).await.stand.personen[0].foto);
    assert_eq!(haal(&server, &format!("fotos/personen/{id}"), SLEUTEL).await.0, 404);
    server.stop().await;
}

async fn plaats(server: &Server, naam: &str, inhoud: &[u8]) -> (u16, serde_json::Value) {
    let a = reqwest::Client::new()
        .post(format!("http://{}/api/bestand?naam={naam}", server.adres()))
        .bearer_auth(SLEUTEL)
        .body(inhoud.to_vec())
        .send()
        .await
        .unwrap();
    (a.status().as_u16(), a.json().await.unwrap_or_default())
}

/// De vormgevingstab plaatst een programmalogo of lettertype. Elk bestand krijgt een eigen naam
/// bij zijn inhoud, zodat een Renderer met een kopie van het oude logo nooit het verkeerde
/// toont en de Graphic in de Ether het oude logo houdt.
#[tokio::test]
async fn de_regie_plaatst_een_logo_of_lettertype_onder_een_nieuwe_naam_en_de_renderer_haalt_het_op() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    // VOORBEELDGEGEVENS: het begin van een PNG en van een TrueType-lettertype
    let logo = b"\x89PNG\r\n\x1a\n voorbeeldlogo";
    let (status, antwoord) = plaats(&server, "Mijn logo.png", logo).await;
    assert_eq!(status, 200, "{antwoord}");
    let naam = antwoord["naam"].as_str().unwrap().to_string();
    assert!(gedeeld::sjabloon::geldige_bestandsnaam(&naam), "{naam}");
    assert!(naam.ends_with(".png"), "{naam}");
    assert_eq!(haal(&server, &naam, SLEUTEL).await, (200, logo.to_vec()));

    // Hetzelfde bestand nog eens: dezelfde naam. Een ander logo: een andere naam.
    assert_eq!(plaats(&server, "Mijn logo.png", logo).await.1["naam"], naam.as_str());
    let ander = plaats(&server, "Mijn logo.png", b"\x89PNG\r\n\x1a\n ander logo").await.1;
    assert_ne!(ander["naam"], naam.as_str());

    let letter = plaats(&server, "Huisletter-Bold.ttf", b"\0\x01\0\0 letters").await;
    assert_eq!(letter.0, 200);
    assert!(letter.1["naam"].as_str().unwrap().ends_with(".ttf"));

    // Geen beeld en geen lettertype: geweigerd, met de reden.
    let (status, antwoord) = plaats(&server, "virus.exe", b"MZ niets").await;
    assert_eq!(status, 422);
    assert!(antwoord["reden"].as_str().unwrap().contains("PNG"), "{antwoord}");
    server.stop().await;
}

/// Een stand.json van vóór 22 november 2026 had de eerste standaardstijl: die wordt bij het
/// laden de huidige standaard. Een zelf aangepaste Stijl blijft.
#[tokio::test]
async fn een_oude_stand_met_de_oude_standaardstijl_krijgt_de_nieuwe_standaard() {
    use gedeeld::sjabloon::Stijl;
    let oude_stijl = serde_json::json!({
        "kleuren": {"hoofd": "#12357a", "accent": "#6f9fe0", "tekst": "#ffffff", "vlak": "#080f29", "achtergrond": "#0d1220"},
        "lettertypes": {"kop": "DejaVuSans-Bold.ttf", "tekst": "DejaVuSans.ttf", "cijfers": "DejaVuSans-Bold.ttf"},
        "programmalogo": {"bestand": null, "tekst": "UITSLAGAVOND"},
        "snelheid": 100
    });
    for (stijl, verwacht) in [
        (oude_stijl.clone(), Stijl::default()),
        ({
            let mut eigen = oude_stijl.clone();
            eigen["snelheid"] = 80.into();
            eigen
        }, {
            let mut eigen: Stijl = serde_json::from_value(oude_stijl.clone()).unwrap();
            eigen.snelheid = 80;
            eigen
        }),
    ] {
        let map = tempfile::tempdir().unwrap();
        let oud = serde_json::json!({"volgnummer": 12, "stand": {"stijl": stijl}});
        std::fs::write(map.path().join("stand.json"), oud.to_string()).unwrap();
        let server = start(map.path()).await;
        let s = stand(&server).await;
        assert_eq!(s.volgnummer, 12);
        assert_eq!(s.stand.stijl, verwacht);
        server.stop().await;
    }
}
