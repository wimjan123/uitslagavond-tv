//! Controle na de uitzending: de officiële uitslag uit Debat Direct en uit de `Stemming`-regels
//! van de open data, met vastgelegde antwoorden (`tests/opendata/`, zie de README daar). Geen
//! netwerk: een nep-API op 127.0.0.1 geeft die antwoorden terug. De invoer van de Regie in deze
//! tests is voorbeeldgegevens; de officiële uitslagen zijn echt (19 en 12 november 2026).

use std::time::Duration;

use axum::Router;
use axum::extract::{Path, RawQuery};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use gedeeld::stemmingen::{
    Besluit, Bron, Controlestatus, GetikteFractie, OpgehaaldeUitslag, Stem, Stemgedrag,
};
use gedeeld::{GenummerdeStand, Herkomst, Opdracht, Stand};
use jiff::civil::{Date, date};
use server::controle::{debat_direct, open_data};
use server::{Instellingen, OpenData, Server};

const KAMER: &str = include_str!("opendata/kamerleden-2026-11-22.json");
const KAMER_15_NOV: &str = include_str!("opendata/kamerleden-2026-11-15.json");
const LIJST_19_NOV: &str = include_str!("opendata/stemmingslijst-2026-11-19.json");
const LIJST_12_NOV: &str = include_str!("opendata/stemmingslijst-2026-11-12.json");
const STEMMING_19_NOV: &str = include_str!("opendata/stemming-2026-11-19.json");
const DD_AGENDA_19_NOV: &str = include_str!("opendata/debatdirect-agenda-2026-11-19.json");
const DD_DEBAT_19_NOV: &str = include_str!("opendata/debatdirect-debat-2026-11-19.json");
const DD_DEBAT_12_NOV: &str = include_str!("opendata/debatdirect-debat-2026-11-12.json");
const DD_ACTORS: &str = include_str!("opendata/debatdirect-actors-2026-11-22.json");
const SLEUTEL: &str = "proefsleutel";

// Zaken op de lijst van donderdag 19 november (`Zaak.Id`).
const WILDERS: &str = "50f703f7-dff2-463c-a4e9-308e6d11d3b0"; // 36848, nr. 113
const PRICKAERTZ: &str = "73809005-7e0b-476b-8241-0fb9b3c68a95"; // 23645, nr. 911
const GOUDZWAARD: &str = "a8845029-36bd-4470-a1f2-adb37d596c30"; // gewijzigd, was nr. 912
const KROGER: &str = "eba11117-946e-4174-a1fd-73d283bf3e9c"; // 21501-32, nr. 1856
const VAN_DER_PLAS: &str = "a6eeb36b-2714-42c9-bd68-bd2aaaa544ab"; // 21501-32, nr. 1855, aangehouden

// Fracties (`Fractie.Id` in de open data).
const PVV: &str = "65129918-f256-4975-9da4-488da34d6695";
const SP: &str = "a3689bb6-3914-4d5c-a6a8-42e24582e299";
const FVD: &str = "e0b7b638-de3c-47cc-85bd-341dd65ea33d";
const JA21: &str = "fab6e7e1-9d63-446a-a6e1-e1c74e3f679e";
const CDA: &str = "62c1a13c-85ff-40ed-90f7-a9546d61f869";

/// Een Stand met de Kamer en de Stemmingslijst van `dag`.
fn stand_met(kamer: &str, peildatum: Date, lijst: &str, dag: Date) -> Stand {
    let h = Herkomst {
        door: "Proef",
        volgnummer: 1,
        tijd_ms: 0,
    };
    let personen = server::opendata::lees_pagina(kamer).unwrap().personen;
    let k = server::opendata::kamer_op(&personen, peildatum);
    let mut s = Stand::default();
    s.voer_uit(
        &Opdracht::LeesKamer {
            opgehaald: format!("{peildatum}T08:00:00+01:00"),
            peildatum: peildatum.to_string(),
            kamerleden: k.kamerleden,
            fracties: k.fracties,
        },
        &h,
    )
    .unwrap();
    let activiteiten = server::stemmingslijst::lees_pagina(lijst)
        .unwrap()
        .activiteiten;
    let punten = server::stemmingslijst::punten_op(&activiteiten, dag);
    s.voer_uit(
        &Opdracht::LeesStemmingslijst {
            dag: dag.to_string(),
            ingelezen: "2026-11-22T08:00:00+01:00".into(),
            punten,
        },
        &h,
    )
    .unwrap();
    s
}

