//! Headless 3D physical movement, actions and combat.
mod actions;
mod combat;
mod geometry;
mod movement;
mod physics;
mod weapons;

pub use actions::{ActionIntent, Facing, MotionState};
pub use combat::{
    CombatEffect, CombatEvent, CombatFeedback, CombatIntent, Combatant, Enemy, Health, SpawnPoint,
};
pub use geometry::{CollisionWorld, SolidBox};
pub use movement::{Body, MoveIntent};
pub use physics::{
    CROUCHED_HEIGHT, PhysicsPose, STANDING_HEIGHT, SurfaceHit, character_collider, spawn_box,
    spawn_terrain, surface_below, teleport_actor,
};
pub use weapons::{WeaponMode, WeaponSet};

use avian3d::physics_transform::PhysicsTransformConfig;
use avian3d::prelude::*;
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
        if !app.is_plugin_added::<TransformPlugin>() {
            app.add_plugins(TransformPlugin);
        }
        app.add_plugins(PhysicsPlugins::new(FixedUpdate))
            .insert_resource(PhysicsTransformConfig {
                transform_to_position: false,
                ..default()
            })
            .insert_resource(Gravity(Vec3::NEG_Y * actions::GRAVITY))
            .insert_resource(SubstepCount(6))
            .insert_resource(Time::<Fixed>::from_hz(TICK_RATE))
            .init_resource::<SimulationActive>()
            .init_resource::<CollisionWorld>()
            .init_resource::<CombatFeedback>()
            .configure_sets(
                FixedUpdate,
                PhysicsSystems::StepSimulation.run_if(|active: Res<SimulationActive>| active.0),
            )
            .add_systems(
                FixedUpdate,
                (
                    physics::fixed_clock,
                    physics::install,
                    physics::remember_prop_poses,
                    physics::motor,
                    physics::resize_characters,
                )
                    .chain()
                    .before(PhysicsSystems::Prepare)
                    .run_if(|active: Res<SimulationActive>| active.0),
            )
            .add_systems(
                FixedUpdate,
                (physics::publish, combat::advance, physics::flush_teleports)
                    .chain()
                    .after(SpatialQuerySystems)
                    .after(PhysicsSystems::Last)
                    .run_if(|active: Res<SimulationActive>| active.0),
            );
    }
}
