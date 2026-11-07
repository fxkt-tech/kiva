use super::{
    DECISION_TICKS,
    observations::{self, SeenUnit},
};
use crate::{
    gameplay::{Controller, MapDefinition, MatchState, UnitRole, WaveMember, opponent},
    simulation::*,
};
use bevy::prelude::*;

#[derive(Component)]
struct MinionBrain {
    life: UnitLifeRef,
    next_tick: u64,
    target: Option<UnitLifeRef>,
}

pub(super) fn decide(world: &mut World) {
    if !world.resource::<MatchState>().can_advance() || world.resource::<SimulationControl>().paused
    {
        return;
    }
    let tick = world.resource::<SimulationTick>().0;
    let observations = observations::observe(world);
    let map = world.resource::<MapDefinition>().clone();
    let controlled: Vec<_> = world
        .query::<(Entity, &Controller, &UnitState, &Unit)>()
        .iter(world)
        .filter(|(_, controller, state, _)| **controller == Controller::MinionAi && state.alive())
        .map(|(entity, _, state, unit)| (entity, state.life_ref(entity), unit.team()))
        .collect();
    for (entity, life, team) in controlled {
        let seen = observations.for_team(team);
        if world
            .get::<MinionBrain>(entity)
            .is_none_or(|brain| brain.life != life)
        {
            world.entity_mut(entity).insert((
                MinionBrain {
                    life,
                    next_tick: tick + DECISION_TICKS,
                    target: None,
                },
                SoftAvoidance,
            ));
            continue;
        }
        if world.get::<MinionBrain>(entity).expect("brain").next_tick > tick {
            continue;
        }
        world
            .get_mut::<MinionBrain>(entity)
            .expect("brain")
            .next_tick = tick + DECISION_TICKS;
        let Some(unit) = seen.iter().find(|unit| unit.entity == entity) else {
            continue;
        };
        let current = world
            .get::<MinionBrain>(entity)
            .expect("brain")
            .target
            .and_then(|life| observations::current_target(seen, life));
        let candidate = seen
            .iter()
            .filter(|other| {
                other.team != unit.team
                    && other.role != UnitRole::Base
                    && in_band(other.position, unit.position, unit.team, &map, 5.5)
            })
            .min_by(|a, b| {
                (!a.is_minion())
                    .cmp(&(!b.is_minion()))
                    .then_with(|| {
                        a.position
                            .distance_squared(unit.position)
                            .total_cmp(&b.position.distance_squared(unit.position))
                    })
                    .then_with(|| a.entity.to_bits().cmp(&b.entity.to_bits()))
            });
        let current = current.filter(|other| {
            other.role != UnitRole::Base
                && in_band(other.position, unit.position, unit.team, &map, 8.0)
        });
        let preparing = matches!(
            world.get::<UnitAction>(entity),
            Some(UnitAction::AttackWindup { .. })
        );
        let target = match (current, candidate) {
            (Some(current), Some(candidate))
                if !preparing && !current.is_minion() && candidate.is_minion() =>
            {
                Some(candidate)
            }
            (current, candidate) => current.or(candidate),
        };
        let target = target.or_else(|| {
            seen.iter().find(|other| {
                other.role == UnitRole::Base
                    && other.team == opponent(unit.team)
                    && other.position.distance(unit.position)
                        <= attack_reach(world, entity, other) + 0.5
            })
        });
        let target_life = target.map(|target| target.life);
        world.get_mut::<MinionBrain>(entity).expect("brain").target = target_life;
        if is_dashing(world, entity)
            || matches!(
                world.get::<UnitAction>(entity),
                Some(UnitAction::Casting { .. } | UnitAction::Channeling { .. })
            )
        {
            continue;
        }
        if let Some(target) = target {
            if world
                .get::<ControlState>(entity)
                .is_some_and(ControlState::can_attack)
            {
                observations::submit(world, entity, UnitCommand::AttackTarget(target.entity));
            }
        } else if !matches!(
            world.get::<UnitAction>(entity),
            Some(UnitAction::AttackWindup { .. })
        ) && world
            .get::<ControlState>(entity)
            .is_some_and(ControlState::can_move)
        {
            let axis = map.lane_direction(unit.team);
            let progress = (unit.position - map.base_position(unit.team)).dot(axis);
            let distance =
                ((progress.max(0.0) / 2.0).floor() * 2.0 + 4.0).clamp(0.0, map.lane_length() - 1.0);
            let slot = world
                .get::<WaveMember>(entity)
                .map_or(1, |member| member.slot() % 3);
            let lateral = (f32::from(slot) - 1.0) * 0.7;
            let point = map.base_position(unit.team)
                + axis * distance
                + map.lane_lateral(unit.team) * lateral;
            if unit.position.distance_squared(point) > 0.16 {
                observations::submit(world, entity, UnitCommand::MoveTo(point));
            }
        }
    }
}

fn attack_reach(world: &World, source: Entity, target: &SeenUnit) -> f32 {
    let unit = world.get::<Unit>(source).expect("observed unit");
    unit.attributes().attack_range() + unit.definition().radius + target.radius
}

fn in_band(point: Vec2, origin: Vec2, team: Team, map: &MapDefinition, radius: f32) -> bool {
    point.distance_squared(origin) <= radius * radius
        && (point - map.base_position(team))
            .dot(map.lane_lateral(team))
            .abs()
            <= map.lane_width() * 0.5 + 1.0
}
