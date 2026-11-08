//! Screen-space unit hits. Action adapters decide which candidate is legal.
use super::{camera::GameCamera, session::ClientSession, web};
use crate::simulation::{Unit, UnitState, team_can_see};
use bevy::{
    camera::primitives::Aabb,
    mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    prelude::*,
};
use std::cmp::Ordering;

/// Root profile used only while actual model mesh bounds are not ready.
#[derive(Component)]
pub(super) struct Pickable {
    pub offset: Vec3,
    pub half_size: Vec3,
}

/// Actual model geometry, excluding ground markers, effects and range previews.
#[derive(Component)]
pub(super) struct PickGeometry;

const EDGE_TOLERANCE: f32 = 6.0;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum HitKind {
    Model,
    Padding,
}

struct Hit {
    id: Entity,
    kind: HitKind,
    distance: f32,
    depth: f32,
}
impl Hit {
    fn compare(&self, other: &Self) -> Ordering {
        self.kind
            .cmp(&other.kind)
            .then_with(|| self.distance.total_cmp(&other.distance))
            .then_with(|| self.depth.total_cmp(&other.depth))
            .then_with(|| self.id.to_bits().cmp(&other.id.to_bits()))
    }
}

/// Input runs before fixed steps/transform propagation, so these transforms
/// are exactly the last displayed frame. No second geometry cache is needed.
pub(super) fn candidates(world: &mut World, point: Vec2) -> Vec<Entity> {
    if !point.is_finite() {
        return Vec::new();
    }
    let Some((camera, camera_transform)) = ({
        let mut query = world.query_filtered::<(&Camera, &GlobalTransform), With<GameCamera>>();
        query
            .single(world)
            .ok()
            .map(|(camera, transform)| (camera.clone(), *transform))
    }) else {
        return Vec::new();
    };
    let Some(viewport) = camera.logical_viewport_size() else {
        return Vec::new();
    };
    let projection = camera.clip_from_view();
    if !viewport.is_finite()
        || viewport.cmple(Vec2::ZERO).any()
        || !projection.is_finite()
        || projection.determinant() == 0.0
    {
        return Vec::new();
    }
    let view = PickingView {
        camera: &camera,
        transform: &camera_transform,
        viewport,
    };
    let team = world
        .get_resource::<ClientSession>()
        .and_then(|session| session.player())
        .and_then(|id| world.get::<Unit>(id))
        .map(Unit::team);
    let roots: Vec<_> = world
        .query_filtered::<Entity, (With<Pickable>, With<GlobalTransform>)>()
        .iter(world)
        .collect();
    let mut hits = Vec::new();
    for id in roots {
        if !world.get::<UnitState>(id).is_some_and(UnitState::alive)
            || world.get::<Unit>(id).is_none()
            || !visible(world, id)
            || team.is_some_and(|team| !team_can_see(world, team, id))
        {
            continue;
        }
        let root = world.get::<GlobalTransform>(id).unwrap();
        let mut best: Option<Hit> = None;
        let mut bounded_model = false;
        let mut stack = vec![(id, false, true)];
        while let Some((part, selected_geometry, parent_visible)) = stack.pop() {
            let shown = parent_visible && visible(world, part);
            let selected_geometry = selected_geometry || world.get::<PickGeometry>(part).is_some();
            if let Some(children) = world.get::<Children>(part) {
                stack.extend(
                    children
                        .iter()
                        .map(|child| (child, selected_geometry, shown)),
                );
            }
            if !selected_geometry || world.get::<Mesh3d>(part).is_none() {
                continue;
            }
            if !shown {
                bounded_model = true;
                continue;
            }
            if let Some(skin) = world.get::<SkinnedMesh>(part) {
                bounded_model = true;
                if let Some(hit) = skinned_hit(world, &view, id, part, skin, point)
                    && best.as_ref().is_none_or(|old| hit.compare(old).is_lt())
                {
                    best = Some(hit);
                }
                continue;
            }
            let Some(bounds) = world.get::<Aabb>(part) else {
                continue;
            };
            let Some(transform) = world.get::<GlobalTransform>(part) else {
                continue;
            };
            bounded_model = true;
            if let Some(hit) = view.part_hit(
                id,
                point,
                transform,
                bounds.center.into(),
                bounds.half_extents.into(),
            ) && best.as_ref().is_none_or(|old| hit.compare(old).is_lt())
            {
                best = Some(hit);
            }
        }
        // The shared profile covers the asset-ready transition and rendererless
        // actors. Existing mesh geometry is never replaced by one big root box.
        if !bounded_model {
            let profile = world.get::<Pickable>(id).unwrap();
            if let Some(hit) = view.part_hit(id, point, root, profile.offset, profile.half_size)
                && best.as_ref().is_none_or(|old| hit.compare(old).is_lt())
            {
                best = Some(hit);
            }
        }
        if let Some(hit) = best {
            hits.push(hit);
        }
    }
    hits.sort_by(Hit::compare);
    hits.into_iter().map(|hit| hit.id).collect()
}

