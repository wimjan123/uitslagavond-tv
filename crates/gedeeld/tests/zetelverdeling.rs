//! Het Sjabloon Zetelverdeling in de Zaal: de Graphic houdt haar cijfers vast, Toon en Bijwerken
//! kleuren de Stoelen in de Uitslagindeling.

use gedeeld::cijfers::Invoer;
use gedeeld::uitslagindeling::{Uitslagvolgorde, zaal_voor_set};
use gedeeld::{Graphic, Herkomst, Indeling, Opdracht, Stand, Weigering};

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    /// VOORBEELDGEGEVENS: twee verzonnen Partijen en twee sets.
    fn new() -> Self {
        let mut p = Proef { stand: Stand::default(), volgnummer: 0 };
        let invoer = Invoer::uit_json(
            r##"{
            "partijen": [
                { "sleutel": "rood", "naam": "Rode Partij", "kort": "ROOD", "kleur": "#d4202c" },
                { "sleutel": "blauw", "naam": "Blauwe Partij", "kort": "BLAUW", "kleur": "#12357a" }
            ],
            "sets": [
                { "id": "prognose", "soort": "prognose", "label": "Prognose 21.00",
                  "zetels": { "rood": 80, "blauw": 60 } },
                { "id": "uitslag", "soort": "uitslag", "label": "Uitslag",
                  "zetels": { "rood": 70, "blauw": 80 } }
            ]
        }"##,
        );
        p.doe(Opdracht::LeesInvoer { bestand: "proef.json".into(), ontvangen: "nu".into(), invoer })
            .unwrap();
        p.doe(Opdracht::ZetUitslagvolgorde {
            volgorde: Uitslagvolgorde { lijst: vec!["blauw".into(), "rood".into()], ..Default::default() },
        })
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

    fn zaal(&self, set: &str) -> Vec<gedeeld::Kleur> {
        zaal_voor_set(&self.stand.uitslagindeling, &self.stand.cijfers, set).unwrap()
    }
}

fn zetelverdeling(set: &str) -> Graphic {
    Graphic::Zetelverdeling { set: set.into(), namen: true, zetels: true, indeling: Indeling::Uitslagindeling, cijfers: None }
}

#[test]
fn toon_kleurt_de_stoelen_in_de_uitslagindeling() {
    let mut p = Proef::new();
    assert_eq!(p.stand.uitslagindeling.partijen, ["blauw", "rood"]);
    p.doe(Opdracht::ZetKlaar { graphic: zetelverdeling("prognose"), standpunt: None }).unwrap();
    assert!(p.stand.stoelkleuren.is_empty(), "klaarzetten kleurt de Ether nog niet");
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.stoelkleuren, p.zaal("prognose"));

    // een Titelkaart erna laat de kleuren staan
    let kaart = Graphic::Titelkaart { kop: "Even pauze".into(), regel: String::new() };
    p.doe(Opdracht::ZetKlaar { graphic: kaart, standpunt: None }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.stoelkleuren, p.zaal("prognose"));
}

