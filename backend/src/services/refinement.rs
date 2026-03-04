// ST-007: Refinement service (refine, revert, history)

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::api::claude::{
    strip_markdown_fences, ClaudeClientTrait, ClaudeError, ConversationMessage,
};
use crate::db::models::{SetlistConversationRow, SetlistVersionRow, VersionTrackRow};
use crate::db::refinement as db;
use crate::db::setlists as db_setlists;
use crate::services::camelot::{camelot_score, parse_camelot};
use crate::services::quick_commands::{parse_quick_command, QuickCommand};

const MAX_TURNS: usize = 20;
const REFINEMENT_MODEL: &str = "claude-sonnet-4-20250514";
const MAX_TOKENS: u32 = 4096;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum RefinementError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("Turn limit exceeded: {limit} turns")]
    TurnLimitExceeded { limit: usize },

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for RefinementError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self {
            RefinementError::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m.clone()),
            RefinementError::InvalidRequest(m) => {
                (StatusCode::BAD_REQUEST, "INVALID_REQUEST", m.clone())
            }
            RefinementError::LlmError(m) => {
                (StatusCode::SERVICE_UNAVAILABLE, "LLM_ERROR", m.clone())
            }
            RefinementError::TurnLimitExceeded { limit } => (
                StatusCode::TOO_MANY_REQUESTS,
                "TURN_LIMIT_EXCEEDED",
                format!("Maximum of {limit} refinement turns reached"),
            ),
            RefinementError::GenerationFailed(m) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "GENERATION_FAILED",
                m.clone(),
            ),
            RefinementError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                format!("Database error: {e}"),
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

