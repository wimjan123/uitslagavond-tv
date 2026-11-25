//! De Persoonskaart (#54, besluit #25): een kaart aan de zijkant met één persoon, een Kamerlid of
//! iemand die de Regie zelf toevoegt (een eigen persoon).
//!
//! Altijd naam, foto en Fractie (logo en kleur); de functie alleen bij een fractievoorzitter; één
//! vrije regel die leeg mag blijven; en met een vinkje leeftijd, woonplaats, geboorteplaats en de
//! eerste dag in de Kamer. Bij klaarzetten vult de server de gegevens van de persoon in
//! ([`Persoonsgegevens`]); daarna houdt de Graphic die vast, zoals elke Graphic.
//!
//! Heeft een Kamerlid een Stoel in de Zaalindeling, dan licht die Stoel op en gaat de camera naar
//! het shot van die Stoel ([`crate::camera::shot_op_stoel`]). Zonder Stoel licht niets op en gaat
//! de camera naar het Standpunt van het Sjabloon.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::kamer::{Foto, Kamerlid};
use crate::{Stand, Stoel, Weigering};

/// Langste naam van een eigen persoon, in tekens.
pub const MAX_NAAM: usize = 60;
/// Langste vrije regel op de Persoonskaart, in tekens.
pub const MAX_VRIJE_REGEL: usize = 80;
/// Het Standpunt van het Sjabloon: waar de camera heen gaat bij een persoon zonder Stoel.
pub const STANDPUNT_ZONDER_STOEL: &str = "S";

/// Wie er op de kaart staat.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "soort")]
#[ts(export)]
pub enum Wie {
    /// Een Kamerlid van de Kamer die zit, met `Persoon.Id` uit de open data.
    Kamerlid { id: String },
    /// Een persoon die de Regie zelf toevoegde ([`EigenPersoon`]).
    Eigen { id: String },
}

/// Welke extra gegevens op de kaart staan. In het invulformulier heeft elk een vinkje; welke
/// standaard aan staan, staat in [`Stand::persoonskaart_extra`].
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Extra {
    pub leeftijd: bool,
    pub woonplaats: bool,
    pub geboorteplaats: bool,
    /// De eerste dag in de Kamer.
    pub in_de_kamer_sinds: bool,
}

impl Default for Extra {
    /// Leeftijd en woonplaats, zoals in de vormgeving.
    fn default() -> Self {
        Extra {
            leeftijd: true,
            woonplaats: true,
            geboorteplaats: false,
            in_de_kamer_sinds: false,
        }
    }
}

/// De gegevens van de persoon zoals ze waren bij klaarzetten. De server vult ze in.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Persoonsgegevens {
    pub naam: String,
    /// Op het vlak als er geen foto is, zoals "DG".
    pub voorletters: String,
    /// Bestandsnaam van de foto bij de server (`GET /api/bestand/<naam>`), of geen foto.
    #[serde(default)]
    #[ts(optional)]
    pub foto: Option<String>,
    /// Sleutel van de Partij: logo, kleur en korte naam.
    #[serde(default)]
    #[ts(optional)]
    pub partij: Option<String>,
    /// Alleen bij een fractievoorzitter.
    #[serde(default)]
    #[ts(optional)]
    pub functie: Option<String>,
    /// De Stoel in de Zaalindeling; een eigen persoon heeft er nooit een.
    #[serde(default)]
    #[ts(optional)]
    pub stoel: Option<Stoel>,
    #[serde(default)]
    #[ts(optional)]
    pub leeftijd: Option<u16>,
    #[serde(default)]
    #[ts(optional)]
    pub woonplaats: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub geboorteplaats: Option<String>,
    /// `JJJJ-MM-DD`.
    #[serde(default)]
    #[ts(optional)]
    pub in_de_kamer_sinds: Option<String>,
}

