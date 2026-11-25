//! De Zetelbalk en Twee sets naast elkaar: welke Partijen een eigen deel krijgen ("grootste N
//! plus overig") en de pijltjes voor het verschil tussen twee sets.

use std::collections::BTreeMap;

use gedeeld::cijfers::{Cijferset, OVERIG, Soort};
use gedeeld::zetelbalk::{Balkdeel, Pijl, delen, grootste, pijl};

/// VOORBEELDGEGEVENS: een set met deze Zetels.
fn set(zetels: &[(&str, u16)]) -> Cijferset {
    Cijferset {
        id: "proef".into(),
        soort: Soort::Prognose,
        label: "Proef".into(),
        tijd: None,
        bron: None,
        zetels: zetels
            .iter()
            .map(|(p, z)| (p.to_string(), *z))
            .collect::<BTreeMap<_, _>>(),
        stemmen: None,
        vergrendeld: false,
        versie: 1,
    }
}

fn volgorde(sleutels: &[&str]) -> Vec<String> {
    sleutels.iter().map(|s| s.to_string()).collect()
}

fn deel(partij: Option<&str>, zetels: u32) -> Balkdeel {
    Balkdeel {
        partij: partij.map(str::to_owned),
        zetels,
    }
}

#[test]
fn de_grootste_n_partijen_staan_op_volgorde_en_de_rest_samen_achteraan_als_overig() {
    let s = set(&[
        ("a", 10),
        ("b", 40),
        ("c", 5),
        ("d", 30),
        ("e", 25),
        (OVERIG, 3),
    ]);
    let v = volgorde(&["a", "b", "c", "d", "e"]);

    let gekozen = grootste(&s, &v, Some(3));
    assert_eq!(gekozen, ["b", "d", "e"], "op volgorde, niet op grootte");
    assert_eq!(
        delen(&s, &v, &gekozen),
        [
            deel(Some("b"), 40),
            deel(Some("d"), 30),
            deel(Some("e"), 25),
            deel(None, 18)
        ],
        "overig: a (10), c (5) en de Zetels die de set zelf overig noemt (3)"
    );
}

#[test]
fn alle_partijen_met_zetels_zonder_overig_als_er_niets_over_is() {
    let s = set(&[("a", 10), ("b", 0), ("c", 5)]);
    let v = volgorde(&["c", "b", "a"]);
    let gekozen = grootste(&s, &v, None);
    assert_eq!(
        gekozen,
        ["c", "a"],
        "een Partij zonder Zetels krijgt geen deel"
    );
    assert_eq!(
        delen(&s, &v, &gekozen),
        [deel(Some("c"), 5), deel(Some("a"), 10)]
    );
    // N groter dan het aantal Partijen: ook alle
    assert_eq!(grootste(&s, &v, Some(10)), ["c", "a"]);
}

#[test]
fn bij_gelijke_zetels_gaat_de_partij_die_links_staat_voor() {
    let s = set(&[("a", 20), ("b", 20), ("c", 20), ("d", 30)]);
    let v = volgorde(&["c", "a", "d", "b"]);
    assert_eq!(grootste(&s, &v, Some(2)), ["c", "d"]);
}

#[test]
fn een_partij_buiten_de_volgorde_telt_als_overig() {
    let s = set(&[("a", 20), ("onbekend", 7)]);
    let v = volgorde(&["a"]);
    let gekozen = grootste(&s, &v, None);
    assert_eq!(
        delen(&s, &v, &gekozen),
        [deel(Some("a"), 20), deel(None, 7)]
    );
}

#[test]
fn twee_sets_krijgen_dezelfde_partijen_en_wie_in_de_ene_set_niet_bij_de_grootste_hoort_is_overig() {
    // de grootste N komen uit de nieuwste set; de vorige set gebruikt dezelfde Partijen
    let vorige = set(&[("a", 30), ("b", 10), ("c", 20)]);
    let nieuwste = set(&[("a", 25), ("b", 28), ("c", 7)]);
    let v = volgorde(&["a", "b", "c"]);
    let gekozen = grootste(&nieuwste, &v, Some(2));
    assert_eq!(gekozen, ["a", "b"]);
    assert_eq!(
        delen(&vorige, &v, &gekozen),
        [deel(Some("a"), 30), deel(Some("b"), 10), deel(None, 20)]
    );
    assert_eq!(
        delen(&nieuwste, &v, &gekozen),
        [deel(Some("a"), 25), deel(Some("b"), 28), deel(None, 7)]
    );
}

#[test]
fn het_pijltje_wijst_van_de_vorige_set_naar_de_nieuwste() {
    let vorige = [deel(Some("a"), 30), deel(Some("b"), 10), deel(None, 20)];
    assert_eq!(pijl(&vorige, &deel(Some("a"), 25)), Pijl::Omlaag(5));
    assert_eq!(pijl(&vorige, &deel(Some("b"), 28)), Pijl::Omhoog(18));
    assert_eq!(
        pijl(&vorige, &deel(None, 20)),
        Pijl::Gelijk,
        "overig tegen overig"
    );
    // een Partij die in de vorige set geen deel had (nieuw): omhoog vanaf nul
    assert_eq!(pijl(&vorige, &deel(Some("d"), 4)), Pijl::Omhoog(4));
}

