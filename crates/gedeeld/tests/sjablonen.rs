//! Sjablonen uit Blokken, ingevuld met een Graphic, de cijfers en de Stijl (ADR 0003).

use gedeeld::cijfers::Invoer;
use gedeeld::sjabloon::{
    Animatie, Beweging, Blok, Bloksoort, Erin, Eruit, Getalbron, Getalwaarde, IngevuldVerloop, Inhoud,
    Invulling, Kleurkeuze, OmTekst, Verloop, tekst, vlak_met,
    LETTER_KOP, LETTER_TEKST, Letterkeuze, Partijbron, Plek, Sjabloon, Stijl, Stijlkleur,
    Stijlletter, Tekstbron, Uitlijning, vast_sjabloon, vaste_sjablonen,
};
use gedeeld::{Graphic, Herkomst, Kleur, Opdracht, Stand};

fn kleur(hex: &str) -> Kleur {
    hex.parse().unwrap()
}

fn titelkaart() -> Graphic {
    Graphic::Titelkaart {
        kop: "Wie wint de verkiezing?".into(),
        regel: "Voorbeeldgegevens".into(),
    }
}

/// Vult een Graphic in met de Stand, zoals de Renderer dat doet.
fn vul_in(graphic: &Graphic, stand: &Stand) -> Vec<gedeeld::sjabloon::IngevuldBlok> {
    let invulling = graphic.invulling();
    let sjabloon = vast_sjabloon(&invulling.sjabloon).expect("vast Sjabloon");
    sjabloon.vul_in(&invulling, &stand.cijfers, &stand.stijl)
}

/// De kop van de Titelkaart: de tekst met de inhoud van het veld `kop`.
fn kop(blokken: &[gedeeld::sjabloon::IngevuldBlok]) -> (&str, Kleur, &str) {
    blokken
        .iter()
        .find_map(|b| match &b.inhoud {
            Inhoud::Tekst { tekst, letter, .. } if tekst == "Wie wint de verkiezing?" => {
                Some((tekst.as_str(), b.kleur, letter.as_str()))
            }
            _ => None,
        })
        .expect("de Titelkaart toont de kop")
}

#[test]
fn de_titelkaart_volgt_een_andere_stijl_in_kleur_en_lettertype() {
    let mut stand = Stand::default();
    let blokken = vul_in(&titelkaart(), &stand);
    let (_, k, letter) = kop(&blokken);
    assert_eq!(k, kleur("#ffd166"), "standaardstijl: de kop in geel (hoofd)");
    assert_eq!(letter, LETTER_KOP, "standaardstijl: de kop in de kopletter");

    stand.stijl.kleuren.hoofd = kleur("#e000e0");
    stand.stijl.lettertypes.kop = "DejaVuSerif-Bold.ttf".into();
    let blokken = vul_in(&titelkaart(), &stand);
    let (_, k, letter) = kop(&blokken);
    assert_eq!(k, kleur("#e000e0"));
    assert_eq!(letter, "DejaVuSerif-Bold.ttf");
}

#[test]
fn vaste_sjablonen_gebruiken_alleen_kleuren_en_lettertypes_uit_de_stijl() {
    let vaste = vaste_sjablonen();
    assert!(vaste.iter().any(|s| s.id == "titelkaart"));
    for s in &vaste {
        for b in &s.blokken {
            assert!(
                matches!(b.kleur, Kleurkeuze::Stijl(_)),
                "{}: eigen kleur in {:?}",
                s.id,
                b
            );
            if let Some(l) = b.soort.letter() {
                assert!(
                    matches!(l, Letterkeuze::Stijl(_)),
                    "{}: eigen lettertype in {:?}",
                    s.id,
                    b
                );
            }
        }
    }
}

fn blok(soort: Bloksoort, kleur: Kleurkeuze) -> Blok {
    Blok {
        plek: Plek {
            x: 0,
            y: 0,
            breedte: 400,
            hoogte: 100,
        },
        soort,
        kleur,
        animatie: Animatie {
            erin: Erin::Vervagen,
            eruit: Eruit::Vervagen,
            start_na_ms: 0,
        },
    }
}

