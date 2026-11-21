//! Yancai camera post chain. Pass 1 (HDR, before tonemapping) re-shades
//! deferred pixels into light bands and draws ink outlines; pass 2 (display
//! values, before FXAA) maps colour toward the mineral palette over paper.
//! Both run only while their components are on a camera.
use bevy::{
    anti_alias::fxaa::fxaa,
    asset::{load_internal_asset, uuid_handle},
    camera::Exposure,
    core_pipeline::{
        Core3d, Core3dSystems, FullscreenShader,
        fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
        prepass::ViewPrepassTextures,
        tonemapping::tonemapping,
    },
    ecs::{query::QueryItem, schedule::ScheduleConfigs, system::BoxedSystem},
    pbr::{DistanceFog, FogFalloff},
    prelude::*,
    render::{
        GpuResourceAppExt, Render, RenderApp, RenderStartup, RenderSystems,
        diagnostic::RecordDiagnostics,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, LoadOp,
            Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines, StoreOp,
            TextureFormat, TextureSampleType,
            binding_types::{sampler, texture_2d, texture_depth_2d, uniform_buffer},
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        sync_component::SyncComponent,
        view::{ExtractedView, ViewTarget},
    },
    shader::{Shader, ShaderRef},
};

const SHADE_SHADER: Handle<Shader> = uuid_handle!("3c1d9b0e-5a47-4f8e-9e21-7b6a0c4d2f13");
const PAPER_SHADER: Handle<Shader> = uuid_handle!("a84e2f61-0c3b-4d95-8f7a-1e5b9c6d3a20");

/// Pass 1 settings. `light_over_pi` is the key illuminance / pi, the same
/// normalisation the forest material uses for its forward bands.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(in crate::client) struct YancaiShade {
    pub light_over_pi: f32,
    pub bands: f32,
}

/// Pass 2 settings, uploaded as-is.
#[derive(Component, ExtractComponent, Clone, Copy, Debug, PartialEq, ShaderType)]
#[extract_app(RenderApp)]
pub(in crate::client) struct YancaiPaper {
    /// Pull toward the nearest mineral pigment hue.
    pub palette: f32,
    /// Paper fibre and blotch contrast.
    pub paper: f32,
    /// Darkening where colour fields meet, like pooled pigment.
    pub pooling: f32,
    pub _pad: f32,
}

impl Default for YancaiPaper {
    fn default() -> Self {
        Self {
            palette: 0.3,
            paper: 0.07,
            pooling: 0.18,
            _pad: 0.0,
        }
    }
}

impl FullscreenMaterial for YancaiPaper {
    fn fragment_shader() -> ShaderRef {
        PAPER_SHADER.into()
    }

    fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
        // Display-referred colour, and FXAA still smooths the ink lines.
        system
            .in_set(Core3dSystems::PostProcess)
            .after(tonemapping)
            .before(fxaa)
    }
}

#[derive(Component, ShaderType, Clone, Copy, Default)]
pub(in crate::client) struct YancaiShadeUniform {
    view_from_clip: Mat4,
    /// Linear fog colour and alpha; Bevy mixes it into lit colour unscaled.
    fog_color: Vec4,
    /// x = exposure * light_over_pi, y = bands, z = exp² fog density.
    params: Vec4,
}

impl SyncComponent<RenderApp> for YancaiShade {
    type Target = YancaiShadeUniform;
}

impl ExtractComponent<RenderApp> for YancaiShade {
    type QueryData = (
        &'static Self,
        &'static Camera,
        Option<&'static Exposure>,
        Option<&'static DistanceFog>,
    );
    type QueryFilter = With<Camera3d>;
    type Out = YancaiShadeUniform;

    fn extract_component(
        (shade, camera, exposure, fog): QueryItem<Self::QueryData>,
    ) -> Option<Self::Out> {
        Some(uniform(
            *shade,
            camera.clip_from_view(),
            exposure.copied().unwrap_or_default().exposure(),
            fog,
        ))
    }
}

