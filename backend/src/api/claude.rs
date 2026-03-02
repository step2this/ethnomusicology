use serde::{Deserialize, Serialize};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ClaudeError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Claude API error: {0}")]
    Api(String),

    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Malformed response: {0}")]
    MalformedResponse(String),

    #[error("Request timed out")]
    Timeout,
}

// ---------------------------------------------------------------------------
// LLM response types (what Claude produces)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSetlistResponse {
    pub tracks: Vec<LlmTrackEntry>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTrackEntry {
    pub position: i32,
    pub title: String,
    pub artist: String,
    pub bpm: Option<f64>,
    pub key: Option<String>,
    pub camelot: Option<String>,
    pub energy: Option<i32>,
    pub transition_note: Option<String>,
    pub source: Option<String>,
    pub track_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Trait for mock injection
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait ClaudeClientTrait: Send + Sync {
    async fn generate_setlist(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
        max_tokens: u32,
    ) -> Result<String, ClaudeError>;
}

// ---------------------------------------------------------------------------
// Claude Messages API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct ClaudeClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl ClaudeClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

/// Build the system prompt for DJ setlist generation.
pub fn build_system_prompt(catalog_text: &str) -> String {
    format!(
        r#"You are an expert DJ assistant. Your job is to create setlists from a music catalog based on user prompts.

## Rules
1. Select tracks that match the user's mood, genre, and energy requirements.
2. Consider BPM progression and key compatibility (Camelot wheel) for smooth transitions.
3. If a track exists in the catalog, use its exact ID and set source to "catalog".
4. If you suggest tracks not in the catalog, set source to "suggestion" and track_id to null.
5. Provide transition notes explaining how to mix between consecutive tracks.
6. Energy levels range from 1 (chill) to 10 (peak energy).
7. Camelot keys follow the format: number (1-12) + letter (A or B), e.g., "8A", "11B".

## Catalog
{catalog_text}

## Output Format
Respond with ONLY valid JSON (no markdown fences, no explanation):
{{
  "tracks": [
    {{
      "position": 1,
      "title": "Track Name",
      "artist": "Artist Name",
      "bpm": 124.5,
      "key": "A minor",
      "camelot": "8A",
      "energy": 5,
      "transition_note": "Blend low-end, match kick",
      "source": "catalog",
      "track_id": "uuid-or-null"
    }}
  ],
  "notes": "Brief description of the set flow..."
}}"#
    )
}

/// Strip markdown code fences from Claude responses.
pub fn strip_markdown_fences(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        rest.strip_suffix("```").unwrap_or(rest).trim()
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest.strip_suffix("```").unwrap_or(rest).trim()
    } else {
        trimmed
    }
}

/// Maximum retries for 429 rate-limited responses.
const MAX_RETRIES_RATE_LIMITED: u32 = 3;
/// Maximum retries for 5xx server errors.
const MAX_RETRIES_SERVER_ERROR: u32 = 2;
/// Default retry-after duration when header is missing (seconds).
const DEFAULT_RETRY_AFTER_SECS: u64 = 5;
/// Base delay for server-error exponential backoff (seconds).
const SERVER_ERROR_BASE_DELAY_SECS: u64 = 1;

