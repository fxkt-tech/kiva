//! Sole position owner: normal routes, autonomous dashes and external motion.
use super::{
    model::{ForcedMotion, Movement},
    *,
};
use bevy::prelude::*;

/// Opt-in separation for ordinary walking. It is a visual crowd aid, not a
/// hard collider: it cannot block heroes or alter committed motion endpoints.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SoftAvoidance;

/// Observes committed autonomous movement without exposing its writable state.
pub fn is_dashing(world: &World, unit: Entity) -> bool {
    world
        .get::<Movement>(unit)
        .is_some_and(|movement| movement.dash.is_some())
        || world.get::<super::compound::CompoundMotion>(unit).is_some()
}

/// Ordinary movement and most committed motions forbid new casts. Link motion
/// may explicitly allow casts, enabling compound combinations without content IDs.
pub fn motion_allows_cast(world: &World, unit: Entity) -> bool {
    if world
        .get::<Movement>(unit)
        .is_some_and(|movement| movement.dash.is_some())
    {
        return false;
    }
    world
        .get::<super::compound::CompoundMotion>(unit)
        .is_none_or(|motion| {
            matches!(
                motion.kind,
                super::compound::MotionKind::Link {
                    can_cast_during_motion: true,
                    ..
                }
            )
        })
}

pub(super) fn reset_pose(pose: &mut UnitPose, home: Vec2) {
    pose.position = home;
    pose.previous = home;
}
pub(super) fn clear_route(movement: &mut Movement) {
    movement.route.clear();
    movement.goal = None;
    movement.repath_remaining = 0.0;
}
pub(super) fn interrupt_autonomous(world: &mut World, unit: Entity) {
    if let Some(mut movement) = world.get_mut::<Movement>(unit) {
        clear_route(&mut movement);
        movement.dash = None;
    }
    if world
        .get::<super::compound::CompoundMotion>(unit)
        .is_some_and(|motion| matches!(motion.kind, super::compound::MotionKind::Link { .. }))
        && world
            .get::<ControlState>(unit)
            .is_some_and(|control| !control.can_cast())
    {
        world
            .entity_mut(unit)
            .remove::<super::compound::CompoundMotion>();
        let pose = world.get::<UnitPose>(unit).expect("pose");
        let radius = world.get::<Unit>(unit).expect("unit").stats.radius;
        let map = world.resource::<NavigationMap>();
        if !map.walkable(pose.position, radius)
            && let Some(free) = nearest_landing(map, pose.position, radius)
        {
            let mut pose = world.get_mut::<UnitPose>(unit).expect("pose");
            pose.position = free;
        }
    }
}
pub(super) fn clear_all(world: &mut World, unit: Entity) {
    world
        .entity_mut(unit)
        .remove::<super::compound::CompoundMotion>();
    if let Some(mut movement) = world.get_mut::<Movement>(unit) {
        *movement = Movement::default();
    }
}
pub(super) fn start_compound(
    world: &mut World,
    unit: Entity,
    motion: super::compound::CompoundMotion,
) {
    clear_route(&mut world.get_mut::<Movement>(unit).expect("movement"));
    world.entity_mut(unit).insert(motion);
}
/// Preflight every displacement against all proposed terrain before moving
/// anyone. Existing walls remain solid during displacement; a new ring must
/// never push an actor through an older wall or out of an enclosed pocket.
pub(super) fn plan_ring_overlap(
    world: &mut World,
    proposed: &NavigationMap,
    center: Vec2,
    ring: f32,
    thickness: f32,
) -> Option<Vec<(Entity, Vec2)>> {
    let mut units: Vec<_> = world
        .query::<(Entity, &Unit, &UnitState, &UnitPose)>()
        .iter(world)
        .filter(|(_, unit, state, pose)| {
            state.alive()
                && unit.class() == UnitClass::Actor
                && (pose.position.distance(center) - ring).abs() <= thickness + unit.stats.radius
        })
        .map(|(id, unit, _, pose)| (id, unit.stats.radius, pose.position))
        .collect();
    units.sort_by_key(|(id, _, _)| id.to_bits());
    let original = world.resource::<NavigationMap>();
    let mut plan = Vec::with_capacity(units.len());
    for (id, radius, position) in units {
        let offset = position - center;
        let direction = super::effects::direction(offset, Vec2::X);
        let distance = if offset.length() < ring {
            (ring - thickness - radius - 0.01).max(0.0)
        } else {
            ring + thickness + radius + 0.01
        };
        let clear = |point: Vec2| {
            proposed.walkable(point, radius) && original.line_clear(position, point, radius)
        };
        let preferred = center + direction * distance;
        let next = if clear(preferred) {
            preferred
        } else {
            // Search only the immediate ring neighborhood. If old terrain
            // encloses the actor, reject this ring rather than teleporting it
            // across the map or leaving it inside collision geometry.
            let limit = ring + thickness + radius;
            (1..=(limit / 0.05).ceil() as usize).find_map(|step| {
                let distance = (step as f32 * 0.05).min(limit);
                (0..64).find_map(|spoke| {
                    let offset = Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / 64.0)
                        .rotate(direction);
                    let point = position + offset * distance;
                    clear(point).then_some(point)
                })
            })?
        };
        plan.push((id, next));
    }
    Some(plan)
}

