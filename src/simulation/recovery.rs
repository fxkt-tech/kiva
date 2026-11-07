//! Constrained fixed-tick restoration. Modes select recipients; only this
//! boundary writes their restored resources after checking both current lives.
use super::*;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecoverySpec {
    pub radius: f32,
    pub health_fraction_per_second: f32,
    pub mana_fraction_per_second: f32,
    pub out_of_combat_ticks: u32,
}
impl RecoverySpec {
    pub fn valid(self) -> bool {
        self.radius.is_finite()
            && self.radius > 0.0
            && [
                self.health_fraction_per_second,
                self.mana_fraction_per_second,
            ]
            .into_iter()
            .all(|rate| rate.is_finite() && (0.0..=1.0).contains(&rate))
            && (self.health_fraction_per_second > 0.0 || self.mana_fraction_per_second > 0.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RecoveryAmount {
    pub health: f32,
    pub mana: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryError {
    InvalidDefinition,
    Paused,
    InvalidSource,
    InvalidTarget,
    WrongTeam,
    OutOfRange,
    InCombat,
    AlreadyRestored,
}

/// Applies at most one fixed tick's recovery to this target life. The caller
/// chooses eligible mode roles (for example heroes); it cannot choose an amount
/// or bypass the source/target/range/combat checks. Repeated calls in the same
/// tick, even through different zones, cannot multiply recovery.
pub fn restore_in_zone(
    world: &mut World,
    source: UnitLifeRef,
    target: UnitLifeRef,
    spec: RecoverySpec,
) -> Result<RecoveryAmount, RecoveryError> {
    if !spec.valid() {
        return Err(RecoveryError::InvalidDefinition);
    }
    if world.resource::<SimulationControl>().paused {
        return Err(RecoveryError::Paused);
    }
    if !source.is_current(world)
        || world
            .get::<Unit>(source.entity())
            .is_none_or(|unit| unit.class() != UnitClass::Structure)
    {
        return Err(RecoveryError::InvalidSource);
    }
    if !target.is_current(world)
        || world
            .get::<Unit>(target.entity())
            .is_none_or(|unit| unit.class() != UnitClass::Actor)
    {
        return Err(RecoveryError::InvalidTarget);
    }
    let source_unit = world
        .get::<Unit>(source.entity())
        .expect("validated source");
    let target_unit = world
        .get::<Unit>(target.entity())
        .expect("validated target");
    if source_unit.team() != target_unit.team() {
        return Err(RecoveryError::WrongTeam);
    }
    let origin = world
        .get::<UnitPose>(source.entity())
        .ok_or(RecoveryError::InvalidSource)?
        .position();
    let point = world
        .get::<UnitPose>(target.entity())
        .ok_or(RecoveryError::InvalidTarget)?
        .position();
    if (point.as_dvec2() - origin.as_dvec2()).length() > f64::from(spec.radius) {
        return Err(RecoveryError::OutOfRange);
    }
    let tick = world.resource::<SimulationTick>().0;
    let state = world
        .get::<UnitState>(target.entity())
        .expect("validated target");
    if state
        .last_damaged_tick
        .is_some_and(|damaged| tick.saturating_sub(damaged) < u64::from(spec.out_of_combat_ticks))
    {
        return Err(RecoveryError::InCombat);
    }
    if state.last_recovery_tick == Some(tick) {
        return Err(RecoveryError::AlreadyRestored);
    }
    let definition = target_unit.definition();
    let next = |current: f32, maximum: f32, rate: f32| {
        (f64::from(current) + f64::from(maximum) * f64::from(rate) / TICK_RATE)
            .min(f64::from(maximum)) as f32
    };
    let health = next(
        state.health,
        definition.max_health,
        spec.health_fraction_per_second,
    );
    let mana = next(
        state.mana,
        definition.max_mana,
        spec.mana_fraction_per_second,
    );
    let amount = RecoveryAmount {
        health: health - state.health,
        mana: mana - state.mana,
    };
    let mut state = world
        .get_mut::<UnitState>(target.entity())
        .expect("validated target");
    state.health = health;
    state.mana = mana;
    state.last_recovery_tick = Some(tick);
    Ok(amount)
}

/// Natural regeneration is immutable unit content and follows the same fixed
/// clock as other recovery. It cannot resurrect a dead life.
pub(super) fn regenerate_health(world: &mut World, entity: Entity) {
    let unit = world.get::<Unit>(entity).expect("unit");
    let (maximum, rate) = (unit.stats.max_health, unit.stats.health_regen);
    if rate == 0.0 {
        return;
    }
    let mut state = world.get_mut::<UnitState>(entity).expect("unit state");
    if state.alive() {
        state.health = (state.health + rate * TICK_SECONDS).min(maximum);
    }
}
