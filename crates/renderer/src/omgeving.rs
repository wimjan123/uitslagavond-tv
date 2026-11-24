//! Reflecties voor metaal en glas (#65): het omgevingsbeeld uit `omgeving.bin`.
//!
//! Cycles neemt de zaal op als kubus vanuit het midden van de doos om alles wat vast staat
//! (`omgeving` in `zaal/export.py`). De Renderer zet die kubus in een reflectieprobe met die doos
//! als grens, zodat een spiegeling op de vloer het plafond op de goede plek toont. Bevy filtert de
//! kubus op de GPU voor ruwe en gladde vlakken; daarna zet de Renderer het filteren stil, want de
//! zaal verandert niet. De probe geeft alleen glans: licht en schaduw blijven uit de bak, het
//! daglicht en het zachte licht van alle kanten.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::light::{GeneratedEnvironmentMapLight, LightProbe, ParallaxCorrection};
use bevy::pbr::generate::{
    GeneratorBindGroups, GeneratorPipelines, RenderEnvironmentMap, filtering_system,
};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, PipelineCache, TextureDimension, TextureFormat, TextureViewDescriptor,
    TextureViewDimension,
};
use bevy::render::{Render, RenderApp};
use serde::Deserialize;

use crate::lichtkaart;

pub struct OmgevingPlugin;

impl Plugin for OmgevingPlugin {
    fn build(&self, app: &mut App) {
        let gefilterd = Gefilterd::default();
        let zwart = app.world_mut().resource_mut::<Assets<Image>>().add(zwart());
        app.insert_resource(gefilterd.clone())
            .insert_resource(ZwarteKubus(zwart))
            .insert_resource(Reflecties { klaar: true })
            .add_systems(Update, zet_stil);
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render
                .insert_resource(gefilterd)
                .add_systems(Render, meld_gefilterd.after(filtering_system));
        }
    }
}

/// De reflectieprobe van de Zaal.
#[derive(Component)]
pub struct Reflectieprobe;

/// Of de reflecties klaar zijn: de kubus is gefilterd en het filteren staat stil. Zonder
/// omgevingsbeeld meteen waar.
#[derive(Resource)]
pub struct Reflecties {
    pub klaar: bool,
}

/// Gedeeld door de wereld van de app en die van de GPU: de kubus is minstens één keer gefilterd.
#[derive(Resource, Clone, Default)]
pub struct Gefilterd(Arc<AtomicBool>);

impl Gefilterd {
    pub fn meld(&self) {
        self.0.store(true, Ordering::Release);
    }

    fn gedaan(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Resource)]
struct ZwarteKubus(Handle<Image>);

/// Zet de reflectieprobe neer als de Zaal een omgevingsbeeld heeft. Geeft `false` zonder.
pub fn zet_neer(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    reflecties: &mut Reflecties,
    map: &Path,
) -> bool {
    let (bin, json) = (map.join("omgeving.bin"), map.join("omgeving.json"));
    if !bin.exists() || !json.exists() {
        return false;
    }
    let tekst =
        std::fs::read_to_string(&json).unwrap_or_else(|e| panic!("{}: {e}", json.display()));
    let opname: Opname =
        serde_json::from_str(&tekst).unwrap_or_else(|e| panic!("{}: {e}", json.display()));
    let kubus = lees(&bin).unwrap_or_else(|e| panic!("{e}"));
    let (plek, grens) = opname.probe();
    commands.spawn((
        Reflectieprobe,
        LightProbe::default(),
        plek,
        grens,
        // Cycles en de bak rekenen in dezelfde eenheden, dus sterkte 1.
        GeneratedEnvironmentMapLight {
            environment_map: images.add(kubus),
            intensity: 1.0,
            rotation: Quat::IDENTITY,
            affects_lightmapped_mesh_diffuse: false,
        },
    ));
    reflecties.klaar = false;
    true
}

/// Na het filteren: niet meer elk beeld opnieuw filteren, en alleen de glans houden.
fn zet_stil(
    mut commands: Commands,
    gefilterd: Res<Gefilterd>,
    zwart: Res<ZwarteKubus>,
    mut reflecties: ResMut<Reflecties>,
    mut probes: Query<
        (Entity, &mut EnvironmentMapLight),
        (With<Reflectieprobe>, With<GeneratedEnvironmentMapLight>),
    >,
) {
    if !gefilterd.gedaan() {
        return;
    }
    for (probe, mut kaarten) in &mut probes {
        commands
            .entity(probe)
            .remove::<GeneratedEnvironmentMapLight>();
        kaarten.diffuse_map = zwart.0.clone();
        kaarten.affects_lightmapped_mesh_diffuse = false;
        reflecties.klaar = true;
        info!("reflecties klaar: omgevingsbeeld gefilterd");
    }
}