pub(super) fn apply_terrain_relocations(world: &mut World, plan: Vec<(Entity, Vec2)>) {
    for (id, next) in plan {
        if let Some(mut pose) = world.get_mut::<UnitPose>(id) {
            pose.position = next;
            pose.previous = next;
        }
        clear_route(&mut world.get_mut::<Movement>(id).expect("movement"));
    }
}
pub(super) fn start_dash(world: &mut World, unit: Entity, destination: Vec2, speed: f32) {
    let mut movement = world.get_mut::<Movement>(unit).expect("unit movement");
    clear_route(&mut movement);
    movement.dash = Some((destination, speed));
}
pub(super) fn start_forced(
    world: &mut World,
    target: UnitLifeRef,
    spec: DisplacementSpec,
    center: Vec2,
    impact_direction: Vec2,
    fallback: Vec2,
) {
    if world
        .get::<super::compound::CompoundMotion>(target.entity())
        .is_some_and(|motion| matches!(motion.kind, super::compound::MotionKind::Leap { .. }))
    {
        return;
    }
    if !target.is_current(world)
        || world
            .get::<Unit>(target.entity())
            .is_none_or(|unit| unit.class() == UnitClass::Structure)
    {
        return;
    }
    let position = world
        .get::<UnitPose>(target.entity())
        .expect("unit pose")
        .position;
    let toward = center - position;
    let direction = match spec.direction {
        DisplacementDirection::AlongImpact => super::effects::direction(impact_direction, fallback),
        DisplacementDirection::AwayFromCenter => super::effects::direction(-toward, fallback),
        DisplacementDirection::TowardCenter => super::effects::direction(toward, fallback),
    };
    let distance = if spec.direction == DisplacementDirection::TowardCenter {
        spec.distance.min(toward.length())
    } else {
        spec.distance
    };
    let destination = position + direction * distance;
    interrupt_autonomous(world, target.entity());
    {
        let mut movement = world
            .get_mut::<Movement>(target.entity())
            .expect("unit movement");
        movement.motion_sequence = movement
            .motion_sequence
            .checked_add(1)
            .expect("motion instance space exhausted");
        movement.forced = Some(ForcedMotion {
            id: movement.motion_sequence,
            target,
            destination,
            speed: spec.speed,
        });
    }
    super::actions::interrupt_control(world, target.entity(), StatusKind::Stun);
    super::statuses::synchronize(world, target.entity());
}

/// Detect truncated travel proportionally, including sub-millimeter motion.
/// Exact requested results stay unobstructed even at very small speeds.
fn sweep_blocked(start: Vec2, requested: Vec2, next: Vec2) -> bool {
    if next == requested {
        return false;
    }
    let delta = requested - start;
    let achieved = next - start;
    let requested_distance = f64::from(delta.x).hypot(f64::from(delta.y));
    let achieved_distance = f64::from(achieved.x).hypot(f64::from(achieved.y));
    achieved_distance < requested_distance * (1.0 - 1e-5)
}

