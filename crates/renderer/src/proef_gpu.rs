//! Proeven met de echte Zaal op een GPU. Ze staan uit in `cargo test`, want CI heeft geen GPU
//! en geen Zaal (die maakt `zaal/maak.sh`). Draaien:
//!
//!     UITSLAGAVOND_ZAAL=<map> [UITSLAGAVOND_BEWIJS=<map>] \
//!         cargo test -p renderer --release -- --ignored --test-threads 1
//!
//! Met `UITSLAGAVOND_BEWIJS` slaat de proef de beelden op als PNG.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bevy::camera::primitives::Aabb;
use bevy::camera::{CameraProjection, RenderTarget};
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::PollType;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use gedeeld::{GenummerdeStand, Kleur, Rol, Stand};

use crate::beeld::{Doel, HoofdCamera};
use crate::omgeving::Reflecties;
use crate::volging::Standen;
use crate::zaal::{Modelcamera, StoelLeer, ZaalKlaar};
use crate::{BREEDTE, HOOGTE, Instelling, maak_app};

/// Beelden na het laden voordat de proef kijkt: texturen en schaduw staan dan zeker goed.
const OPWARMEN: u32 = 60;

fn zaalmap() -> PathBuf {
    std::env::var_os("UITSLAGAVOND_ZAAL")
        .map(PathBuf::from)
        .expect("zet UITSLAGAVOND_ZAAL op de map van zaal/maak.sh")
}

fn stand(volgnummer: u64, kleur: &str) -> GenummerdeStand {
    let kleur: Kleur = kleur.parse().unwrap();
    GenummerdeStand {
        volgnummer,
        stand: Stand {
            stoelkleuren: vec![kleur; 150],
            ..Default::default()
        },
    }
}

/// Camera voor deze proef: plek, doel (assen van Bevy) en verticale beeldhoek. Vanaf de
/// tribune, net onder de rand, met een wijde lens: zo staan alle 150 Stoelen in beeld en
/// boven de balk met tekst.
fn overzicht() -> (Vec3, Vec3, f32) {
    (
        Vec3::new(0.0, 4.85, -13.0),
        Vec3::new(0.0, -7.0, -2.0),
        95f32.to_radians(),
    )
}

