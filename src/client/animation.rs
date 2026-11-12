//! Authored skeletal animation driven by read-only simulation snapshots and actor-tagged events.
mod attack_clips;
#[cfg(test)]
mod attack_replay;
mod naraka;
mod poses;
mod preview;

#[cfg(test)]
pub(super) use attack_replay::AttackReplay;
pub(super) use preview::{PreviewClip, PreviewRig, preview_catalog};

use bevy::prelude::*;

use super::{
    ActorView,
    skeleton::{AnimationLibrary, blend},
    source_animation::{SourceAnimationLibrary, SourceRig, SourceScope},
};
use crate::simulation::{
    ActionKind, ActionSnapshot, AttackClock, Body, Boss, BossState, CombatEffect, CombatFeedback,
    Combatant, DefenseKind, DefenseSnapshot, DefenseState, Enemy, Facing, Health, HitStop,
    MeleeStage, MotionState, SimulationActive, WeaponMode, WeaponPair,
};
use poses::Gesture;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PoseState {
    Idle,
    Walk,
    Run,
    Air(u8),
    Land,
    Dodge,
    Crouch,
    Slide,
    Parry,
    Aim,
    Attack(Gesture),
    Down,
    Deflected,
}

#[derive(Clone)]
struct Playback {
    kind: Gesture,
    elapsed: f32,
}

#[derive(Clone)]
pub(super) struct SourcePose {
    pub(super) scope: SourceScope,
    pub(super) rig: super::source_animation::SourceRig,
    pub(super) model: Vec<Mat4>,
    pub(super) weight: f32,
}

/// Read alongside the actual sampling branches; the web UI never derives crops.
#[derive(Clone, Default)]
pub(super) struct PoseTrace {
    pub(super) state: &'static str,
    pub(super) cc0_clip: Option<String>,
    pub(super) cc0_seconds: f32,
    pub(super) source_clip: Option<String>,
    pub(super) source_rig: Option<&'static str>,
    pub(super) source_layers: Option<String>,
    pub(super) source_scope: Option<&'static str>,
    pub(super) adaptation: Option<&'static str>,
    pub(super) fallback: Option<&'static str>,
    pub(super) melee_stage: Option<MeleeStage>,
    pub(super) defense: Option<DefenseKind>,
    pub(super) defense_seconds: f32,
    pub(super) defense_gesture: Option<(DefenseKind, f32)>,
    pub(super) defense_interrupted: bool,
    pub(super) source_seconds: f32,
    pub(super) source_weight: f32,
    pub(super) transition: f32,
    pub(super) crouch_weight: f32,
    pub(super) companion_ik: bool,
}

#[derive(Component, Clone)]
pub(super) struct RigPose {
    /// View the asset's imported current reference, which need not equal bind.
    pub(super) reference_pose: bool,
    trace: PoseTrace,
    pub(super) source: Option<SourcePose>,
    /// Render-only grip targets, retargeted with each companion body's own lengths.
    pub(super) companion_hands: Option<[Vec3; 2]>,
    source_local: Option<Vec<Transform>>,
    source_from: Option<Vec<Transform>>,
    source_clip: Option<(SourceRig, usize)>,
    source_blend: f32,
    local: Vec<Transform>,
    pub(super) model: Vec<Transform>,
    from: Vec<Transform>,
    state: PoseState,
    clock: f32,
    gait_time: f32,
    transition: f32,
    air_time: f32,
    was_airborne: bool,
    jumps_used: u8,
    crouch_weight: f32,
    event_sequence: u64,
    action_id: u64,
    playback: Option<Playback>,
    restart: bool,
    interrupted_defense: Option<u64>,
}

impl RigPose {
    pub fn new(library: &AnimationLibrary) -> Self {
        let local = poses::idle(library, 0.0, WeaponMode::Melee);
        Self {
            reference_pose: false,
            trace: PoseTrace {
                state: "idle",
                cc0_clip: Some("Sword_Idle".into()),
                transition: 1.0,
                ..default()
            },
            source: None,
            companion_hands: None,
            source_local: None,
            source_from: None,
            source_clip: None,
            source_blend: 0.0,
            model: library.model_pose(&local),
            from: local.clone(),
            local,
            state: PoseState::Idle,
            clock: 0.0,
            gait_time: 0.0,
            transition: 1.0,
            air_time: 0.0,
            was_airborne: false,
            jumps_used: 0,
            crouch_weight: 0.0,
            event_sequence: 0,
            action_id: 0,
            playback: None,
            restart: false,
            interrupted_defense: None,
        }
    }

    pub(super) fn trace(&self) -> &PoseTrace {
        &self.trace
    }

    /// One display frame. Both adapters supply snapshots/cues; state priority,
    /// cue deduplication, crops, transitions and IK stay here.
    fn frame(
        &mut self,
        library: &AnimationLibrary,
        source: Option<&SourceAnimationLibrary>,
        frame: DisplayFrame,
        events: impl IntoIterator<Item = (u64, u64, CombatEffect)>,
        dt: f32,
    ) {
        self.reference_pose = false;
        self.companion_hands = None;
        let native_player = !frame.blade
            && source.is_some_and(|library| library.for_rig(SourceRig::Naraka).is_some());
        if let Some((action_id, effect)) = frame.action
            && action_id != self.action_id
        {
            if let Some(effect) = effect {
                self.observe(library, effect, frame.pair, false);
            }
            self.action_id = action_id;
        }
        let previous = self.event_sequence;
        let mut observed_hit = false;
        for (sequence, action_id, effect) in events {
            if sequence > previous
                && !(native_player
                    && matches!(
                        effect,
                        CombatEffect::Melee
                            | CombatEffect::MeleeStart
                            | CombatEffect::Deflect
                            | CombatEffect::Block
                            | CombatEffect::Parry
                    ))
                && !(action_id == self.action_id
                    && matches!(
                        effect,
                        CombatEffect::Melee | CombatEffect::Hammer | CombatEffect::ChainSweep
                    ))
            {
                observed_hit |= matches!(effect, CombatEffect::Hit | CombatEffect::SniperImpact);
                self.observe(library, effect, frame.pair, frame.blade);
            }
        }
        // A long render frame may both observe and finish a hit playback.
        // Preserve the causal interruption before advance() retires that pose.
        if observed_hit
            && native_player
            && let Some(defense) = frame.defense
            && matches!(
                defense.last_gesture_kind,
                DefenseKind::Parry | DefenseKind::Deflect
            )
        {
            self.interrupted_defense = Some(defense.gesture_id);
        }
        self.event_sequence = frame.latest_sequence;
        self.advance(library, frame.input, dt);
        if let Some(source) = source {
            self.advance_sources(
                source,
                frame.input,
                frame.action_snapshot,
                frame.defense,
                native_player,
                dt,
            );
        }
    }

    pub(super) fn sample_companion(
        &mut self,
        library: &AnimationLibrary,
        state: &crate::simulation::CompanionState,
        pitch: f32,
        grips: [Vec3; 2],
    ) {
        let elapsed = state.elapsed;
        let shots = state.shot_count;
        let last = shots
            .checked_sub(1)
            .and_then(|index| state.kind.shot_times().get(index as usize))
            .copied();
        let recoil = last.map_or(0.0, |time| {
            (1.0 - (elapsed - time) / 0.10).clamp(0.0, 1.0) * 0.06
        });
        self.companion_pose(library, elapsed, pitch, grips, recoil);
    }

    /// The same native donor and explicit physical grip correction are used
    /// by the live companion and its inspection scenario.
    pub(super) fn companion_source(
        &mut self,
        sources: &SourceAnimationLibrary,
        pair: WeaponPair,
        elapsed: f32,
        shot_age: Option<f32>,
    ) {
        let Some(source) = sources.for_rig(SourceRig::Naraka) else {
            return;
        };
        let attack = shot_age.is_some_and(|age| (0.0..0.40).contains(&age));
        let loadout = super::loadout::of(pair);
        let selected = if attack {
            &loadout.companion_attack
        } else {
            &loadout.companion_hold
        };
        let (name, adaptation) = (selected.name, selected.adaptation);
        let Some(clip) = source.clip(name) else {
            return;
        };
        let seconds = if attack {
            shot_age.unwrap() / 0.40 * source.duration(clip)
        } else {
            elapsed
        };
        let target = source.sample(clip, seconds, !attack, false);
        self.source = Some(SourcePose {
            scope: SourceScope::UpperBody,
            rig: SourceRig::Naraka,
            model: source.model_pose(&target),
            weight: 1.0,
        });
        self.source_local = Some(target);
        self.source_clip = Some((SourceRig::Naraka, clip));
        self.trace.source_clip = Some(name.into());
        self.trace.source_rig = Some(SourceRig::Naraka.id());
        self.trace.source_layers = Some(source.layer_description(clip));
        self.trace.source_scope = Some(SourceScope::UpperBody.label());
        self.trace.source_seconds = source.sample_time(clip, seconds, !attack);
        self.trace.source_weight = 1.0;
        self.trace.adaptation = Some(adaptation);
    }

    /// Inspect only the companion hold/aim calculation, with no shot sequence.
    pub(super) fn sample_companion_hold(
        &mut self,
        library: &AnimationLibrary,
        elapsed: f32,
        pitch: f32,
        grips: [Vec3; 2],
    ) {
        self.source = None;
        self.source_local = None;
        self.source_from = None;
        self.source_clip = None;
        self.companion_pose(library, elapsed, pitch, grips, 0.0);
    }

    fn companion_pose(
        &mut self,
        library: &AnimationLibrary,
        elapsed: f32,
        pitch: f32,
        grips: [Vec3; 2],
        recoil: f32,
    ) {
        self.reference_pose = false;
        let mut local = poses::idle(library, elapsed, WeaponMode::Ranged);
        poses::aim_ranged(library, &mut local, pitch, recoil);
        self.model = library.model_pose(&local);
        // Palette retargeting solves the female arms against these weapon-local
        // grips while retaining the authored lower-body and recoil choreography.
        self.companion_hands = Some(grips);
        for (side, hand) in [("r", grips[0]), ("l", grips[1])] {
            let wrist = library.bone(&format!("hand_{side}"));
            self.model[wrist].translation = hand;
        }
        self.trace = PoseTrace {
            state: "companion hold/aim",
            cc0_clip: Some("Sword_Idle".into()),
            cc0_seconds: library.sample_time("Sword_Idle", elapsed, true),
            companion_ik: true,
            transition: 1.0,
            ..default()
        };
    }

