use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::api::claude::{
    build_enhanced_system_prompt, build_enhanced_user_prompt, strip_markdown_fences,
    ClaudeClientTrait, ClaudeError, LlmSetlistResponse,
};
use crate::db::imports as db_imports;
use crate::db::models::{SetlistRow, SetlistTrackRow, TrackRow};
use crate::db::setlists as db;
use crate::services::arrangement::{self, ArrangementTrack};
use crate::services::camelot::{parse_camelot, EnergyProfile};

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

    #[error("Invalid energy profile: {0}")]
    InvalidEnergyProfile(String),

    #[error("Invalid BPM range: {0}")]
    InvalidBpmRange(String),

    #[error("Playlist not found: {0}")]
    PlaylistNotFound(String),
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
            SetlistError::InvalidEnergyProfile(m) => {
                (StatusCode::BAD_REQUEST, "INVALID_ENERGY_PROFILE", m.clone())
            }
            SetlistError::InvalidBpmRange(m) => {
                (StatusCode::BAD_REQUEST, "INVALID_BPM_RANGE", m.clone())
            }
            SetlistError::PlaylistNotFound(m) => {
                (StatusCode::NOT_FOUND, "PLAYLIST_NOT_FOUND", m.clone())
            }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_warning: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bpm_warnings: Vec<BpmWarning>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BpmWarning {
    pub from_position: i32,
    pub to_position: i32,
    pub bpm_delta: f64,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

pub struct GenerateSetlistRequest {
    pub user_id: String,
    pub prompt: String,
    pub track_count: Option<u32>,
    pub energy_profile: Option<EnergyProfile>,
    pub source_playlist_id: Option<String>,
    pub seed_tracklist: Option<String>,
    pub creative_mode: Option<bool>,
    pub bpm_range: Option<BpmRange>,
    pub verify: bool,
}

pub struct BpmRange {
    pub min: f64,
    pub max: f64,
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
    pub spotify_uri: Option<String>,
    pub confidence: Option<String>,
    pub verification_flag: Option<String>,
    pub verification_note: Option<String>,
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
            spotify_uri: row.spotify_uri,
            confidence: row.confidence,
            verification_flag: row.verification_flag,
            verification_note: row.verification_note,
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

/// Legacy generate_setlist function — delegates to the new request-based overload for backward compat.
pub async fn generate_setlist(
    pool: &sqlx::SqlitePool,
    claude: &dyn ClaudeClientTrait,
    user_id: &str,
    prompt: &str,
    track_count: Option<u32>,
) -> Result<SetlistResponse, SetlistError> {
    let req = GenerateSetlistRequest {
        user_id: user_id.to_string(),
        prompt: prompt.to_string(),
        track_count,
        energy_profile: None,
        source_playlist_id: None,
        seed_tracklist: None,
        creative_mode: None,
        bpm_range: None,
        verify: false,
    };
    generate_setlist_from_request(pool, claude, req).await
}

pub async fn generate_setlist_from_request(
    pool: &sqlx::SqlitePool,
    claude: &dyn ClaudeClientTrait,
    req: GenerateSetlistRequest,
) -> Result<SetlistResponse, SetlistError> {
    // Validate prompt
    let prompt = req.prompt.trim().to_string();
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
    let count = match req.track_count {
        Some(c) if !(MIN_TRACK_COUNT..=MAX_TRACK_COUNT).contains(&c) => {
            return Err(SetlistError::InvalidRequest(format!(
                "track_count must be between {MIN_TRACK_COUNT} and {MAX_TRACK_COUNT}, got {c}"
            )));
        }
        Some(c) => c,
        None => DEFAULT_TRACK_COUNT,
    };

    // Validate BPM range
    if let Some(ref bpm_range) = req.bpm_range {
        if bpm_range.min < 60.0 || bpm_range.max > 200.0 || bpm_range.min > bpm_range.max {
            return Err(SetlistError::InvalidBpmRange(format!(
                "BPM range must be min >= 60, max <= 200, min <= max. Got min={}, max={}",
                bpm_range.min, bpm_range.max
            )));
        }
    }

    // Catalog loading: source_playlist_id filtering or full catalog
    let mut extra_notes: Vec<String> = Vec::new();
    let catalog = if let Some(ref playlist_id) = req.source_playlist_id {
        // Verify the import exists
        let import = db_imports::get_import(pool, playlist_id)
            .await
            .map_err(|e| SetlistError::Database(e.to_string()))?;
        if import.is_none() {
            return Err(SetlistError::PlaylistNotFound(format!(
                "Source playlist '{playlist_id}' not found"
            )));
        }

        let playlist_tracks = db_imports::get_tracks_by_import_id(pool, playlist_id)
            .await
            .map_err(|e| SetlistError::Database(e.to_string()))?;

        if playlist_tracks.is_empty() {
            return Err(SetlistError::EmptyCatalog);
        }

        // Extension 3c: Check if ALL tracks need enrichment (0 enriched)
        let enriched_count = count_enriched_tracks(&playlist_tracks);
        if enriched_count == 0 {
            extra_notes.push(
                "Source playlist tracks are not yet enriched. Using full catalog.".to_string(),
            );
            db::load_catalog_tracks(pool).await?
        } else {
            // Extension 24b: Check if SOME tracks still need enrichment
            let unenriched = playlist_tracks.len() - enriched_count;
            if unenriched > 0 {
                extra_notes.push(
                    "Some tracks are still being enriched. Generate again after enrichment completes for better results.".to_string(),
                );
            }
            playlist_tracks
        }
    } else {
        db::load_catalog_tracks(pool).await?
    };

    // Empty catalog is OK — LLM will generate purely from suggestions

    // Build catalog IDs set for validation
    let catalog_ids: std::collections::HashSet<String> =
        catalog.iter().map(|t| t.id.clone()).collect();

    // Build prompts using enhanced prompt builders
    let catalog_text = serialize_catalog(&catalog);
    let energy_profile_str = req.energy_profile.as_ref().map(|p| p.to_string());
    let creative_mode = req.creative_mode.unwrap_or(false);
    let bpm_range_tuple = req.bpm_range.as_ref().map(|r| (r.min, r.max));

    let system_blocks =
        build_enhanced_system_prompt(&catalog_text, energy_profile_str.as_deref(), creative_mode);
    let user_text = format!("Create a setlist of {count} tracks based on this prompt: {prompt}");
    let user_blocks =
        build_enhanced_user_prompt(&user_text, req.seed_tracklist.as_deref(), bpm_range_tuple);

    // Call Claude with enhanced blocks
    let (raw_response, _cache_metrics) = claude
        .generate_with_blocks(system_blocks.clone(), user_blocks, DEFAULT_MODEL, 4096)
        .await
        .map_err(SetlistError::from)?;

    // Parse response (with retry on failure)
    let cleaned = strip_markdown_fences(&raw_response);
    let llm_response: LlmSetlistResponse = match serde_json::from_str(cleaned) {
        Ok(r) => r,
        Err(first_err) => {
            // Retry with stricter prompt
            tracing::warn!("First parse failed: {first_err}, retrying with stricter prompt");
            let retry_text = format!(
                "{user_text}\n\nIMPORTANT: Respond with ONLY valid JSON. No markdown fences, no explanation text."
            );
            let retry_user_blocks = build_enhanced_user_prompt(
                &retry_text,
                req.seed_tracklist.as_deref(),
                bpm_range_tuple,
            );
            let (retry_response, _) = claude
                .generate_with_blocks(system_blocks, retry_user_blocks, DEFAULT_MODEL, 4096)
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
        user_id: req.user_id.clone(),
        prompt: prompt.clone(),
        model: DEFAULT_MODEL.to_string(),
        notes: llm_response.notes.clone(),
        harmonic_flow_score: None,
        energy_profile: req.energy_profile.as_ref().map(|p| p.to_string()),
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

    // Process tracks: build responses in memory first, verify optionally, then persist
    let mut track_responses = Vec::with_capacity(valid_entries.len());

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
            spotify_uri: None,
            confidence: entry
                .confidence
                .as_deref()
                .map(|c| c.to_lowercase())
                .filter(|c| matches!(c.as_str(), "high" | "medium" | "low")),
            verification_flag: None,
            verification_note: None,
        });
    }

    // Optional verification pass (SP-007)
    if req.verify {
        match verify_setlist(claude, &track_responses).await {
            Ok(verified) => {
                track_responses = verified;
            }
            Err(e) => {
                tracing::warn!("Verification failed, using unverified tracks: {e}");
                extra_notes.push(
                    "Verification unavailable, confidence scores are self-reported.".to_string(),
                );
            }
        }
    }

    // Persist tracks AFTER verification so DB has final state
    let mut any_track_write_failed = db_write_failed;
    if !db_write_failed {
        for track in &track_responses {
            let track_row = SetlistTrackRow {
                id: uuid::Uuid::new_v4().to_string(),
                setlist_id: setlist_id.clone(),
                track_id: track.track_id.clone(),
                position: track.position,
                original_position: track.original_position,
                title: track.title.clone(),
                artist: track.artist.clone(),
                bpm: track.bpm,
                key: track.key.clone(),
                camelot: track.camelot.clone(),
                energy: track.energy,
                transition_note: track.transition_note.clone(),
                transition_score: track.transition_score,
                source: track.source.clone(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: track.confidence.clone(),
                verification_flag: track.verification_flag.clone(),
                verification_note: track.verification_note.clone(),
            };
            if let Err(e) = db::insert_setlist_track(pool, &track_row).await {
                tracing::error!("Failed to persist setlist track: {e}");
                any_track_write_failed = true;
            }
        }
    }

    // Build notes with possible DB warning and extra notes
    let mut notes = llm_response.notes;
    if !extra_notes.is_empty() {
        let extra = extra_notes.join(" ");
        notes = Some(match notes {
            Some(n) => format!("{n} {extra}"),
            None => extra,
        });
    }
    if any_track_write_failed {
        let warning = " [WARNING: Setlist was not saved to database. You may want to regenerate.]";
        notes = Some(match notes {
            Some(n) => format!("{n}{warning}"),
            None => warning.to_string(),
        });
    }

    // Seed match count (postcondition 13)
    if let Some(seed_text) = &req.seed_tracklist {
        let track_rows: Vec<SetlistTrackRow> = track_responses
            .iter()
            .map(|r| SetlistTrackRow {
                id: String::new(),
                setlist_id: String::new(),
                track_id: r.track_id.clone(),
                position: r.position,
                original_position: r.original_position,
                title: r.title.clone(),
                artist: r.artist.clone(),
                bpm: r.bpm,
                key: r.key.clone(),
                camelot: r.camelot.clone(),
                energy: r.energy,
                transition_note: r.transition_note.clone(),
                transition_score: r.transition_score,
                source: r.source.clone(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: r.confidence.clone(),
                verification_flag: None,
                verification_note: None,
            })
            .collect();
        let match_count = compute_seed_match_count(seed_text, &track_rows);
        let seed_line_count = seed_text.lines().filter(|l| !l.trim().is_empty()).count();
        let seed_note =
            format!("{match_count} of {seed_line_count} seed tracks matched your catalog.");
        notes = Some(match notes {
            Some(n) => format!("{n} {seed_note}"),
            None => seed_note,
        });
    }

    // Quality validation
    let catalog_percentage = compute_catalog_percentage(&track_responses);
    let catalog_warning = compute_catalog_warning(catalog_percentage);
    let bpm_warnings = compute_bpm_warnings_from_responses(&track_responses);

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
        prompt: prompt.clone(),
        model: DEFAULT_MODEL.to_string(),
        tracks: track_responses,
        notes,
        harmonic_flow_score: None,
        score_breakdown: None,
        created_at,
        energy_profile: req.energy_profile.as_ref().map(|p| p.to_string()),
        catalog_percentage: Some(catalog_percentage),
        catalog_warning,
        bpm_warnings,
    })
}

/// Count tracks that have BPM, key, or energy data (i.e., are enriched).
fn count_enriched_tracks(tracks: &[TrackRow]) -> usize {
    tracks
        .iter()
        .filter(|t| t.bpm.is_some() || t.camelot_key.is_some() || t.energy.is_some())
        .count()
}

pub async fn get_setlist(
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<SetlistResponse, SetlistError> {
    let setlist = db::get_setlist(pool, id)
        .await?
        .ok_or_else(|| SetlistError::NotFound(format!("Setlist {id} not found")))?;

    let tracks = db::get_setlist_tracks(pool, id).await?;
    let track_responses: Vec<SetlistTrackResponse> =
        tracks.into_iter().map(SetlistTrackResponse::from).collect();

    let bpm_warnings = compute_bpm_warnings_from_responses(&track_responses);
    let catalog_percentage = compute_catalog_percentage(&track_responses);
    let catalog_warning = compute_catalog_warning(catalog_percentage);

    Ok(SetlistResponse {
        id: setlist.id,
        prompt: setlist.prompt,
        model: setlist.model,
        tracks: track_responses,
        notes: setlist.notes,
        harmonic_flow_score: setlist.harmonic_flow_score,
        score_breakdown: None,
        created_at: setlist
            .created_at
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        energy_profile: setlist.energy_profile,
        catalog_percentage: Some(catalog_percentage),
        catalog_warning,
        bpm_warnings,
    })
}

/// L1: Extracted arrange logic from route handler into service layer.
/// Loads setlist, runs arrangement algorithm, persists results, returns response.
///
/// If `energy_profile` is `Some`, uses that profile for arrangement scoring.
/// If `None`, falls back to the stored `energy_profile` on the setlist row.
/// If neither exists (pre-ST-006 setlists), uses default energy arc scoring.
pub async fn arrange_setlist(
    pool: &sqlx::SqlitePool,
    id: &str,
    energy_profile: Option<EnergyProfile>,
) -> Result<SetlistResponse, SetlistError> {
    // Load the setlist
    let setlist_row = db::get_setlist(pool, id)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?
        .ok_or_else(|| SetlistError::NotFound(format!("Setlist {id} not found")))?;

    let tracks = db::get_setlist_tracks(pool, id)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?;

    // Resolve energy profile: explicit param > stored on setlist > None (default)
    let resolved_profile = energy_profile.or_else(|| {
        setlist_row
            .energy_profile
            .as_deref()
            .and_then(|s| s.parse::<EnergyProfile>().ok())
    });

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
            energy_profile: setlist_row.energy_profile,
            catalog_percentage: None,
            catalog_warning: None,
            bpm_warnings: vec![],
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

    // Run arrangement algorithm with resolved energy profile
    let result = arrangement::arrange_tracks(&arrangement_tracks, resolved_profile);

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
            spotify_uri: track.spotify_uri.clone(),
            confidence: None,
            verification_flag: None,
            verification_note: None,
        });
    }

    // C1: Map score_breakdown from ArrangementResult into response
    let bpm_warnings = compute_bpm_warnings_from_responses(&track_responses);

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
        energy_profile: setlist_row.energy_profile,
        catalog_percentage: None,
        catalog_warning: None,
        bpm_warnings,
    })
}

