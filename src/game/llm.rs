#[cfg(not(target_arch = "wasm32"))]
use async_openai::Client;
#[cfg(not(target_arch = "wasm32"))]
use async_openai::config::OpenAIConfig;
#[cfg(not(target_arch = "wasm32"))]
use async_openai::types::chat::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
    CreateChatCompletionRequestArgs, ResponseFormat,
};

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

#[derive(Debug, Default)]
pub struct OpenAiCompatibleClient;

impl LlmClient for OpenAiCompatibleClient {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = request;
            Err(LlmError::RequestFailed(
                "synchronous LLM calls are unavailable in browsers".into(),
            ))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|err| LlmError::RequestFailed(err.to_string()))?;

            runtime.block_on(self.complete_async(request))
        }
    }
}

impl OpenAiCompatibleClient {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn complete_async(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let chat_request = build_chat_request(&request)?;
        let config = OpenAIConfig::new()
            .with_api_key(request.api_key)
            .with_api_base(request.base_url.trim_end_matches('/'));
        let client = Client::with_config(config);

        let response = client
            .chat()
            .create(chat_request)
            .await
            .map_err(|err| LlmError::RequestFailed(err.to_string()))?;

        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| LlmError::InvalidResponse("missing completion content".to_string()))?;

        Ok(LlmResponse { content })
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn complete_async(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        // Always use the same-origin proxy; never send credentials to the browser.
        let body = serialize_chat_request(&request)?;
        let response = gloo_net::http::Request::post("/api/chat/completions")
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|err| LlmError::RequestFailed(err.to_string()))?
            .send()
            .await
            .map_err(|err| LlmError::RequestFailed(err.to_string()))?;
        if !response.ok() {
            return Err(LlmError::RequestFailed(format!(
                "LLM proxy returned HTTP {}",
                response.status()
            )));
        }
        let value = response
            .json::<serde_json::Value>()
            .await
            .map_err(|err| LlmError::InvalidResponse(err.to_string()))?;
        let content = value["choices"][0]["message"]["content"]
            .as_str()
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| LlmError::InvalidResponse("missing completion content".into()))?;
        Ok(LlmResponse {
            content: content.into(),
        })
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn serialize_chat_request(request: &LlmRequest) -> Result<String, LlmError> {
    let mut body = serde_json::json!({
        "model": request.model,
        "messages": [
            {"role": "system", "content": request.system},
            {"role": "user", "content": request.user},
        ],
        "temperature": request.temperature,
    });
    if request.response_format == LlmResponseFormat::JsonObject {
        body["response_format"] = serde_json::json!({"type": "json_object"});
    }
    serde_json::to_string(&body).map_err(|err| LlmError::InvalidResponse(err.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn build_chat_request(request: &LlmRequest) -> Result<CreateChatCompletionRequest, LlmError> {
    let system = ChatCompletionRequestSystemMessageArgs::default()
        .content(request.system.clone())
        .build()
        .map_err(|err| LlmError::InvalidResponse(err.to_string()))?;
    let user = ChatCompletionRequestUserMessageArgs::default()
        .content(request.user.clone())
        .build()
        .map_err(|err| LlmError::InvalidResponse(err.to_string()))?;

    let mut builder = CreateChatCompletionRequestArgs::default();
    builder
        .model(request.model.clone())
        .messages(vec![
            ChatCompletionRequestMessage::System(system),
            ChatCompletionRequestMessage::User(user),
        ])
        .temperature(request.temperature);

    if request.response_format == LlmResponseFormat::JsonObject {
        builder.response_format(ResponseFormat::JsonObject);
    }

    builder
        .build()
        .map_err(|err| LlmError::InvalidResponse(err.to_string()))
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
            system_prompt: "test player".to_string(),
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
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
            serde_json::to_value(build_chat_request(&request).unwrap()).unwrap(),
        );

        assert!(body.contains("\"model\":\"model\""));
        assert!(body.contains("\"role\":\"system\""));
        assert!(body.contains("\"role\":\"user\""));
        assert!(body.contains("\"type\":\"json_object\""));
    }
}
