use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::crate_models::{CrateRow, CrateSummary, CrateTrackRow};
use crate::db::crates;
use crate::db::setlists as db_setlists;
use crate::error::AppError;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct CrateRouteState {
    pub pool: SqlitePool,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CreateCrateRequest {
    name: String,
    description: Option<String>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ListCratesResponse {
    crates: Vec<CrateSummary>,
}

#[derive(Serialize)]
struct CrateDetailResponse {
    #[serde(flatten)]
    crate_row: CrateRow,
    tracks: Vec<CrateTrackRow>,
}

#[derive(Serialize)]
struct AddSetlistResponse {
    tracks_added: i64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

fn extract_user_id(headers: &HeaderMap) -> &str {
    headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("dev-user")
}

async fn list_crates_handler(
    State(state): State<Arc<CrateRouteState>>,
    headers: HeaderMap,
) -> Result<Json<ListCratesResponse>, AppError> {
    let user_id = extract_user_id(&headers);
    let crate_list = crates::list_crates(&state.pool, user_id)
        .await
        .map_err(AppError::Database)?;
    Ok(Json(ListCratesResponse { crates: crate_list }))
}

async fn create_crate_handler(
    State(state): State<Arc<CrateRouteState>>,
    headers: HeaderMap,
    Json(req): Json<CreateCrateRequest>,
) -> Result<(StatusCode, Json<CrateRow>), AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    let user_id = extract_user_id(&headers);
    let id = Uuid::new_v4().to_string();
    crates::create_crate(
        &state.pool,
        &id,
        user_id,
        &req.name,
        req.description.as_deref(),
    )
    .await
    .map_err(AppError::Database)?;
    let row = crates::get_crate(&state.pool, &id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("crate not found after insert".to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_crate_handler(
    State(state): State<Arc<CrateRouteState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<CrateDetailResponse>, AppError> {
    let user_id = extract_user_id(&headers);
    let crate_row = crates::get_crate(&state.pool, &id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("crate {id} not found")))?;
    if crate_row.user_id != user_id {
        return Err(AppError::NotFound(format!("crate {id} not found")));
    }
    let tracks = crates::get_crate_tracks(&state.pool, &id)
        .await
        .map_err(AppError::Database)?;
    Ok(Json(CrateDetailResponse { crate_row, tracks }))
}

async fn delete_crate_handler(
    State(state): State<Arc<CrateRouteState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let user_id = extract_user_id(&headers);
    let crate_row = crates::get_crate(&state.pool, &id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("crate {id} not found")))?;
    if crate_row.user_id != user_id {
        return Err(AppError::NotFound(format!("crate {id} not found")));
    }
    crates::delete_crate(&state.pool, &id)
        .await
        .map_err(AppError::Database)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_setlist_handler(
    State(state): State<Arc<CrateRouteState>>,
    Path((id, setlist_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<AddSetlistResponse>, AppError> {
    let user_id = extract_user_id(&headers);
    let crate_row = crates::get_crate(&state.pool, &id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("crate {id} not found")))?;
    if crate_row.user_id != user_id {
        return Err(AppError::NotFound(format!("crate {id} not found")));
    }
    let setlist_row = db_setlists::get_setlist(&state.pool, &setlist_id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("setlist {setlist_id} not found")))?;
    if setlist_row.user_id != user_id {
        return Err(AppError::NotFound(format!(
            "setlist {setlist_id} not found"
        )));
    }
    let tracks_added = crates::add_tracks_from_setlist(&state.pool, &id, &setlist_id)
        .await
        .map_err(AppError::Database)?;
    Ok(Json(AddSetlistResponse { tracks_added }))
}

async fn remove_track_handler(
    State(state): State<Arc<CrateRouteState>>,
    Path((id, track_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let user_id = extract_user_id(&headers);
    let crate_row = crates::get_crate(&state.pool, &id)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("crate {id} not found")))?;
    if crate_row.user_id != user_id {
        return Err(AppError::NotFound(format!("crate {id} not found")));
    }
    let affected = crates::remove_crate_track(&state.pool, &id, &track_id)
        .await
        .map_err(AppError::Database)?;
    if affected == 0 {
        return Err(AppError::NotFound(format!(
            "track {track_id} not found in crate {id}"
        )));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn crate_routes(state: Arc<CrateRouteState>) -> Router {
    Router::new()
        .route(
            "/crates",
            get(list_crates_handler).post(create_crate_handler),
        )
        .route(
            "/crates/{id}",
            get(get_crate_handler).delete(delete_crate_handler),
        )
        .route(
            "/crates/{id}/add-setlist/{setlist_id}",
            post(add_setlist_handler),
        )
        .route(
            "/crates/{id}/tracks/{track_id}",
            delete(remove_track_handler),
        )
        .with_state(state)
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

    async fn setup() -> (Router, SqlitePool) {
        let pool = crate::db::create_test_pool().await;
        let state = Arc::new(CrateRouteState { pool: pool.clone() });
        (crate_routes(state), pool)
    }

    async fn get_json(app: Router, uri: &str) -> (u16, serde_json::Value) {
        let resp = app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header("X-User-Id", "user-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (status, json)
    }

    async fn post_json(
        app: Router,
        uri: &str,
        body: serde_json::Value,
    ) -> (u16, serde_json::Value) {
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("content-type", "application/json")
                    .header("X-User-Id", "user-1")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (status, json)
    }

    async fn delete_req(app: Router, uri: &str) -> u16 {
        let resp = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(uri)
                    .header("X-User-Id", "user-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        resp.status().as_u16()
    }

    #[tokio::test]
    async fn test_list_crates_empty() {
        let (app, _) = setup().await;
        let (status, json) = get_json(app, "/crates").await;
        assert_eq!(status, 200);
        assert_eq!(json["crates"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_create_crate_returns_201() {
        let (app, _) = setup().await;
        let (status, json) = post_json(
            app,
            "/crates",
            serde_json::json!({ "name": "Friday Night", "description": "House set" }),
        )
        .await;
        assert_eq!(status, 201);
        assert!(json["id"].is_string());
        assert_eq!(json["name"], "Friday Night");
        assert_eq!(json["description"], "House set");
    }

    #[tokio::test]
    async fn test_create_crate_empty_name_returns_400() {
        let (app, _) = setup().await;
        let (status, json) = post_json(app, "/crates", serde_json::json!({ "name": "" })).await;
        assert_eq!(status, 400);
        assert_eq!(json["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn test_get_crate_not_found() {
        let (app, _) = setup().await;
        let (status, json) = get_json(app, "/crates/nonexistent").await;
        assert_eq!(status, 404);
        assert_eq!(json["error"]["code"], "NOT_FOUND");
    }

    #[tokio::test]
    async fn test_get_crate_returns_tracks() {
        let (_, pool) = setup().await;

        // Create crate directly in DB
        crates::create_crate(&pool, "c1", "user-1", "Alpha", None)
            .await
            .unwrap();
        sqlx::query("INSERT INTO crate_tracks (id, crate_id, title, artist) VALUES (?, ?, ?, ?)")
            .bind("ct1")
            .bind("c1")
            .bind("Track A")
            .bind("Artist A")
            .execute(&pool)
            .await
            .unwrap();

        let app = crate_routes(Arc::new(CrateRouteState { pool }));
        let (status, json) = get_json(app, "/crates/c1").await;
        assert_eq!(status, 200);
        assert_eq!(json["id"], "c1");
        assert_eq!(json["tracks"].as_array().unwrap().len(), 1);
        assert_eq!(json["tracks"][0]["title"], "Track A");
    }

    #[tokio::test]
    async fn test_delete_crate_returns_204() {
        let (_, pool) = setup().await;
        crates::create_crate(&pool, "c1", "user-1", "Alpha", None)
            .await
            .unwrap();

        let app = crate_routes(Arc::new(CrateRouteState { pool }));
        let status = delete_req(app, "/crates/c1").await;
        assert_eq!(status, 204);
    }

    #[tokio::test]
    async fn test_delete_crate_not_found_returns_404() {
        let (app, _) = setup().await;
        let status = delete_req(app, "/crates/nonexistent").await;
        assert_eq!(status, 404);
    }

    #[tokio::test]
    async fn test_add_setlist_to_crate() {
        let (_, pool) = setup().await;

        crates::create_crate(&pool, "c1", "user-1", "Alpha", None)
            .await
            .unwrap();

        // Insert setlist + tracks
        sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES (?, ?, ?, ?)")
            .bind("sl-1")
            .bind("user-1")
            .bind("test")
            .bind("test")
            .execute(&pool)
            .await
            .unwrap();
        for i in 1..=3i32 {
            sqlx::query(
                "INSERT INTO setlist_tracks (id, setlist_id, position, original_position, title, artist, source) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(format!("st{i}"))
            .bind("sl-1")
            .bind(i)
            .bind(i)
            .bind(format!("Track {i}"))
            .bind(format!("Artist {i}"))
            .bind("suggestion")
            .execute(&pool)
            .await
            .unwrap();
        }

        let app = crate_routes(Arc::new(CrateRouteState { pool }));
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/crates/c1/add-setlist/sl-1")
                    .header("X-User-Id", "user-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["tracks_added"], 3);
    }

    #[tokio::test]
    async fn test_remove_track_from_crate() {
        let (_, pool) = setup().await;
        crates::create_crate(&pool, "c1", "user-1", "Alpha", None)
            .await
            .unwrap();
        sqlx::query("INSERT INTO crate_tracks (id, crate_id, title, artist) VALUES (?, ?, ?, ?)")
            .bind("ct1")
            .bind("c1")
            .bind("Track A")
            .bind("Artist A")
            .execute(&pool)
            .await
            .unwrap();

        let app = crate_routes(Arc::new(CrateRouteState { pool }));
        let status = delete_req(app, "/crates/c1/tracks/ct1").await;
        assert_eq!(status, 204);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_track_returns_404() {
        let (_, pool) = setup().await;
        crates::create_crate(&pool, "c1", "user-1", "Alpha", None)
            .await
            .unwrap();

        let app = crate_routes(Arc::new(CrateRouteState { pool }));
        let status = delete_req(app, "/crates/c1/tracks/nonexistent").await;
        assert_eq!(status, 404);
    }
}
