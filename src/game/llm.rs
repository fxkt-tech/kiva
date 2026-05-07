use crate::game::domain::PlayerAiConfig;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Default)]
pub struct OpenAiCompatibleClient;

impl LlmClient for OpenAiCompatibleClient {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/chat/completions", request.base_url.trim_end_matches('/'));
        let body = serialize_chat_request(&request)?;
        let value: ChatCompletionResponse = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", request.api_key))
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|err| LlmError::RequestFailed(err.to_string()))?
            .into_json()
            .map_err(|err| LlmError::InvalidResponse(err.to_string()))?;

        let content = value
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| LlmError::InvalidResponse("missing completion content".to_string()))?;

        Ok(LlmResponse { content })
    }
}

pub fn serialize_chat_request(request: &LlmRequest) -> Result<String, LlmError> {
    let body = ChatCompletionRequest {
        model: &request.model,
        messages: vec![
            ChatMessage {
                role: "system",
                content: &request.system,
            },
            ChatMessage {
                role: "user",
                content: &request.user,
            },
        ],
        temperature: request.temperature,
        response_format: match request.response_format {
            LlmResponseFormat::Text => None,
            LlmResponseFormat::JsonObject => Some(ResponseFormat {
                format_type: "json_object",
            }),
        },
    };

    serde_json::to_string(&body).map_err(|err| LlmError::InvalidResponse(err.to_string()))
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat<'a>>,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct ResponseFormat<'a> {
    #[serde(rename = "type")]
    format_type: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
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

    #[test]
    fn chat_request_serializes_system_and_user_messages() {
        let request = LlmRequest {
            base_url: "https://example.test/v1".to_string(),
            api_key: "key".to_string(),
            model: "model".to_string(),
            system: "system".to_string(),
            user: "user".to_string(),
            temperature: 0.7,
            response_format: LlmResponseFormat::JsonObject,
        };

        let body = serialize_chat_request(&request).unwrap();

        assert!(body.contains("\"model\":\"model\""));
        assert!(body.contains("\"role\":\"system\""));
        assert!(body.contains("\"role\":\"user\""));
        assert!(body.contains("\"type\":\"json_object\""));
    }
}
