//! Debat Direct van de Tweede Kamer: de uitslag per Fractie, ongeveer twee uur na de Stemming.
//!
//! Debat Direct is de interne API van de app van de Kamer, zonder documentatie of voorwaarden
//! (zie het onderzoek bij #13). Hij kan zonder aankondiging veranderen. Per dag staat er een
//! agenda; een debat van het soort "Stemmingen" heeft `votings` met per Stemming `votingRounds`:
//! de titel (met het nummer van de motie), de uitslag, begin en einde in de Zaal en de stemmen.
//! Bij een ronde van het soort "Fractie" stemt elke partij, bij "Hoofd" (hoofdelijk) elk
//! Kamerlid. De partijen en personen hebben eigen id's; `actors` geeft hun naam.

use gedeeld::kamer::{Fractie, Kamer};
use gedeeld::stemmingen::{
    AfwijkendeStem, Besluit, Bron, GetikteFractie, OfficieleUitslag, OpgehaaldeUitslag, Stem,
    Stemming,
};
use serde::Deserialize;

use super::{gewoon, over_tekst};

/// Het adres van de API.
pub const BASIS: &str = "https://cdn.debatdirect.tweedekamer.nl/api";

pub fn agenda(basis: &str, dag: jiff::civil::Date) -> String {
    format!("{}/agenda/{dag}", basis.trim_end_matches('/'))
}

pub fn debat(basis: &str, dag: jiff::civil::Date, id: &str) -> String {
    format!("{}/agenda/{dag}/debates/{id}", basis.trim_end_matches('/'))
}

pub fn actors(basis: &str) -> String {
    format!("{}/actors", basis.trim_end_matches('/'))
}

