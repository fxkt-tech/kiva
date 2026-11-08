//! Model choice, picking bounds and label anchor share one presentation profile.
use crate::{gameplay::UnitRole, simulation::*};
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub(super) struct UnitAppearance {
    pub kind: ModelKind,
    pub label_offset: Vec3,
    pub pick_center: Vec3,
    pub pick_half_size: Vec3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ModelKind {
    Hero,
    MeleeMinion,
    RangedMinion,
    Base,
}

impl UnitAppearance {
    pub fn new(role: Option<UnitRole>, radius: f32) -> Self {
        let (kind, height, label_height) = match role {
            Some(UnitRole::Base) => (ModelKind::Base, 2.8, 3.2),
            Some(UnitRole::MeleeMinion) => (ModelKind::MeleeMinion, 1.2, 1.45),
            Some(UnitRole::RangedMinion) => (ModelKind::RangedMinion, 1.35, 1.6),
            // Raw simulation actors have no mode identity, but can still be
            // displayed by the reusable presentation tests and tools.
            Some(UnitRole::Hero) | None => (ModelKind::Hero, 2.0, 2.2),
        };
        Self {
            kind,
            label_offset: Vec3::Y * label_height,
            pick_center: Vec3::Y * (height * 0.5),
            pick_half_size: Vec3::new(radius + 0.1, height * 0.5, radius + 0.1),
        }
    }

    pub fn visible(self, alive: bool) -> Visibility {
        if alive || self.kind == ModelKind::Base {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    }
}

#[derive(Resource)]
pub(super) struct UnitModelAssets {
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    cylinder: Handle<Mesh>,
    platform: Handle<Mesh>,
    crystal: Handle<Mesh>,
    dark: Handle<StandardMaterial>,
    metal: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
}

pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(UnitModelAssets {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        sphere: meshes.add(Sphere::new(1.0).mesh().uv(10, 8)),
        cylinder: meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(8)),
        platform: meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(12)),
        crystal: meshes.add(Sphere::new(1.0).mesh().ico(0).expect("crystal mesh")),
        dark: materials.add(Color::srgb_u8(25, 37, 39)),
        metal: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(194, 218, 216),
            metallic: 0.65,
            perceptual_roughness: 0.4,
            ..default()
        }),
        gold: materials.add(StandardMaterial {
            base_color: Color::srgb_u8(215, 179, 106),
            metallic: 0.5,
            ..default()
        }),
    });
}

#[derive(Component)]
pub(super) struct MinionModel {
    owner: Entity,
}

#[derive(Component)]
pub(super) struct BaseCore {
    owner: Entity,
}

pub(super) fn spawn_simple_model(
    commands: &mut Commands,
    owner: Entity,
    appearance: UnitAppearance,
    assets: &UnitModelAssets,
    material: Handle<StandardMaterial>,
) {
    commands.entity(owner).with_children(|unit| {
        if appearance.kind == ModelKind::Base {
            unit.spawn((
                Mesh3d(assets.platform.clone()),
                MeshMaterial3d(assets.dark.clone()),
                Transform::from_xyz(0.0, 0.15, 0.0).with_scale(Vec3::new(1.9, 0.3, 1.9)),
                super::picking::PickGeometry,
            ));
            unit.spawn((
                Mesh3d(assets.platform.clone()),
                MeshMaterial3d(assets.gold.clone()),
                Transform::from_xyz(0.0, 0.34, 0.0).with_scale(Vec3::new(1.4, 0.1, 1.4)),
                super::picking::PickGeometry,
            ));
            unit.spawn((
                BaseCore { owner },
                super::picking::PickGeometry,
                Visibility::Inherited,
                Mesh3d(assets.crystal.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, 1.45, 0.0).with_scale(Vec3::new(0.85, 1.3, 0.85)),
            ));
            return;
        }
        let ranged = appearance.kind == ModelKind::RangedMinion;
        unit.spawn((
            MinionModel { owner },
            super::picking::PickGeometry,
            Transform::default(),
            Visibility::Inherited,
        ))
        .with_children(|model| {
            for (mesh, material, translation, scale) in [
                (
                    assets.cube.clone(),
                    material.clone(),
                    Vec3::new(0.0, 0.57, 0.0),
                    Vec3::new(0.38, 0.42, 0.26),
                ),
                (
                    assets.sphere.clone(),
                    material.clone(),
                    Vec3::new(0.0, 0.96, 0.0),
                    Vec3::splat(0.18),
                ),
                (
                    assets.cube.clone(),
                    assets.dark.clone(),
                    Vec3::new(-0.11, 0.23, 0.0),
                    Vec3::new(0.13, 0.4, 0.15),
                ),
                (
                    assets.cube.clone(),
                    assets.dark.clone(),
                    Vec3::new(0.11, 0.23, 0.0),
                    Vec3::new(0.13, 0.4, 0.15),
                ),
            ] {
                model.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(translation).with_scale(scale),
                ));
            }
            if ranged {
                model.spawn((
                    Mesh3d(assets.cylinder.clone()),
                    MeshMaterial3d(assets.dark.clone()),
                    Transform::from_xyz(0.3, 0.65, -0.1).with_scale(Vec3::new(0.045, 1.0, 0.045)),
                ));
                model.spawn((
                    Mesh3d(assets.crystal.clone()),
                    MeshMaterial3d(assets.gold.clone()),
                    Transform::from_xyz(0.3, 1.2, -0.1).with_scale(Vec3::splat(0.13)),
                ));
            } else {
                model.spawn((
                    Mesh3d(assets.cube.clone()),
                    MeshMaterial3d(assets.metal.clone()),
                    Transform::from_xyz(0.3, 0.67, -0.2)
                        .with_rotation(Quat::from_rotation_x(-0.4))
                        .with_scale(Vec3::new(0.08, 0.65, 0.04)),
                ));
                model.spawn((
                    Mesh3d(assets.cube.clone()),
                    MeshMaterial3d(assets.gold.clone()),
                    Transform::from_xyz(-0.27, 0.61, -0.1).with_scale(Vec3::new(0.13, 0.4, 0.32)),
                ));
            }
        });
    });
}

pub(super) fn sync_simple_models(
    frame: Res<super::presentation::PresentationFrame>,
    tick: Res<SimulationTick>,
    units: Query<(&UnitPose, &UnitAction, &UnitState)>,
    mut minions: Query<(&MinionModel, &mut Transform)>,
    mut cores: Query<(&BaseCore, &mut Visibility)>,
) {
    if !frame.should_sync() {
        return;
    }
    let seconds = (tick.0 as f32 + frame.fraction()) * TICK_SECONDS;
    for (model, mut transform) in &mut minions {
        let Ok((pose, action, _)) = units.get(model.owner) else {
            continue;
        };
        let moving = pose.position().distance_squared(pose.interpolated(0.0)) > 1e-8;
        transform.translation.y = if moving {
            (seconds * 14.0).sin().abs() * 0.06
        } else {
            0.0
        };
        transform.rotation = match action {
            UnitAction::AttackWindup { .. } => Quat::from_rotation_x(0.15),
            UnitAction::AttackBackswing {
                remaining,
                duration,
            } => {
                let weight = ((*remaining as f32 - frame.fraction()) / (*duration).max(1) as f32)
                    .clamp(0.0, 1.0);
                Quat::from_rotation_x(-0.3 * weight)
            }
            _ => Quat::IDENTITY,
        };
    }
    for (core, mut visibility) in &mut cores {
        if let Ok((_, _, state)) = units.get(core.owner) {
            *visibility = if state.alive() {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