// ---------------------------------------------------------------------------
// Quality validation (T9)
// ---------------------------------------------------------------------------

/// Compute BPM warnings from SetlistTrackRow slices (used by get_setlist and arrange).
pub fn compute_bpm_warnings(tracks: &[SetlistTrackRow]) -> Vec<BpmWarning> {
    let mut warnings = Vec::new();
    for i in 0..tracks.len().saturating_sub(1) {
        if let (Some(bpm_a), Some(bpm_b)) = (tracks[i].bpm, tracks[i + 1].bpm) {
            let delta = bpm_b - bpm_a;
            if delta.abs() > 6.0 {
                warnings.push(BpmWarning {
                    from_position: tracks[i].position,
                    to_position: tracks[i + 1].position,
                    bpm_delta: delta,
                });
            }
        }
    }
    warnings
}

/// Compute BPM warnings from SetlistTrackResponse slices (used during generation).
fn compute_bpm_warnings_from_responses(tracks: &[SetlistTrackResponse]) -> Vec<BpmWarning> {
    let mut warnings = Vec::new();
    for i in 0..tracks.len().saturating_sub(1) {
        if let (Some(bpm_a), Some(bpm_b)) = (tracks[i].bpm, tracks[i + 1].bpm) {
            let delta = bpm_b - bpm_a;
            if delta.abs() > 6.0 {
                warnings.push(BpmWarning {
                    from_position: tracks[i].position,
                    to_position: tracks[i + 1].position,
                    bpm_delta: delta,
                });
            }
        }
    }
    warnings
}

