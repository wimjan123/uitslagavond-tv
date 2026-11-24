//! Kamerleden uit de open data van de Tweede Kamer, met vastgelegde antwoorden van de API
//! (`tests/opendata/`, zie de README daar). Geen netwerk: een nep-API op 127.0.0.1 geeft die
//! antwoorden terug. De foto's in deze tests zijn voorbeeldgegevens (een paar bytes).

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::{Path as UrlDeel, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use gedeeld::cijfers::{Invoer, TIJDELIJKE_KLEUREN};
use gedeeld::kamer::Foto;
use gedeeld::{BerichtVanServer, GenummerdeStand, Herkomst, Opdracht, Stand};

const H: Herkomst = Herkomst { door: "Proef", volgnummer: 1, tijd_ms: 0 };
use jiff::civil::{Date, date};
use server::opendata::{OpgehaaldeKamer, kamer_op, lees_pagina};
use server::{Instellingen, OpenData, Server};

const OP_15_NOV: &str = include_str!("opendata/kamerleden-2026-11-15.json");
const OP_22_NOV: &str = include_str!("opendata/kamerleden-2026-11-22.json");
const SLEUTEL: &str = "proefsleutel";

/// Voorbeeldfoto: het begin van een JPEG, meer niet.
const VOORBEELDFOTO: &[u8] = b"\xFF\xD8\xFF\xE0 voorbeeldfoto uit de nep-API";
const EIGEN_FOTO: &[u8] = b"\x89PNG\r\n\x1a\n eigen voorbeeldfoto van de omroep";

const BECKER: &str = "a962ffcb-a300-45e1-8850-70c7173c233e";
const VAN_GRONINGEN: &str = "afc7a846-0edb-4172-a6ac-f924caf1b69b";
const AL_BIYATI: &str = "01be9ad7-0444-483a-9983-5f66f1402f6a";
const EL_ABASSI: &str = "209ce05e-66ee-4999-928c-00f152b40ead";

fn kamer(antwoord: &str, peildatum: Date) -> OpgehaaldeKamer {
    let pagina = lees_pagina(antwoord).expect("vastgelegd antwoord is leesbaar");
    assert!(pagina.volgende.is_none());
    kamer_op(&pagina.personen, peildatum)
}

fn lees_kamer(stand: &mut Stand, k: &OpgehaaldeKamer, peildatum: &str) {
    stand
        .voer_uit(&Opdracht::LeesKamer {
            opgehaald: format!("{peildatum}T12:00:00+01:00"),
            peildatum: peildatum.into(),
            kamerleden: k.kamerleden.clone(),
            fracties: k.fracties.clone(),
        }, &H)
        .expect("Kamer gelezen");
}

fn namen(stand: &Stand) -> Vec<String> {
    stand.kamer.kamerleden.iter().map(|k| k.naam()).collect()
}

#[test]
fn ophalen_geeft_150_kamerleden_in_17_fracties_met_fractievoorzitters_en_een_kamerlid_zonder_foto()
{
    let k = kamer(OP_22_NOV, date(2026, 11, 22));
    assert_eq!(k.kamerleden.len(), 150);
    assert_eq!(k.fracties.len(), 17);
    assert_eq!(
        k.fracties.iter().map(|f| u32::from(f.zetels)).sum::<u32>(),
        150
    );
    assert_eq!(
        k.kamerleden.iter().filter(|l| l.fractievoorzitter).count(),
        17
    );

    let el_abassi = k.kamerleden.iter().find(|l| l.id == EL_ABASSI).unwrap();
    assert_eq!(el_abassi.naam(), "Ismail el Abassi");
    assert_eq!(el_abassi.van, "2023-12-06");
    assert_eq!(el_abassi.woonplaats.as_deref(), Some("Utrecht"));
    assert_eq!(el_abassi.geboortedatum.as_deref(), Some("1983-02-27"));
    assert_eq!(el_abassi.foto, Foto::OpenData);
    let denk = k
        .fracties
        .iter()
        .find(|f| f.id == el_abassi.fractie)
        .unwrap();
    assert_eq!((denk.afkorting.as_str(), denk.zetels), ("DENK", 3));

    // De open data heeft voor één Kamerlid geen foto: de Persoonskaart toont dan voorletters.
    let zonder: Vec<_> = k
        .kamerleden
        .iter()
        .filter(|l| l.foto == Foto::Geen)
        .collect();
    assert_eq!(zonder.len(), 1);
    assert_eq!(zonder[0].id, AL_BIYATI);
    assert_eq!(zonder[0].voorletters(), "IB");
}

#[test]
fn de_eerste_dag_in_de_kamer_is_de_vroegste_fractiezetel_ook_uit_een_eerdere_periode() {
    // VOORBEELDGEGEVENS in de vorm van de open data: een Kamerlid dat in 2010 begon, eruit ging
    // en sinds 2023 weer in de Kamer zit
    let antwoord = r#"{"value": [{
        "Id": "p1", "Roepnaam": "Proef", "Achternaam": "Persoon",
        "FractieZetelPersoon": [
            {"FractieZetel_Id": "z-oud", "Functie": "Lid", "Van": "2010-06-17T00:00:00+02:00",
             "TotEnMet": "2012-09-19T00:00:00+02:00",
             "FractieZetel": {"Fractie": {"Id": "f-oud", "Afkorting": "OUD", "NaamNL": "Oud", "AantalZetels": 0}}},
            {"FractieZetel_Id": "z-nu", "Functie": "Fractievoorzitter", "Van": "2023-12-06T00:00:00+01:00",
             "TotEnMet": null,
             "FractieZetel": {"Fractie": {"Id": "f-nu", "Afkorting": "NU", "NaamNL": "Nu", "AantalZetels": 3}}}
        ]}]}"#;
    let k = kamer(antwoord, date(2026, 11, 22));
    let lid = &k.kamerleden[0];
    assert_eq!((lid.fractie.as_str(), lid.van.as_str()), ("f-nu", "2023-12-06"));
    assert!(lid.fractievoorzitter);
    assert_eq!(lid.in_de_kamer_sinds.as_deref(), Some("2010-06-17"));
    assert_eq!(k.fracties.len(), 1, "alleen de Fractie van nu");

    // de vraag aan de API haalt de hele geschiedenis van FractieZetelPersoon op
    let (_, parameters) = server::opendata::vraag("https://api", date(2026, 11, 22));
    let expand = &parameters.iter().find(|(k, _)| *k == "$expand").unwrap().1;
    assert!(expand.starts_with("FractieZetelPersoon($filter=Verwijderd eq false;"), "{expand}");
}