// De Sjablonen Zetelbalk, Twee sets naast elkaar en Uitslag per Partij, via de Stand.

mod sjablonen {
    use gedeeld::cijfers::{Invoer, OVERIG};
    use gedeeld::sjabloon::{Inhoud, Zeteldeel, vast_sjabloon};
    use gedeeld::uitslagindeling::{Uitslagvolgorde, zaal_voor_set};
    use gedeeld::zetelbalk::Pijl;
    use gedeeld::{Graphic, Herkomst, Opdracht, Stand, Weigering};

    struct Proef {
        stand: Stand,
        volgnummer: u64,
    }

    impl Proef {
        /// VOORBEELDGEGEVENS: vier verzonnen Partijen en twee sets; de prognose kwam later.
        fn new() -> Self {
            let mut p = Proef {
                stand: Stand::default(),
                volgnummer: 0,
            };
            let invoer = Invoer::uit_json(
                r##"{
                "partijen": [
                    { "sleutel": "rood", "naam": "Rode Partij", "kort": "ROOD", "kleur": "#d4202c" },
                    { "sleutel": "blauw", "naam": "Blauwe Partij", "kort": "BLAUW", "kleur": "#12357a" },
                    { "sleutel": "groen", "naam": "Groene Partij", "kort": "GROEN", "kleur": "#2dbe4f" },
                    { "sleutel": "geel", "naam": "Gele Partij", "kort": "GEEL", "kleur": "#e8c21a" }
                ],
                "sets": [
                    { "id": "exitpoll", "soort": "exitpoll", "label": "Exitpoll 21.00",
                      "zetels": { "rood": 50, "blauw": 40, "groen": 30, "geel": 20 } },
                    { "id": "prognose", "soort": "prognose", "label": "Prognose 23.00",
                      "zetels": { "rood": 45, "blauw": 50, "groen": 35, "geel": 15 } }
                ]
            }"##,
            );
            p.doe(Opdracht::LeesInvoer {
                bestand: "proef.json".into(),
                ontvangen: "nu".into(),
                invoer,
            })
            .unwrap();
            let lijst = ["groen", "rood", "blauw", "geel"]
                .map(String::from)
                .to_vec();
            p.doe(Opdracht::ZetUitslagvolgorde {
                volgorde: Uitslagvolgorde {
                    lijst,
                    ..Default::default()
                },
            })
            .unwrap();
            p
        }

        fn doe(&mut self, opdracht: Opdracht) -> Result<(), Weigering> {
            let mut nieuw = self.stand.clone();
            let h = Herkomst {
                door: "Regie 1",
                volgnummer: self.volgnummer + 1,
                tijd_ms: 1_000,
            };
            nieuw.voer_uit(&opdracht, &h)?;
            self.stand = nieuw;
            self.volgnummer += 1;
            Ok(())
        }

        fn toon(&mut self, graphic: Graphic) {
            self.doe(Opdracht::ZetKlaar {
                graphic,
                standpunt: None,
            })
            .unwrap();
            self.doe(Opdracht::Toon { versie: None }).unwrap();
        }

        fn zaal(&self, set: &str) -> Vec<gedeeld::Kleur> {
            zaal_voor_set(&self.stand.uitslagindeling, &self.stand.cijfers, set).unwrap()
        }

        /// De Blokken van de Graphic in de Ether, zoals de Renderer ze invult.
        fn ether(&self) -> Vec<Inhoud> {
            let g = &self.stand.ether.as_ref().expect("in de Ether").graphic;
            let invulling = g.invulling();
            let sjabloon = vast_sjabloon(&invulling.sjabloon).expect("vast Sjabloon");
            let cijfers = g.cijfers().expect("de Graphic houdt haar cijfers vast");
            sjabloon
                .vul_in(&invulling, &cijfers, &self.stand.stijl)
                .into_iter()
                .map(|b| b.inhoud)
                .collect()
        }

