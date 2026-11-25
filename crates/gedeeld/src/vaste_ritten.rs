//! De zes vaste Cameraritten: ze komen mee met het systeem, net als de vaste Standpunten, op de
//! toetsen Q, W, E, T, Y en U. Hun opname staat niet in de Stand (die gaat bij elke Opdracht naar
//! elk scherm): de software rekent hem hier uit, 100 standen per seconde. In de Stand staan alleen
//! hun naam en schuifjes, zodat de Regie ze kan bijschaven en terugzetten.
//!
//! Assen van Bevy: y omhoog, de Voorzitter aan de kant van +z, de banken in de halve cirkel aan de
//! kant van -z (straal 12 m om (0, 0)). Draai 0 kijkt langs -z, positief draait naar links. Elke
//! stand blijft in de onzichtbare ruimte van de camera (zie de proef in de server).

use std::sync::LazyLock;

use crate::camera::TIKKEN_PER_SECONDE;
use crate::camerarit::{Bijschaving, Camerarit, RitEind};

/// De toetsen van de vaste Cameraritten, op volgorde. De toets is ook de sleutel.
pub const VASTE_TOETSEN: [&str; 6] = ["Q", "W", "E", "T", "Y", "U"];

/// Eén vaste Camerarit: hoe hij heet, hoe lang hij duurt en wat hij aan het eind doet.
struct Vast {
    letter: &'static str,
    naam: &'static str,
    seconden: u32,
    eind: RitEind,
    /// De stand op voortgang `t` (0 tot 1): plek, waar de camera naar kijkt, en beeldhoek in graden.
    stand: fn(f32) -> ([f32; 3], [f32; 3], f32),
}

const VAST: [Vast; 6] = [
    Vast { letter: "Q", naam: "Onthulling", seconden: 16, eind: RitEind::Stoppen, stand: onthulling },
    Vast { letter: "W", naam: "Zwenk linkerflank naar rechterflank", seconden: 24, eind: RitEind::Stoppen, stand: zwenk },
    Vast { letter: "E", naam: "Rondje boven de zaal", seconden: 50, eind: RitEind::Herhalen, stand: rondje },
    Vast { letter: "T", naam: "Panorama vanuit het midden", seconden: 20, eind: RitEind::Stoppen, stand: panorama },
    Vast { letter: "Y", naam: "Kraan omhoog naar overzicht", seconden: 14, eind: RitEind::Stoppen, stand: kraan },
    Vast { letter: "U", naam: "Naar de katheder", seconden: 14, eind: RitEind::Stoppen, stand: katheder },
];

