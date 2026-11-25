//! De Sjablonen voor Stemmingen (#56): de Stoelen per stem in de Zaalindeling, de voorlopige
//! vulling in de Uitslagindeling, en wat de drie Sjablonen invullen. Alles via `Stand::voer_uit`.
//! VOORBEELDGEGEVENS: verzonnen Kamerleden en vijf Fracties die samen 150 Zetels hebben.

use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use gedeeld::sjabloon::{GRIJS, STEM_TEGEN, STEM_VOOR};
use gedeeld::stemmingen::{AfwijkendeStem, GetikteFractie, Lijstpunt, Stem, Stemgedrag};
use gedeeld::uitslagindeling::stoelen_van_links_naar_rechts;
use gedeeld::zaalindeling::{Bezetting, Zaalindeling};
use gedeeld::{Graphic, Herkomst, Indeling, Kleur, Opdracht, Stand, Stoel};

/// (id, afkorting, Zetels). Zonder partijlijst komen ze erbij als Fractie zonder lijst, met de
/// afkorting als sleutel; in de standaardvolgorde: sp, gl-pvda, d66, vvd, pvv.
const FRACTIES: [(&str, &str, u16); 5] = [
    ("f-pvv", "PVV", 40),
    ("f-gl", "GL-PvdA", 35),
    ("f-vvd", "VVD", 30),
    ("f-d66", "D66", 25),
    ("f-sp", "SP", 20),
];

fn lid(id: &str, achternaam: &str, fractie: &str, kieslijst: Option<u16>) -> Kamerlid {
    Kamerlid {
        id: id.into(),
        roepnaam: "Proef".into(),
        initialen: "P.".into(),
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
        kieslijst,
        in_de_kamer_sinds: None,
    }
}

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    /// De voorbeeld-Kamer met vier Kamerleden: drie van SP (kieslijst 2, 1 en 3) en één van PVV
    /// zonder plek op de kieslijst. Eén motie op de lijst.
    fn nieuw() -> Proef {
        let mut p = Proef { stand: Stand::default(), volgnummer: 0 };
        p.doe(Opdracht::LeesKamer {
            opgehaald: "2026-11-24T08:00:00+01:00".into(),
            peildatum: "2026-11-24".into(),
            kamerleden: vec![
                lid("k-sp2", "Tweede", "f-sp", Some(2)),
                lid("k-sp1", "Eerste", "f-sp", Some(1)),
                lid("k-sp3", "Derde", "f-sp", Some(3)),
                lid("k-pvv", "Zonderlijst", "f-pvv", None),
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
        });
        p.doe(Opdracht::LeesStemmingslijst {
            dag: "2026-11-24".into(),
            ingelezen: "2026-11-24T08:00:00+01:00".into(),
            punten: vec![Lijstpunt {
                id: "m1".into(),
                soort: Some("Motie".into()),
                nummer: Some("37020-XX, nr. 1".into()),
                titel: "Motie van het lid Een over gratis schoolfruit".into(),
                indieners: None,
                agendapunt: None,
            }],
        });
        p
    }

    fn doe(&mut self, o: Opdracht) {
        self.volgnummer += 1;
        let h = Herkomst { door: "Regie links", volgnummer: self.volgnummer, tijd_ms: 0 };
        self.stand.voer_uit(&o, &h).unwrap_or_else(|e| panic!("{o:?}: {e}"));
    }

    /// SP tegen, de rest voor; Kamerlid Eerste (SP) stemt voor.
    fn stem_en_leg_vast(&mut self, hoofdelijk: bool) -> Graphic {
        self.doe(Opdracht::ZetStemgedrag {
            stemming: "m1".into(),
            stemgedrag: Stemgedrag {
                getikt: vec![GetikteFractie { fractie: "f-sp".into(), stem: Stem::Tegen }],
                overige: Some(Stem::Voor),
                afwijkend: vec![AfwijkendeStem { kamerlid: "k-sp1".into(), stem: Stem::Voor }],
                hoofdelijk,
                ..Default::default()
            },
        });
        self.doe(Opdracht::LegStemmingVast { stemming: "m1".into() });
        self.stand.klaar.clone().expect("klaargezet").graphic
    }
}

