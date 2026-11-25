//! Sjablonen bewerken in de vormgevingstab (#62): een vast Sjabloon aanpassen en terugzetten,
//! een nieuwe standaard na een update, een eigen Sjabloon samenstellen en tonen.

use std::collections::BTreeMap;

use gedeeld::sjabloon::{
    self, Achtergrond, BewerktSjabloon, Blok, Bloksoort, Kleurkeuze, Kleurovergang, Overgangsvorm, Sjabloon,
    Stijlkleur, Stijlletter, Stoelen, Tekstbron, vast_sjabloon,
};
use gedeeld::{Graphic, Herkomst, Opdracht, Stand, Weigering};

/// Voert Opdrachten uit zoals de server: elk met het volgende volgnummer.
struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    fn new() -> Self {
        Proef { stand: Stand::default(), volgnummer: 0 }
    }

    fn door(&mut self, werkplek: &str, opdracht: Opdracht) -> Result<(), Weigering> {
        let mut nieuw = self.stand.clone();
        let herkomst = Herkomst { door: werkplek, volgnummer: self.volgnummer + 1, tijd_ms: 1_000 };
        nieuw.voer_uit(&opdracht, &herkomst)?;
        self.stand = nieuw;
        self.volgnummer += 1;
        Ok(())
    }

    fn klaar_versie(&self) -> u64 {
        self.stand.klaar.as_ref().expect("er staat iets klaar").versie
    }
}

fn titelkaart(kop: &str) -> Graphic {
    Graphic::Titelkaart { kop: kop.into(), regel: "Voorbeeldgegevens".into() }
}

/// De Titelkaart met een kleinere kop: een aanpassing door de omroep.
fn kleinere_kop() -> Sjabloon {
    let mut s = vast_sjabloon("titelkaart").unwrap();
    if let Bloksoort::Tekst { grootte, .. } = &mut s.blokken[2].soort {
        *grootte = 70;
    }
    s
}

#[test]
fn een_aangepast_vast_sjabloon_gaat_terug_naar_standaard() {
    let mut p = Proef::new();
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: kleinere_kop() }).unwrap();
    assert_eq!(p.stand.sjabloon("titelkaart").unwrap().into_owned(), kleinere_kop());
    assert!(p.stand.bewerkt("titelkaart").is_some());

    p.door("Vormgeving", Opdracht::WisSjabloon { id: "titelkaart".into() }).unwrap();
    assert_eq!(p.stand.sjabloon("titelkaart").unwrap().into_owned(), vast_sjabloon("titelkaart").unwrap());
    assert!(p.stand.bewerkt("titelkaart").is_none());
}

#[test]
fn opslaan_wat_gelijk_is_aan_de_standaard_telt_niet_als_aangepast() {
    let mut p = Proef::new();
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: kleinere_kop() }).unwrap();
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: vast_sjabloon("titelkaart").unwrap() }).unwrap();
    assert!(p.stand.sjablonen.is_empty());
}

/// Een Stand die bewaard is met een oudere versie van de software: de Titelkaart was toen
/// aangepast op een standaard die er nu anders uitziet.
fn stand_van_een_oude_versie() -> Stand {
    let mut stand = Stand::default();
    stand.sjablonen.push(BewerktSjabloon { sjabloon: kleinere_kop(), standaard: Some("0123456789abcdef".into()) });
    let json = serde_json::to_string(&stand).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn na_een_update_meldt_een_aangepast_vast_sjabloon_een_nieuwe_standaard() {
    let stand = stand_van_een_oude_versie();
    let bewerkt = stand.bewerkt("titelkaart").unwrap();
    assert!(bewerkt.nieuwe_standaard());
    // de aanpassing van de omroep blijft
    assert_eq!(stand.sjabloon("titelkaart").unwrap().into_owned(), kleinere_kop());

    // Net aangepast in deze versie: geen melding.
    let mut p = Proef::new();
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: kleinere_kop() }).unwrap();
    assert!(!p.stand.bewerkt("titelkaart").unwrap().nieuwe_standaard());
    // Opnieuw opslaan houdt de oude basis, dus de melding blijft tot de omroep kiest.
    let mut p = Proef { stand: stand_van_een_oude_versie(), volgnummer: 0 };
    let mut nog_kleiner = kleinere_kop();
    nog_kleiner.snelheid = 120;
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: nog_kleiner }).unwrap();
    assert!(p.stand.bewerkt("titelkaart").unwrap().nieuwe_standaard());
}

