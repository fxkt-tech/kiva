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
        scope: crate::client::source_animation::SourceScope::FullBody,
        rig: crate::client::source_animation::SourceRig::Yanyun,
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

#[test]
fn native_naraka_hands_use_their_own_negative_axes_and_keep_all_three_targets_lengths() {
    for appearance in [
        Appearance::Lihuowang,
        Appearance::Bailingmiao,
        Appearance::Choulingke,
    ] {
        let (world, assets) = bundle(appearance);
        let source = world
            .resource::<SourceAnimationLibrary>()
            .for_rig(SourceRig::Naraka)
            .expect("verified native Naraka library");
        let prefix = if appearance == Appearance::Choulingke {
            "Bip001"
        } else {
            "gMan"
        };
        for clip in [
            "male_hengdao_idle_01",
            "male_hengdao_attack_light_01",
            "male_hengdao_shock_01",
            "male_hengdao_attack_shock_light_01",
        ] {
            let clip = source.clip(clip).unwrap();
            for seconds in [0.0, source.duration(clip) * 0.4, source.duration(clip)] {
                let mut rig = RigPose::new(world.resource::<AnimationLibrary>());
                rig.source = Some(SourcePose {
                    scope: crate::client::source_animation::SourceScope::FullBody,
                    rig: SourceRig::Naraka,
                    model: source.model_pose(&source.sample(clip, seconds, false, false)),
                    weight: 1.0,
                });
                let model = &rig.source.as_ref().unwrap().model;
                for side in ["L", "R"] {
                    let source_joint = |suffix: &str| {
                        model[source.bone(&format!("gMan {side} {suffix}")).unwrap()]
                    };
                    let target_joint =
                        |suffix: &str| matrix(&assets, &rig, &format!("{prefix} {side} {suffix}"));
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
                        let suffixes = [
                            "Hand".to_owned(),
                            format!("Finger{digit}"),
                            format!("Finger{digit}1"),
                            format!("Finger{digit}2"),
                        ];
                        for link in 1..=3 {
                            let name = format!("{prefix} {side} {}", suffixes[link]);
                            assert!(
                                bone(&assets, &name).naraka_finger.is_some(),
                                "{appearance:?} {name}"
                            );
                            let child = target_joint(&suffixes[link]);
                            let parent = target_joint(&suffixes[link - 1]);
                            let rest_child = bone(&assets, &name).bind;
                            let rest_parent =
                                bone(&assets, &format!("{prefix} {side} {}", suffixes[link - 1]))
                                    .bind;
                            assert!(
                                (child.w_axis.xyz().distance(parent.w_axis.xyz())
                                    - rest_child.w_axis.xyz().distance(rest_parent.w_axis.xyz()))
                                .abs()
                                    < 0.00002,
                                "{appearance:?} {name}: own segment changed"
                            );
                            let (actual, wanted) = if link < 3 {
                                (
                                    target_joint(&suffixes[link + 1]).w_axis.xyz()
                                        - child.w_axis.xyz(),
                                    source_joint(&suffixes[link + 1]).w_axis.xyz()
                                        - source_joint(&suffixes[link]).w_axis.xyz(),
                                )
                            } else {
                                let source_index = source
                                    .bone(&format!("gMan {side} {}", suffixes[link]))
                                    .unwrap();
                                let previous_index = source
                                    .bone(&format!("gMan {side} {}", suffixes[link - 1]))
                                    .unwrap();
                                let native_axis =
                                    Transform::from_matrix(source.rest_model[previous_index])
                                        .rotation
                                        .inverse()
                                        * (source.rest_model[source_index].w_axis.xyz()
                                            - source.rest_model[previous_index].w_axis.xyz());
                                assert!(
                                    native_axis.x < 0.0,
                                    "real donor distal direction must remain -X"
                                );
                                let own_axis = if let Some(nub) = assets
                                    .parts
                                    .iter()
                                    .filter(|part| !part.blade)
                                    .flat_map(|part| &part.bones)
                                    .find(|bone| {
                                        bone.name.rsplit("::").next().unwrap()
                                            == format!("{prefix} {side} Finger{digit}Nub")
                                    }) {
                                    Transform::from_matrix(rest_child).rotation.inverse()
                                        * (nub.bind.w_axis.xyz() - rest_child.w_axis.xyz())
                                } else {
                                    Transform::from_matrix(rest_parent).rotation.inverse()
                                        * (rest_child.w_axis.xyz() - rest_parent.w_axis.xyz())
                                };
                                (
                                    Transform::from_matrix(child).rotation * own_axis,
                                    Transform::from_matrix(source_joint(&suffixes[link])).rotation
                                        * native_axis,
                                )
                            };
                            let actual = (target_palm.inverse() * actual).normalize();
                            let wanted = (source_palm.inverse() * wanted).normalize();
                            assert!(
                                actual.distance(wanted) < 0.001,
                                "{appearance:?} {side} finger{digit} link{link} t{seconds}: {actual:?} vs {wanted:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

fn lower_body_skin_vertices(world: &World, assets: &AvatarAssets, rig: &RigPose) -> Vec<Vec3> {
    let meshes = world.resource::<Assets<Mesh>>();
    let inverse = world.resource::<Assets<SkinnedMeshInverseBindposes>>();
    let deltas = assets.pose_deltas(rig);
    let source_deltas = assets.source_deltas(rig);
    let mut result = Vec::new();
    for part in assets.parts.iter().filter(|part| !part.blade) {
        let palette: Vec<_> = part
            .bones
            .iter()
            .zip(inverse.get(&part.inverse_bindposes).unwrap().iter())
            .map(|(bone, bind)| {
                assets
                    .joint_pose(rig, &deltas, source_deltas.as_deref(), bone)
                    .to_matrix()
                    * *bind
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
                panic!("indices")
            };
            let VertexAttributeValues::Float32x4(weights) =
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT).unwrap()
            else {
                panic!("weights")
            };
            for vertex in 0..positions.len() {
                if (0..4).any(|slot| {
                    weights[vertex][slot] > 0.0
                        && (2..=14).contains(&part.bones[joints[vertex][slot] as usize].driver)
                }) {
                    continue;
                }
                let position = Vec3::from(positions[vertex]);
                result.push(
                    (0..4)
                        .map(|slot| {
                            palette[joints[vertex][slot] as usize].transform_point3(position)
                                * weights[vertex][slot]
                        })
                        .sum(),
                );
            }
        }
    }
    result
}

#[test]
fn raw_native_upper_body_layers_keep_actual_pelvis_legs_and_skinned_feet_on_the_shared_baseline() {
    for appearance in [
        Appearance::Lihuowang,
        Appearance::Bailingmiao,
        Appearance::Choulingke,
    ] {
        let (world, assets) = bundle(appearance);
        let cc0 = world.resource::<AnimationLibrary>();
        let sources = world.resource::<SourceAnimationLibrary>();
        let source = sources.for_rig(SourceRig::Naraka).unwrap();
        let baseline = RigPose::new(cc0);
        let baseline_drivers = assets.evaluated_drivers(&baseline);
        let baseline_vertices = lower_body_skin_vertices(&world, &assets, &baseline);
        assert!(baseline_vertices.len() > 100);
        for (name, duration) in source.clips().into_iter().filter(|(name, _)| {
            name.contains("_sniper_") || name.contains("_fk_") || *name == "male_bow_attack_01"
        }) {
            let mut preview = crate::client::animation::PreviewRig::new(cc0);
            for (sample, seconds) in [0.0, duration * 0.5, duration].into_iter().enumerate() {
                assert!(preview.seek(
                    cc0,
                    sources,
                    &format!("naraka:{name}"),
                    crate::simulation::WeaponPair::HammerMusket,
                    false,
                    seconds,
                    false
                ));
                let rig = preview.pose();
                assert_eq!(rig.source.as_ref().unwrap().scope, SourceScope::UpperBody);
                assert!(
                    rig.companion_hands.is_none(),
                    "raw source must not apply weapon IK"
                );
                let drivers = assets.evaluated_drivers(rig);
                for (index, (actual, baseline)) in drivers.iter().zip(&baseline_drivers).enumerate()
                {
                    if (2..=14).contains(&index) {
                        continue;
                    }
                    assert!(
                        actual.translation.distance(baseline.translation) < 0.000002,
                        "{appearance:?} {name}: lower driver{index} sank"
                    );
                    assert!(actual.rotation.dot(baseline.rotation).abs() > 0.999999);
                }
                if sample == 1 {
                    let actual_vertices = lower_body_skin_vertices(&world, &assets, rig);
                    assert_eq!(actual_vertices.len(), baseline_vertices.len());
                    for (actual, baseline) in actual_vertices.iter().zip(&baseline_vertices) {
                        assert!(
                            actual.distance(*baseline) < 0.00002,
                            "{appearance:?} {name}: actual lower-body LBS changed by {:?}",
                            actual - *baseline
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn native_finger_curls_reach_real_weighted_vertices_and_fades_keep_own_lengths() {
    for appearance in [Appearance::Lihuowang, Appearance::Bailingmiao] {
        let (world, assets) = bundle(appearance);
        let source = world
            .resource::<SourceAnimationLibrary>()
            .for_rig(SourceRig::Naraka)
            .unwrap();
        let baseline = RigPose::new(world.resource::<AnimationLibrary>());
        let rigid = finger_vertices_in_hand(&world, &assets, &baseline);
        let clip = source.clip("male_hengdao_idle_01").unwrap();
        let model = source.model_pose(&source.sample(clip, 0.4, false, false));
        let mut rig = baseline.clone();
        rig.source = Some(SourcePose {
            scope: SourceScope::FullBody,
            rig: SourceRig::Naraka,
            model,
            weight: 1.0,
        });
        let curled = finger_vertices_in_hand(&world, &assets, &rig);
        assert_eq!(rigid.len(), curled.len());
        assert!(rigid.len() > 100);
        let displacement = rigid
            .iter()
            .zip(&curled)
            .map(|(a, b)| a.distance(*b))
            .fold(0.0_f32, f32::max);
        assert!(
            displacement > 0.005,
            "{appearance:?}: native curls did not reach actual finger-weighted skin ({displacement}m)"
        );
        for weight in [0.0, 0.25, 0.5, 0.75, 1.0] {
            rig.source.as_mut().unwrap().weight = weight;
            for side in ["L", "R"] {
                for digit in 0..5 {
                    let names = [
                        format!("gMan {side} Hand"),
                        format!("gMan {side} Finger{digit}"),
                        format!("gMan {side} Finger{digit}1"),
                        format!("gMan {side} Finger{digit}2"),
                    ];
                    for link in 0..3 {
                        let posed_length = matrix(&assets, &rig, &names[link])
                            .w_axis
                            .xyz()
                            .distance(matrix(&assets, &rig, &names[link + 1]).w_axis.xyz());
                        let own_length = bone(&assets, &names[link])
                            .bind
                            .w_axis
                            .xyz()
                            .distance(bone(&assets, &names[link + 1]).bind.w_axis.xyz());
                        assert!(
                            (posed_length - own_length).abs() < 0.00002,
                            "{appearance:?} {side} finger{digit} segment{link} source weight{weight}: own length changed"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn crouched_attacks_and_crouched_or_moving_defense_keep_real_lower_skin_and_weapon_grips() {
    for appearance in [
        Appearance::Lihuowang,
        Appearance::Bailingmiao,
        Appearance::Choulingke,
    ] {
        let (world, assets) = bundle(appearance);
        let cc0 = world.resource::<AnimationLibrary>();
        let sources = world.resource::<SourceAnimationLibrary>();
        let prefix = if appearance == Appearance::Choulingke {
            "Bip001"
        } else {
            "gMan"
        };
        for scenario in ["crouch_attack", "crouch_guard", "walk_guard"] {
            let mut preview = crate::client::animation::PreviewRig::new(cc0);
            for seconds in [1.0 / 60.0, 0.05, 0.10, 0.20, 0.40] {
                assert!(preview.seek(
                    cc0,
                    sources,
                    &format!("game:{scenario}"),
                    crate::simulation::WeaponPair::HammerMusket,
                    false,
                    seconds,
                    false
                ));
                let rig = preview.pose();
                assert_eq!(rig.source.as_ref().unwrap().scope, SourceScope::UpperBody);
                let expected = if scenario == "crouch_attack" {
                    "male_hengdao_attack_light_01"
                } else {
                    "male_hengdao_shock_01"
                };
                assert_eq!(rig.trace().source_clip.as_deref(), Some(expected));
                assert!(rig.trace().adaptation.unwrap().contains("crouch/gait"));
                let mut baseline = rig.clone();
                baseline.source = None;
                let drivers = assets.evaluated_drivers(rig);
                let lower = assets.evaluated_drivers(&baseline);
                for (index, (actual, wanted)) in drivers.iter().zip(&lower).enumerate() {
                    if (2..=14).contains(&index) {
                        continue;
                    }
                    assert!(
                        actual.translation.distance(wanted.translation) < 0.000002,
                        "{appearance:?} {scenario} {seconds}: lower driver {index} displaced"
                    );
                    assert!(actual.rotation.dot(wanted.rotation).abs() > 0.999999);
                }
                let actual_skin = lower_body_skin_vertices(&world, &assets, rig);
                let baseline_skin = lower_body_skin_vertices(&world, &assets, &baseline);
                assert_eq!(actual_skin.len(), baseline_skin.len());
                for (actual, wanted) in actual_skin.iter().zip(&baseline_skin) {
                    assert!(
                        actual.distance(*wanted) < 0.00002,
                        "{appearance:?} {scenario}: actual pelvis/leg/foot skin displaced"
                    );
                }
                if scenario.starts_with("crouch") {
                    let weight = rig.trace().crouch_weight;
                    if seconds < 0.12 {
                        assert!(weight > 0.0 && weight < 1.0);
                    } else {
                        assert_eq!(weight, 1.0);
                    }
                }
                let pose = assets.attachment_pose(rig);
                for (side, semantic) in [("R", "hand_r"), ("L", "hand_l")] {
                    let wrist = matrix(&assets, rig, &format!("{prefix} {side} Hand"))
                        .w_axis
                        .xyz();
                    let middle = matrix(&assets, rig, &format!("{prefix} {side} Finger2"))
                        .w_axis
                        .xyz();
                    let hand = cc0.bone(semantic);
                    assert!(pose[hand].translation.distance(wrist) < 0.00002);
                    assert!(
                        (pose[hand].rotation * Vec3::Y).dot((middle - wrist).normalize()) > 0.9999
                    );
                    let mount = crate::client::stickman::attachment_transform(
                        &crate::client::stickman::Attachment::Weapon {
                            bone: hand,
                            mode: crate::simulation::WeaponMode::Melee,
                        },
                        &pose,
                    );
                    assert!(
                        mount
                            .translation
                            .distance(wrist + (middle - wrist).normalize() * 0.055)
                            < 0.00002,
                        "{appearance:?} {scenario} {side}: signed physical grip shifted"
                    );
                }
            }
        }
    }
}
