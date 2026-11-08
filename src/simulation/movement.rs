use super::CollisionWorld;
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
    pub fn interpolated(&self, alpha: f32) -> Vec2 {
        self.previous.lerp(self.position, alpha.clamp(0.0, 1.0))
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

pub(super) fn advance(
    time: Res<Time<Fixed>>,
    world: Res<CollisionWorld>,
    mut bodies: Query<(&mut Body, &MoveIntent)>,
) {
    for (mut body, intent) in &mut bodies {
        body.previous = body.position;
        body.position = world.slide(
            body.position,
            intent.0 * body.speed * time.delta_secs(),
            body.half_extent,
        );
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
}
