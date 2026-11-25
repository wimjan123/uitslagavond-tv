//! De Uitslagindeling: Zetels op Stoelen, per Partij één taartpunt van voor tot achter.

use std::collections::BTreeMap;

use gedeeld::cijfers::{Cijfers, Invoer};
use gedeeld::sjabloon::GRIJS;
use gedeeld::uitslagindeling::{Uitslagvolgorde, stoelen_van_links_naar_rechts, zaal_voor_set};
use gedeeld::zaalkaart::STOELPLEKKEN;
use gedeeld::{Kleur, Stoel};

/// VOORBEELDGEGEVENS: vier verzonnen Partijen en een tussenstand van 140 Zetels.
fn cijfers() -> Cijfers {
    let invoer = Invoer::uit_json(
        r##"{
        "partijen": [
            { "sleutel": "rood", "naam": "Rode Partij", "kleur": "#d4202c" },
            { "sleutel": "blauw", "naam": "Blauwe Partij", "kleur": "#12357a" },
            { "sleutel": "groen", "naam": "Groene Partij", "kleur": "#00ae41" },
            { "sleutel": "geel", "naam": "Gele Partij", "kleur": "#f2a900" }
        ],
        "sets": [
            { "id": "tussenstand", "soort": "tussenstand", "label": "Tussenstand",
              "zetels": { "rood": 30, "blauw": 50, "groen": 1, "geel": 59 } }
        ]
    }"##,
    );
    let mut c = Cijfers::default();
    c.lees_invoer("proef.json", "nu", &invoer);
    c
}

fn kleur(hex: &str) -> Kleur {
    hex.parse().unwrap()
}

#[test]
fn de_stoelen_lopen_van_links_naar_rechts_in_beeld_vanaf_de_voorzitter() {
    let volgorde = stoelen_van_links_naar_rechts();
    assert_eq!(volgorde.len(), 150);
    let mut gezien = volgorde.clone();
    gezien.sort();
    assert_eq!(
        gezien,
        (0..150).collect::<Vec<_>>(),
        "elke Stoel precies één keer"
    );
    // Vak 1 staat rechts in beeld vanaf de Voorzitter (#7), dus vak 6 links.
    let alle: Vec<Stoel> = Stoel::alle().collect();
    assert_eq!(alle[volgorde[0]].vak, 6);
    assert_eq!(alle[volgorde[149]].vak, 1);
    // De hoek vanaf het midden van de Zaal stijgt van links naar rechts.
    let hoek = |i: usize| STOELPLEKKEN[i][0].atan2(-STOELPLEKKEN[i][2]);
    assert!(volgorde.windows(2).all(|w| hoek(w[0]) <= hoek(w[1])));
}

