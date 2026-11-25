//! De server van buiten bekeken: via WebSocket en HTTP, zoals dashboard, knoppen en Renderers hem gebruiken.

use std::path::Path;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use gedeeld::cijfers::{Bewerking, Soort};
use gedeeld::volger::{Gebeurtenis, Volger, volg, volg_als};
use gedeeld::{
    Afzender, BerichtAanServer, BerichtVanServer, GenummerdeStand, Graphic, Opdracht, RendererOpMachine,
    Rol, Stand,
};
use server::{Instellingen, Server};
use tokio_tungstenite::tungstenite::{self, Message};

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

/// Wacht (zonder de runtime te blokkeren) op de eerste Stand van de volger die aan `past` voldoet.
async fn wacht_op_stand(volger: &Volger, past: impl Fn(&GenummerdeStand) -> bool) -> GenummerdeStand {
    let tot = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        while let Ok(g) = volger.gebeurtenissen.try_recv() {
            if let Gebeurtenis::Stand(s) = g {
                if past(&s) {
                    return s;
                }
            }
        }
        assert!(tokio::time::Instant::now() < tot, "geen passende Stand binnen 10 s");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Verbindt zoals het dashboard: sleutel in de URL, dan Hallo als Werkplek.
async fn dashboard(server: &Server, naam: &str) -> Ws {
    let url = format!("ws://{}/ws?sleutel={SLEUTEL}", server.adres());
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.expect("verbinden");
    stuur(&mut ws, &BerichtAanServer::Hallo {
        afzender: Afzender::Werkplek { naam: naam.into() },
    })
    .await;
    ws
}

async fn stuur(ws: &mut Ws, b: &BerichtAanServer) {
    ws.send(Message::text(serde_json::to_string(b).unwrap())).await.unwrap();
}

/// Het volgende tekstbericht, zonder de camera.
async fn ontvang(ws: &mut Ws) -> BerichtVanServer {
    loop {
        let m = tokio::time::timeout(Duration::from_secs(10), ws.next())
            .await
            .expect("bericht binnen 10 s")
            .expect("verbinding open")
            .expect("geldig bericht");
        if let Message::Text(t) = m {
            // de camera komt 10 keer per seconde; deze proeven kijken naar de rest
            match serde_json::from_str(&t).expect("bekend bericht") {
                BerichtVanServer::Camera(_) => continue,
                b => return b,
            }
        }
    }
}

/// Een Titelkaart klaarzetten (voorbeeldgegevens).
fn zet_klaar(kop: &str) -> Opdracht {
    Opdracht::ZetKlaar {
        graphic: Graphic::Titelkaart { kop: kop.into(), regel: "Voorbeeldgegevens".into() },
        standpunt: None,
    }
}

/// Het volgende tekstbericht dat geen Stand of Renderers-melding is: het antwoord op een Opdracht.
async fn antwoord(ws: &mut Ws) -> BerichtVanServer {
    loop {
        match ontvang(ws).await {
            BerichtVanServer::Stand(_)
            | BerichtVanServer::Renderers { .. }
            | BerichtVanServer::Bewerkingen { .. } => {}
            b => return b,
        }
    }
}

/// De volgende Stand met minstens dit volgnummer.
async fn stand_vanaf(ws: &mut Ws, volgnummer: u64) -> GenummerdeStand {
    loop {
        if let BerichtVanServer::Stand(s) = ontvang(ws).await {
            if s.volgnummer >= volgnummer {
                return s;
            }
        }
    }
}

fn http() -> reqwest::Client {
    reqwest::Client::new()
}

#[tokio::test]
async fn opdracht_uit_dashboard_geeft_renderer_nieuwe_stand_met_hoger_volgnummer() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let renderer = volg(&server.adres().to_string(), SLEUTEL, Rol::Ether);
    let eerste = wacht_op_stand(&renderer, |_| true).await;

    let mut ws = dashboard(&server, "Regie links").await;
    assert!(matches!(ontvang(&mut ws).await, BerichtVanServer::Stand(_)));
    stuur(&mut ws, &BerichtAanServer::Opdracht { opdracht: zet_klaar("Goedenavond, Den Haag") })
        .await;
    let BerichtVanServer::Uitgevoerd { volgnummer } = antwoord(&mut ws).await else {
        panic!("Opdracht niet uitgevoerd");
    };

    let nieuw = wacht_op_stand(&renderer, |s| s.volgnummer > eerste.volgnummer).await;
    let klaar = nieuw.stand.klaar.expect("Graphic klaar");
    assert_eq!(klaar.klaargezet_door, "Regie links");
    assert_eq!(nieuw.volgnummer, volgnummer);
}

