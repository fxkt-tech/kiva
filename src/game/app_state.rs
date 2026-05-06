use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::domain::{PlayerId, Role};
use crate::game::session::GameSession;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppScreen {
    #[default]
    Start,
    RoleReveal,
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
pub enum InfoTab {
    #[default]
    PublicRecords,
    MyClues,
    MyMarks,
}

#[derive(Resource, Debug, Default)]
pub struct ActiveInfoTab {
    pub tab: InfoTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrustMark {
    Trusted,
    #[default]
    Neutral,
    Suspicious,
}

#[derive(Resource, Debug, Default)]
pub struct PlayerMarks {
    pub trust: HashMap<PlayerId, TrustMark>,
    pub notes: HashMap<PlayerId, String>,
}

#[derive(Resource, Debug, Default)]
pub struct PendingInput {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WitchIntent {
    #[default]
    None,
    Save,
    Poison,
}

#[derive(Resource, Debug, Default)]
pub struct PlayerAction {
    pub selected_target: Option<PlayerId>,
    pub witch_intent: WitchIntent,
    pub hunter_shot_pending: bool,
}

#[derive(Resource, Debug, Default)]
pub struct NeedsGameRedraw {
    pub value: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPhase {
    Night,
    DaySpeech,
    Vote,
    HunterShot,
    Review,
}

#[derive(Resource, Debug)]
pub struct FlowState {
    pub day: u32,
    pub phase: FlowPhase,
    pub public_records: Vec<String>,
    pub my_clues: Vec<String>,
    pub winner: Option<String>,
    pub human_role: Option<Role>,
}

impl Default for FlowState {
    fn default() -> Self {
        Self {
            day: 1,
            phase: FlowPhase::Night,
            public_records: vec!["第 1 夜：游戏开始。".to_string()],
            my_clues: Vec::new(),
            winner: None,
            human_role: None,
        }
    }
}
