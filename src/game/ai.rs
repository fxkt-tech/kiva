use crate::game::decision::{
    AiDecisionKind, TargetDecision, parse_speech_decision, parse_target_decision, validate_vote,
};
use crate::game::domain::{PlayerId, Role};
use crate::game::events::{EventLog, EventVisibility, GameEvent};
use crate::game::llm::{LlmClient, LlmError, LlmRequest};
use crate::game::prompt::build_prompt;
use crate::game::rules::VoteCast;
use crate::game::session::GameSession;

#[derive(Debug, Default)]
pub struct NoopLlmClient;

impl LlmClient for NoopLlmClient {
    fn complete(&self, _request: LlmRequest) -> Result<crate::game::llm::LlmResponse, LlmError> {
        Err(LlmError::NotConfigured)
    }
}

pub fn choose_wolf_target(session: &GameSession, actor: PlayerId) -> Option<PlayerId> {
    let actor_role = session.player(actor).map(|player| player.role);

    session
        .alive_players()
        .find(|player| Some(player.role) != actor_role && player.role != Role::Werewolf)
        .map(|player| player.id)
}

#[allow(dead_code)]
pub fn choose_seer_target(session: &GameSession, checked: &[PlayerId]) -> Option<PlayerId> {
    session
        .alive_players()
        .find(|player| !checked.contains(&player.id))
        .map(|player| player.id)
}

pub fn choose_vote_target(session: &GameSession, actor: PlayerId) -> Option<PlayerId> {
    session
        .alive_players()
        .find(|player| player.id != actor)
        .map(|player| player.id)
}

#[allow(dead_code)]
pub fn generate_speech(session: &GameSession, actor: PlayerId, day: u32) -> String {
    let Some(player) = session.player(actor) else {
        return "我先过。".to_string();
    };

    let speech = match player.role {
        Role::Werewolf => "今天先听发言，不要太早定票。".to_string(),
        Role::Seer => format!("第 {day} 天我会重点看投票和站边。"),
        Role::Witch => "我先不跳身份，大家把怀疑点聊清楚。".to_string(),
        Role::Hunter => "我会看谁在强行带节奏。".to_string(),
        Role::Villager => "目前信息还少，我先听后面的发言。".to_string(),
    };

    format!(
        "{} 号 / {} / {}：{}",
        actor.0,
        player.name,
        player.role.label(),
        speech
    )
}

pub fn generate_day_speech(
    client: &impl LlmClient,
    session: &GameSession,
    log: &mut EventLog,
    actor: PlayerId,
    day: u32,
) -> Result<(), LlmError> {
    let speech = match build_day_speech_with_llm(client, session, log, actor, day) {
        Ok(speech) => speech,
        Err(LlmError::NotConfigured) => fallback_speech_content(session, actor, day),
        Err(err) => return Err(err),
    };

    log.append(
        GameEvent::DaySpeech {
            day,
            speaker: actor,
            content: speech,
        },
        EventVisibility::Public,
    );
    Ok(())
}

pub fn generate_vote(
    client: &impl LlmClient,
    session: &GameSession,
    log: &mut EventLog,
    actor: PlayerId,
    day: u32,
) -> Result<VoteCast, LlmError> {
    let decision = match build_vote_with_llm(client, session, log, actor, day) {
        Ok(decision) => decision,
        Err(LlmError::NotConfigured) => fallback_vote_decision(session, actor),
        Err(err) => return Err(err),
    };

    log.append(
        GameEvent::VoteCast {
            day,
            voter: actor,
            target: decision.target,
            reason: decision.reason.clone(),
        },
        EventVisibility::Public,
    );

    Ok(VoteCast {
        voter: actor,
        target: decision.target,
        reason: decision.reason,
    })
}

fn build_vote_with_llm(
    client: &impl LlmClient,
    session: &GameSession,
    log: &EventLog,
    actor: PlayerId,
    day: u32,
) -> Result<TargetDecision, LlmError> {
    let Some(player) = session.player(actor) else {
        return Ok(fallback_vote_decision(session, actor));
    };

    let prompt = build_prompt(session, actor, AiDecisionKind::Vote { day }, log);
    let request = LlmRequest::new(&player.ai, prompt.user)?;
    let response = client.complete(LlmRequest {
        system: prompt.system,
        ..request
    })?;
    let decision = parse_target_decision(&response.content)
        .map_err(|err| LlmError::InvalidResponse(format!("{err:?}")))?;
    validate_vote(session, actor, &decision)
        .map_err(|err| LlmError::InvalidResponse(format!("{err:?}")))?;

    Ok(decision)
}

