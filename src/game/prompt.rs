use crate::game::decision::AiDecisionKind;
use crate::game::domain::PlayerId;
use crate::game::events::{GameEvent, LoggedEvent};
use crate::game::session::GameSession;
use crate::game::visibility::visible_events;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptParts {
    pub system: String,
    pub user: String,
}

#[allow(dead_code)]
pub fn build_prompt(
    session: &GameSession,
    actor: PlayerId,
    decision_kind: AiDecisionKind,
    log: &crate::game::events::EventLog,
) -> PromptParts {
    let Some(player) = session.player(actor) else {
        return PromptParts {
            system: "你是狼人杀玩家。".to_string(),
            user: "当前玩家不存在，返回合法 JSON。".to_string(),
        };
    };

    let visible_records = visible_events(session, actor, log)
        .iter()
        .map(|event| summarize_event(event))
        .filter(|summary| !summary.is_empty())
        .collect::<Vec<_>>();

    PromptParts {
        system: player.ai.system_prompt.clone(),
        user: format!(
            "你是 {} 号玩家 {}，身份是 {}，当前状态：{}。\n\
            当前动作：{}。\n\
            你只能参考以下可见记录：\n{}\n\
            输出要求：{}",
            actor.0,
            player.name,
            player.role.label(),
            if player.alive { "存活" } else { "死亡" },
            decision_label(&decision_kind),
            if visible_records.is_empty() {
                "无".to_string()
            } else {
                visible_records.join("\n")
            },
            output_schema_hint(&decision_kind)
        ),
    }
}

fn summarize_event(logged: &LoggedEvent) -> String {
    match &logged.event {
        GameEvent::GameStarted { day } => format!("第 {day} 夜：游戏开始。"),
        GameEvent::PhaseStarted { phase, day } => format!("第 {day} 天：进入 {:?}。", phase),
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
        GameEvent::WolfChat {
            night,
            speaker,
            content,
        } => format!("第 {night} 夜狼人讨论：{} 号说：{content}", speaker.0),
        GameEvent::WolfKillChosen {
            night,
            actor,
            target,
            reason,
        } => format!("第 {night} 夜：狼人 {} 号选择刀 {} 号：{reason}", actor.0, target.0),
        GameEvent::SeerChecked {
            night,
            target,
            camp,
            ..
        } => format!("第 {night} 夜：你查验 {} 号，结果为 {:?}。", target.0, camp),
        GameEvent::WitchMedicineUsed {
            night,
            action,
            target,
            reason,
            ..
        } => format!("第 {night} 夜：你选择 {:?} {:?}：{reason}", action, target.map(|id| id.0)),
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
            day,
            hunter,
            target,
            reason,
        } => match target {
            Some(target) => format!(
                "第 {day} 天：猎人 {} 号开枪带走 {} 号：{reason}",
                hunter.0, target.0
            ),
            None => format!("第 {day} 天：猎人 {} 号不开枪：{reason}", hunter.0),
        },
        GameEvent::GameEnded { winner } => format!("游戏结束：{:?}。", winner),
    }
}

fn decision_label(kind: &AiDecisionKind) -> &'static str {
    match kind {
        AiDecisionKind::DaySpeech { .. } => "白天发言",
        AiDecisionKind::WolfChat { .. } => "狼人夜晚讨论",
        AiDecisionKind::WolfKill { .. } => "狼人刀人",
        AiDecisionKind::WitchMedicine { .. } => "女巫用药",
        AiDecisionKind::HunterShot { .. } => "猎人开枪",
        AiDecisionKind::SeerCheck { .. } => "预言家验人",
        AiDecisionKind::Vote { .. } => "白天投票",
    }
}

fn output_schema_hint(kind: &AiDecisionKind) -> &'static str {
    match kind {
        AiDecisionKind::DaySpeech { .. } | AiDecisionKind::WolfChat { .. } => {
            r#"只输出 JSON：{"speech":"..."}"#
        }
        AiDecisionKind::WitchMedicine { .. } => {
            r#"只输出 JSON：{"action":"save|poison|skip","target":数字或null,"reason":"..."}"#
        }
        AiDecisionKind::HunterShot { .. } => {
            r#"只输出 JSON：{"action":"shoot|skip","target":数字或null,"reason":"..."}"#
        }
        AiDecisionKind::WolfKill { .. }
        | AiDecisionKind::SeerCheck { .. }
        | AiDecisionKind::Vote { .. } => r#"只输出 JSON：{"target":数字,"reason":"..."}"#,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::decision::AiDecisionKind;
    use crate::game::domain::{PlayerId, Role};
    use crate::game::events::{EventLog, EventVisibility, GameEvent};
    use crate::game::session::GameSession;

    #[test]
    fn villager_prompt_excludes_wolf_chat() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());
        let mut log = EventLog::default();
        log.append(
            GameEvent::DaySpeech {
                day: 1,
                speaker: PlayerId(7),
                content: "公开发言".to_string(),
            },
            EventVisibility::Public,
        );
        log.append(
            GameEvent::WolfChat {
                night: 1,
                speaker: PlayerId(1),
                content: "私聊刀人".to_string(),
            },
            EventVisibility::Wolves,
        );

        let prompt = build_prompt(&session, PlayerId(7), AiDecisionKind::DaySpeech { day: 1 }, &log);

        assert!(prompt.user.contains("公开发言"));
        assert!(!prompt.user.contains("私聊刀人"));
    }
}
