//! De Sjablonen Winst en verlies en Partij uitgelicht, via de Stand (#50).

use gedeeld::cijfers::Invoer;
use gedeeld::sjabloon::{GRIJS, Inhoud, Zeteldeel, vast_sjabloon};
use gedeeld::uitslagindeling::{Uitslagvolgorde, zaal_voor_set};
use gedeeld::{Graphic, Herkomst, Kleur, Opdracht, Stand, Weigering};

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    /// VOORBEELDGEGEVENS: verzonnen Partijen en drie sets. Paars heeft Oud-paars als
    /// Voorganger, Wit is nieuw, Geel verdwijnt.
    fn new() -> Self {
        let mut p = Proef { stand: Stand::default(), volgnummer: 0 };
        let invoer = Invoer::uit_json(
            r##"{
            "partijen": [
                { "sleutel": "rood", "naam": "Rode Partij", "kort": "ROOD", "kleur": "#d4202c" },
                { "sleutel": "blauw", "naam": "Blauwe Partij", "kort": "BLAUW", "kleur": "#12357a" },
                { "sleutel": "groen", "naam": "Groene Partij", "kort": "GROEN", "kleur": "#2dbe4f" },
                { "sleutel": "geel", "naam": "Gele Partij", "kort": "GEEL", "kleur": "#e8c21a" },
                { "sleutel": "oudpaars", "naam": "Oud-paars", "kort": "OUDP", "kleur": "#5a2d91" },
                { "sleutel": "paars", "naam": "Paarse Partij", "kort": "PAARS", "kleur": "#9b2d8f",
                  "voorgangers": ["oudpaars"] },
                { "sleutel": "wit", "naam": "Witte Partij", "kort": "WIT", "kleur": "#eeeeee" }
            ],
            "sets": [
                { "id": "vorige", "soort": "uitslag", "label": "Uitslag 2023",
                  "zetels": { "rood": 40, "blauw": 30, "groen": 20, "geel": 10, "oudpaars": 15 } },
                { "id": "peiling", "soort": "peiling", "label": "Peiling",
                  "zetels": { "rood": 30, "blauw": 30, "groen": 30, "geel": 30, "paars": 30 } },
                { "id": "uitslag", "soort": "uitslag", "label": "Uitslag 2025",
                  "zetels": { "rood": 35, "blauw": 40, "groen": 20, "paars": 25, "wit": 12 } }
            ],
            "vergelijk_met": "vorige"
        }"##,
        );
        p.doe(Opdracht::LeesInvoer { bestand: "proef.json".into(), ontvangen: "nu".into(), invoer })
            .unwrap();
        let lijst = ["groen", "rood", "blauw", "oudpaars", "paars", "wit", "geel"].map(String::from).to_vec();
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

    fn zaal(&self, set: &str) -> Vec<Kleur> {
        zaal_voor_set(&self.stand.uitslagindeling, &self.stand.cijfers, set).unwrap()
    }

    /// De Blokken van de Graphic in de Ether, zoals de Renderer ze invult.
    fn ether(&self) -> Vec<Inhoud> {
        let g = &self.stand.ether.as_ref().expect("in de Ether").graphic;
        let invulling = g.invulling();
        let sjabloon = vast_sjabloon(&invulling.sjabloon).expect("vast Sjabloon");
        let cijfers = g.cijfers().expect("de Graphic houdt haar cijfers vast");
        sjabloon.vul_in(&invulling, &cijfers, &self.stand.stijl).into_iter().map(|b| b.inhoud).collect()
    }
}

fn kleur(hex: &str) -> Kleur {
    hex.parse().unwrap()
}

#[test]
fn partij_uitgelicht_kleurt_alleen_de_stoelen_van_die_partij_en_de_rest_grijs() {
    let mut p = Proef::new();
    p.toon(Graphic::PartijUitgelicht { set: "uitslag".into(), partij: "blauw".into(), cijfers: None });
    let blauw = kleur("#12357a");
    let zaal = p.zaal("uitslag");
    let kleuren = &p.stand.stoelkleuren;
    assert_eq!(kleuren.len(), 150);
    for (stoel, (nu, in_zaal)) in kleuren.iter().zip(&zaal).enumerate() {
        let verwacht = if *in_zaal == blauw { blauw } else { GRIJS };
        assert_eq!(*nu, verwacht, "Stoel {stoel}");
    }
    assert_eq!(kleuren.iter().filter(|k| **k == blauw).count(), 40, "haar 40 Stoelen op haar plek");
}

