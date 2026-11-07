//! Candidate generation follows the actual equipped definition, never a slot
//! name or content ID. All candidates still pass the shared combat preflight.
use super::observations::SeenUnit;
use crate::simulation::*;
use bevy::prelude::*;

#[derive(Default)]
pub(super) struct Targets {
    pub allowed: Vec<UnitLifeRef>,
    pub focus: Option<UnitLifeRef>,
}
impl Targets {
    pub(super) fn contains(&self, unit: &SeenUnit) -> bool {
        self.allowed.contains(&unit.life)
    }
}

pub(super) fn choose(
    world: &World,
    hero: &SeenUnit,
    seen: &[SeenUnit],
    targets: &Targets,
    retreat: Option<Vec2>,
) -> Option<UnitCommand> {
    let mut sources = Vec::new();
    if let Some(slots) = world.get::<AbilitySlots>(hero.entity) {
        for slot in 0..4 {
            if let Some(ability) = slots.ability(slot) {
                sources.push((CastSource::AbilitySlot(slot), ability));
            }
        }
    }
    if let Some(items) = world.get::<EquipmentSlots>(hero.entity) {
        for slot in 0..EQUIPMENT_SLOTS {
            if let Some(item) = items.item(slot)
                && let Some(ability) = world
                    .resource::<EquipmentCatalog>()
                    .get(item.equipment())
                    .and_then(|definition| definition.active)
            {
                sources.push((
                    CastSource::EquipmentSlot {
                        slot,
                        instance: item.instance(),
                    },
                    ability,
                ));
            }
        }
    }
    let mut best: Option<(f32, UnitCommand)> = None;
    for (source, id) in sources {
        let Some(definition) = world.resource::<AbilityCatalog>().get(id) else {
            continue;
        };
        let candidates = candidates(world, hero, seen, targets, definition, source, retreat);
        for (target, score) in candidates {
            if score <= 0.0 || preview_cast(world, hero.entity, source, target).is_err() {
                continue;
            }
            let command = match source {
                CastSource::AbilitySlot(slot) => UnitCommand::CastAbility { slot, target },
                CastSource::EquipmentSlot { slot, instance } => UnitCommand::UseEquipment {
                    slot,
                    instance,
                    target,
                },
            };
            if best.as_ref().is_none_or(|(current, _)| score > *current) {
                best = Some((score, command));
            }
        }
    }
    best.map(|(_, command)| command)
}

