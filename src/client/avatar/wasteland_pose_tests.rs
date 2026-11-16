//! Explicit offline snapshots of the shared display driver for Blender review.
//! This module is absent from product builds and never changes simulation/assets.
use super::*;
use crate::{
    client::{
        animation::{AttackReplay, PreviewRig},
        stickman::{Attachment, attachment_transform},
    },
    simulation::{TrackId, WeaponMode, WeaponPair},
};
use bevy::ecs::system::RunSystemOnce;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    path::Path,
};

const STEP: f32 = 1.0 / 60.0;

struct Exporter {
    world: World,
    assets: AvatarAssets,
    owner: Entity,
    /// First outfit slot per native name, matching extract_reference.py.
    joints: BTreeMap<String, (Entity, SourceBone)>,
}

struct Snapshot {
    name: String,
    pair: WeaponPair,
    track: Option<TrackId>,
    time: f32,
    duration: f32,
    source_clip: Option<String>,
    source_time: f32,
    joints: BTreeMap<String, Mat4>,
    hand_grips: [Mat4; 2],
    weapon_mount: Mat4,
}

impl Exporter {
    fn new() -> Self {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<AnimationLibrary>();
        let assets = AvatarAssets::loaded(&mut world, Appearance::Lihuowang);
        let owner = world.spawn(Transform::IDENTITY).id();
        spawn_with_appearance(&mut world, owner, Appearance::Lihuowang);
        let native = world
            .resource::<SourceAnimationLibrary>()
            .for_rig(SourceRig::Naraka)
            .expect("the bundled native player rig");
        let mut joints = BTreeMap::new();
        for part in &assets.parts {
            if !part.name.starts_with("naraka::outfit::") {
                continue;
            }
            for bone in &part.bones {
                let name = bone.name.rsplit("::").next().unwrap();
                if !name.starts_with("gMan ") || native.bone(name).is_none() {
                    continue;
                }
                // Select by full SourceBone identity: a name can occur in
                // several real palettes, not all with the same driver slot.
                let entity = world
                    .iter_entities()
                    .find(|entity| {
                        entity.get::<PaletteJoint>().is_some_and(|joint| {
                            joint.owner == owner
                                && joint.bone.name == bone.name
                                && joint.bone.driver == bone.driver
                                && joint.bone.bind == bone.bind
                        })
                    })
                    .expect("spawned palette slot")
                    .id();
                if let Some((_, previous)) = joints.get(name) {
                    let previous: &SourceBone = previous;
                    assert!(previous.bind.abs_diff_eq(bone.bind, 0.0001));
                } else {
                    joints.insert(name.to_owned(), (entity, bone.clone()));
                }
            }
        }
        assert_eq!(joints.len(), 52, "22 body and 30 finger target joints");
        Self {
            world,
            assets,
            owner,
            joints,
        }
    }

    fn rest(&self) -> BTreeMap<String, Mat4> {
        self.joints
            .iter()
            .map(|(name, (_, bone))| (name.clone(), bone.bind))
            .collect()
    }

    fn snapshot(
        &mut self,
        rig: &RigPose,
        name: String,
        pair: WeaponPair,
        track: Option<TrackId>,
        time: f32,
        duration: f32,
    ) -> Snapshot {
        self.world.entity_mut(self.owner).insert(rig.clone());
        self.world.run_system_once(sync).unwrap();
        let deltas = self.assets.pose_deltas(rig);
        let source_deltas = self.assets.source_deltas(rig);
        let joints: BTreeMap<_, _> = self
            .joints
            .iter()
            .map(|(name, (entity, bone))| {
                let actual = self.world.get::<Transform>(*entity).unwrap().to_matrix();
                assert!(actual.is_finite() && actual.determinant().abs() > 0.0001);
                assert!(
                    actual.abs_diff_eq(
                        self.assets
                            .joint_pose(rig, &deltas, source_deltas.as_deref(), bone)
                            .to_matrix(),
                        0.000001,
                    )
                );
                (name.clone(), actual)
            })
            .collect();
        let attachments = self.assets.attachment_pose(rig);
        let library = self.world.resource::<AnimationLibrary>();
        let hand_grips = ["hand_r", "hand_l"].map(|name| {
            let matrix = attachments[library.bone(name)].to_matrix();
            assert!(matrix.is_finite());
            matrix
        });
        for (side, grip) in ["R", "L"].into_iter().zip(hand_grips) {
            let wrist = joints[&format!("gMan {side} Hand")].w_axis.xyz();
            assert!(grip.w_axis.xyz().distance(wrist) < 0.00001);
        }
        let weapon_mount = attachment_transform(
            &Attachment::Weapon {
                bone: library.bone("hand_r"),
                mode: WeaponMode::Melee,
            },
            &attachments,
        )
        .to_matrix();
        assert!(weapon_mount.is_finite());
        Snapshot {
            name,
            pair,
            track,
            time,
            duration,
            source_clip: rig.trace().source_clip.clone(),
            source_time: rig.trace().source_seconds,
            joints,
            hand_grips,
            weapon_mount,
        }
    }
}

fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => write!(out, "\\u{:04x}", ch as u32).unwrap(),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn matrix_json(matrix: Mat4) -> String {
    let columns = matrix.to_cols_array_2d();
    let rows: [[f32; 4]; 4] =
        std::array::from_fn(|row| std::array::from_fn(|column| columns[column][row]));
    format!("{rows:?}")
}

fn joints_json(joints: &BTreeMap<String, Mat4>) -> String {
    let entries: Vec<_> = joints
        .iter()
        .map(|(name, matrix)| format!("{}:{}", quote(name), matrix_json(*matrix)))
        .collect();
    format!("{{{}}}", entries.join(","))
}

fn snapshot_json(pose: &Snapshot) -> String {
    format!(
        "{{\"name\":{},\"pair\":{},\"track\":{},\"time\":{},\"duration\":{},\"source_clip\":{},\"source_time\":{},\"joints\":{},\"hand_grips\":{{\"R\":{},\"L\":{}}},\"weapon_mount\":{}}}",
        quote(&pose.name),
        quote(&format!("{:?}", pose.pair)),
        pose.track
            .map_or("null".into(), |track| quote(&format!("{track:?}"))),
        pose.time,
        pose.duration,
        pose.source_clip.as_deref().map_or("null".into(), quote),
        pose.source_time,
        joints_json(&pose.joints),
        matrix_json(pose.hand_grips[0]),
        matrix_json(pose.hand_grips[1]),
        matrix_json(pose.weapon_mount),
    )
}

#[test]
fn runtime_snapshots_use_complete_palettes_and_real_wrist_grips() {
    let mut exporter = Exporter::new();
    let library = AnimationLibrary::default();
    let sources = SourceAnimationLibrary::bundled();
    let mut preview = PreviewRig::new(&library);
    assert!(preview.seek(
        &library,
        &sources,
        "reference",
        WeaponPair::default(),
        false,
        0.0,
        false,
    ));
    let rest = exporter.rest();
    let reference = exporter.snapshot(
        preview.pose(),
        "reference".into(),
        WeaponPair::default(),
        None,
        0.0,
        0.0,
    );
    for (name, matrix) in &rest {
        // Runtime Transform decomposes the imported float32 matrix. Tiny
        // non-orthogonality in original finger rests is not replaced with IBM.
        assert!(
            matrix.abs_diff_eq(reference.joints[name], 0.000002),
            "{name}: reference Transform decomposition exceeded tolerance"
        );
    }
    for pair in WeaponPair::ALL {
        let track = TrackId::PlayerMelee {
            pair,
            stage: crate::simulation::MeleeStage::One,
        };
        let mut replay = AttackReplay::new(&library, pair, false);
        replay.idle(&library, &sources, 40);
        replay.begin();
        for tick in 0..=24 {
            replay.show(
                &library,
                &sources,
                track,
                tick as f32 * STEP,
                track.duration(),
            );
        }
        let pose = exporter.snapshot(
            replay.pose(),
            format!("{pair:?}:attack"),
            pair,
            Some(track),
            24.0 * STEP,
            track.duration(),
        );
        assert_eq!(
            pose.joints.keys().collect::<Vec<_>>(),
            rest.keys().collect::<Vec<_>>()
        );
        assert!(pose.source_clip.is_some());
        assert!(
            pose.joints["gMan R Hand"]
                .w_axis
                .xyz()
                .distance(rest["gMan R Hand"].w_axis.xyz())
                > 0.03
        );
    }
}

#[test]
fn authored_zhuque_keeps_the_existing_actual_body_palette_and_weapon_grips() {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<AnimationLibrary>();
    let old = AvatarAssets::loaded(&mut world, Appearance::Lihuowang);
    let new = AvatarAssets::loaded(&mut world, Appearance::Zhuque);
    let library = world.resource::<AnimationLibrary>();
    let sources = world.resource::<SourceAnimationLibrary>();
    let native = |assets: &AvatarAssets, rig: &RigPose| {
        let deltas = assets.pose_deltas(rig);
        let source = assets.source_deltas(rig);
        let mut output = BTreeMap::new();
        for part in assets.parts.iter().filter(|part| !part.blade) {
            for bone in &part.bones {
                let name = bone.name.rsplit("::").next().unwrap();
                if name.starts_with("gMan ") {
                    output.entry(name.to_owned()).or_insert_with(|| {
                        assets
                            .joint_pose(rig, &deltas, source.as_deref(), bone)
                            .to_matrix()
                    });
                }
            }
        }
        output
    };
    for pair in WeaponPair::ALL {
        let mut preview = PreviewRig::new(library);
        for scenario in [
            "reference",
            "game:run",
            "game:jump",
            "game:crouch",
            "game:attack",
            "game:attack2",
            "game:attack3",
        ] {
            assert!(preview.seek(library, sources, scenario, pair, false, 0.25, false));
            let rig = preview.pose();
            let expected = native(&old, rig);
            let actual = native(&new, rig);
            assert_eq!(actual.len(), 52);
            for (name, matrix) in &actual {
                assert!(
                    matrix.abs_diff_eq(expected[name], 0.00001),
                    "{scenario}: {name}"
                );
            }
            let a = old.attachment_pose(rig);
            let b = new.attachment_pose(rig);
            for name in ["hand_r", "hand_l"] {
                let index = library.bone(name);
                assert!(
                    a[index]
                        .to_matrix()
                        .abs_diff_eq(b[index].to_matrix(), 0.00001)
                );
            }
        }
    }
}

