//! Current-pose source weapon meshes. Firing axes and body mounts are Kiva adaptations.
use super::model_asset::{self, Reader};
use crate::simulation::WeaponPair;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mount {
    Muzzle,
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
pub(super) struct RangedModel {
    /// Right hand, then left hand, in the authoritative firing-source frame.
    pub grips: [Vec3; 2],
    parts: Vec<ModelPart>,
}

#[derive(Resource, Default)]
struct RangedModels {
    musket: Option<RangedModel>,
    bow: Option<RangedModel>,
}

impl RangedModel {
    pub(super) fn shared(world: &mut World, pair: WeaponPair) -> Self {
        world.init_resource::<RangedModels>();
        let cache = world.resource::<RangedModels>();
        let cached = match pair {
            WeaponPair::HammerMusket => &cache.musket,
            WeaponPair::ChainAxeBow => &cache.bow,
        };
        if let Some(model) = cached {
            return model.clone();
        }
        world.init_resource::<Assets<Image>>();
        let data: &[u8] = match pair {
            WeaponPair::HammerMusket => include_bytes!("../../assets/yinbingluo/weapon.bin"),
            WeaponPair::ChainAxeBow => include_bytes!("../../assets/yizhiying/weapon.bin"),
        };
        let mut reader = Reader(data);
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
                0 => Mount::Muzzle,
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
        let mut cache = world.resource_mut::<RangedModels>();
        match pair {
            WeaponPair::HammerMusket => cache.musket = Some(model.clone()),
            WeaponPair::ChainAxeBow => cache.bow = Some(model.clone()),
        }
        model
    }
}

pub(super) fn spawn(world: &mut World, parent: Entity, pair: WeaponPair, mount: Mount) {
    let model = RangedModel::shared(world, pair);
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
            let model = RangedModel::shared(&mut world, pair);
            let parts = model
                .parts
                .iter()
                .filter(|part| part.mount == Mount::Muzzle)
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
                assert!(model.parts.iter().all(|part| part.mount == Mount::Muzzle));
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
            let reused = RangedModel::shared(&mut world, pair);
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
}
