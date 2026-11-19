//! The platform geometry shared by physics, the station scenery and its puddle.
use bevy::prelude::*;

use super::Boundary;
use crate::simulation::spawn_box;

#[derive(Resource, Clone, Copy)]
pub struct StationLayout {
    /// Inside edges of the platform rim in world x/z.
    pub walk_min: Vec2,
    pub walk_max: Vec2,
    /// Player, sister and pillar positions are at their feet.
    pub spawn: Vec3,
    pub sister: Vec3,
    pub pillar: Vec3,
    pub pillar_size: Vec3,
    pub puddle: Vec3,
    pub puddle_half_size: Vec2,
    pub window: Vec3,
}

pub const LAYOUT: StationLayout = StationLayout {
    walk_min: Vec2::new(-4.5, -32.0),
    walk_max: Vec2::new(4.3, 7.0),
    spawn: Vec3::new(-0.45, 0.0, 2.5),
    sister: Vec3::new(0.9, 0.0, -1.3),
    pillar: Vec3::new(1.22, 0.0, -0.42),
    pillar_size: Vec3::new(0.42, 3.32, 0.42),
    puddle: Vec3::new(0.0, 0.022, 1.0),
    puddle_half_size: Vec2::new(3.8, 4.0),
    window: Vec3::new(5.1, 1.65, -4.05),
};

// The fighting area is beyond the initial 22m automatic targeting range.
pub(super) const ENEMY_SPAWN: Vec2 = Vec2::new(0.0, -29.0);

const CURB: f32 = 0.2;

impl StationLayout {
    pub fn curbs(&self) -> [Boundary; 4] {
        let midpoint = (self.walk_min + self.walk_max) * 0.5;
        let size = self.walk_max - self.walk_min;
        let side = |x| Boundary {
            center: Vec3::new(x, 0.35, midpoint.y),
            size: Vec3::new(CURB, 0.7, size.y),
        };
        let end = |z| Boundary {
            center: Vec3::new(midpoint.x, 0.35, z),
            size: Vec3::new(size.x + 2.0 * CURB, 0.7, CURB),
        };
        [
            side(self.walk_min.x - CURB * 0.5),
            side(self.walk_max.x + CURB * 0.5),
            end(self.walk_min.y - CURB * 0.5),
            end(self.walk_max.y + CURB * 0.5),
        ]
    }
}

pub(super) fn setup(world: &mut World) {
    let layout = LAYOUT;
    let midpoint = (layout.walk_min + layout.walk_max) * 0.5;
    let size = layout.walk_max - layout.walk_min;
    spawn_box(
        world,
        Vec3::new(midpoint.x, -0.20, midpoint.y),
        Vec3::new(size.x + 2.0 * CURB, 0.40, size.y + 2.0 * CURB),
        false,
    );
    for curb in layout.curbs() {
        let entity = spawn_box(world, curb.center, curb.size, false);
        world.entity_mut(entity).insert(curb);
    }
    spawn_box(
        world,
        layout.pillar + Vec3::Y * (layout.pillar_size.y * 0.5),
        layout.pillar_size,
        false,
    );
    world.insert_resource(layout);
}