fn eigen(blokken: Vec<Blok>) -> Sjabloon {
    Sjabloon {
        id: "proef".into(),
        naam: "Proef".into(),
        groep: "Eigen".into(),
        snelheid: 100,
        achtergrond: Default::default(),
        stoelen: Default::default(),
        standpunt: None,
        blokken,
    }
}

#[test]
fn een_blok_met_een_eigen_kleur_of_lettertype_volgt_de_stijl_niet_meer() {
    let tekst = |kleur, letter| {
        blok(
            Bloksoort::Tekst {
                bron: Tekstbron::Vast("ë é".into()),
                letter,
                grootte: 40,
                uitlijning: Uitlijning::Links,
                een_regel: false,
            },
            kleur,
        )
    };
    let sjabloon = eigen(vec![
        tekst(
            Kleurkeuze::Stijl(Stijlkleur::Accent),
            Letterkeuze::Stijl(Stijlletter::Tekst),
        ),
        tekst(
            Kleurkeuze::Eigen(kleur("#123456")),
            Letterkeuze::Eigen("Eigen.otf".into()),
        ),
    ]);
    let mut stijl = Stijl::default();
    stijl.kleuren.accent = kleur("#00ff00");
    stijl.lettertypes.tekst = "Ander.ttf".into();
    let uit = sjabloon.vul_in(&Invulling::default(), &Default::default(), &stijl);
    let letters: Vec<_> = uit
        .iter()
        .map(|b| match &b.inhoud {
            Inhoud::Tekst { letter, .. } => (b.kleur, letter.clone()),
            i => panic!("geen tekst: {i:?}"),
        })
        .collect();
    assert_eq!(letters[0], (kleur("#00ff00"), "Ander.ttf".to_string()));
    assert_eq!(letters[1], (kleur("#123456"), "Eigen.otf".to_string()));
}

/// VOORBEELDGEGEVENS: drie partijen, waarvan één met logo, en een set onder 150 Zetels.
fn stand_met_cijfers() -> Stand {
    let invoer = Invoer::uit_json(
        r##"{
          "partijen": [
            {"sleutel": "a", "naam": "Partij A", "kort": "A", "kleur": "#d4202c", "logo": "logos/a.png"},
            {"sleutel": "b", "naam": "Partij B", "kort": "PvB", "kleur": "#00ae41"},
            {"sleutel": "c", "naam": "Partij C", "kort": "C", "kleur": "#0e2c5a"}
          ],
          "sets": [
            {"id": "toen", "soort": "uitslag", "label": "Uitslag toen", "zetels": {"a": 40, "b": 30, "c": 80}},
            {"id": "nu", "soort": "prognose", "label": "Prognose 22:00", "zetels": {"a": 45, "b": 25, "c": 70}}
          ],
          "vergelijk_met": "toen"
        }"##,
    );
    let mut stand = Stand::default();
    stand
        .voer_uit(
            &Opdracht::LeesInvoer {
                bestand: "proef.json".into(),
                ontvangen: "nu".into(),
                invoer,
            },
            &Herkomst {
                door: "Proef",
                volgnummer: 1,
                tijd_ms: 0,
            },
        )
        .unwrap();
    stand
}

fn met_partij(partij: &str) -> Invulling {
    Invulling {
        partij: Some(partij.into()),
        set: Some("nu".into()),
        ..Default::default()
    }
}

#[test]
fn een_partijlogo_zonder_logo_toont_de_korte_naam_in_de_partijkleur() {
    let stand = stand_met_cijfers();
    let s = eigen(vec![blok(
        Bloksoort::Partijlogo {
            bron: Partijbron::Partij,
            hoek: 0,
        },
        Kleurkeuze::Stijl(Stijlkleur::Tekst),
    )]);
    let logo = |p| {
        s.vul_in(&met_partij(p), &stand.cijfers, &stand.stijl)
            .remove(0)
            .inhoud
    };
    assert_eq!(
        logo("a"),
        Inhoud::Partijlogo {
            logo: Some("logos/a.png".into()),
            kort: "A".into(),
            kleur: kleur("#d4202c"),
            hoek: 0
        }
    );
    assert_eq!(
        logo("b"),
        Inhoud::Partijlogo {
            logo: None,
            kort: "PvB".into(),
            kleur: kleur("#00ae41"),
            hoek: 0
        }
    );
}

