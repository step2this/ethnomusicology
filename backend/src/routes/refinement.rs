// ST-007: Refinement route handlers

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

use crate::api::claude::ClaudeClientTrait;
use crate::services::refinement::{self, HistoryResponse, RefinementError, RefinementResponse};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct RefinementRouteState {
    pub pool: PgPool,
    pub claude: Arc<dyn ClaudeClientTrait>,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct RefineRequest {
    pub message: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn refine_handler(
    State(state): State<Arc<RefinementRouteState>>,
    Path(setlist_id): Path<String>,
    Json(body): Json<RefineRequest>,
) -> Result<Json<RefinementResponse>, RefinementError> {
    let response = refinement::refine_setlist(
        &state.pool,
        state.claude.as_ref(),
        &setlist_id,
        "anonymous",
        &body.message,
    )
    .await?;
    Ok(Json(response))
}

async fn revert_handler(
    State(state): State<Arc<RefinementRouteState>>,
    Path((setlist_id, version_number)): Path<(String, i32)>,
) -> Result<Json<RefinementResponse>, RefinementError> {
    let response = refinement::revert_setlist(&state.pool, &setlist_id, version_number).await?;
    Ok(Json(response))
}

async fn history_handler(
    State(state): State<Arc<RefinementRouteState>>,
    Path(setlist_id): Path<String>,
) -> Result<Json<HistoryResponse>, RefinementError> {
    let response = refinement::get_history(&state.pool, &setlist_id).await?;
    Ok(Json(response))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn refinement_router(state: Arc<RefinementRouteState>) -> Router {
    Router::new()
        .route("/setlists/{id}/refine", post(refine_handler))
        .route(
            "/setlists/{id}/revert/{version_number}",
            post(revert_handler),
        )
        .route("/setlists/{id}/history", get(history_handler))
        .with_state(state)
}
