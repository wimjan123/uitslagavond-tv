//! De Uitslagindeling: de Zetels van een set op de 150 Stoelen, per Partij één taartpunt van
//! voor tot achter (besluit #7).
//!
//! De Stoelen staan gesorteerd op hun hoek vanaf het midden van de Zaal, van links naar rechts
//! in beeld vanaf de hoofdcamera achter de Voorzitter ([`stoelen_van_links_naar_rechts`]). De
//! Partijen krijgen ze op volgorde: de vaste lijst van het programma ([`Uitslagvolgorde`]), die
//! de Regie vóór de uitzending instelt. Zetels zonder Partij zijn grijs, rechts achteraan.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::cijfers::{Cijfers, Cijferset, Partij, Soort, ZETELS};
use crate::kamer::Kamer;
use crate::zaalindeling::Zaalindeling;
use crate::sjabloon::GRIJS;
use crate::zaalkaart::STOELPLEKKEN;
use crate::{AANTAL_STOELEN, Kleur};

/// Het id van de set die een Zetelverdeling in de Zaalindeling vasthoudt.
pub const ZAALINDELING: &str = "zaalindeling";
/// Het id van de set met de Zetels van de huidige Kamer, uit de open data: per Fractie haar
/// Zetels. De Coalitiebouwer werkt er ook mee (besluit #7).
pub const HUIDIGE_KAMER: &str = "huidige-kamer";

/// De standaardvolgorde: politiek van links naar rechts, met de sleutels van het invoerbestand
/// (`cijfers/uitslag-tk2025.json`). De Regie stelt de echte lijst vóór de uitzending in.
pub const STANDAARD_VOLGORDE: [&str; 18] = [
    "sp", "pvdd", "bij1", "gl-pvda", "denk", "volt", "d66", "cu", "50plus", "cda", "nsc", "bbb",
    "vvd", "sgp", "ja21", "bvnl", "pvv", "fvd",
];

/// De volgorde van de Partijen in de Uitslagindeling, zoals de Regie die vóór de uitzending
/// instelt. Verandert niet tijdens de uitzending.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Uitslagvolgorde {
    /// Sleutels van Partijen (en Fracties zonder lijst), van links naar rechts.
    pub lijst: Vec<String>,
    /// Per Fractie zonder lijst de sleutel van de Fractie waar ze uit kwam. Staat ze zelf niet in
    /// de lijst, dan komt ze direct rechts naast die Fractie.
    pub herkomst: BTreeMap<String, String>,
}

impl Default for Uitslagvolgorde {
    fn default() -> Self {
        Uitslagvolgorde {
            lijst: STANDAARD_VOLGORDE.iter().map(|s| s.to_string()).collect(),
            herkomst: BTreeMap::new(),
        }
    }
}

/// De volgorde waarin de Partijen van de partijlijst nu in de Uitslagindeling staan.
#[derive(Serialize, Deserialize, TS, Clone, Debug, Default, PartialEq, Eq)]
#[ts(export)]
pub struct Volgorde {
    /// Elke Partij van de partijlijst, van links naar rechts.
    pub partijen: Vec<String>,
    /// Partijen met Zetels die niet in de lijst staan: ze staan rechts achteraan en de Regie
    /// krijgt een waarschuwing.
    pub niet_op_lijst: Vec<String>,
}

impl Volgorde {
    /// De plaats van een Partij van links naar rechts; een onbekende Partij achteraan.
    pub fn plaats(&self, sleutel: &str) -> usize {
        self.partijen
            .iter()
            .position(|p| p == sleutel)
            .unwrap_or(self.partijen.len())
    }
}

