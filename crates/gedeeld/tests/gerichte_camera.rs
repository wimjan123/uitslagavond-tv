//! Gerichte camerabewegingen voor Partij uitgelicht en Persoonskaart, en passendheid van
//! Standpunten en Cameraritten per Graphic en Sjabloon.

use gedeeld::camera::{self, CameraProfiel, Partijbeweging, Stoelbeweging};
use gedeeld::cijfers::Invoer;
use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use gedeeld::persoonskaart::{Extra, Wie};
use gedeeld::sjabloon::vast_sjabloon;
use gedeeld::zaalindeling::{Bezetting, Zaalindeling};
use gedeeld::{Graphic, Herkomst, Opdracht, Stand, Stoel, Weigering};

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    fn new() -> Self {
        let mut p = Proef { stand: Stand::default(), volgnummer: 0 };
        let invoer = Invoer::uit_json(
            r##"{
                "partijen": [
                    { "sleutel": "rood", "naam": "Rode Partij", "kort": "ROOD", "kleur": "#d4202c" },
                    { "sleutel": "blauw", "naam": "Blauwe Partij", "kort": "BLAUW", "kleur": "#1e64c8" },
                    { "sleutel": "nul", "naam": "Nul Partij", "kort": "NUL", "kleur": "#888888" }
                ],
                "sets": [
                    { "id": "exitpoll", "soort": "exitpoll", "label": "Exitpoll 21.00", "zetels": { "rood": 40, "blauw": 110, "nul": 0 } }
                ]
            }"##,
        );
        p.doe(Opdracht::LeesInvoer { bestand: "proef.json".into(), ontvangen: "nu".into(), invoer }).unwrap();
        let lid = |id: &str, roepnaam: &str, achternaam: &str| Kamerlid {
            id: id.into(),
            roepnaam: roepnaam.into(),
            initialen: "P.".into(),
            tussenvoegsel: None,
            achternaam: achternaam.into(),
            fractie: "rood-f".into(),
            zetel: format!("z-{id}"),
            fractievoorzitter: false,
            van: "2025-11-12".into(),
            geboortedatum: None,
            geboorteplaats: None,
            woonplaats: None,
            foto: Foto::Geen,
            in_de_kamer_sinds: None,
            kieslijst: None,
        };
        p.doe(Opdracht::LeesKamer {
            opgehaald: "nu".into(),
            peildatum: "2026-11-22".into(),
            kamerleden: vec![lid("sanne", "Sanne", "Vries"), lid("bram", "Bram", "Bes")],
            fracties: vec![Fractie { id: "rood-f".into(), afkorting: "ROOD".into(), naam: "Rode Partij".into(), zetels: 2 }],
        })
        .unwrap();
        let zaalindeling = Zaalindeling {
            kamerperiode: "2025".into(),
            bron: String::new(),
            stoelen: vec![Bezetting { vak: 2, rij: 3, nummer: 1, kamerlid: Some("sanne".into()), naam: "Sanne Vries".into() }],
        };
        p.doe(Opdracht::LeesZaalindeling { bestand: "proef.json".into(), zaalindeling }).unwrap();
        p
    }

    fn doe(&mut self, o: Opdracht) -> Result<(), Weigering> {
        self.volgnummer += 1;
        let h = Herkomst { door: "Proef", volgnummer: self.volgnummer, tijd_ms: 0 };
        self.stand.voer_uit(&o, &h)
    }
}

fn partij_uitgelicht(partij: &str) -> Graphic {
    Graphic::PartijUitgelicht { set: "exitpoll".into(), partij: partij.into(), cijfers: None }
}

fn persoonskaart(id: &str) -> Graphic {
    Graphic::Persoonskaart {
        wie: Wie::Kamerlid { id: id.into() },
        regel: String::new(),
        extra: Extra::default(),
        gegevens: None,
    }
}

#[test]
fn partij_uitgelicht_kan_meerdere_bewegingen_naar_die_partij_kiezen() {
    let mut p = Proef::new();
    for b in Partijbeweging::ALLE {
        p.doe(Opdracht::ZetKlaar { graphic: partij_uitgelicht("rood"), standpunt: Some(b.sleutel().into()) })
            .unwrap();
        let klaar = p.stand.klaar.as_ref().unwrap();
        let (herkend, van, tot) = Partijbeweging::uit_doelsleutel(&klaar.standpunt()).expect("uitgevulde sleutel");
        assert_eq!(herkend, b);
        assert_eq!(tot - van + 1, 40, "precies de 40 zetels van Rood");

        p.doe(Opdracht::Toon { versie: None }).unwrap();
        assert_eq!(p.stand.camera.standpunt, b.doelsleutel(van, tot));
        assert!(p.stand.camerastand_van(&p.stand.camera.standpunt).is_some());
    }
    // ook overzicht A, G en Blijft staan passen bij Partij uitgelicht
    for ok in ["A", "G", camera::BLIJF] {
        p.doe(Opdracht::ZetKlaar { graphic: partij_uitgelicht("blauw"), standpunt: Some(ok.into()) }).unwrap();
    }
    // bij 0 zetels valt een partijbeweging netjes terug op Standpunt A
    p.doe(Opdracht::ZetKlaar { graphic: partij_uitgelicht("nul"), standpunt: Some(camera::PARTIJ_BOOG.into()) })
        .unwrap();
    assert_eq!(p.stand.klaar.as_ref().unwrap().standpunt(), "A");
}