/// In de wereld van de GPU, na het filteren van Bevy: is er deze keer echt gefilterd?
fn meld_gefilterd(
    gefilterd: Res<Gefilterd>,
    kaarten: Query<(), (With<GeneratorBindGroups>, With<RenderEnvironmentMap>)>,
    pipelines: Option<Res<GeneratorPipelines>>,
    cache: Res<PipelineCache>,
) {
    let Some(p) = pipelines else { return };
    let klaar = [
        p.copy,
        p.downsample_first,
        p.downsample_second,
        p.radiance,
        p.irradiance,
    ]
    .into_iter()
    .all(|id| cache.get_compute_pipeline(id).is_some());
    if klaar && !kaarten.is_empty() {
        gefilterd.meld();
    }
}

/// `omgeving.json`: waar de kubus is opgenomen en de halve maten van de zaal, in de assen van Bevy.
#[derive(Deserialize, Clone, Debug)]
pub struct Opname {
    pub midden: [f32; 3],
    pub half: [f32; 3],
}

/// Zoveel ruimer dan de zaal is het gebied waarin de probe telt, zodat geen vlak er net buiten valt.
const RUIMTE: f32 = 1.05;

impl Opname {
    /// Plek en grens van de reflectieprobe. Bevy leest de kubus in de assen van de probe; met
    /// een gelijke schaal op elke as blijft een richting in de zaal dezelfde richting in de kubus.
    /// De doos van de zaal gaat daarom als grens apart mee, in de maat van de probe.
    pub fn probe(&self) -> (Transform, ParallaxCorrection) {
        let half = Vec3::from(self.half);
        let zijde = 2.0 * half.max_element() * RUIMTE;
        let plek =
            Transform::from_translation(Vec3::from(self.midden)).with_scale(Vec3::splat(zijde));
        (plek, ParallaxCorrection::Custom(half / zijde))
    }
}

/// Leest `omgeving.bin`: de zes vlakken onder elkaar in het formaat van de lichtkaart
/// (breedte N, hoogte 6N, volgorde +X, -X, +Y, -Y, +Z, -Z), als kubustextuur.
pub fn lees(pad: &Path) -> Result<Image, String> {
    let vlakken = lichtkaart::lees(pad);
    let Extent3d { width, height, .. } = vlakken.texture_descriptor.size;
    if height != 6 * width || !width.is_power_of_two() {
        return Err(format!(
            "{}: geen kubus van zes vierkante vlakken met een macht van twee ({width} bij {height})",
            pad.display()
        ));
    }
    let mut kubus = Image::new(
        Extent3d {
            width,
            height: width,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        vlakken.data.unwrap_or_default(),
        TextureFormat::Rgb9e5Ufloat,
        RenderAssetUsages::RENDER_WORLD,
    );
    kubus.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..Default::default()
    });
    kubus.sampler = ImageSampler::linear();
    Ok(kubus)
}

