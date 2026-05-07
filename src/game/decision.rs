use serde::Deserialize;

use crate::game::domain::{PlayerId, Role};
use crate::game::rules::DeathReason;
use crate::game::session::GameSession;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AiDecisionKind {
    DaySpeech {
        day: u32,
    },
    WolfChat {
        night: u32,
    },
    WolfKill {
        night: u32,
    },
    WitchMedicine {
        night: u32,
        wolf_target: Option<PlayerId>,
        has_save: bool,
        has_poison: bool,
    },
    HunterShot {
        day: u32,
        death_reason: DeathReason,
    },
    SeerCheck {
        night: u32,
        checked: Vec<PlayerId>,
    },
    Vote {
        day: u32,
    },
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct SpeechDecision {
    pub speech: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct TargetDecision {
    #[serde(deserialize_with = "deserialize_player_id")]
    pub target: PlayerId,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct WitchDecision {
    pub action: WitchActionDecision,
    #[serde(default, deserialize_with = "deserialize_optional_player_id")]
    pub target: Option<PlayerId>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum WitchActionDecision {
    Save,
    Poison,
    Skip,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct HunterDecision {
    pub action: HunterActionDecision,
    #[serde(default, deserialize_with = "deserialize_optional_player_id")]
    pub target: Option<PlayerId>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum HunterActionDecision {
    Shoot,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DecisionError {
    InvalidJson(String),
    EmptySpeech,
    EmptyReason,
    IllegalTarget(PlayerId),
    MedicineUnavailable,
    MissingWolfTarget,
    MissingTarget,
}

#[allow(dead_code)]
pub fn parse_speech_decision(json: &str) -> Result<SpeechDecision, DecisionError> {
    let decision: SpeechDecision =
        serde_json::from_str(json).map_err(|err| DecisionError::InvalidJson(err.to_string()))?;
    if decision.speech.trim().is_empty() {
        return Err(DecisionError::EmptySpeech);
    }
    Ok(decision)
}

#[allow(dead_code)]
pub fn parse_target_decision(json: &str) -> Result<TargetDecision, DecisionError> {
    serde_json::from_str(json).map_err(|err| DecisionError::InvalidJson(err.to_string()))
}

pub fn validate_vote(
    session: &GameSession,
    actor: PlayerId,
    decision: &TargetDecision,
) -> Result<(), DecisionError> {
    require_reason(&decision.reason)?;
    let Some(target) = session.player(decision.target) else {
        return Err(DecisionError::IllegalTarget(decision.target));
    };

    if !target.alive || decision.target == actor {
        return Err(DecisionError::IllegalTarget(decision.target));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_wolf_kill(
    session: &GameSession,
    actor: PlayerId,
    decision: &TargetDecision,
) -> Result<(), DecisionError> {
    require_reason(&decision.reason)?;
    let Some(target) = session.player(decision.target) else {
        return Err(DecisionError::IllegalTarget(decision.target));
    };

    if !target.alive || target.role == Role::Werewolf || decision.target == actor {
        return Err(DecisionError::IllegalTarget(decision.target));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_witch_decision(
    wolf_target: Option<PlayerId>,
    has_save: bool,
    has_poison: bool,
    decision: &WitchDecision,
) -> Result<(), DecisionError> {
    require_reason(&decision.reason)?;
    match decision.action {
        WitchActionDecision::Save => {
            if !has_save {
                return Err(DecisionError::MedicineUnavailable);
            }
            if wolf_target.is_none() {
                return Err(DecisionError::MissingWolfTarget);
            }
        }
        WitchActionDecision::Poison => {
            if !has_poison {
                return Err(DecisionError::MedicineUnavailable);
            }
            if decision.target.is_none() {
                return Err(DecisionError::MissingTarget);
            }
        }
        WitchActionDecision::Skip => {}
    }

    Ok(())
}

#[allow(dead_code)]
fn require_reason(reason: &str) -> Result<(), DecisionError> {
    if reason.trim().is_empty() {
        Err(DecisionError::EmptyReason)
    } else {
        Ok(())
    }
}

fn deserialize_player_id<'de, D>(deserializer: D) -> Result<PlayerId, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let id = usize::deserialize(deserializer)?;
    Ok(PlayerId(id))
}

fn deserialize_optional_player_id<'de, D>(deserializer: D) -> Result<Option<PlayerId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<usize>::deserialize(deserializer).map(|id| id.map(PlayerId))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::domain::{PlayerId, Role};
    use crate::game::session::GameSession;

    #[test]
    fn wolf_kill_rejects_werewolf_target() {
        let session = GameSession::new_with_roles(Role::nine_player_deck());
        let decision = TargetDecision {
            target: PlayerId(2),
            reason: "测试".to_string(),
        };

        assert!(validate_wolf_kill(&session, PlayerId(1), &decision).is_err());
    }

    #[test]
    fn witch_rejects_save_when_no_wolf_target() {
        let decision = WitchDecision {
            action: WitchActionDecision::Save,
            target: None,
            reason: "测试".to_string(),
        };

        assert!(validate_witch_decision(None, true, true, &decision).is_err());
    }

    #[test]
    fn target_decision_parses_numeric_player_id() {
        let decision = parse_target_decision(r#"{"target":4,"reason":"测试"}"#).unwrap();

        assert_eq!(decision.target, PlayerId(4));
        assert_eq!(decision.reason, "测试");
    }
}