fn candidates(
    world: &World,
    hero: &SeenUnit,
    seen: &[SeenUnit],
    targets: &Targets,
    definition: &AbilityDefinition,
    source: CastSource,
    retreat: Option<Vec2>,
) -> Vec<(AbilityTarget, f32)> {
    if let Some(candidates) =
        super::compound_spells::candidates(world, hero, seen, targets, definition, source, retreat)
    {
        return candidates;
    }
    if definition.execution.is_dash() {
        if is_dashing(world, hero.entity) {
            return vec![];
        }
        let point = if let Some(home) = retreat {
            if hero.position.distance(home) < 2.0 {
                return vec![];
            }
            home
        } else {
            let Some(target) = seen
                .iter()
                .filter(|unit| {
                    unit.team != hero.team
                        && targets.focus == Some(unit.life)
                        && targets.contains(unit)
                        && unit.role == crate::gameplay::UnitRole::Hero
                        && unit.health_fraction() < 0.3
                        && hero.health_fraction() > 0.6
                        && unit.position.distance(hero.position) <= 12.0
                })
                .min_by(|a, b| {
                    a.position
                        .distance_squared(hero.position)
                        .total_cmp(&b.position.distance_squared(hero.position))
                })
            else {
                return vec![];
            };
            let reach = world
                .get::<Unit>(hero.entity)
                .expect("hero")
                .attributes()
                .attack_range();
            if target.position.distance(hero.position) < reach + 1.0 {
                return vec![];
            }
            target.position
                + (hero.position - target.position).normalize_or(Vec2::X) * (reach * 0.65)
        };
        let target = AbilityTarget::Point(point);
        let offset = (point - hero.position).clamp_length_max(definition.range);
        let end = world.resource::<NavigationMap>().sweep(
            hero.position,
            hero.position + offset,
            hero.radius,
        );
        let end_threat = threat(seen, hero.team, end);
        let current_threat = threat(seen, hero.team, hero.position);
        let unsafe_end = if retreat.is_some() {
            end_threat > current_threat + 0.1
        } else {
            !offensive_landing_allowed(seen, hero, end)
        };
        if end.distance(hero.position) < 1.0 || unsafe_end {
            return vec![];
        }
        return vec![(target, if retreat.is_some() { 100.0 } else { 25.0 })];
    }
    if retreat.is_some() {
        return vec![];
    }
    let (impact, channel) = match &definition.execution {
        AbilityExecution::Impact(impact) => (impact, false),
        AbilityExecution::Channel { pulse, .. } => (pulse, true),
        AbilityExecution::Dash { .. } => unreachable!(),
        AbilityExecution::Line(_)
        | AbilityExecution::Shield(_)
        | AbilityExecution::Standard(_)
        | AbilityExecution::Leap(_)
        | AbilityExecution::Empower(_)
        | AbilityExecution::Volley(_)
        | AbilityExecution::Scout(_)
        | AbilityExecution::Arrow(_) => unreachable!("compound execution dispatched above"),
    };
    let enemies: Vec<_> = seen
        .iter()
        .filter(|unit| {
            unit.team != hero.team
                && targets.contains(unit)
                && unit.role != crate::gameplay::UnitRole::Base
                && unit.position.distance(hero.position) <= definition.range + 3.0
        })
        .collect();
    if channel && (hero.health_fraction() < 0.55 || threat(seen, hero.team, hero.position) > 4.0) {
        return vec![];
    }
    match definition.targeting {
        Targeting::Enemy => enemies
            .into_iter()
            .map(|enemy| {
                (
                    AbilityTarget::Unit(enemy.entity),
                    value(world, hero, enemy, &impact.effects),
                )
            })
            .collect(),
        Targeting::Direction => {
            let Delivery::Projectile { speed, radius } = impact.delivery else {
                return vec![];
            };
            enemies
                .into_iter()
                .filter_map(|enemy| {
                    let horizon = (definition.windup
                        + hero.position.distance(enemy.position) / speed)
                        .min(0.7);
                    let point = enemy.position + (enemy.velocity * horizon).clamp_length_max(3.0);
                    let direction = (point - hero.position).normalize_or_zero();
                    if direction == Vec2::ZERO {
                        return None;
                    }
                    // Extrapolate only current visible motion, then score the
                    // first collision. A screen still prevents a guaranteed
                    // hero hit, while a moving target can be led off its old ray.
                    let hit = seen
                        .iter()
                        .filter(|other| {
                            other.team != hero.team && other.role != crate::gameplay::UnitRole::Base
                        })
                        .filter_map(|other| {
                            let time = (definition.windup
                                + hero.position.distance(other.position) / speed)
                                .min(0.7);
                            let point =
                                other.position + (other.velocity * time).clamp_length_max(3.0);
                            let relative = point - hero.position;
                            let along = relative.dot(direction);
                            let width = (relative - direction * along).length();
                            (along >= 0.0
                                && along <= definition.range
                                && width <= other.radius + radius + 0.25)
                                .then_some((along, other))
                        })
                        .min_by(|a, b| {
                            a.0.total_cmp(&b.0)
                                .then_with(|| a.1.entity.to_bits().cmp(&b.1.entity.to_bits()))
                        });
                    let score = hit
                        .filter(|(_, hit)| targets.contains(hit))
                        .map_or(0.0, |(_, hit)| value(world, hero, hit, &impact.effects));
                    Some((AbilityTarget::Direction(direction), score))
                })
                .collect()
        }
        Targeting::Point => {
            let Delivery::Area { radius, delay } = impact.delivery else {
                return vec![];
            };
            enemies
                .into_iter()
                .map(|enemy| {
                    let horizon = (definition.windup + delay).min(0.7);
                    let point = enemy.position + (enemy.velocity * horizon).clamp_length_max(3.0);
                    let score = seen
                        .iter()
                        .filter(|other| {
                            other.team != hero.team
                                && targets.contains(other)
                                && other.role != crate::gameplay::UnitRole::Base
                                && (other.position
                                    + (other.velocity * horizon).clamp_length_max(3.0))
                                .distance(point)
                                    <= radius + other.radius
                        })
                        .map(|other| value(world, hero, other, &impact.effects))
                        .sum();
                    (AbilityTarget::Point(point), score)
                })
                .collect()
        }
        Targeting::SelfCast => {
            let Delivery::Area { radius, .. } = impact.delivery else {
                return vec![];
            };
            let score = seen
                .iter()
                .filter(|other| {
                    other.team != hero.team
                        && targets.contains(other)
                        && other.role != crate::gameplay::UnitRole::Base
                        && other.position.distance(hero.position) <= radius + other.radius
                })
                .map(|other| value(world, hero, other, &impact.effects))
                .sum();
            vec![(AbilityTarget::SelfCast, score)]
        }
    }
}

