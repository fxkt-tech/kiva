use super::{
    DECISION_TICKS,
    observations::{self, SeenUnit},
    pursuit::Pursuit,
    spells,
};
use crate::{
    gameplay::{Controller, MapDefinition, MatchState, UnitRole, opponent},
    simulation::*,
};
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tactic {
    Push,
    Engage,
    Retreat,
    Recover,
    Defend,
}
#[derive(Component)]
struct HeroBrain {
    life: UnitLifeRef,
    next_tick: u64,
    tactic: Tactic,
    target: Option<UnitLifeRef>,
    target_until: u64,
    pursuit: Pursuit,
}
impl HeroBrain {
    fn new(life: UnitLifeRef, tick: u64) -> Self {
        Self {
            life,
            next_tick: tick + DECISION_TICKS,
            tactic: Tactic::Push,
            target: None,
            target_until: 0,
            pursuit: Pursuit::default(),
        }
    }
}

pub(super) fn decide(world: &mut World) {
    if !world.resource::<MatchState>().can_advance() || world.resource::<SimulationControl>().paused
    {
        return;
    }
    let tick = world.resource::<SimulationTick>().0;
    let seen = observations::observe(world);
    let map = world.resource::<MapDefinition>().clone();
    let controlled: Vec<_> = world
        .query::<(Entity, &Controller, &UnitState)>()
        .iter(world)
        .filter(|(_, controller, _)| **controller == Controller::HeroAi)
        .map(|(entity, _, state)| (entity, state.alive(), state.life_ref(entity)))
        .collect();
    for (entity, alive, life) in controlled {
        if !alive {
            world.entity_mut(entity).remove::<HeroBrain>();
            continue;
        }
        if world
            .get::<HeroBrain>(entity)
            .is_none_or(|brain| brain.life != life)
        {
            world.entity_mut(entity).insert(HeroBrain::new(life, tick));
            continue;
        }
        if world.get::<HeroBrain>(entity).expect("brain").next_tick > tick {
            continue;
        }
        world.get_mut::<HeroBrain>(entity).expect("brain").next_tick = tick + DECISION_TICKS;
        let Some(hero) = seen.iter().find(|unit| unit.entity == entity) else {
            continue;
        };
        let home = map.base_position(hero.team) + map.lane_direction(hero.team) * 0.8;
        let previous = world.get::<HeroBrain>(entity).expect("brain").tactic;
        let friendly_cover: f32 = seen
            .iter()
            .filter(|unit| {
                unit.team == hero.team
                    && unit.entity != entity
                    && unit.position.distance(hero.position) < 8.0
                    && unit.role != UnitRole::Base
            })
            .map(|unit| {
                if unit.role == UnitRole::Hero {
                    1.5
                } else {
                    0.5
                }
            })
            .sum();
        let outnumbered = hero.health_fraction() < 0.6
            && spells::threat(&seen, hero.team, hero.position) > friendly_cover + 2.5;
        let withdrawing = hero.health_fraction() < 0.25
            || outnumbered
            || (matches!(previous, Tactic::Retreat | Tactic::Recover)
                && hero.health_fraction() < 0.75);
        if is_dashing(world, entity) {
            continue;
        }
        let action = world.get::<UnitAction>(entity).expect("hero action");
        let committed = matches!(
            action,
            UnitAction::Casting { .. }
                | UnitAction::Channeling { .. }
                | UnitAction::AttackWindup { .. }
        );
        if committed && !withdrawing {
            continue;
        }
        if withdrawing {
            let recovering = hero.position.distance(home) < 0.6;
            let mut brain = world.get_mut::<HeroBrain>(entity).expect("brain");
            brain.tactic = if recovering {
                Tactic::Recover
            } else {
                Tactic::Retreat
            };
            brain.target = None;
            brain.pursuit.clear();
            if !world
                .get::<ControlState>(entity)
                .expect("control")
                .can_move()
            {
                continue;
            }
            if !recovering {
                if let Some(command) =
                    spells::choose(world, hero, &seen, &spells::Targets::default(), Some(home))
                {
                    observations::submit(world, entity, command);
                } else {
                    observations::submit(world, entity, UnitCommand::MoveTo(home));
                }
            } else {
                observations::submit(world, entity, UnitCommand::Stop);
            }
            continue;
        }
        let defended = seen.iter().any(|unit| {
            unit.team != hero.team
                && unit.role != UnitRole::Base
                && unit.position.distance(map.base_position(hero.team)) < 6.5
        });
        let reach = world
            .get::<Unit>(entity)
            .expect("hero")
            .attributes()
            .attack_range()
            + hero.radius;
        world
            .get_mut::<HeroBrain>(entity)
            .expect("brain")
            .pursuit
            .refresh(hero, &seen, &map, tick, reach);
        let brain = world.get::<HeroBrain>(entity).expect("brain");
        let target = select_target(brain, hero, &seen, &map, defended, tick);
        let spell_targets = spells::Targets {
            allowed: seen
                .iter()
                .filter(|unit| brain.pursuit.allows(hero, unit, &map, tick))
                .map(|unit| unit.life)
                .collect(),
            focus: target.map(|target| target.life),
        };
        {
            let mut brain = world.get_mut::<HeroBrain>(entity).expect("brain");
            if brain.target != target.map(|target| target.life) {
                brain.target_until = tick + 30;
            }
            brain.target = target.map(|target| target.life);
            brain.pursuit.select(hero, target, tick);
            brain.tactic = if defended {
                Tactic::Defend
            } else if target.is_some() {
                Tactic::Engage
            } else {
                Tactic::Push
            };
        }
        if let Some(command) = dodge(world, hero, &seen) {
            observations::submit(world, entity, command);
            continue;
        }
        if let Some(command) = spells::choose(world, hero, &seen, &spell_targets, None) {
            observations::submit(world, entity, command);
            continue;
        }
        if let Some(target) = target {
            let unit = world.get::<Unit>(entity).expect("hero");
            let preferred = unit.attributes().attack_range() * 0.75;
            let cooling = world
                .get::<UnitState>(entity)
                .expect("state")
                .attack_cooldown()
                > 0.05;
            if cooling
                && target.position.distance(hero.position) < preferred * 0.8
                && world
                    .get::<ControlState>(entity)
                    .expect("control")
                    .can_move()
            {
                let point = target.position
                    + (hero.position - target.position).normalize_or(map.lane_lateral(hero.team))
                        * preferred;
                if safe_walk(world, hero, point, &seen) {
                    observations::submit(world, entity, UnitCommand::MoveTo(point));
                    continue;
                }
            }
            if world
                .get::<ControlState>(entity)
                .expect("control")
                .can_attack()
            {
                observations::submit(world, entity, UnitCommand::AttackTarget(target.entity));
            }
            continue;
        }
        advance_lane(world, hero, &seen, &map);
    }
}

