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

pub(super) fn valid_enemy_life(world: &World, source: Entity, target: UnitLifeRef) -> bool {
    target.is_current(world) && valid_enemy(world, source, target.entity())
}

pub(super) fn resolve_target(
    world: &World,
    unit: Entity,
    definition: &AbilityDefinition,
    target: AbilityTarget,
) -> Result<AbilityTarget, Rejection> {
    let origin = world
        .get::<UnitPose>(unit)
        .ok_or(Rejection::InvalidUnit)?
        .position;
    let point = match (definition.targeting, target) {
        (Targeting::Direction, AbilityTarget::Direction(direction))
            if direction.is_finite() && direction.length_squared() > 1e-6 =>
        {
            return Ok(target);
        }
        (Targeting::Point, AbilityTarget::Point(point)) if point.is_finite() => {
            if definition.execution.is_dash() {
                // Preserve direction for finite clicks whose f32 length overflows.
                let offset = point.as_dvec2() - origin.as_dvec2();
                let distance = offset.length();
                let range = f64::from(definition.range);
                let destination = if distance > range {
                    (origin.as_dvec2() + offset * (range / distance)).as_vec2()
                } else {
                    point
                };
                return Ok(AbilityTarget::Point(destination));
            }
            point
        }
        (Targeting::Enemy, AbilityTarget::Unit(target))
            if valid_enemy(world, unit, target)
                && world
                    .get::<Unit>(target)
                    .is_some_and(|unit| unit.class() == UnitClass::Actor) =>
        {
            world
                .get::<UnitPose>(target)
                .ok_or(Rejection::InvalidTarget)?
                .position
        }
        (Targeting::SelfCast, AbilityTarget::SelfCast) => return Ok(target),
        _ => return Err(Rejection::InvalidTarget),
    };
    if origin.distance(point) > definition.range + 0.001 {
        return Err(Rejection::OutOfRange);
    }
    Ok(target)
}

fn apply_command(world: &mut World, entity: Entity, command: UnitCommand) -> Result<(), Rejection> {
    let state = world
        .get::<UnitState>(entity)
        .ok_or(Rejection::InvalidUnit)?;
    if !state.alive() {
        return Err(Rejection::Dead);
    }
    if !matches!(command, UnitCommand::Stop)
        && world
            .get::<Movement>(entity)
            .is_some_and(|m| m.dash.is_some())
    {
        return Err(Rejection::Busy);
    }
    match command {
        UnitCommand::MoveTo(point) | UnitCommand::AttackMove(point) => {
            if !world
                .get::<ControlState>(entity)
                .is_some_and(ControlState::can_move)
            {
                return Err(Rejection::Controlled);
            }
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
            if !world
                .get::<ControlState>(entity)
                .is_some_and(ControlState::can_attack)
            {
                return Err(Rejection::Controlled);
            }
            if !valid_enemy(world, entity, target) {
                return Err(Rejection::InvalidTarget);
            }
            let target_life = world
                .get::<UnitState>(target)
                .expect("validated target state")
                .life_ref(target);
            if matches!(world.get::<UnitOrder>(entity), Some(UnitOrder::AttackTarget(current)) if *current == target_life)
                && matches!(
                    world.get::<UnitAction>(entity),
                    Some(
                        UnitAction::Ready
                            | UnitAction::AttackWindup { .. }
                            | UnitAction::AttackBackswing { .. }
                    )
                )
            {
                return Ok(());
            }
            *world
                .get_mut::<UnitOrder>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitOrder::AttackTarget(target_life);
            *world
                .get_mut::<UnitAction>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitAction::Ready;
            super::movement::clear_route(
                &mut *world
                    .get_mut::<Movement>(entity)
                    .ok_or(Rejection::InvalidUnit)?,
            );
        }
        UnitCommand::CastAbility { slot, target } => {
            super::actions::begin_cast(world, entity, CastSource::AbilitySlot(slot), target)?;
        }
        UnitCommand::UseEquipment {
            slot,
            instance,
            target,
        } => {
            super::actions::begin_cast(
                world,
                entity,
                CastSource::EquipmentSlot { slot, instance },
                target,
            )?;
        }
        UnitCommand::Stop => {
            *world
                .get_mut::<UnitOrder>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitOrder::Idle;
            *world
                .get_mut::<UnitAction>(entity)
                .ok_or(Rejection::InvalidUnit)? = UnitAction::Ready;
            super::movement::clear_route(
                &mut *world
                    .get_mut::<Movement>(entity)
                    .ok_or(Rejection::InvalidUnit)?,
            );
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

type OrderActor<'a> = (
    Entity,
    &'a Unit,
    &'a UnitState,
    &'a UnitPose,
    &'a ControlState,
    &'a mut UnitOrder,
    &'a mut UnitAction,
    &'a mut Movement,
);

pub(super) fn resolve_orders(
    map: Res<NavigationMap>,
    targets: Query<(Entity, &Unit, &UnitState, &UnitPose)>,
    mut actors: Query<OrderActor<'_>>,
    mut events: MessageWriter<SimulationEvent>,
) {
    for (entity, unit, state, pose, control, mut order, mut action, mut movement) in &mut actors {
        if !state.alive()
            || !matches!(*action, UnitAction::Ready)
            || movement.dash.is_some()
            || movement.forced.is_some()
            || (!control.can_move() && !control.can_attack())
        {
            continue;
        }
        movement.repath_remaining = (movement.repath_remaining - TICK_SECONDS).max(0.0);
        let valid = |target: UnitLifeRef| {
            targets
                .get(target.entity())
                .ok()
                .filter(|(_, other, life, _)| {
                    other.team != unit.team
                        && life.alive()
                        && life.life_ref(target.entity()) == target
                })
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
                        p.position.distance(pose.position) > unit.attributes().attack_range() + 6.0
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
                                    <= unit.attributes().attack_range() + 3.0
                        })
                        .min_by(|a, b| {
                            a.3.position
                                .distance_squared(pose.position)
                                .total_cmp(&b.3.position.distance_squared(pose.position))
                                .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
                        })
                        .map(|(id, _, state, _)| state.life_ref(id));
                }
                *target
            }
            _ => None,
        };
        let goal = if let Some(target) = target {
            let Ok((_, other, _, target_pose)) = targets.get(target.entity()) else {
                continue;
            };
            let reach = unit.attributes().attack_range() + unit.stats.radius + other.stats.radius;
            if pose.position.distance(target_pose.position) <= reach
                && map.line_clear(pose.position, target_pose.position, 0.05)
            {
                movement.route.clear();
                movement.goal = None;
                if control.can_attack()
                    && state.attack_cooldown == 0
                    && let Some((windup, interval)) = unit
                        .attributes()
                        .attack_timing(unit.stats.attack_windup_fraction)
                {
                    *action = UnitAction::AttackWindup {
                        target,
                        remaining: windup,
                        interval,
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
        if !control.can_move() {
            super::movement::clear_route(&mut movement);
            continue;
        }
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
