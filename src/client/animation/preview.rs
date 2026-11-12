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
    ("crouch_attack", "蹲伏普攻 · 上身动作组合"),
    ("crouch_guard", "蹲伏防御 · 上身动作组合"),
    ("walk_guard", "行走防御 · 上身动作组合"),
    ("attack", "近战普攻 · 第 1 段"),
    ("attack2", "近战普攻 · 第 2 段"),
    ("attack3", "近战普攻 · 第 3 段"),
    ("attack_sequence", "近战普攻 · 三段连贯"),
    ("jump", "一段跳 / 下落 / 落地"),
    ("double_jump", "二段跳 / 下落 / 落地"),
    ("dodge_front", "向前闪避"),
    ("dodge_sprint", "疾跑闪避（实际 0.26 秒）"),
    ("dodge_back", "向后闪避"),
    ("dodge_left", "向左闪避"),
    ("dodge_right", "向右闪避"),
    ("crouch_walk", "蹲伏行走"),
    ("slide", "滑铲（现有动作回退）"),
    ("guard", "持续防御"),
    ("deflect", "拆招反馈"),
    ("parry", "弹反反馈"),
    ("parry_interrupt_walk", "弹反尾势 · 短走打断检查"),
    ("parry_interrupt_hit", "弹反尾势 · 受击打断检查"),
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
        ("source", "燕云源片段对照（已适配根运动）", source.clips()),
    ] {
        clips.extend(entries.into_iter().map(|(name, duration)| PreviewClip {
            id: format!("{prefix}:{name}"),
            label: name.into(),
            group,
            duration,
        }));
    }
    if let Some(naraka) = source.for_rig(SourceRig::Naraka) {
        clips.extend(
            naraka
                .clips()
                .into_iter()
                .map(|(name, duration)| PreviewClip {
                    id: format!("naraka:{name}"),
                    label: name.into(),
                    group: "永劫源片段对照（原生 Humanoid 烘焙）",
                    duration,
                }),
        );
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
            if self.id != id
                || self.pair != pair
                || self.blade != blade
                || frame < self.frame
                || self.frame == 0 && frame > 0
            {
                self.rig = RigPose::new(library);
                self.frame = 0;
            }
            let start = if frame == 0 && source.for_rig(SourceRig::Naraka).is_some() {
                0
            } else {
                self.frame + 1
            };
            for step in start..=frame {
                let seconds = step as f32 * STEP;
                let (mut input, effect) = scenario_input(scenario);
                let (action_snapshot, defense) =
                    scenario_snapshot(scenario, (seconds - STEP).max(0.0), pair);
                let action_snapshot = action_snapshot.filter(|_| !blade);
                scenario_motion(scenario, seconds, &mut input);
                if let Some(snapshot) = defense {
                    input.parrying = snapshot.kind != DefenseKind::Idle;
                }
                let action =
                    action_snapshot.map(|action| (action.action_id, Some(CombatEffect::Melee)));
                let event = if scenario == "parry_interrupt_hit" && step == 12 {
                    Some(CombatEffect::Hit)
                } else {
                    (step == 1).then_some(effect).flatten()
                };
                let latest_sequence = if scenario == "parry_interrupt_hit" && step >= 12 {
                    2
                } else {
                    u64::from(step >= 1 && effect.is_some())
                };
                self.rig.frame(
                    library,
                    Some(source),
                    DisplayFrame {
                        input,
                        pair,
                        blade,
                        action,
                        latest_sequence,
                        action_snapshot,
                        defense,
                    },
                    event.into_iter().map(|kind| (latest_sequence, 1, kind)),
                    if step == 0 { 0.0 } else { STEP },
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
        } else if let Some((source, name)) = id
            .strip_prefix("source:")
            .or_else(|| id.strip_prefix("yanyun:"))
            .map(|name| (source, name))
            .or_else(|| {
                id.strip_prefix("naraka:").and_then(|name| {
                    source
                        .for_rig(SourceRig::Naraka)
                        .map(|source| (source, name))
                })
            })
        {
            let Some((_, duration)) = source.clips().into_iter().find(|&(value, _)| value == name)
            else {
                return false;
            };
            let clip = source.clip(name).expect("catalog clip");
            let time = clip_time(seconds, duration, looping);
            self.rig = RigPose::new(library);
            self.rig.source = Some(SourcePose {
                scope: source.scope(clip),
                rig: source.rig(),
                model: source.model_pose(&source.sample(clip, time, false, false)),
                weight: 1.0,
            });
            self.rig.trace = PoseTrace {
                state: "source comparison (no hold override)",
                source_rig: Some(source.rig().id()),
                source_layers: Some(source.layer_description(clip)),
                source_scope: Some(source.scope(clip).label()),
                adaptation: Some(
                    "native source matrices; declared target layer composition / root normalization; no weapon IK",
                ),
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
        boss_attack: None,
        alive: true,
        airborne: false,
        jumps_used: 0,
        speed: 0.0,
        sprinting: false,
        dodging: false,
        dodge_elapsed: 0.0,
        dodge_duration: 0.42,
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
        "crouch" | "crouch_attack" | "crouch_guard" => input.crouched = true,
        "guard" => input.parrying = true,
        "death" => input.alive = false,
        _ => {}
    }
    if matches!(id, "crouch_guard" | "walk_guard") {
        input.parrying = true;
    }
    if id == "walk_guard" {
        input.speed = 3.8 * 0.35;
    }
    let effect = match id {
        "attack" | "crouch_attack" => Some(CombatEffect::Melee),
        "deflect" => Some(CombatEffect::Deflect),
        "parry" | "parry_interrupt_walk" | "parry_interrupt_hit" => Some(CombatEffect::Parry),
        "hit" => Some(CombatEffect::Hit),
        _ => None,
    };
    (input, effect)
}

/// Synthetic read-only snapshots exercise the exact live display adapter. They
/// do not install simulation, hit resolution or audio in the inspection App.
fn scenario_snapshot(
    id: &str,
    seconds: f32,
    pair: WeaponPair,
) -> (Option<ActionSnapshot>, Option<DefenseSnapshot>) {
    let stage = match id {
        "attack" | "crouch_attack" => Some(MeleeStage::One),
        "attack2" => Some(MeleeStage::Two),
        "attack3" => Some(MeleeStage::Three),
        "attack_sequence" => {
            match (seconds / pair.melee_profile(MeleeStage::One).duration) as u32 {
                0 => Some(MeleeStage::One),
                1 => Some(MeleeStage::Two),
                2 => Some(MeleeStage::Three),
                _ => None,
            }
        }
        _ => None,
    };
    let action = stage.and_then(|stage| {
        let duration = pair.melee_profile(stage).duration;
        let offset = if id == "attack_sequence" {
            match stage {
                MeleeStage::One => 0.0,
                MeleeStage::Two => duration,
                MeleeStage::Three => duration * 2.0,
            }
        } else {
            0.0
        };
        (seconds - offset < duration).then_some(ActionSnapshot {
            action_id: match stage {
                MeleeStage::One => 1,
                MeleeStage::Two => 2,
                MeleeStage::Three => 3,
            },
            kind: ActionKind::Melee,
            pair,
            melee_stage: Some(stage),
            target: None,
            elapsed: seconds - offset,
            duration,
            shot_count: 0,
            aim_position: Vec3::ZERO,
            source_position: Vec3::ZERO,
            companion_position: Vec3::ZERO,
        })
    });
    let defense = match id {
        "guard" | "crouch_guard" | "walk_guard" => Some(DefenseSnapshot {
            kind: if seconds < 0.20 {
                DefenseKind::Deflect
            } else {
                DefenseKind::Guard
            },
            elapsed: (seconds - 0.20).max(0.0),
            remaining: (0.20 - seconds).max(0.0),
            gesture_id: 1,
            last_gesture_kind: DefenseKind::Deflect,
            gesture_elapsed: seconds,
        }),
        "deflect" | "parry" | "parry_interrupt_walk" | "parry_interrupt_hit" => {
            let parry = id != "deflect";
            let duration = if parry { 0.18 } else { 0.20 };
            let kind = if parry {
                DefenseKind::Parry
            } else {
                DefenseKind::Deflect
            };
            Some(DefenseSnapshot {
                kind: if seconds < duration {
                    kind
                } else {
                    DefenseKind::Idle
                },
                elapsed: seconds,
                remaining: (duration - seconds).max(0.0),
                gesture_id: 1,
                last_gesture_kind: kind,
                gesture_elapsed: seconds,
            })
        }
        _ => None,
    };
    (action, defense)
}

fn scenario_motion(id: &str, seconds: f32, input: &mut Inputs) {
    if id == "parry_interrupt_walk" && (0.25..0.35).contains(&seconds) {
        input.speed = 3.8;
    }
    if id == "crouch_walk" {
        input.crouched = true;
        input.speed = 2.0;
    }
    if id == "slide" {
        input.sliding = seconds < 0.60;
    }
    if id.starts_with("dodge_") {
        input.dodge_duration = if id == "dodge_sprint" { 0.26 } else { 0.42 };
        input.dodging = seconds < input.dodge_duration;
        input.dodge_elapsed = seconds;
        input.dodge_yaw = match id {
            "dodge_left" => std::f32::consts::FRAC_PI_2,
            "dodge_right" => -std::f32::consts::FRAC_PI_2,
            "dodge_back" => std::f32::consts::PI,
            _ => 0.0,
        };
    }
    if matches!(id, "jump" | "double_jump") {
        input.airborne = seconds < if id == "double_jump" { 1.0 } else { 0.7 };
        input.jumps_used = if !input.airborne {
            0
        } else if id == "double_jump" && seconds >= 0.25 {
            2
        } else {
            1
        };
    }
    if matches!(id, "deflect" | "parry") {
        input.parrying = seconds < if id == "parry" { 0.18 } else { 0.20 };
    }
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
        assert_eq!(a.interrupted_defense, b.interrupted_defense);
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
        let source = SourceAnimationLibrary::bundled();
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
                let mut initial = PreviewRig::new(&library);
                assert!(initial.seek(&library, &source, &id, pair, false, 0.0, false));
                same_pose(initial.pose(), direct.pose());
            }
        }
    }

    #[test]
    fn catalog_is_complete_and_all_raw_samples_are_finite_without_hold_overrides() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::bundled();
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
        let source = SourceAnimationLibrary::bundled();
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
        let source = SourceAnimationLibrary::bundled();
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
        assert_eq!(
            trace.source_clip.as_deref(),
            Some("male_hengdao_attack_light_01")
        );
        assert_eq!(trace.source_rig, Some(SourceRig::Naraka.id()));
        let native = source.for_rig(SourceRig::Naraka).unwrap();
        let expected = (0.1 - STEP) / 0.60
            * native.duration(native.clip("male_hengdao_attack_light_01").unwrap());
        assert!((trace.source_seconds - expected).abs() < 0.00001);
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
            true,
            27.0 / HZ,
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
            true,
            1.0,
            false
        ));
        assert!(preview.pose().trace().source_clip.is_none());
        assert_eq!(preview.pose().trace().source_weight, 0.0);
    }

    #[test]
    fn longest_game_seek_has_bounded_replay_work() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::bundled();
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
    #[test]
    fn seeking_reconstructs_short_walk_and_hit_interruptions_without_reviving_the_same_riposte() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::bundled();
        for id in ["game:parry_interrupt_walk", "game:parry_interrupt_hit"] {
            let mut streamed = PreviewRig::new(&library);
            for step in 0..=33 {
                assert!(streamed.seek(
                    &library,
                    &source,
                    id,
                    WeaponPair::HammerMusket,
                    false,
                    step as f32 / HZ,
                    false
                ));
            }
            let expected = streamed.pose().clone();
            assert_eq!(expected.interrupted_defense, Some(1));
            assert_eq!(
                expected.trace.source_clip.as_deref(),
                Some("male_hengdao_idle_01")
            );
            let mut direct = PreviewRig::new(&library);
            assert!(direct.seek(
                &library,
                &source,
                id,
                WeaponPair::HammerMusket,
                false,
                0.55,
                false
            ));
            same_pose(&expected, direct.pose());
            assert!(direct.seek(
                &library,
                &source,
                id,
                WeaponPair::HammerMusket,
                false,
                0.10,
                false
            ));
            assert_eq!(direct.pose().interrupted_defense, None);
            assert_eq!(
                direct.pose().trace.source_clip.as_deref(),
                Some("male_hengdao_attack_shock_light_01")
            );
            assert!(direct.seek(
                &library,
                &source,
                id,
                WeaponPair::HammerMusket,
                false,
                0.55,
                false
            ));
            same_pose(&expected, direct.pose());
        }
    }
}
