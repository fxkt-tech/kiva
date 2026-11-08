//! Art recipes describe local geometry without allocating ECS or assets.
use super::{
    drawing::{Geometry, Layer},
    recipes::Shape,
    sampling::{LiveKind, Request},
};
use crate::client::skill_assets::{MeshKey, SpriteForward, TextureKey, mesh_spec, texture_spec};
use bevy::{math::Affine2, prelude::*};

fn mesh_layer(key: MeshKey, transform: Transform) -> Layer {
    Layer::new(Geometry::Mesh(key), mesh_spec(key).texture, transform)
}

/// Align authored image forward with local -Z, independently of the carrier yaw.
fn directed_sprite(texture: TextureKey, width: f32, length: f32) -> Transform {
    match texture_spec(texture).sprite_forward {
        SpriteForward::Right => ground(Vec2::new(length, width), 0.0).with_rotation(
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        ),
        SpriteForward::Up | SpriteForward::None => ground(Vec2::new(width, length), 0.0),
    }
}

fn ground(size: Vec2, height: f32) -> Transform {
    Transform::from_xyz(0.0, height, 0.0)
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(size.x, size.y, 1.0))
}
fn fit_mesh(key: MeshKey, size: Vec3, center: Vec3) -> Transform {
    let bounds = mesh_spec(key).bounds;
    let extent = bounds.size();
    let scale = Vec3::new(
        if extent.x > 1e-5 {
            size.x / extent.x
        } else {
            1.0
        },
        if extent.y > 1e-5 {
            size.y / extent.y
        } else {
            1.0
        },
        if extent.z > 1e-5 {
            size.z / extent.z
        } else {
            1.0
        },
    );
    Transform::from_translation(center - bounds.center() * scale).with_scale(scale)
}
fn ring_mesh(key: MeshKey, radius: f32) -> Transform {
    let bounds = mesh_spec(key).bounds;
    let extent = Vec3::from_array(bounds.min)
        .abs()
        .max(Vec3::from_array(bounds.max).abs());
    Transform::from_scale(Vec3::splat(radius / extent.x.max(extent.z).max(0.001)))
}

pub(super) fn burst_layers(shape: Shape) -> Vec<Layer> {
    let mut layers = match shape {
        Shape::Thrust { length, width } => {
            let mut layer = Layer::new(
                Geometry::Quad,
                TextureKey::JarvanThrustStreak,
                directed_sprite(TextureKey::JarvanThrustStreak, width.max(0.25), length)
                    .with_translation(Vec3::new(0.0, 0.0, -length * 0.5)),
            );
            layer.tint = Color::srgba(1.0, 0.84, 0.42, 0.95);
            vec![layer]
        }
        Shape::Nova { radius } => {
            let mut layer = mesh_layer(
                MeshKey::JarvanNovaRing,
                ring_mesh(MeshKey::JarvanNovaRing, radius),
            );
            // Reviewed W shockwave2: source uvRotation=90 degrees maps the
            // radial V coordinate onto texture U (1-V), not a white wedge.
            layer.uv_override = Some(Affine2::from_scale_angle_translation(
                Vec2::ONE,
                std::f32::consts::FRAC_PI_2,
                Vec2::X,
            ));
            layer.tint = Color::srgba(1.0, 0.84, 0.42, 0.1569);
            vec![layer]
        }
        Shape::StandardDrop => vec![mesh_layer(
            MeshKey::JarvanStandard,
            fit_mesh(
                MeshKey::JarvanStandard,
                Vec3::new(0.32, 2.2, 0.1),
                Vec3::Y * 1.1,
            ),
        )],
        Shape::WallBreak { radius } => wall_layers(radius, true),
        Shape::GroundImpact { radius } => vec![
            mesh_layer(
                MeshKey::JarvanImpactGround,
                fit_mesh(
                    MeshKey::JarvanImpactGround,
                    Vec3::new(radius * 2.0, 0.25, radius * 2.0),
                    Vec3::ZERO,
                ),
            ),
            Layer::new(
                Geometry::Quad,
                TextureKey::JarvanCracks,
                ground(Vec2::splat(radius * 2.0), 0.15),
            ),
        ],
        Shape::CrystalImpact { radius } => {
            let mut parts = vec![Layer::new(
                Geometry::Quad,
                TextureKey::AsheImpactFlash,
                ground(Vec2::splat(radius.max(0.8) * 1.2), 0.02),
            )];
            for index in 0..3 {
                let mut transform = ring_mesh(MeshKey::AsheImpactRing, radius);
                transform.rotation =
                    Quat::from_rotation_y(index as f32 * std::f32::consts::TAU / 3.0);
                parts.push(mesh_layer(MeshKey::AsheImpactRing, transform));
            }
            parts
        }
        Shape::Spark { gold, size } => {
            let mut layer = Layer::new(
                Geometry::Quad,
                if gold {
                    TextureKey::JarvanThrustStreak
                } else {
                    TextureKey::AsheImpactFlash
                },
                Transform::from_scale(Vec3::new(size, size, 1.0)),
            );
            if gold {
                layer.tint = Color::srgba(1.0, 0.82, 0.35, 0.8);
            }
            vec![layer]
        }
    };
    if !matches!(shape, Shape::StandardDrop) {
        for layer in &mut layers {
            layer.fading = true;
        }
    }
    layers
}

