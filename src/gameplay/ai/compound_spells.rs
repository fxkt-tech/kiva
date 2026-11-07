//! Tactical candidates for compound execution semantics. No champion identity,
//! enemy resource reads or hidden positions participate in these decisions.
use super::{
    observations::SeenUnit,
    spells::{Targets, offensive_landing_allowed, value},
};
use crate::{
    gameplay::{MapDefinition, UnitRole, opponent},
    simulation::*,
};
use bevy::prelude::*;
use std::collections::BTreeSet;

pub(super) fn candidates(
    world: &World,
    hero: &SeenUnit,
    seen: &[SeenUnit],
    targets: &Targets,
    definition: &AbilityDefinition,
    source: CastSource,
    retreat: Option<Vec2>,
) -> Option<Vec<(AbilityTarget, f32)>> {
    let enemies = || {
        seen.iter().filter(|unit| {
            unit.team != hero.team && unit.role != UnitRole::Base && targets.contains(unit)
        })
    };
    let tick = world.resource::<SimulationTick>().0;
    if let CastSource::AbilitySlot(slot) = source
        && let Some(state) = world.get::<UnitMechanics>(hero.entity)
        && state.recast(slot)
    {
        let should_open = retreat.is_some()
            || !enemies().any(|unit| unit.position.distance(hero.position) < definition.range);
        return Some(if state.recast_ready(slot, tick) && should_open {
            vec![(AbilityTarget::SelfCast, 95.0)]
        } else {
            vec![]
        });
    }
    Some(match &definition.execution {
        AbilityExecution::Shield(spec) => {
            let nearby = seen
                .iter()
                .filter(|unit| {
                    unit.team != hero.team
                        && unit.role != UnitRole::Base
                        && unit.position.distance(hero.position) <= spec.radius + unit.radius
                })
                .count();
            let already_shielded = world
                .get::<UnitMechanics>(hero.entity)
                .is_some_and(|state| state.shield() > 0.0);
            if nearby == 0
                || already_shielded
                || (retreat.is_none() && hero.health_fraction() > 0.95)
            {
                vec![]
            } else {
                vec![(
                    AbilityTarget::SelfCast,
                    40.0 + (1.0 - hero.health_fraction()) * 45.0,
                )]
            }
        }
        AbilityExecution::Empower(_) => {
            let mechanics = world.get::<UnitMechanics>(hero.entity)?;
            let reach = world.get::<Unit>(hero.entity)?.attributes().attack_range();
            if retreat.is_some()
                || mechanics.empowered()
                || !enemies().any(|enemy| {
                    enemy.position.distance(hero.position) <= reach + hero.radius + enemy.radius
                })
            {
                vec![]
            } else {
                vec![(AbilityTarget::SelfCast, 75.0)]
            }
        }
        AbilityExecution::Standard(spec) => {
            if retreat.is_some() {
                return Some(vec![]);
            }
            let link = linked_line(world, hero.entity);
            let delay = match spec.impact.delivery {
                Delivery::Area { delay, .. } => delay,
                _ => 0.0,
            };
            enemies()
                .filter(|enemy| enemy.position.distance(hero.position) <= definition.range + 1.0)
                .map(|enemy| {
                    let aim =
                        enemy.position + (enemy.velocity * delay.min(0.7)).clamp_length_max(3.0);
                    let beyond = if link {
                        (aim - hero.position).normalize_or(Vec2::X) * 0.6
                    } else {
                        Vec2::ZERO
                    };
                    let point = hero.position
                        + (aim + beyond - hero.position).clamp_length_max(definition.range);
                    let setup =
                        if link && enemy.role == UnitRole::Hero && hero.health_fraction() > 0.4 {
                            45.0
                        } else {
                            0.0
                        };
                    (
                        AbilityTarget::Point(point),
                        value(world, hero, enemy, &spec.impact.effects) + setup,
                    )
                })
                .collect()
        }
        AbilityExecution::Line(spec) => {
            if retreat.is_some()
                || standard_pending(world, hero.entity)
                || (spec.link.is_some() && is_dashing(world, hero.entity))
            {
                return Some(vec![]);
            }
            let standards = own_standards(world, hero.life);
            let mut directions = enemies()
                .map(|enemy| {
                    let point = enemy.position
                        + (enemy.velocity * definition.windup.min(0.7)).clamp_length_max(3.0);
                    (point - hero.position).normalize_or_zero()
                })
                .collect::<Vec<_>>();
            directions.extend(
                standards
                    .iter()
                    .map(|flag| (flag.position() - hero.position).normalize_or_zero()),
            );
            directions
                .into_iter()
                .filter(|direction| *direction != Vec2::ZERO)
                .filter_map(|direction| {
                    let mut score: f32 = enemies()
                        .filter(|enemy| {
                            let point = enemy.position
                                + (enemy.velocity * definition.windup.min(0.7))
                                    .clamp_length_max(3.0);
                            on_line_at(
                                hero.position,
                                direction,
                                definition.range,
                                spec.width * 0.5,
                                enemy,
                                point,
                            )
                        })
                        .map(|enemy| value(world, hero, enemy, &spec.effects))
                        .sum();
                    if let Some(link) = spec.link
                        && let Some(flag) = standards
                            .iter()
                            .filter(|flag| {
                                let relative = flag.position() - hero.position;
                                let along = relative.dot(direction);
                                along > 0.0
                                    && along <= link.reach
                                    && (relative - direction * along).length() <= link.width * 0.5
                            })
                            .max_by(|a, b| {
                                (a.position() - hero.position)
                                    .dot(direction)
                                    .total_cmp(&(b.position() - hero.position).dot(direction))
                            })
                    {
                        // This cast would actually relocate to the flag even
                        // if its damage alone scored well. Reject the whole
                        // candidate when that committed endpoint is unsafe.
                        if !offensive_landing_allowed(seen, hero, flag.position()) {
                            return None;
                        }
                        let distance = flag.position().distance(hero.position);
                        let caught = enemies()
                            .filter(|enemy| {
                                on_line(hero.position, direction, distance, link.width * 0.5, enemy)
                            })
                            .count();
                        if caught > 0 {
                            score += 55.0 + caught as f32 * 12.0;
                        }
                    }
                    Some((AbilityTarget::Direction(direction), score))
                })
                .collect()
        }
        AbilityExecution::Leap(spec) => {
            // Replacing an unfinished link immediately can discard its useful
            // knockup. Human input may do so; AI completes its current setup.
            if retreat.is_some()
                || standard_pending(world, hero.entity)
                || is_dashing(world, hero.entity)
            {
                return Some(vec![]);
            }
            enemies()
                .filter(|enemy| {
                    (!spec.heroes_only || enemy.role == UnitRole::Hero)
                        && enemy.position.distance(hero.position) <= definition.range + enemy.radius
                        && offensive_landing_allowed(seen, hero, enemy.position)
                })
                .map(|enemy| {
                    (
                        AbilityTarget::Unit(enemy.entity),
                        value(world, hero, enemy, &spec.effects) + 10.0,
                    )
                })
                .collect()
        }
        AbilityExecution::Volley(spec) => {
            if retreat.is_some() {
                return Some(vec![]);
            }
            enemies()
                .filter(|enemy| enemy.position.distance(hero.position) <= definition.range)
                .map(|enemy| {
                    let aim = predict(hero, enemy, definition.windup, spec.speed);
                    let direction = (aim - hero.position).normalize_or(Vec2::X);
                    let mut hit = BTreeSet::new();
                    let mut score = 0.0;
                    for arrow in 0..spec.count {
                        let angle = if spec.count == 1 {
                            0.0
                        } else {
                            spec.angle * (f32::from(arrow) / f32::from(spec.count - 1) - 0.5)
                        };
                        let ray = Vec2::from_angle(angle).rotate(direction);
                        if let Some(target) =
                            first_hit(seen, hero, ray, definition, spec.speed, spec.radius, false)
                            && targets.contains(target)
                            && hit.insert(target.entity)
                        {
                            score += value(world, hero, target, &spec.effects);
                        }
                    }
                    (AbilityTarget::Direction(direction), score)
                })
                .collect()
        }
        AbilityExecution::Scout(_) => {
            if retreat.is_some()
                || enemies().any(|enemy| enemy.position.distance(hero.position) < 8.0)
            {
                return Some(vec![]);
            }
            let map = world.resource::<MapDefinition>();
            // Static terrain and base positions are known. Hidden enemy
            // coordinates are never read to choose a scouting destination.
            let direction = (map.base_position(opponent(hero.team)) - hero.position)
                .normalize_or(map.lane_direction(hero.team));
            let sight = world.get::<Unit>(hero.entity)?.definition().vision_radius;
            let point = (hero.position + direction * (sight + 6.0).min(definition.range)).clamp(
                Vec2::splat(-map.half_size() + 1.0),
                Vec2::splat(map.half_size() - 1.0),
            );
            if point_visible(world, hero.team, point) {
                vec![]
            } else {
                vec![(AbilityTarget::Point(point), 5.0)]
            }
        }
        AbilityExecution::Arrow(spec) => {
            if retreat.is_some() {
                return Some(vec![]);
            }
            enemies()
                .filter(|enemy| enemy.role == UnitRole::Hero)
                .map(|enemy| {
                    let direction = (predict(hero, enemy, definition.windup, spec.speed)
                        - hero.position)
                        .normalize_or(Vec2::X);
                    let hit = first_hit(
                        seen,
                        hero,
                        direction,
                        definition,
                        spec.speed,
                        spec.radius,
                        true,
                    );
                    let score = hit
                        .filter(|unit| targets.contains(unit))
                        .map_or(0.0, |hit| {
                            25.0 + seen
                                .iter()
                                .filter(|other| {
                                    other.team != hero.team
                                        && targets.contains(other)
                                        && other.role != UnitRole::Base
                                        && other.position.distance(hit.position)
                                            <= spec.impact_radius + other.radius
                                })
                                .map(|other| value(world, hero, other, &spec.effects))
                                .sum::<f32>()
                        });
                    (AbilityTarget::Direction(direction), score)
                })
                .collect()
        }
        AbilityExecution::Impact(_)
        | AbilityExecution::Dash { .. }
        | AbilityExecution::Channel { .. } => return None,
    })
}

