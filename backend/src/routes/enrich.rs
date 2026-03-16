use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::api::claude::ClaudeClientTrait;
use crate::services::enrichment::{self, EnrichmentError};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct EnrichRouteState {
    pub pool: PgPool,
    pub claude: Arc<dyn ClaudeClientTrait>,
    /// Guards against concurrent enrichment calls. Set to true while a run is in progress.
    pub in_flight: AtomicBool,
}

// ---------------------------------------------------------------------------
// Response type
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct EnrichResponse {
    enriched: u32,
    errors: u32,
    skipped: u32,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

async fn enrich_handler(
    State(state): State<Arc<EnrichRouteState>>,
    headers: HeaderMap,
) -> Result<Json<EnrichResponse>, EnrichApiError> {
    // Reject concurrent enrichment runs with 409 Conflict
    if state
        .in_flight
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(EnrichApiError(EnrichmentError::Concurrent(
            "Enrichment is already running. Try again after the current run completes.".to_string(),
        )));
    }

    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("dev-user");

    let result = enrichment::enrich_tracks(&state.pool, state.claude.as_ref(), user_id).await;

    // Always release the lock, even on error
    state.in_flight.store(false, Ordering::Release);

    let result = result?;

    Ok(Json(EnrichResponse {
        enriched: result.enriched,
        errors: result.errors,
        skipped: result.skipped,
    }))
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct EnrichApiError(EnrichmentError);

impl From<EnrichmentError> for EnrichApiError {
    fn from(e: EnrichmentError) -> Self {
        EnrichApiError(e)
    }
}

impl axum::response::IntoResponse for EnrichApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match &self.0 {
            EnrichmentError::CostCapExceeded(msg) => {
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", msg.clone())
            }
            EnrichmentError::Concurrent(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            EnrichmentError::Claude(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
            EnrichmentError::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
            ),
        };

        let body = serde_json::json!({
            "error": {
                "code": code,
                "message": message,
            }
        });
        (status, Json(body)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn enrich_router(state: Arc<EnrichRouteState>) -> Router {
    Router::new()
        .route("/tracks/enrich", post(enrich_handler))
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

    async fn setup_app(claude_response: &str) -> (Router, PgPool) {
        let pool = crate::db::create_test_pool().await;
        let state = Arc::new(EnrichRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: claude_response.to_string(),
            }),
            in_flight: AtomicBool::new(false),
        });
        (enrich_router(state), pool)
    }

    async fn seed_unenriched_tracks(pool: &PgPool, count: usize) {
        for i in 0..count {
            sqlx::query(
                "INSERT INTO tracks (id, title, source, needs_enrichment) VALUES ($1, $2, 'spotify', TRUE)",
            )
            .bind(format!("t{i}"))
            .bind(format!("Track {i}"))
            .execute(pool)
            .await
            .unwrap();
            sqlx::query("INSERT INTO artists (id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(format!("a{i}"))
                .bind(format!("Artist {i}"))
                .execute(pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ($1, $2)")
                .bind(format!("t{i}"))
                .bind(format!("a{i}"))
                .execute(pool)
                .await
                .unwrap();
        }
    }

    async fn post_enrich(app: Router) -> (u16, serde_json::Value) {
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tracks/enrich")
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

    #[tokio::test]
    async fn test_enrich_success() {
        let (_, pool) = setup_app(&mock_enrichment_response(3)).await;
        seed_unenriched_tracks(&pool, 3).await;

        let app = enrich_router(Arc::new(EnrichRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: mock_enrichment_response(3),
            }),
            in_flight: AtomicBool::new(false),
        }));

        let (status, json) = post_enrich(app).await;

        assert_eq!(status, 200);
        assert_eq!(json["enriched"], 3);
        assert_eq!(json["errors"], 0);
        assert_eq!(json["skipped"], 0);
    }

    #[tokio::test]
    async fn test_enrich_empty_catalog() {
        let (app, _) = setup_app("{}").await;

        let (status, json) = post_enrich(app).await;

        assert_eq!(status, 200);
        assert_eq!(json["enriched"], 0);
        assert_eq!(json["errors"], 0);
        assert_eq!(json["skipped"], 0);
    }

    #[tokio::test]
    async fn test_enrich_cost_cap_exceeded() {
        let (_, pool) = setup_app(&mock_enrichment_response(1)).await;
        seed_unenriched_tracks(&pool, 1).await;

        // Pre-fill usage to the cap (250)
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        sqlx::query(
            "INSERT INTO user_usage (id, user_id, date, enrichment_count) VALUES ('u1', 'dev-user', $1, 250)",
        )
        .bind(&today)
        .execute(&pool)
        .await
        .unwrap();

        let app = enrich_router(Arc::new(EnrichRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: mock_enrichment_response(1),
            }),
            in_flight: AtomicBool::new(false),
        }));

        let (status, json) = post_enrich(app).await;

        assert_eq!(status, 429);
        assert_eq!(json["error"]["code"], "RATE_LIMITED");
    }

    #[tokio::test]
    async fn test_enrich_concurrent_returns_409() {
        let pool = crate::db::create_test_pool().await;
        // Pre-set in_flight to true to simulate a concurrent run
        let state = Arc::new(EnrichRouteState {
            pool: pool.clone(),
            claude: Arc::new(MockClaude {
                response: mock_enrichment_response(1),
            }),
            in_flight: AtomicBool::new(true),
        });

        let app = enrich_router(state);
        let (status, json) = post_enrich(app).await;

        assert_eq!(status, 409);
        assert_eq!(json["error"]["code"], "CONFLICT");
        pool.close().await;
    }
}
