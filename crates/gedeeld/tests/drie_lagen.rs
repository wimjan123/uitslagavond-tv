//! Tests voor de drie onafhankelijke lagen (Camera, Stoelbeeld, Graphic), Overvloeien (Fade),
//! en uitgebreid Preset-beheer (ADR 0005).

use gedeeld::camera::{OvergangSoort, BLIJF, OVERVLOEI_MAX_MS, OVERVLOEI_MIN_MS};
use gedeeld::cijfers::Invoer;
use gedeeld::sjabloon::{Kleurovergang, Overgangsvorm};
use gedeeld::stoelbeeld::{Stoelbeeld, Stoelbron};
use gedeeld::{Graphic, Herkomst, Indeling, Kleur, Opdracht, Stand, Weigering, AANTAL_STOELEN};

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    fn nieuw() -> Self {
        Self { stand: Stand::default(), volgnummer: 0 }
    }

    fn doe(&mut self, o: Opdracht) -> Result<(), Weigering> {
        self.volgnummer += 1;
        let h = Herkomst { door: "Regie", volgnummer: self.volgnummer, tijd_ms: self.volgnummer * 1000 };
        self.stand.voer_uit(&o, &h)
    }

    fn lees_invoer(&mut self) {
        let json = r##"{
            "partijen": [
                { "sleutel": "alfa", "naam": "Alfa", "kort": "ALFA", "kleur": "#1f6fb2" },
                { "sleutel": "beta", "naam": "Beta", "kort": "BETA", "kleur": "#d33b2c" }
            ],
            "sets": [
                { "id": "exitpoll", "soort": "peiling", "label": "Exitpoll", "zetels": { "alfa": 80, "beta": 70 } }
            ]
        }"##;
        self.doe(Opdracht::LeesInvoer {
            bestand: "cijfers.json".into(),
            ontvangen: "21.00".into(),
            invoer: Invoer::uit_json(json),
        })
        .unwrap();
    }
}

#[test]
fn camera_klaarzetten_en_toon_start_camera_zonder_live_graphic_weg_te_halen() {
    let mut p = Proef::nieuw();
    // Eerst een Graphic live zetten op Standpunt A
    p.doe(Opdracht::ZetKlaar {
        graphic: Graphic::Titelkaart { kop: "Welkom".into(), regel: String::new() },
        standpunt: Some("A".into()),
    })
    .unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert!(p.stand.ether.is_some());
    assert_eq!(p.stand.camera.standpunt, "A");

    // Zet alleen een Camerarit klaar in Preview (Q is een vaste Camerarit)
    p.doe(Opdracht::ZetCameraKlaar { letter: Some("Q".into()) }).unwrap();
    assert_eq!(p.stand.camera.klaar_standpunt.as_deref(), Some("Q"));
    // Live camera staat nog op A tot Take
    assert_eq!(p.stand.camera.standpunt, "A");

    // Take met Fade (Overvloeien)
    p.doe(Opdracht::ZetOvergang { overgang: Some(OvergangSoort::Overvloeien), overvloei_ms: Some(800) }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();

    assert_eq!(p.stand.camera.standpunt, "Q", "Camerarit start op Live bij Take");
    assert!(p.stand.camera.knip, "Overvloeien knipt de camera op het beginpunt");
    assert!(p.stand.camera.overvloei_sinds > 0, "Fade-moment is vastgelegd");
    assert_eq!(p.stand.camera.overvloei_ms, 800);
    assert!(p.stand.camera.klaar_standpunt.is_none(), "klaargezette camera is verbruikt");
    assert!(p.stand.ether.is_some(), "live Graphic bleef ongemoeid in beeld");
}

#[test]
fn graphic_over_lopende_camerarit_laat_camera_lopen_met_blijf() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::KiesStandpunt { letter: "Q".into(), knip: false }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "Q");
    let sinds = p.stand.camera.sinds;

    p.doe(Opdracht::ZetKlaar {
        graphic: Graphic::Titelkaart { kop: "Over rit".into(), regel: String::new() },
        standpunt: Some(BLIJF.into()),
    })
    .unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "Q");
    assert_eq!(p.stand.camera.sinds, sinds, "Camerarit werd niet onderbroken");
    assert!(p.stand.ether.is_some());
}