/// Een beeld bijwerken en wachten tot de GPU klaar is.
pub(crate) fn stap(app: &mut App) {
    app.update();
    app.world()
        .resource::<RenderDevice>()
        .wgpu_device()
        .poll(PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .unwrap();
}

/// Vraagt een opname van het beeld dat in de volgende `stap` getekend wordt.
pub(crate) fn neem(app: &mut App, opslag: &Arc<Mutex<Option<Image>>>) {
    let RenderTarget::Image(doel) = app.world().resource::<Doel>().0.clone() else {
        unreachable!()
    };
    let opslag = opslag.clone();
    app.world_mut()
        .spawn(Screenshot::image(doel.handle.clone()))
        .observe(move |v: On<ScreenshotCaptured>| {
            *opslag.lock().unwrap() = Some(v.image.clone());
        });
}

/// Is deze pixel duidelijk magenta (veel rood en blauw, weinig groen)?
fn magenta(p: &[u8]) -> bool {
    let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
    r > g + 25 && b > g + 25
}

#[test]
#[ignore = "heeft een GPU en de Zaal uit zaal/maak.sh nodig"]
fn alle_150_stoelen_hebben_na_de_kleurgolf_de_nieuwe_kleur() {
    let mut app = maak_app(&Instelling {
        rol: Rol::Ether,
        zonder_venster: true,
        info: false,
        zaal: Some(zaalmap()),
        kopie: std::env::temp_dir().join("uitslagavond-proef-kopie"),
        bron: None,
    });
    // 50 beelden per seconde, zoals de uitzending, ook als de GPU sneller is
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(20),
    ));
    // VOORBEELDGEGEVENS: eerst alle Stoelen groen
    app.world_mut().resource_mut::<Standen>().huidig = Some(stand(1, "#00ae41"));
    app.finish();
    app.cleanup();
    stap(&mut app);
    // Vanaf de hoofdcamera staan de buitenste Stoelen buiten beeld; deze proef kijkt daarom
    // van achter de tribune, waar alle 150 Stoelen in beeld zijn.
    let (plek, doel, fov) = overzicht();
    {
        let world = app.world_mut();
        let (mut t, mut p) = world
            .query_filtered::<(&mut Transform, &mut Projection), With<HoofdCamera>>()
            .single_mut(world)
            .unwrap();
        *t = Transform::from_translation(plek).looking_at(doel, Vec3::Y);
        if let Projection::Perspective(p) = p.as_mut() {
            p.fov = fov;
        }
    }
    let mut na_laden = 0;
    for _ in 0..3000 {
        stap(&mut app);
        if app.world().resource::<ZaalKlaar>().0 {
            na_laden += 1;
            if na_laden == OPWARMEN {
                break;
            }
        }
    }
    assert_eq!(na_laden, OPWARMEN, "de Zaal laadt niet");

    // beeld N: nog groen; dan zegt de Stand magenta, en na de kleurgolf van een seconde
    // (55 beelden) is elke Stoel magenta
    let voor = Arc::new(Mutex::new(None));
    let na = Arc::new(Mutex::new(None));
    neem(&mut app, &voor);
    stap(&mut app);
    {
        let mut standen = app.world_mut().resource_mut::<Standen>();
        let vorige = standen.huidig.replace(stand(2, "#e000e0"));
        standen.vorige = vorige;
    }
    for _ in 0..55 {
        stap(&mut app);
    }
    neem(&mut app, &na);
    stap(&mut app);
    for _ in 0..10 {
        if voor.lock().unwrap().is_some() && na.lock().unwrap().is_some() {
            break;
        }
        stap(&mut app);
    }
    let voor = voor
        .lock()
        .unwrap()
        .take()
        .expect("geen opname van beeld N")
        .try_into_dynamic()
        .unwrap()
        .to_rgb8();
    let na = na
        .lock()
        .unwrap()
        .take()
        .expect("geen opname na de kleurgolf")
        .try_into_dynamic()
        .unwrap()
        .to_rgb8();
    if let Some(map) = std::env::var_os("UITSLAGAVOND_BEWIJS").map(PathBuf::from) {
        std::fs::create_dir_all(&map).unwrap();
        voor.save(map.join("stoelen-voor.png")).unwrap();
        na.save(map.join("stoelen-na.png")).unwrap();
    }

    // Per Stoel: de plek van het leer in beeld, en daaromheen een vakje van 15 bij 15 pixels.
    let world = app.world_mut();
    let (projectie, cam_plek) = world
        .query_filtered::<(&Projection, &GlobalTransform), With<HoofdCamera>>()
        .single(world)
        .unwrap();
    let Projection::Perspective(projectie) = projectie.clone() else {
        panic!("de hoofdcamera kijkt niet in perspectief")
    };
    let naar_beeld = PerspectiveProjection {
        aspect_ratio: BREEDTE as f32 / HOOGTE as f32,
        ..projectie
    }
    .get_clip_from_view()
        * cam_plek.to_matrix().inverse();
    let mut per_stoel = vec![Vec::new(); 150];
    for (leer, aabb, plek) in world
        .query::<(&StoelLeer, &Aabb, &GlobalTransform)>()
        .iter(world)
    {
        per_stoel[leer.index].push(plek.transform_point(Vec3::from(aabb.center)));
    }
    let mut niet_gewisseld = Vec::new();
    for (i, punten) in per_stoel.iter().enumerate() {
        assert!(!punten.is_empty(), "Stoel {i} heeft geen leer");
        let midden = punten.iter().copied().sum::<Vec3>() / punten.len() as f32;
        let clip = naar_beeld * midden.extend(1.0);
        let ndc = clip.truncate() / clip.w;
        let cx = ((ndc.x + 1.0) / 2.0 * BREEDTE as f32).round() as i32;
        let cy = ((1.0 - ndc.y) / 2.0 * HOOGTE as f32).round() as i32;
        let mut nu = 0;
        let mut eerder = 0;
        for y in cy - 7..=cy + 7 {
            for x in cx - 7..=cx + 7 {
                if x < 0 || y < 0 || x >= voor.width() as i32 || y >= voor.height() as i32 {
                    continue;
                }
                nu += magenta(&na.get_pixel(x as u32, y as u32).0) as u32;
                eerder += magenta(&voor.get_pixel(x as u32, y as u32).0) as u32;
            }
        }
        if nu < 3 || eerder > 0 {
            niet_gewisseld.push((i, format!("({cx}, {cy}): {eerder} magenta voor, {nu} na")));
        }
    }
    assert!(
        niet_gewisseld.is_empty(),
        "{} Stoelen tonen de nieuwe kleur niet na de kleurgolf: {niet_gewisseld:?}",
        niet_gewisseld.len()
    );
}

