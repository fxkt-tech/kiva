//! One-shot unit-targeted purposes. Walking stays in movement; beginning and
//! committing a cast stay in actions. No rule here depends on a content ID.
use super::{model::Movement, *};
use bevy::prelude::*;

pub(super) fn request(
    world: &mut World,
    unit: Entity,
    source: CastSource,
    target: AbilityTarget,
) -> Result<(), Rejection> {
    match preview_cast(world, unit, source, target) {
        Ok(_) => {
            actions::begin_cast(world, unit, source, target)?;
            if matches!(
                world.get::<UnitOrder>(unit),
                Some(UnitOrder::CastTarget { .. })
            ) {
                clear(world, unit);
            }
            Ok(())
        }
        Err(Rejection::OutOfRange) => {
            let AbilityTarget::Unit(target) = target else {
                return Err(Rejection::OutOfRange);
            };
            if !world
                .get::<ControlState>(unit)
                .is_some_and(ControlState::can_move)
            {
                return Err(Rejection::Controlled);
            }
            if is_dashing(world, unit) {
                return Err(Rejection::Busy);
            }
            let (ability, _, _) = actions::cast_binding(world, unit, source)?;
            let definition = world
                .resource::<AbilityCatalog>()
                .get(ability)
                .ok_or(Rejection::NoAbility)?;
            if definition.targeting != Targeting::Enemy {
                return Err(Rejection::OutOfRange);
            }
            let life = world
                .get::<UnitState>(target)
                .ok_or(Rejection::InvalidTarget)?
                .life_ref(target);
            if matches!(world.get::<UnitOrder>(unit), Some(UnitOrder::CastTarget { source: old_source, ability: old_ability, target: old_target })
                if *old_source == source && *old_ability == ability && *old_target == life)
            {
                return Ok(());
            }
            let target_position = world
                .get::<UnitPose>(target)
                .ok_or(Rejection::InvalidTarget)?
                .position;
            // Complete navigation before replacing a valid purpose or action.
            let route = plan(world, unit, target_position, definition.range)?;
            set_route(world, unit, target_position, route);
            *world.get_mut::<UnitOrder>(unit).expect("validated order") = UnitOrder::CastTarget {
                source,
                ability,
                target: life,
            };
            *world.get_mut::<UnitAction>(unit).expect("validated action") = UnitAction::Ready;
            Ok(())
        }
        Err(reason) => Err(reason),
    }
}

fn plan(world: &World, unit: Entity, target: Vec2, range: f32) -> Result<Vec<Vec2>, Rejection> {
    let pose = world.get::<UnitPose>(unit).ok_or(Rejection::InvalidUnit)?;
    let radius = world
        .get::<Unit>(unit)
        .ok_or(Rejection::InvalidUnit)?
        .stats
        .radius;
    let map = world.resource::<NavigationMap>();
    let mut route = map
        .path_to_range(pose.position, target, (range - 0.001).max(0.0), radius)
        .map_err(|_| Rejection::NoPath)?;
    if let Some(&endpoint) = route.last() {
        // A moving enemy can leave the old range boundary during this very
        // movement tick. Keep a safe inward continuation instead of stopping
        // there forever; advance() still checks the actual range every tick.
        // Sweep preserves a valid cast position when the center is unreachable.
        let continuation = map.sweep(endpoint, target, radius);
        if continuation.distance_squared(endpoint) > 1e-6 {
            route.push(continuation);
        }
    }
    Ok(route)
}

fn set_route(world: &mut World, unit: Entity, target: Vec2, route: Vec<Vec2>) {
    let mut movement = world.get_mut::<Movement>(unit).expect("unit movement");
    movement.route = route.into();
    movement.goal = Some(target);
    movement.repath_remaining = 0.2;
}

fn clear(world: &mut World, unit: Entity) {
    *world.get_mut::<UnitOrder>(unit).expect("unit order") = UnitOrder::Idle;
    movement::clear_route(&mut world.get_mut::<Movement>(unit).expect("unit movement"));
}

fn reject(world: &mut World, unit: Entity, reason: Rejection) {
    clear(world, unit);
    world.write_message(SimulationEvent::CommandRejected { unit, reason });
}

pub(super) fn cancel_equipment(world: &mut World, unit: Entity, instance: EquipmentInstanceId) {
    if matches!(world.get::<UnitOrder>(unit), Some(UnitOrder::CastTarget {
        source: CastSource::EquipmentSlot { instance: current, .. }, ..
    }) if *current == instance)
    {
        clear(world, unit);
    }
}

pub(super) fn advance(world: &mut World) {
    let mut pending: Vec<_> = world
        .query::<(Entity, &UnitOrder)>()
        .iter(world)
        .filter_map(|(unit, order)| match *order {
            UnitOrder::CastTarget {
                source,
                ability,
                target,
            } => Some((unit, source, ability, target)),
            _ => None,
        })
        .collect();
    pending.sort_by_key(|(unit, ..)| unit.to_bits());
    for (unit, source, ability, target) in pending {
        if !target.is_current(world) {
            reject(world, unit, Rejection::InvalidTarget);
            continue;
        }
        if !matches!(actions::cast_binding(world, unit, source), Ok((current, ..)) if current == ability)
        {
            reject(world, unit, Rejection::NoAbility);
            continue;
        }
        match preview_cast(world, unit, source, AbilityTarget::Unit(target.entity())) {
            Ok(_) => {
                // Clearing the one-shot purpose before windup prevents retries
                // after a later interrupt or target escaping at commit.
                clear(world, unit);
                if let Err(reason) =
                    actions::begin_cast(world, unit, source, AbilityTarget::Unit(target.entity()))
                {
                    world.write_message(SimulationEvent::CommandRejected { unit, reason });
                }
            }
            Err(Rejection::OutOfRange) => {
                if !world
                    .get::<ControlState>(unit)
                    .is_some_and(ControlState::can_move)
                {
                    movement::clear_route(
                        &mut world.get_mut::<Movement>(unit).expect("unit movement"),
                    );
                    continue;
                }
                let target_position = world
                    .get::<UnitPose>(target.entity())
                    .expect("validated target")
                    .position;
                let needs_path = {
                    let mut movement = world.get_mut::<Movement>(unit).expect("unit movement");
                    movement.repath_remaining = (movement.repath_remaining - TICK_SECONDS).max(0.0);
                    movement.repath_remaining <= 0.0
                        && (movement.route.is_empty()
                            || movement
                                .goal
                                .is_none_or(|old| old.distance(target_position) > 0.6))
                };
                if needs_path {
                    let range = world
                        .resource::<AbilityCatalog>()
                        .get(ability)
                        .expect("validated ability")
                        .range;
                    match plan(world, unit, target_position, range) {
                        Ok(route) => set_route(world, unit, target_position, route),
                        Err(reason) => reject(world, unit, reason),
                    }
                }
            }
            Err(reason) => reject(world, unit, reason),
        }
    }
}
