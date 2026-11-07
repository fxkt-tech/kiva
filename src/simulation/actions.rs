//! Action clocks and the single commit point for ability costs and cooldowns.
use super::{
    model::Movement,
    orders::{valid_enemy, validate_target},
    *,
};
use bevy::prelude::*;

pub(super) fn advance_actions(world: &mut World) {
    let actors: Vec<Entity> = world
        .query_filtered::<Entity, With<Unit>>()
        .iter(world)
        .collect();
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
                if !valid_enemy(world, entity, target) {
                    UnitAction::Ready
                } else if remaining > 1 {
                    UnitAction::AttackWindup {
                        target,
                        remaining: remaining - 1,
                        interval,
                    }
                } else {
                    if release_attack(world, entity, target, interval) {
                        UnitAction::AttackBackswing {
                            remaining: interval,
                            duration: interval,
                        }
                    } else {
                        UnitAction::Ready
                    }
                }
            }
            UnitAction::Casting {
                slot,
                ability,
                target,
                remaining,
            } => {
                if remaining > TICK_SECONDS + 1e-6 {
                    UnitAction::Casting {
                        slot,
                        ability,
                        target,
                        remaining: remaining - TICK_SECONDS,
                    }
                } else {
                    commit_ability(world, entity, slot, ability, target)
                }
            }
            UnitAction::Channeling {
                remaining,
                next_pulse,
                interval,
                radius,
                damage,
                recovery,
            } => {
                let remaining = remaining - TICK_SECONDS;
                let active_time = TICK_SECONDS.min(remaining + TICK_SECONDS);
                let mut next_pulse = next_pulse - active_time;
                while next_pulse <= 1e-5 {
                    let pose = world.get::<UnitPose>(entity).expect("unit pose").position;
                    let team = world.get::<Unit>(entity).expect("unit").team;
                    world.spawn(AreaEffect {
                        source: entity,
                        team,
                        position: pose,
                        radius,
                        remaining: 0.0,
                        duration: 0.0,
                        damage,
                    });
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
                        radius,
                        damage,
                        recovery,
                    }
                }
            }
        };
        *world.get_mut::<UnitAction>(entity).expect("unit action") = next;
    }
}

fn release_attack(world: &mut World, entity: Entity, target: Entity, interval: u32) -> bool {
    let unit = world.get::<Unit>(entity).expect("unit");
    let attributes = *unit.attributes();
    let radius = unit.stats.radius;
    let team = unit.team;
    let origin = world.get::<UnitPose>(entity).expect("unit pose").position;
    let target_position = world.get::<UnitPose>(target).expect("target pose").position;
    let target_radius = world.get::<Unit>(target).expect("target").stats.radius;
    if origin.distance(target_position) > attributes.attack_range() + radius + target_radius
        || !world
            .resource::<NavigationMap>()
            .line_clear(origin, target_position, 0.05)
    {
        return false;
    }
    let damage = DamageSpec {
        kind: DamageType::Physical,
        base: 0.0,
        ad_ratio: 1.0,
        ap_ratio: 0.0,
    }
    .snapshot(&attributes, DamageOrigin::BasicAttack)
    .expect("finite attack damage");
    world
        .get_mut::<UnitState>(entity)
        .expect("unit state")
        .attack_cooldown = interval;
    world.spawn(Projectile {
        source: entity,
        team,
        position: origin,
        previous: origin,
        direction: (target_position - origin).normalize_or_zero(),
        remaining: 100.0,
        speed: 22.0,
        radius: 0.12,
        damage,
        homing: Some(target),
    });
    world.write_message(SimulationEvent::AttackReleased {
        source: entity,
        target,
    });
    true
}

fn commit_ability(
    world: &mut World,
    entity: Entity,
    slot: usize,
    id: AbilityId,
    target: AbilityTarget,
) -> UnitAction {
    let Some(definition) = world.resource::<AbilityCatalog>().get(id).cloned() else {
        return UnitAction::Ready;
    };
    let validation = validate_target(world, entity, &definition, target).and_then(|()| {
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
    let damage_snapshot = match &definition.effect {
        AbilityEffect::Projectile { damage, .. }
        | AbilityEffect::Area { damage, .. }
        | AbilityEffect::Channel { damage, .. } => {
            let Some(snapshot) = damage.snapshot(
                world.get::<Unit>(entity).expect("unit").attributes(),
                DamageOrigin::Ability(id),
            ) else {
                world.write_message(SimulationEvent::CommandRejected {
                    unit: entity,
                    reason: Rejection::InvalidDamage,
                });
                return UnitAction::Ready;
            };
            Some(snapshot)
        }
        AbilityEffect::Dash { .. } => None,
    };
    // This is the only commit point. Cancellation before it spends nothing;
    // interruption after it never refunds costs or removes released effects.
    world.get_mut::<UnitState>(entity).expect("unit state").mana -= definition.mana_cost;
    world
        .get_mut::<AbilitySlots>(entity)
        .expect("unit slots")
        .cooldowns[slot] = definition.cooldown;
    let origin = world.get::<UnitPose>(entity).expect("unit pose").position;
    let team = world.get::<Unit>(entity).expect("unit").team;
    let point = match target {
        AbilityTarget::Point(p) => p,
        AbilityTarget::Unit(t) => {
            world
                .get::<UnitPose>(t)
                .expect("validated target pose")
                .position
        }
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
    match definition.effect {
        AbilityEffect::Projectile {
            speed,
            radius,
            damage: _,
        } => {
            let AbilityTarget::Direction(direction) = target else {
                unreachable!("registered projectile targeting");
            };
            world.spawn(Projectile {
                source: entity,
                team,
                position: origin,
                previous: origin,
                direction: direction.normalize(),
                remaining: definition.range,
                speed,
                radius,
                damage: damage_snapshot.expect("damaging effect snapshot"),
                homing: None,
            });
            recovery
        }
        AbilityEffect::Area {
            radius,
            delay,
            damage: _,
        } => {
            world.spawn(AreaEffect {
                source: entity,
                team,
                position: point,
                radius,
                remaining: delay,
                duration: delay,
                damage: damage_snapshot.expect("damaging effect snapshot"),
            });
            recovery
        }
        AbilityEffect::Dash { speed } => {
            world
                .get_mut::<Movement>(entity)
                .expect("unit movement")
                .dash = Some((point, speed));
            recovery
        }
        AbilityEffect::Channel {
            duration,
            interval,
            radius,
            damage: _,
        } => UnitAction::Channeling {
            remaining: duration,
            next_pulse: interval,
            interval,
            radius,
            damage: damage_snapshot.expect("damaging effect snapshot"),
            recovery: definition.recovery,
        },
    }
}