impl Uitslagvolgorde {
    /// De volgorde van alle Partijen in `cijfers`: eerst de lijst, dan elke Fractie zonder lijst
    /// direct rechts naast haar herkomst, dan de rest rechts achteraan in de volgorde van de
    /// partijlijst.
    pub fn bepaal(&self, cijfers: &Cijfers) -> Volgorde {
        let bekend = |s: &str| cijfers.partij(s).is_some();
        let mut uit: Vec<String> = Vec::new();
        for s in &self.lijst {
            if bekend(s) && !uit.contains(s) {
                uit.push(s.clone());
            }
        }
        let naast = |p: &Partij| {
            self.herkomst
                .get(&p.sleutel)
                .filter(|h| p.fractie_zonder_lijst && bekend(h) && **h != p.sleutel)
        };
        let mut niet_op_lijst = Vec::new();
        let met_zetels = |s: &str| cijfers.sets.iter().any(|set| set.zetels_van(s) > 0);
        for p in &cijfers.partijen {
            if !uit.contains(&p.sleutel) && naast(p).is_none() {
                uit.push(p.sleutel.clone());
                if met_zetels(&p.sleutel) {
                    niet_op_lijst.push(p.sleutel.clone());
                }
            }
        }
        // Fracties zonder lijst naast hun herkomst; ook naast een andere Fractie zonder lijst.
        let mut over: Vec<&Partij> = cijfers
            .partijen
            .iter()
            .filter(|p| !uit.contains(&p.sleutel))
            .collect();
        while !over.is_empty() {
            let voor = over.len();
            over.retain(|p| {
                let h = naast(p).expect("alleen Fracties met herkomst blijven over");
                let Some(mut i) = uit.iter().position(|s| s == h) else {
                    return true;
                };
                // na eerdere afsplitsingen van dezelfde Fractie
                while uit
                    .get(i + 1)
                    .is_some_and(|s| self.herkomst.get(s) == Some(h) && !self.lijst.contains(s))
                {
                    i += 1;
                }
                uit.insert(i + 1, p.sleutel.clone());
                false
            });
            if over.len() == voor {
                // een kring van herkomsten: rechts achteraan
                for p in over.drain(..) {
                    uit.push(p.sleutel.clone());
                    if met_zetels(&p.sleutel) {
                        niet_op_lijst.push(p.sleutel.clone());
                    }
                }
            }
        }
        Volgorde {
            partijen: uit,
            niet_op_lijst,
        }
    }
}

/// De 150 Stoelen (als plaats in [`crate::Stoel::alle`]) van links naar rechts in beeld vanaf
/// de hoofdcamera achter de Voorzitter: gesorteerd op hun hoek vanaf het midden van de Zaal,
/// bij gelijke hoek de voorste eerst.
pub fn stoelen_van_links_naar_rechts() -> Vec<usize> {
    let hoek = |i: usize| STOELPLEKKEN[i][0].atan2(-STOELPLEKKEN[i][2]);
    let straal = |i: usize| STOELPLEKKEN[i][0].hypot(STOELPLEKKEN[i][2]);
    let mut v: Vec<usize> = (0..AANTAL_STOELEN).collect();
    v.sort_by(|&a, &b| {
        hoek(a)
            .total_cmp(&hoek(b))
            .then(straal(a).total_cmp(&straal(b)))
    });
    v
}

/// De lijn bij Zetel 76 in de Zaal (besluit #7): tussen de 75e en de 76e Stoel van links, van
/// voor de voorste rij tot achter de achterste, boven gezien recht vanuit het midden van de Zaal.
/// Begin en eind in meter in de assen van [`STOELPLEKKEN`], op de hoogte van de rugleuningen.
pub fn lijn_bij_76() -> ([f32; 3], [f32; 3]) {
    let hoek = |i: usize| STOELPLEKKEN[i][0].atan2(-STOELPLEKKEN[i][2]);
    let straal = |p: &[f32; 3]| p[0].hypot(p[2]);
    let volgorde = stoelen_van_links_naar_rechts();
    let laatste = crate::coalitie::MEERDERHEID as usize - 1;
    let h = (hoek(volgorde[laatste - 1]) + hoek(volgorde[laatste])) / 2.0;
    let (binnen, buiten) = STOELPLEKKEN
        .iter()
        .fold((f32::MAX, 0.0f32), |(b, u), p| (b.min(straal(p)), u.max(straal(p))));
    let (laag, hoog) = STOELPLEKKEN
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), p| (l.min(p[1]), h.max(p[1])));
    let punt = |r: f32, y: f32| [r * h.sin(), y, -r * h.cos()];
    (punt(binnen - 0.6, laag), punt(buiten + 0.6, hoog))
}

