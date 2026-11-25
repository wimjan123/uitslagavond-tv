//! De Graphic-stroom op de Stand: klaarzetten, Toon, Bijwerken en Uit beeld.

use gedeeld::{Graphic, Herkomst, Opdracht, Overgang, Stand, Weigering};

fn titelkaart(kop: &str) -> Graphic {
    Graphic::Titelkaart {
        kop: kop.into(),
        regel: "Live vanuit Den Haag".into(),
    }
}

/// Voert Opdrachten uit zoals de server: elk met het volgende volgnummer.
struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    fn new() -> Self {
        Proef {
            stand: Stand::default(),
            volgnummer: 0,
        }
    }

    fn door(&mut self, werkplek: &str, opdracht: Opdracht) -> Result<(), Weigering> {
        let mut nieuw = self.stand.clone();
        let herkomst = Herkomst {
            door: werkplek,
            volgnummer: self.volgnummer + 1,
            tijd_ms: 1_000,
        };
        nieuw.voer_uit(&opdracht, &herkomst)?;
        self.stand = nieuw;
        self.volgnummer += 1;
        Ok(())
    }

    fn klaar_versie(&self) -> u64 {
        self.stand
            .klaar
            .as_ref()
            .expect("er staat iets klaar")
            .versie
    }

    fn klaar_id(&self) -> u64 {
        self.stand.klaar.as_ref().expect("er staat iets klaar").id
    }
}

#[test]
fn toon_zet_de_klaargezette_graphic_in_de_ether() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("De Uitslagavond"),
            standpunt: None,
        },
    )
    .unwrap();
    let versie = p.klaar_versie();
    p.door(
        "Regie",
        Opdracht::Toon {
            versie: Some(versie),
        },
    )
    .unwrap();

    let ether = p.stand.ether.as_ref().expect("Graphic in de Ether");
    assert_eq!(ether.graphic, titelkaart("De Uitslagavond"));
    assert_eq!(ether.getoond_door, "Regie");
    assert_eq!(p.stand.klaar.as_ref().unwrap().klaargezet_door, "Regie");
}

#[test]
fn toon_weigert_na_wijziging_door_een_andere_werkplek_en_noemt_die() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("De Uitslagavond"),
            standpunt: None,
        },
    )
    .unwrap();
    let gezien = p.klaar_versie();
    let id = p.klaar_id();
    p.door(
        "Tablet",
        Opdracht::WijzigKlaar {
            id,
            graphic: titelkaart("Uitslagavond 2025"),
            standpunt: None,
        },
    )
    .unwrap();

    let w = p
        .door(
            "Regie",
            Opdracht::Toon {
                versie: Some(gezien),
            },
        )
        .unwrap_err();
    assert_eq!(w, Weigering::GewijzigdDoor("Tablet".into()));
    assert_eq!(w.to_string(), "gewijzigd door Tablet");
    assert!(p.stand.ether.is_none());

    let nu = p.klaar_versie();
    p.door("Regie", Opdracht::Toon { versie: Some(nu) })
        .unwrap();
    assert_eq!(
        p.stand.ether.as_ref().unwrap().graphic,
        titelkaart("Uitslagavond 2025")
    );
}

#[test]
fn toon_weigert_ook_als_een_ander_een_nieuwe_graphic_klaarzette() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Eerste"),
            standpunt: None,
        },
    )
    .unwrap();
    let gezien = p.klaar_versie();
    p.door(
        "Invoer",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Tweede"),
            standpunt: None,
        },
    )
    .unwrap();
    assert_eq!(
        p.door(
            "Regie",
            Opdracht::Toon {
                versie: Some(gezien)
            }
        ),
        Err(Weigering::GewijzigdDoor("Invoer".into()))
    );
}

#[test]
fn toon_zonder_versie_toont_wat_klaarstaat() {
    // Een Stream Deck-knop die niets meestuurt.
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Knop"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Stream Deck", Opdracht::Toon { versie: None })
        .unwrap();
    assert_eq!(p.stand.ether.as_ref().unwrap().graphic, titelkaart("Knop"));
}

#[test]
fn toon_weigert_zonder_klaargezette_graphic_of_als_die_al_in_beeld_is() {
    let mut p = Proef::new();
    assert_eq!(
        p.door("Regie", Opdracht::Toon { versie: None }),
        Err(Weigering::NietsKlaar)
    );
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Eén keer"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(
        p.door("Regie", Opdracht::Toon { versie: None }),
        Err(Weigering::AlInBeeld)
    );
}