/// De rijen van de Winst-en-verliesgrafiek in de Ether, met de as.
fn grafiek(p: &Proef) -> (Vec<Zeteldeel>, u32, u32) {
    p.ether()
        .into_iter()
        .find_map(|i| match i {
            Inhoud::Uitslaggrafiek { rijen, schaal, stap, as_in_het_midden: true, .. } => Some((rijen, schaal, stap)),
            _ => None,
        })
        .expect("een Winst-en-verliesgrafiek")
}

/// Korte naam, Zetels nu, verschil en nieuw van elke rij.
fn rijen(rijen: &[Zeteldeel]) -> Vec<(&str, u16, i32, bool)> {
    rijen
        .iter()
        .map(|r| {
            let w = r.winst.expect("elke rij heeft winst of verlies");
            (r.kort.as_str(), r.zetels, w.verschil, w.nieuw)
        })
        .collect()
}

/// Alle teksten in de Ether.
fn teksten(p: &Proef) -> Vec<String> {
    p.ether()
        .into_iter()
        .filter_map(|i| match i {
            Inhoud::Tekst { tekst, .. } => Some(tekst),
            _ => None,
        })
        .collect()
}

#[test]
fn winst_en_verlies_van_winst_naar_verlies_met_nieuw_en_een_verdwenen_partij_op_nul() {
    let mut p = Proef::new();
    // zonder keuze: de vergelijkingsset van de cijfers ("vorige")
    p.toon(Graphic::WinstEnVerlies { set: "uitslag".into(), vergelijk_met: None, cijfers: None });
    assert_eq!(p.stand.stoelkleuren, p.zaal("uitslag"), "de Zaal toont de set, zonder de verdwenen Partij");
    let (r, schaal, stap) = grafiek(&p);
    assert_eq!(
        rijen(&r),
        [
            ("WIT", 12, 12, true),
            // Blauw en Paars allebei +10: op volgorde; Paars tegen de som van haar Voorganger
            ("BLAUW", 40, 10, false),
            ("PAARS", 25, 10, false),
            ("GROEN", 20, 0, false),
            ("ROOD", 35, -5, false),
            ("GEEL", 0, -10, false),
        ],
        "geen eigen rij voor de Voorganger Oud-paars"
    );
    let geel = r.last().unwrap();
    assert_eq!((geel.kleur, geel.dekking), (p.stand.stijl.kleuren.hoofd, 60), "een verdwenen Partij in Hoofd op 60 procent");
    assert_eq!((r[0].kleur, r[0].dekking), (kleur("#eeeeee"), 100));
    assert_eq!((schaal, stap), (20, 5), "de as zoals de Uitslaggrafiek, naar beide kanten");
    assert!(teksten(&p).contains(&"zetels tegenover Uitslag 2023".to_string()));
}

#[test]
fn winst_en_verlies_tegen_een_andere_set_die_de_regie_kiest() {
    let mut p = Proef::new();
    p.toon(Graphic::WinstEnVerlies { set: "uitslag".into(), vergelijk_met: Some("peiling".into()), cijfers: None });
    assert_eq!(
        rijen(&grafiek(&p).0),
        [
            ("WIT", 12, 12, true),
            ("BLAUW", 40, 10, false),
            ("ROOD", 35, 5, false),
            ("PAARS", 25, -5, false),
            ("GROEN", 20, -10, false),
            ("GEEL", 0, -30, false),
        ]
    );
    assert!(teksten(&p).contains(&"zetels tegenover Peiling".to_string()));
    let g = p.stand.ether.clone().unwrap().graphic;
    p.doe(Opdracht::ZetZetels { set: "peiling".into(), partij: "rood".into(), zetels: 35 }).unwrap();
    assert!(p.stand.nieuwere_cijfers(&g), "de vergelijkingsset veranderde");
}

#[test]
fn winst_en_verlies_zonder_vergelijkingsset_of_tegen_zichzelf_wordt_geweigerd() {
    let mut p = Proef::new();
    p.doe(Opdracht::KiesVergelijkingsset { set: None }).unwrap();
    let g = Graphic::WinstEnVerlies { set: "uitslag".into(), vergelijk_met: None, cijfers: None };
    assert_eq!(p.doe(Opdracht::ZetKlaar { graphic: g, standpunt: None }), Err(Weigering::GeenVergelijkingsset));
    let g = Graphic::WinstEnVerlies { set: "uitslag".into(), vergelijk_met: Some("uitslag".into()), cijfers: None };
    assert_eq!(p.doe(Opdracht::ZetKlaar { graphic: g, standpunt: None }), Err(Weigering::TweeKeerDezelfdeSet));
}
