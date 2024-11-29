# Ollama API Proxy for LLM Providers

This application serves as a proxy that implements the Ollama API interface but forwards requests to different LLM providers like Anthropic's Claude and Perplexity AI. This allows IDE plugins that support Ollama to work with these alternative LLM providers.

Looking at you, JetBrains.

## Features

- Implements Ollama's API endpoints:
    - `/api/chat` - For chat completions
    - `/api/tags` - Lists available models
    - `/api/show` - Shows model details
    - `/` - Health check endpoint
- Supports multiple LLM providers:
    - Perplexity AI (Llama models)
    - Anthropic (Claude models)
- Configurable server settings
- Easy provider switching via configuration

## Installation

1. Clone the repository
2. Install Rust if you haven't already (https://rustup.rs/)
3. Create a `Config.toml` file in the project root

## Configuration

Create a `Config.toml` file with the following structure:

```toml
# Provider configuration
provider_type = "perplexity"  # or "anthropic"
perplexity_api_key = "your-perplexity-key"
anthropic_api_key = "your-anthropic-key"

# Server configuration
[server]
host = "127.0.0.1"
port = 11434
```

## Available Models

### Perplexity AI
- `llama-3.1-sonar-small-128k-online` (8B parameters)
- `llama-3.1-sonar-large-128k-online` (70B parameters)
- `llama-3.1-sonar-huge-128k-online` (405B parameters)

### Anthropic
- `claude-3-5-sonnet-20241022`
- `claude-3-5-haiku-20241022`
- `claude-3-opus-20240229`

## Usage

1. Start the server:
    ```bash
    cargo run
    ```

2. Configure your IDE's Ollama plugin to use the proxy URL:
    ```
    http://localhost:11434  # or your configured host:port
    ```

## API Endpoints

### GET /
Returns "Ollama is running" to indicate the server is up.

### GET /api/tags
Lists all available models for the configured provider.

### POST /api/chat
Handles chat completions. Example request:
```json
{
  "model": "llama2",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "Hello!"
    }
  ],
  "options": {
    "temperature": 0.7,
    "top_p": 0.9
  }
}
```


### POST /api/show
Shows details about a specific model. Example request:
```json
{
  "name": "llama2"
}
```

## Development

The application is built with:
- Rust
- Axum web framework
- Tokio async runtime
- Reqwest for HTTP clients

The architecture follows a trait-based approach for provider implementations, making it easy to add new LLM providers.

## Error Handling

The application includes comprehensive error handling for:
- API communication errors
- Request parsing errors
- Configuration errors
- Invalid model selections

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
