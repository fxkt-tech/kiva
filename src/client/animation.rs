//! Authored skeletal animation driven by read-only simulation snapshots and actor-tagged events.
mod poses;

use bevy::prelude::*;

use super::{
    ActorView,
    skeleton::{AnimationLibrary, blend},
    source_animation::SourceAnimationLibrary,
};
use crate::simulation::{
    ActionKind, Body, Boss, BossState, CombatEffect, CombatFeedback, Combatant, DefenseState,
    Enemy, Facing, Health, HitStop, MotionState, SimulationActive, WeaponMode, WeaponPair,
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

struct Playback {
    kind: Gesture,
    elapsed: f32,
}

pub(super) struct SourcePose {
    pub(super) model: Vec<Mat4>,
    pub(super) weight: f32,
}

#[derive(Component)]
pub(super) struct RigPose {
    pub(super) source: Option<SourcePose>,
    /// Render-only grip targets, retargeted with each companion body's own lengths.
    pub(super) companion_hands: Option<[Vec3; 2]>,
    source_local: Option<Vec<Transform>>,
    source_from: Option<Vec<Transform>>,
    source_clip: Option<usize>,
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
}

impl RigPose {
    pub fn new(library: &AnimationLibrary) -> Self {
        let local = poses::idle(library, 0.0, WeaponMode::Melee);
        Self {
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
        let kind = state.kind;
        let mut local = poses::idle(library, elapsed, WeaponMode::Ranged);
        let markers: &[f32] = match kind {
            ActionKind::HammerCombo => &[0.30, 0.42, 0.54],
            ActionKind::ChainCombo => &[0.12, 0.28, 0.44],
            ActionKind::AxeShot => &[0.15],
            _ => &[0.18],
        };
        let last = shots
            .checked_sub(1)
            .and_then(|index| markers.get(index as usize))
            .copied();
        let recoil = last.map_or(0.0, |time| {
            (1.0 - (elapsed - time) / 0.10).clamp(0.0, 1.0) * 0.06
        });
        poses::aim_ranged(library, &mut local, pitch, recoil);
        self.model = library.model_pose(&local);
        // Palette retargeting solves the female arms against these weapon-local
        // grips while retaining the authored lower-body and recoil choreography.
        self.companion_hands = Some(grips);
        for (side, hand) in [("r", grips[0]), ("l", grips[1])] {
            let wrist = library.bone(&format!("hand_{side}"));
            self.model[wrist].translation = hand;
        }
    }

    /// Uses the already selected display state and clock. These are explicit
    /// Kiva crops/retargets of source motion, not recovered gameplay hit frames.
    fn advance_source(&mut self, library: &SourceAnimationLibrary, input: Inputs, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let selection = if input.mode == WeaponMode::Ranged
            || (input.crouched && !input.airborne)
            || input.sliding
        {
            None
        } else {
            match self.state {
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
        let selected = selection
            .and_then(|(name, time, looping)| library.clip(name).map(|clip| (clip, time, looping)));
        if let Some((clip, seconds, looping)) = selected {
            let mut target = library.sample(clip, seconds, looping, input.airborne);
            // A dodge uses one forward source clip; the simulation's accepted
            // direction supplies only this display-space facing adjustment.
            if self.state == PoseState::Dodge {
                let root = library.bone("Bip001").unwrap();
                target[root].rotation =
                    Quat::from_rotation_y(input.dodge_yaw) * target[root].rotation;
            }
            if self.source_clip != Some(clip) || self.transition <= dt {
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
                model: library.model_pose(&target),
                weight: (weight + dt / 0.055).min(1.0),
            });
            self.source_local = Some(target);
            self.source_clip = Some(clip);
        } else if let Some(source) = &mut self.source {
            source.weight = (source.weight - dt / 0.12).max(0.0);
            if source.weight == 0.0 {
                self.source = None;
                self.source_local = None;
                self.source_from = None;
                self.source_clip = None;
            }
        }
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
            CombatEffect::Melee => {
                if set == WeaponPair::ChainAxeBow {
                    Gesture::Axe
                } else {
                    Gesture::Hammer
                }
            }
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
            | CombatEffect::Rejected => return,
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
        let mut target = match state {
            PoseState::Idle => poses::idle(library, self.clock, input.mode),
            PoseState::Walk | PoseState::Run => library.sample(
                if state == PoseState::Run {
                    "Jog_Fwd_Loop"
                } else {
                    "Walk_Loop"
                },
                self.gait_time,
                true,
            ),
            PoseState::Air(jumps) => airborne_pose(library, self.air_time, jumps),
            PoseState::Land => library.sample("Jump_Land", 0.15 + self.clock * 4.0, false),
            PoseState::Dodge => {
                let mut pose = library.sample(
                    "Roll",
                    input.dodge_elapsed / 0.42 * library.duration("Roll"),
                    false,
                );
                pose[0].rotation = Quat::from_rotation_y(input.dodge_yaw) * pose[0].rotation;
                pose
            }
            PoseState::Crouch => {
                if input.speed > 0.01 {
                    let mut pose = library.sample("Walk_Loop", self.gait_time, true);
                    poses::hold_guard(library, &mut pose, input.mode);
                    pose
                } else {
                    poses::idle(library, self.clock, input.mode)
                }
            }
            PoseState::Slide => {
                let mut pose = library.sample("Sword_Idle", 0.0, false);
                poses::hold_guard(library, &mut pose, input.mode);
                pose[0].rotation = Quat::from_rotation_y(input.slide_yaw) * pose[0].rotation;
                pose
            }
            PoseState::Parry => library.sample("Sword_Block", 0.2 + self.clock.min(0.2), false),
            PoseState::Aim => poses::idle(library, self.clock, WeaponMode::Ranged),
            PoseState::Attack(kind) => poses::gesture(
                library,
                kind,
                self.playback
                    .as_ref()
                    .map_or(0.0, |playback| playback.elapsed),
                input.mode,
            ),
            PoseState::Deflected => poses::gesture(library, Gesture::HeavyHit, 0.18, input.mode),
            PoseState::Down => library.sample(
                "Death01",
                self.clock / 0.8 * library.duration("Death01"),
                false,
            ),
        };
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
    alive: bool,
    airborne: bool,
    jumps_used: u8,
    speed: f32,
    sprinting: bool,
    dodging: bool,
    dodge_elapsed: f32,
    dodge_yaw: f32,
    crouched: bool,
    sliding: bool,
    slide_yaw: f32,
    parrying: bool,
    sniper: Option<SniperPose>,
    mode: WeaponMode,
    boss_deflected: Option<(f32, f32)>,
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
        let Ok((body, motion, health, facing, combat, _, boss, defense)) = actors.get(view.owner)
        else {
            continue;
        };
        let set = combat.map_or(WeaponPair::HammerMusket, Combatant::weapon_pair);
        let mode = WeaponMode::Melee;
        if let Some(action) = combat.and_then(Combatant::active_action)
            && action.action_id != rig.action_id
        {
            let effect = match action.kind {
                ActionKind::Melee => Some(CombatEffect::Melee),
                ActionKind::HammerSlam => Some(CombatEffect::Hammer),
                ActionKind::ChainSweep => Some(CombatEffect::ChainSweep),
                _ => None,
            };
            if let Some(effect) = effect {
                rig.observe(&library, effect, set, false);
            }
            rig.action_id = action.action_id;
        }
        let previous_sequence = rig.event_sequence;
        for event in feedback
            .events()
            .filter(|event| event.sequence > previous_sequence && event.actor == view.owner)
        {
            if !(event.action_id == rig.action_id
                && matches!(
                    event.kind,
                    CombatEffect::Melee | CombatEffect::Hammer | CombatEffect::ChainSweep
                ))
            {
                rig.observe(&library, event.kind, set, combat.is_none());
            }
        }
        rig.event_sequence = latest_sequence;
        let direction = motion.dodge_direction();
        let dodge_yaw = (-direction.x).atan2(-direction.y)
            - (-facing.direction().x).atan2(-facing.direction().y);
        let slide_direction = motion.slide_direction();
        let slide_yaw = (-slide_direction.x).atan2(-slide_direction.y)
            - (-facing.direction().x).atan2(-facing.direction().y);
        let speed = body.step_displacement().length() / fixed.delta_secs().max(0.0001);
        let input = Inputs {
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
        rig.advance(&library, input, dt);
        if let Some(source) = &source_library {
            rig.advance_source(source, input, dt);
        }
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
            boss_deflected: None,
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
        }
    }

    #[test]
    fn original_named_motion_uses_state_priority_and_keeps_shared_moves_distinct() {
        let library = AnimationLibrary::default();
        let source = SourceAnimationLibrary::default();
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
        assert_eq!(rig.source_clip, source.clip("skill_ylk_1"));
        assert_eq!(rig.source.as_ref().unwrap().weight, 1.0);
        let held = rig.source.as_ref().unwrap().model.clone();
        step(&mut rig, input, 0.0);
        assert_eq!(held, rig.source.as_ref().unwrap().model);
        input.parrying = true;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source.clip("skill_huajie_01__shuangqiangt")
        );
        input.dodging = true;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(rig.source_clip, source.clip("skill_shoujia_dodge_f"));
        input.dodging = false;
        input.parrying = false;
        rig.playback = None;
        input.airborne = true;
        input.jumps_used = 1;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source.clip("b_fight_env_jump_01_start_Lfoot__zuiquan")
        );
        for _ in 0..20 {
            step(&mut rig, input, 1.0 / 60.0);
        }
        assert_eq!(
            rig.source_clip,
            source.clip("b_fight_env_jump_02_loop__zuiquan")
        );
        input.jumps_used = 2;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source.clip("b_fight_env_jump_01_start_Lfoot__zuiquan")
        );
        input.airborne = false;
        step(&mut rig, input, 1.0 / 60.0);
        assert_eq!(
            rig.source_clip,
            source.clip("b_fight_env_jump_03_to_idle__zuiquan")
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
        let rig = actor_rig(&app, boss);
        assert_eq!(rig.state, PoseState::Deflected);
        assert!(rig.playback.is_none());
        assert_eq!(
            rig.source_clip,
            app.world()
                .resource::<SourceAnimationLibrary>()
                .clip("weakhit_b")
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
}
