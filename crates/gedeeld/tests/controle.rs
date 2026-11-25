//! Controle na de uitzending: de officiële uitslag naast de invoer van de Regie, de vier
//! statussen, "Neem officieel over" en "Onze invoer klopt". Alles via `Stand::voer_uit`, zoals de
//! server het doet. De Kamer hier is voorbeeldgegevens: vijf Fracties die samen 150 Zetels hebben.
//! De echte antwoorden van Debat Direct en de open data staan in de tests van de server.

use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use gedeeld::stemmingen::{
    AfwijkendeStem, Besluit, Bron, Controlestatus, GetikteFractie, Keuze, Lijstpunt,
    OfficieleUitslag, OpgehaaldeUitslag, Stem, Stemgedrag, StemmingWeigering, Verschil,
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
    /// De voorbeeld-Kamer en een voorbeelddag met vier moties.
    fn nieuw() -> Proef {
        let mut p = Proef {
            stand: Stand::default(),
            volgnummer: 0,
        };
        p.doe(Opdracht::LeesKamer {
            opgehaald: "2026-11-24T08:00:00+01:00".into(),
            peildatum: "2026-11-24".into(),
            kamerleden: vec![kamerlid("k-vvd", "Afwijker", "f-vvd")],
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
            punten: ["m1", "m2", "m3", "m4"].iter().map(|id| punt(id)).collect(),
        })
        .unwrap();
        p
    }

    fn doe(&mut self, o: Opdracht) -> Result<(), Weigering> {
        self.volgnummer += 1;
        let h = Herkomst {
            door: "Regie links",
            volgnummer: self.volgnummer,
            tijd_ms: 1_000 * self.volgnummer,
        };
        self.stand.voer_uit(&o, &h)
    }

    /// PVV, VVD en D66 voor, de rest tegen: aangenomen met 95 tegen 55.
    fn voer_in(&mut self, stemming: &str, voor: &[&str]) {
        let sg = Stemgedrag {
            getikt: getikt(voor, Stem::Voor),
            overige: Some(Stem::Tegen),
            ..Default::default()
        };
        self.doe(Opdracht::ZetStemgedrag {
            stemming: stemming.into(),
            stemgedrag: sg,
        })
        .unwrap();
    }

    fn officieel(&mut self, opgehaald: &str, uitslagen: Vec<OpgehaaldeUitslag>) {
        self.doe(Opdracht::LeesOfficieleUitslagen {
            opgehaald: opgehaald.into(),
            uitslagen,
        })
        .unwrap();
    }

    fn status(&self, stemming: &str) -> Option<Controlestatus> {
        self.stand
            .stemmingen
            .stemming(stemming)
            .unwrap()
            .controle
            .status
    }
}

fn punt(id: &str) -> Lijstpunt {
    Lijstpunt {
        id: id.into(),
        soort: Some("Motie".into()),
        nummer: Some(format!("37020-XX, nr. {}", &id[1..])),
        titel: format!("Motie van het lid Voorbeeld over zaak {id}"),
        indieners: Some("Voorbeeld".into()),
        agendapunt: None,
    }
}

fn getikt(fracties: &[&str], stem: Stem) -> Vec<GetikteFractie> {
    fracties
        .iter()
        .map(|f| GetikteFractie {
            fractie: (*f).into(),
            stem,
        })
        .collect()
}

/// Een officiële uitslag waarin `voor` voor stemde en de rest tegen.
fn uitslag(stemming: &str, bron: Bron, besluit: Besluit, voor: &[&str]) -> OpgehaaldeUitslag {
    OpgehaaldeUitslag {
        stemming: stemming.into(),
        uitslag: OfficieleUitslag {
            bron,
            besluit,
            hoofdelijk: false,
            fracties: FRACTIES
                .iter()
                .map(|(f, ..)| GetikteFractie {
                    fractie: (*f).into(),
                    stem: if voor.contains(f) {
                        Stem::Voor
                    } else {
                        Stem::Tegen
                    },
                })
                .collect(),
            afwijkend: vec![],
            hamerslag: Some("2026-11-24T15:06:44+01:00".into()),
        },
    }
}