#[test]
fn stoelbeeld_alleen_kleurt_stoelen_zonder_graphic_en_met_eigen_overgang_en_kracht() {
    let mut p = Proef::nieuw();
    p.lees_invoer();

    let sb = Stoelbeeld {
        bron: Stoelbron::Partijen {
            set: "exitpoll".into(),
            partijen: vec!["alfa".into()],
        },
        overgang: Some(Kleurovergang { vorm: Overgangsvorm::Vervagen, duur_ms: 900 }),
        kleurkracht: Some(85),
        label: Some("Alfa uitgelicht".into()),
    };
    p.doe(Opdracht::ZetStoelbeeldKlaar { stoelbeeld: Some(sb.clone()) }).unwrap();
    assert_eq!(p.stand.klaar_stoelkleuren.len(), AANTAL_STOELEN);
    assert!(p.stand.stoelkleuren.is_empty(), "nog niet live tot Take");

    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert!(p.stand.ether.is_none(), "geen 2D Graphic nodig voor Stoelbeeld");
    assert_eq!(p.stand.stoelkleuren.len(), AANTAL_STOELEN);
    assert_eq!(p.stand.kleurovergang.vorm, Overgangsvorm::Vervagen);
    assert_eq!(p.stand.ether_kleurkracht, Some(85));
    assert_eq!(p.stand.ether_stoelbeeld, Some(sb));

    // Direct losse stoelen kleuren
    let mut kleuren = vec![Kleur { r: 76, g: 79, b: 85 }; AANTAL_STOELEN];
    kleuren[0] = Kleur { r: 240, g: 180, b: 76 };
    let los = Stoelbeeld {
        bron: Stoelbron::Los { kleuren },
        overgang: None,
        kleurkracht: Some(100),
        label: Some("Losse stoel".into()),
    };
    p.doe(Opdracht::ZetStoelbeeldDirect { stoelbeeld: los }).unwrap();
    assert_eq!(p.stand.stoelkleuren.len(), AANTAL_STOELEN);
    assert_eq!(p.stand.ether_kleurkracht, Some(100));

    // AllesUit haalt stoelen en graphic samen weg
    p.doe(Opdracht::AllesUit).unwrap();
    assert!(p.stand.stoelkleuren.is_empty());
    assert!(p.stand.ether_stoelbeeld.is_none());
    assert!(p.stand.ether_kleurkracht.is_none());
}

#[test]
fn uitgebreide_preset_met_stoelbeeld_bewerken_en_verplaatsen() {
    let mut p = Proef::nieuw();
    p.lees_invoer();

    let sb = Stoelbeeld {
        bron: Stoelbron::Set { set: "exitpoll".into(), indeling: Indeling::Uitslagindeling },
        overgang: Some(Kleurovergang::default()),
        kleurkracht: Some(70),
        label: Some("Exitpoll Zaal".into()),
    };
    // Preset 1: alleen Stoelbeeld + Standpunt D + Fade
    p.doe(Opdracht::BewaarPresetUitgebreid {
        naam: Some("Exitpoll Zaal".into()),
        graphic: None,
        standpunt: Some("D".into()),
        stoelbeeld: Some(sb.clone()),
        overgang: Some(OvergangSoort::Overvloeien),
        overvloei_ms: Some(500),
    })
    .unwrap();

    // Preset 2: gewone Titelkaart
    p.doe(Opdracht::BewaarPresetUitgebreid {
        naam: Some("Opening".into()),
        graphic: Some(Graphic::Titelkaart { kop: "Uitslagavond".into(), regel: String::new() }),
        standpunt: Some("A".into()),
        stoelbeeld: None,
        overgang: None,
        overvloei_ms: None,
    })
    .unwrap();

    assert_eq!(p.stand.presets.len(), 2);
    assert_eq!(p.stand.presets[0].nummer, 1);
    assert!(p.stand.presets[0].zonder_graphic);
    assert_eq!(p.stand.presets[0].naam.as_deref(), Some("Exitpoll Zaal"));

    // Verplaats Preset 2 naar positie 1
    p.doe(Opdracht::VerplaatsPreset { nummer: 2, naar: 1 }).unwrap();
    assert_eq!(p.stand.presets[0].naam.as_deref(), Some("Opening"));
    assert_eq!(p.stand.presets[0].nummer, 1);
    assert_eq!(p.stand.presets[1].naam.as_deref(), Some("Exitpoll Zaal"));
    assert_eq!(p.stand.presets[1].nummer, 2);

    // Preset 2 (Stoelbeeld + Standpunt D + Fade) klaarzetten en tonen
    p.doe(Opdracht::ZetPresetKlaar { nummer: 2 }).unwrap();
    assert!(p.stand.klaar.is_none());
    assert_eq!(p.stand.klaar_stoelbeeld, Some(sb));
    assert_eq!(p.stand.camera.klaar_standpunt.as_deref(), Some("D"));
    assert_eq!(p.stand.camera.overgang, Some(OvergangSoort::Overvloeien));
    assert_eq!(p.stand.camera.overvloei_ms, 500);

    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "D");
    assert_eq!(p.stand.stoelkleuren.len(), AANTAL_STOELEN);

    // BewerkPreset past naam en overvloei-duur aan (met clamp)
    p.doe(Opdracht::BewerkPreset {
        nummer: 2,
        naam: Some("Exitpoll Breed".into()),
        graphic: None,
        zonder_graphic: true,
        standpunt: Some("F".into()),
        stoelbeeld: None,
        overgang: Some(OvergangSoort::Overvloeien),
        overvloei_ms: Some(9999),
    })
    .unwrap();
    assert_eq!(p.stand.presets[1].naam.as_deref(), Some("Exitpoll Breed"));
    assert_eq!(p.stand.presets[1].overvloei_ms, Some(OVERVLOEI_MAX_MS));

    // Clamp ondergrens
    p.doe(Opdracht::ZetOvergang { overgang: Some(OvergangSoort::Overvloeien), overvloei_ms: Some(10) }).unwrap();
    assert_eq!(p.stand.camera.overvloei_ms, OVERVLOEI_MIN_MS);
}