/// Compute catalog percentage: count of tracks with source == "catalog" / total × 100.
pub fn compute_catalog_percentage(tracks: &[SetlistTrackResponse]) -> f64 {
    if tracks.is_empty() {
        return 0.0;
    }
    let catalog_count = tracks.iter().filter(|t| t.source == "catalog").count();
    (catalog_count as f64 / tracks.len() as f64) * 100.0
}

/// Compute catalog warning: returns warning string if percentage < 30%.
pub fn compute_catalog_warning(percentage: f64) -> Option<String> {
    if percentage < 30.0 {
        Some(format!(
            "Only {percentage:.0}% of tracks are from your catalog. Consider importing more music."
        ))
    } else {
        None
    }
}

/// Count how many seed entries match tracks by title or artist (case-insensitive fuzzy match).
pub fn compute_seed_match_count(seed_text: &str, tracks: &[SetlistTrackRow]) -> u32 {
    let seed_lines: Vec<String> = seed_text
        .lines()
        .map(|l| l.trim().to_lowercase())
        .filter(|l| !l.is_empty())
        .collect();

    let mut count = 0u32;
    for line in &seed_lines {
        for track in tracks {
            let title_lower = track.title.to_lowercase();
            let artist_lower = track.artist.to_lowercase();
            if line.contains(&title_lower)
                || title_lower.contains(line.as_str())
                || line.contains(&artist_lower)
                || artist_lower.contains(line.as_str())
            {
                count += 1;
                break;
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Verification loop (SP-007)
// ---------------------------------------------------------------------------

/// Verification prompt for the second-pass fact-checker.
const VERIFICATION_PROMPT: &str = include_str!("../prompts/verification_prompt.md");

#[derive(Debug, Clone, serde::Deserialize)]
pub struct VerificationResponse {
    pub tracks: Vec<VerificationEntry>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct VerificationEntry {
    pub position: i32,
    pub title: String,
    pub artist: String,
    pub original_title: Option<String>,
    pub original_artist: Option<String>,
    pub confidence: Option<String>,
    pub flag: Option<String>,
    pub correction: Option<String>,
}

/// Run a second-pass verification on a generated setlist.
/// Returns the original tracks with confidence adjusted and flags applied.
pub async fn verify_setlist(
    claude: &dyn ClaudeClientTrait,
    tracks: &[SetlistTrackResponse],
) -> Result<Vec<SetlistTrackResponse>, SetlistError> {
    // Build a simplified JSON of the tracks for the fact-checker
    let tracks_for_review: Vec<serde_json::Value> = tracks
        .iter()
        .map(|t| {
            serde_json::json!({
                "position": t.position,
                "title": t.title,
                "artist": t.artist,
                "bpm": t.bpm,
                "key": t.key,
                "energy": t.energy,
                "confidence": t.confidence,
            })
        })
        .collect();

    let user_prompt = serde_json::to_string_pretty(&serde_json::json!({
        "tracks": tracks_for_review,
    }))
    .unwrap_or_default();

    let response = claude
        .generate_setlist(VERIFICATION_PROMPT, &user_prompt, DEFAULT_MODEL, 4096)
        .await
        .map_err(SetlistError::from)?;

    let cleaned = crate::api::claude::strip_markdown_fences(&response);
    let verification: VerificationResponse = match serde_json::from_str(cleaned) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Verification parse failed: {e}, skipping verification");
            return Ok(tracks.to_vec());
        }
    };

    // Build a lookup from position to verification entry
    let verify_map: std::collections::HashMap<i32, &VerificationEntry> = verification
        .tracks
        .iter()
        .map(|v| (v.position, v))
        .collect();

    // Apply verification results to original tracks
    let mut verified_tracks = tracks.to_vec();
    for track in &mut verified_tracks {
        if let Some(v) = verify_map.get(&track.position) {
            // Update confidence from verification
            if let Some(ref conf) = v.confidence {
                track.confidence = Some(conf.clone());
            }
            // If the verifier flagged and replaced, update title/artist
            if v.flag.as_deref() == Some("replaced") {
                let original_title = track.title.clone();
                let original_artist = track.artist.clone();
                tracing::info!(
                    "Verification replaced pos {}: '{}' by '{}' → '{}' by '{}'",
                    track.position,
                    original_title,
                    original_artist,
                    v.title,
                    v.artist
                );
                track.title = v.title.clone();
                track.artist = v.artist.clone();
                // Replacement tracks are at best medium confidence
                if track.confidence.as_deref() != Some("low") {
                    track.confidence = Some("medium".to_string());
                }
                track.verification_note = Some(format!(
                    "Replaced: was '{}' by '{}'",
                    original_title, original_artist
                ));
            }
            // If flagged as wrong_artist or no_such_track, downgrade confidence
            if matches!(
                v.flag.as_deref(),
                Some("wrong_artist") | Some("no_such_track") | Some("constructed_title")
            ) {
                track.confidence = Some("low".to_string());
                track.verification_note = v.correction.clone();
                tracing::warn!(
                    "Verification flagged pos {}: '{}' by '{}' as {:?} — {}",
                    track.position,
                    track.title,
                    track.artist,
                    v.flag,
                    v.correction.as_deref().unwrap_or("no correction")
                );
            }
            // Propagate the flag to the response
            track.verification_flag = v.flag.clone();
        }
    }

    // Warn about position mismatches
    let unmatched: Vec<i32> = verification
        .tracks
        .iter()
        .filter(|v| !tracks.iter().any(|t| t.position == v.position))
        .map(|v| v.position)
        .collect();
    if !unmatched.is_empty() {
        tracing::warn!(
            "Verification returned {} tracks with positions not in input: {:?}",
            unmatched.len(),
            unmatched
        );
    }

    if let Some(ref summary) = verification.summary {
        tracing::info!("Verification summary: {summary}");
    }

    Ok(verified_tracks)
}

// ---------------------------------------------------------------------------
// Shared test mock (M6)
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::api::claude::{CacheMetrics, ClaudeClientTrait, ClaudeError, RequestContentBlock};

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

        async fn generate_with_blocks(
            &self,
            _system_blocks: Vec<RequestContentBlock>,
            _user_blocks: Vec<RequestContentBlock>,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<(String, CacheMetrics), ClaudeError> {
            Ok((self.response.clone(), CacheMetrics::default()))
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

        async fn generate_with_blocks(
            &self,
            _system_blocks: Vec<crate::api::claude::RequestContentBlock>,
            _user_blocks: Vec<crate::api::claude::RequestContentBlock>,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<(String, crate::api::claude::CacheMetrics), ClaudeError> {
            Ok((
                "This is not JSON at all!".to_string(),
                crate::api::claude::CacheMetrics::default(),
            ))
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

        async fn generate_with_blocks(
            &self,
            _system_blocks: Vec<crate::api::claude::RequestContentBlock>,
            _user_blocks: Vec<crate::api::claude::RequestContentBlock>,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<(String, crate::api::claude::CacheMetrics), ClaudeError> {
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

        async fn generate_with_blocks(
            &self,
            _system_blocks: Vec<crate::api::claude::RequestContentBlock>,
            _user_blocks: Vec<crate::api::claude::RequestContentBlock>,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<(String, crate::api::claude::CacheMetrics), ClaudeError> {
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
    async fn test_empty_catalog_generates_suggestions() {
        let pool = crate::db::create_test_pool().await; // empty catalog
        let claude = MockClaude {
            response: valid_llm_json(None), // track_id=null → all suggestions
        };
        let result = generate_setlist(&pool, &claude, "user1", "chill vibes", None).await;
        let resp = result.unwrap();
        assert!(!resp.tracks.is_empty());
        // All tracks should be suggestions since catalog is empty
        for track in &resp.tracks {
            assert_eq!(track.source, "suggestion");
        }
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

    // --- T7: arrange_setlist with EnergyProfile ---

    /// Helper: create a setlist with multiple tracks for arrangement tests.
    /// Returns (pool, setlist_id).
    async fn setup_setlist_for_arrange(energy_profile: Option<&str>) -> (sqlx::SqlitePool, String) {
        let pool = setup_pool_with_tracks().await;
        let setlist_id = uuid::Uuid::new_v4().to_string();

        let setlist_row = SetlistRow {
            id: setlist_id.clone(),
            user_id: "user1".to_string(),
            prompt: "test arrange".to_string(),
            model: "test-model".to_string(),
            notes: None,
            harmonic_flow_score: None,
            energy_profile: energy_profile.map(|s| s.to_string()),
            created_at: None,
        };
        db::insert_setlist(&pool, &setlist_row).await.unwrap();

        // Insert 5 tracks with varied energy/BPM/key for meaningful arrangement
        let track_data = [
            ("t7-1", "Track Low", "Artist A", 120.0, "8A", 2.0),
            ("t7-2", "Track High", "Artist B", 128.0, "9A", 9.0),
            ("t7-3", "Track Mid", "Artist C", 124.0, "10A", 5.0),
            ("t7-4", "Track MidLow", "Artist D", 122.0, "7A", 3.0),
            ("t7-5", "Track MidHigh", "Artist E", 130.0, "8A", 7.0),
        ];

        for (i, (id, title, artist, bpm, camelot, energy)) in track_data.iter().enumerate() {
            let track_row = SetlistTrackRow {
                id: id.to_string(),
                setlist_id: setlist_id.clone(),
                track_id: None,
                position: (i + 1) as i32,
                original_position: (i + 1) as i32,
                title: title.to_string(),
                artist: artist.to_string(),
                bpm: Some(*bpm),
                key: None,
                camelot: Some(camelot.to_string()),
                energy: Some(*energy),
                transition_note: None,
                transition_score: None,
                source: "suggestion".to_string(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: None,
                verification_flag: None,
                verification_note: None,
            };
            db::insert_setlist_track(&pool, &track_row).await.unwrap();
        }

        (pool, setlist_id)
    }

    #[tokio::test]
    async fn test_arrange_with_explicit_profile() {
        let (pool, id) = setup_setlist_for_arrange(None).await;
        let result = arrange_setlist(&pool, &id, Some(EnergyProfile::WarmUp))
            .await
            .unwrap();
        assert_eq!(result.tracks.len(), 5);
        assert!(result.harmonic_flow_score.is_some());
    }

    #[tokio::test]
    async fn test_arrange_reads_stored_profile() {
        let (pool, id) = setup_setlist_for_arrange(Some("warm-up")).await;

        // Call with None — should read "warm-up" from stored setlist
        let result_stored = arrange_setlist(&pool, &id, None).await.unwrap();

        // Call with explicit WarmUp — should produce same result
        let result_explicit = arrange_setlist(&pool, &id, Some(EnergyProfile::WarmUp))
            .await
            .unwrap();

        // Both should produce the same track ordering
        let stored_order: Vec<String> = result_stored
            .tracks
            .iter()
            .map(|t| t.title.clone())
            .collect();
        let explicit_order: Vec<String> = result_explicit
            .tracks
            .iter()
            .map(|t| t.title.clone())
            .collect();
        assert_eq!(stored_order, explicit_order);
    }

    #[tokio::test]
    async fn test_arrange_pre_st006_setlist_uses_default() {
        // No stored profile, no explicit profile → default energy arc
        let (pool, id) = setup_setlist_for_arrange(None).await;
        let result = arrange_setlist(&pool, &id, None).await.unwrap();
        assert_eq!(result.tracks.len(), 5);
        assert!(result.harmonic_flow_score.is_some());
        // Should still produce a valid arrangement
        assert!(result.harmonic_flow_score.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_arrange_explicit_profile_overrides_stored() {
        // Store "steady" but pass "warm-up" explicitly
        let (pool, id) = setup_setlist_for_arrange(Some("steady")).await;

        let result_override = arrange_setlist(&pool, &id, Some(EnergyProfile::WarmUp))
            .await
            .unwrap();
        let result_stored = arrange_setlist(&pool, &id, None).await.unwrap();

        let override_order: Vec<String> = result_override
            .tracks
            .iter()
            .map(|t| t.title.clone())
            .collect();
        let stored_order: Vec<String> = result_stored
            .tracks
            .iter()
            .map(|t| t.title.clone())
            .collect();

        // The two profiles should produce different energy arc scores at minimum,
        // which may produce different orderings
        assert!(
            override_order != stored_order
                || result_override.score_breakdown.as_ref().unwrap().energy_arc
                    != result_stored.score_breakdown.as_ref().unwrap().energy_arc,
            "Explicit profile should override stored profile — at least energy_arc scores should differ"
        );
    }

    #[tokio::test]
    async fn test_arrange_different_profiles_affect_scoring() {
        let (pool, id) = setup_setlist_for_arrange(None).await;

        let result_warmup = arrange_setlist(&pool, &id, Some(EnergyProfile::WarmUp))
            .await
            .unwrap();
        let result_peaktime = arrange_setlist(&pool, &id, Some(EnergyProfile::PeakTime))
            .await
            .unwrap();

        // Different profiles should produce different energy arc scores
        let warmup_energy = result_warmup.score_breakdown.as_ref().unwrap().energy_arc;
        let peaktime_energy = result_peaktime.score_breakdown.as_ref().unwrap().energy_arc;

        // With tracks having energy 2,9,5,3,7 — WarmUp (3→7) and PeakTime (7→9→7)
        // should score the same arrangement differently
        assert!(
            (warmup_energy - peaktime_energy).abs() > 0.01,
            "WarmUp energy_arc={warmup_energy:.2} vs PeakTime energy_arc={peaktime_energy:.2} should differ"
        );
    }

    // -----------------------------------------------------------------------
    // T6: Extended generate_setlist_from_request tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_generate_with_energy_profile_stored() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "warm up set".to_string(),
            track_count: None,
            energy_profile: Some(EnergyProfile::WarmUp),
            source_playlist_id: None,
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();

        // Energy profile should be on the response
        assert_eq!(resp.energy_profile.as_deref(), Some("warm-up"));

        // Energy profile should be persisted in DB
        let row = db::get_setlist(&pool, &resp.id).await.unwrap().unwrap();
        assert_eq!(row.energy_profile.as_deref(), Some("warm-up"));
    }

    #[tokio::test]
    async fn test_generate_with_source_playlist_filters_catalog() {
        let pool = setup_pool_with_tracks().await;
        let user_id = crate::db::create_test_user(&pool).await;

        // Create an import with linked tracks
        crate::db::imports::create_import(&pool, "imp-gen", &user_id, "pl1", Some("Test PL"))
            .await
            .unwrap();
        crate::db::imports::insert_import_tracks(&pool, "imp-gen", &["t1".to_string()])
            .await
            .unwrap();

        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let req = GenerateSetlistRequest {
            user_id: user_id.clone(),
            prompt: "test playlist filter".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: Some("imp-gen".to_string()),
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();
        assert!(!resp.tracks.is_empty());
    }

    #[tokio::test]
    async fn test_generate_with_empty_filtered_catalog() {
        let pool = setup_pool_with_tracks().await;
        let user_id = crate::db::create_test_user(&pool).await;

        // Create an import with no linked tracks
        crate::db::imports::create_import(&pool, "imp-empty", &user_id, "pl2", None)
            .await
            .unwrap();

        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: user_id.clone(),
            prompt: "test".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: Some("imp-empty".to_string()),
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        let result = generate_setlist_from_request(&pool, &claude, req).await;
        assert!(matches!(result, Err(SetlistError::EmptyCatalog)));
    }

    #[tokio::test]
    async fn test_generate_with_nonexistent_playlist() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "test".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: Some("nonexistent-playlist".to_string()),
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        let result = generate_setlist_from_request(&pool, &claude, req).await;
        assert!(matches!(result, Err(SetlistError::PlaylistNotFound(_))));
    }

    #[tokio::test]
    async fn test_generate_ext_3c_unenriched_falls_back_to_full_catalog() {
        let pool = setup_pool_with_tracks().await;
        let user_id = crate::db::create_test_user(&pool).await;

        // Create a track with no BPM/key/energy (unenriched)
        sqlx::query("INSERT INTO tracks (id, title, source) VALUES (?, ?, ?)")
            .bind("t-unenriched")
            .bind("Unenriched Track")
            .bind("spotify")
            .execute(&pool)
            .await
            .unwrap();

        // Create import linking only the unenriched track
        crate::db::imports::create_import(&pool, "imp-unenriched", &user_id, "pl3", None)
            .await
            .unwrap();
        crate::db::imports::insert_import_tracks(
            &pool,
            "imp-unenriched",
            &["t-unenriched".to_string()],
        )
        .await
        .unwrap();

        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let req = GenerateSetlistRequest {
            user_id: user_id.clone(),
            prompt: "test ext 3c".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: Some("imp-unenriched".to_string()),
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();

        // Should fall back to full catalog and include warning
        assert!(resp
            .notes
            .as_deref()
            .unwrap_or("")
            .contains("not yet enriched"));
        assert!(resp
            .notes
            .as_deref()
            .unwrap_or("")
            .contains("Using full catalog"),);
    }

    #[tokio::test]
    async fn test_generate_ext_24b_partial_enrichment_warning() {
        let pool = setup_pool_with_tracks().await;
        let user_id = crate::db::create_test_user(&pool).await;

        // t1 is already enriched (has BPM, key). Add an unenriched track.
        sqlx::query("INSERT INTO tracks (id, title, source) VALUES (?, ?, ?)")
            .bind("t-partial")
            .bind("Partial Track")
            .bind("spotify")
            .execute(&pool)
            .await
            .unwrap();

        // Link both to import
        crate::db::imports::create_import(&pool, "imp-partial", &user_id, "pl4", None)
            .await
            .unwrap();
        crate::db::imports::insert_import_tracks(
            &pool,
            "imp-partial",
            &["t1".to_string(), "t-partial".to_string()],
        )
        .await
        .unwrap();

        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let req = GenerateSetlistRequest {
            user_id: user_id.clone(),
            prompt: "test ext 24b".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: Some("imp-partial".to_string()),
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();

        // Should include partial enrichment warning
        assert!(resp
            .notes
            .as_deref()
            .unwrap_or("")
            .contains("still being enriched"));
    }

    #[tokio::test]
    async fn test_generate_with_seed_tracklist() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "house set".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: None,
            seed_tracklist: Some(
                "1. Daft Punk - Around the World\n2. Chemical Brothers".to_string(),
            ),
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        // Should not error — seed_tracklist is passed to Claude prompt
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();
        assert!(!resp.tracks.is_empty());
    }

    #[tokio::test]
    async fn test_generate_with_creative_mode() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "creative set".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: None,
            seed_tracklist: None,
            creative_mode: Some(true),
            bpm_range: None,
            verify: false,
        };
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();
        assert!(!resp.tracks.is_empty());
    }

    #[tokio::test]
    async fn test_generate_with_bpm_range() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "constrained bpm".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: None,
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: Some(BpmRange {
                min: 120.0,
                max: 130.0,
            }),
            verify: false,
        };
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();
        assert!(!resp.tracks.is_empty());
    }

    #[tokio::test]
    async fn test_generate_with_invalid_bpm_range_min_too_low() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "test".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: None,
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: Some(BpmRange {
                min: 50.0,
                max: 130.0,
            }),
            verify: false,
        };
        let result = generate_setlist_from_request(&pool, &claude, req).await;
        assert!(matches!(result, Err(SetlistError::InvalidBpmRange(_))));
    }

    #[tokio::test]
    async fn test_generate_with_invalid_bpm_range_max_too_high() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "test".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: None,
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: Some(BpmRange {
                min: 100.0,
                max: 210.0,
            }),
            verify: false,
        };
        let result = generate_setlist_from_request(&pool, &claude, req).await;
        assert!(matches!(result, Err(SetlistError::InvalidBpmRange(_))));
    }

    #[tokio::test]
    async fn test_generate_with_invalid_bpm_range_min_gt_max() {
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "test".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: None,
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: Some(BpmRange {
                min: 140.0,
                max: 120.0,
            }),
            verify: false,
        };
        let result = generate_setlist_from_request(&pool, &claude, req).await;
        assert!(matches!(result, Err(SetlistError::InvalidBpmRange(_))));
    }

    #[tokio::test]
    async fn test_generate_catalog_percentage_computed() {
        let pool = setup_pool_with_tracks().await;
        // Response with catalog track
        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "catalog test".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: None,
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();
        // Single track from catalog = 100%
        assert_eq!(resp.catalog_percentage, Some(100.0));
        assert!(resp.catalog_warning.is_none());
    }

    #[tokio::test]
    async fn test_generate_catalog_warning_triggered() {
        let pool = setup_pool_with_tracks().await;
        // Response with suggestion track (not catalog)
        let claude = MockClaude {
            response: valid_llm_json(None),
        };
        let req = GenerateSetlistRequest {
            user_id: "user1".to_string(),
            prompt: "suggestion test".to_string(),
            track_count: None,
            energy_profile: None,
            source_playlist_id: None,
            seed_tracklist: None,
            creative_mode: None,
            bpm_range: None,
            verify: false,
        };
        let resp = generate_setlist_from_request(&pool, &claude, req)
            .await
            .unwrap();
        // Single suggestion track = 0% catalog
        assert_eq!(resp.catalog_percentage, Some(0.0));
        assert!(resp.catalog_warning.is_some());
    }

    #[tokio::test]
    async fn test_generate_without_new_params_backward_compat() {
        // Verify that generate_setlist (legacy) still works identically
        let pool = setup_pool_with_tracks().await;
        let claude = MockClaude {
            response: valid_llm_json(Some("t1")),
        };
        let resp = generate_setlist(&pool, &claude, "user1", "chill vibes", None)
            .await
            .unwrap();
        assert!(!resp.id.is_empty());
        assert_eq!(resp.tracks.len(), 1);
        assert_eq!(resp.tracks[0].title, "Desert Rose");
        // New fields should be present but with default values
        assert!(resp.energy_profile.is_none());
        assert!(resp.catalog_percentage.is_some()); // Always computed
    }

    // -----------------------------------------------------------------------
    // T9: Quality validation pure function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bpm_warnings_no_warnings() {
        let tracks = vec![
            SetlistTrackRow {
                id: "1".into(),
                setlist_id: "s".into(),
                track_id: None,
                position: 1,
                original_position: 1,
                title: "A".into(),
                artist: "X".into(),
                bpm: Some(128.0),
                key: None,
                camelot: None,
                energy: None,
                transition_note: None,
                transition_score: None,
                source: "suggestion".into(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: None,
                verification_flag: None,
                verification_note: None,
            },
            SetlistTrackRow {
                id: "2".into(),
                setlist_id: "s".into(),
                track_id: None,
                position: 2,
                original_position: 2,
                title: "B".into(),
                artist: "Y".into(),
                bpm: Some(132.0),
                key: None,
                camelot: None,
                energy: None,
                transition_note: None,
                transition_score: None,
                source: "suggestion".into(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: None,
                verification_flag: None,
                verification_note: None,
            },
        ];
        let warnings = compute_bpm_warnings(&tracks);
        assert!(
            warnings.is_empty(),
            "Deltas <= 6 should produce no warnings"
        );
    }

    #[test]
    fn test_bpm_warnings_correct_flag() {
        let tracks = vec![
            SetlistTrackRow {
                id: "1".into(),
                setlist_id: "s".into(),
                track_id: None,
                position: 1,
                original_position: 1,
                title: "A".into(),
                artist: "X".into(),
                bpm: Some(120.0),
                key: None,
                camelot: None,
                energy: None,
                transition_note: None,
                transition_score: None,
                source: "suggestion".into(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: None,
                verification_flag: None,
                verification_note: None,
            },
            SetlistTrackRow {
                id: "2".into(),
                setlist_id: "s".into(),
                track_id: None,
                position: 2,
                original_position: 2,
                title: "B".into(),
                artist: "Y".into(),
                bpm: Some(128.5),
                key: None,
                camelot: None,
                energy: None,
                transition_note: None,
                transition_score: None,
                source: "suggestion".into(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: None,
                verification_flag: None,
                verification_note: None,
            },
        ];
        let warnings = compute_bpm_warnings(&tracks);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].from_position, 1);
        assert_eq!(warnings[0].to_position, 2);
        assert!((warnings[0].bpm_delta - 8.5).abs() < 0.01);
    }

    #[test]
    fn test_bpm_warnings_missing_bpm_skipped() {
        let tracks = vec![
            SetlistTrackRow {
                id: "1".into(),
                setlist_id: "s".into(),
                track_id: None,
                position: 1,
                original_position: 1,
                title: "A".into(),
                artist: "X".into(),
                bpm: Some(120.0),
                key: None,
                camelot: None,
                energy: None,
                transition_note: None,
                transition_score: None,
                source: "suggestion".into(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: None,
                verification_flag: None,
                verification_note: None,
            },
            SetlistTrackRow {
                id: "2".into(),
                setlist_id: "s".into(),
                track_id: None,
                position: 2,
                original_position: 2,
                title: "B".into(),
                artist: "Y".into(),
                bpm: None,
                key: None,
                camelot: None,
                energy: None,
                transition_note: None,
                transition_score: None,
                source: "suggestion".into(),
                acquisition_info: None,
                spotify_uri: None,
                confidence: None,
                verification_flag: None,
                verification_note: None,
            },
        ];
        let warnings = compute_bpm_warnings(&tracks);
        assert!(warnings.is_empty(), "Missing BPM should be skipped");
    }

    #[test]
    fn test_catalog_percentage_all_catalog() {
        let tracks = vec![SetlistTrackResponse {
            position: 1,
            title: "A".into(),
            artist: "X".into(),
            bpm: None,
            key: None,
            camelot: None,
            energy: None,
            transition_note: None,
            transition_score: None,
            original_position: 1,
            source: "catalog".into(),
            track_id: Some("t1".into()),
            spotify_uri: None,
            confidence: None,
            verification_flag: None,
            verification_note: None,
        }];
        assert_eq!(compute_catalog_percentage(&tracks), 100.0);
    }

    #[test]
    fn test_catalog_percentage_all_suggestions() {
        let tracks = vec![SetlistTrackResponse {
            position: 1,
            title: "A".into(),
            artist: "X".into(),
            bpm: None,
            key: None,
            camelot: None,
            energy: None,
            transition_note: None,
            transition_score: None,
            original_position: 1,
            source: "suggestion".into(),
            track_id: None,
            spotify_uri: None,
            confidence: None,
            verification_flag: None,
            verification_note: None,
        }];
        assert_eq!(compute_catalog_percentage(&tracks), 0.0);
    }

    #[test]
    fn test_catalog_warning_triggers_at_29() {
        let warning = compute_catalog_warning(29.0);
        assert!(warning.is_some());
    }

    #[test]
    fn test_catalog_warning_does_not_trigger_at_30() {
        let warning = compute_catalog_warning(30.0);
        assert!(warning.is_none());
    }

    #[test]
    fn test_seed_match_count_exact() {
        let tracks = vec![SetlistTrackRow {
            id: "1".into(),
            setlist_id: "s".into(),
            track_id: None,
            position: 1,
            original_position: 1,
            title: "Desert Rose".into(),
            artist: "Sting".into(),
            bpm: None,
            key: None,
            camelot: None,
            energy: None,
            transition_note: None,
            transition_score: None,
            source: "suggestion".into(),
            acquisition_info: None,
            spotify_uri: None,
            confidence: None,
            verification_flag: None,
            verification_note: None,
        }];
        assert_eq!(compute_seed_match_count("Desert Rose", &tracks), 1);
    }

    #[test]
    fn test_seed_match_count_partial() {
        let tracks = vec![SetlistTrackRow {
            id: "1".into(),
            setlist_id: "s".into(),
            track_id: None,
            position: 1,
            original_position: 1,
            title: "Desert Rose (Extended Mix)".into(),
            artist: "Sting".into(),
            bpm: None,
            key: None,
            camelot: None,
            energy: None,
            transition_note: None,
            transition_score: None,
            source: "suggestion".into(),
            acquisition_info: None,
            spotify_uri: None,
            confidence: None,
            verification_flag: None,
            verification_note: None,
        }];
        assert_eq!(
            compute_seed_match_count("desert rose", &tracks),
            1,
            "Case-insensitive partial match"
        );
    }

    #[test]
    fn test_seed_match_count_no_match() {
        let tracks = vec![SetlistTrackRow {
            id: "1".into(),
            setlist_id: "s".into(),
            track_id: None,
            position: 1,
            original_position: 1,
            title: "Something Else".into(),
            artist: "Other Artist".into(),
            bpm: None,
            key: None,
            camelot: None,
            energy: None,
            transition_note: None,
            transition_score: None,
            source: "suggestion".into(),
            acquisition_info: None,
            spotify_uri: None,
            confidence: None,
            verification_flag: None,
            verification_note: None,
        }];
        assert_eq!(compute_seed_match_count("Desert Rose", &tracks), 0);
    }

    // -----------------------------------------------------------------------
    // T4: verify_setlist flag propagation tests
    // -----------------------------------------------------------------------

    fn make_response_track(
        position: i32,
        title: &str,
        artist: &str,
        confidence: Option<&str>,
    ) -> SetlistTrackResponse {
        SetlistTrackResponse {
            position,
            title: title.into(),
            artist: artist.into(),
            bpm: None,
            key: None,
            camelot: None,
            energy: None,
            transition_note: None,
            transition_score: None,
            original_position: position,
            source: "suggestion".into(),
            track_id: None,
            spotify_uri: None,
            confidence: confidence.map(|s| s.into()),
            verification_flag: None,
            verification_note: None,
        }
    }

    #[tokio::test]
    async fn test_verify_setlist_propagates_flag_and_note() {
        let tracks = vec![
            make_response_track(1, "Real Track", "Real Artist", Some("high")),
            make_response_track(2, "Fake Track", "Wrong Artist", Some("medium")),
            make_response_track(3, "Old Title", "Original Artist", Some("high")),
        ];

        let mock_response = serde_json::json!({
            "tracks": [
                {
                    "position": 1,
                    "title": "Real Track",
                    "artist": "Real Artist",
                    "confidence": "high",
                    "flag": null,
                    "correction": null
                },
                {
                    "position": 2,
                    "title": "Fake Track",
                    "artist": "Wrong Artist",
                    "confidence": "low",
                    "flag": "wrong_artist",
                    "correction": "Artist name is incorrect"
                },
                {
                    "position": 3,
                    "title": "New Title",
                    "artist": "Original Artist",
                    "confidence": "medium",
                    "flag": "replaced",
                    "correction": null
                }
            ],
            "summary": "Two issues found"
        });

        let claude = MockClaude {
            response: mock_response.to_string(),
        };

        let result = verify_setlist(&claude, &tracks).await.unwrap();
        assert_eq!(result.len(), 3);

        // Track 1: no flag, confidence unchanged
        assert_eq!(result[0].verification_flag, None);
        assert_eq!(result[0].verification_note, None);
        assert_eq!(result[0].confidence.as_deref(), Some("high"));

        // Track 2: wrong_artist flag propagated, confidence downgraded, note set
        assert_eq!(result[1].verification_flag.as_deref(), Some("wrong_artist"));
        assert_eq!(
            result[1].verification_note.as_deref(),
            Some("Artist name is incorrect")
        );
        assert_eq!(result[1].confidence.as_deref(), Some("low"));

        // Track 3: replaced — title updated, note describes original, flag set
        assert_eq!(result[2].verification_flag.as_deref(), Some("replaced"));
        assert_eq!(result[2].title, "New Title");
        assert_eq!(
            result[2].verification_note.as_deref(),
            Some("Replaced: was 'Old Title' by 'Original Artist'")
        );
        assert_eq!(result[2].confidence.as_deref(), Some("medium"));
    }
}
