//! Action clocks, interruptions and the single atomic skill commit point.
use super::{
    effects::{self, ImpactTarget},
    orders::{valid_enemy_life, validate_target},
    *,
};
use bevy::prelude::*;

pub(super) fn interrupt_control(world: &mut World, entity: Entity, kind: StatusKind) {
    let cancel = match kind {
        StatusKind::Stun | StatusKind::Airborne => true,
        StatusKind::Root => match world.get::<UnitAction>(entity) {
            Some(UnitAction::Casting { ability, .. }) => world
                .resource::<AbilityCatalog>()
                .get(*ability)
                .is_some_and(|d| d.execution.is_dash()),
            _ => false,
        },
        StatusKind::Slow => false,
    };
    if cancel && let Some(mut action) = world.get_mut::<UnitAction>(entity) {
        *action = UnitAction::Ready;
    }
}

pub(super) fn advance_actions(world: &mut World) {
    let mut actors: Vec<Entity> = world
        .query_filtered::<Entity, With<Unit>>()
        .iter(world)
        .collect();
    // Snapshot sequence allocation must not depend on archetype/query order.
    actors.sort_by_key(|id| id.to_bits());
    for entity in actors {
        if !world.get::<UnitState>(entity).is_some_and(UnitState::alive) {
            continue;
        }
        {
            let unit = world.get::<Unit>(entity).expect("unit");
            let (max_mana, regen) = (unit.stats.max_mana, unit.stats.mana_regen);
            let mut state = world.get_mut::<UnitState>(entity).expect("unit state");
            state.attack_cooldown = state.attack_cooldown.saturating_sub(1);
            state.mana = (state.mana + regen * TICK_SECONDS).min(max_mana);
        }
        for cooldown in &mut world
            .get_mut::<AbilitySlots>(entity)
            .expect("unit slots")
            .cooldowns
        {
            *cooldown = (*cooldown - TICK_SECONDS).max(0.0);
        }
        let action = world
            .get::<UnitAction>(entity)
            .expect("unit action")
            .clone();
        let next = match action {
            UnitAction::Ready => continue,
            UnitAction::Recovery { remaining } if remaining > TICK_SECONDS + 1e-6 => {
                UnitAction::Recovery {
                    remaining: remaining - TICK_SECONDS,
                }
            }
            UnitAction::Recovery { .. } => UnitAction::Ready,
            UnitAction::AttackBackswing {
                remaining,
                duration,
            } => {
                if remaining > 1 {
                    UnitAction::AttackBackswing {
                        remaining: remaining - 1,
                        duration,
                    }
                } else {
                    UnitAction::Ready
                }
            }
            UnitAction::AttackWindup {
                target,
                remaining,
                interval,
            } => {
                if !world
                    .get::<ControlState>(entity)
                    .is_some_and(ControlState::can_attack)
                    || !valid_enemy_life(world, entity, target)
                {
                    UnitAction::Ready
                } else if remaining > 1 {
                    UnitAction::AttackWindup {
                        target,
                        remaining: remaining - 1,
                        interval,
                    }
                } else if release_attack(world, entity, target, interval) {
                    UnitAction::AttackBackswing {
                        remaining: interval,
                        duration: interval,
                    }
                } else {
                    UnitAction::Ready
                }
            }
            UnitAction::Casting {
                slot,
                ability,
                target,
                target_life,
                remaining,
            } => {
                if remaining > TICK_SECONDS + 1e-6 {
                    UnitAction::Casting {
                        slot,
                        ability,
                        target,
                        target_life,
                        remaining: remaining - TICK_SECONDS,
                    }
                } else {
                    commit_ability(world, entity, slot, ability, target, target_life)
                }
            }
            UnitAction::Channeling {
                remaining,
                next_pulse,
                interval,
                pulse,
                recovery,
            } => {
                let remaining = remaining - TICK_SECONDS;
                let active_time = TICK_SECONDS.min(remaining + TICK_SECONDS);
                let mut next_pulse = next_pulse - active_time;
                while next_pulse <= 1e-5 {
                    let position = world.get::<UnitPose>(entity).expect("unit pose").position;
                    effects::emit(world, pulse.clone(), ImpactTarget::SelfCast, position, 0.0);
                    next_pulse += interval;
                }
                if remaining <= 1e-5 {
                    UnitAction::Recovery {
                        remaining: recovery,
                    }
                } else {
                    UnitAction::Channeling {
                        remaining,
                        next_pulse,
                        interval,
                        pulse,
                        recovery,
                    }
                }
            }
        };
        *world.get_mut::<UnitAction>(entity).expect("unit action") = next;
    }
}

