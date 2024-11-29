use chrono::Utc;
use super::*;
use reqwest::Client;
use serde_json::json;

pub struct AnthropicProvider {
    api_key: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> Result<ChatResponse, LLMError> {
        // Extract system message if present
        let (system_messages, chat_messages): (Vec<Message>, Vec<Message>) = messages.into_iter()
            .partition(|m| m.role == "system");

        // Get the first system message content or empty string
        let system_content = system_messages.first()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let request = json!({
            "model": "claude-3-5-sonnet-20241022",
            "system": system_content,
            "messages": chat_messages.iter().map(|m| json!({
                "role": m.role,
                "content": m.content
            })).collect::<Vec<_>>(),
            "temperature": options.temperature,
            "top_p": options.top_p,
            "max_tokens": options.max_tokens.unwrap_or(4096)
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMError::ApiError(error_text));
        }

        let response_json: Value = response.json()
            .await
            .map_err(|e| LLMError::ParseError(e.to_string()))?;

        Ok(ChatResponse {
            content: response_json["content"][0]["text"]
                .as_str()
                .ok_or_else(|| LLMError::ParseError("Missing content".to_string()))?
                .to_string(),
            prompt_tokens: response_json["usage"]["input_tokens"]
                .as_i64()
                .map(|n| n as i32),
            completion_tokens: response_json["usage"]["output_tokens"]
                .as_i64()
                .map(|n| n as i32),
            model: "claude-3-5-sonnet-20241022".to_string(),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    fn get_models(&self) -> Vec<Value> {
        let current_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string();

        vec![
            json!({
            "name": "claude-3-5-sonnet-20241022",
            "model": "claude-3-5-sonnet-20241022",
            "modified_at": current_time,
            "size": 175_000_000_000_i64,
            "digest": "anthropic-claude-3-5-sonnet",
            "details": {
                "parent_model": "",
                "format": "gguf",
                "family": "claude",
                "families": ["claude"],
                "parameter_size": "175B",
                "quantization_level": "Q4_K_M"
            }
        }),
            json!({
            "name": "claude-3-5-haiku-20241022",
            "model": "claude-3-5-haiku-20241022",
            "modified_at": current_time,
            "size": 175_000_000_000_i64,
            "digest": "anthropic-claude-3-5-haiku",
            "details": {
                "parent_model": "",
                "format": "gguf",
                "family": "claude",
                "families": ["claude"],
                "parameter_size": "175B",
                "quantization_level": "Q4_K_M"
            }
        }),
            json!({
            "name": "claude-3-opus-20240229",
            "model": "claude-3-opus-20240229",
            "modified_at": current_time,
            "size": 175_000_000_000_i64,
            "digest": "anthropic-claude-3-opus",
            "details": {
                "parent_model": "",
                "format": "gguf",
                "family": "claude",
                "families": ["claude"],
                "parameter_size": "175B",
                "quantization_level": "Q4_K_M"
            }
        })
        ]
    }
    async fn get_model_details(&self, model_name: &str) -> Result<Value, LLMError> {
        let current_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string();

        let (description, parameter_size) = match model_name {
            "claude-3-5-sonnet-20241022" => (
                "Our most intelligent model",
                "200B"
            ),
            "claude-3-5-haiku-20241022" => (
                "Our fastest model",
                "100B"
            ),
            "claude-3-opus-20240229" => (
                "Powerful model for highly complex tasks",
                "400B"
            ),
            _ => return Err(LLMError::ParseError("Model not found".to_string()))
        };

        Ok(json!({
        "license": "Anthropic Research License",
        "system": description,
        "details": {
            "parent_model": "",
            "format": "gguf",
            "family": "claude",
            "families": ["claude"],
            "parameter_size": parameter_size,
            "quantization_level": "Q4_K_M"
        },
        "model_info": {
            "general.architecture": "claude",
            "general.file_type": 15,
            "general.context_length": 200000,
            "general.parameter_count": 200_000_000_000_i64
        },
        "modified_at": current_time
    }))
    }

}