#[test]
fn getallen_tonen_zetels_en_winst_met_plusteken() {
    let stand = stand_met_cijfers();
    let getal = |bron| {
        blok(
            Bloksoort::Getal {
                bron,
                letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                grootte: 90,
                uitlijning: Uitlijning::Rechts,
            },
            Kleurkeuze::Stijl(Stijlkleur::Tekst),
        )
    };
    let s = eigen(vec![getal(Getalbron::Zetels), getal(Getalbron::Winst)]);
    let teksten = |p| -> Vec<String> {
        s.vul_in(&met_partij(p), &stand.cijfers, &stand.stijl)
            .into_iter()
            .map(|b| match b.inhoud {
                Inhoud::Getal { waarde, .. } => waarde.tekst(),
                i => panic!("geen getal: {i:?}"),
            })
            .collect()
    };
    assert_eq!(teksten("a"), ["45", "+5"]);
    assert_eq!(teksten("b"), ["25", "\u{2212}5"]);
    assert_eq!(teksten("c"), ["70", "\u{2212}10"]);
    // optellen telt naar het getal
    let w = Getalwaarde::Getal {
        waarde: 5,
        plus: true,
        achter: String::new(),
    };
    assert_eq!(w.tekst_bij(3), "+3");
}

#[test]
fn de_zetelbalk_toont_de_partijen_op_volgorde_en_een_rest_voor_wat_ontbreekt_aan_150() {
    let mut stand = stand_met_cijfers();
    stand
        .voer_uit(
            &Opdracht::ZetZetels {
                set: "nu".into(),
                partij: "c".into(),
                zetels: 60,
            },
            &Herkomst {
                door: "Proef",
                volgnummer: 2,
                tijd_ms: 0,
            },
        )
        .unwrap();
    let s = eigen(vec![blok(
        Bloksoort::Zetelbalk {
            letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
            lijn_bij_76: true,
            naamletter: Letterkeuze::Stijl(Stijlletter::Tekst),
            labels: false,
            vorige_set: false,
            pijlen: false,
            coalitie: false,
        },
        Kleurkeuze::Stijl(Stijlkleur::Tekst),
    )]);
    let Inhoud::Zetelbalk {
        delen, lijn_bij_76, ..
    } = s
        .vul_in(&met_partij("a"), &stand.cijfers, &stand.stijl)
        .remove(0)
        .inhoud
    else {
        panic!("geen zetelbalk")
    };
    assert!(lijn_bij_76);
    let kort: Vec<_> = delen.iter().map(|d| (d.kort.as_str(), d.zetels)).collect();
    assert_eq!(kort, [("A", 45), ("PvB", 25), ("C", 60), ("", 20)]);
    assert_eq!(
        delen[3].partij, None,
        "de rest hoort bij geen partij"
    );
}

#[test]
fn een_stijl_zonder_beweging_krijgt_de_standaard_en_een_onmogelijke_beweging_wordt_geweigerd() {
    let mut json = serde_json::to_value(Stijl::default()).unwrap();
    json.as_object_mut().unwrap().remove("beweging");
    let stijl: Stijl = serde_json::from_value(json).unwrap();
    assert_eq!(stijl.beweging, Beweging::default());
    assert!(stijl.controleer().is_ok());

    let fout = |zet: fn(&mut Beweging)| {
        let mut s = Stijl::default();
        zet(&mut s.beweging);
        s.controleer().is_err()
    };
    assert!(fout(|b| b.duur_ms = 50), "te kort");
    assert!(fout(|b| b.lijn.opent_ms = 0), "het vlak gaat nooit open");
    assert!(fout(|b| b.lijn.tekst_weg_pct = b.lijn.vlak_dicht_pct), "tekst en vlak tegelijk weg");
    assert!(fout(|b| b.lijn.vlak_dicht_pct = 100), "de lijn verdwijnt nooit");
}

