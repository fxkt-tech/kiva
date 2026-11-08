//! Procedural display only. Collision geometry is supplied by the level data.

use bevy::{light::CascadeShadowConfigBuilder, prelude::*};

use crate::{
    gameplay::{Level, Player, Prop, PropKind},
    simulation::Body,
};

use super::controls::ViewYaw;

struct SceneAssets {
    cube: Handle<Mesh>,
    capsule: Handle<Mesh>,
    sphere: Handle<Mesh>,
    floor: Handle<StandardMaterial>,
    grid: Handle<StandardMaterial>,
    slate: Handle<StandardMaterial>,
    dark: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
    copper: Handle<StandardMaterial>,
    teal: Handle<StandardMaterial>,
    accent: Handle<StandardMaterial>,
}

impl SceneAssets {
    fn new(world: &mut World) -> Self {
        let (cube, capsule, sphere) = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            (
                meshes.add(Cuboid::default()),
                meshes.add(Capsule3d::new(0.5, 1.0)),
                meshes.add(Sphere::new(0.5).mesh().uv(16, 12)),
            )
        };
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let mut matte = |color| {
            materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.86,
                ..default()
            })
        };
        Self {
            cube,
            capsule,
            sphere,
            floor: matte(Color::srgb(0.19, 0.24, 0.27)),
            grid: matte(Color::srgb(0.26, 0.32, 0.34)),
            slate: matte(Color::srgb(0.28, 0.37, 0.41)),
            dark: matte(Color::srgb(0.075, 0.11, 0.14)),
            stone: matte(Color::srgb(0.39, 0.45, 0.45)),
            copper: matte(Color::srgb(0.55, 0.30, 0.15)),
            teal: matte(Color::srgb(0.10, 0.56, 0.56)),
            accent: matte(Color::srgb(1.0, 0.64, 0.23)),
        }
    }
}

pub(super) fn setup(world: &mut World) {
    let assets = SceneAssets::new(world);
    let (half_size, props) = {
        let level = world.resource::<Level>();
        (level.half_size, level.props.clone())
    };

    world.insert_resource(ClearColor(Color::srgb(0.065, 0.09, 0.12)));
    world.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.72, 0.84, 1.0),
        brightness: 400.0,
        ..default()
    });
    world.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.89, 0.73),
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-16.0, 28.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            first_cascade_far_bound: 24.0,
            maximum_distance: 65.0,
            ..default()
        }
        .build(),
    ));

    spawn_ground(world, &assets, half_size);
    for prop in &props {
        spawn_prop(world, &assets, prop);
    }
    spawn_player(world, &assets);
}

pub(super) fn sync_player(
    time: Res<Time<Fixed>>,
    view: Res<ViewYaw>,
    mut players: Query<(&Body, &mut Transform), With<Player>>,
) {
    for (body, mut transform) in &mut players {
        let position = body.interpolated(time.overstep_fraction());
        transform.translation = Vec3::new(position.x, 0.0, position.y);
        transform.rotation = Quat::from_rotation_y(view.yaw());
    }
}

fn spawn_ground(world: &mut World, assets: &SceneAssets, half_size: f32) {
    let diameter = half_size * 2.0;
    spawn_box(
        world,
        assets,
        &assets.floor,
        Vec3::new(0.0, -0.15, 0.0),
        Vec3::new(diameter + 1.0, 0.3, diameter + 1.0),
    );

    // Narrow raised seams give movement and rotation a stable scale reference.
    let grid_steps = (half_size / 2.0).floor() as i32;
    for index in -grid_steps..=grid_steps {
        let offset = index as f32 * 2.0;
        spawn_box(
            world,
            assets,
            &assets.grid,
            Vec3::new(offset, 0.003, 0.0),
            Vec3::new(0.035, 0.006, diameter),
        );
        spawn_box(
            world,
            assets,
            &assets.grid,
            Vec3::new(0.0, 0.003, offset),
            Vec3::new(diameter, 0.006, 0.035),
        );
    }

    // The rim's inside faces coincide with CollisionWorld's playable bounds.
    for sign in [-1.0, 1.0] {
        spawn_box(
            world,
            assets,
            &assets.slate,
            Vec3::new(sign * (half_size + 0.25), 0.35, 0.0),
            Vec3::new(0.5, 0.7, diameter + 1.0),
        );
        spawn_box(
            world,
            assets,
            &assets.slate,
            Vec3::new(0.0, 0.35, sign * (half_size + 0.25)),
            Vec3::new(diameter, 0.7, 0.5),
        );
        for offset in [-18.0, -6.0, 6.0, 18.0] {
            spawn_box(
                world,
                assets,
                &assets.accent,
                Vec3::new(offset, 0.71, sign * (half_size + 0.25)),
                Vec3::new(0.7, 0.04, 0.3),
            );
        }
    }

    // A quiet starting pad remains non-solid, with contrasting corners.
    spawn_box(
        world,
        assets,
        &assets.slate,
        Vec3::new(0.0, 0.008, 0.0),
        Vec3::new(3.8, 0.016, 3.8),
    );
    for x in [-1.65, 1.65] {
        for z in [-1.65, 1.65] {
            spawn_box(
                world,
                assets,
                &assets.teal,
                Vec3::new(x, 0.019, z),
                Vec3::new(0.35, 0.02, 0.35),
            );
        }
    }
}