#[test]
fn de_nieuwe_standaard_overnemen_of_de_eigen_versie_houden() {
    let mut p = Proef { stand: stand_van_een_oude_versie(), volgnummer: 0 };
    p.door("Vormgeving", Opdracht::HoudSjabloon { id: "titelkaart".into() }).unwrap();
    let bewerkt = p.stand.bewerkt("titelkaart").unwrap();
    assert!(!bewerkt.nieuwe_standaard());
    assert_eq!(bewerkt.sjabloon, kleinere_kop());

    let mut p = Proef { stand: stand_van_een_oude_versie(), volgnummer: 0 };
    p.door("Vormgeving", Opdracht::WisSjabloon { id: "titelkaart".into() }).unwrap();
    assert_eq!(p.stand.sjabloon("titelkaart").unwrap().into_owned(), vast_sjabloon("titelkaart").unwrap());
}

#[test]
fn een_bewerkt_sjabloon_geldt_pas_bij_de_volgende_toon() {
    let mut p = Proef::new();
    p.door("Regie", Opdracht::ZetKlaar { graphic: titelkaart("In beeld"), standpunt: None }).unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    let gezien = p.klaar_versie();

    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: kleinere_kop() }).unwrap();
    // De Graphic in de Ether houdt haar vorm.
    let ether = p.stand.ether.as_ref().unwrap();
    assert_eq!(ether.sjabloon.as_ref(), vast_sjabloon("titelkaart").as_ref());
    // De klaargezette Graphic krijgt de nieuwe vorm en telt als net veranderd.
    let klaar = p.stand.klaar.as_ref().unwrap();
    assert_ne!(klaar.versie, gezien);
    assert_eq!(klaar.gewijzigd_door, "Vormgeving");
    assert_eq!(p.door("Regie", Opdracht::Toon { versie: Some(gezien) }), Err(Weigering::GewijzigdDoor("Vormgeving".into())));

    // Bijwerken houdt de vorm, de volgende Toon neemt de nieuwe.
    p.door("Regie", Opdracht::Bijwerken).unwrap();
    assert_eq!(p.stand.ether.as_ref().unwrap().sjabloon.as_ref(), vast_sjabloon("titelkaart").as_ref());
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.ether.as_ref().unwrap().sjabloon.as_ref(), Some(&kleinere_kop()));
}

#[test]
fn een_ander_sjabloon_opslaan_laat_de_klaargezette_graphic_met_rust() {
    let mut p = Proef::new();
    p.door("Regie", Opdracht::ZetKlaar { graphic: titelkaart("Klaar"), standpunt: None }).unwrap();
    let versie = p.klaar_versie();
    let mut balk = vast_sjabloon(sjabloon::ZETELBALK).unwrap();
    balk.snelheid = 80;
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: balk }).unwrap();
    assert_eq!(p.klaar_versie(), versie);
}