#[test]
fn een_nieuw_kamerlid_telt_mee_vanaf_de_dag_van_de_beediging() {
    // Bente Becker zit tot en met 16 november, Dieke van Groningen vanaf 17 november.
    let zit = |dag: i8, id: &str| {
        kamer(OP_15_NOV, date(2026, 11, dag))
            .kamerleden
            .iter()
            .any(|k| k.id == id)
    };
    assert!(zit(16, BECKER) && !zit(16, VAN_GRONINGEN));
    assert!(!zit(17, BECKER) && zit(17, VAN_GRONINGEN));
    assert_eq!(kamer(OP_15_NOV, date(2026, 11, 16)).kamerleden.len(), 150);
    assert_eq!(kamer(OP_15_NOV, date(2026, 11, 17)).kamerleden.len(), 150);
}

#[test]
fn een_wissel_staat_in_de_melding_met_de_opvolger_op_dezelfde_fractiezetel() {
    let mut stand = Stand::default();
    lees_kamer(
        &mut stand,
        &kamer(OP_15_NOV, date(2026, 11, 15)),
        "2026-11-15",
    );
    let eerste = stand.kamer.laatste_ophaal.clone().unwrap();
    assert!(
        eerste.nieuw.is_empty() && eerste.vertrokken.is_empty(),
        "{eerste:?}"
    );
    assert!(namen(&stand).contains(&"Bente Becker".into()));

    lees_kamer(
        &mut stand,
        &kamer(OP_22_NOV, date(2026, 11, 22)),
        "2026-11-22",
    );
    let m = stand.kamer.laatste_ophaal.clone().unwrap();
    assert_eq!(m.peildatum, "2026-11-22");
    assert_eq!(m.vertrokken, ["Bente Becker"]);
    assert_eq!(m.nieuw, ["Dieke van Groningen"]);
    assert_eq!(m.opvolgers.len(), 1);
    assert_eq!(
        (
            m.opvolgers[0].opvolger.as_str(),
            m.opvolgers[0].vertrokken.as_str()
        ),
        ("Dieke van Groningen", "Bente Becker")
    );
    assert!(m.andere_fractie.is_empty());
    assert!(!namen(&stand).contains(&"Bente Becker".into()));
    assert_eq!(stand.kamer.kamerleden.len(), 150);
    let vvd = stand.kamer.kamerlid(VAN_GRONINGEN).unwrap();
    assert_eq!(stand.kamer.partij_van(vvd), Some("vvd"));
}

