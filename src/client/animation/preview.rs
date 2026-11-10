//! Inspection inputs use the live display driver, never another crop/IK recipe.
use super::*;

const HZ: f32 = 60.0;
const STEP: f32 = 1.0 / HZ;

pub(in crate::client) struct PreviewClip {
    pub(in crate::client) id: String,
    pub(in crate::client) label: String,
    pub(in crate::client) group: &'static str,
    pub(in crate::client) duration: f32,
}

const SCENARIOS: &[(&str, &str)] = &[
    ("idle", "待机"),
    ("walk", "原地行走"),
    ("run", "原地跑步"),
    ("crouch", "蹲伏"),
    ("attack", "当前近战普攻"),
    ("guard", "持续防御"),
    ("deflect", "拆招反馈"),
    ("parry", "弹反反馈"),
    ("hit", "受击"),
    ("death", "倒地"),
    ("companion_hold", "妹妹持握姿态"),
    ("companion_aim", "妹妹抬枪／拉弓瞄准姿态"),
];

pub(in crate::client) fn preview_catalog(
    library: &AnimationLibrary,
    source: &SourceAnimationLibrary,
) -> Vec<PreviewClip> {
    let mut clips: Vec<_> = SCENARIOS
        .iter()
        .map(|&(id, label)| PreviewClip {
            id: format!("game:{id}"),
            label: label.into(),
            group: "游戏单体检查",
            duration: 3.0,
        })
        .collect();
    clips.push(PreviewClip {
        id: "reference".into(),
        label: "角色导入参考姿态".into(),
        group: "参考姿态",
        duration: 0.0,
    });
    for (prefix, group, entries) in [
        ("cc0", "CC0片段对照", library.clips()),
        ("source", "源片段对照（已适配根运动）", source.clips()),
    ] {
        clips.extend(entries.into_iter().map(|(name, duration)| PreviewClip {
            id: format!("{prefix}:{name}"),
            label: name.into(),
            group,
            duration,
        }));
    }
    clips
}

/// Keeps the whole controller history. Forward playback is incremental; seeking
/// back or changing scenario resets before replaying the exact same frames.
pub(in crate::client) struct PreviewRig {
    rig: RigPose,
    id: String,
    pair: WeaponPair,
    blade: bool,
    frame: u32,
}

impl PreviewRig {
    pub(in crate::client) fn new(library: &AnimationLibrary) -> Self {
        Self {
            rig: RigPose::new(library),
            id: String::new(),
            pair: WeaponPair::HammerMusket,
            blade: false,
            frame: 0,
        }
    }

    pub(in crate::client) fn pose(&self) -> &RigPose {
        &self.rig
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "explicit inspection context and clock"
    )]
    pub(in crate::client) fn seek(
        &mut self,
        library: &AnimationLibrary,
        source: &SourceAnimationLibrary,
        id: &str,
        pair: WeaponPair,
        blade: bool,
        seconds: f32,
        looping: bool,
    ) -> bool {
        if !seconds.is_finite() || seconds < 0.0 {
            return false;
        }
        if let Some(scenario) = id.strip_prefix("game:") {
            if !SCENARIOS.iter().any(|&(name, _)| name == scenario) {
                return false;
            }
            let frame = (seconds.min(3.0) * HZ).round() as u32;
            if self.id != id || self.pair != pair || self.blade != blade || frame < self.frame {
                self.rig = RigPose::new(library);
                self.frame = 0;
            }
            for step in self.frame + 1..=frame {
                let (input, effect) = scenario_input(scenario);
                let action = (step == 1 && scenario == "attack" && !blade)
                    .then_some((1, Some(CombatEffect::Melee)));
                let event = (step == 1).then_some(effect).flatten();
                self.rig.frame(
                    library,
                    Some(source),
                    DisplayFrame {
                        input,
                        pair,
                        blade,
                        action,
                        latest_sequence: u64::from(effect.is_some()),
                    },
                    event.into_iter().map(|kind| (1, 1, kind)),
                    STEP,
                );
            }
            self.frame = frame;
        } else if id == "reference" {
            self.rig = RigPose::new(library);
            self.rig.reference_pose = true;
            self.rig.trace = PoseTrace {
                state: "asset reference (not bind pose)",
                transition: 1.0,
                ..default()
            };
        } else if let Some(name) = id.strip_prefix("cc0:") {
            let Some((_, duration)) = library
                .clips()
                .into_iter()
                .find(|&(value, _)| value == name)
            else {
                return false;
            };
            let time = clip_time(seconds, duration, looping);
            self.rig = RigPose::new(library);
            self.rig.local = library.sample(name, time, false);
            self.rig.model = library.model_pose(&self.rig.local);
            self.rig.trace = PoseTrace {
                state: "CC0 comparison (no hold override)",
                cc0_clip: Some(name.into()),
                cc0_seconds: time,
                transition: 1.0,
                ..default()
            };
        } else if let Some(name) = id.strip_prefix("source:") {
            let Some((_, duration)) = source.clips().into_iter().find(|&(value, _)| value == name)
            else {
                return false;
            };
            let clip = source.clip(name).expect("catalog clip");
            let time = clip_time(seconds, duration, looping);
            self.rig = RigPose::new(library);
            self.rig.source = Some(SourcePose {
                model: source.model_pose(&source.sample(clip, time, false, false)),
                weight: 1.0,
            });
            self.rig.trace = PoseTrace {
                state: "source comparison (no hold override)",
                source_clip: Some(name.into()),
                source_seconds: time,
                source_weight: 1.0,
                transition: 1.0,
                ..default()
            };
        } else {
            return false;
        }
        self.id = id.into();
        self.pair = pair;
        self.blade = blade;
        true
    }
}

