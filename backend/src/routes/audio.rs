use axum::{extract::Query, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DeezerSearchParams {
    q: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    1
}

#[derive(Serialize, Deserialize)]
struct DeezerTrack {
    id: u64,
    title: String,
    preview: String,
    duration: u64,
    artist: DeezerArtist,
}

#[derive(Serialize, Deserialize)]
struct DeezerArtist {
    name: String,
}

#[derive(Deserialize)]
struct DeezerResponse {
    data: Vec<DeezerTrack>,
}

async fn deezer_search(Query(params): Query<DeezerSearchParams>) -> impl IntoResponse {
    let client = reqwest::Client::new();
    match client
        .get("https://api.deezer.com/search")
        .query(&[("q", &params.q), ("limit", &params.limit.to_string())])
        .send()
        .await
    {
        Ok(resp) => match resp.json::<DeezerResponse>().await {
            Ok(data) => Json(serde_json::json!({ "data": data.data })).into_response(),
            Err(e) => (
                axum::http::StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": { "code": "DEEZER_PARSE_ERROR", "message": e.to_string() } })),
            )
                .into_response(),
        },
        Err(e) => (
            axum::http::StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": { "code": "DEEZER_FETCH_ERROR", "message": e.to_string() } })),
        )
            .into_response(),
    }
}

pub fn audio_router() -> Router {
    Router::new().route("/audio/deezer-search", get(deezer_search))
}