#[tokio::test]
async fn server_weigert_verbinding_zonder_of_met_verkeerde_sleutel() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let a = server.adres();

    for url in [format!("ws://{a}/ws"), format!("ws://{a}/ws?sleutel=fout")] {
        match tokio_tungstenite::connect_async(&url).await {
            Err(tungstenite::Error::Http(r)) => assert_eq!(r.status(), 401, "{url}"),
            anders => panic!("{url} niet geweigerd: {anders:?}"),
        }
    }

    let opdracht = zet_klaar("zonder sleutel");
    let r = http().post(format!("http://{a}/api/opdracht")).json(&opdracht).send().await.unwrap();
    assert_eq!(r.status(), 401);
    let r = http().get(format!("http://{a}/api/stand")).bearer_auth("fout").send().await.unwrap();
    assert_eq!(r.status(), 401);

    let stand: GenummerdeStand = http()
        .get(format!("http://{a}/api/stand"))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stand.stand, Stand::default());
}

#[tokio::test]
async fn server_start_niet_zonder_sleutel() {
    let map = tempfile::tempdir().unwrap();
    let r = Server::start(Instellingen {
        adres: "127.0.0.1:0".parse().unwrap(),
        map: map.path().to_owned(),
        sleutel: String::new(),
        dashboard: None,
        invoer: None,
        zaalindeling: None,
        open_data: None,
    })
    .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn stand_is_er_nog_na_herstart() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let opdracht = zet_klaar("Na de herstart");
    let r = http()
        .post(format!("http://{}/api/opdracht", server.adres()))
        .bearer_auth(SLEUTEL)
        .json(&opdracht)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    server.stop().await;

    let server = start(map.path()).await;
    let stand: GenummerdeStand = http()
        .get(format!("http://{}/api/stand", server.adres()))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let klaar = stand.stand.klaar.expect("Graphic klaar na herstart");
    assert_eq!(klaar.graphic, Graphic::Titelkaart { kop: "Na de herstart".into(), regel: "Voorbeeldgegevens".into() });
    assert_eq!(stand.volgnummer, 1);
}

#[tokio::test]
async fn logboek_bewaart_tijd_afzender_en_opdracht() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let opdracht = zet_klaar("Uit de Stream Deck");
    http()
        .post(format!("http://{}/api/opdracht", server.adres()))
        .bearer_auth(SLEUTEL)
        .header("x-afzender", "Stream Deck")
        .json(&opdracht)
        .send()
        .await
        .unwrap();
    let mut ws = dashboard(&server, "Regie rechts").await;
    ontvang(&mut ws).await;
    stuur(&mut ws, &BerichtAanServer::Opdracht { opdracht: zet_klaar(&"x".repeat(gedeeld::MAX_KOP + 1)) })
        .await;
    assert!(matches!(antwoord(&mut ws).await, BerichtVanServer::Geweigerd { .. }));
    server.stop().await;

    let logboek = std::fs::read_to_string(map.path().join("logboek.jsonl")).unwrap();
    let regels: Vec<serde_json::Value> =
        logboek.lines().map(|r| serde_json::from_str(r).unwrap()).collect();
    assert_eq!(regels.len(), 2);

    let knop = &regels[0];
    assert_eq!(knop["afzender"], serde_json::json!({ "soort": "Knop", "naam": "Stream Deck" }));
    assert_eq!(knop["opdracht"], serde_json::to_value(&opdracht).unwrap());
    assert_eq!(knop["volgnummer"], 1);
    let tijd: jiff::Timestamp = knop["tijd"].as_str().unwrap().parse().unwrap();
    assert!(jiff::Timestamp::now().duration_since(tijd).as_secs().abs() < 60);

    let geweigerd = &regels[1];
    assert_eq!(geweigerd["afzender"], serde_json::json!({ "soort": "Werkplek", "naam": "Regie rechts" }));
    assert!(geweigerd["geweigerd"].is_string());
}

