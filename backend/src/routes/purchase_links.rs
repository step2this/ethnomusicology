use std::sync::Arc;

use axum::{extract::Query, extract::State, routing::get, Json, Router};
use serde::Deserialize;

use crate::services::purchase_links::{
    build_purchase_links, AffiliateConfig, PurchaseLinkResponse,
};

#[derive(Deserialize)]
pub struct PurchaseLinkQuery {
    pub title: Option<String>,
    pub artist: Option<String>,
}

pub struct PurchaseLinkRouteState {
    pub affiliate_config: AffiliateConfig,
}

async fn get_purchase_links(
    State(state): State<Arc<PurchaseLinkRouteState>>,
    Query(params): Query<PurchaseLinkQuery>,
) -> Json<PurchaseLinkResponse> {
    let title = params.title.as_deref().unwrap_or("");
    let artist = params.artist.as_deref().unwrap_or("");
    Json(build_purchase_links(title, artist, &state.affiliate_config))
}

pub fn purchase_link_router(state: Arc<PurchaseLinkRouteState>) -> Router {
    Router::new()
        .route("/purchase-links", get(get_purchase_links))
        .with_state(state)
}
