//! Zonder venster: tekenen in een beeld in het geheugen, en dat beeld opslaan als PNG.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};

use crate::beeld::Doel;
use crate::volging::Standen;
use crate::zaal::ZaalKlaar;

/// Hoe lang de Renderer op een Stand wacht voordat hij opgeeft.
const GEDULD: Duration = Duration::from_secs(60);
/// Beelden tussen de Stand en de opname, zodat letters en balk zeker getekend zijn.
const OPWARMEN: u32 = 10;

/// Een beeld in het geheugen als doel voor de camera.
pub fn maak_doel(app: &mut App, (breedte, hoogte): (u32, u32)) -> Doel {
    let mut beeld = Image::new_target_texture(breedte, hoogte, TextureFormat::Rgba8UnormSrgb, None);
    beeld.asset_usage = RenderAssetUsages::RENDER_WORLD;
    let handle = app.world_mut().resource_mut::<Assets<Image>>().add(beeld);
    Doel(RenderTarget::from(handle))
}

pub struct OpnamePlugin {
    pub pad: PathBuf,
    pub volgnummer: u64,
}

impl Plugin for OpnamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Opname {
            pad: self.pad.clone(),
            volgnummer: self.volgnummer,
            opgewarmd: 0,
            genomen: false,
            begin: Instant::now(),
        })
        .add_systems(Last, neem_opname);
    }
}

#[derive(Resource)]
struct Opname {
    pad: PathBuf,
    volgnummer: u64,
    opgewarmd: u32,
    genomen: bool,
    begin: Instant,
}

fn neem_opname(
    mut commands: Commands,
    mut opname: ResMut<Opname>,
    standen: Res<Standen>,
    zaal: Res<ZaalKlaar>,
    doel: Res<Doel>,
    mut exit: MessageWriter<AppExit>,
) {
    if opname.genomen {
        return;
    }
    let klaar = standen
        .huidig
        .as_ref()
        .is_some_and(|s| s.volgnummer >= opname.volgnummer)
        && zaal.0;
    if !klaar {
        if opname.begin.elapsed() > GEDULD {
            error!(
                "geen Stand met volgnummer {} of geen Zaal binnen {GEDULD:?}",
                opname.volgnummer
            );
            exit.write(AppExit::error());
        }
        return;
    }
    opname.opgewarmd += 1;
    if opname.opgewarmd < OPWARMEN {
        return;
    }
    opname.genomen = true;
    let RenderTarget::Image(beeld) = &doel.0 else {
        unreachable!("zonder venster is het doel een beeld")
    };
    let pad = opname.pad.clone();
    commands
        .spawn(Screenshot::image(beeld.handle.clone()))
        .observe(
            move |vangst: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                let opgeslagen = vangst
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|e| e.to_string())
                    .and_then(|b| b.to_rgb8().save(&pad).map_err(|e| e.to_string()));
                match opgeslagen {
                    Ok(()) => {
                        info!("beeld opgeslagen: {}", pad.display());
                        exit.write(AppExit::Success);
                    }
                    Err(e) => {
                        error!("beeld niet opgeslagen: {e}");
                        exit.write(AppExit::error());
                    }
                }
            },
        );
}

/// Een reeks beelden voor bewijs, zoals een camera-overgang: vanaf de Stand met `volgnummer`
/// elke `elke` een PNG, `seconden` lang, dan stoppen. De beelden heten `000.png`, `001.png`, ...
pub struct ReeksPlugin {
    pub map: PathBuf,
    pub volgnummer: u64,
    pub elke: Duration,
    pub seconden: Duration,
}

impl Plugin for ReeksPlugin {
    fn build(&self, app: &mut App) {
        std::fs::create_dir_all(&self.map)
            .unwrap_or_else(|e| panic!("{}: {e}", self.map.display()));
        app.insert_resource(Reeks {
            map: self.map.clone(),
            volgnummer: self.volgnummer,
            elke: self.elke,
            seconden: self.seconden,
            begin: None,
            genomen: 0,
            schrijvers: Vec::new(),
            gestart: Instant::now(),
        })
        .add_systems(Last, neem_reeks);
    }
}

#[derive(Resource)]
struct Reeks {
    map: PathBuf,
    volgnummer: u64,
    elke: Duration,
    seconden: Duration,
    begin: Option<Instant>,
    genomen: u32,
    /// PNG schrijven duurt lang: dat gebeurt naast het tekenen.
    schrijvers: Vec<std::thread::JoinHandle<()>>,
    gestart: Instant,
}

fn neem_reeks(
    mut commands: Commands,
    mut reeks: ResMut<Reeks>,
    standen: Res<Standen>,
    zaal: Res<ZaalKlaar>,
    doel: Res<Doel>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(begin) = reeks.begin else {
        let klaar = standen
            .huidig
            .as_ref()
            .is_some_and(|s| s.volgnummer >= reeks.volgnummer)
            && zaal.0;
        if klaar {
            reeks.begin = Some(Instant::now());
        } else if reeks.gestart.elapsed() > GEDULD {
            error!(
                "geen Stand met volgnummer {} of geen Zaal binnen {GEDULD:?}",
                reeks.volgnummer
            );
            exit.write(AppExit::error());
        }
        return;
    };
    if begin.elapsed() > reeks.seconden {
        for s in reeks.schrijvers.drain(..) {
            let _ = s.join();
        }
        info!("{} beelden in {}", reeks.genomen, reeks.map.display());
        exit.write(AppExit::Success);
        return;
    }
    if begin.elapsed() < reeks.elke * reeks.genomen {
        return;
    }
    let RenderTarget::Image(beeld) = &doel.0 else {
        unreachable!("zonder venster is het doel een beeld")
    };
    let pad = reeks.map.join(format!("{:03}.png", reeks.genomen));
    reeks.genomen += 1;
    commands
        .spawn(Screenshot::image(beeld.handle.clone()))
        .observe(
            move |vangst: On<ScreenshotCaptured>, mut reeks: ResMut<Reeks>| {
                let beeld = vangst.image.clone();
                let pad = pad.clone();
                reeks
                    .schrijvers
                    .push(std::thread::spawn(move || match beeld.try_into_dynamic() {
                        Ok(b) => {
                            if let Err(e) = b.to_rgb8().save(&pad) {
                                error!("{}: {e}", pad.display());
                            }
                        }
                        Err(e) => error!("{}: {e}", pad.display()),
                    }));
            },
        );
}