#[tokio::test]
async fn server_weigert_renderer_met_andere_versie() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let url = format!("ws://{}/ws?sleutel={SLEUTEL}", server.adres());
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    stuur(&mut ws, &BerichtAanServer::Hallo {
        afzender: Afzender::Renderer {
            rol: Rol::Reserve,
            versie: "0.0.0-anders".into(),
            machine: "studio-reserve".into(),
        },
    })
    .await;
    assert!(matches!(ontvang(&mut ws).await, BerichtVanServer::Geweigerd { .. }));
}

#[tokio::test]
async fn toon_weigert_na_wijziging_door_een_andere_werkplek_en_slaagt_daarna() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let ether = volg(&server.adres().to_string(), SLEUTEL, Rol::Ether);
    let mut regie = dashboard(&server, "Regie").await;
    let mut rechts = dashboard(&server, "Regie rechts").await;

    stuur(&mut regie, &BerichtAanServer::Opdracht { opdracht: zet_klaar("De Uitslagavond") }).await;
    let BerichtVanServer::Uitgevoerd { volgnummer } = antwoord(&mut regie).await else {
        panic!("klaarzetten mislukt");
    };
    let gezien = stand_vanaf(&mut regie, volgnummer).await.stand.klaar.unwrap();

    let wijziging = Opdracht::WijzigKlaar {
        id: gezien.id,
        graphic: Graphic::Titelkaart { kop: "Uitslagavond 2025".into(), regel: "Voorbeeldgegevens".into() },
        standpunt: None,
    };
    stuur(&mut rechts, &BerichtAanServer::Opdracht { opdracht: wijziging }).await;
    let BerichtVanServer::Uitgevoerd { volgnummer: _ } = antwoord(&mut rechts).await else {
        panic!("wijzigen mislukt");
    };

    // De Regie drukt op Toon met de versie die zijn scherm liet zien: die is al veranderd.
    stuur(&mut regie, &BerichtAanServer::Opdracht { opdracht: Opdracht::Toon { versie: Some(gezien.versie) } })
        .await;
    assert_eq!(antwoord(&mut regie).await, BerichtVanServer::Geweigerd { reden: "gewijzigd door Regie rechts".into() });

    // Zijn scherm toont nu de nieuwe versie; daarmee slaagt Toon.
    let nu: GenummerdeStand = http()
        .get(format!("http://{}/api/stand", server.adres()))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let nu = nu.stand.klaar.unwrap();
    assert_eq!(nu.gewijzigd_door, "Regie rechts");
    stuur(&mut regie, &BerichtAanServer::Opdracht { opdracht: Opdracht::Toon { versie: Some(nu.versie) } })
        .await;
    let BerichtVanServer::Uitgevoerd { volgnummer } = antwoord(&mut regie).await else {
        panic!("Toon mislukt na de nieuwe versie");
    };

    let in_de_ether = wacht_op_stand(&ether, |s| s.volgnummer >= volgnummer).await;
    let graphic = in_de_ether.stand.ether.expect("Graphic in de Ether");
    assert_eq!(graphic.graphic, nu.graphic);
    assert_eq!(graphic.getoond_door, "Regie");
}

/// De Stand zoals de server hem nu heeft, via HTTP.
async fn huidige_stand(server: &Server) -> Stand {
    let s: GenummerdeStand = http()
        .get(format!("http://{}/api/stand", server.adres()))
        .bearer_auth(SLEUTEL)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    s.stand
}

