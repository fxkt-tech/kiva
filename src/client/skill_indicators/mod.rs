//! Local aiming projection. It never owns a command, combat timer or hit test.
mod geometry;
mod render;

use super::{
    abilities,
    ability_targeting::{self, TargetState},
    camera::GameCamera,
    controls::{PlayerController, PointerState},
    picking,
    scene::UnitVisual,
    session::ClientSession,
    web,
};
use crate::simulation::*;
use bevy::{camera::visibility::VisibilitySystems, prelude::*};
use serde::Serialize;

pub(super) struct SkillIndicatorsPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum SkillIndicatorsSet {
    Project,
}

impl Plugin for SkillIndicatorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IndicatorView>()
            .add_systems(Startup, render::setup)
            .add_systems(Update, render::prepare_fans)
            .add_systems(
                PostUpdate,
                update
                    .in_set(SkillIndicatorsSet::Project)
                    .after(super::camera::update)
                    .after(bevy::transform::TransformSystems::Propagate)
                    .after(bevy::camera::CameraUpdateSystems)
                    .before(VisibilitySystems::VisibilityPropagate)
                    .before(VisibilitySystems::CalculateBounds)
                    .before(VisibilitySystems::CheckVisibility),
            );
    }
}

#[derive(Resource, Clone, Default, Serialize)]
#[serde(transparent)]
pub(super) struct IndicatorView(pub Option<IndicatorSnapshot>);

#[derive(Clone, Serialize)]
pub(super) struct IndicatorSnapshot {
    pub ability_id: u16,
    pub shape: &'static str,
    pub status: &'static str,
    pub rejection: Option<String>,
    pub origin: [f32; 2],
    pub target: Option<[f32; 2]>,
    pub cursor_ground: Option<[f32; 2]>,
    pub target_id: Option<String>,
    pub range: f32,
    pub width: Option<f32>,
    pub radius: Option<f32>,
    pub rays: u8,
    pub angle: f32,
    pub textured: bool,
}

struct IndicatorFrame {
    view: IndicatorSnapshot,
    geometry: geometry::Geometry,
}

fn ground(camera: &Camera, transform: &GlobalTransform, css: Vec2) -> Option<Vec2> {
    let size = camera.logical_viewport_size()?;
    if !css.is_finite() || !size.is_finite() || size.cmple(Vec2::ZERO).any() {
        return None;
    }
    let ray = camera
        .viewport_to_world(transform, web::css_to_viewport(css, size))
        .ok()?;
    let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    let point = ray.get_point(distance);
    let point = Vec2::new(point.x, point.z);
    point.is_finite().then_some(point)
}

fn project(world: &mut World) -> Option<IndicatorFrame> {
    // A focused HUD button can leave the canvas' Window unfocused on Web.
    // Match the controls adapter's document-level focus contract.
    let focused = web::document_focused().unwrap_or_else(|| {
        !world
            .query::<&Window>()
            .iter(world)
            .any(|window| !window.focused)
    });
    if world.resource::<SimulationControl>().paused
        || !focused
        || web::pointer_captured() == Some(false)
    {
        return None;
    }
    let player = world.resource::<ClientSession>().player()?;
    if !world.get::<UnitState>(player)?.alive() {
        return None;
    }
    let source = world.resource::<PlayerController>().aim?.source()?;
    let pointer = world.resource::<PointerState>();
    let css = pointer.position?;
    if pointer.over_ui || web::pointer_over_ui(css) {
        return None;
    }
    let (camera, camera_transform) = world
        .query_filtered::<(&Camera, &GlobalTransform), With<GameCamera>>()
        .single(world)
        .ok()
        .map(|(camera, transform)| (camera.clone(), *transform))?;
    // The input ground sample belongs to the prior displayed camera. Reproject
    // only for this drawing frame; keep the single pointer owner untouched.
    let cursor_ground = ground(&camera, &camera_transform, css);
    let targeting = abilities::resolve(
        source,
        world.get::<AbilitySlots>(player)?,
        world.get::<EquipmentSlots>(player)?,
        world.resource::<AbilityCatalog>(),
        world.resource::<EquipmentCatalog>(),
    )?
    .definition
    .targeting;
    let candidates = if targeting == Targeting::Enemy {
        picking::candidates(world, css)
    } else {
        Vec::new()
    };
    let intent =
        ability_targeting::resolve_intent(world, player, source, cursor_ground, &candidates)?;
    let (status, rejection) = match &intent.state {
        TargetState::Ready => ("ready", None),
        TargetState::ApproachingEnemy => ("approaching", None),
        TargetState::Invalid(reason) => ("invalid", Some(format!("{reason:?}"))),
        TargetState::NoTarget => ("no-target", None),
    };
    let origin = world
        .get::<GlobalTransform>(player)
        .filter(|_| world.get::<UnitVisual>(player).is_some())?
        .translation();
    let origin = Vec2::new(origin.x, origin.z);
    if !origin.is_finite() {
        return None;
    }
    let target = match intent.resolved.or(intent.original) {
        Some(AbilityTarget::Unit(id)) => Some(id),
        _ => None,
    };
    let target_position = target.and_then(|id| {
        world.get::<GlobalTransform>(id).map(|transform| {
            let position = transform.translation();
            Vec2::new(position.x, position.z)
        })
    });
    let definition = world.resource::<AbilityCatalog>().get(intent.ability)?;
    let geometry = geometry::derive(definition, &intent, origin, target_position);
    Some(IndicatorFrame {
        view: IndicatorSnapshot {
            ability_id: intent.ability.0,
            shape: geometry.shape,
            status,
            rejection,
            origin: origin.to_array(),
            target: geometry.target.map(|point| point.to_array()),
            cursor_ground: cursor_ground.map(|point| point.to_array()),
            target_id: target.map(|id| id.to_bits().to_string()),
            range: geometry.range,
            width: geometry.width,
            radius: geometry.radius,
            rays: geometry.rays,
            angle: geometry.angle,
            textured: false,
        },
        geometry,
    })
}

fn update(world: &mut World) {
    let mut frame = project(world);
    let textured = render::draw(world, frame.as_ref());
    if let Some(frame) = &mut frame {
        frame.view.textured = textured;
    }
    world.resource_mut::<IndicatorView>().0 = frame.map(|frame| frame.view);
}

#[cfg(test)]
mod tests;
