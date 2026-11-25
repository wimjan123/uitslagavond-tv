//! De Kleurovergang in de Zaal, als rekensom: bij een nieuwe verdeling blijven de Stoelen staan en
//! verandert alleen hun kleur. Als Golf van links naar rechts (besluit #7), door Vervagen, alle
//! Stoelen tegelijk, of als Opbouw, één Partij tegelijk. De snelheid van de Stijl maakt de
//! overgang korter of langer.

use gedeeld::Kleur;
use gedeeld::sjabloon::{Kleurovergang, Overgangsvorm};

/// Het deel van de golf waarin één Stoel van de oude naar de nieuwe kleur overvloeit.
const OVERVLOEI: f32 = 0.25;

/// Bij de Opbouw: hoe lang de Stoelen eerst donker worden, als deel van de tijd per Partij.
const INLEIDING: f32 = 0.5;
/// Bij de Opbouw: hoe lang een Partij oplicht, als deel van de tijd per Partij.
const OPLICHTEN: f32 = 0.4;
/// Bij de Opbouw: hoe snel een Partij na haar stap dimt (en haar gloed verliest), en aan het eind
/// weer opkomt, als deel van de tijd per Partij.
const NAGLOED: f32 = 0.25;
/// Bij de Opbouw: hoe ver een Partij na haar stap naar de wachtkleur zakt, zodat alleen de Partij
/// van het label vol oplicht. Aan het eind komen ze allemaal tegelijk terug.
pub const DIMMEN: f32 = 0.7;

/// De duur van de overgang in seconden bij deze snelheid van de Stijl (in procent); bij de
/// Opbouw de tijd per Partij.
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
        // de Opbouw rekent zelf (zie [`Opbouw`]); hier telt hij als Vervagen
        Overgangsvorm::Vervagen | Overgangsvorm::Opbouw => zacht((t / duur.max(1e-3)).clamp(0.0, 1.0)),
    }
}

/// De Opbouw: eerst worden alle Stoelen donker (de inleiding), dan licht elke stap op in zijn
/// kleur, de grootste groep eerst. Een stap is een groep Stoelen met dezelfde nieuwe kleur.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Opbouw {
    /// De kleur van elke stap, op volgorde.
    pub stappen: Vec<Kleur>,
    /// De stap van elke Stoel; `None`: hij krijgt geen kleur en gaat in de inleiding meteen
    /// naar zijn eigen leer.
    pub stap_van: Vec<Option<usize>>,
    /// Seconden per stap.
    pub per_stap: f32,
}

/// Waar een Stoel in de Opbouw is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Opbouwdeel {
    /// Van de oude kleur naar de wachtkleur (of zijn eigen leer), 0 tot 1.
    Donker(f32),
    /// Van de wachtkleur naar de nieuwe kleur, 0 tot 1.
    Oplichten(f32),
}

impl Opbouw {
    /// De stappen bij deze nieuwe kleuren (in de volgorde van [`gedeeld::Stoel::alle`]) en de
    /// plaats van elke Stoel van links naar rechts: per kleur een stap, de grootste eerst, bij
    /// gelijke grootte de meest linkse eerst.
    pub fn nieuw(naar: &[Option<Kleur>], rang: &[usize], per_stap: f32) -> Self {
        // per kleur: hoeveel Stoelen en de meest linkse
        let mut groepen: Vec<(Kleur, usize, usize)> = Vec::new();
        for (i, k) in naar.iter().enumerate() {
            let Some(k) = *k else { continue };
            match groepen.iter_mut().find(|g| g.0 == k) {
                Some(g) => {
                    g.1 += 1;
                    g.2 = g.2.min(rang[i]);
                }
                None => groepen.push((k, 1, rang[i])),
            }
        }
        groepen.sort_by_key(|&(_, n, links)| (std::cmp::Reverse(n), links));
        let stappen: Vec<Kleur> = groepen.into_iter().map(|g| g.0).collect();
        let stap_van = naar.iter().map(|k| k.and_then(|k| stappen.iter().position(|s| *s == k))).collect();
        Opbouw { stappen, stap_van, per_stap }
    }

    fn inleiding(&self) -> f32 {
        INLEIDING * self.per_stap
    }

    /// Wanneer de laatste stap voorbij is.
    fn alle_stappen(&self) -> f32 {
        self.inleiding() + self.stappen.len() as f32 * self.per_stap
    }

    /// Hoe lang de hele Opbouw duurt, met het opkomen van de gedimde Partijen aan het eind.
    pub fn duur(&self) -> f32 {
        self.alle_stappen() + OPLICHTEN.max(NAGLOED) * self.per_stap
    }

    /// Hoe ver Stoel `i` op `t` seconden gedimd is (0 tot [`DIMMEN`]): na zijn stap zakt hij,
    /// zodat alleen de Partij van het label vol oplicht; na de laatste stap komt alles tegelijk terug.
    pub fn gedimd(&self, i: usize, t: f32) -> f32 {
        let Some(k) = self.stap_van.get(i).copied().flatten() else { return 0.0 };
        let einde = self.inleiding() + (k + 1) as f32 * self.per_stap;
        let (alle, overgang) = (self.alle_stappen(), NAGLOED * self.per_stap);
        if t < einde || einde >= alle {
            0.0
        } else if t < alle {
            DIMMEN * zacht(((t - einde) / overgang).min(1.0))
        } else {
            DIMMEN * (1.0 - zacht(((t - alle) / overgang).min(1.0)))
        }
    }

