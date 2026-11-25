//! Stemmingen: de Stemmingslijst, de invoer "volg de Voorzitter", de uitkomst uit de Zetels en
//! Vastleggen en klaarzetten. Alles via `Stand::voer_uit`, zoals de server het doet.
//! De Kamer hier is voorbeeldgegevens: vijf Fracties die samen 150 Zetels hebben.

use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use gedeeld::stemmingen::{
    AfwijkendeStem, Besluit, GetikteFractie, Lijstpunt, Stem, Stemgedrag, StemmingWeigering,
    Uitkomst,
};
use gedeeld::{Graphic, Herkomst, Opdracht, Stand, Weigering};

/// Voorbeeldgegevens: (id, afkorting, Zetels).
const FRACTIES: [(&str, &str, u16); 5] = [
    ("f-pvv", "PVV", 40),
    ("f-gl", "GL-PvdA", 35),
    ("f-vvd", "VVD", 30),
    ("f-d66", "D66", 25),
    ("f-sp", "SP", 20),
];

fn kamerlid(id: &str, achternaam: &str, fractie: &str) -> Kamerlid {
    Kamerlid {
        id: id.into(),
        roepnaam: "Voorbeeld".into(),
        initialen: "V.".into(),
        tussenvoegsel: None,
        achternaam: achternaam.into(),
        fractie: fractie.into(),
        zetel: format!("zetel-{id}"),
        fractievoorzitter: false,
        van: "2025-11-12".into(),
        geboortedatum: None,
        geboorteplaats: None,
        woonplaats: None,
        foto: Foto::Geen,
        in_de_kamer_sinds: None,
        kieslijst: None,
    }
}

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    /// Een Stand met de voorbeeld-Kamer en de stemmingslijst van een voorbeelddag met twee moties.
    fn nieuw() -> Proef {
        let mut p = Proef {
            stand: Stand::default(),
            volgnummer: 0,
        };
        p.doe(Opdracht::LeesKamer {
            opgehaald: "2026-11-24T08:00:00+01:00".into(),
            peildatum: "2026-11-24".into(),
            kamerleden: vec![
                kamerlid("k-vvd", "Afwijker", "f-vvd"),
                kamerlid("k-pvv", "Tegenstemmer", "f-pvv"),
            ],
            fracties: FRACTIES
                .iter()
                .map(|(id, afk, zetels)| Fractie {
                    id: (*id).into(),
                    afkorting: (*afk).into(),
                    naam: format!("Fractie {afk}"),
                    zetels: *zetels,
                })
                .collect(),
        })
        .unwrap();
        p.doe(Opdracht::LeesStemmingslijst {
            dag: "2026-11-24".into(),
            ingelezen: "2026-11-24T08:00:00+01:00".into(),
            punten: vec![
                punt("m1", "Motie van het lid Een over de eerste zaak"),
                punt("m2", "Motie van het lid Twee over de tweede zaak"),
            ],
        })
        .unwrap();
        p
    }

    fn doe(&mut self, o: Opdracht) -> Result<(), Weigering> {
        self.volgnummer += 1;
        let h = Herkomst {
            door: "Regie links",
            volgnummer: self.volgnummer,
            tijd_ms: 0,
        };
        self.stand.voer_uit(&o, &h)
    }

    fn stem(&mut self, stemming: &str, stemgedrag: Stemgedrag) -> Result<(), Weigering> {
        self.doe(Opdracht::ZetStemgedrag {
            stemming: stemming.into(),
            stemgedrag,
        })
    }

    fn uitkomst(&self, stemming: &str) -> Uitkomst {
        self.stand
            .stemmingen
            .stemming(stemming)
            .expect("Stemming bestaat")
            .uitkomst
            .clone()
    }
}

fn punt(id: &str, titel: &str) -> Lijstpunt {
    Lijstpunt {
        id: id.into(),
        soort: Some("Motie".into()),
        nummer: Some("37020-XX, nr. 1".into()),
        titel: titel.into(),
        indieners: None,
        agendapunt: Some("Moties ingediend bij het voorbeelddebat".into()),
    }
}

fn getikt(fracties: &[(&str, Stem)]) -> Vec<GetikteFractie> {
    fracties
        .iter()
        .map(|(f, s)| GetikteFractie {
            fractie: (*f).into(),
            stem: *s,
        })
        .collect()
}