pub(super) fn live_layers(request: &Request) -> Vec<Layer> {
    match request.kind {
        LiveKind::Projectile if request.ability == 24 => {
            // Both source meshes share their particle-local pivot. Preserve
            // their relative head/tail offsets instead of recentering each.
            let size = mesh_spec(MeshKey::AsheCrystalArrow).bounds.size();
            let scale = (3.8 / size.z.max(0.01)).min(request.radius * 1.2 / size.x.max(0.01));
            [MeshKey::AsheCrystalArrow, MeshKey::AsheCrystalTrail]
                .into_iter()
                .map(|mesh| mesh_layer(mesh, Transform::from_scale(Vec3::splat(scale))))
                .collect()
        }
        LiveKind::Projectile => {
            let mut layers = Vec::new();
            for index in 0..request.arrows.max(1) {
                let side = index as f32 - (request.arrows.max(1) - 1) as f32 * 0.5;
                for (texture, size) in [
                    (
                        if request.ability == 21 {
                            TextureKey::AsheQArrow
                        } else {
                            TextureKey::AsheVolleyArrow
                        },
                        Vec2::new(1.15, 0.28),
                    ),
                    (TextureKey::AsheVolleyGlow, Vec2::new(1.4, 0.35)),
                ] {
                    let transform = directed_sprite(texture, size.y, size.x)
                        .with_translation(Vec3::new(side * 0.15, 0.005 * layers.len() as f32, 0.0));
                    layers.push(Layer::new(Geometry::Quad, texture, transform));
                }
            }
            layers
        }
        LiveKind::Area => vec![Layer::new(
            Geometry::Quad,
            TextureKey::IndicatorAoe,
            ground(Vec2::splat(request.radius * 2.0), 0.07),
        )],
        LiveKind::Standard => vec![
            mesh_layer(
                MeshKey::JarvanStandard,
                fit_mesh(
                    MeshKey::JarvanStandard,
                    Vec3::new(0.32, 2.2, 0.1),
                    Vec3::Y * 1.1,
                ),
            ),
            Layer::new(
                Geometry::Quad,
                TextureKey::JarvanFlag,
                Transform::from_xyz(0.26, 1.85, 0.02).with_scale(Vec3::new(0.6, 0.45, 1.0)),
            ),
        ],
        LiveKind::Terrain => wall_layers(request.radius, false),
        LiveKind::Scout => vec![mesh_layer(
            MeshKey::AsheScout,
            fit_mesh(
                MeshKey::AsheScout,
                Vec3::new(1.0, 0.36, 0.72),
                Vec3::Y * 1.8,
            ),
        )],
        LiveKind::Reveal(_) if request.ability == 24 => vec![Layer::new(
            Geometry::Quad,
            TextureKey::AsheImpactFlash,
            ground(Vec2::splat(0.7), 0.09),
        )],
        LiveKind::Reveal(destination) => vec![Layer::new(
            Geometry::Ring,
            TextureKey::AsheScanRing,
            ground(
                Vec2::splat(if destination { request.radius } else { 0.5 }),
                0.09,
            ),
        )],
        LiveKind::Shield => {
            let mut layer = Layer::new(
                Geometry::Quad,
                TextureKey::JarvanShield,
                // The source anchor stays world-vertical. This local +Z
                // places only the bubble surface toward the viewer, so it
                // does not cut through the opaque torso like a central slice.
                Transform::from_xyz(0.0, 0.0, 0.5).with_scale(Vec3::new(2.0, 2.35, 1.0)),
            );
            layer.tint = Color::srgba(1.0, 0.84, 0.42, 0.8);
            vec![layer]
        }
        LiveKind::Focus(empowered) => {
            let mut transform = fit_mesh(
                MeshKey::AsheQSymbol,
                Vec3::new(0.8, 0.08, 0.6),
                Vec3::new(0.0, 0.15, -0.5),
            );
            if empowered {
                transform.scale *= 1.2;
            }
            let mut layer = mesh_layer(MeshKey::AsheQSymbol, transform);
            layer.frame = Some(u8::from(empowered));
            vec![layer]
        }
    }
}

fn wall_layers(radius: f32, fading: bool) -> Vec<Layer> {
    (0..24)
        .map(|index| {
            let angle = index as f32 * std::f32::consts::TAU / 24.0;
            let rotation = Quat::from_rotation_y(-angle);
            let size = Vec3::new(0.55, 1.25 + (index % 3) as f32 * 0.08, radius * 0.27);
            let mut transform = fit_mesh(MeshKey::JarvanRock, size, Vec3::Y * size.y * 0.5);
            transform.translation = rotation * transform.translation
                + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
            transform.rotation = rotation;
            let mut layer = mesh_layer(MeshKey::JarvanRock, transform);
            layer.frame = Some((index % 4) as u8);
            layer.fading = fading;
            layer
        })
        .collect()
}
