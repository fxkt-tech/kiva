//! Authored woodland surface detail layered onto Bevy's standard PBR pipeline.
//! This adapts the rebuilt arena; it is not recovered Messiah shader source.
use bevy::{
    asset::{load_internal_asset, uuid_handle},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

const FOREST_SHADER: Handle<Shader> = uuid_handle!("45c0843d-dad9-4b11-a4be-596740c6c1f8");
const FOREST_VIEW_SHADER: Handle<Shader> = uuid_handle!("beef28bb-05ab-46e4-97b7-e0c4ce7cfe85");
const FOREST_PREPASS_SHADER: Handle<Shader> = uuid_handle!("84e1955d-1b58-4f5e-8db3-58fe387f277f");

pub(super) type ForestMaterial = ExtendedMaterial<StandardMaterial, ForestSurface>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub(super) struct ForestSurface {
    /// Surface kind (soil, culm, stone, leaf), detail strength, spatial scale
    /// and soil wetness. Litter shares soil detail with wetness zero.
    #[uniform(100)]
    pub style: Vec4,
    /// Optional opaque moss/grass ground layer; source alpha is not an ORM.
    #[texture(101)]
    #[sampler(102)]
    pub cover_base: Option<Handle<Image>>,
    #[texture(103)]
    #[sampler(104)]
    pub cover_normal: Option<Handle<Image>>,
    #[uniform(105)]
    pub cover_strength: f32,
}

impl MaterialExtension for ForestSurface {
    fn fragment_shader() -> ShaderRef {
        FOREST_SHADER.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        FOREST_SHADER.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        FOREST_PREPASS_SHADER.into()
    }
}

pub(super) struct ForestMaterialPlugin;

impl Plugin for ForestMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            FOREST_VIEW_SHADER,
            "embedded/shaders/forest_view.wesl",
            |source, _path| Shader::from_wesl(source, "embedded://kiva/forest_view.wesl")
        );
        load_internal_asset!(
            app,
            FOREST_PREPASS_SHADER,
            "embedded/shaders/forest_prepass.wesl",
            Shader::from_wesl
        );
        load_internal_asset!(
            app,
            FOREST_SHADER,
            "embedded/shaders/forest.wesl",
            Shader::from_wesl
        );
        app.add_plugins(MaterialPlugin::<ForestMaterial>::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::shader::{ShaderCache, ShaderCacheSource, ShaderDefVal, ShaderImport};

    #[test]
    fn woodland_shader_composes_forward_and_deferred_pbr_interfaces() {
        // Run Bevy's actual WESL resolver/conditional compiler without a GPU.
        // The minimal imported interfaces isolate our shader's composition;
        // real PBR binding/lighting validation additionally runs in the browser.
        let mut cache = ShaderCache::new((), |_, source, _| match source {
            ShaderCacheSource::Wgsl(source) => Ok(source),
            _ => panic!("expected WGSL after WESL composition"),
        });
        let mut shaders = Assets::<Shader>::default();
        let modules = [
            (
                "bevy_pbr::render::mesh_view_bindings",
                "struct View { world_position: vec3<f32>, lod_view_world_position: vec3<f32> }; @group(0) @binding(0) var<uniform> view: View;",
            ),
            (
                "bevy_pbr::render::forward_io",
                "struct VertexOutput { @builtin(position) position: vec4<f32>, world_position: vec4<f32>, world_tangent: vec4<f32>, previous_world_position: vec4<f32>, unclipped_depth: f32 }; struct FragmentOutput { @location(0) color: vec4<f32>, @location(1) normal: vec4<f32>, @location(2) motion_vector: vec2<f32>, @builtin(frag_depth) frag_depth: f32 };",
            ),
            (
                "bevy_pbr::prepass::io",
                "import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};",
            ),
            (
                "bevy_pbr::render::pbr_fragment",
                "import bevy_pbr::render::forward_io::VertexOutput; struct SurfaceMaterial { base_color: vec4<f32>, perceptual_roughness: f32, clearcoat: f32, clearcoat_perceptual_roughness: f32, flags: u32, alpha_cutoff: f32 }; struct Surface { material: SurfaceMaterial, N: vec3<f32>, world_normal: vec3<f32>, clearcoat_N: vec3<f32> }; fn pbr_input_from_standard_material(in: VertexOutput, front: bool) -> Surface { return Surface(); }",
            ),
            (
                "bevy_pbr::render::pbr_functions",
                "import bevy_pbr::render::pbr_fragment::Surface; fn alpha_discard(flags: u32, cutoff: f32, color: vec4<f32>) -> vec4<f32> { return color; } fn calculate_tbn_mikktspace(normal: vec3<f32>, tangent: vec4<f32>) -> mat3x3<f32> { return mat3x3<f32>(); } fn apply_normal_mapping(flags: u32, tbn: mat3x3<f32>, double: bool, front: bool, normal: vec3<f32>) -> vec3<f32> { return normalize(normal); } fn apply_pbr_lighting(surface: Surface) -> vec4<f32> { return surface.material.base_color; } fn main_pass_post_lighting_processing(surface: Surface, color: vec4<f32>) -> vec4<f32> { return color; }",
            ),
            (
                "bevy_pbr::deferred::functions",
                "import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput}; import bevy_pbr::render::pbr_fragment::Surface; fn deferred_output(in: VertexOutput, surface: Surface) -> FragmentOutput { return FragmentOutput(); }",
            ),
            (
                "bevy_pbr::render::pbr_prepass_functions",
                "import bevy_pbr::prepass::io::VertexOutput; fn prepass_sample_color_and_alpha_discard(in: VertexOutput) {} fn calculate_motion_vector(world: vec4<f32>, previous: vec4<f32>) -> vec2<f32> { return vec2<f32>(0.0); }",
            ),
        ];
        for (path, source) in modules {
            let mut shader = Shader::from_wesl(source, path);
            shader.import_path = ShaderImport::Custom(path.into());
            let handle = shaders.add(shader.clone());
            cache.set_shader(handle.id(), shader);
        }
        cache.set_shader(
            FOREST_VIEW_SHADER.id(),
            Shader::from_wesl(
                include_str!("embedded/shaders/forest_view.wesl"),
                "embedded://kiva/forest_view.wesl",
            ),
        );
        cache.set_shader(
            FOREST_SHADER.id(),
            Shader::from_wesl(include_str!("embedded/shaders/forest.wesl"), "forest.wesl"),
        );
        for deferred in [false, true] {
            let source = cache
                .get(
                    0,
                    FOREST_SHADER.id(),
                    &[
                        ShaderDefVal::UInt("MATERIAL_BIND_GROUP".into(), 3),
                        ShaderDefVal::Bool("PREPASS_PIPELINE".into(), deferred),
                        ShaderDefVal::Bool("VERTEX_TANGENTS".into(), true),
                    ],
                )
                .unwrap();
            assert!(source.contains("@fragment"));
            assert!(source.contains("@binding(100)"));
            assert!(source.contains("@binding(0)"));
            assert!(source.contains("discard;"));
        }
        cache.set_shader(
            FOREST_PREPASS_SHADER.id(),
            Shader::from_wesl(
                include_str!("embedded/shaders/forest_prepass.wesl"),
                "forest_prepass.wesl",
            ),
        );
        // Depth-only camera/shadow, normal/motion, and emulated unclipped
        // shadow depth must all link the same camera/leaf coverage helper.
        for (fragment, normal, motion, unclipped) in [
            (false, false, false, false),
            (true, true, false, false),
            (true, true, true, false),
            (true, false, false, true),
        ] {
            let source = cache
                .get(
                    0,
                    FOREST_PREPASS_SHADER.id(),
                    &[
                        ShaderDefVal::UInt("MATERIAL_BIND_GROUP".into(), 3),
                        ShaderDefVal::Bool("PREPASS_PIPELINE".into(), true),
                        ShaderDefVal::Bool("PREPASS_FRAGMENT".into(), fragment),
                        ShaderDefVal::Bool("NORMAL_PREPASS".into(), normal),
                        ShaderDefVal::Bool("MOTION_VECTOR_PREPASS".into(), motion),
                        ShaderDefVal::Bool("UNCLIPPED_DEPTH_ORTHO_EMULATION".into(), unclipped),
                    ],
                )
                .unwrap();
            assert!(source.contains("@fragment"));
            assert!(source.contains("discard;"));
            assert!(source.contains("lod_view_world_position"));
        }
    }

    #[test]
    fn camera_leaf_clipping_keeps_standard_vertices_and_shadow_support() {
        // GPU skinning and shadow vertices remain the standard Bevy shaders;
        // colour/deferred/depth fragments share camera leaf coverage.
        assert!(matches!(ForestSurface::vertex_shader(), ShaderRef::Default));
        assert!(!matches!(
            ForestSurface::prepass_fragment_shader(),
            ShaderRef::Default
        ));
        assert!(ForestSurface::enable_shadows());
    }
}
