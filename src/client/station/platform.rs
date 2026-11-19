//! Platform, rain canopy, a parked carriage and scenery beyond the fenced walk.
use super::paint::{Paint, field, finish, ink};
use crate::gameplay::station::LAYOUT;
use bevy::{light::NotShadowCaster, prelude::*};

const WOOD: [u8; 3] = [117, 60, 47];
const ASH: [u8; 3] = [89, 81, 69];
const IRON: [u8; 3] = [57, 59, 56];
const RUST: [u8; 3] = [124, 86, 57];
const STONE: [u8; 3] = [94, 98, 89];
const WINDOW_WIDTH: f32 = 2.30;
const WINDOW_HEIGHT: f32 = 1.22;

fn timber(p: &mut Paint, x: f32, z: f32, h: f32) {
    let base = Vec3::new(x, 0., z);
    p.block(base + Vec3::Y * 0.12, Vec3::new(0.37, 0.24, 0.38), STONE);
    p.block(base + Vec3::Y * (h / 2.), Vec3::new(0.25, h, 0.25), WOOD);
    for (y, w, t) in [
        (0.28, 0.32, 0.08),
        (h - 0.28, 0.31, 0.10),
        (h - 0.14, 0.48, 0.11),
    ] {
        p.block(base + Vec3::Y * y, Vec3::new(w, t, w), ASH);
    }
    // Flaked lacquer exposing grey fibres on the two visible faces.
    for k in 0..75 {
        let a = field(k, 23);
        let b = field(k, 151);
        let c = field(k, 717);
        let y = 0.31 + b * (h - 0.65);
        let center = base + Vec3::new((a - 0.5) * 0.238, y, 0.1265);
        let dx = Vec3::X;
        let dy = Vec3::Y;
        let points = [
            center - dx * (0.003 + c * 0.008),
            center + dy * (0.05 + c * 0.16),
            center + dx * (0.002 + a * 0.005),
            center - dy * 0.04,
        ];
        let tint = if k % 3 == 0 {
            [161, 115, 79]
        } else {
            [82, 69, 58]
        };
        p.polygon(&points, tint);
        let side =
            points.map(|v| base + Quat::from_rotation_y(std::f32::consts::FRAC_PI_2) * (v - base));
        p.polygon(&side, tint);
    }
}