/// Verbindt zoals de tablet van de Presentator op /tablet.
async fn tablet(server: &Server) -> Ws {
    let url = format!("ws://{}/ws?sleutel={SLEUTEL}", server.adres());
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.expect("verbinden");
    stuur(&mut ws, &BerichtAanServer::Hallo { afzender: Afzender::Tablet { naam: "Tablet".into() } }).await;
    ws
}

#[tokio::test]
async fn de_tablet_zet_alleen_klaar_en_toon_van_de_regie_weigert_daarna_met_gewijzigd_door_tablet() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let mut regie = dashboard(&server, "Regie").await;
    let mut tablet = tablet(&server).await;

    // De Regie bewaart een Preset, zet hem op de tablet en zet zelf iets anders klaar (voorbeeldgegevens).
    let preset = Opdracht::BewaarPreset {
        graphic: Graphic::Titelkaart { kop: "Van de tablet".into(), regel: "Voorbeeldgegevens".into() },
        standpunt: None,
    };
    for opdracht in [preset, Opdracht::ZetPresetOpTablet { nummer: 1, aan: true }, zet_klaar("De Uitslagavond")] {
        stuur(&mut regie, &BerichtAanServer::Opdracht { opdracht }).await;
        assert!(matches!(antwoord(&mut regie).await, BerichtVanServer::Uitgevoerd { .. }), "Regie geweigerd");
    }
    let gezien = huidige_stand(&server).await.klaar.unwrap();

    // De tablet kan niet tonen, alleen klaarzetten.
    stuur(&mut tablet, &BerichtAanServer::Opdracht { opdracht: Opdracht::Toon { versie: Some(gezien.versie) } }).await;
    assert_eq!(
        antwoord(&mut tablet).await,
        BerichtVanServer::Geweigerd { reden: "de tablet zet alleen klaar: de Regie zet het in de Ether".into() }
    );
    stuur(&mut tablet, &BerichtAanServer::Opdracht { opdracht: Opdracht::ZetPresetKlaar { nummer: 1 } }).await;
    assert!(matches!(antwoord(&mut tablet).await, BerichtVanServer::Uitgevoerd { .. }), "tablet kon niet klaarzetten");

    // De Regie drukt op Toon met wat zijn scherm liet zien: dat is niet meer wat klaarstaat.
    stuur(&mut regie, &BerichtAanServer::Opdracht { opdracht: Opdracht::Toon { versie: Some(gezien.versie) } }).await;
    assert_eq!(antwoord(&mut regie).await, BerichtVanServer::Geweigerd { reden: "gewijzigd door Tablet".into() });

    let nu = huidige_stand(&server).await;
    assert!(nu.ether.is_none(), "niets in de Ether");
    let klaar = nu.klaar.unwrap();
    assert_eq!(klaar.klaargezet_door, "Tablet");
    stuur(&mut regie, &BerichtAanServer::Opdracht { opdracht: Opdracht::Toon { versie: Some(klaar.versie) } }).await;
    assert!(matches!(antwoord(&mut regie).await, BerichtVanServer::Uitgevoerd { .. }), "met de nieuwe versie toont de Regie");

    // Het logboek noemt de tablet als afzender, ook van de weigering.
    let logboek = std::fs::read_to_string(map.path().join("logboek.jsonl")).unwrap();
    assert!(logboek.lines().any(|r| r.contains(r#""soort":"Tablet""#) && r.contains("de tablet zet alleen klaar")), "{logboek}");
}

/// Verbindt zoals de Renderer met rol Voorbeeld.
async fn voorbeeld_renderer(server: &Server) -> Ws {
    let url = format!("ws://{}/ws?sleutel={SLEUTEL}", server.adres());
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    stuur(&mut ws, &BerichtAanServer::Hallo {
        afzender: Afzender::Renderer {
            rol: Rol::Voorbeeld,
            versie: gedeeld::VERSIE.into(),
            machine: "regie-pc".into(),
        },
    })
    .await;
    ws
}

/// Het volgende binaire bericht: een klein beeld.
async fn beeld(ws: &mut Ws) -> Vec<u8> {
    loop {
        let m = tokio::time::timeout(Duration::from_secs(10), ws.next())
            .await
            .expect("beeld binnen 10 s")
            .expect("verbinding open")
            .expect("geldig bericht");
        if let Message::Binary(b) = m {
            return b.to_vec();
        }
    }
}

#[tokio::test]
async fn kleine_beelden_van_het_voorbeeld_gaan_naar_elke_werkplek() {
    use gedeeld::Monitor;
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let mut regie = dashboard(&server, "Regie").await;
    let mut voorbeeld = voorbeeld_renderer(&server).await;
    ontvang(&mut voorbeeld).await;

    let mut ether_beeld = vec![Monitor::Ether.byte()];
    ether_beeld.extend_from_slice(b"\xff\xd8 een JPEG");
    voorbeeld.send(Message::binary(ether_beeld.clone())).await.unwrap();
    assert_eq!(beeld(&mut regie).await, ether_beeld);

    // Een Werkplek die later opent, krijgt meteen het laatste beeld.
    let mut later = dashboard(&server, "Invoer").await;
    assert_eq!(beeld(&mut later).await, ether_beeld);

    // Een Werkplek kan geen beelden sturen.
    regie.send(Message::binary(vec![Monitor::Voorbeeld.byte(), 1, 2, 3])).await.unwrap();
    assert!(matches!(antwoord(&mut regie).await, BerichtVanServer::Geweigerd { .. }));

    // Valt het Voorbeeld weg, dan krijgt elke Werkplek "geen beeld" (alleen de byte).
    drop(voorbeeld);
    let mut leeg = Vec::new();
    while leeg.len() < 2 {
        leeg.push(beeld(&mut later).await);
    }
    leeg.sort();
    assert_eq!(leeg, vec![vec![Monitor::Voorbeeld.byte()], vec![Monitor::Ether.byte()]]);
}

/// De volgende melding over de Renderers die aan `past` voldoet.
async fn renderers(
    ws: &mut Ws,
    past: impl Fn(&[RendererOpMachine], &[RendererOpMachine]) -> bool,
) -> (Vec<RendererOpMachine>, Vec<RendererOpMachine>) {
    loop {
        if let BerichtVanServer::Renderers { verbonden, geweigerd, .. } = ontvang(ws).await {
            if past(&verbonden, &geweigerd) {
                return (verbonden, geweigerd);
            }
        }
    }
}

#[tokio::test]
async fn werkplek_ziet_welke_renderers_verbonden_zijn_met_machine_en_versie() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let mut regie = dashboard(&server, "Regie").await;

    renderers(&mut regie, |v, _| v.is_empty()).await;
    let ether = volg_als(&server.adres().to_string(), SLEUTEL, Rol::Ether, gedeeld::VERSIE, "studio-ether");
    let (verbonden, _) = renderers(&mut regie, |v, _| !v.is_empty()).await;
    assert_eq!(
        verbonden,
        vec![RendererOpMachine { rol: Rol::Ether, machine: "studio-ether".into(), versie: gedeeld::VERSIE.into() }]
    );
    drop(ether);
    renderers(&mut regie, |v, _| v.is_empty()).await;
}

#[tokio::test]
async fn werkplek_ziet_welke_machine_met_een_andere_versie_geweigerd_is() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let mut regie = dashboard(&server, "Regie").await;
    let BerichtVanServer::Renderers { versie, .. } = antwoord_of_renderers(&mut regie).await else {
        unreachable!()
    };
    assert_eq!(versie, gedeeld::VERSIE);

    let oud = volg_als(&server.adres().to_string(), SLEUTEL, Rol::Reserve, "0.0.9+oud", "studio-reserve");
    let reden = wacht_op(&oud, |g| match g {
        Gebeurtenis::Geweigerd(r) => Some(r.clone()),
        _ => None,
    })
    .await;
    assert!(reden.contains("0.0.9+oud") && reden.contains(gedeeld::VERSIE), "{reden}");
    let (verbonden, geweigerd) = renderers(&mut regie, |_, g| !g.is_empty()).await;
    assert!(verbonden.is_empty(), "een geweigerde Renderer telt niet als verbonden");
    assert_eq!(
        geweigerd,
        vec![RendererOpMachine { rol: Rol::Reserve, machine: "studio-reserve".into(), versie: "0.0.9+oud".into() }]
    );
    // de Renderer blijft het proberen; zolang hij dat doet blijft hij in de lijst, ook na de
    // bewaartijd van een weigering
    tokio::time::sleep(server::WEIGERING_BLIJFT + Duration::from_secs(1)).await;
    let mut later = dashboard(&server, "Later").await;
    let (_, geweigerd) = renderers(&mut later, |_, _| true).await;
    assert_eq!(geweigerd.len(), 1, "een Renderer die het blijft proberen staat er nog");

    // Staat hij uit, dan verdwijnt hij na de bewaartijd uit de lijst.
    drop(oud);
    renderers(&mut regie, |_, g| g.is_empty()).await;
}

