//! Local, reviewed visual assets shared by skill effects and aiming indicators.
mod catalog;
mod loading;

pub(super) use catalog::{MeshKey, SpriteForward, TextureKey, mesh_spec, texture_spec};
pub(super) use loading::{SkillAssets, SkillAssetsPlugin, SkillAssetsSet};

#[cfg(test)]
mod tests;
