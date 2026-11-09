//! Headless kinematic movement. No window, camera or display state lives here.
mod actions;
mod combat;
mod geometry;
mod movement;
mod weapons;

pub use actions::{ActionIntent, Facing, MotionState};
pub use combat::{
    CombatEffect, CombatEvent, CombatFeedback, CombatIntent, Combatant, Enemy, Health, SpawnPoint,
};
pub use geometry::{CollisionWorld, SolidBox};
pub use movement::{Body, MoveIntent};
pub use weapons::{WeaponMode, WeaponSet};

use bevy::prelude::*;

pub const TICK_RATE: f64 = 60.0;

/// Client capture pauses the entire authoritative simulation, including AI and cooldowns.
#[derive(Resource)]
pub struct SimulationActive(pub bool);

impl Default for SimulationActive {
    fn default() -> Self {
        Self(true)
    }
}

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(TICK_RATE))
            .init_resource::<SimulationActive>()
            .init_resource::<CombatFeedback>()
            .add_systems(
                FixedUpdate,
                (movement::advance, combat::advance)
                    .chain()
                    .run_if(|active: Res<SimulationActive>| active.0),
            );
    }
}
