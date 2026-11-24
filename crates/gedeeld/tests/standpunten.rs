//! Eigen Standpunten en de Vrije camera in de Stand (#44).

use gedeeld::camera::Camerastand;
use gedeeld::{Graphic, Herkomst, Opdracht, Stand, Weigering};

fn doe(stand: &mut Stand, volgnummer: &mut u64, o: Opdracht) -> Result<(), Weigering> {
    *volgnummer += 1;
    let h = Herkomst { door: "Proef", volgnummer: *volgnummer, tijd_ms: 0 };
    stand.voer_uit(&o, &h)
}

fn plek(x: f32) -> Camerastand {
    Camerastand::uit_graden([x, 2.5, -4.0], 10.0, -8.0, 50.0)
}

fn bewaar(naam: &str, x: f32) -> Opdracht {
    Opdracht::BewaarStandpunt { naam: naam.into(), stand: Some(plek(x)) }
}

fn titelkaart() -> Graphic {
    Graphic::Titelkaart { kop: "Voorbeeldgegevens".into(), regel: String::new() }
}

/// De toetsen van de eigen Standpunten, in volgorde van de lijst.
fn toetsen(stand: &Stand) -> Vec<Option<&str>> {
    stand.standpunten.iter().filter(|s| !s.vast).map(|s| s.toets()).collect()
}

#[test]
fn eigen_standpunten_zonder_maximum_de_eerste_drie_op_j_k_en_l() {
    let mut stand = Stand::default();
    let mut v = 0;
    for (i, naam) in ["Een", "Twee", "Drie", "Vier", "Vijf"].iter().enumerate() {
        doe(&mut stand, &mut v, bewaar(naam, i as f32)).unwrap();
    }
    assert_eq!(stand.standpunten.len(), 11);
    assert_eq!(toetsen(&stand), [Some("J"), Some("K"), Some("L"), None, None]);
    let vier = stand.standpunten.iter().find(|s| s.naam == "Vier").unwrap();
    assert_eq!(vier.stand, plek(3.0));
    assert!(!vier.vast);
    // elke sleutel is anders, ook zonder toets
    let mut letters: Vec<&str> = stand.standpunten.iter().map(|s| s.letter.as_str()).collect();
    letters.sort();
    letters.dedup();
    assert_eq!(letters.len(), 11);
    // de vaste houden hun eigen letter als toets
    assert_eq!(stand.standpunt("A").unwrap().toets(), Some("A"));
}

#[test]
fn een_vrijgekomen_toets_gaat_naar_het_volgende_nieuwe_standpunt() {
    let mut stand = Stand::default();
    let mut v = 0;
    for naam in ["Een", "Twee", "Drie", "Vier"] {
        doe(&mut stand, &mut v, bewaar(naam, 0.0)).unwrap();
    }
    let twee = stand.standpunten.iter().find(|s| s.naam == "Twee").unwrap().letter.clone();
    doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: twee }).unwrap();
    doe(&mut stand, &mut v, bewaar("Vijf", 0.0)).unwrap();
    let vijf = stand.standpunten.iter().find(|s| s.naam == "Vijf").unwrap();
    assert_eq!(vijf.toets(), Some("K"));
}

#[test]
fn een_toets_verplaatsen_haalt_hem_bij_een_ander_weg() {
    let mut stand = Stand::default();
    let mut v = 0;
    for naam in ["Een", "Twee", "Drie", "Vier"] {
        doe(&mut stand, &mut v, bewaar(naam, 0.0)).unwrap();
    }
    let vier = stand.standpunten.iter().find(|s| s.naam == "Vier").unwrap().letter.clone();
    doe(&mut stand, &mut v, Opdracht::ZetStandpuntToets { letter: vier.clone(), toets: Some("J".into()) })
        .unwrap();
    assert_eq!(toetsen(&stand), [None, Some("K"), Some("L"), Some("J")]);
    // geen toets mag ook, maar alleen J, K en L bestaan voor eigen Standpunten
    doe(&mut stand, &mut v, Opdracht::ZetStandpuntToets { letter: vier.clone(), toets: None }).unwrap();
    assert_eq!(toetsen(&stand), [None, Some("K"), Some("L"), None]);
    let fout = doe(&mut stand, &mut v, Opdracht::ZetStandpuntToets { letter: vier, toets: Some("A".into()) });
    assert_eq!(fout, Err(Weigering::OnbekendeToets("A".into())));
    // een vast Standpunt houdt zijn letter
    let fout = doe(&mut stand, &mut v, Opdracht::ZetStandpuntToets { letter: "A".into(), toets: None });
    assert_eq!(fout, Err(Weigering::VastStandpunt("A".into())));
}

#[test]
fn naam_geven_en_overschrijven() {
    let mut stand = Stand::default();
    let mut v = 0;
    doe(&mut stand, &mut v, bewaar("Oud", 1.0)).unwrap();
    let j = stand.standpunten[6].letter.clone();
    doe(&mut stand, &mut v, Opdracht::HernoemStandpunt { letter: j.clone(), naam: "  Nieuw ".into() }).unwrap();
    assert_eq!(stand.standpunt(&j).unwrap().naam, "Nieuw");
    doe(&mut stand, &mut v, Opdracht::OverschrijfStandpunt { letter: j.clone(), stand: Some(plek(2.0)) })
        .unwrap();
    assert_eq!(stand.standpunt(&j).unwrap().stand, plek(2.0));
    // een vast Standpunt mag je opnieuw opslaan, maar niet hernoemen of wissen
    doe(&mut stand, &mut v, Opdracht::OverschrijfStandpunt { letter: "G".into(), stand: Some(plek(3.0)) })
        .unwrap();
    assert_eq!(stand.standpunt("G").unwrap().stand, plek(3.0));
    assert!(stand.standpunt("G").unwrap().vast);
    let fout = doe(&mut stand, &mut v, Opdracht::HernoemStandpunt { letter: "G".into(), naam: "X".into() });
    assert_eq!(fout, Err(Weigering::VastStandpunt("G".into())));
    let fout = doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: "G".into() });
    assert_eq!(fout, Err(Weigering::VastStandpunt("G".into())));
    let fout = doe(&mut stand, &mut v, Opdracht::HernoemStandpunt { letter: j, naam: " ".into() });
    assert_eq!(fout, Err(Weigering::StandpuntZonderNaam));
}

