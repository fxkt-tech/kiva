//! Sole owner of runtime position changes, including swept dashes and respawns.
use super::{model::Movement, *};
use bevy::prelude::*;

pub(super) fn reset_pose(pose: &mut UnitPose, home: Vec2) {
    pose.position = home;
    pose.previous = home;
}

pub(super) fn move_units(
    map: Res<NavigationMap>,
    mut actors: Query<(&Unit, &UnitState, &UnitAction, &mut UnitPose, &mut Movement)>,
) {
    for (unit, state, action, mut pose, mut movement) in &mut actors {
        pose.previous = pose.position;
        if !state.alive() {
            continue;
        }
        if let Some((destination, speed)) = movement.dash {
            let delta = destination - pose.position;
            let requested = pose.position + delta.clamp_length_max(speed * TICK_SECONDS);
            let next = map.sweep(pose.position, requested, unit.stats.radius);
            let blocked = next.distance_squared(requested) > 1e-6;
            if delta.length_squared() > 1e-6 {
                pose.facing = delta.normalize();
            }
            pose.position = next;
            if blocked || next.distance(destination) < 0.01 {
                movement.dash = None;
            }
            continue;
        }
        if !matches!(action, UnitAction::Ready) {
            continue;
        }
        let mut budget = unit.stats.move_speed * TICK_SECONDS;
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
                movement.route.clear();
                movement.goal = None;
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
}
