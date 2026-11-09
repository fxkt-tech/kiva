use super::CollisionWorld;
use super::actions::{ActionIntent, Facing, MotionState};
use bevy::prelude::*;

#[derive(Component)]
pub struct Body {
    previous: Vec2,
    position: Vec2,
    speed: f32,
    half_extent: f32,
}

impl Body {
    pub fn new(position: Vec2, speed: f32, half_extent: f32) -> Self {
        assert!(
            position.is_finite()
                && speed.is_finite()
                && speed > 0.0
                && half_extent.is_finite()
                && half_extent > 0.0
        );
        Self {
            previous: position,
            position,
            speed,
            half_extent,
        }
    }
    pub fn position(&self) -> Vec2 {
        self.position
    }
    /// Actual displacement of the last fixed step, including collision resolution.
    pub fn step_displacement(&self) -> Vec2 {
        self.position - self.previous
    }
    pub fn interpolated(&self, alpha: f32) -> Vec2 {
        self.previous.lerp(self.position, alpha.clamp(0.0, 1.0))
    }

    pub(super) fn teleport(&mut self, position: Vec2) {
        assert!(position.is_finite());
        self.previous = position;
        self.position = position;
    }
}

#[derive(Component, Default)]
pub struct MoveIntent(Vec2);
impl MoveIntent {
    pub fn set(&mut self, direction: Vec2) {
        self.0 = if direction.is_finite() {
            direction.clamp_length_max(1.0)
        } else {
            Vec2::ZERO
        };
    }
    pub fn stop(&mut self) {
        self.0 = Vec2::ZERO;
    }
}

type MovementComponents = (
    &'static mut Body,
    &'static MoveIntent,
    Option<&'static mut MotionState>,
    Option<&'static mut ActionIntent>,
    Option<&'static Facing>,
    Option<&'static super::combat::Health>,
);

