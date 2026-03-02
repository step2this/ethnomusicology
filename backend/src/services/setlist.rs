use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::api::claude::{
    build_system_prompt, strip_markdown_fences, ClaudeClientTrait, ClaudeError, LlmSetlistResponse,
};
use crate::db::models::{SetlistRow, SetlistTrackRow, TrackRow};
use crate::db::setlists as db;
use crate::services::arrangement::{self, ArrangementTrack};
use crate::services::camelot::parse_camelot;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SetlistError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Empty catalog: no tracks available")]
    EmptyCatalog,

    #[error("Claude API error: {0}")]
    ClaudeError(String),

    #[error("Service busy: {0}")]
    ServiceBusy(String),

    #[error("Request timed out")]
    Timeout,

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl IntoResponse for SetlistError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self {
            SetlistError::InvalidRequest(m) => {
                (StatusCode::BAD_REQUEST, "INVALID_REQUEST", m.clone())
            }
            SetlistError::EmptyCatalog => (
                StatusCode::BAD_REQUEST,
                "EMPTY_CATALOG",
                "No tracks in catalog. Import music first.".to_string(),
            ),
            SetlistError::ClaudeError(m) => {
                (StatusCode::SERVICE_UNAVAILABLE, "LLM_ERROR", m.clone())
            }
            SetlistError::ServiceBusy(m) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_BUSY", m.clone())
            }
            SetlistError::Timeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "TIMEOUT",
                "Request timed out".to_string(),
            ),
            SetlistError::GenerationFailed(m) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "GENERATION_FAILED",
                m.clone(),
            ),
            SetlistError::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m.clone()),
            SetlistError::Database(m) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                format!("Database error: {m}"),
            ),
        };

        let body = serde_json::json!({
            "error": {
                "code": code,
                "message": msg,
            }
        });
        (status, axum::Json(body)).into_response()
    }
}

impl From<ClaudeError> for SetlistError {
    fn from(e: ClaudeError) -> Self {
        match e {
            ClaudeError::RateLimited { retry_after_secs } => {
                SetlistError::ServiceBusy(format!("Rate limited, retry after {retry_after_secs}s"))
            }
            ClaudeError::Timeout => SetlistError::Timeout,
            other => SetlistError::ClaudeError(other.to_string()),
        }
    }
}

