//! Deterministic woodland presentation of the gameplay-owned training arena.
use bevy::{
    asset::RenderAssetUsages,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    light::CascadeShadowConfigBuilder,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::gameplay::{
    Bamboo, Boundary, Level, TerrainGeometry, path_distance, terrain, variation,
};

use super::forest_material::{ForestMaterial, ForestSurface};
use crate::simulation::SimulationActive;

#[derive(Resource, Default)]
pub(super) struct WindClock(f32);

#[derive(Component)]
pub(super) struct WindDecoration {
    center: Vec3,
    phase: f32,
}

#[derive(Resource)]
struct ForestLighting(EnvironmentMapLight);

fn texture(world: &mut World, bytes: &'static [u8], srgb: bool, repeat: bool) -> Handle<Image> {
    let mut sampler = ImageSamplerDescriptor::linear();
    sampler.set_anisotropic_filter(4);
    if repeat {
        sampler.set_address_mode(ImageAddressMode::Repeat);
    }
    let image = Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        srgb,
        ImageSampler::Descriptor(sampler),
        RenderAssetUsages::default(),
    )
    .expect("verified embedded forest texture");
    world.resource_mut::<Assets<Image>>().add(image)
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EnvironmentPart {
    Ground,
    Ledge,
    Trunks,
    Rocks,
    Perimeter,
    Nodes,
    Leaves,
    Litter,
}

pub(super) fn fog() -> DistanceFog {
    DistanceFog {
        color: Color::srgb(0.22, 0.31, 0.32),
        directional_light_color: Color::srgba(0.75, 0.80, 0.72, 0.12),
        directional_light_exponent: 18.0,
        falloff: FogFalloff::from_visibility_squared(42.0),
    }
}

/// Called after camera creation, including in headless scene fixtures.
pub(super) fn setup_camera_fog(world: &mut World) {
    let cameras: Vec<_> = world
        .query_filtered::<Entity, With<Camera3d>>()
        .iter(world)
        .collect();
    let lighting = world
        .get_resource::<ForestLighting>()
        .map(|lighting| lighting.0.clone());
    for camera in cameras {
        world.entity_mut(camera).insert(fog());
        if let Some(lighting) = &lighting {
            world.entity_mut(camera).insert(lighting.clone());
        }
    }
}

pub(super) fn setup(world: &mut World) {
    let (ground, ramp, trunks, rocks, bamboo, boundaries) = {
        let level = world.resource::<Level>();
        (
            level.ground.clone(),
            level.ramp.clone(),
            level.forest_trunks.clone(),
            level.forest_rocks.clone(),
            level.bamboo.clone(),
            level.boundaries(),
        )
    };
    world.insert_resource(ClearColor(Color::srgb(0.13, 0.20, 0.21)));
    world.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.70, 0.82, 0.88),
        brightness: 240.0,
        ..default()
    });
    world.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.96, 0.87),
            illuminance: 6_500.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-14.0, 28.0, 8.0).looking_at(Vec3::new(0.0, 0.0, -4.0), Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            first_cascade_far_bound: 18.0,
            maximum_distance: 55.0,
            ..default()
        }
        .build(),
    ));
    world.spawn((
        DirectionalLight {
            color: Color::srgb(0.65, 0.79, 0.91),
            illuminance: 950.0,
            ..default()
        },
        Transform::from_xyz(8.0, 6.0, -12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<ForestMaterial>>();
    world.init_resource::<WindClock>();
    let lighting = EnvironmentMapLight::hemispherical_gradient(
        &mut world.resource_mut::<Assets<Image>>(),
        Color::srgb(0.52, 0.66, 0.72),
        Color::srgb(0.25, 0.34, 0.28),
        Color::srgb(0.15, 0.17, 0.10),
    )
    .with_intensity(310.0);
    world.insert_resource(ForestLighting(lighting));
    // The adjacent manifest retains Model -> material -> named texture refs.
    // Source parameter blocks were unavailable; response scalars are authored.
    let leaves_base = texture(
        world,
        include_bytes!("../../assets/choulingke/environment/leaves.png"),
        true,
        false,
    );
    let leaves_normal = texture(
        world,
        include_bytes!("../../assets/choulingke/environment/leaves_normal.png"),
        false,
        false,
    );
    let bamboo_base = texture(
        world,
        include_bytes!("../../assets/choulingke/environment/bamboo.png"),
        true,
        true,
    );
    let bamboo_normal = texture(
        world,
        include_bytes!("../../assets/choulingke/environment/bamboo_normal.png"),
        false,
        true,
    );
    let litter_base = texture(
        world,
        include_bytes!("../../assets/choulingke/environment/litter.png"),
        true,
        false,
    );
    let litter_normal = texture(
        world,
        include_bytes!("../../assets/choulingke/environment/litter_normal.png"),
        false,
        false,
    );
    let (soil, bamboo_material, stone, foliage, litter_material) = {
        let mut materials = world.resource_mut::<Assets<ForestMaterial>>();
        let mut surface = |kind, roughness, thin| {
            let maps = if kind == 1.0 {
                Some((&bamboo_base, &bamboo_normal))
            } else if kind == 3.0 {
                Some((&leaves_base, &leaves_normal))
            } else if kind == 4.0 {
                Some((&litter_base, &litter_normal))
            } else {
                None
            };
            let cutout = kind == 3.0 || kind == 4.0;
            materials.add(ForestMaterial {
                base: StandardMaterial {
                    base_color: Color::WHITE,
                    base_color_texture: maps.map(|(base, _)| base.clone()),
                    normal_map_texture: maps.map(|(_, normal)| normal.clone()),
                    alpha_mode: if cutout {
                        AlphaMode::Mask(0.28)
                    } else {
                        AlphaMode::Opaque
                    },
                    perceptual_roughness: roughness,
                    clearcoat: if kind == 1.0 { 0.12 } else { 0.0 },
                    clearcoat_perceptual_roughness: 0.3,
                    diffuse_transmission: thin,
                    double_sided: cutout || thin > 0.0,
                    cull_mode: if cutout || thin > 0.0 {
                        None
                    } else {
                        Some(bevy::render::render_resource::Face::Back)
                    },
                    reflectance: 0.35,
                    ..default()
                },
                extension: ForestSurface {
                    style: Vec4::new(if kind == 4.0 { 0.0 } else { kind }, 1.0, 1.0, 0.0),
                },
            })
        };
        (
            surface(0.0, 0.93, 0.0),
            surface(1.0, 0.6, 0.0),
            surface(2.0, 0.87, 0.0),
            surface(3.0, 0.64, 0.28),
            surface(4.0, 0.92, 0.0),
        )
    };
    let ground_colors = ground.vertices.iter().map(|v| ground_color(*v)).collect();
    spawn_mesh(
        world,
        &ground,
        ground_colors,
        &soil,
        EnvironmentPart::Ground,
    );
    let ramp_colors = ramp.vertices.iter().map(|v| ground_color(*v)).collect();
    spawn_mesh(world, &ramp, ramp_colors, &soil, EnvironmentPart::Ledge);
    let trunk_colors = trunks
        .vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            let stem = &bamboo[index / 42];
            let height = ((vertex.y - stem.base.y) / stem.height).clamp(0.0, 1.0);
            let shade = 0.85 + variation((index / 42) as u32, 21) * 0.3;
            [
                (0.82 + height * 0.12) * shade,
                (0.88 + height * 0.10) * shade,
                (0.78 + height * 0.10) * shade,
                1.0,
            ]
        })
        .collect();
    spawn_mesh(
        world,
        &trunks,
        trunk_colors,
        &bamboo_material,
        EnvironmentPart::Trunks,
    );
    let rock_colors = rocks
        .vertices
        .iter()
        .map(|vertex| {
            let moss = ((1.2 - vertex.y) * 0.04).max(0.0);
            [0.18, 0.19 + moss, 0.155, 1.0]
        })
        .collect();
    spawn_mesh(world, &rocks, rock_colors, &stone, EnvironmentPart::Rocks);
    let perimeter = perimeter_geometry(boundaries);
    let perimeter_colors = perimeter
        .vertices
        .iter()
        .map(|vertex| {
            if vertex.y > 0.6 {
                [0.13, 0.19, 0.09, 1.0]
            } else {
                [0.18, 0.19, 0.16, 1.0]
            }
        })
        .collect();
    spawn_mesh(
        world,
        &perimeter,
        perimeter_colors,
        &stone,
        EnvironmentPart::Perimeter,
    );
    let mut nodes = Decoration::default();
    let mut leaves: [Decoration; 4] = std::array::from_fn(|_| Decoration::default());
    for (index, stem) in bamboo.iter().enumerate() {
        decorate_bamboo(index as u32, stem, &mut nodes, &mut leaves[index % 4]);
    }
    spawn_decoration(world, nodes, &bamboo_material, EnvironmentPart::Nodes);
    for (index, mut leaves) in leaves.into_iter().enumerate() {
        let center = leaves.geometry.vertices.iter().copied().sum::<Vec3>()
            / leaves.geometry.vertices.len() as f32;
        for vertex in &mut leaves.geometry.vertices {
            *vertex -= center;
        }
        let entity = spawn_decoration(world, leaves, &foliage, EnvironmentPart::Leaves);
        world.entity_mut(entity).insert((
            Transform::from_translation(center),
            WindDecoration {
                center,
                phase: index as f32 * 1.7,
            },
        ));
    }
    let mut litter = Decoration::default();
    for index in 0..260 {
        let point = Vec2::new(variation(index, 60) - 0.5, variation(index, 61) - 0.5) * 49.0;
        let center = Vec3::new(point.x, terrain::hill_height(point) + 0.014, point.y);
        let yaw = variation(index, 62) * std::f32::consts::TAU;
        let direction = Vec3::new(yaw.cos(), 0.0, yaw.sin()) * (0.3 + variation(index, 63) * 0.35);
        let side = Vec3::new(-direction.z, 0.0, direction.x);
        let color = [0.87 + variation(index, 64) * 0.12, 0.95, 0.86, 1.0];
        litter.quad(
            [
                center - direction - side,
                center - direction + side,
                center + direction + side,
                center + direction - side,
            ],
            color,
        );
    }
    spawn_decoration(world, litter, &litter_material, EnvironmentPart::Litter);
}

