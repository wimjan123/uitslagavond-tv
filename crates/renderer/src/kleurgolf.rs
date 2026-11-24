//! De kleurgolf in de Zaal, als rekensom: bij een nieuwe verdeling blijven de Stoelen staan en
//! verandert alleen hun kleur, als een golf van links naar rechts van ongeveer één seconde
//! (besluit #7). De snelheid van de Stijl maakt de golf korter of langer.

/// Duur van de hele golf bij snelheid 100, in seconden.
const GOLF: f32 = 1.0;
/// Het deel van de golf waarin één Stoel van de oude naar de nieuwe kleur overvloeit.
const OVERVLOEI: f32 = 0.25;

/// De duur van de golf in seconden bij deze snelheid van de Stijl (in procent).
pub fn duur(snelheid: u16) -> f32 {
    GOLF * 100.0 / f32::from(snelheid.max(10))
}

/// Hoe ver de Stoel op plaats `rang` van links (0 tot en met `laatste`) is op `t` seconden in een
/// golf van `duur` seconden: 0 nog de oude kleur, 1 de nieuwe.
pub fn voortgang(rang: usize, laatste: usize, t: f32, duur: f32) -> f32 {
    let deel = rang as f32 / laatste.max(1) as f32;
    let start = deel * (1.0 - OVERVLOEI) * duur;
    ((t - start) / (OVERVLOEI * duur)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn de_golf_loopt_in_een_seconde_van_links_naar_rechts() {
        let d = duur(100);
        assert_eq!(d, 1.0);
        // de meest linkse Stoel begint meteen en is na een kwart seconde klaar
        assert_eq!(voortgang(0, 149, 0.0, d), 0.0);
        assert_eq!(voortgang(0, 149, 0.25, d), 1.0);
        // de meest rechtse begint na driekwart seconde en is na een seconde klaar
        assert_eq!(voortgang(149, 149, 0.75, d), 0.0);
        assert!(voortgang(149, 149, 0.875, d) > 0.4 && voortgang(149, 149, 0.875, d) < 0.6);
        assert_eq!(voortgang(149, 149, 1.0, d), 1.0);
        // halverwege: links klaar, rechts nog niet begonnen
        assert_eq!(voortgang(20, 149, 0.5, d), 1.0);
        assert_eq!(voortgang(130, 149, 0.5, d), 0.0);
    }

    #[test]
    fn de_snelheid_van_de_stijl_maakt_de_golf_korter_of_langer() {
        assert_eq!(duur(200), 0.5);
        assert_eq!(duur(50), 2.0);
        assert_eq!(voortgang(149, 149, 0.5, duur(200)), 1.0);
    }
}
