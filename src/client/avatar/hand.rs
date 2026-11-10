//! Named, anatomical finger transfer. Palette data and inverse binds stay intact.
use super::super::animation::SourcePose;
use super::{Part, SourceAnimationLibrary, SourceBone};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
struct Segment {
    source: usize,
    own_local: Transform,
    source_frame_in_joint: Quat,
    own_frame_in_joint: Quat,
}

/// The three weighted joints are explicit chains. The donor's unweighted Nub
/// supplies only the distal forward direction; it is never another target bone.
#[derive(Clone)]
pub(super) struct FingerBinding {
    pub(super) hand_rest: Mat4,
    source_palm: [usize; 4],
    own_palm_in_hand: Quat,
    segments: Vec<Segment>,
}

/// Anatomical +X follows palm length, +Y across index->little, +Z is its normal.
fn palm_frame(points: [Vec3; 4]) -> Option<Quat> {
    let forward = (points[2] - points[0]).try_normalize()?;
    let across = points[3] - points[1];
    let normal = forward.cross(across).try_normalize()?;
    let across = normal.cross(forward).try_normalize()?;
    Some(Quat::from_mat3(&Mat3::from_cols(forward, across, normal)).normalize())
}

fn finger_frame(forward: Vec3, normal: Vec3) -> Option<Quat> {
    let forward = forward.try_normalize()?;
    let normal = (normal - forward * normal.dot(forward)).try_normalize()?;
    let across = normal.cross(forward).try_normalize()?;
    Some(Quat::from_mat3(&Mat3::from_cols(forward, across, normal)).normalize())
}

