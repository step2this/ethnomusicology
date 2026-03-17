use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use crate::api::claude::ClaudeClientTrait;
use crate::db::models::SetlistSummary;
use crate::db::setlists as db;
use crate::services::camelot::EnergyProfile;
use crate::services::setlist::{
    self, BpmRange, GenerateSetlistRequest, SetlistError, SetlistResponse,
};

// ---------------------------------------------------------------------------
// State (M1: renamed from SetlistState to SetlistRouteState)
// ---------------------------------------------------------------------------

pub struct SetlistRouteState {
    pub pool: PgPool,
    pub claude: Arc<dyn ClaudeClientTrait>,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub track_count: Option<u32>,
    #[serde(default)]
    pub energy_profile: Option<String>,
    #[serde(default)]
    pub source_playlist_id: Option<String>,
    #[serde(default)]
    pub seed_tracklist: Option<String>,
    #[serde(default)]
    pub creative_mode: Option<bool>,
    #[serde(default)]
    pub bpm_range: Option<BpmRangeRequest>,
    #[serde(default)]
    pub verify: Option<bool>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Serialize)]
pub struct ListSetlistsResponse {
    pub setlists: Vec<SetlistSummary>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Deserialize)]
pub struct RenameRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct DuplicateRequest {
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct BpmRangeRequest {
    pub min: f64,
    pub max: f64,
}

#[derive(Deserialize)]
pub struct ArrangeRequest {
    #[serde(default)]
    pub energy_profile: Option<String>,
}

// ---------------------------------------------------------------------------
// Response types (extended for ST-006)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct BpmWarningResponse {
    pub from_position: i32,
    pub to_position: i32,
    pub bpm_delta: f64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn generate_setlist_handler(
    State(state): State<Arc<SetlistRouteState>>,
    headers: HeaderMap,
    Json(req): Json<GenerateRequest>,
) -> Result<(axum::http::StatusCode, Json<SetlistResponse>), SetlistError> {
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default-user");

    // Parse energy_profile string into enum
    let energy_profile = match req.energy_profile {
        Some(ref s) => Some(
            s.parse::<EnergyProfile>()
                .map_err(SetlistError::InvalidEnergyProfile)?,
        ),
        None => None,
    };

    let service_req = GenerateSetlistRequest {
        user_id: user_id.to_string(),
        prompt: req.prompt,
        track_count: req.track_count,
        energy_profile,
        source_playlist_id: req.source_playlist_id,
        seed_tracklist: req.seed_tracklist,
        creative_mode: req.creative_mode,
        bpm_range: req.bpm_range.map(|r| BpmRange {
            min: r.min,
            max: r.max,
        }),
        verify: req.verify.unwrap_or(false),
        name: req.name,
    };

    let response =
        setlist::generate_setlist_from_request(&state.pool, state.claude.as_ref(), service_req)
            .await?;

    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

/// L1: Route handler now delegates entirely to service function.
/// T8: Accepts optional JSON body with energy_profile for arrangement.
/// Uses Option<Json<ArrangeRequest>> so clients sending no body don't get 400.
async fn arrange_setlist_handler(
    State(state): State<Arc<SetlistRouteState>>,
    Path(id): Path<String>,
    body: Option<Json<ArrangeRequest>>,
) -> Result<Json<SetlistResponse>, SetlistError> {
    let energy_profile = match body {
        Some(Json(req)) => match req.energy_profile {
            Some(ref s) => Some(
                s.parse::<EnergyProfile>()
                    .map_err(SetlistError::InvalidEnergyProfile)?,
            ),
            None => None,
        },
        None => None,
    };

    let response = setlist::arrange_setlist(&state.pool, &id, energy_profile).await?;
    Ok(Json(response))
}

async fn get_setlist_handler(
    State(state): State<Arc<SetlistRouteState>>,
    Path(id): Path<String>,
) -> Result<Json<SetlistResponse>, SetlistError> {
    let response = setlist::get_setlist(&state.pool, &id).await?;
    Ok(Json(response))
}

async fn list_setlists_handler(
    State(state): State<Arc<SetlistRouteState>>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListSetlistsResponse>, SetlistError> {
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default-user");

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let (setlists, total) = tokio::try_join!(
        db::list_setlists(&state.pool, user_id, per_page, offset),
        db::count_setlists(&state.pool, user_id),
    )
    .map_err(|e| SetlistError::Database(e.to_string()))?;

    Ok(Json(ListSetlistsResponse {
        setlists,
        total,
        page,
        per_page,
    }))
}

async fn delete_setlist_handler(
    State(state): State<Arc<SetlistRouteState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, SetlistError> {
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default-user");
    let row = db::get_setlist(&state.pool, &id)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?
        .ok_or_else(|| SetlistError::NotFound(format!("Setlist {id} not found")))?;
    if row.user_id != user_id {
        return Err(SetlistError::NotFound(format!("Setlist {id} not found")));
    }
    let deleted = db::delete_setlist(&state.pool, &id)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(SetlistError::NotFound(format!("Setlist {id} not found")))
    }
}

async fn rename_setlist_handler(
    State(state): State<Arc<SetlistRouteState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<RenameRequest>,
) -> Result<Json<SetlistResponse>, SetlistError> {
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default-user");
    let row = db::get_setlist(&state.pool, &id)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?
        .ok_or_else(|| SetlistError::NotFound(format!("Setlist {id} not found")))?;
    if row.user_id != user_id {
        return Err(SetlistError::NotFound(format!("Setlist {id} not found")));
    }
    db::update_setlist_name(&state.pool, &id, &req.name)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?;
    let response = setlist::get_setlist(&state.pool, &id).await?;
    Ok(Json(response))
}

async fn duplicate_setlist_handler(
    State(state): State<Arc<SetlistRouteState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Option<Json<DuplicateRequest>>,
) -> Result<(StatusCode, Json<SetlistResponse>), SetlistError> {
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default-user");
    let row = db::get_setlist(&state.pool, &id)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?
        .ok_or_else(|| SetlistError::NotFound(format!("Setlist {id} not found")))?;
    if row.user_id != user_id {
        return Err(SetlistError::NotFound(format!("Setlist {id} not found")));
    }
    let new_name = body.as_ref().and_then(|b| b.name.as_deref());
    let new_id = db::duplicate_setlist(&state.pool, &id, new_name)
        .await
        .map_err(|e| SetlistError::Database(e.to_string()))?
        .ok_or_else(|| SetlistError::NotFound(format!("Setlist {id} not found")))?;
    let response = setlist::get_setlist(&state.pool, &new_id).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn setlist_router(state: Arc<SetlistRouteState>) -> Router {
    Router::new()
        .route("/setlists", get(list_setlists_handler))
        .route("/setlists/generate", post(generate_setlist_handler))
        .route("/setlists/{id}/arrange", post(arrange_setlist_handler))
        .route("/setlists/{id}/duplicate", post(duplicate_setlist_handler))
        .route(
            "/setlists/{id}",
            get(get_setlist_handler)
                .delete(delete_setlist_handler)
                .patch(rename_setlist_handler),
        )
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::setlist::test_utils::MockClaude;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn valid_llm_json() -> String {
        r#"{
            "tracks": [
                {
                    "position": 1,
                    "title": "Test Track",
                    "artist": "Test Artist",
                    "bpm": 128.0,
                    "key": "C minor",
                    "camelot": "5A",
                    "energy": 5,
                    "transition_note": "Blend in",
                    "source": "suggestion",
                    "track_id": null
                }
            ],
            "notes": "Test set"
        }"#
        .to_string()
    }

    async fn setup_app(claude_response: &str) -> (Router, sqlx::PgPool) {
        let pool = crate::db::create_test_pool().await;

        // Seed a track so catalog is not empty
        sqlx::query("INSERT INTO tracks (id, title, source, bpm, camelot_key, energy) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind("t1")
            .bind("Seed Track")
            .bind("spotify")
            .bind(128.0)
            .bind("8A")
            .bind(5.0)
            .execute(&pool)
            .await
            .unwrap();

        let state = Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: claude_response.to_string(),
            }),
        });

        (setlist_router(state), pool)
    }

