//! Procedural display only. Collision geometry is supplied by the level data.

use bevy::{
    asset::RenderAssetUsages,
    light::CascadeShadowConfigBuilder,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    gameplay::{Boundary, Level, Player, Prop, PropKind, TerrainGeometry},
    simulation::{Body, PhysicsPose, SimulationActive},
};

use super::{ActorView, PlayerView, PropView, controls::ViewYaw};

struct SceneAssets {
    cube: Handle<Mesh>,
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
        let cube = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Cuboid::default())
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
    let (half_size, ground, ramp) = {
        let level = world.resource::<Level>();
        (level.half_size, level.ground.clone(), level.ramp.clone())
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
    spawn_terrain(world, &assets, &ground, true);
    spawn_terrain(world, &assets, &ramp, false);
    let props: Vec<_> = world
        .query::<(Entity, &Prop, &PhysicsPose)>()
        .iter(world)
        .map(|(entity, prop, pose)| (entity, *prop, pose.position()))
        .collect();
    for (owner, prop, position) in props {
        spawn_prop(world, &assets, owner, &prop, position);
    }
    let boundaries: Vec<_> = world
        .query::<(Entity, &Boundary)>()
        .iter(world)
        .map(|(entity, boundary)| (entity, *boundary))
        .collect();
    for (owner, boundary) in boundaries {
        let root = world
            .spawn((
                PropView { owner },
                Transform::from_translation(boundary.center),
                Visibility::default(),
            ))
            .id();
        let body = spawn_box(world, &assets, &assets.slate, Vec3::ZERO, boundary.size);
        world.entity_mut(root).add_child(body);
    }
    spawn_player(world, &assets);
}

pub(super) fn sync_player(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    view: Res<ViewYaw>,
    players: Query<&Body, With<Player>>,
    mut views: Query<(&ActorView, &mut Transform), With<PlayerView>>,
) {
    let alpha = super::display_alpha(&time, &active);
    for (view_root, mut transform) in &mut views {
        let Ok(body) = players.get(view_root.owner) else {
            continue;
        };
        transform.translation = body.interpolated_world(alpha);
        transform.rotation = Quat::from_rotation_y(view.yaw());
    }
}

pub(super) fn sync_props(
    time: Res<Time<Fixed>>,
    active: Res<SimulationActive>,
    props: Query<&PhysicsPose>,
    mut views: Query<(&PropView, &mut Transform)>,
) {
    let alpha = super::display_alpha(&time, &active);
    for (view, mut transform) in &mut views {
        if let Ok(pose) = props.get(view.owner) {
            transform.translation = pose.interpolated_position(alpha);
            transform.rotation = pose.interpolated_rotation(alpha);
        }
    }
}

fn spawn_terrain(
    world: &mut World,
    assets: &SceneAssets,
    geometry: &TerrainGeometry,
    ground: bool,
) {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        geometry
            .vertices
            .iter()
            .map(|vertex| vertex.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_indices(Indices::U32(
        geometry.triangles.iter().flatten().copied().collect(),
    ));
    mesh.compute_smooth_normals();
    if ground {
        let colors: Vec<[f32; 4]> = geometry
            .vertices
            .iter()
            .map(|vertex| {
                let hill = (vertex.y / 2.6).clamp(0.0, 1.0);
                let contour = if vertex.y > 0.08 && (vertex.y * 2.0).fract() < 0.08 {
                    0.04
                } else {
                    0.0
                };
                [
                    0.19 + hill * 0.13 + contour,
                    0.24 + hill * 0.18 + contour,
                    0.27 + hill * 0.10 + contour,
                    1.0,
                ]
            })
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
    let material = if ground {
        world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.86,
                ..default()
            })
    } else {
        assets.slate.clone()
    };
    spawn_mesh(world, &mesh, &material, Transform::default());
}

fn spawn_ground(world: &mut World, assets: &SceneAssets, half_size: f32) {
    let diameter = half_size * 2.0;
    spawn_box(
        world,
        assets,
        &assets.floor,
        Vec3::new(0.0, -0.16, 0.0),
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

    // Painted caps remain decoration; the physical rim uses shared Boundary data.
    for sign in [-1.0, 1.0] {
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
    // Flush route marks guide players to the two terrain test areas.
    for x in [-2.5, -3.5, -4.5] {
        spawn_box(
            world,
            assets,
            &assets.teal,
            Vec3::new(x, 0.012, 0.0),
            Vec3::new(0.42, 0.006, 0.12),
        );
    }
    for x in [2.5, 4.5, 6.5, 8.5, 10.5] {
        spawn_box(
            world,
            assets,
            &assets.accent,
            Vec3::new(x, 0.012, 0.0),
            Vec3::new(0.42, 0.006, 0.12),
        );
    }
    for z in [1.5, 3.5, 5.5, 7.5] {
        spawn_box(
            world,
            assets,
            &assets.accent,
            Vec3::new(11.0, 0.012, z),
            Vec3::new(0.12, 0.006, 0.42),
        );
    }
}

fn spawn_prop(world: &mut World, assets: &SceneAssets, owner: Entity, prop: &Prop, position: Vec3) {
    let root = world
        .spawn((
            PropView { owner },
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .id();
    let base = Vec3::NEG_Y * prop.size.y * 0.5;
    let material = match prop.kind {
        PropKind::Crate => &assets.copper,
        PropKind::Pillar => &assets.slate,
        PropKind::Stone => &assets.stone,
    };
    let body = spawn_box(
        world,
        assets,
        material,
        base + Vec3::Y * prop.size.y * 0.5,
        prop.size,
    );
    world.entity_mut(root).add_child(body);

    match prop.kind {
        PropKind::Crate => {
            // Packing straps stay within the same horizontal collision footprint.
            for sign in [-1.0, 1.0] {
                let strap = spawn_box(
                    world,
                    assets,
                    &assets.dark,
                    base + Vec3::new(sign * prop.size.x * 0.30, prop.size.y + 0.015, 0.0),
                    Vec3::new(0.10, 0.03, prop.size.z),
                );
                world.entity_mut(root).add_child(strap);
                let strap = spawn_box(
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
                world.entity_mut(root).add_child(strap);
            }
        }
        PropKind::Pillar => {
            let foot = spawn_box(
                world,
                assets,
                &assets.dark,
                base + Vec3::Y * 0.12,
                Vec3::new(prop.size.x, 0.24, prop.size.z),
            );
            world.entity_mut(root).add_child(foot);
            let cap = spawn_box(
                world,
                assets,
                &assets.teal,
                base + Vec3::Y * (prop.size.y + 0.025),
                Vec3::new(prop.size.x * 0.75, 0.05, prop.size.z * 0.75),
            );
            world.entity_mut(root).add_child(cap);
        }
        PropKind::Stone => {
            let cap = spawn_box(
                world,
                assets,
                &assets.slate,
                base + Vec3::Y * (prop.size.y + 0.035),
                Vec3::new(prop.size.x * 0.78, 0.07, prop.size.z * 0.72),
            );
            world.entity_mut(root).add_child(cap);
        }
    }
}

fn spawn_player(world: &mut World, assets: &SceneAssets) {
    let mut players = world.query_filtered::<(Entity, &Body), With<Player>>();
    let (player, body) = players.single(world).expect("one demo player");
    let position = body.world_position();
    let view = world
        .spawn((
            ActorView { owner: player },
            PlayerView,
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .id();
    super::stickman::spawn(world, view, &assets.teal, &assets.accent);
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
