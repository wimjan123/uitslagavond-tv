//! Terug in beeld: Uit beeld ongedaan maken. De Graphic komt terug met de cijfers die erin
//! zaten, wat klaarstaat blijft gelijk, en de camera beweegt zoals bij Toon.

use gedeeld::camera;
use gedeeld::cijfers::Invoer;
use gedeeld::{Graphic, Herkomst, Indeling, Opdracht, Stand, Weigering};

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    /// VOORBEELDGEGEVENS: twee verzonnen Partijen en een set.
    fn new() -> Self {
        let mut p = Proef { stand: Stand::default(), volgnummer: 0 };
        let invoer = Invoer::uit_json(
            r##"{
            "partijen": [
                { "sleutel": "rood", "naam": "Rode Partij", "kort": "ROOD", "kleur": "#d4202c" },
                { "sleutel": "blauw", "naam": "Blauwe Partij", "kort": "BLAUW", "kleur": "#12357a" }
            ],
            "sets": [
                { "id": "prognose", "soort": "prognose", "label": "Prognose 21.00",
                  "zetels": { "rood": 80, "blauw": 70 } }
            ]
        }"##,
        );
        p.doe(Opdracht::LeesInvoer { bestand: "proef.json".into(), ontvangen: "nu".into(), invoer }).unwrap();
        p
    }

    fn doe(&mut self, opdracht: Opdracht) -> Result<(), Weigering> {
        let mut nieuw = self.stand.clone();
        let h = Herkomst { door: "Regie 1", volgnummer: self.volgnummer + 1, tijd_ms: (self.volgnummer + 1) * 1_000 };
        nieuw.voer_uit(&opdracht, &h)?;
        self.stand = nieuw;
        self.volgnummer += 1;
        Ok(())
    }

    /// Uit beeld en terug, met wat er in de Ether stond.
    fn terug(&mut self, ether: &gedeeld::GraphicInBeeld) -> Result<(), Weigering> {
        self.doe(Opdracht::TerugInBeeld { graphic: ether.graphic.clone(), standpunt: ether.standpunt.clone() })
    }
}

fn zetelverdeling() -> Graphic {
    Graphic::Zetelverdeling { set: "prognose".into(), namen: true, zetels: true, indeling: Indeling::Uitslagindeling, cijfers: None }
}

fn titelkaart(kop: &str) -> Graphic {
    Graphic::Titelkaart { kop: kop.into(), regel: String::new() }
}

#[test]
fn terug_in_beeld_zet_de_oude_cijfers_terug_en_laat_klaar_staan() {
    let mut p = Proef::new();
    p.doe(Opdracht::ZetKlaar { graphic: zetelverdeling(), standpunt: None }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    let oud = p.stand.ether.clone().unwrap();
    let kleuren = p.stand.stoelkleuren.clone();
    p.doe(Opdracht::UitBeeld).unwrap();

    // Intussen krijgt de set nieuwere cijfers, en de Regie zet iets anders klaar.
    p.doe(Opdracht::ZetZetels { set: "prognose".into(), partij: "rood".into(), zetels: 75 }).unwrap();
    p.doe(Opdracht::ZetKlaar { graphic: titelkaart("Straks"), standpunt: None }).unwrap();
    p.doe(Opdracht::KleurUit).unwrap();
    let klaar = p.stand.klaar.clone();

    p.terug(&oud).unwrap();
    let ether = p.stand.ether.clone().unwrap();
    assert_eq!(ether.graphic, oud.graphic, "precies de Graphic van toen");
    assert_eq!(ether.graphic.cijferset(), Some(("prognose", 1)), "met de oude cijfers");
    assert!(p.stand.nieuwere_cijfers(&ether.graphic));
    assert_eq!(p.stand.stoelkleuren, kleuren, "de Stoelen krijgen de kleuren van toen");
    assert_eq!(p.stand.klaar, klaar, "wat klaarstaat blijft gelijk");
    assert_eq!((ether.id, ether.versie, ether.getoond), (p.volgnummer, p.volgnummer, p.volgnummer));
    assert_eq!(ether.getoond_door, "Regie 1");
    assert!(!p.stand.kan_bijwerken(), "wat klaarstaat is een andere Graphic");
}

#[test]
fn terug_in_beeld_weigert_als_er_al_iets_in_beeld_staat() {
    let mut p = Proef::new();
    p.doe(Opdracht::ZetKlaar { graphic: titelkaart("Eerste"), standpunt: None }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    let voor = p.stand.clone();
    let w = p.doe(Opdracht::TerugInBeeld { graphic: titelkaart("Oud"), standpunt: None }).unwrap_err();
    assert_eq!(w, Weigering::EtherNietLeeg);
    assert_eq!(w.to_string(), "er staat al een Graphic in beeld: haal die eerst uit beeld");
    assert_eq!(p.stand, voor);
}

#[test]
fn terug_in_beeld_controleert_graphic_en_standpunt() {
    let mut p = Proef::new();
    assert_eq!(p.doe(Opdracht::TerugInBeeld { graphic: titelkaart(""), standpunt: None }), Err(Weigering::KopLeeg));
    assert_eq!(
        p.doe(Opdracht::TerugInBeeld { graphic: titelkaart("Oud"), standpunt: Some("P".into()) }),
        Err(Weigering::OnbekendStandpunt("P".into()))
    );
    assert!(p.stand.ether.is_none());
}

#[test]
fn terug_in_beeld_beweegt_de_camera_zoals_toon_behalve_bij_blijf() {
    let mut p = Proef::new();
    p.doe(Opdracht::KiesStandpunt { letter: "Q".into(), knip: false }).unwrap();
    let voor = p.stand.camera.clone();
    p.doe(Opdracht::TerugInBeeld { graphic: titelkaart("Oud"), standpunt: Some(camera::BLIJF.into()) }).unwrap();
    assert!(p.stand.ether.is_some());
    assert_eq!(p.stand.camera, voor, "de camera blijft staan");

    p.doe(Opdracht::UitBeeld).unwrap();
    p.doe(Opdracht::TerugInBeeld { graphic: titelkaart("Oud"), standpunt: Some("D".into()) }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "D");
    assert_eq!(p.stand.camera.sinds, p.volgnummer);
}
