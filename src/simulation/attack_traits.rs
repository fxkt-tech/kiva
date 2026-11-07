//! Attack traits share ordinary attack identity; chained item effects never recurse.
use super::{mechanics, *};
use bevy::prelude::*;

pub(super) fn released(world: &mut World, entity: Entity, target: UnitLifeRef) {
    let slots = world.get::<AbilitySlots>(entity).expect("slots").ids;
    let spec = slots
        .iter()
        .flatten()
        .filter_map(|id| world.resource::<AbilityCatalog>().get(*id))
        .find_map(|definition| match definition.execution {
            AbilityExecution::Empower(spec) => Some(spec),
            _ => None,
        });
    let Some(spec) = spec else {
        return;
    };
    let is_structure = world
        .get::<Unit>(target.entity())
        .is_some_and(|unit| unit.class() == UnitClass::Structure);
    let tick = world.resource::<SimulationTick>().0;
    let mut state = world.get_mut::<UnitMechanics>(entity).expect("mechanics");
    if state.empowered() {
        return;
    }
    if !is_structure {
        state.focus = state.focus.saturating_add(1).min(spec.required_stacks);
    }
    if state.focus > 0 {
        state.focus_until = tick + mechanics::ticks(spec.stack_duration);
    }
}
pub(super) fn prepare_hit(
    world: &World,
    target: UnitLifeRef,
    payload: &ImpactSnapshot,
) -> Vec<(usize, DamageSnapshot, f32)> {
    if payload.origin != DamageOrigin::BasicAttack || !target.is_current(world) {
        return vec![];
    }
    let unit = world.get::<Unit>(target.entity()).expect("target");
    if unit.class() == UnitClass::Structure {
        return vec![];
    }
    let health = world
        .get::<UnitState>(target.entity())
        .expect("target state")
        .health;
    let tick = world.resource::<SimulationTick>().0;
    payload
        .unit_passives
        .iter()
        .enumerate()
        .filter_map(|(index, passive)| {
            let AttackPassive::CurrentHealth {
                ratio,
                minimum,
                non_hero_cap,
                target_cooldown,
            } = *passive
            else {
                return None;
            };
            if world
                .resource::<mechanics::MechanicMemory>()
                .cooldowns
                .get(&(payload.source, index, target))
                .is_some_and(|until| *until > tick)
            {
                return None;
            }
            let mut raw = (health * ratio).max(minimum);
            if unit.stats.kind != UnitKind::Hero {
                raw = raw.min(non_hero_cap);
            }
            Some((
                index,
                DamageSnapshot::from_raw(
                    raw,
                    DamageType::Physical,
                    DamageOrigin::UnitPassive { passive: index },
                ),
                target_cooldown,
            ))
        })
        .collect()
}
pub(super) fn finish_hit(
    world: &mut World,
    target: UnitLifeRef,
    payload: &ImpactSnapshot,
    passives: Vec<(usize, DamageSnapshot, f32)>,
) {
    let tick = world.resource::<SimulationTick>().0;
    for (index, damage, cooldown) in passives {
        // A lethal body does not execute on-hit effects, but still consumes the
        // target-specific activation that belonged to this committed attack.
        world
            .resource_mut::<mechanics::MechanicMemory>()
            .cooldowns
            .insert(
                (payload.source, index, target),
                tick + mechanics::ticks(cooldown),
            );
        if target.is_current(world) {
            super::combat::apply_damage(world, payload.source, target, damage);
        }
    }
}