fn stand_19_nov() -> Stand {
    stand_met(KAMER, date(2026, 11, 22), LIJST_19_NOV, date(2026, 11, 19))
}

fn uit_debat_direct(stand: &Stand, debat: &str) -> Vec<OpgehaaldeUitslag> {
    let debat = debat_direct::lees_debat(debat).unwrap();
    let actors = debat_direct::lees_actors(DD_ACTORS).unwrap();
    debat_direct::uitslagen(&[debat], &actors, &stand.stemmingen.lijst, &stand.kamer)
}

fn uit_open_data(stand: &Stand) -> Vec<OpgehaaldeUitslag> {
    let pagina = open_data::lees_pagina(STEMMING_19_NOV).unwrap();
    assert!(pagina.volgende.is_none());
    open_data::uitslagen(&pagina.activiteiten, &stand.stemmingen.lijst, &stand.kamer)
}

fn van<'a>(u: &'a [OpgehaaldeUitslag], stemming: &str) -> &'a OpgehaaldeUitslag {
    u.iter()
        .find(|u| u.stemming == stemming)
        .unwrap_or_else(|| panic!("geen uitslag voor {stemming}"))
}

fn voor(u: &OpgehaaldeUitslag) -> Vec<&str> {
    let mut v: Vec<&str> = u
        .uitslag
        .fracties
        .iter()
        .filter(|f| f.stem == Stem::Voor)
        .map(|f| f.fractie.as_str())
        .collect();
    v.sort();
    v
}

#[test]
fn debat_direct_geeft_de_stemmingen_van_19_november_per_fractie() {
    let stand = stand_19_nov();
    let u = uit_debat_direct(&stand, DD_DEBAT_19_NOV);

    // Zes rondes: vijf op nummer, de gewijzigde motie van Goudzwaard ("nr. ??, was nr. 912") op
    // de tekst na "over". De aangehouden motie van Van der Plas heeft geen ronde.
    let mut ids: Vec<&str> = u.iter().map(|u| u.stemming.as_str()).collect();
    ids.sort();
    let mut verwacht = vec![
        WILDERS,
        PRICKAERTZ,
        GOUDZWAARD,
        KROGER,
        "0e6421c0-a03c-4994-9679-fcd466c00e1a",
        "6d2f0407-cae5-47bc-9ae6-23b0b10d1e33",
    ];
    verwacht.sort();
    assert_eq!(ids, verwacht);

    let w = van(&u, WILDERS);
    assert_eq!(w.uitslag.bron, Bron::DebatDirect);
    assert_eq!(w.uitslag.besluit, Besluit::Verworpen);
    assert!(!w.uitslag.hoofdelijk);
    assert_eq!(
        w.uitslag.fracties.len(),
        17,
        "elke Fractie van Debat Direct hoort bij een Fractie van de Kamer"
    );
    let mut pvv_sp_fvd = vec![PVV, SP, FVD];
    pvv_sp_fvd.sort();
    assert_eq!(voor(w), pvv_sp_fvd);
    assert_eq!(
        w.uitslag.hamerslag.as_deref(),
        Some("2026-11-19T13:34:52+01:00")
    );
    assert!(
        u.iter()
            .all(|u| !u.uitslag.fracties.is_empty() && u.uitslag.afwijkend.is_empty())
    );
}

#[test]
fn de_open_data_geeft_dezelfde_stemmen_als_debat_direct() {
    let stand = stand_19_nov();
    let od = uit_open_data(&stand);
    let dd = uit_debat_direct(&stand, DD_DEBAT_19_NOV);
    assert_eq!(od.len(), 6, "de aangehouden motie heeft geen Stemming");
    // In deze regels heeft 50PLUS een ander Fractie-id dan in de Kamer; de naam vindt hem toch.
    for u in &od {
        assert_eq!(u.uitslag.bron, Bron::OpenData);
        assert_eq!(u.uitslag.hamerslag, None);
        let d = van(&dd, &u.stemming);
        assert_eq!(u.uitslag.besluit, d.uitslag.besluit, "{}", u.stemming);
        let gesorteerd = |f: &[GetikteFractie]| {
            let mut f: Vec<(String, Stem)> =
                f.iter().map(|g| (g.fractie.clone(), g.stem)).collect();
            f.sort();
            f
        };
        assert_eq!(
            gesorteerd(&u.uitslag.fracties),
            gesorteerd(&d.uitslag.fracties),
            "{}",
            u.stemming
        );
    }
}