    /// Hoe sterk Stoel `i` op `t` seconden gloeit (0 tot 1): de stap die aan de beurt is, licht
    /// extra op, zodat je ook bij een donkere partijkleur ziet welk blok bij het label hoort. Na
    /// zijn stap zakt hij terug naar zijn eigen kleur.
    pub fn gloed(&self, i: usize, t: f32) -> f32 {
        let Some(k) = self.stap_van.get(i).copied().flatten() else { return 0.0 };
        let start = self.inleiding() + k as f32 * self.per_stap;
        let einde = start + self.per_stap;
        if t < start {
            0.0
        } else if t < einde {
            zacht(((t - start) / (OPLICHTEN * self.per_stap)).min(1.0))
        } else {
            1.0 - zacht(((t - einde) / (NAGLOED * self.per_stap)).min(1.0))
        }
    }

    /// Waar Stoel `i` is op `t` seconden.
    pub fn deel(&self, i: usize, t: f32) -> Opbouwdeel {
        let inleiding = self.inleiding();
        match self.stap_van.get(i).copied().flatten() {
            Some(k) if t >= inleiding => {
                let start = inleiding + k as f32 * self.per_stap;
                Opbouwdeel::Oplichten(zacht(((t - start) / (OPLICHTEN * self.per_stap)).clamp(0.0, 1.0)))
            }
            _ => Opbouwdeel::Donker(zacht((t / inleiding.max(1e-3)).clamp(0.0, 1.0))),
        }
    }

    /// De stap die op `t` seconden oplicht, met de seconden sinds hij begon; `None` in de
    /// inleiding en na de laatste stap.
    pub fn stap(&self, t: f32) -> Option<(usize, f32)> {
        let na = t - self.inleiding();
        if na < 0.0 {
            return None;
        }
        let k = (na / self.per_stap.max(1e-3)) as usize;
        (k < self.stappen.len()).then(|| (k, na - k as f32 * self.per_stap))
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

    fn k(r: u8) -> Option<Kleur> {
        Some(Kleur { r, g: 0, b: 0 })
    }

    #[test]
    fn de_opbouw_begint_met_de_grootste_partij_en_bij_gelijke_grootte_links() {
        // van links naar rechts: twee van 1, drie van 2, geen kleur, twee van 3
        let naar = [k(1), k(1), k(2), k(2), k(2), None, k(3), k(3)];
        let rang: Vec<usize> = (0..naar.len()).collect();
        let o = Opbouw::nieuw(&naar, &rang, 1.0);
        assert_eq!(o.stappen, [k(2), k(1), k(3)].map(Option::unwrap));
        assert_eq!(o.stap_van, [Some(1), Some(1), Some(0), Some(0), Some(0), None, Some(2), Some(2)]);
        assert!((o.duur() - 3.9).abs() < 1e-6, "een halve stap donker, drie stappen en de nagloed");
    }

    #[test]
    fn in_de_opbouw_licht_elke_stap_pas_op_als_hij_aan_de_beurt_is() {
        let naar = [k(1), k(2), k(2), None];
        let o = Opbouw::nieuw(&naar, &[0, 1, 2, 3], 1.5);
        let inleiding = 0.75;
        // eerst wordt alles donker, ook de Stoel zonder kleur (die gaat naar zijn leer)
        assert_eq!(o.deel(0, 0.0), Opbouwdeel::Donker(0.0));
        assert_eq!(o.deel(3, inleiding), Opbouwdeel::Donker(1.0));
        assert_eq!(o.stap(0.5), None);
        // stap 0 (de twee van kleur 2) licht op, de Stoel van kleur 1 wacht nog donker
        assert_eq!(o.deel(1, inleiding + 0.6), Opbouwdeel::Oplichten(1.0));
        assert_eq!(o.deel(0, inleiding + 1.0), Opbouwdeel::Oplichten(0.0));
        assert_eq!(o.stap(inleiding + 1.0), Some((0, 1.0)));
        // stap 1 begint na anderhalve seconde
        assert_eq!(o.stap(inleiding + 1.6).map(|s| s.0), Some(1));
        assert_eq!(o.deel(0, inleiding + 1.5 + 0.6), Opbouwdeel::Oplichten(1.0));
        assert_eq!(o.stap(o.duur()), None, "na de laatste stap");
    }

    #[test]
    fn de_stap_die_aan_de_beurt_is_gloeit_en_zakt_daarna_terug() {
        let o = Opbouw::nieuw(&[k(1), k(2)], &[0, 1], 1.0);
        // stap 0 loopt van 0,5 tot 1,5 seconde
        assert_eq!(o.gloed(0, 0.4), 0.0);
        assert_eq!(o.gloed(0, 1.0), 1.0);
        assert!(o.gloed(0, 1.6) > 0.0 && o.gloed(0, 1.6) < 1.0, "na zijn stap zakt hij terug");
        assert_eq!(o.gloed(0, 1.75), 0.0);
        assert_eq!(o.gloed(1, 1.0), 0.0, "de volgende wacht nog");
    }

    #[test]
    fn na_haar_stap_dimt_een_partij_tot_de_laatste_stap_voorbij_is() {
        let o = Opbouw::nieuw(&[k(1), k(2), k(2), None], &[0, 1, 2, 3], 1.0);
        // stap 0 (kleur 2) van 0,5 tot 1,5, stap 1 (kleur 1) van 1,5 tot 2,5
        assert_eq!(o.gedimd(1, 1.0), 0.0, "tijdens haar stap vol");
        assert_eq!(o.gedimd(1, 2.0), DIMMEN, "tijdens de volgende stap gedimd");
        assert_eq!(o.gedimd(0, 2.0), 0.0, "de Partij van het label niet");
        assert_eq!(o.gedimd(0, 2.7), 0.0, "de laatste stap dimt nooit");
        assert_eq!(o.gedimd(1, o.duur()), 0.0, "aan het eind komt alles terug");
        assert_eq!(o.gedimd(3, 2.0), 0.0, "zonder kleur niets");
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
