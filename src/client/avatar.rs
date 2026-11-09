//! Offline-extracted character geometry, immutable shared assets and pose retargeting.
//! Original named hierarchy/channels drive matching palettes; structural anchors
//! supply missing-action/accessory fallback. Simulation remains root authority.
use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::DynamicSkinnedMeshBounds,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    mesh::{
        Indices, PrimitiveTopology, VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
};

use super::{
    ActorView,
    animation::RigPose,
    skeleton::{AnimationLibrary, blend},
    source_animation::SourceAnimationLibrary,
};
use crate::simulation::{Enemy, Health};

#[derive(Clone)]
struct MaterialVariants {
    normal: Handle<StandardMaterial>,
    hit: Handle<StandardMaterial>,
    telegraph: Handle<StandardMaterial>,
}

#[derive(Clone)]
struct Primitive {
    mesh: Handle<Mesh>,
    material: MaterialVariants,
}

#[derive(Clone)]
struct SourceBone {
    source: Option<usize>,
    name: String,
    driver: usize,
    bind: Mat4,
}

#[derive(Clone)]
struct Part {
    name: String,
    blade: bool,
    bones: Vec<SourceBone>,
    inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
    primitives: Vec<Primitive>,
}

#[derive(Clone)]
struct Driver {
    source: Option<usize>,
    source_inverse_rest: Mat4,
    bone: usize,
    parent: Option<usize>,
    anchor: Vec3,
    reference: Transform,
}

#[derive(Resource, Clone)]
pub(super) struct AvatarAssets {
    drivers: Vec<Driver>,
    parts: Vec<Part>,
    right_hand: usize,
    source_hand: usize,
    source_hand_rest: Quat,
}

#[derive(Component)]
pub(super) struct PaletteJoint {
    source: Option<usize>,
    owner: Entity,
    driver: usize,
    bind: Mat4,
}

#[derive(Component)]
pub(super) struct AvatarSurface {
    owner: Entity,
    variants: MaterialVariants,
}

impl AvatarSurface {
    pub(super) fn material_for(&self, hit: bool, telegraph: bool) -> Handle<StandardMaterial> {
        if hit {
            &self.variants.hit
        } else if telegraph {
            &self.variants.telegraph
        } else {
            &self.variants.normal
        }
        .clone()
    }
}

// Explicit LE decoding of a trusted build artifact, matching import-choulingke.py.
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        self.bytes(N).try_into().unwrap()
    }
    fn bytes(&mut self, length: usize) -> &[u8] {
        let (head, tail) = self.0.split_at(length);
        self.0 = tail;
        head
    }
    fn u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }
    fn u16(&mut self) -> usize {
        u16::from_le_bytes(self.take()) as usize
    }
    fn u32(&mut self) -> usize {
        u32::from_le_bytes(self.take()) as usize
    }
    fn f32(&mut self) -> f32 {
        f32::from_le_bytes(self.take())
    }
    fn string(&mut self) -> String {
        let length = self.u16();
        String::from_utf8(self.bytes(length).to_vec()).unwrap()
    }
    fn vec3(&mut self) -> Vec3 {
        Vec3::new(self.f32(), self.f32(), self.f32())
    }
    fn matrix(&mut self) -> Mat4 {
        Mat4::from_cols_array(&std::array::from_fn(|_| self.f32()))
    }
}