pub(super) fn value(world: &World, hero: &SeenUnit, enemy: &SeenUnit, effects: &[Effect]) -> f32 {
    let source = world
        .get::<Unit>(hero.entity)
        .expect("observed hero")
        .attributes();
    let target = world
        .get::<Unit>(enemy.entity)
        .expect("observed enemy")
        .attributes();
    let mut score = 0.0;
    for effect in effects {
        score += match effect {
            Effect::Damage(spec) => {
                spec.estimate_damage(source, target, enemy.health)
                    .unwrap_or(0.0)
                    * 0.12
            }
            Effect::ApplyStatus(status) => {
                let already = world
                    .get::<UnitStatuses>(enemy.entity)
                    .is_some_and(|statuses| {
                        statuses.iter().any(|current| {
                            current.kind() == status.kind() && current.remaining_seconds() > 0.25
                        })
                    });
                if already {
                    1.0
                } else {
                    match status.kind() {
                        StatusKind::Stun | StatusKind::Airborne => 18.0,
                        StatusKind::Root => 12.0,
                        StatusKind::Slow => 6.0,
                    }
                }
            }
            Effect::Displace(_) => 8.0,
            Effect::BonusAttackDamage { damage, ratio } => {
                let base_ad = world
                    .get::<Unit>(hero.entity)
                    .expect("hero")
                    .definition()
                    .base_attributes
                    .attack_damage;
                let mut estimated = *damage;
                estimated.base += (source.attack_damage() - base_ad).max(0.0) * ratio;
                estimated
                    .estimate_damage(source, target, enemy.health)
                    .unwrap_or(0.0)
                    * 0.12
            }
            Effect::ArmorReduction { .. } => {
                if target.armor() > 0.0 {
                    6.0
                } else {
                    0.0
                }
            }
        };
    }
    if enemy.role == crate::gameplay::UnitRole::Hero {
        score += 15.0;
    }
    if enemy.health_fraction() < 0.3 {
        score += 8.0;
    }
    score
}

pub(super) fn threat(seen: &[SeenUnit], team: Team, point: Vec2) -> f32 {
    seen.iter()
        .filter(|unit| unit.team != team && unit.role != crate::gameplay::UnitRole::Base)
        .map(|unit| presence(unit, point))
        .sum()
}

/// One policy covers every offensive relocation. Existing friendly actors may
/// support the landing; the caster and passive bases do not count as support.
/// All inputs belong to the same already visibility-filtered observation.
pub(super) fn offensive_landing_allowed(seen: &[SeenUnit], hero: &SeenUnit, landing: Vec2) -> bool {
    let risk = |point| {
        let support: f32 = seen
            .iter()
            .filter(|unit| {
                unit.team == hero.team
                    && unit.entity != hero.entity
                    && unit.role != crate::gameplay::UnitRole::Base
            })
            .map(|unit| presence(unit, point) * unit.health_fraction().clamp(0.0, 1.0))
            .sum();
        (threat(seen, hero.team, point) - support).max(0.0)
    };
    let landing_risk = risk(landing);
    landing_risk <= 2.3 && landing_risk <= risk(hero.position) + 1.5
}

fn presence(unit: &SeenUnit, point: Vec2) -> f32 {
    let weight = if unit.role == crate::gameplay::UnitRole::Hero {
        2.0
    } else {
        0.7
    };
    weight * (1.0 - unit.position.distance(point) / 8.0).max(0.0)
}
