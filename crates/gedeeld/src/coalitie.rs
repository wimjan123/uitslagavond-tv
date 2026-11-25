//! De Coalitiebouwer: de teller naar een meerderheid van 76 Zetels, en de Stoelen in de
//! Uitslagindeling met de gekozen Partijen samen als één blok links (besluit #7).
//!
//! De gekozen Partijen schuiven samen naar links, in de volgorde van de Uitslagindeling; de andere
//! Partijen staan daarna, ook op volgorde, in grijs (in de Zaal donker neutraal, net als Zetels
//! zonder Partij).

use crate::cijfers::Cijferset;

/// Zoveel Zetels is een meerderheid in de Tweede Kamer.
pub const MEERDERHEID: u32 = 76;

/// De Zetels van een coalitie samen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Telling {
    pub zetels: u32,
}

impl Telling {
    /// De Zetels van de `gekozen` Partijen in `set` samen. Een Partij die er twee keer in staat,
    /// telt één keer.
    pub fn van(set: &Cijferset, gekozen: &[String]) -> Telling {
        let mut gezien: Vec<&str> = Vec::new();
        let mut zetels = 0;
        for p in gekozen {
            if !gezien.contains(&p.as_str()) {
                gezien.push(p);
                zetels += set.zetels_van(p);
            }
        }
        Telling { zetels }
    }

    /// Of de coalitie 76 Zetels of meer heeft.
    pub fn meerderheid(&self) -> bool {
        self.zetels >= MEERDERHEID
    }

    /// Hoeveel Zetels er nog nodig zijn voor 76; 0 bij een meerderheid.
    pub fn tekort(&self) -> u32 {
        MEERDERHEID.saturating_sub(self.zetels)
    }

    /// De regel onder "zetels samen": "1 tekort voor 76", of "meerderheid".
    pub fn regel(&self) -> String {
        if self.meerderheid() {
            "meerderheid".into()
        } else {
            format!("{} tekort voor {MEERDERHEID}", self.tekort())
        }
    }
}

/// De korte namen als kop: "D66, CDA, VVD en JA21".
pub fn namen<S: AsRef<str>>(korte: &[S]) -> String {
    match korte {
        [] => String::new(),
        [een] => een.as_ref().into(),
        [voor @ .., laatste] => {
            let voor: Vec<&str> = voor.iter().map(AsRef::as_ref).collect();
            format!("{} en {}", voor.join(", "), laatste.as_ref())
        }
    }
}

/// De `gekozen` Partijen in de volgorde van de Uitslagindeling (`volgorde`). Gekozen Partijen
/// die niet in de volgorde staan, komen achteraan.
pub fn op_volgorde(volgorde: &[String], gekozen: &[String]) -> Vec<String> {
    let mut uit: Vec<String> = volgorde
        .iter()
        .filter(|p| gekozen.contains(p))
        .cloned()
        .collect();
    for p in gekozen {
        if !uit.contains(p) {
            uit.push(p.clone());
        }
    }
    uit
}

/// De volgorde van de Stoelen in de Coalitiebouwer: eerst de gekozen Partijen, dan de rest, elk
/// in de volgorde van de Uitslagindeling. Per Partij of hij in de coalitie zit.
pub fn blokken(volgorde: &[String], gekozen: &[String]) -> Vec<(String, bool)> {
    let coalitie = op_volgorde(volgorde, gekozen);
    let rest = volgorde.iter().filter(|p| !coalitie.contains(p));
    coalitie
        .iter()
        .map(|p| (p.clone(), true))
        .chain(rest.map(|p| (p.clone(), false)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cijfers::Soort;

    fn set(zetels: &[(&str, u16)]) -> Cijferset {
        Cijferset {
            id: "nu".into(),
            soort: Soort::Uitslag,
            label: "Voorbeeld".into(),
            tijd: None,
            bron: None,
            zetels: zetels.iter().map(|(p, z)| (p.to_string(), *z)).collect(),
            stemmen: None,
            vergrendeld: false,
            versie: 1,
        }
    }

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn de_teller_telt_de_zetels_van_de_gekozen_partijen() {
        let set = set(&[
            ("d66", 26),
            ("cda", 18),
            ("vvd", 22),
            ("ja21", 9),
            ("pvv", 26),
        ]);
        assert_eq!(
            Telling::van(&set, &s(&["d66", "cda", "vvd", "ja21"])).zetels,
            75
        );
        assert_eq!(Telling::van(&set, &[]).zetels, 0);
        // twee keer dezelfde Partij telt één keer; een Partij zonder Zetels telt 0
        assert_eq!(Telling::van(&set, &s(&["d66", "d66", "sp"])).zetels, 26);
    }

    #[test]
    fn een_meerderheid_is_76_zetels_of_meer() {
        assert!(!Telling { zetels: 75 }.meerderheid());
        assert!(Telling { zetels: 76 }.meerderheid());
        assert!(Telling { zetels: 150 }.meerderheid());
        assert!(!Telling { zetels: 0 }.meerderheid());
    }

    #[test]
    fn de_regel_noemt_het_tekort_tot_76_of_de_meerderheid() {
        assert_eq!(Telling { zetels: 75 }.tekort(), 1);
        assert_eq!(Telling { zetels: 75 }.regel(), "1 tekort voor 76");
        assert_eq!(Telling { zetels: 40 }.regel(), "36 tekort voor 76");
        assert_eq!(Telling { zetels: 76 }.tekort(), 0);
        assert_eq!(Telling { zetels: 76 }.regel(), "meerderheid");
        assert_eq!(Telling { zetels: 101 }.regel(), "meerderheid");
    }

    #[test]
    fn de_kop_noemt_de_partijen_met_komma_s_en_en() {
        assert_eq!(
            namen(&["D66", "CDA", "VVD", "JA21"]),
            "D66, CDA, VVD en JA21"
        );
        assert_eq!(namen(&["D66", "CDA"]), "D66 en CDA");
        assert_eq!(namen(&["D66"]), "D66");
        assert_eq!(namen::<&str>(&[]), "");
    }

    #[test]
    fn de_gekozen_partijen_staan_in_de_volgorde_van_de_uitslagindeling() {
        let volgorde = s(&["sp", "d66", "cda", "vvd", "ja21", "pvv"]);
        assert_eq!(
            op_volgorde(&volgorde, &s(&["ja21", "d66", "vvd"])),
            s(&["d66", "vvd", "ja21"])
        );
        // een onbekende Partij achteraan
        assert_eq!(
            op_volgorde(&volgorde, &s(&["nieuw", "cda"])),
            s(&["cda", "nieuw"])
        );
    }

    #[test]
    fn de_coalitie_schuift_als_een_blok_naar_links_en_de_rest_volgt_op_volgorde() {
        let volgorde = s(&["sp", "d66", "cda", "vvd", "ja21", "pvv"]);
        let b = blokken(&volgorde, &s(&["vvd", "d66"]));
        let verwacht: Vec<(String, bool)> = [
            ("d66", true),
            ("vvd", true),
            ("sp", false),
            ("cda", false),
            ("ja21", false),
            ("pvv", false),
        ]
        .iter()
        .map(|(p, c)| (p.to_string(), *c))
        .collect();
        assert_eq!(b, verwacht);
    }
}
