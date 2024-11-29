mod config;
mod llm;

use axum::{
    body::Body,
    http::{Request, Response},
    routing::post,
    Router,
};
use axum::routing::get;
use serde_json::{json, Value};
use hyper::StatusCode;
use tokio::net::TcpListener;
use crate::llm::{ChatOptions, Message};
use std::sync::Arc;
use crate::llm::{LLMProvider};
use axum::extract::State;

async fn handle_chat(
    State(provider): State<Arc<dyn LLMProvider + Send + Sync>>,
    request: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|e| {
            println!("Error reading request body: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let chat_request: Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| {
            println!("Error parsing request JSON: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let messages = chat_request["messages"]
        .as_array()
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .map(|m| Message {
            role: m["role"].as_str().unwrap_or("user").to_string(),
            content: m["content"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    let options = ChatOptions {
        temperature: chat_request["options"]["temperature"].as_f64().unwrap_or(0.7) as f32,
        top_p: chat_request["options"]["top_p"].as_f64().unwrap_or(0.9) as f32,
        max_tokens: None,
    };

    let response = provider.chat(messages, options).await
        .map_err(|e| {
            println!("LLM API error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let chat_response = json!({
        "model": response.model,
        "created_at": response.created_at,
        "message": {
            "role": "assistant",
            "content": response.content
        },
        "done": true,
        "total_duration": 0,
        "load_duration": 0,
        "prompt_eval_count": response.prompt_tokens,
        "eval_count": response.completion_tokens,
        "eval_duration": 0
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(chat_response.to_string()))
        .unwrap())
}


async fn root() -> &'static str {
    "Ollama is running"
}
async fn log_request(
    request: Request<Body>,
) -> Response<Body> {
    println!("Received request: {} {}", request.method(), request.uri());

    // Try to read and print the body
    match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => {
            if let Ok(body_str) = String::from_utf8(bytes.to_vec()) {
                println!("Request body: {}", body_str);
            }
        }
        Err(e) => println!("Could not read body: {}", e),
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}

async fn get_tags(
    State(provider): State<Arc<dyn LLMProvider + Send + Sync>>,
) -> Response<Body> {
    let response = json!({
        "models": provider.get_models()
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response.to_string()))
        .unwrap()
}
async fn show_model(
    State(provider): State<Arc<dyn LLMProvider + Send + Sync>>,
    request: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let request_body: Value = serde_json::from_slice(&body_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let model_name = request_body["name"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let response = provider.get_model_details(model_name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response.to_string()))
        .unwrap())
}
#[tokio::main]
async fn main() {
    let config = config::load_config();
    let provider = config::create_provider(&config);
    let server_config = config::get_server_config(&config);

    let addr = format!("{}:{}", server_config.host, server_config.port);

    let app = Router::new()
        .route("/", get(root))
        .route("/api/tags", get(get_tags))
        .route("/api/chat", post(handle_chat))
        .route("/api/show", post(show_model))
        .fallback(log_request)
        .with_state(provider);

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Ollama proxy server running on {}", addr);

    axum::serve(listener, app).await.unwrap();
}