    /// Uses the already selected display state and clock. These are explicit
    /// Kiva crops/retargets of source motion, not recovered gameplay hit frames.
    #[cfg(test)]
    fn advance_source(&mut self, library: &SourceAnimationLibrary, input: Inputs, dt: f32) {
        self.advance_sources(library, input, None, None, false, dt);
    }

    fn advance_sources(
        &mut self,
        libraries: &SourceAnimationLibrary,
        input: Inputs,
        action: Option<ActionSnapshot>,
        defense: Option<DefenseSnapshot>,
        native_player: bool,
        dt: f32,
    ) {
        if dt <= 0.0 && self.source.is_some() {
            return;
        }
        if native_player {
            self.latch_defense_interruption(input, action, defense);
        }
        let first = self.source_local.is_none() && self.source_clip.is_none();
        self.trace.fallback = if native_player && input.sliding {
            Some("CC0 slide; no verified Naraka replacement")
        } else if native_player && action.is_some_and(|action| action.kind != ActionKind::Melee) {
            Some("existing Kiva skill choreography (Yanyun / CC0)")
        } else {
            None
        };
        let legacy_selection = if input.mode == WeaponMode::Ranged
            || (input.crouched && !input.airborne)
            || input.sliding
        {
            None
        } else {
            let clocked = input
                .attack
                .and_then(|clock| attack_clips::of(clock.track).map(|row| (row, clock)))
                .filter(|(row, _)| row.rig == SourceRig::Yanyun);
            match self.state {
                PoseState::Attack(
                    Gesture::Slash | Gesture::Axe | Gesture::Hammer | Gesture::Windup,
                ) if let Some((row, clock)) = clocked => {
                    Some((row.clip, row.position(&clock), false))
                }
                PoseState::Attack(Gesture::Slash | Gesture::Axe | Gesture::Hammer) => {
                    let duration = if self.state == PoseState::Attack(Gesture::Hammer) {
                        0.6
                    } else {
                        0.42
                    };
                    Some((
                        "skill_ylk_1",
                        0.9 + (self.clock / duration).clamp(0.0, 1.0) * 0.9,
                        false,
                    ))
                }
                PoseState::Attack(Gesture::Windup) => Some((
                    "skill_ylk_1",
                    0.3 + (self.clock / 0.4).clamp(0.0, 1.0) * 0.6,
                    false,
                )),
                PoseState::Deflected => Some((
                    "weakhit_b",
                    input.boss_deflected.map_or(0.0, |(elapsed, duration)| {
                        elapsed / duration.max(0.001) * 0.8
                    }),
                    false,
                )),
                PoseState::Attack(Gesture::Hit | Gesture::HeavyHit) => {
                    Some(("weakhit_b", (self.clock / 0.3).min(1.0) * 0.3, false))
                }
                PoseState::Parry | PoseState::Attack(Gesture::Guard) => Some((
                    "skill_huajie_01__shuangqiangt",
                    0.1 + (self.clock / 0.25).min(1.0) * 0.55,
                    false,
                )),
                PoseState::Dodge => Some((
                    "skill_shoujia_dodge_f",
                    0.2 + (input.dodge_elapsed / 0.42).min(1.0) * 0.8,
                    false,
                )),
                PoseState::Air(_) if self.air_time < 0.18 => Some((
                    "b_fight_env_jump_01_start_Lfoot__zuiquan",
                    0.2 + self.air_time / 0.18 * 0.5,
                    false,
                )),
                PoseState::Air(_) => Some((
                    "b_fight_env_jump_02_loop__zuiquan",
                    self.air_time - 0.18,
                    true,
                )),
                PoseState::Land => Some((
                    "b_fight_env_jump_03_to_idle__zuiquan",
                    0.1 + self.clock * 3.0,
                    false,
                )),
                _ => None,
            }
        };
        // Enemies and the Boss play their attack row on the simulation's own
        // attack clock; the native player reaches the same rows below.
        let attack_selection = input
            .attack
            .filter(|_| {
                !native_player
                    && input.alive
                    && !input.airborne
                    && input.boss_deflected.is_none()
                    && !matches!(
                        self.state,
                        PoseState::Attack(Gesture::Hit | Gesture::HeavyHit)
                    )
            })
            .and_then(|clock| attack_clips::of(clock.track).map(|row| (row, clock)))
            .filter(|(row, _)| libraries.for_rig(row.rig).is_some())
            .map(|(row, clock)| naraka::Selection {
                rig: row.rig,
                name: row.clip,
                seconds: row.position(&clock),
                looping: false,
                adaptation: "Kiva attack clock; attack clip table",
            });
        let selection = attack_selection.or_else(|| {
            if native_player {
                self.naraka_selection(input, action, defense).or_else(|| {
                    legacy_selection.map(|(name, seconds, looping)| naraka::Selection {
                        rig: SourceRig::Yanyun,
                        name,
                        seconds,
                        looping,
                        adaptation: "explicit Yanyun fallback for Kiva skill / CC0 slide",
                    })
                })
            } else {
                legacy_selection.map(|(name, seconds, looping)| naraka::Selection {
                    rig: SourceRig::Yanyun,
                    name,
                    seconds,
                    looping,
                    adaptation: "Yanyun original hierarchy",
                })
            }
        });
        self.trace.melee_stage = input
            .attack
            .and_then(|clock| match clock.track {
                crate::simulation::TrackId::Boss { stage } => Some(stage),
                _ => None,
            })
            .or_else(|| action.and_then(|action| action.melee_stage));
        self.trace.defense = defense.map(|defense| defense.kind);
        self.trace.defense_interrupted =
            defense.is_some_and(|defense| self.interrupted_defense == Some(defense.gesture_id));
        self.trace.defense_seconds = defense.map_or(0.0, |defense| defense.elapsed);
        self.trace.defense_gesture =
            defense.map(|defense| (defense.last_gesture_kind, defense.gesture_elapsed));
        let selected = selection.and_then(|selection| {
            let library = libraries.for_rig(selection.rig)?;
            let clip = library.clip(selection.name)?;
            let normalized = selection.rig == SourceRig::Naraka
                && (selection.name.starts_with("male_hengdao_attack_light_")
                    || selection.name.contains("_dodge_")
                    || selection.name == "male_hengdao_attack_shock_light_01"
                    || selection.name.contains("jump_inplace_up")
                    || selection.name == "male_hengdao_jump_down_to_land_01");
            let seconds = selection.seconds
                * if normalized {
                    library.duration(clip)
                } else {
                    1.0
                };
            Some((
                library,
                clip,
                seconds,
                selection.looping,
                selection.adaptation,
            ))
        });
        // Cross-rig transitions go through the common target reference. Their
        // independent node arrays must never be zipped or interpreted together.
        if selected.as_ref().is_some_and(|(library, ..)| {
            self.source
                .as_ref()
                .is_some_and(|source| source.rig != library.rig())
        }) {
            if let Some(source) = &mut self.source {
                source.weight = (source.weight - dt / 0.08).max(0.0);
            }
            if self
                .source
                .as_ref()
                .is_some_and(|source| source.weight > 0.0)
            {
                self.trace.source_weight = self.source.as_ref().unwrap().weight;
                return;
            }
            self.source = None;
            self.source_local = None;
            self.source_from = None;
            self.source_clip = None;
        }
        if let Some((library, clip, seconds, looping, adaptation)) = selected {
            let name = library.clip_name(clip).expect("selected source clip");
            let defense_clip = matches!(
                name,
                "male_hengdao_shock_01" | "male_hengdao_attack_shock_light_01"
            );
            let crouched_melee =
                input.crouched && action.is_some_and(|action| action.kind == ActionKind::Melee);
            let composed_lower = library.rig() == SourceRig::Naraka
                && !input.airborne
                && (crouched_melee || defense_clip && (input.crouched || input.speed > 0.01));
            let scope = if composed_lower {
                SourceScope::UpperBody
            } else {
                library.scope(clip)
            };
            self.trace.source_rig = Some(library.rig().id());
            self.trace.source_layers = Some(library.layer_description(clip));
            self.trace.source_scope = Some(scope.label());
            self.trace.adaptation = Some(if composed_lower {
                "Kiva native upper body / fingers + actual CC0 crouch/gait lower body (0.12s crouch blend)"
            } else {
                adaptation
            });
            self.trace.source_clip = library.clip_name(clip).map(str::to_owned);
            self.trace.source_seconds = library.sample_time(clip, seconds, looping);
            let mut target = library.sample(clip, seconds, looping, input.airborne);
            // A dodge uses one forward source clip; the simulation's accepted
            // direction supplies only this display-space facing adjustment.
            if self.state == PoseState::Dodge && library.rig() == SourceRig::Yanyun {
                let root = library.bone("Bip001").unwrap();
                target[root].rotation =
                    Quat::from_rotation_y(input.dodge_yaw) * target[root].rotation;
            }
            if self.source_clip != Some((library.rig(), clip)) || self.transition <= dt {
                self.source_from = self.source_local.clone();
                self.source_blend = 0.0;
            }
            self.source_blend += dt;
            if let Some(from) = &self.source_from {
                let weight = poses::smooth(self.source_blend / 0.08);
                for (value, previous) in target.iter_mut().zip(from) {
                    *value = blend(*previous, *value, weight);
                }
            }
            let weight = self.source.as_ref().map_or(0.0, |source| source.weight);
            self.source = Some(SourcePose {
                scope,
                rig: library.rig(),
                model: library.model_pose(&target),
                weight: if first && dt == 0.0 {
                    1.0
                } else {
                    (weight + dt / 0.055).min(1.0)
                },
            });
            self.source_local = Some(target);
            self.source_clip = Some((library.rig(), clip));
        } else if let Some(source) = &mut self.source {
            source.weight = (source.weight - dt / 0.12).max(0.0);
            if source.weight == 0.0 {
                self.source = None;
                self.source_local = None;
                self.source_from = None;
                self.source_clip = None;
                self.trace.source_clip = None;
                self.trace.source_rig = None;
                self.trace.source_layers = None;
                self.trace.source_scope = None;
                self.trace.adaptation = None;
                self.trace.source_seconds = 0.0;
            }
        }
        self.trace.source_weight = self.source.as_ref().map_or(0.0, |source| source.weight);
    }

