use chrono::Utc;
use super::*;
use reqwest::Client;
use serde_json::json;

pub struct PerplexityProvider {
    api_key: String,
    client: Client,
}

impl PerplexityProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for PerplexityProvider {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> Result<ChatResponse, LLMError> {
        let request = json!({
            "model": "llama-3.1-sonar-small-128k-online",
            "messages": messages.iter().map(|m| json!({
                "role": m.role,
                "content": m.content
            })).collect::<Vec<_>>(),
            "temperature": options.temperature,
            "top_p": options.top_p
        });

        let response = self.client
            .post("https://api.perplexity.ai/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            content: response_json["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| LLMError::ParseError("Missing content".to_string()))?
                .to_string(),
            prompt_tokens: response_json["usage"]["prompt_tokens"]
                .as_i64()
                .map(|n| n as i32),
            completion_tokens: response_json["usage"]["completion_tokens"]
                .as_i64()
                .map(|n| n as i32),
            model: "llama-3.1-sonar-small-128k-online".to_string(),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    fn get_models(&self) -> Vec<Value> {
        let current_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string();

        vec![
            json!({
            "name": "llama-3.1-sonar-small-128k-online",
            "model": "llama-3.1-sonar-small-128k-online",
            "modified_at": current_time,
            "size": 8_000_000_000_i64,
            "digest": "perplexity-llama-3-small",
            "details": {
                "parent_model": "",
                "format": "gguf",
                "family": "llama",
                "families": ["llama"],
                "parameter_size": "8B",
                "quantization_level": "Q4_K_M"
            }
        }),
            json!({
            "name": "llama-3.1-sonar-large-128k-online",
            "model": "llama-3.1-sonar-large-128k-online",
            "modified_at": current_time,
            "size": 70_000_000_000_i64,
            "digest": "perplexity-llama-3-large",
            "details": {
                "parent_model": "",
                "format": "gguf",
                "family": "llama",
                "families": ["llama"],
                "parameter_size": "70B",
                "quantization_level": "Q4_K_M"
            }
        }),
            json!({
            "name": "llama-3.1-sonar-huge-128k-online",
            "model": "llama-3.1-sonar-huge-128k-online",
            "modified_at": current_time,
            "size": 405_000_000_000_i64,
            "digest": "perplexity-llama-3-huge",
            "details": {
                "parent_model": "",
                "format": "gguf",
                "family": "llama",
                "families": ["llama"],
                "parameter_size": "405B",
                "quantization_level": "Q4_K_M"
            }
        })
        ]
    }


    async fn get_model_details(&self, model_name: &str) -> Result<Value, LLMError> {
        let response = self.client
            .get("https://api.perplexity.ai/models")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| LLMError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMError::ApiError(error_text));
        }


        // Get parameter size based on model name
        let parameter_size = if model_name.contains("small") {
            "8B"
        } else if model_name.contains("large") {
            "70B"
        } else {
            "405B"
        };

        let parameter_count = if model_name.contains("small") {
            8_000_000_000_i64
        } else if model_name.contains("large") {
            70_000_000_000_i64
        } else {
            405_000_000_000_i64
        };

        let current_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string();

        Ok(json!({
        "license": "Perplexity API License",
        "system": "A chat between a human and an AI assistant",
        "details": {
            "parent_model": "",
            "format": "gguf",
            "family": "llama",
            "families": ["llama"],
            "parameter_size": parameter_size,
            "quantization_level": "Q4_K_M"
        },
        "model_info": {
            "general.architecture": "llama",
            "general.file_type": 15,
            "general.parameter_count": parameter_count,
            "general.context_length": 128000
        },
        "modified_at": current_time
    }))
    }
}