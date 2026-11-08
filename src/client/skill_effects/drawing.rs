//! Atomic visual groups own every temporary material they allocate.
//! Callers choose layers and a lifetime; cleanup never relies on asset GC.
use super::{ManagedEffect, SkillAssets};
use crate::client::skill_assets::{MeshKey, TextureKey, texture_spec};
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    math::Affine2,
    prelude::*,
};

#[derive(Resource)]
struct GeometryAssets {
    quad: Handle<Mesh>,
    ring: Handle<Mesh>,
}
pub(super) fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(GeometryAssets {
        quad: meshes.add(Rectangle::new(1.0, 1.0)),
        ring: meshes.add(Annulus::new(0.87, 1.0)),
    });
}

struct Part {
    material: Handle<StandardMaterial>,
    tint: Color,
}
#[derive(Clone, Copy)]
pub(super) enum Geometry {
    Quad,
    Ring,
    Mesh(MeshKey),
}
pub(super) struct Layer {
    pub geometry: Geometry,
    pub texture: TextureKey,
    pub transform: Transform,
    pub tint: Color,
    pub frame: Option<u8>,
    pub uv_override: Option<Affine2>,
    pub fading: bool,
}
impl Layer {
    pub fn new(geometry: Geometry, texture: TextureKey, transform: Transform) -> Self {
        Self {
            geometry,
            texture,
            transform,
            tint: Color::WHITE,
            frame: None,
            uv_override: None,
            fading: false,
        }
    }
}

fn spawn_part(world: &mut World, root: Entity, layer: Layer) -> Option<Part> {
    let (mesh, template) = {
        let assets = world.get_resource::<SkillAssets>()?;
        assets.image(layer.texture)?;
        let mesh = match layer.geometry {
            Geometry::Mesh(key) => assets.mesh(key)?.clone(),
            Geometry::Quad => world.get_resource::<GeometryAssets>()?.quad.clone(),
            Geometry::Ring => world.get_resource::<GeometryAssets>()?.ring.clone(),
        };
        (mesh, assets.material(layer.texture)?.clone())
    };
    let mut material = world
        .resource::<Assets<StandardMaterial>>()
        .get(&template)?
        .clone();
    material.base_color = layer.tint;
    if layer.fading && material.alpha_mode == AlphaMode::Opaque {
        material.alpha_mode = AlphaMode::Blend;
    }
    if let Some(frame) = layer.frame {
        material.uv_transform = texture_spec(layer.texture).atlas.uv_transform(frame)?;
    }
    if let Some(uv) = layer.uv_override {
        material.uv_transform = uv;
    }
    let transparent = material.alpha_mode != AlphaMode::Opaque;
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(material);
    let mut entity = world.spawn((
        ManagedEffect,
        Mesh3d(mesh),
        MeshMaterial3d(material.clone()),
        layer.transform,
        Visibility::Inherited,
        ChildOf(root),
    ));
    if transparent {
        entity.insert((NotShadowCaster, NotShadowReceiver));
    }
    Some(Part {
        material,
        tint: layer.tint,
    })
}
pub(super) struct EffectGroup {
    root: Entity,
    parts: Vec<Part>,
}
impl EffectGroup {
    pub fn root(&self) -> Entity {
        self.root
    }
    pub fn layers(&self) -> usize {
        self.parts.len()
    }
    pub fn spawn(world: &mut World, parent: Option<Entity>, layers: Vec<Layer>) -> Option<Self> {
        if layers.is_empty() {
            return None;
        }
        let root = world
            .spawn((ManagedEffect, Transform::default(), Visibility::Inherited))
            .id();
        if let Some(parent) = parent {
            world.entity_mut(root).insert(ChildOf(parent));
        }
        let mut parts = Vec::new();
        for layer in layers {
            let Some(part) = spawn_part(world, root, layer) else {
                Self { root, parts }.dispose(world);
                return None;
            };
            parts.push(part);
        }
        Some(Self { root, parts })
    }
    pub fn dispose(self, world: &mut World) {
        // Linked carrier children may already be gone after death/reset.
        if let Ok(entity) = world.get_entity_mut(self.root) {
            entity.despawn();
        }
        for part in self.parts {
            world
                .resource_mut::<Assets<StandardMaterial>>()
                .remove(part.material.id());
        }
    }
    pub fn fade(&self, world: &mut World, opacity: f32) {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        for part in &self.parts {
            if let Some(mut material) = materials.get_mut(&part.material) {
                material.base_color = part
                    .tint
                    .with_alpha(part.tint.alpha() * opacity.clamp(0.0, 1.0));
            }
        }
    }
}