    fn observe(
        &mut self,
        library: &AnimationLibrary,
        effect: CombatEffect,
        set: WeaponPair,
        blade: bool,
    ) {
        let kind = match effect {
            CombatEffect::Melee if blade => Gesture::Slash,
            CombatEffect::Melee => match super::loadout::of(set).swing {
                super::loadout::Swing::Hammer => Gesture::Hammer,
                super::loadout::Swing::Axe => Gesture::Axe,
            },
            CombatEffect::Shot => Gesture::Shot,
            CombatEffect::Hammer => Gesture::Hammer,
            CombatEffect::ChainSweep => Gesture::Axe,
            CombatEffect::AxeThrow => Gesture::Bow,
            CombatEffect::Hit => Gesture::Hit,
            CombatEffect::SniperImpact => Gesture::HeavyHit,
            CombatEffect::Deflect | CombatEffect::Block | CombatEffect::Parry => Gesture::Guard,
            CombatEffect::EnemyWindup => Gesture::Windup,
            CombatEffect::Respawn => {
                *self = Self::new(library);
                return;
            }
            CombatEffect::Defeat
            | CombatEffect::Dodge
            | CombatEffect::SniperShot
            | CombatEffect::ShotImpact
            | CombatEffect::Rejected
            | CombatEffect::Enrage
            | CombatEffect::MeleeStart => return,
        };
        self.playback = Some(Playback { kind, elapsed: 0.0 });
        self.restart = true;
    }

    fn advance(&mut self, library: &AnimationLibrary, input: Inputs, dt: f32) {
        let landed = self.was_airborne && !input.airborne;
        self.was_airborne = input.airborne;
        let jumped = input.jumps_used > self.jumps_used;
        self.jumps_used = input.jumps_used;
        self.air_time = if input.airborne {
            if jumped { dt } else { self.air_time + dt }
        } else {
            0.0
        };
        if let Some(playback) = &mut self.playback {
            playback.elapsed += dt;
            if playback.elapsed >= playback.kind.duration() {
                self.playback = None;
            }
        }
        if input.sniper.is_some_and(|sniper| sniper.aiming)
            && self
                .playback
                .as_ref()
                .is_some_and(|p| !matches!(p.kind, Gesture::Hit | Gesture::HeavyHit))
        {
            self.playback = None;
        }
        let state = if !input.alive {
            PoseState::Down
        } else if input.boss_deflected.is_some() {
            self.playback = None;
            PoseState::Deflected
        } else if let Some(playback) = self
            .playback
            .as_ref()
            .filter(|playback| matches!(playback.kind, Gesture::Hit | Gesture::HeavyHit))
        {
            PoseState::Attack(playback.kind)
        } else if input.dodging {
            PoseState::Dodge
        } else if input.sliding {
            PoseState::Slide
        } else if input.parrying {
            PoseState::Parry
        } else if input.sniper.is_some_and(|sniper| sniper.aiming) {
            PoseState::Aim
        } else if let Some(playback) = &self.playback {
            PoseState::Attack(playback.kind)
        } else if input.airborne {
            PoseState::Air(input.jumps_used)
        } else if landed || (self.state == PoseState::Land && self.clock < 0.16) {
            PoseState::Land
        } else if input.crouched {
            PoseState::Crouch
        } else if input.speed > 0.01 {
            if input.sprinting {
                PoseState::Run
            } else {
                PoseState::Walk
            }
        } else {
            PoseState::Idle
        };
        let restart = std::mem::take(&mut self.restart);
        if state != self.state || restart {
            self.from = self.local.clone();
            self.state = state;
            self.clock = 0.0;
            self.gait_time = 0.0;
            self.transition = 0.0;
        }
        self.clock += dt;
        self.transition += dt;
        self.gait_time += dt
            * match state {
                PoseState::Run => 1.6 * input.speed / 6.0,
                PoseState::Walk => 1.2 * input.speed / 3.8,
                PoseState::Crouch => 1.2 * input.speed / 3.8,
                PoseState::Parry => 0.85 * input.speed / 3.8,
                _ => 0.0,
            };
        let (mut target, sampled) = match state {
            PoseState::Idle => trace_pose(poses::idle_with_trace(library, self.clock, input.mode)),
            PoseState::Walk | PoseState::Run => sample_traced(
                library,
                if state == PoseState::Run {
                    "Jog_Fwd_Loop"
                } else {
                    "Walk_Loop"
                },
                self.gait_time,
                true,
            ),
            // A two-clip airborne blend has no single CC0 source sample.
            PoseState::Air(jumps) => (airborne_pose(library, self.air_time, jumps), None),
            PoseState::Land => sample_traced(library, "Jump_Land", 0.15 + self.clock * 4.0, false),
            PoseState::Dodge => {
                let (mut pose, sample) = sample_traced(
                    library,
                    "Roll",
                    input.dodge_elapsed / 0.42 * library.duration("Roll"),
                    false,
                );
                pose[0].rotation = Quat::from_rotation_y(input.dodge_yaw) * pose[0].rotation;
                (pose, sample)
            }
            PoseState::Crouch => {
                if input.speed > 0.01 {
                    let (mut pose, sample) =
                        sample_traced(library, "Walk_Loop", self.gait_time, true);
                    poses::hold_guard(library, &mut pose, input.mode);
                    (pose, sample)
                } else {
                    trace_pose(poses::idle_with_trace(library, self.clock, input.mode))
                }
            }
            PoseState::Slide => {
                let (mut pose, sample) = sample_traced(library, "Sword_Idle", 0.0, false);
                poses::hold_guard(library, &mut pose, input.mode);
                pose[0].rotation = Quat::from_rotation_y(input.slide_yaw) * pose[0].rotation;
                (pose, sample)
            }
            PoseState::Parry => {
                sample_traced(library, "Sword_Block", 0.2 + self.clock.min(0.2), false)
            }
            PoseState::Aim => trace_pose(poses::idle_with_trace(
                library,
                self.clock,
                WeaponMode::Ranged,
            )),
            PoseState::Attack(kind) => trace_pose(poses::gesture_with_trace(
                library,
                kind,
                self.playback.as_ref().map_or(0.0, |p| p.elapsed),
                input.mode,
            )),
            PoseState::Deflected => trace_pose(poses::gesture_with_trace(
                library,
                Gesture::HeavyHit,
                0.18,
                input.mode,
            )),
            PoseState::Down => sample_traced(
                library,
                "Death01",
                self.clock / 0.8 * library.duration("Death01"),
                false,
            ),
        };
        self.trace.state = match state {
            PoseState::Idle => "idle",
            PoseState::Walk => "walk",
            PoseState::Run => "run",
            PoseState::Air(_) => "airborne blend",
            PoseState::Land => "land",
            PoseState::Dodge => "dodge",
            PoseState::Crouch => "crouch",
            PoseState::Slide => "slide",
            PoseState::Parry => "defense",
            PoseState::Aim => "aim",
            PoseState::Attack(_) => "gesture",
            PoseState::Down => "death",
            PoseState::Deflected => "deflected",
        };
        self.trace.cc0_clip = sampled.as_ref().map(|sample| sample.name.into());
        self.trace.cc0_seconds = sampled.map_or(0.0, |sample| sample.seconds);
        self.trace.companion_ik = false;
        if matches!(state, PoseState::Air(_)) {
            poses::hold_guard(library, &mut target, input.mode);
        }
        if input.airborne
            && !matches!(
                state,
                PoseState::Air(_) | PoseState::Dodge | PoseState::Down
            )
        {
            replace_lower_body(
                library,
                &mut target,
                &airborne_pose(library, self.air_time, input.jumps_used),
            );
        } else if matches!(state, PoseState::Parry | PoseState::Aim) && input.speed > 0.01 {
            replace_lower_body(
                library,
                &mut target,
                &library.sample("Walk_Loop", self.gait_time, true),
            );
        }
        if let Some(sniper) = input.sniper
            && !matches!(
                state,
                PoseState::Down
                    | PoseState::Dodge
                    | PoseState::Attack(Gesture::Hit | Gesture::HeavyHit)
            )
        {
            poses::aim_ranged(library, &mut target, sniper.pitch, sniper.recoil);
        }
        let seconds = match state {
            PoseState::Attack(Gesture::Hit | Gesture::HeavyHit) => 0.035,
            PoseState::Attack(_) => 0.055,
            _ => 0.12,
        };
        let weight = poses::smooth(self.transition / seconds);
        self.trace.transition = weight;
        let mut posed: Vec<_> = self
            .from
            .iter()
            .copied()
            .zip(target)
            .map(|(a, b)| blend(a, b, weight))
            .collect();
        if input.airborne && state != PoseState::Down {
            // Physics supplies the world jump height. Authored pelvis motion may
            // bend the body, but must not lift the whole character a second time.
            pin_airborne_pelvis(library, &mut posed);
        }
        // Keep the transition source before planted-foot IK; storing the lowered
        // pose here would apply the pelvis offset again on the next transition.
        self.local = posed.clone();
        self.crouch_weight = (self.crouch_weight
            + if input.crouched || input.sliding {
                dt / 0.12
            } else {
                -dt / 0.12
            })
        .clamp(0.0, 1.0);
        if self.crouch_weight > 0.0 {
            poses::crouch_pose(library, &mut posed, poses::smooth(self.crouch_weight));
        }
        self.model = library.model_pose(&posed);
        self.trace.crouch_weight = self.crouch_weight;
    }
}