/// Decorative leaf cohorts sway by a few centimetres. Their actual Transform
/// drives the standard shadow pass too; collidable culms and authority stay fixed.
pub(super) fn animate_foliage(
    time: Res<Time>,
    active: Res<SimulationActive>,
    mut clock: ResMut<WindClock>,
    mut leaves: Query<(&WindDecoration, &mut Transform)>,
) {
    if !active.0 {
        return;
    }
    clock.0 += time.delta_secs();
    for (leaf, mut transform) in &mut leaves {
        let phase = clock.0 * 0.8 + leaf.phase;
        transform.translation = leaf.center
            + Vec3::new(
                phase.sin() * 0.035,
                (phase * 1.3).sin() * 0.008,
                (phase * 0.73).cos() * 0.025,
            );
        transform.rotation = Quat::from_rotation_z(phase.sin() * 0.0015);
    }
}

fn perimeter_geometry(boundaries: [Boundary; 4]) -> TerrainGeometry {
    let mut geometry = TerrainGeometry::default();
    for boundary in boundaries {
        let start = geometry.vertices.len() as u32;
        let half = boundary.size * 0.5;
        geometry.vertices.extend([
            boundary.center + Vec3::new(-half.x, -half.y, -half.z),
            boundary.center + Vec3::new(half.x, -half.y, -half.z),
            boundary.center + Vec3::new(half.x, -half.y, half.z),
            boundary.center + Vec3::new(-half.x, -half.y, half.z),
            boundary.center + Vec3::new(-half.x, half.y, -half.z),
            boundary.center + Vec3::new(half.x, half.y, -half.z),
            boundary.center + Vec3::new(half.x, half.y, half.z),
            boundary.center + Vec3::new(-half.x, half.y, half.z),
        ]);
        geometry.triangles.extend(
            [
                [0, 1, 2],
                [0, 2, 3],
                [4, 6, 5],
                [4, 7, 6],
                [0, 4, 5],
                [0, 5, 1],
                [1, 5, 6],
                [1, 6, 2],
                [2, 6, 7],
                [2, 7, 3],
                [3, 7, 4],
                [3, 4, 0],
            ]
            .map(|triangle| triangle.map(|index| start + index)),
        );
    }
    geometry
}