#[test]
fn zonder_camerastand_of_met_een_kapotte_wordt_bewaren_geweigerd() {
    let mut stand = Stand::default();
    let mut v = 0;
    let fout = doe(&mut stand, &mut v, Opdracht::BewaarStandpunt { naam: "X".into(), stand: None });
    assert_eq!(fout, Err(Weigering::GeenCamerastand));
    let mut kapot = plek(0.0);
    kapot.helling = f32::NAN;
    let fout = doe(&mut stand, &mut v, Opdracht::BewaarStandpunt { naam: "X".into(), stand: Some(kapot) });
    assert_eq!(fout, Err(Weigering::GeenCamerastand));
    assert_eq!(stand.standpunten.len(), 6);
}

#[test]
fn wissen_kan_niet_zolang_een_graphic_het_standpunt_gebruikt_en_de_weigering_zegt_waar() {
    let mut stand = Stand::default();
    let mut v = 0;
    doe(&mut stand, &mut v, bewaar("Close-up", 1.0)).unwrap();
    let j = stand.standpunten[6].letter.clone();
    doe(&mut stand, &mut v, Opdracht::ZetKlaar { graphic: titelkaart(), standpunt: Some(j.clone()) })
        .unwrap();
    let fout = doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: j.clone() }).unwrap_err();
    assert_eq!(fout.to_string(), "Standpunt J (Close-up) wordt gebruikt door de klaargezette Graphic Titelkaart \"Voorbeeldgegevens\"");

    // in de Ether, en de camera van de Ether staat er nu ook op
    doe(&mut stand, &mut v, Opdracht::Toon { versie: None }).unwrap();
    let fout = doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: j.clone() }).unwrap_err();
    let tekst = fout.to_string();
    assert!(tekst.contains("de Graphic in de Ether"), "{tekst}");
    assert!(tekst.contains("de camera van de Ether"), "{tekst}");

    // pas als niets het meer gebruikt, gaat het
    doe(&mut stand, &mut v, Opdracht::ZetKlaar { graphic: titelkaart(), standpunt: None }).unwrap();
    doe(&mut stand, &mut v, Opdracht::UitBeeld).unwrap();
    assert_eq!(
        stand.gebruik_van_standpunt(&j).iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["de camera van de Ether"]
    );
    doe(&mut stand, &mut v, Opdracht::KiesStandpunt { letter: "A".into(), knip: false }).unwrap();
    doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: j.clone() }).unwrap();
    assert!(stand.standpunt(&j).is_none());
    assert_eq!(stand.standpunten.len(), 6);
}

#[test]
fn een_standpunt_kiezen_zet_de_vrije_camera_uit() {
    let mut stand = Stand::default();
    let mut v = 0;
    doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: true }).unwrap();
    assert!(stand.camera.vrij);
    let sinds = stand.camera.sinds;
    // aan- en uitzetten is geen nieuwe keuze voor een Standpunt
    doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: false }).unwrap();
    assert!(!stand.camera.vrij);
    assert_eq!(stand.camera.sinds, sinds);
    doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: true }).unwrap();
    doe(&mut stand, &mut v, Opdracht::KiesStandpunt { letter: "G".into(), knip: false }).unwrap();
    assert!(!stand.camera.vrij);
    assert_eq!(stand.camera.standpunt, "G");
}

#[test]
fn een_oude_stand_zonder_vrije_camera_en_toetsen_laadt_nog() {
    let json = r#"{"camera":{"standpunt":"A","knip":true,"sinds":0},
        "standpunten":[{"letter":"A","naam":"Totaal vanaf voorzitter","vast":true,
        "stand":{"positie":[0,4.5,4.45],"draai":0,"helling":-0.4,"beeldhoek":1.7}}]}"#;
    let s: Stand = serde_json::from_str(json).unwrap();
    assert!(!s.camera.vrij);
    assert_eq!(s.standpunt("A").unwrap().toets(), Some("A"));
}

#[test]
fn wissen_kan_niet_zolang_een_preset_het_standpunt_gebruikt() {
    let mut stand = Stand::default();
    let mut v = 0;
    doe(&mut stand, &mut v, bewaar("Close-up", 1.0)).unwrap();
    let j = stand.standpunten[6].letter.clone();
    doe(&mut stand, &mut v, Opdracht::BewaarPreset { graphic: titelkaart(), standpunt: Some(j.clone()) })
        .unwrap();
    let fout = doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: j.clone() }).unwrap_err();
    assert_eq!(fout.to_string(), "Standpunt J (Close-up) wordt gebruikt door Preset 1 Titelkaart \"Voorbeeldgegevens\"");
    doe(&mut stand, &mut v, Opdracht::WisPreset { nummer: 1 }).unwrap();
    doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: j }).unwrap();
}
