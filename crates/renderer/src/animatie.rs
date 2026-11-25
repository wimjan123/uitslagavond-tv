//! De animaties van een Blok, als rekensom: hoe een Blok erbij staat op een moment.
//!
//! Een Blok komt in beeld met een van de manieren van [`Erin`], gaat weg met een van de
//! manieren van [`Eruit`], en wacht eerst "start na". De tijden en maten komen uit de
//! [`Beweging`] van de Stijl; de snelheid van de Stijl en van het Sjabloon maken alles korter
//! of langer. Geen tijdlijn: elk Blok heeft zijn eigen vaste beweging.

use bevy::math::Vec2;
use gedeeld::sjabloon::{Animatie, Beweging, Erin, Eruit, Inhoud};

/// De Beweging als getallen voor de rekensom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Maten {
    /// Duur van in beeld komen bij snelheid 100, in seconden.
    pub duur: f32,
    /// Hoe ver een Blok invliegt of wegvliegt, in pixels van het 1080p-beeld.
    vlieg: f32,
    /// Bij invliegen is het Blok na dit deel van de beweging helemaal zichtbaar.
    zichtbaar_na: f32,
    /// Openen vanuit lijn bij snelheid 100, in seconden.
    lijn_verschijnt: f32,
    vlak_opent: f32,
    tekst_vervaagt: f32,
    /// Hoe dik de lijn is, in pixels van het 1080p-beeld.
    pub lijn: f32,
    /// Sluiten naar lijn, als deel van het weggaan: tot hier vervaagt de tekst, dan gaat het
    /// vlak dicht en tot het eind verdwijnt de lijn.
    tekst_weg: f32,
    vlak_dicht: f32,
}

impl From<&Beweging> for Maten {
    fn from(b: &Beweging) -> Self {
        let s = |ms: u16| f32::from(ms) / 1000.0;
        let deel = |pct: u8| f32::from(pct) / 100.0;
        Maten {
            duur: s(b.duur_ms),
            vlieg: f32::from(b.vlieg_px),
            zichtbaar_na: deel(b.zichtbaar_na_pct).max(0.01),
            lijn_verschijnt: s(b.lijn.verschijnt_ms),
            vlak_opent: s(b.lijn.opent_ms),
            tekst_vervaagt: s(b.lijn.tekst_ms),
            lijn: f32::from(b.lijn.dikte_px),
            tekst_weg: deel(b.lijn.tekst_weg_pct),
            vlak_dicht: deel(b.lijn.vlak_dicht_pct),
        }
    }
}

impl Default for Maten {
    fn default() -> Self {
        Maten::from(&Beweging::default())
    }
}

impl Maten {
    /// Hoe lang Openen vanuit lijn duurt bij snelheid 100, in seconden.
    fn lijn_totaal(&self) -> f32 {
        (self.lijn_verschijnt + self.vlak_opent + self.tekst_vervaagt).max(0.001)
    }
}

/// Wat voor Blok het is, voor animaties die een vlak anders laten bewegen dan letters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aard {
    /// Alles zonder eigen letters vooraan: vlakken, lijnen, balken, logo's.
    Vlak,
    Tekst,
    /// Een Getal: telt bij Optellen.
    Getal,
}

impl Aard {
    pub fn van(inhoud: &Inhoud) -> Aard {
        match inhoud {
            // de Partij met de functie verschijnt als tekst, als het vlak eromheen open is
            Inhoud::Tekst { .. } | Inhoud::Partijregel { .. } => Aard::Tekst,
            Inhoud::Getal { .. } => Aard::Getal,
            _ => Aard::Vlak,
        }
    }
}

/// Hoe een Blok erbij staat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Toestand {
    /// Doorzichtigheid: 0 onzichtbaar, 1 helemaal zichtbaar.
    pub dekking: f32,
    /// Verschuiving in pixels van het 1080p-beeld.
    pub schuif: Vec2,
    pub schaal: f32,
    /// Welk deel van de breedte zichtbaar is, vanaf links (opbouwen).
    pub breedte: f32,
    /// Welk deel van de hoogte zichtbaar is, rond het midden (inklappen).
    pub hoogte: f32,
    /// Hoe ver een Getal is met optellen: 0 bij nul, 1 bij het eigen getal.
    pub teller: f32,
    /// Zo hoog is het Blok minstens, in pixels, zolang `hoogte` onder 1 is: de lijn van
    /// Openen vanuit lijn.
    pub lijn: f32,
}

