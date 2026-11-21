//! Deterministic woodland presentation of the gameplay-owned training arena.
use bevy::{
    asset::RenderAssetUsages,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    light::CascadeShadowConfigBuilder,
    material::OpaqueRendererMethod,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::gameplay::{Bamboo, Boundary, Level, TerrainGeometry, path_distance, variation};

use super::forest_material::{ForestMaterial, ForestSurface};
use super::render_style::RenderStyle;
use crate::simulation::SimulationActive;

mod ground_cover;

#[derive(Resource, Default)]
pub(super) struct WindClock(f32);

#[derive(Component)]
pub(super) struct WindDecoration {
    center: Vec3,
    phase: f32,
    strength: f32,
}

#[derive(Resource)]
struct ForestLighting(EnvironmentMapLight);

/// Every file read by [`texture`] here and in the ground cover.
pub(super) const TEXTURES: [&str; 14] = [
    "environment/bamboo/leaves.png",
    "environment/bamboo/leaves_normal.png",
    "environment/bamboo/bamboo.png",
    "environment/bamboo/bamboo_normal.png",
    "environment/bamboo/litter.png",
    "environment/bamboo/litter_normal.png",
    "environment/bamboo/ground_base.png",
    "environment/bamboo/ground_normal.png",
    "environment/bamboo/forest_floor_base.png",
    "environment/bamboo/forest_floor_normal.png",
    "environment/bamboo/grass_base.png",
    "environment/bamboo/grass_normal.png",
    "environment/bamboo/grass_blades_base.png",
    "environment/bamboo/grass_blades_normal.png",
];

fn texture(world: &mut World, path: &'static str, srgb: bool, repeat: bool) -> Handle<Image> {
    debug_assert!(TEXTURES.contains(&path), "{path} is not in the manifest");
    let bytes = &super::bundle::take(world, path);
    let mut sampler = ImageSamplerDescriptor::linear();
    sampler.set_anisotropic_filter(4);
    if repeat {
        sampler.set_address_mode(ImageAddressMode::Repeat);
    }
    let mut image = Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        srgb,
        ImageSampler::Descriptor(sampler),
        RenderAssetUsages::default(),
    )
    .expect("verified forest texture");
    if repeat {
        super::texture_mips::generate(&mut image, !srgb);
    }
    world.resource_mut::<Assets<Image>>().add(image)
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EnvironmentPart {
    Ground,
    Trunks,
    Rocks,
    Perimeter,
    Nodes,
    Leaves,
    Litter,
    Grass,
    FineGrass,
    Pebbles,
}

/// The two bamboo-clearing directional lights, retuned in place on a switch.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SceneLight {
    Key,
    Fill,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LightSpec {
    pub color: Color,
    pub illuminance: f32,
}

/// Everything about the clearing's light and air that a render style owns.
/// Light poses, shadow cascades and the environment-map image stay shared.
#[derive(Clone, Debug)]
pub(super) struct SceneLighting {
    pub clear: Color,
    pub ambient: Color,
    pub ambient_brightness: f32,
    pub key: LightSpec,
    pub fill: LightSpec,
    pub environment_intensity: f32,
    pub fog: DistanceFog,
}

pub(super) fn bamboo_lighting(style: RenderStyle) -> SceneLighting {
    match style {
        // The shipped look; switching back must reproduce it exactly.
        RenderStyle::Pbr => SceneLighting {
            clear: Color::srgb(0.12, 0.19, 0.26),
            ambient: Color::srgb(0.63, 0.77, 0.90),
            ambient_brightness: 200.0,
            key: LightSpec {
                color: Color::srgb(1.0, 0.97, 0.91),
                illuminance: 4_800.0,
            },
            fill: LightSpec {
                color: Color::srgb(0.57, 0.74, 0.94),
                illuminance: 850.0,
            },
            environment_intensity: 310.0,
            fog: DistanceFog {
                color: Color::srgb(0.19, 0.27, 0.34),
                directional_light_color: Color::srgba(0.83, 0.82, 0.72, 0.10),
                directional_light_exponent: 18.0,
                falloff: FogFalloff::from_visibility_squared(44.0),
            },
        },
        // Initial yancai values, to be tuned against screenshots: paper-warm
        // key, azurite fill, little sky gradient, distance fades to paper.
        RenderStyle::Yancai => SceneLighting {
            clear: Color::srgb(0.80, 0.78, 0.70),
            ambient: Color::srgb(0.62, 0.70, 0.74),
            ambient_brightness: 260.0,
            key: LightSpec {
                color: Color::srgb(1.0, 0.94, 0.82),
                illuminance: 4_800.0,
            },
            fill: LightSpec {
                color: Color::srgb(0.42, 0.58, 0.70),
                illuminance: 600.0,
            },
            environment_intensity: 90.0,
            fog: DistanceFog {
                color: Color::srgb(0.78, 0.77, 0.70),
                directional_light_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                directional_light_exponent: 18.0,
                falloff: FogFalloff::from_visibility_squared(72.0),
            },
        },
    }
}

/// The forest material's share of a style: forward light bands are measured
/// against the key light so they agree with the deferred post pass.
pub(super) fn forest_style(style: RenderStyle) -> Vec4 {
    match style {
        RenderStyle::Pbr => Vec4::ZERO,
        RenderStyle::Yancai => Vec4::new(
            1.0,
            bamboo_lighting(style).key.illuminance / std::f32::consts::PI,
            3.0,
            0.0,
        ),
    }
}

fn style_forest_materials(materials: &mut Assets<ForestMaterial>, style: RenderStyle) {
    let wanted = forest_style(style);
    let stale: Vec<_> = materials
        .iter()
        .filter(|(_, material)| material.extension.yancai != wanted)
        .map(|(id, _)| id)
        .collect();
    for id in stale {
        if let Some(mut material) = materials.get_mut(id) {
            material.extension.yancai = wanted;
        }
    }
}

fn current_lighting(world: &World) -> SceneLighting {
    bamboo_lighting(
        world
            .get_resource::<RenderStyle>()
            .copied()
            .unwrap_or_default(),
    )
}

/// Applies the selected style's lighting without respawning anything.
pub(super) fn apply_style(
    style: Res<RenderStyle>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut lights: Query<(&SceneLight, &mut DirectionalLight)>,
    mut cameras: Query<(&mut DistanceFog, Option<&mut EnvironmentMapLight>), With<Camera3d>>,
    mut materials: ResMut<Assets<ForestMaterial>>,
) {
    style_forest_materials(&mut materials, *style);
    let profile = bamboo_lighting(*style);
    clear.0 = profile.clear;
    ambient.color = profile.ambient;
    ambient.brightness = profile.ambient_brightness;
    for (role, mut light) in &mut lights {
        let spec = match role {
            SceneLight::Key => profile.key,
            SceneLight::Fill => profile.fill,
        };
        light.color = spec.color;
        light.illuminance = spec.illuminance;
    }
    for (mut fog, environment) in &mut cameras {
        *fog = profile.fog.clone();
        if let Some(mut environment) = environment {
            environment.intensity = profile.environment_intensity;
        }
    }
}

/// Called after camera creation, including in headless scene fixtures.
pub(super) fn setup_camera_fog(world: &mut World) {
    let cameras: Vec<_> = world
        .query_filtered::<Entity, With<Camera3d>>()
        .iter(world)
        .collect();
    let fog = current_lighting(world).fog;
    let lighting = world
        .get_resource::<ForestLighting>()
        .map(|lighting| lighting.0.clone());
    for camera in cameras {
        world.entity_mut(camera).insert(fog.clone());
        if let Some(lighting) = &lighting {
            world.entity_mut(camera).insert(lighting.clone());
        }
    }
}

pub(super) fn setup(world: &mut World) {
    let (ground, trunks, rocks, bamboo, boundaries) = {
        let level = world.resource::<Level>();
        (
            level.ground.clone(),
            level.forest_trunks.clone(),
            level.forest_rocks.clone(),
            level.bamboo.clone(),
            level.boundaries(),
        )
    };
    let profile = current_lighting(world);
    world.insert_resource(ClearColor(profile.clear));
    world.insert_resource(GlobalAmbientLight {
        color: profile.ambient,
        brightness: profile.ambient_brightness,
        ..default()
    });
    world.spawn((
        SceneLight::Key,
        DirectionalLight {
            color: profile.key.color,
            illuminance: profile.key.illuminance,
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
        SceneLight::Fill,
        DirectionalLight {
            color: profile.fill.color,
            illuminance: profile.fill.illuminance,
            ..default()
        },
        Transform::from_xyz(8.0, 6.0, -12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<ForestMaterial>>();
    world.init_resource::<WindClock>();
    let lighting = EnvironmentMapLight::hemispherical_gradient(
        &mut world.resource_mut::<Assets<Image>>(),
        Color::srgb(0.44, 0.59, 0.73),
        Color::srgb(0.24, 0.32, 0.31),
        Color::srgb(0.13, 0.15, 0.13),
    )
    .with_intensity(profile.environment_intensity);
    world.insert_resource(ForestLighting(lighting));
    // The adjacent manifest retains Model -> material -> named texture refs.
    // Source parameter blocks were unavailable; response scalars are authored.
    let leaves_base = texture(world, "environment/bamboo/leaves.png", true, false);
    let leaves_normal = texture(world, "environment/bamboo/leaves_normal.png", false, false);
    let bamboo_base = texture(world, "environment/bamboo/bamboo.png", true, true);
    let bamboo_normal = texture(world, "environment/bamboo/bamboo_normal.png", false, true);
    let litter_base = texture(world, "environment/bamboo/litter.png", true, false);
    let litter_normal = texture(world, "environment/bamboo/litter_normal.png", false, false);
    let ground_base = texture(world, "environment/bamboo/ground_base.png", true, true);
    let ground_normal = texture(world, "environment/bamboo/ground_normal.png", false, true);
    let cover_base = texture(
        world,
        "environment/bamboo/forest_floor_base.png",
        true,
        true,
    );
    let cover_normal = texture(
        world,
        "environment/bamboo/forest_floor_normal.png",
        false,
        true,
    );
    let (soil, bamboo_material, stone, foliage, litter_material) = {
        let mut materials = world.resource_mut::<Assets<ForestMaterial>>();
        let mut surface = |kind, roughness, thin| {
            let maps = if kind == 0.0 {
                Some((&ground_base, &ground_normal))
            } else if kind == 1.0 {
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
                    // Deferred GBuffer cannot retain thin-surface transmission.
                    opaque_render_method: if cutout {
                        OpaqueRendererMethod::Forward
                    } else {
                        OpaqueRendererMethod::Auto
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
                    style: Vec4::new(
                        if kind == 4.0 { 0.0 } else { kind },
                        1.0,
                        1.0,
                        if kind == 0.0 { 1.0 } else { 0.0 },
                    ),
                    cover_base: (kind == 0.0).then(|| cover_base.clone()),
                    cover_normal: (kind == 0.0).then(|| cover_normal.clone()),
                    cover_strength: if kind == 0.0 { 0.85 } else { 0.0 },
                    yancai: Vec4::ZERO,
                },
            })
        };
        (
            surface(0.0, 0.82, 0.0),
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
        None,
    );
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
        None,
    );
    let rock_colors = rocks
        .vertices
        .iter()
        .map(|vertex| {
            let moss = ((1.2 - vertex.y) * 0.04).max(0.0);
            [0.18, 0.19 + moss, 0.155, 1.0]
        })
        .collect();
    spawn_mesh(
        world,
        &rocks,
        rock_colors,
        &stone,
        EnvironmentPart::Rocks,
        None,
    );
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
        None,
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
                strength: 1.0,
            },
        ));
    }
    ground_cover::setup(world, &stone, &litter_material);
    let style = world
        .get_resource::<RenderStyle>()
        .copied()
        .unwrap_or_default();
    style_forest_materials(&mut world.resource_mut::<Assets<ForestMaterial>>(), style);
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
            ) * leaf.strength;
        transform.rotation = Quat::from_rotation_z(phase.sin() * 0.0015 * leaf.strength);
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
    // The recovered albedo supplies soil colour and fine detail. Vertex tint
    // only separates the worn path slightly; it must not recolour it olive.
    let grain = (point.x * 2.7 + point.y * 1.3).sin() * (point.y * 3.1).cos() * 0.008;
    [
        0.90 + worn * 0.07 + grain,
        0.93 + worn * 0.04 + grain,
        0.94 + worn * 0.02 + grain,
        1.0,
    ]
}

