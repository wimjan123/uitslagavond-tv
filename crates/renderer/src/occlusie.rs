//! Occlusie uit de diepte: een stap na het tekenen van de Zaal die hoeken, plooien en de vloer
//! naast bureaus en banken iets donkerder maakt (zie `occlusie.wgsl`). Dezelfde stap legt ook het
//! licht op de vloer ([`Vloerlicht`]): de lichtkring onder een blok Stoelen en de lichtstrook bij 76.
//!
//! Het licht van de Zaal is gebakken, en Bevy's eigen SSAO werkt niet op gebakken licht: die dekt
//! alleen het omgevingslicht af. Deze stap werkt op het hele beeld vóór de toonweergave, dus ook op
//! het gebakken licht, en leest de diepte van de hoofdpas (met MSAA), zonder extra pas over de
//! Zaal. Het patroon ligt vast: geen ruis die bij compressie gaat flikkeren.

use bevy::core_pipeline::{Core3d, Core3dSystems, FullscreenShader, tonemapping::tonemapping};
use bevy::prelude::*;
use bevy::render::{
    Render, RenderApp, RenderStartup, RenderSystems,
    extract_component::{
        ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
        UniformComponentPlugin,
    },
    render_resource::{
        BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
        CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, Operations,
        PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
        Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType,
        SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, TextureSampleType,
        binding_types::{sampler, texture_2d, texture_depth_2d_multisampled, uniform_buffer},
    },
    renderer::{RenderContext, RenderDevice, ViewQuery},
    view::{ExtractedView, ViewDepthStencilTexture, ViewTarget},
};
use bevy::shader::Shader;

/// Occlusie op een camera met MSAA 4 (zoals [`crate::beeld::zaalcamera`]). De near en de
/// beeldhoek volgen de camera elk beeld.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Debug)]
#[extract_app(RenderApp)]
pub struct Occlusie {
    near: f32,
    tan_x: f32,
    tan_y: f32,
    /// Binnen deze afstand in meter dekt iets af.
    pub straal: f32,
    /// 0 geen occlusie, 1 volle occlusie.
    pub sterkte: f32,
    /// Wat dichter bij de camera is dan dit (meter), wordt donkerder, tot [`Self::voorgrond_donker`].
    pub voorgrond: f32,
    /// Hoeveel donkerder de voorgrond wordt (0 niet, 1 zwart).
    pub voorgrond_donker: f32,
    _c: f32,
    /// Omhoog, in de ruimte van de camera.
    omhoog: Vec4,
    /// Het midden van de Zaal op de vloer (de oorsprong), in de ruimte van de camera.
    oorsprong: Vec4,
    /// De twee lichtkringen: het midden in de ruimte van de camera en de straal (w), en het extra
    /// licht per kanaal (0: uit).
    kring_a: Vec4,
    kring_a_licht: Vec4,
    kring_b: Vec4,
    kring_b_licht: Vec4,
    /// De lichtstrook: begin en eind in de ruimte van de camera, met de halve breedte (w van het
    /// begin) en het extra licht.
    strook_van: Vec4,
    strook_naar: Vec4,
    strook_licht: Vec4,
}

impl Occlusie {
    pub fn new(straal: f32, sterkte: f32) -> Self {
        Occlusie {
            near: 0.05,
            tan_x: 1.0,
            tan_y: 1.0,
            straal,
            sterkte,
            voorgrond: 0.0,
            voorgrond_donker: 0.0,
            _c: 0.0,
            omhoog: Vec4::Y,
            oorsprong: Vec4::ZERO,
            kring_a: Vec4::ZERO,
            kring_a_licht: Vec4::ZERO,
            kring_b: Vec4::ZERO,
            kring_b_licht: Vec4::ZERO,
            strook_van: Vec4::ZERO,
            strook_naar: Vec4::ZERO,
            strook_licht: Vec4::ZERO,
        }
    }
}

/// Licht op de vloer, in de assen van de Zaal: een zachte kring of strook die de vloer lichter
/// maakt, zoals een spot van boven. Een echte spot
/// maakte ook de rugleuningen en de randen van de bureaus zo licht dat de partijkleur verbleekte
/// (council-ronde 6); dit licht valt alleen op de vloer (tapijt en treden, niet op bureaus en
/// zittingen) en verandert de kleur van wat er ligt niet.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct Vloerlicht {
    pub kringen: [Option<Lichtkring>; 2],
    pub strook: Option<Lichtstrook>,
}

