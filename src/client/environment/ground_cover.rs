//! Small, non-colliding forest-floor dressing. All solids remain Level-owned.
use super::{
    Decoration, EnvironmentPart, ForestMaterial, ForestSurface, WindDecoration, spawn_decoration,
    texture,
};
use crate::gameplay::{path_distance, terrain, variation};
use bevy::{material::OpaqueRendererMethod, prelude::*};

#[derive(Debug)]
struct Patch {
    point: Vec2,
    seed: u32,
}

/// Keep a two-metre spawn disk, the central fighting lane, winding routes and
/// ledge clear. The former whole-clearing ellipse pushed all cover off camera.
/// Margins account for each grass card's half-width and its small wind sway.
fn cover_allowed(point: Vec2) -> bool {
    let spawn = point.length() < 2.6;
    let target =
        point.distance(Vec2::new(0.0, -4.0)) < 2.5 || point.distance(Vec2::new(0.0, -6.0)) < 2.5;
    let fighting_lane = point.x.abs() < 2.45 && (-8.6..=1.5).contains(&point.y);
    let ledge = (-7.1..=-1.9).contains(&point.x)
        && (-0.5..=terrain::ramp_summit() + 3.8).contains(&point.y);
    !spawn
        && !target
        && !fighting_lane
        && !ledge
        && path_distance(point) > 2.4
        && point.abs().max_element() < 24.0
}

/// Flat litter and tiny stone chips can occur on the dirt between grass clumps.
/// They have no collision and never cover the player's immediate spawn disk.
fn floor_detail_allowed(point: Vec2) -> bool {
    point.length() > 2.85
        && point.distance(Vec2::new(0.0, -4.0)) > 1.1
        && point.distance(Vec2::new(0.0, -6.0)) > 1.1
        && !((-6.7..=-2.3).contains(&point.x)
            && (0.0..=terrain::ramp_summit() + 3.5).contains(&point.y))
}

fn patches() -> Vec<Patch> {
    let mut patches = Vec::new();
    // Irregular elliptical patches replace the visible rows of equal tufts.
    // Cluster centres and their density do not depend on a spatial grid.
    for cluster in 0..110 {
        let center = Vec2::new(variation(cluster, 100) - 0.5, variation(cluster, 101) - 0.5) * 46.0;
        let angle = variation(cluster, 102) * std::f32::consts::TAU;
        let along = Vec2::new(angle.cos(), angle.sin());
        let across = Vec2::new(-angle.sin(), angle.cos());
        let length = 0.9 + variation(cluster, 103) * 1.7;
        let width = 0.45 + variation(cluster, 104) * 0.9;
        let count = 7 + (variation(cluster, 105) * 12.0) as u32;
        for sample in 0..count {
            let seed = cluster * 32 + sample;
            let angle = variation(seed, 106) * std::f32::consts::TAU;
            let radius = variation(seed, 107).sqrt();
            let point =
                center + (along * (angle.cos() * length) + across * (angle.sin() * width)) * radius;
            if cover_allowed(point)
                && patches
                    .iter()
                    .all(|patch: &Patch| patch.point.distance_squared(point) > 0.14_f32.powi(2))
            {
                patches.push(Patch { point, seed });
            }
        }
    }
    patches
}

