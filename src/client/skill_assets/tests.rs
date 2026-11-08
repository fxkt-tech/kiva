use super::catalog::{SpriteForward, VfxBlend};
use super::*;
use bevy::prelude::*;
use serde_json::Value;
use std::path::Path;

fn packaged(path: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(path),
    )
    .unwrap()
}

#[test]
fn every_asset_key_has_a_local_packaged_specification() {
    let catalog: Value = serde_json::from_slice(&packaged("vfx/catalog.json")).unwrap();
    assert_eq!(
        catalog["textures"].as_array().unwrap().len(),
        TextureKey::ALL.len()
    );
    assert_eq!(
        catalog["meshes"].as_array().unwrap().len(),
        MeshKey::ALL.len()
    );
    for key in TextureKey::ALL {
        let spec = texture_spec(key);
        assert_eq!(spec.key, key);
        assert!(spec.path.starts_with("vfx/textures/"));
        let png = packaged(&spec.path);
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(
            u32::from_be_bytes(png[16..20].try_into().unwrap()),
            spec.size[0]
        );
        assert_eq!(
            u32::from_be_bytes(png[20..24].try_into().unwrap()),
            spec.size[1]
        );
        assert!(!spec.source_system.is_empty());
        assert!(!spec.source_emitter.is_empty());
        assert!(spec.atlas.frames > 0);
        assert!(spec.atlas.frames as u16 <= spec.atlas.columns as u16 * spec.atlas.rows as u16);
        assert!(spec.atlas.uv_transform(spec.atlas.default_frame).is_some());
    }
}

#[test]
fn indicators_use_black_safe_add_and_only_single_source_frames() {
    for key in [
        TextureKey::IndicatorAoe,
        TextureKey::IndicatorTarget,
        TextureKey::IndicatorRange,
        TextureKey::IndicatorLineBody,
        TextureKey::IndicatorLineHead,
        TextureKey::IndicatorFan,
    ] {
        let spec = texture_spec(key);
        assert_eq!(spec.blend, VfxBlend::Add);
        assert_eq!(
            (spec.atlas.columns, spec.atlas.rows, spec.atlas.frames),
            (1, 1, 1)
        );
        assert!(spec.source_blend.is_none());
    }
    assert_eq!(
        texture_spec(TextureKey::IndicatorFan).sprite_forward,
        SpriteForward::Up
    );
    assert_eq!(
        texture_spec(TextureKey::AsheVolleyArrow).sprite_forward,
        SpriteForward::Right
    );
}

#[test]
fn source_grid_frames_sample_one_symbol_or_rock_instead_of_the_whole_sheet() {
    let symbols = texture_spec(TextureKey::AsheQSymbol).atlas;
    let first = symbols.uv_transform(0).unwrap();
    let second = symbols.uv_transform(1).unwrap();
    assert_eq!(first.transform_point2(Vec2::ZERO), Vec2::ZERO);
    assert_eq!(first.transform_point2(Vec2::ONE), Vec2::new(0.5, 1.0));
    assert_eq!(second.transform_point2(Vec2::ZERO), Vec2::new(0.5, 0.0));
    assert_eq!(second.transform_point2(Vec2::ONE), Vec2::ONE);
    assert!(symbols.uv_transform(2).is_none());
    let rock = texture_spec(TextureKey::JarvanRockAtlas).atlas;
    let fourth = rock.uv_transform(3).unwrap();
    assert_eq!(fourth.transform_point2(Vec2::ZERO), Vec2::splat(0.5));
    assert_eq!(fourth.transform_point2(Vec2::ONE), Vec2::ONE);
    assert!(rock.uv_transform(4).is_none());
}

fn attribute(document: &Value, binary: &[u8], name: &str, width: usize) -> Vec<Vec<f32>> {
    let index = document["meshes"][0]["primitives"][0]["attributes"][name]
        .as_u64()
        .unwrap() as usize;
    let accessor = &document["accessors"][index];
    assert_eq!(accessor["componentType"], 5126);
    let view = &document["bufferViews"][accessor["bufferView"].as_u64().unwrap() as usize];
    let start = view["byteOffset"].as_u64().unwrap_or(0) as usize;
    (0..accessor["count"].as_u64().unwrap() as usize)
        .map(|vertex| {
            (0..width)
                .map(|component| {
                    let at = start + (vertex * width + component) * 4;
                    f32::from_le_bytes(binary[at..at + 4].try_into().unwrap())
                })
                .collect()
        })
        .collect()
}