#[derive(Deserialize, Debug)]
pub struct Agenda {
    #[serde(default)]
    pub debates: Vec<AgendaDebat>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgendaDebat {
    pub id: String,
    pub debate_type: Option<String>,
}

impl Agenda {
    /// De debatten met Stemmingen.
    pub fn stemmingen(&self) -> impl Iterator<Item = &str> {
        self.debates
            .iter()
            .filter(|d| d.debate_type.as_deref() == Some("Stemmingen"))
            .map(|d| d.id.as_str())
    }
}

#[derive(Deserialize, Debug)]
pub struct Debat {
    #[serde(default)]
    pub votings: Vec<Voting>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Voting {
    #[serde(default)]
    pub voting_rounds: Vec<Ronde>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Ronde {
    #[serde(rename = "type")]
    pub soort: Option<String>,
    #[serde(default)]
    pub title: String,
    pub result: Option<String>,
    /// Zoals "2026-11-19T13:34:52+0100".
    pub ended_at: Option<String>,
    /// Paren van partij- of persoon-id en "Voor" of "Tegen".
    #[serde(default)]
    pub votes: Vec<(String, String)>,
}

#[derive(Deserialize, Debug)]
pub struct Actors {
    #[serde(default)]
    pub parties: Vec<Partij>,
    #[serde(default)]
    pub politicians: Vec<Persoon>,
}

#[derive(Deserialize, Debug)]
pub struct Partij {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Persoon {
    pub id: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    pub party_id: Option<String>,
}

pub fn lees_agenda(json: &str) -> anyhow::Result<Agenda> {
    Ok(serde_json::from_str(json)?)
}

pub fn lees_debat(json: &str) -> anyhow::Result<Debat> {
    Ok(serde_json::from_str(json)?)
}

pub fn lees_actors(json: &str) -> anyhow::Result<Actors> {
    Ok(serde_json::from_str(json)?)
}

/// De uitslagen van de rondes die bij een Stemming op de lijst horen. Een ronde zonder uitslag
/// (de Kamer houdt hem nog tegen, of hij heet "Onbekend") telt niet.
pub fn uitslagen(
    debatten: &[Debat],
    actors: &Actors,
    lijst: &[Stemming],
    kamer: &Kamer,
) -> Vec<OpgehaaldeUitslag> {
    let rondes = debatten
        .iter()
        .flat_map(|d| &d.votings)
        .flat_map(|v| &v.voting_rounds);
    let mut uit: Vec<OpgehaaldeUitslag> = Vec::new();
    for r in rondes {
        let besluit = match r.result.as_deref() {
            Some("Aangenomen") => Besluit::Aangenomen,
            Some("Verworpen") => Besluit::Verworpen,
            _ => continue,
        };
        if r.votes.is_empty() {
            continue;
        }
        let Some(stemming) = bij_stemming(&r.title, lijst) else {
            tracing::debug!(
                "Debat Direct: geen Stemming op de lijst bij \"{}\"",
                r.title
            );
            continue;
        };
        if uit.iter().any(|u| u.stemming == stemming) {
            continue;
        }
        let hoofdelijk = r.soort.as_deref() == Some("Hoofd");
        let (fracties, afwijkend) = if hoofdelijk {
            per_kamerlid(&r.votes, actors, kamer)
        } else {
            (per_fractie(&r.votes, actors, kamer), vec![])
        };
        if fracties.is_empty() {
            continue;
        }
        uit.push(OpgehaaldeUitslag {
            stemming: stemming.to_owned(),
            uitslag: OfficieleUitslag {
                bron: Bron::DebatDirect,
                besluit,
                hoofdelijk,
                fracties,
                afwijkend,
                hamerslag: r.ended_at.as_deref().map(met_dubbele_punt),
            },
        });
    }
    uit
}

/// "+0100" aan het eind wordt "+01:00", zoals elders in de Stand.
fn met_dubbele_punt(t: &str) -> String {
    let n = t.len();
    let zone = t.get(n.saturating_sub(5)..).unwrap_or("");
    if n > 5
        && (zone.starts_with('+') || zone.starts_with('-'))
        && zone[1..].bytes().all(|b| b.is_ascii_digit())
    {
        format!("{}:{}", &t[..n - 2], &t[n - 2..])
    } else {
        t.to_owned()
    }
}

/// De Stemming bij de titel van een ronde, zoals "de motie-Wilders over het opzeggen van het
/// vertrouwen in het kabinet (36848, nr. 113)": op nummer, anders op de tekst na "over", anders
/// (voor een wetsvoorstel, "1.  Wijziging van ...") op de hele titel. Alleen als er precies één past.
fn bij_stemming<'a>(titel: &str, lijst: &'a [Stemming]) -> Option<&'a str> {
    let enige = |past: &dyn Fn(&Stemming) -> bool| {
        let mut p = lijst.iter().filter(|s| past(s));
        match (p.next(), p.next()) {
            (Some(s), None) => Some(s.punt.id.as_str()),
            _ => None,
        }
    };
    let nummer = titel
        .rfind('(')
        .and_then(|i| titel[i + 1..].strip_suffix(')'))
        .map(gewoon)
        .filter(|n| n.contains(" nr "));
    if let Some(n) = &nummer {
        if let Some(s) = enige(&|s| s.punt.nummer.as_deref().map(gewoon).as_ref() == Some(n)) {
            return Some(s);
        }
    }
    if let Some(o) = over_tekst(titel) {
        if let Some(s) = enige(&|s| over_tekst(&s.punt.titel).as_ref() == Some(&o)) {
            return Some(s);
        }
    }
    let zonder_nummer = titel
        .trim_start()
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start_matches('.');
    let t = gewoon(zonder_nummer);
    enige(&|s| gewoon(&s.punt.titel) == t)
}

fn stem(s: &str) -> Option<Stem> {
    match s {
        "Voor" => Some(Stem::Voor),
        "Tegen" => Some(Stem::Tegen),
        _ => None,
    }
}

/// De Fractie van de Kamer bij een partij van Debat Direct. De namen verschillen iets
/// ("Christen Democratisch Appel" tegen "Christen-Democratisch Appèl", "Het Juiste Antwoord JA21"
/// tegen "JA21"), dus: gelijke naam of afkorting, anders de afkorting als los woord in de naam.
fn fractie_bij<'a>(partij: &str, actors: &Actors, kamer: &'a Kamer) -> Option<&'a Fractie> {
    let naam = gewoon(
        &actors
            .parties
            .iter()
            .find(|p| p.id.eq_ignore_ascii_case(partij))?
            .name,
    );
    let gelijk = kamer
        .fracties
        .iter()
        .find(|f| gewoon(&f.naam) == naam || gewoon(&f.afkorting) == naam);
    gelijk.or_else(|| {
        let woorden: Vec<&str> = naam.split(' ').collect();
        let mut p = kamer
            .fracties
            .iter()
            .filter(|f| woorden.contains(&gewoon(&f.afkorting).as_str()));
        match (p.next(), p.next()) {
            (Some(f), None) => Some(f),
            _ => None,
        }
    })
}

