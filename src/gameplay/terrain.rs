//! Deterministic, renderer-independent surfaces shared by meshes and colliders.

use bevy::prelude::*;

pub const HILL_PEAK: Vec3 = Vec3::new(16.5, 2.6, 8.5);
const GRID_SPACING: f32 = 0.5;

#[derive(Clone, Debug, Default)]
pub struct TerrainGeometry {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
}

impl TerrainGeometry {
    pub fn bamboo_trunks(stems: &[super::Bamboo]) -> Self {
        let mut geometry = Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        };
        for stem in stems {
            let offset = geometry.vertices.len() as u32;
            for level in 0..=4 {
                let fraction = level as f32 / 4.0;
                let center = stem.center_at(fraction);
                let radius = stem.radius_at(fraction);
                for side in 0..8 {
                    let angle = side as f32 * std::f32::consts::TAU / 8.0;
                    geometry
                        .vertices
                        .push(center + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius));
                }
            }
            for level in 0..4 {
                for side in 0..8 {
                    let a = offset + level * 8 + side;
                    let b = offset + level * 8 + (side + 1) % 8;
                    geometry
                        .triangles
                        .extend([[a, a + 8, b], [b, a + 8, b + 8]]);
                }
            }
            let bottom = geometry.vertices.len() as u32;
            geometry.vertices.push(stem.base);
            let top = geometry.vertices.len() as u32;
            geometry.vertices.push(stem.center_at(1.0));
            for side in 0..8 {
                let next = (side + 1) % 8;
                geometry
                    .triangles
                    .push([bottom, offset + side, offset + next]);
                geometry
                    .triangles
                    .push([top, offset + 32 + next, offset + 32 + side]);
            }
        }
        geometry
    }

    pub fn forest_rocks() -> Self {
        let mut geometry = Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        };
        for (center, size) in [
            (Vec2::new(-19.0, -17.0), Vec3::new(2.8, 1.3, 2.1)),
            (Vec2::new(18.0, -18.0), Vec3::new(2.0, 1.6, 3.1)),
            (Vec2::new(-18.0, 17.0), Vec3::new(2.7, 1.2, 2.3)),
            (Vec2::new(20.0, 19.0), Vec3::new(2.8, 1.7, 2.2)),
            (Vec2::new(-11.0, 18.0), Vec3::new(1.8, 0.7, 2.3)),
            (Vec2::new(6.0, 20.0), Vec3::new(2.7, 1.1, 1.9)),
        ] {
            let base = Vec3::new(center.x, hill_height(center) - 0.03, center.y);
            let offset = geometry.vertices.len() as u32;
            for (height, width) in [(0.0, 0.86), (0.48, 1.0), (0.9, 0.58)] {
                for side in 0..8 {
                    let angle = side as f32 * std::f32::consts::TAU / 8.0;
                    geometry.vertices.push(
                        base + Vec3::new(
                            angle.cos() * size.x * width * 0.5,
                            size.y * height,
                            angle.sin() * size.z * width * 0.5,
                        ),
                    );
                }
            }
            for level in 0..2 {
                for side in 0..8 {
                    let a = offset + level * 8 + side;
                    let b = offset + level * 8 + (side + 1) % 8;
                    geometry
                        .triangles
                        .extend([[a, a + 8, b], [b, a + 8, b + 8]]);
                }
            }
            let bottom = geometry.vertices.len() as u32;
            geometry.vertices.push(base);
            let top = geometry.vertices.len() as u32;
            geometry.vertices.push(base + Vec3::Y * size.y);
            for side in 0..8 {
                let next = (side + 1) % 8;
                geometry
                    .triangles
                    .push([bottom, offset + side, offset + next]);
                geometry
                    .triangles
                    .push([top, offset + 16 + next, offset + 16 + side]);
            }
        }
        geometry
    }

    /// One continuous surface includes the woodland clearing and its small hill.
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
}

pub fn hill_height(point: Vec2) -> f32 {
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
}