fn uitkomst(voor: u16, tegen: u16, open: u16, besluit: Option<Besluit>) -> Uitkomst {
    Uitkomst {
        voor,
        tegen,
        open,
        besluit,
    }
}

#[test]
fn getikte_fracties_en_overige_tegen_geven_de_uitkomst_uit_de_zetels() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[("f-pvv", Stem::Voor), ("f-vvd", Stem::Voor)]),
            overige: Some(Stem::Tegen),
            ..Default::default()
        },
    )
    .unwrap();
    // PVV 40 + VVD 30 voor; GL-PvdA 35 + D66 25 + SP 20 tegen
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(70, 80, 0, Some(Besluit::Verworpen))
    );

    p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[
                ("f-pvv", Stem::Voor),
                ("f-vvd", Stem::Voor),
                ("f-d66", Stem::Voor),
            ]),
            overige: Some(Stem::Tegen),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(95, 55, 0, Some(Besluit::Aangenomen))
    );
}

#[test]
fn zonder_overige_blijven_fracties_open_en_is_er_nog_geen_besluit() {
    let mut p = Proef::nieuw();
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(0, 0, 150, None),
        "een nieuwe Stemming is helemaal open"
    );
    p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[("f-pvv", Stem::Voor), ("f-sp", Stem::Tegen)]),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(p.uitkomst("m1"), uitkomst(40, 20, 90, None));
}

#[test]
fn overige_voor_vult_de_rest_met_voor() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[("f-sp", Stem::Tegen), ("f-d66", Stem::Tegen)]),
            overige: Some(Stem::Voor),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(105, 45, 0, Some(Besluit::Aangenomen))
    );
}

#[test]
fn bij_staken_der_stemmen_is_de_motie_verworpen() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[("f-pvv", Stem::Voor), ("f-gl", Stem::Voor)]),
            overige: Some(Stem::Tegen),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(75, 75, 0, Some(Besluit::Verworpen))
    );
}

#[test]
fn met_algemene_stemmen_stemt_elke_fractie_voor() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            algemene_stemmen: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(150, 0, 0, Some(Besluit::Aangenomen))
    );
}

#[test]
fn algemene_stemmen_met_een_uitzondering_wordt_geweigerd() {
    let mut p = Proef::nieuw();
    let fout = p.stem(
        "m1",
        Stemgedrag {
            algemene_stemmen: true,
            getikt: getikt(&[("f-sp", Stem::Tegen)]),
            ..Default::default()
        },
    );
    assert_eq!(
        fout,
        Err(Weigering::Stemming(
            StemmingWeigering::AlgemeneStemmenMetUitzondering
        ))
    );
    let fout = p.stem(
        "m1",
        Stemgedrag {
            algemene_stemmen: true,
            afwijkend: vec![AfwijkendeStem {
                kamerlid: "k-vvd".into(),
                stem: Stem::Tegen,
            }],
            ..Default::default()
        },
    );
    assert_eq!(
        fout,
        Err(Weigering::Stemming(
            StemmingWeigering::AlgemeneStemmenMetUitzondering
        ))
    );
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(0, 0, 150, None),
        "een weigering verandert niets"
    );
}

#[test]
fn een_afwijkende_stem_verschuift_een_zetel_naar_de_andere_kant() {
    let mut p = Proef::nieuw();
    let basis = Stemgedrag {
        getikt: getikt(&[("f-pvv", Stem::Voor)]),
        overige: Some(Stem::Tegen),
        ..Default::default()
    };
    // Een Kamerlid van de VVD (die tegen stemt) stemt voor.
    p.stem(
        "m1",
        Stemgedrag {
            afwijkend: vec![AfwijkendeStem {
                kamerlid: "k-vvd".into(),
                stem: Stem::Voor,
            }],
            ..basis.clone()
        },
    )
    .unwrap();
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(41, 109, 0, Some(Besluit::Verworpen))
    );

    // En een Kamerlid van de PVV (die voor stemt) stemt tegen.
    p.stem(
        "m1",
        Stemgedrag {
            afwijkend: vec![
                AfwijkendeStem {
                    kamerlid: "k-vvd".into(),
                    stem: Stem::Voor,
                },
                AfwijkendeStem {
                    kamerlid: "k-pvv".into(),
                    stem: Stem::Tegen,
                },
            ],
            ..basis
        },
    )
    .unwrap();
    assert_eq!(
        p.uitkomst("m1"),
        uitkomst(40, 110, 0, Some(Besluit::Verworpen))
    );
}

