//! Source weapon instances and read-only chain-axe presentation.
use crate::simulation::{WeaponMode, WeaponPair};
use bevy::prelude::*;

#[derive(Resource, Clone)]
struct TetherAssets {
    cube: Handle<Mesh>,
    gold: Handle<StandardMaterial>,
}
impl TetherAssets {
    fn shared(world: &mut World) -> Self {
        if let Some(assets) = world.get_resource::<Self>() {
            return assets.clone();
        }
        let cube = world.resource_mut::<Assets<Mesh>>().add(Cuboid::default());
        let gold = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.66, 0.18),
                perceptual_roughness: 0.65,
                ..default()
            });
        let assets = Self { cube, gold };
        world.insert_resource(assets.clone());
        assets
    }
}

pub(super) fn melee_name(pair: WeaponPair) -> &'static str {
    super::loadout::of(pair).melee_name
}

/// Source melee extends +Y from the grip; ranged points -Z from the firing source.
pub(super) fn spawn(world: &mut World, root: Entity, pair: WeaponPair, mode: WeaponMode) {
    super::weapon_models::spawn(world, root, pair, mode, super::weapon_models::Mount::Weapon);
}

#[derive(Component)]
pub(super) struct FlyingAxe;
#[derive(Component)]
pub(super) struct Tether;
/// Persistent views; fixed-step snapshots alone determine throw and return progress.
pub(super) fn setup_tether(world: &mut World) {
    let assets = TetherAssets::shared(world);
    let axe = world
        .spawn((FlyingAxe, Transform::IDENTITY, Visibility::Hidden))
        .id();
    // The chained melee weapon is the one which flies.
    let tethered = WeaponPair::ALL
        .into_iter()
        .find(|pair| super::loadout::of(*pair).tethered)
        .expect("one weapon pair is thrown on a chain");
    spawn(world, axe, tethered, WeaponMode::Melee);
    world.spawn((
        Tether,
        Mesh3d(assets.cube),
        MeshMaterial3d(assets.gold),
        Transform::IDENTITY,
        Visibility::Hidden,
    ));
}
type TetherData = (
    &'static mut Transform,
    &'static mut Visibility,
    Has<FlyingAxe>,
);
type TetherViews = Or<(With<FlyingAxe>, With<Tether>)>;
pub(super) fn sync_tether(world: &mut World) {
    use crate::{
        client::{
            PlayerView,
            stickman::{Attachment, BoneVisual},
        },
        gameplay::Player,
        simulation::{Combatant, HitStop, Projectile, SimulationActive},
    };
    let time = world.resource::<Time<Fixed>>();
    let dt = time.delta_secs();
    let alpha = super::display_alpha(
        time,
        world.resource::<SimulationActive>(),
        world.resource::<HitStop>(),
    );
    let combat = world
        .query_filtered::<&Combatant, With<Player>>()
        .single(world)
        .expect("one player");
    // The chain of a pull holds the axe in the target until the closing return.
    let pull = combat
        .active_action()
        .filter(|a| super::loadout::of(a.pair).tethered && combat.axe_away());
    // A thrown axe is a projectile: out to what it hits, then back to the hand.
    let thrown = world
        .query::<&Projectile>()
        .iter(world)
        .find(|projectile| projectile.is_thrown())
        .map(|projectile| {
            (
                projectile.interpolated(alpha),
                (projectile.age() - dt * (1. - alpha)).max(0.),
            )
        });
    let (view, root) = world
        .query_filtered::<(Entity, &Transform), With<PlayerView>>()
        .single(world)
        .map(|(view, transform)| (view, *transform))
        .expect("one player view");
    // Attachment transforms have just been updated from this frame's retargeted wrist.
    // Compose the local grip directly: GlobalTransform propagation runs after Update.
    let hand = world
        .query::<(&BoneVisual, &Transform)>()
        .iter(world)
        .find(|(visual, _)| {
            visual.owner == view
                && matches!(
                    visual.attachment,
                    Attachment::Weapon {
                        mode: WeaponMode::Melee,
                        ..
                    }
                )
        })
        .map(|(_, transform)| root.transform_point(transform.translation))
        .expect("player has a melee wrist attachment");
    let mut views = world.query_filtered::<TetherData, TetherViews>();
    for (mut transform, mut visibility, is_axe) in views.iter_mut(world) {
        // The axe spins on rule time, so it freezes with the simulation.
        let (point, spin) = if let Some((point, age)) = thrown {
            (point, age * 20.)
        } else if let Some(a) = pull
            && let Some((start, duration)) = a.kind.return_window()
        {
            let elapsed = (a.elapsed - dt * (1. - alpha)).max(0.);
            (
                a.aim_position
                    .lerp(hand, ((elapsed - start) / duration).clamp(0., 1.)),
                elapsed * 20.,
            )
        } else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *visibility = Visibility::Visible;
        if is_axe {
            *transform =
                Transform::from_translation(point).with_rotation(Quat::from_rotation_z(spin));
        } else {
            *transform = Transform::from_translation((point + hand) * 0.5)
                .looking_at(point, Vec3::Y)
                .with_scale(Vec3::new(0.018, 0.018, point.distance(hand).max(0.01)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{
            ActorView, PlayerView, animation,
            avatar::{self, Appearance, AvatarAssetCache, AvatarAssets},
            combat::{self, WeaponVisual},
            controls::ViewYaw,
            scene,
            skeleton::AnimationLibrary,
            stickman::{self, Attachment, BoneVisual},
        },
        gameplay::{EnemyMode, Player, set_enemy_mode},
        simulation::{
            ActionKind, Body, CombatIntent, Combatant, Health, HitStop, MoveIntent,
            SimulationActive, SimulationPlugin, teleport_actor,
        },
    };
    use std::time::Duration;

    fn scene() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<AnimationLibrary>()
            .init_resource::<ViewYaw>()
            .add_systems(
                Startup,
                (
                    crate::gameplay::setup,
                    crate::gameplay::manual_targeting,
                    scene::setup,
                    combat::setup,
                    setup_tether,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    scene::sync_player,
                    combat::sync_weapons,
                    animation::animate,
                    avatar::sync,
                    stickman::sync_attachments,
                    sync_tether,
                )
                    .chain(),
            );
        app.finish();
        app.update();
        tick(&mut app, 12);
        let world = app.world_mut();
        let player = world
            .query_filtered::<Entity, With<Player>>()
            .single(world)
            .unwrap();
        let view = world
            .query_filtered::<Entity, With<PlayerView>>()
            .single(world)
            .unwrap();
        (app, player, view)
    }
    fn tick(app: &mut App, n: usize) {
        for _ in 0..n {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(Duration::from_secs_f64(1. / 60.));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }
    fn render(world: &mut World) {
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f64(1. / 60.));
        world.run_schedule(Update);
    }
    fn held(world: &mut World, pair: WeaponPair) -> Entity {
        world
            .query::<(Entity, &WeaponVisual)>()
            .iter(world)
            .find(|(_, w)| w.pair() == pair)
            .unwrap()
            .0
    }
    fn handles(world: &World, root: Entity) -> Vec<(Handle<Mesh>, Handle<StandardMaterial>)> {
        world
            .get::<Children>(root)
            .unwrap()
            .iter()
            .map(|child| {
                (
                    world.get::<Mesh3d>(child).unwrap().0.clone(),
                    world
                        .get::<MeshMaterial3d<StandardMaterial>>(child)
                        .unwrap()
                        .0
                        .clone(),
                )
            })
            .collect()
    }
    fn asset_counts(world: &World) -> (usize, usize, usize) {
        (
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<StandardMaterial>>().len(),
            world.resource::<Assets<Image>>().len(),
        )
    }
    fn throw(app: &mut App, player: Entity) {
        let needs_target = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .target()
            .is_none();
        {
            let mut intent = app.world_mut().get_mut::<CombatIntent>(player).unwrap();
            if needs_target {
                // The only target is the real roster actor, selected by the normal intent.
                intent.request_toggle_target();
            }
            intent.request_pair(WeaponPair::ChainAxeBow);
        }
        tick(app, 1);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_ranged_skill();
        // Until the axe has left the hand; a close target sends it straight back.
        for _ in 0..13 {
            tick(app, 1);
            if app.world().get::<Combatant>(player).unwrap().axe_away() {
                break;
            }
        }
        assert!(app.world().get::<Combatant>(player).unwrap().axe_away());
        render(app.world_mut());
    }
    fn assert_tether_at_grip(world: &mut World, view: Entity) {
        let root = *world.get::<Transform>(view).unwrap();
        let weapon = held(world, WeaponPair::ChainAxeBow);
        let grip = root.transform_point(world.get::<Transform>(weapon).unwrap().translation);
        let line = *world
            .query_filtered::<&Transform, With<Tether>>()
            .single(world)
            .unwrap();
        let start = line.transform_point(Vec3::Z * 0.5);
        let end = line.transform_point(Vec3::NEG_Z * 0.5);
        let axe = *world
            .query_filtered::<&Transform, With<FlyingAxe>>()
            .single(world)
            .unwrap();
        assert!(
            start.distance(grip) < 0.0001,
            "rope starts on the rendered wrist grip"
        );
        assert!(
            end.distance(axe.translation) < 0.0001,
            "rope ends at the same flying grip origin"
        );
    }
    fn assert_returned(world: &mut World, player: Entity) {
        assert!(!world.get::<Combatant>(player).unwrap().axe_away());
        for visibility in world
            .query_filtered::<&Visibility, TetherViews>()
            .iter(world)
        {
            assert_eq!(*visibility, Visibility::Hidden);
        }
        let weapon = held(world, WeaponPair::ChainAxeBow);
        assert_eq!(
            *world.get::<Visibility>(weapon).unwrap(),
            Visibility::Visible
        );
    }

    #[test]
    fn source_melee_uses_the_retargeted_wrist_for_both_bodies_and_shared_flying_meshes() {
        let (mut app, player, view) = scene();
        let world = app.world_mut();
        let axe = world
            .query_filtered::<Entity, With<FlyingAxe>>()
            .single(world)
            .unwrap();
        let chain = held(world, WeaponPair::ChainAxeBow);
        let hammer = held(world, WeaponPair::HammerMusket);
        let chain_handles = handles(world, chain);
        let hammer_handles = handles(world, hammer);
        assert_eq!(chain_handles, handles(world, axe));
        assert_ne!(chain_handles, hammer_handles);
        let counts = asset_counts(world);
        for appearance in [Appearance::Choulingke, Appearance::Lihuowang] {
            let world = app.world_mut();
            let position = world.get::<Body>(player).unwrap().world_position();
            avatar::replace(world, view, appearance);
            render(world);
            assert_eq!(handles(world, chain), chain_handles);
            assert_eq!(handles(world, hammer), hammer_handles);
            let rig = world.get::<animation::RigPose>(view).unwrap();
            let assets = AvatarAssets::cached(
                world.get_resource::<AvatarAssets>(),
                world.get_resource::<AvatarAssetCache>(),
                appearance,
            )
            .unwrap();
            let expected = stickman::attachment_transform(
                &world.get::<BoneVisual>(chain).unwrap().attachment,
                &assets.attachment_pose(rig),
            );
            assert_eq!(*world.get::<Transform>(chain).unwrap(), expected);
            let bone = world.resource::<AnimationLibrary>().bone("hand_r");
            let pose = assets.attachment_pose(rig);
            let wrist_forward = pose[bone].rotation * Vec3::Z;
            assert!((expected.rotation * Vec3::Y).distance(wrist_forward) < 0.0001);
            assert_eq!(
                world.get::<Body>(player).unwrap().world_position(),
                position
            );
            assert_eq!(world.get::<ActorView>(view).unwrap().owner, player);
            let Attachment::Weapon { mode, .. } =
                world.get::<BoneVisual>(chain).unwrap().attachment;
            assert_eq!(mode, WeaponMode::Melee);
        }
        assert_eq!(asset_counts(app.world()), counts);
    }

    #[test]
    fn real_throw_pause_resume_and_return_keep_one_axe_and_reuse_assets() {
        let (mut app, player, view) = scene();
        let counts = asset_counts(app.world());
        for appearance in [Appearance::Choulingke, Appearance::Lihuowang] {
            avatar::replace(app.world_mut(), view, appearance);
            throw(&mut app, player);
            let world = app.world_mut();
            let weapon = held(world, WeaponPair::ChainAxeBow);
            assert_eq!(
                *world.get::<Visibility>(weapon).unwrap(),
                Visibility::Hidden
            );
            assert_tether_at_grip(world, view);
            // Entering pause chooses current snapshots; repeated zero/multiple-step frames stay still.
            world.resource_mut::<SimulationActive>().0 = false;
            render(world);
            let position = world.get::<Body>(player).unwrap().world_position();
            let previous_rig = world.get::<animation::RigPose>(view).unwrap().model.clone();
            let previous_handles = handles(world, weapon);
            let replacement = if appearance == Appearance::Choulingke {
                Appearance::Lihuowang
            } else {
                Appearance::Choulingke
            };
            avatar::replace(world, view, replacement);
            render(world);
            assert_eq!(
                world.get::<Body>(player).unwrap().world_position(),
                position
            );
            assert_eq!(
                world.get::<animation::RigPose>(view).unwrap().model,
                previous_rig
            );
            assert_eq!(handles(world, weapon), previous_handles);
            assert_eq!(
                *world.get::<Visibility>(weapon).unwrap(),
                Visibility::Hidden
            );
            assert_tether_at_grip(world, view);
            let entities = world.entities().count_spawned();
            let pose = world.get::<animation::RigPose>(view).unwrap().model.clone();
            let axe = *world
                .query_filtered::<&Transform, With<FlyingAxe>>()
                .single(world)
                .unwrap();
            let action_time = world
                .get::<Combatant>(player)
                .unwrap()
                .active_action()
                .unwrap()
                .elapsed;
            tick(&mut app, 60);
            for _ in 0..3 {
                render(app.world_mut());
            }
            let world = app.world_mut();
            assert_eq!(
                *world
                    .query_filtered::<&Transform, With<FlyingAxe>>()
                    .single(world)
                    .unwrap(),
                axe
            );
            assert_eq!(world.get::<animation::RigPose>(view).unwrap().model, pose);
            assert_eq!(
                world
                    .get::<Combatant>(player)
                    .unwrap()
                    .active_action()
                    .unwrap()
                    .elapsed,
                action_time
            );
            assert_tether_at_grip(world, view);
            world.resource_mut::<SimulationActive>().0 = true;
            tick(&mut app, 50);
            render(app.world_mut());
            assert_returned(app.world_mut(), player);
            assert_eq!(asset_counts(app.world()), counts);
            // The axe in flight was one simulation projectile; it is gone
            // once the axe is back, and nothing else was left behind.
            assert_eq!(app.world().entities().count_spawned(), entities - 1);
            assert!(
                app.world_mut()
                    .query::<&crate::simulation::Projectile>()
                    .iter(app.world())
                    .next()
                    .is_none()
            );
            tick(&mut app, 260);
        }
    }

    #[test]
    fn chain_pull_roster_cancellation_and_death_clear_source_axe_views() {
        let (mut app, player, view) = scene();
        throw(&mut app, player);
        tick(&mut app, 7);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_combo();
        tick(&mut app, 12);
        assert_eq!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .active_action()
                .unwrap()
                .kind,
            ActionKind::ChainCombo
        );
        render(app.world_mut());
        assert_tether_at_grip(app.world_mut(), view);
        tick(&mut app, 33);
        render(app.world_mut());
        assert_tether_at_grip(app.world_mut(), view);
        tick(&mut app, 20);
        render(app.world_mut());
        assert_returned(app.world_mut(), player);
        tick(&mut app, 260);
        throw(&mut app, player);
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        set_enemy_mode(app.world_mut(), EnemyMode::Combat);
        render(app.world_mut());
        assert_returned(app.world_mut(), player);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        set_enemy_mode(app.world_mut(), EnemyMode::Target);
        tick(&mut app, 280);
        throw(&mut app, player);
        teleport_actor(app.world_mut(), player, Vec3::new(0., -20., 0.));
        tick(&mut app, 1);
        render(app.world_mut());
        assert!(!app.world().get::<Health>(player).unwrap().is_alive());
        assert_returned(app.world_mut(), player);
    }

    #[test]
    fn moving_player_tether_reads_this_frame_grip_after_zero_and_multiple_fixed_steps() {
        let (mut app, player, view) = scene();
        app.world_mut()
            .get_mut::<MoveIntent>(player)
            .unwrap()
            .set(Vec2::X);
        throw(&mut app, player);
        let root_before = *app.world().get::<Transform>(view).unwrap();
        let position = app.world().get::<Body>(player).unwrap().world_position();
        let action_time = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .active_action()
            .unwrap()
            .elapsed;
        render(app.world_mut());
        assert_eq!(
            app.world().get::<Transform>(view).unwrap().translation,
            root_before.translation
        );
        assert_eq!(
            app.world().get::<Body>(player).unwrap().world_position(),
            position
        );
        assert_eq!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .active_action()
                .unwrap()
                .elapsed,
            action_time
        );
        assert_tether_at_grip(app.world_mut(), view);
        tick(&mut app, 3);
        render(app.world_mut());
        assert!(
            app.world()
                .get::<Transform>(view)
                .unwrap()
                .translation
                .distance(root_before.translation)
                > 0.05
        );
        assert_tether_at_grip(app.world_mut(), view);
    }

    #[test]
    fn actual_sniper_hit_stop_freezes_the_held_source_hammer_grip_and_resumes() {
        let (mut app, player, view) = scene();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        tick(&mut app, 1);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        tick(&mut app, 13);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_combo();
        for _ in 0..40 {
            tick(&mut app, 1);
            render(app.world_mut());
            if app.world().resource::<HitStop>().is_active() {
                break;
            }
        }
        assert!(app.world().resource::<HitStop>().is_active());
        let world = app.world_mut();
        let hammer = held(world, WeaponPair::HammerMusket);
        let grip = *world.get::<Transform>(hammer).unwrap();
        let mesh_handles = handles(world, hammer);
        let pose = world.get::<animation::RigPose>(view).unwrap().model.clone();
        let position = world.get::<Body>(player).unwrap().world_position();
        let elapsed = world
            .get::<Combatant>(player)
            .unwrap()
            .active_action()
            .unwrap()
            .elapsed;
        for _ in 0..3 {
            tick(&mut app, 1);
            render(app.world_mut());
            let world = app.world_mut();
            assert!(world.resource::<HitStop>().is_active());
            assert_eq!(*world.get::<Transform>(hammer).unwrap(), grip);
            assert_eq!(world.get::<animation::RigPose>(view).unwrap().model, pose);
            assert_eq!(
                world.get::<Body>(player).unwrap().world_position(),
                position
            );
            assert_eq!(handles(world, hammer), mesh_handles);
        }
        tick(&mut app, 1);
        render(app.world_mut());
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .active_action()
                .unwrap()
                .elapsed
                > elapsed
        );
    }
}