pub(super) fn advance(
    time: Res<Time<Fixed>>,
    world: Res<CollisionWorld>,
    mut bodies: Query<MovementComponents>,
) {
    for (mut body, intent, motion, mut actions, facing, health) in &mut bodies {
        body.previous = body.position;
        if health.is_some_and(|health| !health.is_alive()) {
            if let Some(mut motion) = motion {
                motion.reset();
            }
            if let Some(mut actions) = actions {
                actions.clear();
            }
            continue;
        }
        let delta = if let Some(mut motion) = motion {
            let mut idle = ActionIntent::default();
            motion.advance(
                actions.as_deref_mut().unwrap_or(&mut idle),
                intent.0,
                facing.map_or(Vec2::NEG_Y, Facing::direction),
                body.speed,
                time.delta_secs(),
            )
        } else {
            intent.0 * body.speed * time.delta_secs()
        };
        body.position = world.slide(body.position, delta, body.half_extent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::SimulationPlugin;

    fn walk(direction: Vec2) -> Vec2 {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(CollisionWorld::new(100.0, vec![]));
        let mut intent = MoveIntent::default();
        intent.set(direction);
        let actor = app
            .world_mut()
            .spawn((Body::new(Vec2::ZERO, 6.0, 0.35), intent))
            .id();
        // The fixed schedule normally receives its delta from Bevy's loop.
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
        for _ in 0..60 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world().get::<Body>(actor).unwrap().position()
    }

    #[test]
    fn straight_and_diagonal_input_cover_the_same_distance_in_one_second() {
        assert!((walk(Vec2::X).length() - 6.0).abs() < 0.001);
        assert!((walk(Vec2::ONE).length() - 6.0).abs() < 0.001);
    }

    #[test]
    fn released_or_invalid_input_does_not_move() {
        assert_eq!(walk(Vec2::ZERO), Vec2::ZERO);
        assert_eq!(walk(Vec2::new(f32::NAN, 1.0)), Vec2::ZERO);
    }

    fn actor(world: CollisionWorld) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(world);
        let actor = app
            .world_mut()
            .spawn((
                Body::new(Vec2::ZERO, 6.0, 0.35),
                MoveIntent::default(),
                ActionIntent::default(),
                MotionState::default(),
                Facing::default(),
            ))
            .id();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
        (app, actor)
    }

    fn tick(app: &mut App, count: usize) {
        for _ in 0..count {
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn set_movement(app: &mut App, actor: Entity, direction: Vec2, physically_held: bool) {
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(direction);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .set_movement_held(physically_held);
    }

    fn set_shift(app: &mut App, actor: Entity, held: bool) {
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .set_held(held, false);
    }

    fn begin_sprint(app: &mut App, actor: Entity) {
        set_movement(app, actor, Vec2::X, true);
        set_shift(app, actor, true);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(app, 18);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
    }

    #[test]
    fn sprint_and_block_change_actual_speed_and_block_takes_priority() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .set_held(true, false);
        tick(&mut app, 12);
        let sprint_start = app.world().get::<Body>(actor).unwrap().position().x;
        tick(&mut app, 60);
        let sprint_end = app.world().get::<Body>(actor).unwrap().position().x;
        assert!((sprint_end - sprint_start - 9.6).abs() < 0.001);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .set_held(true, true);
        tick(&mut app, 60);
        let block_end = app.world().get::<Body>(actor).unwrap().position().x;
        assert!((block_end - sprint_end - 2.1).abs() < 0.001);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_blocking());
        assert!(!motion.is_sprinting());
    }

    #[test]
    fn jump_is_latched_until_fixed_step_and_airborne_requests_do_not_restart_it() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        assert!(app.world().get::<MotionState>(actor).unwrap().is_grounded());
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.height() > 0.0);
        assert_eq!(motion.interpolated_height(0.0), 0.0);
        assert_eq!(motion.interpolated_height(1.0), motion.height());
        tick(&mut app, 20);
        let falling_speed = app
            .world()
            .get::<MotionState>(actor)
            .unwrap()
            .vertical_velocity();
        assert!(falling_speed < 0.0);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        tick(&mut app, 1);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .vertical_velocity()
                < falling_speed
        );
        tick(&mut app, 60);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_grounded());
        assert_eq!(motion.height(), 0.0);
    }

    #[test]
    fn dodge_locks_direction_has_delayed_invulnerability_and_cannot_repeat_early() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        app.world_mut()
            .get_mut::<Facing>(actor)
            .unwrap()
            .set(Vec2::X);
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 1);
        assert!(
            !app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_invulnerable()
        );
        tick(&mut app, 3);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_invulnerable()
        );
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::Y);
        tick(&mut app, 14);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_dodging());
        assert!(!motion.is_invulnerable());
        tick(&mut app, 8);
        let position = app.world().get::<Body>(actor).unwrap().position();
        assert!((position.x - 2.4).abs() < 0.001);
        assert_eq!(position.y, 0.0);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 1);
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_dodging());
        tick(&mut app, 60);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 1);
        assert!(app.world().get::<MotionState>(actor).unwrap().is_dodging());
    }

    #[test]
    fn dodge_and_cannon_leap_still_sweep_against_walls() {
        use crate::simulation::SolidBox;

        for leap in [false, true] {
            let world = CollisionWorld::new(
                100.0,
                vec![SolidBox::new(Vec2::new(2.0, 0.0), Vec2::splat(0.5))],
            );
            let (mut app, actor) = actor(world);
            if leap {
                assert!(
                    app.world_mut()
                        .get_mut::<MotionState>(actor)
                        .unwrap()
                        .start_leap(Vec2::ZERO, Vec2::X * 6.0, 0.6, 2.0)
                );
            } else {
                app.world_mut()
                    .get_mut::<Facing>(actor)
                    .unwrap()
                    .set(Vec2::X);
                app.world_mut()
                    .get_mut::<MoveIntent>(actor)
                    .unwrap()
                    .set(Vec2::X);
                app.world_mut()
                    .get_mut::<ActionIntent>(actor)
                    .unwrap()
                    .request_dodge();
            }
            tick(&mut app, 60);
            let position = app.world().get::<Body>(actor).unwrap().position();
            assert!((position.x - 1.15).abs() < 0.001, "{position:?}");
            assert!(
                app.world()
                    .resource::<CollisionWorld>()
                    .is_clear(position, 0.35)
            );
            assert!(app.world().get::<MotionState>(actor).unwrap().is_grounded());
        }
    }

    #[test]
    fn cannon_leap_has_interpolated_height_lands_at_target_and_rejects_airborne_restarts() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        assert!(
            app.world_mut()
                .get_mut::<MotionState>(actor)
                .unwrap()
                .start_leap(Vec2::ZERO, Vec2::X * 6.0, 0.6, 2.0)
        );
        assert!(app.world().get::<MotionState>(actor).unwrap().is_leaping());
        tick(&mut app, 18);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!((motion.height() - 2.0).abs() < 0.001);
        assert!(motion.interpolated_height(0.0) <= motion.interpolated_height(0.5));
        assert!(motion.interpolated_height(0.5) <= motion.height());
        assert!(
            !app.world_mut()
                .get_mut::<MotionState>(actor)
                .unwrap()
                .start_leap(Vec2::ZERO, Vec2::Y * 10.0, 0.6, 2.0)
        );
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        tick(&mut app, 42);
        assert!(app.world().get::<MotionState>(actor).unwrap().is_grounded());
        assert!(
            app.world()
                .get::<Body>(actor)
                .unwrap()
                .position()
                .distance(Vec2::X * 6.0)
                < 0.001
        );
    }

    #[test]
    fn knockup_interrupts_leap_without_changing_horizontal_authority() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        app.world_mut()
            .get_mut::<MotionState>(actor)
            .unwrap()
            .start_leap(Vec2::ZERO, Vec2::X * 6.0, 0.6, 2.0);
        tick(&mut app, 12);
        let position = app.world().get::<Body>(actor).unwrap().position();
        let height = app.world().get::<MotionState>(actor).unwrap().height();
        assert!(
            app.world_mut()
                .get_mut::<MotionState>(actor)
                .unwrap()
                .launch_up(8.0)
        );
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_leaping());
        tick(&mut app, 1);
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), position);
        assert!(app.world().get::<MotionState>(actor).unwrap().height() > height);
        tick(&mut app, 120);
        assert!(app.world().get::<MotionState>(actor).unwrap().is_grounded());
    }

    #[test]
    fn clearing_input_cancels_pending_actions_and_teleport_resets_interpolation() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        let mut actions = app.world_mut().get_mut::<ActionIntent>(actor).unwrap();
        actions.set_held(true, true);
        actions.request_jump();
        actions.request_dodge();
        actions.clear();
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_grounded());
        assert!(!motion.is_blocking());
        assert!(!motion.is_sprinting());
        assert!(!motion.is_dodging());
        app.world_mut()
            .get_mut::<Body>(actor)
            .unwrap()
            .teleport(Vec2::new(12.0, 5.0));
        let body = app.world().get::<Body>(actor).unwrap();
        assert_eq!(body.interpolated(0.0), body.position());
        assert_eq!(body.interpolated(0.5), body.position());
    }

    #[test]
    fn invalid_facing_and_leap_requests_do_not_poison_motion() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        let mut facing = app.world_mut().get_mut::<Facing>(actor).unwrap();
        facing.set(Vec2::new(f32::NAN, 1.0));
        facing.set(Vec2::splat(f32::MAX));
        assert_eq!(facing.direction(), Vec2::NEG_Y);
        let mut motion = app.world_mut().get_mut::<MotionState>(actor).unwrap();
        assert!(!motion.start_leap(Vec2::ZERO, Vec2::X, 0.0, 2.0));
        assert!(!motion.start_leap(Vec2::ZERO, Vec2::splat(f32::NAN), 0.6, 2.0));
        assert!(!motion.launch_up(f32::NAN));
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_grounded());
        assert!(motion.height().is_finite());
    }

    #[test]
    fn paused_simulation_preserves_motion_and_cleared_requests_do_not_replay_on_resume() {
        use crate::simulation::SimulationActive;

        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        tick(&mut app, 5);
        let height = app.world().get::<MotionState>(actor).unwrap().height();
        let velocity = app
            .world()
            .get::<MotionState>(actor)
            .unwrap()
            .vertical_velocity();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        let mut actions = app.world_mut().get_mut::<ActionIntent>(actor).unwrap();
        actions.request_jump();
        actions.request_dodge();
        actions.clear(); // Capture loss clears the pending commands before fixed update.
        tick(&mut app, 120);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert_eq!(motion.height(), height);
        assert_eq!(motion.vertical_velocity(), velocity);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        tick(&mut app, 90);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_grounded());
        assert!(!motion.is_dodging());
        assert_eq!(
            app.world().get::<Body>(actor).unwrap().position(),
            Vec2::ZERO
        );

        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 4);
        let position = app.world().get::<Body>(actor).unwrap().position();
        let remaining = app
            .world()
            .get::<MotionState>(actor)
            .unwrap()
            .dodge_remaining();
        let cooldown = app
            .world()
            .get::<MotionState>(actor)
            .unwrap()
            .dodge_cooldown();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        tick(&mut app, 120);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert_eq!(motion.dodge_remaining(), remaining);
        assert_eq!(motion.dodge_cooldown(), cooldown);
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), position);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        tick(&mut app, 120);
        let position = app.world().get::<Body>(actor).unwrap().position();
        assert!((position.length() - 2.4).abs() < 0.001);
        assert_eq!(position.x, 0.0);
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_dodging());
    }

    #[test]
    fn stationary_shift_tap_dodges_backwards_immediately_without_latching_sprint() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        app.world_mut()
            .get_mut::<Facing>(actor)
            .unwrap()
            .set(Vec2::X);
        set_shift(&mut app, actor, true);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_dodging());
        assert!(!motion.is_invulnerable());
        assert!(!motion.sprint_latched());
        let position = app.world().get::<Body>(actor).unwrap().position();
        assert!(position.x < 0.0);
        assert_eq!(position.y, 0.0);
        set_shift(&mut app, actor, false);
        tick(&mut app, 25);
        let position = app.world().get::<Body>(actor).unwrap().position();
        assert!((position.x + 2.4).abs() < 0.001);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(!motion.is_dodging());
        assert!(!motion.sprint_latched());
    }

    #[test]
    fn short_shift_tap_completes_dodge_then_returns_to_walking() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        set_movement(&mut app, actor, Vec2::X, true);
        set_shift(&mut app, actor, true);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 1);
        set_shift(&mut app, actor, false);
        tick(&mut app, 25);
        let position = app.world().get::<Body>(actor).unwrap().position();
        assert!((position.x - 2.4).abs() < 0.001);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(!motion.is_sprinting());
        assert!(!motion.sprint_latched());
        tick(&mut app, 60);
        let walked = app.world().get::<Body>(actor).unwrap().position().x - position.x;
        assert!((walked - 6.0).abs() < 0.001);
    }

    #[test]
    fn holding_shift_shortens_dodge_before_transitioning_into_sprint() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        set_movement(&mut app, actor, Vec2::X, true);
        set_shift(&mut app, actor, true);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 9);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_dodging());
        assert!(!motion.sprint_latched());
        assert!(!motion.is_sprinting());
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.sprint_latched());
        assert!(motion.is_dodging());
        assert!(!motion.is_sprinting());
        tick(&mut app, 6);
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_dodging());
        let sprint_start = app.world().get::<Body>(actor).unwrap().position().x;
        assert!((1.5..1.8).contains(&sprint_start));
        tick(&mut app, 60);
        let sprinted = app.world().get::<Body>(actor).unwrap().position().x - sprint_start;
        assert!((sprinted - 9.6).abs() < 0.001);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
    }

    #[test]
    fn sprint_latch_survives_shift_release_until_all_direction_keys_are_released() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        begin_sprint(&mut app, actor);
        set_shift(&mut app, actor, false);
        let start = app.world().get::<Body>(actor).unwrap().position();
        tick(&mut app, 60);
        let end = app.world().get::<Body>(actor).unwrap().position();
        assert!((end.x - start.x - 9.6).abs() < 0.001);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
        set_movement(&mut app, actor, Vec2::ZERO, false);
        tick(&mut app, 1);
        assert!(
            !app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
        set_movement(&mut app, actor, Vec2::Y, true);
        tick(&mut app, 60);
        assert!(
            !app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
        let position = app.world().get::<Body>(actor).unwrap().position();
        assert!((position.y - 6.0).abs() < 0.001);
    }

    #[test]
    fn sprint_survives_direction_changes_and_opposing_held_directions() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        begin_sprint(&mut app, actor);
        set_shift(&mut app, actor, false);
        set_movement(&mut app, actor, Vec2::Y, true);
        tick(&mut app, 1);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
        set_movement(&mut app, actor, Vec2::ZERO, true); // W + S, both still physically held.
        let start = app.world().get::<Body>(actor).unwrap().position();
        tick(&mut app, 15);
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), start);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.sprint_latched());
        assert!(!motion.is_sprinting());
        set_movement(&mut app, actor, Vec2::NEG_X, true);
        tick(&mut app, 60);
        let position = app.world().get::<Body>(actor).unwrap().position();
        assert!((start.x - position.x - 9.6).abs() < 0.001);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
    }

    #[test]
    fn capture_loss_cancellation_survives_paused_ticks_and_preserves_airborne_motion() {
        use crate::simulation::SimulationActive;

        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        begin_sprint(&mut app, actor);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.sprint_latched());
        let height = motion.height();
        let velocity = motion.vertical_velocity();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .clear();
        tick(&mut app, 120);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert_eq!(motion.height(), height);
        assert_eq!(motion.vertical_velocity(), velocity);
        // Fresh movement input after capture must not erase the pending cancellation.
        set_movement(&mut app, actor, Vec2::X, true);
        set_shift(&mut app, actor, false);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(!motion.sprint_latched());
        assert!(!motion.is_sprinting());
        assert!(motion.height() > height);
        tick(&mut app, 90);
        assert!(
            !app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
        let start = app.world().get::<Body>(actor).unwrap().position().x;
        tick(&mut app, 60);
        let walked = app.world().get::<Body>(actor).unwrap().position().x - start;
        assert!((walked - 6.0).abs() < 0.001);
    }

    #[test]
    fn all_directions_released_between_fixed_ticks_clears_latched_sprint() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        begin_sprint(&mut app, actor);
        set_shift(&mut app, actor, false);
        tick(&mut app, 1);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
        set_movement(&mut app, actor, Vec2::ZERO, false);
        set_movement(&mut app, actor, Vec2::Y, true);
        let start = app.world().get::<Body>(actor).unwrap().position();
        tick(&mut app, 60);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(!motion.sprint_latched());
        assert!(!motion.is_sprinting());
        let position = app.world().get::<Body>(actor).unwrap().position();
        assert!((position.y - start.y - 6.0).abs() < 0.001);
    }

    #[test]
    fn repressing_shift_late_in_dodge_never_rewinds_position_when_duration_shortens() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        set_movement(&mut app, actor, Vec2::X, true);
        set_shift(&mut app, actor, true);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 1);
        set_shift(&mut app, actor, false);
        tick(&mut app, 10);
        set_shift(&mut app, actor, true);
        let mut previous = app.world().get::<Body>(actor).unwrap().position().x;
        for _ in 0..11 {
            tick(&mut app, 1);
            let position = app.world().get::<Body>(actor).unwrap().position().x;
            assert!(position >= previous, "{position} moved behind {previous}");
            previous = position;
        }
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(!motion.is_dodging());
        assert!(motion.is_sprinting());
    }

    #[test]
    fn shift_release_between_fixed_ticks_restarts_the_continuous_hold_timer() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        set_movement(&mut app, actor, Vec2::X, true);
        set_shift(&mut app, actor, true);
        tick(&mut app, 6);
        set_shift(&mut app, actor, false);
        set_shift(&mut app, actor, true);
        tick(&mut app, 4);
        assert!(
            !app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
        tick(&mut app, 6);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .sprint_latched()
        );
    }

    #[test]
    fn fresh_dodge_press_restarts_hold_timer_even_when_shift_release_was_not_sampled() {
        let (mut app, actor) = actor(CollisionWorld::new(100.0, vec![]));
        begin_sprint(&mut app, actor);
        tick(&mut app, 60);
        let start = app.world().get::<Body>(actor).unwrap().position().x;
        // Release/repress happened within one render interval: only the fresh edge survives.
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        tick(&mut app, 1);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert!(motion.is_dodging());
        assert!(motion.dodge_remaining() > 0.26);
        assert!(motion.sprint_latched());
        set_shift(&mut app, actor, false);
        tick(&mut app, 17);
        assert!(app.world().get::<MotionState>(actor).unwrap().is_dodging());
        tick(&mut app, 8);
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_dodging());
        let end = app.world().get::<Body>(actor).unwrap().position().x;
        assert!((end - start - 2.4).abs() < 0.001);
    }
}
