//! Sound consumes committed combat feedback and never schedules gameplay.
use bevy::prelude::*;

use crate::{
    gameplay::Player,
    simulation::{CombatEffect, CombatEvent, CombatFeedback, SimulationActive},
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaAudio"], js_name = playSniperShot)]
    fn browser_shot() -> bool;
    #[wasm_bindgen(js_namespace = ["window", "kivaAudio"], js_name = playEffect)]
    fn browser_effect(name: &str) -> bool;
    #[wasm_bindgen(js_namespace = ["window", "kivaAudio"], js_name = setActive)]
    fn browser_active(active: bool);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sound {
    Attack,
    Hit,
    Deflect,
    Shot,
}

pub(super) fn sync(
    feedback: Res<CombatFeedback>,
    active: Res<SimulationActive>,
    players: Query<Entity, With<Player>>,
    mut cursor: Local<u64>,
) {
    #[cfg(target_arch = "wasm32")]
    browser_active(active.0);
    for event in feedback.events() {
        if let Some(sound) = consume(&mut cursor, event, active.0, players.contains(event.actor)) {
            play(sound);
        }
    }
}

fn consume(cursor: &mut u64, event: &CombatEvent, active: bool, is_player: bool) -> Option<Sound> {
    if event.sequence <= *cursor {
        return None;
    }
    // Paused events advance the cursor too; resume never replays old attacks.
    *cursor = event.sequence;
    if !active {
        return None;
    }
    match event.kind {
        CombatEffect::Melee | CombatEffect::Hammer | CombatEffect::Spin => Some(Sound::Attack),
        CombatEffect::Hit => Some(Sound::Hit),
        CombatEffect::Deflect => Some(Sound::Deflect),
        CombatEffect::Shot if is_player => Some(Sound::Shot),
        // A leap launch has distinct origin/end. Ordinary cannon impact emits
        // Cannon at end/end after Shot; it must not create a second gunshot.
        CombatEffect::Cannon if is_player && event.position != event.end => Some(Sound::Shot),
        // SniperShot accompanies an ordinary Shot; do not play the same gun twice.
        _ => None,
    }
}

fn play(sound: Sound) {
    #[cfg(target_arch = "wasm32")]
    match sound {
        Sound::Shot => {
            let _ = browser_shot();
        }
        Sound::Attack => {
            let _ = browser_effect("attack");
        }
        Sound::Hit => {
            let _ = browser_effect("hit");
        }
        Sound::Deflect => {
            let _ = browser_effect("deflect");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = sound;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(sequence: u64, kind: CombatEffect) -> CombatEvent {
        CombatEvent {
            sequence,
            actor: Entity::PLACEHOLDER,
            kind,
            position: Vec3::ZERO,
            end: Vec3::Z,
            radius: 0.0,
        }
    }

    #[test]
    fn sounds_follow_actual_feedback_once_for_both_sides_of_combat() {
        let mut cursor = 0;
        for (sequence, kind, player, expected) in [
            (1, CombatEffect::EnemyWindup, false, None),
            (2, CombatEffect::Melee, false, Some(Sound::Attack)),
            (3, CombatEffect::Hit, true, Some(Sound::Hit)),
            (4, CombatEffect::Deflect, true, Some(Sound::Deflect)),
            (5, CombatEffect::Shot, true, Some(Sound::Shot)),
            (6, CombatEffect::SniperShot, true, None),
        ] {
            let event = event(sequence, kind);
            assert_eq!(consume(&mut cursor, &event, true, player), expected);
            assert_eq!(consume(&mut cursor, &event, true, player), None);
        }
    }

    #[test]
    fn paused_feedback_and_older_history_cannot_replay_on_resume() {
        let mut cursor = 0;
        let hit = event(143, CombatEffect::Deflect);
        assert_eq!(consume(&mut cursor, &hit, false, true), None);
        assert_eq!(cursor, 143);
        assert_eq!(consume(&mut cursor, &hit, true, true), None);
        assert_eq!(
            consume(&mut cursor, &event(142, CombatEffect::Melee), true, true),
            None
        );
        assert_eq!(
            consume(&mut cursor, &event(144, CombatEffect::Melee), true, true),
            Some(Sound::Attack)
        );
    }

    #[test]
    fn one_actual_cannon_attack_plays_one_shot_despite_its_splash_feedback() {
        use crate::simulation::{CombatIntent, Combatant, SimulationPlugin, WeaponSet};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SimulationPlugin));
        app.finish();
        crate::gameplay::setup(app.world_mut());
        for _ in 0..12 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .entity_mut(player)
            .insert(Combatant::new(WeaponSet::AxeCannon));
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_switch_weapon();
        app.world_mut().run_schedule(FixedUpdate);
        app.world_mut()
            .get_mut::<CombatIntent>(player)
            .unwrap()
            .request_attack();
        app.world_mut().run_schedule(FixedUpdate);
        let feedback = app.world().resource::<CombatFeedback>();
        assert!(
            feedback
                .events()
                .any(|event| event.kind == CombatEffect::Cannon)
        );
        let mut cursor = 0;
        let shots = feedback
            .events()
            .filter(|event| {
                consume(&mut cursor, event, true, event.actor == player) == Some(Sound::Shot)
            })
            .count();
        assert_eq!(shots, 1);
    }
}