/// Verdeelt de Stoelen: elk deel (kleur en aantal Zetels) krijgt op volgorde de volgende
/// Stoelen van links naar rechts; wat overblijft is grijs. Geeft een kleur per Stoel in de
/// volgorde van [`crate::Stoel::alle`].
pub fn verdeel(delen: impl IntoIterator<Item = (Kleur, u32)>) -> Vec<Kleur> {
    let mut kleuren = vec![GRIJS; AANTAL_STOELEN];
    let mut stoelen = stoelen_van_links_naar_rechts().into_iter();
    for (kleur, zetels) in delen {
        for i in stoelen.by_ref().take(zetels as usize) {
            kleuren[i] = kleur;
        }
    }
    kleuren
}

/// De kleur per Stoel voor set `id` in deze volgorde. `None` als de set er niet is.
pub fn zaal_voor_set(volgorde: &Volgorde, cijfers: &Cijfers, id: &str) -> Option<Vec<Kleur>> {
    let set = cijfers.set(id)?;
    Some(verdeel(delen(volgorde, cijfers, set).map(|(p, z)| {
        (p.kleur.parse().unwrap_or(GRIJS), z)
    })))
}

/// De Partijen met Zetels in `set`, van links naar rechts, met hun Zetels.
fn delen<'a>(
    volgorde: &'a Volgorde,
    cijfers: &'a Cijfers,
    set: &'a Cijferset,
) -> impl Iterator<Item = (&'a Partij, u32)> + 'a {
    volgorde
        .partijen
        .iter()
        .filter_map(|s| cijfers.partij(s))
        .map(|p| (p, set.zetels_van(&p.sleutel)))
        .filter(|(_, z)| *z > 0)
        .scan(0u32, |totaal, (p, z)| {
            // nooit meer dan 150 Stoelen
            let z = z.min(ZETELS - *totaal);
            *totaal += z;
            Some((p, z))
        })
}

/// De cijfers van een Zetelverdeling zoals ze waren bij klaarzetten: de set met zijn versie en
/// de Partijen met Zetels, van links naar rechts. Een Graphic met deze kopie verandert niet als
/// de set verandert, ook niet na een herstart, tot Bijwerken.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Zetelstand {
    pub set: Cijferset,
    pub partijen: Vec<Partij>,
    /// Alleen in de Zaalindeling: per Stoel, in de volgorde van [`crate::Stoel::alle`], de
    /// Partij van de Fractie van wie er zit. Leeg: niemand, grijs.
    #[serde(default)]
    #[ts(optional)]
    pub stoelen: Option<Vec<Option<String>>>,
    /// Alleen bij Twee sets naast elkaar: de oudste van de twee sets. `set` is dan de nieuwste,
    /// die de Zaal toont, en `partijen` zijn de Partijen met Zetels in een van beide.
    #[serde(default)]
    #[ts(optional)]
    pub vorige: Option<Cijferset>,
}

impl Zetelstand {
    /// De kopie van set `id` in deze volgorde. `None` als de set er niet is.
    pub fn van(volgorde: &Volgorde, cijfers: &Cijfers, id: &str) -> Option<Zetelstand> {
        let set = cijfers.set(id)?;
        Some(Zetelstand {
            set: set.clone(),
            partijen: delen(volgorde, cijfers, set)
                .map(|(p, _)| p.clone())
                .collect(),
            stoelen: None,
            vorige: None,
        })
    }

