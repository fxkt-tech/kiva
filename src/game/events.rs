use crate::game::app_state::FlowPhase;
use crate::game::domain::{Camp, PlayerId};
use crate::game::rules::Death;
use crate::game::session::Winner;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum GameEvent {
    GameStarted {
        day: u32,
    },
    PhaseStarted {
        phase: FlowPhase,
        day: u32,
    },
    DaySpeech {
        day: u32,
        speaker: PlayerId,
        content: String,
    },
    VoteCast {
        day: u32,
        voter: PlayerId,
        target: PlayerId,
        reason: String,
    },
    PlayerExiled {
        day: u32,
        player: PlayerId,
    },
    WolfChat {
        night: u32,
        speaker: PlayerId,
        content: String,
    },
    WolfKillChosen {
        night: u32,
        actor: PlayerId,
        target: PlayerId,
        reason: String,
    },
    SeerChecked {
        night: u32,
        seer: PlayerId,
        target: PlayerId,
        camp: Camp,
    },
    WitchMedicineUsed {
        night: u32,
        witch: PlayerId,
        action: WitchMedicineAction,
        target: Option<PlayerId>,
        reason: String,
    },
    NightDeaths {
        night: u32,
        deaths: Vec<Death>,
    },
    HunterShot {
        day: u32,
        hunter: PlayerId,
        target: Option<PlayerId>,
        reason: String,
    },
    GameEnded {
        winner: Winner,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum WitchMedicineAction {
    Save,
    Poison,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EventVisibility {
    Public,
    Wolves,
    ActorOnly(PlayerId),
    WerewolfAndActor(PlayerId),
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggedEvent {
    pub sequence: u64,
    pub event: GameEvent,
    pub visibility: EventVisibility,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventLog {
    events: Vec<LoggedEvent>,
    next_sequence: u64,
}

impl EventLog {
    #[allow(dead_code)]
    pub fn append(&mut self, event: GameEvent, visibility: EventVisibility) {
        self.next_sequence += 1;
        self.events.push(LoggedEvent {
            sequence: self.next_sequence,
            event,
            visibility,
        });
    }

    #[allow(dead_code)]
    pub fn events(&self) -> &[LoggedEvent] {
        &self.events
    }

    #[allow(dead_code)]
    pub fn public_projection(&self) -> Vec<String> {
        self.events
            .iter()
            .filter(|event| event.visibility == EventVisibility::Public)
            .map(format_public_event)
            .collect()
    }
}

#[allow(dead_code)]
fn format_public_event(logged: &LoggedEvent) -> String {
    match &logged.event {
        GameEvent::GameStarted { day } => format!("第 {day} 夜：游戏开始。"),
        GameEvent::PhaseStarted { phase, day } => format!("第 {day} 天：进入{}。", phase.label()),
        GameEvent::DaySpeech {
            day,
            speaker,
            content,
        } => format!("第 {day} 天：{} 号发言：{content}", speaker.0),
        GameEvent::VoteCast {
            day,
            voter,
            target,
            reason,
        } => format!("第 {day} 天：{} 号投票给 {} 号：{reason}", voter.0, target.0),
        GameEvent::PlayerExiled { day, player } => format!("第 {day} 天：{} 号被放逐。", player.0),
        GameEvent::NightDeaths { night, deaths } if deaths.is_empty() => {
            format!("第 {night} 夜：平安夜。")
        }
        GameEvent::NightDeaths { night, deaths } => {
            let deaths = deaths
                .iter()
                .map(|death| format!("{} 号", death.player.0))
                .collect::<Vec<_>>()
                .join("、");
            format!("第 {night} 夜：{deaths}死亡。")
        }
        GameEvent::HunterShot {
            hunter,
            target: Some(target),
            ..
        } => format!("猎人 {} 号开枪带走 {} 号。", hunter.0, target.0),
        GameEvent::HunterShot {
            hunter,
            target: None,
            ..
        } => format!("猎人 {} 号不开枪。", hunter.0),
        GameEvent::GameEnded { winner } => match winner {
            Winner::Good => "游戏结束：好人胜利。".to_string(),
            Winner::Werewolf => "游戏结束：狼人胜利。".to_string(),
        },
        GameEvent::WolfChat { .. }
        | GameEvent::WolfKillChosen { .. }
        | GameEvent::SeerChecked { .. }
        | GameEvent::WitchMedicineUsed { .. } => String::new(),
    }
}

impl FlowPhase {
    #[allow(dead_code)]
    fn label(self) -> &'static str {
        match self {
            FlowPhase::Night => "夜晚",
            FlowPhase::DaySpeech => "白天发言",
            FlowPhase::Vote => "投票",
            FlowPhase::HunterShot => "猎人开枪",
            FlowPhase::Review => "复盘",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_log_assigns_monotonic_sequence_numbers() {
        let mut log = EventLog::default();

        log.append(GameEvent::GameStarted { day: 1 }, EventVisibility::Public);
        log.append(
            GameEvent::PhaseStarted {
                phase: FlowPhase::Night,
                day: 1,
            },
            EventVisibility::Public,
        );

        assert_eq!(log.events()[0].sequence, 1);
        assert_eq!(log.events()[1].sequence, 2);
    }

    #[test]
    fn public_projection_formats_day_speech() {
        let mut log = EventLog::default();
        log.append(
            GameEvent::DaySpeech {
                day: 1,
                speaker: PlayerId(3),
                content: "我先听后置位。".to_string(),
            },
            EventVisibility::Public,
        );

        assert_eq!(
            log.public_projection(),
            vec!["第 1 天：3 号发言：我先听后置位。"]
        );
    }
}
