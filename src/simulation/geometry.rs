//! Flat-arena descriptions for headless tests only; this module is not part of
//! a product build. Product levels assemble their own Avian terrain and props.
use super::spawn_box;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct SolidBox {
    pub center: Vec2,
    pub half_size: Vec2,
}

impl SolidBox {
    pub fn new(center: Vec2, half_size: Vec2) -> Self {
        assert!(center.is_finite() && half_size.is_finite() && half_size.cmpgt(Vec2::ZERO).all());
        Self { center, half_size }
    }
}

/// Insert after `SimulationPlugin`: the next fixed tick builds a flat walled
/// floor of this half size with these pillars, replacing any previous one.
#[derive(Resource)]
pub struct CollisionWorld {
    half_size: f32,
    solids: Vec<SolidBox>,
}

#[derive(Component)]
struct FixtureGeometry;

/// Runs at the start of `physics::install`, before new actors query support.
pub(super) fn install_fixture(world: &mut World) {
    let Some(fixture) = world
        .get_resource_ref::<CollisionWorld>()
        .filter(|fixture| fixture.is_changed())
    else {
        return;
    };
    let (half_size, solids) = (fixture.half_size, fixture.solids.clone());
    let old: Vec<_> = world
        .query_filtered::<Entity, With<FixtureGeometry>>()
        .iter(world)
        .collect();
    for entity in old {
        world.despawn(entity);
    }
    let span = half_size * 2.0 + 1.0;
    let mut boxes = vec![(Vec3::new(0.0, -0.25, 0.0), Vec3::new(span, 0.5, span))];
    boxes.extend(solids.iter().map(|solid| {
        (
            Vec3::new(solid.center.x, 2.0, solid.center.y),
            Vec3::new(solid.half_size.x * 2.0, 4.0, solid.half_size.y * 2.0),
        )
    }));
    for sign in [-1.0, 1.0] {
        for axis in [0, 2] {
            let mut center = Vec3::new(0.0, 0.35, 0.0);
            center[axis] = sign * (half_size + 0.25);
            let mut size = Vec3::new(span, 0.7, span);
            size[axis] = 0.5;
            boxes.push((center, size));
        }
    }
    for (center, size) in boxes {
        let entity = spawn_box(world, center, size, false);
        world.entity_mut(entity).insert(FixtureGeometry);
    }
}

impl CollisionWorld {
    pub fn new(half_size: f32, solids: Vec<SolidBox>) -> Self {
        assert!(half_size.is_finite() && half_size > 0.0);
        Self { half_size, solids }
    }

    pub fn is_clear(&self, point: Vec2, half_extent: f32) -> bool {
        point.is_finite()
            && half_extent.is_finite()
            && half_extent > 0.0
            && point
                .abs()
                .cmple(Vec2::splat(self.half_size - half_extent))
                .all()
            && self.solids.iter().all(|solid| {
                !(point - solid.center)
                    .abs()
                    .cmplt(solid.half_size + Vec2::splat(half_extent))
                    .all()
            })
    }

    pub(super) fn slide(&self, start: Vec2, displacement: Vec2, half_extent: f32) -> Vec2 {
        if !displacement.is_finite() {
            return start;
        }
        let bound = Vec2::splat(self.half_size - half_extent);
        let mut remaining = (start + displacement).clamp(-bound, bound) - start;
        let mut position = start;
        for _ in 0..4 {
            if remaining.length_squared() < 1e-12 {
                break;
            }
            let hit = self
                .solids
                .iter()
                .filter_map(|solid| cast(position, remaining, *solid, half_extent))
                .min_by(|a, b| a.time.total_cmp(&b.time));
            let Some(hit) = hit else {
                position += remaining;
                break;
            };
            // Leave a world-space skin so rounding cannot place the next sweep
            // inside a face. Only the blocked normal component is removed.
            let before_contact = (hit.time - 0.0001 / remaining.length()).max(0.0);
            position += remaining * before_contact;
            remaining *= 1.0 - hit.time;
            remaining -= hit.normal * remaining.dot(hit.normal).min(0.0);
        }
        position.clamp(-bound, bound)
    }
}

struct Hit {
    time: f32,
    normal: Vec2,
}

fn cast(origin: Vec2, delta: Vec2, solid: SolidBox, extent: f32) -> Option<Hit> {
    let min = solid.center - solid.half_size - Vec2::splat(extent);
    let max = solid.center + solid.half_size + Vec2::splat(extent);
    let mut enter = f32::NEG_INFINITY;
    let mut exit = f32::INFINITY;
    let mut normal = Vec2::ZERO;
    for axis in 0..2 {
        if delta[axis].abs() < 1e-8 {
            // A path exactly on a face is tangential, not penetrating.
            if origin[axis] <= min[axis] || origin[axis] >= max[axis] {
                return None;
            }
            continue;
        }
        let a = (min[axis] - origin[axis]) / delta[axis];
        let b = (max[axis] - origin[axis]) / delta[axis];
        if a.min(b) > enter {
            enter = a.min(b);
            normal = Vec2::ZERO;
            normal[axis] = -delta[axis].signum();
        }
        exit = exit.min(a.max(b));
    }
    (enter <= exit && (0.0..=1.0).contains(&enter) && delta.dot(normal) < 0.0).then_some(Hit {
        time: enter,
        normal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> CollisionWorld {
        CollisionWorld::new(20.0, vec![SolidBox::new(Vec2::ZERO, Vec2::new(0.5, 3.0))])
    }

    #[test]
    fn a_long_step_cannot_tunnel_and_preserves_motion_along_the_wall() {
        let world = world();
        let point = world.slide(Vec2::new(-4.0, 0.0), Vec2::new(12.0, 2.0), 0.35);
        assert!((point.x + 0.85).abs() < 0.001);
        assert!((point.y - 2.0).abs() < 0.001);
        assert!(world.is_clear(point, 0.35));
    }

    #[test]
    fn touching_a_face_allows_tangential_motion_and_moving_away() {
        let world = world();
        let start = Vec2::new(-0.85, 0.0);
        assert!(
            world
                .slide(start, Vec2::NEG_X, 0.35)
                .distance(start + Vec2::NEG_X)
                < 0.001
        );
        assert!(world.slide(start, Vec2::Y, 0.35).distance(start + Vec2::Y) < 0.001);
        let blocked = world.slide(start, Vec2::X, 0.35);
        assert!(world.is_clear(blocked, 0.35));
        assert!(blocked.distance(start) < 0.001);
    }

    #[test]
    fn corners_and_map_edges_keep_the_whole_footprint_outside_solids() {
        let world = CollisionWorld::new(10.0, vec![SolidBox::new(Vec2::ZERO, Vec2::ONE)]);
        for delta in [
            Vec2::new(8.0, 8.0),
            Vec2::new(30.0, 1.0),
            Vec2::new(-30.0, -30.0),
        ] {
            let point = world.slide(Vec2::splat(-4.0), delta, 0.35);
            assert!(world.is_clear(point, 0.35));
        }
        assert!(!world.is_clear(Vec2::ZERO, 0.35));
        assert!(!world.is_clear(Vec2::splat(9.9), 0.35));
    }
}