impl Toestand {
    /// Het eindbeeld: het Blok staat er.
    pub const VOL: Toestand = Toestand {
        dekking: 1.0,
        schuif: Vec2::ZERO,
        schaal: 1.0,
        breedte: 1.0,
        hoogte: 1.0,
        teller: 1.0,
        lijn: 0.0,
    };

    /// Of het Blok helemaal niet te zien is.
    #[cfg(test)]
    pub fn onzichtbaar(&self) -> bool {
        self.dekking <= 0.0 || self.breedte <= 0.0 || self.hoogte <= 0.0 || self.schaal <= 0.0
    }
}

/// De schaal van de teller bij een meerderheid op `t` seconden in een puls van `duur`: groeit
/// tot 1,2 en komt zacht terug op 1.
pub fn puls(t: f32, duur: f32) -> f32 {
    if duur <= 0.0 || !(0.0..duur).contains(&t) {
        return 1.0;
    }
    1.0 + 0.2 * (std::f32::consts::PI * t / duur).sin()
}

pub fn zacht_uit(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

fn zacht_in(t: f32) -> f32 {
    t.clamp(0.0, 1.0).powi(3)
}

/// Hoeveel langer alles duurt bij deze snelheden (in procent): 1 bij 100 en 100.
pub fn rek(stijl: u16, sjabloon: u16) -> f32 {
    100.0 / f32::from(stijl.max(10)) * 100.0 / f32::from(sjabloon.max(10))
}

/// Een Blok dat in beeld komt, op deel `t` (0 tot 1) van zijn beweging.
pub fn erin(soort: Erin, aard: Aard, t: f32, m: &Maten) -> Toestand {
    let t = t.clamp(0.0, 1.0);
    let z = zacht_uit(t);
    let vlieg = |richting: Vec2| Toestand {
        dekking: zacht_uit(t / m.zichtbaar_na),
        schuif: richting * m.vlieg * (1.0 - z),
        ..Toestand::VOL
    };
    match soort {
        Erin::Geen => Toestand::VOL,
        Erin::Vervagen => Toestand {
            dekking: z,
            ..Toestand::VOL
        },
        Erin::VanLinks => vlieg(Vec2::new(-1.0, 0.0)),
        Erin::VanRechts => vlieg(Vec2::new(1.0, 0.0)),
        Erin::VanOnder => vlieg(Vec2::new(0.0, 1.0)),
        Erin::Opbouwen => Toestand {
            breedte: z,
            ..Toestand::VOL
        },
        Erin::Groeien => Toestand {
            dekking: z,
            schaal: 0.5 + 0.5 * z,
            ..Toestand::VOL
        },
        Erin::Optellen => Toestand {
            dekking: zacht_uit(t / m.zichtbaar_na),
            teller: z,
            ..Toestand::VOL
        },
        Erin::OpenenVanuitLijn => {
            let totaal = m.lijn_totaal();
            let (lijn, open) = (
                m.lijn_verschijnt / totaal,
                (m.lijn_verschijnt + m.vlak_opent) / totaal,
            );
            match aard {
                Aard::Tekst | Aard::Getal => Toestand {
                    dekking: ((t - open) / (1.0 - open)).clamp(0.0, 1.0),
                    ..Toestand::VOL
                },
                Aard::Vlak if t < lijn => Toestand {
                    dekking: t / lijn,
                    hoogte: 0.0,
                    lijn: m.lijn,
                    ..Toestand::VOL
                },
                Aard::Vlak if t < open => Toestand {
                    hoogte: zacht_uit((t - lijn) / (open - lijn)),
                    lijn: m.lijn,
                    ..Toestand::VOL
                },
                Aard::Vlak => Toestand::VOL,
            }
        }
    }
}

/// Een Blok dat weggaat, op deel `t` (0 tot 1) van zijn beweging.
pub fn eruit(soort: Eruit, aard: Aard, t: f32, m: &Maten) -> Toestand {
    let t = t.clamp(0.0, 1.0);
    let z = zacht_in(t);
    let (tekst_weg, vlak_dicht) = (m.tekst_weg, m.vlak_dicht);
    let vlieg = |richting: Vec2| Toestand {
        dekking: 1.0 - z,
        schuif: richting * m.vlieg * z,
        ..Toestand::VOL
    };
    match soort {
        Eruit::Vervagen => Toestand {
            dekking: 1.0 - z,
            ..Toestand::VOL
        },
        Eruit::NaarLinks => vlieg(Vec2::new(-1.0, 0.0)),
        Eruit::NaarRechts => vlieg(Vec2::new(1.0, 0.0)),
        Eruit::NaarOnder => vlieg(Vec2::new(0.0, 1.0)),
        Eruit::Inklappen => Toestand {
            hoogte: 1.0 - z,
            ..Toestand::VOL
        },
        Eruit::SluitenNaarLijn => match aard {
            Aard::Tekst | Aard::Getal => Toestand {
                dekking: (1.0 - t / tekst_weg.max(0.001)).clamp(0.0, 1.0),
                ..Toestand::VOL
            },
            Aard::Vlak if t < tekst_weg => Toestand::VOL,
            Aard::Vlak if t < vlak_dicht => Toestand {
                hoogte: 1.0 - zacht_in((t - tekst_weg) / (vlak_dicht - tekst_weg)),
                lijn: m.lijn,
                ..Toestand::VOL
            },
            Aard::Vlak => Toestand {
                dekking: (1.0 - (t - vlak_dicht) / (1.0 - vlak_dicht).max(0.001)).clamp(0.0, 1.0),
                hoogte: 0.0,
                lijn: m.lijn,
                ..Toestand::VOL
            },
        },
    }
}

/// De tijden van één Graphic: wanneer elk Blok begint en hoe lang alles duurt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tijden {
    /// Duur van in beeld komen van één Blok, in seconden.
    pub duur: f32,
    /// Hoeveel langer alles duurt door de snelheden ([`rek`]).
    pub rek: f32,
    pub maten: Maten,
}

