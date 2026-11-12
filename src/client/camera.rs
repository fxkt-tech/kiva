use super::controls::ViewYaw;
use super::impact::ImpactState;
use super::loadout::ComboCamera;
use super::{ActorView, PlayerView};
use avian3d::prelude::{Collider, Sensor, ShapeCastConfig, SpatialQuery, SpatialQueryFilter};
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    gameplay::Player,
    simulation::{Body, Combatant, Health, HitStop, SequenceSnapshot, SimulationActive},
};

#[derive(Component)]
pub(super) struct GameCamera;

const FOV: f32 = 50.0_f32.to_radians();
const BODY_CENTER: Vec3 = Vec3::new(0.0, 0.9, 0.0);
const CAMERA_HEIGHT: f32 = 6.0;
const CAMERA_BACK: f32 = CAMERA_HEIGHT * 0.7;
const BODY_SCREEN_Y: f32 = 2.0 / 3.0;
const SNIPER_ENTER_DURATION: f32 = 0.20;
const SNIPER_HEIGHT: f32 = 0.70;
const SNIPER_BACK: f32 = 2.60;
const SNIPER_SHOULDER: f32 = 0.45;
const CAMERA_RADIUS: f32 = 0.15;
const CAMERA_CLEARANCE: f32 = 0.03;

type CameraExclusions = Or<(With<Body>, With<Sensor>)>;
type PlayerViewFilter = (With<PlayerView>, Without<GameCamera>);

#[derive(SystemParam)]
pub(super) struct CameraObstacles<'w, 's> {
    spatial: SpatialQuery<'w, 's>,
    excluded: Query<'w, 's, Entity, CameraExclusions>,
}

impl CameraObstacles<'_, '_> {
    fn clear_position(&self, origin: Vec3, desired: Vec3) -> Vec3 {
        let offset = desired - origin;
        let Ok(direction) = Dir3::new(offset) else {
            return desired;
        };
        let filter = SpatialQueryFilter::from_excluded_entities(self.excluded.iter());
        self.spatial
            .cast_shape(
                &Collider::sphere(CAMERA_RADIUS),
                origin,
                Quat::IDENTITY,
                direction,
                &ShapeCastConfig::from_max_distance(offset.length()),
                &filter,
            )
            .map_or(desired, |hit| {
                origin + direction * (hit.distance - CAMERA_CLEARANCE).max(0.0)
            })
    }
}

fn smooth(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn cinematic_weight(elapsed: f32, duration: f32) -> f32 {
    let return_start = duration - SequenceSnapshot::RETURN_DURATION;
    if elapsed < SNIPER_ENTER_DURATION {
        smooth(elapsed / SNIPER_ENTER_DURATION)
    } else if elapsed <= return_start {
        1.0
    } else {
        1.0 - smooth((elapsed - return_start) / SequenceSnapshot::RETURN_DURATION)
    }
}

fn sniper_pose(feet: Vec3, target: Vec3, yaw: f32) -> Transform {
    let fallback = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
    let forward = (target - feet).with_y(0.0).normalize_or(fallback);
    let right = forward.cross(Vec3::Y);
    Transform::from_translation(
        feet - forward * SNIPER_BACK + right * SNIPER_SHOULDER + Vec3::Y * SNIPER_HEIGHT,
    )
    .looking_at(target, Vec3::Y)
}

fn camera_pose(body_center: Vec3, yaw: f32) -> Transform {
    let heading = Quat::from_rotation_y(yaw);
    let forward = heading * Vec3::NEG_Z;
    // Convert the desired screen position into an angle below the optical axis.
    let body_view_angle = ((2.0 * BODY_SCREEN_Y - 1.0) * (FOV * 0.5).tan()).atan();
    let pitch = CAMERA_HEIGHT.atan2(CAMERA_BACK) - body_view_angle;
    Transform::from_translation(body_center - forward * CAMERA_BACK + Vec3::Y * CAMERA_HEIGHT)
        .with_rotation(heading * Quat::from_rotation_x(-pitch))
}

pub(super) fn setup(
    mut commands: Commands,
    yaw: Res<ViewYaw>,
    player: Single<&Transform, With<PlayerView>>,
) {
    commands.init_resource::<ImpactState>();
    commands.spawn((
        GameCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV,
            ..default()
        }),
        camera_pose(player.translation + BODY_CENTER, yaw.yaw()),
    ));
}

