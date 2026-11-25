//! Haalt de Stand binnen via de volger en bewaart de vorige en de huidige Stand.

use std::sync::Mutex;

use bevy::prelude::*;
use gedeeld::GenummerdeStand;
use gedeeld::Rol;
use crate::camera::Camerabeelden;
use gedeeld::volger::{Beeldzender, Gebeurtenis, Volger, volg};

pub struct VolgingPlugin {
    pub server: String,
    pub sleutel: String,
    pub rol: Rol,
}

impl Plugin for VolgingPlugin {
    fn build(&self, app: &mut App) {
        let volger = volg(&self.server, &self.sleutel, self.rol);
        app.insert_resource(Zender(volger.beeldzender()))
            .insert_resource(Volging(Mutex::new(volger)))
            .add_systems(PreUpdate, ontvang);
    }
}

#[derive(Resource)]
struct Volging(Mutex<Volger>);

/// Stuurt kleine beelden terug naar de server (alleen voor de rol Voorbeeld).
#[derive(Resource)]
pub struct Zender(pub Beeldzender);

/// De laatste twee Standen. Uit het verschil kiest de Renderer later zijn animatie;
/// de eerste Stand na het opstarten toont hij meteen als eindbeeld.
#[derive(Resource, Default)]
pub struct Standen {
    pub vorige: Option<GenummerdeStand>,
    pub huidig: Option<GenummerdeStand>,
}

#[derive(Resource, Default, Clone, PartialEq, Eq)]
pub enum Verbinding {
    #[default]
    Zoeken,
    Verbonden,
    Geweigerd(String),
    Verbroken(String),
}

fn ontvang(
    volging: Res<Volging>,
    mut standen: ResMut<Standen>,
    mut verbinding: ResMut<Verbinding>,
    mut camera: ResMut<Camerabeelden>,
) {
    let volger = volging.0.lock().expect("volger");
    while let Ok(g) = volger.gebeurtenissen.try_recv() {
        match g {
            Gebeurtenis::Stand(s) => {
                let vorige = standen.huidig.replace(s);
                standen.vorige = vorige;
            }
            Gebeurtenis::Camera { beeld, ontvangen } => camera.neem(beeld, ontvangen),
            Gebeurtenis::Verbonden => {
                info!("verbonden met de server");
                *verbinding = Verbinding::Verbonden;
            }
            Gebeurtenis::Geweigerd(r) => {
                error!("de server weigert: {r}");
                *verbinding = Verbinding::Geweigerd(r);
            }
            Gebeurtenis::Verbroken(r) => {
                if *verbinding != Verbinding::Verbroken(r.clone()) {
                    warn!("geen verbinding met de server: {r}");
                }
                *verbinding = Verbinding::Verbroken(r);
            }
        }
    }
}