impl Tijden {
    pub fn new(beweging: &Beweging, stijl_snelheid: u16, sjabloon_snelheid: u16) -> Self {
        let maten = Maten::from(beweging);
        let rek = rek(stijl_snelheid, sjabloon_snelheid);
        Tijden {
            duur: maten.duur * rek,
            rek,
            maten,
        }
    }

    /// "Start na" telt mee met de snelheid.
    fn start(&self, a: &Animatie) -> f32 {
        a.start_na_ms as f32 / 1000.0 * self.rek
    }

    /// Duur van optellen: wat langer dan de rest, zodat je het getal ziet lopen.
    pub fn tellen(&self) -> f32 {
        self.duur * 1.4
    }

    fn duur_van(&self, a: &Animatie, aard: Aard) -> f32 {
        match a.erin {
            Erin::Optellen if aard == Aard::Getal => self.tellen(),
            Erin::OpenenVanuitLijn => self.maten.lijn_totaal() * self.rek,
            _ => self.duur,
        }
    }

    /// Blok met animatie `a`, `sinds` seconden nadat de Graphic in beeld begon te komen.
    /// Optellen werkt alleen bij een Getal; een ander Blok vervaagt dan.
    pub fn erin(&self, a: &Animatie, aard: Aard, sinds: f32) -> Toestand {
        let soort = if a.erin == Erin::Optellen && aard != Aard::Getal {
            Erin::Vervagen
        } else {
            a.erin
        };
        if soort == Erin::Geen {
            return if sinds >= self.start(a) {
                Toestand::VOL
            } else {
                erin(Erin::Vervagen, aard, 0.0, &self.maten)
            };
        }
        erin(
            soort,
            aard,
            (sinds - self.start(a)) / self.duur_van(a, aard),
            &self.maten,
        )
    }

