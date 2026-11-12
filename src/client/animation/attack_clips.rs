//! Which source clip each attack timeline plays, and where in that clip each
//! moment of the simulation's [`AttackClock`] lands. One row per track; the
//! pose code samples these rows instead of keeping a render-time attack clock.
//! The offline hit bake replays the same rows, so changing one here changes
//! the baked hit data too.
use super::naraka::melee_clip;
use crate::client::source_animation::SourceRig;
use crate::simulation::{AttackClock, CombatEffect, MeleeStage, TrackId};

/// How long the Boss holds its windup before the blade comes down.
const BOSS_TELEGRAPH: f32 = 0.72;
/// The ordinary enemy's windup.
const ENEMY_WINDUP: f32 = 0.42;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AttackClip {
    pub rig: SourceRig,
    pub clip: &'static str,
    /// `(fraction of the attack timeline, clip position)`, increasing in the
    /// first coordinate. Native hengdao attack clips take a clip fraction,
    /// every other clip takes seconds.
    keys: [(f32, f32); 4],
}

impl AttackClip {
    /// The clip position for `clock`, linear between keys and held at the ends.
    pub(super) fn position(&self, clock: &AttackClock) -> f32 {
        let t = (clock.elapsed / clock.duration.max(0.001)).clamp(0.0, 1.0);
        let mut previous = self.keys[0];
        if t <= previous.0 {
            return previous.1;
        }
        for &key in &self.keys[1..] {
            if t <= key.0 {
                let span = (key.0 - previous.0).max(1e-6);
                return previous.1 + (key.1 - previous.1) * (t - previous.0) / span;
            }
            previous = key;
        }
        previous.1
    }
}

/// The whole clip, evenly.
const WHOLE: [(f32, f32); 4] = [(0.0, 0.0), (0.0, 0.0), (1.0, 1.0), (1.0, 1.0)];

/// A Boss strike holds the clip's windup through the telegraph, then plays
/// its swings at the clip's own speed and eases through the follow-through.
/// `windup` and `swings` are the clip fractions where the first swing begins
/// and the last one ends; `swing_time` is how long that span takes.
fn boss_keys(duration: f32, windup: f32, swings: f32, swing_time: f32) -> [(f32, f32); 4] {
    [
        (0.0, 0.0),
        (BOSS_TELEGRAPH / duration, windup),
        ((BOSS_TELEGRAPH + swing_time) / duration, swings),
        (1.0, 1.0),
    ]
}

/// The row for `track`, or `None` when the attack has no body clip (shots
/// are performed by the companion).
pub(super) fn of(track: TrackId) -> Option<AttackClip> {
    let duration = track.duration();
    let (rig, clip, keys) = match track {
        TrackId::PlayerMelee { stage, .. } => (SourceRig::Naraka, melee_clip(stage), WHOLE),
        TrackId::Boss { stage } => (
            SourceRig::Naraka,
            melee_clip(stage),
            match stage {
                MeleeStage::One => boss_keys(duration, 0.13, 0.42, 0.22),
                MeleeStage::Two => boss_keys(duration, 0.15, 0.53, 0.40),
                MeleeStage::Three => boss_keys(duration, 0.41, 0.65, 0.25),
            },
        ),
        // The clip winds up until 0.6s and has finished both cuts by 1.2s.
        TrackId::Enemy => (
            SourceRig::Yanyun,
            "skill_ylk_1",
            [
                (0.0, 0.3),
                (ENEMY_WINDUP / duration, 0.6),
                (1.0, 1.2),
                (1.0, 1.2),
            ],
        ),
        TrackId::PlayerSkill(kind) => match kind.strike_effect()? {
            // The hammer shockwave: 0.9–1.8s of the shared skill clip paced
            // for a 0.6s swing, of which the skill plays its own duration.
            CombatEffect::Hammer => (
                SourceRig::Yanyun,
                "skill_ylk_1",
                [
                    (0.0, 0.9),
                    (0.0, 0.9),
                    (1.0, 0.9 + duration / 0.6 * 0.9),
                    (1.0, 0.9 + duration / 0.6 * 0.9),
                ],
            ),
            // The chain sweep: the same span over a 0.42s swing, then held.
            CombatEffect::ChainSweep => (
                SourceRig::Yanyun,
                "skill_ylk_1",
                [(0.0, 0.9), (0.0, 0.9), (0.42 / duration, 1.8), (1.0, 1.8)],
            ),
            _ => return None,
        },
    };
    Some(AttackClip { rig, clip, keys })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{ActionKind, WeaponPair};

    fn at(track: TrackId, elapsed: f32) -> f32 {
        of(track).unwrap().position(&AttackClock {
            action_id: 1,
            track,
            elapsed,
            duration: track.duration(),
        })
    }

    #[test]
    fn rows_map_the_attack_timeline_onto_their_clip() {
        let boss = TrackId::Boss {
            stage: MeleeStage::Two,
        };
        assert_eq!(at(boss, 0.0), 0.0);
        assert!((at(boss, BOSS_TELEGRAPH) - 0.15).abs() < 1e-5);
        assert!((at(boss, BOSS_TELEGRAPH + 0.40) - 0.53).abs() < 1e-5);
        assert!((at(boss, boss.duration()) - 1.0).abs() < 1e-5);
        // Time never runs backwards and the ends are held.
        for track in TrackId::all() {
            let duration = track.duration();
            let mut previous = at(track, -1.0);
            assert_eq!(previous, at(track, 0.0));
            for step in 0..=100 {
                let position = at(track, duration * step as f32 / 100.0);
                assert!(position >= previous - 1e-6, "{track:?} runs backwards");
                previous = position;
            }
            assert_eq!(previous, at(track, duration + 1.0));
        }

        assert!((at(TrackId::Enemy, ENEMY_WINDUP / 2.0) - 0.45).abs() < 1e-5);
        assert!((at(TrackId::Enemy, ENEMY_WINDUP) - 0.6).abs() < 1e-5);

        let melee = TrackId::PlayerMelee {
            pair: WeaponPair::default(),
            stage: MeleeStage::Three,
        };
        assert_eq!(of(melee).unwrap().clip, "male_hengdao_attack_light_03");
        assert!((at(melee, melee.duration() / 2.0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn exactly_the_blade_swinging_attacks_have_a_body_clip() {
        // The ordinary attack's rows are per weapon pair and stage.
        for kind in ActionKind::ALL
            .into_iter()
            .filter(|kind| *kind != ActionKind::Melee)
        {
            let row = of(TrackId::PlayerSkill(kind));
            assert_eq!(
                row.is_some(),
                kind.swings_blade(),
                "{kind:?} clip and blade disagree"
            );
        }
        assert!(TrackId::all().into_iter().all(|track| of(track).is_some()));
    }
}
