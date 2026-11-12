//! Replays one attack through the live display driver at the simulation's
//! tick rate, without a simulation. The offline hit bake and its staleness
//! check read the resulting poses; nothing here is in the product build.
use super::*;
use crate::simulation::TrackId;

pub(in crate::client) const STEP: f32 = 1.0 / 60.0;

pub(in crate::client) struct AttackReplay {
    rig: RigPose,
    pair: WeaponPair,
    /// A sword-bearing enemy or Boss rather than the native player.
    blade: bool,
    sequence: u64,
    action_id: u64,
}

impl AttackReplay {
    pub(in crate::client) fn new(
        library: &AnimationLibrary,
        pair: WeaponPair,
        blade: bool,
    ) -> Self {
        Self {
            rig: RigPose::new(library),
            pair,
            blade,
            sequence: 0,
            action_id: 0,
        }
    }

    pub(in crate::client) fn pose(&self) -> &RigPose {
        &self.rig
    }

    /// Display frames with no attack in progress.
    pub(in crate::client) fn idle(
        &mut self,
        library: &AnimationLibrary,
        sources: &SourceAnimationLibrary,
        frames: u32,
    ) {
        for _ in 0..frames {
            self.frame(library, sources, None, None);
        }
    }

    /// Begin a new attack; the next [`Self::show`] is its first frame.
    pub(in crate::client) fn begin(&mut self) {
        self.action_id += 1;
    }

    /// One display frame showing the current attack at `elapsed`.
    pub(in crate::client) fn show(
        &mut self,
        library: &AnimationLibrary,
        sources: &SourceAnimationLibrary,
        track: TrackId,
        elapsed: f32,
        duration: f32,
    ) -> &RigPose {
        let clock = AttackClock {
            action_id: self.action_id,
            track,
            elapsed,
            duration,
        };
        // Enemies announce their windup once, as the simulation does.
        let event = (self.blade && elapsed == 0.0).then_some(CombatEffect::EnemyWindup);
        self.frame(library, sources, Some(clock), event);
        &self.rig
    }

    fn frame(
        &mut self,
        library: &AnimationLibrary,
        sources: &SourceAnimationLibrary,
        clock: Option<AttackClock>,
        event: Option<CombatEffect>,
    ) {
        let action_snapshot = clock.filter(|_| !self.blade).map(|clock| {
            let (kind, melee_stage) = match clock.track {
                TrackId::PlayerMelee { stage, .. } => (ActionKind::Melee, Some(stage)),
                TrackId::PlayerSkill(kind) => (kind, None),
                TrackId::Boss { .. } | TrackId::Enemy => unreachable!("not a player track"),
            };
            ActionSnapshot {
                action_id: clock.action_id,
                kind,
                pair: self.pair,
                melee_stage,
                target: None,
                elapsed: clock.elapsed,
                duration: clock.duration,
                shot_count: 0,
                aim_position: Vec3::ZERO,
                source_position: Vec3::ZERO,
                companion_position: Vec3::ZERO,
            }
        });
        if event.is_some() {
            self.sequence += 1;
        }
        let sequence = self.sequence;
        self.rig.frame(
            library,
            Some(sources),
            DisplayFrame {
                input: Inputs {
                    attack: clock,
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
                },
                pair: self.pair,
                blade: self.blade,
                action: action_snapshot
                    .map(|action| (action.action_id, action.kind.strike_effect())),
                latest_sequence: sequence,
                action_snapshot,
                defense: None,
            },
            event.map(|kind| (sequence, 0, kind)),
            STEP,
        );
    }
}