    /// Hoe lang het duurt tot alle Blokken in beeld staan (optellen telt als getal: het langst).
    pub fn erin_klaar(&self, animaties: &[Animatie]) -> f32 {
        animaties
            .iter()
            .map(|a| self.start(a) + self.duur_van(a, Aard::Getal))
            .fold(0.0, f32::max)
    }

    /// Duur van weggaan van één Blok.
    fn weg(&self) -> f32 {
        self.duur * 0.7
    }

    /// Het laatste Blok (vooraan) gaat eerst; elk Blok eronder iets later.
    fn weg_start(&self, index: usize, aantal: usize) -> f32 {
        (aantal.saturating_sub(1).saturating_sub(index)) as f32 * self.duur * 0.12
    }

    /// Blok `index` van `aantal` met animatie `a`, `sinds` seconden nadat de Graphic begon weg
    /// te gaan.
    pub fn eruit(
        &self,
        a: &Animatie,
        aard: Aard,
        index: usize,
        aantal: usize,
        sinds: f32,
    ) -> Toestand {
        eruit(
            a.eruit,
            aard,
            (sinds - self.weg_start(index, aantal)) / self.weg(),
            &self.maten,
        )
    }

    /// Hoe lang het duurt tot alle Blokken weg zijn.
    pub fn eruit_klaar(&self, aantal: usize) -> f32 {
        self.weg_start(0, aantal) + self.weg()
    }

    /// Duur van Bijwerken: tekst wisselt in die tijd, getallen tellen, balken schuiven.
    pub fn bijwerken(&self) -> f32 {
        self.duur * 1.4
    }
}

/// Rondt een verschuiving af op hele pixels van het beeld dat de Renderer echt tekent, zodat
/// letters tijdens een beweging scherp blijven. `schaal` is het aantal beeldpixels per pixel
/// van het 1080p-ontwerp (1 voor de Ether, kleiner voor het Voorbeeld).
pub fn op_hele_pixels(v: Vec2, schaal: f32) -> Vec2 {
    (v * schaal).round() / schaal
}

#[cfg(test)]
mod tests {
    use super::*;

    use gedeeld::sjabloon::Lijnbeweging;

    /// Een Beweging met ronde getallen (voorbeeldgegevens), zodat de tijden hieronder na te
    /// rekenen zijn: in beeld komen 0,6 s, Openen vanuit lijn 0,1 + 0,2 + 0,1 s, lijn 3 px.
    fn proef() -> Beweging {
        Beweging {
            duur_ms: 600,
            vlieg_px: 150,
            zichtbaar_na_pct: 40,
            lijn: Lijnbeweging {
                verschijnt_ms: 100,
                opent_ms: 200,
                tekst_ms: 100,
                dikte_px: 3,
                tekst_weg_pct: 40,
                vlak_dicht_pct: 80,
            },
        }
    }

    fn tijden(stijl: u16, sjabloon: u16) -> Tijden {
        Tijden::new(&proef(), stijl, sjabloon)
    }

    fn erin(soort: Erin, aard: Aard, t: f32) -> Toestand {
        super::erin(soort, aard, t, &Maten::from(&proef()))
    }

    fn eruit(soort: Eruit, aard: Aard, t: f32) -> Toestand {
        super::eruit(soort, aard, t, &Maten::from(&proef()))
    }

    fn anim(erin: Erin, eruit: Eruit, start_na_ms: u32) -> Animatie {
        Animatie {
            erin,
            eruit,
            start_na_ms,
        }
    }

    #[test]
    fn elke_animatie_erin_begint_onzichtbaar_en_eindigt_in_het_eindbeeld() {
        for soort in Erin::ALLE {
            assert!(
                erin(soort, Aard::Vlak, 0.0).onzichtbaar(),
                "{soort:?} begint zichtbaar"
            );
            assert_eq!(erin(soort, Aard::Vlak, 1.0), Toestand::VOL, "{soort:?}");
            let half = erin(soort, Aard::Vlak, 0.5);
            assert_ne!(half, Toestand::VOL, "{soort:?} beweegt niet");
        }
    }

