//! A small reusable flat-mesh pool, updated after the displayed camera settles.
use super::{IndicatorFrame, geometry::Geometry};
use crate::client::skill_assets::{SkillAssets, TextureKey};
use crate::simulation::{AbilityCatalog, AbilityExecution};
use bevy::{
    asset::RenderAssetUsages,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::Indices,
    prelude::*,
    render::render_resource::PrimitiveTopology,
};
use std::collections::HashMap;

#[derive(Component)]
pub(super) struct IndicatorPart;

#[derive(Resource)]
pub(super) struct Drawing {
    plane: Handle<Mesh>,
    ring: Handle<Mesh>,
    fan: HashMap<u32, Handle<Mesh>>,
    ready: Handle<StandardMaterial>,
    approaching: Handle<StandardMaterial>,
    invalid: Handle<StandardMaterial>,
    pool: Vec<Entity>,
}

pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut material = |color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            alpha_mode: AlphaMode::Add,
            ..default()
        })
    };
    commands.insert_resource(Drawing {
        plane: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
        ring: meshes.add(Annulus::new(0.985, 1.0).mesh().resolution(96)),
        fan: default(),
        ready: material(Color::srgba(0.18, 0.8, 0.78, 0.65)),
        approaching: material(Color::srgba(1.0, 0.64, 0.12, 0.85)),
        invalid: material(Color::srgba(1.0, 0.14, 0.11, 0.85)),
        pool: Vec::new(),
    });
}

/// Asset events flush in PostUpdate. Prepare execution-specific geometry in
/// Update so the very first aiming frame can extract it into the render world.
pub(super) fn prepare_fans(
    catalog: Res<AbilityCatalog>,
    mut drawing: ResMut<Drawing>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !catalog.is_changed() {
        return;
    }
    for (_, definition) in catalog.iter() {
        if let AbilityExecution::Volley(spec) = &definition.execution {
            drawing
                .fan
                .entry(spec.angle.to_bits())
                .or_insert_with(|| meshes.add(fan(spec.angle)));
        }
    }
}

enum Surface {
    Texture(TextureKey),
    Ring,
    Edge,
    Fan(f32),
}
struct Piece {
    surface: Surface,
    transform: Transform,
}

fn plane(point: Vec2, width: f32, length: f32, direction: Vec2, height: f32) -> Transform {
    Transform::from_xyz(point.x, height, point.y)
        .with_rotation(Quat::from_rotation_y((-direction.x).atan2(-direction.y)))
        .with_scale(Vec3::new(width.max(0.001), 1.0, length.max(0.001)))
}

fn circle(pieces: &mut Vec<Piece>, point: Vec2, radius: f32, texture: TextureKey) {
    if radius <= 0.0 {
        return;
    }
    pieces.push(Piece {
        surface: Surface::Texture(texture),
        transform: plane(point, radius * 2.0, radius * 2.0, Vec2::NEG_Y, 0.07),
    });
    pieces.push(Piece {
        surface: Surface::Ring,
        transform: Transform::from_xyz(point.x, 0.075, point.y)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
    });
}

fn edge(pieces: &mut Vec<Piece>, start: Vec2, end: Vec2) {
    let delta = end - start;
    if let Some(direction) = delta.try_normalize() {
        pieces.push(Piece {
            surface: Surface::Edge,
            transform: plane((start + end) * 0.5, 0.035, delta.length(), direction, 0.08),
        });
    }
}

fn ray(pieces: &mut Vec<Piece>, origin: Vec2, direction: Vec2, length: f32, width: f32) {
    let head = (width * 1.4).clamp(0.25, length.max(0.25)).min(length);
    if length > head {
        pieces.push(Piece {
            surface: Surface::Texture(TextureKey::IndicatorLineBody),
            transform: plane(
                origin + direction * ((length - head) * 0.5),
                width,
                length - head,
                direction,
                0.08,
            ),
        });
    }
    pieces.push(Piece {
        surface: Surface::Texture(TextureKey::IndicatorLineHead),
        transform: plane(
            origin + direction * (length - head * 0.5),
            width,
            head,
            direction,
            0.085,
        ),
    });
    let side = Vec2::new(-direction.y, direction.x) * width * 0.5;
    edge(pieces, origin + side, origin + direction * length + side);
    edge(pieces, origin - side, origin + direction * length - side);
}

