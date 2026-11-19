//! Small batches of painted geometry. Vertex pigments are linear for Bevy's PBR shader.
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

pub(super) type Pigment = [u8; 3];

#[derive(Default)]
pub(super) struct Paint {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl Paint {
    pub fn face(&mut self, points: &[Vec3], uv: &[Vec2], pigment: Pigment, shade: f32) {
        let normal = (points[1] - points[0])
            .cross(points[2] - points[0])
            .normalize_or_zero();
        if normal == Vec3::ZERO {
            return;
        }
        let start = self.positions.len() as u32;
        let color = Color::srgb_u8(pigment[0], pigment[1], pigment[2]).to_linear();
        let color = [
            color.red * shade,
            color.green * shade,
            color.blue * shade,
            1.0,
        ];
        for (point, uv) in points.iter().zip(uv) {
            self.positions.push(point.to_array());
            self.normals.push(normal.to_array());
            self.colors.push(color);
            self.uvs.push(uv.to_array());
        }
        for index in 1..points.len() as u32 - 1 {
            self.indices
                .extend([start, start + index, start + index + 1]);
        }
    }

    pub fn polygon(&mut self, points: &[Vec3], pigment: Pigment) {
        let normal = (points[1] - points[0])
            .cross(points[2] - points[0])
            .normalize_or_zero();
        let shade = 0.70 + 0.30 * normal.dot(Vec3::new(-0.40, 0.74, 0.54)).max(0.0);
        let uv: Vec<_> = points
            .iter()
            .map(|p| {
                // Keep grain density in world units on every oriented face.
                let a = normal.abs();
                let uv = if a.y >= a.x.max(a.z) {
                    p.xz()
                } else if a.x >= a.z {
                    p.zy()
                } else {
                    p.xy()
                };
                uv * 2.6
            })
            .collect();
        self.face(points, &uv, pigment, shade);
    }

    pub fn block(&mut self, center: Vec3, size: Vec3, pigment: Pigment) {
        let scaled = |x, y, z| center + size * Vec3::new(x, y, z) * 0.5;
        let layer = |y| {
            [
                scaled(-1., y, 1.),
                scaled(1., y, 1.),
                scaled(1., y, -1.),
                scaled(-1., y, -1.),
            ]
        };
        let a = layer(-1.);
        let b = layer(1.);
        self.polygon(&b, pigment);
        self.polygon(&[a[3], a[2], a[1], a[0]], pigment);
        for i in 0..4 {
            let j = (i + 1) % 4;
            self.polygon(&[a[i], a[j], b[j], b[i]], pigment);
        }
    }

    /// Tube between arbitrary anchors. Elliptical sections can taper; x/z radii
    /// are measured perpendicular to the axis. Useful for cast iron and branches.
    pub fn tube(&mut self, a: Vec3, b: Vec3, radii: Vec2, sides: usize, pigment: Pigment) {
        let axis = (b - a).normalize_or(Vec3::Y);
        let right = axis.any_orthonormal_vector();
        let up = axis.cross(right);
        let ring = |i: usize| {
            let (sin, cos) = (std::f32::consts::TAU * i as f32 / sides as f32).sin_cos();
            cos * right + sin * up
        };
        for i in 0..sides {
            let u = ring(i);
            let v = ring(i + 1);
            self.polygon(
                &[
                    a + u * radii.x,
                    a + v * radii.x,
                    b + v * radii.y,
                    b + u * radii.y,
                ],
                pigment,
            );
        }
        let cap: Vec<_> = (0..sides).map(|i| b + ring(i) * radii.y).collect();
        self.polygon(&cap, pigment);
    }

    /// Horizontal elliptical bands around a tailored form. Offset band centers
    /// model bent limbs, folded garments and the hairline.
    pub fn bands(&mut self, bands: &[(Vec3, Vec2)], sides: usize, pigment: Pigment) {
        let at = |band: &(Vec3, Vec2), i: usize| {
            let (s, c) = (std::f32::consts::TAU * i as f32 / sides as f32).sin_cos();
            band.0 + Vec3::new(c * band.1.x, 0., s * band.1.y)
        };
        for pair in bands.windows(2) {
            for i in 0..sides {
                self.polygon(
                    &[
                        at(&pair[0], i + 1),
                        at(&pair[0], i),
                        at(&pair[1], i),
                        at(&pair[1], i + 1),
                    ],
                    pigment,
                );
            }
        }
        for (band, reverse) in [(bands[0], false), (*bands.last().unwrap(), true)] {
            let mut ring: Vec<_> = (0..sides).map(|i| at(&band, i)).collect();
            if reverse {
                ring.reverse();
            }
            self.polygon(&ring, pigment);
        }
    }

    pub fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

pub fn field(index: usize, salt: u32) -> f32 {
    let mut n = (index as u32).wrapping_mul(0x9e3779b9).wrapping_add(salt);
    n ^= n >> 16;
    n = n.wrapping_mul(0x85ebca6b);
    n ^= n >> 13;
    n as f32 / u32::MAX as f32
}

pub(super) fn finish<M: Material>(
    world: &mut World,
    mesh: Mesh,
    mat: &Handle<M>,
    transform: Transform,
) -> Entity {
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
    world
        .spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform))
        .id()
}

pub(super) fn ink(world: &mut World, texture: Option<Handle<Image>>) -> Handle<StandardMaterial> {
    world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color_texture: texture,
            unlit: true,
            cull_mode: None,
            ..default()
        })
}