#[test]
fn een_stand_zonder_stijl_krijgt_de_standaardstijl() {
    let s: Stand = serde_json::from_str("{}").unwrap();
    assert_eq!(s.stijl, Stijl::default());
    assert_eq!(s.stijl.lettertypes.kop, LETTER_KOP);
    assert_eq!(s.stijl.lettertypes.tekst, LETTER_TEKST);
    assert_eq!(s.stijl.snelheid, 100);
}

#[test]
fn bestandsnamen_blijven_in_de_map_van_de_server() {
    use gedeeld::sjabloon::geldige_bestandsnaam;
    for goed in ["Fira-Bold.ttf", "logos/pvv.png", "foto_12.jpg"] {
        assert!(geldige_bestandsnaam(goed), "{goed}");
    }
    for fout in [
        "",
        "../geheim",
        "/etc/passwd",
        "logos/../../x",
        ".verborgen",
        "a\\b",
        "a b.ttf",
        "logos//a.png",
    ] {
        assert!(!geldige_bestandsnaam(fout), "{fout}");
    }
}

fn ongeveer(a: [f32; 4], b: [f32; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.003)
}

/// Het verloop van een Vlak met deze kleur, dekking en dit verloop, in de standaardstijl.
fn verloop_van(kleur: Stijlkleur, dekking: u8, v: Verloop) -> IngevuldVerloop {
    let s = eigen(vec![blok(vlak_met(dekking, Some(v), 0, None, None), Kleurkeuze::Stijl(kleur))]);
    match s.vul_in(&Invulling::default(), &Default::default(), &Stijl::default()).remove(0).inhoud {
        Inhoud::Vlak { verloop: Some(v), .. } => v,
        i => panic!("geen verloop: {i:?}"),
    }
}

#[test]
fn een_kleurverloop_loopt_gelijkmatig_tussen_twee_stijlkleuren_en_is_effen_buiten_van_en_tot() {
    // Het paneel: Achtergrond naar Vlak, beide op 95 procent, tot 62 procent.
    let v = verloop_van(
        Stijlkleur::Achtergrond,
        95,
        Verloop { naar: Kleurkeuze::Stijl(Stijlkleur::Vlak), naar_dekking: 95, richting: 120, van: 0, tot: 62 },
    );
    let rgb = |r: f32, g: f32, b: f32, a: f32| [r / 255.0, g / 255.0, b / 255.0, a];
    assert!(ongeveer(v.op(0.0), rgb(58.0, 63.0, 75.0, 0.95)));
    // halverwege 0 en 62 procent: het gemiddelde van beide kleuren
    assert!(ongeveer(v.op(0.31), rgb(43.0, 47.0, 56.5, 0.95)), "{:?}", v.op(0.31));
    assert!(ongeveer(v.op(0.62), rgb(28.0, 31.0, 38.0, 0.95)));
    assert!(ongeveer(v.op(0.9), rgb(28.0, 31.0, 38.0, 0.95)), "na tot: effen");
}

#[test]
fn een_verloop_naar_doorzichtig_houdt_de_kleur_en_laat_alleen_de_dekking_zakken() {
    // De waas van de Titelkaart: Achtergrond op 97 procent naar 0, tussen 30 en 90 procent.
    let v = verloop_van(
        Stijlkleur::Achtergrond,
        97,
        Verloop { naar: Kleurkeuze::Stijl(Stijlkleur::Achtergrond), naar_dekking: 0, richting: 90, van: 30, tot: 90 },
    );
    let k = [58.0 / 255.0, 63.0 / 255.0, 75.0 / 255.0];
    assert!(ongeveer(v.op(0.1), [k[0], k[1], k[2], 0.97]), "voor van: effen");
    assert!(ongeveer(v.op(0.6), [k[0], k[1], k[2], 0.485]), "{:?}", v.op(0.6));
    assert!(ongeveer(v.op(1.0), [k[0], k[1], k[2], 0.0]));
}

