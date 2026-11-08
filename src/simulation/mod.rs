//! Headless simulation: commands, definitions and read-only ECS views form its
//! interface. No system reads a keyboard, window, camera, mesh or browser object.
mod abilities;
mod actions;
mod attack_traits;
mod attributes;
mod battlefield;
mod cast_orders;
mod characters;
mod combat;
mod compound;
mod damage;
mod effects;
mod equipment;
mod mechanics;
mod mechanics_defs;
mod model;
mod vision;
pub use mechanics::{BattlefieldObject, ObjectKind, UnitMechanics};
pub use mechanics_defs::{
    ArrowSpec, AttackPassive, DashLinkSpec, EmpowerSpec, LeapSpec, LineSpec, RingTerrainSpec,
    ScoutSpec, ShieldSpec, StandardSpec, UnitKind, VolleySpec,
};
pub use vision::{TeamVision, point_visible, team_can_see};
mod movement;
mod navigation;
mod orders;
mod recovery;
mod statuses;
pub use abilities::{
    AbilityCatalog, AbilityDefinition, AbilityExecution, AbilityId, AbilityLoadout, Delivery,
    DisplacementDirection, DisplacementSpec, Effect, ImpactDefinition, Targeting,
};
pub use actions::preview_cast;
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
pub use equipment::{
    EQUIPMENT_SLOTS, EquipmentCatalog, EquipmentDefinition, EquipmentError, EquipmentId,
    EquipmentInstanceId, EquipmentPassive, EquipmentSlots, EquippedItem, equip_equipment,
    unequip_equipment,
};
use model::Movement;
pub use model::{
    AbilitySlots, AbilityTarget, AreaEffect, BasicAttackDelivery, CastSource, CommandQueue,
    Projectile, Rejection, SimulationControl, SimulationEvent, SimulationTick, Team, Unit,
    UnitAction, UnitClass, UnitCommand, UnitLifeRef, UnitOrder, UnitPose, UnitSpec, UnitState,
    UnitStats,
};
pub use movement::{SoftAvoidance, is_dashing, leap_progress, motion_allows_cast};
pub use navigation::{NavigationMap, Obstacle, PathError};
pub use recovery::{RecoveryAmount, RecoveryError, RecoverySpec, restore_in_zone};
pub use statuses::{
    ControlState, StatusDuration, StatusKind, StatusSpec, StatusView, UnitStatuses,
};

pub const TICK_RATE: f64 = 60.0;
pub const TICK_SECONDS: f32 = 1.0 / TICK_RATE as f32;
/// A complete atomic battle tick. Modes may order their decisions/outcome around
/// this set without gaining access to the private battle systems.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    Step,
}
pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(TICK_RATE))
            .init_resource::<CommandQueue>()
            .init_resource::<AttributeChanges>()
            .init_resource::<AbilityCatalog>()
            .init_resource::<CharacterCatalog>()
            .init_resource::<EquipmentCatalog>()
            .init_resource::<SimulationControl>()
            .init_resource::<SimulationTick>()
            .init_resource::<effects::PendingImpacts>()
            .init_resource::<mechanics::MechanicMemory>()
            .init_resource::<TeamVision>()
            .init_resource::<statuses::StatusSequence>()
            .add_message::<SimulationEvent>()
            .add_systems(
                FixedUpdate,
                (
                    begin_tick,
                    combat::respawn_units,
                    attributes::consume_changes,
                    mechanics::before_commands,
                    compound::advance_objects,
                    vision::refresh,
                    orders::consume_commands,
                    cast_orders::advance,
                    actions::advance_actions,
                    orders::resolve_orders,
                    movement::move_units,
                    compound::advance_motion_impacts,
                    combat::advance_projectiles,
                    combat::advance_areas,
                    statuses::advance_old,
                    effects::resolve,
                    vision::refresh,
                )
                    .chain()
                    .in_set(SimulationSet::Step)
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
        .query_filtered::<Entity, Or<(
            With<Unit>,
            With<Projectile>,
            With<AreaEffect>,
            With<BattlefieldObject>,
        )>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.despawn(entity);
    }
    world.resource_mut::<NavigationMap>().clear_temporary();
    *world.resource_mut::<mechanics::MechanicMemory>() = default();
    world.resource_mut::<CommandQueue>().clear();
    world.resource_mut::<AttributeChanges>().clear_pending();
    world
        .resource_mut::<effects::PendingImpacts>()
        .clear_pending();
    world.resource_mut::<Messages<SimulationEvent>>().clear();
    world.resource_mut::<SimulationTick>().0 = 0;
    vision::refresh(world);
}

/// Checks a complete unit configuration without spawning or allocating any
/// identities. Multi-unit mode setup can preflight the same rules atomically.
pub fn validate_unit_spec(world: &World, spec: &UnitSpec) -> Result<(), &'static str> {
    if !spec.stats.valid()
        || spec.name.is_empty()
        || spec
            .respawn_delay
            .is_some_and(|v| !v.is_finite() || v <= 0.0)
    {
        return Err("invalid unit definition");
    }
    if spec.stats.class == UnitClass::Structure
        && (spec.respawn_delay.is_some() || spec.abilities.iter().any(Option::is_some))
    {
        return Err("structures cannot have abilities or respawn");
    }
    if world
        .resource::<NavigationMap>()
        .resolve_placement(spec.position, spec.stats.radius)
        .is_none()
    {
        return Err("unit spawn is blocked");
    }
    if abilities::validate_loadout(&spec.abilities, world.resource::<AbilityCatalog>()).is_err() {
        return Err("unknown equipped ability");
    }
    mechanics::validate_loadout(&spec.abilities, world.resource::<AbilityCatalog>())?;
    Ok(())
}

/// Validates construction; all runtime mutation stays inside this module.
pub fn spawn_unit(world: &mut World, spec: UnitSpec) -> Result<Entity, &'static str> {
    validate_unit_spec(world, &spec)?;
    let position = world
        .resource::<NavigationMap>()
        .resolve_placement(spec.position, spec.stats.radius)
        .expect("validated birth placement");
    let slow_source = world
        .resource_mut::<AttributeChanges>()
        .allocate_source()
        .ok_or("attribute source space exhausted")?;
    let health = spec.stats.max_health;
    let mana = spec.stats.max_mana;
    let control = ControlState::for_class(spec.stats.class);
    let entity = world
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
                last_damaged_tick: None,
                last_recovery_tick: None,
            },
            UnitPose {
                position,
                previous: position,
                facing: Vec2::NEG_Y,
            },
            AbilitySlots {
                ids: spec.abilities,
                cooldowns: [0.0; 4],
            },
            EquipmentSlots::default(),
            UnitOrder::default(),
            UnitAction::default(),
            Movement::default(),
            UnitStatuses::new(slow_source),
            control,
        ))
        .id();
    if let Err(reason) = mechanics::initialize(world, entity) {
        world.despawn(entity);
        return Err(reason);
    }
    vision::refresh(world);
    Ok(entity)
}