    /// De kopie van twee sets: de set die later in de cijfers staat is de nieuwste. `None` als
    /// een van beide er niet is.
    pub fn van_twee(volgorde: &Volgorde, cijfers: &Cijfers, a: &str, b: &str) -> Option<Zetelstand> {
        let plaats = |id: &str| cijfers.sets.iter().position(|s| s.id == id);
        let (pa, pb) = (plaats(a)?, plaats(b)?);
        let (oud, nieuw) = if pa <= pb { (a, b) } else { (b, a) };
        Zetelstand::tegen(volgorde, cijfers, nieuw, oud)
    }

    /// De kopie van set `id` met set `vorige` ernaast (Winst en verlies, Twee sets naast elkaar):
    /// de Partijen met Zetels in een van beide. `None` als een van beide er niet is.
    pub fn tegen(volgorde: &Volgorde, cijfers: &Cijfers, id: &str, vorige: &str) -> Option<Zetelstand> {
        let (oud, nieuw) = (cijfers.set(vorige)?, cijfers.set(id)?);
        let partijen = volgorde
            .partijen
            .iter()
            .filter(|s| oud.zetels_van(s) > 0 || nieuw.zetels_van(s) > 0)
            .filter_map(|s| cijfers.partij(s).cloned())
            .collect();
        Some(Zetelstand {
            set: nieuw.clone(),
            partijen,
            stoelen: None,
            vorige: Some(oud.clone()),
        })
    }

    /// De Kamer die zit in de Zaalindeling: per Stoel de Partij van wie er zit, en een set met
    /// het aantal Stoelen per Partij voor de labels.
    pub fn van_zaal(volgorde: &Volgorde, cijfers: &Cijfers, kamer: &Kamer, zaal: &Zaalindeling) -> Zetelstand {
        let stoelen: Vec<Option<String>> = zaal
            .kamerleden(kamer)
            .into_iter()
            .map(|k| k.and_then(|k| kamer.partij_van(k)).map(str::to_owned))
            .collect();
        let mut zetels = BTreeMap::new();
        for p in stoelen.iter().flatten() {
            *zetels.entry(p.clone()).or_insert(0u16) += 1;
        }
        let partijen = volgorde
            .partijen
            .iter()
            .filter(|s| zetels.contains_key(*s))
            .filter_map(|s| cijfers.partij(s).cloned())
            .collect();
        Zetelstand {
            set: Cijferset {
                id: ZAALINDELING.into(),
                soort: Soort::Uitslag,
                label: "Huidige Kamer".into(),
                tijd: kamer.peildatum.clone(),
                bron: Some(zaal.bron.clone()),
                zetels,
                stemmen: None,
                vergrendeld: false,
                versie: 0,
            },
            partijen,
            stoelen: Some(stoelen),
            vorige: None,
        }
    }

    /// De kopie van de Zetels van de huidige Kamer: per Fractie haar Zetels uit de open data, bij
    /// de Partij waar ze bij hoort. Fracties van dezelfde Partij tellen samen.
    pub fn van_kamer(volgorde: &Volgorde, cijfers: &Cijfers, kamer: &Kamer) -> Zetelstand {
        let mut zetels: BTreeMap<String, u16> = BTreeMap::new();
        for f in &kamer.fracties {
            if let Some(p) = kamer.partij_van.get(&f.id) {
                *zetels.entry(p.clone()).or_insert(0) += f.zetels;
            }
        }
        let set = Cijferset {
            id: HUIDIGE_KAMER.into(),
            soort: Soort::Uitslag,
            label: "Huidige Kamer".into(),
            tijd: kamer.peildatum.clone(),
            bron: Some("open data Tweede Kamer".into()),
            zetels,
            stemmen: None,
            vergrendeld: false,
            versie: 0,
        };
        Zetelstand {
            partijen: delen(volgorde, cijfers, &set).map(|(p, _)| p.clone()).collect(),
            set,
            stoelen: None,
            vorige: None,
        }
    }