/// Een lichtkring op de vloer: rond `midden` (horizontaal gemeten) tot `straal` meter, zacht
/// aflopend vanaf ongeveer de helft. `licht` is het extra licht per kanaal: 1 is twee keer zo licht.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lichtkring {
    pub midden: Vec3,
    pub straal: f32,
    pub licht: LinearRgba,
}

/// Een lichtstrook op de vloer van `van` naar `naar`, `breedte` meter breed met een zachte rand,
/// met in het midden een kern van `kern` meter breed die `kern_licht` keer zoveel extra licht
/// krijgt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lichtstrook {
    pub van: Vec3,
    pub naar: Vec3,
    pub breedte: f32,
    pub licht: LinearRgba,
    pub kern: f32,
    pub kern_licht: f32,
}

fn licht4(l: LinearRgba) -> Vec4 {
    Vec4::new(l.red, l.green, l.blue, 0.0)
}

pub struct OcclusiePlugin;

impl Plugin for OcclusiePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<Occlusie>::default(),
            UniformComponentPlugin::<Occlusie>::default(),
        ))
        .init_resource::<Vloerlicht>()
        .add_systems(PostUpdate, volg_de_camera.after(TransformSystems::Propagate));
        let shader = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .add(Shader::from_wgsl(include_str!("occlusie.wgsl"), file!()));
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .insert_resource(OcclusieShader(shader))
            .init_resource::<SpecializedRenderPipelines<OcclusiePijplijn>>()
            .add_systems(RenderStartup, maak_pijplijn)
            .add_systems(Render, kies_pijplijn.in_set(RenderSystems::Prepare))
            .add_systems(
                Core3d,
                occlusie
                    .in_set(Core3dSystems::PostProcess)
                    .before(bevy::post_process::bloom::bloom)
                    .before(tonemapping),
            );
    }
}

/// Near en beeldhoek van de camera naar de stap, en het licht op de vloer in de ruimte van de
/// camera.
fn volg_de_camera(mut q: Query<(&Projection, &GlobalTransform, &mut Occlusie)>, vloerlicht: Res<Vloerlicht>) {
    for (projectie, plek, mut o) in &mut q {
        let Projection::Perspective(p) = projectie else {
            continue;
        };
        let tan_y = (p.fov / 2.0).tan();
        let (near, tan_x) = (p.near, tan_y * p.aspect_ratio);
        if o.near != near || o.tan_y != tan_y || o.tan_x != tan_x {
            (o.near, o.tan_x, o.tan_y) = (near, tan_x, tan_y);
        }
        let nieuw = in_de_camera(&vloerlicht, &plek.to_matrix().inverse());
        let oud = (o.omhoog, o.oorsprong, o.kring_a, o.kring_a_licht, o.kring_b, o.kring_b_licht, o.strook_van, o.strook_naar, o.strook_licht);
        if oud != nieuw {
            (o.omhoog, o.oorsprong, o.kring_a, o.kring_a_licht, o.kring_b, o.kring_b_licht, o.strook_van, o.strook_naar, o.strook_licht) = nieuw;
        }
    }
}

type Vloerwaarden = (Vec4, Vec4, Vec4, Vec4, Vec4, Vec4, Vec4, Vec4, Vec4);

/// Het licht op de vloer in de ruimte van een camera met deze matrix (van de wereld naar de
/// camera), zoals de shader het leest.
fn in_de_camera(v: &Vloerlicht, naar_camera: &Mat4) -> Vloerwaarden {
    let punt = |p: Vec3| naar_camera.transform_point3(p);
    let kring = |k: Option<Lichtkring>| {
        k.filter(|k| k.straal > 0.0).map_or((Vec4::ZERO, Vec4::ZERO), |k| (punt(k.midden).extend(k.straal), licht4(k.licht)))
    };
    let (ka, kal) = kring(v.kringen[0]);
    let (kb, kbl) = kring(v.kringen[1]);
    let (sv, sn, sl) = v.strook.filter(|s| s.breedte > 0.0).map_or((Vec4::ZERO, Vec4::ZERO, Vec4::ZERO), |s| {
        (punt(s.van).extend(s.breedte / 2.0), punt(s.naar).extend(s.kern / 2.0), licht4(s.licht).with_w(s.kern_licht))
    });
    let omhoog = naar_camera.transform_vector3(Vec3::Y).normalize_or(Vec3::Y).extend(0.0);
    (omhoog, punt(Vec3::ZERO).extend(0.0), ka, kal, kb, kbl, sv, sn, sl)
}