impl From<ClaudeError> for RefinementError {
    fn from(e: ClaudeError) -> Self {
        RefinementError::LlmError(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// LLM types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRefinementResponse {
    pub actions: Vec<LlmAction>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LlmAction {
    #[serde(rename = "replace")]
    Replace {
        position: usize,
        title: String,
        artist: String,
        bpm: Option<f64>,
        key: Option<String>,
    },
    #[serde(rename = "add")]
    Add {
        after_position: usize,
        title: String,
        artist: String,
        bpm: Option<f64>,
        key: Option<String>,
    },
    #[serde(rename = "remove")]
    Remove { position: usize },
    #[serde(rename = "reorder")]
    Reorder {
        from_position: usize,
        to_position: usize,
    },
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct RefinementResponse {
    pub version_number: i32,
    pub tracks: Vec<VersionTrackRow>,
    pub explanation: String,
    pub change_warning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub versions: Vec<SetlistVersionRow>,
    pub conversations: Vec<SetlistConversationRow>,
}

// ---------------------------------------------------------------------------
// Main functions
// ---------------------------------------------------------------------------

pub async fn refine_setlist(
    pool: &SqlitePool,
    claude: &dyn ClaudeClientTrait,
    setlist_id: &str,
    _user_id: &str,
    message: &str,
) -> Result<RefinementResponse, RefinementError> {
    // 1. Load setlist (404 if not found)
    let _setlist = db_setlists::get_setlist(pool, setlist_id)
        .await?
        .ok_or_else(|| RefinementError::NotFound(format!("Setlist {setlist_id} not found")))?;

    // 2. Check empty message
    if message.trim().is_empty() {
        return Err(RefinementError::InvalidRequest(
            "Message cannot be empty".to_string(),
        ));
    }

    // 3. Count user turns and check limit
    let conversations = db::get_conversations_by_setlist(pool, setlist_id).await?;
    let turn_count = conversations.iter().filter(|c| c.role == "user").count();
    if turn_count >= MAX_TURNS {
        return Err(RefinementError::TurnLimitExceeded { limit: MAX_TURNS });
    }

    // 4. Try quick command first — no LLM needed
    if let Some(quick_cmd) = parse_quick_command(message) {
        return handle_quick_command(pool, setlist_id, message, quick_cmd).await;
    }

    // 5. LLM path — bootstrap version 0 if no versions exist
    let latest_before_bootstrap = db::get_latest_version(pool, setlist_id).await?;
    let current_tracks = if let Some(latest) = latest_before_bootstrap {
        db::get_version_tracks(pool, &latest.id).await?
    } else {
        bootstrap_version(pool, setlist_id).await?
    };

    // 6. Build message history for multi-turn context
    let mut messages = conversations_to_messages(&conversations);
    messages.push(ConversationMessage {
        role: "user".to_string(),
        content: message.to_string(),
    });

    // 7. Build system prompt
    let system_prompt = build_refinement_system_prompt(&current_tracks);

    // 8. Call Claude converse() — retry once on parse failure
    let llm_text = claude
        .converse(
            &system_prompt,
            messages.clone(),
            REFINEMENT_MODEL,
            MAX_TOKENS,
        )
        .await
        .map_err(RefinementError::from)?;

    let parsed = match parse_refinement_response(&llm_text) {
        Ok(r) => r,
        Err(_) => {
            // Retry with the bad response + nudge
            let mut retry_msgs = messages.clone();
            retry_msgs.push(ConversationMessage {
                role: "assistant".to_string(),
                content: llm_text.clone(),
            });
            retry_msgs.push(ConversationMessage {
                role: "user".to_string(),
                content: "Please respond with valid JSON exactly as specified.".to_string(),
            });
            let retry_text = claude
                .converse(&system_prompt, retry_msgs, REFINEMENT_MODEL, MAX_TOKENS)
                .await
                .map_err(RefinementError::from)?;
            parse_refinement_response(&retry_text)?
        }
    };

    // 9. Validate actions
    validate_actions(&parsed.actions, current_tracks.len())?;

    // 10. Change warning
    let change_warning = compute_change_warning(&parsed.actions, current_tracks.len());

    // 11. Apply actions in memory
    let new_tracks_raw = apply_actions(current_tracks, &parsed.actions);

    // 12. Next version number
    let latest_version = db::get_latest_version(pool, setlist_id).await?;
    let next_version_num = latest_version.map(|v| v.version_number + 1).unwrap_or(1);

    // 13. Persist new version + tracks in one transaction
    let new_version_id = uuid::Uuid::new_v4().to_string();
    let new_version = SetlistVersionRow {
        id: new_version_id.clone(),
        setlist_id: setlist_id.to_string(),
        version_number: next_version_num,
        parent_version_id: None,
        action: Some("refine".to_string()),
        action_summary: Some(truncate(&parsed.explanation, 200)),
        created_at: None,
    };
    let new_tracks: Vec<VersionTrackRow> = new_tracks_raw
        .into_iter()
        .map(|t| VersionTrackRow {
            id: uuid::Uuid::new_v4().to_string(),
            version_id: new_version_id.clone(),
            ..t
        })
        .collect();

    let mut tx = pool.begin().await?;
    db::insert_version(&mut tx, &new_version).await?;
    db::insert_version_tracks(&mut tx, &new_tracks).await?;
    tx.commit().await?;

    // 14. Insert conversation messages
    insert_conversation_pair(
        pool,
        setlist_id,
        &new_version_id,
        message,
        &parsed.explanation,
    )
    .await?;

    // 15. Recompute harmonic flow score
    let score = compute_harmonic_score(&new_tracks);
    db_setlists::update_setlist_harmonic_score(pool, setlist_id, score).await?;

    Ok(RefinementResponse {
        version_number: next_version_num,
        tracks: new_tracks,
        explanation: parsed.explanation,
        change_warning,
    })
}

pub async fn revert_setlist(
    pool: &SqlitePool,
    setlist_id: &str,
    target_version_number: i32,
) -> Result<RefinementResponse, RefinementError> {
    let target_version = db::get_version_by_number(pool, setlist_id, target_version_number)
        .await?
        .ok_or_else(|| {
            RefinementError::NotFound(format!(
                "Version {target_version_number} not found for setlist {setlist_id}"
            ))
        })?;

    let target_tracks = db::get_version_tracks(pool, &target_version.id).await?;

    let latest = db::get_latest_version(pool, setlist_id).await?;
    let next_version_num = latest.map(|v| v.version_number + 1).unwrap_or(1);

    let new_version_id = uuid::Uuid::new_v4().to_string();
    let new_version = SetlistVersionRow {
        id: new_version_id.clone(),
        setlist_id: setlist_id.to_string(),
        version_number: next_version_num,
        parent_version_id: Some(target_version.id.clone()),
        action: Some("revert".to_string()),
        action_summary: Some(format!("Reverted to version {target_version_number}")),
        created_at: None,
    };
    let new_tracks: Vec<VersionTrackRow> = target_tracks
        .into_iter()
        .map(|t| VersionTrackRow {
            id: uuid::Uuid::new_v4().to_string(),
            version_id: new_version_id.clone(),
            ..t
        })
        .collect();

    let mut tx = pool.begin().await?;
    db::insert_version(&mut tx, &new_version).await?;
    db::insert_version_tracks(&mut tx, &new_tracks).await?;
    tx.commit().await?;

    let explanation = format!("Reverted to version {target_version_number}");
    Ok(RefinementResponse {
        version_number: next_version_num,
        tracks: new_tracks,
        explanation,
        change_warning: None,
    })
}

pub async fn get_history(
    pool: &SqlitePool,
    setlist_id: &str,
) -> Result<HistoryResponse, RefinementError> {
    let versions = db::get_versions_by_setlist(pool, setlist_id).await?;
    let conversations = db::get_conversations_by_setlist(pool, setlist_id).await?;
    Ok(HistoryResponse {
        versions,
        conversations,
    })
}

// ---------------------------------------------------------------------------
// Quick command handling
// ---------------------------------------------------------------------------

async fn handle_quick_command(
    pool: &SqlitePool,
    setlist_id: &str,
    message: &str,
    cmd: QuickCommand,
) -> Result<RefinementResponse, RefinementError> {
    match cmd {
        QuickCommand::Undo => {
            let versions = db::get_versions_by_setlist(pool, setlist_id).await?;
            if versions.len() < 2 {
                return Err(RefinementError::InvalidRequest(
                    "Nothing to undo".to_string(),
                ));
            }
            let prev = &versions[versions.len() - 2];
            revert_setlist(pool, setlist_id, prev.version_number).await
        }
        QuickCommand::RevertToVersion(n) => revert_setlist(pool, setlist_id, n).await,
        QuickCommand::Shuffle | QuickCommand::SortByBpm | QuickCommand::Reverse => {
            let latest = db::get_latest_version(pool, setlist_id).await?;
            let current_tracks = if let Some(v) = latest {
                db::get_version_tracks(pool, &v.id).await?
            } else {
                bootstrap_version(pool, setlist_id).await?
            };

            let (action_name, mut new_tracks) = match &cmd {
                QuickCommand::Shuffle => {
                    let mut t = current_tracks.clone();
                    t.shuffle(&mut rand::thread_rng());
                    renumber_version_tracks(&mut t);
                    ("shuffle", t)
                }
                QuickCommand::SortByBpm => {
                    let mut t = current_tracks.clone();
                    t.sort_by(|a, b| match (a.bpm, b.bpm) {
                        (Some(x), Some(y)) => {
                            x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    });
                    renumber_version_tracks(&mut t);
                    ("sort_by_bpm", t)
                }
                QuickCommand::Reverse => {
                    let mut t: Vec<_> = current_tracks.iter().rev().cloned().collect();
                    renumber_version_tracks(&mut t);
                    ("reverse", t)
                }
                _ => unreachable!(),
            };

            let latest_version = db::get_latest_version(pool, setlist_id).await?;
            let next_version_num = latest_version.map(|v| v.version_number + 1).unwrap_or(1);

            let new_version_id = uuid::Uuid::new_v4().to_string();
            let new_version = SetlistVersionRow {
                id: new_version_id.clone(),
                setlist_id: setlist_id.to_string(),
                version_number: next_version_num,
                parent_version_id: None,
                action: Some(action_name.to_string()),
                action_summary: Some(message.to_string()),
                created_at: None,
            };

            // Re-assign version_id
            for t in &mut new_tracks {
                t.id = uuid::Uuid::new_v4().to_string();
                t.version_id = new_version_id.clone();
            }

            let mut tx = pool.begin().await?;
            db::insert_version(&mut tx, &new_version).await?;
            db::insert_version_tracks(&mut tx, &new_tracks).await?;
            tx.commit().await?;

            let explanation = format!("Applied quick command: {action_name}");
            insert_conversation_pair(pool, setlist_id, &new_version_id, message, &explanation)
                .await?;

            Ok(RefinementResponse {
                version_number: next_version_num,
                tracks: new_tracks,
                explanation,
                change_warning: None,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn bootstrap_version(
    pool: &SqlitePool,
    setlist_id: &str,
) -> Result<Vec<VersionTrackRow>, RefinementError> {
    let setlist_tracks = db_setlists::get_setlist_tracks(pool, setlist_id).await?;
    let v0_id = uuid::Uuid::new_v4().to_string();
    let v0 = SetlistVersionRow {
        id: v0_id.clone(),
        setlist_id: setlist_id.to_string(),
        version_number: 0,
        parent_version_id: None,
        action: Some("bootstrap".to_string()),
        action_summary: Some("Initial snapshot".to_string()),
        created_at: None,
    };
    let version_tracks: Vec<VersionTrackRow> = setlist_tracks
        .iter()
        .map(|st| VersionTrackRow {
            id: uuid::Uuid::new_v4().to_string(),
            version_id: v0_id.clone(),
            track_id: st.track_id.clone(),
            position: st.position,
            original_position: st.original_position,
            title: st.title.clone(),
            artist: st.artist.clone(),
            bpm: st.bpm,
            key: st.key.clone(),
            camelot: st.camelot.clone(),
            energy: st.energy,
            transition_note: st.transition_note.clone(),
            transition_score: st.transition_score,
            source: st.source.clone(),
            acquisition_info: st.acquisition_info.clone(),
        })
        .collect();

    let mut tx = pool.begin().await?;
    db::insert_version(&mut tx, &v0).await?;
    db::insert_version_tracks(&mut tx, &version_tracks).await?;
    tx.commit().await?;

    Ok(version_tracks)
}

async fn insert_conversation_pair(
    pool: &SqlitePool,
    setlist_id: &str,
    version_id: &str,
    user_message: &str,
    assistant_message: &str,
) -> Result<(), RefinementError> {
    let user_msg = SetlistConversationRow {
        id: uuid::Uuid::new_v4().to_string(),
        setlist_id: setlist_id.to_string(),
        version_id: Some(version_id.to_string()),
        role: "user".to_string(),
        content: user_message.to_string(),
        created_at: None,
    };
    let assistant_msg = SetlistConversationRow {
        id: uuid::Uuid::new_v4().to_string(),
        setlist_id: setlist_id.to_string(),
        version_id: Some(version_id.to_string()),
        role: "assistant".to_string(),
        content: assistant_message.to_string(),
        created_at: None,
    };
    db::insert_conversation(pool, &user_msg).await?;
    db::insert_conversation(pool, &assistant_msg).await?;
    Ok(())
}

fn renumber_version_tracks(tracks: &mut [VersionTrackRow]) {
    for (i, track) in tracks.iter_mut().enumerate() {
        track.position = (i + 1) as i32;
        track.transition_note = None;
        track.transition_score = None;
    }
}

fn conversations_to_messages(conversations: &[SetlistConversationRow]) -> Vec<ConversationMessage> {
    conversations
        .iter()
        .map(|c| ConversationMessage {
            role: c.role.clone(),
            content: c.content.clone(),
        })
        .collect()
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", &s[..max_chars.saturating_sub(3)])
    }
}

fn compute_harmonic_score(tracks: &[VersionTrackRow]) -> f64 {
    if tracks.len() <= 1 {
        return 100.0;
    }
    let mut total = 0.0;
    let mut count = 0usize;
    for window in tracks.windows(2) {
        if let (Some(a), Some(b)) = (
            window[0].camelot.as_deref().and_then(parse_camelot),
            window[1].camelot.as_deref().and_then(parse_camelot),
        ) {
            total += camelot_score(&a, &b);
            count += 1;
        }
    }
    if count == 0 {
        50.0
    } else {
        total / count as f64
    }
}

// ---------------------------------------------------------------------------
// Pure functions (also used by tests)
// ---------------------------------------------------------------------------

pub fn build_refinement_system_prompt(tracks: &[VersionTrackRow]) -> String {
    let track_list: String = tracks
        .iter()
        .map(|t| {
            let bpm_str = t
                .bpm
                .map(|b| format!("{b:.0} BPM"))
                .unwrap_or_else(|| "? BPM".to_string());
            let key_str = t.key.as_deref().unwrap_or("?");
            format!(
                "{}. {} - {} ({}, key: {})",
                t.position, t.title, t.artist, bpm_str, key_str
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are an expert DJ assistant helping refine setlists for optimal flow and energy.
You understand harmonic mixing (Camelot wheel), BPM transitions, and crowd energy management.

CURRENT SETLIST ({count} tracks):
{track_list}

When the user asks to modify the setlist, respond with a JSON object containing:
{{
  "actions": [
    // Array of action objects - see types below
  ],
  "explanation": "Brief explanation of the changes made"
}}

Action types:
- Replace: {{"type": "replace", "position": N, "title": "...", "artist": "...", "bpm": null_or_number, "key": null_or_string}}
- Add: {{"type": "add", "after_position": N, "title": "...", "artist": "...", "bpm": null_or_number, "key": null_or_string}}
  (use after_position: 0 to add at the beginning)
- Remove: {{"type": "remove", "position": N}}
- Reorder: {{"type": "reorder", "from_position": N, "to_position": N}}

Positions are 1-indexed. Be precise and minimal — only include changes needed.
Respond ONLY with the JSON object, no additional text."#,
        count = tracks.len(),
        track_list = track_list,
    )
}

pub fn parse_refinement_response(text: &str) -> Result<LlmRefinementResponse, RefinementError> {
    let stripped = strip_markdown_fences(text);
    serde_json::from_str::<LlmRefinementResponse>(stripped).map_err(|e| {
        RefinementError::GenerationFailed(format!("Failed to parse LLM response: {e}"))
    })
}

pub fn validate_actions(actions: &[LlmAction], track_count: usize) -> Result<(), RefinementError> {
    for action in actions {
        match action {
            LlmAction::Replace { position, .. } => {
                if *position < 1 || *position > track_count {
                    return Err(RefinementError::InvalidRequest(format!(
                        "Replace position {position} out of range (1-{track_count})"
                    )));
                }
            }
            LlmAction::Remove { position } => {
                if *position < 1 || *position > track_count {
                    return Err(RefinementError::InvalidRequest(format!(
                        "Remove position {position} out of range (1-{track_count})"
                    )));
                }
            }
            LlmAction::Reorder {
                from_position,
                to_position,
            } => {
                if *from_position < 1 || *from_position > track_count {
                    return Err(RefinementError::InvalidRequest(format!(
                        "Reorder from_position {from_position} out of range (1-{track_count})"
                    )));
                }
                if *to_position < 1 || *to_position > track_count {
                    return Err(RefinementError::InvalidRequest(format!(
                        "Reorder to_position {to_position} out of range (1-{track_count})"
                    )));
                }
            }
            LlmAction::Add { after_position, .. } => {
                if *after_position > track_count {
                    return Err(RefinementError::InvalidRequest(format!(
                        "Add after_position {after_position} out of range (0-{track_count})"
                    )));
                }
            }
        }
    }
    Ok(())
}

pub fn apply_actions(
    mut tracks: Vec<VersionTrackRow>,
    actions: &[LlmAction],
) -> Vec<VersionTrackRow> {
    for action in actions {
        match action {
            LlmAction::Replace {
                position,
                title,
                artist,
                bpm,
                key,
            } => {
                let idx = position.saturating_sub(1);
                if idx < tracks.len() {
                    tracks[idx].title = title.clone();
                    tracks[idx].artist = artist.clone();
                    tracks[idx].bpm = *bpm;
                    tracks[idx].key = key.clone();
                    tracks[idx].track_id = None;
                    tracks[idx].camelot = None;
                    tracks[idx].transition_note = None;
                    tracks[idx].transition_score = None;
                    tracks[idx].source = "suggestion".to_string();
                }
            }
            LlmAction::Add {
                after_position,
                title,
                artist,
                bpm,
                key,
            } => {
                let new_track = VersionTrackRow {
                    id: uuid::Uuid::new_v4().to_string(),
                    version_id: String::new(), // re-assigned by caller
                    track_id: None,
                    position: 0,
                    original_position: 0,
                    title: title.clone(),
                    artist: artist.clone(),
                    bpm: *bpm,
                    key: key.clone(),
                    camelot: None,
                    energy: None,
                    transition_note: None,
                    transition_score: None,
                    source: "suggestion".to_string(),
                    acquisition_info: None,
                };
                tracks.insert(*after_position, new_track);
            }
            LlmAction::Remove { position } => {
                let idx = position.saturating_sub(1);
                if idx < tracks.len() {
                    tracks.remove(idx);
                }
            }
            LlmAction::Reorder {
                from_position,
                to_position,
            } => {
                let from_idx = from_position.saturating_sub(1);
                let to_idx = to_position.saturating_sub(1);
                if from_idx < tracks.len() && to_idx < tracks.len() {
                    let track = tracks.remove(from_idx);
                    tracks.insert(to_idx, track);
                }
            }
        }
    }

    // Renumber positions 1-based
    for (i, track) in tracks.iter_mut().enumerate() {
        track.position = (i + 1) as i32;
    }

    tracks
}

pub fn compute_change_warning(actions: &[LlmAction], track_count: usize) -> Option<String> {
    if track_count == 0 {
        return None;
    }
    let changed: std::collections::HashSet<usize> = actions
        .iter()
        .flat_map(|a| match a {
            LlmAction::Replace { position, .. } => vec![*position],
            LlmAction::Remove { position } => vec![*position],
            LlmAction::Reorder {
                from_position,
                to_position,
            } => vec![*from_position, *to_position],
            LlmAction::Add { .. } => vec![],
        })
        .collect();

    let pct = changed.len() as f64 / track_count as f64;
    if pct > 0.5 {
        Some(format!(
            "Warning: {:.0}% of tracks ({}/{}) will be modified by this refinement.",
            pct * 100.0,
            changed.len(),
            track_count
        ))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::claude::{CacheMetrics, ClaudeError, RequestContentBlock};
    use crate::db::create_test_pool;
    use sqlx::SqlitePool;

    // -----------------------------------------------------------------------
    // Mock Claude
    // -----------------------------------------------------------------------

    struct MockClaude {
        responses: std::sync::Mutex<std::collections::VecDeque<String>>,
    }

    impl MockClaude {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses.into_iter().collect()),
            }
        }

        fn single(response: &str) -> Self {
            Self::new(vec![response.to_string()])
        }
    }

    #[async_trait::async_trait]
    impl ClaudeClientTrait for MockClaude {
        async fn generate_setlist(
            &self,
            _: &str,
            _: &str,
            _: &str,
            _: u32,
        ) -> Result<String, ClaudeError> {
            unreachable!("not used in refinement tests")
        }

        async fn generate_with_blocks(
            &self,
            _: Vec<RequestContentBlock>,
            _: Vec<RequestContentBlock>,
            _: &str,
            _: u32,
        ) -> Result<(String, CacheMetrics), ClaudeError> {
            unreachable!("not used in refinement tests")
        }

        async fn converse(
            &self,
            _system: &str,
            _messages: Vec<ConversationMessage>,
            _model: &str,
            _max_tokens: u32,
        ) -> Result<String, ClaudeError> {
            let mut queue = self.responses.lock().unwrap();
            if let Some(r) = queue.pop_front() {
                Ok(r)
            } else {
                Err(ClaudeError::Api("no more mock responses".to_string()))
            }
        }
    }

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    async fn insert_setlist(pool: &SqlitePool) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind("user-1")
            .bind("test prompt")
            .bind("claude-test")
            .execute(pool)
            .await
            .unwrap();
        id
    }

    async fn insert_setlist_tracks(pool: &SqlitePool, setlist_id: &str, count: usize) {
        for i in 1..=count {
            let track_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO setlist_tracks \
                 (id, setlist_id, position, original_position, title, artist, bpm, key, camelot, source) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&track_id)
            .bind(setlist_id)
            .bind(i as i32)
            .bind(i as i32)
            .bind(format!("Track {i}"))
            .bind("Artist")
            .bind(120.0 + i as f64)
            .bind(format!("A{i}"))
            .bind(format!("{}A", i % 12 + 1))
            .bind("suggestion")
            .execute(pool)
            .await
            .unwrap();
        }
    }

    fn replace_response(position: usize, title: &str, artist: &str) -> String {
        serde_json::json!({
            "actions": [{"type": "replace", "position": position, "title": title, "artist": artist, "bpm": null, "key": null}],
            "explanation": format!("Replaced track at position {position}")
        })
        .to_string()
    }

    fn add_response(after_position: usize, title: &str, artist: &str) -> String {
        serde_json::json!({
            "actions": [{"type": "add", "after_position": after_position, "title": title, "artist": artist, "bpm": 130.0, "key": "C"}],
            "explanation": format!("Added track after position {after_position}")
        })
        .to_string()
    }

    fn remove_response(position: usize) -> String {
        serde_json::json!({
            "actions": [{"type": "remove", "position": position}],
            "explanation": format!("Removed track at position {position}")
        })
        .to_string()
    }

    fn reorder_response(from: usize, to: usize) -> String {
        serde_json::json!({
            "actions": [{"type": "reorder", "from_position": from, "to_position": to}],
            "explanation": format!("Moved track from {from} to {to}")
        })
        .to_string()
    }

    // -----------------------------------------------------------------------
    // Unit tests: apply_actions
    // -----------------------------------------------------------------------

    fn make_version_track(pos: i32, title: &str) -> VersionTrackRow {
        VersionTrackRow {
            id: uuid::Uuid::new_v4().to_string(),
            version_id: "v1".to_string(),
            track_id: None,
            position: pos,
            original_position: pos,
            title: title.to_string(),
            artist: "Artist".to_string(),
            bpm: Some(120.0),
            key: Some("A".to_string()),
            camelot: Some("8A".to_string()),
            energy: None,
            transition_note: None,
            transition_score: None,
            source: "suggestion".to_string(),
            acquisition_info: None,
        }
    }

    #[test]
    fn test_apply_replace() {
        let tracks = vec![
            make_version_track(1, "Alpha"),
            make_version_track(2, "Beta"),
            make_version_track(3, "Gamma"),
        ];
        let actions = vec![LlmAction::Replace {
            position: 2,
            title: "Delta".to_string(),
            artist: "New Artist".to_string(),
            bpm: Some(128.0),
            key: None,
        }];
        let result = apply_actions(tracks, &actions);
        assert_eq!(result[1].title, "Delta");
        assert_eq!(result[1].artist, "New Artist");
        assert_eq!(result[1].bpm, Some(128.0));
        assert_eq!(result[0].title, "Alpha");
        assert_eq!(result[2].title, "Gamma");
    }

    #[test]
    fn test_apply_add() {
        let tracks = vec![
            make_version_track(1, "Alpha"),
            make_version_track(2, "Beta"),
        ];
        let actions = vec![LlmAction::Add {
            after_position: 1,
            title: "Inserted".to_string(),
            artist: "New".to_string(),
            bpm: Some(130.0),
            key: None,
        }];
        let result = apply_actions(tracks, &actions);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].title, "Alpha");
        assert_eq!(result[1].title, "Inserted");
        assert_eq!(result[2].title, "Beta");
        // Positions renumbered
        assert_eq!(result[0].position, 1);
        assert_eq!(result[1].position, 2);
        assert_eq!(result[2].position, 3);
    }

    #[test]
    fn test_apply_add_at_start() {
        let tracks = vec![
            make_version_track(1, "Alpha"),
            make_version_track(2, "Beta"),
        ];
        let actions = vec![LlmAction::Add {
            after_position: 0,
            title: "First".to_string(),
            artist: "Artist".to_string(),
            bpm: None,
            key: None,
        }];
        let result = apply_actions(tracks, &actions);
        assert_eq!(result[0].title, "First");
        assert_eq!(result[1].title, "Alpha");
    }

    #[test]
    fn test_apply_remove() {
        let tracks = vec![
            make_version_track(1, "Alpha"),
            make_version_track(2, "Beta"),
            make_version_track(3, "Gamma"),
        ];
        let actions = vec![LlmAction::Remove { position: 2 }];
        let result = apply_actions(tracks, &actions);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Alpha");
        assert_eq!(result[1].title, "Gamma");
        assert_eq!(result[0].position, 1);
        assert_eq!(result[1].position, 2);
    }

    #[test]
    fn test_apply_reorder() {
        let tracks = vec![
            make_version_track(1, "Alpha"),
            make_version_track(2, "Beta"),
            make_version_track(3, "Gamma"),
        ];
        let actions = vec![LlmAction::Reorder {
            from_position: 3,
            to_position: 1,
        }];
        let result = apply_actions(tracks, &actions);
        assert_eq!(result[0].title, "Gamma");
        assert_eq!(result[1].title, "Alpha");
        assert_eq!(result[2].title, "Beta");
    }

    // -----------------------------------------------------------------------
    // Unit tests: validate_actions
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_replace_out_of_range() {
        let actions = vec![LlmAction::Replace {
            position: 5,
            title: "X".to_string(),
            artist: "Y".to_string(),
            bpm: None,
            key: None,
        }];
        assert!(validate_actions(&actions, 3).is_err());
    }

    #[test]
    fn test_validate_remove_out_of_range() {
        let actions = vec![LlmAction::Remove { position: 0 }];
        assert!(validate_actions(&actions, 3).is_err());
    }

    #[test]
    fn test_validate_add_out_of_range() {
        let actions = vec![LlmAction::Add {
            after_position: 10,
            title: "X".to_string(),
            artist: "Y".to_string(),
            bpm: None,
            key: None,
        }];
        assert!(validate_actions(&actions, 3).is_err());
    }

    #[test]
    fn test_validate_valid_actions() {
        let actions = vec![
            LlmAction::Replace {
                position: 2,
                title: "X".to_string(),
                artist: "Y".to_string(),
                bpm: None,
                key: None,
            },
            LlmAction::Remove { position: 3 },
            LlmAction::Add {
                after_position: 0,
                title: "Z".to_string(),
                artist: "W".to_string(),
                bpm: None,
                key: None,
            },
        ];
        assert!(validate_actions(&actions, 5).is_ok());
    }

    // -----------------------------------------------------------------------
    // Unit tests: parse_refinement_response
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_valid_response() {
        let json = r#"{"actions": [{"type": "remove", "position": 1}], "explanation": "Removed first track"}"#;
        let parsed = parse_refinement_response(json).unwrap();
        assert_eq!(parsed.explanation, "Removed first track");
        assert_eq!(parsed.actions.len(), 1);
    }

    #[test]
    fn test_parse_fenced_response() {
        let json = "```json\n{\"actions\": [], \"explanation\": \"No changes\"}\n```";
        let parsed = parse_refinement_response(json).unwrap();
        assert_eq!(parsed.explanation, "No changes");
        assert!(parsed.actions.is_empty());
    }

    #[test]
    fn test_parse_malformed_response() {
        assert!(parse_refinement_response("not json at all").is_err());
    }

    // -----------------------------------------------------------------------
    // Unit tests: compute_change_warning
    // -----------------------------------------------------------------------

    #[test]
    fn test_change_warning_triggered() {
        let actions = vec![
            LlmAction::Replace {
                position: 1,
                title: "X".to_string(),
                artist: "Y".to_string(),
                bpm: None,
                key: None,
            },
            LlmAction::Replace {
                position: 2,
                title: "A".to_string(),
                artist: "B".to_string(),
                bpm: None,
                key: None,
            },
            LlmAction::Replace {
                position: 3,
                title: "C".to_string(),
                artist: "D".to_string(),
                bpm: None,
                key: None,
            },
        ];
        let warning = compute_change_warning(&actions, 4);
        assert!(warning.is_some());
    }

    #[test]
    fn test_change_warning_not_triggered() {
        let actions = vec![LlmAction::Replace {
            position: 1,
            title: "X".to_string(),
            artist: "Y".to_string(),
            bpm: None,
            key: None,
        }];
        let warning = compute_change_warning(&actions, 10);
        assert!(warning.is_none());
    }

    // -----------------------------------------------------------------------
    // Integration tests: refine_setlist (with DB + MockClaude)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_refine_replace_action() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 3).await;

        let claude = MockClaude::single(&replace_response(2, "New Track", "New Artist"));
        let resp = refine_setlist(&pool, &claude, &setlist_id, "user-1", "Replace track 2")
            .await
            .unwrap();

        assert_eq!(resp.version_number, 1);
        assert_eq!(resp.tracks[1].title, "New Track");
    }

    #[tokio::test]
    async fn test_refine_add_action() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 2).await;

        let claude = MockClaude::single(&add_response(1, "Inserted", "DJ"));
        let resp = refine_setlist(&pool, &claude, &setlist_id, "user-1", "Add after track 1")
            .await
            .unwrap();

        assert_eq!(resp.tracks.len(), 3);
        assert_eq!(resp.tracks[1].title, "Inserted");
    }

    #[tokio::test]
    async fn test_refine_remove_action() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 3).await;

        let claude = MockClaude::single(&remove_response(2));
        let resp = refine_setlist(&pool, &claude, &setlist_id, "user-1", "Remove track 2")
            .await
            .unwrap();

        assert_eq!(resp.tracks.len(), 2);
        assert_eq!(resp.tracks[0].title, "Track 1");
        assert_eq!(resp.tracks[1].title, "Track 3");
    }