fn framing(sequence: &SequenceSnapshot) -> ComboCamera {
    super::loadout::of(sequence.pair).combo_camera
}

pub(super) fn follow(
    yaw: Res<ViewYaw>,
    timing: (
        Res<Time<Fixed>>,
        Res<SimulationActive>,
        Res<HitStop>,
        Res<ImpactState>,
    ),
    player: Single<(&Transform, &ActorView), PlayerViewFilter>,
    actors: Query<&Combatant, With<Player>>,
    targets: Query<(&Body, &Health)>,
    obstacles: CameraObstacles,
    mut camera: Single<&mut Transform, (With<GameCamera>, Without<PlayerView>)>,
) {
    let (fixed, active, hit_stop, impact) = timing;
    let (view, owner) = *player;
    let normal = camera_pose(view.translation + BODY_CENTER, yaw.yaw());
    let sequence = actors.get(owner.owner).ok().and_then(Combatant::sequence);
    let mut pose = normal;
    if let Some(sequence) = sequence {
        let alpha = super::display_alpha(&fixed, &active, &hit_stop);
        let elapsed = (sequence.elapsed() - fixed.delta_secs() * (1.0 - alpha)).max(0.0);
        let weight = cinematic_weight(elapsed, sequence.duration);
        if weight > 0.0 {
            let target = targets
                .get(sequence.target())
                .ok()
                .filter(|(_, health)| health.is_alive())
                .map_or_else(
                    || sequence.aim_position(),
                    |(body, _)| body.interpolated_world(alpha) + body.aim_offset(),
                );
            let rear = if framing(&sequence) == ComboCamera::EyeLevel {
                let mut pose = sniper_pose(view.translation, target, yaw.yaw());
                pose.translation.y = view.translation.y + 1.2;
                pose.look_at(target, Vec3::Y);
                pose
            } else {
                sniper_pose(view.translation, target, yaw.yaw())
            };
            let desired = normal.translation.lerp(rear.translation, weight);
            let position =
                obstacles.clear_position(view.translation + Vec3::Y * SNIPER_HEIGHT, desired);
            pose = Transform::from_translation(position)
                .with_rotation(normal.rotation.slerp(rear.rotation, weight));
            if position.distance_squared(desired) > 0.0001 {
                pose.look_at(target, Vec3::Y);
            }
        }
    }
    if sequence
        .is_some_and(|sequence| framing(&sequence) == ComboCamera::Sniper && !sequence.cancelled)
        && let Some((displacement, rotation)) = impact.recoil(owner.owner)
    {
        let desired = pose.translation + pose.rotation * displacement;
        pose.translation =
            obstacles.clear_position(view.translation + Vec3::Y * SNIPER_HEIGHT, desired);
        pose.rotation *= rotation;
    }
    **camera = pose;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::scene,
        gameplay,
        simulation::{CombatIntent, Health, SimulationPlugin, spawn_box, teleport_actor},
    };
    use bevy::camera::CameraProjection;
    use std::time::Duration;

    fn spawn_view(world: &mut World) {
        let owner = world
            .query_filtered::<Entity, With<Player>>()
            .single(world)
            .unwrap();
        world.spawn((PlayerView, ActorView { owner }, Transform::default()));
    }

    fn arena() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<ViewYaw>()
            .add_systems(
                Startup,
                (
                    gameplay::setup,
                    gameplay::manual_targeting,
                    spawn_view,
                    setup,
                )
                    .chain(),
            )
            .add_systems(Update, (scene::sync_player, follow).chain());
        app.finish();
        app.update();
        tick(&mut app, 12);
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        request(&mut app, player, CombatIntent::request_toggle_target);
        let target = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .target()
            .unwrap();
        (app, player, target)
    }

    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(Duration::from_secs_f64(1.0 / 60.0));
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn request(app: &mut App, player: Entity, command: impl FnOnce(&mut CombatIntent)) {
        command(&mut app.world_mut().get_mut::<CombatIntent>(player).unwrap());
        tick(app, 1);
    }

    fn start_sniping(app: &mut App, player: Entity) {
        request(app, player, CombatIntent::request_melee_skill);
        tick(app, 13);
        request(app, player, CombatIntent::request_combo);
        assert!(
            app.world()
                .get::<Combatant>(player)
                .unwrap()
                .is_combo_active()
        );
    }

    fn render(app: &mut App) -> Transform {
        app.world_mut().run_schedule(Update);
        *app.world_mut()
            .query_filtered::<&Transform, With<GameCamera>>()
            .single(app.world())
            .unwrap()
    }

    fn view_feet(app: &mut App) -> Vec3 {
        app.world_mut()
            .query_filtered::<&Transform, With<PlayerView>>()
            .single(app.world())
            .unwrap()
            .translation
    }

    fn assert_same_pose(actual: Transform, expected: Transform) {
        assert!(actual.translation.distance(expected.translation) < 0.0001);
        assert!(actual.rotation.dot(expected.rotation).abs() > 0.99999);
        assert!(actual.translation.is_finite() && actual.rotation.is_finite());
    }

    #[test]
    fn actual_perspective_places_the_body_center_at_the_lower_third_at_every_heading() {
        for aspect in [4.0 / 3.0, 16.0 / 9.0, 1.0] {
            let projection = PerspectiveProjection {
                fov: FOV,
                aspect_ratio: aspect,
                ..default()
            }
            .get_clip_from_view();
            for yaw in [0.0, -std::f32::consts::FRAC_PI_2, 2.4] {
                let center = Vec3::new(17.0, 0.9, -9.0);
                let camera = camera_pose(center, yaw);
                let view = Mat4::from_scale_rotation_translation(
                    camera.scale,
                    camera.rotation,
                    camera.translation,
                )
                .inverse();
                let clip = projection * view * center.extend(1.0);
                let ndc = clip.truncate() / clip.w;
                assert!(ndc.x.abs() < 0.0001);
                let screen_fraction = (1.0 - ndc.y) * 0.5;
                assert!(
                    (screen_fraction - 2.0 / 3.0).abs() < 0.0001,
                    "{screen_fraction}"
                );
            }
        }
    }

    #[test]
    fn committed_sniper_camera_reaches_low_rear_aim_before_damage_and_restores_top_down() {
        let (mut app, player, target) = arena();
        let normal = render(&mut app);
        start_sniping(&mut app, player);
        assert_same_pose(render(&mut app), normal);
        tick(&mut app, 14);
        let focused = render(&mut app);
        let feet = view_feet(&mut app);
        let target_body = app.world().get::<Body>(target).unwrap();
        let aim = target_body.interpolated_world(0.0) + target_body.aim_offset();
        let expected = sniper_pose(feet, aim, 0.0);
        assert_same_pose(focused, expected);
        assert!((focused.translation.y - feet.y - SNIPER_HEIGHT).abs() < 0.001);
        assert!((focused.translation - feet).dot(Vec3::NEG_Z) < -2.5);
        assert!((focused.rotation * Vec3::NEG_Z).y > 0.1);
        assert_eq!(
            app.world().get::<Health>(target).unwrap().current(),
            999_980.0
        );
        assert!(normal.translation.distance(focused.translation) > 6.0);

        tick(&mut app, 60);
        assert!(
            !app.world()
                .get::<Combatant>(player)
                .unwrap()
                .is_combo_active()
        );
        let restored = render(&mut app);
        assert_same_pose(
            restored,
            camera_pose(view_feet(&mut app) + BODY_CENTER, 0.0),
        );
        assert_eq!(
            app.world().get::<Health>(target).unwrap().current(),
            999_896.0
        );
    }

    #[test]
    fn cinematic_uses_actual_elevation_and_pause_freezes_its_pose_without_physical_writes() {
        let (mut app, player, target) = arena();
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 0.75, -2.0),
            Vec3::new(9.0, 1.5, 9.0),
            false,
        );
        teleport_actor(app.world_mut(), player, Vec3::new(0.0, 1.5, 0.0));
        teleport_actor(app.world_mut(), target, Vec3::new(0.0, 1.5, -4.0));
        tick(&mut app, 12);
        start_sniping(&mut app, player);
        tick(&mut app, 14);
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        let player_position = app.world().get::<Body>(player).unwrap().world_position();
        let target_position = app.world().get::<Body>(target).unwrap().world_position();
        let focused = render(&mut app);
        assert!((focused.translation.y - player_position.y - SNIPER_HEIGHT).abs() < 0.001);
        assert!(focused.translation.y > 2.0);
        for _ in 0..3 {
            tick(&mut app, 60);
            assert_same_pose(render(&mut app), focused);
        }
        assert_eq!(
            app.world().get::<Body>(player).unwrap().world_position(),
            player_position
        );
        assert_eq!(
            app.world().get::<Body>(target).unwrap().world_position(),
            target_position
        );
        assert_eq!(
            app.world().get::<Health>(target).unwrap().current(),
            999_980.0
        );
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        tick(&mut app, 60);
        let restored = render(&mut app);
        assert_same_pose(
            restored,
            camera_pose(view_feet(&mut app) + BODY_CENTER, 0.0),
        );
    }

    #[test]
    fn lethal_sniper_shot_keeps_the_last_live_aim_during_recovery() {
        let (mut app, player, target) = arena();
        app.world_mut().entity_mut(target).insert(Health::new(45.0));
        start_sniping(&mut app, player);
        tick(&mut app, 20);
        assert!(!app.world().get::<Health>(target).unwrap().is_alive());
        let sequence = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .sequence()
            .unwrap();
        assert!(sequence.has_fired());
        let focused = render(&mut app);
        assert!(sequence.aim_position().y > 1.5);
        teleport_actor(app.world_mut(), target, Vec3::new(0., -5., -4.));
        assert_same_pose(render(&mut app), focused);

        tick(&mut app, 60);
        assert!(
            !app.world()
                .get::<Combatant>(player)
                .unwrap()
                .is_combo_active()
        );
        let restored = render(&mut app);
        assert_same_pose(
            restored,
            camera_pose(view_feet(&mut app) + BODY_CENTER, 0.0),
        );
    }

    #[test]
    fn cinematic_sphere_sweep_keeps_the_camera_in_front_of_a_wall_behind_the_player() {
        let (mut app, player, _) = arena();
        spawn_box(
            app.world_mut(),
            Vec3::new(0.0, 0.9, 1.5),
            Vec3::new(3.0, 1.8, 0.2),
            false,
        );
        tick(&mut app, 1);
        start_sniping(&mut app, player);
        tick(&mut app, 14);
        let camera = render(&mut app);
        assert!(camera.translation.z > 0.9 && camera.translation.z < 1.25);
        assert!((camera.translation.y - SNIPER_HEIGHT).abs() < 0.03);
        assert!((camera.rotation * Vec3::NEG_Z).y > 0.0);
    }

    #[test]
    fn ordinary_sister_q_keeps_the_existing_top_down_camera() {
        let (mut app, player, _) = arena();
        request(&mut app, player, CombatIntent::request_ranged_skill);
        tick(&mut app, 14);
        assert!(
            !app.world()
                .get::<Combatant>(player)
                .unwrap()
                .is_combo_active()
        );
        let actual = render(&mut app);
        assert_same_pose(actual, camera_pose(view_feet(&mut app) + BODY_CENTER, 0.));
    }

    #[test]
    fn chain_combo_uses_level_camera_and_roster_clear_restores_even_when_paused() {
        let (mut app, player, _) = arena();
        request(&mut app, player, |i| {
            i.request_pair(crate::simulation::WeaponPair::ChainAxeBow)
        });
        request(&mut app, player, CombatIntent::request_ranged_skill);
        tick(&mut app, 19);
        request(&mut app, player, CombatIntent::request_combo);
        tick(&mut app, 14);
        let camera = render(&mut app);
        let feet = view_feet(&mut app);
        assert!((camera.translation.y - feet.y - 1.2).abs() < 0.02);
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        crate::gameplay::set_enemy_mode(app.world_mut(), crate::gameplay::EnemyMode::Combat);
        let actual = render(&mut app);
        assert_same_pose(actual, camera_pose(view_feet(&mut app) + BODY_CENTER, 0.));
    }

    #[test]
    fn actual_sniper_recoil_pauses_decays_exactly_and_keeps_final_wall_clearance() {
        for wall in [false, true] {
            let (mut app, player, target) = arena();
            app.init_resource::<Assets<Mesh>>()
                .init_resource::<Assets<StandardMaterial>>();
            super::super::impact::setup(app.world_mut());
            app.add_systems(
                Update,
                super::super::impact::update
                    .after(scene::sync_player)
                    .before(follow),
            );
            if wall {
                spawn_box(
                    app.world_mut(),
                    Vec3::new(0.0, 0.9, 1.5),
                    Vec3::new(3.0, 1.8, 0.2),
                    false,
                );
                tick(&mut app, 1);
            }
            start_sniping(&mut app, player);
            tick(&mut app, 14);
            render(&mut app);
            assert!(
                app.world()
                    .resource::<ImpactState>()
                    .recoil(player)
                    .is_none()
            );
            tick(&mut app, 6);
            let kicked = render(&mut app);
            assert!(
                app.world()
                    .resource::<ImpactState>()
                    .recoil(player)
                    .is_some()
            );
            let feet = view_feet(&mut app);
            let alpha = super::super::display_alpha(
                app.world().resource::<Time<Fixed>>(),
                app.world().resource::<SimulationActive>(),
                app.world().resource::<HitStop>(),
            );
            let target_body = app.world().get::<Body>(target).unwrap();
            let aim = target_body.interpolated_world(alpha) + target_body.aim_offset();
            let base = sniper_pose(feet, aim, 0.0);
            if wall {
                assert!(kicked.translation.z > 0.9 && kicked.translation.z < 1.25);
            } else {
                assert!((kicked.translation.distance(base.translation) - 0.152).abs() < 0.005);
                assert!((kicked.rotation * Vec3::NEG_Z).y > (base.rotation * Vec3::NEG_Z).y);
            }
            app.world_mut().resource_mut::<SimulationActive>().0 = false;
            let paused = render(&mut app);
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0));
            assert_same_pose(render(&mut app), paused);
            app.world_mut().resource_mut::<SimulationActive>().0 = true;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(0.15));
            let restored = render(&mut app);
            assert!(
                app.world()
                    .resource::<ImpactState>()
                    .recoil(player)
                    .is_none()
            );
            if wall {
                assert!(restored.translation.z > 0.9 && restored.translation.z < 1.25);
            } else {
                assert_same_pose(restored, base);
            }
        }
    }
    #[test]
    fn paused_roster_replacement_after_a_shot_discards_camera_recoil() {
        let (mut app, player, _) = arena();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>();
        super::super::impact::setup(app.world_mut());
        app.add_systems(
            Update,
            super::super::impact::update
                .after(scene::sync_player)
                .before(follow),
        );
        start_sniping(&mut app, player);
        tick(&mut app, 20);
        render(&mut app);
        assert!(
            app.world()
                .resource::<ImpactState>()
                .recoil(player)
                .is_some()
        );
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        crate::gameplay::set_enemy_mode(app.world_mut(), crate::gameplay::EnemyMode::Combat);
        let actual = render(&mut app);
        assert_same_pose(actual, camera_pose(view_feet(&mut app) + BODY_CENTER, 0.));
    }
}