#[derive(Default)]
struct Decoration {
    geometry: TerrainGeometry,
    colors: Vec<[f32; 4]>,
    uvs: Option<Vec<[f32; 2]>>,
}

impl Decoration {
    fn quad(&mut self, vertices: [Vec3; 4], color: [f32; 4]) {
        let start = self.geometry.vertices.len() as u32;
        self.geometry.vertices.extend(vertices);
        self.geometry
            .triangles
            .extend([[start, start + 1, start + 2], [start, start + 2, start + 3]]);
        self.colors.extend([color; 4]);
        if let Some(uvs) = &mut self.uvs {
            uvs.extend([[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
        }
    }

    fn atlas_quad(&mut self, vertices: [Vec3; 4], color: [f32; 4], uv: [f32; 4]) {
        let start = self.geometry.vertices.len();
        assert!(self.uvs.is_some(), "atlas decoration needs explicit UVs");
        self.quad(vertices, color);
        let [left, top, right, bottom] = uv;
        self.uvs.as_mut().unwrap()[start..].copy_from_slice(&[
            [left, bottom],
            [right, bottom],
            [right, top],
            [left, top],
        ]);
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
    for branch in 0..4 {
        let fraction = 0.56 + (branch / 2) as f32 * 0.24;
        let angle = variation(index, 30 + branch) * std::f32::consts::TAU;
        let direction = Vec3::new(angle.cos(), 0.22, angle.sin()).normalize();
        let across = Vec3::new(-angle.sin(), 0.0, angle.cos());
        let start = stem.center_at(fraction);
        // The verified alpha atlas contains a whole branch cluster. A broad
        // card preserves its own stems/leaves; one tiny card per leaf would
        // shrink the atlas detail away and destroy coverage.
        let length = 1.65 + variation(index, 40 + branch) * 1.0;
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
        decoration.uvs,
    )
}

fn spawn_mesh(
    world: &mut World,
    geometry: &TerrainGeometry,
    colors: Vec<[f32; 4]>,
    material: &Handle<ForestMaterial>,
    part: EnvironmentPart,
    uv_override: Option<Vec<[f32; 2]>>,
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
    let uvs = uv_override.unwrap_or_else(|| {
        geometry
            .vertices
            .iter()
            .enumerate()
            .map(|(index, point)| match part {
                EnvironmentPart::Leaves | EnvironmentPart::Litter => card_uvs[index % 4],
                // The verified grass atlas root ends at v≈.91. Crop its transparent
                // lower padding so stems reach the physical ground.
                EnvironmentPart::Grass => {
                    [[0.0, 0.92], [1.0, 0.92], [1.0, 0.02], [0.0, 0.02]][index % 4]
                }
                EnvironmentPart::Trunks => [((index % 42) % 8) as f32 / 8.0, -point.y / 3.6],
                // 2.5m per original 1024px soil tile is a Kiva scale adaptation.
                EnvironmentPart::Ground => [point.x / 2.5, point.z / 2.5],
                _ => [point.x * 0.25, point.z * 0.25],
            })
            .collect()
    });
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
        let level = world.resource::<Level>();
        let expected = [
            (EnvironmentPart::Ground, level.ground.clone()),
            (EnvironmentPart::Trunks, level.forest_trunks.clone()),
            (EnvironmentPart::Rocks, level.forest_rocks.clone()),
            (
                EnvironmentPart::Perimeter,
                perimeter_geometry(level.boundaries()),
            ),
        ];
        setup(&mut world);
        let mut query = world.query::<(
            &EnvironmentPart,
            &Mesh3d,
            Option<&avian3d::prelude::Collider>,
        )>();
        let parts: Vec<_> = query
            .iter(&world)
            .map(|(part, mesh, collider)| {
                assert!(
                    collider.is_none(),
                    "presentation cannot add collision authority"
                );
                (*part, mesh.0.clone())
            })
            .collect();
        assert_eq!(parts.len(), 19);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), 19);
        assert_eq!(world.resource::<Assets<ForestMaterial>>().len(), 7);
        let materials = world.resource::<Assets<ForestMaterial>>();
        let images = world.resource::<Assets<Image>>();
        let textured: Vec<_> = materials
            .iter()
            .filter(|(_, material)| material.base.base_color_texture.is_some())
            .collect();
        assert_eq!(textured.len(), 6);
        for (_, material) in textured {
            let base = images
                .get(material.base.base_color_texture.as_ref().unwrap())
                .unwrap();
            let normal = images
                .get(material.base.normal_map_texture.as_ref().unwrap())
                .unwrap();
            assert!(base.texture_descriptor.format.is_srgb());
            assert!(!normal.texture_descriptor.format.is_srgb());
            let ImageSampler::Descriptor(sampler) = &base.sampler else {
                panic!("forest textures must use explicit samplers");
            };
            for image in [base, normal] {
                let expected_mips = if sampler.address_mode_u == ImageAddressMode::Repeat {
                    image.texture_descriptor.size.width.ilog2() + 1
                } else {
                    1
                };
                assert_eq!(image.texture_descriptor.mip_level_count, expected_mips);
            }
        }
        for (part, source) in expected {
            let (_, handle) = parts.iter().find(|(found, _)| *found == part).unwrap();
            let mesh = world.resource::<Assets<Mesh>>().get(handle).unwrap();
            let VertexAttributeValues::Float32x3(vertices) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
            else {
                panic!("solid forest parts must expose their source positions");
            };
            assert_eq!(
                vertices,
                &source
                    .vertices
                    .iter()
                    .map(|v| v.to_array())
                    .collect::<Vec<_>>(),
                "{part:?} changed collision-owned positions"
            );
            assert_eq!(
                mesh.indices().unwrap().iter().collect::<Vec<_>>(),
                source
                    .triangles
                    .iter()
                    .flatten()
                    .map(|index| *index as usize)
                    .collect::<Vec<_>>(),
                "{part:?} changed collision-owned triangles"
            );
        }
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
    fn textured_soil_preserves_albedo_and_uses_metric_uvs_with_a_dry_material_baseline() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        crate::gameplay::setup(&mut world);
        setup(&mut world);
        let (_, mesh, material) = world
            .query::<(&EnvironmentPart, &Mesh3d, &MeshMaterial3d<ForestMaterial>)>()
            .iter(&world)
            .find(|(part, _, _)| **part == EnvironmentPart::Ground)
            .unwrap();
        let material = world
            .resource::<Assets<ForestMaterial>>()
            .get(&material.0)
            .unwrap();
        assert_eq!(material.base.alpha_mode, AlphaMode::Opaque);
        assert_eq!(material.base.metallic, 0.0);
        assert_eq!(material.base.perceptual_roughness, 0.82);
        assert_eq!(material.extension.style.w, 1.0);
        assert_eq!(material.extension.cover_strength, 0.85);
        let image = world
            .resource::<Assets<Image>>()
            .get(material.base.base_color_texture.as_ref().unwrap())
            .unwrap();
        let ImageSampler::Descriptor(sampler) = &image.sampler else {
            panic!("soil must use its explicit repeating sampler");
        };
        assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
        assert_eq!(sampler.address_mode_v, ImageAddressMode::Repeat);
        assert_eq!(image.texture_descriptor.mip_level_count, 11);
        for (handle, srgb) in [
            (material.extension.cover_base.as_ref().unwrap(), true),
            (material.extension.cover_normal.as_ref().unwrap(), false),
        ] {
            let image = world.resource::<Assets<Image>>().get(handle).unwrap();
            assert_eq!(image.texture_descriptor.format.is_srgb(), srgb);
            assert_eq!(image.texture_descriptor.size.width, 1024);
            assert_eq!(image.texture_descriptor.size.height, 1024);
            assert_eq!(image.texture_descriptor.mip_level_count, 11);
        }
        let mesh = world.resource::<Assets<Mesh>>().get(&mesh.0).unwrap();
        let VertexAttributeValues::Float32x3(positions) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!("soil positions");
        };
        let VertexAttributeValues::Float32x2(uvs) = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("soil UVs");
        };
        for (position, uv) in positions.iter().zip(uvs) {
            assert!((uv[0] * 2.5 - position[0]).abs() < 0.00001);
            assert!((uv[1] * 2.5 - position[2]).abs() < 0.00001);
        }
        let VertexAttributeValues::Float32x4(colors) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!("soil tint");
        };
        assert!(colors.iter().all(|rgba| {
            rgba[..3]
                .iter()
                .all(|channel| (0.88..=1.0).contains(channel))
                && rgba[3] == 1.0
        }));
    }

    /// Every style-owned value on the clearing, comparable across switches.
    fn style_snapshot(world: &mut World) -> String {
        let mut lights: Vec<_> = world
            .query::<(&SceneLight, &DirectionalLight)>()
            .iter(world)
            .map(|(role, light)| format!("{role:?} {:?} {}", light.color, light.illuminance))
            .collect();
        lights.sort();
        let camera = world
            .query_filtered::<Entity, With<super::super::camera::GameCamera>>()
            .single(world)
            .unwrap();
        let fog = world.get::<DistanceFog>(camera).unwrap();
        let ambient = world.resource::<GlobalAmbientLight>();
        let mut styled: Vec<_> = world
            .resource::<Assets<ForestMaterial>>()
            .iter()
            .map(|(_, material)| format!("{:?}", material.extension.yancai))
            .collect();
        styled.dedup();
        format!(
            "materials {styled:?} | {:?} | {:?} {} | {lights:?} | {:?} {:?} {} {:?} | env {:?} | ssr {:?} | post {:?} {:?} | normals {}",
            world.resource::<ClearColor>().0,
            ambient.color,
            ambient.brightness,
            fog.color,
            fog.directional_light_color,
            fog.directional_light_exponent,
            fog.falloff,
            world
                .get::<EnvironmentMapLight>(camera)
                .map(|light| light.intensity),
            world
                .get::<bevy::pbr::ScreenSpaceReflections>(camera)
                .map(|ssr| (ssr.linear_steps, ssr.thickness)),
            world.get::<super::super::render_style::yancai_post::YancaiShade>(camera),
            world.get::<super::super::render_style::yancai_post::YancaiPaper>(camera),
            world
                .get::<bevy::core_pipeline::prepass::NormalPrepass>(camera)
                .is_some(),
        )
    }

    fn styled_clearing() -> App {
        use super::super::{camera::GameCamera, render_style, rendering};
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(RenderStyle::default())
            .insert_resource(render_style::LevelStyles::for_level(
                crate::gameplay::GameLevel::BambooForest,
            ))
            .add_systems(
                Update,
                (apply_style, rendering::apply_style).run_if(render_style::switched),
            );
        crate::gameplay::setup(app.world_mut());
        setup(app.world_mut());
        app.world_mut().spawn((GameCamera, Camera3d::default()));
        rendering::setup(app.world_mut());
        setup_camera_fog(app.world_mut());
        app.update();
        app
    }

    #[test]
    fn switching_styles_round_trips_the_shipped_lighting_without_growing_the_scene() {
        let mut app = styled_clearing();
        let world = app.world_mut();
        let shipped = style_snapshot(world);
        let counts = |world: &mut World| {
            (
                world.query::<Entity>().iter(world).count(),
                world.resource::<Assets<Mesh>>().len(),
                world.resource::<Assets<ForestMaterial>>().len(),
                world.resource::<Assets<Image>>().len(),
            )
        };
        let before = counts(world);
        // The setup reads the same Pbr profile that switching applies.
        let pbr = bamboo_lighting(RenderStyle::Pbr);
        assert!(shipped.contains(&format!("{:?}", pbr.clear)));
        assert!(shipped.contains("Some(310.0)"));
        assert!(shipped.contains("ssr Some(") && shipped.ends_with("normals true"));

        *app.world_mut().resource_mut::<RenderStyle>() = RenderStyle::Yancai;
        app.update();
        let yancai = style_snapshot(app.world_mut());
        assert_ne!(yancai, shipped);
        assert!(yancai.contains("ssr None") && yancai.ends_with("normals true"));
        // The post chain uses the same key-light normalisation as the
        // forest material's forward bands.
        assert!(yancai.contains(&format!(
            "post Some(YancaiShade {{ light_over_pi: {:?}, bands: 3.0 }}) Some(",
            forest_style(RenderStyle::Yancai).y
        )));
        assert!(shipped.contains("post None None"));
        // Every forest material switches together, to the key-light bands.
        assert!(yancai.starts_with(&format!(
            "materials [\"{:?}\"]",
            forest_style(RenderStyle::Yancai)
        )));
        assert!(shipped.starts_with(&format!("materials [\"{:?}\"]", Vec4::ZERO)));
        assert!(yancai.contains(&format!(
            "env Some({:?})",
            bamboo_lighting(RenderStyle::Yancai).environment_intensity
        )));

        for _ in 0..3 {
            for style in [RenderStyle::Pbr, RenderStyle::Yancai] {
                *app.world_mut().resource_mut::<RenderStyle>() = style;
                app.update();
            }
        }
        *app.world_mut().resource_mut::<RenderStyle>() = RenderStyle::Pbr;
        app.update();
        let world = app.world_mut();
        assert_eq!(style_snapshot(world), shipped);
        assert_eq!(counts(world), before);
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
    fn only_decorative_foliage_sways_and_pause_preserves_pose_and_shared_assets() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Time>()
            .insert_resource(SimulationActive(true))
            .add_systems(Update, animate_foliage);
        crate::gameplay::setup(app.world_mut());
        setup(app.world_mut());
        let asset_counts = (
            app.world().resource::<Assets<Mesh>>().len(),
            app.world().resource::<Assets<ForestMaterial>>().len(),
            app.world().resource::<Assets<Image>>().len(),
        );
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
        assert_eq!(before.len(), 12);
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
        assert_eq!(
            asset_counts,
            (
                app.world().resource::<Assets<Mesh>>().len(),
                app.world().resource::<Assets<ForestMaterial>>().len(),
                app.world().resource::<Assets<Image>>().len(),
            )
        );
    }
}
