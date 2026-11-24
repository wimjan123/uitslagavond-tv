//! Wat een Stream Deck met Bitfocus Companion nodig heeft naast `POST /api/opdracht` (zie
//! `docs/stream-deck.md`).
//!
//! - `GET /api/knoppen/<veld>`: één veld van de knopstatus als platte tekst. Companion zet het
//!   antwoord in een variabele en kleurt een knop met een vergelijking op die variabele.
//!   `GET /api/knoppen` geeft alle velden tegelijk als JSON.
//! - `POST /api/vrije-camera/wissel`: zet de Vrije camera om, zoals V op het dashboard. Een
//!   knop van Companion kent de Stand niet en kan dus niet zelf `aan` kiezen.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use gedeeld::{Opdracht, Stand};

use crate::{Gedeeld, antwoord, knop};

/// De knopstatus: per veld een korte tekst, `ja`/`nee` of `aan`/`uit`, leeg als er niets is.
fn status(s: &Stand) -> BTreeMap<&'static str, String> {
    let ja = |b: bool| if b { "ja" } else { "nee" }.to_string();
    let preset = |p: Option<u16>| p.map(|n| n.to_string()).unwrap_or_default();
    let klaar = s.klaar.as_ref();
    BTreeMap::from([
        ("klaar_versie", klaar.map(|k| k.versie.to_string()).unwrap_or_default()),
        ("klaar_door", klaar.map(|k| k.gewijzigd_door.clone()).unwrap_or_default()),
        ("kan_tonen", ja(klaar.is_some() && !s.staat_klaar_in_beeld())),
        ("in_beeld", ja(s.ether.is_some())),
        ("kan_bijwerken", ja(s.kan_bijwerken())),
        ("standpunt", s.camera.standpunt.clone()),
        ("vrije_camera", if s.camera.vrij { "aan" } else { "uit" }.into()),
        (
            "preset_klaar",
            preset(klaar.and_then(|k| s.presets.iter().find(|p| p.is_bron_van_klaar(k))).map(|p| p.nummer)),
        ),
        (
            "preset_live",
            preset(s.ether.as_ref().and_then(|e| s.presets.iter().find(|p| p.is_bron_van_ether(e))).map(|p| p.nummer)),
        ),
    ])
}

pub(crate) async fn alle_velden(State(g): State<Arc<Gedeeld>>) -> Json<BTreeMap<&'static str, String>> {
    Json(status(&g.uitzending.borrow().stand))
}

pub(crate) async fn een_veld(State(g): State<Arc<Gedeeld>>, Path(naam): Path<String>) -> Response {
    match status(&g.uitzending.borrow().stand).remove(naam.as_str()) {
        Some(waarde) => ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], waarde).into_response(),
        None => (StatusCode::NOT_FOUND, "geen veld met die naam").into_response(),
    }
}

pub(crate) async fn wissel_vrije_camera(State(g): State<Arc<Gedeeld>>, koppen: HeaderMap) -> Response {
    let aan = !g.uitzending.borrow().stand.camera.vrij;
    antwoord(g.voer_uit(&knop(&koppen), &Opdracht::ZetVrijeCamera { aan }).await)
}
