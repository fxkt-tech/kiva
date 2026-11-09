//! Read-only combat presentation. Meshes and materials are shared, effects are transient.
use bevy::prelude::*;

use crate::{
    gameplay::Player,
    simulation::{
        Body, CombatEffect, CombatFeedback, Combatant, Enemy, Facing, Health, MotionState,
        SimulationActive, WeaponMode, WeaponSet,
    },
};

#[derive(Resource)]
pub(super) struct VisualAssets {
    cube: Handle<Mesh>,
    ring: Handle<Mesh>,
    sphere: Handle<Mesh>,
    enemy: Handle<StandardMaterial>,
    dark: Handle<StandardMaterial>,
    metal: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    teal: Handle<StandardMaterial>,
    hit: Handle<StandardMaterial>,
}

#[derive(Component)]
pub(super) struct WeaponVisual(WeaponSet, WeaponMode);
#[derive(Component)]
pub(super) struct EnemyBody(Entity);
#[derive(Component)]
pub(super) struct EnemyHealthBar(Entity);
#[derive(Component)]
pub(super) enum Marker {
    Target,
    Aim,
    Guard,
}
#[derive(Component)]
pub(super) struct Effect {
    remaining: f32,
    duration: f32,
    expansion: f32,
    scale: Vec3,
}
#[derive(Resource, Default)]
pub(super) struct EffectCursor(u64);

pub(super) fn setup(world: &mut World) {
    let (cube, ring, sphere) = {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        (
            meshes.add(Cuboid::default()),
            meshes.add(Torus::new(0.94, 1.0).mesh().minor_resolution(6)),
            meshes.add(Sphere::new(0.5).mesh().uv(12, 8)),
        )
    };
    let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
    let mut material = |color, unlit| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit,
            perceptual_roughness: 0.65,
            ..default()
        })
    };
    let assets = VisualAssets {
        cube,
        ring,
        sphere,
        enemy: material(Color::srgb(0.72, 0.30, 0.22), false),
        dark: material(Color::srgb(0.08, 0.12, 0.15), false),
        metal: material(Color::srgb(0.61, 0.67, 0.66), false),
        gold: material(Color::srgb(1.0, 0.70, 0.23), true),
        teal: material(Color::srgb(0.35, 0.89, 0.83), true),
        hit: material(Color::srgb(1.0, 0.93, 0.75), true),
    };
    let enemies: Vec<_> = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect();
    for enemy in enemies {
        world
            .entity_mut(enemy)
            .insert((Transform::default(), Visibility::default()));
        for part in super::stickman::spawn(world, enemy, &assets.enemy, &assets.gold) {
            world.entity_mut(part).insert(EnemyBody(enemy));
        }
        let bar = mesh(world, &assets.cube, &assets.teal, Transform::default());
        world.entity_mut(bar).insert(EnemyHealthBar(enemy));
    }

    let player = world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .expect("one player");
    for (set, mode) in [
        (WeaponSet::HammerSniper, WeaponMode::Melee),
        (WeaponSet::HammerSniper, WeaponMode::Ranged),
        (WeaponSet::AxeCannon, WeaponMode::Melee),
        (WeaponSet::AxeCannon, WeaponMode::Ranged),
    ] {
        let root = world
            .spawn((
                WeaponVisual(set, mode),
                Transform::from_xyz(0.40, 1.0, -0.30),
                Visibility::Hidden,
            ))
            .id();
        let parts: &[(Vec3, Vec3, bool)] = match (set, mode) {
            (WeaponSet::HammerSniper, WeaponMode::Melee) => &[
                (Vec3::ZERO, Vec3::new(0.08, 0.8, 0.08), false),
                (Vec3::new(0.0, 0.38, 0.0), Vec3::new(0.65, 0.30, 0.32), true),
            ],
            (WeaponSet::AxeCannon, WeaponMode::Melee) => &[
                (Vec3::ZERO, Vec3::new(0.08, 0.8, 0.08), false),
                (
                    Vec3::new(-0.18, 0.28, 0.0),
                    Vec3::new(0.46, 0.38, 0.08),
                    true,
                ),
            ],
            (WeaponSet::HammerSniper, WeaponMode::Ranged) => &[
                (Vec3::ZERO, Vec3::new(0.14, 0.16, 0.85), false),
                (
                    Vec3::new(0.0, 0.0, -0.63),
                    Vec3::new(0.06, 0.06, 0.50),
                    true,
                ),
                (
                    Vec3::new(0.0, 0.13, -0.10),
                    Vec3::new(0.10, 0.10, 0.25),
                    true,
                ),
            ],
            (WeaponSet::AxeCannon, WeaponMode::Ranged) => &[
                (Vec3::ZERO, Vec3::new(0.28, 0.28, 0.75), false),
                (
                    Vec3::new(0.0, 0.0, -0.40),
                    Vec3::new(0.38, 0.38, 0.18),
                    true,
                ),
            ],
        };
        for &(position, size, accent) in parts {
            let child = mesh(
                world,
                &assets.cube,
                if accent { &assets.metal } else { &assets.dark },
                Transform::from_translation(position).with_scale(size),
            );
            world.entity_mut(root).add_child(child);
        }
        world.entity_mut(player).add_child(root);
        super::stickman::attach_weapon(world, player, root, mode);
    }
    for (marker, color, size) in [
        (Marker::Target, &assets.gold, 0.65),
        (Marker::Aim, &assets.teal, 0.45),
        (Marker::Guard, &assets.teal, 0.7),
    ] {
        let entity = mesh(
            world,
            &assets.ring,
            color,
            Transform::from_scale(Vec3::splat(size)),
        );
        world
            .entity_mut(entity)
            .insert((marker, Visibility::Hidden));
    }
    world.insert_resource(assets);
    world.init_resource::<EffectCursor>();
}