#[test]
fn een_fractie_die_niet_in_de_partijlijst_staat_krijgt_een_tijdelijke_kleur_de_korte_naam_en_een_waarschuwing()
 {
    // De partijlijst van de verkiezing van 2025 (Kiesraad): daarin staan PRO, Groep Markuszower
    // en Lid Keijzer nog niet.
    let mut stand = Stand::default();
    stand
        .voer_uit(&Opdracht::LeesInvoer {
            bestand: "uitslag-tk2025.json".into(),
            ontvangen: "12.00".into(),
            invoer: Invoer::uit_json(include_str!("../../../cijfers/uitslag-tk2025.json")),
        }, &H)
        .unwrap();
    let k = kamer(OP_22_NOV, date(2026, 11, 22));
    lees_kamer(&mut stand, &k, "2026-11-22");

    let mut nieuw = stand.kamer.laatste_ophaal.clone().unwrap().fracties_nieuw;
    nieuw.sort();
    assert_eq!(nieuw, ["Groep Markuszower", "Lid Keijzer", "PRO"]);
    for (sleutel, kort, naam) in [
        ("pro", "PRO", "Progressief Nederland"),
        (
            "groep-markuszower",
            "Groep Markuszower",
            "Groep Markuszower",
        ),
        ("lid-keijzer", "Lid Keijzer", "Lid Keijzer"),
    ] {
        let p = stand
            .cijfers
            .partij(sleutel)
            .unwrap_or_else(|| panic!("{sleutel} ontbreekt"));
        assert_eq!((p.kort.as_str(), p.naam.as_str()), (kort, naam));
        assert!(
            p.kleur_tijdelijk && p.fractie_zonder_lijst && p.logo.is_none(),
            "{p:?}"
        );
        assert!(TIJDELIJKE_KLEUREN.contains(&p.kleur.as_str()), "{p:?}");
    }
    // De andere 14 Fracties horen bij een Partij die er al was.
    let partij = |afkorting: &str| {
        let f = stand
            .kamer
            .fracties
            .iter()
            .find(|f| f.afkorting == afkorting)
            .unwrap();
        stand.kamer.partij_van[&f.id].clone()
    };
    assert_eq!(partij("ChristenUnie"), "cu");
    assert_eq!(partij("FVD"), "fvd");
    assert_eq!(partij("50PLUS"), "50plus");
    assert_eq!(partij("BBB"), "bbb");
    assert_eq!(stand.cijfers.partijen.len(), 27 + 3);

    // Opnieuw ophalen voegt niets dubbel toe en waarschuwt niet opnieuw.
    lees_kamer(&mut stand, &k, "2026-11-22");
    assert!(
        stand
            .kamer
            .laatste_ophaal
            .as_ref()
            .unwrap()
            .fracties_nieuw
            .is_empty()
    );
    assert_eq!(stand.cijfers.partijen.len(), 30);
}

