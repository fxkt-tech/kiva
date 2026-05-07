use crate::game::domain::PlayerAiConfig;

#[allow(dead_code)]
pub trait LlmClient {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmRequest {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system: String,
    pub user: String,
    pub temperature: f32,
    pub response_format: LlmResponseFormat,
}

impl LlmRequest {
    #[allow(dead_code)]
    pub fn new(config: &PlayerAiConfig, user: String) -> Result<Self, LlmError> {
        if config.base_url.trim().is_empty()
            || config.api_key.trim().is_empty()
            || config.model.trim().is_empty()
        {
            return Err(LlmError::NotConfigured);
        }

        Ok(Self {
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            system: config.system_prompt.clone(),
            user,
            temperature: 0.7,
            response_format: LlmResponseFormat::JsonObject,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LlmResponseFormat {
    Text,
    JsonObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct LlmResponse {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LlmError {
    NotConfigured,
    RequestFailed(String),
    InvalidResponse(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::domain::PlayerAiConfig;

    #[test]
    fn request_from_unconfigured_player_returns_not_configured() {
        let config = PlayerAiConfig {
            base_url: "".to_string(),
            api_key: "".to_string(),
            model: "".to_string(),
            system_prompt: "你是测试玩家。".to_string(),
        };

        let err = LlmRequest::new(&config, "user prompt".to_string()).unwrap_err();

        assert_eq!(err, LlmError::NotConfigured);
    }
}