fn clip_time(seconds: f32, duration: f32, looping: bool) -> f32 {
    // An explicit seek to the last frame remains observable even when looping.
    if seconds <= duration || !looping {
        seconds.min(duration)
    } else {
        seconds.rem_euclid(duration)
    }
}

fn scenario_input(id: &str) -> (Inputs, Option<CombatEffect>) {
    let mut input = Inputs {
        alive: true,
        airborne: false,
        jumps_used: 0,
        speed: 0.0,
        sprinting: false,
        dodging: false,
        dodge_elapsed: 0.0,
        dodge_yaw: 0.0,
        crouched: false,
        sliding: false,
        slide_yaw: 0.0,
        parrying: false,
        sniper: None,
        mode: WeaponMode::Melee,
        boss_deflected: None,
    };
    match id {
        "walk" => input.speed = 3.8,
        "run" => {
            input.speed = 6.0;
            input.sprinting = true;
        }
        "crouch" => input.crouched = true,
        "guard" => input.parrying = true,
        "death" => input.alive = false,
        _ => {}
    }
    let effect = match id {
        "attack" => Some(CombatEffect::Melee),
        "deflect" => Some(CombatEffect::Deflect),
        "parry" => Some(CombatEffect::Parry),
        "hit" => Some(CombatEffect::Hit),
        _ => None,
    };
    (input, effect)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same_pose(a: &RigPose, b: &RigPose) {
        for (a, b) in a.model.iter().zip(&b.model) {
            assert!(a.translation.distance(b.translation) < 0.000001);
            assert!(a.rotation.dot(b.rotation).abs() > 0.999999);
        }
        assert_eq!(a.source_clip, b.source_clip);
        assert_eq!(a.clock, b.clock);
        assert_eq!(a.crouch_weight, b.crouch_weight);
        assert_eq!(a.action_id, b.action_id);
        assert_eq!(a.event_sequence, b.event_sequence);
        match (&a.source, &b.source) {
            (Some(a), Some(b)) => {
                assert_eq!(a.weight, b.weight);
                assert_eq!(a.model, b.model);
            }
            (None, None) => {}
            _ => panic!("different source layers"),
        }
    }

    #[test]
    fn reset_seek_backward_incremental_and_configuration_changes_reconstruct_history() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::default();
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            for &(id, _) in SCENARIOS {
                let id = format!("game:{id}");
                let mut streamed = PreviewRig::new(&library);
                for frame in 0..=71 {
                    assert!(streamed.seek(
                        &library,
                        &source,
                        &id,
                        pair,
                        false,
                        frame as f32 / HZ,
                        false
                    ));
                }
                let expected = streamed.pose().clone();
                let mut direct = PreviewRig::new(&library);
                assert!(direct.seek(&library, &source, &id, pair, false, 71.0 / HZ, false));
                same_pose(&expected, direct.pose());
                for frame in [180.0, 0.0, 3.0, 21.0, 71.0] {
                    assert!(direct.seek(&library, &source, &id, pair, false, frame / HZ, false));
                }
                same_pose(&expected, direct.pose());
                assert!(direct.seek(
                    &library,
                    &source,
                    "source:skill_ylk_1",
                    pair,
                    false,
                    0.5,
                    false
                ));
                assert!(direct.seek(&library, &source, &id, pair, false, 71.0 / HZ, false));
                same_pose(&expected, direct.pose());
                assert!(direct.seek(&library, &source, &id, pair, false, 0.0, false));
                same_pose(&RigPose::new(&library), direct.pose());
            }
        }
    }

    #[test]
    fn catalog_is_complete_and_all_raw_samples_are_finite_without_hold_overrides() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::default();
        let clips = preview_catalog(&library, &source);
        assert_eq!(
            clips
                .iter()
                .filter(|clip| clip.id.starts_with("cc0:"))
                .count(),
            19
        );
        assert_eq!(
            clips
                .iter()
                .filter(|clip| clip.id.starts_with("source:"))
                .count(),
            35
        );
        let mut ids = std::collections::HashSet::new();
        for clip in clips {
            assert!(ids.insert(clip.id.clone()));
            assert!(!clip.label.is_empty() && !clip.group.is_empty());
            let mut preview = PreviewRig::new(&library);
            for time in [0.0, clip.duration * 0.5, clip.duration] {
                assert!(preview.seek(
                    &library,
                    &source,
                    &clip.id,
                    WeaponPair::HammerMusket,
                    false,
                    time,
                    false
                ));
                assert!(preview.pose().model.iter().all(Transform::is_finite));
                assert!(preview.pose().companion_hands.is_none());
                if let Some(source) = &preview.pose().source {
                    assert!(source.model.iter().all(|m| m.is_finite()));
                }
            }
        }
    }

    #[test]
    fn invalid_input_does_not_mutate_the_last_pose_and_loop_seek_keeps_the_tail() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::default();
        let mut preview = PreviewRig::new(&library);
        assert!(preview.seek(
            &library,
            &source,
            "game:attack",
            WeaponPair::HammerMusket,
            false,
            0.2,
            false
        ));
        let expected = preview.pose().clone();
        for (id, time) in [
            ("source:missing", 0.1),
            ("cc0:missing", 0.1),
            ("game:missing", 0.1),
            ("game:idle", f32::NAN),
            ("game:idle", -0.1),
        ] {
            assert!(!preview.seek(
                &library,
                &source,
                id,
                WeaponPair::HammerMusket,
                false,
                time,
                false
            ));
            same_pose(&expected, preview.pose());
        }
        let duration = library.duration("Death01");
        assert!(preview.seek(
            &library,
            &source,
            "cc0:Death01",
            WeaponPair::HammerMusket,
            false,
            duration,
            true
        ));
        let expected = library.model_pose(&library.sample("Death01", duration, false));
        assert_eq!(preview.pose().model, expected);
    }

    #[test]
    fn trace_keeps_fading_source_and_actual_normalized_sample_time() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::default();
        let mut preview = PreviewRig::new(&library);
        assert!(preview.seek(
            &library,
            &source,
            "game:attack",
            WeaponPair::HammerMusket,
            false,
            0.1,
            false
        ));
        let trace = preview.pose().trace();
        assert_eq!(trace.source_clip.as_deref(), Some("skill_ylk_1"));
        assert!((trace.source_seconds - 1.05).abs() < 0.00001);
        assert_eq!(
            trace.source_weight,
            preview.pose().source.as_ref().unwrap().weight
        );
        let duration = library.duration("Sword_Idle");
        assert!(preview.seek(
            &library,
            &source,
            "cc0:Sword_Idle",
            WeaponPair::HammerMusket,
            false,
            duration + 0.2,
            true
        ));
        assert!((preview.pose().trace().cc0_seconds - 0.2).abs() < 0.00001);
        assert!(preview.seek(
            &library,
            &source,
            "game:attack",
            WeaponPair::HammerMusket,
            false,
            37.0 / HZ,
            false
        ));
        let trace = preview.pose().trace();
        assert_eq!(trace.source_clip.as_deref(), Some("skill_ylk_1"));
        assert!(trace.source_weight > 0.0 && trace.source_weight < 1.0);
        assert!(preview.seek(
            &library,
            &source,
            "game:attack",
            WeaponPair::HammerMusket,
            false,
            1.0,
            false
        ));
        assert!(preview.pose().trace().source_clip.is_none());
        assert_eq!(preview.pose().trace().source_weight, 0.0);
    }

    #[test]
    fn longest_game_seek_has_bounded_replay_work() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::default();
        let mut preview = PreviewRig::new(&library);
        let mut timings = Vec::new();
        for _ in 0..10 {
            assert!(preview.seek(
                &library,
                &source,
                "game:guard",
                WeaponPair::HammerMusket,
                false,
                0.0,
                false
            ));
            let started = std::time::Instant::now();
            assert!(preview.seek(
                &library,
                &source,
                "game:guard",
                WeaponPair::HammerMusket,
                false,
                3.0,
                false
            ));
            timings.push(started.elapsed().as_micros());
            assert_eq!(preview.frame, 180);
            assert_eq!(preview.pose().source.as_ref().unwrap().weight, 1.0);
        }
        timings.sort();
        eprintln!(
            "inspection native guard 180-step seek: median={}us, max={}us",
            timings[5], timings[9]
        );
    }
}