fn predict(hero: &SeenUnit, enemy: &SeenUnit, windup: f32, speed: f32) -> Vec2 {
    let horizon = (windup + hero.position.distance(enemy.position) / speed).min(0.7);
    enemy.position + (enemy.velocity * horizon).clamp_length_max(3.0)
}

fn on_line(origin: Vec2, direction: Vec2, range: f32, radius: f32, enemy: &SeenUnit) -> bool {
    on_line_at(origin, direction, range, radius, enemy, enemy.position)
}

fn on_line_at(
    origin: Vec2,
    direction: Vec2,
    range: f32,
    radius: f32,
    enemy: &SeenUnit,
    position: Vec2,
) -> bool {
    let relative = position - origin;
    let along = relative.dot(direction);
    along >= 0.0
        && along <= range + enemy.radius
        && (relative - direction * along).length() <= radius + enemy.radius
}

fn first_hit<'a>(
    seen: &'a [SeenUnit],
    hero: &SeenUnit,
    direction: Vec2,
    definition: &AbilityDefinition,
    speed: f32,
    radius: f32,
    heroes_only: bool,
) -> Option<&'a SeenUnit> {
    seen.iter()
        .filter(|other| {
            other.team != hero.team
                && other.role != UnitRole::Base
                && (!heroes_only || other.role == UnitRole::Hero)
                && on_line_at(
                    hero.position,
                    direction,
                    definition.range,
                    radius,
                    other,
                    predict(hero, other, definition.windup, speed),
                )
        })
        .min_by(|a, b| {
            (predict(hero, a, definition.windup, speed) - hero.position)
                .dot(direction)
                .total_cmp(
                    &(predict(hero, b, definition.windup, speed) - hero.position).dot(direction),
                )
                .then_with(|| a.entity.to_bits().cmp(&b.entity.to_bits()))
        })
}