/// Een nep-API met de vastgelegde antwoorden. Foto's geeft hij voor iedereen behalve Al Biyati.
async fn nep_api(antwoord: &'static str) -> (String, Arc<Mutex<u32>>, Arc<Mutex<u32>>) {
    let vragen = Arc::new(Mutex::new(0u32));
    let fotos = Arc::new(Mutex::new(0u32));
    let geteld = fotos.clone();
    let app = Router::new()
        .route(
            "/Persoon",
            get(move |State(n): State<Arc<Mutex<u32>>>| async move {
                *n.lock().unwrap() += 1;
                ([(header::CONTENT_TYPE, "application/json")], antwoord)
            }),
        )
        .route(
            "/{persoon}/resource",
            get(move |UrlDeel(persoon): UrlDeel<String>| async move {
                *geteld.lock().unwrap() += 1;
                if persoon.contains(AL_BIYATI) {
                    StatusCode::NOT_FOUND.into_response()
                } else {
                    ([(header::CONTENT_TYPE, "image/jpeg")], VOORBEELDFOTO).into_response()
                }
            }),
        )
        .with_state(vragen.clone());
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adres = l.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    (format!("http://{adres}"), vragen, fotos)
}

async fn start(map: &Path, basis: String) -> Server {
    start_elke(map, basis, Duration::from_secs(3600)).await
}

async fn start_elke(map: &Path, basis: String, elke: Duration) -> Server {
    Server::start(Instellingen {
        adres: "127.0.0.1:0".parse().unwrap(),
        map: map.to_owned(),
        sleutel: SLEUTEL.into(),
        dashboard: None,
        invoer: None,
        zaalindeling: None,
        open_data: Some(OpenData {
            basis,
            elke,
            peildatum: Some(date(2026, 11, 22)),
            debat_direct: None,
        }),
    })
    .await
    .expect("server start")
}

fn http(server: &Server, pad: &str) -> String {
    format!("http://{}{pad}", server.adres())
}

