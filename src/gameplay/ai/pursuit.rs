//! A hero engagement has a spatial leash and a bounded out-of-range chase.
use super::observations::{SeenUnit, current_target};
use crate::{
    gameplay::{MapDefinition, UnitRole},
    simulation::UnitLifeRef,
};
use bevy::prelude::*;

#[derive(Default)]
pub(super) struct Pursuit {
    active: Option<Chase>,
    rejoin_until: u64,
}
struct Chase {
    life: UnitLifeRef,
    origin: Vec2,
    deadline: u64,
}
impl Pursuit {
    pub fn refresh(
        &mut self,
        hero: &SeenUnit,
        seen: &[SeenUnit],
        map: &MapDefinition,
        tick: u64,
        reach: f32,
    ) {
        let Some(chase) = &mut self.active else {
            return;
        };
        let Some(target) = current_target(seen, chase.life) else {
            self.active = None;
            return;
        };
        let in_range = hero.position.distance(target.position) <= reach + target.radius;
        if !in_lane(target, map, hero.team)
            || hero.position.distance(chase.origin) > 6.0
            || (!in_range && tick >= chase.deadline)
            || hero.position.distance(target.position) > 12.0
        {
            self.active = None;
            self.rejoin_until = tick + 180;
        } else if in_range {
            chase.deadline = tick + 240;
        }
    }

    pub fn allows(
        &self,
        hero: &SeenUnit,
        target: &SeenUnit,
        map: &MapDefinition,
        tick: u64,
    ) -> bool {
        target.team != hero.team
            && target.role != UnitRole::Base
            && hero.position.distance(target.position) <= 12.0
            && (target.role != UnitRole::Hero
                || (tick >= self.rejoin_until && in_lane(target, map, hero.team)))
    }

    pub fn select(&mut self, hero: &SeenUnit, target: Option<&SeenUnit>, tick: u64) {
        let target = target.filter(|target| target.role == UnitRole::Hero);
        if let Some(target) = target {
            if self
                .active
                .as_ref()
                .is_none_or(|chase| chase.life != target.life)
            {
                self.active = Some(Chase {
                    life: target.life,
                    origin: hero.position,
                    deadline: tick + 240,
                });
            }
        } else {
            self.active = None;
        }
    }

    pub fn clear(&mut self) {
        self.active = None;
    }
}

fn in_lane(target: &SeenUnit, map: &MapDefinition, team: crate::simulation::Team) -> bool {
    (target.position - map.base_position(team))
        .dot(map.lane_lateral(team))
        .abs()
        <= map.lane_width() * 0.5 + 2.0
}