fn canopy(p: &mut Paint) {
    let x = LAYOUT.walk_min.x - 0.08;
    let h = 3.32;
    for k in 0..7 {
        let z = 3.2 - k as f32 * 4.6;
        timber(p, x, z, h);
        timber(p, x - 2.6, z, h - 0.12);
        p.block(
            Vec3::new(x - 1.08, h, z),
            Vec3::new(4.12, 0.16, 0.18),
            [70, 61, 52],
        );
        for direction in [-1., 1.] {
            let a = Vec3::new(x, h - 0.68, z);
            p.tube(
                a,
                a + Vec3::new(0., 0.64, direction * 0.81),
                Vec2::splat(0.084),
                4,
                [100, 77, 56],
            );
            p.tube(
                a,
                a + Vec3::new(direction * 0.79, 0.64, 0.),
                Vec2::splat(0.082),
                4,
                [100, 77, 56],
            );
        }
        if k % 2 == 1 {
            let c = Vec3::new(x + 0.74, h - 0.52, z);
            p.tube(
                c + Vec3::Y * 0.17,
                c + Vec3::Y * 0.52,
                Vec2::splat(0.012),
                6,
                IRON,
            );
            p.bands(
                &[
                    (c - Vec3::Y * 0.14, Vec2::splat(0.08)),
                    (c - Vec3::Y * 0.115, Vec2::splat(0.12)),
                    (c + Vec3::Y * 0.12, Vec2::splat(0.115)),
                    (c + Vec3::Y * 0.16, Vec2::splat(0.06)),
                ],
                6,
                [181, 145, 101],
            );
            p.tube(
                c - Vec3::Y * 0.17,
                c - Vec3::Y * 0.13,
                Vec2::new(0.073, 0.141),
                6,
                IRON,
            );
            p.tube(
                c + Vec3::Y * 0.125,
                c + Vec3::Y * 0.21,
                Vec2::new(0.155, 0.055),
                6,
                IRON,
            );
            for j in 0..6 {
                let ang = j as f32 * std::f32::consts::TAU / 6.;
                let s = Vec3::new(ang.sin(), 0., ang.cos()) * 0.119;
                p.tube(
                    c + s - Vec3::Y * 0.13,
                    c + s + Vec3::Y * 0.13,
                    Vec2::splat(0.012),
                    5,
                    IRON,
                );
            }
        }
    }
    for j in 0..12 {
        let t = j as f32 / 11.;
        let cx = x - 3.04 + t * 3.25;
        let y = h + 0.16 + (1. - t) * 0.54;
        p.block(
            Vec3::new(cx, y, -11.),
            Vec3::new(0.292, 0.11, 37.),
            if j % 4 == 0 {
                [76, 70, 62]
            } else {
                [66, 62, 56]
            },
        );
    }
    for (j, offset) in [-2.90, -1.88, -0.84, 0.13].iter().enumerate() {
        p.block(
            Vec3::new(x + offset, h + 0.12 + (3 - j) as f32 * 0.15, -11.),
            Vec3::new(0.11, 0.17, 37.),
            [83, 64, 50],
        );
    }
    // Maple branches lace through the open end of the eaves.
    for branch in 0..14 {
        let z = 3.0 - field(branch, 11) * 21.;
        let base = Vec3::new(x - 0.29, 1.19 + field(branch, 9), z);
        let tip = base
            + Vec3::new(
                0.52 + field(branch, 7) * 0.52,
                0.41 + field(branch, 199) * 0.98,
                -0.34,
            );
        p.tube(base, tip, Vec2::new(0.035, 0.009), 6, [67, 60, 47]);
        for i in 0..32 {
            let k = branch * 32 + i;
            let mid = base.lerp(tip, field(k, 81));
            let leaf = mid
                + Vec3::new(
                    (field(k, 252) - 0.36) * 0.62,
                    (field(k, 19) - 0.28) * 0.4,
                    (field(k, 91) - 0.5) * 0.45,
                );
            p.tube(mid, leaf, Vec2::new(0.008, 0.003), 4, [79, 64, 48]);
            let w = 0.026 + field(k, 103) * 0.058;
            let tint = [[137, 63, 40], [170, 78, 46], [123, 96, 57], [106, 98, 60]][k % 4];
            let rot = Quat::from_rotation_y((field(k, 71) - 0.5) * 2.)
                * Quat::from_rotation_z((field(k, 31) - 0.5) * 1.8);
            let axis = rot * Vec3::NEG_Y;
            let cross = rot * Vec3::X;
            // Five lobes of maple, with folded midribs across the tips.
            let edges = [
                (-0.21, -0.22),
                (-0.76, -0.27),
                (-0.41, 0.02),
                (-0.91, 0.34),
                (-0.32, 0.24),
                (-0.38, 0.72),
                (-0.13, 0.46),
                (0., 1.02),
                (0.16, 0.44),
                (0.46, 0.65),
                (0.32, 0.24),
                (0.88, 0.28),
                (0.42, 0.01),
                (0.73, -0.32),
                (0.19, -0.21),
            ];
            for j in 0..edges.len() {
                let at = |(u, v): (f32, f32)| leaf + w * (cross * u + axis * v);
                p.polygon(
                    &[
                        leaf + Vec3::Z * 0.009,
                        at(edges[j]),
                        at(edges[(j + 1) % edges.len()]),
                    ],
                    tint,
                );
            }
        }
    }
}

fn tracks(p: &mut Paint) {
    let right = LAYOUT.walk_max.x + 0.15;
    // Ballast under sleepers and the lowered verge at the back of the platform.
    p.block(
        Vec3::new(right + 4.2, -0.91, -16.),
        Vec3::new(8.7, 0.40, 55.),
        [96, 94, 80],
    );
    p.block(
        Vec3::new(LAYOUT.walk_min.x - 5., -0.59, -13.),
        Vec3::new(10., 0.30, 55.),
        [109, 116, 94],
    );
    for i in 0..2400 {
        let x = right + field(i, 17) * 4.9;
        let z = -42. + field(i, 97) * 51.;
        let c = Vec3::new(x, -0.706, z);
        let r = 0.025 + 0.051 * field(i, 31);
        p.polygon(
            &[
                c - Vec3::X * r,
                c + Vec3::Z * r * 0.7,
                c + Vec3::X * r * 0.82 + Vec3::Y * r * 0.39,
                c - Vec3::Z * r,
            ],
            [
                [123, 119, 101],
                [77, 82, 77],
                [128, 117, 99],
                [100, 104, 90],
            ][i % 4],
        );
    }
    for z in -42..9 {
        p.block(
            Vec3::new(right + 2.12, -0.65, z as f32),
            Vec3::new(2.5, 0.11, 0.20),
            [82, 75, 64],
        );
        if z % 2 == 0 {
            p.block(
                Vec3::new(right - 0.07, -0.22, z as f32),
                Vec3::new(0.23, 0.44, 1.994),
                [105, 105, 90],
            );
        }
    }
    for x in [right + 1.40, right + 2.82] {
        p.block(Vec3::new(x, -0.565, -17.), Vec3::new(0.11, 0.12, 53.), RUST);
        p.block(
            Vec3::new(x, -0.492, -17.),
            Vec3::new(0.09, 0.027, 53.),
            [158, 151, 133],
        );
    }
    for j in 0..6 {
        let z = 1. - j as f32 * 7.5;
        let x = right + 4.53;
        let y = 4.8 + field(j, 31) * 0.6;
        p.tube(
            Vec3::new(x, -0.7, z),
            Vec3::new(x, y, z - 0.29),
            Vec2::new(0.075, 0.041),
            7,
            ASH,
        );
        p.block(
            Vec3::new(x, y - 0.16, z - 0.29),
            Vec3::new(1.18, 0.067, 0.089),
            IRON,
        );
        for offset in [-0.46, -0.17, 0.40] {
            let a = Vec3::new(x + offset, y - 0.08, z - 0.29);
            p.tube(
                a,
                a + Vec3::Y * 0.13,
                Vec2::splat(0.045),
                6,
                [137, 152, 140],
            );
            if j == 5 {
                continue;
            }
            let dy = (field(j + 1, 31) - field(j, 31)) * 0.6;
            let cable = |t: f32| a + Vec3::new(0., 0.13 + dy * t - 0.90 * t * (1. - t), -7.5 * t);
            for i in 0..8 {
                p.tube(
                    cable(i as f32 / 8.),
                    cable((i + 1) as f32 / 8.),
                    Vec2::splat(0.012),
                    4,
                    IRON,
                );
            }
        }
    }
}