#[test]
fn de_lijn_van_een_verloop_loopt_zoals_in_css_van_hoek_tot_hoek() {
    let met = |richting| {
        verloop_van(
            Stijlkleur::Achtergrond,
            100,
            Verloop { naar: Kleurkeuze::Stijl(Stijlkleur::Vlak), naar_dekking: 100, richting, van: 0, tot: 100 },
        )
    };
    let (b, h) = (1728.0, 330.0);
    // 90 graden: van links naar rechts
    assert!((met(90).positie(0.0, 100.0, b, h) - 0.0).abs() < 1e-4);
    assert!((met(90).positie(432.0, 0.0, b, h) - 0.25).abs() < 1e-4);
    // 180 graden: van boven naar onder
    assert!((met(180).positie(500.0, h, b, h) - 1.0).abs() < 1e-4);
    // schuin: de hoek linksboven op 0, rechtsonder op 1, het midden op een half
    let v = met(120);
    assert!(v.positie(0.0, 0.0, b, h).abs() < 1e-4, "{}", v.positie(0.0, 0.0, b, h));
    assert!((v.positie(b, h, b, h) - 1.0).abs() < 1e-4);
    assert!((v.positie(b / 2.0, h / 2.0, b, h) - 0.5).abs() < 1e-4);
}

#[test]
fn een_naambalk_is_zo_breed_als_de_langste_tekst_plus_de_marge_en_nooit_breder_dan_zijn_plek() {
    let om = OmTekst { blokken: vec![1, 2], marge: 48 };
    let balk = Plek { x: 96, y: 744, breedte: 1728, hoogte: 210 };
    let regel = |x, breedte| Plek { x, y: 0, breedte, hoogte: 50 };
    // kop 560 breed vanaf x 140, regel 840 breed: tot 980, plus 48 marge, min 96
    let b = om.breedte(balk, [
        (regel(140, 1640), Uitlijning::Links, 560.0),
        (regel(140, 1640), Uitlijning::Links, 840.0),
    ]);
    assert_eq!(b, 932.0);
    // een tekst die breder is dan zijn Plek telt als zo breed als zijn Plek; de balk blijft
    // binnen zijn eigen Plek
    let b = om.breedte(balk, [(regel(140, 1640), Uitlijning::Links, 2400.0)]);
    assert_eq!(b, 1728.0);
    // rechts uitgelijnd: de rechterrand van de Plek; in het midden: de helft van wat overblijft
    assert_eq!(om.breedte(balk, [(regel(100, 400), Uitlijning::Rechts, 50.0)]), 452.0);
    assert_eq!(om.breedte(balk, [(regel(100, 400), Uitlijning::Midden, 200.0)]), 352.0);
    // zonder tekst: alleen de marge
    assert_eq!(om.breedte(balk, []), 48.0);
}

#[test]
fn de_teksten_in_een_naambalk_blijven_op_een_regel_en_andere_teksten_niet() {
    let stand = Stand::default();
    let blokken = vul_in(&titelkaart(), &stand);
    let een_regel: Vec<(String, bool)> = blokken
        .iter()
        .filter_map(|b| match &b.inhoud {
            Inhoud::Tekst { tekst, een_regel, .. } => Some((tekst.clone(), *een_regel)),
            _ => None,
        })
        .collect();
    assert_eq!(
        een_regel,
        [("Wie wint de verkiezing?".to_string(), true), ("Voorbeeldgegevens".to_string(), true)]
    );
    let s = eigen(vec![blok(
        tekst(Tekstbron::Vast("los".into()), Stijlletter::Tekst, 40),
        Kleurkeuze::Stijl(Stijlkleur::Tekst),
    )]);
    let los = s.vul_in(&Invulling::default(), &Default::default(), &Stijl::default());
    assert!(matches!(los[0].inhoud, Inhoud::Tekst { een_regel: false, .. }));
}

