//! Headless simulation: commands, definitions and read-only ECS views form its
//! interface. No system reads a keyboard, window, camera, mesh or browser object.
mod abilities;
mod actions;
mod combat;
mod model;
mod movement;
mod navigation;
mod orders;
pub use abilities::{AbilityCatalog, AbilityDefinition, AbilityEffect, AbilityId, Targeting};
use bevy::prelude::*;
pub use model::{
    AbilitySlots, AbilityTarget, AreaEffect, CommandQueue, Projectile, Rejection,
    SimulationControl, SimulationEvent, SimulationTick, Team, Unit, UnitAction, UnitCommand,
    UnitOrder, UnitPose, UnitSpec, UnitState, UnitStats,
};
use model::{Movement, PendingDamage};
pub use navigation::{NavigationMap, Obstacle, PathError};

pub const TICK_RATE: f64 = 60.0;
pub const TICK_SECONDS: f32 = 1.0 / TICK_RATE as f32;
pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(TICK_RATE))
            .init_resource::<CommandQueue>()
            .init_resource::<AbilityCatalog>()
            .init_resource::<SimulationControl>()
            .init_resource::<SimulationTick>()
            .init_resource::<PendingDamage>()
            .add_message::<SimulationEvent>()
            .add_systems(
                FixedUpdate,
                (
                    begin_tick,
                    combat::respawn_units,
                    orders::consume_commands,
                    actions::advance_actions,
                    orders::resolve_orders,
                    movement::move_units,
                    combat::advance_projectiles,
                    combat::advance_areas,
                    combat::resolve_damage,
                )
                    .chain()
                    .run_if(|control: Res<SimulationControl>| !control.paused),
            );
    }
}
fn begin_tick(mut tick: ResMut<SimulationTick>) {
    tick.0 += 1;
}

/// Validates construction; all runtime mutation stays inside this module.
pub fn spawn_unit(world: &mut World, spec: UnitSpec) -> Result<Entity, &'static str> {
    if !spec.stats.valid()
        || spec.name.is_empty()
        || spec
            .respawn_delay
            .is_some_and(|v| !v.is_finite() || v <= 0.0)
    {
        return Err("invalid unit definition");
    }
    if !world
        .resource::<NavigationMap>()
        .walkable(spec.position, spec.stats.radius)
    {
        return Err("unit spawn is blocked");
    }
    if spec
        .abilities
        .iter()
        .flatten()
        .any(|id| world.resource::<AbilityCatalog>().get(*id).is_none())
    {
        return Err("unknown equipped ability");
    }
    let health = spec.stats.max_health;
    let mana = spec.stats.max_mana;
    Ok(world
        .spawn((
            Unit {
                name: spec.name,
                team: spec.team,
                stats: spec.stats,
                home: spec.position,
                respawn_delay: spec.respawn_delay,
            },
            UnitState {
                health,
                mana,
                attack_cooldown: 0.0,
                respawn_remaining: None,
            },
            UnitPose {
                position: spec.position,
                previous: spec.position,
                facing: Vec2::NEG_Y,
            },
            AbilitySlots {
                ids: spec.abilities,
                cooldowns: [0.0; 4],
            },
            UnitOrder::default(),
            UnitAction::default(),
            Movement::default(),
        ))
        .id())
}
