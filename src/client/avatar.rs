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

/// A display asset selection; combat weapon sets and actors remain simulation-owned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Appearance {
    #[default]
    Choulingke,
    Lihuowang,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AvatarAppearance(pub Appearance);

#[derive(Component)]
struct AvatarPart;

#[derive(Component)]
struct BladeSurface;

#[derive(Component)]
struct BladeAppearance(Appearance);

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
    // Display rest model. AV03 retains imported poses that differ from bind pose.
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
    appearance: Appearance,
    drivers: Vec<Driver>,
    parts: Vec<Part>,
    right_hand: usize,
    source_hand: usize,
    source_hand_rest: Quat,
}

/// The original set retains its existing resource; the second set loads once on demand.
#[derive(Resource, Default)]
pub(super) struct AvatarAssetCache {
    lihuowang: Option<AvatarAssets>,
}

#[derive(Component)]
pub(super) struct PaletteJoint {
    owner: Entity,
    bone: SourceBone,
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
    fn shared(world: &mut World, appearance: Appearance) -> Self {
        world.init_resource::<AvatarAssetCache>();
        if let Some(assets) = Self::cached(
            world.get_resource::<Self>(),
            world.get_resource::<AvatarAssetCache>(),
            appearance,
        ) {
            return assets.clone();
        }
        {
            world.init_resource::<SourceAnimationLibrary>();
            world.init_resource::<Assets<Image>>();
            world.init_resource::<Assets<SkinnedMeshInverseBindposes>>();
            let data: &[u8] = match appearance {
                Appearance::Choulingke => include_bytes!("../../assets/choulingke/character.bin"),
                Appearance::Lihuowang => include_bytes!("../../assets/lihuowang/character.bin"),
            };
            let mut reader = Reader(data);
            let format = reader.take::<8>();
            assert!(matches!(&format, b"KIVAAV02" | b"KIVAAV03"));
            let source_pose = &format == b"KIVAAV03";
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
                let double_sided = !source_pose || reader.u8() != 0;
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
                    double_sided,
                    cull_mode: (!double_sided).then_some(bevy::render::render_resource::Face::Back),
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
                    let rest = if source_pose {
                        reader.matrix()
                    } else {
                        inverse.inverse()
                    };
                    assert!(rest.is_finite() && rest.determinant().abs() > 0.0001);
                    bones.push(SourceBone {
                        source: if appearance == Appearance::Choulingke {
                            world.resource::<SourceAnimationLibrary>().bone(&name)
                        } else {
                            None
                        },
                        name,
                        driver,
                        bind: rest,
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
            let assets = Self {
                appearance,
                source_hand,
                source_hand_rest,
                drivers,
                parts,
                right_hand,
            };
            match appearance {
                Appearance::Choulingke => world.insert_resource(assets),
                Appearance::Lihuowang => {
                    world.resource_mut::<AvatarAssetCache>().lihuowang = Some(assets);
                }
            }
        }
        Self::cached(
            world.get_resource::<Self>(),
            world.get_resource::<AvatarAssetCache>(),
            appearance,
        )
        .expect("requested bundled appearance is cached")
        .clone()
    }

    pub(super) fn cached<'a>(
        original: Option<&'a Self>,
        cache: Option<&'a AvatarAssetCache>,
        appearance: Appearance,
    ) -> Option<&'a Self> {
        match appearance {
            Appearance::Choulingke => original,
            Appearance::Lihuowang => cache?.lihuowang.as_ref(),
        }
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

    /// Reuse motion deltas, rather than another character's absolute bone
    /// positions. The target's own anchor segments retain its proportions.
    fn source_driver_pose(&self, model: &[Mat4]) -> Vec<Transform> {
        let pelvis = &self.drivers[1];
        let pelvis_motion = pelvis.source.map_or(Vec3::ZERO, |index| {
            let rest = pelvis.source_inverse_rest.inverse().w_axis.xyz();
            let ratio = if rest.y.abs() > 0.0001 {
                pelvis.anchor.y / rest.y
            } else {
                1.0
            };
            (model[index].w_axis.xyz() - rest) * ratio
        });
        let mut result: Vec<Transform> = Vec::with_capacity(self.drivers.len());
        for (index, driver) in self.drivers.iter().enumerate() {
            let rotation = driver.source.map_or(Quat::IDENTITY, |source| {
                Transform::from_matrix(model[source] * driver.source_inverse_rest).rotation
            });
            let translation = driver.parent.map_or(Vec3::ZERO, |parent| {
                result[parent].translation
                    + result[parent].rotation * (driver.anchor - self.drivers[parent].anchor)
            }) + if index == 1 {
                pelvis_motion
            } else {
                Vec3::ZERO
            };
            result.push(Transform::from_translation(translation).with_rotation(rotation));
        }
        result
    }

    fn source_deltas(&self, rig: &RigPose) -> Option<Vec<Mat4>> {
        if self.appearance != Appearance::Lihuowang {
            return None;
        }
        let source = rig.source.as_ref()?;
        Some(
            self.source_driver_pose(&source.model)
                .iter()
                .zip(&self.drivers)
                .map(|(posed, driver)| posed.to_matrix() * Mat4::from_translation(-driver.anchor))
                .collect(),
        )
    }

    fn joint_pose(
        &self,
        rig: &RigPose,
        deltas: &[Mat4],
        source_deltas: Option<&[Mat4]>,
        bone: &SourceBone,
    ) -> Transform {
        let fallback = Transform::from_matrix(deltas[bone.driver] * bone.bind);
        rig.source.as_ref().map_or(fallback, |source| {
            let matrix = if self.appearance == Appearance::Lihuowang {
                source_deltas.map(|deltas| deltas[bone.driver] * bone.bind)
            } else {
                let driver = &self.drivers[bone.driver];
                bone.source.map(|index| source.model[index]).or_else(|| {
                    driver
                        .source
                        .map(|index| source.model[index] * driver.source_inverse_rest * bone.bind)
                })
            };
            matrix.map_or(fallback, |matrix| {
                blend(fallback, Transform::from_matrix(matrix), source.weight)
            })
        })
    }

    pub(super) fn attachment_pose(&self, rig: &RigPose) -> Vec<Transform> {
        let drivers = self.driver_pose(&rig.model);
        let mut result = rig.model.clone();
        let hand = &self.drivers[self.right_hand];
        // Preserve the authored hand orientation used by weapon corrections and
        // ranged aiming; only align its attachment position to the retargeted hand.
        result[hand.bone].translation = drivers[self.right_hand].translation;
        if let Some(source) = &rig.source {
            let source_hand = if self.appearance == Appearance::Lihuowang {
                let posed = self.source_driver_pose(&source.model)[self.right_hand];
                Transform::from_translation(posed.translation)
                    .with_rotation(posed.rotation * hand.reference.rotation)
            } else {
                let posed = Transform::from_matrix(source.model[self.source_hand]);
                Transform::from_translation(posed.translation).with_rotation(
                    posed.rotation * self.source_hand_rest.inverse() * hand.reference.rotation,
                )
            };
            result[hand.bone] = blend(result[hand.bone], source_hand, source.weight);
        }
        result
    }
}

pub(super) fn spawn(world: &mut World, owner: Entity) -> Vec<Entity> {
    spawn_with_appearance(world, owner, Appearance::Choulingke)
}

pub(super) fn spawn_with_appearance(
    world: &mut World,
    owner: Entity,
    appearance: Appearance,
) -> Vec<Entity> {
    let assets = AvatarAssets::shared(world, appearance);
    if world.get::<RigPose>(owner).is_none() {
        let rig = RigPose::new(world.resource::<AnimationLibrary>());
        world.entity_mut(owner).insert(rig);
    }
    let rig = world.get::<RigPose>(owner).unwrap();
    let deltas = assets.deltas(&rig.model);
    let source_deltas = assets.source_deltas(rig);
    let poses: Vec<Vec<_>> = assets
        .parts
        .iter()
        .filter(|part| !part.blade)
        .map(|part| {
            part.bones
                .iter()
                .map(|bone| assets.joint_pose(rig, &deltas, source_deltas.as_deref(), bone))
                .collect()
        })
        .collect();
    let mut surfaces = Vec::new();
    for (part, poses) in assets.parts.iter().filter(|part| !part.blade).zip(poses) {
        let mut joints = Vec::new();
        for (bone, pose) in part.bones.iter().zip(poses) {
            let joint = world
                .spawn((
                    AvatarPart,
                    Name::new(bone.name.clone()),
                    pose,
                    PaletteJoint {
                        owner,
                        bone: bone.clone(),
                    },
                ))
                .id();
            world.entity_mut(owner).add_child(joint);
            joints.push(joint);
        }
        for primitive in &part.primitives {
            let entity = world
                .spawn((
                    AvatarPart,
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
    world.entity_mut(owner).insert(AvatarAppearance(appearance));
    surfaces
}

/// Only the avatar's own display nodes are replaced. Rig state and other
/// children (weapon roots, markers) keep their identity and current playback.
pub(super) fn replace(world: &mut World, owner: Entity, appearance: Appearance) {
    if world.get::<AvatarAppearance>(owner) == Some(&AvatarAppearance(appearance)) {
        return;
    }
    let parts: Vec<_> = world
        .get::<Children>(owner)
        .into_iter()
        .flatten()
        .copied()
        .filter(|&child| world.get::<AvatarPart>(child).is_some())
        .collect();
    for part in parts {
        world.despawn(part);
    }
    spawn_with_appearance(world, owner, appearance);
}

/// Confirmed weapon1 long blade, shared by both simulation melee sets.
pub(super) fn spawn_blade(world: &mut World, parent: Entity) {
    spawn_blade_with_appearance(world, parent, Appearance::Choulingke);
}

pub(super) fn spawn_blade_with_appearance(
    world: &mut World,
    parent: Entity,
    appearance: Appearance,
) {
    let assets = AvatarAssets::shared(world, appearance);
    let blade = assets
        .parts
        .iter()
        .find(|part| part.blade)
        .expect("bundled long blade");
    for primitive in &blade.primitives {
        let entity = world
            .spawn((
                BladeSurface,
                Mesh3d(primitive.mesh.clone()),
                MeshMaterial3d(primitive.material.normal.clone()),
                Transform::IDENTITY,
            ))
            .id();
        world.entity_mut(parent).add_child(entity);
    }
    world.entity_mut(parent).insert(BladeAppearance(appearance));
}

pub(super) fn replace_blade(world: &mut World, parent: Entity, appearance: Appearance) {
    if world
        .get::<BladeAppearance>(parent)
        .is_some_and(|current| current.0 == appearance)
    {
        return;
    }
    let parts: Vec<_> = world
        .get::<Children>(parent)
        .into_iter()
        .flatten()
        .copied()
        .filter(|&child| world.get::<BladeSurface>(child).is_some())
        .collect();
    for part in parts {
        world.despawn(part);
    }
    spawn_blade_with_appearance(world, parent, appearance);
}

pub(super) fn sync(
    assets: Option<Res<AvatarAssets>>,
    cache: Option<Res<AvatarAssetCache>>,
    actors: Query<(
        Entity,
        &RigPose,
        Option<&ActorView>,
        Option<&AvatarAppearance>,
    )>,
    health: Query<(&Health, Option<&Enemy>)>,
    mut joints: Query<(&PaletteJoint, &mut Transform)>,
    mut surfaces: Query<
        (&AvatarSurface, &mut MeshMaterial3d<StandardMaterial>),
        Without<super::combat::EnemyBody>,
    >,
) {
    let mut poses = HashMap::new();
    let mut tints = HashMap::new();
    for (entity, pose, view, appearance) in &actors {
        let appearance = appearance.map_or(Appearance::default(), |current| current.0);
        let Some(selected) = AvatarAssets::cached(assets.as_deref(), cache.as_deref(), appearance)
        else {
            continue;
        };
        poses.insert(
            entity,
            (
                selected,
                selected.deltas(&pose.model),
                selected.source_deltas(pose),
            ),
        );
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
        if let Some((selected, deltas, source_deltas)) = poses.get(&joint.owner)
            && let Ok((_, rig, _, _)) = actors.get(joint.owner)
        {
            *transform = selected.joint_pose(rig, deltas, source_deltas.as_deref(), &joint.bone);
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
        sample_matching_surface(world, owner, None)
    }

    fn sample_matching_surface(
        world: &mut World,
        owner: Entity,
        name_prefix: Option<&str>,
    ) -> Vec<Vec3> {
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
            if name_prefix.is_some_and(|prefix| {
                entity
                    .get::<Name>()
                    .is_none_or(|name| !name.as_str().starts_with(prefix))
            }) {
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
    fn wangxin_hat_follows_the_head_and_is_reclaimed_when_switching_appearance() {
        let mut app = test_app(false);
        let world = app.world_mut();
        let root = Transform::from_xyz(2.0, 0.8, -3.0);
        let owner = world
            .spawn((Body::at(root.translation, 6.0, 0.35), root))
            .id();
        spawn_with_appearance(world, owner, Appearance::Lihuowang);
        let library = world.resource::<AnimationLibrary>();
        let head = library.bone("Head");
        let rest = library.model_pose(
            &library
                .bones
                .iter()
                .map(|bone| bone.rest)
                .collect::<Vec<_>>(),
        );
        world.get_mut::<RigPose>(owner).unwrap().model = rest;
        world.run_schedule(Update);
        let before = sample_matching_surface(world, owner, Some("naraka::hat::"));
        assert!(
            !before.is_empty(),
            "the selected appearance must include the original hat"
        );
        assert_eq!(
            world
                .query::<(&AvatarSurface, &Name, &SkinnedMesh)>()
                .iter(world)
                .filter(|(surface, name, skin)| {
                    surface.owner == owner
                        && name.as_str().starts_with("naraka::hat::")
                        && !skin.joints.is_empty()
                })
                .count(),
            4,
        );
        world.get_mut::<RigPose>(owner).unwrap().model[head].rotation *=
            Quat::from_rotation_y(0.35);
        world.run_schedule(Update);
        let turned = sample_matching_surface(world, owner, Some("naraka::hat::"));
        assert!(changed(&before, &turned) > 0.01);
        replace(world, owner, Appearance::Choulingke);
        world.run_schedule(Update);
        assert!(sample_matching_surface(world, owner, Some("naraka::hat::")).is_empty());
        replace(world, owner, Appearance::Lihuowang);
        world.run_schedule(Update);
        let counts = (
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<Image>>().len(),
            world.resource::<Assets<StandardMaterial>>().len(),
            world.entities().len(),
        );
        replace(world, owner, Appearance::Choulingke);
        replace(world, owner, Appearance::Lihuowang);
        world.run_schedule(Update);
        assert!(
            changed(
                &turned,
                &sample_matching_surface(world, owner, Some("naraka::hat::")),
            ) < 0.00001
        );
        assert_eq!(
            counts,
            (
                world.resource::<Assets<Mesh>>().len(),
                world.resource::<Assets<Image>>().len(),
                world.resource::<Assets<StandardMaterial>>().len(),
                world.entities().len(),
            )
        );
        assert_eq!(*world.get::<Transform>(owner).unwrap(), root);
        assert_eq!(
            world.get::<Body>(owner).unwrap().world_position(),
            root.translation
        );
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
    fn replacing_appearance_preserves_playback_and_other_children_and_reuses_both_sets() {
        let mut app = test_app(false);
        let world = app.world_mut();
        let owner = world
            .spawn((
                Body::new(Vec2::new(3.0, -2.0), 6.0, 0.35),
                Transform::from_xyz(3.0, 0.7, -2.0),
            ))
            .id();
        spawn(world, owner);
        let first_mesh = world
            .query::<(&AvatarSurface, &Mesh3d)>()
            .iter(world)
            .find(|(surface, _)| surface.owner == owner)
            .unwrap()
            .1
            .0
            .clone();
        let blade = world.spawn(Transform::IDENTITY).id();
        let unrelated = world.spawn(Transform::from_xyz(0.0, 1.0, 0.0)).id();
        world.entity_mut(owner).add_children(&[blade, unrelated]);
        spawn_blade(world, blade);
        super::super::stickman::attach_weapon(
            world,
            owner,
            blade,
            crate::simulation::WeaponMode::Melee,
        );
        let (model, source_pose) = {
            let cc0 = world.resource::<AnimationLibrary>();
            let source = world.resource::<SourceAnimationLibrary>();
            let clip = source.clip("skill_ylk_1").unwrap();
            (
                cc0.model_pose(&cc0.sample("Walk_Loop", 0.4, true)),
                source.model_pose(&source.sample(clip, 1.3, false, false)),
            )
        };
        {
            let mut rig = world.get_mut::<RigPose>(owner).unwrap();
            rig.model = model.clone();
            rig.source = Some(super::super::animation::SourcePose {
                model: source_pose,
                weight: 0.7,
            });
        }
        let source_pointer = world
            .get::<RigPose>(owner)
            .unwrap()
            .source
            .as_ref()
            .unwrap()
            .model
            .as_ptr();
        let root_transform = *world.get::<Transform>(owner).unwrap();
        let actor_position = world.get::<Body>(owner).unwrap().world_position();
        replace(world, owner, Appearance::Lihuowang);
        replace_blade(world, blade, Appearance::Lihuowang);
        let assets = world
            .resource::<AvatarAssetCache>()
            .lihuowang
            .as_ref()
            .unwrap();
        assert!(assets.parts.iter().filter(|part| !part.blade).count() >= 16);
        assert!(assets.parts.iter().any(|part| part.name.contains("face")));
        assert!(assets.parts.iter().any(|part| part.name.contains("hair")));
        assert!(
            assets
                .parts
                .iter()
                .flat_map(|part| &part.bones)
                .all(|bone| bone.source.is_none())
        );
        assert!(
            world
                .query::<(&AvatarSurface, &Mesh3d)>()
                .iter(world)
                .filter(|(surface, _)| surface.owner == owner)
                .all(|(_, mesh)| mesh.0 != first_mesh)
        );
        let counts = (
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<Image>>().len(),
            world.resource::<Assets<StandardMaterial>>().len(),
            world
                .resource::<Assets<SkinnedMeshInverseBindposes>>()
                .len(),
            world.entities().len(),
        );
        for _ in 0..3 {
            replace(world, owner, Appearance::Choulingke);
            replace_blade(world, blade, Appearance::Choulingke);
            replace(world, owner, Appearance::Lihuowang);
            replace_blade(world, blade, Appearance::Lihuowang);
        }
        assert_eq!(
            counts,
            (
                world.resource::<Assets<Mesh>>().len(),
                world.resource::<Assets<Image>>().len(),
                world.resource::<Assets<StandardMaterial>>().len(),
                world
                    .resource::<Assets<SkinnedMeshInverseBindposes>>()
                    .len(),
                world.entities().len(),
            )
        );
        let rig = world.get::<RigPose>(owner).unwrap();
        assert_eq!(rig.model, model);
        assert_eq!(rig.source.as_ref().unwrap().model.as_ptr(), source_pointer);
        assert_eq!(rig.source.as_ref().unwrap().weight, 0.7);
        assert_eq!(*world.get::<Transform>(owner).unwrap(), root_transform);
        assert_eq!(
            world.get::<Body>(owner).unwrap().world_position(),
            actor_position
        );
        assert_eq!(world.get::<ChildOf>(blade).unwrap().parent(), owner);
        assert_eq!(world.get::<ChildOf>(unrelated).unwrap().parent(), owner);
        assert_eq!(
            world
                .get::<super::super::stickman::BoneVisual>(blade)
                .unwrap()
                .owner,
            owner
        );
        world.run_schedule(Update);
        assert!(
            sample_surface(world, owner)
                .iter()
                .all(|point| point.is_finite())
        );
    }

    #[test]
    fn naraka_source_motion_keeps_its_own_segments_and_wrist_and_changes_real_vertices() {
        let mut app = test_app(false);
        let world = app.world_mut();
        let owner = world.spawn(Transform::IDENTITY).id();
        spawn_with_appearance(world, owner, Appearance::Lihuowang);
        let rest_source = world
            .resource::<SourceAnimationLibrary>()
            .rest_model
            .clone();
        world.get_mut::<RigPose>(owner).unwrap().source =
            Some(super::super::animation::SourcePose {
                model: rest_source,
                weight: 1.0,
            });
        world.run_schedule(Update);
        let rest_points = sample_surface(world, owner);
        let model = {
            let source = world.resource::<SourceAnimationLibrary>();
            source.model_pose(&source.sample(
                source.clip("skill_ylk_1").unwrap(),
                1.5,
                false,
                false,
            ))
        };
        world
            .get_mut::<RigPose>(owner)
            .unwrap()
            .source
            .as_mut()
            .unwrap()
            .model = model;
        world.run_schedule(Update);
        let moved_points = sample_surface(world, owner);
        assert!(changed(&rest_points, &moved_points) > 0.05);
        assert!(
            moved_points
                .iter()
                .all(|point| point.abs().max_element() < 5.0)
        );
        let assets = world
            .resource::<AvatarAssetCache>()
            .lihuowang
            .as_ref()
            .unwrap();
        let rig = world.get::<RigPose>(owner).unwrap();
        let source_drivers = assets.source_driver_pose(&rig.source.as_ref().unwrap().model);
        for (index, driver) in assets.drivers.iter().enumerate().skip(2) {
            let parent = driver.parent.unwrap();
            assert!(
                (source_drivers[index]
                    .translation
                    .distance(source_drivers[parent].translation)
                    - driver.anchor.distance(assets.drivers[parent].anchor))
                .abs()
                    < 0.00001
            );
        }
        let hand = assets.drivers[assets.right_hand].bone;
        let attachment = assets.attachment_pose(rig);
        assert!(
            attachment[hand]
                .translation
                .distance(source_drivers[assets.right_hand].translation)
                < 0.00001
        );
        let blade = super::super::stickman::attachment_transform(
            &super::super::stickman::Attachment::Weapon {
                bone: hand,
                mode: crate::simulation::WeaponMode::Melee,
            },
            &attachment,
        );
        assert!((blade.rotation * Vec3::Y).distance(attachment[hand].rotation * Vec3::Z) < 0.00001);
        assert_eq!(*world.get::<Transform>(owner).unwrap(), Transform::IDENTITY);
    }

    #[test]
    fn real_simulation_and_animation_drive_the_skin_through_walk_crouch_jump_attack_and_pause() {
        check_simulation_animation(Appearance::Choulingke);
    }

    #[test]
    fn naraka_skin_follows_actual_walk_crouch_jump_attack_and_pause() {
        check_simulation_animation(Appearance::Lihuowang);
    }

    fn check_simulation_animation(appearance: Appearance) {
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
        spawn_with_appearance(app.world_mut(), view, appearance);
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
