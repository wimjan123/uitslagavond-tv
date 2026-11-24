//! De delen van een Zetelbalk: welke Partijen een eigen deel krijgen en wat samen "overig" is,
//! en het pijltje voor het verschil tussen twee sets (Twee sets naast elkaar).
//!
//! Een Zetelbalk toont alle Partijen van een set, of de grootste N plus "overig". De delen staan
//! in de volgorde van de Uitslagindeling; overig staat achteraan. Bij Twee sets komen de
//! grootste N uit de nieuwste set, en de vorige set gebruikt dezelfde Partijen, zodat beide
//! balken dezelfde volgorde hebben.

use crate::cijfers::{Cijferset, Regel};

/// Een deel van een Zetelbalk, of een rij van de Uitslaggrafiek.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Balkdeel {
    /// De sleutel van de Partij; `None`: overig, alle Partijen zonder eigen deel samen.
    pub partij: Option<String>,
    pub zetels: u32,
}

/// Het verschil van een deel tegen de vorige set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pijl {
    Omhoog(u32),
    Omlaag(u32),
    Gelijk,
}

/// De Partijen uit `volgorde` die een eigen deel krijgen: alle met Zetels in `set`, of de `n`
/// grootste. Bij gelijke Zetels gaat de Partij voor die verder links staat. Op volgorde.
pub fn grootste(set: &Cijferset, volgorde: &[String], n: Option<u16>) -> Vec<String> {
    let mut met_zetels: Vec<(usize, &String)> = volgorde
        .iter()
        .enumerate()
        .filter(|(_, p)| set.zetels_van(p) > 0)
        .collect();
    if let Some(n) = n {
        met_zetels.sort_by_key(|(plaats, p)| (std::cmp::Reverse(set.zetels_van(p)), *plaats));
        met_zetels.truncate(usize::from(n));
        met_zetels.sort_by_key(|(plaats, _)| *plaats);
    }
    met_zetels.into_iter().map(|(_, p)| p.clone()).collect()
}

/// De delen van `set`: de `gekozen` Partijen met Zetels in de volgorde van `volgorde`, daarna
/// overig met alle andere Zetels (ook die de set zelf overig noemt), als dat er zijn.
pub fn delen(set: &Cijferset, volgorde: &[String], gekozen: &[String]) -> Vec<Balkdeel> {
    let mut uit: Vec<Balkdeel> = volgorde
        .iter()
        .filter(|p| gekozen.contains(p))
        .map(|p| Balkdeel { partij: Some(p.clone()), zetels: set.zetels_van(p) })
        .filter(|d| d.zetels > 0)
        .collect();
    let eigen: u32 = uit.iter().map(|d| d.zetels).sum();
    let overig = set.totaal().saturating_sub(eigen);
    if overig > 0 {
        uit.push(Balkdeel { partij: None, zetels: overig });
    }
    uit
}

/// Het pijltje bij `deel` van de nieuwste set: het verschil met hetzelfde deel (dezelfde Partij,
/// of overig) in de `vorige` set. Geen deel in de vorige set: nul Zetels.
pub fn pijl(vorige: &[Balkdeel], deel: &Balkdeel) -> Pijl {
    let was = vorige
        .iter()
        .find(|v| v.partij == deel.partij)
        .map_or(0, |v| v.zetels);
    match deel.zetels.cmp(&was) {
        std::cmp::Ordering::Greater => Pijl::Omhoog(deel.zetels - was),
        std::cmp::Ordering::Less => Pijl::Omlaag(was - deel.zetels),
        std::cmp::Ordering::Equal => Pijl::Gelijk,
    }
}

/// De rijen van de Uitslaggrafiek: van groot naar klein, bij gelijke Zetels op volgorde, en
/// overig onderaan.
pub fn van_groot_naar_klein(mut delen: Vec<Balkdeel>) -> Vec<Balkdeel> {
    delen.sort_by_key(|d| (d.partij.is_none(), std::cmp::Reverse(d.zetels)));
    delen
}

/// De as van de Uitslaggrafiek bij een grootste balk van `grootste` Zetels: tot waar hij loopt
/// en om de hoeveel Zetels een rasterlijn staat. Er blijft ruimte voor het getal achter de
/// grootste balk.
pub fn as_van(grootste: u32) -> (u32, u32) {
    let schaal = (grootste + 4).div_ceil(5).max(1) * 5;
    (schaal, if schaal <= 50 { 5 } else { 10 })
}

/// De rijen van de Winst-en-verliesgrafiek: van de grootste winst naar het grootste verlies, bij
/// een gelijk verschil op `volgorde`.
pub fn van_winst_naar_verlies(mut regels: Vec<Regel>, volgorde: &[String]) -> Vec<Regel> {
    let plaats = |p: &str| volgorde.iter().position(|v| v == p).unwrap_or(volgorde.len());
    regels.sort_by_key(|r| (std::cmp::Reverse(r.verschil), plaats(&r.partij)));
    regels
}