impl AvatarAssets {
    fn shared(world: &mut World) -> Self {
        if !world.contains_resource::<Self>() {
            world.init_resource::<SourceAnimationLibrary>();
            world.init_resource::<Assets<Image>>();
            world.init_resource::<Assets<SkinnedMeshInverseBindposes>>();
            let mut reader = Reader(include_bytes!("../../assets/choulingke/character.bin"));
            assert_eq!(&reader.take::<8>(), b"KIVAAV02");
            let header_length = reader.u32();
            reader.bytes(header_length); // Offline provenance; no runtime paths or JSON dependency.
            let driver_count = reader.u16();
            let (drivers, right_hand) = {
                let library = world.resource::<AnimationLibrary>();
                let reference = library.model_pose(
                    &library
                        .bones
                        .iter()
                        .map(|bone| bone.rest)
                        .collect::<Vec<_>>(),
                );
                let mut records = Vec::new();
                for _ in 0..driver_count {
                    let bone = library.bone(&reader.string());
                    records.push((bone, reader.vec3()));
                }
                let source_library = world.resource::<SourceAnimationLibrary>();
                // The same explicit structural anchors used by the importer;
                // independent accessory palettes follow these real source
                // anchors when a source clip replaces the CC0 fallback.
                let source_names = [
                    "Scene Root",
                    "Bip001 Pelvis",
                    "Bip001 Spine",
                    "Bip001 Spine1",
                    "Bip001 Spine2",
                    "Bip001 Neck",
                    "Bip001 Head",
                    "Bip001 L Clavicle",
                    "Bone L UpperArm Twist",
                    "Bone L ForeArm Twist",
                    "Bip001 L Hand",
                    "Bip001 R Clavicle",
                    "Bone R UpperArm Twist",
                    "Bone R ForeArm Twist",
                    "Bip001 R Hand",
                    "Bip001 LThighTwist",
                    "Bip001 L Calf",
                    "Bip001 L Foot",
                    "Bip001 L Toe0",
                    "Bip001 RThighTwist",
                    "Bip001 R Calf",
                    "Bip001 R Foot",
                    "Bip001 R Toe0",
                ];
                let drivers = records
                    .iter()
                    .enumerate()
                    .map(|(index, &(bone, anchor))| Driver {
                        source: source_library.bone(source_names[index]),
                        source_inverse_rest: source_library
                            .bone(source_names[index])
                            .map_or(Mat4::IDENTITY, |source| {
                                source_library.rest_model[source].inverse()
                            }),
                        bone,
                        parent: library.bones[bone].parent.map(|parent| {
                            records
                                .iter()
                                .position(|(bone, _)| *bone == parent)
                                .unwrap()
                        }),
                        anchor,
                        reference: reference[bone],
                    })
                    .collect();
                let hand = records
                    .iter()
                    .position(|(bone, _)| *bone == library.bone("hand_r"))
                    .unwrap();
                (drivers, hand)
            };
            let texture_count = reader.u16();
            let mut textures = Vec::new();
            for _ in 0..texture_count {
                reader.string(); // Exact texture GUID is also retained in the provenance header.
                let srgb = reader.u8() != 0;
                let length = reader.u32();
                // Authored UVs include negative tiles and U > 1 (tops, pants,
                // sheath/blade). Clamp collapses these regions to edge colours.
                let mut sampler = ImageSamplerDescriptor::linear();
                sampler.address_mode_u = ImageAddressMode::Repeat;
                sampler.address_mode_v = ImageAddressMode::Repeat;
                let image = Image::from_buffer(
                    reader.bytes(length),
                    ImageType::Extension("png"),
                    CompressedImageFormats::NONE,
                    srgb,
                    ImageSampler::Descriptor(sampler),
                    RenderAssetUsages::default(),
                )
                .expect("verified bundled avatar PNG");
                textures.push(world.resource_mut::<Assets<Image>>().add(image));
            }
            let mut materials = Vec::new();
            let material_count = reader.u16();
            for _ in 0..material_count {
                reader.string();
                let base = i16::from_le_bytes(reader.take());
                let normal = i16::from_le_bytes(reader.take());
                let surface = i16::from_le_bytes(reader.take());
                let alpha = reader.u8() != 0;
                let roughness = reader.f32();
                let metallic = reader.f32();
                let reflectance = reader.f32();
                let anisotropy_strength = reader.f32();
                let clearcoat = reader.f32();
                let clearcoat_perceptual_roughness = reader.f32();
                let diffuse_transmission = reader.f32();
                let tint = reader.vec3();
                let alpha_cutoff = reader.f32();
                let material = StandardMaterial {
                    base_color: Color::linear_rgb(tint.x, tint.y, tint.z),
                    base_color_texture: (base >= 0).then(|| textures[base as usize].clone()),
                    normal_map_texture: (normal >= 0).then(|| textures[normal as usize].clone()),
                    metallic_roughness_texture: (surface >= 0)
                        .then(|| textures[surface as usize].clone()),
                    occlusion_texture: (surface >= 0).then(|| textures[surface as usize].clone()),
                    perceptual_roughness: roughness,
                    metallic,
                    reflectance,
                    anisotropy_strength,
                    anisotropy_rotation: std::f32::consts::FRAC_PI_2,
                    clearcoat,
                    clearcoat_perceptual_roughness,
                    diffuse_transmission,
                    alpha_mode: if alpha {
                        AlphaMode::Mask(alpha_cutoff)
                    } else {
                        AlphaMode::Opaque
                    },
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                };
                let mut assets = world.resource_mut::<Assets<StandardMaterial>>();
                let variants = MaterialVariants {
                    normal: assets.add(material.clone()),
                    hit: assets.add(StandardMaterial {
                        emissive: LinearRgba::rgb(0.9, 0.5, 0.2),
                        ..material.clone()
                    }),
                    telegraph: assets.add(StandardMaterial {
                        emissive: LinearRgba::rgb(0.45, 0.24, 0.025),
                        ..material
                    }),
                };
                materials.push(variants);
            }
            let part_count = reader.u16();
            let mut parts = Vec::new();
            for _ in 0..part_count {
                let name = reader.string();
                let blade = reader.u8() != 0;
                let bone_count = reader.u16();
                let mut bones = Vec::new();
                let mut inverse_binds = Vec::new();
                for _ in 0..bone_count {
                    let name = reader.string();
                    let driver = reader.u16();
                    assert!(driver < driver_count);
                    let inverse = reader.matrix();
                    assert!(inverse.is_finite() && inverse.determinant().abs() > 0.0001);
                    bones.push(SourceBone {
                        source: world.resource::<SourceAnimationLibrary>().bone(&name),
                        name,
                        driver,
                        bind: inverse.inverse(),
                    });
                    inverse_binds.push(inverse);
                }
                let inverse_bindposes = world
                    .resource_mut::<Assets<SkinnedMeshInverseBindposes>>()
                    .add(inverse_binds);
                let primitive_count = reader.u16();
                let mut primitives = Vec::new();
                for _ in 0..primitive_count {
                    let material = materials[reader.u16()].clone();
                    let count = reader.u32();
                    let index_count = reader.u32();
                    let (mut positions, mut normals, mut uvs, mut joints, mut weights) =
                        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
                    for _ in 0..count {
                        positions.push(reader.vec3().to_array());
                        normals.push(reader.vec3().to_array());
                        uvs.push([reader.f32(), reader.f32()]);
                        let indices = reader.take::<4>().map(u16::from);
                        assert!(indices.iter().all(|&index| (index as usize) < bone_count));
                        joints.push(indices);
                        let bytes = reader.take::<4>();
                        assert_eq!(bytes.iter().map(|&value| value as u16).sum::<u16>(), 255);
                        weights.push(bytes.map(|value| value as f32 / 255.0));
                    }
                    let indices: Vec<_> = (0..index_count).map(|_| reader.u32() as u32).collect();
                    assert!(indices.iter().all(|&index| (index as usize) < count));
                    let mut mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    );
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                    mesh.insert_indices(Indices::U32(indices));
                    mesh.generate_tangents()
                        .expect("verified avatar normals/UVs for normal mapping");
                    if !blade {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_JOINT_INDEX,
                            VertexAttributeValues::Uint16x4(joints),
                        );
                        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights);
                        mesh.generate_skinned_mesh_bounds()
                            .expect("verified avatar skin bounds");
                    }
                    primitives.push(Primitive {
                        mesh: world.resource_mut::<Assets<Mesh>>().add(mesh),
                        material,
                    });
                }
                parts.push(Part {
                    name,
                    blade,
                    bones,
                    inverse_bindposes,
                    primitives,
                });
            }
            assert!(reader.0.is_empty());
            let source_library = world.resource::<SourceAnimationLibrary>();
            let source_hand = source_library.bone("Bip001 R Hand").unwrap();
            let source_hand_rest =
                Transform::from_matrix(source_library.rest_model[source_hand]).rotation;
            world.insert_resource(Self {
                source_hand,
                source_hand_rest,
                drivers,
                parts,
                right_hand,
            });
        }
        world.resource::<Self>().clone()
    }

    /// Existing model-space rotations provide animation deltas; the extracted
    /// structural anchor offsets retain this character's proportions. Pelvis
    /// translation carries the existing crouch/jump-pose adjustment exactly once.
    fn driver_pose(&self, model: &[Transform]) -> Vec<Transform> {
        let mut result: Vec<Transform> = Vec::with_capacity(self.drivers.len());
        for (index, driver) in self.drivers.iter().enumerate() {
            let rotation = model[driver.bone].rotation * driver.reference.rotation.inverse();
            let translation = driver.parent.map_or(Vec3::ZERO, |parent| {
                result[parent].translation
                    + result[parent].rotation * (driver.anchor - self.drivers[parent].anchor)
            }) + if index == 1 {
                model[driver.bone].translation - driver.reference.translation
            } else {
                Vec3::ZERO
            };
            result.push(Transform::from_translation(translation).with_rotation(rotation));
        }
        result
    }

    fn deltas(&self, model: &[Transform]) -> Vec<Mat4> {
        self.driver_pose(model)
            .iter()
            .zip(&self.drivers)
            .map(|(posed, driver)| posed.to_matrix() * Mat4::from_translation(-driver.anchor))
            .collect()
    }

    pub(super) fn attachment_pose(&self, rig: &RigPose) -> Vec<Transform> {
        let drivers = self.driver_pose(&rig.model);
        let mut result = rig.model.clone();
        let hand = &self.drivers[self.right_hand];
        // Preserve the authored hand orientation used by weapon corrections and
        // ranged aiming; only align its attachment position to the retargeted hand.
        result[hand.bone].translation = drivers[self.right_hand].translation;
        if let Some(source) = &rig.source {
            let posed = Transform::from_matrix(source.model[self.source_hand]);
            let source_hand = Transform::from_translation(posed.translation).with_rotation(
                posed.rotation * self.source_hand_rest.inverse() * hand.reference.rotation,
            );
            result[hand.bone] = blend(result[hand.bone], source_hand, source.weight);
        }
        result
    }
}

