//! De Coalitiebouwer via de Stand: de Sjablonen Coalitie in de Zaal en Coalitiebalk, de Stoelen
//! met de coalitie als één blok links, en het directe bijwerken in de Ether (besluiten #6 en #7).

use gedeeld::cijfers::Invoer;
use gedeeld::coalitie::donker;
use gedeeld::kamer::{Fractie, Kamerlid};
use gedeeld::sjabloon::{Getalwaarde, Inhoud, Zeteldeel, vast_sjabloon};
use gedeeld::uitslagindeling::{HUIDIGE_KAMER, Uitslagvolgorde, stoelen_van_links_naar_rechts};
use gedeeld::{Graphic, Herkomst, Kleur, Opdracht, Overgang, Stand, Weigering};

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    /// VOORBEELDGEGEVENS: vier verzonnen Partijen en een exitpoll.
    fn new() -> Self {
        let mut p = Proef { stand: Stand::default(), volgnummer: 0 };
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
                  "zetels": { "rood": 50, "blauw": 40, "groen": 30, "geel": 20 } }
            ]
        }"##,
        );
        p.doe(Opdracht::LeesInvoer { bestand: "proef.json".into(), ontvangen: "nu".into(), invoer })
            .unwrap();
        let lijst = ["groen", "rood", "blauw", "geel"].map(String::from).to_vec();
        p.doe(Opdracht::ZetUitslagvolgorde { volgorde: Uitslagvolgorde { lijst, ..Default::default() } })
            .unwrap();
        p
    }

    fn doe(&mut self, opdracht: Opdracht) -> Result<(), Weigering> {
        let mut nieuw = self.stand.clone();
        let h = Herkomst { door: "Regie 1", volgnummer: self.volgnummer + 1, tijd_ms: 1_000 };
        nieuw.voer_uit(&opdracht, &h)?;
        self.stand = nieuw;
        self.volgnummer += 1;
        Ok(())
    }

    fn toon(&mut self, graphic: Graphic) {
        self.doe(Opdracht::ZetKlaar { graphic, standpunt: None }).unwrap();
        self.doe(Opdracht::Toon { versie: None }).unwrap();
    }

    fn zet(&mut self, partijen: &[&str]) -> Result<(), Weigering> {
        self.doe(Opdracht::ZetCoalitie { partijen: partijen.iter().map(|p| p.to_string()).collect() })
    }

    /// De Blokken van de Graphic in de Ether, zoals de Renderer ze invult.
    fn ether(&self) -> Vec<Inhoud> {
        let g = &self.stand.ether.as_ref().expect("in de Ether").graphic;
        let invulling = g.invulling();
        let sjabloon = vast_sjabloon(&invulling.sjabloon).expect("vast Sjabloon");
        let cijfers = g.cijfers().expect("de Graphic houdt haar cijfers vast");
        sjabloon.vul_in(&invulling, &cijfers, &self.stand.stijl).into_iter().map(|b| b.inhoud).collect()
    }

    fn teksten(&self) -> Vec<String> {
        self.ether()
            .into_iter()
            .filter_map(|i| match i {
                Inhoud::Tekst { tekst, .. } => Some(tekst),
                _ => None,
            })
            .collect()
    }

    /// Het getal van de teller en of het een meerderheid is.
    fn teller(&self) -> (i64, bool) {
        self.ether()
            .into_iter()
            .find_map(|i| match i {
                Inhoud::Getal { waarde: Getalwaarde::Getal { waarde, .. }, meerderheid, .. } => {
                    Some((waarde, meerderheid))
                }
                _ => None,
            })
            .expect("een teller")
    }

    fn balk(&self) -> Vec<Zeteldeel> {
        self.ether()
            .into_iter()
            .find_map(|i| match i {
                Inhoud::Zetelbalk { delen, .. } => Some(delen),
                _ => None,
            })
            .expect("een Zetelbalk")
    }
}

fn kleur(hex: &str) -> Kleur {
    hex.parse().unwrap()
}

fn coalitie(set: &str, partijen: &[&str]) -> Graphic {
    Graphic::Coalitie {
        set: set.into(),
        partijen: partijen.iter().map(|p| p.to_string()).collect(),
        cijfers: None,
    }
}

