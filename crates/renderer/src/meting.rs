//! De tijd per beeld meten, zonder venster: zo snel als het kan, en na elk beeld wachten
//! tot de GPU klaar is. Zo is de gemeten tijd de echte tijd die één beeld kost.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::render_resource::PollType;
use bevy::render::renderer::RenderDevice;
use serde::Serialize;

use crate::zaal::ZaalKlaar;

/// Beelden na het laden van de Zaal die niet meetellen (texturen uploaden, shaders bouwen).
const OPWARMEN: u32 = 120;

#[derive(Serialize, Debug)]
pub struct Rapport {
    pub beelden: usize,
    pub seconden: f64,
    pub gemiddeld_ms: f64,
    pub p50_ms: f64,
    pub p99_ms: f64,
    pub p999_ms: f64,
    pub max_ms: f64,
    pub boven_20ms: usize,
}

/// Percentielen van de tijden in milliseconden (dichtstbijzijnde rang).
pub fn rapport(tijden_ms: &[f64]) -> Rapport {
    let mut s = tijden_ms.to_vec();
    s.sort_by(f64::total_cmp);
    let p = |q: f64| s[((s.len() as f64 * q).ceil() as usize).clamp(1, s.len()) - 1];
    let som: f64 = s.iter().sum();
    Rapport {
        beelden: s.len(),
        seconden: som / 1000.0,
        gemiddeld_ms: som / s.len() as f64,
        p50_ms: p(0.5),
        p99_ms: p(0.99),
        p999_ms: p(0.999),
        max_ms: s[s.len() - 1],
        boven_20ms: s.iter().filter(|t| **t > 20.0).count(),
    }
}

/// Een runner voor de app: beelden achter elkaar tot `duur` gemeten is, dan het rapport.
pub fn runner(duur: Duration, uitslag: Option<PathBuf>) -> impl FnOnce(App) -> AppExit {
    move |mut app: App| {
        app.finish();
        app.cleanup();
        let mut tijden = Vec::new();
        let mut opgewarmd = 0;
        let mut begin: Option<Instant> = None;
        let mut vorige = Instant::now();
        loop {
            app.update();
            app.world()
                .resource::<RenderDevice>()
                .wgpu_device()
                .poll(PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .expect("GPU");
            let nu = Instant::now();
            if let Some(exit) = app.should_exit() {
                return exit;
            }
            if begin.is_some() {
                tijden.push((nu - vorige).as_secs_f64() * 1000.0);
            } else if app.world().resource::<ZaalKlaar>().0 {
                opgewarmd += 1;
                if opgewarmd == OPWARMEN {
                    info!("meting gestart: {duur:?}");
                    begin = Some(nu);
                }
            }
            vorige = nu;
            if begin.is_some_and(|b| nu - b >= duur) {
                break;
            }
        }
        let r = rapport(&tijden);
        let json = serde_json::to_string_pretty(&r).expect("json");
        println!("{json}");
        if let Some(pad) = uitslag {
            std::fs::write(&pad, &json).unwrap_or_else(|e| panic!("{}: {e}", pad.display()));
        }
        if r.p99_ms < 20.0 {
            AppExit::Success
        } else {
            AppExit::error()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentielen_van_een_bekende_reeks() {
        // 1000 beelden van 1 tot 1000 ms / 100: p99 is het 990e beeld, 9,90 ms
        let t: Vec<f64> = (1..=1000).map(|i| i as f64 / 100.0).collect();
        let r = rapport(&t);
        assert_eq!(r.beelden, 1000);
        assert_eq!(r.p50_ms, 5.0);
        assert_eq!(r.p99_ms, 9.9);
        assert_eq!(r.p999_ms, 9.99);
        assert_eq!(r.max_ms, 10.0);
        assert_eq!(r.boven_20ms, 0);
    }

    #[test]
    fn een_enkele_uitschieter_telt_mee_boven_20_ms() {
        let mut t = vec![10.0; 199];
        t.push(31.0);
        let r = rapport(&t);
        assert_eq!(r.p99_ms, 10.0);
        assert_eq!(r.max_ms, 31.0);
        assert_eq!(r.boven_20ms, 1);
    }
}
