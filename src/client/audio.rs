//! Sound consumes committed facts. Its clock never advances gameplay.
use bevy::prelude::*;
use std::collections::HashMap;

use super::controls::CaptureState;
use crate::{
    gameplay::Player,
    simulation::{
        ActionKind, ActionSnapshot, CombatEffect, CombatEvent, CombatFeedback, Combatant, Health,
        MeleeStage, MotionEvent, MotionFeedback, MotionKind, SimulationActive, WeaponPair,
    },
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaAudio"], js_name = playEffect)]
    fn browser_effect(name: &str) -> bool;
    #[wasm_bindgen(js_namespace = ["window", "kivaAudio"], js_name = cueOffset)]
    fn browser_offset(name: &str) -> f32;
    #[wasm_bindgen(js_namespace = ["window", "kivaAudio"], js_name = setActive)]
    fn browser_active(active: bool);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sound {
    Attack,
    Hit,
    Deflect,
    Naraka(&'static str),
}
impl Sound {
    fn name(self) -> &'static str {
        match self {
            Self::Attack => "attack",
            Self::Hit => "hit",
            Self::Deflect => "deflect",
            Self::Naraka(name) => name,
        }
    }
}

#[derive(Clone, Copy)]
struct ActorAudio {
    entity: Entity,
    pair: WeaponPair,
    alive: bool,
    action: Option<ActionSnapshot>,
}

struct PendingAttack {
    actor: Entity,
    action_id: u64,
    pair: WeaponPair,
    stage: MeleeStage,
    sound: Sound,
    at: f32,
}
impl PendingAttack {
    fn elapsed(&self, actor: &ActorAudio) -> Option<f32> {
        let action = actor.action?;
        (actor.alive
            && actor.entity == self.actor
            && actor.pair == self.pair
            && action.action_id == self.action_id
            && action.pair == self.pair
            && action.melee_stage == Some(self.stage)
            && action.kind == ActionKind::Melee)
            .then_some(action.elapsed)
    }
}

#[derive(Default)]
pub(super) struct AudioState {
    cancellation_epoch: u64,
    combat_cursor: super::FeedbackCursor,
    motion_cursor: super::FeedbackCursor,
    starts_seen: HashMap<Entity, u64>,
    pending: Vec<PendingAttack>,
}
impl AudioState {
    fn frame<'a>(
        &mut self,
        active: bool,
        cancellation_epoch: u64,
        actors: &[ActorAudio],
        combat: impl Iterator<Item = &'a CombatEvent>,
        motion: impl Iterator<Item = &'a MotionEvent>,
        offset: impl Fn(Sound) -> f32,
    ) -> Vec<Sound> {
        // An active final capture state can still contain a blur/recapture or
        // overflow cancellation. Drop that frame's facts and pending cues;
        // the next frame resumes without touching the committed simulation.
        let cancelled = self.cancellation_epoch != cancellation_epoch;
        self.cancellation_epoch = cancellation_epoch;
        let active = active && !cancelled;
        let mut sounds = Vec::new();
        self.starts_seen
            .retain(|entity, _| actors.iter().any(|actor| actor.entity == *entity));
        if !active {
            self.pending.clear();
        }
        self.pending
            .retain(|cue| actors.iter().any(|actor| cue.elapsed(actor).is_some()));
        for event in combat {
            let fresh = event.sequence > self.combat_cursor.0;
            let owner = actors
                .iter()
                .any(|actor| actor.entity == event.source_owner);
            let defender = actors.iter().any(|actor| actor.entity == event.actor);
            if let Some(sound) = consume(&mut self.combat_cursor, event, active, owner, defender) {
                sounds.push(sound);
                if sound == Sound::Naraka("naraka_parry") {
                    // The original riposte whoosh accompanies an actual settled
                    // parry. Empty attempts remain silent; it is not proof of a hit.
                    sounds.push(Sound::Naraka("naraka_riposte"));
                }
            }
            if active
                && fresh
                && event.kind == CombatEffect::MeleeStart
                && let Some(actor) = actors.iter().find(|actor| actor.entity == event.actor)
                && actor.entity == event.source_owner
                && let (Some(stage), Some(pair)) = (event.melee_stage, event.pair)
                && !self
                    .starts_seen
                    .get(&actor.entity)
                    .is_some_and(|id| *id >= event.action_id)
            {
                let sound = Sound::Naraka(match stage {
                    MeleeStage::One => "naraka_attack_1",
                    MeleeStage::Two => "naraka_attack_2",
                    MeleeStage::Three => "naraka_attack_3",
                });
                let at = offset(sound);
                let at = if at.is_finite() && (0.0..=2.0).contains(&at) {
                    at
                } else {
                    0.0
                };
                let cue = PendingAttack {
                    actor: actor.entity,
                    action_id: event.action_id,
                    pair,
                    stage,
                    sound,
                    at,
                };
                if cue
                    .elapsed(actor)
                    .is_some_and(|elapsed| elapsed <= at + 0.12)
                {
                    self.starts_seen.insert(actor.entity, event.action_id);
                    // One whoosh currently, with room for two explicitly authored
                    // cue points per action; never a browser wall-clock queue.
                    if self.pending.len() < 8
                        && self
                            .pending
                            .iter()
                            .filter(|pending| {
                                pending.actor == actor.entity
                                    && pending.action_id == event.action_id
                            })
                            .count()
                            < 2
                    {
                        self.pending.push(cue);
                    }
                }
            }
        }
        for event in motion {
            if !self.motion_cursor.advance(event.sequence) {
                continue;
            }
            if !active
                || !actors
                    .iter()
                    .any(|actor| actor.entity == event.actor && actor.alive)
            {
                continue;
            }
            sounds.push(Sound::Naraka(match event.kind {
                MotionKind::Jump => "naraka_jump",
                MotionKind::Dodge => "naraka_dodge",
                MotionKind::Land => "naraka_land",
                MotionKind::Footstep => "naraka_step",
            }));
        }
        self.pending.retain(|cue| {
            let elapsed = actors.iter().find_map(|actor| cue.elapsed(actor));
            match elapsed {
                Some(elapsed) if elapsed < cue.at => true,
                Some(elapsed) if elapsed <= cue.at + 0.12 => {
                    sounds.push(cue.sound);
                    false
                }
                _ => false,
            }
        });
        sounds
    }
}

