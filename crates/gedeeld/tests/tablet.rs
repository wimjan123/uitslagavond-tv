//! De tablet van de Presentator (besluit #10, T2): hij zet een Preset klaar die de Regie op de
//! tablet zette en tikt Partijen aan in de Coalitiebouwer. Hij zet zelf niets in de Ether. De
//! server controleert dat met `Stand::controleer_afzender` vóór `Stand::voer_uit`; deze proef
//! doet hetzelfde. De Graphics hier zijn voorbeeldgegevens.

use gedeeld::cijfers::Invoer;
use gedeeld::{Afzender, Graphic, Herkomst, Opdracht, Stand, Weigering};

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

fn regie() -> Afzender {
    Afzender::Werkplek {
        naam: "Regie".into(),
    }
}

fn tablet() -> Afzender {
    Afzender::Tablet {
        naam: "Tablet".into(),
    }
}

impl Proef {
    /// Twee Presets (voorbeeldgegevens): 1 staat op de tablet, 2 niet.
    fn nieuw() -> Proef {
        let mut p = Proef {
            stand: Stand::default(),
            volgnummer: 0,
        };
        for kop in ["Op de tablet", "Alleen voor de Regie"] {
            let graphic = Graphic::Titelkaart {
                kop: kop.into(),
                regel: "Voorbeeldgegevens".into(),
            };
            p.doe(
                &regie(),
                Opdracht::BewaarPreset {
                    graphic,
                    standpunt: None,
                },
            )
            .unwrap();
        }
        p.doe(
            &regie(),
            Opdracht::ZetPresetOpTablet {
                nummer: 1,
                aan: true,
            },
        )
        .unwrap();
        p
    }

    /// Zoals de server: eerst de afzender controleren, dan uitvoeren.
    fn doe(&mut self, afzender: &Afzender, o: Opdracht) -> Result<(), Weigering> {
        self.stand.controleer_afzender(afzender, &o)?;
        self.volgnummer += 1;
        let naam = afzender.naam();
        let h = Herkomst {
            door: &naam,
            volgnummer: self.volgnummer,
            tijd_ms: self.volgnummer * 1000,
        };
        self.stand.voer_uit(&o, &h)
    }
}

#[test]
fn de_tablet_zet_een_preset_klaar_maar_toont_niets() {
    let mut p = Proef::nieuw();
    p.doe(&tablet(), Opdracht::ZetPresetKlaar { nummer: 1 })
        .unwrap();
    let klaar = p.stand.klaar.clone().expect("de Preset staat klaar");
    assert_eq!(klaar.klaargezet_door, "Tablet");

    assert_eq!(
        p.doe(
            &tablet(),
            Opdracht::Toon {
                versie: Some(klaar.versie)
            }
        ),
        Err(Weigering::NietVanafTablet)
    );
    assert_eq!(
        p.doe(&tablet(), Opdracht::Toon { versie: None }),
        Err(Weigering::NietVanafTablet)
    );
    assert!(p.stand.ether.is_none(), "de tablet zet niets in de Ether");

    // De Regie drukt op Toon.
    p.doe(
        &regie(),
        Opdracht::Toon {
            versie: Some(klaar.versie),
        },
    )
    .unwrap();
    assert_eq!(
        p.stand.ether.as_ref().map(|e| e.getoond_door.as_str()),
        Some("Regie")
    );
}