        /// De delen van elke Zetelbalk in de Ether, van boven naar onder.
        fn balken(&self) -> Vec<Vec<Zeteldeel>> {
            self.ether()
                .into_iter()
                .filter_map(|i| match i {
                    Inhoud::Zetelbalk { delen, .. } => Some(delen),
                    _ => None,
                })
                .collect()
        }
    }

    /// Korte naam, Zetels en pijltje van elk deel.
    fn kort(delen: &[Zeteldeel]) -> Vec<(&str, u16, Option<Pijl>)> {
        delen
            .iter()
            .map(|d| (d.kort.as_str(), d.zetels, d.pijl))
            .collect()
    }

    #[test]
    fn een_zetelbalk_met_de_grootste_twee_kleurt_de_stoelen_zoals_de_zetelverdeling() {
        let mut p = Proef::new();
        p.toon(Graphic::Zetelbalk {
            set: "exitpoll".into(),
            grootste: Some(2),
            cijfers: None,
        });
        assert_eq!(
            p.stand.stoelkleuren,
            p.zaal("exitpoll"),
            "alle Partijen in de Zaal, ook overig"
        );
        let balken = p.balken();
        assert_eq!(balken.len(), 1);
        assert_eq!(
            kort(&balken[0]),
            [
                ("ROOD", 50, None),
                ("BLAUW", 40, None),
                ("overig", 50, None),
                ("", 10, None)
            ],
            "de grootste twee op volgorde, overig, en wat nog niet verdeeld is"
        );
        assert_eq!(balken[0][2].partij.as_deref(), Some(OVERIG));
        assert_eq!(balken[0][3].partij, None);
    }

    #[test]
    fn twee_sets_de_oudste_boven_met_pijltjes_onder_en_de_nieuwste_in_de_zaal() {
        let mut p = Proef::new();
        // de Regie kiest de nieuwste eerst: het systeem zet de oudste boven
        p.toon(Graphic::TweeSets {
            set: "prognose".into(),
            tweede: "exitpoll".into(),
            grootste: Some(2),
            cijfers: None,
        });
        assert_eq!(p.stand.stoelkleuren, p.zaal("prognose"));
        let balken = p.balken();
        assert_eq!(balken.len(), 2);
        // de grootste twee van de nieuwste set: blauw (50) en rood (45), op volgorde
        assert_eq!(
            kort(&balken[0]),
            [
                ("ROOD", 50, None),
                ("BLAUW", 40, None),
                ("overig", 50, None),
                ("", 10, None)
            ]
        );
        assert_eq!(
            kort(&balken[1]),
            [
                ("ROOD", 45, Some(Pijl::Omlaag(5))),
                ("BLAUW", 50, Some(Pijl::Omhoog(10))),
                ("overig", 50, Some(Pijl::Gelijk)),
                ("", 5, None),
            ]
        );
    }

    #[test]
    fn twee_sets_krijgen_nieuwere_cijfers_als_een_van_beide_verandert() {
        let mut p = Proef::new();
        p.toon(Graphic::TweeSets {
            set: "exitpoll".into(),
            tweede: "prognose".into(),
            grootste: None,
            cijfers: None,
        });
        let g = p.stand.ether.clone().unwrap().graphic;
        assert!(!p.stand.nieuwere_cijfers(&g));
        p.doe(Opdracht::ZetZetels {
            set: "exitpoll".into(),
            partij: "geel".into(),
            zetels: 30,
        })
        .unwrap();
        assert!(p.stand.nieuwere_cijfers(&g), "de vorige set veranderde");
        p.doe(Opdracht::Bijwerken).unwrap();
        assert_eq!(
            p.balken()[0]
                .iter()
                .find(|d| d.kort == "GEEL")
                .unwrap()
                .zetels,
            30
        );
    }

    #[test]
    fn twee_keer_dezelfde_set_wordt_geweigerd() {
        let mut p = Proef::new();
        let g = Graphic::TweeSets {
            set: "exitpoll".into(),
            tweede: "exitpoll".into(),
            grootste: None,
            cijfers: None,
        };
        assert_eq!(
            p.doe(Opdracht::ZetKlaar {
                graphic: g,
                standpunt: None
            }),
            Err(Weigering::TweeKeerDezelfdeSet)
        );
    }

    #[test]
    fn de_uitslag_per_partij_staat_van_groot_naar_klein_met_overig_onderaan() {
        let mut p = Proef::new();
        p.toon(Graphic::UitslagPerPartij {
            set: "prognose".into(),
            grootste: Some(3),
            cijfers: None,
        });
        assert_eq!(p.stand.stoelkleuren, p.zaal("prognose"));
        let Some(Inhoud::Uitslaggrafiek {
            rijen,
            schaal,
            stap,
            ..
        }) = p
            .ether()
            .into_iter()
            .find(|i| matches!(i, Inhoud::Uitslaggrafiek { .. }))
        else {
            panic!("geen Uitslaggrafiek")
        };
        assert_eq!(
            kort(&rijen),
            [
                ("BLAUW", 50, None),
                ("ROOD", 45, None),
                ("GROEN", 35, None),
                ("overig", 15, None)
            ]
        );
        assert_eq!((schaal, stap), (55, 10));
    }
}

#[test]
fn de_as_van_de_uitslaggrafiek_heeft_ruimte_achter_de_grootste_balk() {
    use gedeeld::zetelbalk::as_van;
    // zoals de mockup: 26 Zetels op een as tot 30, rasterlijnen om de 5
    assert_eq!(as_van(26), (30, 5));
    assert_eq!(as_van(30), (35, 5));
    assert_eq!(as_van(46), (50, 5));
    // een grote Partij: rasterlijnen om de 10
    assert_eq!(as_van(50), (55, 10));
    assert_eq!(as_van(0), (5, 5));
}