pub(super) fn setup(
    world: &mut World,
    stone: &Handle<ForestMaterial>,
    litter_material: &Handle<ForestMaterial>,
) {
    let patches = patches();
    let mut grass: [Decoration; 4] = std::array::from_fn(|_| Decoration::default());
    let mut blades: [Decoration; 4] = std::array::from_fn(|_| Decoration {
        uvs: Some(Vec::new()),
        ..default()
    });
    let mut pebbles = Decoration::default();
    let mut litter = Decoration::default();
    for (index, patch) in patches.iter().enumerate() {
        fine_grass(&mut blades[index % 4], patch.point, patch.seed);
        // The broad-leaf source is an occasional smaller understory accent.
        if variation(patch.seed, 302) < 0.14 {
            grass_tuft(&mut grass[index % 4], patch.point, patch.seed, 0);
        }
        let chip_point = patch.point
            + Vec2::new(
                variation(patch.seed, 150) - 0.5,
                variation(patch.seed, 160) - 0.5,
            ) * 1.7;
        if cover_allowed(chip_point) && variation(patch.seed, 164) < 0.35 {
            pebble(&mut pebbles, chip_point, patch.seed, 0);
        }
        // Litter is concentrated around tuft clusters, rather than a uniform
        // scatter that covers the readable clearing and spawn point.
        let leaf_point = patch.point
            + Vec2::new(
                variation(patch.seed, 170) - 0.5,
                variation(patch.seed, 180) - 0.5,
            ) * 1.5;
        if variation(patch.seed, 184) < 0.55 {
            litter_card(&mut litter, leaf_point, patch.seed, 0);
        }
    }
    // Sparse islands of fallen leaves and small chips make the close fighting
    // ground read as forest floor while retaining its open movement space.
    for index in 0..130 {
        let point = Vec2::new(
            (variation(index, 251) - 0.5) * 19.0,
            -3.5 + (variation(index, 252) - 0.5) * 18.0,
        );
        if !floor_detail_allowed(point) {
            continue;
        }
        litter_card(&mut litter, point, index, 3);
        let chip_point = point + Vec2::new(0.24, -0.12);
        if index % 3 == 0 && floor_detail_allowed(chip_point) {
            pebble(&mut pebbles, chip_point, index, 4);
        }
    }
    let grass_base = texture(
        world,
        include_bytes!("../../../assets/choulingke/environment/grass_base.png"),
        true,
        false,
    );
    let grass_normal = texture(
        world,
        include_bytes!("../../../assets/choulingke/environment/grass_normal.png"),
        false,
        false,
    );
    let blades_base = texture(
        world,
        include_bytes!("../../../assets/choulingke/environment/grass_blades_base.png"),
        true,
        false,
    );
    let blades_normal = texture(
        world,
        include_bytes!("../../../assets/choulingke/environment/grass_blades_normal.png"),
        false,
        false,
    );
    let grass_material = world
        .resource_mut::<Assets<ForestMaterial>>()
        .add(ForestMaterial {
            base: StandardMaterial {
                base_color_texture: Some(grass_base),
                normal_map_texture: Some(grass_normal),
                alpha_mode: AlphaMode::Mask(0.46),
                opaque_render_method: OpaqueRendererMethod::Forward,
                perceptual_roughness: 0.85,
                diffuse_transmission: 0.18,
                double_sided: true,
                cull_mode: None,
                ..default()
            },
            extension: ForestSurface {
                style: Vec4::new(3.0, 1.0, 1.0, 0.0),
                ..default()
            },
        });
    let blades_material = world
        .resource_mut::<Assets<ForestMaterial>>()
        .add(ForestMaterial {
            base: StandardMaterial {
                base_color_texture: Some(blades_base),
                normal_map_texture: Some(blades_normal),
                alpha_mode: AlphaMode::Mask(0.38),
                opaque_render_method: OpaqueRendererMethod::Forward,
                perceptual_roughness: 0.88,
                diffuse_transmission: 0.22,
                double_sided: true,
                cull_mode: None,
                ..default()
            },
            extension: ForestSurface {
                style: Vec4::new(3.0, 1.0, 1.0, 0.0),
                ..default()
            },
        });
    for (index, grass) in grass.into_iter().enumerate() {
        spawn_wind_cover(
            world,
            grass,
            &grass_material,
            EnvironmentPart::Grass,
            index,
            0.35,
        );
    }
    for (index, blades) in blades.into_iter().enumerate() {
        spawn_wind_cover(
            world,
            blades,
            &blades_material,
            EnvironmentPart::FineGrass,
            index,
            0.55,
        );
    }
    spawn_decoration(world, pebbles, stone, EnvironmentPart::Pebbles);
    spawn_decoration(world, litter, litter_material, EnvironmentPart::Litter);
}

fn spawn_wind_cover(
    world: &mut World,
    mut cover: Decoration,
    material: &Handle<ForestMaterial>,
    part: EnvironmentPart,
    cohort: usize,
    strength: f32,
) {
    let center = cover.geometry.vertices.iter().copied().sum::<Vec3>()
        / cover.geometry.vertices.len() as f32;
    for vertex in &mut cover.geometry.vertices {
        *vertex -= center;
    }
    let entity = spawn_decoration(world, cover, material, part);
    world.entity_mut(entity).insert((
        Transform::from_translation(center),
        WindDecoration {
            center,
            phase: cohort as f32 * 2.1 + 0.7,
            strength,
        },
    ));
}

fn grass_tuft(grass: &mut Decoration, point: Vec2, seed: u32, tuft: u32) {
    let yaw = variation(seed, 130 + tuft) * std::f32::consts::TAU;
    let height = 0.26 + variation(seed, 140 + tuft) * 0.24;
    for card in 0..2 {
        let angle = yaw + card as f32 * std::f32::consts::FRAC_PI_2;
        let width =
            Vec2::new(angle.cos(), angle.sin()) * (0.18 + variation(seed, 145 + tuft) * 0.11);
        let left = point - width;
        let right = point + width;
        let bottom_left = Vec3::new(left.x, terrain::hill_height(left) + 0.006, left.y);
        let bottom_right = Vec3::new(right.x, terrain::hill_height(right) + 0.006, right.y);
        let rise = Vec3::new(angle.sin() * 0.08, height, angle.cos() * 0.08);
        let tint = 0.90 + variation(seed, 148 + tuft) * 0.10;
        grass.quad(
            [
                bottom_left,
                bottom_right,
                bottom_right + rise,
                bottom_left + rise,
            ],
            [tint, tint, tint, 1.0],
        );
    }
}

