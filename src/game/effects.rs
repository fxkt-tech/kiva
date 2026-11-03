use super::{
    camera::OrbitCamera,
    combat::{Action, Fighter, Impacts, Side},
    input::Controls,
    player::{Appearance, SwordTip},
};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct EffectAssets {
    cube: Handle<Mesh>,
    spark: Handle<StandardMaterial>,
    block: Handle<StandardMaterial>,
    trail: Handle<StandardMaterial>,
}
#[derive(Component)]
pub(super) struct Fleeting {
    remaining: f32,
    total: f32,
    velocity: Vec3,
    base_scale: Vec3,
}
#[derive(Component)]
pub(super) struct DamageNumber {
    position: Vec3,
    remaining: f32,
}

pub(super) fn setup_effects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut material = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    };
    commands.insert_resource(EffectAssets {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        spark: material(Color::srgb_u8(240, 226, 165)),
        block: material(Color::srgb_u8(255, 171, 55)),
        trail: material(Color::srgb_u8(142, 227, 218)),
    });
}

pub(super) fn impact_effects(
    mut commands: Commands,
    impacts: Res<Impacts>,
    assets: Res<EffectAssets>,
    mut orbit: ResMut<OrbitCamera>,
) {
    for impact in &impacts.0 {
        orbit.shake = (orbit.shake + if impact.heavy { 0.045 } else { 0.022 }).min(0.07);
        for index in 0..9 {
            let angle = index as f32 * 2.399;
            let velocity =
                Vec3::new(angle.cos(), 0.6 + (index % 3) as f32 * 0.45, angle.sin()) * 1.7;
            let scale = Vec3::splat(if index % 2 == 0 { 0.045 } else { 0.025 });
            commands.spawn((
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(if impact.blocked {
                    assets.block.clone()
                } else {
                    assets.spark.clone()
                }),
                Transform::from_translation(impact.position).with_scale(scale),
                Fleeting {
                    remaining: 0.25,
                    total: 0.25,
                    velocity,
                    base_scale: scale,
                },
            ));
        }
        let color = if impact.blocked {
            Color::srgb_u8(255, 192, 76)
        } else if impact.target_side == Side::Player {
            Color::srgb_u8(255, 112, 105)
        } else {
            Color::srgb_u8(169, 243, 226)
        };
        commands.spawn((
            DamageNumber {
                position: impact.position + Vec3::Y * 0.5,
                remaining: 0.85,
            },
            Text::new(format!(
                "{}{:.0}",
                if impact.blocked { "BLOCK " } else { "-" },
                impact.damage
            )),
            TextFont::from_font_size(26.0),
            TextColor(color),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
        ));
    }
}

pub(super) fn update_effects(
    mut commands: Commands,
    time: Res<Time>,
    controls: Res<Controls>,
    mut particles: Query<(Entity, &mut Fleeting, &mut Transform)>,
    fighters: Query<(&Fighter, &Appearance)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut particle, mut transform) in &mut particles {
        if controls.reset {
            commands.entity(entity).despawn();
            continue;
        }
        if !controls.active {
            continue;
        }
        particle.remaining -= time.delta_secs();
        if particle.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += particle.velocity * time.delta_secs();
        particle.velocity.y -= 3.0 * time.delta_secs();
        transform.scale = particle.base_scale * (particle.remaining / particle.total).sqrt();
    }
    for (fighter, appearance) in &fighters {
        if let Some(mut material) = materials.get_mut(&appearance.material) {
            material.base_color = appearance
                .color
                .mix(&Color::WHITE, (fighter.flash / 0.18).clamp(0.0, 1.0) * 0.9);
        }
    }
}

pub(super) fn sword_trails(
    mut commands: Commands,
    controls: Res<Controls>,
    assets: Res<EffectAssets>,
    fighters: Query<&Fighter>,
    mut tips: Query<(&mut SwordTip, &GlobalTransform)>,
) {
    for (mut tip, transform) in &mut tips {
        let position = transform.translation();
        let active = fighters.get(tip.owner).is_ok_and(|fighter|
            matches!(&fighter.action, Action::Attack(attack) if attack.phase() == "STRIKE"));
        if !controls.active || !active || controls.reset {
            tip.previous = None;
            continue;
        }
        if let Some(previous) = tip.previous {
            let length = previous.distance(position);
            if length > 0.015 && length < 2.5 {
                let scale = Vec3::new(0.045, 0.025, length);
                commands.spawn((
                    Mesh3d(assets.cube.clone()),
                    MeshMaterial3d(assets.trail.clone()),
                    Transform::from_translation((previous + position) * 0.5)
                        .looking_at(position, Vec3::Y)
                        .with_scale(scale),
                    Fleeting {
                        remaining: 0.12,
                        total: 0.12,
                        velocity: Vec3::ZERO,
                        base_scale: scale,
                    },
                ));
            }
        }
        tip.previous = Some(position);
    }
}

pub(super) fn damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    controls: Res<Controls>,
    camera: Single<(&Camera, &GlobalTransform), With<super::camera::GameCamera>>,
    mut numbers: Query<(Entity, &mut DamageNumber, &mut Node, &mut TextColor)>,
) {
    for (entity, mut number, mut node, mut color) in &mut numbers {
        if controls.reset {
            commands.entity(entity).despawn();
            continue;
        }
        if controls.active {
            number.remaining -= time.delta_secs();
        }
        if number.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let position = number.position + Vec3::Y * (0.85 - number.remaining) * 0.7;
        if let Ok(screen) = camera.0.world_to_viewport(camera.1, position) {
            node.left = px(screen.x - 22.0);
            node.top = px(screen.y);
            color.0.set_alpha((number.remaining / 0.25).clamp(0.0, 1.0));
        } else {
            color.0.set_alpha(0.0);
        }
    }
}