fn uniform(
    shade: YancaiShade,
    clip_from_view: Mat4,
    exposure: f32,
    fog: Option<&DistanceFog>,
) -> YancaiShadeUniform {
    // Only exponential-squared fog is undone; others are left in place.
    let (fog_color, density) = match fog {
        Some(DistanceFog {
            color,
            falloff: FogFalloff::ExponentialSquared { density },
            ..
        }) => (LinearRgba::from(*color).to_vec4(), *density),
        _ => (Vec4::ZERO, 0.0),
    };
    YancaiShadeUniform {
        view_from_clip: clip_from_view.inverse(),
        fog_color,
        params: Vec4::new(exposure * shade.light_over_pi, shade.bands, density, 0.0),
    }
}

pub(in crate::client) fn configure(app: &mut App) {
    load_internal_asset!(
        app,
        SHADE_SHADER,
        "../embedded/shaders/yancai_shade.wesl",
        Shader::from_wesl
    );
    load_internal_asset!(
        app,
        PAPER_SHADER,
        "../embedded/shaders/yancai_paper.wesl",
        Shader::from_wesl
    );
    app.add_plugins((
        ExtractComponentPlugin::<YancaiShade>::default(),
        UniformComponentPlugin::<YancaiShadeUniform>::default(),
        FullscreenMaterialPlugin::<YancaiPaper>::default(),
    ));
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_gpu_resource::<SpecializedRenderPipelines<ShadePipeline>>()
        .add_systems(RenderStartup, init_pipeline)
        .add_systems(Render, prepare_pipelines.in_set(RenderSystems::Prepare))
        .add_systems(
            Core3d,
            shade_pass
                .in_set(Core3dSystems::PostProcess)
                .before(tonemapping),
        );
}

#[derive(Resource)]
struct ShadePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    fullscreen: FullscreenShader,
}

fn init_pipeline(
    mut commands: Commands,
    device: Res<RenderDevice>,
    fullscreen: Res<FullscreenShader>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "yancai_shade_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                texture_depth_2d(),
                texture_2d(TextureSampleType::Uint),
                texture_2d(TextureSampleType::Uint),
                texture_2d(TextureSampleType::Float { filterable: false }),
                uniform_buffer::<YancaiShadeUniform>(true),
            ),
        ),
    );
    commands.insert_resource(ShadePipeline {
        layout,
        sampler: device.create_sampler(&SamplerDescriptor::default()),
        fullscreen: fullscreen.clone(),
    });
}

impl SpecializedRenderPipeline for ShadePipeline {
    type Key = TextureFormat;

