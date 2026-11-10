//! Fixed reusable mesh lines. No gizmo renderer or per-frame asset creation.
use super::session::DebugOptions;
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct DebugDraw {
    lines: Vec<Entity>,
    materials: Vec<Handle<StandardMaterial>>,
}
#[derive(Clone, Copy)]
pub(super) struct Line {
    pub from: Vec3,
    pub to: Vec3,
    pub color: usize,
}
impl DebugDraw {
    pub fn new(world: &mut World) -> Self {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::new(1., 1., 1.));
        let colors = [
            Color::srgb(0.28, 0.33, 0.39),
            Color::srgb(1., 0.22, 0.2),
            Color::srgb(0.26, 0.9, 0.45),
            Color::srgb(0.22, 0.6, 1.),
            Color::srgb(1., 0.78, 0.15),
            Color::srgb(0.7, 0.4, 1.),
            Color::srgb(0.2, 0.95, 0.95),
        ];
        let materials = colors
            .into_iter()
            .map(|color| {
                world
                    .resource_mut::<Assets<StandardMaterial>>()
                    .add(StandardMaterial {
                        base_color: color,
                        unlit: true,
                        ..default()
                    })
            })
            .collect::<Vec<_>>();
        let lines = (0..192)
            .map(|_| {
                world
                    .spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(materials[0].clone()),
                        Transform::IDENTITY,
                        Visibility::Hidden,
                    ))
                    .id()
            })
            .collect();
        Self { lines, materials }
    }
    pub fn submit(&self, world: &mut World, lines: &[Line]) {
        for (index, &entity) in self.lines.iter().enumerate() {
            if let Some(line) = lines.get(index) {
                let delta = line.to - line.from;
                let length = delta.length();
                if !length.is_finite() || length < 0.00001 {
                    world
                        .entity_mut(entity)
                        .insert((Visibility::Hidden, InheritedVisibility::HIDDEN));
                    continue;
                }
                let transform = Transform::from_translation((line.from + line.to) * 0.5)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, delta / length))
                    .with_scale(Vec3::new(
                        if line.color == 0 { 0.002 } else { 0.006 },
                        length,
                        if line.color == 0 { 0.002 } else { 0.006 },
                    ));
                world.entity_mut(entity).insert((
                    transform,
                    GlobalTransform::from(transform),
                    Visibility::Visible,
                    InheritedVisibility::VISIBLE,
                    MeshMaterial3d(
                        self.materials[line.color.min(self.materials.len() - 1)].clone(),
                    ),
                ));
            } else {
                world
                    .entity_mut(entity)
                    .insert((Visibility::Hidden, InheritedVisibility::HIDDEN));
            }
        }
    }
}
pub(super) fn base(options: DebugOptions) -> Vec<Line> {
    let mut lines = Vec::new();
    if options.grid {
        for index in -10..=10 {
            let v = index as f32 * 0.25;
            lines.push(Line {
                from: Vec3::new(-2.5, 0.004, v),
                to: Vec3::new(2.5, 0.004, v),
                color: 0,
            });
            lines.push(Line {
                from: Vec3::new(v, 0.004, -2.5),
                to: Vec3::new(v, 0.004, 2.5),
                color: 0,
            });
        }
    }
    if options.axes {
        axes(&mut lines, Transform::IDENTITY, 0.45);
    }
    lines
}
pub(super) fn axes(lines: &mut Vec<Line>, frame: Transform, size: f32) {
    for (axis, color) in [(Vec3::X, 1), (Vec3::Y, 2), (Vec3::Z, 3)] {
        lines.push(Line {
            from: frame.translation,
            to: frame.translation + frame.rotation * axis * size,
            color,
        });
    }
}
pub(super) fn cross(lines: &mut Vec<Line>, position: Vec3, color: usize) {
    for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
        lines.push(Line {
            from: position - axis * 0.025,
            to: position + axis * 0.025,
            color,
        });
    }
}
pub(super) fn bounds(lines: &mut Vec<Line>, min: Vec3, max: Vec3) {
    for axis in 0..3 {
        for a in 0..2 {
            for b in 0..2 {
                let mut from = min;
                let mut to = min;
                let first = (axis + 1) % 3;
                let second = (axis + 2) % 3;
                from[first] = if a == 0 { min[first] } else { max[first] };
                to[first] = from[first];
                from[second] = if b == 0 { min[second] } else { max[second] };
                to[second] = from[second];
                to[axis] = max[axis];
                lines.push(Line { from, to, color: 6 });
            }
        }
    }
}