#[test]
fn hoofdelijk_in_de_uitslagindeling_vult_de_kamerleden_voorlopig_op_volgorde_van_de_kieslijst() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::ZetStemmingIndeling { indeling: Indeling::Uitslagindeling });
    let g = p.stem_en_leg_vast(true);
    let Graphic::HoofdelijkeStemming { uitslag } = &g else { panic!("{g:?}") };
    assert!(uitslag.voorlopig, "het dashboard markeert dit als voorlopig");
    assert_eq!(uitslag.indeling, Indeling::Uitslagindeling);

    // SP staat links in de standaardvolgorde: haar blok zijn de eerste 20 Stoelen van links.
    // Daarin eerst haar Kamerleden op kieslijstvolgorde: Eerste (voor), Tweede, Derde (tegen),
    // daarna Stoelen zonder Kamerlid in de stem van de Fractie (tegen). De rest stemt voor.
    let links = stoelen_van_links_naar_rechts();
    let mut verwacht = vec![STEM_VOOR; 150];
    for &i in &links[1..20] {
        verwacht[i] = STEM_TEGEN;
    }
    assert_eq!(g.stoelkleuren().expect("kleurt de Stoelen"), verwacht);

    // De Namenlijst: de Kamerleden van SP op kieslijstvolgorde, met hun eigen stem.
    let sp: Vec<(&str, Stem)> = uitslag
        .kamerleden
        .iter()
        .filter(|k| k.fractie == "SP")
        .map(|k| (k.naam.as_str(), k.stem))
        .collect();
    assert_eq!(sp, [("Eerste", Stem::Voor), ("Tweede", Stem::Tegen), ("Derde", Stem::Tegen)]);
}

#[test]
fn een_gewone_stemming_in_de_uitslagindeling_kleurt_elke_fractie_als_een_blok() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::ZetStemmingIndeling { indeling: Indeling::Uitslagindeling });
    let g = p.stem_en_leg_vast(false);
    let Graphic::UitslagStemming { uitslag } = &g else { panic!("{g:?}") };
    assert!(!uitslag.voorlopig);
    // Zonder hoofdelijke stemming heeft niemand een Stoel: de afwijkende stem telt mee (131 voor)
    // maar kleurt niets.
    assert_eq!((uitslag.voor, uitslag.tegen), (131, 19));
    let links = stoelen_van_links_naar_rechts();
    let mut verwacht = vec![STEM_VOOR; 150];
    for &i in &links[..20] {
        verwacht[i] = STEM_TEGEN;
    }
    assert_eq!(g.stoelkleuren().unwrap(), verwacht);
}

#[test]
fn in_de_zaalindeling_krijgt_alleen_de_stoel_van_de_afwijkende_stem_de_andere_kleur() {
    let mut p = Proef::nieuw();
    // Eerste en Tweede (SP) en Zonderlijst (PVV) hebben een Stoel; Derde (SP) niet. Alle andere
    // Stoelen zijn leeg.
    let stoelen = [(Stoel { vak: 1, rij: 1, nummer: 1 }, "k-sp1"), (Stoel { vak: 1, rij: 1, nummer: 2 }, "k-sp2"), (Stoel { vak: 6, rij: 6, nummer: 6 }, "k-pvv")];
    p.doe(Opdracht::LeesZaalindeling {
        bestand: "proef.json".into(),
        zaalindeling: Zaalindeling {
            kamerperiode: "2025".into(),
            bron: "voorbeeldgegevens".into(),
            stoelen: stoelen
                .iter()
                .map(|(s, k)| Bezetting { vak: s.vak, rij: s.rij, nummer: s.nummer, kamerlid: Some((*k).into()), naam: String::new() })
                .collect(),
        },
    });
    let g = p.stem_en_leg_vast(false);
    let Graphic::UitslagStemming { uitslag } = &g else { panic!("{g:?}") };
    assert_eq!(uitslag.indeling, Indeling::Zaalindeling, "standaard de Zaalindeling");
    assert!(!uitslag.voorlopig);
    let kleuren = g.stoelkleuren().unwrap();
    let kleur = |s: Stoel| kleuren[s.index().unwrap()];
    assert_eq!(kleur(stoelen[0].0), STEM_VOOR, "Eerste stemt anders dan SP");
    assert_eq!(kleur(stoelen[1].0), STEM_TEGEN, "Tweede stemt met SP");
    assert_eq!(kleur(stoelen[2].0), STEM_VOOR, "PVV stemt voor");
    let grijs = kleuren.iter().filter(|k| **k == GRIJS).count();
    assert_eq!(grijs, 147, "lege Stoelen kleuren niet");
    let _: Kleur = GRIJS;
}

// --- De drie Sjablonen ---

use gedeeld::sjabloon::{Inhoud, IngevuldBlok, Stijl, tekst_op, vast_sjabloon};

fn vul_in(g: &Graphic, stand: &Stand) -> Vec<IngevuldBlok> {
    let i = g.invulling();
    let sjabloon = vast_sjabloon(&i.sjabloon).unwrap_or_else(|| panic!("geen vast Sjabloon {}", i.sjabloon));
    sjabloon.vul_in(&i, &stand.cijfers, &Stijl::default())
}

