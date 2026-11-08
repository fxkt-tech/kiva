//! The visible single lane is derived from the same map as navigation/spawns.
use crate::{gameplay::MapDefinition, simulation::Team};
use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

#[derive(Component, Clone, Copy)]
pub(super) enum TerrainSurface {
    Ground,
    Road,
}

#[derive(Resource)]
pub(super) struct TerrainTexture(Handle<Image>);

pub(super) fn load_texture(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(TerrainTexture(server.load("textures/league-rift.png")));
}

/// Keep navigation and map geometry authoritative. The painted source tile is
/// sampled through bounded grass/stone UV patches, not used as a second map.
pub(super) fn apply_texture(
    mut commands: Commands,
    texture: Res<TerrainTexture>,
    images: Res<Assets<Image>>,
    map: Res<MapDefinition>,
    surfaces: Query<(Entity, &TerrainSurface)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if images.get(&texture.0).is_none() || surfaces.is_empty() {
        return;
    }
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture.0.clone()),
        unlit: true,
        perceptual_roughness: 1.0,
        ..default()
    });
    for (entity, surface) in &surfaces {
        let mesh = match surface {
            TerrainSurface::Ground => texture_tiles(
                map.half_size() * 2.0,
                map.half_size() * 2.0,
                32,
                32,
                [[0.905, 0.235], [0.98, 0.235], [0.905, 0.31], [0.98, 0.31]],
            ),
            TerrainSurface::Road => texture_tiles(
                map.lane_length() + 2.4,
                map.lane_width(),
                10,
                1,
                [
                    [0.445, 0.945],
                    [0.725, 0.685],
                    [0.355, 0.855],
                    [0.635, 0.595],
                ],
            ),
        };
        commands
            .entity(entity)
            .insert((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.clone())))
            .remove::<TerrainSurface>();
    }
}

fn texture_tiles(length: f32, width: f32, columns: usize, rows: usize, uv: [[f32; 2]; 4]) -> Mesh {
    let mut positions = Vec::with_capacity(columns * rows * 4);
    let mut normals = Vec::with_capacity(columns * rows * 4);
    let mut uvs = Vec::with_capacity(columns * rows * 4);
    let mut indices = Vec::with_capacity(columns * rows * 6);
    for row in 0..rows {
        for column in 0..columns {
            let x0 = (column as f32 / columns as f32 - 0.5) * length;
            let x1 = ((column + 1) as f32 / columns as f32 - 0.5) * length;
            let z0 = (row as f32 / rows as f32 - 0.5) * width;
            let z1 = ((row + 1) as f32 / rows as f32 - 0.5) * width;
            let base = positions.len() as u32;
            positions.extend([[x0, 0.0, z0], [x1, 0.0, z0], [x0, 0.0, z1], [x1, 0.0, z1]]);
            normals.extend([[0.0, 1.0, 0.0]; 4]);
            // This is a painted atlas patch, not a seamless repeat texture.
            // Mirror adjacent tiles so every shared world edge samples the
            // same texels, including the road's slanted grass/stone boundary.
            for corner in 0..4 {
                let horizontal = (corner % 2) ^ (column % 2);
                let vertical = (corner / 2) ^ (row % 2);
                uvs.push(uv[vertical * 2 + horizontal]);
            }
            indices.extend([base, base + 2, base + 1, base + 1, base + 2, base + 3]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

pub(super) fn spawn_map(
    commands: &mut Commands,
    map: &MapDefinition,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let half = map.half_size();
    let ground = materials.add(Color::srgb_u8(35, 61, 54));
    let road = materials.add(Color::srgb_u8(76, 88, 73));
    let stone = materials.add(Color::srgb_u8(73, 87, 76));
    let foliage = materials.add(Color::srgb_u8(27, 70, 51));
    let foliage_light = materials.add(Color::srgb_u8(39, 86, 61));
    commands.spawn((
        TerrainSurface::Ground,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(half * 2.0, half * 2.0))),
        MeshMaterial3d(ground),
        Transform::default(),
    ));

    let start = map.base_position(Team::Ally);
    let end = map.base_position(Team::Enemy);
    let center = (start + end) * 0.5;
    let direction = map.lane_direction(Team::Ally);
    let rotation = Quat::from_rotation_y(-direction.y.atan2(direction.x));
    commands.spawn((
        TerrainSurface::Road,
        Mesh3d(meshes.add(Cuboid::new(
            map.lane_length() + 2.4,
            0.035,
            map.lane_width(),
        ))),
        MeshMaterial3d(road),
        Transform::from_xyz(center.x, 0.02, center.y).with_rotation(rotation),
    ));

    let border_long = meshes.add(Cuboid::new(half * 2.0 + 0.5, 0.45, 0.45));
    let border_cross = meshes.add(Cuboid::new(0.45, 0.45, half * 2.0 + 0.5));
    for side in [-1.0, 1.0] {
        commands.spawn((
            Mesh3d(border_long.clone()),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(0.0, 0.14, side * (half + 0.23)),
        ));
        commands.spawn((
            Mesh3d(border_cross.clone()),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(side * (half + 0.23), 0.14, 0.0),
        ));
    }
    for obstacle in map.obstacles() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                obstacle.half_size.x * 2.0,
                1.0,
                obstacle.half_size.y * 2.0,
            ))),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(obstacle.center.x, 0.5, obstacle.center.y),
        ));
    }

    // The tree belt is outside the playable square, so decoration cannot add
    // invisible navigation obstacles or obstruct the sole mid lane.
    let trunk = meshes.add(Cylinder::new(0.18, 1.3));
    let crown = meshes.add(Sphere::new(1.0).mesh().ico(1).expect("tree mesh"));
    for side in 0..4 {
        for index in 0..9 {
            let along = -half + 1.0 + index as f32 * (half * 2.0 - 2.0) / 8.0;
            let outside = half + 1.6 + (index % 2) as f32 * 0.7;
            let point = match side {
                0 => Vec2::new(along, outside),
                1 => Vec2::new(along, -outside),
                2 => Vec2::new(outside, along),
                _ => Vec2::new(-outside, along),
            };
            let height = 2.0 + (index % 3) as f32 * 0.35;
            commands.spawn((
                Mesh3d(trunk.clone()),
                MeshMaterial3d(stone.clone()),
                Transform::from_xyz(point.x, 0.65, point.y),
            ));
            commands.spawn((
                Mesh3d(crown.clone()),
                MeshMaterial3d(if index % 2 == 0 {
                    foliage.clone()
                } else {
                    foliage_light.clone()
                }),
                Transform::from_xyz(point.x, height, point.y).with_scale(Vec3::new(
                    1.0,
                    height * 0.8,
                    1.0,
                )),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn painted_terrain_has_identical_uvs_on_both_sides_of_every_tile_edge() {
        // A slanted source patch exercises both the grass and road cases.
        let mesh = texture_tiles(
            12.0,
            8.0,
            3,
            2,
            [[0.4, 0.9], [0.7, 0.65], [0.3, 0.8], [0.6, 0.55]],
        );
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("terrain UVs");
        };
        for row in 0..2 {
            for column in 0..2 {
                let left = (row * 3 + column) * 4;
                let right = left + 4;
                assert_eq!(uvs[left + 1], uvs[right]);
                assert_eq!(uvs[left + 3], uvs[right + 2]);
            }
        }
        for column in 0..3 {
            let top = column * 4;
            let bottom = top + 3 * 4;
            assert_eq!(uvs[top + 2], uvs[bottom]);
            assert_eq!(uvs[top + 3], uvs[bottom + 1]);
        }
    }
}
