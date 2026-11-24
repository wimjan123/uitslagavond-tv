//! Haalt de Kamer uit de open data, bij het starten en daarna steeds na een vaste tijd, en
//! bewaart de portretfoto's in de map van de server.
//!
//! - `fotos/open-data/<id>.jpg`: de foto uit de open data. De server haalt hem opnieuw als hij
//!   ontbreekt of als de open data een andere grootte noemt dan bij het vorige ophalen
//!   (`fotos/open-data/lengte.json`).
//! - `fotos/eigen/<id>.jpg` of `.png`: een eigen foto van de omroep (via `PUT /api/foto/<id>`).
//!   Die gaat voor.
//! - `fotos/personen/<id>.jpg` of `.png`: de foto van een eigen persoon van de Persoonskaart.
//!
//! Lukt het ophalen niet, dan blijft de Kamer zoals hij was; de Stand staat op schijf, dus
//! na een herstart is hij er meteen. Verandert er niets, dan stuurt de server geen Opdracht.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use futures_util::StreamExt;
use gedeeld::kamer::{Foto, Kamerlid};
use gedeeld::{Afzender, Opdracht};
use jiff::civil::Date;

use crate::opendata::{self, OpgehaaldeKamer};
use crate::{Gedeeld, OpenData};

/// Zoveel foto's tegelijk ophalen.
const TEGELIJK: usize = 4;
/// Na een mislukte poging eerder opnieuw proberen dan de vaste tijd.
const OPNIEUW: Duration = Duration::from_secs(60);

pub(crate) fn volg(g: Arc<Gedeeld>, inst: OpenData, map: PathBuf) {
    tokio::spawn(async move {
        let afzender = Afzender::OpenData {
            bron: inst.basis.clone(),
        };
        let mut gestopt = g.gestopt.clone();
        let client = match reqwest::Client::builder()
            .user_agent(concat!("uitslagavond-tv/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("open data: geen HTTP-client: {e}");
                return;
            }
        };
        loop {
            let peildatum = inst
                .peildatum
                .unwrap_or_else(|| jiff::ZonedDateTime::now().date());
            let wacht = tokio::select! {
                r = ronde(&g, &afzender, &client, &inst.basis, peildatum, &map) => match r {
                    Ok(()) => inst.elke,
                    Err(e) => {
                        tracing::warn!("open data niet opgehaald: {e:#}");
                        OPNIEUW.min(inst.elke)
                    }
                },
                _ = gestopt.changed() => return,
            };
            tokio::select! {
                _ = tokio::time::sleep(wacht) => {}
                _ = gestopt.changed() => return,
            }
        }
    });
}

async fn ronde(
    g: &Gedeeld,
    afzender: &Afzender,
    client: &reqwest::Client,
    basis: &str,
    peildatum: Date,
    map: &Path,
) -> anyhow::Result<()> {
    let mut k = haal_kamer_op(client, basis, peildatum).await?;
    anyhow::ensure!(
        !k.kamerleden.is_empty(),
        "de open data noemt geen enkel Kamerlid"
    );
    haal_fotos_op(client, basis, map, &k).await;
    for l in &mut k.kamerleden {
        l.foto = foto_op_schijf(map, &l.id);
    }

    let (k_nu, peil_nu) = {
        let s = g.uitzending.borrow();
        (s.stand.kamer.clone(), s.stand.kamer.peildatum.clone())
    };
    let peildatum = peildatum.to_string();
    let zelfde = peil_nu.as_deref() == Some(peildatum.as_str())
        && zelfde_kamerleden(&k_nu.kamerleden, &k.kamerleden)
        && k_nu.fracties.len() == k.fracties.len()
        && k.fracties.iter().all(|f| k_nu.fractie(&f.id) == Some(f));
    if zelfde {
        tracing::debug!("open data: Kamer ongewijzigd");
        return Ok(());
    }
    let opdracht = Opdracht::LeesKamer {
        opgehaald: jiff::ZonedDateTime::now()
            .strftime("%Y-%m-%dT%H:%M:%S%:z")
            .to_string(),
        peildatum,
        kamerleden: k.kamerleden,
        fracties: k.fracties,
    };
    g.voer_uit(afzender, &opdracht)
        .await
        .map_err(|e| anyhow::anyhow!("Kamer niet ingelezen: {e}"))?;
    tracing::info!("open data: Kamer ingelezen");
    Ok(())
}

fn zelfde_kamerleden(a: &[Kamerlid], b: &[Kamerlid]) -> bool {
    a.len() == b.len() && b.iter().all(|k| a.iter().any(|o| o == k))
}

async fn haal_kamer_op(
    client: &reqwest::Client,
    basis: &str,
    peildatum: Date,
) -> anyhow::Result<OpgehaaldeKamer> {
    let (adres, parameters) = opendata::vraag(basis, peildatum);
    let mut verzoek = client.get(adres).query(&parameters);
    let mut personen = Vec::new();
    // Hoogstens 20 pagina's van 250: ruim genoeg voor 150 Kamerleden.
    for _ in 0..20 {
        let tekst = verzoek
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .context("open data niet bereikbaar")?
            .error_for_status()?
            .text()
            .await?;
        let pagina =
            opendata::lees_pagina(&tekst).context("antwoord van de open data onleesbaar")?;
        personen.extend(pagina.personen);
        match pagina.volgende {
            Some(volgende) => verzoek = client.get(volgende),
            None => return Ok(opendata::kamer_op(&personen, peildatum)),
        }
    }
    anyhow::bail!("de open data geeft meer dan 20 pagina's")
}

