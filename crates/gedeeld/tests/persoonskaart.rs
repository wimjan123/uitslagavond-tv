//! De Persoonskaart: een Kamerlid of een eigen persoon, met de Stoel die oplicht en de camera
//! naar het shot van die Stoel.

use gedeeld::camera::{shot_op_stoel, stoelsleutel};
use gedeeld::cijfers::Invoer;
use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use gedeeld::persoonskaart::{EigenPersoon, Extra, Persoonsgegevens, Wie};
use gedeeld::sjabloon::{Inhoud, Stijl, vast_sjabloon};
use gedeeld::zaalindeling::{Bezetting, Zaalindeling};
use gedeeld::{Graphic, Herkomst, Kleur, Opdracht, Stand, Stoel, Weigering};

/// 22 november 2026, 12.00 uur UTC.
const VANDAAG_MS: u64 = 1_795_348_800_000;

/// VOORBEELDGEGEVENS: verzonnen Kamerleden en Fracties.
fn lid(id: &str, roepnaam: &str, achternaam: &str, fractie: &str) -> Kamerlid {
    Kamerlid {
        id: id.into(),
        roepnaam: roepnaam.into(),
        initialen: "P.".into(),
        tussenvoegsel: None,
        achternaam: achternaam.into(),
        fractie: fractie.into(),
        zetel: format!("z-{id}"),
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
    /// Twee Kamerleden van Rood: Sanne de Vries (fractievoorzitter, met foto) op Stoel 2-3-1, en
    /// Bram Bes zonder Stoel.
    fn new() -> Self {
        let mut p = Proef { stand: Stand::default(), volgnummer: 0 };
        let invoer = Invoer::uit_json(
            r##"{ "partijen": [
                { "sleutel": "rood", "naam": "Rode Partij", "kort": "ROOD", "kleur": "#d4202c" }
            ], "sets": [] }"##,
        );
        p.doe(Opdracht::LeesInvoer { bestand: "proef.json".into(), ontvangen: "nu".into(), invoer })
            .unwrap();
        let mut sanne = lid("sanne", "Sanne", "Vries", "rood-f");
        sanne.tussenvoegsel = Some("de".into());
        sanne.fractievoorzitter = true;
        sanne.geboortedatum = Some("1984-11-23".into());
        sanne.woonplaats = Some("Utrecht".into());
        sanne.geboorteplaats = Some("Zwolle".into());
        sanne.in_de_kamer_sinds = Some("2017-03-23".into());
        sanne.foto = Foto::OpenData;
        p.doe(Opdracht::LeesKamer {
            opgehaald: "nu".into(),
            peildatum: "2026-11-22".into(),
            kamerleden: vec![sanne, lid("bram", "Bram", "Bes", "rood-f")],
            fracties: vec![Fractie { id: "rood-f".into(), afkorting: "ROOD".into(), naam: "Rode Partij".into(), zetels: 2 }],
        })
        .unwrap();
        let stoelen = vec![Bezetting { vak: 2, rij: 3, nummer: 1, kamerlid: Some("sanne".into()), naam: "Sanne de Vries".into() }];
        let zaalindeling = Zaalindeling { kamerperiode: "2025".into(), bron: String::new(), stoelen };
        p.doe(Opdracht::LeesZaalindeling { bestand: "proef.json".into(), zaalindeling }).unwrap();
        p
    }

    fn doe(&mut self, o: Opdracht) -> Result<(), Weigering> {
        self.volgnummer += 1;
        let h = Herkomst { door: "Proef", volgnummer: self.volgnummer, tijd_ms: VANDAAG_MS };
        self.stand.voer_uit(&o, &h)
    }

    fn zet_klaar(&mut self, wie: Wie, regel: &str) -> Result<(), Weigering> {
        let graphic = Graphic::Persoonskaart { wie, regel: regel.into(), extra: Extra::default(), gegevens: None };
        self.doe(Opdracht::ZetKlaar { graphic, standpunt: None })
    }

    fn gegevens_klaar(&self) -> Persoonsgegevens {
        match &self.stand.klaar.as_ref().unwrap().graphic {
            Graphic::Persoonskaart { gegevens, .. } => gegevens.clone().expect("ingevuld bij klaarzetten"),
            g => panic!("geen Persoonskaart: {g:?}"),
        }
    }
}