fn window_frame(p: &mut Paint, c: Vec3, w: f32, h: f32) {
    let trim = [115, 102, 74];
    p.block(
        c + Vec3::X * 0.065,
        Vec3::new(0.09, h + 0.17, w + 0.18),
        [47, 59, 56],
    );
    for sign in [-1., 1.] {
        p.block(
            c + Vec3::Y * h * 0.5 * sign,
            Vec3::new(0.085, 0.07, w + 0.1),
            trim,
        );
        p.block(
            c + Vec3::Z * w * 0.5 * sign,
            Vec3::new(0.08, h, 0.078),
            trim,
        );
    }
    p.block(
        c + Vec3::Y * (h * 0.31),
        Vec3::new(0.065, 0.036, w),
        [77, 77, 62],
    );
}

fn carriage(p: &mut Paint, panel: &mut Paint) {
    let c = LAYOUT.window;
    let length = 17.1;
    let center = Vec3::new(c.x + 1.42, 1.52, c.z - 3.10);
    p.block(center, Vec3::new(2.60, 2.53, length), [57, 74, 67]);
    p.block(center - Vec3::Y * 1.31, Vec3::new(2.49, 0.22, length), IRON);
    for (y, w) in [(2.84, 2.76), (2.95, 2.63), (3.05, 2.39), (3.11, 1.98)] {
        p.block(
            Vec3::new(center.x, y, center.z),
            Vec3::new(w, 0.14, length + 0.06),
            [69, 73, 66],
        );
    }
    for k in -3..=1 {
        let z = c.z + k as f32 * 3.04;
        let at = Vec3::new(c.x, c.y, z);
        window_frame(p, at, WINDOW_WIDTH, WINDOW_HEIGHT);
        // Opaque, oxidised blinds behind the frame.
        for slat in 0..8 {
            let y = WINDOW_HEIGHT * ((slat as f32 + 0.5) / 8. - 0.5);
            p.block(
                at + Vec3::new(0.018, y, 0.),
                Vec3::new(0.024, WINDOW_HEIGHT / 8. - 0.014, WINDOW_WIDTH - 0.08),
                if slat % 3 == 0 {
                    [90, 99, 86]
                } else {
                    [77, 88, 80]
                },
            );
        }
        let x = c.x + 0.098;
        let pt = |az, ay| Vec3::new(x, ay, z + az);
        panel.face(
            &[
                pt(-1.51, 0.32),
                pt(1.51, 0.32),
                pt(1.51, 0.96),
                pt(-1.51, 0.96),
            ],
            &[Vec2::Y, Vec2::ONE, Vec2::X, Vec2::ZERO],
            [255; 3],
            0.83,
        );
        for dz in [-1.54, 1.48] {
            p.block(
                Vec3::new(c.x + 0.07, 1.51, z + dz),
                Vec3::new(0.075, 2.35, 0.053),
                [104, 97, 77],
            );
        }
    }
    for dz in [-6.8, -5.05, 4.95, 6.70] {
        let z = center.z + dz;
        p.tube(
            Vec3::new(c.x + 0.36, -0.17, z),
            Vec3::new(c.x + 0.58, -0.17, z),
            Vec2::splat(0.37),
            18,
            IRON,
        );
    }
}

