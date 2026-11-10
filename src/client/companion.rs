//! Bai Lingmiao's independent render rig. Source and clocks come from simulation.
use super::{
    animation::RigPose,
    avatar::{self, Appearance, AvatarAssetCache, AvatarAssets},
    skeleton::AnimationLibrary,
    weapon_models::{self, Mount, WeaponModel},
    weapons,
};
use crate::simulation::{
    Companion, CompanionState, HitStop, SimulationActive, WeaponMode, WeaponPair,
};
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct CompanionView {
    pub owner: Entity,
    pub performer: Entity,
}
#[derive(Component)]
struct SisterBodyMount {
    root: Entity,
    pair: WeaponPair,
}
#[derive(Component)]
pub(super) struct SisterWeapon {
    pub root: Entity,
    pub pair: WeaponPair,
}

/// The actual muzzle/arrow-source frame used by both the live companion and
/// pose inspection. A ranged model's origin is not its right-hand grip.
pub(super) struct CompanionFrame {
    pub(super) weapon: Transform,
    pub(super) pitch: f32,
    pub(super) grips: [Vec3; 2],
}

pub(super) fn local_frame(
    root: Transform,
    source: Vec3,
    aim: Vec3,
    model_grips: [Vec3; 2],
) -> CompanionFrame {
    let direction = (aim - source).normalize_or(root.rotation * Vec3::NEG_Z);
    let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, root.rotation.inverse() * direction);
    let position = root.compute_affine().inverse().transform_point3(source);
    CompanionFrame {
        weapon: Transform::from_translation(position).with_rotation(rotation),
        pitch: direction.y.atan2(direction.xz().length().max(0.001)),
        grips: model_grips.map(|grip| position + rotation * grip),
    }
}

pub(super) fn setup(world: &mut World) {
    let actors: Vec<_> = world
        .query::<(Entity, &Companion)>()
        .iter(world)
        .map(|(performer, c)| (performer, c.owner))
        .collect();
    for (performer, owner) in actors {
        if world
            .query::<&CompanionView>()
            .iter(world)
            .any(|view| view.performer == performer)
        {
            continue;
        }
        let rig = RigPose::new(world.resource::<AnimationLibrary>());
        let root = world
            .spawn((
                CompanionView { owner, performer },
                rig,
                Transform::IDENTITY,
                Visibility::Hidden,
            ))
            .id();
        avatar::spawn_with_appearance(world, root, Appearance::Bailingmiao);
        let body_mount = world
            .spawn((
                SisterBodyMount {
                    root,
                    pair: WeaponPair::ChainAxeBow,
                },
                Transform::IDENTITY,
                Visibility::Hidden,
            ))
            .id();
        weapon_models::spawn(
            world,
            body_mount,
            WeaponPair::ChainAxeBow,
            WeaponMode::Ranged,
            Mount::Spine,
        );
        world.entity_mut(root).add_child(body_mount);
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            let weapon = world
                .spawn((
                    SisterWeapon { root, pair },
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ))
                .id();
            weapons::spawn(world, weapon, pair, WeaponMode::Ranged);
            world.entity_mut(root).add_child(weapon);
        }
    }
}