fn fine_grass(grass: &mut Decoration, point: Vec2, seed: u32) {
    // Each region contains one inspected blade from the original dry-grass
    // atlas (14.1% opaque texels overall). No generated bitmap is substituted.
    let regions = [
        [10.0, 114.0, 38.0, 446.0],
        [58.0, 18.0, 98.0, 494.0],
        [132.0, 8.0, 170.0, 504.0],
        [250.0, 20.0, 286.0, 502.0],
        [288.0, 20.0, 328.0, 504.0],
    ];
    let count = 6 + (variation(seed, 310) * 7.0) as u32;
    let height_scale = 0.65 + variation(seed, 311) * 0.55;
    for blade in 0..count {
        let key = seed * 16 + blade;
        let angle = variation(key, 312) * std::f32::consts::TAU;
        let root =
            point + Vec2::new(angle.cos(), angle.sin()) * (variation(key, 313).sqrt() * 0.20);
        let width = Vec2::new(-angle.sin(), angle.cos()) * (0.018 + variation(key, 314) * 0.018);
        let left = root - width;
        let right = root + width;
        let bottom_left = Vec3::new(left.x, terrain::hill_height(left) + 0.006, left.y);
        let bottom_right = Vec3::new(right.x, terrain::hill_height(right) + 0.006, right.y);
        let height = (0.28 + variation(key, 315) * 0.31) * height_scale;
        let lean =
            Vec3::new(angle.cos(), 0.0, angle.sin()) * height * (0.10 + variation(key, 316) * 0.28);
        let region = regions[(variation(key, 317) * regions.len() as f32) as usize % regions.len()];
        // A restrained green/dry mix is a Kiva colour adaptation of the source
        // grey-ochre blades, not recovered original shader parameters.
        let dry = variation(key, 318);
        let color = [0.66 + dry * 0.23, 0.93, 0.63 + dry * 0.10, 1.0];
        grass.atlas_quad(
            [
                bottom_left,
                bottom_right,
                bottom_right + Vec3::Y * height + lean,
                bottom_left + Vec3::Y * height + lean,
            ],
            color,
            region.map(|pixel| pixel / 512.0),
        );
    }
}

fn pebble(decoration: &mut Decoration, point: Vec2, seed: u32, chip: u32) {
    let center = Vec3::new(point.x, terrain::hill_height(point), point.y);
    let radius = 0.055 + variation(seed, 190 + chip) * 0.09;
    let height = 0.035 + variation(seed, 200 + chip) * 0.045;
    let start = decoration.geometry.vertices.len() as u32;
    decoration.geometry.vertices.extend([
        center - Vec3::Y * 0.012,
        center + Vec3::X * radius,
        center + Vec3::Z * radius * 0.7,
        center - Vec3::X * radius,
        center - Vec3::Z * radius * 0.7,
        center + Vec3::Y * height,
    ]);
    for side in 1..=4 {
        let next = side % 4 + 1;
        decoration.geometry.triangles.extend([
            [start, start + side, start + next],
            [start + 5, start + next, start + side],
        ]);
    }
    let shade = 0.29 + variation(seed, 210 + chip) * 0.14;
    decoration
        .colors
        .extend([[shade * 0.94, shade, shade * 1.06, 1.0]; 6]);
}