fn per_fractie(votes: &[(String, String)], actors: &Actors, kamer: &Kamer) -> Vec<GetikteFractie> {
    votes
        .iter()
        .filter_map(|(partij, s)| {
            let f = fractie_bij(partij, actors, kamer);
            if f.is_none() {
                tracing::warn!("Debat Direct: partij {partij} niet in de Kamer");
            }
            Some(GetikteFractie {
                fractie: f?.id.clone(),
                stem: stem(s)?,
            })
        })
        .collect()
}

/// Bij een hoofdelijke stemming: per Fractie de stem van de meeste Kamerleden, en wie anders stemde.
fn per_kamerlid(
    votes: &[(String, String)],
    actors: &Actors,
    kamer: &Kamer,
) -> (Vec<GetikteFractie>, Vec<AfwijkendeStem>) {
    let stemmen: Vec<(&Persoon, &Fractie, Stem)> = votes
        .iter()
        .filter_map(|(id, s)| {
            let p = actors
                .politicians
                .iter()
                .find(|p| p.id.eq_ignore_ascii_case(id))?;
            let f = fractie_bij(p.party_id.as_deref()?, actors, kamer)?;
            Some((p, f, stem(s)?))
        })
        .collect();
    let mut fracties = Vec::new();
    let mut afwijkend = Vec::new();
    for f in &kamer.fracties {
        let van_f: Vec<_> = stemmen.iter().filter(|(_, g, _)| g.id == f.id).collect();
        let voor = van_f.iter().filter(|(.., s)| *s == Stem::Voor).count();
        if van_f.is_empty() {
            continue;
        }
        let meeste = if voor * 2 >= van_f.len() {
            Stem::Voor
        } else {
            Stem::Tegen
        };
        fracties.push(GetikteFractie {
            fractie: f.id.clone(),
            stem: meeste,
        });
        for (p, _, s) in van_f.iter().filter(|(.., s)| *s != meeste) {
            match kamerlid_bij(p, f, kamer) {
                Some(k) => afwijkend.push(AfwijkendeStem {
                    kamerlid: k,
                    stem: *s,
                }),
                None => tracing::warn!(
                    "Debat Direct: {} {} niet in de Kamer",
                    p.first_name,
                    p.last_name
                ),
            }
        }
    }
    (fracties, afwijkend)
}

/// Het Kamerlid bij een persoon van Debat Direct, binnen zijn Fractie: elk woord van de
/// achternaam (met tussenvoegsel) staat in de naam, en de voornaam past.
fn kamerlid_bij(p: &Persoon, f: &Fractie, kamer: &Kamer) -> Option<String> {
    let naam = gewoon(&format!("{} {}", p.first_name, p.last_name));
    let woorden: Vec<&str> = naam.split(' ').collect();
    let voornaam = gewoon(&p.first_name);
    let mut kandidaten = kamer.kamerleden.iter().filter(|k| {
        k.fractie == f.id
            && gewoon(&k.achternaam)
                .split(' ')
                .all(|w| woorden.contains(&w))
            && (voornaam.is_empty() || gewoon(&k.roepnaam) == voornaam)
    });
    match (kandidaten.next(), kandidaten.next()) {
        (Some(k), None) => Some(k.id.clone()),
        _ => None,
    }
}
