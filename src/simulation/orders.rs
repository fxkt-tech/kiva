//! Validates intentions before replacing an existing order. Invalid input has no
//! partial effects: it cannot cancel a cast, consume mana, or erase a valid route.
use super::{model::Movement, *};
use bevy::prelude::*;

pub(super) fn valid_enemy(world: &World, source: Entity, target: Entity) -> bool {
    match (
        world.get::<Unit>(source),
        world.get::<Unit>(target),
        world.get::<UnitState>(target),
    ) {
        (Some(a), Some(b), Some(state)) => a.team != b.team && state.alive(),
        _ => false,
    }
}

pub(super) fn validate_target(
    world: &World,
    unit: Entity,
    definition: &AbilityDefinition,
    target: AbilityTarget,
) -> Result<(), Rejection> {
    let origin = world
        .get::<UnitPose>(unit)
        .ok_or(Rejection::InvalidUnit)?
        .position;
    let point = match (definition.targeting, target) {
        (Targeting::Direction, AbilityTarget::Direction(direction))
            if direction.is_finite() && direction.length_squared() > 1e-6 =>
        {
            return Ok(());
        }
        (Targeting::Point, AbilityTarget::Point(point)) if point.is_finite() => point,
        (Targeting::Enemy, AbilityTarget::Unit(target)) if valid_enemy(world, unit, target) => {
            world
                .get::<UnitPose>(target)
                .ok_or(Rejection::InvalidTarget)?
                .position
        }
        (Targeting::SelfCast, AbilityTarget::SelfCast) => return Ok(()),
        _ => return Err(Rejection::InvalidTarget),
    };
    if origin.distance(point) > definition.range + 0.001 {
        return Err(Rejection::OutOfRange);
    }
    Ok(())
}

fn apply_command(world: &mut World, entity: Entity, command: UnitCommand) -> Result<(), Rejection> {
    let state = world
        .get::<UnitState>(entity)
        .ok_or(Rejection::InvalidUnit)?;
    if !state.alive() {
        return Err(Rejection::Dead);
    }
    if world
        .get::<Movement>(entity)
        .is_some_and(|m| m.dash.is_some())
    {
        return Err(Rejection::Busy);
    }
    match command {
        UnitCommand::MoveTo(point) | UnitCommand::AttackMove(point) => {
            let pose = world
                .get::<UnitPose>(entity)
                .ok_or(Rejection::InvalidUnit)?;
            let radius = world
                .get::<Unit>(entity)
                .ok_or(Rejection::InvalidUnit)?
                .stats
                .radius;
            let path = world
                .resource::<NavigationMap>()
                .path(pose.position, point, radius)
                .map_err(|_| Rejection::NoPath)?;
            let destination = *path.last().ok_or(Rejection::NoPath)?;
            let mut movement = world
                .get_mut::<Movement>(entity)
                .ok_or(Rejection::InvalidUnit)?;
            movement.route = path.into();
            movement.goal = Some(destination);
            movement.repath_remaining = 0.2;
            *world
                .get_mut::<UnitOrder>(entity)
                .ok_or(Rejection::InvalidUnit)? = match command {
                UnitCommand::MoveTo(_) => UnitOrder::MoveTo(destination),
                _ => UnitOrder::AttackMove {
                    destination,
                    target: None,
                },
            };
            *world
                .get_mut::<UnitAction>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitAction::Ready;
            world.write_message(SimulationEvent::MoveOrdered {
                unit: entity,
                destination,
            });
        }
        UnitCommand::AttackTarget(target) => {
            if !valid_enemy(world, entity, target) {
                return Err(Rejection::InvalidTarget);
            }
            if matches!(world.get::<UnitOrder>(entity), Some(UnitOrder::AttackTarget(current)) if *current == target)
                && matches!(
                    world.get::<UnitAction>(entity),
                    Some(UnitAction::Ready | UnitAction::AttackWindup { .. })
                )
            {
                return Ok(());
            }
            *world
                .get_mut::<UnitOrder>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitOrder::AttackTarget(target);
            *world
                .get_mut::<UnitAction>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitAction::Ready;
            *world
                .get_mut::<Movement>(entity)
                .ok_or(Rejection::InvalidUnit)? = Movement::default();
        }
        UnitCommand::CastAbility { slot, target } => {
            let slots = world
                .get::<AbilitySlots>(entity)
                .ok_or(Rejection::NoAbility)?;
            let id = slots.ability(slot).ok_or(Rejection::NoAbility)?;
            if slots.cooldowns[slot] > 1e-5 {
                return Err(Rejection::OnCooldown);
            }
            let definition = world
                .resource::<AbilityCatalog>()
                .get(id)
                .ok_or(Rejection::NoAbility)?;
            if state.mana < definition.mana_cost {
                return Err(Rejection::InsufficientMana);
            }
            if !matches!(
                world.get::<UnitAction>(entity),
                Some(
                    UnitAction::Ready
                        | UnitAction::Recovery { .. }
                        | UnitAction::AttackWindup { .. }
                )
            ) {
                return Err(Rejection::Busy);
            }
            validate_target(world, entity, definition, target)?;
            let windup = definition.windup;
            *world
                .get_mut::<UnitAction>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitAction::Casting {
                slot,
                ability: id,
                target,
                remaining: windup,
            };
            *world
                .get_mut::<Movement>(entity)
                .ok_or(Rejection::InvalidUnit)? = Movement::default();
        }
        UnitCommand::Stop => {
            *world
                .get_mut::<UnitOrder>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitOrder::Idle;
            *world
                .get_mut::<UnitAction>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitAction::Ready;
            *world
                .get_mut::<Movement>(entity)
                .ok_or(Rejection::InvalidUnit)? = Movement::default();
        }
    }
    Ok(())
}