fn build_day_speech_with_llm(
    client: &impl LlmClient,
    session: &GameSession,
    log: &EventLog,
    actor: PlayerId,
    day: u32,
) -> Result<String, LlmError> {
    let Some(player) = session.player(actor) else {
        return Ok("我先过。".to_string());
    };

    let prompt = build_prompt(session, actor, AiDecisionKind::DaySpeech { day }, log);
    let request = LlmRequest::new(&player.ai, prompt.user)?;
    let response = client.complete(LlmRequest {
        system: prompt.system,
        ..request
    })?;
    let decision = parse_speech_decision(&response.content)
        .map_err(|err| LlmError::InvalidResponse(format!("{err:?}")))?;

    Ok(decision.speech)
}

fn fallback_speech_content(session: &GameSession, actor: PlayerId, day: u32) -> String {
    let Some(player) = session.player(actor) else {
        return "我先过。".to_string();
    };

    match player.role {
        Role::Werewolf => "今天先听发言，不要太早定票。".to_string(),
        Role::Seer => format!("第 {day} 天我会重点看投票和站边。"),
        Role::Witch => "我先不跳身份，大家把怀疑点聊清楚。".to_string(),
        Role::Hunter => "我会看谁在强行带节奏。".to_string(),
        Role::Villager => "目前信息还少，我先听后面的发言。".to_string(),
    }
}

fn fallback_vote_decision(session: &GameSession, actor: PlayerId) -> TargetDecision {
    let target = choose_vote_target(session, actor).unwrap_or(actor);
    TargetDecision {
        target,
        reason: "未配置 LLM，使用默认投票。".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::domain::{PlayerId, Role};
    use crate::game::events::{EventLog, GameEvent};
    use crate::game::llm::{LlmError, LlmRequest, LlmResponse};
    use crate::game::session::GameSession;

    struct FakeLlmClient {
        response: &'static str,
    }

    impl crate::game::llm::LlmClient for FakeLlmClient {
        fn complete(&self, _request: LlmRequest) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                content: self.response.to_string(),
            })
        }
    }

    #[test]
    fn wolf_target_is_living_non_wolf() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());

        let target = choose_wolf_target(&session, PlayerId(1)).unwrap();

        let player = session.player(target).unwrap();
        assert!(player.alive);
        assert_ne!(player.role, Role::Werewolf);
    }

    #[test]
    fn seer_target_is_living_and_unchecked() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());
        let checked = vec![PlayerId(4), PlayerId(5)];

        let target = choose_seer_target(&session, &checked).unwrap();

        assert!(session.player(target).unwrap().alive);
        assert!(!checked.contains(&target));
    }

    #[test]
    fn vote_target_is_living_and_not_self() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());

        let target = choose_vote_target(&session, PlayerId(2)).unwrap();

        assert!(session.player(target).unwrap().alive);
        assert_ne!(target, PlayerId(2));
    }

    #[test]
    fn speech_does_not_reveal_hidden_roles_for_villager() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());

        let speech = generate_speech(&session, PlayerId(9), 1);

        assert!(!speech.contains("1号是狼人"));
        assert!(!speech.contains("1 号是狼人"));
        assert!(speech.contains("9 号 / AI-09 /"));
    }

    #[test]
    fn day_speech_uses_fake_llm_and_appends_typed_event() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());
        let actor = PlayerId(1);
        let player = session.player_mut(actor).unwrap();
        player.ai.base_url = "https://example.test/v1".to_string();
        player.ai.api_key = "test-key".to_string();
        player.ai.model = "test-model".to_string();
        let mut log = EventLog::default();
        let client = FakeLlmClient {
            response: r#"{"speech":"我先听后置位。"}"#,
        };

        generate_day_speech(&client, &session, &mut log, actor, 1).unwrap();

        assert!(matches!(
            &log.events()[0].event,
            GameEvent::DaySpeech { speaker: PlayerId(1), content, .. } if content == "我先听后置位。"
        ));
    }

    #[test]
    fn day_speech_falls_back_when_player_llm_is_unconfigured() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());
        let mut log = EventLog::default();
        let client = NoopLlmClient;

        generate_day_speech(&client, &session, &mut log, PlayerId(9), 1).unwrap();

        assert!(matches!(
            &log.events()[0].event,
            GameEvent::DaySpeech { speaker: PlayerId(9), content, .. } if content.contains("信息还少")
        ));
    }

    #[test]
    fn vote_uses_fake_llm_and_appends_typed_event() {
        let mut session = GameSession::new_with_roles(Role::nine_player_deck());
        let actor = PlayerId(1);
        let player = session.player_mut(actor).unwrap();
        player.ai.base_url = "https://example.test/v1".to_string();
        player.ai.api_key = "test-key".to_string();
        player.ai.model = "test-model".to_string();
        let mut log = EventLog::default();
        let client = FakeLlmClient {
            response: r#"{"target":4,"reason":"发言可疑。"}"#,
        };

        let vote = generate_vote(&client, &session, &mut log, actor, 1).unwrap();

        assert_eq!(vote.target, PlayerId(4));
        assert!(matches!(
            &log.events()[0].event,
            GameEvent::VoteCast { voter: PlayerId(1), target: PlayerId(4), .. }
        ));
    }
}
