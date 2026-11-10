//! Offline-extracted character geometry, immutable shared assets and pose retargeting.
//! Original named hierarchy/channels drive matching palettes; structural anchors
//! supply missing-action/accessory fallback. Simulation remains root authority.
use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::DynamicSkinnedMeshBounds,
    mesh::{
        Indices, PrimitiveTopology, VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
};

use super::{
    ActorView,
    animation::RigPose,
    model_asset::Reader,
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
    Bailingmiao,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AvatarAppearance(pub Appearance);

#[derive(Component)]
struct AvatarPart;

#[derive(Component)]
struct BladeSurface;

#[cfg(test)]
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
    left_hand: usize,
    spine: usize,
    feet: [usize; 2],
    toes: [usize; 2],
    source_hand: usize,
    source_hand_rest: Quat,
}

/// Body sets load once and share meshes, palettes, textures and material variants.
#[derive(Resource, Default)]
pub(super) struct AvatarAssetCache {
    lihuowang: Option<AvatarAssets>,
    bailingmiao: Option<AvatarAssets>,
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
                Appearance::Bailingmiao => include_bytes!("../../assets/bailingmiao/character.bin"),
            };
            let mut reader = Reader(data);
            let format = reader.take::<8>();
            assert!(matches!(&format, b"KIVAAV02" | b"KIVAAV03"));
            let source_pose = &format == b"KIVAAV03";
            let header_length = reader.u32();
            reader.bytes(header_length); // Offline provenance; no runtime paths or JSON dependency.
            let driver_count = reader.u16();
            let (drivers, right_hand, left_hand, spine, feet, toes) = {
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
                let index_for = |name| {
                    records
                        .iter()
                        .position(|(bone, _)| *bone == library.bone(name))
                        .unwrap()
                };
                (
                    drivers,
                    index_for("hand_r"),
                    index_for("hand_l"),
                    index_for("spine_03"),
                    [index_for("foot_r"), index_for("foot_l")],
                    [index_for("ball_r"), index_for("ball_l")],
                )
            };
            let materials = super::model_asset::materials(world, &mut reader, source_pose)
                .into_iter()
                .map(|material| {
                    let mut assets = world.resource_mut::<Assets<StandardMaterial>>();
                    MaterialVariants {
                        normal: assets.add(material.clone()),
                        hit: assets.add(StandardMaterial {
                            emissive: LinearRgba::rgb(0.9, 0.5, 0.2),
                            ..material.clone()
                        }),
                        telegraph: assets.add(StandardMaterial {
                            emissive: LinearRgba::rgb(0.45, 0.24, 0.025),
                            ..material
                        }),
                    }
                })
                .collect::<Vec<_>>();
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
                left_hand,
                spine,
                feet,
                toes,
            };
            match appearance {
                Appearance::Choulingke => world.insert_resource(assets),
                Appearance::Lihuowang => {
                    world.resource_mut::<AvatarAssetCache>().lihuowang = Some(assets);
                }
                Appearance::Bailingmiao => {
                    world.resource_mut::<AvatarAssetCache>().bailingmiao = Some(assets);
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
            Appearance::Bailingmiao => cache?.bailingmiao.as_ref(),
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

    /// Solve a two-segment chain in the target body's model space. Rotation
    /// deltas retain the palette's actual source axes and inverse bind matrices.
    fn solve_limb(&self, posed: &mut [Transform], end: usize, target: Vec3, pole: Vec3) {
        let middle = self.drivers[end].parent.expect("limb has a middle driver");
        let start = self.drivers[middle].parent.expect("limb has a root driver");
        let origin = posed[start].translation;
        let upper_offset = self.drivers[middle].anchor - self.drivers[start].anchor;
        let lower_offset = self.drivers[end].anchor - self.drivers[middle].anchor;
        let upper_length = upper_offset.length();
        let lower_length = lower_offset.length();
        let direction = (target - origin).normalize_or(Vec3::NEG_Z);
        let distance = target.distance(origin).clamp(
            (upper_length - lower_length).abs() + 0.00001,
            upper_length + lower_length - 0.00001,
        );
        let along = (upper_length * upper_length - lower_length * lower_length
            + distance * distance)
            / (2. * distance);
        let bend = (pole - direction * pole.dot(direction)).normalize_or(Vec3::Y);
        let middle_position = origin
            + direction * along
            + bend * (upper_length * upper_length - along * along).max(0.).sqrt();
        let end_position = origin + direction * distance;
        posed[start].rotation = Quat::from_rotation_arc(
            (posed[start].rotation * upper_offset).normalize(),
            (middle_position - origin).normalize(),
        ) * posed[start].rotation;
        posed[middle].translation = middle_position;
        posed[middle].rotation = Quat::from_rotation_arc(
            (posed[middle].rotation * lower_offset).normalize(),
            (end_position - middle_position).normalize(),
        ) * posed[middle].rotation;
        posed[end].translation = end_position;
    }

    /// Companion grips use this body's female structural reference. At steep
    /// elevation, adjust the relaxed stance to reach the fixed stock while both
    /// feet stay planted; leg and arm solves preserve the target segment lengths.
    fn posed_drivers(&self, rig: &RigPose) -> Vec<Transform> {
        let mut posed = self.driver_pose(&rig.model);
        if let Some(targets) = rig.companion_hands {
            let hands = [self.right_hand, self.left_hand];
            // The short clavicle segment can shrug toward a high stock before
            // asking the legs to lower the torso. Its length stays unchanged.
            for (wrist, target) in hands.into_iter().zip(targets) {
                let elbow = self.drivers[wrist].parent.unwrap();
                let upper = self.drivers[elbow].parent.unwrap();
                let clavicle = self.drivers[upper].parent.unwrap();
                let reach = self.drivers[wrist]
                    .anchor
                    .distance(self.drivers[elbow].anchor)
                    + self.drivers[elbow]
                        .anchor
                        .distance(self.drivers[upper].anchor)
                    - 0.0001;
                if posed[upper].translation.distance(target) > reach {
                    let offset = self.drivers[upper].anchor - self.drivers[clavicle].anchor;
                    let toward_target = target - posed[clavicle].translation;
                    let distance = toward_target.length().max(0.0001);
                    let direction = toward_target / distance;
                    let current = (posed[clavicle].rotation * offset).normalize();
                    let angle = current.angle_between(direction);
                    // The least law-of-cosines rotation starts at zero when the
                    // grip crosses reach, keeping shoulder/sleeve motion continuous.
                    let allowed = ((distance * distance + offset.length_squared() - reach * reach)
                        / (2. * distance * offset.length()))
                    .clamp(-1., 1.)
                    .acos();
                    let weight = ((angle - allowed) / angle.max(0.0001)).clamp(0., 1.);
                    let correction =
                        Quat::IDENTITY.slerp(Quat::from_rotation_arc(current, direction), weight);
                    posed[clavicle].rotation = correction * posed[clavicle].rotation;
                    posed[upper].translation =
                        posed[clavicle].translation + posed[clavicle].rotation * offset;
                }
            }
            let (mut minimum, mut maximum) = (f32::NEG_INFINITY, f32::INFINITY);
            for (wrist, target) in hands.into_iter().zip(targets) {
                let elbow = self.drivers[wrist].parent.unwrap();
                let upper = self.drivers[elbow].parent.unwrap();
                let reach = self.drivers[wrist]
                    .anchor
                    .distance(self.drivers[elbow].anchor)
                    + self.drivers[elbow]
                        .anchor
                        .distance(self.drivers[upper].anchor)
                    - 0.0001;
                let delta = posed[upper].translation - target;
                let available_y = (reach * reach - delta.xz().length_squared()).max(0.).sqrt();
                minimum = minimum.max(-delta.y - available_y);
                maximum = maximum.min(-delta.y + available_y);
            }
            // A downward shot may stand up from the relaxed CC0 pose. Bound
            // that rise by both real leg lengths so feet never lift or stretch.
            let maximum_rise = self
                .feet
                .into_iter()
                .map(|foot| {
                    let knee = self.drivers[foot].parent.unwrap();
                    let hip = self.drivers[knee].parent.unwrap();
                    let reach = self.drivers[foot]
                        .anchor
                        .distance(self.drivers[knee].anchor)
                        + self.drivers[knee].anchor.distance(self.drivers[hip].anchor)
                        - 0.0001;
                    let delta = posed[hip].translation - posed[foot].translation;
                    (reach * reach - delta.xz().length_squared()).max(0.).sqrt() - delta.y
                })
                .fold(f32::INFINITY, f32::min)
                .max(0.);
            let vertical = if minimum > 0. {
                minimum
            } else {
                maximum.min(0.)
            }
            .clamp(-0.50, maximum_rise);
            if vertical.abs() > 0.000001 {
                // Root->pelvis translation is a pose control, never actor motion.
                let feet = self
                    .toes
                    .into_iter()
                    .zip(self.feet)
                    .map(|(toe, foot)| {
                        let knee = self.drivers[foot].parent.unwrap();
                        let hip = self.drivers[knee].parent.unwrap();
                        (
                            toe,
                            foot,
                            posed[toe],
                            posed[foot],
                            posed[knee].translation - posed[hip].translation,
                        )
                    })
                    .collect::<Vec<_>>();
                for driver in posed.iter_mut().skip(1) {
                    driver.translation.y += vertical;
                }
                for (toe, foot, toe_pose, foot_pose, pole) in feet {
                    self.solve_limb(&mut posed, foot, foot_pose.translation, pole);
                    posed[foot].rotation = foot_pose.rotation;
                    posed[toe] = toe_pose;
                }
            }
            for (wrist, target, side) in [
                (self.right_hand, targets[0], 1.),
                (self.left_hand, targets[1], -1.),
            ] {
                self.solve_limb(&mut posed, wrist, target, Vec3::new(side, -1., 0.3));
            }
        }
        posed
    }

    fn pose_deltas(&self, rig: &RigPose) -> Vec<Mat4> {
        self.posed_drivers(rig)
            .iter()
            .zip(&self.drivers)
            .map(|(posed, driver)| posed.to_matrix() * Mat4::from_translation(-driver.anchor))
            .collect()
    }

    #[cfg(test)]
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
        if self.appearance == Appearance::Choulingke {
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
        if rig.reference_pose {
            // AV03 preserves the imported source rest separately from IBM.
            return Transform::from_matrix(bone.bind);
        }
        let fallback = Transform::from_matrix(deltas[bone.driver] * bone.bind);
        rig.source.as_ref().map_or(fallback, |source| {
            let matrix = if self.appearance != Appearance::Choulingke {
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

    /// Converted accessory vertices use body axes, so apply the spine's motion
    /// delta rather than the CC0 skeleton's authored bone-axis rotation.
    pub(super) fn body_mount(&self, rig: &RigPose) -> Transform {
        self.evaluated_drivers(rig)[self.spine]
    }

    /// Semantic body axes evaluated from the very same rest and motion deltas
    /// as the skin. Source and structural previews therefore expose the actual
    /// left hand and spine instead of unrelated CC0 reference positions.
    pub(super) fn evaluated_drivers(&self, rig: &RigPose) -> Vec<Transform> {
        let fallback = if rig.reference_pose {
            self.drivers
                .iter()
                .map(|driver| Transform::from_translation(driver.anchor))
                .collect()
        } else {
            self.posed_drivers(rig)
        };
        let Some(source) = rig.source.as_ref().filter(|_| !rig.reference_pose) else {
            return fallback;
        };
        let source_pose = self.source_driver_pose(&source.model);
        fallback
            .iter()
            .zip(source_pose)
            .enumerate()
            .map(|(index, (fallback, mut posed))| {
                if self.appearance == Appearance::Choulingke
                    && let Some(source_index) = self.drivers[index].source
                {
                    let matrix = source.model[source_index];
                    posed.translation = matrix.w_axis.xyz();
                    posed.rotation =
                        Transform::from_matrix(matrix * self.drivers[index].source_inverse_rest)
                            .rotation;
                }
                blend(*fallback, posed, source.weight)
            })
            .collect()
    }

    pub(super) fn structural_bones(&self, rig: &RigPose) -> Vec<(Option<usize>, Transform)> {
        self.drivers
            .iter()
            .zip(self.evaluated_drivers(rig))
            .map(|(driver, pose)| (driver.parent, pose))
            .collect()
    }

    pub(super) fn attachment_pose(&self, rig: &RigPose) -> Vec<Transform> {
        let drivers = self.evaluated_drivers(rig);
        let mut result = rig.model.clone();
        if rig.reference_pose {
            for (driver, posed) in self.drivers.iter().zip(&drivers) {
                result[driver.bone] = Transform::from_translation(posed.translation)
                    .with_rotation(driver.reference.rotation);
            }
            return result;
        }
        let fallback_drivers = self.posed_drivers(rig);
        if self.appearance == Appearance::Bailingmiao {
            for (driver, posed) in self.drivers.iter().zip(&fallback_drivers) {
                result[driver.bone] = Transform::from_translation(posed.translation)
                    .with_rotation(posed.rotation * driver.reference.rotation);
            }
        }
        // Keep the established right-hand path below byte-for-byte equivalent;
        // expose the other semantic joints without introducing viewer offsets.
        for (index, driver) in self.drivers.iter().enumerate() {
            if index == self.right_hand {
                continue;
            }
            result[driver.bone].translation = drivers[index].translation;
            if rig.source.is_some() {
                result[driver.bone].rotation = drivers[index].rotation * driver.reference.rotation;
            }
        }
        let hand = &self.drivers[self.right_hand];
        // Preserve the authored hand orientation used by weapon corrections and
        // ranged aiming; only align its attachment position to the retargeted hand.
        result[hand.bone].translation = fallback_drivers[self.right_hand].translation;
        if let Some(source) = &rig.source {
            let source_hand = if self.appearance != Appearance::Choulingke {
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
    let deltas = assets.pose_deltas(rig);
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
    #[cfg(test)]
    world.entity_mut(parent).insert(BladeAppearance(appearance));
}

#[cfg(test)]
fn replace_blade(world: &mut World, parent: Entity, appearance: Appearance) {
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
                selected.pose_deltas(pose),
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
        MoveIntent, SimulationActive, SimulationPlugin, SpawnPoint, WeaponPair,
    };
    use bevy::image::{ImageAddressMode, ImageSampler};
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
    fn source_semantic_hands_and_spine_match_actual_skin_at_each_blend_weight() {
        let mut app = test_app(false);
        let world = app.world_mut();
        for appearance in [
            Appearance::Choulingke,
            Appearance::Lihuowang,
            Appearance::Bailingmiao,
        ] {
            let owner = world.spawn(Transform::IDENTITY).id();
            spawn_with_appearance(world, owner, appearance);
            let assets = AvatarAssets::cached(
                world.get_resource::<AvatarAssets>(),
                world.get_resource::<AvatarAssetCache>(),
                appearance,
            )
            .unwrap()
            .clone();
            let library = world.resource::<SourceAnimationLibrary>();
            let clip = library.clip("skill_ylk_1").unwrap();
            let source_model = library.model_pose(&library.sample(clip, 1.35, false, false));
            for weight in [0.0, 0.5, 1.0] {
                let mut rig = RigPose::new(world.resource::<AnimationLibrary>());
                rig.source = Some(super::super::animation::SourcePose {
                    model: source_model.clone(),
                    weight,
                });
                let semantic = assets.attachment_pose(&rig);
                let deltas = assets.pose_deltas(&rig);
                let source_deltas = assets.source_deltas(&rig);
                for index in [assets.right_hand, assets.left_hand, assets.spine] {
                    let driver = &assets.drivers[index];
                    let bone = assets
                        .parts
                        .iter()
                        .filter(|part| !part.blade)
                        .flat_map(|part| &part.bones)
                        .filter(|bone| bone.driver == index)
                        .min_by(|a, b| {
                            a.bind
                                .w_axis
                                .xyz()
                                .distance_squared(driver.anchor)
                                .total_cmp(&b.bind.w_axis.xyz().distance_squared(driver.anchor))
                        })
                        .unwrap();
                    let local_anchor = bone.bind.inverse().transform_point3(driver.anchor);
                    let palette = assets.joint_pose(&rig, &deltas, source_deltas.as_deref(), bone);
                    let actual = palette.transform_point(local_anchor);
                    assert!(
                        semantic[driver.bone].translation.distance(actual) < 0.0003,
                        "{appearance:?} driver {index} weight {weight}: semantic {:?}, skin {:?}",
                        semantic[driver.bone].translation,
                        actual
                    );
                }
                // Existing weapon orientation/offset must remain identical to
                // the previously established right-wrist path.
                let fallback = assets.posed_drivers(&rig);
                let hand = &assets.drivers[assets.right_hand];
                let mut previous = rig.model[hand.bone];
                if appearance == Appearance::Bailingmiao {
                    previous.rotation =
                        fallback[assets.right_hand].rotation * hand.reference.rotation;
                    previous.scale = Vec3::ONE;
                }
                previous.translation = fallback[assets.right_hand].translation;
                let source_hand = if appearance == Appearance::Choulingke {
                    let posed = Transform::from_matrix(source_model[assets.source_hand]);
                    Transform::from_translation(posed.translation).with_rotation(
                        posed.rotation
                            * assets.source_hand_rest.inverse()
                            * hand.reference.rotation,
                    )
                } else {
                    let posed = assets.source_driver_pose(&source_model)[assets.right_hand];
                    Transform::from_translation(posed.translation)
                        .with_rotation(posed.rotation * hand.reference.rotation)
                };
                assert_eq!(semantic[hand.bone], blend(previous, source_hand, weight));
            }
            let mut rig = RigPose::new(world.resource::<AnimationLibrary>());
            rig.reference_pose = true;
            let deltas = assets.pose_deltas(&rig);
            for bone in assets
                .parts
                .iter()
                .filter(|part| !part.blade)
                .flat_map(|part| &part.bones)
            {
                assert_eq!(
                    assets.joint_pose(&rig, &deltas, None, bone),
                    Transform::from_matrix(bone.bind)
                );
            }
            world.despawn(owner);
        }
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
    fn bailingmiao_uses_female_palettes_and_real_weapon_grips_without_stretching_arms() {
        use crate::client::weapon_models::WeaponModel;
        use crate::simulation::CompanionState;
        let mut app = test_app(false);
        let world = app.world_mut();
        let owner = world.spawn(Transform::IDENTITY).id();
        let surfaces = spawn_with_appearance(world, owner, Appearance::Bailingmiao);
        assert!(
            surfaces.len() >= 18,
            "outfit, Lianmu and female face surfaces"
        );
        let female = AvatarAssets::shared(world, Appearance::Bailingmiao);
        let male = AvatarAssets::shared(world, Appearance::Lihuowang);
        assert_eq!(female.drivers.len(), 23);
        assert!(female.parts.iter().all(|part| !part.blade));
        assert!(
            female
                .parts
                .iter()
                .flat_map(|part| &part.bones)
                .all(|bone| bone.source.is_none())
        );
        assert!(
            female
                .drivers
                .iter()
                .zip(&male.drivers)
                .any(|(female, male)| (female.anchor - male.anchor).length() > 0.001),
            "own female structural reference"
        );
        let resting = sample_surface(world, owner);
        let mut poses = Vec::new();
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            let model = WeaponModel::shared(world, pair, crate::simulation::WeaponMode::Ranged);
            for pitch in [0_f32, 0.35, -0.25, 0.8, 1.0, -0.8] {
                let source = Vec3::new(0., 1.2, -0.55);
                let grips = model
                    .grips
                    .map(|grip| source + Quat::from_rotation_x(pitch) * grip);
                let state = CompanionState {
                    pair,
                    elapsed: 0.31,
                    shot_count: 1,
                    ..default()
                };
                world.resource_scope(|world, library: Mut<AnimationLibrary>| {
                    world
                        .get_mut::<RigPose>(owner)
                        .unwrap()
                        .sample_companion(&library, &state, pitch, grips);
                });
                world.run_schedule(Update);
                let rig = world.get::<RigPose>(owner).unwrap();
                let drivers = female.posed_drivers(rig);
                let ground_pose = female.driver_pose(&rig.model);
                for foot in female.feet {
                    assert!(
                        drivers[foot]
                            .translation
                            .distance(ground_pose[foot].translation)
                            < 0.0001
                    );
                    assert!(
                        drivers[foot]
                            .rotation
                            .angle_between(ground_pose[foot].rotation)
                            < 0.0001
                    );
                }
                for (index, driver) in female.drivers.iter().enumerate().skip(2) {
                    let parent = driver.parent.unwrap();
                    assert!(
                        (drivers[index]
                            .translation
                            .distance(drivers[parent].translation)
                            - driver.anchor.distance(female.drivers[parent].anchor))
                        .abs()
                            < 0.0001,
                        "{pair:?} pitch={pitch} structural segment={index}"
                    );
                }
                for (upper, elbow, wrist, target) in [(12, 13, 14, grips[0]), (8, 9, 10, grips[1])]
                {
                    assert!(
                        drivers[wrist].translation.distance(target) < 0.0001,
                        "{pair:?} pitch={pitch} wrist={wrist}: {:?} target={target:?}",
                        drivers[wrist].translation
                    );
                    for (a, b) in [(upper, elbow), (elbow, wrist)] {
                        assert!(
                            (drivers[a].translation.distance(drivers[b].translation)
                                - female.drivers[a].anchor.distance(female.drivers[b].anchor))
                            .abs()
                                < 0.0001
                        );
                    }
                    // Check the actual palette entity used by SkinnedMesh, not
                    // just the unrendered RigPose's intended hand coordinate.
                    let (joint, palette) = world
                        .query::<(Entity, &PaletteJoint)>()
                        .iter(world)
                        .find(|(_, joint)| joint.owner == owner && joint.bone.driver == wrist)
                        .unwrap();
                    let reference = palette
                        .bone
                        .bind
                        .inverse()
                        .transform_point3(female.drivers[wrist].anchor);
                    assert!(
                        world
                            .get::<Transform>(joint)
                            .unwrap()
                            .transform_point(reference)
                            .distance(target)
                            < 0.0001
                    );
                }
                let moved = sample_surface(world, owner);
                assert!(changed(&resting, &moved) > 0.02);
                assert!(
                    moved
                        .iter()
                        .all(|point| point.is_finite() && point.abs().max_element() < 4.)
                );
                poses.push(moved);
            }
        }
        assert!(changed(&poses[0], &poses[6]) > 0.02);
        // Sweep across reach thresholds. A barely unreachable grip must not
        // snap the clavicle by its full length between adjacent aim samples.
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            let model = WeaponModel::shared(world, pair, crate::simulation::WeaponMode::Ranged);
            let mut previous: Option<Vec<Transform>> = None;
            for step in -200..=200 {
                let pitch = step as f32 * 0.005;
                let grips = model
                    .grips
                    .map(|grip| Vec3::new(0., 1.2, -0.55) + Quat::from_rotation_x(pitch) * grip);
                let state = CompanionState {
                    pair,
                    elapsed: 0.31,
                    ..default()
                };
                world.resource_scope(|world, library: Mut<AnimationLibrary>| {
                    world
                        .get_mut::<RigPose>(owner)
                        .unwrap()
                        .sample_companion(&library, &state, pitch, grips);
                });
                let posed = female.posed_drivers(world.get::<RigPose>(owner).unwrap());
                for (hand, target) in [female.right_hand, female.left_hand].into_iter().zip(grips) {
                    assert!(
                        posed[hand].translation.distance(target) < 0.0001,
                        "swept {pair:?} pitch={pitch} hand={hand}"
                    );
                }
                if let Some(previous) = previous {
                    for index in [1, 7, 8, 11, 12] {
                        assert!(
                            posed[index]
                                .translation
                                .distance(previous[index].translation)
                                < 0.025,
                            "continuous {pair:?} pitch={pitch} driver={index}"
                        );
                    }
                }
                previous = Some(posed);
            }
        }
        world.run_schedule(Update);
        let paused = sample_surface(world, owner);
        world.run_schedule(Update);
        assert_eq!(sample_surface(world, owner), paused);
        let counts = (
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<Image>>().len(),
            world.resource::<Assets<StandardMaterial>>().len(),
        );
        let other = world.spawn(Transform::IDENTITY).id();
        spawn_with_appearance(world, other, Appearance::Bailingmiao);
        assert_eq!(
            (
                world.resource::<Assets<Mesh>>().len(),
                world.resource::<Assets<Image>>().len(),
                world.resource::<Assets<StandardMaterial>>().len()
            ),
            counts
        );
    }

    #[test]
    fn actual_hammer_and_chain_combos_keep_female_skin_grips_sources_and_feet_aligned() {
        use crate::{
            client::{
                animation, combat,
                companion::{self, CompanionView, SisterWeapon},
                controls::ViewYaw,
                scene,
                weapon_models::WeaponModel,
            },
            gameplay::Player,
            simulation::{CombatEffect, CombatFeedback, CompanionState},
        };
        fn tick(app: &mut App, count: usize) {
            for _ in 0..count {
                app.world_mut()
                    .resource_mut::<Time<Fixed>>()
                    .advance_by(Duration::from_secs_f64(1. / 60.));
                app.world_mut().run_schedule(FixedUpdate);
            }
        }
        fn request(app: &mut App, player: Entity, command: impl FnOnce(&mut CombatIntent)) {
            command(&mut app.world_mut().get_mut::<CombatIntent>(player).unwrap());
            tick(app, 1);
        }
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, SimulationPlugin))
                .init_resource::<Assets<Mesh>>()
                .init_resource::<Assets<StandardMaterial>>()
                .init_resource::<AnimationLibrary>()
                .init_resource::<ViewYaw>()
                .add_systems(
                    Startup,
                    (
                        crate::gameplay::setup,
                        scene::setup,
                        combat::setup,
                        companion::setup,
                    )
                        .chain(),
                )
                .add_systems(
                    Update,
                    (
                        scene::sync_player,
                        animation::animate,
                        companion::sync,
                        sync,
                    )
                        .chain(),
                );
            app.finish();
            app.update();
            tick(&mut app, 12);
            let player = app
                .world_mut()
                .query_filtered::<Entity, With<Player>>()
                .single(app.world())
                .unwrap();
            request(&mut app, player, CombatIntent::request_toggle_target);
            if pair == WeaponPair::HammerMusket {
                request(&mut app, player, CombatIntent::request_melee_skill);
                tick(&mut app, 13);
            } else {
                request(&mut app, player, |intent| intent.request_pair(pair));
                request(&mut app, player, CombatIntent::request_ranged_skill);
                tick(&mut app, 19);
            }
            request(&mut app, player, CombatIntent::request_combo);
            let sequence = app
                .world()
                .get::<Combatant>(player)
                .unwrap()
                .sequence()
                .expect("real committed combo");
            assert_eq!(sequence.pair, pair);
            let mut highest_pitch = 0_f32;
            for _ in 0..65 {
                let before = app.world().get::<Body>(player).unwrap().world_position();
                app.world_mut().run_schedule(Update);
                assert_eq!(
                    app.world().get::<Body>(player).unwrap().world_position(),
                    before
                );
                let world = app.world_mut();
                let (root, performer) = world
                    .query::<(Entity, &CompanionView)>()
                    .iter(world)
                    .find(|(_, view)| view.owner == player)
                    .map(|(entity, view)| (entity, view.performer))
                    .unwrap();
                let state = *world.get::<CompanionState>(performer).unwrap();
                if state.visible && state.action_id == sequence.action_id {
                    let model =
                        WeaponModel::shared(world, pair, crate::simulation::WeaponMode::Ranged);
                    let root_transform = *world.get::<Transform>(root).unwrap();
                    let weapon = *world
                        .query::<(&SisterWeapon, &Transform)>()
                        .iter(world)
                        .find(|(weapon, _)| weapon.root == root && weapon.pair == pair)
                        .unwrap()
                        .1;
                    let source = root_transform.transform_point(weapon.translation);
                    let expected =
                        state.source_position + (root_transform.translation - state.position);
                    assert!(source.distance(expected) < 0.0001);
                    let forward = root_transform.rotation * weapon.rotation * Vec3::NEG_Z;
                    let aim = (state.aim_position - expected).normalize();
                    assert!(forward.distance(aim) < 0.0001);
                    highest_pitch = highest_pitch.max(aim.y.atan2(aim.xz().length()));
                    let female = AvatarAssets::cached(
                        world.get_resource::<AvatarAssets>(),
                        world.get_resource::<AvatarAssetCache>(),
                        Appearance::Bailingmiao,
                    )
                    .unwrap()
                    .clone();
                    let rig = world.get::<RigPose>(root).unwrap();
                    let posed = female.posed_drivers(rig);
                    let ground = female.driver_pose(&rig.model);
                    for foot in female.feet {
                        assert!(
                            posed[foot].translation.distance(ground[foot].translation) < 0.0001
                        );
                    }
                    for (wrist, grip) in [female.right_hand, female.left_hand]
                        .into_iter()
                        .zip(model.grips)
                    {
                        let target = weapon.transform_point(grip);
                        assert!(
                            posed[wrist].translation.distance(target) < 0.0001,
                            "{pair:?} elapsed={} wrist={wrist} gap={}",
                            state.elapsed,
                            posed[wrist].translation.distance(target)
                        );
                        let (joint, palette) = world
                            .query::<(Entity, &PaletteJoint)>()
                            .iter(world)
                            .find(|(_, joint)| joint.owner == root && joint.bone.driver == wrist)
                            .unwrap();
                        let point = palette
                            .bone
                            .bind
                            .inverse()
                            .transform_point3(female.drivers[wrist].anchor);
                        assert!(
                            world
                                .get::<Transform>(joint)
                                .unwrap()
                                .transform_point(point)
                                .distance(target)
                                < 0.0001
                        );
                    }
                }
                tick(&mut app, 1);
            }
            let events = app
                .world()
                .resource::<CombatFeedback>()
                .events()
                .filter(|event| {
                    event.kind == CombatEffect::Shot && event.action_id == sequence.action_id
                })
                .collect::<Vec<_>>();
            assert_eq!(events.len(), 3, "each real three-shot sequence completed");
            if pair == WeaponPair::HammerMusket {
                assert!(
                    highest_pitch > 0.3,
                    "actual Hammer E airborne aim was exercised"
                );
            }
            app.world_mut().run_schedule(Update);
            let visibility = app
                .world_mut()
                .query_filtered::<&Visibility, With<CompanionView>>()
                .single(app.world())
                .unwrap();
            assert_eq!(*visibility, Visibility::Hidden);
        }
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
                Combatant::new(WeaponPair::HammerMusket),
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