/// Iemand die de Regie zelf toevoegt, zoals een lijsttrekker die nog geen Kamerlid is. Bewaard bij
/// het programma (in de Stand), dus ook na een herstart en in een Preset.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct EigenPersoon {
    /// Leeg bij een nieuwe persoon: de server geeft een id, zoals "p12".
    #[serde(default)]
    pub id: String,
    pub naam: String,
    /// Of de server een foto heeft (`PUT /api/foto/<id>`).
    #[serde(default)]
    pub foto: bool,
    /// Sleutel van de Partij.
    #[serde(default)]
    #[ts(optional)]
    pub partij: Option<String>,
    /// De vrije regel die het formulier voorstelt.
    #[serde(default)]
    pub regel: String,
    /// `JJJJ-MM-DD`, voor de leeftijd.
    #[serde(default)]
    #[ts(optional)]
    pub geboortedatum: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub woonplaats: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub geboorteplaats: Option<String>,
}

/// De bestandsnaam van de foto van een Kamerlid bij de server, of geen foto.
pub fn foto_van_kamerlid(k: &Kamerlid) -> Option<String> {
    match k.foto {
        Foto::Geen => None,
        Foto::OpenData => Some(format!("fotos/open-data/{}", k.id)),
        Foto::Eigen => Some(format!("fotos/eigen/{}", k.id)),
    }
}

/// De bestandsnaam van de foto van een eigen persoon bij de server.
pub fn foto_van_persoon(id: &str) -> String {
    format!("fotos/personen/{id}")
}

/// De voorletters van een naam: de eerste letter van het eerste en het laatste woord met een
/// hoofdletter, zonder tussenvoegsel ("Sanne de Vries" wordt "SV").
pub fn voorletters(naam: &str) -> String {
    let woorden: Vec<&str> = naam.split_whitespace().collect();
    let eerste = |w: &&str| w.chars().find(|c| c.is_alphabetic());
    let voor = woorden.first().and_then(eerste);
    let achter = woorden
        .iter()
        .skip(1)
        .rev()
        .find(|w| w.chars().next().is_some_and(char::is_uppercase))
        .or(woorden.get(1..).and_then(<[&str]>::last))
        .and_then(eerste);
    voor.into_iter()
        .chain(achter)
        .flat_map(char::to_uppercase)
        .collect()
}