fn ground_color(vertex: Vec3) -> [f32; 4] {
    let point = vertex.xz();
    let trail = (1.0 - (path_distance(point) - 0.8) / 1.5).clamp(0.0, 1.0);
    let clearing_radius = ((point.x / 8.5).powi(2) + ((point.y + 4.0) / 9.5).powi(2)).sqrt();
    let clearing = ((1.0 - clearing_radius) * 2.0).clamp(0.0, 0.65);
    let worn = trail.max(clearing);
    let grain = (point.x * 2.7 + point.y * 1.3).sin() * (point.y * 3.1).cos() * 0.012;
    [
        0.075 + worn * 0.085 + grain,
        0.105 + worn * 0.035 + grain,
        0.065 + worn * 0.025 + grain * 0.5,
        1.0,
    ]
}

#[derive(Default)]
struct Decoration {
    geometry: TerrainGeometry,
    colors: Vec<[f32; 4]>,
}

impl Decoration {
    fn quad(&mut self, vertices: [Vec3; 4], color: [f32; 4]) {
        let start = self.geometry.vertices.len() as u32;
        self.geometry.vertices.extend(vertices);
        self.geometry
            .triangles
            .extend([[start, start + 1, start + 2], [start, start + 2, start + 3]]);
        self.colors.extend([color; 4]);
    }
}