fn spawn_prop(world: &mut World, assets: &SceneAssets, prop: &Prop) {
    let base = Vec3::new(prop.center.x, 0.0, prop.center.y);
    let material = match prop.kind {
        PropKind::Crate => &assets.copper,
        PropKind::Pillar => &assets.slate,
        PropKind::Stone => &assets.stone,
    };
    spawn_box(
        world,
        assets,
        material,
        base + Vec3::Y * prop.size.y * 0.5,
        prop.size,
    );

    match prop.kind {
        PropKind::Crate => {
            // Packing straps stay within the same horizontal collision footprint.
            for sign in [-1.0, 1.0] {
                spawn_box(
                    world,
                    assets,
                    &assets.dark,
                    base + Vec3::new(sign * prop.size.x * 0.30, prop.size.y + 0.015, 0.0),
                    Vec3::new(0.10, 0.03, prop.size.z),
                );
                spawn_box(
                    world,
                    assets,
                    &assets.dark,
                    base + Vec3::new(
                        sign * prop.size.x * 0.30,
                        prop.size.y * 0.5,
                        -prop.size.z * 0.5 + 0.015,
                    ),
                    Vec3::new(0.10, prop.size.y, 0.03),
                );
            }
        }
        PropKind::Pillar => {
            spawn_box(
                world,
                assets,
                &assets.dark,
                base + Vec3::Y * 0.12,
                Vec3::new(prop.size.x, 0.24, prop.size.z),
            );
            spawn_box(
                world,
                assets,
                &assets.teal,
                base + Vec3::Y * (prop.size.y + 0.025),
                Vec3::new(prop.size.x * 0.75, 0.05, prop.size.z * 0.75),
            );
        }
        PropKind::Stone => {
            spawn_box(
                world,
                assets,
                &assets.slate,
                base + Vec3::Y * (prop.size.y + 0.035),
                Vec3::new(prop.size.x * 0.78, 0.07, prop.size.z * 0.72),
            );
        }
    }
}

fn spawn_player(world: &mut World, assets: &SceneAssets) {
    let mut players = world.query_filtered::<(Entity, &Body), With<Player>>();
    let (player, body) = players.single(world).expect("one demo player");
    let position = body.position();
    world.entity_mut(player).insert((
        Transform::from_xyz(position.x, 0.0, position.y),
        Visibility::default(),
    ));

    let mut parts = Vec::new();
    // Unit capsules are two metres tall; the scales below are actual half heights.
    for x in [-0.12, 0.12] {
        parts.push(spawn_mesh(
            world,
            &assets.capsule,
            &assets.dark,
            Transform::from_xyz(x, 0.39, 0.0).with_scale(Vec3::new(0.17, 0.28, 0.17)),
        ));
        parts.push(spawn_box(
            world,
            assets,
            &assets.dark,
            Vec3::new(x, 0.12, -0.045),
            Vec3::new(0.17, 0.20, 0.32),
        ));
    }
    parts.push(spawn_mesh(
        world,
        &assets.capsule,
        &assets.teal,
        Transform::from_xyz(0.0, 1.0, 0.0).with_scale(Vec3::new(0.44, 0.41, 0.33)),
    ));
    parts.push(spawn_box(
        world,
        assets,
        &assets.dark,
        Vec3::new(0.0, 0.72, 0.0),
        Vec3::new(0.42, 0.11, 0.30),
    ));
    for x in [-0.265, 0.265] {
        parts.push(spawn_mesh(
            world,
            &assets.capsule,
            &assets.teal,
            Transform::from_xyz(x, 1.06, -0.035).with_scale(Vec3::new(0.15, 0.27, 0.15)),
        ));
    }
    parts.push(spawn_mesh(
        world,
        &assets.sphere,
        &assets.slate,
        Transform::from_xyz(0.0, 1.57, 0.0).with_scale(Vec3::new(0.38, 0.40, 0.38)),
    ));
    parts.push(spawn_box(
        world,
        assets,
        &assets.accent,
        Vec3::new(0.0, 1.59, -0.17),
        Vec3::new(0.27, 0.09, 0.05),
    ));
    parts.push(spawn_box(
        world,
        assets,
        &assets.dark,
        Vec3::new(0.21, 1.01, -0.17),
        Vec3::new(0.08, 0.12, 0.15),
    ));
    parts.push(spawn_box(
        world,
        assets,
        &assets.accent,
        Vec3::new(0.21, 1.01, -0.245),
        Vec3::new(0.05, 0.07, 0.02),
    ));
    world.entity_mut(player).add_children(&parts);
}

fn spawn_box(
    world: &mut World,
    assets: &SceneAssets,
    material: &Handle<StandardMaterial>,
    center: Vec3,
    size: Vec3,
) -> Entity {
    spawn_mesh(
        world,
        &assets.cube,
        material,
        Transform::from_translation(center).with_scale(size),
    )
}

fn spawn_mesh(
    world: &mut World,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    world
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
        ))
        .id()
}