pub(super) fn spawn(
    world: &mut World,
    plain: &Handle<StandardMaterial>,
    landscape: Handle<Image>,
    floor: Handle<Image>,
    coach: Handle<Image>,
) {
    let mut scenery = Paint::default();
    for curb in LAYOUT.curbs() {
        scenery.block(curb.center, curb.size, STONE);
    }
    // Remnant of the collapsed east bay of the rain shelter. The entire
    // solid post matches the gameplay collider, including its footprint.
    let foot = LAYOUT.pillar;
    let size = LAYOUT.pillar_size;
    scenery.block(foot + Vec3::Y * size.y * 0.5, size, WOOD);
    for (y, thick, tint) in [
        (0.18, 0.36, STONE),
        (0.51, 0.04, RUST),
        (size.y - 0.3, 0.12, ASH),
    ] {
        scenery.block(
            foot + Vec3::Y * y,
            Vec3::new(size.x + 0.012, thick, size.z + 0.012),
            tint,
        );
    }
    // One exposed roof truss still ties this post to the intact west eaves.
    // Its cross beam and diagonal brace can be seen above the reflected actors.
    let top = foot + Vec3::Y * size.y;
    let west = LAYOUT.walk_min.x - 0.08;
    scenery.block(
        Vec3::new((west + top.x) * 0.5, top.y + 0.05, top.z),
        Vec3::new(top.x - west + 0.52, 0.16, 0.18),
        ASH,
    );
    scenery.block(
        top + Vec3::new(0., 0.12, -0.85),
        Vec3::new(0.18, 0.19, 2.52),
        WOOD,
    );
    for dx in [0.33, 0.83] {
        scenery.tube(
            top - Vec3::Y * dx * 0.8,
            top - Vec3::X * dx,
            Vec2::splat(0.063),
            4,
            ASH,
        );
    }
    for dz in [-1.65, -0.55] {
        let end = top + Vec3::new(0., 0.20, dz);
        scenery.tube(
            end,
            end + Vec3::new(-1.63, 0.13, 0.13),
            Vec2::new(0.052, 0.039),
            4,
            ASH,
        );
    }
    canopy(&mut scenery);
    tracks(&mut scenery);
    let mut panel = Paint::default();
    carriage(&mut scenery, &mut panel);
    finish(world, scenery.mesh(), plain, Transform::IDENTITY);
    let coach = ink(world, Some(coach));
    finish(world, panel.mesh(), &coach, Transform::IDENTITY);

    let mut ground = Paint::default();
    // UV V=0 carries the painted safety line, always along the platform edge.
    let right = LAYOUT.walk_max.x;
    let left = LAYOUT.walk_min.x;
    let front = LAYOUT.walk_max.y;
    let back = LAYOUT.walk_min.y;
    for i in 0..24 {
        let a = i as f32 / 24.;
        let b = (i + 1) as f32 / 24.;
        let x = |t: f32| left + (right - left) * t;
        let uv = |t, z: f32| Vec2::new(-z / 6., 1. - t);
        let shade = 0.49 + 0.47 * (1.5_f32 * a + 0.12).clamp(0., 1.);
        ground.face(
            &[
                Vec3::new(x(a), 0., front),
                Vec3::new(x(b), 0., front),
                Vec3::new(x(b), 0., back),
                Vec3::new(x(a), 0., back),
            ],
            &[uv(a, front), uv(b, front), uv(b, back), uv(a, back)],
            [255; 3],
            shade,
        );
    }
    let floor = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color_texture: Some(floor),
            perceptual_roughness: 0.95,
            ..default()
        });
    finish(world, ground.mesh(), &floor, Transform::IDENTITY);

    // Continue the painted mountain horizon through the cloud cover overhead.
    // Both cameras see this world-space dome. Its upper latitudes use only the
    // sky portion of the landscape, converging on one texel at the zenith.
    let mut sky = Paint::default();
    let ring = |sector: usize, elevation: usize| {
        let azimuth = std::f32::consts::TAU * sector as f32 / 64.;
        let latitude = (-18. + 108. * elevation as f32 / 18.).to_radians();
        let (s, c) = azimuth.sin_cos();
        let (y, r) = latitude.sin_cos();
        let direction = Vec3::new(r * s, y, -r * c);
        let u = 0.5 + 0.49 * r * (azimuth + 0.50).sin();
        let v = (0.71 - 0.68 * y).clamp(0.02, 0.98);
        (direction * 85., Vec2::new(u, v))
    };
    for latitude in 0..18 {
        for sector in 0..64 {
            let corners = [
                ring(sector, latitude),
                ring(sector + 1, latitude),
                ring(sector + 1, latitude + 1),
                ring(sector, latitude + 1),
            ];
            sky.face(
                &corners.map(|(pos, _)| pos),
                &corners.map(|(_, uv)| uv),
                [255; 3],
                1.,
            );
        }
    }
    let clouds = ink(world, Some(landscape));
    let dome = finish(world, sky.mesh(), &clouds, Transform::IDENTITY);
    world.entity_mut(dome).insert(NotShadowCaster);
}