/// Skinning moves vertices relative to their mesh node. Per-joint bounds retain
/// the same displayed-part contract as rigid limbs instead of making the empty
/// space between an extended weapon and a body selectable.
fn skinned_hit(
    world: &World,
    view: &PickingView<'_>,
    owner: Entity,
    part: Entity,
    skin: &SkinnedMesh,
    point: Vec2,
) -> Option<Hit> {
    let meshes = world.get_resource::<Assets<Mesh>>()?;
    let poses = world.get_resource::<Assets<SkinnedMeshInverseBindposes>>()?;
    let mesh = meshes.get(&world.get::<Mesh3d>(part)?.0)?;
    let inverse = poses.get(&skin.inverse_bindposes)?;
    let bounds = mesh.skinned_mesh_bounds()?;
    let mut best: Option<Hit> = None;
    for (index, bounds) in bounds.iter() {
        let index = usize::from(index.0);
        let Some(joint) = skin.joints.get(index) else {
            continue;
        };
        let Some(bind) = inverse.get(index) else {
            continue;
        };
        let Some(transform) = world.get::<GlobalTransform>(*joint) else {
            continue;
        };
        let matrix = transform.to_matrix() * *bind;
        if !matrix.is_finite() || matrix.determinant().abs() < 1e-12 {
            continue;
        }
        if let Some(hit) = view.part_hit(
            owner,
            point,
            &GlobalTransform::from(matrix),
            bounds.center,
            bounds.half_size,
        ) && best.as_ref().is_none_or(|old| hit.compare(old).is_lt())
        {
            best = Some(hit);
        }
    }
    best
}

fn visible(world: &World, entity: Entity) -> bool {
    world
        .get::<Visibility>(entity)
        .is_none_or(|visibility| *visibility != Visibility::Hidden)
        && world
            .get::<InheritedVisibility>(entity)
            .is_none_or(|visibility| visibility.get())
}

struct PickingView<'a> {
    camera: &'a Camera,
    transform: &'a GlobalTransform,
    viewport: Vec2,
}

impl PickingView<'_> {
    fn part_hit(
        &self,
        id: Entity,
        point: Vec2,
        transform: &GlobalTransform,
        center: Vec3,
        half: Vec3,
    ) -> Option<Hit> {
        let mut corners = Vec::with_capacity(8);
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let world = transform.transform_point(center + half * Vec3::new(x, y, z));
                    if let Ok(screen) = self.camera.world_to_viewport(self.transform, world) {
                        let point = web::viewport_to_css(screen, self.viewport);
                        if point.is_finite() {
                            corners.push(point);
                        }
                    }
                }
            }
        }
        let hull = convex_hull(corners);
        if hull.len() < 2 {
            return None;
        }
        let distance = polygon_distance(&hull, point);
        let depth = (transform.transform_point(center) - self.transform.translation())
            .dot(*self.transform.forward());
        (distance <= EDGE_TOLERANCE && depth.is_finite() && depth > 0.0).then_some(Hit {
            id,
            kind: if distance <= 0.001 {
                HitKind::Model
            } else {
                HitKind::Padding
            },
            distance,
            depth,
        })
    }
}

fn convex_hull(mut points: Vec<Vec2>) -> Vec<Vec2> {
    points.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
    points.dedup_by(|a, b| a.abs_diff_eq(*b, 0.0001));
    if points.len() < 3 {
        return points;
    }
    let half = |points: Vec<Vec2>| {
        let mut result: Vec<Vec2> = Vec::new();
        for point in points {
            while result.len() >= 2 {
                let a = result[result.len() - 2];
                let b = result[result.len() - 1];
                if (b - a).perp_dot(point - b) > 0.0 {
                    break;
                }
                result.pop();
            }
            result.push(point);
        }
        result.pop();
        result
    };
    let lower = half(points.clone());
    let upper = half(points.into_iter().rev().collect());
    lower.into_iter().chain(upper).collect()
}

fn polygon_distance(polygon: &[Vec2], point: Vec2) -> f32 {
    let mut inside = polygon.len() >= 3;
    let mut nearest = f32::INFINITY;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        let edge = b - a;
        inside &= edge.perp_dot(point - a) >= -0.0001;
        let fraction =
            ((point - a).dot(edge) / edge.length_squared().max(0.000001)).clamp(0.0, 1.0);
        nearest = nearest.min(point.distance(a + edge * fraction));
    }
    if inside { 0.0 } else { nearest }
}

#[cfg(test)]
mod tests;
