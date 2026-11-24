//! Let op een bestand dat de Regie bijhoudt (het invoerbestand of de Zaalindeling) en leest het
//! in zodra het verandert.
//!
//! De server kijkt elke 200 ms of de inhoud anders is. Een nieuwe inhoud leest hij pas in als
//! die bij de volgende keer kijken nog gelijk is, zodat een half geschreven bestand niet als
//! onleesbaar binnenkomt.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gedeeld::cijfers::Invoer;
use gedeeld::zaalindeling::Zaalindeling;
use gedeeld::{Afzender, Opdracht};

use crate::Gedeeld;

const KIJKEN: Duration = Duration::from_millis(200);

/// Het invoerbestand met partijlijst en sets.
pub(crate) fn volg(g: Arc<Gedeeld>, pad: PathBuf) {
    let naam = pad.display().to_string();
    let afzender = Afzender::Invoerbestand { pad: naam.clone() };
    volg_bestand(g, pad, afzender, move |inhoud| {
        Ok(Opdracht::LeesInvoer {
            bestand: naam.clone(),
            ontvangen: jiff::ZonedDateTime::now().strftime("%H.%M.%S").to_string(),
            invoer: Invoer::uit_json(inhoud),
        })
    });
}

/// Het bestand met de Zaalindeling. Een onleesbaar bestand laat de Zaalindeling zoals hij was.
pub(crate) fn volg_zaalindeling(g: Arc<Gedeeld>, pad: PathBuf) {
    let naam = pad.display().to_string();
    let afzender = Afzender::Zaalindeling { pad: naam.clone() };
    volg_bestand(g, pad, afzender, move |inhoud| {
        Ok(Opdracht::LeesZaalindeling {
            bestand: naam.clone(),
            zaalindeling: Zaalindeling::uit_json(inhoud)?,
        })
    });
}

fn volg_bestand(
    g: Arc<Gedeeld>,
    pad: PathBuf,
    afzender: Afzender,
    lees: impl Fn(&str) -> Result<Opdracht, String> + Send + 'static,
) {
    tokio::spawn(async move {
        let naam = pad.display().to_string();
        let mut gestopt = g.gestopt.clone();
        let mut verwerkt: Option<Vec<u8>> = None;
        let mut kandidaat: Option<Vec<u8>> = None;
        let mut ontbrak = false;
        loop {
            match tokio::fs::read(&pad).await {
                Ok(inhoud) => {
                    ontbrak = false;
                    if verwerkt.as_ref() != Some(&inhoud) {
                        if kandidaat.as_ref() == Some(&inhoud) {
                            let uitkomst = match lees(&String::from_utf8_lossy(&inhoud)) {
                                Ok(opdracht) => g.voer_uit(&afzender, &opdracht).await.map(|_| ()),
                                Err(e) => Err(e),
                            };
                            if let Err(e) = uitkomst {
                                tracing::error!("{naam} niet ingelezen: {e}");
                            }
                            verwerkt = Some(inhoud);
                            kandidaat = None;
                        } else {
                            kandidaat = Some(inhoud);
                        }
                    }
                }
                Err(e) if !ontbrak => {
                    tracing::warn!("{naam} niet te lezen: {e}");
                    ontbrak = true;
                }
                Err(_) => {}
            }
            tokio::select! {
                _ = tokio::time::sleep(KIJKEN) => {}
                _ = gestopt.changed() => return,
            }
        }
    });
}