pub(super) fn move_units(world: &mut World) {
    let mut finished = Vec::new();
    // Every participant observes the same pre-movement positions. Reading
    // already moved neighbors would make steering depend on ECS iteration.
    let mut neighbors: Vec<_> = world
        .query_filtered::<(Entity, &Unit, &UnitState, &UnitPose), With<SoftAvoidance>>()
        .iter(world)
        .filter(|(_, unit, state, _)| state.alive() && unit.class() == UnitClass::Actor)
        .map(|(entity, unit, _, pose)| (entity, pose.position(), unit.definition().radius))
        .collect();
    neighbors.sort_by_key(|(entity, _, _)| entity.to_bits());
    world.resource_scope(|world, map: Mut<NavigationMap>| {
        let mut actors = world.query::<(
            Entity,
            &Unit,
            &UnitState,
            &ControlState,
            &UnitAction,
            &mut UnitPose,
            &mut Movement,
            Has<SoftAvoidance>,
            Option<&mut super::compound::CompoundMotion>,
        )>();
        for (entity, unit, state, control, action, mut pose, mut movement, avoids, compound) in
            actors.iter_mut(world)
        {
            pose.previous = pose.position;
            if !state.alive() {
                continue;
            }
            if let Some(mut motion) = compound {
                let delta = motion.destination - pose.position;
                let requested = pose.position + delta.clamp_length_max(motion.speed * TICK_SECONDS);
                let traverses = matches!(
                    motion.kind,
                    super::compound::MotionKind::Leap { .. }
                        | super::compound::MotionKind::Link {
                            traverse_terrain: true,
                            ..
                        }
                );
                let mut next = if traverses {
                    requested.clamp(
                        Vec2::splat(-map.half_size() + unit.stats.radius + 0.001),
                        Vec2::splat(map.half_size() - unit.stats.radius - 0.001),
                    )
                } else {
                    map.sweep(pose.position, requested, unit.stats.radius)
                };
                let mut landed = next.distance(motion.destination) < 0.01
                    || requested == pose.position
                    || sweep_blocked(pose.position, requested, next);
                if let Some((remaining, total)) = motion.lifetime {
                    let remaining = remaining.saturating_sub(1);
                    motion.lifetime = Some((remaining, total));
                    landed = remaining == 0;
                    if landed {
                        next = motion.destination.clamp(
                            Vec2::splat(-map.half_size() + unit.stats.radius + 0.001),
                            Vec2::splat(map.half_size() - unit.stats.radius - 0.001),
                        );
                    }
                }
                if traverses && landed && !map.walkable(next, unit.stats.radius) {
                    if let Some(free) = nearest_landing(&map, next, unit.stats.radius) {
                        next = free;
                    } else {
                        next = motion.origin;
                        landed = true;
                    }
                }
                if delta.length_squared() > 1e-6 {
                    pose.facing = super::effects::direction(delta, pose.facing);
                }
                motion.done = landed;
                pose.position = next;
                continue;
            }
            if let Some(motion) = &movement.forced {
                let id = motion.id;
                let destination = motion.destination;
                let delta = destination - pose.position;
                let facing = super::effects::direction(delta, pose.facing);
                let distance = f64::from(delta.x).hypot(f64::from(delta.y));
                let step = distance.min(f64::from(motion.speed) * f64::from(TICK_SECONDS)) as f32;
                let requested = pose.position + facing * step;
                let next = map.sweep(pose.position, requested, unit.stats.radius);
                let ended = motion.target != state.life_ref(entity)
                    || sweep_blocked(pose.position, requested, next)
                    || requested == pose.position
                    || next.distance(destination) < 0.01;
                if motion.target == state.life_ref(entity) {
                    pose.position = next;
                    pose.facing = facing;
                }
                if ended && movement.forced.as_ref().is_some_and(|m| m.id == id) {
                    movement.forced = None;
                    clear_route(&mut movement);
                    finished.push(entity);
                }
                continue;
            }
            if let Some((destination, speed)) = movement.dash {
                let delta = destination - pose.position;
                let requested = pose.position + delta.clamp_length_max(speed * TICK_SECONDS);
                let next = map.sweep(pose.position, requested, unit.stats.radius);
                let blocked = sweep_blocked(pose.position, requested, next)
                    || (speed > 0.0 && requested == pose.position);
                if delta.length_squared() > 1e-6 {
                    pose.facing = delta.normalize();
                }
                pose.position = next;
                if blocked || next.distance(destination) < 0.01 {
                    movement.dash = None;
                    clear_route(&mut movement);
                }
                continue;
            }
            if !control.can_move() || !matches!(action, UnitAction::Ready) {
                continue;
            }
            let mut budget = unit.attributes().move_speed() * TICK_SECONDS;
            while let Some(&destination) = movement.route.front() {
                let delta = destination - pose.position;
                let distance = delta.length();
                if distance < 0.001 {
                    movement.route.pop_front();
                    continue;
                }
                let forward = delta / distance;
                pose.facing = forward;
                if avoids {
                    let lateral = separation(
                        entity,
                        pose.position,
                        unit.stats.radius,
                        pose.facing,
                        &neighbors,
                    );
                    pose.facing = (pose.facing + lateral).normalize_or(pose.facing);
                }
                let mut requested = pose.position + pose.facing * distance.min(budget);
                let mut next = map.sweep(pose.position, requested, unit.stats.radius);
                if avoids && next.distance_squared(requested) > 1e-6 {
                    // Separation is optional. If its side-step meets a wall,
                    // retain the valid route's forward motion rather than
                    // repeatedly clearing/replanning into that same wall.
                    let straight = pose.position + forward * distance.min(budget);
                    let straight_end = map.sweep(pose.position, straight, unit.stats.radius);
                    if straight_end.distance_squared(straight) <= 1e-6
                        || (straight_end - pose.position).dot(forward)
                            > (next - pose.position).dot(forward)
                    {
                        requested = straight;
                        next = straight_end;
                        pose.facing = forward;
                    }
                }
                pose.position = next;
                if next.distance_squared(requested) > 1e-6 {
                    clear_route(&mut movement);
                    break;
                }
                if distance > budget || (avoids && next.distance(destination) > 0.001) {
                    break;
                }
                budget -= distance;
                movement.route.pop_front();
                if budget <= 0.001 {
                    break;
                }
            }
        }
    });
    for unit in finished {
        super::statuses::synchronize(world, unit);
    }
}

