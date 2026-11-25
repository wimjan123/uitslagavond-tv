//! De camera's volgen de server (ADR 0002): die stuurt 100 camerastanden per seconde, de
//! Renderer vult de beelden ertussen vloeiend in.
//!
//! De Renderer tekent een paar tikken achter de nieuwste stand, zodat er bijna altijd een stand
//! voor en na het beeld is. Hoe laat het bij de server is, schat hij uit de snelst aangekomen
//! standen; zo tekenen Ether en Reserve op hetzelfde moment dezelfde camera.

use std::collections::VecDeque;
use std::time::Instant;

use bevy::prelude::*;
use gedeeld::Rol;
use gedeeld::camera::{Camerabeeld, Camerapunt, Camerastand, Standpunt, TIKKEN_PER_SECONDE};

use crate::Rolinfo;
use crate::beeld::HoofdCamera;
use crate::volging::Standen;

/// Hoeveel tikken de Renderer achter de geschatte tijd van de server tekent (40 ms).
const VERTRAGING: f64 = 4.0;
/// Hoeveel tikken per binnengekomen stand de schatting van de klok mag zakken: zo past hij zich
/// aan als het netwerk trager wordt, en springt hij terug als een stand snel aankomt.
const KLOK_ZAKT: f64 = 0.002;
/// Hoeveel standen de Renderer bewaart (2 seconden).
const BEWAAR: usize = 200;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Camerabeelden>()
            .add_systems(PostUpdate, zet_camera.before(TransformSystems::Propagate));
    }
}

/// Een camera die het Voorbeeld meetekent, naast de hoofdcamera: die volgt de Ether.
#[derive(Component)]
pub struct EtherCamera;

/// De laatste camerastanden van de server.
#[derive(Resource)]
pub struct Camerabeelden {
    begin: Instant,
    reeks: VecDeque<Camerabeeld>,
    /// Geschat verschil tussen het tiknummer van de server en onze klok in tikken.
    klok: Option<f64>,
}

impl Default for Camerabeelden {
    fn default() -> Self {
        Camerabeelden {
            begin: Instant::now(),
            reeks: VecDeque::new(),
            klok: None,
        }
    }
}

impl Camerabeelden {
    fn tikken(&self, moment: Instant) -> f64 {
        moment.saturating_duration_since(self.begin).as_secs_f64() * f64::from(TIKKEN_PER_SECONDE)
    }

    /// Neemt een stand aan die op `ontvangen` binnenkwam.
    pub fn neem(&mut self, beeld: Camerabeeld, ontvangen: Instant) {
        let verschil = beeld.tik as f64 - self.tikken(ontvangen);
        self.klok = Some(match self.klok {
            Some(k) if beeld.tik + 1 >= self.reeks.back().map_or(0, |b| b.tik) => {
                verschil.max(k - KLOK_ZAKT)
            }
            // de server is opnieuw gestart: opnieuw beginnen
            _ => {
                self.reeks.clear();
                verschil
            }
        });
        if self.reeks.back().is_some_and(|b| b.tik >= beeld.tik) {
            return;
        }
        self.reeks.push_back(beeld);
        while self.reeks.len() > BEWAAR {
            self.reeks.pop_front();
        }
    }

    /// De camera's van de Ether en het Voorbeeld op dit moment, of `None` zonder standen.
    pub fn op(&self, moment: Instant) -> Option<(Camerastand, Camerastand)> {
        let r = self.tikken(moment) + self.klok? - VERTRAGING;
        let na = self.reeks.iter().position(|b| b.tik as f64 > r);
        let (a, b) = match na {
            None => {
                let l = self.reeks.back()?;
                return Some((l.ether.stand, l.voorbeeld.stand));
            }
            Some(0) => {
                let e = self.reeks.front()?;
                return Some((e.ether.stand, e.voorbeeld.stand));
            }
            Some(i) => (&self.reeks[i - 1], &self.reeks[i]),
        };
        let f = ((r - a.tik as f64) / (b.tik - a.tik) as f64).clamp(0.0, 1.0) as f32;
        Some((
            tussen(&a.ether, &b.ether, f),
            tussen(&a.voorbeeld, &b.voorbeeld, f),
        ))
    }
}

