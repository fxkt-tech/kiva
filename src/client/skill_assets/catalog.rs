use bevy::{math::Affine2, prelude::*};
use serde::Deserialize;
use std::{collections::HashSet, sync::LazyLock};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub(in crate::client) enum TextureKey {
    AsheQArrow,
    AsheQSymbol,
    AsheVolleyArrow,
    AsheVolleyGlow,
    AsheScoutAtlas,
    AsheScanRing,
    AsheCrystalArrow,
    AsheCrystalTrail,
    AsheImpactRing,
    AsheImpactFlash,
    JarvanThrustStreak,
    JarvanShield,
    JarvanNovaRing,
    JarvanStandardAtlas,
    JarvanFlag,
    JarvanRockAtlas,
    JarvanGroundAtlas,
    JarvanCracks,
    IndicatorAoe,
    IndicatorTarget,
    IndicatorRange,
    IndicatorLineBody,
    IndicatorLineHead,
    IndicatorFan,
}
impl TextureKey {
    pub const ALL: [Self; 24] = [
        Self::AsheQArrow,
        Self::AsheQSymbol,
        Self::AsheVolleyArrow,
        Self::AsheVolleyGlow,
        Self::AsheScoutAtlas,
        Self::AsheScanRing,
        Self::AsheCrystalArrow,
        Self::AsheCrystalTrail,
        Self::AsheImpactRing,
        Self::AsheImpactFlash,
        Self::JarvanThrustStreak,
        Self::JarvanShield,
        Self::JarvanNovaRing,
        Self::JarvanStandardAtlas,
        Self::JarvanFlag,
        Self::JarvanRockAtlas,
        Self::JarvanGroundAtlas,
        Self::JarvanCracks,
        Self::IndicatorAoe,
        Self::IndicatorTarget,
        Self::IndicatorRange,
        Self::IndicatorLineBody,
        Self::IndicatorLineHead,
        Self::IndicatorFan,
    ];
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub(in crate::client) enum MeshKey {
    AsheQSymbol,
    AsheCrystalArrow,
    AsheCrystalTrail,
    AsheScout,
    AsheImpactRing,
    JarvanStandard,
    JarvanRock,
    JarvanImpactGround,
    JarvanNovaRing,
}
impl MeshKey {
    pub const ALL: [Self; 9] = [
        Self::AsheQSymbol,
        Self::AsheCrystalArrow,
        Self::AsheCrystalTrail,
        Self::AsheScout,
        Self::AsheImpactRing,
        Self::JarvanStandard,
        Self::JarvanRock,
        Self::JarvanImpactGround,
        Self::JarvanNovaRing,
    ];
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub(in crate::client) enum VfxBlend {
    Opaque,
    Blend,
    Add,
}
impl From<VfxBlend> for AlphaMode {
    fn from(value: VfxBlend) -> Self {
        match value {
            VfxBlend::Opaque => Self::Opaque,
            VfxBlend::Blend => Self::Blend,
            VfxBlend::Add => Self::Add,
        }
    }
}

/// Direction in the source image. It is not a gameplay direction.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub(in crate::client) enum SpriteForward {
    None,
    Up,
    Right,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::client) struct AtlasSpec {
    pub columns: u8,
    pub rows: u8,
    pub frames: u8,
    pub default_frame: u8,
}
impl AtlasSpec {
    /// Select one reviewed grid cell. Nonuniform mesh UV atlases remain 1×1.
    pub fn uv_transform(self, frame: u8) -> Option<Affine2> {
        if self.columns == 0 || self.rows == 0 || frame >= self.frames {
            return None;
        }
        let scale = Vec2::new(1.0 / self.columns as f32, 1.0 / self.rows as f32);
        let offset = Vec2::new((frame % self.columns) as f32, (frame / self.columns) as f32);
        Some(Affine2::from_scale_angle_translation(
            scale,
            0.0,
            offset * scale,
        ))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::client) struct TextureSpec {
    pub key: TextureKey,
    pub path: String,
    pub size: [u32; 2],
    pub blend: VfxBlend,
    pub atlas: AtlasSpec,
    pub sprite_forward: SpriteForward,
    pub source_system: String,
    pub source_emitter: String,
    pub source_blend: Option<u8>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::client) struct MeshBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
impl MeshBounds {
    pub fn center(self) -> Vec3 {
        (Vec3::from_array(self.min) + Vec3::from_array(self.max)) * 0.5
    }
    pub fn size(self) -> Vec3 {
        Vec3::from_array(self.max) - Vec3::from_array(self.min)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::client) struct MeshSpec {
    pub key: MeshKey,
    pub path: String,
    pub texture: TextureKey,
    pub source_bounds: MeshBounds,
    pub bounds: MeshBounds,
    pub source_to_world: f32,
    /// Row-major source-axis to imported-axis rotation; scale is separate.
    pub source_to_game: [[f32; 3]; 3],
    pub forward: Option<[f32; 3]>,
    pub up: [f32; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    format: u8,
    textures: Vec<TextureSpec>,
    meshes: Vec<MeshSpec>,
}
impl Catalog {
    fn validate(&self) -> Result<(), String> {
        if self.format != 1 {
            return Err("unsupported checked-in VFX catalog format".into());
        }
        let texture_keys: HashSet<_> = self.textures.iter().map(|spec| spec.key).collect();
        let mesh_keys: HashSet<_> = self.meshes.iter().map(|spec| spec.key).collect();
        if self.textures.len() != TextureKey::ALL.len()
            || texture_keys.len() != TextureKey::ALL.len()
            || !TextureKey::ALL.iter().all(|key| texture_keys.contains(key))
            || self.meshes.len() != MeshKey::ALL.len()
            || mesh_keys.len() != MeshKey::ALL.len()
            || !MeshKey::ALL.iter().all(|key| mesh_keys.contains(key))
        {
            return Err("VFX catalog keys must be unique and complete".into());
        }
        for spec in &self.textures {
            let atlas = spec.atlas;
            if !local_path(&spec.path, "vfx/textures/", ".png")
                || spec.size.contains(&0)
                || atlas.columns == 0
                || atlas.rows == 0
                || atlas.frames == 0
                || atlas.frames as u16 > atlas.columns as u16 * atlas.rows as u16
                || atlas.default_frame >= atlas.frames
                || !spec.size[0].is_multiple_of(atlas.columns as u32)
                || !spec.size[1].is_multiple_of(atlas.rows as u32)
                || spec.source_system.trim().is_empty()
                || spec.source_emitter.trim().is_empty()
            {
                return Err(format!(
                    "invalid texture/grid/source metadata for {:?}",
                    spec.key
                ));
            }
            if let Some(mode) = spec.source_blend
                && ![1, 3, 4, 5].contains(&mode)
            {
                return Err(format!("unreviewed source blend for {:?}", spec.key));
            }
        }
        for spec in &self.meshes {
            if !local_path(&spec.path, "vfx/meshes/", ".glb")
                || !texture_keys.contains(&spec.texture)
                || !valid_bounds(spec.source_bounds)
                || !valid_bounds(spec.bounds)
                || !spec.source_to_world.is_finite()
                || spec.source_to_world <= 0.0
            {
                return Err(format!(
                    "invalid mesh/bounds/scale metadata for {:?}",
                    spec.key
                ));
            }
            let axes = spec.source_to_game.map(Vec3::from_array);
            let up = Vec3::from_array(spec.up);
            if !axes.iter().all(|axis| unit_axis(*axis))
                || axes[0].dot(axes[1]).abs() > 1e-5
                || axes[0].dot(axes[2]).abs() > 1e-5
                || axes[1].dot(axes[2]).abs() > 1e-5
                || (axes[0].dot(axes[1].cross(axes[2])) - 1.0).abs() > 1e-5
                || !unit_axis(up)
                || spec.forward.is_some_and(|forward| {
                    let forward = Vec3::from_array(forward);
                    !unit_axis(forward) || forward.dot(up).abs() > 1e-5
                })
            {
                return Err(format!(
                    "mesh axes must be finite orthonormal rotations: {:?}",
                    spec.key
                ));
            }
            let (mut min, mut max) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
            for corner in 0..8 {
                let point = Vec3::new(
                    if corner & 1 == 0 {
                        spec.source_bounds.min[0]
                    } else {
                        spec.source_bounds.max[0]
                    },
                    if corner & 2 == 0 {
                        spec.source_bounds.min[1]
                    } else {
                        spec.source_bounds.max[1]
                    },
                    if corner & 4 == 0 {
                        spec.source_bounds.min[2]
                    } else {
                        spec.source_bounds.max[2]
                    },
                );
                let converted =
                    Vec3::new(axes[0].dot(point), axes[1].dot(point), axes[2].dot(point))
                        * spec.source_to_world;
                min = min.min(converted);
                max = max.max(converted);
            }
            if !Vec3::from_array(spec.bounds.min)
                .cmpge(min - Vec3::splat(1e-4))
                .all()
                || !Vec3::from_array(spec.bounds.max)
                    .cmple(max + Vec3::splat(1e-4))
                    .all()
            {
                return Err(format!(
                    "imported bounds exceed the converted source for {:?}",
                    spec.key
                ));
            }
        }
        Ok(())
    }
}

fn local_path(path: &str, prefix: &str, suffix: &str) -> bool {
    path.starts_with(prefix)
        && path.ends_with(suffix)
        && !path.contains("..")
        && !path.contains("://")
}
fn valid_bounds(bounds: MeshBounds) -> bool {
    let min = Vec3::from_array(bounds.min);
    let max = Vec3::from_array(bounds.max);
    min.is_finite() && max.is_finite() && min.cmple(max).all() && (max - min).max_element() > 0.0
}
fn unit_axis(axis: Vec3) -> bool {
    axis.is_finite() && (axis.length_squared() - 1.0).abs() < 1e-5
}
fn parse_catalog(source: &str) -> Result<Catalog, String> {
    let catalog: Catalog = serde_json::from_str(source).map_err(|error| error.to_string())?;
    catalog.validate()?;
    Ok(catalog)
}

#[cfg(test)]
pub(super) fn validate_json(source: &str) -> Result<(), String> {
    parse_catalog(source).map(|_| ())
}

static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    // This is compiled-in, hash-verified importer output. The packaged-assets
    // regression checks its complete schema and keys before a build is shipped.
    parse_catalog(include_str!("../../../assets/vfx/catalog.json"))
        .expect("checked-in VFX catalog must satisfy its source/grid/axis contracts")
});

pub(in crate::client) fn texture_spec(key: TextureKey) -> &'static TextureSpec {
    CATALOG
        .textures
        .iter()
        .find(|spec| spec.key == key)
        .expect("every TextureKey has a packaged specification")
}
pub(in crate::client) fn mesh_spec(key: MeshKey) -> &'static MeshSpec {
    CATALOG
        .meshes
        .iter()
        .find(|spec| spec.key == key)
        .expect("every MeshKey has a packaged specification")
}
