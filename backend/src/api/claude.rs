use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::services::camelot::EnergyProfile;

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
    pub confidence: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversation message types (for multi-turn conversations)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

// ---------------------------------------------------------------------------
// Request content block types (for prompt caching)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum RequestContentBlock {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub control_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
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

    async fn generate_with_blocks(
        &self,
        system_blocks: Vec<RequestContentBlock>,
        user_blocks: Vec<RequestContentBlock>,
        model: &str,
        max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError>;

    async fn converse(
        &self,
        system_prompt: &str,
        messages: Vec<ConversationMessage>,
        model: &str,
        max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        let _ = (system_prompt, messages, model, max_tokens);
        unimplemented!("converse not implemented for this client")
    }
}

// ---------------------------------------------------------------------------
// Claude Messages API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    pub content: Vec<ContentBlock>,
    #[serde(default)]
    pub usage: Option<UsageBlock>,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageBlock {
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
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
                .timeout(std::time::Duration::from_secs(90))
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

    /// Shared retry loop for sending requests to the Claude Messages API.
    async fn send_with_retries(
        &self,
        body: serde_json::Value,
    ) -> Result<MessagesResponse, ClaudeError> {
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

            return Ok(resp);
        }
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

// ---------------------------------------------------------------------------
// Enhanced prompt builders
// ---------------------------------------------------------------------------

/// Music verification skill document, injected into system prompt.
const MUSIC_SKILL: &str = include_str!("../prompts/music_skill.md");

/// Build the enhanced system prompt as structured content blocks with cache control.
pub fn build_enhanced_system_prompt(
    catalog_text: &str,
    energy_profile: Option<&EnergyProfile>,
    creative_mode: bool,
) -> Vec<RequestContentBlock> {
    let mut persona = String::from(
        r#"You are an expert DJ assistant and music curator. Your job is to create perfectly flowing setlists.

## Camelot Wheel Rules
- Compatible key transitions: same key, ±1 position (e.g., 8A→9A), parallel mode (e.g., 8A→8B)
- Avoid jumps > 2 positions on the Camelot wheel
- Energy-compatible transitions: adjacent tracks should differ by at most 2 energy levels

## Transition Techniques
- Harmonic mixing: blend tracks in compatible keys for smooth transitions
- BPM matching: keep BPM changes within ±6 BPM between adjacent tracks
- Energy flow: build and release energy intentionally, not randomly
- EQ blending: use low-end swap for kicks, high-pass for melodic transitions"#,
    );

    if let Some(profile) = energy_profile {
        persona.push_str("\n\n## Energy Profile\n");
        match profile {
            EnergyProfile::WarmUp => persona.push_str(
                "Start with low energy tracks (3-4), gradually building to medium-high (7). This is a warm-up set.",
            ),
            EnergyProfile::PeakTime => persona.push_str(
                "Maintain high energy (7-9) throughout. This is a peak-time set.",
            ),
            EnergyProfile::Journey => persona.push_str(
                "Start low (3), build to a peak (9) in the middle, then wind down (4). Take the audience on a journey.",
            ),
            EnergyProfile::Steady => persona.push_str(
                "Maintain consistent medium energy (5-7). Keep the vibe steady and groovy.",
            ),
        }
    }

    if creative_mode {
        persona.push_str("\n\n## Creative Mode\nBe creative and unexpected. Include surprising but compatible track combinations. Prioritize interesting transitions over safe choices.");
    }

    persona.push_str(
        r#"

## Output Format
Respond with ONLY valid JSON (no markdown fences, no explanation):
{
  "tracks": [
    {
      "position": 1,
      "title": "Track Name",
      "artist": "Artist Name",
      "bpm": 124.5,
      "key": "A minor",
      "camelot": "8A",
      "energy": 5,
      "transition_note": "Blend low-end, match kick",
      "source": "catalog",
      "track_id": "uuid-or-null",
      "confidence": "high"
    }
  ],
  "notes": "Brief description of the set flow..."
}"#,
    );

    let catalog_block = if catalog_text.is_empty() {
        "## No Catalog Available\nNo imported tracks are available. Generate all tracks as suggestions (set source to \"suggestion\", track_id to null). Use your knowledge of music to create an excellent setlist.".to_string()
    } else {
        format!(
            "## Available Catalog\nUse tracks from this catalog when possible (set source to \"catalog\" and use exact track_id). You may suggest tracks not in the catalog (set source to \"suggestion\", track_id to null).\n\n{}",
            catalog_text
        )
    };

    vec![
        // Static skill doc first for best cache hit rates (never changes)
        RequestContentBlock::Text {
            text: MUSIC_SKILL.to_string(),
            cache_control: Some(CacheControl {
                control_type: "ephemeral".to_string(),
            }),
        },
        // Persona varies with energy_profile + creative_mode
        RequestContentBlock::Text {
            text: persona,
            cache_control: Some(CacheControl {
                control_type: "ephemeral".to_string(),
            }),
        },
        // Catalog is most variable (per-user)
        RequestContentBlock::Text {
            text: catalog_block,
            cache_control: Some(CacheControl {
                control_type: "ephemeral".to_string(),
            }),
        },
    ]
}

/// Build the enhanced user prompt as structured content blocks.
pub fn build_enhanced_user_prompt(
    prompt: &str,
    seed_tracklist: Option<&str>,
    bpm_range: Option<(f64, f64)>,
) -> Vec<RequestContentBlock> {
    let mut text = prompt.to_string();

    if let Some(tracklist) = seed_tracklist {
        text.push_str(&format!(
            "\n\nHere are reference tracks to inspire the set:\n{}",
            tracklist
        ));
    }

    if let Some((min, max)) = bpm_range {
        text.push_str(&format!("\n\nConstrain BPM to {}-{} range.", min, max));
    }

    vec![RequestContentBlock::Text {
        text,
        cache_control: None,
    }]
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

        let resp = self.send_with_retries(body).await?;

        let text = resp
            .content
            .into_iter()
            .find(|b| b.content_type == "text")
            .and_then(|b| b.text)
            .ok_or_else(|| {
                ClaudeError::MalformedResponse("No text content in response".to_string())
            })?;

        Ok(text)
    }

    async fn generate_with_blocks(
        &self,
        system_blocks: Vec<RequestContentBlock>,
        user_blocks: Vec<RequestContentBlock>,
        model: &str,
        max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        let body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "system": system_blocks,
            "messages": [
                { "role": "user", "content": user_blocks }
            ]
        });

        let resp = self.send_with_retries(body).await?;

        let metrics = resp
            .usage
            .map(|u| CacheMetrics {
                cache_creation_input_tokens: u.cache_creation_input_tokens,
                cache_read_input_tokens: u.cache_read_input_tokens,
            })
            .unwrap_or_default();

        let text = resp
            .content
            .into_iter()
            .find(|b| b.content_type == "text")
            .and_then(|b| b.text)
            .ok_or_else(|| {
                ClaudeError::MalformedResponse("No text content in response".to_string())
            })?;

        Ok((text, metrics))
    }

    async fn converse(
        &self,
        system_prompt: &str,
        messages: Vec<ConversationMessage>,
        model: &str,
        max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        let messages_json: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": &m.role,
                    "content": &m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": model,
            "max_tokens": max_tokens,
            "system": system_prompt,
            "messages": messages_json,
        });

        let resp = self.send_with_retries(body).await?;

        resp.content
            .into_iter()
            .find(|b| b.content_type == "text")
            .and_then(|b| b.text)
            .ok_or_else(|| {
                ClaudeError::MalformedResponse("No text content in response".to_string())
            })
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

    // -----------------------------------------------------------------------
    // Enhanced prompt builder tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_enhanced_system_prompt_includes_energy_profile() {
        let blocks =
            build_enhanced_system_prompt("track1\ntrack2", Some(&EnergyProfile::WarmUp), false);
        assert_eq!(blocks.len(), 3);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(
            text.contains("Start with low energy tracks (3-4)"),
            "Expected warm-up energy text, got: {}",
            text
        );
    }

    #[test]
    fn test_enhanced_system_prompt_peak_time_profile() {
        let blocks = build_enhanced_system_prompt("catalog", Some(&EnergyProfile::PeakTime), false);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(text.contains("Maintain high energy (7-9)"));
    }

    #[test]
    fn test_enhanced_system_prompt_journey_profile() {
        let blocks = build_enhanced_system_prompt("catalog", Some(&EnergyProfile::Journey), false);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(text.contains("Start low (3), build to a peak (9)"));
    }

    #[test]
    fn test_enhanced_system_prompt_steady_profile() {
        let blocks = build_enhanced_system_prompt("catalog", Some(&EnergyProfile::Steady), false);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(text.contains("consistent medium energy (5-7)"));
    }

    #[test]
    fn test_enhanced_system_prompt_no_energy_profile() {
        let blocks = build_enhanced_system_prompt("catalog", None, false);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(!text.contains("Energy Profile"));
    }

    #[test]
    fn test_enhanced_system_prompt_creative_mode() {
        let blocks = build_enhanced_system_prompt("catalog", None, true);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(text.contains("Be creative and unexpected"));
        assert!(text.contains("surprising but compatible"));
    }

    #[test]
    fn test_enhanced_system_prompt_no_creative_mode() {
        let blocks = build_enhanced_system_prompt("catalog", None, false);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(!text.contains("Creative Mode"));
    }

    #[test]
    fn test_enhanced_system_prompt_catalog_in_third_block() {
        let catalog = "t1 | Desert Rose | 102 | 8A\nt2 | Habibi | 128 | 9A";
        let blocks = build_enhanced_system_prompt(catalog, None, false);
        assert_eq!(blocks.len(), 3);
        let RequestContentBlock::Text { ref text, .. } = blocks[2];
        assert!(text.contains("Desert Rose"));
        assert!(text.contains("Habibi"));
        assert!(text.contains("Available Catalog"));
    }

    #[test]
    fn test_enhanced_system_prompt_skill_block() {
        let blocks = build_enhanced_system_prompt("catalog", None, false);
        let RequestContentBlock::Text { ref text, .. } = blocks[0];
        assert!(
            text.contains("Track Verification Skill"),
            "Expected skill doc in block[0]"
        );
        assert!(
            text.contains("confidence"),
            "Skill doc should mention confidence field"
        );
    }

    #[test]
    fn test_enhanced_system_prompt_cache_control_on_all_blocks() {
        let blocks = build_enhanced_system_prompt("catalog", Some(&EnergyProfile::WarmUp), true);
        for block in &blocks {
            let RequestContentBlock::Text {
                ref cache_control, ..
            } = block;
            assert!(cache_control.is_some(), "Expected cache_control on block");
            assert_eq!(cache_control.as_ref().unwrap().control_type, "ephemeral");
        }
    }

    #[test]
    fn test_enhanced_user_prompt_basic() {
        let blocks = build_enhanced_user_prompt("Play me some house music", None, None);
        assert_eq!(blocks.len(), 1);
        let RequestContentBlock::Text { ref text, .. } = blocks[0];
        assert_eq!(text, "Play me some house music");
    }

    #[test]
    fn test_enhanced_user_prompt_with_seed_tracklist() {
        let seed = "1. Daft Punk - Around the World\n2. Chemical Brothers - Block Rockin Beats";
        let blocks = build_enhanced_user_prompt("Give me a 90s set", Some(seed), None);
        let RequestContentBlock::Text { ref text, .. } = blocks[0];
        assert!(text.contains("reference tracks to inspire the set"));
        assert!(text.contains("Daft Punk"));
        assert!(text.contains("Chemical Brothers"));
    }

    #[test]
    fn test_enhanced_user_prompt_with_bpm_range() {
        let blocks = build_enhanced_user_prompt("Deep house set", None, Some((120.0, 128.0)));
        let RequestContentBlock::Text { ref text, .. } = blocks[0];
        assert!(text.contains("Constrain BPM to 120-128 range."));
    }

    #[test]
    fn test_enhanced_user_prompt_with_all_options() {
        let blocks = build_enhanced_user_prompt(
            "Festival opener",
            Some("1. Track A\n2. Track B"),
            Some((125.0, 135.0)),
        );
        let RequestContentBlock::Text { ref text, .. } = blocks[0];
        assert!(text.contains("Festival opener"));
        assert!(text.contains("reference tracks"));
        assert!(text.contains("Constrain BPM to 125-135 range."));
    }

    #[test]
    fn test_enhanced_user_prompt_no_cache_control() {
        let blocks = build_enhanced_user_prompt("test", None, None);
        let RequestContentBlock::Text {
            ref cache_control, ..
        } = blocks[0];
        assert!(
            cache_control.is_none(),
            "User prompt blocks should not have cache_control"
        );
    }

    #[test]
    fn test_request_content_block_serialization() {
        let block = RequestContentBlock::Text {
            text: "hello".to_string(),
            cache_control: Some(CacheControl {
                control_type: "ephemeral".to_string(),
            }),
        };
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "hello");
        assert_eq!(json["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_request_content_block_serialization_no_cache() {
        let block = RequestContentBlock::Text {
            text: "hello".to_string(),
            cache_control: None,
        };
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "hello");
        assert!(json.get("cache_control").is_none());
    }

    #[test]
    fn test_cache_metrics_default() {
        let metrics = CacheMetrics::default();
        assert_eq!(metrics.cache_creation_input_tokens, 0);
        assert_eq!(metrics.cache_read_input_tokens, 0);
    }

    #[test]
    fn test_parse_response_with_usage() {
        let json = r#"{
            "content": [
                { "type": "text", "text": "hello" }
            ],
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50,
                "cache_creation_input_tokens": 1500,
                "cache_read_input_tokens": 800
            }
        }"#;
        let resp: MessagesResponse = serde_json::from_str(json).unwrap();
        let usage = resp.usage.unwrap();
        assert_eq!(usage.cache_creation_input_tokens, 1500);
        assert_eq!(usage.cache_read_input_tokens, 800);
    }

    #[test]
    fn test_parse_response_without_usage() {
        let json = r#"{
            "content": [
                { "type": "text", "text": "hello" }
            ]
        }"#;
        let resp: MessagesResponse = serde_json::from_str(json).unwrap();
        assert!(resp.usage.is_none());
    }

    // -----------------------------------------------------------------------
    // Wiremock HTTP tests
    // -----------------------------------------------------------------------

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

    #[tokio::test]
    async fn test_generate_with_blocks_sends_correct_json() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "content": [{"type": "text", "text": "{\"tracks\":[],\"notes\":\"ok\"}"}],
                "id": "msg_1",
                "model": "claude-sonnet-4-20250514",
                "role": "assistant",
                "type": "message",
                "usage": {
                    "input_tokens": 100,
                    "output_tokens": 50,
                    "cache_creation_input_tokens": 1200,
                    "cache_read_input_tokens": 500
                }
            })))
            .mount(&mock_server)
            .await;

        let client = ClaudeClient::new("test-key").with_base_url(mock_server.uri());

        let system_blocks = vec![RequestContentBlock::Text {
            text: "You are a DJ".to_string(),
            cache_control: Some(CacheControl {
                control_type: "ephemeral".to_string(),
            }),
        }];
        let user_blocks = vec![RequestContentBlock::Text {
            text: "Play house music".to_string(),
            cache_control: None,
        }];

        let result = client
            .generate_with_blocks(system_blocks, user_blocks, "claude-sonnet-4-20250514", 4096)
            .await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let (text, metrics) = result.unwrap();
        assert_eq!(text, "{\"tracks\":[],\"notes\":\"ok\"}");
        assert_eq!(metrics.cache_creation_input_tokens, 1200);
        assert_eq!(metrics.cache_read_input_tokens, 500);
    }

    #[tokio::test]
    async fn test_generate_with_blocks_retries_on_429() {
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
                if n == 0 {
                    ResponseTemplate::new(429).insert_header("retry-after", "0")
                } else {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "content": [{"type": "text", "text": "ok"}],
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
            .generate_with_blocks(
                vec![RequestContentBlock::Text {
                    text: "sys".to_string(),
                    cache_control: None,
                }],
                vec![RequestContentBlock::Text {
                    text: "user".to_string(),
                    cache_control: None,
                }],
                "claude-sonnet-4-20250514",
                100,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_enhanced_system_prompt_includes_camelot_rules() {
        let blocks = build_enhanced_system_prompt("catalog", None, false);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(text.contains("Camelot Wheel Rules"));
        assert!(text.contains("Transition Techniques"));
    }

    #[test]
    fn test_enhanced_system_prompt_includes_output_format() {
        let blocks = build_enhanced_system_prompt("catalog", None, false);
        let RequestContentBlock::Text { ref text, .. } = blocks[1];
        assert!(text.contains("Output Format"));
        assert!(text.contains("ONLY valid JSON"));
    }

    #[tokio::test]
    async fn test_converse_sends_multi_turn_messages() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "content": [{"type": "text", "text": "I replaced track 5 with a darker track."}],
                "id": "msg_conv_1",
                "model": "claude-sonnet-4-20250514",
                "role": "assistant",
                "type": "message"
            })))
            .mount(&mock_server)
            .await;

        let client = ClaudeClient::new("test-key").with_base_url(mock_server.uri());

        let messages = vec![
            ConversationMessage {
                role: "user".to_string(),
                content: "Generate a house setlist".to_string(),
            },
            ConversationMessage {
                role: "assistant".to_string(),
                content: "{\"actions\": [], \"explanation\": \"Here's your setlist\"}".to_string(),
            },
            ConversationMessage {
                role: "user".to_string(),
                content: "Swap track 5 for something darker".to_string(),
            },
        ];

        let result = client
            .converse(
                "You are a DJ assistant",
                messages,
                "claude-sonnet-4-20250514",
                4096,
            )
            .await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        assert_eq!(result.unwrap(), "I replaced track 5 with a darker track.");
    }

    #[tokio::test]
    async fn test_converse_handles_api_error() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad request"))
            .mount(&mock_server)
            .await;

        let client = ClaudeClient::new("test-key").with_base_url(mock_server.uri());

        let messages = vec![ConversationMessage {
            role: "user".to_string(),
            content: "test".to_string(),
        }];

        let result = client
            .converse("system", messages, "claude-sonnet-4-20250514", 100)
            .await;

        assert!(matches!(result, Err(ClaudeError::Api(_))));
    }
}