const PVV_VVD_D66: [&str; 3] = ["f-pvv", "f-vvd", "f-d66"];

#[test]
fn elke_ingevoerde_stemming_krijgt_een_van_de_vier_statussen() {
    let mut p = Proef::nieuw();
    p.voer_in("m1", &PVV_VVD_D66); // 95 voor: aangenomen
    p.voer_in("m2", &PVV_VVD_D66);
    p.voer_in("m3", &PVV_VVD_D66);
    // m4 heeft geen invoer: die hoort niet in de controle.
    assert_eq!(p.status("m1"), Some(Controlestatus::WachtOpOfficieel));
    assert_eq!(p.status("m4"), None);

    p.officieel(
        "2026-11-24T17:10:00+01:00",
        vec![
            // Gelijk aan de invoer.
            uitslag("m1", Bron::DebatDirect, Besluit::Aangenomen, &PVV_VVD_D66),
            // D66 stemde echt tegen: 70 voor, de motie is verworpen.
            uitslag(
                "m2",
                Bron::DebatDirect,
                Besluit::Verworpen,
                &["f-pvv", "f-vvd"],
            ),
            // SP stemde echt ook voor: 115 voor, nog steeds aangenomen.
            uitslag(
                "m3",
                Bron::DebatDirect,
                Besluit::Aangenomen,
                &["f-pvv", "f-vvd", "f-d66", "f-sp"],
            ),
            uitslag("m4", Bron::DebatDirect, Besluit::Verworpen, &[]),
        ],
    );

    assert_eq!(p.status("m1"), Some(Controlestatus::Gelijk));
    assert_eq!(p.status("m2"), Some(Controlestatus::UitkomstAnders));
    assert_eq!(p.status("m3"), Some(Controlestatus::VerschilPerFractie));
    assert_eq!(
        p.status("m4"),
        None,
        "zonder invoer geen status, ook niet met een officiële uitslag"
    );

    let m2 = &p.stand.stemmingen.stemming("m2").unwrap().controle;
    assert_eq!(
        m2.verschillen,
        [Verschil {
            wie: "D66".into(),
            kamerlid: None,
            onze: Some(Stem::Voor),
            officieel: Some(Stem::Tegen)
        }]
    );
    let o = m2.officiele_uitkomst.clone().unwrap();
    assert_eq!(
        (o.voor, o.tegen, o.besluit),
        (70, 80, Some(Besluit::Verworpen))
    );
}

#[test]
fn een_afwijkende_stem_die_de_regie_miste_is_een_verschil_per_fractie() {
    let mut p = Proef::nieuw();
    p.voer_in("m1", &PVV_VVD_D66);
    let mut u = uitslag("m1", Bron::OpenData, Besluit::Aangenomen, &PVV_VVD_D66);
    u.uitslag.afwijkend = vec![AfwijkendeStem {
        kamerlid: "k-vvd".into(),
        stem: Stem::Tegen,
    }];
    p.officieel("2026-11-25T09:30:00+01:00", vec![u]);

    assert_eq!(p.status("m1"), Some(Controlestatus::VerschilPerFractie));
    assert_eq!(
        p.stand
            .stemmingen
            .stemming("m1")
            .unwrap()
            .controle
            .verschillen,
        [Verschil {
            wie: "Voorbeeld Afwijker (VVD)".into(),
            kamerlid: Some("k-vvd".into()),
            onze: Some(Stem::Voor),
            officieel: Some(Stem::Tegen),
        }]
    );
}