#[test]
fn de_tekst_op_een_partijkleur_is_licht_of_donker_zoals_in_de_mockup() {
    let (licht, donker) = (Stijl::default().kleuren.tekst, Stijl::default().kleuren.vlak);
    // de voorbeeldkleuren van docs/vormgeving/5-stemming.jpg en de tekstkleur die daar op staat
    let mockup = [
        ("#2DBE4F", donker), ("#FF7A1A", donker), ("#D0213A", licht), ("#0A8A74", licht),
        ("#22B5C2", donker), ("#5E8F23", licht), ("#1E9BE6", licht), ("#F0412A", licht),
        ("#9B2D8F", licht), ("#5A2D91", licht), ("#2B63D9", licht), ("#3A4FB8", licht),
        ("#A3243A", licht), ("#A8C81E", donker), ("#E6A11B", donker),
    ];
    for (achter, verwacht) in mockup {
        assert_eq!(tekst_op(achter.parse().unwrap(), licht, donker), verwacht, "op {achter}");
    }
}

#[test]
fn de_kop_is_de_titel_zonder_indieners() {
    let mut p = Proef::nieuw();
    let g = p.stem_en_leg_vast(false);
    assert_eq!(g.invulling().velden["titel"], "Motie over gratis schoolfruit");
}

#[test]
fn de_uitslag_van_een_stemming_vult_balk_uitkomst_en_fracties_voor_en_tegen() {
    let mut p = Proef::nieuw();
    let g = p.stem_en_leg_vast(false);
    let blokken = vul_in(&g, &p.stand);

    let balk = blokken.iter().find_map(|b| match &b.inhoud {
        Inhoud::Stemmingsbalk { voor, tegen, .. } => Some((*voor, *tegen)),
        _ => None,
    });
    assert_eq!(balk, Some((131, 19)));
    let uitkomst = blokken.iter().find_map(|b| match &b.inhoud {
        Inhoud::Uitkomst { tekst, achter, .. } => Some((tekst.clone(), *achter)),
        _ => None,
    });
    assert_eq!(uitkomst, Some(("AANGENOMEN".to_string(), STEM_VOOR)));
    let lijst = blokken.iter().find_map(|b| match &b.inhoud {
        Inhoud::Fractielijst { voor, tegen, .. } => Some((voor.clone(), tegen.clone())),
        _ => None,
    });
    let (voor, tegen) = lijst.expect("een Fractielijst");
    let kort = |l: &[gedeeld::sjabloon::Fractielabel]| l.iter().map(|f| (f.kort.clone(), f.aantal)).collect::<Vec<_>>();
    // in de volgorde van de Kamer: grootste eerst; SP is verdeeld (Eerste stemt voor), dus SP
    // staat aan beide kanten met het aantal Kamerleden
    let heel = |k: &str| (k.to_string(), None);
    assert_eq!(kort(&voor), [heel("PVV"), heel("GL-PvdA"), heel("VVD"), heel("D66"), ("SP".to_string(), Some(1))]);
    assert_eq!(kort(&tegen), [("SP".to_string(), Some(19))]);
}

#[test]
fn de_namenlijst_zet_de_namen_per_fractie_bij_hun_eigen_stem() {
    let mut p = Proef::nieuw();
    let g = p.stem_en_leg_vast(true);
    let blokken = vul_in(&g, &p.stand);
    let (voor, tegen) = blokken
        .iter()
        .find_map(|b| match &b.inhoud {
            Inhoud::Namenlijst { voor, tegen, .. } => Some((voor.clone(), tegen.clone())),
            _ => None,
        })
        .expect("een Namenlijst");
    let groepen = |g: &[gedeeld::sjabloon::Naamgroep]| {
        g.iter().map(|n| (n.fractie.kort.clone(), n.namen.clone())).collect::<Vec<_>>()
    };
    // Eerste (SP) stemt voor en staat bij voor onder SP; PVV en SP hebben Kamerleden, de andere
    // Fracties alleen hun naam.
    assert_eq!(
        groepen(&voor),
        [
            ("PVV".to_string(), vec!["Zonderlijst".to_string()]),
            ("GL-PvdA".to_string(), vec![]),
            ("VVD".to_string(), vec![]),
            ("D66".to_string(), vec![]),
            ("SP".to_string(), vec!["Eerste".to_string()]),
        ]
    );
    assert_eq!(groepen(&tegen), [("SP".to_string(), vec!["Tweede".to_string(), "Derde".to_string()])]);
}
