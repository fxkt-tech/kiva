//! Explicit Kiva display adaptations of verified native source clips.
//! Timings here never execute hits, movement or successful defenses.
use super::*;

// Verified female_fk overrides share these male-named original FK clips.
// Target lower body and physical grips remain the established Kiva choreography.
pub(super) const GUN_HOLD: &str = "male_fk_hold_idle_01";
pub(super) const GUN_ATTACK: &str = "male_fk_attack_01";

pub(super) struct Selection {
    pub rig: SourceRig,
    pub name: &'static str,
    pub seconds: f32,
    pub looping: bool,
    pub adaptation: &'static str,
}
impl Selection {
    fn native(name: &'static str, seconds: f32, looping: bool) -> Self {
        Self {
            rig: SourceRig::Naraka,
            name,
            seconds,
            looping,
            adaptation: "native Humanoid bake; Kiva retarget",
        }
    }
}

impl RigPose {
    /// Simulation keeps the short effective window. Presentation separately
    /// remembers a preempted gesture so its tail cannot revive when a brief
    /// walk/hit/new action finishes before the original 0.60 seconds.
    pub(super) fn latch_defense_interruption(
        &mut self,
        input: Inputs,
        action: Option<ActionSnapshot>,
        defense: Option<DefenseSnapshot>,
    ) {
        let Some(defense) = defense else {
            return;
        };
        if !matches!(
            defense.last_gesture_kind,
            DefenseKind::Parry | DefenseKind::Deflect
        ) {
            return;
        }
        let hard_preemption = !input.alive
            || input.airborne
            || input.dodging
            || input.sliding
            || action.is_some()
            || matches!(
                self.state,
                PoseState::Attack(Gesture::Hit | Gesture::HeavyHit)
            );
        let movement_preemption = input.speed > 0.01 && defense.kind == DefenseKind::Idle;
        if hard_preemption || movement_preemption {
            self.interrupted_defense = Some(defense.gesture_id);
        }
    }

