//! Headless 3D physical movement, actions and combat.
mod abilities;
mod actions;
mod boss;
mod combat;
mod defense;
#[cfg(test)]
mod geometry;
mod impact;
mod motion_feedback;
mod movement;
mod physics;
mod resources;
mod weapons;

pub use abilities::{
    ActionAvailability, ActionKind, ActionSnapshot, AttackClock, ComboWindow, Companion,
    CompanionState, MeleeStage, SequenceSnapshot, TrackId,
};
pub use actions::{ActionIntent, Facing, MotionState};
pub use boss::{Boss, BossPhase, BossState};
pub(crate) use combat::ensure_companions as combat_setup_companions;
pub use combat::{
    Arsenal, CombatEffect, CombatEvent, CombatFeedback, CombatIntent, CombatStatus, Combatant,
    Enemy, Energy, Health, HitContext, HitOutcome, HitProbe, HitProbes, HitTrack, Hurtbox,
    ProbeShape, Projectile, SpawnPoint, clear_projectiles,
};
pub use defense::{DefenseEdge, DefenseKind, DefenseSnapshot, DefenseState};
#[cfg(test)]
pub(crate) use geometry::{CollisionWorld, SolidBox};
pub use impact::HitStop;
pub use motion_feedback::{MotionEvent, MotionFeedback, MotionKind};
pub use movement::{Body, MoveIntent};
pub use physics::{
    CROUCHED_HEIGHT, PhysicsPose, STANDING_HEIGHT, SurfaceHit, character_collider, spawn_box,
    spawn_terrain, surface_below, teleport_actor,
};
pub use resources::{SlowEffect, Stamina};
pub use weapons::{MeleeProfile, WeaponMode, WeaponPair};

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
            .init_resource::<CombatFeedback>()
            .init_resource::<MotionFeedback>()
            .init_resource::<combat::ActionIds>()
            .init_resource::<HitStop>()
            .init_resource::<HitProbes>()
            .configure_sets(
                FixedUpdate,
                PhysicsSystems::StepSimulation.run_if(impact::simulation_running),
            )
            .add_systems(
                FixedUpdate,
                impact::begin_tick
                    .before(physics::fixed_clock)
                    .before(PhysicsSystems::StepSimulation),
            )
            .add_systems(
                FixedUpdate,
                (
                    physics::fixed_clock,
                    physics::install,
                    combat::ensure_companions,
                    combat::tick_slows,
                    combat::drain_rage,
                    combat::release_rage,
                    combat::prepare,
                    physics::motor,
                    physics::resize_characters,
                )
                    .chain()
                    .before(PhysicsSystems::Prepare)
                    .run_if(impact::simulation_running),
            )
            .add_systems(
                FixedUpdate,
                (
                    physics::publish,
                    combat::tick_probes,
                    combat::resolve_actions,
                    combat::advance_projectiles,
                    combat::advance_enemies,
                    boss::advance,
                    combat::publish_availability,
                    combat::recover_stamina,
                    combat::publish_companions,
                    physics::flush_teleports,
                )
                    .chain()
                    .after(SpatialQuerySystems)
                    .after(PhysicsSystems::Last)
                    .run_if(impact::simulation_running),
            );
    }
}
