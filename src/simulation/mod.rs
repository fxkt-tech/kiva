//! Headless simulation: commands, definitions and read-only ECS views form its
//! interface. No system reads a keyboard, window, camera, mesh or browser object.
mod abilities;
mod actions;
mod attributes;
mod characters;
mod combat;
mod damage;
mod effects;
mod model;
mod movement;
mod navigation;
mod orders;
mod statuses;
pub use abilities::{
    AbilityCatalog, AbilityDefinition, AbilityExecution, AbilityId, AbilityLoadout, Delivery,
    DisplacementDirection, DisplacementSpec, Effect, ImpactDefinition, Targeting,
};
pub use attributes::{
    AttributeChanges, AttributeDelta, AttributeModifier, AttributeRejection, BaseAttributes,
    EffectiveAttributes, MAX_ATTACK_SPEED, MIN_ATTACK_SPEED, ModifierRemoval, ModifierSourceId,
};
use bevy::prelude::*;
pub use characters::{
    CharacterCatalog, CharacterDefinition, CharacterError, CharacterId, CharacterIdentity,
    CharacterSpawnSpec, LoadoutSelection, spawn_character,
};
pub use damage::{DamageOrigin, DamageSnapshot, DamageSpec, DamageType};
pub use effects::ImpactSnapshot;
use model::Movement;
pub use model::{
    AbilitySlots, AbilityTarget, AreaEffect, CommandQueue, Projectile, Rejection,
    SimulationControl, SimulationEvent, SimulationTick, Team, Unit, UnitAction, UnitCommand,
    UnitLifeRef, UnitOrder, UnitPose, UnitSpec, UnitState, UnitStats,
};
pub use navigation::{NavigationMap, Obstacle, PathError};
pub use statuses::{
    ControlState, StatusDuration, StatusKind, StatusSpec, StatusView, UnitStatuses,
};

pub const TICK_RATE: f64 = 60.0;
pub const TICK_SECONDS: f32 = 1.0 / TICK_RATE as f32;
pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(TICK_RATE))
            .init_resource::<CommandQueue>()
            .init_resource::<AttributeChanges>()
            .init_resource::<AbilityCatalog>()
            .init_resource::<CharacterCatalog>()
            .init_resource::<SimulationControl>()
            .init_resource::<SimulationTick>()
            .init_resource::<effects::PendingImpacts>()
            .init_resource::<statuses::StatusSequence>()
            .add_message::<SimulationEvent>()
            .add_systems(
                FixedUpdate,
                (
                    begin_tick,
                    combat::respawn_units,
                    attributes::consume_changes,
                    orders::consume_commands,
                    actions::advance_actions,
                    orders::resolve_orders,
                    movement::move_units,
                    combat::advance_projectiles,
                    combat::advance_areas,
                    statuses::advance_old,
                    effects::resolve,
                )
                    .chain()
                    .run_if(|control: Res<SimulationControl>| !control.paused),
            );
    }
}
fn begin_tick(mut tick: ResMut<SimulationTick>) {
    tick.0 += 1;
}

/// Ends the current trial and leaves the simulation paused at tick zero.
///
/// Despawning runtime roots also removes their linked children. Navigation,
/// content catalogs and monotonic identity allocators remain in this World.
pub fn reset_simulation(world: &mut World) {
    world.resource_mut::<SimulationControl>().paused = true;
    let entities: Vec<_> = world
        .query_filtered::<Entity, Or<(With<Unit>, With<Projectile>, With<AreaEffect>)>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.despawn(entity);
    }
    world.resource_mut::<CommandQueue>().clear();
    world.resource_mut::<AttributeChanges>().clear_pending();
    world
        .resource_mut::<effects::PendingImpacts>()
        .clear_pending();
    world.resource_mut::<Messages<SimulationEvent>>().clear();
    world.resource_mut::<SimulationTick>().0 = 0;
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
    if abilities::validate_loadout(&spec.abilities, world.resource::<AbilityCatalog>()).is_err() {
        return Err("unknown equipped ability");
    }
    let slow_source = world
        .resource_mut::<AttributeChanges>()
        .allocate_source()
        .ok_or("attribute source space exhausted")?;
    let health = spec.stats.max_health;
    let mana = spec.stats.max_mana;
    Ok(world
        .spawn((
            Unit {
                name: spec.name,
                team: spec.team,
                attributes: attributes::AttributeState::new(spec.stats.base_attributes),
                stats: spec.stats,
                home: spec.position,
                respawn_delay: spec.respawn_delay,
            },
            UnitState {
                health,
                life_epoch: 1,
                mana,
                attack_cooldown: 0,
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
            UnitStatuses::new(slow_source),
            ControlState::default(),
        ))
        .id())
}