#[test]
fn een_afwijkende_stem_in_een_open_fractie_telt_mee_en_de_rest_blijft_open() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            afwijkend: vec![AfwijkendeStem {
                kamerlid: "k-vvd".into(),
                stem: Stem::Tegen,
            }],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(p.uitkomst("m1"), uitkomst(0, 1, 149, None));
}

#[test]
fn een_onbekende_of_dubbele_fractie_of_onbekend_kamerlid_wordt_geweigerd() {
    let mut p = Proef::nieuw();
    let fout = p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[("f-bestaat-niet", Stem::Voor)]),
            ..Default::default()
        },
    );
    assert_eq!(
        fout,
        Err(Weigering::Stemming(StemmingWeigering::OnbekendeFractie(
            "f-bestaat-niet".into()
        )))
    );
    let fout = p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[("f-sp", Stem::Voor), ("f-sp", Stem::Tegen)]),
            ..Default::default()
        },
    );
    assert_eq!(
        fout,
        Err(Weigering::Stemming(StemmingWeigering::FractieTweeKeer(
            "SP".into()
        )))
    );
    let fout = p.stem(
        "m1",
        Stemgedrag {
            afwijkend: vec![AfwijkendeStem {
                kamerlid: "k-niemand".into(),
                stem: Stem::Voor,
            }],
            ..Default::default()
        },
    );
    assert_eq!(
        fout,
        Err(Weigering::Stemming(StemmingWeigering::OnbekendKamerlid(
            "k-niemand".into()
        )))
    );
    let fout = p.stem("m9", Stemgedrag::default());
    assert_eq!(
        fout,
        Err(Weigering::Stemming(StemmingWeigering::OnbekendeStemming(
            "m9".into()
        )))
    );
}

#[test]
fn vastleggen_en_klaarzetten_zet_de_uitslag_met_de_stemming_als_gegevens_klaar() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[
                ("f-pvv", Stem::Voor),
                ("f-vvd", Stem::Voor),
                ("f-d66", Stem::Voor),
            ]),
            overige: Some(Stem::Tegen),
            afwijkend: vec![AfwijkendeStem {
                kamerlid: "k-pvv".into(),
                stem: Stem::Tegen,
            }],
            ..Default::default()
        },
    )
    .unwrap();
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m1".into(),
    })
    .unwrap();

    let klaar = p.stand.klaar.clone().expect("er staat een Graphic klaar");
    assert_eq!(klaar.klaargezet_door, "Regie links");
    let Graphic::UitslagStemming { uitslag } = &klaar.graphic else {
        panic!(
            "verwacht de Uitslag van een Stemming, kreeg {:?}",
            klaar.graphic
        )
    };
    assert_eq!(uitslag.stemming, "m1");
    assert_eq!(uitslag.titel, "Motie van het lid Een over de eerste zaak");
    assert_eq!(
        (uitslag.voor, uitslag.tegen, uitslag.besluit),
        (94, 56, Besluit::Aangenomen)
    );
    let stemmen: Vec<(&str, Stem)> = uitslag
        .fracties
        .iter()
        .map(|f| (f.afkorting.as_str(), f.stem))
        .collect();
    assert_eq!(
        stemmen,
        [
            ("PVV", Stem::Voor),
            ("GL-PvdA", Stem::Tegen),
            ("VVD", Stem::Voor),
            ("D66", Stem::Voor),
            ("SP", Stem::Tegen)
        ]
    );
    assert_eq!(uitslag.afwijkend.len(), 1);
    assert_eq!(uitslag.afwijkend[0].naam, "Voorbeeld Tegenstemmer");
    assert_eq!(uitslag.afwijkend[0].fractie, "PVV");
    assert_eq!(uitslag.afwijkend[0].stem, Stem::Tegen);

    assert_eq!(uitslag.samenvatting(), "Aangenomen: 94 voor, 56 tegen");
    // Het eigen Sjabloon, met de titel zonder indieners als kop.
    let invulling = klaar.graphic.invulling();
    assert_eq!(invulling.sjabloon, "uitslag-stemming");
    assert_eq!(invulling.velden["titel"], "Motie over de eerste zaak");
    assert_eq!(invulling.stemming.as_ref(), Some(uitslag));
    assert!(p.stand.stemmingen.stemming("m1").unwrap().vastgelegd);
}