#[test]
fn een_hoofdelijke_stemming_in_debat_direct_telt_per_kamerlid() {
    // Donderdag 12 november: het Belastingplan 2027, hoofdelijk, aangenomen.
    let stand = stand_met(
        KAMER_15_NOV,
        date(2026, 11, 15),
        LIJST_12_NOV,
        date(2026, 11, 12),
    );
    let u = uit_debat_direct(&stand, DD_DEBAT_12_NOV);
    let bp = u
        .iter()
        .find(|u| {
            stand
                .stemmingen
                .stemming(&u.stemming)
                .unwrap()
                .punt
                .nummer
                .as_deref()
                == Some("37022")
        })
        .expect("het wetsvoorstel 37022 komt op titel uit Debat Direct");
    assert!(bp.uitslag.hoofdelijk);
    assert_eq!(bp.uitslag.besluit, Besluit::Aangenomen);
    // De uitkomst uit de stemmen per Kamerlid klopt met de Zetels van de Kamer.
    let mut s = stand.clone();
    let id = bp.stemming.clone();
    s.voer_uit(
        &Opdracht::LeesOfficieleUitslagen {
            opgehaald: "2026-11-12T23:30:00+01:00".into(),
            uitslagen: u.clone(),
        },
        &Herkomst {
            door: "Proef",
            volgnummer: 2,
            tijd_ms: 0,
        },
    )
    .unwrap();
    let o = s
        .stemmingen
        .stemming(&id)
        .unwrap()
        .controle
        .officiele_uitkomst
        .clone()
        .unwrap();
    assert_eq!(o.voor + o.tegen + o.open, 150);
    assert!(o.voor > o.tegen, "aangenomen: {o:?}");
    assert_eq!(o.besluit, Some(Besluit::Aangenomen));
}

// --- De server haalt het zelf op ---

