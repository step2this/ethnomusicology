use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum DeezerError {
    #[error("HTTP request error: {0}")]
    Request(String),

    #[error("Failed to parse Deezer response: {0}")]
    ParseError(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<reqwest::Error> for DeezerError {
    fn from(e: reqwest::Error) -> Self {
        DeezerError::Request(e.to_string())
    }
}

impl From<sqlx::Error> for DeezerError {
    fn from(e: sqlx::Error) -> Self {
        DeezerError::Database(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Deezer API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DeezerTrack {
    id: i64,
    title: String,
    preview: String,
    duration: u64,
    artist: DeezerArtist,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DeezerArtist {
    name: String,
}

#[derive(Debug, Deserialize)]
struct DeezerSearchResponse {
    data: Vec<DeezerTrack>,
}

// ---------------------------------------------------------------------------
// Main enrichment function
// ---------------------------------------------------------------------------

/// Search Deezer for tracks that don't have deezer_id set and populate
/// deezer_id and deezer_preview_url if a match is found.
pub async fn enrich_tracks_with_deezer(pool: &PgPool) -> Result<usize, anyhow::Error> {
    // 1. Get tracks that need Deezer enrichment (deezer_id IS NULL)
    let tracks_to_enrich: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, title, artist FROM (
            SELECT t.id, t.title,
                   STRING_AGG(a.name, ', ') AS artist
            FROM tracks t
            LEFT JOIN track_artists ta ON t.id = ta.track_id
            LEFT JOIN artists a ON ta.artist_id = a.id
            WHERE t.deezer_id IS NULL
            GROUP BY t.id
        ) sub LIMIT 100",
    )
    .fetch_all(pool)
    .await?;

    if tracks_to_enrich.is_empty() {
        return Ok(0);
    }

    let client = Client::new();
    let mut enriched_count = 0;

    // 2. For each track, search Deezer
    for (track_id, title, artist) in tracks_to_enrich {
        let artist_str = artist.unwrap_or_else(|| "Unknown".to_string());
        let search_query = format!("{} {}", &artist_str, &title);

        // Rate limiting: 100ms delay between requests to be respectful to the free API
        sleep(Duration::from_millis(100)).await;

        // Call Deezer search API
        match search_deezer(&client, &search_query).await {
            Ok(Some(deezer_track)) => {
                // Update the track with Deezer info (skip empty preview URLs)
                let preview = if deezer_track.preview.is_empty() {
                    None
                } else {
                    Some(&deezer_track.preview)
                };
                match sqlx::query(
                    "UPDATE tracks SET deezer_id = $1, deezer_preview_url = $2 WHERE id = $3",
                )
                .bind(deezer_track.id)
                .bind(preview)
                .bind(&track_id)
                .execute(pool)
                .await
                {
                    Ok(_) => enriched_count += 1,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to update track {} with Deezer data: {}",
                            track_id,
                            e
                        );
                    }
                }
            }
            Ok(None) => {
                // No match found — mark with sentinel deezer_id = -1 so we don't retry
                tracing::debug!("No Deezer match found for: {}", &search_query);
                let _ = sqlx::query("UPDATE tracks SET deezer_id = -1 WHERE id = $1")
                    .bind(&track_id)
                    .execute(pool)
                    .await;
            }
            Err(e) => {
                // Log the error but continue with the next track
                tracing::warn!(
                    "Deezer search error for track {} ({}): {}",
                    track_id,
                    &search_query,
                    e
                );
            }
        }
    }

    Ok(enriched_count)
}

// ---------------------------------------------------------------------------
// Helper function to search Deezer
// ---------------------------------------------------------------------------

async fn search_deezer(client: &Client, query: &str) -> Result<Option<DeezerTrack>, DeezerError> {
    let url = "https://api.deezer.com/search";

    let response = client
        .get(url)
        .query(&[("q", query), ("limit", "1")])
        .send()
        .await?;

    let search_result: DeezerSearchResponse = response
        .json()
        .await
        .map_err(|e| DeezerError::ParseError(e.to_string()))?;

    Ok(search_result.data.first().cloned())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enrich_tracks_with_deezer_empty_catalog() {
        let pool = crate::db::create_test_pool().await;

        let result = enrich_tracks_with_deezer(&pool).await.unwrap();

        assert_eq!(result, 0);
        pool.close().await;
    }

    #[tokio::test]
    async fn test_enrich_tracks_with_deezer_skips_already_enriched() {
        let pool = crate::db::create_test_pool().await;

        // Insert a track that already has deezer_id
        sqlx::query(
            "INSERT INTO tracks (id, title, source, deezer_id, deezer_preview_url)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind("t1")
        .bind("Test Track")
        .bind("spotify")
        .bind(12345i64)
        .bind("https://example.com/preview.mp3")
        .execute(&pool)
        .await
        .unwrap();

        let result = enrich_tracks_with_deezer(&pool).await.unwrap();

        // Should return 0 because the track is already enriched
        assert_eq!(result, 0);
        pool.close().await;
    }

    #[tokio::test]
    async fn test_enrich_tracks_with_deezer_finds_unenriched() {
        let pool = crate::db::create_test_pool().await;

        // Insert an unenriched track (deezer_id IS NULL)
        sqlx::query("INSERT INTO tracks (id, title, source) VALUES ($1, $2, $3)")
            .bind("t1")
            .bind("Test Track")
            .bind("spotify")
            .execute(&pool)
            .await
            .unwrap();

        // Insert an artist and link
        sqlx::query("INSERT INTO artists (id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind("a1")
            .bind("Test Artist")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ($1, $2)")
            .bind("t1")
            .bind("a1")
            .execute(&pool)
            .await
            .unwrap();

        // Query should find the track (we won't test the actual Deezer API call here,
        // just that the function recognizes tracks that need enrichment)
        let tracks_to_enrich: Vec<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, title, artist FROM (
                SELECT t.id, t.title,
                       STRING_AGG(a.name, ', ') AS artist
                FROM tracks t
                LEFT JOIN track_artists ta ON t.id = ta.track_id
                LEFT JOIN artists a ON ta.artist_id = a.id
                WHERE t.deezer_id IS NULL
                GROUP BY t.id
            ) sub",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(tracks_to_enrich.len(), 1);
        assert_eq!(tracks_to_enrich[0].0, "t1");
        assert_eq!(tracks_to_enrich[0].1, "Test Track");
        assert_eq!(tracks_to_enrich[0].2, Some("Test Artist".to_string()));
        pool.close().await;
    }
}
