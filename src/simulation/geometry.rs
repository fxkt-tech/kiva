//! Legacy planar fixture descriptions. Product motion and queries use Avian's 3D world.
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

#[derive(Resource)]
pub struct CollisionWorld {
    half_size: f32,
    solids: Vec<SolidBox>,
    fixture: bool,
}

impl Default for CollisionWorld {
    fn default() -> Self {
        Self {
            half_size: 25.0,
            solids: Vec::new(),
            fixture: false,
        }
    }
}

impl CollisionWorld {
    pub fn new(half_size: f32, solids: Vec<SolidBox>) -> Self {
        assert!(half_size.is_finite() && half_size > 0.0);
        Self {
            half_size,
            solids,
            fixture: true,
        }
    }

    pub(super) fn fixture_geometry(&self) -> Option<(f32, Vec<SolidBox>)> {
        self.fixture.then(|| (self.half_size, self.solids.clone()))
    }

    pub fn half_size(&self) -> f32 {
        self.half_size
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

    #[cfg(test)]
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

#[cfg(test)]
struct Hit {
    time: f32,
    normal: Vec2,
}

#[cfg(test)]
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