/// Een zwarte kubus: zo geeft de probe geen licht, alleen glans.
pub fn zwart() -> Image {
    let mut kubus = Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        vec![0; 6 * 8],
        TextureFormat::Rgba16Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    kubus.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..Default::default()
    });
    kubus
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Schrijft een bestand zoals `schrijf_lichtkaart` in `zaal/export.py`, met pixel i = i.
    fn schrijf(pad: &Path, breedte: u32, hoogte: u32) {
        let mut data = b"LK95".to_vec();
        data.extend(breedte.to_le_bytes());
        data.extend(hoogte.to_le_bytes());
        for i in 0..breedte * hoogte {
            data.extend(i.to_le_bytes());
        }
        std::fs::write(pad, data).unwrap();
    }

    /// Een probe zoals `zet_neer` hem neerzet, met de kaarten die Bevy erbij maakt.
    fn app_met_probe() -> (App, Entity, Handle<Image>, Handle<Image>) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .add_plugins(OmgevingPlugin);
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let bron = images.add(zwart());
        let licht = images.add(zwart());
        let glans = images.add(zwart());
        let probe = app
            .world_mut()
            .spawn((
                Reflectieprobe,
                GeneratedEnvironmentMapLight {
                    environment_map: bron,
                    intensity: 1.0,
                    ..default()
                },
                EnvironmentMapLight {
                    diffuse_map: licht.clone(),
                    specular_map: glans.clone(),
                    intensity: 1.0,
                    ..default()
                },
            ))
            .id();
        app.world_mut().resource_mut::<Reflecties>().klaar = false;
        (app, probe, licht, glans)
    }

    #[test]
    fn na_het_filteren_staat_de_probe_stil_en_geeft_alleen_glans() {
        let (mut app, probe, licht, glans) = app_met_probe();
        app.update();
        app.update();
        assert!(!app.world().resource::<Reflecties>().klaar);
        assert!(
            app.world()
                .get::<GeneratedEnvironmentMapLight>(probe)
                .is_some()
        );

        // de GPU meldt dat de kubus gefilterd is
        app.world().resource::<Gefilterd>().meld();
        app.update();
        assert!(app.world().resource::<Reflecties>().klaar);
        assert!(
            app.world()
                .get::<GeneratedEnvironmentMapLight>(probe)
                .is_none(),
            "het filteren moet stoppen"
        );
        let kaarten = app.world().get::<EnvironmentMapLight>(probe).unwrap();
        assert_eq!(kaarten.specular_map, glans, "de gefilterde glans blijft");
        assert_ne!(kaarten.diffuse_map, licht, "geen licht van de probe");
        assert!(!kaarten.affects_lightmapped_mesh_diffuse);
        let images = app.world().resource::<Assets<Image>>();
        let diffuus = images.get(&kaarten.diffuse_map).unwrap();
        assert!(diffuus.data.as_ref().unwrap().iter().all(|b| *b == 0));
    }

    #[test]
    fn zes_vlakken_onder_elkaar_worden_een_kubus() {
        let map = tempfile::tempdir().unwrap();
        let pad = map.path().join("omgeving.bin");
        schrijf(&pad, 4, 24);
        let kubus = lees(&pad).unwrap();
        let maat = kubus.texture_descriptor.size;
        assert_eq!(
            (maat.width, maat.height, maat.depth_or_array_layers),
            (4, 4, 6)
        );
        assert_eq!(
            kubus.texture_view_descriptor.unwrap().dimension,
            Some(TextureViewDimension::Cube)
        );
        // vlak 2 (+Y) begint bij pixel 2 * 16, linksboven
        let data = kubus.data.unwrap();
        let pixel = |i: usize| u32::from_le_bytes(data[4 * i..4 * i + 4].try_into().unwrap());
        assert_eq!(pixel(2 * 16), 32);
        assert_eq!(pixel(6 * 16 - 1), 95);
    }

    #[test]
    fn een_beeld_dat_geen_kubus_is_wordt_geweigerd() {
        let map = tempfile::tempdir().unwrap();
        let pad = map.path().join("omgeving.bin");
        schrijf(&pad, 4, 20);
        assert!(lees(&pad).is_err(), "hoogte is geen zes keer de breedte");
        schrijf(&pad, 6, 36);
        assert!(
            lees(&pad).is_err(),
            "Bevy filtert alleen een macht van twee"
        );
    }

    #[test]
    fn de_probe_omvat_de_zaal_en_de_grens_is_de_doos_van_de_zaal() {
        // de maten van de Zaal uit zaal/maak.sh (omgeving.json)
        let opname = Opname {
            midden: [0.0, 4.2, -6.25],
            half: [20.86, 4.5, 13.75],
        };
        let (plek, grens) = opname.probe();
        let ParallaxCorrection::Custom(grens) = grens else {
            panic!("de grens moet de doos van de zaal zijn")
        };
        let naar_probe = plek.to_matrix().inverse();
        // een hoek van de zaal: binnen de probe, en precies op de grens
        let hoek = naar_probe.transform_point3(Vec3::new(20.86, 8.7, 7.5));
        assert!(
            hoek.abs().max_element() < 0.5,
            "hoek buiten de probe: {hoek}"
        );
        assert!(
            (hoek - grens).abs().max_element() < 1e-4,
            "{hoek} is niet {grens}"
        );
        // een richting in de zaal blijft dezelfde richting in de kubus
        let r = naar_probe.transform_vector3(Vec3::new(1.0, 2.0, 3.0));
        assert!(r.normalize().angle_between(Vec3::new(1.0, 2.0, 3.0)) < 1e-5);
    }
}