fn decorate_bamboo(index: u32, stem: &Bamboo, nodes: &mut Decoration, leaves: &mut Decoration) {
    let joints = (stem.height / 0.9).floor() as u32;
    for joint in 1..joints {
        let fraction = joint as f32 / joints as f32;
        let center = stem.center_at(fraction);
        let radius = stem.radius_at(fraction) + 0.009;
        for side in 0..8 {
            let a = side as f32 * std::f32::consts::TAU / 8.0;
            let b = (side + 1) as f32 * std::f32::consts::TAU / 8.0;
            let radial_a = Vec3::new(a.cos(), 0.0, a.sin()) * radius;
            let radial_b = Vec3::new(b.cos(), 0.0, b.sin()) * radius;
            nodes.quad(
                [
                    center + radial_a,
                    center + radial_a + Vec3::Y * 0.025,
                    center + radial_b + Vec3::Y * 0.025,
                    center + radial_b,
                ],
                [0.23, 0.31, 0.12, 1.0],
            );
        }
    }
    for branch in 0..6 {
        let fraction = 0.48 + (branch / 2) as f32 * 0.18;
        let angle = variation(index, 30 + branch) * std::f32::consts::TAU;
        let direction = Vec3::new(angle.cos(), 0.22, angle.sin()).normalize();
        let across = Vec3::new(-angle.sin(), 0.0, angle.cos());
        let start = stem.center_at(fraction);
        // The verified alpha atlas contains a whole branch cluster. A broad
        // card preserves its own stems/leaves; one tiny card per leaf would
        // shrink the atlas detail away and destroy coverage.
        let length = 1.8 + variation(index, 40 + branch) * 1.25;
        let end = start + direction * length;
        let width = across * length * 0.48;
        let shade = 0.85 + variation(index, 70 + branch) * 0.15;
        leaves.quad(
            [start - width, start + width, end + width, end - width],
            [shade * 0.9, shade, shade * 0.86, 1.0],
        );
    }
}

fn spawn_decoration(
    world: &mut World,
    decoration: Decoration,
    material: &Handle<ForestMaterial>,
    part: EnvironmentPart,
) -> Entity {
    spawn_mesh(
        world,
        &decoration.geometry,
        decoration.colors,
        material,
        part,
    )
}