#[test]
fn een_hoofdelijke_stemming_zet_de_hoofdelijke_stemming_klaar() {
    let mut p = Proef::nieuw();
    p.stem(
        "m2",
        Stemgedrag {
            getikt: getikt(&[("f-gl", Stem::Tegen)]),
            overige: Some(Stem::Voor),
            hoofdelijk: true,
            ..Default::default()
        },
    )
    .unwrap();
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m2".into(),
    })
    .unwrap();
    let klaar = p.stand.klaar.clone().unwrap();
    assert!(
        matches!(klaar.graphic, Graphic::HoofdelijkeStemming { .. }),
        "{:?}",
        klaar.graphic
    );
}

#[test]
fn met_algemene_stemmen_vastgelegd_heet_ook_zo_in_beeld() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            algemene_stemmen: true,
            ..Default::default()
        },
    )
    .unwrap();
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m1".into(),
    })
    .unwrap();
    let Some(Graphic::UitslagStemming { uitslag }) = p.stand.klaar.map(|k| k.graphic) else {
        panic!()
    };
    assert!(uitslag.algemene_stemmen);
    assert_eq!(uitslag.samenvatting(), "Met algemene stemmen aangenomen");
}

#[test]
fn vastleggen_weigert_zolang_er_fracties_open_zijn() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[("f-pvv", Stem::Voor)]),
            ..Default::default()
        },
    )
    .unwrap();
    let fout = p.doe(Opdracht::LegStemmingVast {
        stemming: "m1".into(),
    });
    assert_eq!(
        fout,
        Err(Weigering::Stemming(StemmingWeigering::FractiesOpen(110)))
    );
    assert!(p.stand.klaar.is_none());
    assert!(!p.stand.stemmingen.stemming("m1").unwrap().vastgelegd);
}

#[test]
fn opnieuw_vastleggen_van_dezelfde_stemming_wijzigt_de_klaargezette_graphic() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            algemene_stemmen: true,
            ..Default::default()
        },
    )
    .unwrap();
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m1".into(),
    })
    .unwrap();
    let eerst = p.stand.klaar.clone().unwrap();
    p.doe(Opdracht::Toon {
        versie: Some(eerst.versie),
    })
    .unwrap();

    // Een correctie: toch niet met algemene stemmen.
    p.stem(
        "m1",
        Stemgedrag {
            getikt: getikt(&[("f-sp", Stem::Tegen)]),
            overige: Some(Stem::Voor),
            ..Default::default()
        },
    )
    .unwrap();
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m1".into(),
    })
    .unwrap();
    let daarna = p.stand.klaar.clone().unwrap();
    assert_eq!(
        daarna.id, eerst.id,
        "dezelfde Graphic, zodat Bijwerken de Ether bijwerkt"
    );
    assert_ne!(daarna.versie, eerst.versie);
    assert!(p.stand.kan_bijwerken());

    // Een andere Stemming vastleggen is een nieuwe Graphic.
    p.stem(
        "m2",
        Stemgedrag {
            algemene_stemmen: true,
            ..Default::default()
        },
    )
    .unwrap();
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m2".into(),
    })
    .unwrap();
    assert_ne!(p.stand.klaar.clone().unwrap().id, eerst.id);
}

#[test]
fn een_vergrendelde_stemming_weigert_ander_stemgedrag_tot_ontgrendelen() {
    let mut p = Proef::nieuw();
    p.stem(
        "m1",
        Stemgedrag {
            algemene_stemmen: true,
            ..Default::default()
        },
    )
    .unwrap();
    p.doe(Opdracht::VergrendelStemming {
        stemming: "m1".into(),
    })
    .unwrap();
    let fout = p.stem("m1", Stemgedrag::default());
    assert_eq!(
        fout,
        Err(Weigering::Stemming(StemmingWeigering::Vergrendeld))
    );
    assert_eq!(p.uitkomst("m1").besluit, Some(Besluit::Aangenomen));
    // Klaarzetten van een vergrendelde Stemming mag wel: dat verandert de invoer niet.
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m1".into(),
    })
    .unwrap();

    p.doe(Opdracht::OntgrendelStemming {
        stemming: "m1".into(),
    })
    .unwrap();
    p.stem("m1", Stemgedrag::default()).unwrap();
}