pub(super) fn sync(
    feedback: Res<CombatFeedback>,
    motion: Res<MotionFeedback>,
    active: Res<SimulationActive>,
    capture: Res<CaptureState>,
    players: Query<(Entity, &Combatant, &Health), With<Player>>,
    mut state: Local<AudioState>,
) {
    #[cfg(target_arch = "wasm32")]
    browser_active(active.0);
    let actors: Vec<_> = players
        .iter()
        .map(|(entity, combat, health)| ActorAudio {
            entity,
            pair: combat.weapon_pair(),
            alive: health.is_alive(),
            action: combat.active_action(),
        })
        .collect();
    for sound in state.frame(
        active.0,
        capture.cancellation_epoch(),
        &actors,
        feedback.events(),
        motion.events(),
        cue_offset,
    ) {
        play(sound);
    }
}

fn cue_offset(sound: Sound) -> f32 {
    #[cfg(target_arch = "wasm32")]
    {
        browser_offset(sound.name())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = sound;
        0.0
    }
}

fn consume(
    cursor: &mut super::FeedbackCursor,
    event: &CombatEvent,
    active: bool,
    is_player: bool,
    defender: bool,
) -> Option<Sound> {
    // Inactive frames advance the cursor too; resume never replays old facts.
    if !cursor.advance(event.sequence) || !active {
        return None;
    }
    match event.kind {
        CombatEffect::Melee if event.melee_stage.is_some() => None,
        CombatEffect::Melee | CombatEffect::Hammer | CombatEffect::ChainSweep => {
            Some(Sound::Attack)
        }
        CombatEffect::Hit if defender || (is_player && event.melee_stage.is_some()) => {
            Some(Sound::Naraka("naraka_hit"))
        }
        CombatEffect::Hit => Some(Sound::Hit),
        CombatEffect::Dodge if defender => Some(Sound::Naraka("naraka_dodge_success")),
        CombatEffect::Deflect if defender => Some(Sound::Naraka("naraka_deflect")),
        CombatEffect::Block if defender => Some(Sound::Naraka("naraka_block")),
        CombatEffect::Parry if defender => Some(Sound::Naraka("naraka_parry")),
        CombatEffect::Deflect | CombatEffect::Block | CombatEffect::Parry => Some(Sound::Deflect),
        CombatEffect::Defeat if defender => Some(Sound::Naraka("naraka_death")),
        // Native flintlock/bow family sounds are Kiva adaptations, not media
        // exclusive to the displayed weapon skin or the companion character.
        CombatEffect::Shot if is_player => event
            .pair
            .map(|pair| Sound::Naraka(super::loadout::of(pair).shot_sound)),
        // SniperShot accompanies Shot; successful dodge is handled separately from motion start.
        _ => None,
    }
}