pub(super) fn spawn(world: &mut World, owner: Entity) -> Vec<Entity> {
    let assets = AvatarAssets::shared(world);
    let rig = RigPose::new(world.resource::<AnimationLibrary>());
    let deltas = assets.deltas(&rig.model);
    let mut surfaces = Vec::new();
    for part in assets.parts.iter().filter(|part| !part.blade) {
        let mut joints = Vec::new();
        for bone in &part.bones {
            let joint = world
                .spawn((
                    Name::new(bone.name.clone()),
                    Transform::from_matrix(deltas[bone.driver] * bone.bind),
                    PaletteJoint {
                        source: bone.source,
                        owner,
                        driver: bone.driver,
                        bind: bone.bind,
                    },
                ))
                .id();
            world.entity_mut(owner).add_child(joint);
            joints.push(joint);
        }
        for primitive in &part.primitives {
            let entity = world
                .spawn((
                    Name::new(part.name.clone()),
                    Mesh3d(primitive.mesh.clone()),
                    MeshMaterial3d(primitive.material.normal.clone()),
                    Transform::IDENTITY,
                    SkinnedMesh {
                        inverse_bindposes: part.inverse_bindposes.clone(),
                        joints: joints.clone(),
                    },
                    DynamicSkinnedMeshBounds,
                    AvatarSurface {
                        owner,
                        variants: primitive.material.clone(),
                    },
                ))
                .id();
            world.entity_mut(owner).add_child(entity);
            surfaces.push(entity);
        }
    }
    world.entity_mut(owner).insert(rig);
    surfaces
}