fn advance_lane(world: &mut World, hero: &SeenUnit, seen: &[SeenUnit], map: &MapDefinition) {
    let enemy_base = seen
        .iter()
        .find(|unit| unit.team == opponent(hero.team) && unit.role == UnitRole::Base);
    if let Some(base) = enemy_base {
        let unit = world.get::<Unit>(hero.entity).expect("hero");
        if hero.position.distance(base.position)
            < unit.attributes().attack_range() + hero.radius + base.radius + 0.5
            && world
                .get::<ControlState>(hero.entity)
                .expect("control")
                .can_attack()
        {
            observations::submit(world, hero.entity, UnitCommand::AttackTarget(base.entity));
            return;
        }
    }
    if !world
        .get::<ControlState>(hero.entity)
        .expect("control")
        .can_move()
    {
        return;
    }
    let axis = map.lane_direction(hero.team);
    let progress = (hero.position - map.base_position(hero.team)).dot(axis);
    let front = seen
        .iter()
        .filter(|unit| unit.team == hero.team && unit.is_minion())
        .map(|unit| (unit.position - map.base_position(hero.team)).dot(axis))
        .max_by(f32::total_cmp);
    let goal = front
        .map_or((progress + 3.0).min(map.lane_length() * 0.45), |front| {
            (front - 2.4).max(2.0)
        })
        .clamp(1.0, map.lane_length() - 3.0);
    let point = map.base_position(hero.team) + axis * goal + map.lane_lateral(hero.team) * 1.0;
    if hero.position.distance_squared(point) > 0.25 {
        observations::submit(world, hero.entity, UnitCommand::MoveTo(point));
    } else {
        observations::submit(world, hero.entity, UnitCommand::Stop);
    }
}

