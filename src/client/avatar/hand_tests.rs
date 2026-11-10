//! Independent geometric oracles for the actual bundled hand palettes and skin.
use super::*;
use crate::client::animation::SourcePose;

fn bundle(appearance: Appearance) -> (World, AvatarAssets) {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<AnimationLibrary>();
    let assets = AvatarAssets::shared(&mut world, appearance);
    (world, assets)
}

fn bone<'a>(assets: &'a AvatarAssets, name: &str) -> &'a SourceBone {
    assets
        .parts
        .iter()
        .filter(|part| !part.blade)
        .flat_map(|part| &part.bones)
        .find(|bone| bone.name.rsplit("::").next() == Some(name))
        .unwrap()
}

fn matrix(assets: &AvatarAssets, rig: &RigPose, name: &str) -> Mat4 {
    assets
        .joint_pose(
            rig,
            &assets.pose_deltas(rig),
            assets.source_deltas(rig).as_deref(),
            bone(assets, name),
        )
        .to_matrix()
}

/// This oracle derives a frame from actual joint positions, independently of
/// the cached retarget calibration. X=length, Y=index->little, Z=palm normal.
fn palm(hand: Vec3, index: Vec3, middle: Vec3, little: Vec3) -> Quat {
    let x = (middle - hand).normalize();
    let z = x.cross(little - index).normalize();
    let y = z.cross(x).normalize();
    Quat::from_mat3(&Mat3::from_cols(x, y, z)).normalize()
}

fn source_rig(world: &World, seconds: f32, weight: f32) -> RigPose {
    let source = world.resource::<SourceAnimationLibrary>();
    let mut rig = RigPose::new(world.resource::<AnimationLibrary>());
    rig.source = Some(SourcePose {
        model: source.model_pose(&source.sample(
            source.clip("skill_ylk_1").unwrap(),
            seconds,
            false,
            false,
        )),
        weight,
    });
    rig
}

#[test]
fn every_real_naraka_finger_joint_is_bound_including_partial_thumb_palettes() {
    for appearance in [Appearance::Lihuowang, Appearance::Bailingmiao] {
        let (_, assets) = bundle(appearance);
        let mut count = 0;
        for part in assets.parts.iter().filter(|part| !part.blade) {
            for bone in &part.bones {
                let name = bone.name.rsplit("::").next().unwrap();
                if (name.starts_with("gMan R Finger") || name.starts_with("gMan L Finger"))
                    && !name.ends_with("Nub")
                {
                    count += 1;
                    assert!(
                        bone.finger.is_some(),
                        "{appearance:?} / {} / {name}: supported palette joint lost finger transfer",
                        part.name
                    );
                }
            }
        }
        assert!(count >= 30);
        if appearance == Appearance::Lihuowang {
            let partial = assets
                .parts
                .iter()
                .find(|part| part.name == "naraka::outfit::body")
                .unwrap();
            assert!(
                !partial
                    .bones
                    .iter()
                    .any(|bone| bone.name.ends_with("gMan L Finger0"))
            );
            assert!(
                partial
                    .bones
                    .iter()
                    .filter(|bone| bone.name.ends_with("gMan L Finger01")
                        || bone.name.ends_with("gMan L Finger02"))
                    .all(|bone| bone.finger.is_some())
            );
        }
    }
}