fn own_standards(world: &World, owner: UnitLifeRef) -> Vec<&BattlefieldObject> {
    let tick = world.resource::<SimulationTick>().0;
    own_standard_objects(world, owner)
        .into_iter()
        .filter(|object| object.active(tick))
        .collect()
}

fn own_standard_objects(world: &World, owner: UnitLifeRef) -> Vec<&BattlefieldObject> {
    let Some(mut objects) = world.try_query::<&BattlefieldObject>() else {
        return vec![];
    };
    objects
        .iter(world)
        .filter(|object| object.kind() == ObjectKind::Standard && object.owner() == owner)
        .collect()
}

fn linked_line(world: &World, owner: Entity) -> bool {
    world.get::<AbilitySlots>(owner).is_some_and(|slots| {
        (0..4)
            .filter_map(|slot| slots.ability(slot))
            .filter_map(|id| world.resource::<AbilityCatalog>().get(id))
            .any(|definition| {
                matches!(
                    definition.execution,
                    AbilityExecution::Line(LineSpec { link: Some(_), .. })
                )
            })
    })
}

fn standard_pending(world: &World, owner: Entity) -> bool {
    let Some(state) = world.get::<UnitState>(owner) else {
        return false;
    };
    let tick = world.resource::<SimulationTick>().0;
    own_standard_objects(world, state.life_ref(owner))
        .iter()
        .any(|object| object.remaining_seconds(tick) > 0.0 && !object.active(tick))
}
