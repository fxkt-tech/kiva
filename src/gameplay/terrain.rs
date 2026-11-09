//! Deterministic, renderer-independent surfaces shared by meshes and colliders.

use bevy::prelude::*;

pub const HILL_PEAK: Vec3 = Vec3::new(16.5, 2.6, 8.5);
pub const RAMP_HEIGHT: f32 = 1.5;
pub const RAMP_ANGLE: f32 = 20.0_f32.to_radians();
pub const RAMP_LEFT: f32 = -6.0;
pub const RAMP_RIGHT: f32 = -3.0;
pub const RAMP_START: f32 = 1.0;
const GRID_SPACING: f32 = 0.5;

#[derive(Clone, Debug)]
pub struct TerrainGeometry {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
}

impl TerrainGeometry {
    /// One continuous surface includes the flat courtyard and its small hill.
    /// The extra half metre matches the existing visible ground and low rim.
    pub fn ground(half_size: f32) -> Self {
        assert!(half_size.is_finite() && half_size > 0.0);
        let extent = half_size + 0.5;
        let cells = (extent * 2.0 / GRID_SPACING).ceil() as usize;
        let stride = cells + 1;
        let step = extent * 2.0 / cells as f32;
        let mut vertices = Vec::with_capacity(stride * stride);
        for z in 0..=cells {
            for x in 0..=cells {
                let x = -extent + x as f32 * step;
                let z = -extent + z as f32 * step;
                vertices.push(Vec3::new(x, hill_height(Vec2::new(x, z)), z));
            }
        }
        let mut triangles = Vec::with_capacity(cells * cells * 2);
        for z in 0..cells {
            for x in 0..cells {
                let a = (z * stride + x) as u32;
                let b = a + 1;
                let c = a + stride as u32;
                let d = c + 1;
                // z × x points up. Render and collision use this exact split.
                triangles.extend([[a, c, b], [b, c, d]]);
            }
        }
        Self {
            vertices,
            triangles,
        }
    }

    /// A closed solid with one uninterrupted ramp-to-platform top surface.
    pub fn ramp() -> Self {
        let summit = ramp_summit();
        let end = summit + 3.0;
        let mut vertices = Vec::with_capacity(8);
        for x in [RAMP_LEFT, RAMP_RIGHT] {
            vertices.extend([
                Vec3::new(x, 0.0, RAMP_START),
                Vec3::new(x, RAMP_HEIGHT, summit),
                Vec3::new(x, RAMP_HEIGHT, end),
                Vec3::new(x, 0.0, end),
            ]);
        }
        let triangles = vec![
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 4],
            [4, 1, 5],
            [1, 2, 5],
            [5, 2, 6],
            [2, 3, 6],
            [6, 3, 7],
            [0, 4, 3],
            [4, 7, 3],
        ];
        Self {
            vertices,
            triangles,
        }
    }
}

pub fn ramp_summit() -> f32 {
    RAMP_START + RAMP_HEIGHT / RAMP_ANGLE.tan()
}

fn hill_height(point: Vec2) -> f32 {
    let dx = point.x - HILL_PEAK.x;
    let dz = point.y - HILL_PEAK.z;
    let radius_x = if dx < 0.0 { 5.5 } else { 3.0 };
    let q = (dx / radius_x).powi(2) + (dz / 4.5).powi(2);
    if q < 1.0 {
        HILL_PEAK.y * (1.0 - q).powi(2)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn normal(geometry: &TerrainGeometry, triangle: &[u32; 3]) -> Vec3 {
        let [a, b, c] = triangle.map(|index| geometry.vertices[index as usize]);
        (b - a).cross(c - a)
    }

    #[test]
    fn ground_is_one_finite_upward_mesh_with_a_flat_outer_edge() {
        let geometry = TerrainGeometry::ground(25.0);
        assert_eq!(geometry.vertices.len(), 103 * 103);
        assert_eq!(geometry.triangles.len(), 102 * 102 * 2);
        for vertex in &geometry.vertices {
            assert!(vertex.is_finite());
            assert!((0.0..=HILL_PEAK.y).contains(&vertex.y));
            if vertex.x.abs() == 25.5 || vertex.z.abs() == 25.5 {
                assert_eq!(vertex.y, 0.0);
            }
        }
        assert!(geometry.vertices.contains(&HILL_PEAK));
        for triangle in &geometry.triangles {
            assert!(
                triangle
                    .iter()
                    .all(|index| (*index as usize) < geometry.vertices.len())
            );
            assert!(normal(&geometry, triangle).y > 0.0);
        }
    }

    #[test]
    fn small_hill_has_a_walkable_approach_and_an_over_limit_flank() {
        let geometry = TerrainGeometry::ground(25.0);
        let mut west_angle = 0.0_f32;
        let mut east_angle = 0.0_f32;
        for triangle in &geometry.triangles {
            let [a, b, c] = triangle.map(|index| geometry.vertices[index as usize]);
            let center = (a + b + c) / 3.0;
            let angle = normal(&geometry, triangle)
                .normalize()
                .angle_between(Vec3::Y);
            if center.x < HILL_PEAK.x {
                west_angle = west_angle.max(angle);
            } else {
                east_angle = east_angle.max(angle);
            }
        }
        assert!(west_angle > 30.0_f32.to_radians());
        assert!(west_angle < 45.0_f32.to_radians());
        assert!(east_angle > 45.0_f32.to_radians());
        assert!(east_angle < 60.0_f32.to_radians());
        for point in [
            Vec2::new(11.0, 8.5),
            Vec2::new(19.5, 8.5),
            Vec2::new(16.5, 4.0),
            Vec2::new(16.5, 13.0),
        ] {
            assert_eq!(hill_height(point), 0.0);
        }
    }

    #[test]
    fn ramp_and_platform_form_a_closed_solid_with_the_requested_top_slope() {
        let geometry = TerrainGeometry::ramp();
        let mut edges = HashMap::new();
        let mut sloped_tops = 0;
        let mut flat_tops = 0;
        for triangle in &geometry.triangles {
            let normal = normal(&geometry, triangle);
            assert!(normal.length_squared() > 0.0001);
            if normal.y > 0.0 {
                let angle = normal.normalize().angle_between(Vec3::Y);
                if angle > 0.01 {
                    assert!((angle - RAMP_ANGLE).abs() < 0.0001);
                    sloped_tops += 1;
                } else {
                    flat_tops += 1;
                }
            }
            for (a, b) in [
                (triangle[0], triangle[1]),
                (triangle[1], triangle[2]),
                (triangle[2], triangle[0]),
            ] {
                let key = (a.min(b), a.max(b));
                let (count, orientation) = edges.entry(key).or_insert((0, 0));
                *count += 1;
                *orientation += if a < b { 1 } else { -1 };
            }
        }
        assert_eq!(sloped_tops, 2);
        assert_eq!(flat_tops, 2);
        assert!(
            edges
                .values()
                .all(|&(count, direction)| count == 2 && direction == 0)
        );
        assert_eq!(geometry.vertices[0].y, 0.0);
        assert_eq!(geometry.vertices[4].y, 0.0);
        assert_eq!(geometry.vertices[1].z, geometry.vertices[5].z);
        assert_eq!(geometry.vertices[1].y, RAMP_HEIGHT);
        assert!((ramp_summit() - 5.1212).abs() < 0.0001);
    }
}
