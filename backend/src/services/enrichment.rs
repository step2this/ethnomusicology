use crate::api::claude::{strip_markdown_fences, ClaudeClientTrait, ClaudeError};
use crate::db::models::TrackRow;
use crate::services::camelot;
use serde::Deserialize;
use sqlx::SqlitePool;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum EnrichmentError {
    #[error("Claude API error: {0}")]
    Claude(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Cost cap exceeded: {0}")]
    CostCapExceeded(String),
    #[error("Concurrent enrichment: {0}")]
    Concurrent(String),
}

impl From<sqlx::Error> for EnrichmentError {
    fn from(e: sqlx::Error) -> Self {
        EnrichmentError::Database(e.to_string())
    }
}

impl From<ClaudeError> for EnrichmentError {
    fn from(e: ClaudeError) -> Self {
        EnrichmentError::Claude(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// LLM response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct LlmEnrichmentResponse {
    tracks: Vec<LlmEnrichmentEntry>,
}

#[derive(Debug, Deserialize)]
struct LlmEnrichmentEntry {
    position: usize,
    bpm: Option<f64>,
    #[allow(dead_code)]
    key: Option<String>,
    camelot: Option<String>,
    energy: Option<i32>,
}

// ---------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct EnrichmentResult {
    pub enriched: u32,
    pub errors: u32,
    pub skipped: u32,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const BATCH_SIZE: usize = 50;
const MAX_BATCHES: usize = 5;
const ENRICHMENT_MODEL: &str = "claude-sonnet-4-20250514";
const ENRICHMENT_MAX_TOKENS: u32 = 4096;
const DAILY_CAP: usize = MAX_BATCHES * BATCH_SIZE;

const ENRICHMENT_SYSTEM_PROMPT: &str = r#"You are a music metadata expert. For each track below, estimate:
- BPM (beats per minute, as a number like 128.0)
- Musical key (e.g., "A minor", "C major", "F# minor")
- Camelot wheel notation (e.g., "8A", "11B")
- Energy level (1-10 scale, where 1=ambient/chill, 10=peak-time banger)

Use your knowledge of these songs. If you don't know a track, make your best estimate based on the artist's typical style.

Respond with ONLY valid JSON (no markdown fences):
{"tracks": [{"position": 1, "bpm": 128.0, "key": "A minor", "camelot": "8A", "energy": 7}, ...]}"#;

// ---------------------------------------------------------------------------
// Main function
// ---------------------------------------------------------------------------

pub async fn enrich_tracks(
    pool: &SqlitePool,
    claude: &dyn ClaudeClientTrait,
    user_id: &str,
) -> Result<EnrichmentResult, EnrichmentError> {
    // 1. Get unenriched tracks
    let tracks = crate::db::tracks::get_unenriched_tracks(pool, MAX_BATCHES * BATCH_SIZE).await?;

    if tracks.is_empty() {
        return Ok(EnrichmentResult {
            enriched: 0,
            errors: 0,
            skipped: 0,
        });
    }

    // 2. Check daily usage cap
    let daily_count = crate::db::tracks::get_daily_enrichment_count(pool, user_id).await?;
    if daily_count >= DAILY_CAP as i64 {
        return Err(EnrichmentError::CostCapExceeded(format!(
            "Daily enrichment limit of {DAILY_CAP} tracks reached ({daily_count} used today)"
        )));
    }

    // 3. Process in batches
    let mut total_enriched: u32 = 0;
    let mut total_errors: u32 = 0;
    let mut total_skipped: u32 = 0;

    for batch in tracks.chunks(BATCH_SIZE) {
        let user_prompt = build_user_prompt(batch);

        // Call Claude
        let raw_response = match claude
            .generate_setlist(
                ENRICHMENT_SYSTEM_PROMPT,
                &user_prompt,
                ENRICHMENT_MODEL,
                ENRICHMENT_MAX_TOKENS,
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Claude API error during enrichment: {e}");
                for track in batch {
                    let _ = crate::db::tracks::mark_enrichment_error(
                        pool,
                        &track.id,
                        &format!("Claude API error: {e}"),
                    )
                    .await;
                }
                total_errors += batch.len() as u32;
                continue;
            }
        };

        // Parse response
        let cleaned = strip_markdown_fences(&raw_response);
        let llm_response: LlmEnrichmentResponse = match serde_json::from_str(cleaned) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to parse enrichment response: {e}");
                for track in batch {
                    let _ = crate::db::tracks::mark_enrichment_error(
                        pool,
                        &track.id,
                        &format!("Parse error: {e}"),
                    )
                    .await;
                }
                total_errors += batch.len() as u32;
                continue;
            }
        };

        // Track which positions were responded to
        let mut responded_indices = std::collections::HashSet::new();

        // Process each entry
        for entry in &llm_response.tracks {
            if entry.position == 0 || entry.position > batch.len() {
                total_skipped += 1;
                continue;
            }
            let idx = entry.position - 1;

            responded_indices.insert(idx);
            let track = &batch[idx];

            // Validate camelot notation
            let camelot_str = entry
                .camelot
                .as_deref()
                .and_then(|c| camelot::parse_camelot(c).map(|k| k.to_string()));

            // Validate BPM and energy ranges
            let bpm = entry.bpm.filter(|b| (0.0..300.0).contains(b));
            let energy_f64 = entry
                .energy
                .filter(|e| (1..=10).contains(e))
                .map(|e| e as f64);

            match crate::db::tracks::update_track_dj_metadata(
                pool,
                &track.id,
                bpm,
                camelot_str.as_deref(),
                energy_f64,
                None,
            )
            .await
            {
                Ok(()) => total_enriched += 1,
                Err(e) => {
                    tracing::error!("Failed to update track {}: {e}", track.id);
                    let _ = crate::db::tracks::mark_enrichment_error(
                        pool,
                        &track.id,
                        &format!("DB update error: {e}"),
                    )
                    .await;
                    total_errors += 1;
                }
            }
        }

        // Mark tracks not in response as skipped
        for (i, track) in batch.iter().enumerate() {
            if !responded_indices.contains(&i) {
                let _ = crate::db::tracks::mark_enrichment_error(
                    pool,
                    &track.id,
                    "Not included in LLM response",
                )
                .await;
                total_skipped += 1;
            }
        }
    }

    // 4. Increment usage counter
    if total_enriched > 0 {
        crate::db::tracks::increment_enrichment_usage(pool, user_id, total_enriched as i64).await?;
    }

    Ok(EnrichmentResult {
        enriched: total_enriched,
        errors: total_errors,
        skipped: total_skipped,
    })
}