    /// De kleur per Stoel in de Coalitiebouwer: de `gekozen` Partijen samen als één blok links,
    /// daarna de rest donker, elk in de volgorde van de Uitslagindeling (besluit #7). Altijd in
    /// de Uitslagindeling, ook als de kopie uit de Zaalindeling komt.
    pub fn coalitiekleuren(&self, gekozen: &[String]) -> Vec<Kleur> {
        let volgorde: Vec<String> = self.partijen.iter().map(|p| p.sleutel.clone()).collect();
        let kleur = |s: &str| {
            self.partijen.iter().find(|p| p.sleutel == s).and_then(|p| p.kleur.parse().ok()).unwrap_or(GRIJS)
        };
        verdeel(crate::coalitie::blokken(&volgorde, gekozen).into_iter().map(|(p, in_coalitie)| {
            let k = kleur(&p);
            (if in_coalitie { k } else { crate::coalitie::donker(k) }, self.set.zetels_van(&p))
        }))
    }

    /// De kopie als cijfers, zodat een Sjabloon er zijn bronnen in opzoekt.
    pub fn als_cijfers(&self) -> Cijfers {
        Cijfers {
            partijen: self.partijen.clone(),
            sets: self.vorige.iter().chain([&self.set]).cloned().collect(),
            ..Cijfers::default()
        }
    }

    /// De kleur per Stoel, in de volgorde van [`crate::Stoel::alle`].
    pub fn stoelkleuren(&self) -> Vec<Kleur> {
        if let Some(stoelen) = &self.stoelen {
            let kleur = |s: &str| {
                self.partijen.iter().find(|p| p.sleutel == s).and_then(|p| p.kleur.parse().ok())
            };
            return stoelen.iter().map(|p| p.as_deref().and_then(kleur).unwrap_or(GRIJS)).collect();
        }
        verdeel(self.partijen.iter().map(|p| {
            (p.kleur.parse().unwrap_or(GRIJS), self.set.zetels_van(&p.sleutel))
        }))
    }

    /// De plaatsen van links naar rechts ([`stoelen_van_links_naar_rechts`]) van de eerste en de
    /// laatste Stoel van `partij`, tot en met. In de Uitslagindeling is dat haar blok; in de
    /// Zaalindeling alles van haar meest linkse tot haar meest rechtse Kamerlid. `None` zonder
    /// Zetels.
    pub fn plaatsen_van(&self, partij: &str) -> Option<(usize, usize)> {
        if let Some(stoelen) = &self.stoelen {
            let volgorde = stoelen_van_links_naar_rechts();
            let van_haar: Vec<usize> = (0..volgorde.len())
                .filter(|&r| stoelen.get(volgorde[r]).and_then(|p| p.as_deref()) == Some(partij))
                .collect();
            return Some((*van_haar.first()?, *van_haar.last()?));
        }
        let mut begin = 0;
        for p in &self.partijen {
            let zetels = self.set.zetels_van(&p.sleutel) as usize;
            if p.sleutel == partij {
                return (zetels > 0).then(|| (begin, begin + zetels - 1));
            }
            begin += zetels;
        }
        None
    }

    /// De kleur per Stoel bij Partij uitgelicht: de Stoelen van `partij` op hun plek in de
    /// Uitslagindeling en in haar kleur, alle andere grijs.
    pub fn stoelkleuren_van(&self, partij: &str) -> Vec<Kleur> {
        verdeel(self.partijen.iter().map(|p| {
            let kleur = if p.sleutel == partij { p.kleur.parse().unwrap_or(GRIJS) } else { GRIJS };
            (kleur, self.set.zetels_van(&p.sleutel))
        }))
    }
}