/// Bij elke foto de grootte die de open data noemde toen de server hem ophaalde. Die grootte
/// klopt niet met het bestand zelf, maar verandert wel als de Kamer een nieuwe foto plaatst.
const LENGTES: &str = "lengte.json";

/// Haalt de foto's die ontbreken of waarvan de open data een andere grootte noemt dan bij het
/// vorige ophalen. Een foto die mislukt, staat in het logboek van de server en wordt de volgende
/// ronde opnieuw geprobeerd.
async fn haal_fotos_op(client: &reqwest::Client, basis: &str, map: &Path, k: &OpgehaaldeKamer) {
    let dir = map.join("fotos").join("open-data");
    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        tracing::error!("map {} niet te maken: {e}", dir.display());
        return;
    }
    let index = dir.join(LENGTES);
    let mut lengtes: BTreeMap<String, u64> = tokio::fs::read(&index)
        .await
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let voor = lengtes.clone();
    let mut nodig = Vec::new();
    for (id, &grootte) in &k.fotogrootte {
        let pad = dir.join(format!("{id}.jpg"));
        match (pad.is_file(), lengtes.get(id)) {
            (true, Some(&eerder)) if eerder == grootte => {}
            // Een foto van vóór de index: houden en de grootte onthouden.
            (true, None) => {
                lengtes.insert(id.clone(), grootte);
            }
            _ => nodig.push((id.clone(), grootte, pad)),
        }
    }
    if !nodig.is_empty() {
        tracing::info!("open data: {} foto's ophalen", nodig.len());
    }
    let gelukt: Vec<(String, u64)> = futures_util::stream::iter(nodig)
        .map(|(id, grootte, pad)| async move {
            match haal_foto_op(client, basis, &id, &pad).await {
                Ok(()) => Some((id, grootte)),
                Err(e) => {
                    tracing::warn!("foto van {id} niet opgehaald: {e:#}");
                    None
                }
            }
        })
        .buffer_unordered(TEGELIJK)
        .filter_map(|r| async move { r })
        .collect()
        .await;
    lengtes.extend(gelukt);
    if lengtes != voor {
        let tekst = serde_json::to_vec_pretty(&lengtes).unwrap_or_default();
        if let Err(e) = schrijf_veilig(&index, &tekst).await {
            tracing::error!("{} niet geschreven: {e:#}", index.display());
        }
    }
}

async fn haal_foto_op(
    client: &reqwest::Client,
    basis: &str,
    id: &str,
    pad: &Path,
) -> anyhow::Result<()> {
    let r = client
        .get(opendata::foto(basis, id))
        .timeout(Duration::from_secs(120))
        .send()
        .await?;
    if r.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(());
    }
    let bytes = r.error_for_status()?.bytes().await?;
    anyhow::ensure!(soort_beeld(&bytes).is_some(), "geen JPEG of PNG");
    schrijf_veilig(pad, &bytes).await
}

/// Eerst naar een los bestand, dan hernoemen: een half geschreven foto staat er nooit.
pub(crate) async fn schrijf_veilig(pad: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let tijdelijk = pad.with_extension("nieuw");
    tokio::fs::write(&tijdelijk, bytes).await?;
    tokio::fs::rename(&tijdelijk, pad).await?;
    Ok(())
}

/// "jpg" of "png" aan de eerste bytes, of `None` als het geen van beide is.
pub(crate) fn soort_beeld(b: &[u8]) -> Option<&'static str> {
    if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else {
        None
    }
}

/// Alleen een id zoals in de open data (letters, cijfers en streepjes), zodat het nooit een pad wordt.
pub(crate) fn geldig_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

pub(crate) fn eigen_foto(map: &Path, id: &str) -> Option<PathBuf> {
    portret(map, "eigen", id)
}

/// Een foto in `fotos/<soort>/<id>.jpg` of `.png`: `open-data` en `eigen` voor Kamerleden,
/// `personen` voor eigen personen van de Persoonskaart.
pub(crate) fn portret(map: &Path, soort: &str, id: &str) -> Option<PathBuf> {
    if !matches!(soort, "open-data" | "eigen" | "personen") || !geldig_id(id) {
        return None;
    }
    ["jpg", "png"]
        .iter()
        .map(|ext| map.join("fotos").join(soort).join(format!("{id}.{ext}")))
        .find(|p| p.is_file())
}

pub(crate) fn foto_uit_open_data(map: &Path, id: &str) -> Option<PathBuf> {
    Some(
        map.join("fotos")
            .join("open-data")
            .join(format!("{id}.jpg")),
    )
    .filter(|p| p.is_file())
}

/// De foto die de server voor een Kamerlid heeft: eerst een eigen foto, dan die uit de open data.
pub(crate) fn foto_op_schijf(map: &Path, id: &str) -> Foto {
    if eigen_foto(map, id).is_some() {
        Foto::Eigen
    } else if foto_uit_open_data(map, id).is_some() {
        Foto::OpenData
    } else {
        Foto::Geen
    }
}