fn release_attack(world: &mut World, entity: Entity, target: UnitLifeRef, interval: u32) -> bool {
    let unit = world.get::<Unit>(entity).expect("unit");
    let origin = world.get::<UnitPose>(entity).expect("unit pose").position;
    let target_position = world
        .get::<UnitPose>(target.entity())
        .expect("target pose")
        .position;
    let target_radius = world
        .get::<Unit>(target.entity())
        .expect("target")
        .stats
        .radius;
    if origin.distance(target_position)
        > unit.attributes().attack_range() + unit.stats.radius + target_radius
        || !world
            .resource::<NavigationMap>()
            .line_clear(origin, target_position, 0.05)
    {
        return false;
    }
    let definition = ImpactDefinition {
        delivery: Delivery::Projectile {
            speed: 22.0,
            radius: 0.12,
        },
        effects: vec![Effect::Damage(DamageSpec {
            kind: DamageType::Physical,
            base: 0.0,
            ad_ratio: 1.0,
            ap_ratio: 0.0,
        })],
    };
    let payload = effects::snapshot(world, entity, DamageOrigin::BasicAttack, &definition)
        .expect("finite basic attack");
    world
        .get_mut::<UnitState>(entity)
        .expect("unit state")
        .attack_cooldown = interval;
    effects::emit(world, payload, ImpactTarget::Unit(target), origin, 100.0);
    world.write_message(SimulationEvent::AttackReleased {
        source: entity,
        target: target.entity(),
    });
    true
}

fn commit_ability(
    world: &mut World,
    entity: Entity,
    slot: usize,
    id: AbilityId,
    target: AbilityTarget,
    target_life: Option<UnitLifeRef>,
) -> UnitAction {
    let Some(definition) = world.resource::<AbilityCatalog>().get(id).cloned() else {
        return UnitAction::Ready;
    };
    let validation = validate_target(world, entity, &definition, target).and_then(|()| {
        if target_life.is_some_and(|life| !life.is_current(world)) {
            return Err(Rejection::InvalidTarget);
        }
        if !world
            .get::<ControlState>(entity)
            .is_some_and(|c| c.permits(&definition))
        {
            return Err(Rejection::Controlled);
        }
        if world.get::<UnitState>(entity).expect("unit state").mana < definition.mana_cost {
            Err(Rejection::InsufficientMana)
        } else {
            Ok(())
        }
    });
    if let Err(reason) = validation {
        world.write_message(SimulationEvent::CommandRejected {
            unit: entity,
            reason,
        });
        return UnitAction::Ready;
    }
    let impact = match &definition.execution {
        AbilityExecution::Impact(impact) | AbilityExecution::Channel { pulse: impact, .. } => {
            let Some(snapshot) =
                effects::snapshot(world, entity, DamageOrigin::Ability(id), impact)
            else {
                world.write_message(SimulationEvent::CommandRejected {
                    unit: entity,
                    reason: Rejection::InvalidDamage,
                });
                return UnitAction::Ready;
            };
            Some(snapshot)
        }
        AbilityExecution::Dash { .. } => None,
    };
    // All targets, permissions and every effect snapshot are valid before costs.
    world.get_mut::<UnitState>(entity).expect("unit state").mana -= definition.mana_cost;
    world
        .get_mut::<AbilitySlots>(entity)
        .expect("unit slots")
        .cooldowns[slot] = definition.cooldown;
    let origin = world.get::<UnitPose>(entity).expect("unit pose").position;
    let point = match target {
        AbilityTarget::Point(p) => p,
        AbilityTarget::Unit(t) => world.get::<UnitPose>(t).expect("validated pose").position,
        _ => origin,
    };
    world.write_message(SimulationEvent::AbilityCommitted {
        unit: entity,
        ability: id,
        position: point,
    });
    let recovery = UnitAction::Recovery {
        remaining: definition.recovery,
    };
    match definition.execution {
        AbilityExecution::Impact(_) => {
            let target = match target {
                AbilityTarget::Unit(_) => {
                    ImpactTarget::Unit(target_life.expect("locked enemy life"))
                }
                AbilityTarget::Point(point) => ImpactTarget::Point(point),
                AbilityTarget::Direction(direction) => ImpactTarget::Direction(direction),
                AbilityTarget::SelfCast => ImpactTarget::SelfCast,
            };
            effects::emit(
                world,
                impact.expect("prepared impact"),
                target,
                origin,
                definition.range,
            );
            recovery
        }
        AbilityExecution::Dash { speed } => {
            super::movement::start_dash(world, entity, point, speed);
            recovery
        }
        AbilityExecution::Channel {
            duration, interval, ..
        } => UnitAction::Channeling {
            remaining: duration,
            next_pulse: interval,
            interval,
            pulse: impact.expect("prepared pulse"),
            recovery: definition.recovery,
        },
    }
}
