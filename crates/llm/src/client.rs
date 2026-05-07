use crate::config::Config;
use crate::convert::{messages_to_request, response_from_value};
use async_trait::async_trait;
use lithify_core::{LLMClient, LLMError, Message, Response};
use tracing::{info, warn};

/// Anthropic Claude API client implementing [`LLMClient`].
pub struct AnthropicClient {
    config: Config,
    http: reqwest::Client,
}

impl AnthropicClient {
    /// Create a new client from the given configuration.
    ///
    /// The internal `reqwest::Client` is configured with the timeout from `config`.
    pub fn new(config: Config) -> Self {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("reqwest::Client::builder() should not fail with default settings");
        Self { config, http }
    }
}

#[async_trait]
impl LLMClient for AnthropicClient {
    async fn chat(&self, messages: &[Message]) -> Result<Response, LLMError> {
        let url = format!("{}/v1/messages", self.config.base_url);
        let body = messages_to_request(&self.config.model, self.config.max_tokens, messages);

        info!(
            model = %self.config.model,
            message_count = messages.len(),
            "sending request to Anthropic API"
        );

        let response = self
            .http
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                warn!(error = %e, "network error calling Anthropic API");
                LLMError::Network(e.to_string())
            })?;

        let status = response.status();
        if status.is_success() {
            let value = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| LLMError::Api(format!("failed to parse response: {e}")))?;

            let resp = response_from_value(value)?;
            info!(
                input_tokens = resp.usage.input_tokens,
                output_tokens = resp.usage.output_tokens,
                "received response from Anthropic API"
            );
            Ok(resp)
        } else if status == 429 {
            warn!("rate limited by Anthropic API");
            Err(LLMError::RateLimited)
        } else {
            let body_text = response.text().await.unwrap_or_default();
            warn!(status = status.as_u16(), "API error from Anthropic");
            Err(LLMError::Api(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body_text
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lithify_core::{ContentBlock, Role};
    use std::time::Duration;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(api_key: &str, base_url: &str) -> Config {
        Config {
            api_key: api_key.to_string(),
            model: "claude-test".into(),
            max_tokens: 100,
            timeout: Duration::from_secs(5),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    #[tokio::test]
    async fn chat_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("x-api-key", "sk-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "content": [{"type": "text", "text": "Hello!"}],
                "usage": {"input_tokens": 5, "output_tokens": 3},
            })))
            .mount(&mock_server)
            .await;

        let cfg = test_config("sk-test", &mock_server.uri());
        let client = AnthropicClient::new(cfg);

        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hi".into())],
        }];

        let resp = client.chat(&msgs).await.unwrap();
        assert!(matches!(&resp.content[0], ContentBlock::Text(t) if t == "Hello!"));
        assert_eq!(resp.usage.input_tokens, 5);
        assert_eq!(resp.usage.output_tokens, 3);
    }

    #[tokio::test]
    async fn chat_rate_limited() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let cfg = test_config("sk-test", &mock_server.uri());
        let client = AnthropicClient::new(cfg);

        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hi".into())],
        }];

        let err = client.chat(&msgs).await.unwrap_err();
        assert!(matches!(err, LLMError::RateLimited));
    }

    #[tokio::test]
    async fn chat_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": {"type": "invalid_request_error", "message": "Bad request"}
            })))
            .mount(&mock_server)
            .await;

        let cfg = test_config("sk-test", &mock_server.uri());
        let client = AnthropicClient::new(cfg);

        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hi".into())],
        }];

        let err = client.chat(&msgs).await.unwrap_err();
        assert!(matches!(err, LLMError::Api(_)));
        assert!(err.to_string().contains("400"));
    }

    #[tokio::test]
    async fn chat_json_parse_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
            .mount(&mock_server)
            .await;

        let cfg = test_config("sk-test", &mock_server.uri());
        let client = AnthropicClient::new(cfg);

        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hi".into())],
        }];

        let err = client.chat(&msgs).await.unwrap_err();
        assert!(matches!(err, LLMError::Api(_)));
    }

    #[tokio::test]
    async fn chat_network_error() {
        // Connect to a non-routable address to simulate a network error
        let cfg = test_config("sk-test", "http://127.0.0.1:1");
        let client = AnthropicClient::new(cfg);

        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hi".into())],
        }];

        let err = client.chat(&msgs).await.unwrap_err();
        assert!(matches!(err, LLMError::Network(_)));
    }

    // Real LLM integration test — only runs when ANTHROPIC_API_KEY is set.
    #[tokio::test]
    #[ignore]
    async fn real_llm_text_only() {
        let cfg = Config::from_env().expect("ANTHROPIC_API_KEY must be set");
        let client = AnthropicClient::new(cfg);

        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text(
                "Just say 'hello' and nothing else.".into(),
            )],
        }];

        let resp = client.chat(&msgs).await.expect("API call failed");
        assert!(!resp.content.is_empty(), "response should have content");
        assert!(resp.usage.input_tokens > 0, "should have input tokens");
        assert!(resp.usage.output_tokens > 0, "should have output tokens");
    }
}
