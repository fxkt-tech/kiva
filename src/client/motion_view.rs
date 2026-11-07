//! Render-only motion facts use the same interpolation boundary as the root.
use super::presentation::PresentationFrame;
use crate::simulation::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub(super) struct MotionView {
    heights: HashMap<Entity, f32>,
    dashing: HashSet<Entity>,
}
impl MotionView {
    pub fn height(&self, unit: Entity) -> f32 {
        self.heights.get(&unit).copied().unwrap_or(0.0)
    }
    pub fn dashing(&self, unit: Entity) -> bool {
        self.dashing.contains(&unit)
    }
}
pub(super) fn sample(world: &mut World) {
    let frame = world.resource::<PresentationFrame>();
    if !frame.should_sync() {
        return;
    }
    let fraction = frame.fraction();
    let ids: Vec<_> = world
        .query_filtered::<Entity, With<Unit>>()
        .iter(world)
        .collect();
    let mut view = MotionView::default();
    for id in ids {
        if is_dashing(world, id) {
            view.dashing.insert(id);
        }
        if let Some(progress) = leap_progress(world, id, fraction) {
            view.heights
                .insert(id, (progress * std::f32::consts::PI).sin() * 2.2);
        }
    }
    world.insert_resource(view);
}