/// Een stand tussen twee standen in. Over een knip heen vult hij niets in.
fn tussen(a: &Camerapunt, b: &Camerapunt, f: f32) -> Camerastand {
    if a.snede != b.snede {
        return if f < 1.0 { a.stand } else { b.stand };
    }
    let (a, b) = (a.stand, b.stand);
    let hoek = |x: f32, y: f32| {
        use std::f32::consts::{PI, TAU};
        x + ((y - x + PI).rem_euclid(TAU) - PI) * f
    };
    Camerastand {
        positie: [0, 1, 2].map(|i| a.positie[i] + (b.positie[i] - a.positie[i]) * f),
        draai: hoek(a.draai, b.draai),
        helling: hoek(a.helling, b.helling),
        beeldhoek: a.beeldhoek + (b.beeldhoek - a.beeldhoek) * f,
    }
}

/// Plek en lens van een Bevy-camera bij een camerastand.
pub fn transform(stand: &Camerastand) -> Transform {
    Transform::from_translation(Vec3::from(stand.positie)).with_rotation(Quat::from_euler(
        EulerRot::YXZ,
        stand.draai,
        stand.helling,
        0.0,
    ))
}

pub fn projectie(stand: &Camerastand) -> Projection {
    Projection::Perspective(PerspectiveProjection {
        fov: stand.beeldhoek,
        near: 0.05,
        far: 200.0,
        ..default()
    })
}

/// Waar de camera staat zolang er nog geen stand van de server is: het gekozen Standpunt (of het
/// begin van de gekozen Camerarit) uit de Stand, of anders Totaal vanaf voorzitter.
pub fn zonder_server(standen: &Standen, rol: Rol) -> Camerastand {
    let Some(s) = standen.huidig.as_ref().map(|s| &s.stand) else {
        return Standpunt::vaste()[0].stand;
    };
    let letter = match (&s.klaar, rol) {
        (Some(k), Rol::Voorbeeld) => k.standpunt(),
        _ => std::borrow::Cow::Borrowed(s.camera.standpunt.as_str()),
    };
    s.camerastand_van(&letter)
        .unwrap_or(Standpunt::vaste()[0].stand)
}

fn zet(stand: &Camerastand, t: &mut Transform, p: &mut Projection) {
    let nieuw = transform(stand);
    if *t != nieuw {
        *t = nieuw;
    }
    let nieuw = projectie(stand);
    if !matches!((&*p, &nieuw), (Projection::Perspective(a), Projection::Perspective(b)) if a.fov == b.fov)
    {
        *p = nieuw;
    }
}

