//! Sole position owner: normal routes, autonomous dashes and external motion.
use super::{
    model::{ForcedMotion, Movement},
    *,
};
use bevy::prelude::*;

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
}
pub(super) fn clear_all(world: &mut World, unit: Entity) {
    if let Some(mut movement) = world.get_mut::<Movement>(unit) {
        *movement = Movement::default();
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
    if !target.is_current(world) {
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
    world.resource_scope(|world, map: Mut<NavigationMap>| {
        let mut actors = world.query::<(
            Entity,
            &Unit,
            &UnitState,
            &ControlState,
            &UnitAction,
            &mut UnitPose,
            &mut Movement,
        )>();
        for (entity, unit, state, control, action, mut pose, mut movement) in actors.iter_mut(world)
        {
            pose.previous = pose.position;
            if !state.alive() {
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
                pose.facing = delta / distance;
                let requested = pose.position + pose.facing * distance.min(budget);
                let next = map.sweep(pose.position, requested, unit.stats.radius);
                pose.position = next;
                if next.distance_squared(requested) > 1e-6 {
                    clear_route(&mut movement);
                    break;
                }
                if distance > budget {
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