#[async_trait::async_trait]
impl ClaudeClientTrait for ClaudeClient {
    async fn generate_setlist(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
        max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        let body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "system": system_prompt,
            "messages": [
                { "role": "user", "content": user_prompt }
            ]
        });

        let mut rate_limit_attempts = 0u32;
        let mut server_error_attempts = 0u32;

        loop {
            let response = self
                .http
                .post(format!("{}/v1/messages", self.base_url))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        ClaudeError::Timeout
                    } else {
                        ClaudeError::Http(e)
                    }
                })?;

            let status = response.status().as_u16();

            if status == 429 {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_RETRY_AFTER_SECS);

                rate_limit_attempts += 1;
                if rate_limit_attempts > MAX_RETRIES_RATE_LIMITED {
                    return Err(ClaudeError::RateLimited {
                        retry_after_secs: retry_after,
                    });
                }

                tracing::warn!(
                    attempt = rate_limit_attempts,
                    status = 429,
                    retry_after_secs = retry_after,
                    "Rate limited by Claude API, retrying"
                );
                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                continue;
            }

            if (500..=503).contains(&status) {
                server_error_attempts += 1;
                if server_error_attempts > MAX_RETRIES_SERVER_ERROR {
                    let body_text = response.text().await.unwrap_or_default();
                    return Err(ClaudeError::Api(format!(
                        "Server error {status}: {body_text}"
                    )));
                }

                let delay_secs = SERVER_ERROR_BASE_DELAY_SECS * 2u64.pow(server_error_attempts - 1);
                tracing::warn!(
                    attempt = server_error_attempts,
                    status = status,
                    delay_secs = delay_secs,
                    "Server error from Claude API, retrying with backoff"
                );
                tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                continue;
            }

            if status != 200 {
                let body_text = response.text().await.unwrap_or_default();
                return Err(ClaudeError::Api(format!("HTTP {status}: {body_text}")));
            }

            let resp: MessagesResponse = response.json().await.map_err(|e| {
                ClaudeError::MalformedResponse(format!("Failed to parse Messages response: {e}"))
            })?;

            let text = resp
                .content
                .into_iter()
                .find(|b| b.content_type == "text")
                .and_then(|b| b.text)
                .ok_or_else(|| {
                    ClaudeError::MalformedResponse("No text content in response".to_string())
                })?;

            return Ok(text);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt_contains_keys() {
        let prompt = build_system_prompt("ID | Title | BPM | Key");
        assert!(prompt.contains("tracks"));
        assert!(prompt.contains("camelot"));
        assert!(prompt.contains("Catalog"));
    }

    #[test]
    fn test_parse_valid_messages_response() {
        let json = r#"{
            "content": [
                { "type": "text", "text": "{\"tracks\":[], \"notes\": \"test\"}" }
            ],
            "id": "msg_123",
            "model": "claude-sonnet-4-20250514",
            "role": "assistant",
            "type": "message"
        }"#;
        let resp: MessagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content.len(), 1);
        assert_eq!(
            resp.content[0].text.as_deref(),
            Some("{\"tracks\":[], \"notes\": \"test\"}")
        );
    }

    #[test]
    fn test_parse_malformed_response_missing_content() {
        let json = r#"{"id": "msg_123"}"#;
        let result: Result<MessagesResponse, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_llm_setlist_response() {
        let json = r#"{
            "tracks": [
                {
                    "position": 1,
                    "title": "Desert Rose",
                    "artist": "Sting",
                    "bpm": 102.0,
                    "key": "A minor",
                    "camelot": "8A",
                    "energy": 5,
                    "transition_note": "Open with atmospheric pads",
                    "source": "catalog",
                    "track_id": "abc-123"
                }
            ],
            "notes": "A chill desert set"
        }"#;
        let resp: LlmSetlistResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.tracks.len(), 1);
        assert_eq!(resp.tracks[0].title, "Desert Rose");
        assert_eq!(resp.tracks[0].energy, Some(5));
        assert_eq!(resp.notes.as_deref(), Some("A chill desert set"));
    }

    #[test]
    fn test_deserialize_with_missing_optionals() {
        let json = r#"{
            "tracks": [
                {
                    "position": 1,
                    "title": "Unknown",
                    "artist": "Unknown",
                    "bpm": null,
                    "key": null,
                    "camelot": null,
                    "energy": null,
                    "transition_note": null,
                    "source": null,
                    "track_id": null
                }
            ],
            "notes": null
        }"#;
        let resp: LlmSetlistResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.tracks.len(), 1);
        assert!(resp.tracks[0].bpm.is_none());
        assert!(resp.tracks[0].key.is_none());
        assert!(resp.notes.is_none());
    }

    #[test]
    fn test_strip_markdown_fences() {
        let with_json_fence = "```json\n{\"tracks\": []}\n```";
        assert_eq!(strip_markdown_fences(with_json_fence), "{\"tracks\": []}");

        let with_plain_fence = "```\n{\"tracks\": []}\n```";
        assert_eq!(strip_markdown_fences(with_plain_fence), "{\"tracks\": []}");

        let no_fence = "{\"tracks\": []}";
        assert_eq!(strip_markdown_fences(no_fence), "{\"tracks\": []}");
    }

    #[test]
    fn test_error_display_messages() {
        let timeout = ClaudeError::Timeout;
        assert_eq!(timeout.to_string(), "Request timed out");

        let rate_limited = ClaudeError::RateLimited {
            retry_after_secs: 10,
        };
        assert_eq!(rate_limited.to_string(), "Rate limited, retry after 10s");

        let api_err = ClaudeError::Api("something broke".to_string());
        assert_eq!(api_err.to_string(), "Claude API error: something broke");

        let malformed = ClaudeError::MalformedResponse("bad json".to_string());
        assert_eq!(malformed.to_string(), "Malformed response: bad json");
    }

    #[tokio::test]
    async fn test_timeout_from_slow_server() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Respond after 10 seconds -- our client has a 1ms timeout below
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
            .mount(&mock_server)
            .await;

        // Build a client with a very short timeout (50ms) to reliably trigger timeout
        let client = ClaudeClient {
            http: reqwest::Client::builder()
                .timeout(Duration::from_millis(50))
                .build()
                .unwrap(),
            api_key: "test-key".to_string(),
            base_url: mock_server.uri(),
        };

        let result = client
            .generate_setlist("system", "user prompt", "claude-sonnet-4-20250514", 1024)
            .await;

        assert!(result.is_err());
        assert!(
            matches!(result.as_ref().unwrap_err(), ClaudeError::Timeout),
            "Expected ClaudeError::Timeout, got {:?}",
            result.unwrap_err()
        );
    }

    #[tokio::test]
    async fn test_retry_on_429_then_success() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

        let call_count = Arc::new(AtomicU32::new(0));
        let counter = call_count.clone();

        struct RateLimitThenOk(Arc<AtomicU32>);
        impl Respond for RateLimitThenOk {
            fn respond(&self, _req: &Request) -> ResponseTemplate {
                let n = self.0.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    // First two calls return 429
                    ResponseTemplate::new(429).insert_header("retry-after", "0")
                } else {
                    // Third call succeeds
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "content": [{"type": "text", "text": "{\"tracks\":[],\"notes\":\"ok\"}"}],
                        "id": "msg_1",
                        "model": "claude-sonnet-4-20250514",
                        "role": "assistant",
                        "type": "message"
                    }))
                }
            }
        }

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(RateLimitThenOk(counter))
            .mount(&mock_server)
            .await;

        let client = ClaudeClient::new("test-key").with_base_url(mock_server.uri());
        let result = client
            .generate_setlist("system", "user", "claude-sonnet-4-20250514", 100)
            .await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_429_exhausts_retries() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
            .mount(&mock_server)
            .await;

        let client = ClaudeClient::new("test-key").with_base_url(mock_server.uri());
        let result = client
            .generate_setlist("system", "user", "claude-sonnet-4-20250514", 100)
            .await;

        assert!(
            matches!(result, Err(ClaudeError::RateLimited { .. })),
            "Expected RateLimited error, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_retry_on_500_then_success() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

        let call_count = Arc::new(AtomicU32::new(0));
        let counter = call_count.clone();

        struct ServerErrThenOk(Arc<AtomicU32>);
        impl Respond for ServerErrThenOk {
            fn respond(&self, _req: &Request) -> ResponseTemplate {
                let n = self.0.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    ResponseTemplate::new(503)
                } else {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "content": [{"type": "text", "text": "{\"tracks\":[],\"notes\":\"recovered\"}"}],
                        "id": "msg_2",
                        "model": "claude-sonnet-4-20250514",
                        "role": "assistant",
                        "type": "message"
                    }))
                }
            }
        }

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ServerErrThenOk(counter))
            .mount(&mock_server)
            .await;

        let client = ClaudeClient::new("test-key").with_base_url(mock_server.uri());
        let result = client
            .generate_setlist("system", "user", "claude-sonnet-4-20250514", 100)
            .await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_5xx_exhausts_retries() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let call_count = Arc::new(AtomicU32::new(0));
        let counter = call_count.clone();

        let mock_server = MockServer::start().await;

        struct AlwaysServerErr(Arc<AtomicU32>);
        impl wiremock::Respond for AlwaysServerErr {
            fn respond(&self, _req: &wiremock::Request) -> ResponseTemplate {
                self.0.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(500).set_body_string("internal error")
            }
        }

        Mock::given(method("POST"))
            .respond_with(AlwaysServerErr(counter))
            .mount(&mock_server)
            .await;

        let client = ClaudeClient::new("test-key").with_base_url(mock_server.uri());
        let result = client
            .generate_setlist("system", "user", "claude-sonnet-4-20250514", 100)
            .await;

        assert!(
            matches!(result, Err(ClaudeError::Api(_))),
            "Expected Api error, got {:?}",
            result
        );
        // 1 initial + 2 retries = 3
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }
}