/// Confirmed weapon1 long blade, shared by both simulation melee sets.
pub(super) fn spawn_blade(world: &mut World, parent: Entity) {
    let assets = AvatarAssets::shared(world);
    let blade = assets
        .parts
        .iter()
        .find(|part| part.blade)
        .expect("bundled long blade");
    for primitive in &blade.primitives {
        let entity = world
            .spawn((
                Mesh3d(primitive.mesh.clone()),
                MeshMaterial3d(primitive.material.normal.clone()),
                Transform::IDENTITY,
            ))
            .id();
        world.entity_mut(parent).add_child(entity);
    }
}

pub(super) fn sync(
    assets: Option<Res<AvatarAssets>>,
    actors: Query<(Entity, &RigPose, Option<&ActorView>)>,
    health: Query<(&Health, Option<&Enemy>)>,
    mut joints: Query<(&PaletteJoint, &mut Transform)>,
    mut surfaces: Query<
        (&AvatarSurface, &mut MeshMaterial3d<StandardMaterial>),
        Without<super::combat::EnemyBody>,
    >,
) {
    let Some(assets) = assets else { return };
    let mut poses = HashMap::new();
    let mut tints = HashMap::new();
    for (entity, pose, view) in &actors {
        poses.insert(entity, assets.deltas(&pose.model));
        let tint =
            view.and_then(|view| health.get(view.owner).ok())
                .map_or(0, |(health, enemy)| {
                    if health.hit_remaining() > 0.0 {
                        1
                    } else if enemy.is_some_and(Enemy::is_winding_up) {
                        2
                    } else {
                        0
                    }
                });
        tints.insert(entity, tint);
    }
    for (joint, mut transform) in &mut joints {
        if let Some(deltas) = poses.get(&joint.owner) {
            let fallback = Transform::from_matrix(deltas[joint.driver] * joint.bind);
            *transform = actors
                .get(joint.owner)
                .ok()
                .and_then(|(_, rig, _)| rig.source.as_ref())
                .map_or(fallback, |source| {
                    let driver = &assets.drivers[joint.driver];
                    let matrix = joint.source.map(|index| source.model[index]).or_else(|| {
                        driver.source.map(|index| {
                            source.model[index] * driver.source_inverse_rest * joint.bind
                        })
                    });
                    matrix.map_or(fallback, |matrix| {
                        blend(fallback, Transform::from_matrix(matrix), source.weight)
                    })
                });
        }
    }
    for (surface, mut material) in &mut surfaces {
        material.0 = surface.material_for(
            tints.get(&surface.owner) == Some(&1),
            tints.get(&surface.owner) == Some(&2),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{
        ActionIntent, Body, CollisionWorld, CombatIntent, Combatant, Facing, MotionState,
        MoveIntent, SimulationActive, SimulationPlugin, SpawnPoint, WeaponSet,
    };
    use std::time::Duration;

    fn test_app(animated: bool) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(CollisionWorld::new(100.0, vec![]))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<AnimationLibrary>();
        if animated {
            app.add_systems(Update, (super::super::animation::animate, sync).chain());
        } else {
            app.add_systems(Update, sync);
        }
        app.finish();
        app
    }

    fn sample_surface(world: &mut World, owner: Entity) -> Vec<Vec3> {
        let assets = world.resource::<Assets<Mesh>>();
        let binds = world.resource::<Assets<SkinnedMeshInverseBindposes>>();
        let mut result = Vec::new();
        for entity in world.iter_entities() {
            let Some(surface) = entity.get::<AvatarSurface>() else {
                continue;
            };
            if surface.owner != owner {
                continue;
            }
            let skin = entity.get::<SkinnedMesh>().unwrap();
            let mesh = assets.get(&entity.get::<Mesh3d>().unwrap().0).unwrap();
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            let VertexAttributeValues::Uint16x4(joints) =
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX).unwrap()
            else {
                panic!("joint indices")
            };
            let VertexAttributeValues::Float32x4(weights) =
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT).unwrap()
            else {
                panic!("weights")
            };
            let inverse = binds.get(&skin.inverse_bindposes).unwrap();
            for index in (0..positions.len()).step_by(43) {
                let mut vertex = Vec3::ZERO;
                for slot in 0..4 {
                    let joint = joints[index][slot] as usize;
                    let matrix = world
                        .get::<Transform>(skin.joints[joint])
                        .unwrap()
                        .to_matrix()
                        * inverse[joint];
                    vertex += matrix.transform_point3(Vec3::from(positions[index]))
                        * weights[index][slot];
                }
                assert!(vertex.is_finite());
                result.push(vertex);
            }
        }
        result
    }

    fn changed(first: &[Vec3], second: &[Vec3]) -> f32 {
        assert_eq!(first.len(), second.len());
        first
            .iter()
            .zip(second)
            .map(|(a, b)| a.distance(*b))
            .fold(0.0, f32::max)
    }

    #[test]
    fn bundled_skin_cancels_at_rest_and_has_real_base_and_linear_normal_maps() {
        let mut app = test_app(false);
        let world = app.world_mut();
        let owner = world.spawn(Transform::IDENTITY).id();
        let surfaces = spawn(world, owner);
        let assets = world.resource::<AvatarAssets>();
        assert_eq!(assets.parts.len(), 14);
        assert_eq!(surfaces.len(), 15); // 17 source slots minus two hand-held blade slots.
        let library = world.resource::<AnimationLibrary>();
        let rest = library.model_pose(
            &library
                .bones
                .iter()
                .map(|bone| bone.rest)
                .collect::<Vec<_>>(),
        );
        for delta in assets.deltas(&rest) {
            assert!(delta.abs_diff_eq(Mat4::IDENTITY, 0.00001));
        }
        for part in &assets.parts {
            let inverse = world
                .resource::<Assets<SkinnedMeshInverseBindposes>>()
                .get(&part.inverse_bindposes)
                .unwrap();
            assert!(
                part.bones
                    .iter()
                    .zip(inverse.iter())
                    .all(|(bone, inverse)| (bone.bind * *inverse)
                        .abs_diff_eq(Mat4::IDENTITY, 0.00001))
            );
            for primitive in &part.primitives {
                let material = world
                    .resource::<Assets<StandardMaterial>>()
                    .get(&primitive.material.normal)
                    .unwrap();
                let images = world.resource::<Assets<Image>>();
                assert!(material.base_color_texture.is_some());
                assert!(material.metallic_roughness_texture.is_some());
                assert_eq!(
                    material.metallic_roughness_texture,
                    material.occlusion_texture
                );
                let surface = images
                    .get(material.metallic_roughness_texture.as_ref().unwrap())
                    .unwrap();
                assert!(!surface.texture_descriptor.format.is_srgb());
                if let Some(normal) = &material.normal_map_texture {
                    assert!(
                        !images
                            .get(normal)
                            .unwrap()
                            .texture_descriptor
                            .format
                            .is_srgb()
                    );
                }
                let base = images
                    .get(material.base_color_texture.as_ref().unwrap())
                    .unwrap();
                assert!(base.texture_descriptor.format.is_srgb());
                assert!(matches!(&base.sampler, ImageSampler::Descriptor(sampler)
                    if sampler.address_mode_u == ImageAddressMode::Repeat
                        && sampler.address_mode_v == ImageAddressMode::Repeat));
                assert!(base.width() <= 512 && base.height() <= 512);
            }
        }
        world.get_mut::<RigPose>(owner).unwrap().model = rest;
        world.run_schedule(Update);
        let sampled = sample_surface(world, owner);
        assert!(sampled.len() > 1000);
        let mut original = Vec::new();
        for entity in world.iter_entities() {
            if entity.get::<AvatarSurface>().is_some() {
                let mesh = world
                    .resource::<Assets<Mesh>>()
                    .get(&entity.get::<Mesh3d>().unwrap().0)
                    .unwrap();
                original.extend(
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap()
                        .iter()
                        .step_by(43)
                        .copied()
                        .map(Vec3::from),
                );
            }
        }
        assert!(changed(&sampled, &original) < 0.00001);
    }

    #[test]
    fn original_hierarchy_matches_body_binds_and_wrist_delta_drives_the_grip() {
        let mut app = test_app(false);
        let world = app.world_mut();
        let owner = world.spawn(Transform::IDENTITY).id();
        spawn(world, owner);
        let assets = world.resource::<AvatarAssets>().clone();
        let source = world.resource::<SourceAnimationLibrary>();
        let mut matches = 0;
        for part in &assets.parts {
            let inverse = world
                .resource::<Assets<SkinnedMeshInverseBindposes>>()
                .get(&part.inverse_bindposes)
                .unwrap();
            for (bone, inverse) in part.bones.iter().zip(inverse.iter()) {
                if let Some(index) = bone.source {
                    matches += 1;
                    assert!(
                        (source.rest_model[index] * *inverse).abs_diff_eq(Mat4::IDENTITY, 0.0004),
                        "{} / {}",
                        part.name,
                        bone.name
                    );
                }
            }
        }
        assert!(matches > 200);
        let clip = source.clip("skill_ylk_1").unwrap();
        let model = source.model_pose(&source.sample(clip, 1.5, false, false));
        let mut rig = RigPose::new(world.resource::<AnimationLibrary>());
        let cc0 = world.resource::<AnimationLibrary>();
        rig.model = cc0.model_pose(&cc0.bones.iter().map(|bone| bone.rest).collect::<Vec<_>>());
        rig.source = Some(super::super::animation::SourcePose {
            model: source.rest_model.clone(),
            weight: 1.0,
        });
        let aligned_rest = assets.attachment_pose(&rig);
        let hand = &assets.drivers[assets.right_hand];
        assert!(
            aligned_rest[hand.bone]
                .rotation
                .angle_between(hand.reference.rotation)
                < 0.001
        );
        rig.source = Some(super::super::animation::SourcePose {
            model: model.clone(),
            weight: 1.0,
        });
        let pose = assets.attachment_pose(&rig);
        assert!(
            pose[hand.bone]
                .translation
                .distance(model[assets.source_hand].w_axis.xyz())
                < 0.00001
        );
        let blade = super::super::stickman::attachment_transform(
            &super::super::stickman::Attachment::Weapon {
                bone: hand.bone,
                mode: crate::simulation::WeaponMode::Melee,
            },
            &pose,
        );
        assert!(blade.is_finite());
        assert!((blade.rotation * Vec3::Y).distance(pose[hand.bone].rotation * Vec3::Z) < 0.00001);
        world.entity_mut(owner).insert(rig);
        world.run_schedule(Update);
        let points = sample_surface(world, owner);
        assert!(
            points
                .iter()
                .all(|p| p.is_finite() && p.abs().max_element() < 5.0)
        );
        assert_eq!(*world.get::<Transform>(owner).unwrap(), Transform::IDENTITY);
    }

    #[test]
    fn real_simulation_and_animation_drive_the_skin_through_walk_crouch_jump_attack_and_pause() {
        let mut app = test_app(true);
        let actor = app
            .world_mut()
            .spawn((
                Body::new(Vec2::ZERO, 6.0, 0.35),
                MoveIntent::default(),
                ActionIntent::default(),
                MotionState::default(),
                Facing::default(),
                Health::new(100.0),
                Combatant::new(WeaponSet::HammerSniper),
                CombatIntent::default(),
                SpawnPoint(Vec2::ZERO),
            ))
            .id();
        let view = app
            .world_mut()
            .spawn((ActorView { owner: actor }, Transform::IDENTITY))
            .id();
        spawn(app.world_mut(), view);
        let frame = |app: &mut App, count: usize| {
            for _ in 0..count {
                let step = Duration::from_secs_f64(1.0 / 60.0);
                app.world_mut()
                    .resource_mut::<Time<Fixed>>()
                    .advance_by(step);
                app.world_mut().run_schedule(FixedUpdate);
                app.world_mut().resource_mut::<Time>().advance_by(step);
                app.world_mut().run_schedule(Update);
            }
        };
        frame(&mut app, 12);
        let idle = sample_surface(app.world_mut(), view);
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        frame(&mut app, 20);
        let walking = sample_surface(app.world_mut(), view);
        assert!(changed(&idle, &walking) > 0.05);
        app.world_mut().get_mut::<MoveIntent>(actor).unwrap().stop();
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_crouch_toggle();
        frame(&mut app, 15);
        assert!(app.world().get::<MotionState>(actor).unwrap().is_crouched());
        let crouched = sample_surface(app.world_mut(), view);
        assert!(changed(&idle, &crouched) > 0.15);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        frame(&mut app, 5);
        assert!(app.world().get::<MotionState>(actor).unwrap().height() > 0.1);
        assert!(app.world().get::<RigPose>(view).unwrap().source.is_some());
        let jumping = sample_surface(app.world_mut(), view);
        assert!(changed(&crouched, &jumping) > 0.1);
        frame(&mut app, 90);
        if app.world().get::<MotionState>(actor).unwrap().is_crouched() {
            app.world_mut()
                .get_mut::<ActionIntent>(actor)
                .unwrap()
                .request_crouch_toggle();
            frame(&mut app, 15);
        }
        let standing = sample_surface(app.world_mut(), view);
        app.world_mut()
            .get_mut::<CombatIntent>(actor)
            .unwrap()
            .request_attack();
        frame(&mut app, 8);
        assert_eq!(
            app.world()
                .get::<RigPose>(view)
                .unwrap()
                .source
                .as_ref()
                .unwrap()
                .weight,
            1.0
        );
        let attacking = sample_surface(app.world_mut(), view);
        assert!(changed(&standing, &attacking) > 0.05);
        let position = app.world().get::<Body>(actor).unwrap().world_position();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        frame(&mut app, 20);
        assert!(changed(&attacking, &sample_surface(app.world_mut(), view)) < 0.000001);
        assert_eq!(
            app.world().get::<Body>(actor).unwrap().world_position(),
            position
        );
        assert_eq!(
            *app.world().get::<Transform>(view).unwrap(),
            Transform::IDENTITY
        );
    }
}
