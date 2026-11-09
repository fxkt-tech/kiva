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

pub(super) type ForestMaterial = ExtendedMaterial<StandardMaterial, ForestSurface>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub(super) struct ForestSurface {
    /// Surface kind (soil, culm, stone, leaf), detail strength and spatial scale.
    #[uniform(100)]
    pub style: Vec4,
}

impl MaterialExtension for ForestSurface {
    fn fragment_shader() -> ShaderRef {
        FOREST_SHADER.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        FOREST_SHADER.into()
    }
}

pub(super) struct ForestMaterialPlugin;

impl Plugin for ForestMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            FOREST_SHADER,
            "../../assets/shaders/forest.wesl",
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
                "struct View { world_position: vec3<f32> }; @group(0) @binding(0) var<uniform> view: View;",
            ),
            (
                "bevy_pbr::render::forward_io",
                "struct VertexOutput { @builtin(position) position: vec4<f32>, world_position: vec4<f32> }; struct FragmentOutput { @location(0) color: vec4<f32> };",
            ),
            (
                "bevy_pbr::prepass::io",
                "import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};",
            ),
            (
                "bevy_pbr::render::pbr_fragment",
                "import bevy_pbr::render::forward_io::VertexOutput; struct SurfaceMaterial { base_color: vec4<f32>, perceptual_roughness: f32, clearcoat: f32, clearcoat_perceptual_roughness: f32, flags: u32, alpha_cutoff: f32 }; struct Surface { material: SurfaceMaterial, N: vec3<f32> }; fn pbr_input_from_standard_material(in: VertexOutput, front: bool) -> Surface { return Surface(); }",
            ),
            (
                "bevy_pbr::render::pbr_functions",
                "import bevy_pbr::render::pbr_fragment::Surface; fn alpha_discard(flags: u32, cutoff: f32, color: vec4<f32>) -> vec4<f32> { return color; } fn apply_pbr_lighting(surface: Surface) -> vec4<f32> { return surface.material.base_color; } fn main_pass_post_lighting_processing(surface: Surface, color: vec4<f32>) -> vec4<f32> { return color; }",
            ),
            (
                "bevy_pbr::deferred::functions",
                "import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput}; import bevy_pbr::render::pbr_fragment::Surface; fn deferred_output(in: VertexOutput, surface: Surface) -> FragmentOutput { return FragmentOutput(surface.material.base_color); }",
            ),
        ];
        for (path, source) in modules {
            let mut shader = Shader::from_wesl(source, path);
            shader.import_path = ShaderImport::Custom(path.into());
            let handle = shaders.add(shader.clone());
            cache.set_shader(handle.id(), shader);
        }
        cache.set_shader(
            FOREST_SHADER.id(),
            Shader::from_wesl(
                include_str!("../../assets/shaders/forest.wesl"),
                "forest.wesl",
            ),
        );
        for deferred in [false, true] {
            let source = cache
                .get(
                    0,
                    FOREST_SHADER.id(),
                    &[
                        ShaderDefVal::UInt("MATERIAL_BIND_GROUP".into(), 3),
                        ShaderDefVal::Bool("PREPASS_PIPELINE".into(), deferred),
                    ],
                )
                .unwrap();
            assert!(source.contains("@fragment"));
            assert!(source.contains("@binding(100)"));
            assert!(source.contains("@binding(0)"));
            assert!(source.contains("discard;"));
        }
    }

    #[test]
    fn camera_leaf_clipping_keeps_standard_vertex_depth_and_shadow_passes() {
        // The view-dependent fade belongs to colour/deferred rendering; branch
        // cards retain the source alpha mask in Bevy's standard shadow pass.
        assert!(matches!(ForestSurface::vertex_shader(), ShaderRef::Default));
        assert!(matches!(
            ForestSurface::prepass_fragment_shader(),
            ShaderRef::Default
        ));
        assert!(ForestSurface::enable_shadows());
    }
}