/// Een eigen Sjabloon (voorbeeldgegevens): een donker Vlak met een kop uit een invulveld.
fn eigen_sjabloon() -> Sjabloon {
    Sjabloon {
        id: "eigen-opkomst".into(),
        naam: "Opkomst".into(),
        groep: "Eigen".into(),
        snelheid: 100,
        blokken: vec![
            Blok {
                plek: sjabloon::plek(120, 760, 900, 200),
                soort: sjabloon::vlak(90),
                kleur: Kleurkeuze::Stijl(Stijlkleur::Vlak),
                animatie: sjabloon::animatie(sjabloon::Erin::VanLinks, sjabloon::Eruit::NaarLinks, 0),
            },
            Blok {
                plek: sjabloon::plek(160, 790, 820, 100),
                soort: sjabloon::tekst(Tekstbron::Veld("kop".into()), Stijlletter::Kop, 80),
                kleur: Kleurkeuze::Stijl(Stijlkleur::Hoofd),
                animatie: sjabloon::animatie(sjabloon::Erin::Vervagen, sjabloon::Eruit::Vervagen, 300),
            },
        ],
        achtergrond: Achtergrond::ZaalDonker,
        stoelen: Stoelen::Houden,
        standpunt: Some("G".into()),
        kleurovergang: None,
    }
}

fn eigen_graphic(kop: &str) -> Graphic {
    Graphic::Eigen {
        id: "eigen-opkomst".into(),
        velden: BTreeMap::from([("kop".into(), kop.into())]),
        set: None,
        partij: None,
        cijfers: None,
    }
}

#[test]
fn een_eigen_sjabloon_komt_in_de_ether_met_zijn_standpunt_en_stoelen() {
    let mut p = Proef::new();
    p.stand.stoelkleuren = vec!["#123456".parse().unwrap(); gedeeld::AANTAL_STOELEN];
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: eigen_sjabloon() }).unwrap();
    assert!(p.stand.bewerkt("eigen-opkomst").unwrap().standaard.is_none());

    p.door("Regie", Opdracht::ZetKlaar { graphic: eigen_graphic("Opkomst 78 procent"), standpunt: None }).unwrap();
    assert_eq!(p.stand.klaar.as_ref().unwrap().standpunt(), "G");
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();

    let ether = p.stand.ether.as_ref().unwrap();
    assert_eq!(ether.sjabloon.as_ref(), Some(&eigen_sjabloon()));
    assert_eq!(ether.graphic.invulling().velden["kop"], "Opkomst 78 procent");
    assert_eq!(p.stand.camera.standpunt, "G");
    // De Stoelen houden hun kleur.
    assert_eq!(p.stand.stoelkleuren[0], "#123456".parse().unwrap());
}

#[test]
fn een_graphic_van_een_onbekend_eigen_sjabloon_wordt_geweigerd() {
    let mut p = Proef::new();
    assert_eq!(
        p.door("Regie", Opdracht::ZetKlaar { graphic: eigen_graphic("Niets"), standpunt: None }),
        Err(Weigering::OnbekendSjabloon("eigen-opkomst".into()))
    );
}

#[test]
fn een_eigen_sjabloon_dat_klaarstaat_mag_niet_weg() {
    let mut p = Proef::new();
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: eigen_sjabloon() }).unwrap();
    p.door("Regie", Opdracht::ZetKlaar { graphic: eigen_graphic("Klaar"), standpunt: None }).unwrap();
    assert!(matches!(
        p.door("Vormgeving", Opdracht::WisSjabloon { id: "eigen-opkomst".into() }),
        Err(Weigering::SjabloonInGebruik(_))
    ));
    p.door("Regie", Opdracht::ZetKlaar { graphic: titelkaart("Iets anders"), standpunt: None }).unwrap();
    p.door("Vormgeving", Opdracht::WisSjabloon { id: "eigen-opkomst".into() }).unwrap();
    assert!(p.stand.sjabloon("eigen-opkomst").is_none());
}

#[test]
fn een_sjabloon_dat_niet_kan_wordt_geweigerd() {
    let mut p = Proef::new();
    let mut zonder_naam = eigen_sjabloon();
    zonder_naam.naam = " ".into();
    let mut vreemd_id = eigen_sjabloon();
    vreemd_id.id = "titelkaart2".into();
    let mut pad = eigen_sjabloon();
    pad.achtergrond = Achtergrond::Afbeelding { bestand: "../geheim.png".into() };
    let mut verwijzing = eigen_sjabloon();
    verwijzing.blokken[0].soort =
        sjabloon::vlak_met(90, None, 0, None, Some(sjabloon::OmTekst { blokken: vec![7], marge: 10 }));
    let mut standpunt = eigen_sjabloon();
    standpunt.standpunt = Some("P".into());
    for s in [zonder_naam, vreemd_id, pad, verwijzing, standpunt] {
        assert!(p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: s.clone() }).is_err(), "{s:?}");
    }
    assert!(p.stand.sjablonen.is_empty());
}