#[derive(Clone, Copy)]
struct SniperPose {
    pitch: f32,
    aiming: bool,
    recoil: f32,
}

#[derive(Clone, Copy)]
struct Inputs {
    /// The actor's current attack timeline, if it is attacking.
    attack: Option<AttackClock>,
    alive: bool,
    airborne: bool,
    jumps_used: u8,
    speed: f32,
    sprinting: bool,
    dodging: bool,
    dodge_elapsed: f32,
    dodge_duration: f32,
    dodge_yaw: f32,
    crouched: bool,
    sliding: bool,
    slide_yaw: f32,
    parrying: bool,
    sniper: Option<SniperPose>,
    mode: WeaponMode,
    boss_deflected: Option<(f32, f32)>,
}

struct DisplayFrame {
    input: Inputs,
    pair: WeaponPair,
    blade: bool,
    action: Option<(u64, Option<CombatEffect>)>,
    latest_sequence: u64,
    action_snapshot: Option<ActionSnapshot>,
    defense: Option<DefenseSnapshot>,
}

fn trace_pose(
    (pose, trace): (Vec<Transform>, poses::ClipSample),
) -> (Vec<Transform>, Option<poses::ClipSample>) {
    (pose, Some(trace))
}

fn sample_traced(
    library: &AnimationLibrary,
    name: &'static str,
    seconds: f32,
    looping: bool,
) -> (Vec<Transform>, Option<poses::ClipSample>) {
    (
        library.sample(name, seconds, looping),
        Some(poses::ClipSample {
            name,
            seconds: library.sample_time(name, seconds, looping),
        }),
    )
}

fn airborne_pose(library: &AnimationLibrary, clock: f32, jumps: u8) -> Vec<Transform> {
    if jumps >= 2 {
        return library.sample("NinjaJump_Idle_Loop", clock, true);
    }
    let start = library.sample("Jump_Start", 0.30 + clock.min(0.14) * 3.5, false);
    let air = library.sample("Jump_Loop", (clock - 0.14).max(0.0), true);
    let weight = ((clock - 0.14) / 0.10).clamp(0.0, 1.0);
    start
        .into_iter()
        .zip(air)
        .map(|(a, b)| blend(a, b, weight))
        .collect()
}

fn replace_lower_body(library: &AnimationLibrary, local: &mut [Transform], lower: &[Transform]) {
    let pelvis = library.bone("pelvis");
    local[pelvis] = lower[pelvis];
    let first_leg = library.bone("thigh_l");
    local[first_leg..].copy_from_slice(&lower[first_leg..]);
}

fn pin_airborne_pelvis(library: &AnimationLibrary, local: &mut [Transform]) {
    let pelvis = library.bone("pelvis");
    let parent = library.bones[pelvis].parent.expect("pelvis has a parent");
    let model = library.model_pose(local);
    let standing = library.model_pose(&library.sample("Sword_Idle", 0.0, false));
    let offset = Vec3::Y * (standing[pelvis].translation.y - model[pelvis].translation.y);
    local[pelvis].translation += model[parent]
        .compute_affine()
        .inverse()
        .transform_vector3(offset);
}

type AnimatedActors = (
    &'static Body,
    &'static MotionState,
    &'static Health,
    &'static Facing,
    Option<&'static Combatant>,
    Option<&'static Enemy>,
    Option<&'static Boss>,
    Option<&'static DefenseState>,
);