/// De eerste melding over de Renderers.
async fn antwoord_of_renderers(ws: &mut Ws) -> BerichtVanServer {
    loop {
        if let b @ BerichtVanServer::Renderers { .. } = ontvang(ws).await {
            return b;
        }
    }
}

/// Wacht (zonder de runtime te blokkeren) op een gebeurtenis van de volger waar `past` iets uit haalt.
async fn wacht_op<T>(volger: &Volger, past: impl Fn(&Gebeurtenis) -> Option<T>) -> T {
    let tot = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        while let Ok(g) = volger.gebeurtenissen.try_recv() {
            if let Some(t) = past(&g) {
                return t;
            }
        }
        assert!(tokio::time::Instant::now() < tot, "geen passende gebeurtenis binnen 10 s");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn start_op(map: &Path, adres: std::net::SocketAddr) -> Server {
    Server::start(Instellingen {
        adres,
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

/// Zet een Titelkaart in de Ether (voorbeeldgegevens) en geeft het volgnummer.
async fn toon_titelkaart(server: &Server, kop: &str) -> u64 {
    let mut regie = dashboard(server, "Regie").await;
    voer_uit(&mut regie, zet_klaar(kop)).await;
    voer_uit(&mut regie, Opdracht::Toon { versie: None }).await
}

#[tokio::test]
async fn renderer_houdt_de_stand_vast_als_de_server_wegvalt_en_verbindt_weer_als_hij_terugkomt() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let adres = server.adres();
    let ether = volg(&adres.to_string(), SLEUTEL, Rol::Ether);
    let volgnummer = toon_titelkaart(&server, "Voor de storing").await;
    let voor = wacht_op_stand(&ether, |s| s.volgnummer >= volgnummer).await;

    server.stop().await;
    wacht_op(&ether, |g| matches!(g, Gebeurtenis::Verbroken(_)).then_some(())).await;

    // De server komt terug op hetzelfde adres en leest de Stand van schijf; de Renderer verbindt
    // uit zichzelf opnieuw en krijgt dezelfde Stand, dus er verandert niets in beeld.
    let server = start_op(map.path(), adres).await;
    wacht_op(&ether, |g| matches!(g, Gebeurtenis::Verbonden).then_some(())).await;
    let na = wacht_op_stand(&ether, |_| true).await;
    assert_eq!(na, voor);
    assert!(na.stand.ether.is_some());

    // En hij volgt weer: een nieuwe wijziging komt binnen.
    let mut regie = dashboard(&server, "Regie").await;
    let nieuw = voer_uit(&mut regie, Opdracht::UitBeeld).await;
    let s = wacht_op_stand(&ether, |s| s.volgnummer >= nieuw).await;
    assert!(s.stand.ether.is_none());
}

#[tokio::test]
async fn een_herstarte_renderer_krijgt_meteen_de_hele_stand_met_de_graphic_in_de_ether() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let adres = server.adres().to_string();
    let ether = volg(&adres, SLEUTEL, Rol::Ether);
    let volgnummer = toon_titelkaart(&server, "Al in beeld").await;
    let bij_de_ether = wacht_op_stand(&ether, |s| s.volgnummer >= volgnummer).await;

    // De Reserve start halverwege: de eerste Stand die hij krijgt is meteen de hele Stand.
    let reserve = volg(&adres, SLEUTEL, Rol::Reserve);
    let eerste = wacht_op(&reserve, |g| match g {
        Gebeurtenis::Stand(s) => Some(s.clone()),
        _ => None,
    })
    .await;
    assert_eq!(eerste, bij_de_ether);
    assert!(eerste.stand.ether.is_some());
}

/// Verzamelt een tijd lang de camerabeelden van een volger, op tiknummer.
fn camerabeelden(volger: &Volger, in_: &mut std::collections::BTreeMap<u64, gedeeld::camera::Camerabeeld>) {
    while let Ok(g) = volger.gebeurtenissen.try_recv() {
        if let Gebeurtenis::Camera { beeld: c, .. } = g {
            in_.insert(c.tik, c);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn twee_renderers_krijgen_dezelfde_reeks_camerastanden() {
    use std::collections::BTreeMap;
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let adres = server.adres().to_string();
    let ether = volg(&adres, SLEUTEL, Rol::Ether);
    let reserve = volg(&adres, SLEUTEL, Rol::Reserve);
    let begin = wacht_op_stand(&ether, |_| true).await;
    wacht_op_stand(&reserve, |_| true).await;

    // de Regie stuurt de camera van Totaal (A) naar Bovenaf (G): een overgang van 2 s
    let antwoord = http()
        .post(format!("http://{adres}/api/opdracht"))
        .bearer_auth(SLEUTEL)
        .json(&Opdracht::KiesStandpunt { letter: "G".into(), knip: false })
        .send()
        .await
        .unwrap();
    assert!(antwoord.status().is_success());

    let (mut e, mut r) = (BTreeMap::new(), BTreeMap::new());
    let tot = tokio::time::Instant::now() + Duration::from_secs(3);
    while tokio::time::Instant::now() < tot {
        camerabeelden(&ether, &mut e);
        camerabeelden(&reserve, &mut r);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    for (naam, reeks) in [("Ether", &e), ("Reserve", &r)] {
        let (eerste, laatste) = (*reeks.keys().next().unwrap(), *reeks.keys().last().unwrap());
        assert_eq!(laatste - eerste + 1, reeks.len() as u64, "{naam} mist tikken");
        assert!(reeks.len() >= 250, "{naam}: {} beelden in 3 s", reeks.len());
    }
    let samen: Vec<u64> = e.keys().filter(|t| r.contains_key(t)).copied().collect();
    assert!(samen.len() >= 250, "{} gedeelde tikken", samen.len());
    for t in &samen {
        assert_eq!(e[t], r[t], "tik {t} verschilt tussen Ether en Reserve");
    }
    // de reeks laat de overgang zien: van A naar G
    let a = begin.stand.standpunt("A").unwrap().stand;
    let g = begin.stand.standpunt("G").unwrap().stand;
    let posities: Vec<[f32; 3]> = samen.iter().map(|t| e[t].ether.stand.positie).collect();
    assert_eq!(posities.first(), Some(&a.positie));
    assert_eq!(e[samen.last().unwrap()].ether.stand, g);
    let tussen = posities.iter().filter(|p| **p != a.positie && **p != g.positie).count();
    assert!((150..=210).contains(&tussen), "{tussen} tikken onderweg, verwacht ongeveer 200");
}

/// Voert een Opdracht uit vanaf deze Werkplek en geeft het volgnummer.
async fn voer_uit(ws: &mut Ws, opdracht: Opdracht) -> u64 {
    stuur(ws, &BerichtAanServer::Opdracht { opdracht }).await;
    match antwoord(ws).await {
        BerichtVanServer::Uitgevoerd { volgnummer } => volgnummer,
        b => panic!("niet uitgevoerd: {b:?}"),
    }
}

/// De volgende melding over welke sets de andere Werkplekken open hebben, die aan `past` voldoet.
async fn bewerkingen(ws: &mut Ws, past: impl Fn(&[Bewerking]) -> bool) -> Vec<Bewerking> {
    loop {
        if let BerichtVanServer::Bewerkingen { open } = ontvang(ws).await {
            if past(&open) {
                return open;
            }
        }
    }
}

fn zetels(ws_stand: &GenummerdeStand, partij: &str) -> u32 {
    ws_stand.stand.cijfers.set("exitpoll").unwrap().zetels_van(partij)
}

#[tokio::test]
async fn twee_werkplekken_bewerken_dezelfde_set_per_vakje_telt_de_laatste_wijziging() {
    let map = tempfile::tempdir().unwrap();
    let server = start(map.path()).await;
    let mut regie = dashboard(&server, "Regie").await;
    let mut invoer = dashboard(&server, "Invoer").await;

    // Voorbeeldgegevens: drie Partijen en een lege Exitpoll.
    for (naam, kort) in [("Alfa Partij", "ALFA"), ("Beta Unie", "BETA"), ("Gamma Lijst", "GAM")] {
        let o = Opdracht::VoegPartijToe { naam: naam.into(), kort: kort.into(), kleur: None, logo: None, fractie_zonder_lijst: false };
        voer_uit(&mut regie, o).await;
    }
    voer_uit(&mut regie, Opdracht::MaakSet { setsoort: Soort::Exitpoll, label: "Exitpoll".into() }).await;

    // Allebei openen ze de Exitpoll. Elk scherm ziet dat de ander hem open heeft; niets gaat op slot.
    stuur(&mut regie, &BerichtAanServer::Bewerk { set: Some("exitpoll".into()) }).await;
    stuur(&mut invoer, &BerichtAanServer::Bewerk { set: Some("exitpoll".into()) }).await;
    let bij_regie = bewerkingen(&mut regie, |b| !b.is_empty()).await;
    assert_eq!(bij_regie, [Bewerking { werkplek: "Invoer".into(), set: "exitpoll".into() }]);
    let bij_invoer = bewerkingen(&mut invoer, |b| !b.is_empty()).await;
    assert_eq!(bij_invoer, [Bewerking { werkplek: "Regie".into(), set: "exitpoll".into() }]);

    // Verschillende vakjes: allebei gaan door.
    let zet = |partij: &str, zetels: u16| Opdracht::ZetZetels { set: "exitpoll".into(), partij: partij.into(), zetels };
    voer_uit(&mut regie, zet("alfa", 30)).await;
    voer_uit(&mut invoer, zet("beta", 20)).await;
    // Hetzelfde vakje: de laatste wijziging telt.
    voer_uit(&mut regie, zet("gam", 10)).await;
    let laatste = voer_uit(&mut invoer, zet("gam", 12)).await;

    for ws in [&mut regie, &mut invoer] {
        let s = stand_vanaf(ws, laatste).await;
        assert_eq!((zetels(&s, "alfa"), zetels(&s, "beta"), zetels(&s, "gam")), (30, 20, 12));
    }

    // Invoer sluit de set (of valt weg): de Regie ziet niemand meer bewerken.
    stuur(&mut invoer, &BerichtAanServer::Bewerk { set: None }).await;
    bewerkingen(&mut regie, |b| b.is_empty()).await;
    stuur(&mut invoer, &BerichtAanServer::Bewerk { set: Some("exitpoll".into()) }).await;
    bewerkingen(&mut regie, |b| !b.is_empty()).await;
    drop(invoer);
    bewerkingen(&mut regie, |b| b.is_empty()).await;
}
