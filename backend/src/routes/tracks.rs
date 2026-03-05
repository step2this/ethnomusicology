use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::db::models::TrackRow;
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Response types (matching openapi.yaml TrackListResponse)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct TrackResponse {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_ms: Option<i64>,
    pub bpm: Option<f64>,
    pub key: Option<String>,
    pub energy: Option<f64>,
    pub source: String,
    pub source_id: Option<String>,
    pub preview_url: Option<String>,
    pub album_art_url: Option<String>,
    pub date_added: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrackListResponse {
    pub data: Vec<TrackResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: i64,
}

impl From<TrackRow> for TrackResponse {
    fn from(row: TrackRow) -> Self {
        // Prefer Deezer preview URL if available, fall back to Spotify
        let preview_url = row.deezer_preview_url.or(row.spotify_preview_url);

        TrackResponse {
            id: row.id,
            title: row.title,
            artist: row.artist.unwrap_or_default(),
            album: row.album,
            duration_ms: row.duration_ms,
            bpm: row.bpm,
            key: row.camelot_key,
            energy: row.energy,
            source: row.source,
            source_id: row.spotify_uri,
            preview_url,
            album_art_url: row.album_art_url,
            date_added: row
                .created_at
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListTracksParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    #[serde(default = "default_sort")]
    pub sort: String,
    #[serde(default = "default_order")]
    pub order: String,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    25
}
fn default_sort() -> String {
    "date_added".to_string()
}
fn default_order() -> String {
    "desc".to_string()
}

// ---------------------------------------------------------------------------
// Valid sort and order values
// ---------------------------------------------------------------------------

const VALID_SORTS: &[&str] = &["title", "artist", "bpm", "key", "date_added"];
const VALID_ORDERS: &[&str] = &["asc", "desc"];

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

async fn list_tracks(
    State(pool): State<SqlitePool>,
    Query(params): Query<ListTracksParams>,
) -> Result<Json<TrackListResponse>, AppError> {
    // Validate params
    if params.page < 1 {
        return Err(AppError::BadRequest(
            "Page must be a positive integer".to_string(),
        ));
    }
    if params.per_page < 1 || params.per_page > 100 {
        return Err(AppError::BadRequest(
            "per_page must be between 1 and 100".to_string(),
        ));
    }
    if !VALID_SORTS.contains(&params.sort.as_str()) {
        return Err(AppError::BadRequest(
            "Invalid sort field. Must be one of: title, artist, bpm, key, date_added".to_string(),
        ));
    }
    if !VALID_ORDERS.contains(&params.order.as_str()) {
        return Err(AppError::BadRequest(
            "Invalid order. Must be one of: asc, desc".to_string(),
        ));
    }

    let (rows, total) = crate::db::tracks::list_tracks_paginated(
        &pool,
        params.page,
        params.per_page,
        &params.sort,
        &params.order,
    )
    .await
    .map_err(AppError::Database)?;

    let total_pages = if total == 0 {
        0
    } else {
        (total + params.per_page as i64 - 1) / params.per_page as i64
    };

    let data: Vec<TrackResponse> = rows.into_iter().map(TrackResponse::from).collect();

    Ok(Json(TrackListResponse {
        data,
        page: params.page,
        per_page: params.per_page,
        total,
        total_pages,
    }))
}

// ---------------------------------------------------------------------------
// Retry errored tracks handler
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct RetryErroredResponse {
    reset: u64,
}

async fn retry_errored_tracks(
    State(pool): State<SqlitePool>,
) -> Result<Json<RetryErroredResponse>, AppError> {
    let reset = crate::db::tracks::retry_errored_tracks(&pool)
        .await
        .map_err(AppError::Database)?;
    Ok(Json(RetryErroredResponse { reset }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn tracks_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/tracks", get(list_tracks))
        .route("/tracks/retry-errored", post(retry_errored_tracks))
        .with_state(pool)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn setup() -> Router {
        let pool = crate::db::create_test_pool().await;
        tracks_router(pool)
    }

    async fn setup_with_data() -> Router {
        let pool = crate::db::create_test_pool().await;

        // Insert test tracks
        sqlx::query("INSERT INTO tracks (id, title, album, duration_ms, spotify_uri, source) VALUES (?, ?, ?, ?, ?, ?)")
            .bind("t1").bind("Alpha Song").bind("Album A").bind(180000i64).bind("spotify:track:a1").bind("spotify")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO tracks (id, title, source) VALUES (?, ?, ?)")
            .bind("t2")
            .bind("Beta Song")
            .bind("spotify")
            .execute(&pool)
            .await
            .unwrap();

        // Insert artist and link
        sqlx::query("INSERT INTO artists (id, name) VALUES (?, ?)")
            .bind("a1")
            .bind("Test Artist")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES (?, ?)")
            .bind("t1")
            .bind("a1")
            .execute(&pool)
            .await
            .unwrap();

        tracks_router(pool)
    }

    #[tokio::test]
    async fn test_list_tracks_default_params() {
        let app = setup_with_data().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tracks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["page"], 1);
        assert_eq!(json["per_page"], 25);
        assert!(json["data"].is_array());
    }

    #[tokio::test]
    async fn test_list_tracks_invalid_page_zero() {
        let app = setup().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tracks?page=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn test_list_tracks_invalid_per_page() {
        let app = setup().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tracks?per_page=200")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    async fn test_list_tracks_invalid_sort() {
        let app = setup().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tracks?sort=invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    async fn test_list_tracks_empty_catalog() {
        let app = setup().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tracks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"].as_array().unwrap().len(), 0);
        assert_eq!(json["total"], 0);
        assert_eq!(json["total_pages"], 0);
    }

    #[tokio::test]
    async fn test_list_tracks_page_beyond_total() {
        let app = setup_with_data().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tracks?page=999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"].as_array().unwrap().len(), 0);
        assert!(json["total"].as_i64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_retry_errored_no_errored_tracks_returns_zero() {
        let pool = crate::db::create_test_pool().await;
        let app = tracks_router(pool);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tracks/retry-errored")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["reset"], 0);
    }

    #[tokio::test]
    async fn test_retry_errored_resets_errored_tracks() {
        let pool = crate::db::create_test_pool().await;

        // Insert two tracks with enrichment errors
        sqlx::query(
            "INSERT INTO tracks (id, title, source, needs_enrichment, enrichment_error) VALUES (?, ?, 'spotify', 0, 'Parse error')",
        )
        .bind("t-err-1")
        .bind("Error Track 1")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO tracks (id, title, source, needs_enrichment, enrichment_error) VALUES (?, ?, 'spotify', 0, 'Claude API error')",
        )
        .bind("t-err-2")
        .bind("Error Track 2")
        .execute(&pool)
        .await
        .unwrap();

        let app = tracks_router(pool.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tracks/retry-errored")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["reset"], 2);

        // Verify tracks now have needs_enrichment = 1 and error cleared
        let (needs, error): (i32, Option<String>) = sqlx::query_as(
            "SELECT needs_enrichment, enrichment_error FROM tracks WHERE id = 't-err-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(needs, 1);
        assert!(error.is_none());
    }
}
