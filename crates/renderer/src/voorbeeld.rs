//! De rol Voorbeeld: draait op de regie-pc zonder venster en stuurt kleine beelden naar het
//! dashboard. Hij tekent twee beelden: de klaargezette Graphic (het Voorbeeld) en de Graphic
//! in de Ether, zoals de Ether hem tekent. Een beeld gaat alleen weg als het veranderd is.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::tasks::AsyncComputeTaskPool;
use bevy::time::common_conditions::on_timer;
use gedeeld::Monitor;

use crate::beeld::{Doel, zaalcamera};
use crate::camera::EtherCamera;
use crate::graphic::Bron;
use crate::opname::maak_doel;
use crate::volging::Zender;

/// Grootte van de beelden voor het dashboard: scherp 720p (ADR 0005).
pub const KLEIN: (u32, u32) = (1280, 720);
/// Hoe vaak het Voorbeeld een beeld vangt: 25 keer per seconde. Een stilstaand beeld gaat
/// dankzij de vingerafdruk niet opnieuw weg.
const ELKE: Duration = Duration::from_millis(40);
/// Zoveel vangsten per Monitor mogen tegelijk onderweg zijn.
const ONDERWEG: u8 = 4;
/// Kwaliteit van de JPEG (1 tot 100).
const KWALITEIT: u8 = 84;

pub struct VoorbeeldPlugin;

impl Plugin for VoorbeeldPlugin {
    fn build(&self, app: &mut App) {
        // De Graphics zijn ontworpen voor 1080p; zo passen ze precies in het kleine beeld.
        app.insert_resource(UiScale(KLEIN.0 as f32 / crate::BREEDTE as f32));
        let ether = maak_doel(app, KLEIN);
        app.insert_resource(EtherDoel(ether))
            .init_resource::<Vangsten>()
            .add_systems(Startup, maak_ether_camera)
            .add_systems(Update, vang.run_if(on_timer(ELKE)));
    }
}

#[derive(Resource)]
struct EtherDoel(Doel);

/// Per Monitor: hoeveel vangsten onderweg zijn, het nummer van de laatste vangst, en wat er
/// verstuurd is. JPEG maken gebeurt naast het tekenen, dus dat deel is gedeeld.
#[derive(Resource, Default)]
struct Vangsten {
    onderweg: [u8; 2],
    nummer: [u64; 2],
    verstuurd: [Arc<Mutex<Verstuurd>>; 2],
}

/// Het laatst verstuurde beeld van één Monitor.
#[derive(Default)]
struct Verstuurd {
    nummer: u64,
    vingerafdruk: Option<u64>,
}

impl Verstuurd {
    /// Of een vangst weg moet: niet als er al een nieuwere weg is (de vangsten maken hun JPEG
    /// naast elkaar, dus een oudere kan later klaar zijn), en niet als het beeld gelijk bleef.
    fn neem(&mut self, nummer: u64, vingerafdruk: u64) -> bool {
        if nummer <= self.nummer {
            return false;
        }
        self.nummer = nummer;
        if self.vingerafdruk == Some(vingerafdruk) {
            return false;
        }
        self.vingerafdruk = Some(vingerafdruk);
        true
    }
}

fn maak_ether_camera(mut commands: Commands, doel: Res<EtherDoel>) {
    commands.spawn((
        EtherCamera,
        zaalcamera(),
        Camera { order: 1, ..default() },
        doel.0.0.clone(),
        Bron::Ether,
    ));
}


fn vang(
    mut commands: Commands,
    voorbeeld: Res<Doel>,
    ether: Res<EtherDoel>,
    mut vangsten: ResMut<Vangsten>,
) {
    for (i, (monitor, doel)) in [(Monitor::Voorbeeld, &voorbeeld.0), (Monitor::Ether, &ether.0.0)]
        .into_iter()
        .enumerate()
    {
        if vangsten.onderweg[i] >= ONDERWEG {
            continue;
        }
        let RenderTarget::Image(beeld) = doel else {
            unreachable!("het Voorbeeld tekent altijd in een beeld")
        };
        vangsten.onderweg[i] += 1;
        vangsten.nummer[i] += 1;
        let nummer = vangsten.nummer[i];
        let verstuurd = vangsten.verstuurd[i].clone();
        commands
            .spawn(Screenshot::image(beeld.handle.clone()))
            .observe(
                move |vangst: On<ScreenshotCaptured>,
                      zender: Res<Zender>,
                      mut vangsten: ResMut<Vangsten>| {
                    vangsten.onderweg[i] -= 1;
                    let beeld = vangst.image.clone();
                    let zender = zender.0.clone();
                    let verstuurd = verstuurd.clone();
                    AsyncComputeTaskPool::get()
                        .spawn(async move {
                            maak_jpeg(beeld, nummer, &verstuurd, |jpeg| {
                                zender.stuur(monitor, jpeg)
                            })
                        })
                        .detach();
                },
            );
    }
}

/// Maakt van een vangst een JPEG en stuurt hem, als hij weg moet ([`Verstuurd::neem`]).
fn maak_jpeg(beeld: Image, nummer: u64, verstuurd: &Mutex<Verstuurd>, stuur: impl FnOnce(Vec<u8>)) {
    let rgb = match beeld.try_into_dynamic() {
        Ok(b) => b.to_rgb8(),
        Err(e) => {
            error!("klein beeld niet te lezen: {e}");
            return;
        }
    };
    let mut h = DefaultHasher::new();
    rgb.as_raw().hash(&mut h);
    // Het slot blijft vast tot het beeld weg is, zodat de beelden op volgorde gaan.
    let mut verstuurd = verstuurd.lock().expect("verstuurd");
    if !verstuurd.neem(nummer, h.finish()) {
        return;
    }
    let mut jpeg = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, KWALITEIT);
    if let Err(e) = rgb.write_with_encoder(encoder) {
        error!("klein beeld niet te maken: {e}");
        return;
    }
    stuur(jpeg);
}

#[cfg(test)]
mod tests {
    use super::Verstuurd;

    #[test]
    fn alleen_een_nieuwer_en_ander_beeld_gaat_weg() {
        let mut v = Verstuurd::default();
        assert!(v.neem(1, 10), "het eerste beeld");
        assert!(!v.neem(2, 10), "gelijk gebleven");
        assert!(v.neem(4, 20), "veranderd");
        assert!(!v.neem(3, 30), "ouder dan wat al weg is");
        assert!(v.neem(5, 10), "weer veranderd");
    }
}