#[test]
fn de_tablet_mag_niets_buiten_klaarzetten_en_de_coalitiebouwer() {
    let mut p = Proef::nieuw();
    p.doe(&regie(), Opdracht::ZetPresetKlaar { nummer: 2 })
        .unwrap();
    p.doe(&regie(), Opdracht::Toon { versie: None }).unwrap();
    let verboden = [
        Opdracht::UitBeeld,
        Opdracht::Bijwerken,
        Opdracht::KiesStandpunt {
            letter: "A".into(),
            knip: false,
        },
        Opdracht::ZetVrijeCamera { aan: true },
        Opdracht::ZetKlaar {
            graphic: Graphic::Titelkaart {
                kop: "Eigen".into(),
                regel: String::new(),
            },
            standpunt: None,
        },
        Opdracht::ZetPresetOpTablet {
            nummer: 2,
            aan: true,
        },
        Opdracht::WisPreset { nummer: 1 },
        Opdracht::ZetStijl {
            stijl: p.stand.stijl.clone(),
        },
    ];
    let voor = p.stand.clone();
    for o in verboden {
        assert_eq!(
            p.doe(&tablet(), o.clone()),
            Err(Weigering::NietVanafTablet),
            "{o:?}"
        );
    }
    assert_eq!(p.stand, voor, "de Stand blijft gelijk");
}

#[test]
fn de_tablet_zet_alleen_presets_klaar_die_op_de_tablet_staan() {
    let mut p = Proef::nieuw();
    assert_eq!(
        p.doe(&tablet(), Opdracht::ZetPresetKlaar { nummer: 2 }),
        Err(Weigering::NietOpTablet(2))
    );
    assert_eq!(
        p.doe(&tablet(), Opdracht::ZetPresetKlaar { nummer: 9 }),
        Err(Weigering::OnbekendePreset(9))
    );
    assert!(p.stand.klaar.is_none());

    // De Regie haalt Preset 1 van de tablet: daarna kan de tablet hem niet meer klaarzetten.
    p.doe(
        &regie(),
        Opdracht::ZetPresetOpTablet {
            nummer: 1,
            aan: false,
        },
    )
    .unwrap();
    assert_eq!(
        p.doe(&tablet(), Opdracht::ZetPresetKlaar { nummer: 1 }),
        Err(Weigering::NietOpTablet(1))
    );
}

#[test]
fn de_tablet_tikt_partijen_aan_in_de_coalitie_in_beeld() {
    let mut p = Proef::nieuw();
    // Voorbeeldgegevens: twee Partijen en een set "Exitpoll".
    let json = r##"{ "partijen": [
        { "sleutel": "alfa", "naam": "Alfa", "kort": "ALFA", "kleur": "#1f6fb2" },
        { "sleutel": "beta", "naam": "Beta", "kort": "BETA", "kleur": "#d33b2c" } ],
      "sets": [ { "id": "exitpoll", "soort": "peiling", "label": "Exitpoll", "zetels": { "alfa": 80, "beta": 70 } } ] }"##;
    let invoer = Opdracht::LeesInvoer {
        bestand: "cijfers.json".into(),
        ontvangen: "21.00".into(),
        invoer: Invoer::uit_json(json),
    };
    p.doe(&regie(), invoer).unwrap();
    let coalitie = Graphic::Coalitie {
        set: "exitpoll".into(),
        partijen: vec![],
        cijfers: None,
    };
    p.doe(
        &regie(),
        Opdracht::ZetKlaar {
            graphic: coalitie,
            standpunt: None,
        },
    )
    .unwrap();
    p.doe(&regie(), Opdracht::Toon { versie: None }).unwrap();

    p.doe(
        &tablet(),
        Opdracht::ZetCoalitie {
            partijen: vec!["alfa".into()],
        },
    )
    .unwrap();
    let Some(Graphic::Coalitie { partijen, .. }) = p.stand.ether.as_ref().map(|e| &e.graphic)
    else {
        panic!("de Coalitie staat in de Ether");
    };
    assert_eq!(
        partijen,
        &["alfa"],
        "een Coalitie in beeld werkt direct bij"
    );
}

#[test]
fn een_werkplek_van_de_regie_mag_alles() {
    let mut p = Proef::nieuw();
    p.doe(&regie(), Opdracht::ZetPresetKlaar { nummer: 2 })
        .unwrap();
    p.doe(&regie(), Opdracht::Toon { versie: None }).unwrap();
    p.doe(&regie(), Opdracht::UitBeeld).unwrap();
}