fn build_user_prompt(tracks: &[TrackRow]) -> String {
    let mut lines = Vec::with_capacity(tracks.len());
    for (i, t) in tracks.iter().enumerate() {
        let artist = t.artist.as_deref().unwrap_or("Unknown Artist");
        lines.push(format!("{}. \"{}\" by \"{}\"", i + 1, t.title, artist));
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::setlist::test_utils::MockClaude;

    async fn setup_pool_with_unenriched_tracks(count: usize) -> SqlitePool {
        let pool = crate::db::create_test_pool().await;
        for i in 0..count {
            sqlx::query(
                "INSERT INTO tracks (id, title, source, needs_enrichment) VALUES (?, ?, 'spotify', 1)",
            )
            .bind(format!("t{i}"))
            .bind(format!("Track {i}"))
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query("INSERT OR IGNORE INTO artists (id, name) VALUES (?, ?)")
                .bind(format!("a{i}"))
                .bind(format!("Artist {i}"))
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES (?, ?)")
                .bind(format!("t{i}"))
                .bind(format!("a{i}"))
                .execute(&pool)
                .await
                .unwrap();
        }
        pool
    }

    fn mock_enrichment_response(count: usize) -> String {
        let entries: Vec<String> = (0..count)
            .map(|i| {
                format!(
                    r#"{{"position": {}, "bpm": {}.0, "key": "C major", "camelot": "8B", "energy": {}}}"#,
                    i + 1,
                    120 + i,
                    (i % 10) + 1
                )
            })
            .collect();
        format!(r#"{{"tracks": [{}]}}"#, entries.join(","))
    }

    // Test 1: Successful batch enrichment — verify BPM/key/energy populated in DB
    #[tokio::test]
    async fn test_successful_enrichment() {
        let pool = setup_pool_with_unenriched_tracks(3).await;
        let claude = MockClaude {
            response: mock_enrichment_response(3),
        };

        let result = enrich_tracks(&pool, &claude, "user1").await.unwrap();

        assert_eq!(result.enriched, 3);
        assert_eq!(result.errors, 0);
        assert_eq!(result.skipped, 0);

        // Verify DB was updated for track t0
        let row: (Option<f64>, Option<String>, Option<f64>) =
            sqlx::query_as("SELECT bpm, camelot_key, energy FROM tracks WHERE id = 't0'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row.0, Some(120.0));
        assert_eq!(row.1.as_deref(), Some("8B"));
        assert_eq!(row.2, Some(1.0));

        // Verify DB was updated for track t2
        let row2: (Option<f64>, Option<String>, Option<f64>) =
            sqlx::query_as("SELECT bpm, camelot_key, energy FROM tracks WHERE id = 't2'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row2.0, Some(122.0));
        assert_eq!(row2.1.as_deref(), Some("8B"));
        assert_eq!(row2.2, Some(3.0));
    }

    // Test 2: Empty unenriched tracks — returns zeros immediately
    #[tokio::test]
    async fn test_empty_unenriched_returns_zeros() {
        let pool = crate::db::create_test_pool().await;
        let claude = MockClaude {
            response: "{}".to_string(),
        };

        let result = enrich_tracks(&pool, &claude, "user1").await.unwrap();

        assert_eq!(result.enriched, 0);
        assert_eq!(result.errors, 0);
        assert_eq!(result.skipped, 0);
    }

    // Test 3: Cost cap exceeded — returns EnrichmentError::CostCapExceeded
    #[tokio::test]
    async fn test_cost_cap_exceeded() {
        let pool = setup_pool_with_unenriched_tracks(1).await;

        // Pre-fill usage to the cap (250)
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        sqlx::query(
            "INSERT INTO user_usage (id, user_id, date, enrichment_count) VALUES ('u1', 'user1', ?, 250)",
        )
        .bind(&today)
        .execute(&pool)
        .await
        .unwrap();

        let claude = MockClaude {
            response: mock_enrichment_response(1),
        };

        let result = enrich_tracks(&pool, &claude, "user1").await;

        assert!(matches!(result, Err(EnrichmentError::CostCapExceeded(_))));
    }

    // Test 4: Malformed LLM response — tracks get enrichment_error set
    #[tokio::test]
    async fn test_malformed_response_marks_errors() {
        let pool = setup_pool_with_unenriched_tracks(2).await;
        let claude = MockClaude {
            response: "This is not JSON at all!".to_string(),
        };

        let result = enrich_tracks(&pool, &claude, "user1").await.unwrap();

        assert_eq!(result.enriched, 0);
        assert_eq!(result.errors, 2);

        // Verify tracks have enrichment_error set
        let error: Option<String> =
            sqlx::query_scalar("SELECT enrichment_error FROM tracks WHERE id = 't0'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(error.is_some());
        assert!(error.unwrap().contains("Parse error"));
    }

    // Test 5: Usage tracking — verify user_usage table incremented
    #[tokio::test]
    async fn test_usage_counter_incremented() {
        let pool = setup_pool_with_unenriched_tracks(3).await;
        let claude = MockClaude {
            response: mock_enrichment_response(3),
        };

        enrich_tracks(&pool, &claude, "user1").await.unwrap();

        let count = crate::db::tracks::get_daily_enrichment_count(&pool, "user1")
            .await
            .unwrap();

        assert_eq!(count, 3);
    }

    // Test 6: Verify needs_enrichment flag and enriched_at timestamp after enrichment
    #[tokio::test]
    async fn test_db_metadata_flags_updated() {
        let pool = setup_pool_with_unenriched_tracks(1).await;
        let claude = MockClaude {
            response: mock_enrichment_response(1),
        };

        enrich_tracks(&pool, &claude, "user1").await.unwrap();

        // Check needs_enrichment is now 0
        let needs: i32 = sqlx::query_scalar("SELECT needs_enrichment FROM tracks WHERE id = 't0'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(needs, 0);

        // Check enriched_at is set
        let enriched_at: Option<String> =
            sqlx::query_scalar("SELECT enriched_at FROM tracks WHERE id = 't0'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(enriched_at.is_some());
    }
}