    #[tokio::test]
    async fn test_refine_reorder_action() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 3).await;

        let claude = MockClaude::single(&reorder_response(3, 1));
        let resp = refine_setlist(&pool, &claude, &setlist_id, "user-1", "Move last to first")
            .await
            .unwrap();

        assert_eq!(resp.tracks[0].title, "Track 3");
    }

    #[tokio::test]
    async fn test_bootstrap_version_0_created() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 2).await;

        // No versions yet
        let before = db::get_latest_version(&pool, &setlist_id).await.unwrap();
        assert!(before.is_none());

        let claude = MockClaude::single(&replace_response(1, "New", "Artist"));
        refine_setlist(&pool, &claude, &setlist_id, "user-1", "change first")
            .await
            .unwrap();

        // Should have v0 (bootstrap) + v1 (refine)
        let versions = db::get_versions_by_setlist(&pool, &setlist_id)
            .await
            .unwrap();
        assert!(versions.len() >= 2);
        assert_eq!(versions[0].version_number, 0);
        assert_eq!(versions[0].action.as_deref(), Some("bootstrap"));
    }

    #[tokio::test]
    async fn test_quick_command_bypass_no_llm_call() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 3).await;

        // Bootstrap version 0 first
        bootstrap_version(&pool, &setlist_id).await.unwrap();

        // A MockClaude that should never be called
        let claude = MockClaude::new(vec![]);
        let resp = refine_setlist(&pool, &claude, &setlist_id, "user-1", "reverse")
            .await
            .unwrap();

        // Tracks should be reversed: 3, 2, 1
        assert_eq!(resp.tracks[0].title, "Track 3");
        assert_eq!(resp.tracks[2].title, "Track 1");
    }

    #[tokio::test]
    async fn test_malformed_response_retry_succeeds() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 2).await;

        // First call returns garbage, second returns valid JSON
        let claude = MockClaude::new(vec![
            "this is not json at all".to_string(),
            replace_response(1, "Retry Track", "Retry Artist"),
        ]);
        let resp = refine_setlist(&pool, &claude, &setlist_id, "user-1", "change something")
            .await
            .unwrap();

        assert_eq!(resp.tracks[0].title, "Retry Track");
    }

    #[tokio::test]
    async fn test_change_warning_in_response() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 4).await;

        // Replace 3 out of 4 tracks (75% > 50%)
        let big_change = serde_json::json!({
            "actions": [
                {"type": "replace", "position": 1, "title": "X1", "artist": "A", "bpm": null, "key": null},
                {"type": "replace", "position": 2, "title": "X2", "artist": "A", "bpm": null, "key": null},
                {"type": "replace", "position": 3, "title": "X3", "artist": "A", "bpm": null, "key": null}
            ],
            "explanation": "Big change"
        })
        .to_string();
        let claude = MockClaude::single(&big_change);
        let resp = refine_setlist(&pool, &claude, &setlist_id, "user-1", "big change")
            .await
            .unwrap();

        assert!(resp.change_warning.is_some());
    }

    #[tokio::test]
    async fn test_revert_creates_new_version() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 2).await;

        // Create v0 + v1 via refine
        let claude = MockClaude::single(&replace_response(1, "V1 Track", "Artist"));
        refine_setlist(&pool, &claude, &setlist_id, "user-1", "refine")
            .await
            .unwrap();

        // Revert to v0
        let resp = revert_setlist(&pool, &setlist_id, 0).await.unwrap();
        assert_eq!(resp.version_number, 2);
        // Tracks should match v0 (original bootstrap)
        assert_eq!(resp.tracks[0].title, "Track 1");
    }

    #[tokio::test]
    async fn test_get_history_returns_versions_and_conversations() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;
        insert_setlist_tracks(&pool, &setlist_id, 2).await;

        let claude = MockClaude::single(&replace_response(1, "Changed", "Artist"));
        refine_setlist(&pool, &claude, &setlist_id, "user-1", "change it")
            .await
            .unwrap();

        let history = get_history(&pool, &setlist_id).await.unwrap();
        assert!(!history.versions.is_empty());
        assert!(!history.conversations.is_empty());
        let user_convos: Vec<_> = history
            .conversations
            .iter()
            .filter(|c| c.role == "user")
            .collect();
        assert!(!user_convos.is_empty());
    }

    #[tokio::test]
    async fn test_turn_limit_exceeded() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;

        // Insert 20 user conversations manually
        for _ in 0..20 {
            let row = SetlistConversationRow {
                id: uuid::Uuid::new_v4().to_string(),
                setlist_id: setlist_id.clone(),
                version_id: None,
                role: "user".to_string(),
                content: "turn".to_string(),
                created_at: None,
            };
            db::insert_conversation(&pool, &row).await.unwrap();
        }

        let claude = MockClaude::single("{}");
        let result = refine_setlist(&pool, &claude, &setlist_id, "user-1", "one more turn").await;
        assert!(matches!(
            result,
            Err(RefinementError::TurnLimitExceeded { .. })
        ));
    }

    #[tokio::test]
    async fn test_empty_message_returns_error() {
        let pool = create_test_pool().await;
        let setlist_id = insert_setlist(&pool).await;

        let claude = MockClaude::single("{}");
        let result = refine_setlist(&pool, &claude, &setlist_id, "user-1", "   ").await;
        assert!(matches!(result, Err(RefinementError::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn test_setlist_not_found() {
        let pool = create_test_pool().await;
        let claude = MockClaude::single("{}");
        let result = refine_setlist(&pool, &claude, "nonexistent-id", "user-1", "refine").await;
        assert!(matches!(result, Err(RefinementError::NotFound(_))));
    }
}