    fn specialize(&self, format: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("yancai_shade_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: self.fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: SHADE_SHADER,
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

#[derive(Component)]
struct ShadePipelineId(CachedRenderPipelineId);

fn prepare_pipelines(
    mut commands: Commands,
    cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<ShadePipeline>>,
    pipeline: Res<ShadePipeline>,
    views: Query<(Entity, &ExtractedView, Has<YancaiShadeUniform>)>,
) {
    // Render-world views persist across frames: a camera that leaves yancai
    // must lose its pipeline id, or the pass would keep running on a stale
    // uniform index.
    for (entity, view, styled) in &views {
        if styled {
            let id = pipelines.specialize(&cache, &pipeline, view.target_format);
            commands.entity(entity).insert(ShadePipelineId(id));
        } else {
            commands.entity(entity).remove::<ShadePipelineId>();
        }
    }
}

/// Needs the deferred gbuffer, lighting-pass ids and the resident normal
/// prepass; a camera missing any of them is left untouched.
fn shade_pass(
    view: ViewQuery<
        (
            &ViewTarget,
            &ShadePipelineId,
            &ViewPrepassTextures,
            &DynamicUniformIndex<YancaiShadeUniform>,
        ),
        With<YancaiShadeUniform>,
    >,
    shade: Res<ShadePipeline>,
    cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<YancaiShadeUniform>>,
    mut ctx: RenderContext,
) {
    let (target, pipeline_id, prepass, index) = view.into_inner();
    let Some(pipeline) = cache.get_render_pipeline(pipeline_id.0) else {
        return;
    };
    let (Some(depth), Some(gbuffer), Some(lighting_id), Some(normal)) = (
        prepass.depth_only_view(),
        prepass.deferred.as_ref(),
        prepass.deferred_lighting_pass_id.as_ref(),
        prepass.normal_view(),
    ) else {
        return;
    };
    let Some(binding) = uniforms.uniforms().binding() else {
        return;
    };
    let post = target.post_process_write();
    let bind_group = ctx.render_device().create_bind_group(
        Some("yancai_shade_bind_group"),
        &cache.get_bind_group_layout(&shade.layout),
        &BindGroupEntries::sequential((
            post.source,
            &shade.sampler,
            depth,
            &gbuffer.texture.default_view,
            &lighting_id.texture.default_view,
            normal,
            binding.clone(),
        )),
    );
    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();
    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("yancai_shade"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Default::default()),
                store: StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    let span = diagnostics.pass_span(&mut pass, "yancai_shade");
    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[index.index()]);
    pass.draw(0..3, 0..1);
    span.end(&mut pass);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::shader::{ShaderCache, ShaderCacheSource, ShaderImport};

    #[test]
    fn the_shade_uniform_undoes_only_exponential_squared_fog() {
        let shade = YancaiShade {
            light_over_pi: 1_000.0,
            bands: 3.0,
        };
        let clip = {
            use bevy::camera::CameraProjection;
            PerspectiveProjection::default().get_clip_from_view()
        };
        let fog = DistanceFog {
            color: Color::srgb(1.0, 1.0, 1.0),
            falloff: FogFalloff::ExponentialSquared { density: 0.04 },
            ..default()
        };
        let with_fog = uniform(shade, clip, 0.5, Some(&fog));
        assert_eq!(with_fog.params, Vec4::new(500.0, 3.0, 0.04, 0.0));
        assert_eq!(with_fog.fog_color, Vec4::ONE);
        assert!((with_fog.view_from_clip * clip).abs_diff_eq(Mat4::IDENTITY, 1e-4));
        let linear = DistanceFog {
            falloff: FogFalloff::Linear {
                start: 1.0,
                end: 2.0,
            },
            ..fog
        };
        assert_eq!(uniform(shade, clip, 0.5, Some(&linear)).params.z, 0.0);
        assert_eq!(uniform(shade, clip, 0.5, None).fog_color, Vec4::ZERO);
    }

    #[test]
    fn both_post_shaders_compose_without_a_gpu() {
        let mut cache = ShaderCache::new((), |_, source, _| match source {
            ShaderCacheSource::Wgsl(source) => Ok(source),
            _ => panic!("expected WGSL after WESL composition"),
        });
        let mut shaders = Assets::<Shader>::default();
        let mut stub = Shader::from_wesl(
            "struct FullscreenVertexOutput { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> };",
            "bevy_core_pipeline::fullscreen_vertex_shader",
        );
        stub.import_path =
            ShaderImport::Custom("bevy_core_pipeline::fullscreen_vertex_shader".into());
        let handle = shaders.add(stub.clone());
        cache.set_shader(handle.id(), stub);
        for (id, source, name, needles) in [
            (
                SHADE_SHADER.id(),
                include_str!("../embedded/shaders/yancai_shade.wesl"),
                "yancai_shade.wesl",
                &["@binding(6)", "lighting_id", "prepass_normal", "fog_factor"][..],
            ),
            (
                PAPER_SHADER.id(),
                include_str!("../embedded/shaders/yancai_paper.wesl"),
                "yancai_paper.wesl",
                &["@binding(2)", "palette"][..],
            ),
        ] {
            cache.set_shader(id, Shader::from_wesl(source, name));
            let composed = cache.get(0, id, &[]).unwrap();
            assert!(composed.contains("@fragment"), "{name}");
            for needle in needles {
                assert!(composed.contains(needle), "{name} lacks {needle}");
            }
        }
    }
}