#[test]
fn een_eigen_sjabloon_in_een_preset_mag_niet_weg() {
    let mut p = Proef::new();
    assert_eq!(
        p.door("Regie", Opdracht::BewaarPreset { graphic: eigen_graphic("Bewaard"), standpunt: None }),
        Err(Weigering::OnbekendSjabloon("eigen-opkomst".into()))
    );
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon: eigen_sjabloon() }).unwrap();
    p.door("Regie", Opdracht::BewaarPreset { graphic: eigen_graphic("Bewaard"), standpunt: None }).unwrap();
    assert!(matches!(
        p.door("Vormgeving", Opdracht::WisSjabloon { id: "eigen-opkomst".into() }),
        Err(Weigering::SjabloonInGebruik(_))
    ));
}

#[test]
fn stoelen_uit_in_een_sjabloon_geeft_de_stoelen_hun_eigen_leer_met_de_overgang_van_het_sjabloon() {
    let mut p = Proef::new();
    p.stand.stoelkleuren = vec!["#123456".parse().unwrap(); gedeeld::AANTAL_STOELEN];
    let vervagen = Kleurovergang { vorm: Overgangsvorm::Vervagen, duur_ms: 2_500 };
    let sjabloon = Sjabloon { stoelen: Stoelen::Uit, kleurovergang: Some(vervagen), ..eigen_sjabloon() };
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon }).unwrap();
    p.door("Regie", Opdracht::ZetKlaar { graphic: eigen_graphic("Kleur weg"), standpunt: None }).unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();

    assert!(p.stand.stoelkleuren.is_empty());
    assert_eq!(p.stand.kleurovergang, vervagen);
}

#[test]
fn kleur_uit_neemt_de_overgang_van_de_stijl_ook_als_het_sjabloon_een_eigen_heeft() {
    let mut p = Proef::new();
    let eigen = Kleurovergang { vorm: Overgangsvorm::Vervagen, duur_ms: 4_000 };
    let sjabloon = Sjabloon { stoelen: Stoelen::Gegevens, kleurovergang: Some(eigen), ..eigen_sjabloon() };
    p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon }).unwrap();
    p.door("Regie", Opdracht::ZetKlaar { graphic: eigen_graphic("Graphic blijft"), standpunt: None }).unwrap();
    p.door("Regie", Opdracht::Toon { versie: None }).unwrap();
    p.door("Regie", Opdracht::ZetStoelkleuren { kleuren: vec!["#d4202c".parse().unwrap(); gedeeld::AANTAL_STOELEN] })
        .unwrap();

    p.door("Regie", Opdracht::KleurUit).unwrap();
    assert!(p.stand.stoelkleuren.is_empty());
    assert_eq!(p.stand.kleurovergang, Kleurovergang::default(), "de Golf van de standaardstijl");
    assert!(p.stand.ether.is_some(), "Kleur uit laat de Graphic staan");
    assert_eq!(p.door("Regie", Opdracht::KleurUit), Err(Weigering::KleurAlUit));
}

#[test]
fn een_kleurovergang_die_te_kort_duurt_wordt_geweigerd() {
    let mut p = Proef::new();
    let sjabloon = Sjabloon { kleurovergang: Some(Kleurovergang { vorm: Overgangsvorm::Golf, duur_ms: 0 }), ..eigen_sjabloon() };
    assert!(p.door("Vormgeving", Opdracht::ZetSjabloon { sjabloon }).is_err());
}
