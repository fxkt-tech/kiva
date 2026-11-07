//! The visible single lane is derived from the same map as navigation/spawns.
use crate::{gameplay::MapDefinition, simulation::Team};
use bevy::prelude::*;

pub(super) fn spawn_map(
    commands: &mut Commands,
    map: &MapDefinition,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let half = map.half_size();
    let ground = materials.add(Color::srgb_u8(35, 61, 54));
    let road = materials.add(Color::srgb_u8(76, 88, 73));
    let paving = materials.add(Color::srgb_u8(100, 111, 91));
    let stone = materials.add(Color::srgb_u8(73, 87, 76));
    let foliage = materials.add(Color::srgb_u8(27, 70, 51));
    let foliage_light = materials.add(Color::srgb_u8(39, 86, 61));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(half * 2.0, half * 2.0))),
        MeshMaterial3d(ground),
        Transform::default(),
    ));

    let start = map.base_position(Team::Ally);
    let end = map.base_position(Team::Enemy);
    let center = (start + end) * 0.5;
    let direction = map.lane_direction(Team::Ally);
    let lateral = map.lane_lateral(Team::Ally);
    let rotation = Quat::from_rotation_y(-direction.y.atan2(direction.x));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            map.lane_length() + 2.4,
            0.035,
            map.lane_width(),
        ))),
        MeshMaterial3d(road),
        Transform::from_xyz(center.x, 0.02, center.y).with_rotation(rotation),
    ));
    let tile = meshes.add(Cuboid::new(1.05, 0.035, 1.05));
    let steps = (map.lane_length() / 1.3).floor() as i32;
    for step in 1..steps {
        for row in [-1, 0, 1] {
            if (step + row) % 3 != 0 {
                continue;
            }
            let point = start + direction * (step as f32 * 1.3) + lateral * (row as f32 * 1.2);
            commands.spawn((
                Mesh3d(tile.clone()),
                MeshMaterial3d(paving.clone()),
                Transform::from_xyz(point.x, 0.047, point.y).with_rotation(rotation),
            ));
        }
    }

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