pub(super) fn consume_commands(world: &mut World) {
    let commands: Vec<_> = world.resource_mut::<CommandQueue>().drain().collect();
    for (unit, command) in commands {
        if let Err(reason) = apply_command(world, unit, command) {
            world.write_message(SimulationEvent::CommandRejected { unit, reason });
        }
    }
}

pub(super) fn resolve_orders(
    map: Res<NavigationMap>,
    targets: Query<(Entity, &Unit, &UnitState, &UnitPose)>,
    mut actors: Query<(
        Entity,
        &Unit,
        &UnitState,
        &UnitPose,
        &mut UnitOrder,
        &mut UnitAction,
        &mut Movement,
    )>,
    mut events: MessageWriter<SimulationEvent>,
) {
    for (entity, unit, state, pose, mut order, mut action, mut movement) in &mut actors {
        if !state.alive() || !matches!(*action, UnitAction::Ready) || movement.dash.is_some() {
            continue;
        }
        movement.repath_remaining = (movement.repath_remaining - TICK_SECONDS).max(0.0);
        let valid = |target: Entity| {
            targets
                .get(target)
                .ok()
                .filter(|(_, other, life, _)| other.team != unit.team && life.alive())
        };
        let target = match &mut *order {
            UnitOrder::AttackTarget(target) => {
                if valid(*target).is_none() {
                    *order = UnitOrder::Idle;
                    movement.route.clear();
                    movement.goal = None;
                    continue;
                }
                Some(*target)
            }
            UnitOrder::AttackMove { target, .. } => {
                if target.is_some_and(|id| {
                    valid(id).is_none_or(|(_, _, _, p)| {
                        p.position.distance(pose.position) > unit.stats.attack_range + 6.0
                    })
                }) {
                    *target = None;
                }
                if target.is_none() {
                    *target = targets
                        .iter()
                        .filter(|(_, other, life, p)| {
                            other.team != unit.team
                                && life.alive()
                                && p.position.distance(pose.position)
                                    <= unit.stats.attack_range + 3.0
                        })
                        .min_by(|a, b| {
                            a.3.position
                                .distance_squared(pose.position)
                                .total_cmp(&b.3.position.distance_squared(pose.position))
                                .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
                        })
                        .map(|(id, _, _, _)| id);
                }
                *target
            }
            _ => None,
        };
        let goal = if let Some(target) = target {
            let Ok((_, other, _, target_pose)) = targets.get(target) else {
                continue;
            };
            let reach = unit.stats.attack_range + unit.stats.radius + other.stats.radius;
            if pose.position.distance(target_pose.position) <= reach
                && map.line_clear(pose.position, target_pose.position, 0.05)
            {
                movement.route.clear();
                movement.goal = None;
                if state.attack_cooldown <= 1e-5 {
                    *action = UnitAction::AttackWindup {
                        target,
                        remaining: unit.stats.attack_windup,
                    };
                }
                continue;
            }
            Some(target_pose.position)
        } else {
            match *order {
                UnitOrder::MoveTo(p) | UnitOrder::AttackMove { destination: p, .. } => Some(p),
                _ => None,
            }
        };
        let Some(goal) = goal else {
            movement.route.clear();
            movement.goal = None;
            continue;
        };
        if pose.position.distance(goal) < 0.04 {
            *order = UnitOrder::Idle;
            movement.route.clear();
            movement.goal = None;
            continue;
        }
        if (movement.route.is_empty() || movement.goal.is_none_or(|old| old.distance(goal) > 0.6))
            && movement.repath_remaining <= 0.0
        {
            match map.path(pose.position, goal, unit.stats.radius) {
                Ok(path) => {
                    movement.route = path.into();
                    movement.goal = Some(goal);
                    movement.repath_remaining = 0.2;
                }
                Err(_) => {
                    *order = UnitOrder::Idle;
                    movement.route.clear();
                    movement.goal = None;
                    events.write(SimulationEvent::CommandRejected {
                        unit: entity,
                        reason: Rejection::NoPath,
                    });
                }
            }
        }
    }
}
