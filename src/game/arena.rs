use bevy::prelude::*;

pub(super) fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stone = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(69, 77, 78),
        perceptual_roughness: 0.95,
        ..default()
    });
    let border = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(36, 43, 46),
        ..default()
    });
    let line = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(108, 106, 86),
        perceptual_roughness: 1.0,
        ..default()
    });
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands.spawn((
        Name::new("Training courtyard"),
        Mesh3d(cube.clone()),
        MeshMaterial3d(stone),
        Transform::from_xyz(0.0, -0.14, 0.0).with_scale(Vec3::new(16.0, 0.28, 16.0)),
    ));
    for i in -8..=8 {
        let v = i as f32;
        for (position, scale) in [
            (Vec3::new(v, 0.005, 0.0), Vec3::new(0.014, 0.006, 16.0)),
            (Vec3::new(0.0, 0.005, v), Vec3::new(16.0, 0.006, 0.014)),
        ] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(line.clone()),
                Transform::from_translation(position).with_scale(scale),
            ));
        }
    }
    for (x, z) in [(-7.8, -7.8), (7.8, -7.8), (-7.8, 7.8), (7.8, 7.8)] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(border.clone()),
            Transform::from_xyz(x, 0.65, z).with_scale(Vec3::new(0.5, 1.3, 0.5)),
        ));
    }
    for (position, scale) in [
        (Vec3::new(0.0, 0.12, -8.0), Vec3::new(16.0, 0.24, 0.3)),
        (Vec3::new(0.0, 0.12, 8.0), Vec3::new(16.0, 0.24, 0.3)),
        (Vec3::new(-8.0, 0.12, 0.0), Vec3::new(0.3, 0.24, 16.0)),
        (Vec3::new(8.0, 0.12, 0.0), Vec3::new(0.3, 0.24, 16.0)),
    ] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(border.clone()),
            Transform::from_translation(position).with_scale(scale),
        ));
    }
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb_u8(185, 207, 224),
        brightness: 180.0,
        ..default()
    });
    commands.spawn((
        Name::new("Afternoon sun"),
        DirectionalLight {
            illuminance: 9000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
