//! A separate, render-only temporary sister rig. Source and clocks come from simulation.
use super::{
    animation::RigPose,
    skeleton::AnimationLibrary,
    weapons::{self, WeaponAssets},
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
struct SisterPart {
    root: Entity,
    from: usize,
    to: usize,
    width: f32,
    head: bool,
}
#[derive(Component)]
struct SisterWeapon {
    root: Entity,
    pair: WeaponPair,
}

pub(super) fn setup(world: &mut World) {
    let actors: Vec<_> = world
        .query::<(Entity, &Companion)>()
        .iter(world)
        .map(|(performer, c)| (performer, c.owner))
        .collect();
    let assets = WeaponAssets::shared(world);
    for (performer, owner) in actors {
        let rig = RigPose::new(world.resource::<AnimationLibrary>());
        let root = world
            .spawn((
                CompanionView { owner, performer },
                rig,
                Transform::IDENTITY,
                Visibility::Hidden,
            ))
            .id();
        for (from, to, width, head) in [
            ("pelvis", "spine_03", 0.28, false),
            ("neck_01", "Head", 0.23, true),
            ("upperarm_l", "lowerarm_l", 0.10, false),
            ("lowerarm_l", "hand_l", 0.08, false),
            ("upperarm_r", "lowerarm_r", 0.10, false),
            ("lowerarm_r", "hand_r", 0.08, false),
            ("thigh_l", "calf_l", 0.13, false),
            ("calf_l", "foot_l", 0.10, false),
            ("thigh_r", "calf_r", 0.13, false),
            ("calf_r", "foot_r", 0.10, false),
        ] {
            let library = world.resource::<AnimationLibrary>();
            let from = library.bone(from);
            let to = library.bone(to);
            let part = weapons::part(
                world,
                root,
                if head { &assets.sphere } else { &assets.cube },
                if head { &assets.skin } else { &assets.cloth },
                Transform::IDENTITY,
            );
            world.entity_mut(part).insert(SisterPart {
                root,
                from,
                to,
                width,
                head,
            });
        }
        // Dark hair cap and twin tails give her a distinct silhouette from either brother appearance.
        for (position, size) in [
            (Vec3::new(0., 1.64, 0.045), Vec3::new(0.29, 0.20, 0.24)),
            (Vec3::new(-0.17, 1.48, 0.08), Vec3::new(0.09, 0.36, 0.10)),
            (Vec3::new(0.17, 1.48, 0.08), Vec3::new(0.09, 0.36, 0.10)),
        ] {
            weapons::part(
                world,
                root,
                &assets.sphere,
                &assets.hair,
                Transform::from_translation(position).with_scale(size),
            );
        }
        weapons::part(
            world,
            root,
            &assets.cube,
            &assets.gold,
            Transform::from_xyz(0., 0.94, 0.).with_scale(Vec3::new(0.31, 0.07, 0.19)),
        );
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
        let offset = state.aim_position - state.source_position;
        let pitch = offset.y.atan2(offset.xz().length().max(0.001));
        world.resource_scope(|world, library: Mut<AnimationLibrary>| {
            world.get_mut::<RigPose>(root).unwrap().sample_companion(
                &library,
                &state,
                pitch,
                root_transform
                    .compute_affine()
                    .inverse()
                    .transform_point3(state.source_position + (position - state.position)),
            );
        });
        let model = world.get::<RigPose>(root).unwrap().model.clone();
        let parts: Vec<_> = world
            .query::<(Entity, &SisterPart)>()
            .iter(world)
            .filter(|(_, p)| p.root == root)
            .map(|(e, p)| (e, p.from, p.to, p.width, p.head))
            .collect();
        for (entity, from, to, width, head) in parts {
            let a = model[from].translation;
            let b = model[to].translation;
            let transform = if head {
                Transform::from_translation(b + Vec3::Y * 0.055).with_scale(Vec3::new(
                    width,
                    width * 1.15,
                    width,
                ))
            } else {
                Transform::from_translation((a + b) * 0.5)
                    .with_rotation(Quat::from_rotation_arc(
                        Vec3::Y,
                        (b - a).normalize_or(Vec3::Y),
                    ))
                    .with_scale(Vec3::new(width, a.distance(b).max(0.08), width * 0.8))
            };
            *world.get_mut::<Transform>(entity).unwrap() = transform;
        }
        let source = state.source_position + (position - state.position);
        let direction =
            (state.aim_position - source).normalize_or(root_transform.rotation * Vec3::NEG_Z);
        let local_rotation =
            root_transform.rotation.inverse() * Quat::from_rotation_arc(Vec3::NEG_Z, direction);
        let local_source = root_transform
            .compute_affine()
            .inverse()
            .transform_point3(source);
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
            *world.get_mut::<Transform>(entity).unwrap() =
                Transform::from_translation(local_source).with_rotation(local_rotation);
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
                .is_none()
        );
        let counts = (
            world.entities().len(),
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
                    world.entities().len(),
                    world.resource::<Assets<Mesh>>().len(),
                    world.resource::<Assets<StandardMaterial>>().len()
                ),
                counts
            );
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
            app.world().entities().len(),
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
                    app.world().entities().len(),
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