/// Explicit artifact creation, never a write during ordinary cargo test.
#[test]
#[ignore = "offline Blender review export; set KIVA_POSE_EXPORT to destination JSON"]
fn export_runtime_poses() {
    let destination = std::env::var_os("KIVA_POSE_EXPORT").expect("set KIVA_POSE_EXPORT");
    let mut exporter = Exporter::new();
    let library = AnimationLibrary::default();
    let sources = SourceAnimationLibrary::bundled();
    let rest = exporter.rest();
    let mut snapshots = Vec::new();
    for pair in WeaponPair::ALL {
        for (scenario, times) in [
            ("reference", &[0.0][..]),
            ("game:idle", &[0.0, 0.5, 1.0][..]),
            ("game:walk", &[0.25, 0.5, 0.75][..]),
            ("game:run", &[0.25, 0.5, 0.75][..]),
            ("game:jump", &[0.2, 0.5, 1.0][..]),
            ("game:double_jump", &[0.25, 0.65, 1.1][..]),
            ("game:dodge_front", &[0.1, 0.2, 0.35][..]),
            ("game:dodge_back", &[0.1, 0.2, 0.35][..]),
            ("game:dodge_left", &[0.1, 0.2, 0.35][..]),
            ("game:dodge_right", &[0.1, 0.2, 0.35][..]),
            ("game:crouch", &[0.2, 0.5][..]),
            ("game:guard", &[0.2, 0.5][..]),
            ("game:hit", &[0.1, 0.25][..]),
        ] {
            let mut preview = PreviewRig::new(&library);
            for &time in times {
                assert!(preview.seek(&library, &sources, scenario, pair, false, time, false));
                snapshots.push(exporter.snapshot(
                    preview.pose(),
                    format!("{pair:?}:{scenario}:{time:.2}"),
                    pair,
                    None,
                    time,
                    if scenario == "reference" { 0.0 } else { 3.0 },
                ));
            }
        }
        for track in TrackId::all().into_iter().filter(|track| match *track {
            TrackId::PlayerMelee { pair: own, .. } => own == pair,
            TrackId::PlayerSkill(kind) => kind == pair.melee_skill(),
            _ => false,
        }) {
            let duration = track.duration();
            let last_tick = (duration / STEP).ceil() as u32;
            let selected: BTreeSet<_> = [0.0, 0.15, 0.3, 0.5, 0.7, 0.9, 0.97]
                .map(|fraction| ((duration * fraction / STEP).round() as u32).min(last_tick - 1))
                .into_iter()
                .collect();
            let dense = matches!(
                track,
                TrackId::PlayerMelee {
                    stage: crate::simulation::MeleeStage::One,
                    ..
                }
            );
            let mut replay = AttackReplay::new(&library, pair, false);
            replay.idle(&library, &sources, 40);
            replay.begin();
            for tick in 0..last_tick {
                let time = tick as f32 * STEP;
                replay.show(&library, &sources, track, time, duration);
                if selected.contains(&tick) || dense && tick % 3 == 0 {
                    snapshots.push(exporter.snapshot(
                        replay.pose(),
                        format!("{pair:?}:{track:?}:tick{tick:03}"),
                        pair,
                        Some(track),
                        time,
                        duration,
                    ));
                }
            }
        }
    }
    assert!(snapshots.iter().all(|pose| pose.joints.len() == rest.len()));
    let poses = snapshots
        .iter()
        .map(snapshot_json)
        .collect::<Vec<_>>()
        .join(",\n");
    let json = format!(
        "{{\n\"schema\":\"kiva-runtime-palette-poses-v1\",\n\"matrix_layout\":\"row-major\",\n\"coordinate_system\":\"game-model-metres-Y-up-minus-Z-forward\",\n\"appearance\":\"Lihuowang\",\n\"joint_count\":52,\n\"rest\":{},\n\"provenance\":\"PreviewRig or AttackReplay -> RigPose -> avatar::sync -> actual PaletteJoint Transform; matrices before inverse bind. Synthetic read-only display scenarios, not simulation replay.\",\n\"limits\":\"Existing target palette poses only. No new character skin or in-game visual acceptance. Cloak/auxiliary joints absent. Actor root is not exported; jump height stays simulation-owned.\",\n\"poses\":[\n{}\n]\n}}\n",
        joints_json(&rest),
        poses,
    );
    let destination = Path::new(&destination);
    std::fs::create_dir_all(destination.parent().expect("destination directory")).unwrap();
    std::fs::write(destination, json).unwrap();
    eprintln!(
        "exported {} poses / 52 joints to {}",
        snapshots.len(),
        destination.display()
    );
}