fn zet_camera(
    beelden: Res<Camerabeelden>,
    standen: Res<Standen>,
    rolinfo: Res<Rolinfo>,
    mut hoofd: Query<(&mut Transform, &mut Projection), (With<HoofdCamera>, Without<EtherCamera>)>,
    mut ether: Query<(&mut Transform, &mut Projection), (With<EtherCamera>, Without<HoofdCamera>)>,
) {
    let (e, v) = beelden.op(Instant::now()).unwrap_or_else(|| {
        (
            zonder_server(&standen, Rol::Ether),
            zonder_server(&standen, Rol::Voorbeeld),
        )
    });
    let eigen = if rolinfo.rol == Rol::Voorbeeld { v } else { e };
    for (mut t, mut p) in &mut hoofd {
        zet(&eigen, &mut t, &mut p);
    }
    for (mut t, mut p) in &mut ether {
        zet(&e, &mut t, &mut p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn punt(x: f32, snede: u32) -> Camerapunt {
        Camerapunt {
            stand: Camerastand {
                positie: [x, 2.0, 0.0],
                draai: 0.0,
                helling: 0.0,
                beeldhoek: 1.0,
            },
            snede,
        }
    }

    fn beeld(tik: u64, x: f32, snede: u32) -> Camerabeeld {
        Camerabeeld {
            tik,
            ether: punt(x, snede),
            voorbeeld: punt(-x, 0),
            opname: None,
            voorbeeld_rit: None,
        }
    }

    #[test]
    fn tussen_twee_tikken_vult_de_renderer_vloeiend_in() {
        let mut c = Camerabeelden::default();
        let t0 = c.begin;
        let ms = |n: u64| t0 + Duration::from_millis(n);
        // tik n komt binnen op n * 10 ms; de camera schuift 1 m per tik
        for n in 0..10u64 {
            c.neem(beeld(n, n as f32, 0), ms(n * 10));
        }
        // op 95 ms tekent hij 4 tikken eerder: tik 5,5
        let (e, v) = c.op(ms(95)).unwrap();
        assert!((e.positie[0] - 5.5).abs() < 1e-3, "{:?}", e.positie);
        assert!((v.positie[0] + 5.5).abs() < 1e-3);
        // voorbij de laatste stand houdt hij die vast
        let (e, _) = c.op(ms(500)).unwrap();
        assert_eq!(e.positie[0], 9.0);
    }

    #[test]
    fn een_laat_aangekomen_stand_verschuift_de_klok_niet() {
        let mut c = Camerabeelden::default();
        let t0 = c.begin;
        let ms = |n: u64| t0 + Duration::from_millis(n);
        c.neem(beeld(0, 0.0, 0), ms(0));
        c.neem(beeld(1, 1.0, 0), ms(10));
        c.neem(beeld(2, 2.0, 0), ms(60)); // 40 ms te laat
        c.neem(beeld(3, 3.0, 0), ms(61));
        let (e, _) = c.op(ms(60)).unwrap();
        assert!((e.positie[0] - 2.0).abs() < 0.01, "{:?}", e.positie);
    }

    #[test]
    fn over_een_knip_heen_vult_de_renderer_niets_in() {
        let mut c = Camerabeelden::default();
        let t0 = c.begin;
        let ms = |n: u64| t0 + Duration::from_millis(n);
        c.neem(beeld(0, 0.0, 0), ms(0));
        c.neem(beeld(1, 0.0, 0), ms(10));
        c.neem(beeld(2, 8.0, 1), ms(20));
        c.neem(beeld(3, 8.0, 1), ms(30));
        // tik 1,5: nog de oude plek, niet halverwege
        let (e, _) = c.op(ms(55)).unwrap();
        assert_eq!(e.positie[0], 0.0);
        let (e, _) = c.op(ms(62)).unwrap();
        assert_eq!(e.positie[0], 8.0);
    }

    #[test]
    fn zonder_server_houdt_de_camera_stil_en_na_een_herstart_staat_hij_meteen_op_de_nieuwe_plek() {
        let mut c = Camerabeelden::default();
        let t0 = c.begin;
        let ms = |n: u64| t0 + Duration::from_millis(n);
        // midden in een rit: tik 5000 en verder, de camera schuift 1 m per tik
        for n in 0..10u64 {
            c.neem(beeld(5000 + n, n as f32, 0), ms(n * 10));
        }
        // de server valt weg: het laatste beeld blijft staan
        let (e, _) = c.op(ms(3000)).unwrap();
        assert_eq!(e.positie[0], 9.0);

        // de server komt terug en telt opnieuw vanaf tik 0, met de camera ergens anders
        c.neem(beeld(0, 40.0, 0), ms(4000));
        for t in [4000, 4005, 4030] {
            let (e, _) = c.op(ms(t)).unwrap();
            assert_eq!(
                e.positie[0], 40.0,
                "op {t} ms: geen overgang vanaf de oude plek"
            );
        }
        c.neem(beeld(1, 41.0, 0), ms(4010));
        let (e, _) = c.op(ms(4100)).unwrap();
        assert_eq!(e.positie[0], 41.0);
    }

    #[test]
    fn de_camera_kijkt_zoals_de_server_rekent() {
        // draai 0: langs -z; positief: naar links (-x); helling positief: omhoog
        let vooruit = |d: f32, h: f32| {
            let s = Camerastand {
                positie: [0.0; 3],
                draai: d,
                helling: h,
                beeldhoek: 1.0,
            };
            (transform(&s).forward().as_vec3(), Vec3::from(s.vooruit()))
        };
        for (d, h) in [(0.0, 0.0), (1.0, 0.0), (-0.7, 0.3), (2.5, -1.2)] {
            let (bevy, server) = vooruit(d, h);
            assert!(
                (bevy - server).length() < 1e-5,
                "{d} {h}: {bevy} tegen {server}"
            );
        }
        let (bevy, _) = vooruit(std::f32::consts::FRAC_PI_2, 0.0);
        assert!((bevy - Vec3::NEG_X).length() < 1e-5);
    }
}
