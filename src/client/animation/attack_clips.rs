//! Which source clip each attack timeline plays, and where in that clip each
//! moment of the simulation's [`AttackClock`] lands. One row per track; the
//! pose code samples these rows instead of keeping a render-time attack clock.
//! The offline hit bake replays the same rows, so changing one here changes
//! the baked hit data too.
use super::naraka::melee_clip;
use crate::client::source_animation::SourceRig;
use crate::simulation::{AttackClock, CombatEffect, TrackId};

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

/// The named source clip each Boss strike (a row of
/// [`crate::simulation::Boss::STRIKES`]) plays: the clip, where its span
/// begins and how long the Boss first holds that frame, in seconds. After the
/// hold the span plays at the clip's own speed for the rest of the strike's
/// timeline. A hold only stands in for a windup the clip itself is too quick
/// to telegraph.
///
/// Kiva adaptation: the source 9002xxxx slot -> clip mapping is unrecovered,
/// so the spans are chosen from each clip's swings, not source configuration.
/// The first eight are phase-one `skill_ylk_*` clips, the rest `skill_ylk_P2_*`.
pub(super) const BOSS_CLIPS: [(&str, f32, f32); 17] = [
    ("skill_ylk_12", 0.0, 0.0),
    ("skill_ylk_4", 0.0, 0.0),
    ("skill_ylk_7", 0.0, 0.0),
    ("skill_ylk_10", 0.0, 0.0),
    ("skill_ylk_6", 1.0, 0.0),
    ("skill_ylk_5", 1.2, 0.0),
    ("skill_ylk_1", 0.0, 0.4),
    ("skill_ylk_5_1", 1.2, 0.0),
    ("skill_ylk_P2_11", 0.0, 0.0),
    ("skill_ylk_P2_13", 1.0, 0.0),
    ("skill_ylk_P2_2", 0.6, 0.3),
    ("skill_ylk_P2_4", 0.0, 0.0),
    ("skill_ylk_P2_10", 3.6, 0.0),
    ("skill_ylk_P2_6", 0.6, 0.0),
    ("skill_ylk_P2_14", 0.0, 0.0),
    ("skill_ylk_P2_7", 0.6, 0.0),
    ("skill_ylk_P2_9", 1.6, 0.0),
];

/// Hold the clip at `start` for `hold` seconds, then play on at the clip's
/// speed until the `duration`-second timeline ends.
fn span(start: f32, hold: f32, duration: f32) -> [(f32, f32); 4] {
    let end = start + duration - hold;
    [
        (0.0, start),
        (hold / duration, start),
        (1.0, end),
        (1.0, end),
    ]
}

/// The row for `track`, or `None` when the attack has no body clip (shots
/// are performed by the companion).
pub(super) fn of(track: TrackId) -> Option<AttackClip> {
    let duration = track.duration();
    let (rig, clip, keys) = match track {
        TrackId::PlayerMelee { stage, .. } => (SourceRig::Naraka, melee_clip(stage), WHOLE),
        TrackId::Boss { strike } => {
            let &(clip, start, hold) = BOSS_CLIPS.get(usize::from(strike))?;
            (SourceRig::Yanyun, clip, span(start, hold, duration))
        }
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
    use crate::simulation::{ActionKind, MeleeStage, WeaponPair};

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
        // A Boss strike plays its span at the clip's own speed.
        let boss = TrackId::Boss { strike: 10 };
        assert_eq!(of(boss).unwrap().clip, "skill_ylk_P2_2");
        assert!((at(boss, 0.2) - 0.6).abs() < 1e-5);
        assert!((at(boss, 1.3) - 1.6).abs() < 1e-5);
        assert!((at(boss, boss.duration()) - (0.3 + boss.duration())).abs() < 1e-5);
        // Without a hold the span starts at once.
        let plain = TrackId::Boss { strike: 4 };
        assert!((at(plain, 0.5) - 1.5).abs() < 1e-5);
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

    #[test]
    fn every_boss_strike_plays_a_whole_span_of_its_own_named_clip() {
        let sources = crate::client::source_animation::SourceAnimationLibrary::bundled();
        let yanyun = sources.for_rig(SourceRig::Yanyun).unwrap();
        assert_eq!(BOSS_CLIPS.len(), crate::simulation::Boss::STRIKES.len());
        for (strike, duration) in crate::simulation::Boss::STRIKES.into_iter().enumerate() {
            let strike = strike as u8;
            let row = of(TrackId::Boss { strike }).unwrap();
            assert_eq!(row.rig, SourceRig::Yanyun);
            assert!(row.clip.starts_with("skill_ylk_"), "{strike}: {}", row.clip);
            let clip = yanyun
                .clip(row.clip)
                .unwrap_or_else(|| panic!("{} is not bundled", row.clip));
            let (start, hold, end) = (row.keys[0].1, row.keys[1].0 * duration, row.keys[3].1);
            assert!(
                (end - start + hold - duration).abs() < 1e-5,
                "{strike} is not 1x"
            );
            // The rules telegraph until the blade first becomes dangerous.
            let telegraph = TrackId::Boss { strike }.hit_track().unwrap().strike_time();
            assert!(
                telegraph >= 0.35,
                "{strike} telegraphs for only {telegraph}s"
            );
            assert!(
                start >= 0.0 && end <= yanyun.duration(clip) + 1e-3,
                "{strike}: {start}..{end} leaves {}",
                row.clip
            );
        }
    }
}
