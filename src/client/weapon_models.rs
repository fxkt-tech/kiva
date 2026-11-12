//! Cached current-pose source weapons. Grip/firing axes and body mounts are Kiva adaptations.
use super::model_asset::{self, Reader};
use crate::simulation::{WeaponMode, WeaponPair};
use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mount {
    Weapon,
    Spine,
}

#[derive(Clone)]
struct ModelPart {
    name: String,
    mount: Mount,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Clone)]
pub(super) struct WeaponModel {
    /// Right then left hand, relative to the weapon origin (grip or firing source).
    pub grips: [Vec3; 2],
    parts: Vec<ModelPart>,
}

/// Decoded models by file; each file is decoded once.
#[derive(Resource, Default)]
struct WeaponModels {
    models: HashMap<&'static str, WeaponModel>,
}

pub(super) fn asset_path(pair: WeaponPair, mode: WeaponMode) -> &'static str {
    super::loadout::of(pair).model(mode)
}

impl WeaponModel {
    pub(super) fn shared(world: &mut World, pair: WeaponPair, mode: WeaponMode) -> Self {
        world.init_resource::<WeaponModels>();
        let path = asset_path(pair, mode);
        if let Some(model) = world.resource::<WeaponModels>().models.get(path) {
            return model.clone();
        }
        world.init_resource::<Assets<Image>>();
        let data = super::bundle::take(world, path);
        let mut reader = Reader(&data);
        assert_eq!(&reader.take::<8>(), b"KIVAWP01");
        let length = reader.u32();
        reader.bytes(length); // Offline provenance, without a runtime JSON dependency.
        let materials = model_asset::materials(world, &mut reader, true)
            .into_iter()
            .map(|material| {
                world
                    .resource_mut::<Assets<StandardMaterial>>()
                    .add(material)
            })
            .collect::<Vec<_>>();
        let grips = [reader.vec3(), reader.vec3()];
        let count = reader.u16();
        let mut parts = Vec::new();
        for _ in 0..count {
            let name = reader.string();
            let mount = match reader.u8() {
                0 => Mount::Weapon,
                1 => Mount::Spine,
                _ => panic!("verified weapon attachment"),
            };
            let primitives = reader.u16();
            for _ in 0..primitives {
                let material = materials[reader.u16()].clone();
                let vertex_count = reader.u32();
                let index_count = reader.u32();
                assert!(vertex_count > 0 && index_count > 0 && index_count.is_multiple_of(3));
                assert!(vertex_count <= reader.0.len() / 32);
                let (mut positions, mut normals, mut uvs) = (Vec::new(), Vec::new(), Vec::new());
                for _ in 0..vertex_count {
                    positions.push(reader.vec3().to_array());
                    let normal = reader.vec3();
                    assert!((normal.length() - 1.).abs() < 0.01);
                    normals.push(normal.to_array());
                    uvs.push([reader.f32(), reader.f32()]);
                }
                assert!(index_count <= reader.0.len() / 4);
                let indices = (0..index_count)
                    .map(|_| {
                        let index = reader.u32();
                        assert!(index < vertex_count);
                        index as u32
                    })
                    .collect::<Vec<_>>();
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                mesh.insert_indices(Indices::U32(indices));
                parts.push(ModelPart {
                    name: name.clone(),
                    mount,
                    mesh: world.resource_mut::<Assets<Mesh>>().add(mesh),
                    material,
                });
            }
        }
        assert!(reader.0.is_empty());
        let model = Self { grips, parts };
        world
            .resource_mut::<WeaponModels>()
            .models
            .insert(path, model.clone());
        model
    }
}

#[cfg(test)]
impl WeaponModel {
    /// Vertices of the hand-held parts in the grip frame (+Y from the grip).
    pub(super) fn vertices(&self, meshes: &Assets<Mesh>) -> Vec<Vec3> {
        self.parts
            .iter()
            .filter(|part| part.mount == Mount::Weapon)
            .flat_map(|part| {
                meshes
                    .get(&part.mesh)
                    .and_then(|mesh| mesh.attribute(Mesh::ATTRIBUTE_POSITION))
                    .and_then(|positions| positions.as_float3())
                    .expect("weapon mesh keeps CPU positions")
                    .iter()
                    .map(|position| Vec3::from(*position))
            })
            .collect()
    }
}