fn separation(
    entity: Entity,
    position: Vec2,
    radius: f32,
    forward: Vec2,
    neighbors: &[(Entity, Vec2, f32)],
) -> Vec2 {
    let mut lateral = Vec2::ZERO;
    for &(other, point, other_radius) in neighbors {
        if other == entity {
            continue;
        }
        let delta = position - point;
        let distance = delta.length();
        let spacing = (radius + other_radius) * 1.2;
        if distance >= spacing {
            continue;
        }
        let away = if distance > 0.0001 {
            delta / distance
        } else if entity.to_bits() < other.to_bits() {
            Vec2::NEG_X
        } else {
            Vec2::X
        };
        let mut side = away - forward * away.dot(forward);
        if side.length_squared() < 0.01 {
            // Head-on pairs turn in opposite world directions, so they can
            // pass rather than stop or push one another backward.
            side = Vec2::new(-away.y, away.x);
        }
        lateral += side.normalize_or_zero() * (1.0 - distance / spacing);
    }
    lateral.clamp_length_max(0.7)
}

fn nearest_landing(map: &NavigationMap, point: Vec2, radius: f32) -> Option<Vec2> {
    for step in 1..=((map.half_size() * 3.0) / 0.1).ceil() as usize {
        let distance = step as f32 * 0.1;
        for spoke in 0..32 {
            let candidate =
                point + Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / 32.0) * distance;
            if map.walkable(candidate, radius) {
                return Some(candidate);
            }
        }
    }
    None
}

/// Normalized fixed-tick leap phase; presentation chooses its own arc height.
pub fn leap_progress(world: &World, unit: Entity, interpolation: f32) -> Option<f32> {
    let motion = world.get::<super::compound::CompoundMotion>(unit)?;
    let (remaining, total) = motion.lifetime?;
    Some(
        (1.0 - (remaining as f32 + (1.0 - interpolation.clamp(0.0, 1.0))) / total as f32)
            .clamp(0.0, 1.0),
    )
}