async fn stand(server: &Server) -> GenummerdeStand {
    reqwest::Client::new()
        .get(http(server, "/api/stand"))
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

async fn foto(server: &Server, id: &str) -> (StatusCode, Vec<u8>) {
    let r = reqwest::Client::new()
        .get(http(server, &format!("/api/foto/{id}")))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap();
    (r.status(), r.bytes().await.unwrap().to_vec())
}

#[tokio::test]
async fn de_server_haalt_de_kamer_en_de_fotos_op_en_de_omroep_vervangt_een_foto_door_een_eigen_foto()
 {
    let map = tempfile::tempdir().unwrap();
    let (basis, vragen, _) = nep_api(OP_22_NOV).await;
    let server = start(map.path(), basis.clone()).await;

    let s = wacht_op(&server, |s| s.stand.kamer.kamerleden.len() == 150).await;
    let kamer = &s.stand.kamer;
    assert_eq!(kamer.peildatum.as_deref(), Some("2026-11-22"));
    assert_eq!(kamer.kamerlid(AL_BIYATI).unwrap().foto, Foto::Geen);
    assert_eq!(kamer.kamerlid(EL_ABASSI).unwrap().foto, Foto::OpenData);
    assert_eq!(kamer.zonder_foto().count(), 1);

    // De foto's staan in de map van de server en komen via de server bij Renderer en dashboard.
    assert!(
        map.path()
            .join(format!("fotos/open-data/{EL_ABASSI}.jpg"))
            .exists()
    );
    assert_eq!(
        foto(&server, EL_ABASSI).await,
        (StatusCode::OK, VOORBEELDFOTO.to_vec())
    );
    assert_eq!(foto(&server, AL_BIYATI).await.0, StatusCode::NOT_FOUND);
    assert_eq!(
        foto(&server, "../stand.json").await.0,
        StatusCode::NOT_FOUND
    );

    // Een eigen foto van de omroep, ook voor wie in de open data geen foto heeft.
    let client = reqwest::Client::new();
    for id in [EL_ABASSI, AL_BIYATI] {
        let r = client
            .put(http(&server, &format!("/api/foto/{id}")))
            .bearer_auth(SLEUTEL)
            .header("X-Afzender", "Regie links")
            .body(EIGEN_FOTO)
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        assert!(matches!(
            r.json().await.unwrap(),
            BerichtVanServer::Uitgevoerd { .. }
        ));
    }
    let s = wacht_op(&server, |s| s.stand.kamer.zonder_foto().count() == 0).await;
    assert_eq!(s.stand.kamer.kamerlid(EL_ABASSI).unwrap().foto, Foto::Eigen);
    assert_eq!(
        foto(&server, EL_ABASSI).await,
        (StatusCode::OK, EIGEN_FOTO.to_vec())
    );

    // Geen afbeelding: geweigerd. Onbekend Kamerlid: niet gevonden.
    let geen_beeld = client
        .put(http(&server, &format!("/api/foto/{EL_ABASSI}")))
        .bearer_auth(SLEUTEL)
        .body("geen foto")
        .send()
        .await
        .unwrap();
    assert_eq!(geen_beeld.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let onbekend = client
        .put(http(&server, "/api/foto/0000-onbekend"))
        .bearer_auth(SLEUTEL)
        .body(EIGEN_FOTO)
        .send()
        .await
        .unwrap();
    assert_eq!(onbekend.status(), StatusCode::NOT_FOUND);

    // Eigen foto weghalen: terug naar de open data, of naar voorletters.
    for id in [EL_ABASSI, AL_BIYATI] {
        let r = client
            .delete(http(&server, &format!("/api/foto/{id}")))
            .bearer_auth(SLEUTEL)
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }
    let s = wacht_op(&server, |s| s.stand.kamer.zonder_foto().count() == 1).await;
    assert_eq!(
        s.stand.kamer.kamerlid(EL_ABASSI).unwrap().foto,
        Foto::OpenData
    );
    assert_eq!(
        foto(&server, EL_ABASSI).await,
        (StatusCode::OK, VOORBEELDFOTO.to_vec())
    );

    // Een foto zonder sleutel: geweigerd.
    let zonder = client
        .get(http(&server, &format!("/api/foto/{EL_ABASSI}")))
        .send()
        .await
        .unwrap();
    assert_eq!(zonder.status(), StatusCode::UNAUTHORIZED);
    server.stop().await;

    // Na een herstart staat de Kamer er meteen, ook als de open data even niet bereikbaar is.
    let server = start(map.path(), "http://127.0.0.1:9".into()).await;
    assert_eq!(stand(&server).await.stand.kamer.kamerleden.len(), 150);
    assert_eq!(
        foto(&server, EL_ABASSI).await,
        (StatusCode::OK, VOORBEELDFOTO.to_vec())
    );
    server.stop().await;
    assert_eq!(*vragen.lock().unwrap(), 1);
}

#[tokio::test]
async fn de_server_kijkt_steeds_opnieuw_en_haalt_een_foto_alleen_als_hij_ontbreekt_of_verandert() {
    // De grootte die de open data bij een foto noemt, klopt niet met de foto zelf (zo is het
    // ook bij de echte API). Daarop mag de server een foto dus niet steeds opnieuw ophalen.
    let map = tempfile::tempdir().unwrap();
    let (basis, vragen, fotos) = nep_api(OP_22_NOV).await;
    let server = start_elke(map.path(), basis, Duration::from_millis(100)).await;
    wacht_op(&server, |s| s.stand.kamer.kamerleden.len() == 150).await;
    let tot = tokio::time::Instant::now() + Duration::from_secs(10);
    while *vragen.lock().unwrap() < 4 {
        assert!(
            tokio::time::Instant::now() < tot,
            "server kijkt niet opnieuw"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    // 149 foto's, en Al Biyati (geen foto) is één keer per ronde gevraagd.
    let n = *fotos.lock().unwrap();
    assert!(n < 149 + 10, "{n} fotoverzoeken in vier rondes");
    // Ongewijzigd: geen nieuwe Opdracht, dus het volgnummer blijft staan.
    let s = stand(&server).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(stand(&server).await.volgnummer, s.volgnummer);
    server.stop().await;
}

#[test]
fn elk_kamerlid_krijgt_zijn_plek_op_de_lijst_van_de_fractie_uit_de_open_data() {
    let k = kamer(OP_22_NOV, date(2026, 11, 22));
    assert!(k.kamerleden.iter().all(|l| l.kieslijst.is_some()));
    let el_abassi = k.kamerleden.iter().find(|l| l.id == EL_ABASSI).unwrap();
    assert_eq!(el_abassi.kieslijst, Some(3), "DENK: Van Baarle 1, Ergin 2, El Abassi 3");
}