#[test]
fn de_server_overschrijft_de_invoer_nooit_vanzelf() {
    let mut p = Proef::nieuw();
    p.voer_in("m2", &PVV_VVD_D66);
    let voor = p
        .stand
        .stemmingen
        .stemming("m2")
        .unwrap()
        .stemgedrag
        .clone();
    p.officieel(
        "2026-11-24T17:10:00+01:00",
        vec![uitslag(
            "m2",
            Bron::DebatDirect,
            Besluit::Verworpen,
            &["f-pvv"],
        )],
    );

    let m2 = p.stand.stemmingen.stemming("m2").unwrap();
    assert_eq!(m2.stemgedrag, voor);
    assert_eq!(m2.uitkomst.besluit, Some(Besluit::Aangenomen));
}

#[test]
fn neem_officieel_over_zet_het_officiele_stemgedrag_in_de_invoer() {
    let mut p = Proef::nieuw();
    p.voer_in("m2", &PVV_VVD_D66);
    p.officieel(
        "2026-11-24T17:10:00+01:00",
        vec![uitslag(
            "m2",
            Bron::DebatDirect,
            Besluit::Verworpen,
            &["f-pvv", "f-vvd"],
        )],
    );
    // Ook een vergrendelde Stemming: overnemen is een bewuste keuze van de Regie.
    p.doe(Opdracht::VergrendelStemming {
        stemming: "m2".into(),
    })
    .unwrap();

    p.doe(Opdracht::NeemOfficieelOver {
        stemming: "m2".into(),
    })
    .unwrap();

    let m2 = p.stand.stemmingen.stemming("m2").unwrap();
    assert_eq!(m2.stemgedrag.stem_van("f-d66"), Some(Stem::Tegen));
    assert_eq!(
        (m2.uitkomst.voor, m2.uitkomst.tegen, m2.uitkomst.besluit),
        (70, 80, Some(Besluit::Verworpen))
    );
    assert_eq!(m2.controle.status, Some(Controlestatus::Gelijk));
    assert_eq!(m2.controle.afgehandeld, Some(Keuze::OfficieelOvergenomen));
    assert!(m2.vergrendeld);
}

#[test]
fn onze_invoer_klopt_laat_de_invoer_staan_en_markeert_het_verschil_als_afgehandeld() {
    let mut p = Proef::nieuw();
    p.voer_in("m3", &PVV_VVD_D66);
    p.officieel(
        "2026-11-24T17:10:00+01:00",
        vec![uitslag(
            "m3",
            Bron::DebatDirect,
            Besluit::Aangenomen,
            &["f-pvv", "f-vvd", "f-d66", "f-sp"],
        )],
    );

    p.doe(Opdracht::OnzeInvoerKlopt {
        stemming: "m3".into(),
    })
    .unwrap();

    let m3 = p.stand.stemmingen.stemming("m3").unwrap();
    assert_eq!(m3.stemgedrag.stem_van("f-sp"), Some(Stem::Tegen));
    assert_eq!(m3.controle.status, Some(Controlestatus::VerschilPerFractie));
    assert_eq!(m3.controle.afgehandeld, Some(Keuze::OnzeInvoerKlopt));

    // Komt er later een andere officiële uitslag (de open data na Debat Direct), dan geldt de
    // keuze niet meer: de Regie kijkt opnieuw.
    p.officieel(
        "2026-11-25T09:30:00+01:00",
        vec![uitslag(
            "m3",
            Bron::OpenData,
            Besluit::Aangenomen,
            &["f-pvv", "f-vvd", "f-d66", "f-sp", "f-gl"],
        )],
    );
    assert_eq!(
        p.stand
            .stemmingen
            .stemming("m3")
            .unwrap()
            .controle
            .afgehandeld,
        None
    );
}

#[test]
fn zonder_officiele_uitslag_weigert_de_server_beide_keuzes() {
    let mut p = Proef::nieuw();
    p.voer_in("m1", &PVV_VVD_D66);
    for o in [
        Opdracht::NeemOfficieelOver {
            stemming: "m1".into(),
        },
        Opdracht::OnzeInvoerKlopt {
            stemming: "m1".into(),
        },
    ] {
        assert_eq!(
            p.doe(o),
            Err(Weigering::Stemming(StemmingWeigering::GeenOfficieleUitslag))
        );
    }
}