#[derive(Resource)]
struct OcclusieShader(Handle<Shader>);

#[derive(Resource)]
struct OcclusiePijplijn {
    bron: BindGroupLayoutDescriptor,
    diepte: BindGroupLayoutDescriptor,
    sampler: Sampler,
    shader: Handle<Shader>,
    vertex: FullscreenShader,
}

#[derive(Component)]
struct OcclusiePijplijnId(CachedRenderPipelineId);

fn maak_pijplijn(
    mut commands: Commands,
    device: Res<RenderDevice>,
    shader: Res<OcclusieShader>,
    vertex: Res<FullscreenShader>,
) {
    let bron = BindGroupLayoutDescriptor::new(
        "occlusie_bron",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<Occlusie>(true),
            ),
        ),
    );
    let diepte = BindGroupLayoutDescriptor::new(
        "occlusie_diepte",
        &BindGroupLayoutEntries::single(ShaderStages::FRAGMENT, texture_depth_2d_multisampled()),
    );
    commands.insert_resource(OcclusiePijplijn {
        bron,
        diepte,
        sampler: device.create_sampler(&SamplerDescriptor::default()),
        shader: shader.0.clone(),
        vertex: vertex.clone(),
    });
}

impl SpecializedRenderPipeline for OcclusiePijplijn {
    type Key = TextureFormat;

    fn specialize(&self, formaat: TextureFormat) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("occlusie".into()),
            layout: vec![self.bron.clone(), self.diepte.clone()],
            vertex: self.vertex.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: formaat,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

fn kies_pijplijn(
    mut commands: Commands,
    cache: Res<PipelineCache>,
    mut pijplijnen: ResMut<SpecializedRenderPipelines<OcclusiePijplijn>>,
    pijplijn: Option<Res<OcclusiePijplijn>>,
    views: Query<(Entity, &ExtractedView), With<Occlusie>>,
) {
    let Some(pijplijn) = pijplijn else { return };
    for (e, view) in &views {
        let id = pijplijnen.specialize(&cache, &pijplijn, view.target_format);
        commands.entity(e).insert(OcclusiePijplijnId(id));
    }
}

fn occlusie(
    view: ViewQuery<(
        &ViewTarget,
        &ViewDepthStencilTexture,
        &DynamicUniformIndex<Occlusie>,
        &OcclusiePijplijnId,
        &Msaa,
    )>,
    cache: Res<PipelineCache>,
    pijplijn: Option<Res<OcclusiePijplijn>>,
    uniformen: Res<ComponentUniforms<Occlusie>>,
    mut ctx: RenderContext,
) {
    let (doel, diepte, index, id, msaa) = view.into_inner();
    // de shader leest een diepte met meer samples
    if *msaa == Msaa::Off {
        return;
    }
    let (Some(pijplijn), Some(render_pijplijn), Some(instellingen)) =
        (pijplijn, cache.get_render_pipeline(id.0), uniformen.uniforms().binding())
    else {
        return;
    };
    let post = doel.post_process_write();
    let device = ctx.render_device().clone();
    let bron = device.create_bind_group(
        "occlusie_bron",
        &cache.get_bind_group_layout(&pijplijn.bron),
        &BindGroupEntries::sequential((post.source, &pijplijn.sampler, instellingen)),
    );
    let diepte = device.create_bind_group(
        "occlusie_diepte",
        &cache.get_bind_group_layout(&pijplijn.diepte),
        &BindGroupEntries::single(&diepte.attachment.texture.default_view),
    );
    let mut pas = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
        label: Some("occlusie"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pas.set_pipeline(render_pijplijn);
    pas.set_bind_group(0, &bron, &[index.index()]);
    pas.set_bind_group(1, &diepte, &[]);
    pas.draw(0..3, 0..1);
}
