//! Occlusie uit de diepte: een stap na het tekenen van de Zaal die hoeken, plooien en de vloer
//! naast bureaus en banken iets donkerder maakt (zie `occlusie.wgsl`).
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
    _a: f32,
    _b: f32,
    _c: f32,
}

impl Occlusie {
    pub fn new(straal: f32, sterkte: f32) -> Self {
        Occlusie { near: 0.05, tan_x: 1.0, tan_y: 1.0, straal, sterkte, _a: 0.0, _b: 0.0, _c: 0.0 }
    }
}

pub struct OcclusiePlugin;

impl Plugin for OcclusiePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<Occlusie>::default(),
            UniformComponentPlugin::<Occlusie>::default(),
        ))
        .add_systems(PostUpdate, volg_de_camera);
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

/// Near en beeldhoek van de camera naar de stap.
fn volg_de_camera(mut q: Query<(&Projection, &mut Occlusie)>) {
    for (projectie, mut o) in &mut q {
        let Projection::Perspective(p) = projectie else {
            continue;
        };
        let tan_y = (p.fov / 2.0).tan();
        let (near, tan_x) = (p.near, tan_y * p.aspect_ratio);
        if o.near != near || o.tan_y != tan_y || o.tan_x != tan_x {
            (o.near, o.tan_x, o.tan_y) = (near, tan_x, tan_y);
        }
    }
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