/// Een nep-API voor de open data en Debat Direct, met de antwoorden van 19 november.
async fn nep_api() -> String {
    let app = Router::new()
        .route(
            "/Persoon",
            get(|| async { ([(header::CONTENT_TYPE, "application/json")], KAMER) }),
        )
        .route(
            "/Activiteit",
            get(|RawQuery(q): RawQuery| async move {
                let q = q.unwrap_or_default();
                let antwoord = if q.contains("Besluit") {
                    STEMMING_19_NOV
                } else {
                    LIJST_19_NOV
                };
                ([(header::CONTENT_TYPE, "application/json")], antwoord)
            }),
        )
        .route(
            "/dd/actors",
            get(|| async { ([(header::CONTENT_TYPE, "application/json")], DD_ACTORS) }),
        )
        .route(
            "/dd/agenda/{dag}",
            get(|Path(dag): Path<String>| async move {
                if dag == "2026-11-19" {
                    (
                        [(header::CONTENT_TYPE, "application/json")],
                        DD_AGENDA_19_NOV,
                    )
                        .into_response()
                } else {
                    StatusCode::NOT_FOUND.into_response()
                }
            }),
        )
        .route(
            "/dd/agenda/{dag}/debates/{id}",
            get(|Path((dag, id)): Path<(String, String)>| async move {
                if dag == "2026-11-19" && id == "1e7725ce-08dd-46d8-8105-08c4a50e2e67" {
                    (
                        [(header::CONTENT_TYPE, "application/json")],
                        DD_DEBAT_19_NOV,
                    )
                        .into_response()
                } else {
                    // Een ander debat van die dag: geen Stemmingen.
                    (
                        [(header::CONTENT_TYPE, "application/json")],
                        r#"{"votings":[]}"#,
                    )
                        .into_response()
                }
            }),
        );
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

async fn opdracht(server: &Server, o: &Opdracht) {
    let r = reqwest::Client::new()
        .post(format!("http://{}/api/opdracht", server.adres()))
        .bearer_auth(SLEUTEL)
        .json(o)
        .send()
        .await
        .unwrap();
    assert!(
        r.status().is_success(),
        "{o:?}: {}",
        r.text().await.unwrap()
    );
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

fn status(s: &GenummerdeStand, stemming: &str) -> Option<Controlestatus> {
    s.stand
        .stemmingen
        .stemming(stemming)
        .unwrap()
        .controle
        .status
}

/// Voorbeeldinvoer: `voor` getikt voor, de rest "Overige tegen".
fn invoer(stemming: &str, voor: &[impl AsRef<str>]) -> Opdracht {
    Opdracht::ZetStemgedrag {
        stemming: stemming.into(),
        stemgedrag: Stemgedrag {
            getikt: voor
                .iter()
                .map(|f| GetikteFractie {
                    fractie: f.as_ref().into(),
                    stem: Stem::Voor,
                })
                .collect(),
            overige: Some(Stem::Tegen),
            ..Default::default()
        },
    }
}

#[tokio::test]
async fn de_server_haalt_de_officiele_uitslag_op_en_zet_hem_naast_de_invoer() {
    let map = tempfile::tempdir().unwrap();
    let basis = nep_api().await;
    let server = Server::start(Instellingen {
        adres: "127.0.0.1:0".parse().unwrap(),
        map: map.path().to_owned(),
        sleutel: SLEUTEL.into(),
        dashboard: None,
        invoer: None,
        zaalindeling: None,
        open_data: Some(OpenData {
            basis: basis.clone(),
            elke: Duration::from_secs(3600),
            peildatum: Some(date(2026, 11, 19)),
            debat_direct: Some(format!("{basis}/dd")),
        }),
    })
    .await
    .expect("server start");
    wacht_op(&server, |s| {
        s.stand.stemmingen.lijst.len() == 7 && s.stand.kamer.fracties.len() == 17
    })
    .await;

    // Voorbeeldinvoer van de Regie, zoals tijdens de uitzending.
    // Wilders: PVV, SP en FVD voor, de rest tegen. Klopt.
    opdracht(&server, &invoer(WILDERS, &[PVV, SP, FVD])).await;
    // Prickaertz (echt verworpen): per ongeluk ook het CDA voor, dus aangenomen.
    let mut fout = officieel_voor(PRICKAERTZ);
    fout.push(CDA.into());
    opdracht(&server, &invoer(PRICKAERTZ, &fout)).await;
    // Kröger (echt aangenomen): JA21 omgedraaid, uitkomst blijft gelijk.
    let mut anders = officieel_voor(KROGER);
    match anders.iter().position(|f| f == JA21) {
        Some(i) => drop(anders.remove(i)),
        None => anders.push(JA21.into()),
    }
    opdracht(&server, &invoer(KROGER, &anders)).await;
    // Van der Plas: aangehouden, nooit een officiële uitslag.
    opdracht(&server, &invoer(VAN_DER_PLAS, &[PVV])).await;

    let s = wacht_op(&server, |s| {
        status(s, WILDERS) == Some(Controlestatus::Gelijk)
    })
    .await;
    assert_eq!(status(&s, PRICKAERTZ), Some(Controlestatus::UitkomstAnders));
    assert_eq!(status(&s, KROGER), Some(Controlestatus::VerschilPerFractie));
    assert_eq!(
        status(&s, VAN_DER_PLAS),
        Some(Controlestatus::WachtOpOfficieel)
    );
    assert_eq!(status(&s, GOUDZWAARD), None, "niet ingevoerd");

    // De open data gaat voor Debat Direct; de server weet nog wanneer Debat Direct de uitslag had.
    let c = &s.stand.stemmingen.stemming(WILDERS).unwrap().controle;
    let o = c.officieel.as_ref().unwrap();
    assert_eq!(o.bron, Bron::OpenData);
    assert_eq!(o.hamerslag.as_deref(), Some("2026-11-19T13:34:52+01:00"));
    assert!(c.debat_direct_gezien.is_some());
    assert!(c.open_data_gezien.is_some());

    // De invoer bleef staan.
    let p = s.stand.stemmingen.stemming(PRICKAERTZ).unwrap();
    assert_eq!(p.uitkomst.besluit, Some(Besluit::Aangenomen));

    opdracht(
        &server,
        &Opdracht::NeemOfficieelOver {
            stemming: PRICKAERTZ.into(),
        },
    )
    .await;
    let s = wacht_op(&server, |s| {
        status(s, PRICKAERTZ) == Some(Controlestatus::Gelijk)
    })
    .await;
    assert_eq!(
        s.stand
            .stemmingen
            .stemming(PRICKAERTZ)
            .unwrap()
            .uitkomst
            .besluit,
        Some(Besluit::Verworpen)
    );
    server.stop().await;
}

/// Welke Fracties officieel voor stemden, volgens de vastgelegde `Stemming`-regels.
fn officieel_voor(stemming: &str) -> Vec<String> {
    let u = uit_open_data(&stand_19_nov());
    voor(van(&u, stemming))
        .into_iter()
        .map(str::to_owned)
        .collect()
}
