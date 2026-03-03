use axum::{
    body::Body,
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DeezerSearchParams {
    q: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

#[derive(Deserialize)]
pub struct AudioProxyParams {
    url: String,
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
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": { "code": "DEEZER_PARSE_ERROR", "message": e.to_string() } })),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": { "code": "DEEZER_FETCH_ERROR", "message": e.to_string() } })),
        )
            .into_response(),
    }
}

async fn audio_proxy(Query(params): Query<AudioProxyParams>) -> impl IntoResponse {
    // Parse the URL to validate and extract the host
    let parsed_url = match url::Url::parse(&params.url) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": {
                        "code": "INVALID_URL",
                        "message": "Invalid URL format"
                    }
                })),
            )
                .into_response();
        }
    };

    // Validate that the host is dzcdn.net (allow subdomains like *.dzcdn.net)
    let host = match parsed_url.host_str() {
        Some(h) => h,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": {
                        "code": "INVALID_URL",
                        "message": "Invalid URL host"
                    }
                })),
            )
                .into_response();
        }
    };

    if !host.ends_with("dzcdn.net") {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {
                    "code": "INVALID_HOST",
                    "message": "Only dzcdn.net hosts are allowed"
                }
            })),
        )
            .into_response();
    }

    // Fetch the audio data from the Deezer CDN
    let client = reqwest::Client::new();
    match client.get(&params.url).send().await {
        Ok(resp) => {
            // Check if the response status is success
            if !resp.status().is_success() {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({
                        "error": {
                            "code": "CDN_FETCH_ERROR",
                            "message": format!("CDN returned status {}", resp.status())
                        }
                    })),
                )
                    .into_response();
            }

            // Stream the response body with appropriate headers
            match resp.bytes().await {
                Ok(body_bytes) => {
                    let body = Body::from(body_bytes);
                    (StatusCode::OK, [(header::CONTENT_TYPE, "audio/mpeg")], body).into_response()
                }
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({
                        "error": {
                            "code": "CDN_READ_ERROR",
                            "message": e.to_string()
                        }
                    })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": {
                    "code": "CDN_FETCH_ERROR",
                    "message": e.to_string()
                }
            })),
        )
            .into_response(),
    }
}

pub fn audio_router() -> Router {
    Router::new()
        .route("/audio/deezer-search", get(deezer_search))
        .route("/audio/proxy", get(audio_proxy))
}
