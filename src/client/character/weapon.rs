//! Presentation identity selects geometry and gestures, never combat behavior.
use super::model::ModelAssets;
use crate::simulation::CharacterId;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::client) enum WeaponStyle {
    #[default]
    Sword,
    Spear,
    Bow,
}

impl WeaponStyle {
    pub fn for_character(character: Option<CharacterId>) -> Self {
        match character {
            Some(CharacterId(1)) => Self::Spear,
            Some(CharacterId(2)) => Self::Bow,
            _ => Self::Sword,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Sword => "Sword",
            Self::Spear => "Spear",
            Self::Bow => "Bow",
        }
    }
}

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, assets: &ModelAssets, style: WeaponStyle) {
    let mut part =
        |material: Handle<StandardMaterial>, position: Vec3, scale: Vec3, rotation: Quat| {
            parent.spawn((
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(position)
                    .with_scale(scale)
                    .with_rotation(rotation),
            ));
        };
    match style {
        WeaponStyle::Sword => {
            for (material, position, scale) in [
                (
                    assets.dark.clone(),
                    Vec3::new(0.0, 0.0, -0.035),
                    Vec3::new(0.065, 0.065, 0.22),
                ),
                (
                    assets.gold.clone(),
                    Vec3::new(0.0, 0.0, 0.09),
                    Vec3::new(0.29, 0.05, 0.055),
                ),
                (
                    assets.metal.clone(),
                    Vec3::new(0.0, 0.0, 0.56),
                    Vec3::new(0.07, 0.028, 0.91),
                ),
            ] {
                part(material, position, scale, Quat::IDENTITY);
            }
        }
        WeaponStyle::Spear => {
            part(
                assets.dark.clone(),
                Vec3::new(0.0, 0.0, 0.22),
                Vec3::new(0.065, 0.065, 1.65),
                Quat::IDENTITY,
            );
            part(
                assets.gold.clone(),
                Vec3::new(0.0, 0.0, 0.93),
                Vec3::new(0.18, 0.09, 0.14),
                Quat::IDENTITY,
            );
            part(
                assets.metal.clone(),
                Vec3::new(0.0, 0.0, 1.16),
                Vec3::new(0.13, 0.05, 0.42),
                Quat::from_rotation_z(0.4),
            );
            part(
                assets.gold.clone(),
                Vec3::new(0.0, 0.0, -0.63),
                Vec3::new(0.11, 0.1, 0.14),
                Quat::IDENTITY,
            );
        }
        WeaponStyle::Bow => {
            part(
                assets.gold.clone(),
                Vec3::ZERO,
                Vec3::new(0.065, 0.2, 0.065),
                Quat::IDENTITY,
            );
            for sign in [-1.0, 1.0] {
                part(
                    assets.dark.clone(),
                    Vec3::new(0.0, sign * 0.24, 0.08),
                    Vec3::new(0.055, 0.38, 0.055),
                    Quat::from_rotation_x(sign * 0.42),
                );
                part(
                    assets.gold.clone(),
                    Vec3::new(0.0, sign * 0.48, 0.04),
                    Vec3::new(0.045, 0.22, 0.045),
                    Quat::from_rotation_x(-sign * 0.65),
                );
            }
            part(
                assets.metal.clone(),
                Vec3::new(0.0, 0.0, -0.035),
                Vec3::new(0.012, 1.12, 0.012),
                Quat::IDENTITY,
            );
            part(
                assets.dark.clone(),
                Vec3::new(0.0, 0.0, 0.35),
                Vec3::new(0.025, 0.025, 0.8),
                Quat::IDENTITY,
            );
            part(
                assets.metal.clone(),
                Vec3::new(0.0, 0.0, 0.78),
                Vec3::new(0.075, 0.025, 0.15),
                Quat::IDENTITY,
            );
        }
    }
}