/// Drie Standpunten uit het model voor de vergelijking met Cycles (#65): Totaal vanaf de
/// Voorzitter (`zaal-3`), de linkerflank (`overzicht-zaal`) en van bovenaf (`dassenman-zaal`).
/// De zes vaste Standpunten komen pas in #43; deze drie komen er het dichtst bij.
pub const VERGELIJK: [&str; 3] = ["zaal-3", "overzicht-zaal", "dassenman-zaal"];

/// Tekent de drie Standpunten met de eigen kleur van de Stoelen en zonder tekst, zoals Cycles
/// (`zaal/maak.sh <map> --alleen-omgeving --beelden zaal-3,overzicht-zaal,dassenman-zaal`).
/// Met een omgevingsbeeld in de Zaal heten de beelden `bevy-na-<naam>.png`, zonder `bevy-voor-`.
#[test]
#[ignore = "heeft een GPU en de Zaal uit zaal/maak.sh nodig"]
fn drie_standpunten_naast_cycles() {
    let map = zaalmap();
    let met_reflecties = map.join("omgeving.bin").exists();
    let standpunten: Vec<Modelcamera> =
        serde_json::from_str(&std::fs::read_to_string(map.join("standpunten.json")).unwrap())
            .unwrap();
    let mut app = maak_app(&Instelling {
        rol: Rol::Ether,
        zonder_venster: true,
        info: false,
        zaal: Some(map),
        kopie: std::env::temp_dir().join("uitslagavond-proef-kopie"),
        bron: None,
    });
    app.world_mut().resource_mut::<Standen>().huidig = Some(GenummerdeStand {
        volgnummer: 1,
        stand: Stand::default(),
    });
    app.finish();
    app.cleanup();
    let mut na_laden = 0;
    for _ in 0..3000 {
        stap(&mut app);
        if app.world().resource::<ZaalKlaar>().0 {
            na_laden += 1;
            if na_laden == OPWARMEN {
                break;
            }
        }
    }
    assert_eq!(na_laden, OPWARMEN, "de Zaal laadt niet");
    assert!(
        app.world().resource::<Reflecties>().klaar,
        "de reflecties zijn niet klaar"
    );
    let probes = app
        .world_mut()
        .query::<&EnvironmentMapLight>()
        .iter(app.world())
        .count();
    assert_eq!(probes, usize::from(met_reflecties));

    let bewijs = std::env::var_os("UITSLAGAVOND_BEWIJS").map(PathBuf::from);
    for naam in VERGELIJK {
        let s = standpunten
            .iter()
            .find(|s| s.naam == naam)
            .unwrap_or_else(|| panic!("Standpunt {naam} ontbreekt"));
        {
            let world = app.world_mut();
            let (mut t, mut p) = world
                .query_filtered::<(&mut Transform, &mut Projection), With<HoofdCamera>>()
                .single_mut(world)
                .unwrap();
            *t = s.transform();
            if let Projection::Perspective(p) = p.as_mut() {
                p.fov = s.fov();
            }
        }
        // schaduw en belichting even laten bijkomen op de nieuwe plek
        for _ in 0..10 {
            stap(&mut app);
        }
        let beeld = Arc::new(Mutex::new(None));
        neem(&mut app, &beeld);
        for _ in 0..10 {
            stap(&mut app);
            if beeld.lock().unwrap().is_some() {
                break;
            }
        }
        let beeld = beeld
            .lock()
            .unwrap()
            .take()
            .expect("geen opname")
            .try_into_dynamic()
            .unwrap()
            .to_rgb8();
        let gemiddeld = beeld
            .pixels()
            .map(|p| p.0.iter().map(|c| *c as u64).sum::<u64>())
            .sum::<u64>() as f64
            / (3 * beeld.width() * beeld.height()) as f64;
        assert!(
            gemiddeld > 20.0,
            "{naam}: beeld bijna zwart ({gemiddeld:.1})"
        );
        if let Some(map) = &bewijs {
            std::fs::create_dir_all(map).unwrap();
            let soort = if met_reflecties { "na" } else { "voor" };
            beeld
                .save(map.join(format!("bevy-{soort}-{naam}.png")))
                .unwrap();
        }
    }
}