fn select_target<'a>(
    brain: &HeroBrain,
    hero: &SeenUnit,
    seen: &'a [SeenUnit],
    map: &MapDefinition,
    defended: bool,
    tick: u64,
) -> Option<&'a SeenUnit> {
    let eligible = |unit: &&SeenUnit| brain.pursuit.allows(hero, unit, map, tick);
    let current = brain
        .target
        .and_then(|life| observations::current_target(seen, life))
        .filter(eligible);
    let best = seen.iter().filter(eligible).max_by(|a, b| {
        target_value(hero, a, map, defended)
            .total_cmp(&target_value(hero, b, map, defended))
            .then_with(|| b.entity.to_bits().cmp(&a.entity.to_bits()))
    });
    match (current, best) {
        (Some(current), Some(best))
            if brain.target_until > tick
                || target_value(hero, best, map, defended)
                    < target_value(hero, current, map, defended) + 12.0 =>
        {
            Some(current)
        }
        (_, best) => best,
    }
}

fn target_value(hero: &SeenUnit, target: &SeenUnit, map: &MapDefinition, defend: bool) -> f32 {
    let role = if target.role == UnitRole::Hero {
        25.0
    } else {
        12.0
    };
    role + (1.0 - target.health_fraction()) * 12.0 - hero.position.distance(target.position)
        + if defend {
            (8.0 - target.position.distance(map.base_position(hero.team))).max(0.0) * 5.0
        } else {
            0.0
        }
}

fn safe_walk(world: &World, hero: &SeenUnit, point: Vec2, seen: &[SeenUnit]) -> bool {
    let map = world.resource::<NavigationMap>();
    map.walkable(point, hero.radius)
        && map.path(hero.position, point, hero.radius).is_ok()
        && spells::threat(seen, hero.team, point)
            <= spells::threat(seen, hero.team, hero.position) + 0.3
}

fn dodge(world: &mut World, hero: &SeenUnit, seen: &[SeenUnit]) -> Option<UnitCommand> {
    if !world.get::<ControlState>(hero.entity)?.can_move() {
        return None;
    }
    let threats: Vec<_> = world
        .query::<&Projectile>()
        .iter(world)
        .filter(|projectile| projectile.team() != hero.team)
        .map(|projectile| {
            (
                projectile.position(),
                (projectile.position() - projectile.interpolated(0.0)) / TICK_SECONDS,
                projectile.radius(),
            )
        })
        .collect();
    for (position, velocity, radius) in threats {
        if velocity.length_squared() < 0.001 || position.distance(hero.position) > 6.0 {
            continue;
        }
        let time =
            ((hero.position - position).dot(velocity) / velocity.length_squared()).clamp(0.0, 0.35);
        if (position + velocity * time).distance(hero.position) > hero.radius + radius + 0.15 {
            continue;
        }
        let side = Vec2::new(-velocity.y, velocity.x).normalize_or(Vec2::X) * 1.6;
        let choices = [hero.position + side, hero.position - side];
        let point = choices
            .into_iter()
            .filter(|point| safe_walk(world, hero, *point, seen))
            .min_by(|a, b| {
                spells::threat(seen, hero.team, *a).total_cmp(&spells::threat(seen, hero.team, *b))
            });
        if let Some(point) = point {
            return Some(UnitCommand::MoveTo(point));
        }
    }
    None
}