fn pieces(geometry: &Geometry, no_target: bool) -> Vec<Piece> {
    let mut result = Vec::new();
    circle(
        &mut result,
        geometry.origin,
        geometry.range,
        TextureKey::IndicatorRange,
    );
    if no_target {
        return result;
    }
    if geometry.shape == "volley" {
        result.push(Piece {
            surface: Surface::Fan(geometry.angle),
            transform: plane(
                geometry.origin,
                geometry.range,
                geometry.range,
                geometry.direction,
                0.065,
            ),
        });
        for direction in geometry.directions() {
            ray(
                &mut result,
                geometry.origin,
                direction,
                geometry.range,
                geometry.width.unwrap_or(0.1),
            );
        }
    } else if let Some(width) = geometry.width {
        ray(
            &mut result,
            geometry.origin,
            geometry.direction,
            geometry.range,
            width,
        );
    } else if let Some(target) = geometry.target {
        if matches!(geometry.shape, "scout" | "dash") {
            edge(&mut result, geometry.origin, target);
        }
        if let Some(radius) = geometry.radius {
            let texture = if geometry.shape == "leap" {
                TextureKey::IndicatorTarget
            } else {
                TextureKey::IndicatorAoe
            };
            circle(&mut result, target, radius, texture);
        }
        circle(&mut result, target, 0.3, TextureKey::IndicatorTarget);
    }
    result
}

/// Mesh geometry, not the source fan picture, owns the cone angle. UVs remap
/// the reviewed ~60-degree source wedge onto the actual execution's sector.
fn fan(angle: f32) -> Mesh {
    let mut positions = vec![[0.0, 0.0, 0.0]];
    let mut uvs = vec![[0.5, 0.98]];
    let mut indices = Vec::new();
    let steps = 40;
    for index in 0..=steps {
        let fraction = index as f32 / steps as f32;
        let theta = (fraction - 0.5) * angle;
        positions.push([theta.sin(), 0.0, -theta.cos()]);
        let authored = (fraction - 0.5) * 60.0_f32.to_radians();
        uvs.push([0.5 + authored.sin() * 0.84, 0.98 - authored.cos() * 0.96]);
        if index > 0 {
            indices.extend([0, index as u32 + 1, index as u32]);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

pub(super) fn draw(world: &mut World, frame: Option<&IndicatorFrame>) -> bool {
    world.resource_scope(|world, mut drawing: Mut<Drawing>| {
        let Some(frame) = frame else {
            for &id in &drawing.pool {
                if let Some(mut shown) = world.get_mut::<Visibility>(id) {
                    *shown = Visibility::Hidden;
                }
            }
            return false;
        };
        let status = match frame.view.status {
            "invalid" => drawing.invalid.clone(),
            "approaching" => drawing.approaching.clone(),
            _ => drawing.ready.clone(),
        };
        let mut pieces = pieces(&frame.geometry, frame.view.status == "no-target");
        let mut textured = true;
        pieces.retain(|piece| {
            let prepared = match &piece.surface {
                Surface::Fan(angle) => drawing.fan.contains_key(&angle.to_bits()),
                _ => true,
            };
            textured &= prepared;
            prepared
        });
        let used = pieces.len();
        for (index, piece) in pieces.into_iter().enumerate() {
            let (mesh, texture) = match piece.surface {
                Surface::Texture(key) => (drawing.plane.clone(), Some(key)),
                Surface::Ring => (drawing.ring.clone(), None),
                Surface::Edge => (drawing.plane.clone(), None),
                Surface::Fan(angle) => {
                    let mesh = drawing
                        .fan
                        .get(&angle.to_bits())
                        .expect("only prepared fan pieces are retained");
                    (mesh.clone(), Some(TextureKey::IndicatorFan))
                }
            };
            let material = texture
                .and_then(|key| world.get_resource::<SkillAssets>()?.material(key))
                .cloned();
            textured &= texture.is_none() || material.is_some();
            let shown = material.is_some()
                || !matches!(
                    texture,
                    Some(
                        TextureKey::IndicatorAoe
                            | TextureKey::IndicatorTarget
                            | TextureKey::IndicatorRange
                    )
                );
            let material = material.unwrap_or_else(|| status.clone());
            let id = if let Some(&id) = drawing.pool.get(index) {
                id
            } else {
                let id = world
                    .spawn((IndicatorPart, NotShadowCaster, NotShadowReceiver))
                    .id();
                drawing.pool.push(id);
                id
            };
            world.entity_mut(id).insert((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                piece.transform,
                GlobalTransform::from(piece.transform),
                if shown {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ));
        }
        // Shapes have different part counts. Hide the remainder this frame,
        // rather than keeping the previous cone/target after changing slots.
        for &id in drawing.pool.iter().skip(used) {
            if let Some(mut shown) = world.get_mut::<Visibility>(id) {
                *shown = Visibility::Hidden;
            }
        }
        textured
    })
}

#[cfg(test)]
pub(super) fn cached_fan(world: &World, angle: f32) -> Option<&Handle<Mesh>> {
    world.resource::<Drawing>().fan.get(&angle.to_bits())
}