fn kamerlid(id: &str) -> Wie {
    Wie::Kamerlid { id: id.into() }
}

#[test]
fn klaarzetten_neemt_de_gegevens_van_het_kamerlid_over() {
    let mut p = Proef::new();
    p.zet_klaar(kamerlid("sanne"), "Eerste spreker in het debat").unwrap();
    let g = p.gegevens_klaar();
    assert_eq!(g.naam, "Sanne de Vries");
    assert_eq!(g.voorletters, "SV");
    assert_eq!(g.foto.as_deref(), Some("fotos/open-data/sanne"));
    assert_eq!(g.partij.as_deref(), Some("rood"));
    assert_eq!(g.functie.as_deref(), Some("Fractievoorzitter"));
    assert_eq!(g.stoel, Some(Stoel { vak: 2, rij: 3, nummer: 1 }));
    // morgen jarig: nog 41
    assert_eq!(g.leeftijd, Some(41));
    assert_eq!(g.regel(&Extra::default()), "41 jaar · Utrecht");
    let alles = Extra { leeftijd: true, woonplaats: true, geboorteplaats: true, in_de_kamer_sinds: true };
    assert_eq!(g.regel(&alles), "41 jaar · Utrecht · geboren in Zwolle · in de Kamer sinds 2017");

    // een gewoon Kamerlid heeft geen functie; zonder geboortedatum geen leeftijd
    p.zet_klaar(kamerlid("bram"), "").unwrap();
    let g = p.gegevens_klaar();
    assert_eq!((&g.functie, g.leeftijd, g.stoel, &g.foto), (&None, None, None, &None));
    assert_eq!(g.regel(&Extra::default()), "");

    assert_eq!(p.zet_klaar(kamerlid("niemand"), ""), Err(Weigering::OnbekendePersoon("niemand".into())));
}

#[test]
fn bij_toon_licht_de_stoel_op_en_gaat_de_camera_naar_het_shot_van_die_stoel() {
    let mut p = Proef::new();
    p.zet_klaar(kamerlid("sanne"), "").unwrap();
    let stoel = Stoel { vak: 2, rij: 3, nummer: 1 };
    let klaar = p.stand.klaar.as_ref().unwrap();
    assert_eq!(klaar.standpunt(), stoelsleutel(stoel));
    assert_eq!(klaar.graphic.uitgelichte_stoel(), Some(stoel));
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "stoel-2-3-1");
    assert!(!p.stand.camera.knip, "bewegen, zoals naar een Standpunt");
    assert_eq!(p.stand.camerastand_van(&p.stand.camera.standpunt), shot_op_stoel(stoel));
    assert_eq!(p.stand.ether.as_ref().unwrap().graphic.uitgelichte_stoel(), Some(stoel));
    // alle Stoelen grijs (in de Zaal donker neutraal): alleen de uitgelichte Stoel valt op
    assert_eq!(p.stand.stoelkleuren, vec![gedeeld::sjabloon::GRIJS; gedeeld::AANTAL_STOELEN]);
}

#[test]
fn zonder_stoel_licht_niets_op_en_gaat_de_camera_naar_het_standpunt_van_het_sjabloon() {
    let mut p = Proef::new();
    p.zet_klaar(kamerlid("bram"), "").unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "S");
    assert_eq!(p.stand.ether.as_ref().unwrap().graphic.uitgelichte_stoel(), None);
    // het dashboard meldt dat de Stoel ontbreekt
    assert!(p.stand.zaalmelding.zonder_stoel.contains(&"Bram Bes".to_string()));

    // een eigen Standpunt voor deze Graphic gaat voor
    let graphic = p.stand.klaar.as_ref().unwrap().graphic.clone();
    p.doe(Opdracht::ZetKlaar { graphic, standpunt: Some("D".into()) }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "D");
}

