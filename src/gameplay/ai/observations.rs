use crate::{gameplay::UnitRole, simulation::*};
use bevy::prelude::*;

/// Publicly observable combat facts. Enemy plans, queued input and cooldowns
/// are deliberately absent; actual loadout/resource reads are made only for
/// the deciding unit itself.
#[derive(Clone)]
pub(super) struct SeenUnit {
    pub entity: Entity,
    pub life: UnitLifeRef,
    pub role: UnitRole,
    pub team: Team,
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub health: f32,
    pub max_health: f32,
}
impl SeenUnit {
    pub fn is_minion(&self) -> bool {
        matches!(self.role, UnitRole::MeleeMinion | UnitRole::RangedMinion)
    }
    pub fn health_fraction(&self) -> f32 {
        self.health / self.max_health
    }
}

pub(super) struct TeamObservations {
    ally: Vec<SeenUnit>,
    enemy: Vec<SeenUnit>,
}
impl TeamObservations {
    pub fn for_team(&self, team: Team) -> &[SeenUnit] {
        match team {
            Team::Ally => &self.ally,
            Team::Enemy => &self.enemy,
        }
    }
}

pub(super) fn observe(world: &mut World) -> TeamObservations {
    let mut seen: Vec<_> = world
        .query::<(Entity, &Unit, &UnitState, &UnitPose, &UnitRole)>()
        .iter(world)
        .filter(|(_, _, state, _, _)| state.alive())
        .map(|(entity, unit, state, pose, role)| SeenUnit {
            entity,
            life: state.life_ref(entity),
            role: *role,
            team: unit.team(),
            position: pose.position(),
            velocity: (pose.position() - pose.interpolated(0.0)) / TICK_SECONDS,
            radius: unit.definition().radius,
            health: state.health(),
            max_health: unit.definition().max_health,
        })
        .collect();
    seen.sort_by_key(|unit| unit.entity.to_bits());
    let visible = |team| {
        seen.iter()
            .filter(|unit| team_can_see(world, team, unit.entity))
            .cloned()
            .collect()
    };
    TeamObservations {
        ally: visible(Team::Ally),
        enemy: visible(Team::Enemy),
    }
}

pub(super) fn current_target(seen: &[SeenUnit], target: UnitLifeRef) -> Option<&SeenUnit> {
    seen.iter().find(|unit| unit.life == target)
}

pub(super) fn redundant(world: &World, unit: Entity, command: UnitCommand) -> bool {
    match (world.get::<UnitOrder>(unit), command) {
        (Some(UnitOrder::AttackTarget(current)), UnitCommand::AttackTarget(target)) => {
            current.entity() == target
                && world
                    .get::<UnitState>(target)
                    .is_some_and(|state| state.alive() && state.life_ref(target) == *current)
        }
        (Some(UnitOrder::MoveTo(current)), UnitCommand::MoveTo(point)) => {
            current.distance_squared(point) < 0.16
        }
        (Some(UnitOrder::Idle), UnitCommand::Stop) => {
            matches!(world.get::<UnitAction>(unit), Some(UnitAction::Ready))
        }
        _ => false,
    }
}

pub(super) fn submit(world: &mut World, unit: Entity, command: UnitCommand) {
    if !redundant(world, unit, command) {
        world.resource_mut::<CommandQueue>().submit(unit, command);
    }
}