pub(super) fn sync_enemies(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    assets: Res<VisualAssets>,
    mut enemies: Query<(
        &Body,
        &MotionState,
        &Facing,
        &Health,
        &Enemy,
        &mut Transform,
        &mut Visibility,
    )>,
    mut parts: Query<(&EnemyBody, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let alpha = super::display_alpha(&time, &active);
    for (body, motion, facing, _, _, mut transform, mut visibility) in &mut enemies {
        let position = body.interpolated(alpha);
        transform.translation =
            Vec3::new(position.x, motion.interpolated_height(alpha), position.y);
        let direction = facing.direction();
        transform.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.y));
        // Defeated actors remain visible so their imported death pose can play.
        // Health bars and selection still use authoritative health separately.
        *visibility = Visibility::Visible;
    }
    for (part, mut material) in &mut parts {
        if let Ok((_, _, _, health, enemy, _, _)) = enemies.get(part.0) {
            material.0 = if health.hit_remaining() > 0.0 {
                assets.hit.clone()
            } else if enemy.is_winding_up() {
                assets.gold.clone()
            } else {
                assets.enemy.clone()
            };
        }
    }
}

pub(super) fn sync_health_bars(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    view: Res<super::controls::ViewYaw>,
    enemies: Query<(&Body, &MotionState, &Health), With<Enemy>>,
    mut bars: Query<(&EnemyHealthBar, &mut Transform, &mut Visibility)>,
) {
    let alpha = super::display_alpha(&time, &active);
    for (bar, mut transform, mut visibility) in &mut bars {
        let Ok((body, motion, health)) = enemies.get(bar.0) else {
            continue;
        };
        let position = body.interpolated(alpha);
        transform.translation = Vec3::new(
            position.x,
            motion.interpolated_height(alpha) + 2.0,
            position.y,
        );
        transform.scale = Vec3::new(0.7 * health.current() / health.maximum(), 0.035, 0.06);
        transform.rotation = Quat::from_rotation_y(view.yaw());
        *visibility = if health.is_alive() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn sync_weapons(
    player: Single<&Combatant, With<Player>>,
    mut weapons: Query<(&WeaponVisual, &mut Visibility)>,
) {
    for (weapon, mut visibility) in &mut weapons {
        *visibility = if weapon.0 == player.weapon_set() && weapon.1 == player.weapon_mode() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn sync_markers(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    player: Single<(&Body, &MotionState, &Combatant, &Health), With<Player>>,
    targets: Query<(&Body, &Health), With<Enemy>>,
    mut markers: Query<(&Marker, &mut Transform, &mut Visibility)>,
) {
    let (body, motion, combat, health) = *player;
    let alpha = super::display_alpha(&time, &active);
    for (marker, mut transform, mut visibility) in &mut markers {
        let point = match marker {
            Marker::Target => combat
                .target()
                .and_then(|entity| targets.get(entity).ok())
                .filter(|(_, health)| health.is_alive())
                .map(|(body, _)| body.interpolated(alpha)),
            Marker::Aim
                if combat.weapon_set() == WeaponSet::AxeCannon
                    && combat.weapon_mode() == WeaponMode::Ranged =>
            {
                Some(combat.aim())
            }
            Marker::Guard if motion.is_blocking() => Some(body.interpolated(alpha)),
            _ => None,
        };
        if let Some(point) = point.filter(|_| health.is_alive()) {
            transform.translation = Vec3::new(point.x, 0.05, point.y);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(super) fn effects(
    mut commands: Commands,
    time: Res<Time>,
    feedback: Res<CombatFeedback>,
    assets: Res<VisualAssets>,
    mut cursor: ResMut<EffectCursor>,
    mut effects: Query<(Entity, &mut Effect, &mut Transform)>,
) {
    for (entity, mut effect, mut transform) in &mut effects {
        effect.remaining -= time.delta_secs();
        if effect.remaining <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            let progress = 1.0 - effect.remaining / effect.duration;
            transform.scale = effect.scale * (1.0 + effect.expansion * progress);
        }
    }
    for event in feedback.events().filter(|event| event.sequence > cursor.0) {
        let (mesh, material, transform, duration, expansion) = match event.kind {
            CombatEffect::Shot => {
                let length = event.position.distance(event.end).max(0.05);
                let mut transform = Transform::from_translation((event.position + event.end) * 0.5)
                    .with_scale(Vec3::new(0.035, 0.035, length));
                if length > 0.05 {
                    transform = transform.looking_at(event.end, Vec3::Y);
                }
                (&assets.cube, &assets.gold, transform, 0.16, 0.0)
            }
            CombatEffect::Hammer
            | CombatEffect::Cannon
            | CombatEffect::Spin
            | CombatEffect::Melee => (
                &assets.ring,
                if matches!(event.kind, CombatEffect::Cannon) {
                    &assets.teal
                } else {
                    &assets.gold
                },
                Transform::from_translation(if matches!(event.kind, CombatEffect::Spin) {
                    event.position
                } else {
                    Vec3::new(event.position.x, 0.06, event.position.z)
                })
                .with_scale(Vec3::splat(event.radius.max(0.4))),
                0.35,
                0.25,
            ),
            CombatEffect::EnemyWindup => (
                &assets.ring,
                &assets.gold,
                Transform::from_translation(Vec3::new(event.position.x, 0.04, event.position.z))
                    .with_scale(Vec3::splat(event.radius.max(0.6))),
                0.4,
                0.15,
            ),
            _ => (
                &assets.sphere,
                if matches!(
                    event.kind,
                    CombatEffect::Block | CombatEffect::Dodge | CombatEffect::Respawn
                ) {
                    &assets.teal
                } else {
                    &assets.hit
                },
                Transform::from_translation(event.position).with_scale(Vec3::splat(0.25)),
                0.22,
                1.5,
            ),
        };
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
            Effect {
                remaining: duration,
                duration,
                expansion,
                scale: transform.scale,
            },
        ));
    }
    if let Some(event) = feedback.events().last() {
        cursor.0 = event.sequence;
    }
}

fn mesh(
    world: &mut World,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    world
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{
            animation, camera, controls::ViewYaw, scene, skeleton::AnimationLibrary, stickman,
        },
        simulation::{CombatIntent, SimulationPlugin},
    };
    use std::time::Duration;

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<ViewYaw>()
            .init_resource::<AnimationLibrary>()
            .add_systems(
                Startup,
                (crate::gameplay::setup, scene::setup, setup, camera::setup).chain(),
            )
            .add_systems(
                Update,
                (
                    scene::sync_player,
                    sync_enemies,
                    sync_health_bars,
                    sync_weapons,
                    sync_markers,
                    effects,
                    animation::animate,
                    stickman::sync_attachments,
                    camera::follow,
                )
                    .chain(),
            );
        app.update();
        app
    }

    #[test]
    fn headless_scene_consumes_combat_feedback_and_reclaims_effects_without_new_assets() {
        let mut app = headless_app();
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        let material_count = app.world().resource::<Assets<StandardMaterial>>().len();
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        app.world_mut().run_schedule(FixedUpdate);
        app.world_mut().run_schedule(Update);
        assert!(
            app.world_mut()
                .query_filtered::<Entity, With<Effect>>()
                .iter(app.world())
                .count()
                > 0
        );
        let visible_weapons = app
            .world_mut()
            .query::<(&WeaponVisual, &Visibility)>()
            .iter(app.world())
            .filter(|(_, visibility)| **visibility == Visibility::Visible)
            .count();
        assert_eq!(visible_weapons, 1);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<EnemyHealthBar>>()
                .iter(app.world())
                .count(),
            4
        );

        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.world_mut().run_schedule(Update);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Effect>>()
                .iter(app.world())
                .count(),
            0
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        assert_eq!(
            app.world().resource::<Assets<StandardMaterial>>().len(),
            material_count
        );
    }

    #[test]
    fn defeated_enemy_keeps_its_body_for_the_death_clip_and_paused_rigs_stay_frozen() {
        let mut app = headless_app();
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        app.world_mut().run_schedule(FixedUpdate);
        let enemy = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .target()
            .unwrap();
        let head = app.world().resource::<AnimationLibrary>().bone("Head");
        let standing = app.world().get::<animation::RigPose>(enemy).unwrap().model[head]
            .translation
            .y;
        app.world_mut().entity_mut(enemy).insert(Health::new(1.0));
        {
            let mut intent = app.world_mut().get_mut::<CombatIntent>(player).unwrap();
            intent.request_switch_weapon();
            intent.request_attack();
        }
        app.world_mut().run_schedule(FixedUpdate);
        assert!(!app.world().get::<Health>(enemy).unwrap().is_alive());
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0));
        app.world_mut().run_schedule(Update);
        assert_eq!(
            *app.world().get::<Visibility>(enemy).unwrap(),
            Visibility::Visible
        );
        let fallen = app.world().get::<animation::RigPose>(enemy).unwrap().model[head]
            .translation
            .y;
        assert!(
            fallen < standing - 0.3,
            "standing {standing}, fallen {fallen}"
        );
        let frozen: Vec<_> = app
            .world_mut()
            .query::<(Entity, &stickman::BoneVisual, &Transform)>()
            .iter(app.world())
            .map(|(entity, _, transform)| (entity, *transform))
            .collect();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.5));
        app.world_mut().run_schedule(Update);
        for (entity, before) in frozen {
            let after = app.world().get::<Transform>(entity).unwrap();
            assert!(after.translation.distance(before.translation) < 0.0001);
            assert!(after.rotation.dot(before.rotation).abs() > 0.9999);
        }
    }
}