#[test]
fn actual_imported_meshes_preserve_uv_colors_and_report_real_converted_bounds() {
    for key in MeshKey::ALL {
        let spec = mesh_spec(key);
        assert_eq!(spec.key, key);
        assert!(spec.path.starts_with("vfx/meshes/"));
        let glb = packaged(&spec.path);
        assert_eq!(&glb[..4], b"glTF");
        assert_eq!(u32::from_le_bytes(glb[4..8].try_into().unwrap()), 2);
        assert_eq!(
            u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize,
            glb.len()
        );
        let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let document: Value = serde_json::from_slice(&glb[20..20 + json_length]).unwrap();
        let binary = &glb[28 + json_length..];
        let positions = attribute(&document, binary, "POSITION", 3);
        let uvs = attribute(&document, binary, "TEXCOORD_0", 2);
        let colors = attribute(&document, binary, "COLOR_0", 4);
        let normals = attribute(&document, binary, "NORMAL", 3);
        assert!(!positions.is_empty());
        assert_eq!(positions.len() % 3, 0);
        assert_eq!(
            (uvs.len(), colors.len(), normals.len()),
            (positions.len(), positions.len(), positions.len())
        );
        assert!(
            positions
                .iter()
                .chain(&uvs)
                .chain(&colors)
                .chain(&normals)
                .flatten()
                .all(|v| v.is_finite())
        );
        assert!(
            colors
                .iter()
                .flatten()
                .all(|value| (0.0..=1.0).contains(value))
        );
        let min = Vec3::new(
            positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min),
            positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min),
            positions.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min),
        );
        let max = Vec3::new(
            positions
                .iter()
                .map(|p| p[0])
                .fold(f32::NEG_INFINITY, f32::max),
            positions
                .iter()
                .map(|p| p[1])
                .fold(f32::NEG_INFINITY, f32::max),
            positions
                .iter()
                .map(|p| p[2])
                .fold(f32::NEG_INFINITY, f32::max),
        );
        assert!(min.abs_diff_eq(Vec3::from_array(spec.bounds.min), 1e-6));
        assert!(max.abs_diff_eq(Vec3::from_array(spec.bounds.max), 1e-6));
        assert!(spec.bounds.center().is_finite());
        assert!(spec.bounds.size().min_element() >= 0.0);
        assert!(spec.source_bounds.size().is_finite());
        assert_eq!(spec.source_to_world, 0.01);
        assert_eq!(spec.up, [0.0, 1.0, 0.0]);
        assert!(
            spec.source_to_game
                .into_iter()
                .flatten()
                .all(f32::is_finite)
        );
        assert!(TextureKey::ALL.contains(&spec.texture));
        if key == MeshKey::JarvanStandard {
            // This particle's legacy face VCP is entirely zero; multiplying
            // it into the hero atlas made the original standard solid black.
            // Its reviewed adaptation preserves white vertex RGBA and UVs.
            assert!(
                colors
                    .iter()
                    .all(|color| color.iter().all(|channel| *channel == 1.0))
            );
            assert!(uvs.iter().any(|uv| uv[0] < 0.01));
            assert!(uvs.iter().any(|uv| uv[0] > 0.99));
        }
        if key == MeshKey::AsheCrystalArrow || key == MeshKey::AsheCrystalTrail {
            assert_eq!(spec.forward, Some([0.0, 0.0, -1.0]));
            assert!(spec.bounds.size().z > spec.bounds.size().x);
            assert!(colors.iter().any(|color| color[3] == 0.0));
        }
    }
}

#[test]
fn unloaded_assets_explicitly_offer_fallback_without_fabricating_readiness() {
    let assets = SkillAssets::default();
    assert!(!assets.status().ready);
    assert!(assets.status().errors.is_empty());
    assert!(assets.image(TextureKey::IndicatorAoe).is_none());
    assert!(assets.material(TextureKey::IndicatorAoe).is_none());
    assert!(assets.mesh(MeshKey::AsheCrystalArrow).is_none());
}

#[test]
fn startup_validation_rejects_incomplete_invalid_sources_grids_and_axes() {
    let original: Value = serde_json::from_slice(&packaged("vfx/catalog.json")).unwrap();
    assert!(super::catalog::validate_json(&original.to_string()).is_ok());
    let cases = [
        (
            "/textures/0/key",
            serde_json::json!("AsheQSymbol"),
            "unique and complete",
        ),
        (
            "/textures/0/size/0",
            serde_json::json!(0),
            "texture/grid/source",
        ),
        (
            "/textures/0/source_system",
            serde_json::json!(""),
            "texture/grid/source",
        ),
        (
            "/textures/0/source_blend",
            serde_json::json!(255),
            "unreviewed source blend",
        ),
        (
            "/textures/0/atlas/columns",
            serde_json::json!(0),
            "texture/grid/source",
        ),
        (
            "/meshes/0/source_to_world",
            serde_json::json!(-0.01),
            "mesh/bounds/scale",
        ),
        (
            "/meshes/0/source_to_game/0/0",
            serde_json::json!(2),
            "orthonormal",
        ),
        ("/meshes/0/up", serde_json::json!([0, 0, 0]), "orthonormal"),
        (
            "/meshes/1/forward",
            serde_json::json!([0, 1, 0]),
            "orthonormal",
        ),
        (
            "/meshes/0/bounds/max/0",
            serde_json::json!(1000),
            "exceed the converted source",
        ),
    ];
    for (pointer, replacement, expected) in cases {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        let error = super::catalog::validate_json(&changed.to_string()).unwrap_err();
        assert!(error.contains(expected), "{pointer}: {error}");
    }
}

#[test]
fn headless_fixture_uses_the_production_material_recipe_without_an_asset_server() {
    let mut world = World::new();
    let assets = SkillAssets::fixture(&mut world);
    assert!(assets.status().ready);
    assert!(assets.status().errors.is_empty());
    for key in TextureKey::ALL {
        let spec = texture_spec(key);
        let materials = world.resource::<Assets<StandardMaterial>>();
        let material = materials.get(assets.material(key).unwrap()).unwrap();
        assert_eq!(material.alpha_mode, AlphaMode::from(spec.blend));
        assert_eq!(material.base_color_texture.as_ref(), assets.image(key));
        assert_eq!(
            material.uv_transform,
            spec.atlas.uv_transform(spec.atlas.default_frame).unwrap()
        );
        assert!(material.unlit && material.double_sided);
        assert!(
            world
                .resource::<Assets<Image>>()
                .get(assets.image(key).unwrap())
                .is_some()
        );
    }
    for key in MeshKey::ALL {
        assert!(
            world
                .resource::<Assets<Mesh>>()
                .get(assets.mesh(key).unwrap())
                .is_some()
        );
    }
}