#[test]
fn een_stoel_als_standpunt_kan_alleen_als_de_stoel_bestaat() {
    let mut p = Proef::new();
    p.doe(Opdracht::KiesStandpunt { letter: "stoel-1-1-2".into(), knip: false }).unwrap();
    assert_eq!(
        p.doe(Opdracht::KiesStandpunt { letter: "stoel-1-1-3".into(), knip: false }),
        Err(Weigering::OnbekendStandpunt("stoel-1-1-3".into()))
    );
}

#[test]
fn de_graphic_houdt_haar_gegevens_vast_tot_de_regie_een_nieuwe_maakt() {
    let mut p = Proef::new();
    p.zet_klaar(kamerlid("sanne"), "").unwrap();
    p.doe(Opdracht::BewaarPreset { graphic: p.stand.klaar.as_ref().unwrap().graphic.clone(), standpunt: None })
        .unwrap();
    // Sanne verhuist en krijgt een andere Stoel
    let mut kamer = p.stand.kamer.kamerleden.clone();
    kamer.iter_mut().find(|k| k.id == "sanne").unwrap().woonplaats = Some("Zwolle".into());
    p.doe(Opdracht::LeesKamer {
        opgehaald: "later".into(),
        peildatum: "2026-11-23".into(),
        kamerleden: kamer,
        fracties: p.stand.kamer.fracties.clone(),
    })
    .unwrap();
    p.doe(Opdracht::GeefStoel { kamerlid: "sanne".into(), stoel: Stoel { vak: 3, rij: 1, nummer: 1 } }).unwrap();
    assert_eq!(p.gegevens_klaar().woonplaats.as_deref(), Some("Utrecht"));
    let Graphic::Persoonskaart { gegevens: Some(g), .. } = &p.stand.presets[0].graphic else { panic!() };
    assert_eq!((g.woonplaats.as_deref(), g.stoel), (Some("Utrecht"), Some(Stoel { vak: 2, rij: 3, nummer: 1 })));
}

#[test]
fn de_regie_voegt_een_eigen_persoon_toe_wijzigt_en_wist_hem() {
    let mut p = Proef::new();
    // VOORBEELDGEGEVENS: een verzonnen lijsttrekker
    let persoon = EigenPersoon {
        id: String::new(),
        naam: "  Mila Verzonnen ".into(),
        foto: true,
        partij: Some("rood".into()),
        regel: "Lijsttrekker".into(),
        geboortedatum: Some("1990-01-01".into()),
        woonplaats: Some("Leiden".into()),
        geboorteplaats: None,
    };
    p.doe(Opdracht::BewaarPersoon { persoon: persoon.clone() }).unwrap();
    let id = p.stand.personen[0].id.clone();
    assert!(!id.is_empty());
    assert_eq!(p.stand.personen[0].naam, "Mila Verzonnen");
    assert!(!p.stand.personen[0].foto, "de foto zet de server pas na het plaatsen");
    p.doe(Opdracht::ZetFotoPersoon { persoon: id.clone(), foto: true }).unwrap();

    // wijzigen houdt de foto
    let wijziging = EigenPersoon { id: id.clone(), naam: "Mila Verzonnen".into(), foto: false, ..persoon.clone() };
    p.doe(Opdracht::BewaarPersoon { persoon: wijziging }).unwrap();
    assert!(p.stand.personen[0].foto);

    p.zet_klaar(Wie::Eigen { id: id.clone() }, "").unwrap();
    let g = p.gegevens_klaar();
    assert_eq!(g.naam, "Mila Verzonnen");
    assert_eq!(g.voorletters, "MV");
    assert_eq!(g.foto, Some(format!("fotos/personen/{id}")));
    assert_eq!((g.partij.as_deref(), g.stoel, g.leeftijd), (Some("rood"), None, Some(36)));
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.camera.standpunt, "S", "een eigen persoon heeft geen Stoel");

    let leeg = EigenPersoon { naam: " ".into(), ..persoon.clone() };
    assert_eq!(p.doe(Opdracht::BewaarPersoon { persoon: leeg }), Err(Weigering::NaamLeeg));
    let onbekend = EigenPersoon { partij: Some("geel".into()), ..persoon };
    assert!(p.doe(Opdracht::BewaarPersoon { persoon: onbekend }).is_err());

    p.doe(Opdracht::WisPersoon { persoon: id.clone() }).unwrap();
    assert!(p.stand.personen.is_empty());
    assert_eq!(p.doe(Opdracht::WisPersoon { persoon: id.clone() }), Err(Weigering::OnbekendePersoon(id)));
}

