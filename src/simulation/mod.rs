//! Headless kinematic movement. No window, camera or display state lives here.
mod geometry;
mod movement;

pub use geometry::{CollisionWorld, SolidBox};
pub use movement::{Body, MoveIntent};

use bevy::prelude::*;

pub const TICK_RATE: f64 = 60.0;

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(TICK_RATE))
            .add_systems(FixedUpdate, movement::advance);
    }
}