pub(super) fn animate(
    timing: (
        Res<Time>,
        Res<Time<Fixed>>,
        Res<SimulationActive>,
        Res<HitStop>,
    ),
    library: Res<AnimationLibrary>,
    source_library: Option<Res<SourceAnimationLibrary>>,
    feedback: Res<CombatFeedback>,
    actors: Query<AnimatedActors>,
    mut views: Query<(&ActorView, &mut RigPose)>,
) {
    let (time, fixed, active, hit_stop) = timing;
    if !active.0 {
        return;
    }
    let latest_sequence = feedback.events().last().map_or(0, |event| event.sequence);
    for (view, mut rig) in &mut views {
        let Ok((body, motion, health, facing, combat, enemy, boss, defense)) =
            actors.get(view.owner)
        else {
            continue;
        };
        let set = combat.map_or(WeaponPair::default(), Combatant::weapon_pair);
        let mode = WeaponMode::Melee;
        let action = combat
            .and_then(Combatant::active_action)
            .map(|action| (action.action_id, action.kind.strike_effect()));
        let direction = motion.dodge_direction();
        let dodge_yaw = (-direction.x).atan2(-direction.y)
            - (-facing.direction().x).atan2(-facing.direction().y);
        let slide_direction = motion.slide_direction();
        let slide_yaw = (-slide_direction.x).atan2(-slide_direction.y)
            - (-facing.direction().x).atan2(-facing.direction().y);
        let speed = body.step_displacement().length() / fixed.delta_secs().max(0.0001);
        let input = Inputs {
            attack: combat
                .and_then(Combatant::attack_clock)
                .or_else(|| enemy.and_then(Enemy::attack_clock))
                .or_else(|| boss.and_then(Boss::attack_clock)),
            boss_deflected: boss
                .filter(|boss| boss.state() == BossState::Stunned)
                .map(|boss| (boss.elapsed(), boss.duration())),
            alive: health.is_alive(),
            airborne: !motion.is_grounded(),
            jumps_used: motion.jumps_used(),
            speed,
            sprinting: motion.is_sprinting(),
            dodging: motion.is_dodging(),
            dodge_elapsed: motion.dodge_elapsed(),
            dodge_duration: motion.dodge_elapsed() + motion.dodge_remaining(),
            dodge_yaw,
            crouched: motion.is_crouched(),
            sliding: motion.is_sliding(),
            slide_yaw,
            parrying: defense.is_some_and(DefenseState::is_active),
            sniper: None,
            mode,
        };
        let dt = if hit_stop.is_active() {
            0.0
        } else {
            time.delta_secs()
        };
        rig.frame(
            &library,
            source_library.as_deref(),
            DisplayFrame {
                input,
                pair: set,
                blade: combat.is_none(),
                action,
                latest_sequence,
                action_snapshot: combat.and_then(Combatant::active_action),
                defense: defense.map(DefenseState::snapshot),
            },
            feedback
                .events()
                .filter(|event| event.actor == view.owner)
                .map(|event| (event.sequence, event.action_id, event.kind)),
            dt,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{
        ActionIntent, CollisionWorld, CombatIntent, MoveIntent, SimulationPlugin, SpawnPoint,
    };
    use std::time::Duration;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .insert_resource(CollisionWorld::new(100.0, vec![]))
            .init_resource::<AnimationLibrary>()
            .add_systems(Update, animate);
        app.finish();
        app
    }

    fn actor(app: &mut App) -> Entity {
        let rig = RigPose::new(app.world().resource::<AnimationLibrary>());
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
            .spawn((ActorView { owner: actor }, rig))
            .id();
        app.world_mut().entity_mut(actor).insert(TestView(view));
        actor
    }

    #[derive(Component)]
    struct TestView(Entity);

    fn actor_rig(app: &App, actor: Entity) -> &RigPose {
        let view = app.world().get::<TestView>(actor).unwrap().0;
        app.world().get::<RigPose>(view).unwrap()
    }

    fn frame(app: &mut App, fixed: bool) {
        let dt = Duration::from_secs_f64(1.0 / 60.0);
        if fixed {
            app.world_mut().resource_mut::<Time<Fixed>>().advance_by(dt);
            app.world_mut().run_schedule(FixedUpdate);
        }
        app.world_mut().resource_mut::<Time>().advance_by(dt);
        app.world_mut().run_schedule(Update);
    }

    fn pose_difference(a: &[Transform], b: &[Transform]) -> f32 {
        a.iter()
            .zip(b)
            .map(|(a, b)| a.translation.distance(b.translation))
            .fold(0.0, f32::max)
    }

    fn finite(pose: &[Transform]) {
        for bone in pose {
            assert!(bone.translation.is_finite());
            assert!(bone.rotation.is_finite());
            assert!(bone.scale.is_finite());
            assert!((bone.rotation.length() - 1.0).abs() < 0.001);
        }
    }

    fn idle_input() -> Inputs {
        Inputs {
            attack: None,
            boss_deflected: None,
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
        }
    }

    #[test]
    fn inspection_replay_matches_actual_game_adapter_through_attack_recovery_and_crouch() {
        for pair in [WeaponPair::HammerMusket, WeaponPair::ChainAxeBow] {
            for id in [
                "game:idle",
                "game:attack",
                "game:crouch",
                "game:guard",
                "game:crouch_attack",
                "game:crouch_guard",
                "game:walk_guard",
            ] {
                let mut app = app();
                app.init_resource::<SourceAnimationLibrary>();
                let actor = actor(&mut app);
                app.world_mut()
                    .entity_mut(actor)
                    .insert(Combatant::new(pair));
                if id == "game:walk_guard" {
                    app.world_mut()
                        .entity_mut(actor)
                        .insert(Body::new(Vec2::ZERO, 3.8, 0.35));
                }
                // Resolve the actual floor/support and install simulation-owned
                // companions/defense before starting the controlled input trace.
                frame(&mut app, true);
                if matches!(id, "game:crouch_attack" | "game:crouch_guard") {
                    app.world_mut()
                        .get_mut::<ActionIntent>(actor)
                        .unwrap()
                        .request_crouch_toggle();
                    frame(&mut app, true);
                }
                let view = app.world().get::<TestView>(actor).unwrap().0;
                let initial = RigPose::new(app.world().resource::<AnimationLibrary>());
                *app.world_mut().get_mut::<RigPose>(view).unwrap() = initial;
                if id.ends_with("attack") {
                    app.world_mut()
                        .get_mut::<CombatIntent>(actor)
                        .unwrap()
                        .request_attack();
                }
                if id == "game:crouch" {
                    app.world_mut()
                        .get_mut::<ActionIntent>(actor)
                        .unwrap()
                        .request_crouch_toggle();
                }
                if id.ends_with("guard") {
                    app.world_mut()
                        .entity_mut(actor)
                        .insert(DefenseState::default());
                    app.world_mut()
                        .get_mut::<DefenseState>(actor)
                        .unwrap()
                        .push_edge(crate::simulation::DefenseEdge::Pressed);
                }
                if id == "game:walk_guard" {
                    app.world_mut()
                        .get_mut::<MoveIntent>(actor)
                        .unwrap()
                        .set(Vec2::NEG_Y);
                }
                let mut preview = PreviewRig::new(app.world().resource::<AnimationLibrary>());
                for index in 1..=90 {
                    // Advance authoritative action/defense snapshots together
                    // with the same 60 Hz inspection input trace.
                    frame(&mut app, true);
                    assert!(preview.seek(
                        app.world().resource::<AnimationLibrary>(),
                        app.world().resource::<SourceAnimationLibrary>(),
                        id,
                        pair,
                        false,
                        index as f32 / 60.0,
                        false,
                    ));
                    let live = actor_rig(&app, actor);
                    let inspected = preview.pose();
                    assert_eq!(live.state, inspected.state, "{id} frame {index}");
                    for (a, b) in live.model.iter().zip(&inspected.model) {
                        assert!(
                            a.translation.distance(b.translation) < 0.00001,
                            "{id} frame {index}"
                        );
                        assert!(
                            a.rotation.dot(b.rotation).abs() > 0.99999,
                            "{id} frame {index}"
                        );
                    }
                    match (&live.source, &inspected.source) {
                        (Some(a), Some(b)) => {
                            assert_eq!(a.weight, b.weight);
                            for (a, b) in a.model.iter().zip(&b.model) {
                                assert!(
                                    a.abs_diff_eq(*b, 0.00001),
                                    "{id} frame {index}: mismatched source sample"
                                );
                            }
                        }
                        (None, None) => {}
                        _ => panic!("{id} frame {index}: mismatched source"),
                    }
                    assert_eq!(live.trace.source_clip, inspected.trace.source_clip);
                    assert!(
                        (live.trace.source_seconds - inspected.trace.source_seconds).abs()
                            < 0.00001
                    );
                }
            }
        }
    }

    #[test]
    fn original_named_motion_uses_state_priority_and_keeps_shared_moves_distinct() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::bundled();
        let mut rig = RigPose::new(&library);
        let mut input = idle_input();
        let step = |rig: &mut RigPose, input, dt| {
            rig.advance(&library, input, dt);
            rig.advance_source(&source, input, dt);
        };
        rig.observe(
            &library,
            CombatEffect::Melee,
            WeaponPair::HammerMusket,
            false,
        );
        for _ in 0..6 {
            step(&mut rig, input, 1.0 / 60.0);
        }
        assert_eq!(
            rig.source_clip,
            source
                .clip("skill_ylk_1")
                .map(|clip| (SourceRig::Yanyun, clip))
        );
        assert_eq!(rig.source.as_ref().unwrap().weight, 1.0);
        let held = rig.source.as_ref().unwrap().model.clone();
        step(&mut rig, input, 0.0);
        assert_eq!(held, rig.source.as_ref().unwrap().model);
        input.parrying = true;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source
                .clip("skill_huajie_01__shuangqiangt")
                .map(|clip| (SourceRig::Yanyun, clip))
        );
        input.dodging = true;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source
                .clip("skill_shoujia_dodge_f")
                .map(|clip| (SourceRig::Yanyun, clip))
        );
        input.dodging = false;
        input.parrying = false;
        rig.playback = None;
        input.airborne = true;
        input.jumps_used = 1;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source
                .clip("b_fight_env_jump_01_start_Lfoot__zuiquan")
                .map(|clip| (SourceRig::Yanyun, clip))
        );
        for _ in 0..20 {
            step(&mut rig, input, 1.0 / 60.0);
        }
        assert_eq!(
            rig.source_clip,
            source
                .clip("b_fight_env_jump_02_loop__zuiquan")
                .map(|clip| (SourceRig::Yanyun, clip))
        );
        input.jumps_used = 2;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source
                .clip("b_fight_env_jump_01_start_Lfoot__zuiquan")
                .map(|clip| (SourceRig::Yanyun, clip))
        );
        input.airborne = false;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source
                .clip("b_fight_env_jump_03_to_idle__zuiquan")
                .map(|clip| (SourceRig::Yanyun, clip))
        );
        input.mode = WeaponMode::Ranged;
        for _ in 0..10 {
            step(&mut rig, input, 1.0 / 60.0);
        }
        assert!(rig.source.is_none());
        rig.observe(
            &library,
            CombatEffect::Respawn,
            WeaponPair::HammerMusket,
            false,
        );
        assert!(rig.source.is_none());
    }

    #[test]
    fn boss_reuses_all_three_player_clips_on_fixed_attack_time_and_stops_during_rest() {
        let mut app = app();
        app.init_resource::<SourceAnimationLibrary>();
        let player = actor(&mut app);
        app.world_mut()
            .entity_mut(player)
            .insert(Health::new(10_000.0));
        let boss = actor(&mut app);
        app.world_mut()
            .entity_mut(boss)
            .remove::<(Combatant, CombatIntent)>()
            .insert((
                Body::new(Vec2::new(0.0, -4.0), 5.2, 0.3),
                Boss::default(),
                Enemy::default(),
                Health::new(Boss::PHASE_HEALTH),
            ));
        let mut seen = std::collections::HashSet::new();
        let mut rested = false;
        let mut frozen_checked = false;
        for _ in 0..1100 {
            frame(&mut app, true);
            let snapshot = app.world().get::<Boss>(boss).unwrap();
            if snapshot.state() == BossState::Recovering
                && snapshot.duration() == 3.0
                && snapshot.elapsed() > 1.0
            {
                assert!(
                    actor_rig(&app, boss).source.is_none(),
                    "third strike stretched into the rest"
                );
                rested = true;
            }
            if snapshot.state() != BossState::Attacking || snapshot.elapsed() < 0.04 {
                continue;
            }
            let clock = snapshot.attack_clock().unwrap();
            let row = attack_clips::of(clock.track).unwrap();
            let name = row.clip;
            let source = app
                .world()
                .resource::<SourceAnimationLibrary>()
                .for_rig(SourceRig::Naraka)
                .unwrap();
            let expected = row.position(&clock) * source.duration(source.clip(name).unwrap());
            let trace = actor_rig(&app, boss).trace();
            assert_eq!(trace.source_rig, Some(SourceRig::Naraka.id()));
            assert_eq!(trace.source_clip.as_deref(), Some(name));
            assert!((trace.source_seconds - expected).abs() < 0.00001);
            assert!(
                actor_rig(&app, boss)
                    .source
                    .as_ref()
                    .unwrap()
                    .model
                    .iter()
                    .all(|matrix| matrix.is_finite())
            );
            seen.insert(name);
            if !frozen_checked {
                let time = trace.source_seconds;
                for _ in 0..5 {
                    frame(&mut app, false);
                    assert_eq!(actor_rig(&app, boss).trace().source_seconds, time);
                }
                app.world_mut().resource_mut::<SimulationActive>().0 = false;
                let pose = actor_rig(&app, boss).source.as_ref().unwrap().model.clone();
                for _ in 0..5 {
                    frame(&mut app, true);
                }
                assert_eq!(actor_rig(&app, boss).source.as_ref().unwrap().model, pose);
                app.world_mut().resource_mut::<SimulationActive>().0 = true;
                frozen_checked = true;
            }
            if seen.len() == 3 && rested {
                break;
            }
        }
        assert_eq!(seen.len(), 3);
        assert!(rested && frozen_checked);
    }

    #[test]
    fn actual_boss_q_stun_ends_the_old_source_attack() {
        let mut app = app();
        app.init_resource::<SourceAnimationLibrary>();
        let player = actor(&mut app);
        app.world_mut()
            .entity_mut(player)
            .insert(Health::new(10_000.0));
        let boss = actor(&mut app);
        app.world_mut()
            .entity_mut(boss)
            .remove::<(Combatant, CombatIntent)>()
            .insert((
                Body::new(Vec2::new(0.0, -4.0), 5.2, 0.3),
                Boss::default(),
                Enemy::default(),
                Health::new(Boss::PHASE_HEALTH),
            ));
        for _ in 0..500 {
            if app.world().get::<Boss>(boss).unwrap().state() == BossState::Attacking {
                break;
            }
            frame(&mut app, true);
        }
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().state(),
            BossState::Attacking
        );
        let direction = app.world().get::<Body>(boss).unwrap().world_position()
            - app.world().get::<Body>(player).unwrap().world_position();
        app.world_mut()
            .get_mut::<Facing>(player)
            .unwrap()
            .set(direction.xz());
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        frame(&mut app, true);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_ranged_skill();
        for _ in 0..18 {
            frame(&mut app, true);
        }
        assert_eq!(
            app.world().get::<Boss>(boss).unwrap().state(),
            BossState::Stunned
        );
        // The attack donor is Naraka; stun keeps the existing Yanyun donor.
        // Check preemption before letting the render-only cross-rig blend settle.
        assert_eq!(
            actor_rig(&app, boss).trace().source_clip.as_deref(),
            Some("weakhit_b")
        );
        for _ in 0..8 {
            frame(&mut app, false);
        }
        let rig = actor_rig(&app, boss);
        assert_eq!(rig.state, PoseState::Deflected);
        assert!(rig.playback.is_none());
        assert_eq!(
            rig.source_clip,
            app.world()
                .resource::<SourceAnimationLibrary>()
                .clip("weakhit_b")
                .map(|clip| (SourceRig::Yanyun, clip))
        );
        assert_eq!(rig.source.as_ref().unwrap().weight, 1.0);
        assert!(
            app.world()
                .resource::<CombatFeedback>()
                .events()
                .any(|event| event.actor == boss && event.kind == CombatEffect::Hit)
        );
        let pose = rig.source.as_ref().unwrap().model.clone();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        for _ in 0..20 {
            frame(&mut app, false);
        }
        assert_eq!(pose, actor_rig(&app, boss).source.as_ref().unwrap().model);
    }

    #[test]
    fn sniper_aim_replaces_the_hammer_pose_and_waits_for_the_actual_shot_event() {
        let library = AnimationLibrary::default();
        let mut rig = RigPose::new(&library);
        rig.observe(
            &library,
            CombatEffect::Hammer,
            WeaponPair::HammerMusket,
            false,
        );
        let mut input = idle_input();
        input.mode = WeaponMode::Ranged;
        input.sniper = Some(SniperPose {
            pitch: 30.0_f32.to_radians(),
            aiming: true,
            recoil: 0.0,
        });
        for _ in 0..12 {
            rig.advance(&library, input, 1.0 / 60.0);
        }
        assert_eq!(rig.state, PoseState::Aim);
        assert!(rig.playback.is_none());
        assert!((rig.model[library.bone("hand_r")].rotation * Vec3::Z).y > 0.4);
        rig.observe(
            &library,
            CombatEffect::Shot,
            WeaponPair::HammerMusket,
            false,
        );
        input.sniper = Some(SniperPose {
            pitch: 30.0_f32.to_radians(),
            aiming: false,
            recoil: 0.04,
        });
        rig.advance(&library, input, 1.0 / 60.0);
        assert_eq!(rig.state, PoseState::Attack(Gesture::Shot));
        finite(&rig.model);
    }

    #[test]
    fn second_jump_uses_act_pose_and_restarts_air_time_without_adding_root_height() {
        let mut app = app();
        let actor = actor(&mut app);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        for _ in 0..25 {
            frame(&mut app, true);
        }
        let first = actor_rig(&app, actor).model.clone();
        let height = app.world().get::<MotionState>(actor).unwrap().height();
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        frame(&mut app, true);
        let motion = app.world().get::<MotionState>(actor).unwrap();
        assert_eq!(motion.jumps_used(), 2);
        assert!(motion.height() > height);
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Air(2));
        assert!(rig.air_time < 0.02);
        assert!(rig.clock < 0.02);
        for _ in 0..10 {
            frame(&mut app, true);
        }
        let library = app.world().resource::<AnimationLibrary>();
        let pelvis = library.bone("pelvis");
        let standing = library.model_pose(&library.sample("Sword_Idle", 0.0, false));
        let rig = actor_rig(&app, actor);
        assert!(pose_difference(&first, &rig.model) > 0.05);
        assert!((rig.model[pelvis].translation.y - standing[pelvis].translation.y).abs() < 0.001);
        assert_eq!(
            app.world().get::<Body>(actor).unwrap().position(),
            Vec2::ZERO
        );
        finite(&rig.model);
        let clock = rig.clock;
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        frame(&mut app, true);
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Air(2));
        assert!(rig.clock > clock);
        for _ in 0..90 {
            frame(&mut app, true);
            if app.world().get::<MotionState>(actor).unwrap().is_grounded() {
                break;
            }
        }
        assert_eq!(actor_rig(&app, actor).state, PoseState::Land);
        assert_eq!(
            app.world().get::<MotionState>(actor).unwrap().jumps_used(),
            0
        );
    }

    #[test]
    fn real_fixed_motion_drives_walk_run_jump_land_parry_and_roll_poses() {
        let mut app = app();
        let actor = actor(&mut app);
        let idle = actor_rig(&app, actor).model.clone();
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Walk);
        assert!(pose_difference(&idle, &rig.model) > 0.05);
        let walking = rig.model.clone();
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .set_shift_held(true);
        for _ in 0..18 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Run);
        assert!(pose_difference(&walking, &rig.model) > 0.05);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_jump();
        frame(&mut app, true);
        assert_eq!(actor_rig(&app, actor).state, PoseState::Air(1));
        assert!(app.world().get::<MotionState>(actor).unwrap().height() > 0.0);
        let position = app.world().get::<Body>(actor).unwrap().position();
        let height = app.world().get::<MotionState>(actor).unwrap().height();
        for _ in 0..12 {
            frame(&mut app, false);
        }
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), position);
        assert_eq!(
            app.world().get::<MotionState>(actor).unwrap().height(),
            height
        );
        let library = app.world().resource::<AnimationLibrary>();
        let pelvis = library.bone("pelvis");
        let standing = library.model_pose(&library.sample("Sword_Idle", 0.0, false));
        let rig = actor_rig(&app, actor);
        assert!((rig.model[pelvis].translation.y - standing[pelvis].translation.y).abs() < 0.001);
        finite(&rig.model);
        for _ in 0..80 {
            frame(&mut app, true);
            if app.world().get::<MotionState>(actor).unwrap().is_grounded() {
                break;
            }
        }
        assert_eq!(actor_rig(&app, actor).state, PoseState::Land);
        app.world_mut()
            .get_mut::<DefenseState>(actor)
            .unwrap()
            .push_edge(crate::simulation::DefenseEdge::Pressed);
        frame(&mut app, true);
        assert_eq!(actor_rig(&app, actor).state, PoseState::Parry);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_dodge();
        for _ in 0..8 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Dodge);
        assert!(pose_difference(&idle, &rig.model) > 0.1);
        finite(&rig.model);
    }

    #[test]
    fn blocked_actor_idles_and_zero_fixed_render_frames_keep_a_stable_walk_state() {
        let mut app = app();
        let actor = actor(&mut app);
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        frame(&mut app, true);
        for _ in 0..4 {
            frame(&mut app, false);
            assert_eq!(actor_rig(&app, actor).state, PoseState::Walk);
        }
        app.world_mut().insert_resource(CollisionWorld::new(
            100.0,
            vec![crate::simulation::SolidBox::new(
                Vec2::new(1.0, 0.0),
                Vec2::splat(0.5),
            )],
        ));
        for _ in 0..30 {
            frame(&mut app, true);
        }
        assert_eq!(actor_rig(&app, actor).state, PoseState::Idle);
    }

    #[test]
    fn authored_weapon_gestures_change_bones_and_return_to_guard() {
        let library = AnimationLibrary::default();
        for (kind, mode) in [
            (Gesture::Slash, WeaponMode::Melee),
            (Gesture::Axe, WeaponMode::Melee),
            (Gesture::Hammer, WeaponMode::Melee),
            (Gesture::Shot, WeaponMode::Ranged),
            (Gesture::Bow, WeaponMode::Ranged),
            (Gesture::Windup, WeaponMode::Melee),
            (Gesture::Hit, WeaponMode::Melee),
            (Gesture::HeavyHit, WeaponMode::Melee),
            (Gesture::Guard, WeaponMode::Melee),
        ] {
            let a = library.model_pose(&poses::gesture(&library, kind, 0.01, mode));
            let b =
                library.model_pose(&poses::gesture(&library, kind, kind.duration() * 0.6, mode));
            assert!(
                pose_difference(&a, &b) > 0.001,
                "{kind:?} has no body motion"
            );
            finite(&a);
            finite(&b);
        }
        let mut rig = RigPose::new(&library);
        let initial = rig.model.clone();
        rig.observe(
            &library,
            CombatEffect::Hammer,
            WeaponPair::HammerMusket,
            false,
        );
        for _ in 0..12 {
            rig.advance(&library, idle_input(), 1.0 / 60.0);
        }
        assert!(pose_difference(&initial, &rig.model) > 0.05);
        for _ in 0..60 {
            rig.advance(&library, idle_input(), 1.0 / 60.0);
        }
        assert_eq!(rig.state, PoseState::Idle);
        finite(&rig.model);
    }

    #[test]
    fn actor_tagged_feedback_animates_only_the_performer_even_at_the_same_position() {
        use avian3d::prelude::CollisionLayers;
        let mut app = app();
        let performer = actor(&mut app);
        let bystander = actor(&mut app);
        // Isolate event ownership from capsule depenetration while keeping
        // both actors in the real physics world and colliding with the floor.
        for actor in [performer, bystander] {
            app.world_mut()
                .entity_mut(actor)
                .insert(CollisionLayers::new(0b01, 0b10));
        }
        app.world_mut()
            .get_mut::<CombatIntent>(performer)
            .unwrap()
            .request_attack();
        frame(&mut app, true);
        assert_eq!(
            actor_rig(&app, performer).state,
            PoseState::Attack(Gesture::Hammer)
        );
        assert_eq!(actor_rig(&app, bystander).state, PoseState::Idle);
        let clock = actor_rig(&app, performer).clock;
        frame(&mut app, false);
        assert!(actor_rig(&app, performer).clock > clock);
    }

    #[test]
    fn actual_sniper_hit_freezes_both_rigs_then_plays_the_recipients_heavy_flinch() {
        use crate::gameplay::{self, Player};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .init_resource::<AnimationLibrary>()
            .add_systems(Update, animate);
        app.finish();
        gameplay::setup(app.world_mut());
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        frame(&mut app, true);
        let target = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .target()
            .unwrap();
        for owner in [player, target] {
            let rig = RigPose::new(app.world().resource::<AnimationLibrary>());
            let view = app.world_mut().spawn((ActorView { owner }, rig)).id();
            app.world_mut().entity_mut(owner).insert(TestView(view));
        }
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        for _ in 0..14 {
            frame(&mut app, true);
        }
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_combo();
        for _ in 0..40 {
            frame(&mut app, true);
            if app.world().resource::<HitStop>().is_active() {
                break;
            }
        }
        assert!(app.world().resource::<HitStop>().is_active());
        assert_eq!(
            actor_rig(&app, target).state,
            PoseState::Attack(Gesture::HeavyHit)
        );
        assert!(!matches!(
            actor_rig(&app, player).state,
            PoseState::Aim | PoseState::Attack(Gesture::Shot)
        ));
        let held: Vec<_> = [player, target]
            .map(|owner| {
                let rig = actor_rig(&app, owner);
                (
                    rig.clock,
                    rig.model.clone(),
                    app.world().get::<Body>(owner).unwrap().world_position(),
                )
            })
            .into();
        for _ in 0..3 {
            frame(&mut app, true);
            for (owner, (clock, pose, position)) in [player, target].into_iter().zip(&held) {
                let rig = actor_rig(&app, owner);
                assert_eq!(rig.clock, *clock);
                assert_eq!(&rig.model, pose);
                assert_eq!(
                    app.world().get::<Body>(owner).unwrap().world_position(),
                    *position
                );
            }
        }
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, target);
        assert_eq!(rig.state, PoseState::Attack(Gesture::HeavyHit));
        assert!(rig.clock > 0.1);
        assert!(pose_difference(&held[1].1, &rig.model) > 0.001);
        finite(&rig.model);
    }

    #[test]
    fn pause_freezes_pose_clocks_and_feedback_cursor_then_resumes_without_physics_writes() {
        let mut app = app();
        let actor = actor(&mut app);
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        frame(&mut app, true);
        let position = app.world().get::<Body>(actor).unwrap().position();
        let height = app.world().get::<MotionState>(actor).unwrap().height();
        let rig = actor_rig(&app, actor);
        let before = rig.model.clone();
        let clock = rig.clock;
        let sequence = rig.event_sequence;
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        for _ in 0..30 {
            frame(&mut app, false);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(pose_difference(&before, &rig.model), 0.0);
        assert_eq!(rig.clock, clock);
        assert_eq!(rig.event_sequence, sequence);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        for _ in 0..12 {
            frame(&mut app, false);
        }
        assert!(pose_difference(&before, &actor_rig(&app, actor).model) > 0.01);
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), position);
        assert_eq!(
            app.world().get::<MotionState>(actor).unwrap().height(),
            height
        );
    }

    #[test]
    fn death_lowers_the_authored_head_and_respawn_restores_the_rig() {
        let library = AnimationLibrary::default();
        let mut rig = RigPose::new(&library);
        let head = library.bone("Head");
        let standing = rig.model[head].translation.y;
        let mut down = idle_input();
        down.alive = false;
        for _ in 0..60 {
            rig.advance(&library, down, 1.0 / 60.0);
        }
        assert_eq!(rig.state, PoseState::Down);
        assert!(rig.model[head].translation.y < standing - 0.5);
        finite(&rig.model);
        rig.observe(
            &library,
            CombatEffect::Respawn,
            WeaponPair::HammerMusket,
            false,
        );
        rig.advance(&library, idle_input(), 1.0 / 60.0);
        assert_eq!(rig.state, PoseState::Idle);
        assert!((rig.model[head].translation.y - standing).abs() < 0.05);
    }

    #[test]
    fn companion_gun_and_bow_have_independent_hands_and_fixed_shot_clocks() {
        let library = AnimationLibrary::default();
        let mut rig = RigPose::new(&library);
        let mut state = crate::simulation::CompanionState {
            elapsed: 0.31,
            shot_count: 1,
            kind: ActionKind::HammerCombo,
            ..default()
        };
        let muzzle = Vec3::new(0., 1.2, -0.55);
        rig.sample_companion(
            &library,
            &state,
            0.4,
            [muzzle + Vec3::Z * 0.76, muzzle + Vec3::Z * 0.41],
        );
        let gun = rig.model.clone();
        let foot = library.bone("foot_l");
        let hand = library.bone("hand_r");
        state.pair = WeaponPair::ChainAxeBow;
        state.kind = ActionKind::ChainCombo;
        rig.sample_companion(
            &library,
            &state,
            0.4,
            [muzzle + Vec3::Z * 0.35, muzzle + Vec3::Z * 0.08],
        );
        assert!(rig.model[hand].translation.distance(gun[hand].translation) > 0.15);
        assert_eq!(rig.model[foot], gun[foot]);
        finite(&rig.model);
        let paused = rig.model.clone();
        rig.sample_companion(
            &library,
            &state,
            0.4,
            [muzzle + Vec3::Z * 0.35, muzzle + Vec3::Z * 0.08],
        );
        assert_eq!(rig.model, paused);
    }

    #[test]
    fn a_partial_movement_step_slows_gait_without_rewinding_an_existing_walk_cycle() {
        let library = AnimationLibrary::default();
        let mut rig = RigPose::new(&library);
        let mut input = idle_input();
        input.speed = 6.0;
        for _ in 0..120 {
            rig.advance(&library, input, 1.0 / 60.0);
        }
        let before = rig.model.clone();
        let gait = rig.gait_time;
        input.speed = 0.3;
        rig.advance(&library, input, 1.0 / 60.0);
        assert_eq!(rig.state, PoseState::Walk);
        assert!(rig.gait_time > gait);
        assert!(pose_difference(&before, &rig.model) < 0.05);
        finite(&rig.model);
    }

    #[test]
    fn fixed_crouch_keeps_its_foot_anchor_then_walks_and_smoothly_stands_without_foot_drift() {
        let mut app = app();
        let actor = actor(&mut app);
        let head = app.world().resource::<AnimationLibrary>().bone("Head");
        let standing_head = actor_rig(&app, actor).model[head].translation.y;
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_crouch_toggle();
        frame(&mut app, true);
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Crouch);
        assert!(rig.crouch_weight > 0.0 && rig.crouch_weight < 1.0);
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.crouch_weight, 1.0);
        assert!(rig.model[head].translation.y < standing_head - 0.2);
        let library = app.world().resource::<AnimationLibrary>();
        let base = library.model_pose(&rig.local);
        for side in ["l", "r"] {
            let foot = library.bone(&format!("foot_{side}"));
            assert!(rig.model[foot].translation.distance(base[foot].translation) < 0.002);
        }
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .height()
                .abs()
                < 0.002
        );
        assert!(app.world().get::<Body>(actor).unwrap().position().length() < 0.001);
        let crouched = rig.model.clone();
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        for _ in 0..30 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Crouch);
        assert!(pose_difference(&crouched, &rig.model) > 0.01);
        finite(&rig.model);
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_crouch_toggle();
        frame(&mut app, true);
        let rig = actor_rig(&app, actor);
        assert!(rig.crouch_weight > 0.0 && rig.crouch_weight < 1.0);
        let weight = rig.crouch_weight;
        let paused = rig.model.clone();
        let position = app.world().get::<Body>(actor).unwrap().position();
        app.world_mut().resource_mut::<SimulationActive>().0 = false;
        for _ in 0..30 {
            frame(&mut app, false);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.crouch_weight, weight);
        assert_eq!(pose_difference(&paused, &rig.model), 0.0);
        assert_eq!(app.world().get::<Body>(actor).unwrap().position(), position);
        app.world_mut().resource_mut::<SimulationActive>().0 = true;
        app.world_mut().get_mut::<MoveIntent>(actor).unwrap().stop();
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.crouch_weight, 0.0);
        assert_eq!(rig.state, PoseState::Idle);
        assert!(rig.model[head].translation.y > standing_head - 0.1);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .height()
                .abs()
                < 0.002
        );
    }

    #[test]
    fn real_sprint_to_slide_uses_a_crouched_pose_then_keeps_crouching_after_the_slide() {
        let mut app = app();
        let actor = actor(&mut app);
        app.world_mut()
            .get_mut::<MoveIntent>(actor)
            .unwrap()
            .set(Vec2::X);
        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(actor).unwrap();
            intent.set_movement_held(true);
            intent.set_shift_held(true);
            intent.request_dodge();
        }
        for _ in 0..18 {
            frame(&mut app, true);
        }
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .is_sprinting()
        );
        app.world_mut()
            .get_mut::<ActionIntent>(actor)
            .unwrap()
            .request_crouch_toggle();
        frame(&mut app, true);
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Slide);
        assert!(rig.crouch_weight > 0.0);
        for _ in 0..12 {
            frame(&mut app, true);
        }
        let rig = actor_rig(&app, actor);
        assert_eq!(rig.state, PoseState::Slide);
        assert_eq!(rig.crouch_weight, 1.0);
        finite(&rig.model);
        for _ in 0..30 {
            frame(&mut app, true);
        }
        assert!(app.world().get::<MotionState>(actor).unwrap().is_crouched());
        assert!(!app.world().get::<MotionState>(actor).unwrap().is_sliding());
        assert_eq!(actor_rig(&app, actor).state, PoseState::Crouch);
        assert!(
            app.world()
                .get::<MotionState>(actor)
                .unwrap()
                .height()
                .abs()
                < 0.002
        );
    }
    #[test]
    fn actual_committed_stages_follow_fixed_snapshot_time_through_zero_fixed_render_frames() {
        let mut app = app();
        app.init_resource::<SourceAnimationLibrary>();
        let actor = actor(&mut app);
        frame(&mut app, true);
        let view = app.world().get::<TestView>(actor).unwrap().0;
        let initial = RigPose::new(app.world().resource::<AnimationLibrary>());
        *app.world_mut().get_mut::<RigPose>(view).unwrap() = initial;
        app.world_mut()
            .get_mut::<CombatIntent>(actor)
            .unwrap()
            .request_attack();
        frame(&mut app, true);
        let first = actor_rig(&app, actor)
            .source
            .as_ref()
            .unwrap()
            .model
            .clone();
        let first_time = actor_rig(&app, actor).trace.source_seconds;
        assert_eq!(
            actor_rig(&app, actor).trace.source_rig,
            Some(SourceRig::Naraka.id())
        );
        for _ in 0..8 {
            frame(&mut app, false);
            assert_eq!(actor_rig(&app, actor).trace.source_seconds, first_time);
            assert_eq!(actor_rig(&app, actor).source.as_ref().unwrap().model, first);
        }
        let mut stages = std::collections::HashSet::new();
        for _ in 0..120 {
            if !stages.contains(&3) {
                app.world_mut()
                    .get_mut::<CombatIntent>(actor)
                    .unwrap()
                    .request_attack();
            }
            frame(&mut app, true);
            if let Some(snapshot) = app.world().get::<Combatant>(actor).unwrap().active_action() {
                let (number, name) = match snapshot.melee_stage.unwrap() {
                    MeleeStage::One => (1, "male_hengdao_attack_light_01"),
                    MeleeStage::Two => (2, "male_hengdao_attack_light_02"),
                    MeleeStage::Three => (3, "male_hengdao_attack_light_03"),
                };
                stages.insert(number);
                let library = app
                    .world()
                    .resource::<SourceAnimationLibrary>()
                    .for_rig(SourceRig::Naraka)
                    .unwrap();
                let seconds = snapshot.elapsed / snapshot.duration
                    * library.duration(library.clip(name).unwrap());
                let trace = &actor_rig(&app, actor).trace;
                assert_eq!(trace.source_clip.as_deref(), Some(name));
                assert!(
                    (trace.source_seconds - seconds).abs() < 0.00001,
                    "stage{number}: display restarted on damage cue"
                );
            }
        }
        assert_eq!(stages.len(), 3);
    }
    #[test]
    fn attack_poses_follow_the_simulation_clock_not_the_render_rate() {
        let library = AnimationLibrary::default();
        let sources = SourceAnimationLibrary::bundled();
        let skill = ActionSnapshot {
            action_id: 7,
            kind: ActionKind::HammerSlam,
            pair: WeaponPair::HammerMusket,
            melee_stage: None,
            target: None,
            elapsed: 0.3,
            duration: 0.5,
            shot_count: 0,
            aim_position: Vec3::ZERO,
            source_position: Vec3::ZERO,
            companion_position: Vec3::ZERO,
        };
        let enemy = AttackClock {
            action_id: 3,
            track: crate::simulation::TrackId::Enemy,
            elapsed: 0.3,
            duration: crate::simulation::TrackId::Enemy.duration(),
        };
        for (effect, blade, clock, action) in [
            (CombatEffect::Hammer, false, skill.clock(), Some(skill)),
            (CombatEffect::EnemyWindup, true, enemy, None),
        ] {
            let row = attack_clips::of(clock.track).unwrap();
            let rig_library = sources.for_rig(row.rig).unwrap();
            let expected = rig_library.sample_time(
                rig_library.clip(row.clip).unwrap(),
                row.position(&clock),
                false,
            );
            // The same simulation moment, displayed after different amounts of
            // render time: the old render-clock pose differed between them.
            for (dt, frames) in [(1.0 / 30.0, 7), (1.0 / 144.0, 12)] {
                let mut rig = RigPose::new(&library);
                rig.observe(&library, effect, WeaponPair::HammerMusket, blade);
                let mut input = idle_input();
                input.attack = Some(clock);
                for _ in 0..frames {
                    rig.advance(&library, input, dt);
                    rig.advance_sources(&sources, input, action, None, !blade, dt);
                }
                assert_eq!(rig.trace.source_clip.as_deref(), Some(row.clip));
                assert!(
                    (rig.trace.source_seconds - expected).abs() < 1e-5,
                    "{effect:?} at render dt {dt}: {} != {expected}",
                    rig.trace.source_seconds
                );
            }
        }
    }

    #[test]
    fn different_source_hierarchies_fade_without_reusing_local_pose_indices() {
        let library = AnimationLibrary::default();
        let sources = SourceAnimationLibrary::bundled();
        let mut rig = RigPose::new(&library);
        let input = idle_input();
        rig.advance_sources(&sources, input, None, None, true, 0.2);
        assert_eq!(rig.source.as_ref().unwrap().rig, SourceRig::Naraka);
        rig.observe(
            &library,
            CombatEffect::Hammer,
            WeaponPair::HammerMusket,
            false,
        );
        rig.advance(&library, input, 0.02);
        let skill = ActionSnapshot {
            action_id: 7,
            kind: ActionKind::HammerSlam,
            pair: WeaponPair::HammerMusket,
            melee_stage: None,
            target: None,
            elapsed: 0.1,
            duration: 0.5,
            shot_count: 0,
            aim_position: Vec3::ZERO,
            source_position: Vec3::ZERO,
            companion_position: Vec3::ZERO,
        };
        for (action, expected) in [(Some(skill), SourceRig::Yanyun), (None, SourceRig::Naraka)] {
            for _ in 0..16 {
                rig.advance_sources(&sources, input, action, None, true, 0.02);
                if let Some(source) = &rig.source {
                    let count = sources.for_rig(source.rig).unwrap().rest_model.len();
                    assert_eq!(source.model.len(), count);
                    assert_eq!(rig.source_local.as_ref().unwrap().len(), count);
                    if let Some(from) = &rig.source_from {
                        assert_eq!(from.len(), count);
                    }
                    assert!(source.model.iter().all(|matrix| matrix.is_finite()));
                }
            }
            assert_eq!(rig.source.as_ref().unwrap().rig, expected);
            assert_eq!(rig.source.as_ref().unwrap().weight, 1.0);
        }
    }
    #[test]
    fn late_old_melee_feedback_cannot_restart_a_new_committed_stage() {
        let library = AnimationLibrary::default();
        let sources = SourceAnimationLibrary::bundled();
        let mut rig = RigPose::new(&library);
        let snapshot = ActionSnapshot {
            action_id: 9,
            kind: ActionKind::Melee,
            pair: WeaponPair::HammerMusket,
            melee_stage: Some(MeleeStage::Two),
            target: None,
            elapsed: 0.30,
            duration: 0.60,
            shot_count: 0,
            aim_position: Vec3::ZERO,
            source_position: Vec3::ZERO,
            companion_position: Vec3::ZERO,
        };
        let frame = |sequence| DisplayFrame {
            input: idle_input(),
            pair: snapshot.pair,
            blade: false,
            action: Some((snapshot.action_id, Some(CombatEffect::Melee))),
            latest_sequence: sequence,
            action_snapshot: Some(snapshot),
            defense: None,
        };
        rig.frame(
            &library,
            Some(&sources),
            frame(1),
            [(1, 8, CombatEffect::Melee)],
            0.10,
        );
        let clock = rig.clock;
        let model = rig.source.as_ref().unwrap().model.clone();
        rig.frame(
            &library,
            Some(&sources),
            frame(2),
            [(2, 8, CombatEffect::Melee)],
            0.10,
        );
        assert!(
            rig.clock > clock,
            "historical hit marker restarted current body gesture"
        );
        assert_eq!(rig.source.as_ref().unwrap().model, model);
        assert_eq!(rig.trace.melee_stage, Some(MeleeStage::Two));
    }
    #[test]
    fn display_frames_latch_a_preempted_riposte_and_only_a_new_gesture_can_restart_it() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::bundled();
        for hit in [false, true] {
            let mut rig = RigPose::new(&library);
            let frame = |input, age, id, sequence| DisplayFrame {
                input,
                pair: WeaponPair::HammerMusket,
                blade: false,
                action: None,
                latest_sequence: sequence,
                action_snapshot: None,
                defense: Some(DefenseSnapshot {
                    kind: if age < 0.18 {
                        DefenseKind::Parry
                    } else {
                        DefenseKind::Idle
                    },
                    gesture_id: id,
                    last_gesture_kind: DefenseKind::Parry,
                    gesture_elapsed: age,
                    ..default()
                }),
            };
            rig.frame(
                &library,
                Some(&source),
                frame(idle_input(), 0.1, 7, 0),
                [],
                0.1,
            );
            assert_eq!(
                rig.trace.source_clip.as_deref(),
                Some("male_hengdao_attack_shock_light_01")
            );
            let mut interrupt = idle_input();
            if !hit {
                interrupt.speed = 2.0;
            }
            let events = hit.then_some((1, 0, CombatEffect::Hit));
            rig.frame(
                &library,
                Some(&source),
                frame(interrupt, 0.25, 7, u64::from(hit)),
                events,
                0.1,
            );
            assert_eq!(rig.interrupted_defense, Some(7));
            rig.frame(
                &library,
                Some(&source),
                frame(idle_input(), 0.57, 7, u64::from(hit)),
                [],
                0.32,
            );
            assert_eq!(
                rig.trace.source_clip.as_deref(),
                Some("male_hengdao_idle_01"),
                "ended short walk/hit revived the old gesture"
            );
            assert!(rig.trace.defense_interrupted);
            rig.frame(
                &library,
                Some(&source),
                frame(idle_input(), 0.1, 8, u64::from(hit)),
                [],
                0.1,
            );
            assert_eq!(
                rig.trace.source_clip.as_deref(),
                Some("male_hengdao_attack_shock_light_01")
            );
            assert!(!rig.trace.defense_interrupted);
        }
    }
    #[test]
    fn new_light_or_heavy_hit_latches_even_when_its_playback_fits_inside_one_render_frame() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::bundled();
        for effect in [CombatEffect::Hit, CombatEffect::SniperImpact] {
            let mut rig = RigPose::new(&library);
            let frame = |age, sequence| DisplayFrame {
                input: idle_input(),
                pair: WeaponPair::HammerMusket,
                blade: false,
                action: None,
                latest_sequence: sequence,
                action_snapshot: None,
                defense: Some(DefenseSnapshot {
                    kind: if age < 0.18 {
                        DefenseKind::Parry
                    } else {
                        DefenseKind::Idle
                    },
                    gesture_id: 12,
                    last_gesture_kind: DefenseKind::Parry,
                    gesture_elapsed: age,
                    ..default()
                }),
            };
            rig.frame(&library, Some(&source), frame(0.1, 0), [], 0.1);
            rig.frame(
                &library,
                Some(&source),
                frame(0.4, 1),
                [(1, 0, effect)],
                0.32,
            );
            assert!(
                rig.playback.is_none(),
                "hit must really have expired inside this frame"
            );
            assert_eq!(rig.interrupted_defense, Some(12));
            assert_eq!(
                rig.trace.source_clip.as_deref(),
                Some("male_hengdao_idle_01"),
                "expired hit resurrected old riposte"
            );
        }
    }
}