/// Zacht starten en stoppen.
fn zacht(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

/// Van boven de banken steil omlaag kijkend, omlaag en naar voren tot een overzicht op ooghoogte.
fn onthulling(t: f32) -> ([f32; 3], [f32; 3], f32) {
    let t = zacht(t);
    let plek = mix([0.0, 4.1, -2.5], [0.0, 2.4, 2.8], t);
    let doel = mix([0.0, 0.3, -4.2], [0.0, 1.0, -6.0], t);
    (plek, doel, 70.0 - 18.0 * t)
}

/// Van de linkerflank in een boog voor de Voorzitter langs naar de rechterflank.
fn zwenk(t: f32) -> ([f32; 3], [f32; 3], f32) {
    let s = -1.0 + 2.0 * zacht(t);
    let plek = [9.8 * s, 2.8 + 0.5 * (1.0 - s * s), 0.8 + 2.6 * (1.0 - s * s)];
    (plek, [0.0, 0.9, -5.5], 52.0)
}

/// Hoog boven de banken een hele cirkel, steil omlaag kijkend. Begin en eind zijn gelijk: hij
/// herhaalt zonder knip.
fn rondje(t: f32) -> ([f32; 3], [f32; 3], f32) {
    let (midden, straal) = ([0.0, -5.5], 5.0);
    let (s, c) = (std::f32::consts::TAU * t).sin_cos();
    let plek = [midden[0] - straal * s, 4.0, midden[1] + straal * c];
    let doel = [midden[0] + 1.5 * s, 0.6, midden[1] - 1.5 * c];
    (plek, doel, 62.0)
}

/// Laag in het midden van de halve cirkel, langzaam van links naar rechts over de banken.
fn panorama(t: f32) -> ([f32; 3], [f32; 3], f32) {
    let hoek = (75.0 - 150.0 * zacht(t)).to_radians();
    let plek = [0.6 * (-hoek * 0.3).sin(), 1.45, -0.8];
    let doel = [plek[0] - 8.0 * hoek.sin(), 1.6, plek[2] - 8.0 * hoek.cos()];
    (plek, doel, 48.0)
}

/// Van laag achter de Voorzitter omhoog en naar voren, tot het overzicht van bovenaf.
fn kraan(t: f32) -> ([f32; 3], [f32; 3], f32) {
    let t = zacht(t);
    let plek = [0.0, 1.9 + 2.2 * t, 3.6 - 6.0 * t];
    let doel = [0.0, 1.2 - 0.6 * t, -9.0 + 3.0 * t];
    (plek, doel, 58.0 + 12.0 * t)
}

/// Van ver achter in de Zaal over de banken naar de katheder, steeds dichterbij.
fn katheder(t: f32) -> ([f32; 3], [f32; 3], f32) {
    let t = zacht(t);
    let plek = mix([-6.0, 2.7, -7.0], [1.2, 1.75, -0.65], t);
    (plek, [3.19, 1.35, 2.21], 50.0 - 20.0 * t)
}

/// Een stand van de opname: x, y, z, draai, helling en beeldhoek (zie [`crate::camerarit::als_stand`]).
fn punt(plek: [f32; 3], doel: [f32; 3], beeldhoek_graden: f32) -> [f32; 6] {
    let d = [doel[0] - plek[0], doel[1] - plek[1], doel[2] - plek[2]];
    let draai = (-d[0]).atan2(-d[2]);
    let helling = d[1].atan2((d[0] * d[0] + d[2] * d[2]).sqrt());
    [plek[0], plek[1], plek[2], draai, helling, beeldhoek_graden.to_radians()]
}

fn opname(v: &Vast) -> Vec<[f32; 6]> {
    let n = v.seconden * TIKKEN_PER_SECONDE;
    (0..=n)
        .map(|i| {
            let (plek, doel, beeldhoek) = (v.stand)(i as f32 / n as f32);
            punt(plek, doel, beeldhoek)
        })
        .collect()
}

static OPNAMES: LazyLock<Vec<Vec<[f32; 6]>>> = LazyLock::new(|| VAST.iter().map(opname).collect());

/// De opname van de vaste Camerarit met deze sleutel.
pub fn opname_van(letter: &str) -> Option<&'static [[f32; 6]]> {
    VAST.iter().position(|v| v.letter == letter).map(|i| OPNAMES[i].as_slice())
}

/// Of dit de sleutel van een vaste Camerarit is.
pub fn is_vast(letter: &str) -> bool {
    VAST.iter().any(|v| v.letter == letter)
}

/// De vaste Cameraritten zoals ze meekomen: zonder opname in de Stand, met de schuifjes van een
/// nieuwe opname en het eind dat bij de rit hoort.
pub fn vaste() -> Vec<Camerarit> {
    VAST.iter().map(standaard).collect()
}

fn standaard(v: &Vast) -> Camerarit {
    Camerarit {
        letter: v.letter.into(),
        naam: v.naam.into(),
        toets: None,
        vast: true,
        opname: Vec::new(),
        bijschaving: Bijschaving { gladheid: 0.0, eind: v.eind, ..Bijschaving::nieuw(v.seconden as f32) },
    }
}

/// De standaard van de vaste Camerarit met deze sleutel.
pub fn standaard_van(letter: &str) -> Option<Camerarit> {
    VAST.iter().find(|v| v.letter == letter).map(standaard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn het_rondje_herhaalt_en_de_rest_niet() {
        for r in vaste() {
            assert_eq!(r.kan_herhalen().is_ok(), r.letter == "E", "{}", r.naam);
        }
    }

    #[test]
    fn elke_vaste_rit_duurt_zo_lang_als_zijn_opname() {
        for r in vaste() {
            assert!((r.opname_s() - r.bijschaving.duur_s).abs() < 1e-3, "{}", r.naam);
            assert!(r.beginstand().is_some());
        }
    }
}