fn native_name(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

fn bone_index(bones: &[SourceBone], name: &str) -> Option<usize> {
    bones
        .iter()
        .position(|bone| native_name(&bone.name) == name)
}

/// Merge only matching native-rest metadata, not palette indices. Some skins
/// omit an unweighted parent although another surface retains the same joint.
pub(super) fn bind_parts(parts: &mut [Part], library: &SourceAnimationLibrary) {
    let mut rest: HashMap<String, Option<Mat4>> = HashMap::new();
    for part in parts.iter().filter(|part| !part.blade) {
        for bone in &part.bones {
            let value = rest
                .entry(native_name(&bone.name).to_owned())
                .or_insert(Some(bone.bind));
            if value.is_some_and(|previous| !previous.abs_diff_eq(bone.bind, 0.0001)) {
                *value = None;
            }
        }
    }
    for part in parts.iter_mut().filter(|part| !part.blade) {
        bind(&mut part.bones, &rest, library);
    }
}

/// Unsupported or incomplete names keep their original rigid-hand fallback.
/// Matching is semantic, never a palette index from another character.
fn bind(
    bones: &mut [SourceBone],
    rest: &HashMap<String, Option<Mat4>>,
    library: &SourceAnimationLibrary,
) {
    let target_rest = |name: &str| {
        bone_index(bones, name)
            .map(|index| bones[index].bind)
            .or_else(|| rest.get(name).copied().flatten())
    };
    let mut bindings = Vec::new();
    for side in ["L", "R"] {
        let target_names = [
            format!("gMan {side} Hand"),
            format!("gMan {side} Finger1"),
            format!("gMan {side} Finger2"),
            format!("gMan {side} Finger4"),
        ];
        let source_names = [
            format!("Bip001 {side} Hand"),
            format!("Bip001 {side} Finger1"),
            format!("Bip001 {side} Finger2"),
            format!("Bip001 {side} Finger4"),
        ];
        let Some(target_matrices) = target_names
            .iter()
            .map(|name| target_rest(name))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let Some(source_indices) = source_names
            .iter()
            .map(|name| library.bone(name))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let source_palm: [usize; 4] = source_indices.try_into().unwrap();
        let target_palm: [Mat4; 4] = target_matrices.try_into().unwrap();
        let target_hand = target_palm[0];
        let Some(own_frame) = palm_frame(target_palm.map(|matrix| matrix.w_axis.xyz())) else {
            continue;
        };
        let Some(source_frame) =
            palm_frame(source_palm.map(|index| library.rest_model[index].w_axis.xyz()))
        else {
            continue;
        };
        let own_palm_in_hand = Transform::from_matrix(target_hand).rotation.inverse() * own_frame;
        for digit in 0..5 {
            let names = [
                format!("gMan {side} Finger{digit}"),
                format!("gMan {side} Finger{digit}1"),
                format!("gMan {side} Finger{digit}2"),
            ];
            let Some(targets) = names
                .iter()
                .map(|name| target_rest(name))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let source_names = [
                format!("Bip001 {side} Finger{digit}"),
                format!("Bip001 {side} Finger{digit}1"),
                format!("Bip001 {side} Finger{digit}2"),
                format!("Bip001 {side} Finger{digit}Nub"),
            ];
            let Some(sources) = source_names
                .iter()
                .map(|name| library.bone(name))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let own_points: Vec<_> = targets.iter().map(|matrix| matrix.w_axis.xyz()).collect();
            let source_points: Vec<_> = sources
                .iter()
                .map(|&index| library.rest_model[index].w_axis.xyz())
                .collect();
            let flex_normal = |points: &[Vec3], palm: Quat| {
                let palm_normal = palm * Vec3::Z;
                if digit != 0 {
                    return palm_normal;
                }
                // A thumb bends in its own observed plane, not the four-finger
                // plane. Orient both own/source plane normals consistently.
                let Some(mut normal) = (points[1] - points[0])
                    .cross(points[2] - points[1])
                    .try_normalize()
                else {
                    return palm_normal;
                };
                if normal.dot(palm_normal) < 0.0 {
                    normal = -normal;
                }
                normal
            };
            let own_normal = flex_normal(&own_points, own_frame);
            let source_normal = flex_normal(&source_points, source_frame);
            let mut segments = Vec::new();
            for link in 0..3 {
                let bind = targets[link];
                let parent = if link == 0 {
                    target_hand
                } else {
                    targets[link - 1]
                };
                let source_bind = library.rest_model[sources[link]];
                let source_forward = source_points[link + 1] - source_points[link];
                let own_forward = if link < 2 {
                    own_points[link + 1] - own_points[link]
                } else {
                    // Both actual gMan palettes put the preceding child links
                    // on native +X; distal-weighted vertices extend +X too.
                    // This defines direction only, without inventing a Nub length.
                    Transform::from_matrix(bind).rotation * Vec3::X
                };
                let (Some(source_anatomy), Some(own_anatomy)) = (
                    finger_frame(source_forward, source_normal),
                    finger_frame(own_forward, own_normal),
                ) else {
                    segments.clear();
                    break;
                };
                segments.push(Segment {
                    source: sources[link],
                    own_local: Transform::from_matrix(parent.inverse() * bind),
                    source_frame_in_joint: Transform::from_matrix(source_bind).rotation.inverse()
                        * source_anatomy,
                    own_frame_in_joint: Transform::from_matrix(bind).rotation.inverse()
                        * own_anatomy,
                });
            }
            if segments.len() != 3 {
                continue;
            }
            for link in 0..3 {
                if let Some(index) = bone_index(bones, &names[link]) {
                    bindings.push((
                        index,
                        FingerBinding {
                            hand_rest: target_hand,
                            source_palm,
                            own_palm_in_hand,
                            segments: segments[..=link].to_vec(),
                        },
                    ));
                }
            }
        }
    }
    for (index, binding) in bindings {
        bones[index].finger = Some(binding);
    }
}

impl FingerBinding {
    pub(super) fn pose(&self, source: &SourcePose, hand: Transform) -> Option<Transform> {
        let source_palm = palm_frame(
            self.source_palm
                .map(|index| source.model[index].w_axis.xyz()),
        )?;
        let source_to_own_hand = self.own_palm_in_hand * source_palm.inverse();
        let mut parent_source_rotation = Quat::IDENTITY;
        let mut result = hand.to_matrix();
        for segment in &self.segments {
            let source_rotation = Transform::from_matrix(source.model[segment.source]).rotation;
            let desired_in_hand = source_to_own_hand
                * source_rotation
                * segment.source_frame_in_joint
                * segment.own_frame_in_joint.inverse();
            let desired_local = (parent_source_rotation.inverse() * desired_in_hand).normalize();
            let mut local = segment.own_local;
            // Weight local rotation before FK: world-joint position lerp would
            // shorten a bent finger at intermediate source weights.
            local.rotation = local
                .rotation
                .slerp(desired_local, source.weight)
                .normalize();
            result *= local.to_matrix();
            parent_source_rotation = desired_in_hand;
        }
        Some(Transform::from_matrix(result))
    }
}

/// Native hand axes and semantic grip axes are distinct. This reference is
/// derived from actual wrist/MCP geometry, not another rig's native quaternion.
#[derive(Clone)]
pub(super) struct GripReference {
    pub(super) bone: SourceBone,
    pub(super) frame: Transform,
    source_palm: [usize; 4],
    pub(super) palm_bones: [SourceBone; 4],
}

fn grip_rotation(palm: Quat) -> Quat {
    // Existing attachment contract: +Y walks into the palm; +Z is the melee
    // shaft after the shared Rx90 model-axis correction. The head points
    // toward the index/thumb side, and the butt toward the little finger.
    Quat::from_mat3(&Mat3::from_cols(
        -(palm * Vec3::Z),
        palm * Vec3::X,
        -(palm * Vec3::Y),
    ))
    .normalize()
}

pub(super) fn references(
    parts: &[Part],
    original: bool,
    library: &SourceAnimationLibrary,
) -> [Option<GripReference>; 2] {
    ["R", "L"].map(|side| {
        let prefix = if original { "Bip001" } else { "gMan" };
        let names = [
            format!("{prefix} {side} Hand"),
            format!("{prefix} {side} Finger1"),
            format!("{prefix} {side} Finger2"),
            format!("{prefix} {side} Finger4"),
        ];
        let bones: Vec<_> = names
            .iter()
            .map(|name| {
                parts
                    .iter()
                    .filter(|part| !part.blade)
                    .flat_map(|part| &part.bones)
                    .find(|bone| native_name(&bone.name) == name)
            })
            .collect::<Option<Vec<_>>>()?;
        let palm = palm_frame([
            bones[0].bind.w_axis.xyz(),
            bones[1].bind.w_axis.xyz(),
            bones[2].bind.w_axis.xyz(),
            bones[3].bind.w_axis.xyz(),
        ])?;
        let source_names = [
            format!("Bip001 {side} Hand"),
            format!("Bip001 {side} Finger1"),
            format!("Bip001 {side} Finger2"),
            format!("Bip001 {side} Finger4"),
        ];
        let source_palm: Vec<_> = source_names
            .iter()
            .map(|name| library.bone(name))
            .collect::<Option<_>>()?;
        Some(GripReference {
            bone: bones[0].clone(),
            frame: Transform::from_translation(bones[0].bind.w_axis.xyz())
                .with_rotation(grip_rotation(palm)),
            source_palm: source_palm.try_into().unwrap(),
            palm_bones: [
                bones[0].clone(),
                bones[1].clone(),
                bones[2].clone(),
                bones[3].clone(),
            ],
        })
    })
}

impl GripReference {
    pub(super) fn source_rotation(&self, model: &[Mat4]) -> Option<Quat> {
        palm_frame(self.source_palm.map(|index| model[index].w_axis.xyz())).map(grip_rotation)
    }
}

/// Current palette anchors also handle an original source crossfade whose
/// global MCP interpolation is not exactly a rigid wrist transformation.
pub(super) fn current_grip(points: [Vec3; 4]) -> Option<Transform> {
    palm_frame(points)
        .map(|palm| Transform::from_translation(points[0]).with_rotation(grip_rotation(palm)))
}