#[test]
fn partij_uitgelicht_weigert_niet_passende_standpunten_en_ritten() {
    let mut p = Proef::new();
    for fout in ["H", "S", "D", "F", "Q", "W", "U", camera::BOVEN_HET_PANEEL, camera::STOEL_LANGS] {
        let res = p.doe(Opdracht::ZetKlaar { graphic: partij_uitgelicht("rood"), standpunt: Some(fout.into()) });
        assert!(matches!(res, Err(Weigering::StandpuntPastNiet { .. })), "{fout} hoort geweigerd te worden: {res:?}");
    }
}

#[test]
fn persoonskaart_met_stoel_kiest_uit_meerdere_stoelbewegingen_en_weigert_de_rest() {
    let mut p = Proef::new();
    let stoel = Stoel { vak: 2, rij: 3, nummer: 1 };
    for b in Stoelbeweging::ALLE {
        p.doe(Opdracht::ZetKlaar { graphic: persoonskaart("sanne"), standpunt: Some(b.sleutel().into()) }).unwrap();
        let klaar = p.stand.klaar.as_ref().unwrap();
        assert_eq!(klaar.standpunt(), b.doelsleutel(stoel));
        p.doe(Opdracht::Toon { versie: None }).unwrap();
        assert_eq!(p.stand.camera.standpunt, b.doelsleutel(stoel));
        assert!(p.stand.camerastand_van(&p.stand.camera.standpunt).is_some());
    }
    // Blijft staan mag altijd
    p.doe(Opdracht::ZetKlaar { graphic: persoonskaart("sanne"), standpunt: Some(camera::BLIJF.into()) }).unwrap();

    // Standpunten die niet naar de Stoel gaan worden geweigerd voor een Kamerlid met Stoel
    for fout in ["A", "S", "D", "G", "H", "Q", "W", camera::BOVEN_HET_PANEEL, camera::AANSCHUIVEN] {
        let res = p.doe(Opdracht::ZetKlaar { graphic: persoonskaart("sanne"), standpunt: Some(fout.into()) });
        assert!(matches!(res, Err(Weigering::StandpuntPastNiet { .. })), "{fout} hoort geweigerd te worden: {res:?}");
    }
}

#[test]
fn persoonskaart_zonder_stoel_staat_alleen_rustige_zaalshots_toe() {
    let mut p = Proef::new();
    for ok in ["S", "D", "F", "A", "W", "T", "E", camera::BLIJF] {
        p.doe(Opdracht::ZetKlaar { graphic: persoonskaart("bram"), standpunt: Some(ok.into()) }).unwrap();
    }
    for fout in ["H", "U", "G", camera::BOVEN_HET_PANEEL, camera::STOEL_LANGS, camera::PARTIJ_BOOG] {
        let res = p.doe(Opdracht::ZetKlaar { graphic: persoonskaart("bram"), standpunt: Some(fout.into()) });
        assert!(matches!(res, Err(Weigering::StandpuntPastNiet { .. })), "{fout} hoort geweigerd te worden: {res:?}");
    }
}

