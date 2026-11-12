//! Which source clip each attack timeline plays, and where in that clip each
//! moment of the simulation's [`AttackClock`] lands. One row per track; the
//! pose code samples these rows instead of keeping a render-time attack clock.
//! The offline hit bake replays the same rows, so changing one here changes
//! the baked hit data too.
use super::naraka::melee_clip;
use crate::client::source_animation::SourceRig;
use crate::simulation::{AttackClock, CombatEffect, TrackId};

/// The Boss strike's phase boundaries in seconds: telegraph end, active end,
/// recovery end. A test checks them against the real Boss timeline.
pub(super) const BOSS_STRIKE: [f32; 3] = [0.72, 0.94, 1.59];
/// The ordinary enemy's windup end and whole attack, in seconds.
pub(super) const ENEMY_ATTACK: [f32; 2] = [0.42, 0.84];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AttackClip {
    pub rig: SourceRig,
    pub clip: &'static str,
    /// `(fraction of the attack timeline, clip position)`, increasing in the
    /// first coordinate. Native hengdao attack clips take a clip fraction,
    /// every other clip takes seconds.
    keys: &'static [(f32, f32)],
}

impl AttackClip {
    /// The clip position for `clock`, linear between keys and held at the ends.
    pub(super) fn position(&self, clock: &AttackClock) -> f32 {
        let t = (clock.elapsed / clock.duration.max(0.001)).clamp(0.0, 1.0);
        let keys = self.keys;
        let mut previous = keys[0];
        if t <= previous.0 {
            return previous.1;
        }
        for &key in &keys[1..] {
            if t <= key.0 {
                let span = (key.0 - previous.0).max(1e-6);
                return previous.1 + (key.1 - previous.1) * (t - previous.0) / span;
            }
            previous = key;
        }
        previous.1
    }
}

const WHOLE: &[(f32, f32)] = &[(0.0, 0.0), (1.0, 1.0)];
const BOSS_KEYS: &[(f32, f32)] = &[
    (0.0, 0.0),
    (BOSS_STRIKE[0] / BOSS_STRIKE[2], 0.45),
    (BOSS_STRIKE[1] / BOSS_STRIKE[2], 0.65),
    (1.0, 1.0),
];
const ENEMY_KEYS: &[(f32, f32)] = &[
    (0.0, 0.3),
    (0.40 / ENEMY_ATTACK[1], 0.9),
    (ENEMY_ATTACK[0] / ENEMY_ATTACK[1], 0.9),
    (1.0, 1.8),
];
/// The hammer shockwave: 0.9–1.8s of the shared skill clip over a 0.6s
/// swing, of which the 0.5s skill plays the first five sixths.
const SLAM_KEYS: &[(f32, f32)] = &[(0.0, 0.9), (1.0, 0.9 + 0.5 / 0.6 * 0.9)];
/// The chain sweep: the same span over a 0.42s swing, then held.
const SWEEP_KEYS: &[(f32, f32)] = &[(0.0, 0.9), (0.42 / 0.5, 1.8), (1.0, 1.8)];

/// The row for `track`, or `None` when the attack has no body clip (shots
/// are performed by the companion).
pub(super) fn of(track: TrackId) -> Option<AttackClip> {
    let (rig, clip, keys) = match track {
        TrackId::PlayerMelee { stage, .. } => (SourceRig::Naraka, melee_clip(stage), WHOLE),
        TrackId::Boss { stage } => (SourceRig::Naraka, melee_clip(stage), BOSS_KEYS),
        TrackId::Enemy => (SourceRig::Yanyun, "skill_ylk_1", ENEMY_KEYS),
        TrackId::PlayerSkill(kind) => match kind.strike_effect()? {
            CombatEffect::Hammer => (SourceRig::Yanyun, "skill_ylk_1", SLAM_KEYS),
            CombatEffect::ChainSweep => (SourceRig::Yanyun, "skill_ylk_1", SWEEP_KEYS),
            _ => return None,
        },
    };
    Some(AttackClip { rig, clip, keys })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{ActionKind, MeleeStage, WeaponPair};

    fn clock(track: TrackId, elapsed: f32, duration: f32) -> AttackClock {
        AttackClock {
            action_id: 1,
            track,
            elapsed,
            duration,
        }
    }

    #[test]
    fn rows_keep_the_established_pacing() {
        let boss = of(TrackId::Boss {
            stage: MeleeStage::Two,
        })
        .unwrap();
        let at = |seconds| boss.position(&clock(boss_track(), seconds, BOSS_STRIKE[2]));
        assert_eq!(at(0.0), 0.0);
        assert!((at(BOSS_STRIKE[0]) - 0.45).abs() < 1e-5);
        assert!((at(BOSS_STRIKE[1]) - 0.65).abs() < 1e-5);
        assert!((at(BOSS_STRIKE[2]) - 1.0).abs() < 1e-5);

        let enemy = of(TrackId::Enemy).unwrap();
        let at = |seconds| enemy.position(&clock(TrackId::Enemy, seconds, ENEMY_ATTACK[1]));
        assert!((at(0.2) - (0.3 + 0.2 / 0.4 * 0.6)).abs() < 1e-5);
        assert!((at(0.41) - 0.9).abs() < 1e-5);
        assert!((at(0.42 + 0.21) - 1.35).abs() < 1e-5);

        let melee = of(TrackId::PlayerMelee {
            pair: WeaponPair::default(),
            stage: MeleeStage::Three,
        })
        .unwrap();
        assert_eq!(melee.clip, "male_hengdao_attack_light_03");
        let track = TrackId::PlayerMelee {
            pair: WeaponPair::default(),
            stage: MeleeStage::Three,
        };
        assert!((melee.position(&clock(track, 0.3, 0.6)) - 0.5).abs() < 1e-5);
    }

    fn boss_track() -> TrackId {
        TrackId::Boss {
            stage: MeleeStage::Two,
        }
    }

    #[test]
    fn shots_have_no_body_clip_and_area_skills_do() {
        let mut clips = 0;
        for kind in [
            ActionKind::Melee,
            ActionKind::HammerSlam,
            ActionKind::MusketShot,
            ActionKind::ChainSweep,
            ActionKind::AxeShot,
            ActionKind::HammerCombo,
            ActionKind::ChainCombo,
        ] {
            if let Some(row) = of(TrackId::PlayerSkill(kind)) {
                clips += 1;
                assert_eq!(row.rig, SourceRig::Yanyun);
                assert!(kind.strike_effect().is_some());
            }
        }
        assert_eq!(clips, 2);
    }
}