fn coalitiebalk(set: &str, partijen: &[&str]) -> Graphic {
    Graphic::Coalitiebalk {
        set: set.into(),
        partijen: partijen.iter().map(|p| p.to_string()).collect(),
        cijfers: None,
    }
}

/// De kleuren van de Stoelen van links naar rechts, met hoeveel Stoelen achter elkaar.
fn van_links_naar_rechts(kleuren: &[Kleur]) -> Vec<(Kleur, usize)> {
    let mut uit: Vec<(Kleur, usize)> = Vec::new();
    for i in stoelen_van_links_naar_rechts() {
        match uit.last_mut() {
            Some((k, n)) if *k == kleuren[i] => *n += 1,
            _ => uit.push((kleuren[i], 1)),
        }
    }
    uit
}

#[test]
fn de_coalitie_staat_links_als_een_blok_en_de_rest_wordt_donker() {
    let mut p = Proef::new();
    p.toon(coalitie("exitpoll", &["geel", "blauw"]));
    let (rood, blauw, groen, geel) =
        (kleur("#d4202c"), kleur("#12357a"), kleur("#2dbe4f"), kleur("#e8c21a"));
    assert_eq!(
        van_links_naar_rechts(&p.stand.stoelkleuren),
        [
            // de coalitie in de volgorde van de Uitslagindeling (groen, rood, blauw, geel)
            (blauw, 40),
            (geel, 20),
            // de rest op volgorde, donker
            (donker(groen), 30),
            (donker(rood), 50),
            // nog niet verdeeld: grijs
            (gedeeld::sjabloon::GRIJS, 10),
        ]
    );
}

#[test]
fn een_partij_erbij_werkt_de_coalitie_in_de_ether_direct_bij_zonder_bijwerken() {
    let mut p = Proef::new();
    p.toon(coalitie("exitpoll", &["blauw"]));
    let voor = p.stand.ether.clone().unwrap();
    p.zet(&["blauw", "groen"]).unwrap();
    let na = p.stand.ether.clone().unwrap();

    // Uitzondering op "een Graphic in beeld verandert nooit vanzelf" (besluit #6): geen Bijwerken
    // nodig, en het is dezelfde Graphic, dus de Renderer schuift vloeiend (geen wissel).
    assert_eq!(Overgang::tussen(Some(&voor), Some(&na)), Overgang::Bijwerken);
    assert!(matches!(&na.graphic, Graphic::Coalitie { partijen, .. } if partijen == &["blauw", "groen"]));
    assert_eq!(
        van_links_naar_rechts(&p.stand.stoelkleuren)[..2],
        [(kleur("#2dbe4f"), 30), (kleur("#12357a"), 40)],
        "de Zaal kleurt mee"
    );
    // klaar en Ether blijven gelijk: er is niets meer bij te werken
    assert!(!p.stand.kan_bijwerken());
    assert!(matches!(
        &p.stand.klaar.as_ref().unwrap().graphic,
        Graphic::Coalitie { partijen, .. } if partijen == &["blauw", "groen"]
    ));
}

#[test]
fn een_partij_erbij_na_een_nieuwe_stijl_houdt_de_oude_stijl_in_de_ether_en_toon_mag_weer() {
    let mut p = Proef::new();
    p.toon(coalitie("exitpoll", &["blauw"]));
    let oud = p.stand.stijl.clone();
    let mut nieuw = oud.clone();
    nieuw.snelheid = 140;
    p.doe(Opdracht::ZetStijl { stijl: nieuw.clone() }).unwrap();
    p.zet(&["blauw", "groen"]).unwrap();

    // De Ether werkt direct bij, maar houdt de vorm van haar Toon (#61): een nieuwe Stijl geldt
    // pas bij de volgende Toon.
    let e = p.stand.ether.as_ref().unwrap();
    assert!(matches!(&e.graphic, Graphic::Coalitie { partijen, .. } if partijen == &["blauw", "groen"]));
    assert_eq!(e.stijl, oud);
    // Die volgende Toon blijft mogelijk, en neemt de nieuwe Stijl mee.
    assert!(!p.stand.staat_klaar_in_beeld());
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.ether.as_ref().unwrap().stijl, nieuw);
    assert!(p.stand.staat_klaar_in_beeld());
}