fn play(sound: Sound) {
    #[cfg(target_arch = "wasm32")]
    let _ = browser_effect(sound.name());
    #[cfg(not(target_arch = "wasm32"))]
    let _ = sound.name();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(sequence: u64, kind: CombatEffect) -> CombatEvent {
        CombatEvent {
            sequence,
            actor: Entity::PLACEHOLDER,
            source_owner: Entity::PLACEHOLDER,
            pair: Some(WeaponPair::HammerMusket),
            melee_stage: None,
            action_id: 1,
            hit_index: 0,
            kind,
            position: Vec3::ZERO,
            end: Vec3::Z,
            radius: 0.0,
        }
    }

    #[test]
    fn sounds_follow_actual_feedback_once_for_both_sides_of_combat() {
        let mut cursor = crate::client::FeedbackCursor::default();
        for (sequence, kind, player, expected) in [
            (1, CombatEffect::EnemyWindup, false, None),
            (2, CombatEffect::Melee, false, Some(Sound::Attack)),
            (3, CombatEffect::Hit, true, Some(Sound::Hit)),
            (4, CombatEffect::Deflect, true, Some(Sound::Deflect)),
            (
                5,
                CombatEffect::Shot,
                true,
                Some(Sound::Naraka("naraka_gun")),
            ),
            (6, CombatEffect::SniperShot, true, None),
        ] {
            let event = event(sequence, kind);
            assert_eq!(consume(&mut cursor, &event, true, player, false), expected);
            assert_eq!(consume(&mut cursor, &event, true, player, false), None);
        }
    }

    #[test]
    fn paused_feedback_and_older_history_cannot_replay_on_resume() {
        let mut cursor = crate::client::FeedbackCursor::default();
        let hit = event(143, CombatEffect::Deflect);
        assert_eq!(consume(&mut cursor, &hit, false, true, false), None);
        assert_eq!(cursor.0, 143);
        assert_eq!(consume(&mut cursor, &hit, true, true, false), None);
        assert_eq!(
            consume(
                &mut cursor,
                &event(142, CombatEffect::Melee),
                true,
                true,
                false
            ),
            None
        );
        assert_eq!(
            consume(
                &mut cursor,
                &event(144, CombatEffect::Melee),
                true,
                true,
                false
            ),
            Some(Sound::Attack)
        );
    }

    #[test]
    fn shot_sister_triple_plays_each_actual_shot_once_despite_impact_feedback() {
        use crate::simulation::{CombatIntent, Combatant, SimulationPlugin};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        crate::gameplay::setup(app.world_mut());
        crate::gameplay::manual_targeting(app.world_mut());
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        let tick = |app: &mut App, n| {
            for _ in 0..n {
                app.world_mut()
                    .resource_mut::<Time<Fixed>>()
                    .advance_by(std::time::Duration::from_secs_f64(1. / 60.));
                app.world_mut().run_schedule(FixedUpdate);
            }
        };
        tick(&mut app, 12);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_toggle_target();
        tick(&mut app, 1);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_melee_skill();
        tick(&mut app, 14);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_combo();
        tick(&mut app, 45);
        let sister = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .companion()
            .unwrap();
        let mut cursor = crate::client::FeedbackCursor::default();
        let feedback = app.world().resource::<CombatFeedback>();
        let mut shots = 0;
        for event in feedback.events() {
            if consume(
                &mut cursor,
                event,
                true,
                event.source_owner == player,
                false,
            ) == Some(Sound::Naraka("naraka_gun"))
            {
                assert_eq!(event.actor, sister);
                shots += 1;
            }
        }
        assert_eq!(shots, 3);
        assert!(
            feedback
                .events()
                .any(|e| e.kind == CombatEffect::SniperShot)
        );
    }
    #[test]
    fn shot_sound_uses_immutable_event_pair_even_when_consuming_history() {
        let mut cursor = crate::client::FeedbackCursor::default();
        let mut arrow = event(1, CombatEffect::Shot);
        arrow.pair = Some(WeaponPair::ChainAxeBow);
        assert_eq!(
            consume(&mut cursor, &arrow, true, true, false),
            Some(Sound::Naraka("naraka_bow"))
        );
        assert_eq!(
            consume(
                &mut cursor,
                &event(2, CombatEffect::Shot),
                true,
                true,
                false
            ),
            Some(Sound::Naraka("naraka_gun"))
        );
        assert_eq!(consume(&mut cursor, &arrow, true, true, false), None);
        assert_eq!(
            consume(
                &mut cursor,
                &event(3, CombatEffect::SniperShot),
                true,
                true,
                false
            ),
            None
        );
        let mut unidentified = event(4, CombatEffect::Shot);
        unidentified.pair = None;
        assert_eq!(consume(&mut cursor, &unidentified, true, true, false), None);
        assert_eq!(
            consume(
                &mut cursor,
                &event(5, CombatEffect::Shot),
                true,
                false,
                false
            ),
            None
        );
    }

    #[test]
    fn shot_actual_companion_uses_original_family_cue_after_a_later_weapon_switch() {
        use crate::simulation::{CombatIntent, SimulationPlugin};
        for (pair, other, cue) in [
            (
                WeaponPair::HammerMusket,
                WeaponPair::ChainAxeBow,
                "naraka_gun",
            ),
            (
                WeaponPair::ChainAxeBow,
                WeaponPair::HammerMusket,
                "naraka_bow",
            ),
        ] {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, SimulationPlugin));
            app.finish();
            crate::gameplay::setup(app.world_mut());
            crate::gameplay::manual_targeting(app.world_mut());
            let player = app
                .world_mut()
                .query_filtered::<Entity, With<Player>>()
                .single(app.world())
                .unwrap();
            let tick = |app: &mut App, count| {
                for _ in 0..count {
                    app.world_mut()
                        .resource_mut::<Time<Fixed>>()
                        .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
                    app.world_mut().run_schedule(FixedUpdate);
                }
            };
            tick(&mut app, 30);
            {
                let mut intent = app.world_mut().get_mut::<CombatIntent>(player).unwrap();
                intent.request_pair(pair);
                intent.request_toggle_target();
            }
            tick(&mut app, 1);
            app.world_mut()
                .get_mut::<CombatIntent>(player)
                .unwrap()
                .request_ranged_skill();
            tick(&mut app, 60);
            let sister = app
                .world()
                .get::<Combatant>(player)
                .unwrap()
                .companion()
                .unwrap();
            app.world_mut()
                .get_mut::<CombatIntent>(player)
                .unwrap()
                .request_pair(other);
            tick(&mut app, 1);
            let combat = app.world().get::<Combatant>(player).unwrap();
            assert_eq!(combat.weapon_pair(), other);
            let actor = ActorAudio {
                entity: player,
                pair: other,
                alive: true,
                action: combat.active_action(),
            };
            let feedback = app.world().resource::<CombatFeedback>();
            let shots: Vec<_> = feedback
                .events()
                .filter(|event| event.kind == CombatEffect::Shot)
                .collect();
            assert_eq!(shots.len(), 1);
            assert_eq!(shots[0].actor, sister);
            assert_ne!(shots[0].actor, player);
            assert_eq!(shots[0].source_owner, player);
            assert_eq!(shots[0].pair, Some(pair));
            let mut state = AudioState::default();
            let sounds = state.frame(true, 0, &[actor], feedback.events(), [].into_iter(), |_| {
                0.0
            });
            let ranged: Vec<_> = sounds
                .into_iter()
                .filter(|sound| matches!(sound, Sound::Naraka("naraka_gun" | "naraka_bow")))
                .collect();
            assert_eq!(ranged, [Sound::Naraka(cue)]);
            assert!(
                state
                    .frame(true, 0, &[actor], feedback.events(), [].into_iter(), |_| {
                        0.0
                    })
                    .is_empty()
            );
        }
    }

    fn make_actor(stage: MeleeStage, elapsed: f32) -> ActorAudio {
        ActorAudio {
            entity: Entity::from_raw_u32(123).unwrap(),
            pair: WeaponPair::HammerMusket,
            alive: true,
            action: Some(ActionSnapshot {
                action_id: 7,
                kind: ActionKind::Melee,
                pair: WeaponPair::HammerMusket,
                melee_stage: Some(stage),
                target: None,
                elapsed,
                duration: 0.6,
                shot_count: 0,
                aim_position: Vec3::ZERO,
                source_position: Vec3::ZERO,
                companion_position: Vec3::ZERO,
            }),
        }
    }

    fn start(sequence: u64, actor: ActorAudio) -> CombatEvent {
        let mut event = event(sequence, CombatEffect::MeleeStart);
        event.actor = actor.entity;
        event.source_owner = actor.entity;
        event.action_id = actor.action.unwrap().action_id;
        event.pair = Some(actor.pair);
        event.melee_stage = actor.action.unwrap().melee_stage;
        event
    }

    #[test]
    fn committed_stages_sound_once_from_start_and_never_again_at_the_damage_marker() {
        for (stage, name) in [
            (MeleeStage::One, "naraka_attack_1"),
            (MeleeStage::Two, "naraka_attack_2"),
            (MeleeStage::Three, "naraka_attack_3"),
        ] {
            let actor = make_actor(stage, 0.0);
            let mut state = AudioState::default();
            let event = start(1, actor);
            assert_eq!(
                state.frame(
                    true,
                    0,
                    &[actor],
                    [&event].into_iter(),
                    [].into_iter(),
                    |_| { 0.0 }
                ),
                [Sound::Naraka(name)]
            );
            assert!(
                state
                    .frame(
                        true,
                        0,
                        &[actor],
                        [&event].into_iter(),
                        [].into_iter(),
                        |_| { 0.0 }
                    )
                    .is_empty()
            );
            let mut duplicate = event;
            duplicate.sequence = 2;
            let mut marker = event;
            marker.sequence = 3;
            marker.kind = CombatEffect::Melee;
            assert!(
                state
                    .frame(
                        true,
                        0,
                        &[actor],
                        [&duplicate, &marker].into_iter(),
                        [].into_iter(),
                        |_| 0.0
                    )
                    .is_empty()
            );
            marker.sequence = 4;
            marker.kind = CombatEffect::Hit;
            assert_eq!(
                state.frame(
                    true,
                    0,
                    &[actor],
                    [&marker].into_iter(),
                    [].into_iter(),
                    |_| 0.0
                ),
                [Sound::Naraka("naraka_hit")]
            );
        }
    }

    #[test]
    fn late_ended_or_mismatched_starts_are_discarded_without_using_current_pair_or_stage() {
        let original = make_actor(MeleeStage::Two, 0.0);
        let event = start(1, original);
        let mut variants = [original; 6];
        variants[0].action = None;
        variants[1].action.as_mut().unwrap().action_id += 1;
        variants[2].pair = WeaponPair::ChainAxeBow;
        variants[3].action.as_mut().unwrap().melee_stage = Some(MeleeStage::Three);
        variants[4].action.as_mut().unwrap().elapsed = 0.121;
        variants[5].alive = false;
        for changed in variants {
            let mut state = AudioState::default();
            assert!(
                state
                    .frame(
                        true,
                        0,
                        &[changed],
                        [&event].into_iter(),
                        [].into_iter(),
                        |_| 0.0
                    )
                    .is_empty()
            );
            assert!(
                state
                    .frame(
                        true,
                        0,
                        &[original],
                        [&event].into_iter(),
                        [].into_iter(),
                        |_| 0.0
                    )
                    .is_empty()
            );
        }
    }

    #[test]
    fn future_cues_follow_only_the_matching_fixed_action_clock_and_drop_on_invalidation() {
        let mut actor = make_actor(MeleeStage::One, 0.0);
        let event = start(1, actor);
        let mut state = AudioState::default();
        assert!(
            state
                .frame(
                    true,
                    0,
                    &[actor],
                    [&event].into_iter(),
                    [].into_iter(),
                    |_| { 0.2 }
                )
                .is_empty()
        );
        for _ in 0..5 {
            assert!(
                state
                    .frame(true, 0, &[actor], [].into_iter(), [].into_iter(), |_| 0.2)
                    .is_empty()
            );
        }
        actor.action.as_mut().unwrap().elapsed = 0.2;
        assert_eq!(
            state.frame(true, 0, &[actor], [].into_iter(), [].into_iter(), |_| 0.2),
            [Sound::Naraka("naraka_attack_1")]
        );
        assert!(
            state
                .frame(true, 0, &[actor], [].into_iter(), [].into_iter(), |_| 0.2)
                .is_empty()
        );
        for reason in 0..6 {
            let original = make_actor(MeleeStage::One, 0.0);
            let mut state = AudioState::default();
            state.frame(
                true,
                0,
                &[original],
                [&event].into_iter(),
                [].into_iter(),
                |_| 0.2,
            );
            let mut changed = original;
            match reason {
                0 => changed.action = None,
                1 => changed.action.as_mut().unwrap().action_id += 1,
                2 => changed.pair = WeaponPair::ChainAxeBow,
                3 => changed.alive = false,
                4 => changed.action.as_mut().unwrap().elapsed = 0.321,
                _ => {
                    state.frame(
                        false,
                        0,
                        &[original],
                        [].into_iter(),
                        [].into_iter(),
                        |_| 0.2,
                    );
                }
            }
            assert!(
                state
                    .frame(true, 0, &[changed], [].into_iter(), [].into_iter(), |_| 0.2)
                    .is_empty()
            );
            let mut resumed = original;
            resumed.action.as_mut().unwrap().elapsed = 0.2;
            assert!(
                state
                    .frame(
                        true,
                        0,
                        &[resumed],
                        [&event].into_iter(),
                        [].into_iter(),
                        |_| 0.2
                    )
                    .is_empty()
            );
            assert!(state.pending.is_empty());
        }
    }

    #[test]
    fn cancellation_epoch_drops_future_cues_and_that_frames_facts_while_still_active() {
        let mut actor = make_actor(MeleeStage::One, 0.0);
        let start = start(1, actor);
        let mut state = AudioState::default();
        assert!(
            state
                .frame(
                    true,
                    0,
                    &[actor],
                    [&start].into_iter(),
                    [].into_iter(),
                    |_| 0.2
                )
                .is_empty()
        );
        assert_eq!(state.pending.len(), 1);
        actor.action.as_mut().unwrap().elapsed = 0.2;
        let mut hit = start;
        hit.kind = CombatEffect::Hit;
        hit.sequence = 2;
        let motion = MotionEvent {
            sequence: 1,
            actor: actor.entity,
            kind: MotionKind::Jump,
            position: Vec3::ZERO,
            jump_count: 1,
            sprinting: false,
        };
        assert!(
            state
                .frame(
                    true,
                    1,
                    &[actor],
                    [&start, &hit].into_iter(),
                    [&motion].into_iter(),
                    |_| 0.2
                )
                .is_empty()
        );
        assert!(state.pending.is_empty());
        assert!(
            state
                .frame(
                    true,
                    1,
                    &[actor],
                    [&start, &hit].into_iter(),
                    [&motion].into_iter(),
                    |_| 0.2
                )
                .is_empty()
        );
        let resumed_motion = MotionEvent {
            sequence: 2,
            ..motion
        };
        assert_eq!(
            state.frame(
                true,
                1,
                &[actor],
                [].into_iter(),
                [&resumed_motion].into_iter(),
                |_| 0.2
            ),
            [Sound::Naraka("naraka_jump")]
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn actual_control_cancellation_with_no_fixed_step_clears_an_unplayed_offset_cue() {
        use super::super::controls::{ViewYaw, sample};
        use crate::simulation::{CombatIntent, SimulationPlugin};
        use bevy::{
            input::{
                ButtonState,
                keyboard::KeyboardInput,
                mouse::{AccumulatedMouseMotion, MouseButtonInput},
            },
            window::{CursorOptions, PrimaryWindow, WindowFocused},
        };
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin))
            .add_message::<WindowFocused>()
            .add_message::<KeyboardInput>()
            .add_message::<MouseButtonInput>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<CaptureState>()
            .init_resource::<ViewYaw>()
            .add_systems(Update, sample);
        app.finish();
        crate::gameplay::setup(app.world_mut());
        crate::gameplay::manual_targeting(app.world_mut());
        let window = app
            .world_mut()
            .spawn((Window::default(), CursorOptions::default(), PrimaryWindow))
            .id();
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        app.world_mut().write_message(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window,
        });
        app.world_mut().run_schedule(Update);
        let before = app.world().resource::<CaptureState>().cancellation_epoch();
        let mut state = AudioState {
            cancellation_epoch: before,
            ..Default::default()
        };
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
        app.world_mut().run_schedule(FixedUpdate);
        let snapshot = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .active_action()
            .unwrap();
        let actor = ActorAudio {
            entity: player,
            pair: snapshot.pair,
            alive: true,
            action: Some(snapshot),
        };
        assert!(
            state
                .frame(
                    true,
                    before,
                    &[actor],
                    app.world().resource::<CombatFeedback>().events(),
                    [].into_iter(),
                    |_| 0.2
                )
                .is_empty()
        );
        assert_eq!(state.pending.len(), 1);
        // Queue overflow produces the same Cancelled edge as a browser sample
        // ending active after blur/recapture, through the real input sampler.
        for _ in 0..65 {
            for state in [ButtonState::Pressed, ButtonState::Released] {
                app.world_mut().write_message(MouseButtonInput {
                    button: MouseButton::Right,
                    state,
                    window,
                });
            }
        }
        app.world_mut().run_schedule(Update);
        let after = app.world().resource::<CaptureState>().cancellation_epoch();
        assert!(after > before);
        assert!(app.world().resource::<SimulationActive>().0);
        let current = app
            .world()
            .get::<Combatant>(player)
            .unwrap()
            .active_action()
            .unwrap();
        assert_eq!(current.action_id, snapshot.action_id);
        assert_eq!(current.elapsed, snapshot.elapsed);
        assert!(
            state
                .frame(
                    true,
                    after,
                    &[actor],
                    app.world().resource::<CombatFeedback>().events(),
                    [].into_iter(),
                    |_| 0.2
                )
                .is_empty()
        );
        assert!(state.pending.is_empty());
        let mut later = actor;
        later.action.as_mut().unwrap().elapsed = 0.2;
        assert!(
            state
                .frame(
                    true,
                    after,
                    &[later],
                    app.world().resource::<CombatFeedback>().events(),
                    [].into_iter(),
                    |_| 0.2
                )
                .is_empty()
        );
        app.world_mut().run_schedule(Update);
        assert_eq!(
            app.world().resource::<CaptureState>().cancellation_epoch(),
            after
        );
    }

    #[test]
    fn metal_feedback_uses_the_defending_actor_and_empty_attempts_remain_silent() {
        let actor = make_actor(MeleeStage::One, 0.0);
        let attacker = Entity::from_raw_u32(124).unwrap();
        let mut state = AudioState::default();
        assert!(
            state
                .frame(true, 0, &[actor], [].into_iter(), [].into_iter(), |_| 0.0)
                .is_empty()
        );
        for (sequence, kind, name) in [
            (1, CombatEffect::Deflect, "naraka_deflect"),
            (2, CombatEffect::Block, "naraka_block"),
            (3, CombatEffect::Parry, "naraka_parry"),
        ] {
            let mut event = event(sequence, kind);
            event.actor = actor.entity;
            event.source_owner = attacker;
            event.pair = Some(WeaponPair::ChainAxeBow);
            let result = state.frame(
                true,
                0,
                &[actor],
                [&event].into_iter(),
                [].into_iter(),
                |_| 0.0,
            );
            let expected = if kind == CombatEffect::Parry {
                vec![Sound::Naraka(name), Sound::Naraka("naraka_riposte")]
            } else {
                vec![Sound::Naraka(name)]
            };
            assert_eq!(result, expected);
            event.sequence += 3;
            event.actor = attacker;
            event.source_owner = actor.entity;
            // Separate cursor to compare the opposite side of the same result.
            assert_eq!(
                AudioState::default().frame(
                    true,
                    0,
                    &[actor],
                    [&event].into_iter(),
                    [].into_iter(),
                    |_| 0.0
                ),
                [Sound::Deflect]
            );
        }
        let mut death = event(7, CombatEffect::Defeat);
        death.actor = actor.entity;
        death.source_owner = attacker;
        assert_eq!(
            state.frame(
                true,
                0,
                &[actor],
                [&death].into_iter(),
                [].into_iter(),
                |_| { 0.0 }
            ),
            [Sound::Naraka("naraka_death")]
        );
    }

    #[test]
    fn motion_has_an_independent_cursor_and_drops_inactive_nonplayer_or_dead_facts() {
        let actor = make_actor(MeleeStage::One, 0.0);
        let mut state = AudioState::default();
        let motion = |sequence, kind| MotionEvent {
            sequence,
            actor: actor.entity,
            kind,
            position: Vec3::ZERO,
            jump_count: 0,
            sprinting: false,
        };
        for (sequence, kind, name) in [
            (1, MotionKind::Jump, "naraka_jump"),
            (2, MotionKind::Dodge, "naraka_dodge"),
            (3, MotionKind::Land, "naraka_land"),
            (4, MotionKind::Footstep, "naraka_step"),
        ] {
            let motion = motion(sequence, kind);
            assert_eq!(
                state.frame(
                    true,
                    0,
                    &[actor],
                    [].into_iter(),
                    [&motion].into_iter(),
                    |_| 0.0
                ),
                [Sound::Naraka(name)]
            );
            assert!(
                state
                    .frame(
                        true,
                        0,
                        &[actor],
                        [].into_iter(),
                        [&motion].into_iter(),
                        |_| 0.0
                    )
                    .is_empty()
            );
        }
        let paused = motion(5, MotionKind::Jump);
        assert!(
            state
                .frame(
                    false,
                    0,
                    &[actor],
                    [].into_iter(),
                    [&paused].into_iter(),
                    |_| 0.0
                )
                .is_empty()
        );
        assert!(
            state
                .frame(
                    true,
                    0,
                    &[actor],
                    [].into_iter(),
                    [&paused].into_iter(),
                    |_| 0.0
                )
                .is_empty()
        );
        let mut npc = motion(6, MotionKind::Land);
        npc.actor = Entity::PLACEHOLDER;
        assert!(
            state
                .frame(
                    true,
                    0,
                    &[actor],
                    [].into_iter(),
                    [&npc].into_iter(),
                    |_| 0.0
                )
                .is_empty()
        );
        let mut dead = actor;
        dead.alive = false;
        let dead_motion = motion(7, MotionKind::Footstep);
        assert!(
            state
                .frame(
                    true,
                    0,
                    &[dead],
                    [].into_iter(),
                    [&dead_motion].into_iter(),
                    |_| 0.0
                )
                .is_empty()
        );
        // A non-player immunity event must not trigger either player dodge cue.
        let dodge = event(999, CombatEffect::Dodge);
        assert!(
            state
                .frame(
                    true,
                    0,
                    &[actor],
                    [&dodge].into_iter(),
                    [].into_iter(),
                    |_| { 0.0 }
                )
                .is_empty()
        );
        let live_motion = motion(8, MotionKind::Dodge);
        assert_eq!(
            state.frame(
                true,
                0,
                &[actor],
                [].into_iter(),
                [&live_motion].into_iter(),
                |_| 0.0
            ),
            [Sound::Naraka("naraka_dodge")]
        );
    }

    #[test]
    fn successful_dodge_plays_only_for_the_player_once_and_never_replays_after_pause() {
        let actor = make_actor(MeleeStage::One, 0.0);
        let mut state = AudioState::default();
        let mut success = event(1, CombatEffect::Dodge);
        success.actor = actor.entity;
        success.source_owner = Entity::from_raw_u32(456).unwrap();
        let play = |state: &mut AudioState, active, event: &CombatEvent| {
            state.frame(
                active,
                0,
                &[actor],
                [event].into_iter(),
                [].into_iter(),
                |_| 0.0,
            )
        };
        assert_eq!(
            play(&mut state, true, &success),
            [Sound::Naraka("naraka_dodge_success")]
        );
        assert!(play(&mut state, true, &success).is_empty());
        success.sequence = 2;
        assert!(play(&mut state, false, &success).is_empty());
        assert!(play(&mut state, true, &success).is_empty());
        success.sequence = 3;
        success.actor = success.source_owner;
        assert!(play(&mut state, true, &success).is_empty());
    }

    #[test]
    fn real_fixed_update_starts_and_accepted_motion_reach_audio_once() {
        use crate::simulation::{ActionIntent, CombatIntent, SimulationPlugin};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        crate::gameplay::setup(app.world_mut());
        crate::gameplay::manual_targeting(app.world_mut());
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        let tick = |app: &mut App| {
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
            app.world_mut().run_schedule(FixedUpdate);
        };
        let sample = |app: &App, state: &mut AudioState| {
            let combat = app.world().get::<Combatant>(player).unwrap();
            let actor = ActorAudio {
                entity: player,
                pair: combat.weapon_pair(),
                alive: app.world().get::<Health>(player).unwrap().is_alive(),
                action: combat.active_action(),
            };
            state.frame(
                true,
                0,
                &[actor],
                app.world().resource::<CombatFeedback>().events(),
                app.world().resource::<MotionFeedback>().events(),
                |_| 0.0,
            )
        };
        let mut state = AudioState::default();
        for _ in 0..30 {
            tick(&mut app);
            sample(&app, &mut state);
        }
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        let mut last_action = None;
        let mut heard = Vec::new();
        for _ in 0..120 {
            tick(&mut app);
            heard.extend(sample(&app, &mut state));
            // Buffer exactly one successor after each actual accepted start.
            if let Some(action) = app
                .world()
                .get::<Combatant>(player)
                .unwrap()
                .active_action()
                && Some(action.action_id) != last_action
            {
                last_action = Some(action.action_id);
                if action.melee_stage != Some(MeleeStage::Three) {
                    app.world_mut()
                        .get_mut::<CombatIntent>(player)
                        .unwrap()
                        .request_attack();
                }
            }
        }
        let whooshes: Vec<_> = heard
            .iter()
            .copied()
            .filter(|sound| {
                matches!(
                    sound,
                    Sound::Naraka("naraka_attack_1" | "naraka_attack_2" | "naraka_attack_3")
                        | Sound::Attack
                )
            })
            .collect();
        assert_eq!(
            whooshes,
            [
                Sound::Naraka("naraka_attack_1"),
                Sound::Naraka("naraka_attack_2"),
                Sound::Naraka("naraka_attack_3")
            ]
        );
        heard.clear();
        for _ in 0..3 {
            app.world_mut()
                .get_mut::<ActionIntent>(player)
                .unwrap()
                .request_jump();
            tick(&mut app);
            heard.extend(sample(&app, &mut state));
        }
        for _ in 0..100 {
            tick(&mut app);
            heard.extend(sample(&app, &mut state));
        }
        assert_eq!(
            heard
                .iter()
                .filter(|sound| **sound == Sound::Naraka("naraka_jump"))
                .count(),
            2
        );
        assert_eq!(
            heard
                .iter()
                .filter(|sound| **sound == Sound::Naraka("naraka_land"))
                .count(),
            1
        );
        assert!(sample(&app, &mut state).is_empty());
    }
}