/// De dag (jaar, maand, dag) in UTC op `ms` milliseconden sinds 1970.
pub fn dag_van(ms: u64) -> (i64, u32, u32) {
    // Howard Hinnant, "civil_from_days"
    let z = (ms / 86_400_000) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn lees_datum(t: &str) -> Option<(i64, u32, u32)> {
    let mut d = t.get(..10)?.split('-');
    let datum = (
        d.next()?.parse().ok()?,
        d.next()?.parse().ok()?,
        d.next()?.parse().ok()?,
    );
    (1..=12).contains(&datum.1).then_some(datum)
}

/// De leeftijd op `vandaag` bij een geboortedatum (`JJJJ-MM-DD`).
pub fn leeftijd(geboortedatum: &str, vandaag: (i64, u32, u32)) -> Option<u16> {
    let (j, m, d) = lees_datum(geboortedatum)?;
    let jaren = vandaag.0 - j - i64::from((vandaag.1, vandaag.2) < (m, d));
    u16::try_from(jaren).ok()
}

impl Persoonsgegevens {
    /// De regel met de extra gegevens die aan staan en bekend zijn, zoals "42 jaar · Utrecht". De
    /// eerste dag in de Kamer staat er als jaar, zodat de regel op één regel past.
    pub fn regel(&self, extra: &Extra) -> String {
        let mut delen = Vec::new();
        if extra.leeftijd
            && let Some(l) = self.leeftijd
        {
            delen.push(format!("{l} jaar"));
        }
        if extra.woonplaats
            && let Some(w) = &self.woonplaats
        {
            delen.push(w.clone());
        }
        if extra.geboorteplaats
            && let Some(g) = &self.geboorteplaats
        {
            delen.push(format!("geboren in {g}"));
        }
        if extra.in_de_kamer_sinds
            && let Some((jaar, _, _)) = self.in_de_kamer_sinds.as_deref().and_then(lees_datum)
        {
            delen.push(format!("in de Kamer sinds {jaar}"));
        }
        delen.join(" · ")
    }
}

fn tekst(t: &Option<String>) -> Option<String> {
    t.as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
}

impl Stand {
    /// De gegevens van een persoon zoals ze nu zijn, op de dag van `tijd_ms`.
    pub fn persoonsgegevens(&self, wie: &Wie, tijd_ms: u64) -> Result<Persoonsgegevens, Weigering> {
        let vandaag = dag_van(tijd_ms);
        match wie {
            Wie::Kamerlid { id } => {
                let k = self
                    .kamer
                    .kamerlid(id)
                    .ok_or_else(|| Weigering::OnbekendePersoon(id.clone()))?;
                Ok(Persoonsgegevens {
                    naam: k.naam(),
                    voorletters: k.voorletters(),
                    foto: foto_van_kamerlid(k),
                    partij: self.kamer.partij_van(k).map(str::to_owned),
                    functie: k.fractievoorzitter.then(|| "Fractievoorzitter".to_owned()),
                    stoel: self.zaalindeling.stoel_van(id),
                    leeftijd: k
                        .geboortedatum
                        .as_deref()
                        .and_then(|g| leeftijd(g, vandaag)),
                    woonplaats: tekst(&k.woonplaats),
                    geboorteplaats: tekst(&k.geboorteplaats),
                    in_de_kamer_sinds: k.in_de_kamer_sinds.clone(),
                })
            }
            Wie::Eigen { id } => {
                let p = self
                    .persoon(id)
                    .ok_or_else(|| Weigering::OnbekendePersoon(id.clone()))?;
                Ok(Persoonsgegevens {
                    naam: p.naam.trim().to_owned(),
                    voorletters: voorletters(&p.naam),
                    foto: p.foto.then(|| foto_van_persoon(&p.id)),
                    partij: p.partij.clone(),
                    functie: None,
                    stoel: None,
                    leeftijd: p
                        .geboortedatum
                        .as_deref()
                        .and_then(|g| leeftijd(g, vandaag)),
                    woonplaats: tekst(&p.woonplaats),
                    geboorteplaats: tekst(&p.geboorteplaats),
                    in_de_kamer_sinds: None,
                })
            }
        }
    }

    /// De eigen persoon met dit id.
    pub fn persoon(&self, id: &str) -> Option<&EigenPersoon> {
        self.personen.iter().find(|p| p.id == id)
    }

    /// Bewaart een nieuwe eigen persoon (zonder id) of wijzigt er een. Een nieuwe krijgt id
    /// "p" met het volgnummer.
    pub(crate) fn bewaar_persoon(
        &mut self,
        persoon: &EigenPersoon,
        volgnummer: u64,
    ) -> Result<(), Weigering> {
        let naam = persoon.naam.trim();
        if naam.is_empty() {
            return Err(Weigering::NaamLeeg);
        }
        if naam.chars().count() > MAX_NAAM {
            return Err(Weigering::NaamTeLang);
        }
        if persoon.regel.chars().count() > MAX_VRIJE_REGEL {
            return Err(Weigering::RegelTeLang);
        }
        if let Some(p) = &persoon.partij
            && self.cijfers.partij(p).is_none()
        {
            return Err(Weigering::Cijfers(
                crate::cijfers::CijferWeigering::OnbekendePartij(p.clone()),
            ));
        }
        let mut nieuw = persoon.clone();
        nieuw.naam = naam.to_owned();
        if nieuw.id.is_empty() {
            nieuw.id = format!("p{volgnummer}");
            nieuw.foto = false;
            self.personen.push(nieuw);
        } else {
            let oud = self
                .personen
                .iter_mut()
                .find(|p| p.id == nieuw.id)
                .ok_or_else(|| Weigering::OnbekendePersoon(nieuw.id.clone()))?;
            // de foto zet alleen de server, na het plaatsen
            nieuw.foto = oud.foto;
            *oud = nieuw;
        }
        Ok(())
    }

    pub(crate) fn wis_persoon(&mut self, id: &str) -> Result<(), Weigering> {
        let voor = self.personen.len();
        self.personen.retain(|p| p.id != id);
        if self.personen.len() == voor {
            return Err(Weigering::OnbekendePersoon(id.into()));
        }
        Ok(())
    }

    pub(crate) fn zet_foto_persoon(&mut self, id: &str, foto: bool) -> Result<(), Weigering> {
        let p = self
            .personen
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| Weigering::OnbekendePersoon(id.into()))?;
        p.foto = foto;
        Ok(())
    }
}