#[test]
fn graphic_in_beeld_verandert_pas_na_bijwerken() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Uitslagvond"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    let getoond = p.stand.ether.clone().unwrap();

    // Een tikfout verbeteren in de klaargezette Graphic laat de Ether ongemoeid.
    let id = p.klaar_id();
    p.door(
        "Invoer",
        Opdracht::WijzigKlaar {
            id,
            graphic: titelkaart("Uitslagavond"),
            standpunt: None,
        },
    )
    .unwrap();
    assert_eq!(p.stand.ether.as_ref(), Some(&getoond));
    assert!(p.stand.kan_bijwerken());

    p.door("Regie", Opdracht::Bijwerken).unwrap();
    let bijgewerkt = p.stand.ether.clone().unwrap();
    assert_eq!(bijgewerkt.graphic, titelkaart("Uitslagavond"));
    assert_eq!(
        bijgewerkt.sinds_ms, getoond.sinds_ms,
        "bijwerken is geen nieuwe Toon"
    );
    assert_eq!(
        Overgang::tussen(Some(&getoond), Some(&bijgewerkt)),
        Overgang::Bijwerken
    );
    assert!(!p.stand.kan_bijwerken());
    assert_eq!(
        p.door("Regie", Opdracht::Bijwerken),
        Err(Weigering::NietsBijTeWerken)
    );
}

#[test]
fn een_nieuwe_graphic_klaarzetten_raakt_de_ether_niet() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("In beeld"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Volgende"),
            standpunt: None,
        },
    )
    .unwrap();
    assert_eq!(
        p.stand.ether.as_ref().unwrap().graphic,
        titelkaart("In beeld")
    );
    assert!(
        !p.stand.kan_bijwerken(),
        "een andere Graphic is geen nieuwere versie"
    );
    assert_eq!(
        p.door("Regie", Opdracht::Bijwerken),
        Err(Weigering::NietsBijTeWerken)
    );
}

#[test]
fn wijzigen_weigert_als_er_intussen_een_andere_graphic_klaarstaat() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Oud"),
            standpunt: None,
        },
    )
    .unwrap();
    let oud = p.klaar_id();
    p.door(
        "Tablet",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Nieuw"),
            standpunt: None,
        },
    )
    .unwrap();
    assert_eq!(
        p.door(
            "Regie",
            Opdracht::WijzigKlaar {
                id: oud,
                graphic: titelkaart("Oud, verbeterd"),
                standpunt: None
            }
        ),
        Err(Weigering::AndereGraphicKlaar("Tablet".into()))
    );
}

#[test]
fn uit_beeld_haalt_de_graphic_uit_de_ether_en_laat_klaar_staan() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Weg ermee"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    p.door("Regie", Opdracht::UitBeeld).unwrap();
    assert!(p.stand.ether.is_none());
    assert!(p.stand.klaar.is_some());
    assert_eq!(
        p.door("Regie", Opdracht::UitBeeld),
        Err(Weigering::NietsInBeeld)
    );
}

#[test]
fn titelkaart_zonder_kop_of_met_te_lange_tekst_wordt_geweigerd() {
    let mut p = Proef::new();
    let leeg = Graphic::Titelkaart {
        kop: "  ".into(),
        regel: String::new(),
    };
    assert_eq!(
        p.door(
            "Regie",
            Opdracht::ZetKlaar {
                graphic: leeg,
                standpunt: None
            }
        ),
        Err(Weigering::KopLeeg)
    );
    let lang = Graphic::Titelkaart {
        kop: "k".repeat(gedeeld::MAX_KOP + 1),
        regel: String::new(),
    };
    assert_eq!(
        p.door(
            "Regie",
            Opdracht::ZetKlaar {
                graphic: lang,
                standpunt: None
            }
        ),
        Err(Weigering::KopTeLang)
    );
    let regel = Graphic::Titelkaart {
        kop: "Kop".into(),
        regel: "r".repeat(gedeeld::MAX_REGEL + 1),
    };
    assert_eq!(
        p.door(
            "Regie",
            Opdracht::ZetKlaar {
                graphic: regel,
                standpunt: None
            }
        ),
        Err(Weigering::RegelTeLang)
    );
    assert!(p.stand.klaar.is_none());
}

#[test]
fn de_renderer_kiest_de_overgang_uit_twee_standen() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Eerste"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    let eerste = p.stand.ether.clone();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Tweede"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    let tweede = p.stand.ether.clone();

    assert_eq!(Overgang::tussen(None, eerste.as_ref()), Overgang::Erin);
    assert_eq!(
        Overgang::tussen(eerste.as_ref(), tweede.as_ref()),
        Overgang::Wissel
    );
    assert_eq!(
        Overgang::tussen(tweede.as_ref(), tweede.as_ref()),
        Overgang::Geen
    );
    assert_eq!(Overgang::tussen(tweede.as_ref(), None), Overgang::Eruit);
}