    async fn post_json(
        app: Router,
        uri: &str,
        body: serde_json::Value,
    ) -> (u16, serde_json::Value) {
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status().as_u16();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        (status, json)
    }

    async fn post_empty(app: Router, uri: &str) -> (u16, serde_json::Value) {
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status().as_u16();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        (status, json)
    }

    async fn get_json(app: Router, uri: &str) -> (u16, serde_json::Value) {
        let response = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status().as_u16();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn test_generate_returns_201() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) = post_json(
            app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": "chill house vibes"
            }),
        )
        .await;

        assert_eq!(status, 201);
        assert!(json["id"].is_string());
        assert_eq!(json["prompt"], "chill house vibes");
        assert!(json["tracks"].is_array());
    }

    #[tokio::test]
    async fn test_generate_empty_prompt_returns_400() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) = post_json(
            app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": ""
            }),
        )
        .await;

        assert_eq!(status, 400);
        assert_eq!(json["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn test_generate_no_catalog_returns_suggestions() {
        let pool = crate::db::create_test_pool().await;
        let state = Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        });
        let app = setlist_router(state);

        let (status, json) = post_json(
            app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": "test"
            }),
        )
        .await;

        assert_eq!(status, 201);
        // All tracks should be suggestions since catalog is empty
        let tracks = json["tracks"].as_array().unwrap();
        assert!(!tracks.is_empty());
        for track in tracks {
            assert_eq!(track["source"], "suggestion");
        }
        pool.close().await;
    }

    #[tokio::test]
    async fn test_arrange_returns_200() {
        let (_app, pool) = setup_app(&valid_llm_json()).await;

        // First generate a setlist
        let (status, gen_json) = post_json(
            setlist_router(Arc::new(SetlistRouteState {
                pool: pool.clone(),
                claude: Arc::new(MockClaude {
                    response: valid_llm_json(),
                }),
            })),
            "/setlists/generate",
            serde_json::json!({ "prompt": "test" }),
        )
        .await;
        assert_eq!(status, 201);
        let setlist_id = gen_json["id"].as_str().unwrap();

        // Now arrange it
        let arrange_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (status, json) = post_json(
            arrange_app,
            &format!("/setlists/{setlist_id}/arrange"),
            serde_json::json!({}),
        )
        .await;

        assert_eq!(status, 200);
        assert!(json["harmonic_flow_score"].is_number());
        // C1: score_breakdown should be present after arrange
        assert!(json["score_breakdown"].is_object());
        assert!(json["score_breakdown"]["key_compatibility"].is_number());
        assert!(json["score_breakdown"]["bpm_continuity"].is_number());
        assert!(json["score_breakdown"]["energy_arc"].is_number());
    }

    #[tokio::test]
    async fn test_arrange_not_found_returns_404() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) =
            post_json(app, "/setlists/nonexistent/arrange", serde_json::json!({})).await;

        assert_eq!(status, 404);
        assert_eq!(json["error"]["code"], "NOT_FOUND");
    }

    // M5: Test arrange with 0 tracks returns 400 INVALID_REQUEST
    #[tokio::test]
    async fn test_arrange_empty_setlist_returns_400() {
        let pool = crate::db::create_test_pool().await;

        // Insert a setlist with no tracks
        sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES ($1, $2, $3, $4)")
            .bind("empty-setlist")
            .bind("user1")
            .bind("test")
            .bind("test-model")
            .execute(&pool)
            .await
            .unwrap();

        let app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));

        let (status, json) = post_json(
            app,
            "/setlists/empty-setlist/arrange",
            serde_json::json!({}),
        )
        .await;

        assert_eq!(status, 400);
        assert_eq!(json["error"]["code"], "INVALID_REQUEST");
        pool.close().await;
    }

    // M5: Test arrange with 1 track returns setlist unchanged
    #[tokio::test]
    async fn test_arrange_single_track_returns_unchanged() {
        let (_app, pool) = setup_app(&valid_llm_json()).await;

        // Generate a setlist (it has 1 track)
        let gen_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (status, gen_json) = post_json(
            gen_app,
            "/setlists/generate",
            serde_json::json!({ "prompt": "test" }),
        )
        .await;
        assert_eq!(status, 201);
        let setlist_id = gen_json["id"].as_str().unwrap();

        // Arrange the single-track setlist
        let arrange_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (status, json) = post_json(
            arrange_app,
            &format!("/setlists/{setlist_id}/arrange"),
            serde_json::json!({}),
        )
        .await;

        assert_eq!(status, 200);
        assert_eq!(json["harmonic_flow_score"], 100.0);
        assert_eq!(json["tracks"].as_array().unwrap().len(), 1);
        // C1: score_breakdown present for single track
        assert!(json["score_breakdown"].is_object());
        assert_eq!(json["score_breakdown"]["key_compatibility"], 100.0);
    }

    #[tokio::test]
    async fn test_get_setlist_returns_200() {
        let (_, pool) = setup_app(&valid_llm_json()).await;

        // Generate first
        let gen_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (_, gen_json) = post_json(
            gen_app,
            "/setlists/generate",
            serde_json::json!({ "prompt": "test" }),
        )
        .await;
        let setlist_id = gen_json["id"].as_str().unwrap();

        // Now get it
        let get_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (status, json) = get_json(get_app, &format!("/setlists/{setlist_id}")).await;

        assert_eq!(status, 200);
        assert_eq!(json["id"], setlist_id);
        assert!(json["tracks"].is_array());
    }

    #[tokio::test]
    async fn test_get_setlist_not_found_returns_404() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) = get_json(app, "/setlists/nonexistent").await;

        assert_eq!(status, 404);
        assert_eq!(json["error"]["code"], "NOT_FOUND");
    }

    // -----------------------------------------------------------------------
    // T8: Extended route tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_generate_with_all_new_params() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) = post_json(
            app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": "peak time techno",
                "track_count": 5,
                "energy_profile": "peak-time",
                "creative_mode": true,
                "seed_tracklist": "1. Track A\n2. Track B",
                "bpm_range": { "min": 130.0, "max": 145.0 }
            }),
        )
        .await;

        assert_eq!(status, 201);
        assert!(json["id"].is_string());
        assert_eq!(json["energy_profile"], "peak-time");
        assert!(json["catalog_percentage"].is_number());
    }

    #[tokio::test]
    async fn test_generate_with_invalid_energy_profile() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) = post_json(
            app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": "test",
                "energy_profile": "invalid-profile"
            }),
        )
        .await;

        assert_eq!(status, 400);
        assert_eq!(json["error"]["code"], "INVALID_ENERGY_PROFILE");
    }

    #[tokio::test]
    async fn test_generate_with_invalid_bpm_range() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) = post_json(
            app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": "test",
                "bpm_range": { "min": 50.0, "max": 130.0 }
            }),
        )
        .await;

        assert_eq!(status, 400);
        assert_eq!(json["error"]["code"], "INVALID_BPM_RANGE");
    }

    #[tokio::test]
    async fn test_generate_with_nonexistent_playlist() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) = post_json(
            app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": "test",
                "source_playlist_id": "nonexistent-id"
            }),
        )
        .await;

        assert_eq!(status, 404);
        assert_eq!(json["error"]["code"], "PLAYLIST_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_arrange_with_energy_profile() {
        let (_app, pool) = setup_app(&valid_llm_json()).await;

        // Generate first
        let (status, gen_json) = post_json(
            setlist_router(Arc::new(SetlistRouteState {
                pool: pool.clone(),
                claude: Arc::new(MockClaude {
                    response: valid_llm_json(),
                }),
            })),
            "/setlists/generate",
            serde_json::json!({ "prompt": "test" }),
        )
        .await;
        assert_eq!(status, 201);
        let setlist_id = gen_json["id"].as_str().unwrap();

        // Arrange with energy_profile in body
        let arrange_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (status, json) = post_json(
            arrange_app,
            &format!("/setlists/{setlist_id}/arrange"),
            serde_json::json!({ "energy_profile": "warm-up" }),
        )
        .await;

        assert_eq!(status, 200);
        assert!(json["harmonic_flow_score"].is_number());
    }

    #[tokio::test]
    async fn test_arrange_with_empty_body_backward_compat() {
        let (_app, pool) = setup_app(&valid_llm_json()).await;

        // Generate first
        let (status, gen_json) = post_json(
            setlist_router(Arc::new(SetlistRouteState {
                pool: pool.clone(),
                claude: Arc::new(MockClaude {
                    response: valid_llm_json(),
                }),
            })),
            "/setlists/generate",
            serde_json::json!({ "prompt": "test" }),
        )
        .await;
        assert_eq!(status, 201);
        let setlist_id = gen_json["id"].as_str().unwrap();

        // Arrange with NO body at all (backward compat)
        let arrange_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (status, json) =
            post_empty(arrange_app, &format!("/setlists/{setlist_id}/arrange")).await;

        assert_eq!(status, 200);
        assert!(json["harmonic_flow_score"].is_number());
    }

    #[tokio::test]
    async fn test_get_setlist_includes_new_fields() {
        let (_, pool) = setup_app(&valid_llm_json()).await;

        // Generate with energy profile
        let gen_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (_, gen_json) = post_json(
            gen_app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": "test",
                "energy_profile": "journey"
            }),
        )
        .await;
        let setlist_id = gen_json["id"].as_str().unwrap();

        // Get it back
        let get_app = setlist_router(Arc::new(SetlistRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: valid_llm_json(),
            }),
        }));
        let (status, json) = get_json(get_app, &format!("/setlists/{setlist_id}")).await;

        assert_eq!(status, 200);
        assert_eq!(json["energy_profile"], "journey");
    }

    #[tokio::test]
    async fn test_generate_without_new_params_backward_compat() {
        let (app, _) = setup_app(&valid_llm_json()).await;
        let (status, json) = post_json(
            app,
            "/setlists/generate",
            serde_json::json!({
                "prompt": "just a basic prompt"
            }),
        )
        .await;

        assert_eq!(status, 201);
        assert!(json["id"].is_string());
        assert_eq!(json["prompt"], "just a basic prompt");
        // energy_profile should not be present (None serialized as skip)
        assert!(json.get("energy_profile").is_none() || json["energy_profile"].is_null());
    }
}
