// Admin routes — protected by X-Admin-Token header

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct WipeCatalogRequest {
    confirm: bool,
}

#[derive(Serialize)]
struct WipeDeleted {
    tracks: u64,
    artists: u64,
    imports: u64,
}

#[derive(Serialize)]
struct WipeResponse {
    deleted: WipeDeleted,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn internal_error(msg: String) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": {"code": "INTERNAL_ERROR", "message": msg}
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

async fn wipe_catalog(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Json(body): Json<WipeCatalogRequest>,
) -> Result<Json<WipeResponse>, Response> {
    // Auth check
    let expected = std::env::var("ADMIN_TOKEN").unwrap_or_default();
    let provided = headers
        .get("X-Admin-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if expected.is_empty() || provided != expected {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {"code": "FORBIDDEN", "message": "Invalid or missing admin token"}
            })),
        )
            .into_response());
    }

    if !body.confirm {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": {"code": "INVALID_REQUEST", "message": "confirm must be true"}
            })),
        )
            .into_response());
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| internal_error(e.to_string()))?;

    // 1. NULL setlist_tracks.track_id references to spotify tracks
    sqlx::query(
        "UPDATE setlist_tracks SET track_id = NULL \
         WHERE track_id IN (SELECT id FROM tracks WHERE spotify_uri IS NOT NULL)",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| internal_error(e.to_string()))?;

    // 2. NULL setlist_version_tracks.track_id references to spotify tracks
    sqlx::query(
        "UPDATE setlist_version_tracks SET track_id = NULL \
         WHERE track_id IN (SELECT id FROM tracks WHERE spotify_uri IS NOT NULL)",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| internal_error(e.to_string()))?;

    // 3. Clear import_tracks join table
    sqlx::query("DELETE FROM import_tracks")
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error(e.to_string()))?;

    // 4-6. Remove tag/occasion/artist associations for spotify tracks
    sqlx::query(
        "DELETE FROM track_artists WHERE track_id IN (SELECT id FROM tracks WHERE spotify_uri IS NOT NULL)",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| internal_error(e.to_string()))?;

    sqlx::query(
        "DELETE FROM track_tags WHERE track_id IN (SELECT id FROM tracks WHERE spotify_uri IS NOT NULL)",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| internal_error(e.to_string()))?;

    sqlx::query(
        "DELETE FROM track_occasions WHERE track_id IN (SELECT id FROM tracks WHERE spotify_uri IS NOT NULL)",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| internal_error(e.to_string()))?;

    // 7. Remove playlist associations for spotify tracks
    sqlx::query(
        "DELETE FROM playlist_tracks WHERE track_id IN (SELECT id FROM tracks WHERE spotify_uri IS NOT NULL)",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| internal_error(e.to_string()))?;

    // 8. Delete spotify tracks
    let tracks_deleted = sqlx::query("DELETE FROM tracks WHERE spotify_uri IS NOT NULL")
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error(e.to_string()))?
        .rows_affected();

    // 9. Delete orphaned artists
    let artists_deleted = sqlx::query(
        "DELETE FROM artists WHERE id NOT IN (SELECT DISTINCT artist_id FROM track_artists)",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| internal_error(e.to_string()))?
    .rows_affected();

    // 10. Delete all spotify import records
    let imports_deleted = sqlx::query("DELETE FROM spotify_imports")
        .execute(&mut *tx)
        .await
        .map_err(|e| internal_error(e.to_string()))?
        .rows_affected();

    tx.commit()
        .await
        .map_err(|e| internal_error(e.to_string()))?;

    Ok(Json(WipeResponse {
        deleted: WipeDeleted {
            tracks: tracks_deleted,
            artists: artists_deleted,
            imports: imports_deleted,
        },
    }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn admin_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/admin/wipe-catalog", post(wipe_catalog))
        .with_state(pool)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_wipe_missing_token_returns_403() {
        std::env::set_var("ADMIN_TOKEN", "secret-token");
        let pool = crate::db::create_test_pool().await;
        let app = admin_router(pool);

        let body = serde_json::json!({ "confirm": true });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/wipe-catalog")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_wipe_wrong_token_returns_403() {
        std::env::set_var("ADMIN_TOKEN", "secret-token");
        let pool = crate::db::create_test_pool().await;
        let app = admin_router(pool);

        let body = serde_json::json!({ "confirm": true });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/wipe-catalog")
                    .header("content-type", "application/json")
                    .header("X-Admin-Token", "wrong-token")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_wipe_confirm_false_returns_400() {
        std::env::set_var("ADMIN_TOKEN", "secret-token");
        let pool = crate::db::create_test_pool().await;
        let app = admin_router(pool);

        let body = serde_json::json!({ "confirm": false });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/wipe-catalog")
                    .header("content-type", "application/json")
                    .header("X-Admin-Token", "secret-token")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_wipe_success_with_fk_relationships() {
        // Note: all admin tests use the same token value, so env::set_var race is benign.
        std::env::set_var("ADMIN_TOKEN", "secret-token");
        let pool = crate::db::create_test_pool().await;

        // 1. Insert a spotify track + artist + association
        sqlx::query("INSERT INTO tracks (id, title, source, spotify_uri) VALUES (?, ?, ?, ?)")
            .bind("t1")
            .bind("Your Love")
            .bind("spotify")
            .bind("spotify:track:abc123")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO artists (id, name) VALUES (?, ?)")
            .bind("a1")
            .bind("Frankie Knuckles")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES (?, ?)")
            .bind("t1")
            .bind("a1")
            .execute(&pool)
            .await
            .unwrap();

        // 2. Insert a spotify import + import_tracks link
        sqlx::query("INSERT INTO users (id, display_name) VALUES (?, ?)")
            .bind("u1")
            .bind("Test User")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO spotify_imports (id, user_id, spotify_playlist_id, status) VALUES (?, ?, ?, ?)",
        )
        .bind("imp1")
        .bind("u1")
        .bind("playlist123")
        .bind("completed")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO import_tracks (import_id, track_id) VALUES (?, ?)")
            .bind("imp1")
            .bind("t1")
            .execute(&pool)
            .await
            .unwrap();

        // 3. Insert a setlist with a track referencing the spotify track
        sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES (?, ?, ?, ?)")
            .bind("s1")
            .bind("u1")
            .bind("deep house")
            .bind("claude-sonnet")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO setlist_tracks (id, setlist_id, track_id, position, original_position, title, artist, source) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("st1")
        .bind("s1")
        .bind("t1")
        .bind(1)
        .bind(1)
        .bind("Your Love")
        .bind("Frankie Knuckles")
        .bind("catalog")
        .execute(&pool)
        .await
        .unwrap();

        // Call wipe endpoint
        let app = admin_router(pool.clone());

        let body = serde_json::json!({ "confirm": true });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/wipe-catalog")
                    .header("content-type", "application/json")
                    .header("X-Admin-Token", "secret-token")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(resp["deleted"]["tracks"], 1);
        assert_eq!(resp["deleted"]["artists"], 1);
        assert_eq!(resp["deleted"]["imports"], 1);

        // Verify tracks are gone
        let track_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tracks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(track_count.0, 0);

        // Verify setlist_tracks row still exists but track_id is NULL
        let st_row: (Option<String>,) =
            sqlx::query_as("SELECT track_id FROM setlist_tracks WHERE id = 'st1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(
            st_row.0.is_none(),
            "setlist_tracks.track_id should be NULL after wipe"
        );

        // Verify setlist_tracks still has denormalized data
        let st_title: (String,) =
            sqlx::query_as("SELECT title FROM setlist_tracks WHERE id = 'st1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(st_title.0, "Your Love");

        // Verify import_tracks cleared
        let imp_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM import_tracks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(imp_count.0, 0);
    }
}