/// Een andere Stijl (voorbeeldgegevens): rood en geel in plaats van blauw.
fn andere_stijl() -> gedeeld::sjabloon::Stijl {
    let mut s = gedeeld::sjabloon::Stijl::default();
    s.kleuren.hoofd = "#ffd400".parse().unwrap();
    s.kleuren.vlak = "#5a0a0a".parse().unwrap();
    s.snelheid = 140;
    s
}

#[test]
fn een_nieuwe_stijl_geldt_pas_bij_de_volgende_toon() {
    let mut p = Proef::new();
    let oud = p.stand.stijl.clone();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("In beeld"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Volgende"),
            standpunt: None,
        },
    )
    .unwrap();
    let gezien = p.klaar_versie();

    // Opslaan in de vormgevingstab.
    p.door(
        "Vormgeving",
        Opdracht::ZetStijl {
            stijl: andere_stijl(),
        },
    )
    .unwrap();
    assert_eq!(p.stand.stijl, andere_stijl());

    // De Graphic in de Ether houdt haar vorm.
    let ether = p.stand.ether.as_ref().unwrap();
    assert_eq!(ether.stijl, oud);
    assert_eq!(ether.graphic, titelkaart("In beeld"));

    // De klaargezette Graphic krijgt de nieuwe vorm en telt als net veranderd: Toon van wat het
    // scherm daarvoor liet zien weigert.
    let klaar = p.stand.klaar.as_ref().unwrap();
    assert_ne!(klaar.versie, gezien);
    assert_eq!(klaar.gewijzigd_door, "Vormgeving");
    assert_eq!(
        p.door(
            "Regie",
            Opdracht::Toon {
                versie: Some(gezien)
            }
        ),
        Err(Weigering::GewijzigdDoor("Vormgeving".into()))
    );
    assert_eq!(p.stand.ether.as_ref().unwrap().stijl, oud);

    // De volgende Toon brengt de nieuwe vorm in de Ether.
    let nu = p.klaar_versie();
    p.door("Regie", Opdracht::Toon { versie: Some(nu) })
        .unwrap();
    let ether = p.stand.ether.as_ref().unwrap();
    assert_eq!(ether.graphic, titelkaart("Volgende"));
    assert_eq!(ether.stijl, andere_stijl());
}

#[test]
fn bijwerken_houdt_de_vorm_van_de_graphic_in_de_ether() {
    let mut p = Proef::new();
    let oud = p.stand.stijl.clone();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Uitslagvond"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    p.door(
        "Vormgeving",
        Opdracht::ZetStijl {
            stijl: andere_stijl(),
        },
    )
    .unwrap();

    // Dezelfde Graphic staat klaar en in de Ether: Toon mag weer, met de nieuwe vorm.
    assert!(!p.stand.staat_klaar_in_beeld());

    // Een tikfout verbeteren en Bijwerken: de gegevens wel, de vorm niet.
    let id = p.klaar_id();
    p.door(
        "Invoer",
        Opdracht::WijzigKlaar {
            id,
            graphic: titelkaart("Uitslagavond"),
            standpunt: None,
        },
    )
    .unwrap();
    p.door("Regie", Opdracht::Bijwerken).unwrap();
    let ether = p.stand.ether.as_ref().unwrap();
    assert_eq!(ether.graphic, titelkaart("Uitslagavond"));
    assert_eq!(ether.stijl, oud);
}

#[test]
fn dezelfde_stijl_opslaan_verandert_niets_aan_wat_klaarstaat() {
    let mut p = Proef::new();
    p.door(
        "Regie",
        Opdracht::ZetKlaar {
            graphic: titelkaart("Klaar"),
            standpunt: None,
        },
    )
    .unwrap();
    let versie = p.klaar_versie();
    let stijl = p.stand.stijl.clone();
    p.door("Vormgeving", Opdracht::ZetStijl { stijl }).unwrap();
    assert_eq!(p.klaar_versie(), versie);
}

#[test]
fn een_stijl_met_een_onmogelijke_snelheid_of_bestandsnaam_wordt_geweigerd() {
    let mut p = Proef::new();
    let mut traag = andere_stijl();
    traag.snelheid = 5;
    assert!(matches!(
        p.door("Vormgeving", Opdracht::ZetStijl { stijl: traag }),
        Err(Weigering::Stijl(_))
    ));
    let mut pad = andere_stijl();
    pad.lettertypes.kop = "../geheim.ttf".into();
    assert!(matches!(
        p.door("Vormgeving", Opdracht::ZetStijl { stijl: pad }),
        Err(Weigering::Stijl(_))
    ));
    assert_eq!(p.stand.stijl, gedeeld::sjabloon::Stijl::default());
}
