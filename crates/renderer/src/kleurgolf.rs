//! De Kleurovergang in de Zaal, als rekensom: bij een nieuwe verdeling blijven de Stoelen staan en
//! verandert alleen hun kleur. Als Golf van links naar rechts (besluit #7), of door Vervagen, alle
//! Stoelen tegelijk. De snelheid van de Stijl maakt de overgang korter of langer.

use gedeeld::sjabloon::{Kleurovergang, Overgangsvorm};

/// Het deel van de golf waarin één Stoel van de oude naar de nieuwe kleur overvloeit.
const OVERVLOEI: f32 = 0.25;

/// De duur van de overgang in seconden bij deze snelheid van de Stijl (in procent).
pub fn duur(overgang: Kleurovergang, snelheid: u16) -> f32 {
    f32::from(overgang.duur_ms) / 1000.0 * 100.0 / f32::from(snelheid.max(10))
}

/// Hoe ver de Stoel op plaats `rang` van links (0 tot en met `laatste`) is op `t` seconden in een
/// overgang van `duur` seconden: 0 nog de oude kleur, 1 de nieuwe.
pub fn voortgang(vorm: Overgangsvorm, rang: usize, laatste: usize, t: f32, duur: f32) -> f32 {
    match vorm {
        Overgangsvorm::Golf => {
            let deel = rang as f32 / laatste.max(1) as f32;
            let start = deel * (1.0 - OVERVLOEI) * duur;
            ((t - start) / (OVERVLOEI * duur)).clamp(0.0, 1.0)
        }
        Overgangsvorm::Vervagen => zacht((t / duur.max(1e-3)).clamp(0.0, 1.0)),
    }
}

/// Zacht starten en stoppen.
pub fn zacht(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    use Overgangsvorm::{Golf, Vervagen};

    fn golf(rang: usize, laatste: usize, t: f32, duur: f32) -> f32 {
        voortgang(Golf, rang, laatste, t, duur)
    }

    #[test]
    fn de_golf_loopt_in_een_seconde_van_links_naar_rechts() {
        let d = duur(Kleurovergang::default(), 100);
        assert_eq!(d, 1.0);
        // de meest linkse Stoel begint meteen en is na een kwart seconde klaar
        assert_eq!(golf(0, 149, 0.0, d), 0.0);
        assert_eq!(golf(0, 149, 0.25, d), 1.0);
        // de meest rechtse begint na driekwart seconde en is na een seconde klaar
        assert_eq!(golf(149, 149, 0.75, d), 0.0);
        assert!(golf(149, 149, 0.875, d) > 0.4 && golf(149, 149, 0.875, d) < 0.6);
        assert_eq!(golf(149, 149, 1.0, d), 1.0);
        // halverwege: links klaar, rechts nog niet begonnen
        assert_eq!(golf(20, 149, 0.5, d), 1.0);
        assert_eq!(golf(130, 149, 0.5, d), 0.0);
    }

    #[test]
    fn de_snelheid_van_de_stijl_maakt_de_golf_korter_of_langer() {
        let g = Kleurovergang::default();
        assert_eq!(duur(g, 200), 0.5);
        assert_eq!(duur(g, 50), 2.0);
        assert_eq!(golf(149, 149, 0.5, duur(g, 200)), 1.0);
    }

    #[test]
    fn vervagen_laat_alle_stoelen_tegelijk_en_zacht_overvloeien() {
        let d = duur(Kleurovergang::standaard(Vervagen), 100);
        assert_eq!(d, 1.5);
        for rang in [0, 75, 149] {
            assert_eq!(voortgang(Vervagen, rang, 149, 0.0, d), 0.0);
            assert_eq!(voortgang(Vervagen, rang, 149, 0.75, d), 0.5, "halverwege, links en rechts gelijk");
            assert_eq!(voortgang(Vervagen, rang, 149, d, d), 1.0);
        }
        // zacht: in het eerste tiende nog maar een paar procent
        assert!(voortgang(Vervagen, 0, 149, 0.15, d) < 0.05);
    }
}
