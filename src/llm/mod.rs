pub mod perplexity;
pub mod anthropic;

use serde_json::Value;
use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> Result<ChatResponse, LLMError>;
    fn get_models(&self) -> Vec<Value>;
    async fn get_model_details(&self, model_name: &str) -> Result<Value, LLMError>;
}
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ChatOptions {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: Option<i32>,
}

#[derive(Debug)]
pub struct ChatResponse {
    pub content: String,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum LLMError {
    ApiError(String),
    ParseError(String),
}
