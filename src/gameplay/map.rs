//! One immutable map definition drives navigation, spawn geometry and scenery.
use crate::simulation::{NavigationMap, Obstacle, Team};
use bevy::prelude::*;

#[derive(Resource, Clone, Debug)]
pub struct MapDefinition {
    half_size: f32,
    lane_width: f32,
    bases: [Vec2; 2],
    obstacles: Vec<Obstacle>,
}

impl Default for MapDefinition {
    fn default() -> Self {
        Self {
            half_size: 32.0,
            lane_width: 7.0,
            bases: [Vec2::new(-8.0, 24.0), Vec2::new(8.0, -24.0)],
            // The bases are walkable platforms. Blocking their target centers
            // would invalidate generic spawn and attack line-of-sight checks.
            obstacles: vec![],
        }
    }
}

impl MapDefinition {
    pub fn new(
        half_size: f32,
        lane_width: f32,
        bases: [Vec2; 2],
        obstacles: Vec<Obstacle>,
    ) -> Result<Self, &'static str> {
        if !half_size.is_finite()
            || half_size < 8.0
            || !lane_width.is_finite()
            || !(2.0..=half_size).contains(&lane_width)
            || bases
                .iter()
                .any(|p| !p.is_finite() || p.abs().cmpge(Vec2::splat(half_size)).any())
            || bases[0].as_dvec2().distance(bases[1].as_dvec2()) < 8.0
            || obstacles.iter().any(|o| {
                !o.center.is_finite()
                    || !o.half_size.is_finite()
                    || !o.half_size.cmpgt(Vec2::ZERO).all()
            })
        {
            return Err("invalid single-lane map");
        }
        Ok(Self {
            half_size,
            lane_width,
            bases,
            obstacles,
        })
    }

    pub fn half_size(&self) -> f32 {
        self.half_size
    }
    pub fn lane_width(&self) -> f32 {
        self.lane_width
    }
    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }
    pub fn base_position(&self, team: Team) -> Vec2 {
        self.bases[team_index(team)]
    }

    pub fn lane_direction(&self, team: Team) -> Vec2 {
        let from = self.base_position(team);
        let to = self.base_position(opponent(team));
        (to.as_dvec2() - from.as_dvec2()).normalize().as_vec2()
    }

    pub fn lane_lateral(&self, team: Team) -> Vec2 {
        let direction = self.lane_direction(team);
        Vec2::new(-direction.y, direction.x)
    }

    pub fn lane_length(&self) -> f32 {
        self.bases[0].distance(self.bases[1])
    }

    pub fn hero_spawn(&self, team: Team) -> Vec2 {
        self.base_position(team) + self.lane_direction(team) * 2.0 + self.lane_lateral(team) * 1.4
    }

    /// Slots 0..3 are the leading melee row; slots 3..6 are ranged.
    pub fn creep_spawns(&self, team: Team) -> [Vec2; 6] {
        std::array::from_fn(|slot| {
            let forward = if slot < 3 { 4.0 } else { 2.6 };
            let lateral = (slot % 3) as f32 - 1.0;
            self.base_position(team)
                + self.lane_direction(team) * forward
                + self.lane_lateral(team) * lateral * 0.9
        })
    }

    pub fn navigation(&self) -> NavigationMap {
        NavigationMap::new(self.half_size, 1.0, self.obstacles.clone())
    }
}

pub(super) const fn team_index(team: Team) -> usize {
    match team {
        Team::Ally => 0,
        Team::Enemy => 1,
    }
}

pub const fn opponent(team: Team) -> Team {
    match team {
        Team::Ally => Team::Enemy,
        Team::Enemy => Team::Ally,
    }
}