#[test]
fn elke_partij_krijgt_aaneengesloten_stoelen_en_grijs_staat_rechts_achteraan() {
    let c = cijfers();
    let volgorde = Uitslagvolgorde {
        lijst: vec!["rood".into(), "groen".into(), "blauw".into(), "geel".into()],
        herkomst: BTreeMap::new(),
    }
    .bepaal(&c);
    let zaal = zaal_voor_set(&volgorde, &c, "tussenstand").unwrap();
    assert_eq!(zaal.len(), 150);

    // Van links naar rechts in beeld: 30 rood, 1 groen, 50 blauw, 59 geel, 10 grijs.
    let rij: Vec<Kleur> = stoelen_van_links_naar_rechts()
        .iter()
        .map(|&i| zaal[i])
        .collect();
    let mut blokken: Vec<(Kleur, usize)> = Vec::new();
    for k in rij {
        match blokken.last_mut() {
            Some((vorige, n)) if *vorige == k => *n += 1,
            _ => blokken.push((k, 1)),
        }
    }
    assert_eq!(
        blokken,
        vec![
            (kleur("#d4202c"), 30),
            (kleur("#00ae41"), 1),
            (kleur("#12357a"), 50),
            (kleur("#f2a900"), 59),
            (GRIJS, 10),
        ]
    );

    // Een taartpunt loopt van voor tot achter: de 30 Stoelen van rood zitten in alle zes rijen.
    let alle: Vec<Stoel> = Stoel::alle().collect();
    let mut rijen: Vec<u8> = (0..150)
        .filter(|&i| zaal[i] == kleur("#d4202c"))
        .map(|i| alle[i].rij)
        .collect();
    rijen.sort();
    rijen.dedup();
    assert_eq!(rijen, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn een_onbekende_set_geeft_geen_zaal() {
    let c = cijfers();
    let volgorde = Uitslagvolgorde::default().bepaal(&c);
    assert!(zaal_voor_set(&volgorde, &c, "bestaat-niet").is_none());
}

/// VOORBEELDGEGEVENS: een partijlijst met een nieuwe Partij en twee Fracties zonder lijst.
fn partijlijst() -> Cijfers {
    let invoer = Invoer::uit_json(
        r##"{
        "partijen": [
            { "sleutel": "a", "naam": "Partij A" },
            { "sleutel": "b", "naam": "Partij B" },
            { "sleutel": "nieuw", "naam": "Nieuwe Partij" },
            { "sleutel": "groep-b", "naam": "Groep B", "fractie_zonder_lijst": true },
            { "sleutel": "lid-b", "naam": "Lid B", "fractie_zonder_lijst": true },
            { "sleutel": "c", "naam": "Partij C" },
            { "sleutel": "klein", "naam": "Kleine Partij" }
        ],
        "sets": [
            { "id": "uitslag", "soort": "uitslag", "label": "Uitslag",
              "zetels": { "a": 40, "b": 50, "nieuw": 5, "groep-b": 3, "lid-b": 1, "c": 51 } }
        ]
    }"##,
    );
    let mut c = Cijfers::default();
    c.lees_invoer("proef.json", "nu", &invoer);
    c
}

#[test]
fn een_nieuwe_partij_staat_rechts_achteraan_met_een_waarschuwing() {
    let c = partijlijst();
    let v = Uitslagvolgorde {
        lijst: vec![
            "c".into(),
            "a".into(),
            "b".into(),
            "groep-b".into(),
            "lid-b".into(),
        ],
        herkomst: BTreeMap::new(),
    }
    .bepaal(&c);
    assert_eq!(
        v.partijen,
        ["c", "a", "b", "groep-b", "lid-b", "nieuw", "klein"]
    );
    // "klein" heeft nergens Zetels: geen waarschuwing
    assert_eq!(v.niet_op_lijst, ["nieuw"]);
}

#[test]
fn een_fractie_zonder_lijst_staat_direct_rechts_naast_haar_oude_fractie() {
    let c = partijlijst();
    let v = Uitslagvolgorde {
        lijst: vec![
            "c".into(),
            "b".into(),
            "a".into(),
            "nieuw".into(),
            "klein".into(),
        ],
        herkomst: [("groep-b".into(), "b".into()), ("lid-b".into(), "b".into())].into(),
    }
    .bepaal(&c);
    assert_eq!(
        v.partijen,
        ["c", "b", "groep-b", "lid-b", "a", "nieuw", "klein"]
    );
    assert!(v.niet_op_lijst.is_empty());

    // vóór de uitzending verschoven: dan geldt de lijst
    let v = Uitslagvolgorde {
        lijst: vec![
            "lid-b".into(),
            "c".into(),
            "b".into(),
            "a".into(),
            "nieuw".into(),
        ],
        herkomst: [("groep-b".into(), "b".into()), ("lid-b".into(), "b".into())].into(),
    }
    .bepaal(&c);
    assert_eq!(
        v.partijen,
        ["lid-b", "c", "b", "groep-b", "a", "nieuw", "klein"]
    );
}

#[test]
fn de_lijn_bij_76_scheidt_de_eerste_75_stoelen_van_links_van_de_rest() {
    use gedeeld::uitslagindeling::lijn_bij_76;
    let lijn = lijn_bij_76();
    let (voor, achter) = (lijn[0], lijn[lijn.len() - 1]);
    // aan welke kant van de lijn (boven gezien) een punt ligt
    let kant = |p: [f32; 3]| {
        (achter[0] - voor[0]) * (p[2] - voor[2]) - (achter[2] - voor[2]) * (p[0] - voor[0])
    };
    let volgorde = stoelen_van_links_naar_rechts();
    let links = kant(STOELPLEKKEN[volgorde[0]]).signum();
    for (rang, &i) in volgorde.iter().enumerate() {
        let k = kant(STOELPLEKKEN[i]);
        assert!(k.abs() > 0.05, "Stoel {rang} ligt bijna op de lijn");
        assert_eq!(
            k.signum() == links,
            rang < 75,
            "Stoel {rang} ligt aan de verkeerde kant"
        );
    }
    // de lijn loopt van voor de voorste rij tot achter de achterste
    let straal = |p: [f32; 3]| p[0].hypot(p[2]);
    let stralen: Vec<f32> = STOELPLEKKEN.iter().map(|p| straal(*p)).collect();
    assert!(straal(voor) < stralen.iter().copied().fold(f32::MAX, f32::min));
    assert!(straal(achter) > stralen.iter().copied().fold(0.0, f32::max));
    // plat op de vloer van het middenpad: elk stuk is een trede (vlak) of een stootbord (recht
    // omhoog), van voor naar achter, en alle punten op één rechte lijn boven gezien
    for paar in lijn.windows(2) {
        let [a, b] = [paar[0], paar[1]];
        let (vlak, omhoog) = (a[1] == b[1], (a[0], a[2]) == (b[0], b[2]) && b[1] > a[1]);
        assert!(vlak != omhoog, "{a:?} -> {b:?}");
        assert!(straal(b) >= straal(a));
        assert!(kant(b).abs() < 1e-3, "{b:?} ligt niet op de lijn");
    }
    assert!(
        voor[1] < 0.05,
        "de lijn begint op de vloer, niet op de hoogte van de rugleuningen"
    );
    assert_eq!(
        lijn.iter()
            .filter(|p| p[1] > voor[1])
            .map(|p| p[1].to_bits())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        5,
        "vijf treden omhoog"
    );
}