    #[test]
    fn elke_animatie_eruit_begint_in_het_eindbeeld_en_eindigt_onzichtbaar() {
        for soort in Eruit::ALLE {
            assert_eq!(eruit(soort, Aard::Vlak, 0.0), Toestand::VOL, "{soort:?}");
            assert!(
                eruit(soort, Aard::Vlak, 1.0).onzichtbaar(),
                "{soort:?} blijft zichtbaar"
            );
            assert_ne!(
                eruit(soort, Aard::Vlak, 0.5),
                Toestand::VOL,
                "{soort:?} beweegt niet"
            );
        }
    }

    #[test]
    fn invliegen_komt_van_de_goede_kant_en_schuift_nog_als_het_al_helemaal_zichtbaar_is() {
        let erin = |s, t| erin(s, Aard::Vlak, t);
        assert!(erin(Erin::VanLinks, 0.1).schuif.x < 0.0);
        assert!(erin(Erin::VanRechts, 0.1).schuif.x > 0.0);
        assert!(
            erin(Erin::VanOnder, 0.1).schuif.y > 0.0,
            "onder is een positieve y"
        );
        let t = erin(Erin::VanLinks, 0.5);
        assert_eq!(t.dekking, 1.0);
        assert!(t.schuif.x < -5.0, "nog onderweg: {t:?}");
    }

    #[test]
    fn start_na_en_snelheid_verschuiven_en_rekken_de_beweging() {
        let normaal = tijden(100, 100);
        let a = anim(Erin::Vervagen, Eruit::Vervagen, 500);
        assert!(
            normaal.erin(&a, Aard::Vlak, 0.45).onzichtbaar(),
            "wacht nog 0,5 s"
        );
        assert_eq!(normaal.erin(&a, Aard::Vlak, 0.5 + 0.6), Toestand::VOL);
        assert!((normaal.erin_klaar(&[a]) - 1.1).abs() < 1e-5);

        // dubbel zo snel in de Stijl: alles in de helft van de tijd, ook "start na"
        let snel = tijden(200, 100);
        assert!((snel.erin_klaar(&[a]) - 0.55).abs() < 1e-5);
        // het Sjabloon telt erbovenop
        assert!((tijden(200, 200).erin_klaar(&[a]) - 0.275).abs() < 1e-5);
    }

    #[test]
    fn optellen_telt_alleen_bij_een_getal_en_een_ander_blok_vervaagt() {
        let t = tijden(100, 100);
        let a = anim(Erin::Optellen, Eruit::Vervagen, 0);
        let getal = t.erin(&a, Aard::Getal, t.tellen() / 2.0);
        assert!(getal.teller > 0.0 && getal.teller < 1.0);
        let ander = t.erin(&a, Aard::Tekst, t.duur / 2.0);
        assert_eq!(ander, erin(Erin::Vervagen, Aard::Tekst, 0.5));
    }

    #[test]
    fn de_puls_bij_een_meerderheid_groeit_en_komt_terug_op_zijn_maat() {
        assert_eq!(puls(0.0, 0.6), 1.0);
        assert_eq!(puls(0.6, 0.6), 1.0);
        assert_eq!(puls(2.0, 0.6), 1.0, "na de puls blijft hij op zijn maat");
        let midden = puls(0.3, 0.6);
        assert!(midden > 1.15 && midden <= 1.2, "{midden}");
        assert!(puls(0.15, 0.6) > 1.0 && puls(0.15, 0.6) < midden);
    }

    #[test]
    fn het_voorste_blok_gaat_eerst_weg() {
        let t = tijden(100, 100);
        let a = anim(Erin::Vervagen, Eruit::Vervagen, 0);
        let vroeg = t.duur * 0.1;
        assert!(
            t.eruit(&a, Aard::Vlak, 2, 3, vroeg).dekking < 1.0,
            "het voorste Blok is al begonnen"
        );
        assert_eq!(
            t.eruit(&a, Aard::Vlak, 0, 3, vroeg),
            Toestand::VOL,
            "het achterste wacht nog"
        );
        assert!(
            t.eruit(&a, Aard::Vlak, 0, 3, t.eruit_klaar(3) + 1e-4)
                .onzichtbaar()
        );
    }

