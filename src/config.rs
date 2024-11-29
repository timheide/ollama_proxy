// src/config.rs
use config::Config;
use std::sync::Arc;
use crate::llm::LLMProvider;
use crate::llm::anthropic::AnthropicProvider;
use crate::llm::perplexity::PerplexityProvider;

pub struct ServerConfig {
    pub host: String,
    pub port: i64,
}

pub fn load_config() -> Config {
    Config::builder()
        .add_source(config::File::with_name("Config"))
        .build()
        .expect("Failed to load config")
}

pub fn create_provider(config: &Config) -> Arc<dyn LLMProvider + Send + Sync> {
    match config.get_string("provider_type")
        .unwrap_or_else(|_| "perplexity".to_string())
        .as_str()
    {
        "anthropic" => {
            let api_key = config.get_string("anthropic_api_key")
                .expect("Anthropic API key not found in config");
            println!("Using Anthropic provider with Claude models");
            Arc::new(AnthropicProvider::new(api_key))
        },
        _ => {
            let api_key = config.get_string("perplexity_api_key")
                .expect("Perplexity API key not found in config");
            println!("Using Perplexity provider with Llama models");
            Arc::new(PerplexityProvider::new(api_key))
        }
    }
}

pub fn get_server_config(config: &Config) -> ServerConfig {
    ServerConfig {
        host: config.get_string("server.host")
            .unwrap_or_else(|_| "127.0.0.1".to_string()),
        port: config.get_int("server.port")
            .unwrap_or(11435),
    }
}