fn litter_card(litter: &mut Decoration, point: Vec2, seed: u32, leaf: u32) {
    if !floor_detail_allowed(point) {
        return;
    }
    let yaw = variation(seed, 220 + leaf) * std::f32::consts::TAU;
    let length = 0.24 + variation(seed, 230 + leaf) * 0.3;
    let direction = Vec2::new(yaw.cos(), yaw.sin()) * length;
    let side = Vec2::new(-direction.y, direction.x);
    let vertices = [
        point - direction - side,
        point - direction + side,
        point + direction + side,
        point + direction - side,
    ]
    .map(|p| Vec3::new(p.x, terrain::hill_height(p) + 0.018, p.y));
    let shade = 0.88 + variation(seed, 240 + leaf) * 0.12;
    litter.quad(vertices, [shade, shade * 0.97, shade * 0.91, 1.0]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{camera::CameraProjection, ecs::system::RunSystemOnce};

    #[test]
    fn clusters_leave_clearings_paths_and_ledge_unobstructed() {
        let first = patches();
        let second = patches();
        assert!(first.len() > 180, "a continuous forest edge needs cover");
        assert_eq!(first.len(), second.len());
        for (a, b) in first.iter().zip(second) {
            assert_eq!(a.point, b.point);
            assert!(cover_allowed(a.point));
            assert!(a.point.distance(Vec2::ZERO) > 2.6);
            assert!(a.point.distance(Vec2::new(0.0, -6.0)) > 2.5);
        }
        for z in -25..=25 {
            let z = z as f32;
            assert!(!cover_allowed(Vec2::new(3.0 * (z * 0.14).sin(), z)));
        }
    }

    #[test]
    fn grass_card_roots_follow_the_ground_and_leave_spawn_and_routes_open() {
        let mut grass = Decoration::default();
        for patch in patches() {
            grass_tuft(&mut grass, patch.point, patch.seed, 0);
        }
        assert!(grass.geometry.vertices.len() > 1_000);
        for card in grass.geometry.vertices.as_chunks::<4>().0 {
            for root in &card[..2] {
                let point = root.xz();
                assert!(point.length() > 2.0, "grass entered the spawn disk");
                assert!(
                    point.x.abs() > 2.0 || !(-8.1..=1.0).contains(&point.y),
                    "grass entered the central fighting lane"
                );
                assert!(
                    path_distance(point) > 1.9,
                    "grass entered the walking route"
                );
                assert!((root.y - terrain::hill_height(point) - 0.006).abs() < 0.00001);
            }
            for side in 0..2 {
                let rise = card[3 - side].y - card[side].y;
                assert!((0.25..=0.51).contains(&rise), "grass scale changed: {rise}");
            }
        }
    }

    #[test]
    fn fine_grass_uses_separate_blade_regions_with_grounded_varied_heights() {
        let mut grass = Decoration {
            uvs: Some(Vec::new()),
            ..default()
        };
        for patch in patches() {
            fine_grass(&mut grass, patch.point, patch.seed);
        }
        let uvs = grass.uvs.as_ref().unwrap();
        assert_eq!(uvs.len(), grass.geometry.vertices.len());
        let mut regions = std::collections::HashSet::new();
        let mut min_height = f32::MAX;
        let mut max_height = 0.0_f32;
        for (card, uv) in grass
            .geometry
            .vertices
            .as_chunks::<4>()
            .0
            .iter()
            .zip(uvs.as_chunks::<4>().0)
        {
            for root in &card[..2] {
                assert!(root.xz().length() > 2.0);
                assert!(path_distance(root.xz()) > 1.9);
                assert!((root.y - terrain::hill_height(root.xz()) - 0.006).abs() < 0.00001);
            }
            let height = card[3].y - card[0].y;
            assert!((0.18..=0.71).contains(&height));
            min_height = min_height.min(height);
            max_height = max_height.max(height);
            // Full-atlas cards would reproduce a row of identical multi-blade
            // sprites. Each fine card instead samples one narrow source strip.
            assert!((0.05..=0.09).contains(&(uv[1][0] - uv[0][0])));
            assert!(uv[0][1] > uv[3][1]);
            assert!(uv.iter().flatten().all(|v| (0.0..=1.0).contains(v)));
            regions.insert(uv[0][0].to_bits());
        }
        assert_eq!(regions.len(), 5);
        assert!(min_height < 0.25 && max_height > 0.60);
    }

    #[test]
    fn initial_camera_sees_grass_clumps_beside_the_open_fighting_lane() {
        let mut world = World::new();
        world.init_resource::<crate::client::controls::ViewYaw>();
        world.spawn((crate::client::PlayerView, Transform::default()));
        world.run_system_once(crate::client::camera::setup).unwrap();
        let (transform, projection) = world
            .query::<(&Transform, &Projection)>()
            .single(&world)
            .unwrap();
        let Projection::Perspective(mut perspective) = projection.clone() else {
            panic!("initial game camera must be perspective");
        };
        perspective.aspect_ratio = 16.0 / 9.0;
        let clip_from_world = perspective.get_clip_from_view() * transform.to_matrix().inverse();
        let visible: Vec<_> = patches()
            .into_iter()
            .filter(|patch| {
                let point = patch.point;
                let ndc = clip_from_world.project_point3(Vec3::new(
                    point.x,
                    terrain::hill_height(point) + 0.35,
                    point.y,
                ));
                ndc.x.abs() < 0.9 && ndc.y.abs() < 0.8 && (0.0..=1.0).contains(&ndc.z)
            })
            .collect();
        let left = visible.iter().filter(|patch| patch.point.x < -2.0).count();
        let right = visible.iter().filter(|patch| patch.point.x > 2.0).count();
        assert!(
            left >= 2 && right >= 2,
            "grass must frame both sides: {visible:?}"
        );
        assert!(
            visible.len() >= 6,
            "the close forest floor needs visible cover"
        );
    }
}