    #[test]
    fn openen_vanuit_lijn_toont_eerst_een_lijn_dan_het_vlak_en_dan_de_tekst() {
        let t = tijden(100, 100);
        let a = anim(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0);
        let vlak = |s| t.erin(&a, Aard::Vlak, s);
        let tekst = |s| t.erin(&a, Aard::Tekst, s);
        assert!(vlak(0.0).onzichtbaar() && tekst(0.0).onzichtbaar());
        // 50 ms: een dunne lijn die verschijnt, over de hele breedte
        let lijn = vlak(0.05);
        assert!(lijn.dekking > 0.2 && lijn.dekking < 1.0, "{lijn:?}");
        assert_eq!((lijn.hoogte, lijn.breedte, lijn.lijn), (0.0, 1.0, 3.0));
        assert!(tekst(0.05).onzichtbaar());
        // 200 ms: het vlak gaat open
        let open = vlak(0.2);
        assert_eq!(open.dekking, 1.0);
        assert!(open.hoogte > 0.3 && open.hoogte < 1.0, "{open:?}");
        assert!(tekst(0.2).onzichtbaar());
        // 300 ms: het vlak staat, de tekst begint te vervagen
        assert_eq!(vlak(0.3), Toestand::VOL);
        assert!(tekst(0.3).dekking < 0.05);
        assert!(
            (tekst(0.35).dekking - 0.5).abs() < 0.05,
            "{:?}",
            tekst(0.35)
        );
        assert_eq!(tekst(0.4), Toestand::VOL);
        assert!((t.erin_klaar(&[a]) - 0.4).abs() < 1e-5);
        // dubbel zo snel: alles in de helft van de tijd
        assert_eq!(tijden(200, 100).erin(&a, Aard::Vlak, 0.15), Toestand::VOL);
    }

    #[test]
    fn sluiten_naar_lijn_laat_eerst_de_tekst_gaan_en_dan_het_vlak() {
        let t = tijden(100, 100);
        let a = anim(Erin::OpenenVanuitLijn, Eruit::SluitenNaarLijn, 0);
        // één Blok: geen wachten op Blokken ervoor
        // weggaan duurt 0,42 s: de tekst is weg na 40 procent, het vlak dicht na 80 procent
        assert!(
            t.eruit(&a, Aard::Tekst, 0, 1, 0.17).onzichtbaar(),
            "tekst weg na 170 ms"
        );
        assert_eq!(
            t.eruit(&a, Aard::Vlak, 0, 1, 0.1),
            Toestand::VOL,
            "het vlak wacht"
        );
        let dicht = t.eruit(&a, Aard::Vlak, 0, 1, 0.38);
        assert_eq!((dicht.hoogte, dicht.lijn), (0.0, 3.0), "{dicht:?}");
        assert!(dicht.dekking > 0.0);
        assert!(
            t.eruit(&a, Aard::Vlak, 0, 1, t.eruit_klaar(1))
                .onzichtbaar()
        );
    }

    #[test]
    fn de_beweging_van_de_stijl_bepaalt_de_tijden() {
        let mut traag = proef();
        traag.duur_ms = 1200;
        let a = anim(Erin::Vervagen, Eruit::Vervagen, 0);
        assert!((Tijden::new(&traag, 100, 100).erin_klaar(&[a]) - 1.2).abs() < 1e-5);
        assert!(
            (Tijden::new(&traag, 200, 100).erin_klaar(&[a]) - 0.6).abs() < 1e-5,
            "snelheid telt erbovenop"
        );
    }

    #[test]
    fn verschuivingen_vallen_op_hele_pixels_van_het_getekende_beeld() {
        assert_eq!(
            op_hele_pixels(Vec2::new(-17.4, 0.6), 1.0),
            Vec2::new(-17.0, 1.0)
        );
        // het Voorbeeld tekent op een derde: hele pixels daar zijn stappen van 3 ontwerppixels
        let v = op_hele_pixels(Vec2::new(10.0, 0.0), 1.0 / 3.0);
        assert!((v.x - 9.0).abs() < 1e-4, "{v:?}");
    }
}