#[test]
fn all_fingers_follow_actual_source_anatomy_in_the_current_palm_frame() {
    for appearance in [Appearance::Lihuowang, Appearance::Bailingmiao] {
        let (world, assets) = bundle(appearance);
        let source = world.resource::<SourceAnimationLibrary>();
        for seconds in [0.0, 1.35, 2.5] {
            let rig = source_rig(&world, seconds, 1.0);
            let model = &rig.source.as_ref().unwrap().model;
            for side in ["L", "R"] {
                let source_joint =
                    |suffix: &str| model[source.bone(&format!("Bip001 {side} {suffix}")).unwrap()];
                let target_joint =
                    |suffix: &str| matrix(&assets, &rig, &format!("gMan {side} {suffix}"));
                let source_palm = palm(
                    source_joint("Hand").w_axis.xyz(),
                    source_joint("Finger1").w_axis.xyz(),
                    source_joint("Finger2").w_axis.xyz(),
                    source_joint("Finger4").w_axis.xyz(),
                );
                let target_palm = palm(
                    target_joint("Hand").w_axis.xyz(),
                    target_joint("Finger1").w_axis.xyz(),
                    target_joint("Finger2").w_axis.xyz(),
                    target_joint("Finger4").w_axis.xyz(),
                );
                for digit in 0..5 {
                    let target_names = [
                        format!("Finger{digit}"),
                        format!("Finger{digit}1"),
                        format!("Finger{digit}2"),
                    ];
                    let source_names = [
                        format!("Finger{digit}"),
                        format!("Finger{digit}1"),
                        format!("Finger{digit}2"),
                        format!("Finger{digit}Nub"),
                    ];
                    for link in 0..3 {
                        let target = target_joint(&target_names[link]);
                        let actual = if link < 2 {
                            target_joint(&target_names[link + 1]).w_axis.xyz() - target.w_axis.xyz()
                        } else {
                            target.transform_vector3(Vec3::X)
                        };
                        let wanted = source_joint(&source_names[link + 1]).w_axis.xyz()
                            - source_joint(&source_names[link]).w_axis.xyz();
                        let actual = (target_palm.inverse() * actual).normalize();
                        let wanted = (source_palm.inverse() * wanted).normalize();
                        assert!(
                            actual.distance(wanted) < 0.0005,
                            "{appearance:?} {side} finger{digit} link{link} t{seconds}: actual{actual:?} != source{wanted:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn finger_fades_keep_own_lengths_and_rest_local_rotation_at_zero_weight() {
    for appearance in [Appearance::Lihuowang, Appearance::Bailingmiao] {
        let (world, assets) = bundle(appearance);
        let fallback = RigPose::new(world.resource::<AnimationLibrary>());
        let zero = source_rig(&world, 1.35, 0.0);
        for side in ["L", "R"] {
            for digit in 0..5 {
                let names = [
                    format!("gMan {side} Hand"),
                    format!("gMan {side} Finger{digit}"),
                    format!("gMan {side} Finger{digit}1"),
                    format!("gMan {side} Finger{digit}2"),
                ];
                for name in names.iter().skip(1) {
                    assert!(
                        matrix(&assets, &fallback, name)
                            .abs_diff_eq(matrix(&assets, &zero, name), 0.00001),
                        "{appearance:?} {name}: source weight0 changed fallback"
                    );
                }
                for weight in [0.0, 0.25, 0.5, 0.75, 1.0] {
                    let rig = source_rig(&world, 1.35, weight);
                    for link in 0..3 {
                        let parent = matrix(&assets, &rig, &names[link]);
                        let child = matrix(&assets, &rig, &names[link + 1]);
                        let rest_parent = bone(&assets, &names[link]).bind;
                        let rest_child = bone(&assets, &names[link + 1]).bind;
                        let actual = child.w_axis.xyz().distance(parent.w_axis.xyz());
                        let wanted = rest_child.w_axis.xyz().distance(rest_parent.w_axis.xyz());
                        assert!(
                            (actual - wanted).abs() < 0.00001,
                            "{appearance:?} {side} finger{digit} link{link} weight{weight}: {actual} vs own{wanted}"
                        );
                    }
                }
            }
        }
    }
}

fn finger_vertices_in_hand(world: &World, assets: &AvatarAssets, rig: &RigPose) -> Vec<Vec3> {
    let hand = matrix(assets, rig, "gMan R Hand").inverse();
    let deltas = assets.pose_deltas(rig);
    let source_deltas = assets.source_deltas(rig);
    let meshes = world.resource::<Assets<Mesh>>();
    let inverse = world.resource::<Assets<SkinnedMeshInverseBindposes>>();
    let mut result = Vec::new();
    for part in assets.parts.iter().filter(|part| !part.blade) {
        let ibm = inverse.get(&part.inverse_bindposes).unwrap();
        let palette: Vec<_> = part
            .bones
            .iter()
            .zip(ibm.iter())
            .map(|(bone, ibm)| {
                assets
                    .joint_pose(rig, &deltas, source_deltas.as_deref(), bone)
                    .to_matrix()
                    * *ibm
            })
            .collect();
        for primitive in &part.primitives {
            let mesh = meshes.get(&primitive.mesh).unwrap();
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            let VertexAttributeValues::Uint16x4(joints) =
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX).unwrap()
            else {
                panic!("indices");
            };
            let VertexAttributeValues::Float32x4(weights) =
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT).unwrap()
            else {
                panic!("weights");
            };
            for vertex in 0..positions.len() {
                let influenced: f32 = (0..4)
                    .filter(|&slot| {
                        part.bones[joints[vertex][slot] as usize].finger.is_some()
                            && part.bones[joints[vertex][slot] as usize].driver == assets.right_hand
                    })
                    .map(|slot| weights[vertex][slot])
                    .sum();
                if influenced < 0.5 {
                    continue;
                }
                let position = Vec3::from(positions[vertex]);
                let mut posed = Vec3::ZERO;
                for slot in 0..4 {
                    posed += palette[joints[vertex][slot] as usize].transform_point3(position)
                        * weights[vertex][slot];
                }
                result.push(hand.transform_point3(posed));
            }
        }
    }
    result
}

#[test]
fn finger_motion_reaches_actual_lbs_vertices_and_reference_stays_original() {
    for appearance in [Appearance::Lihuowang, Appearance::Bailingmiao] {
        let (world, assets) = bundle(appearance);
        let fallback = RigPose::new(world.resource::<AnimationLibrary>());
        let mut reference = fallback.clone();
        reference.reference_pose = true;
        let deltas = assets.pose_deltas(&reference);
        for part in assets.parts.iter().filter(|part| !part.blade) {
            for bone in &part.bones {
                assert_eq!(
                    assets.joint_pose(&reference, &deltas, None, bone),
                    Transform::from_matrix(bone.bind)
                );
            }
        }
        let first = finger_vertices_in_hand(&world, &assets, &fallback);
        let changed = finger_vertices_in_hand(&world, &assets, &source_rig(&world, 1.35, 1.0));
        assert!(first.len() > 100);
        assert_eq!(first.len(), changed.len());
        let maximum = first
            .iter()
            .zip(changed)
            .map(|(a, b)| a.distance(b))
            .fold(0.0_f32, f32::max);
        assert!(
            maximum > 0.005,
            "{appearance:?}: actual finger-weighted skin stayed rigid relative to wrist ({maximum}m)"
        );
    }
}

#[test]
fn original_choulingke_named_source_palette_is_unchanged() {
    let (world, assets) = bundle(Appearance::Choulingke);
    let rig = source_rig(&world, 1.35, 1.0);
    let deltas = assets.pose_deltas(&rig);
    let source = rig.source.as_ref().unwrap();
    let mut fingers = 0;
    for part in assets.parts.iter().filter(|part| !part.blade) {
        for bone in &part.bones {
            assert!(bone.finger.is_none());
            if let Some(index) = bone.source {
                assert!(
                    assets
                        .joint_pose(&rig, &deltas, None, bone)
                        .to_matrix()
                        .abs_diff_eq(
                            Transform::from_matrix(source.model[index]).to_matrix(),
                            0.00001
                        )
                );
                if bone.name.contains("Finger") {
                    fingers += 1;
                }
            }
        }
    }
    assert!(fingers >= 30);
}

#[test]
fn default_weapon_grip_offset_and_shaft_follow_the_actual_palm() {
    for appearance in [Appearance::Lihuowang, Appearance::Bailingmiao] {
        let (world, assets) = bundle(appearance);
        let rig = RigPose::new(world.resource::<AnimationLibrary>());
        let wrist = matrix(&assets, &rig, "gMan R Hand").w_axis.xyz();
        let middle = matrix(&assets, &rig, "gMan R Finger2").w_axis.xyz();
        let index = matrix(&assets, &rig, "gMan R Finger1").w_axis.xyz();
        let little = matrix(&assets, &rig, "gMan R Finger4").w_axis.xyz();
        let long = (middle - wrist).normalize();
        let across = index - little;
        let across = (across - long * long.dot(across)).normalize();
        let hand =
            assets.attachment_pose(&rig)[world.resource::<AnimationLibrary>().bone("hand_r")];
        let toward_palm = hand.rotation * Vec3::Y;
        let shaft = hand.rotation * Vec3::Z;
        let palm_alignment = long.dot(toward_palm);
        let shaft_alignment = across.dot(shaft);
        assert!(
            palm_alignment > 0.9,
            "{appearance:?}: weapon offset is not along actual palm length ({palm_alignment})"
        );
        assert!(
            shaft_alignment > 0.9,
            "{appearance:?}: shaft axis crosses the actual finger bank ({shaft_alignment})"
        );
    }
}

#[test]
fn asset_reference_grips_use_actual_rest_palms_after_source_sampling() {
    for appearance in [
        Appearance::Lihuowang,
        Appearance::Bailingmiao,
        Appearance::Choulingke,
    ] {
        let (world, assets) = bundle(appearance);
        let mut rig = source_rig(&world, 1.35, 1.0);
        rig.reference_pose = true;
        let semantic = assets.attachment_pose(&rig);
        for side in ["L", "R"] {
            let prefix = if appearance == Appearance::Choulingke {
                "Bip001"
            } else {
                "gMan"
            };
            let actual = |suffix: &str| matrix(&assets, &rig, &format!("{prefix} {side} {suffix}"));
            let wrist = actual("Hand").w_axis.xyz();
            let frame = palm(
                wrist,
                actual("Finger1").w_axis.xyz(),
                actual("Finger2").w_axis.xyz(),
                actual("Finger4").w_axis.xyz(),
            );
            let hand = world.resource::<AnimationLibrary>().bone(if side == "R" {
                "hand_r"
            } else {
                "hand_l"
            });
            assert!(semantic[hand].translation.distance(wrist) < 0.00001);
            assert_eq!(semantic[hand].scale, Vec3::ONE);
            assert!((frame * Vec3::X).dot(semantic[hand].rotation * Vec3::Y) > 0.999);
            assert!((-(frame * Vec3::Y)).dot(semantic[hand].rotation * Vec3::Z) > 0.999);
            let weapon = super::super::stickman::attachment_transform(
                &super::super::stickman::Attachment::Weapon {
                    bone: hand,
                    mode: crate::simulation::WeaponMode::Melee,
                },
                &semantic,
            );
            let toward_palm = weapon.translation - wrist;
            assert!((toward_palm.length() - 0.055).abs() < 0.00001);
            assert!(toward_palm.normalize().dot(frame * Vec3::X) > 0.999);
        }
    }
}

#[test]
fn declared_cc0_and_current_source_palms_drive_actual_skin_and_grips() {
    for appearance in [
        Appearance::Lihuowang,
        Appearance::Bailingmiao,
        Appearance::Choulingke,
    ] {
        let (world, assets) = bundle(appearance);
        let library = world.resource::<AnimationLibrary>();
        let source = world.resource::<SourceAnimationLibrary>();
        for weight in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let rig = source_rig(&world, 1.35, weight);
            let semantic = assets.attachment_pose(&rig);
            for side in ["L", "R"] {
                let prefix = if appearance == Appearance::Choulingke {
                    "Bip001"
                } else {
                    "gMan"
                };
                let actual =
                    |suffix: &str| matrix(&assets, &rig, &format!("{prefix} {side} {suffix}"));
                let p = palm(
                    actual("Hand").w_axis.xyz(),
                    actual("Finger1").w_axis.xyz(),
                    actual("Finger2").w_axis.xyz(),
                    actual("Finger4").w_axis.xyz(),
                );
                let index = library.bone(if side == "R" { "hand_r" } else { "hand_l" });
                let attachment = semantic[index];
                assert_eq!(attachment.scale, Vec3::ONE);
                let long = p * Vec3::X;
                let across = -(p * Vec3::Y);
                assert!(
                    long.dot(attachment.rotation * Vec3::Y) > 0.999,
                    "{appearance:?} {side} weight{weight}: offset axis doesn't follow real palm"
                );
                assert!(
                    across.dot(attachment.rotation * Vec3::Z) > 0.999,
                    "{appearance:?} {side} weight{weight}: shaft doesn't follow real finger bank"
                );
                let grip = super::super::stickman::attachment_transform(
                    &super::super::stickman::Attachment::Weapon {
                        bone: index,
                        mode: crate::simulation::WeaponMode::Melee,
                    },
                    &semantic,
                );
                assert!(
                    ((grip.translation - attachment.translation).length() - 0.055).abs() < 0.00001
                );
                if appearance != Appearance::Choulingke {
                    let source_joint = |suffix: &str| {
                        rig.source.as_ref().unwrap().model
                            [source.bone(&format!("Bip001 {side} {suffix}")).unwrap()]
                    };
                    let source_palm = palm(
                        source_joint("Hand").w_axis.xyz(),
                        source_joint("Finger1").w_axis.xyz(),
                        source_joint("Finger2").w_axis.xyz(),
                        source_joint("Finger4").w_axis.xyz(),
                    );
                    let source_grip = Quat::from_mat3(&Mat3::from_cols(
                        -(source_palm * Vec3::Z),
                        source_palm * Vec3::X,
                        -(source_palm * Vec3::Y),
                    ));
                    let expected = rig.model[index].rotation.slerp(source_grip, weight);
                    assert!((expected * Vec3::Y).distance(long) < 0.0005);
                    assert!((expected * Vec3::Z).distance(across) < 0.0005);
                }
            }
        }
    }
}