#[test]
fn zetelbalk_zetelverdeling_en_winst_en_verlies_weigeren_niet_passende_standpunten() {
    let mut p = Proef::new();
    let balk = Graphic::Zetelbalk { set: "exitpoll".into(), grootste: None, cijfers: None };
    assert!(p.doe(Opdracht::ZetKlaar { graphic: balk.clone(), standpunt: Some(camera::BOVEN_HET_PANEEL.into()) }).is_ok());
    assert!(p.doe(Opdracht::ZetKlaar { graphic: balk.clone(), standpunt: Some("G".into()) }).is_ok());
    assert!(matches!(
        p.doe(Opdracht::ZetKlaar { graphic: balk.clone(), standpunt: Some("H".into()) }),
        Err(Weigering::StandpuntPastNiet { .. })
    ));
    assert!(matches!(
        p.doe(Opdracht::ZetKlaar { graphic: balk, standpunt: Some("A".into()) }),
        Err(Weigering::StandpuntPastNiet { .. })
    ));

    let verdeling = Graphic::Zetelverdeling {
        set: "exitpoll".into(),
        namen: true,
        zetels: true,
        indeling: gedeeld::Indeling::Uitslagindeling,
        cijfers: None,
    };
    for b in camera::Zaalbeweging::ALLE {
        assert!(
            p.doe(Opdracht::ZetKlaar { graphic: verdeling.clone(), standpunt: Some(b.sleutel().into()) }).is_ok(),
            "{b:?} moet bij Zetelverdeling passen"
        );
    }
    assert!(p.doe(Opdracht::ZetKlaar { graphic: verdeling.clone(), standpunt: Some("Q".into()) }).is_ok());
    assert!(matches!(
        p.doe(Opdracht::ZetKlaar { graphic: verdeling.clone(), standpunt: Some("E".into()) }),
        Err(Weigering::StandpuntPastNiet { .. })
    ));
    assert!(matches!(
        p.doe(Opdracht::ZetKlaar { graphic: verdeling, standpunt: Some("H".into()) }),
        Err(Weigering::StandpuntPastNiet { .. })
    ));

    // Paneelbewegingen passen bij Zetelbalk, maar niet bij Zetelverdeling of Coalitie
    let balk2 = Graphic::Zetelbalk { set: "exitpoll".into(), grootste: None, cijfers: None };
    for b in camera::Paneelbeweging::ALLE {
        assert!(p.doe(Opdracht::ZetKlaar { graphic: balk2.clone(), standpunt: Some(b.sleutel().into()) }).is_ok());
    }
    assert!(matches!(
        p.doe(Opdracht::ZetKlaar { graphic: balk2, standpunt: Some(camera::ZAAL_OPBOUW.into()) }),
        Err(Weigering::StandpuntPastNiet { .. })
    ));

    // Coalitiebewegingen passen bij Coalitie in de Zaal
    let coal = Graphic::Coalitie { set: "exitpoll".into(), partijen: vec!["rood".into()], cijfers: None };
    for b in camera::Coalitiebeweging::ALLE {
        assert!(p.doe(Opdracht::ZetKlaar { graphic: coal.clone(), standpunt: Some(b.sleutel().into()) }).is_ok());
    }
    assert!(matches!(
        p.doe(Opdracht::ZetKlaar { graphic: coal, standpunt: Some(camera::PANEEL_ZWEEF.into()) }),
        Err(Weigering::StandpuntPastNiet { .. })
    ));

    // Sjabloon in Vormgeving controleert ook passendheid
    let mut sj = vast_sjabloon("partij-uitgelicht").unwrap();
    sj.standpunt = Some(camera::PARTIJ_INDALEN.into());
    assert!(p.doe(Opdracht::ZetSjabloon { sjabloon: sj.clone() }).is_ok());
    sj.standpunt = Some("H".into());
    assert!(matches!(p.doe(Opdracht::ZetSjabloon { sjabloon: sj }), Err(Weigering::StandpuntPastNiet { .. })));

    // Persoonskaart-sjabloon mag zowel een stoelbeweging als een rustig Zaal-standpunt kiezen
    let mut psj = vast_sjabloon("persoonskaart").unwrap();
    assert_eq!(psj.camera_profiel(), CameraProfiel::PersoonSjabloon);
    psj.standpunt = Some(camera::STOEL_LANGS.into());
    assert!(p.doe(Opdracht::ZetSjabloon { sjabloon: psj }).is_ok());
    // Voor Bram (zonder Stoel) valt stoel-langs netjes terug op S
    p.doe(Opdracht::ZetKlaar { graphic: persoonskaart("bram"), standpunt: None }).unwrap();
    assert_eq!(p.stand.klaar.as_ref().unwrap().standpunt(), "S");
    // Voor Sanne (met Stoel) wordt stoel-langs ingevuld met haar Stoel
    p.doe(Opdracht::ZetKlaar { graphic: persoonskaart("sanne"), standpunt: None }).unwrap();
    assert_eq!(p.stand.klaar.as_ref().unwrap().standpunt(), "stoel-langs-2-3-1");
}

#[test]
fn kies_standpunt_live_werkt_met_gerichte_bewegingen_op_de_graphic_in_de_ether() {
    let mut p = Proef::new();
    // Niets in beeld -> NietsInBeeld
    assert_eq!(
        p.doe(Opdracht::KiesStandpunt { letter: camera::PARTIJ_BOOG.into(), knip: false }),
        Err(Weigering::NietsInBeeld)
    );
    // Partij uitgelicht in de Ether -> partij-boog richt zich op die Partij, stoel-langs wordt geweigerd
    p.doe(Opdracht::ZetKlaar { graphic: partij_uitgelicht("rood"), standpunt: None }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    p.doe(Opdracht::KiesStandpunt { letter: camera::PARTIJ_BOOG.into(), knip: false }).unwrap();
    assert!(p.stand.camera.standpunt.starts_with("partij-boog-"));
    assert!(matches!(
        p.doe(Opdracht::KiesStandpunt { letter: camera::STOEL_LANGS.into(), knip: false }),
        Err(Weigering::StandpuntPastNiet { .. })
    ));

    // Persoonskaart in de Ether -> stoel-indalen richt zich op haar Stoel
    p.doe(Opdracht::ZetKlaar { graphic: persoonskaart("sanne"), standpunt: None }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    p.doe(Opdracht::KiesStandpunt { letter: camera::STOEL_INDALEN.into(), knip: false }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "stoel-indalen-2-3-1");
}
