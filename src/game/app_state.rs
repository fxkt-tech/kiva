use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::domain::PlayerId;
use crate::game::events::EventLog;
use crate::game::rules::DeathReason;
use crate::game::session::GameSession;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppScreen {
    #[default]
    Start,
    Game,
    Review,
}

#[derive(Resource, Debug, Default)]
pub struct SessionResource {
    pub session: Option<GameSession>,
}

#[derive(Resource, Debug, Default)]
pub struct SelectedPlayer {
    pub player: Option<PlayerId>,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum InfoTab {
    #[default]
    PublicRecords,
    MyClues,
    MyMarks,
}

#[derive(Resource, Debug, Default)]
#[allow(dead_code)]
pub struct ActiveInfoTab {
    pub tab: InfoTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum TrustMark {
    Trusted,
    #[default]
    Neutral,
    Suspicious,
}

#[derive(Resource, Debug, Default)]
#[allow(dead_code)]
pub struct PlayerMarks {
    pub trust: HashMap<PlayerId, TrustMark>,
    pub notes: HashMap<PlayerId, String>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct PendingInput {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum WitchIntent {
    #[default]
    None,
    Save,
    Poison,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct PlayerAction {
    pub selected_target: Option<PlayerId>,
    pub witch_intent: WitchIntent,
    pub hunter_shot_pending: bool,
    pub hunter_actor: Option<PlayerId>,
    pub hunter_death_reason: Option<DeathReason>,
}

#[derive(Resource, Debug, Default)]
pub struct NeedsGameRedraw {
    pub value: bool,
}

#[derive(Resource, Debug, Default)]
pub struct AiTaskState {
    pub active: bool,
}

#[derive(Resource, Debug)]
pub struct SpeechPlayback {
    pub queue: Vec<PlayerId>,
    pub current_speaker: Option<PlayerId>,
    pub timer: Timer,
    pub active: bool,
    pub thinking: bool,
}

impl Default for SpeechPlayback {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            current_speaker: None,
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            active: false,
            thinking: false,
        }
    }
}

impl SpeechPlayback {
    pub fn reset(&mut self) {
        self.queue.clear();
        self.current_speaker = None;
        self.timer.reset();
        self.active = false;
        self.thinking = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPhase {
    Night,
    DaySpeech,
    Vote,
    HunterShot,
    Review,
}

#[derive(Resource, Debug, Clone)]
#[allow(dead_code)]
pub struct FlowState {
    pub day: u32,
    pub phase: FlowPhase,
    pub event_log: EventLog,
    pub my_clues: Vec<String>,
    pub winner: Option<String>,
}

impl FlowState {
    #[allow(dead_code)]
    pub fn public_records(&self) -> Vec<String> {
        self.event_log.public_projection()
    }
}

impl Default for FlowState {
    fn default() -> Self {
        Self {
            day: 1,
            phase: FlowPhase::Night,
            event_log: EventLog::default(),
            my_clues: Vec::new(),
            winner: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::events::{EventVisibility, GameEvent};

    #[test]
    fn public_records_are_projected_from_event_log() {
        let mut flow = FlowState::default();
        flow.event_log.append(
            GameEvent::PlayerExiled {
                day: 1,
                player: PlayerId(2),
            },
            EventVisibility::Public,
        );

        assert_eq!(
            flow.public_records(),
            vec!["[DAY 1 EXILE] Seat 2 is exiled.".to_string()]
        );
    }
}