#[test]
fn een_preset_en_de_ether_houden_de_oude_gegevens_na_neem_officieel_over() {
    let mut p = Proef::nieuw();
    p.voer_in("m2", &PVV_VVD_D66);
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m2".into(),
    })
    .unwrap();
    let graphic = p.stand.klaar.clone().unwrap().graphic;
    p.doe(Opdracht::BewaarPreset {
        graphic: graphic.clone(),
        standpunt: None,
    })
    .unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();

    p.officieel(
        "2026-11-24T17:10:00+01:00",
        vec![uitslag(
            "m2",
            Bron::DebatDirect,
            Besluit::Verworpen,
            &["f-pvv", "f-vvd"],
        )],
    );
    p.doe(Opdracht::NeemOfficieelOver {
        stemming: "m2".into(),
    })
    .unwrap();

    assert_eq!(p.stand.presets[0].graphic, graphic);
    assert_eq!(p.stand.ether.as_ref().unwrap().graphic, graphic);
    let Graphic::UitslagStemming { uitslag } = &p.stand.presets[0].graphic else {
        panic!()
    };
    assert_eq!(uitslag.besluit, Besluit::Aangenomen);
}

#[test]
fn de_controle_weet_of_en_wanneer_een_stemming_in_de_ether_was() {
    let mut p = Proef::nieuw();
    p.voer_in("m1", &PVV_VVD_D66);
    p.voer_in("m2", &PVV_VVD_D66);
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m1".into(),
    })
    .unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    let getoond = p.stand.ether.as_ref().unwrap().sinds_ms;
    p.doe(Opdracht::UitBeeld).unwrap();
    p.doe(Opdracht::LegStemmingVast {
        stemming: "m1".into(),
    })
    .unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();

    assert_eq!(
        p.stand.stemmingen.stemming("m1").unwrap().in_ether_ms,
        Some(getoond),
        "de eerste keer telt"
    );
    assert_eq!(p.stand.stemmingen.stemming("m2").unwrap().in_ether_ms, None);
}

#[test]
fn debat_direct_maakt_plaats_voor_de_open_data_maar_niet_andersom() {
    let mut p = Proef::nieuw();
    p.voer_in("m1", &PVV_VVD_D66);
    let dd = uitslag("m1", Bron::DebatDirect, Besluit::Aangenomen, &PVV_VVD_D66);
    p.officieel("2026-11-24T17:10:00+01:00", vec![dd.clone()]);
    p.officieel("2026-11-24T17:11:00+01:00", vec![dd.clone()]);
    let mut od = uitslag("m1", Bron::OpenData, Besluit::Aangenomen, &PVV_VVD_D66);
    od.uitslag.hamerslag = None;
    p.officieel("2026-11-25T09:30:00+01:00", vec![od]);
    p.officieel("2026-11-25T09:40:00+01:00", vec![dd]);

    let c = &p.stand.stemmingen.stemming("m1").unwrap().controle;
    let o = c.officieel.as_ref().unwrap();
    assert_eq!(o.bron, Bron::OpenData);
    // De hamerslag van Debat Direct blijft, en de eerste keer dat elke bron de uitslag had: zo
    // meet de eerste echte Stemmingsdag de vertraging van Debat Direct.
    assert_eq!(o.hamerslag.as_deref(), Some("2026-11-24T15:06:44+01:00"));
    assert_eq!(
        c.debat_direct_gezien.as_deref(),
        Some("2026-11-24T17:10:00+01:00")
    );
    assert_eq!(
        c.open_data_gezien.as_deref(),
        Some("2026-11-25T09:30:00+01:00")
    );
}