fn spawn_mesh(
    world: &mut World,
    geometry: &TerrainGeometry,
    colors: Vec<[f32; 4]>,
    material: &Handle<ForestMaterial>,
    part: EnvironmentPart,
) -> Entity {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        geometry
            .vertices
            .iter()
            .map(|v| v.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    let card_uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    let uvs: Vec<[f32; 2]> = geometry
        .vertices
        .iter()
        .enumerate()
        .map(|(index, point)| match part {
            EnvironmentPart::Leaves | EnvironmentPart::Litter => card_uvs[index % 4],
            EnvironmentPart::Trunks => [((index % 42) % 8) as f32 / 8.0, -point.y / 3.6],
            _ => [point.x * 0.25, point.z * 0.25],
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(
        geometry.triangles.iter().flatten().copied().collect(),
    ));
    mesh.compute_smooth_normals();
    mesh.generate_tangents()
        .expect("forest texture coordinates and normals");
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
    world
        .spawn((
            Name::new(format!("Bamboo forest {part:?}")),
            part,
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            Transform::default(),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn forest_presentation_reuses_exact_solid_geometry_and_a_small_material_set() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        crate::gameplay::setup(&mut world);
        let expected = world.resource::<Level>().forest_trunks.vertices.clone();
        setup(&mut world);
        let mut query = world.query::<(&EnvironmentPart, &Mesh3d)>();
        let parts: Vec<_> = query
            .iter(&world)
            .map(|(part, mesh)| (*part, mesh.0.clone()))
            .collect();
        assert_eq!(parts.len(), 11);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), 11);
        assert_eq!(world.resource::<Assets<ForestMaterial>>().len(), 5);
        let materials = world.resource::<Assets<ForestMaterial>>();
        let images = world.resource::<Assets<Image>>();
        let textured: Vec<_> = materials
            .iter()
            .filter(|(_, material)| material.base.base_color_texture.is_some())
            .collect();
        assert_eq!(textured.len(), 3);
        for (_, material) in textured {
            let base = images
                .get(material.base.base_color_texture.as_ref().unwrap())
                .unwrap();
            let normal = images
                .get(material.base.normal_map_texture.as_ref().unwrap())
                .unwrap();
            assert!(base.texture_descriptor.format.is_srgb());
            assert!(!normal.texture_descriptor.format.is_srgb());
        }
        let trunks = parts
            .iter()
            .find(|(part, _)| *part == EnvironmentPart::Trunks)
            .unwrap();
        let mesh = world.resource::<Assets<Mesh>>().get(&trunks.1).unwrap();
        let VertexAttributeValues::Float32x3(vertices) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!("trunks must expose their source positions");
        };
        assert_eq!(
            vertices,
            &expected.iter().map(|v| v.to_array()).collect::<Vec<_>>()
        );
        for (_, mesh) in parts {
            let mesh = world.resource::<Assets<Mesh>>().get(&mesh).unwrap();
            assert!(mesh.count_vertices() > 0);
            let VertexAttributeValues::Float32x3(normals) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap()
            else {
                panic!("forest mesh must have normals");
            };
            assert!(
                normals
                    .iter()
                    .all(|normal| Vec3::from_array(*normal).is_finite())
            );
            let VertexAttributeValues::Float32x4(tangents) =
                mesh.attribute(Mesh::ATTRIBUTE_TANGENT).unwrap()
            else {
                panic!("textured forest tangents")
            };
            assert!(
                tangents
                    .iter()
                    .all(|tangent| Vec4::from_array(*tangent).is_finite())
            );
            assert_eq!(
                mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap().len(),
                mesh.count_vertices()
            );
        }
    }

    #[test]
    fn camera_fog_is_available_without_a_renderer_or_asset_server() {
        let mut world = World::new();
        let camera = world.spawn(Camera3d::default()).id();
        setup_camera_fog(&mut world);
        let fog = world.get::<DistanceFog>(camera).unwrap();
        assert!(matches!(fog.falloff, FogFalloff::ExponentialSquared { .. }));
    }

    #[test]
    fn only_decorative_leaves_sway_and_pause_preserves_the_exact_pose() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Time>()
            .insert_resource(SimulationActive(true))
            .add_systems(Update, animate_foliage);
        crate::gameplay::setup(app.world_mut());
        setup(app.world_mut());
        let stems: Vec<_> = app
            .world_mut()
            .query::<(Entity, &EnvironmentPart, &Transform)>()
            .iter(app.world())
            .filter(|(_, part, _)| **part == EnvironmentPart::Trunks)
            .map(|(entity, _, transform)| (entity, *transform))
            .collect();
        let before: Vec<_> = app
            .world_mut()
            .query_filtered::<(Entity, &Transform), With<WindDecoration>>()
            .iter(app.world())
            .map(|(entity, transform)| (entity, *transform))
            .collect();
        assert_eq!(before.len(), 4);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(0.3));
        app.world_mut().run_schedule(Update);
        assert!(before.iter().any(|(entity, transform)| {
            app.world().get::<Transform>(*entity).unwrap() != transform
        }));
        for (entity, transform) in stems {
            assert_eq!(*app.world().get::<Transform>(entity).unwrap(), transform);
        }
        let posed: Vec<_> = before
            .iter()
            .map(|(entity, _)| (*entity, *app.world().get::<Transform>(*entity).unwrap()))
            .collect();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(10.0));
        app.world_mut().run_schedule(Update);
        for (entity, transform) in posed {
            assert_eq!(*app.world().get::<Transform>(entity).unwrap(), transform);
        }
    }
}
