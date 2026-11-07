//! A shared immutable observation snapshot for commands, AI and presentation.
use super::*;
use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct TeamVision {
    enabled: bool,
    circles: [Vec<(Vec2, f32)>; 2],
}
fn index(team: Team) -> usize {
    match team {
        Team::Ally => 0,
        Team::Enemy => 1,
    }
}
impl TeamVision {
    pub fn point_visible(&self, team: Team, point: Vec2) -> bool {
        !self.enabled
            || self.circles[index(team)]
                .iter()
                .any(|(center, radius)| center.distance_squared(point) <= radius * radius)
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
pub fn point_visible(world: &World, team: Team, point: Vec2) -> bool {
    world.resource::<TeamVision>().point_visible(team, point)
}
pub fn team_can_see(world: &World, team: Team, entity: Entity) -> bool {
    if let Some(unit) = world.get::<Unit>(entity)
        && unit.team == team
    {
        return true;
    }
    world
        .get::<UnitPose>(entity)
        .is_some_and(|pose| point_visible(world, team, pose.position))
        || world
            .get::<BattlefieldObject>(entity)
            .is_some_and(|object| {
                object.team == team || point_visible(world, team, object.position)
            })
}
pub(super) fn refresh(world: &mut World) {
    let mut vision = TeamVision::default();
    let tick = world.resource::<SimulationTick>().0;
    for (unit, state, pose) in world.query::<(&Unit, &UnitState, &UnitPose)>().iter(world) {
        vision.enabled |= unit.stats.vision_radius > 0.0;
        if state.alive() && unit.stats.vision_radius > 0.0 {
            vision.circles[index(unit.team)].push((pose.position, unit.stats.vision_radius));
        }
    }
    for object in world.query::<&BattlefieldObject>().iter(world) {
        if object.active_at <= tick && object.until > tick && object.vision > 0.0 {
            vision.circles[index(object.team)].push((object.position, object.vision));
        }
    }
    for projectile in world.query::<&Projectile>().iter(world) {
        if let super::mechanics::ProjectileBehavior::Arrow { spec, .. } = &projectile.behavior {
            vision.circles[index(projectile.payload.team)]
                .push((projectile.position, spec.vision_radius));
        }
    }
    *world.resource_mut::<TeamVision>() = vision;
}