#[test]
fn de_graphic_in_de_ether_houdt_haar_cijfers_tot_bijwerken() {
    let mut p = Proef::new();
    p.doe(Opdracht::ZetKlaar { graphic: zetelverdeling("prognose"), standpunt: None }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    let ether = p.stand.ether.clone().unwrap();
    let voor = p.stand.stoelkleuren.clone();
    assert_eq!(ether.graphic.cijferset(), Some(("prognose", 1)));
    assert!(!p.stand.nieuwere_cijfers(&ether.graphic));

    // de set verandert, en de server start opnieuw: de Ether blijft gelijk
    p.doe(Opdracht::ZetZetels { set: "prognose".into(), partij: "rood".into(), zetels: 75 }).unwrap();
    p.stand = serde_json::from_str(&serde_json::to_string(&p.stand).unwrap()).unwrap();
    assert_eq!(p.stand.ether.as_ref(), Some(&ether));
    assert_eq!(p.stand.stoelkleuren, voor);
    assert!(p.stand.nieuwere_cijfers(&ether.graphic), "het dashboard toont: nieuwere cijfers");

    // Bijwerken neemt de nieuwe cijfers over
    assert!(p.stand.kan_bijwerken());
    p.doe(Opdracht::Bijwerken).unwrap();
    let ether = p.stand.ether.clone().unwrap();
    assert_eq!(ether.graphic.cijferset(), Some(("prognose", 2)));
    assert!(!p.stand.nieuwere_cijfers(&ether.graphic));
    assert_eq!(p.stand.stoelkleuren, p.zaal("prognose"));
    assert_ne!(p.stand.stoelkleuren, voor);
    assert_eq!(p.stand.klaar.as_ref().unwrap().graphic, ether.graphic, "het Voorbeeld ook");
    assert_eq!(p.doe(Opdracht::Bijwerken), Err(Weigering::NietsBijTeWerken));
}

#[test]
fn een_andere_set_klaarzetten_en_tonen_is_een_wissel() {
    let mut p = Proef::new();
    p.doe(Opdracht::ZetKlaar { graphic: zetelverdeling("prognose"), standpunt: None }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    p.doe(Opdracht::ZetKlaar { graphic: zetelverdeling("uitslag"), standpunt: None }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(p.stand.stoelkleuren, p.zaal("uitslag"));
}

#[test]
fn een_zetelverdeling_van_een_onbekende_set_wordt_geweigerd() {
    let mut p = Proef::new();
    assert!(matches!(
        p.doe(Opdracht::ZetKlaar { graphic: zetelverdeling("bestaat-niet"), standpunt: None }),
        Err(Weigering::Cijfers(_))
    ));
}

#[test]
fn een_preset_houdt_de_cijfers_van_het_bewaren() {
    let mut p = Proef::new();
    p.doe(Opdracht::BewaarPreset { graphic: zetelverdeling("prognose"), standpunt: None }).unwrap();
    p.doe(Opdracht::ZetZetels { set: "prognose".into(), partij: "rood".into(), zetels: 75 }).unwrap();
    p.doe(Opdracht::ZetPresetKlaar { nummer: 1 }).unwrap();
    let klaar = p.stand.klaar.clone().unwrap();
    assert_eq!(klaar.graphic.cijferset(), Some(("prognose", 1)));
    assert!(p.stand.nieuwere_cijfers(&klaar.graphic));
}

#[test]
fn een_nieuwe_partij_in_de_partijlijst_komt_rechts_achteraan() {
    let mut p = Proef::new();
    let invoer = Invoer::uit_json(
        r#"{ "partijen": [ { "sleutel": "paars", "naam": "Paarse Partij" } ],
             "sets": [ { "id": "uitslag", "soort": "uitslag", "label": "Uitslag",
                         "zetels": { "rood": 70, "blauw": 75, "paars": 5 } } ] }"#,
    );
    p.doe(Opdracht::LeesInvoer { bestand: "proef.json".into(), ontvangen: "nu".into(), invoer })
        .unwrap();
    assert_eq!(p.stand.uitslagindeling.partijen, ["blauw", "rood", "paars"]);
    assert_eq!(p.stand.uitslagindeling.niet_op_lijst, ["paars"]);
}

/// De labels die de Zetelverdeling toont, zoals de Renderer ze invult: met de cijfers van de
/// Graphic zelf.
fn labels(graphic: &Graphic, stand: &Stand) -> Vec<(Option<String>, Option<u16>)> {
    use gedeeld::sjabloon::{Inhoud, vast_sjabloon};
    let invulling = graphic.invulling();
    let sjabloon = vast_sjabloon(&invulling.sjabloon).expect("vast Sjabloon");
    let cijfers = graphic.cijfers().expect("de Graphic houdt haar cijfers vast");
    sjabloon
        .vul_in(&invulling, &cijfers, &stand.stijl)
        .into_iter()
        .find_map(|b| match b.inhoud {
            Inhoud::Partijlabels { labels, .. } => {
                Some(labels.into_iter().map(|l| (l.kort, l.zetels)).collect())
            }
            _ => None,
        })
        .expect("de Zetelverdeling heeft een Blok met Partijlabels")
}

#[test]
fn de_labels_voor_naam_en_zetels_staan_aan_of_uit_in_de_volgorde_van_de_uitslagindeling() {
    let mut p = Proef::new();
    let met = |namen, zetels| Graphic::Zetelverdeling { set: "prognose".into(), namen, zetels, indeling: Indeling::Uitslagindeling, cijfers: None };
    p.doe(Opdracht::ZetKlaar { graphic: met(true, true), standpunt: None }).unwrap();
    let g = p.stand.klaar.clone().unwrap().graphic;
    assert_eq!(
        labels(&g, &p.stand),
        [(Some("BLAUW".into()), Some(60)), (Some("ROOD".into()), Some(80))]
    );

    p.doe(Opdracht::ZetKlaar { graphic: met(true, false), standpunt: None }).unwrap();
    let g = p.stand.klaar.clone().unwrap().graphic;
    assert_eq!(labels(&g, &p.stand), [(Some("BLAUW".into()), None), (Some("ROOD".into()), None)]);

    p.doe(Opdracht::ZetKlaar { graphic: met(false, false), standpunt: None }).unwrap();
    let g = p.stand.klaar.clone().unwrap().graphic;
    assert!(labels(&g, &p.stand).is_empty(), "beide uit: geen labels");
}