#[test]
fn de_standaardvinkjes_staan_in_de_stand() {
    let mut p = Proef::new();
    assert_eq!(p.stand.persoonskaart_extra, Extra::default());
    let extra = Extra { leeftijd: false, woonplaats: false, geboorteplaats: true, in_de_kamer_sinds: true };
    p.doe(Opdracht::ZetStandaardExtra { extra }).unwrap();
    assert_eq!(p.stand.persoonskaart_extra, extra);
}

#[test]
fn het_sjabloon_toont_naam_foto_partij_functie_en_regels() {
    let mut p = Proef::new();
    p.zet_klaar(kamerlid("sanne"), "Eerste spreker in het debat").unwrap();
    let graphic = &p.stand.klaar.as_ref().unwrap().graphic;
    let invulling = graphic.invulling();
    assert_eq!(invulling.sjabloon, "persoonskaart");
    let blokken = vast_sjabloon("persoonskaart").unwrap().vul_in(&invulling, &p.stand.cijfers, &Stijl::default());
    let teksten: Vec<&str> = blokken
        .iter()
        .filter_map(|b| match &b.inhoud {
            Inhoud::Tekst { tekst, .. } => Some(tekst.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(teksten, ["Sanne de Vries", "41 jaar · Utrecht", "Eerste spreker in het debat"]);
    let rood: Kleur = "#d4202c".parse().unwrap();
    assert!(blokken.iter().any(|b| matches!(&b.inhoud,
        Inhoud::Foto { foto: Some(f), voorletters, vlakkleur, letter, .. }
            if f == "fotos/open-data/sanne" && voorletters == "SV" && *vlakkleur == rood && letter == gedeeld::sjabloon::LETTER_KOP)));
    assert!(blokken.iter().any(|b| matches!(&b.inhoud,
        Inhoud::Partijregel { kort, partijkleur, tekst, .. }
            if kort == "ROOD" && *partijkleur == rood && tekst == "Fractievoorzitter")));
    let streepjes_zichtbaar = |blokken: &[gedeeld::sjabloon::IngevuldBlok]| {
        blokken.iter().filter(|b| matches!(b.inhoud, Inhoud::Vlak { dekking, .. } if dekking > 0)).count()
    };
    let met_regel = streepjes_zichtbaar(&blokken);

    // zonder vrije regel verdwijnt ook het oranje streepje ervoor
    p.zet_klaar(kamerlid("sanne"), "").unwrap();
    let invulling = p.stand.klaar.as_ref().unwrap().graphic.invulling();
    let zonder = vast_sjabloon("persoonskaart").unwrap().vul_in(&invulling, &p.stand.cijfers, &Stijl::default());
    assert_eq!(streepjes_zichtbaar(&zonder), met_regel - 1);
}

#[test]
fn voorletters_en_leeftijd() {
    use gedeeld::persoonskaart::{dag_van, leeftijd, voorletters};
    assert_eq!(voorletters("Sanne de Vries"), "SV");
    assert_eq!(voorletters("mila"), "M");
    assert_eq!(voorletters("Jan van der Berg"), "JB");
    assert_eq!(dag_van(VANDAAG_MS), (2026, 11, 22));
    assert_eq!(dag_van(951_782_400_000), (2000, 2, 29));
    assert_eq!(leeftijd("1984-11-22", (2026, 11, 22)), Some(42));
    assert_eq!(leeftijd("1984-11-23", (2026, 11, 22)), Some(41));
    assert_eq!(leeftijd("onbekend", (2026, 11, 22)), None);
}