    pub(super) fn naraka_selection(
        &self,
        input: Inputs,
        action: Option<ActionSnapshot>,
        defense: Option<DefenseSnapshot>,
    ) -> Option<Selection> {
        let clip = |name, seconds, looping| Some(Selection::native(name, seconds, looping));
        if !input.alive {
            return clip("male_die_hurt_stand_light_front_01", self.clock, false);
        }
        if matches!(
            self.state,
            PoseState::Attack(Gesture::Hit | Gesture::HeavyHit)
        ) {
            return clip(
                "male_hurt_stand_light_front_01",
                self.clock / 0.30 * 1.3333334,
                false,
            );
        }
        if input.dodging {
            let quarter = (input.dodge_yaw / std::f32::consts::FRAC_PI_2).round() as i32;
            let name = match quarter.rem_euclid(4) {
                1 => "male_hengdao_dodge_left_01",
                2 => "male_hengdao_dodge_back_01",
                3 => "male_hengdao_dodge_right_01",
                _ => "male_hengdao_dodge_front_01",
            };
            // Final duration is resolved from the bundled clip before sampling.
            return clip(
                name,
                input.dodge_elapsed / input.dodge_duration.max(0.001),
                false,
            );
        }
        if input.sliding {
            return None;
        }
        if let Some(action) = action {
            if action.kind == ActionKind::Melee {
                let name = match action.melee_stage.unwrap_or(MeleeStage::One) {
                    MeleeStage::One => "male_hengdao_attack_light_01",
                    MeleeStage::Two => "male_hengdao_attack_light_02",
                    MeleeStage::Three => "male_hengdao_attack_light_03",
                };
                return clip(
                    name,
                    (action.elapsed / action.duration.max(0.001)).clamp(0.0, 1.0),
                    false,
                );
            }
            // E / Tab preserve the established single/paired choreography.
            return None;
        }
        if !input.airborne
            && let Some(defense) = defense
        {
            if defense.last_gesture_kind == DefenseKind::Parry
                && self.interrupted_defense != Some(defense.gesture_id)
                && defense.gesture_elapsed < 0.60
                && (defense.kind == DefenseKind::Parry || input.speed <= 0.01)
            {
                let mut selection = Selection::native(
                    "male_hengdao_attack_shock_light_01",
                    defense.gesture_elapsed / 0.60,
                    false,
                );
                selection.adaptation =
                    "Kiva 0.60s visual riposte; source success-only clip unproven";
                return Some(selection);
            }
            if matches!(
                defense.kind,
                DefenseKind::Deflect | DefenseKind::Waiting | DefenseKind::Guard
            ) {
                // A bounded entry and a verified source hold frame. This is a
                // Kiva display hold, not a recovered original controller state.
                let time = (defense.gesture_elapsed / 0.20).min(1.0) * 0.40;
                let mut selection = Selection::native("male_hengdao_shock_01", time, false);
                selection.adaptation = "Kiva defensive entry/hold";
                return Some(selection);
            }
            if defense.last_gesture_kind == DefenseKind::Deflect
                && self.interrupted_defense != Some(defense.gesture_id)
                && defense.gesture_elapsed < 0.32
                && input.speed <= 0.01
            {
                return clip(
                    "male_hengdao_shock_01",
                    defense.gesture_elapsed / 0.32 * 0.64,
                    false,
                );
            }
        }
        if input.airborne {
            if self.air_time < 0.18 {
                return clip(
                    if input.jumps_used >= 2 {
                        "male_hengdao_jump_inplace_up_02"
                    } else {
                        "male_hengdao_jump_inplace_up_01"
                    },
                    self.air_time / 0.18,
                    false,
                );
            }
            return clip("male_hengdao_jump_down_loop_01", self.air_time - 0.18, true);
        }
        if self.state == PoseState::Land {
            return clip(
                "male_hengdao_jump_down_to_land_01",
                self.clock / 0.16,
                false,
            );
        }
        if input.crouched {
            return clip(
                if input.speed > 0.01 {
                    "male_hengdao_crouch_promenade_01"
                } else {
                    "male_hengdao_crouch_idle_01"
                },
                self.clock
                    * if input.speed > 0.01 {
                        input.speed / 3.8
                    } else {
                        1.0
                    },
                true,
            );
        }
        if input.speed > 0.01 {
            return clip(
                if input.sprinting {
                    "male_hengdao_run_01"
                } else {
                    "male_hengdao_promenade_01"
                },
                self.clock * input.speed / if input.sprinting { 6.0 } else { 3.8 },
                true,
            );
        }
        clip("male_hengdao_idle_01", self.clock, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> Inputs {
        Inputs {
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
        }
    }
    fn action(stage: MeleeStage) -> ActionSnapshot {
        ActionSnapshot {
            action_id: 42,
            kind: ActionKind::Melee,
            pair: WeaponPair::HammerMusket,
            melee_stage: Some(stage),
            target: None,
            elapsed: 0.3,
            duration: 0.6,
            shot_count: 0,
            aim_position: Vec3::ZERO,
            source_position: Vec3::ZERO,
            companion_position: Vec3::ZERO,
        }
    }
    #[test]
    fn each_committed_stage_uses_its_snapshot_clock_instead_of_a_damage_cue() {
        let rig = RigPose::new(&AnimationLibrary::default());
        for (stage, name) in [
            (MeleeStage::One, "male_hengdao_attack_light_01"),
            (MeleeStage::Two, "male_hengdao_attack_light_02"),
            (MeleeStage::Three, "male_hengdao_attack_light_03"),
        ] {
            let selected = rig
                .naraka_selection(input(), Some(action(stage)), None)
                .unwrap();
            assert_eq!(selected.name, name);
            assert_eq!(selected.seconds, 0.5);
        }
    }
    #[test]
    fn missed_parry_window_keeps_only_the_bounded_visual_tail_and_new_inputs_preempt_it() {
        let mut rig = RigPose::new(&AnimationLibrary::default());
        let snapshot = DefenseSnapshot {
            kind: DefenseKind::Idle,
            elapsed: 0.20,
            gesture_id: 9,
            last_gesture_kind: DefenseKind::Parry,
            gesture_elapsed: 0.38,
            ..default()
        };
        let selection = rig.naraka_selection(input(), None, Some(snapshot)).unwrap();
        assert_eq!(selection.name, "male_hengdao_attack_shock_light_01");
        assert!((selection.seconds - 0.38 / 0.60).abs() < 1e-7);
        let expired = DefenseSnapshot {
            gesture_elapsed: 0.61,
            ..snapshot
        };
        assert_eq!(
            rig.naraka_selection(input(), None, Some(expired))
                .unwrap()
                .name,
            "male_hengdao_idle_01"
        );
        let mut moving = input();
        moving.speed = 2.0;
        assert_eq!(
            rig.naraka_selection(moving, None, Some(snapshot))
                .unwrap()
                .name,
            "male_hengdao_promenade_01"
        );
        assert_eq!(
            rig.naraka_selection(input(), Some(action(MeleeStage::Three)), Some(snapshot))
                .unwrap()
                .name,
            "male_hengdao_attack_light_03"
        );
        rig.state = PoseState::Attack(Gesture::Hit);
        assert_eq!(
            rig.naraka_selection(input(), None, Some(snapshot))
                .unwrap()
                .name,
            "male_hurt_stand_light_front_01"
        );
        let mut dead = input();
        dead.alive = false;
        assert_eq!(
            rig.naraka_selection(dead, None, Some(snapshot))
                .unwrap()
                .name,
            "male_die_hurt_stand_light_front_01"
        );
    }
    #[test]
    fn four_accepted_dodge_directions_select_four_native_clips_without_root_yaw_rotation() {
        let rig = RigPose::new(&AnimationLibrary::default());
        for (yaw, suffix) in [
            (0.0, "front"),
            (std::f32::consts::FRAC_PI_2, "left"),
            (std::f32::consts::PI, "back"),
            (-std::f32::consts::FRAC_PI_2, "right"),
        ] {
            let mut input = input();
            input.dodging = true;
            input.dodge_yaw = yaw;
            input.dodge_elapsed = 0.21;
            let selected = rig.naraka_selection(input, None, None).unwrap();
            assert_eq!(selected.name, format!("male_hengdao_dodge_{suffix}_01"));
            assert_eq!(selected.seconds, 0.5);
        }
    }
    #[test]
    fn actual_moving_deflect_and_guard_keep_shock_while_movement_preempts_only_the_riposte_tail() {
        let rig = RigPose::new(&AnimationLibrary::default());
        let mut moving = input();
        moving.speed = 3.8;
        for kind in [
            DefenseKind::Deflect,
            DefenseKind::Waiting,
            DefenseKind::Guard,
        ] {
            let defense = DefenseSnapshot {
                kind,
                last_gesture_kind: DefenseKind::Deflect,
                gesture_elapsed: 0.25,
                ..default()
            };
            assert_eq!(
                rig.naraka_selection(moving, None, Some(defense))
                    .unwrap()
                    .name,
                "male_hengdao_shock_01"
            );
        }
        let active = DefenseSnapshot {
            kind: DefenseKind::Parry,
            last_gesture_kind: DefenseKind::Parry,
            gesture_elapsed: 0.10,
            ..default()
        };
        assert_eq!(
            rig.naraka_selection(moving, None, Some(active))
                .unwrap()
                .name,
            "male_hengdao_attack_shock_light_01"
        );
        let tail = DefenseSnapshot {
            kind: DefenseKind::Idle,
            gesture_elapsed: 0.30,
            ..active
        };
        assert_eq!(
            rig.naraka_selection(moving, None, Some(tail)).unwrap().name,
            "male_hengdao_promenade_01"
        );
    }
}