pub(super) fn sync(world: &mut World) {
    let alpha = super::display_alpha(
        world.resource::<Time<Fixed>>(),
        world.resource::<SimulationActive>(),
        world.resource::<HitStop>(),
    );
    let roots: Vec<_> = world
        .query::<(Entity, &CompanionView)>()
        .iter(world)
        .map(|(e, v)| (e, v.performer, v.owner))
        .collect();
    for (root, performer, owner) in roots {
        if world
            .get::<Companion>(performer)
            .is_none_or(|companion| companion.owner != owner)
        {
            world.despawn(root);
            continue;
        }
        let Some(state) = world.get::<CompanionState>(performer).copied() else {
            world.despawn(root);
            continue;
        };
        *world.get_mut::<Visibility>(root).unwrap() = if state.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if !state.visible {
            continue;
        }
        let position = state.previous_position.lerp(state.position, alpha);
        let yaw = (-state.facing.x).atan2(-state.facing.y);
        let root_transform =
            Transform::from_translation(position).with_rotation(Quat::from_rotation_y(yaw));
        *world.get_mut::<Transform>(root).unwrap() = root_transform;
        let source = state.source_position + (position - state.position);
        let frame = local_frame(
            root_transform,
            source,
            state.aim_position,
            WeaponModel::shared(world, state.pair, WeaponMode::Ranged).grips,
        );
        world.resource_scope(|world, library: Mut<AnimationLibrary>| {
            world.get_mut::<RigPose>(root).unwrap().sample_companion(
                &library,
                &state,
                frame.pitch,
                frame.grips,
            );
        });
        let assets = AvatarAssets::cached(
            world.get_resource::<AvatarAssets>(),
            world.get_resource::<AvatarAssetCache>(),
            Appearance::Bailingmiao,
        )
        .expect("companion skin is cached");
        let body_pose = assets.body_mount(world.get::<RigPose>(root).unwrap());
        let mounts: Vec<_> = world
            .query::<(Entity, &SisterBodyMount)>()
            .iter(world)
            .filter(|(_, mount)| mount.root == root)
            .map(|(entity, mount)| (entity, mount.pair))
            .collect();
        for (entity, pair) in mounts {
            *world.get_mut::<Transform>(entity).unwrap() = body_pose;
            *world.get_mut::<Visibility>(entity).unwrap() = if pair == state.pair {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        let weapons: Vec<_> = world
            .query::<(Entity, &SisterWeapon)>()
            .iter(world)
            .filter(|(_, w)| w.root == root)
            .map(|(e, w)| (e, w.pair))
            .collect();
        for (entity, pair) in weapons {
            *world.get_mut::<Visibility>(entity).unwrap() = if pair == state.pair {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            *world.get_mut::<Transform>(entity).unwrap() = frame.weapon;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sister_is_one_independent_rig_and_reuses_all_assets() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<AnimationLibrary>();
        world.init_resource::<Time<Fixed>>();
        world.insert_resource(SimulationActive(false));
        world.init_resource::<HitStop>();
        let owner = world.spawn_empty().id();
        let performer = world
            .spawn((Companion { owner }, CompanionState::default()))
            .id();
        setup(&mut world);
        let (root, view) = world
            .query::<(Entity, &CompanionView)>()
            .single(&world)
            .unwrap();
        assert_eq!(view.performer, performer);
        assert_eq!(view.owner, owner);
        assert_ne!(root, owner);
        assert!(world.get::<crate::simulation::Body>(root).is_none());
        assert!(
            world
                .get::<super::super::avatar::AvatarAppearance>(root)
                .is_some_and(|appearance| appearance.0 == Appearance::Bailingmiao)
        );
        let counts = (
            world.entities().count_spawned(),
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<StandardMaterial>>().len(),
        );
        for pair in [
            WeaponPair::HammerMusket,
            WeaponPair::ChainAxeBow,
            WeaponPair::HammerMusket,
        ] {
            let mut s = world.get_mut::<CompanionState>(performer).unwrap();
            s.visible = true;
            s.pair = pair;
            s.source_position = Vec3::new(0., 1.2, -0.55);
            s.aim_position = Vec3::new(0., 2., -4.);
            sync(&mut world);
            assert_eq!(
                (
                    world.entities().count_spawned(),
                    world.resource::<Assets<Mesh>>().len(),
                    world.resource::<Assets<StandardMaterial>>().len()
                ),
                counts
            );
        }
        // Reconciliation can revisit startup data without adding a second skin
        // or weapon tree. Removing the logical performer removes its whole view.
        setup(&mut world);
        assert_eq!(world.entities().count_spawned(), counts.0);
        let mut hierarchy = vec![root];
        let mut cursor = 0;
        while cursor < hierarchy.len() {
            if let Some(children) = world.get::<Children>(hierarchy[cursor]) {
                hierarchy.extend(children.iter());
            }
            cursor += 1;
        }
        world.despawn(performer);
        sync(&mut world);
        assert!(world.get_entity(root).is_err());
        assert!(
            hierarchy
                .iter()
                .all(|&entity| world.get_entity(entity).is_err())
        );
        assert_eq!(
            world.entities().count_spawned(),
            counts.0 - hierarchy.len() as u32 - 1
        );
        assert_eq!(world.resource::<Assets<Mesh>>().len(), counts.1);
        assert_eq!(world.resource::<Assets<StandardMaterial>>().len(), counts.2);
    }
    #[test]
    fn both_source_weapons_keep_their_up_axis_at_every_heading_and_pitch() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<AnimationLibrary>();
        world.init_resource::<Time<Fixed>>();
        world.insert_resource(SimulationActive(false));
        world.init_resource::<HitStop>();
        let owner = world.spawn_empty().id();
        let performer = world
            .spawn((Companion { owner }, CompanionState::default()))
            .id();
        setup(&mut world);
        let root = world
            .query_filtered::<Entity, With<CompanionView>>()
            .single(&world)
            .unwrap();
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            for yaw in [
                0.,
                std::f32::consts::FRAC_PI_2,
                -std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
            ] {
                for pitch in [-0.65, 0.0, 0.65, 1.0] {
                    let yaw_rotation = Quat::from_rotation_y(yaw);
                    let expected_rotation = yaw_rotation * Quat::from_rotation_x(pitch);
                    let position = Vec3::new(2., 0.7, -3.);
                    let source = position + yaw_rotation * Vec3::new(0., 1.2, -0.55);
                    *world.get_mut::<CompanionState>(performer).unwrap() = CompanionState {
                        visible: true,
                        pair,
                        position,
                        previous_position: position,
                        source_position: source,
                        aim_position: source + expected_rotation * Vec3::NEG_Z * 5.,
                        facing: (yaw_rotation * Vec3::NEG_Z).xz(),
                        ..default()
                    };
                    sync(&mut world);
                    let root_pose = *world.get::<Transform>(root).unwrap();
                    let weapon = *world
                        .query::<(&SisterWeapon, &Transform)>()
                        .iter(&world)
                        .find(|(weapon, _)| weapon.root == root && weapon.pair == pair)
                        .unwrap()
                        .1;
                    let actual_rotation = root_pose.rotation * weapon.rotation;
                    assert!(
                        root_pose
                            .transform_point(weapon.translation)
                            .distance(source)
                            < 0.0001
                    );
                    assert!(
                        (actual_rotation * Vec3::NEG_Z).distance(expected_rotation * Vec3::NEG_Z)
                            < 0.0001
                    );
                    assert!(
                        (actual_rotation * Vec3::Y).distance(expected_rotation * Vec3::Y) < 0.0001,
                        "{pair:?} yaw={yaw} pitch={pitch} must stay upright"
                    );
                    assert!(
                        (actual_rotation * Vec3::X).distance(expected_rotation * Vec3::X) < 0.0001
                    );
                    let frame = local_frame(
                        root_pose,
                        source,
                        source + expected_rotation * Vec3::NEG_Z * 5.0,
                        WeaponModel::shared(&mut world, pair, WeaponMode::Ranged).grips,
                    );
                    assert_eq!(frame.weapon, weapon);
                    let mut inspected = RigPose::new(world.resource::<AnimationLibrary>());
                    inspected.sample_companion_hold(
                        world.resource::<AnimationLibrary>(),
                        0.0,
                        frame.pitch,
                        frame.grips,
                    );
                    let actual = world.get::<RigPose>(root).unwrap();
                    assert_eq!(inspected.model, actual.model);
                    assert_eq!(inspected.companion_hands, actual.companion_hands);
                }
            }
        }
    }

    #[test]
    fn real_q_has_one_sister_exact_muzzle_pause_and_roster_cleanup_without_asset_growth() {
        use crate::{
            client::{animation, combat, controls::ViewYaw, scene},
            gameplay::Player,
            simulation::{CombatEffect, CombatFeedback, CombatIntent, SimulationPlugin},
        };
        use std::time::Duration;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<AnimationLibrary>()
            .init_resource::<ViewYaw>()
            .add_systems(
                Startup,
                (crate::gameplay::setup, scene::setup, combat::setup, setup).chain(),
            )
            .add_systems(
                Update,
                (scene::sync_player, animation::animate, sync).chain(),
            );
        app.finish();
        app.update();
        let tick = |app: &mut App, n| {
            for _ in 0..n {
                app.world_mut()
                    .resource_mut::<Time<Fixed>>()
                    .advance_by(Duration::from_secs_f64(1. / 60.));
                app.world_mut().run_schedule(FixedUpdate);
            }
        };
        tick(&mut app, 12);
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        tick(&mut app, 1);
        let counts = (
            app.world().entities().count_spawned(),
            app.world().resource::<Assets<Mesh>>().len(),
            app.world().resource::<Assets<StandardMaterial>>().len(),
        );
        for _ in 0..2 {
            app.world_mut()
                .get_mut::<CombatIntent>(player)
                .unwrap()
                .request_ranged_skill();
            tick(&mut app, 14);
            app.world_mut().run_schedule(Update);
            let world = app.world_mut();
            let (root, view, transform, visibility) = world
                .query::<(Entity, &CompanionView, &Transform, &Visibility)>()
                .single(world)
                .unwrap();
            assert_eq!(view.owner, player);
            assert_eq!(*visibility, Visibility::Visible);
            let performer = view.performer;
            let transform = *transform;
            let weapon = world
                .query::<(&SisterWeapon, &Transform)>()
                .iter(world)
                .find(|(w, _)| w.root == root && w.pair == WeaponPair::HammerMusket)
                .unwrap()
                .1;
            let muzzle = transform.transform_point(weapon.translation);
            let event = world
                .resource::<CombatFeedback>()
                .events()
                .filter(|e| e.kind == CombatEffect::Shot)
                .last()
                .unwrap();
            assert_eq!(event.actor, performer);
            assert_eq!(event.source_owner, player);
            assert!(event.position.distance(muzzle) < 0.0001);
            let rig = world.get::<RigPose>(root).unwrap().model.clone();
            world.resource_mut::<SimulationActive>().0 = false;
            tick(&mut app, 60);
            app.world_mut().run_schedule(Update);
            assert_eq!(app.world().get::<RigPose>(root).unwrap().model, rig);
            assert_eq!(*app.world().get::<Transform>(root).unwrap(), transform);
            app.world_mut().resource_mut::<SimulationActive>().0 = true;
            tick(&mut app, 270);
            app.world_mut().run_schedule(Update);
            assert_eq!(
                *app.world().get::<Visibility>(root).unwrap(),
                Visibility::Hidden
            );
            assert_eq!(
                (
                    app.world().entities().count_spawned(),
                    app.world().resource::<Assets<Mesh>>().len(),
                    app.world().resource::<Assets<StandardMaterial>>().len()
                ),
                counts
            );
        }
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_ranged_skill();
        tick(&mut app, 3);
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        crate::gameplay::set_enemy_mode(app.world_mut(), crate::gameplay::EnemyMode::Combat);
        app.world_mut().run_schedule(Update);
        let visibility = app
            .world_mut()
            .query_filtered::<&Visibility, With<CompanionView>>()
            .single(app.world())
            .unwrap();
        assert_eq!(*visibility, Visibility::Hidden);
    }
}