#[test]
fn de_regie_vinkt_stemmingen_aan_voor_de_uitzending() {
    let mut p = Proef::nieuw();
    assert!(p.stand.stemmingen.lijst.iter().all(|s| !s.voor_uitzending));
    p.doe(Opdracht::ZetVoorUitzending {
        stemming: "m2".into(),
        aan: true,
    })
    .unwrap();
    let aan: Vec<&str> = p
        .stand
        .stemmingen
        .lijst
        .iter()
        .filter(|s| s.voor_uitzending)
        .map(|s| s.punt.id.as_str())
        .collect();
    assert_eq!(aan, ["m2"]);
}

#[test]
fn een_motie_die_niet_in_de_lijst_staat_typt_de_regie_zelf_in() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::VoegStemmingToe {
        titel: "  Motie over de voorbeeldzaak ".into(),
    })
    .unwrap();
    let hand = p.stand.stemmingen.lijst.last().unwrap();
    assert_eq!(hand.punt.titel, "Motie over de voorbeeldzaak");
    assert!(hand.met_de_hand);
    assert!(
        hand.voor_uitzending,
        "met de hand toegevoegd is voor de uitzending"
    );
    assert_eq!(hand.uitkomst, uitkomst(0, 0, 150, None));
    let id = hand.punt.id.clone();
    p.stem(
        &id,
        Stemgedrag {
            algemene_stemmen: true,
            ..Default::default()
        },
    )
    .unwrap();

    let fout = p.doe(Opdracht::VoegStemmingToe {
        titel: "   ".into(),
    });
    assert_eq!(fout, Err(Weigering::Stemming(StemmingWeigering::TitelLeeg)));
}

#[test]
fn opnieuw_inlezen_van_dezelfde_dag_houdt_de_invoer_van_de_regie() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::ZetVoorUitzending {
        stemming: "m1".into(),
        aan: true,
    })
    .unwrap();
    p.stem(
        "m1",
        Stemgedrag {
            algemene_stemmen: true,
            ..Default::default()
        },
    )
    .unwrap();
    p.doe(Opdracht::VoegStemmingToe {
        titel: "Met de hand".into(),
    })
    .unwrap();

    // m2 is van de lijst gehaald (bijvoorbeeld aangehouden), m3 kwam erbij.
    p.doe(Opdracht::LeesStemmingslijst {
        dag: "2026-11-24".into(),
        ingelezen: "2026-11-24T10:00:00+01:00".into(),
        punten: vec![
            punt("m1", "Gewijzigde titel"),
            punt("m3", "Motie van het lid Drie over de derde zaak"),
        ],
    })
    .unwrap();
    let s = &p.stand.stemmingen;
    let ids: Vec<&str> = s.lijst.iter().map(|s| s.punt.id.as_str()).collect();
    assert_eq!(ids[..2], ["m1", "m3"]);
    assert_eq!(ids.len(), 3, "de motie met de hand blijft staan: {ids:?}");
    let m1 = s.stemming("m1").unwrap();
    assert_eq!(m1.punt.titel, "Gewijzigde titel");
    assert!(m1.voor_uitzending);
    assert_eq!(m1.uitkomst.besluit, Some(Besluit::Aangenomen));
    assert_eq!(s.ingelezen.as_deref(), Some("2026-11-24T10:00:00+01:00"));
}

#[test]
fn een_nieuwe_dag_zonder_stemmingen_laat_de_lijst_van_de_vorige_dag_staan() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::LeesStemmingslijst {
        dag: "2026-11-25".into(),
        ingelezen: "2026-11-25T00:10:00+01:00".into(),
        punten: vec![],
    })
    .unwrap();
    assert_eq!(p.stand.stemmingen.dag.as_deref(), Some("2026-11-24"));
    assert_eq!(p.stand.stemmingen.lijst.len(), 2);

    // Een nieuwe dag met Stemmingen vervangt de lijst.
    p.doe(Opdracht::LeesStemmingslijst {
        dag: "2026-11-26".into(),
        ingelezen: "2026-11-26T08:00:00+01:00".into(),
        punten: vec![punt("m7", "Motie van het lid Zeven over de zevende zaak")],
    })
    .unwrap();
    assert_eq!(p.stand.stemmingen.dag.as_deref(), Some("2026-11-26"));
    let ids: Vec<&str> = p
        .stand
        .stemmingen
        .lijst
        .iter()
        .map(|s| s.punt.id.as_str())
        .collect();
    assert_eq!(ids, ["m7"]);
}
