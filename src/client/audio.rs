//! Browser sound consumes actual combat feedback; it never schedules combat.

use bevy::prelude::*;

use crate::{
    gameplay::Player,
    simulation::{CombatEffect, CombatEvent, CombatFeedback, SimulationActive},
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaAudio"], js_name = playSniperShot)]
    fn browser_sniper_shot() -> bool;
}

pub(super) fn sync(
    feedback: Res<CombatFeedback>,
    active: Res<SimulationActive>,
    players: Query<Entity, With<Player>>,
    mut cursor: Local<u64>,
) {
    for event in feedback.events() {
        if consume(&mut cursor, event, active.0, players.contains(event.actor)) {
            play_sniper_shot();
        }
    }
}

fn consume(cursor: &mut u64, event: &CombatEvent, active: bool, is_player: bool) -> bool {
    if event.sequence <= *cursor {
        return false;
    }
    // Advance during capture loss too: resuming must not replay paused sounds.
    *cursor = event.sequence;
    active && is_player && event.kind == CombatEffect::SniperShot
}

fn play_sniper_shot() {
    #[cfg(target_arch = "wasm32")]
    let _ = browser_sniper_shot();
    // Native builds deliberately have no browser or platform audio dependency.
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
    fn only_new_actual_player_sniper_shots_play() {
        let mut cursor = 0;
        assert!(!consume(
            &mut cursor,
            &event(1, CombatEffect::Hammer),
            true,
            true
        ));
        assert!(!consume(
            &mut cursor,
            &event(2, CombatEffect::Shot),
            true,
            true
        ));
        assert!(!consume(
            &mut cursor,
            &event(3, CombatEffect::SniperShot),
            true,
            false
        ));
        assert!(consume(
            &mut cursor,
            &event(4, CombatEffect::SniperShot),
            true,
            true
        ));
        assert!(!consume(
            &mut cursor,
            &event(4, CombatEffect::SniperShot),
            true,
            true
        ));
        assert!(!consume(
            &mut cursor,
            &event(5, CombatEffect::SniperImpact),
            true,
            true
        ));
        assert!(consume(
            &mut cursor,
            &event(6, CombatEffect::SniperShot),
            true,
            true
        ));
        assert_eq!(cursor, 6);
    }

    #[test]
    fn paused_feedback_and_older_history_cannot_replay_on_resume() {
        let mut cursor = 0;
        let shot = event(143, CombatEffect::SniperShot);
        assert!(!consume(&mut cursor, &shot, false, true));
        assert_eq!(cursor, 143);
        assert!(!consume(&mut cursor, &shot, true, true));
        assert!(!consume(
            &mut cursor,
            &event(142, CombatEffect::SniperShot),
            true,
            true
        ));
        assert!(consume(
            &mut cursor,
            &event(144, CombatEffect::SniperShot),
            true,
            true
        ));
    }
}