#[test]
fn de_coalitie_in_de_ether_werkt_ook_bij_als_er_iets_anders_klaarstaat() {
    let mut p = Proef::new();
    p.toon(coalitie("exitpoll", &["blauw"]));
    p.doe(Opdracht::ZetKlaar {
        graphic: Graphic::Titelkaart { kop: "Straks".into(), regel: String::new() },
        standpunt: None,
    })
    .unwrap();
    p.zet(&["blauw", "rood"]).unwrap();
    assert!(matches!(
        &p.stand.ether.as_ref().unwrap().graphic,
        Graphic::Coalitie { partijen, .. } if partijen.len() == 2
    ));
    assert!(matches!(p.stand.klaar.as_ref().unwrap().graphic, Graphic::Titelkaart { .. }), "de Titelkaart blijft");
}

#[test]
fn zonder_coalitie_klaar_of_in_beeld_wordt_een_partij_kiezen_geweigerd() {
    let mut p = Proef::new();
    assert_eq!(p.zet(&["rood"]), Err(Weigering::GeenCoalitie));
}

#[test]
fn de_teller_telt_naar_76_en_meldt_de_meerderheid() {
    let mut p = Proef::new();
    p.toon(coalitiebalk("exitpoll", &["blauw", "geel"]));
    assert_eq!(p.teller(), (60, false));
    let t = p.teksten();
    assert!(t.contains(&"COALITIE".to_string()), "{t:?}");
    assert!(t.contains(&"BLAUW en GEEL".to_string()), "de kop: de Partijen op volgorde, {t:?}");
    assert!(t.contains(&"16 tekort voor 76".to_string()), "{t:?}");

    p.zet(&["blauw", "geel", "groen"]).unwrap();
    assert_eq!(p.teller(), (90, true));
    let t = p.teksten();
    assert!(t.contains(&"GROEN, BLAUW en GEEL".to_string()), "{t:?}");
    assert!(t.contains(&"meerderheid".to_string()), "{t:?}");
}

#[test]
fn de_coalitiebalk_zet_de_coalitie_vooraan_en_de_rest_zonder_label_erachter() {
    let mut p = Proef::new();
    p.toon(coalitiebalk("exitpoll", &["geel", "rood"]));
    let kort: Vec<(&str, u16, u8)> = Vec::new();
    let balk = p.balk();
    let kort = balk.iter().map(|d| (d.kort.as_str(), d.zetels, d.dekking)).fold(kort, |mut v, d| {
        v.push(d);
        v
    });
    assert_eq!(
        kort,
        [
            ("ROOD", 50, 100),
            ("GEEL", 20, 100),
            // de rest in Tekst op 14 procent (neutraal grijs), zonder label (geen korte naam)
            ("", 30, 14),
            ("", 40, 14),
            // nog niet verdeeld
            ("", 10, 14),
        ]
    );
    // elk deel houdt een eigen sleutel, zodat een Partij die erbij komt vloeiend naar links schuift
    let sleutels: Vec<Option<&str>> = balk.iter().map(|d| d.partij.as_deref()).collect();
    assert_eq!(sleutels, [Some("rood"), Some("geel"), Some("rest:groen"), Some("rest:blauw"), None]);
}

#[test]
fn de_coalitie_werkt_ook_met_de_zetels_van_de_huidige_kamer() {
    let mut p = Proef::new();
    // VOORBEELDGEGEVENS: twee Fracties uit de open data, met hun Zetels.
    let fractie = |id: &str, afkorting: &str, zetels| Fractie {
        id: id.into(),
        afkorting: afkorting.into(),
        naam: afkorting.into(),
        zetels,
    };
    p.doe(Opdracht::LeesKamer {
        opgehaald: "2026-11-22T12:00:00+01:00".into(),
        peildatum: "2026-11-22".into(),
        kamerleden: Vec::<Kamerlid>::new(),
        fracties: vec![fractie("f1", "ROOD", 70), fractie("f2", "BLAUW", 80)],
    })
    .unwrap();
    p.toon(coalitiebalk(HUIDIGE_KAMER, &["rood"]));
    assert_eq!(p.teller(), (70, false));
    assert!(p.teksten().contains(&"6 tekort voor 76".to_string()));
    p.zet(&["rood", "blauw"]).unwrap();
    assert_eq!(p.teller(), (150, true));
    assert_eq!(van_links_naar_rechts(&p.stand.stoelkleuren), [(kleur("#d4202c"), 70), (kleur("#12357a"), 80)]);
}