pub(super) fn spawn(
    world: &mut World,
    parent: Entity,
    pair: WeaponPair,
    mode: WeaponMode,
    mount: Mount,
) {
    let model = WeaponModel::shared(world, pair, mode);
    for part in model.parts.iter().filter(|part| part.mount == mount) {
        let entity = world
            .spawn((
                Name::new(part.name.clone()),
                Mesh3d(part.mesh.clone()),
                MeshMaterial3d(part.material.clone()),
                Transform::IDENTITY,
            ))
            .id();
        world.entity_mut(parent).add_child(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::TextureFormat;

    #[test]
    fn source_weapons_keep_textured_geometry_separate_body_mounts_and_cache_handles() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            let model = WeaponModel::shared(&mut world, pair, WeaponMode::Ranged);
            let parts = model
                .parts
                .iter()
                .filter(|part| part.mount == Mount::Weapon)
                .collect::<Vec<_>>();
            assert!(
                parts.len()
                    >= if pair == WeaponPair::HammerMusket {
                        2
                    } else {
                        1
                    },
                "all source weapon renderers"
            );
            let mut vertex_count = 0;
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            for part in &parts {
                let mesh = world.resource::<Assets<Mesh>>().get(&part.mesh).unwrap();
                assert!(
                    mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX).is_none(),
                    "source current-pose LBS is baked"
                );
                let positions = mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                vertex_count += positions.len();
                for p in positions {
                    let p = Vec3::from(*p);
                    assert!(p.is_finite());
                    min = min.min(p);
                    max = max.max(p);
                }
                let material = world
                    .resource::<Assets<StandardMaterial>>()
                    .get(&part.material)
                    .unwrap();
                assert!(material.normal_map_texture.is_none());
                assert!(material.metallic_roughness_texture.is_none());
            }
            assert!(
                vertex_count > 1000,
                "real source meshes, not replacement primitives"
            );
            assert!((max - min).max_element() > 1.2 && (max - min).max_element() < 1.8);
            if pair == WeaponPair::HammerMusket {
                assert!(
                    min.z.abs() < 0.03 && max.z > 1.2,
                    "barrel exit sits at the source and stock is behind it"
                );
                assert!(model.parts.iter().all(|part| part.mount == Mount::Weapon));
            } else {
                assert!(
                    model
                        .parts
                        .iter()
                        .filter(|part| part.mount == Mount::Spine)
                        .count()
                        >= 2,
                    "configured quiver and arrowbag follow the body"
                );
                assert!(
                    max.y > 0.7 && min.y < -0.7,
                    "original bow limbs are vertical"
                );
            }
            assert!(model.parts.iter().any(|part| {
                world
                    .resource::<Assets<StandardMaterial>>()
                    .get(&part.material)
                    .unwrap()
                    .base_color_texture
                    .is_some()
            }));
            let counts = (
                world.resource::<Assets<Mesh>>().len(),
                world.resource::<Assets<StandardMaterial>>().len(),
                world.resource::<Assets<Image>>().len(),
            );
            let reused = WeaponModel::shared(&mut world, pair, WeaponMode::Ranged);
            assert_eq!(model.parts[0].mesh, reused.parts[0].mesh);
            assert_eq!(model.parts[0].material, reused.parts[0].material);
            assert_eq!(
                (
                    world.resource::<Assets<Mesh>>().len(),
                    world.resource::<Assets<StandardMaterial>>().len(),
                    world.resource::<Assets<Image>>().len()
                ),
                counts
            );
        }
        assert!(
            world
                .resource::<Assets<Image>>()
                .iter()
                .all(|(_, image)| image.texture_descriptor.format == TextureFormat::Rgba8UnormSrgb)
        );
    }

    #[test]
    fn melee_models_keep_source_geometry_grip_frame_base_color_and_four_model_cache() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            let model = WeaponModel::shared(&mut world, pair, WeaponMode::Melee);
            assert_eq!(model.grips[0], Vec3::ZERO);
            assert!(model.parts.iter().all(|part| part.mount == Mount::Weapon));
            let mut minimum = Vec3::splat(f32::INFINITY);
            let mut maximum = Vec3::splat(f32::NEG_INFINITY);
            let mut vertices = 0;
            for part in &model.parts {
                let mesh = world.resource::<Assets<Mesh>>().get(&part.mesh).unwrap();
                assert!(mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX).is_none());
                let positions = mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                vertices += positions.len();
                for position in positions {
                    minimum = minimum.min(Vec3::from(*position));
                    maximum = maximum.max(Vec3::from(*position));
                }
                let material = world
                    .resource::<Assets<StandardMaterial>>()
                    .get(&part.material)
                    .unwrap();
                assert!(material.base_color_texture.is_some());
                assert!(material.normal_map_texture.is_none());
                assert!(material.metallic_roughness_texture.is_none());
            }
            assert!(vertices > 1000, "original meshes replace temporary cubes");
            assert!(
                minimum.y < 0. && maximum.y > 0.5,
                "grip lies in the handle, head extends +Y"
            );
            let size = maximum - minimum;
            assert!(size.y > size.x && size.y > size.z);
            assert!(
                (size.y - 1.10).abs() < 0.0001,
                "contract: 1.10m melee display length"
            );
            if pair == WeaponPair::HammerMusket {
                assert_eq!(
                    vertices, 5437,
                    "main and implicit-weight attachment both remain"
                );
                assert_eq!(model.parts.len(), 2);
            } else {
                assert_eq!(
                    vertices, 17504,
                    "main plus complementary blade/back/head/handle regions"
                );
                assert_eq!(model.parts.len(), 5, "independent FX draws stay excluded");
            }
            let parent = world.spawn(Transform::IDENTITY).id();
            spawn(&mut world, parent, pair, WeaponMode::Melee, Mount::Weapon);
            for (child, part) in world
                .get::<Children>(parent)
                .unwrap()
                .iter()
                .zip(&model.parts)
            {
                assert_eq!(world.get::<Mesh3d>(child).unwrap().0, part.mesh);
                assert_eq!(
                    world
                        .get::<MeshMaterial3d<StandardMaterial>>(child)
                        .unwrap()
                        .0,
                    part.material
                );
                assert_eq!(*world.get::<Transform>(child).unwrap(), Transform::IDENTITY);
            }
            WeaponModel::shared(&mut world, pair, WeaponMode::Ranged);
        }
        let counts = (
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<StandardMaterial>>().len(),
            world.resource::<Assets<Image>>().len(),
        );
        for _ in 0..3 {
            for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
                for mode in [WeaponMode::Melee, WeaponMode::Ranged] {
                    WeaponModel::shared(&mut world, pair, mode);
                }
            }
        }
        assert_eq!(
            (
                world.resource::<Assets<Mesh>>().len(),
                world.resource::<Assets<StandardMaterial>>().len(),
                world.resource::<Assets<Image>>().len()
            ),
            counts
        );
        assert!(
            world
                .resource::<Assets<Image>>()
                .iter()
                .all(|(_, image)| image.texture_descriptor.format == TextureFormat::Rgba8UnormSrgb)
        );
    }
}