impl From<sqlx::Error> for SetlistError {
    fn from(e: sqlx::Error) -> Self {
        SetlistError::Database(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Response types (matching API contract)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ScoreBreakdownResponse {
    pub key_compatibility: f64,
    pub bpm_continuity: f64,
    pub energy_arc: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetlistResponse {
    pub id: String,
    pub prompt: String,
    pub model: String,
    pub tracks: Vec<SetlistTrackResponse>,
    pub notes: Option<String>,
    pub harmonic_flow_score: Option<f64>,
    pub score_breakdown: Option<ScoreBreakdownResponse>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetlistTrackResponse {
    pub position: i32,
    pub title: String,
    pub artist: String,
    pub bpm: Option<f64>,
    pub key: Option<String>,
    pub camelot: Option<String>,
    pub energy: Option<f64>,
    pub transition_note: Option<String>,
    pub transition_score: Option<f64>,
    pub original_position: i32,
    pub source: String,
    pub track_id: Option<String>,
}

impl From<SetlistTrackRow> for SetlistTrackResponse {
    fn from(row: SetlistTrackRow) -> Self {
        SetlistTrackResponse {
            position: row.position,
            title: row.title,
            artist: row.artist,
            bpm: row.bpm,
            key: row.key,
            camelot: row.camelot,
            energy: row.energy,
            transition_note: row.transition_note,
            transition_score: row.transition_score,
            original_position: row.original_position,
            source: row.source,
            track_id: row.track_id,
        }
    }
}

// ---------------------------------------------------------------------------
// Catalog serialization
// ---------------------------------------------------------------------------

fn serialize_catalog(tracks: &[TrackRow]) -> String {
    let mut lines = Vec::with_capacity(tracks.len() + 1);
    lines.push("ID | Title - Artist | BPM | Key | Energy".to_string());
    for t in tracks {
        let artist = t.artist.as_deref().unwrap_or("Unknown");
        let bpm = t
            .bpm
            .map(|b| format!("{b:.1}"))
            .unwrap_or_else(|| "--".to_string());
        let key = t.camelot_key.as_deref().unwrap_or("--");
        let energy = t
            .energy
            .map(|e| format!("{e:.0}"))
            .unwrap_or_else(|| "--".to_string());
        lines.push(format!(
            "{} | {} - {} | {} | {} | {}",
            t.id, t.title, artist, bpm, key, energy
        ));
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const MAX_PROMPT_LEN: usize = 2000;
const DEFAULT_TRACK_COUNT: u32 = 10;
const MIN_TRACK_COUNT: u32 = 1;
const MAX_TRACK_COUNT: u32 = 50;

pub async fn generate_setlist(
    pool: &sqlx::SqlitePool,
    claude: &dyn ClaudeClientTrait,
    user_id: &str,
    prompt: &str,
    track_count: Option<u32>,
) -> Result<SetlistResponse, SetlistError> {
    // Validate prompt
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(SetlistError::InvalidRequest(
            "Prompt cannot be empty".to_string(),
        ));
    }
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(SetlistError::InvalidRequest(format!(
            "Prompt too long ({} chars, max {MAX_PROMPT_LEN})",
            prompt.len()
        )));
    }

    // M2: Validate track_count
    let count = match track_count {
        Some(c) if !(MIN_TRACK_COUNT..=MAX_TRACK_COUNT).contains(&c) => {
            return Err(SetlistError::InvalidRequest(format!(
                "track_count must be between {MIN_TRACK_COUNT} and {MAX_TRACK_COUNT}, got {c}"
            )));
        }
        Some(c) => c,
        None => DEFAULT_TRACK_COUNT,
    };

    // Load catalog
    let catalog = db::load_catalog_tracks(pool).await?;
    if catalog.is_empty() {
        return Err(SetlistError::EmptyCatalog);
    }

    // Build catalog IDs set for validation
    let catalog_ids: std::collections::HashSet<String> =
        catalog.iter().map(|t| t.id.clone()).collect();

    // Build prompts
    let catalog_text = serialize_catalog(&catalog);
    let system_prompt = build_system_prompt(&catalog_text);
    let user_prompt = format!("Create a setlist of {count} tracks based on this prompt: {prompt}");

    // Call Claude
    let raw_response = claude
        .generate_setlist(&system_prompt, &user_prompt, DEFAULT_MODEL, 4096)
        .await
        .map_err(SetlistError::from)?;

    // Parse response (with retry on failure)
    let cleaned = strip_markdown_fences(&raw_response);
    let llm_response: LlmSetlistResponse = match serde_json::from_str(cleaned) {
        Ok(r) => r,
        Err(first_err) => {
            // Retry with stricter prompt
            tracing::warn!("First parse failed: {first_err}, retrying with stricter prompt");
            let retry_prompt = format!(
                "{user_prompt}\n\nIMPORTANT: Respond with ONLY valid JSON. No markdown fences, no explanation text."
            );
            let retry_response = claude
                .generate_setlist(&system_prompt, &retry_prompt, DEFAULT_MODEL, 4096)
                .await
                .map_err(SetlistError::from)?;

            let retry_cleaned = strip_markdown_fences(&retry_response);
            serde_json::from_str(retry_cleaned).map_err(|e| {
                SetlistError::GenerationFailed(format!(
                    "Failed to parse LLM response after retry: {e}"
                ))
            })?
        }
    };

    // M3: Filter out entries with missing title or artist, log warnings
    let total_entries = llm_response.tracks.len();
    let valid_entries: Vec<_> = llm_response
        .tracks
        .iter()
        .filter(|entry| {
            if entry.title.is_empty() || entry.artist.is_empty() {
                tracing::warn!(
                    "Skipping LLM track entry with missing title='{}' or artist='{}'",
                    entry.title,
                    entry.artist
                );
                false
            } else {
                true
            }
        })
        .collect();

    let skipped_count = total_entries - valid_entries.len();
    if total_entries > 0 && skipped_count > total_entries / 2 {
        return Err(SetlistError::GenerationFailed(format!(
            "Too many invalid entries from LLM: {skipped_count}/{total_entries} were missing title or artist"
        )));
    }

    // Generate setlist ID and persist
    let setlist_id = uuid::Uuid::new_v4().to_string();

    let setlist_row = SetlistRow {
        id: setlist_id.clone(),
        user_id: user_id.to_string(),
        prompt: prompt.to_string(),
        model: DEFAULT_MODEL.to_string(),
        notes: llm_response.notes.clone(),
        harmonic_flow_score: None,
        created_at: None,
    };

    // M4: Attempt DB write; on failure, return partial response with warning
    let db_write_failed = match db::insert_setlist(pool, &setlist_row).await {
        Ok(()) => false,
        Err(e) => {
            tracing::error!("Failed to persist setlist: {e}");
            true
        }
    };

    // Process tracks
    let mut track_responses = Vec::with_capacity(valid_entries.len());
    let mut any_track_write_failed = db_write_failed;

    for (i, entry) in valid_entries.iter().enumerate() {
        let position = (i + 1) as i32;

        // Validate track_id against catalog
        let (validated_track_id, source) = match &entry.track_id {
            Some(tid) if catalog_ids.contains(tid) => (Some(tid.clone()), "catalog".to_string()),
            Some(_) => {
                // Hallucinated track_id — reclassify as suggestion
                tracing::warn!(
                    "Hallucinated track_id for '{}' by '{}', reclassifying as suggestion",
                    entry.title,
                    entry.artist
                );
                (None, "suggestion".to_string())
            }
            None => (
                None,
                entry
                    .source
                    .clone()
                    .unwrap_or_else(|| "suggestion".to_string()),
            ),
        };

        let track_row = SetlistTrackRow {
            id: uuid::Uuid::new_v4().to_string(),
            setlist_id: setlist_id.clone(),
            track_id: validated_track_id.clone(),
            position,
            original_position: position,
            title: entry.title.clone(),
            artist: entry.artist.clone(),
            bpm: entry.bpm,
            key: entry.key.clone(),
            camelot: entry.camelot.clone(),
            energy: entry.energy.map(|e| e as f64),
            transition_note: entry.transition_note.clone(),
            transition_score: None,
            source: source.clone(),
            acquisition_info: None,
        };

        // M4: Only attempt track DB write if setlist was persisted
        if !db_write_failed {
            if let Err(e) = db::insert_setlist_track(pool, &track_row).await {
                tracing::error!("Failed to persist setlist track: {e}");
                any_track_write_failed = true;
            }
        }

        track_responses.push(SetlistTrackResponse {
            position,
            title: entry.title.clone(),
            artist: entry.artist.clone(),
            bpm: entry.bpm,
            key: entry.key.clone(),
            camelot: entry.camelot.clone(),
            energy: entry.energy.map(|e| e as f64),
            transition_note: entry.transition_note.clone(),
            transition_score: None,
            original_position: position,
            source,
            track_id: validated_track_id,
        });
    }

    // Build notes with possible DB warning
    let mut notes = llm_response.notes;
    if any_track_write_failed {
        let warning = " [WARNING: Setlist was not saved to database. You may want to regenerate.]";
        notes = Some(match notes {
            Some(n) => format!("{n}{warning}"),
            None => warning.to_string(),
        });
    }

    // M4: Use temporary ID if DB write failed, otherwise fetch created_at
    let (final_id, created_at) = if db_write_failed {
        let temp_id = format!("unsaved-{}", uuid::Uuid::new_v4());
        (temp_id, None)
    } else {
        // Re-fetch to get created_at from DB
        let created_at = match db::get_setlist(pool, &setlist_id).await {
            Ok(Some(saved)) => saved
                .created_at
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            _ => None,
        };
        (setlist_id, created_at)
    };

    Ok(SetlistResponse {
        id: final_id,
        prompt: prompt.to_string(),
        model: DEFAULT_MODEL.to_string(),
        tracks: track_responses,
        notes,
        harmonic_flow_score: None,
        score_breakdown: None,
        created_at,
    })
}

pub async fn get_setlist(
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<SetlistResponse, SetlistError> {
    let setlist = db::get_setlist(pool, id)
        .await?
        .ok_or_else(|| SetlistError::NotFound(format!("Setlist {id} not found")))?;

    let tracks = db::get_setlist_tracks(pool, id).await?;

    Ok(SetlistResponse {
        id: setlist.id,
        prompt: setlist.prompt,
        model: setlist.model,
        tracks: tracks.into_iter().map(SetlistTrackResponse::from).collect(),
        notes: setlist.notes,
        harmonic_flow_score: setlist.harmonic_flow_score,
        score_breakdown: None,
        created_at: setlist
            .created_at
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
    })
}

/// L1: Extracted arrange logic from route handler into service layer.
/// Loads setlist, runs arrangement algorithm, persists results, returns response.
pub async fn arrange_setlist(
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<SetlistResponse, SetlistError> {
    // Load the setlist
    let setlist_row = db::get_setlist(pool, id)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?
        .ok_or_else(|| SetlistError::NotFound(format!("Setlist {id} not found")))?;

    let tracks = db::get_setlist_tracks(pool, id)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?;

    // M5: Handle 0 tracks with 400 INVALID_REQUEST
    if tracks.is_empty() {
        return Err(SetlistError::InvalidRequest(
            "Setlist has no tracks to arrange".to_string(),
        ));
    }

    // M5: Handle 1 track — return as-is with perfect score
    if tracks.len() == 1 {
        let track = &tracks[0];
        let track_response = SetlistTrackResponse::from(track.clone());

        // Update harmonic flow score in DB
        db::update_setlist_harmonic_score(pool, id, 100.0)
            .await
            .map_err(|e| SetlistError::Database(e.to_string()))?;

        return Ok(SetlistResponse {
            id: setlist_row.id,
            prompt: setlist_row.prompt,
            model: setlist_row.model,
            tracks: vec![track_response],
            notes: setlist_row.notes,
            harmonic_flow_score: Some(100.0),
            score_breakdown: Some(ScoreBreakdownResponse {
                key_compatibility: 100.0,
                bpm_continuity: 100.0,
                energy_arc: 100.0,
            }),
            created_at: setlist_row
                .created_at
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        });
    }

    // Convert to arrangement tracks
    let arrangement_tracks: Vec<ArrangementTrack> = tracks
        .iter()
        .enumerate()
        .map(|(i, t)| ArrangementTrack {
            index: i,
            camelot: t.camelot.as_deref().and_then(parse_camelot),
            bpm: t.bpm,
            energy: t.energy.map(|e| e as i32),
        })
        .collect();

    // Run arrangement algorithm
    let result = arrangement::arrange_tracks(&arrangement_tracks);

    // Update positions and scores in DB
    for (new_pos, &original_idx) in result.ordered_indices.iter().enumerate() {
        let track = &tracks[original_idx];
        let t_score = if new_pos > 0 {
            Some(result.transition_scores[new_pos - 1])
        } else {
            None
        };
        db::update_setlist_track_position(pool, &track.id, (new_pos + 1) as i32, t_score)
            .await
            .map_err(|e| SetlistError::Database(e.to_string()))?;
    }

    // Update harmonic flow score
    db::update_setlist_harmonic_score(pool, id, result.harmonic_flow_score)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?;

    // Build response with new ordering
    let mut track_responses: Vec<SetlistTrackResponse> = Vec::with_capacity(tracks.len());
    for (new_pos, &original_idx) in result.ordered_indices.iter().enumerate() {
        let track = &tracks[original_idx];
        let t_score = if new_pos > 0 {
            Some(result.transition_scores[new_pos - 1])
        } else {
            None
        };
        track_responses.push(SetlistTrackResponse {
            position: (new_pos + 1) as i32,
            title: track.title.clone(),
            artist: track.artist.clone(),
            bpm: track.bpm,
            key: track.key.clone(),
            camelot: track.camelot.clone(),
            energy: track.energy,
            transition_note: track.transition_note.clone(),
            transition_score: t_score,
            original_position: track.original_position,
            source: track.source.clone(),
            track_id: track.track_id.clone(),
        });
    }

    // C1: Map score_breakdown from ArrangementResult into response
    Ok(SetlistResponse {
        id: setlist_row.id,
        prompt: setlist_row.prompt,
        model: setlist_row.model,
        tracks: track_responses,
        notes: setlist_row.notes,
        harmonic_flow_score: Some(result.harmonic_flow_score),
        score_breakdown: Some(ScoreBreakdownResponse {
            key_compatibility: result.score_breakdown.key_compatibility,
            bpm_continuity: result.score_breakdown.bpm_continuity,
            energy_arc: result.score_breakdown.energy_arc,
        }),
        created_at: setlist_row
            .created_at
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
    })
}

// ---------------------------------------------------------------------------
// Shared test mock (M6)
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::api::claude::{ClaudeClientTrait, ClaudeError};

    pub struct MockClaude {
        pub response: String,
    }

    #[async_trait::async_trait]
    impl ClaudeClientTrait for MockClaude {
        async fn generate_setlist(
            &self,
            _system_prompt: &str,
            _user_prompt: &str,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<String, ClaudeError> {
            Ok(self.response.clone())
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::claude::ClaudeClientTrait;
    use test_utils::MockClaude;

    struct MalformedClaude;

    #[async_trait::async_trait]
    impl ClaudeClientTrait for MalformedClaude {
        async fn generate_setlist(
            &self,
            _system_prompt: &str,
            _user_prompt: &str,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<String, ClaudeError> {
            Ok("This is not JSON at all!".to_string())
        }
    }

    struct RateLimitedClaude;

    #[async_trait::async_trait]
    impl ClaudeClientTrait for RateLimitedClaude {
        async fn generate_setlist(
            &self,
            _system_prompt: &str,
            _user_prompt: &str,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<String, ClaudeError> {
            Err(ClaudeError::RateLimited {
                retry_after_secs: 10,
            })
        }
    }

    struct TimeoutClaude;

    #[async_trait::async_trait]
    impl ClaudeClientTrait for TimeoutClaude {
        async fn generate_setlist(
            &self,
            _system_prompt: &str,
            _user_prompt: &str,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<String, ClaudeError> {
            Err(ClaudeError::Timeout)
        }
    }

    async fn setup_pool_with_tracks() -> sqlx::SqlitePool {
        let pool = crate::db::create_test_pool().await;

        sqlx::query(
            "INSERT INTO tracks (id, title, source, bpm, camelot_key) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("t1")
        .bind("Desert Rose")
        .bind("spotify")
        .bind(102.0)
        .bind("8A")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO artists (id, name) VALUES (?, ?)")
            .bind("a1")
            .bind("Sting")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES (?, ?)")
            .bind("t1")
            .bind("a1")
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    fn valid_llm_json(track_id: Option<&str>) -> String {
        let tid = track_id
            .map(|id| format!("\"{id}\""))
            .unwrap_or_else(|| "null".to_string());
        let source = if track_id.is_some() {
            "catalog"
        } else {
            "suggestion"
        };
        format!(
            r#"{{
            "tracks": [
                {{
                    "position": 1,
                    "title": "Desert Rose",
                    "artist": "Sting",
                    "bpm": 102.0,
                    "key": "A minor",
                    "camelot": "8A",
                    "energy": 5,
                    "transition_note": "Open with atmospheric pads",
                    "source": "{source}",
                    "track_id": {tid}
                }}
            ],
            "notes": "A chill desert set"
        }}"#
        )
    }

    #[tokio::test]
    async fn test_empty_prompt_returns_error() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: "{}".to_string(),
        };
        let result = generate_setlist(&pool, &claude, "user1", "", None).await;
        assert!(matches!(result, Err(SetlistError::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn test_long_prompt_returns_error() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: "{}".to_string(),
        };
        let long_prompt = "x".repeat(2001);
        let result = generate_setlist(&pool, &claude, "user1", &long_prompt, None).await;
        assert!(matches!(result, Err(SetlistError::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn test_empty_catalog_returns_error() {
        let pool = crate::db::create_test_pool().await; // empty
        let claude = MockClaude {
            response: "{}".to_string(),
        };
        let result = generate_setlist(&pool, &claude, "user1", "chill vibes", None).await;
        assert!(matches!(result, Err(SetlistError::EmptyCatalog)));
    }

    #[tokio::test]
    async fn test_valid_generation() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let result = generate_setlist(&pool, &claude, "user1", "chill vibes", None).await;
        let resp = result.unwrap();
        assert_eq!(resp.tracks.len(), 1);
        assert_eq!(resp.tracks[0].title, "Desert Rose");
        assert_eq!(resp.tracks[0].source, "catalog");
        assert_eq!(resp.tracks[0].track_id.as_deref(), Some("t1"));
        assert_eq!(resp.notes.as_deref(), Some("A chill desert set"));
        assert!(!resp.id.is_empty());
        // C1: generate should not have score_breakdown
        assert!(resp.score_breakdown.is_none());
    }

    #[tokio::test]
    async fn test_malformed_json_triggers_retry_then_fails() {
        let pool = setup_pool_with_tracks().await;
        let claude = MalformedClaude;
        let result = generate_setlist(&pool, &claude, "user1", "test", None).await;
        assert!(matches!(result, Err(SetlistError::GenerationFailed(_))));
    }

    #[tokio::test]
    async fn test_hallucinated_track_id_reclassified() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(Some("nonexistent-id")),
        };
        let result = generate_setlist(&pool, &claude, "user1", "test", None).await;
        let resp = result.unwrap();
        assert_eq!(resp.tracks[0].source, "suggestion");
        assert!(resp.tracks[0].track_id.is_none());
    }

    #[tokio::test]
    async fn test_get_setlist_not_found() {
        let pool = crate::db::create_test_pool().await;
        let result = get_setlist(&pool, "nonexistent").await;
        assert!(matches!(result, Err(SetlistError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_get_setlist_returns_tracks_in_order() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let gen_result = generate_setlist(&pool, &claude, "user1", "test", None)
            .await
            .unwrap();

        let fetched = get_setlist(&pool, &gen_result.id).await.unwrap();
        assert_eq!(fetched.tracks.len(), 1);
        assert_eq!(fetched.tracks[0].title, "Desert Rose");
    }

    #[tokio::test]
    async fn test_db_round_trip() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let result = generate_setlist(&pool, &claude, "user1", "deep house", None)
            .await
            .unwrap();

        // Verify via direct DB query
        let row = db::get_setlist(&pool, &result.id).await.unwrap().unwrap();
        assert_eq!(row.prompt, "deep house");
        assert_eq!(row.model, DEFAULT_MODEL);

        let tracks = db::get_setlist_tracks(&pool, &result.id).await.unwrap();
        assert_eq!(tracks.len(), 1);
    }

    #[tokio::test]
    async fn test_catalog_loading_returns_tracks() {
        let pool = setup_pool_with_tracks().await;
        let catalog = db::load_catalog_tracks(&pool).await.unwrap();
        assert!(!catalog.is_empty());
        assert_eq!(catalog[0].title, "Desert Rose");
        assert_eq!(catalog[0].bpm, Some(102.0));
    }

    #[tokio::test]
    async fn test_markdown_fenced_response_parses() {
        let pool = setup_pool_with_tracks().await;
        let fenced = format!("```json\n{}\n```", valid_llm_json(Some("t1")));
        let claude = MockClaude { response: fenced };
        let result = generate_setlist(&pool, &claude, "user1", "test", None).await;
        assert!(result.is_ok());
    }

    // H1: Test rate-limited Claude maps to ServiceBusy
    #[tokio::test]
    async fn test_rate_limited_claude_maps_to_service_busy() {
        let pool = setup_pool_with_tracks().await;
        let claude = RateLimitedClaude;
        let result = generate_setlist(&pool, &claude, "user1", "test", None).await;
        assert!(matches!(result, Err(SetlistError::ServiceBusy(_))));
    }

    // H1: Test timeout Claude maps to Timeout
    #[tokio::test]
    async fn test_timeout_claude_maps_to_timeout() {
        let pool = setup_pool_with_tracks().await;
        let claude = TimeoutClaude;
        let result = generate_setlist(&pool, &claude, "user1", "test", None).await;
        assert!(matches!(result, Err(SetlistError::Timeout)));
    }

    // M2: Test track_count validation
    #[tokio::test]
    async fn test_track_count_zero_returns_error() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let result = generate_setlist(&pool, &claude, "user1", "test", Some(0)).await;
        assert!(matches!(result, Err(SetlistError::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn test_track_count_over_50_returns_error() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let result = generate_setlist(&pool, &claude, "user1", "test", Some(51)).await;
        assert!(matches!(result, Err(SetlistError::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn test_track_count_valid_range() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let result = generate_setlist(&pool, &claude, "user1", "test", Some(5)).await;
        assert!(result.is_ok());
    }

    // M3: Test LLM entries with missing fields
    #[tokio::test]
    async fn test_llm_empty_title_filtered() {
        let pool = setup_pool_with_tracks().await;
        // One valid entry and one with empty title
        let response = r#"{
            "tracks": [
                {
                    "position": 1,
                    "title": "Desert Rose",
                    "artist": "Sting",
                    "bpm": 102.0,
                    "energy": 5,
                    "source": "suggestion"
                },
                {
                    "position": 2,
                    "title": "",
                    "artist": "Unknown",
                    "bpm": null,
                    "energy": null,
                    "source": "suggestion"
                }
            ],
            "notes": "test"
        }"#;
        let claude = MockClaude {
            response: response.to_string(),
        };
        let result = generate_setlist(&pool, &claude, "user1", "test", None).await;
        let resp = result.unwrap();
        assert_eq!(resp.tracks.len(), 1);
        assert_eq!(resp.tracks[0].title, "Desert Rose");
    }

    #[tokio::test]
    async fn test_llm_majority_invalid_returns_error() {
        let pool = setup_pool_with_tracks().await;
        // Two entries with empty title, one valid
        let response = r#"{
            "tracks": [
                {
                    "position": 1,
                    "title": "",
                    "artist": "A",
                    "bpm": null,
                    "energy": null,
                    "source": "suggestion"
                },
                {
                    "position": 2,
                    "title": "",
                    "artist": "B",
                    "bpm": null,
                    "energy": null,
                    "source": "suggestion"
                },
                {
                    "position": 3,
                    "title": "Good Track",
                    "artist": "Artist",
                    "bpm": 128.0,
                    "energy": 5,
                    "source": "suggestion"
                }
            ],
            "notes": "test"
        }"#;
        let claude = MockClaude {
            response: response.to_string(),
        };
        let result = generate_setlist(&pool, &claude, "user1", "test", None).await;
        assert!(matches!(result, Err(SetlistError::GenerationFailed(_))));
    }

    // H5: Energy stored as f64
    #[tokio::test]
    async fn test_energy_stored_as_f64() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let result = generate_setlist(&pool, &claude, "user1", "test", None)
            .await
            .unwrap();
        // Energy 5 from LLM should be stored as 5.0
        assert_eq!(result.tracks[0].energy, Some(5.0));
    }
}
