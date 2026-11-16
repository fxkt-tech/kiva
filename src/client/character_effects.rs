//! Explicitly tagged fire surfaces retain ordinary PBR/skinning and animate only
//! their own UVs/emission. Opaque cloth and static hair-tip glow are unaffected.
use std::collections::HashMap;

use bevy::{
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
};

use crate::simulation::SimulationActive;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SurfaceEffect {
    #[default]
    None,
    Ember,
}

impl SurfaceEffect {
    pub(super) fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Ember,
            _ => panic!("invalid character surface effect {value}"),
        }
    }
}

struct EmberMaterial {
    handle: Handle<StandardMaterial>,
    uv: Affine2,
    emissive: LinearRgba,
    phase: Option<f32>,
}

#[derive(Resource, Default)]
pub(super) struct CharacterEffects {
    materials: HashMap<AssetId<StandardMaterial>, EmberMaterial>,
    game_seconds: f64,
}

/// Capture each cached variant's authored baseline once, before any animation.
/// No appearance names, shader replacements, or per-frame asset creation.
pub(super) fn register(
    world: &mut World,
    handle: &Handle<StandardMaterial>,
    effect: SurfaceEffect,
) {
    if effect == SurfaceEffect::None {
        return;
    }
    world.init_resource::<CharacterEffects>();
    if world
        .resource::<CharacterEffects>()
        .materials
        .contains_key(&handle.id())
    {
        return;
    }
    let material = world
        .resource::<Assets<StandardMaterial>>()
        .get(handle)
        .expect("registered character effect material must exist");
    assert_eq!(
        material.alpha_mode,
        AlphaMode::Blend,
        "ember surfaces require alpha blending"
    );
    let entry = EmberMaterial {
        handle: handle.clone(),
        uv: material.uv_transform,
        emissive: material.emissive,
        phase: None,
    };
    // Fire atlases are dedicated to the tagged fringe. Preserve authored
    // filtering while allowing flow along V; U must not wrap across card edges.
    let textures = [
        material.base_color_texture.clone(),
        material.emissive_texture.clone(),
    ];
    for texture in textures.into_iter().flatten() {
        let mut images = world.resource_mut::<Assets<Image>>();
        let mut image = images
            .get_mut(&texture)
            .expect("registered ember texture must exist");
        let mut sampler = match &image.sampler {
            ImageSampler::Descriptor(sampler) => sampler.clone(),
            ImageSampler::Default => ImageSamplerDescriptor::linear(),
        };
        sampler.address_mode_u = ImageAddressMode::ClampToEdge;
        sampler.address_mode_v = ImageAddressMode::Repeat;
        image.sampler = ImageSampler::Descriptor(sampler);
    }
    world
        .resource_mut::<CharacterEffects>()
        .materials
        .insert(handle.id(), entry);
}

impl CharacterEffects {
    fn apply(&mut self, materials: &mut Assets<StandardMaterial>, seconds: f64) {
        if !seconds.is_finite() || seconds < 0.0 {
            return;
        }
        // Every oscillator and the repeating texture agree at this boundary;
        // bounded phase also preserves precision during long-running sessions.
        let phase = seconds.rem_euclid(8.0) as f32;
        let angle = std::f32::consts::TAU * phase / 8.0;
        let gain = 0.90 + 0.08 * (9.0 * angle).sin() + 0.04 * (21.0 * angle).sin();
        for entry in self.materials.values_mut() {
            if entry.phase == Some(phase) {
                continue;
            }
            let Some(mut material) = materials.get_mut(&entry.handle) else {
                continue;
            };
            // Export flips authored V once: root=1, tail=0 in runtime UVs.
            // Positive sampling offset moves visible features toward lower V,
            // carrying the fire outward from cape root to feather tips.
            material.uv_transform =
                entry.uv * Affine2::from_translation(Vec2::new(0.0, 0.5 * phase));
            material.emissive = LinearRgba::new(
                entry.emissive.red * gain,
                entry.emissive.green * gain,
                entry.emissive.blue * gain,
                entry.emissive.alpha,
            );
            entry.phase = Some(phase);
        }
    }
}

/// Game presentation accumulates active frame deltas, so focus loss/Esc never
/// advances the fire or causes a catch-up jump when the simulation resumes.
pub(super) fn update_game(
    time: Res<Time>,
    active: Res<SimulationActive>,
    effects: Option<ResMut<CharacterEffects>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(mut effects) = effects else { return };
    if !active.0 {
        return;
    }
    effects.game_seconds += time.delta_secs_f64();
    let seconds = effects.game_seconds;
    effects.apply(&mut materials, seconds);
}

/// The model viewer owns playback time. An absolute sample handles paused
/// seeks, loop wraps and repeated frames without accumulating animation drift.
pub(super) fn apply_at(world: &mut World, seconds: f64) {
    if !world.contains_resource::<CharacterEffects>() {
        return;
    }
    world.resource_scope(|world, mut effects: Mut<CharacterEffects>| {
        effects.apply(
            &mut world.resource_mut::<Assets<StandardMaterial>>(),
            seconds,
        );
    });
}

#[cfg(test)]
#[path = "character_effects_tests.rs"]
mod tests;
