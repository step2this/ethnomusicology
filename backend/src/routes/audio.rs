use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;

// ---------------------------------------------------------------------------
// Static resources
// ---------------------------------------------------------------------------

static ITUNES_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

fn itunes_semaphore() -> &'static Semaphore {
    ITUNES_SEMAPHORE.get_or_init(|| Semaphore::new(20))
}

// ---------------------------------------------------------------------------
// Existing param/response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DeezerSearchParams {
    q: String,
    #[serde(default = "default_limit")]
    limit: u32,
    strict: Option<String>,
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

// ---------------------------------------------------------------------------
// New types for unified audio search
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AudioSearchParams {
    title: String,
    artist: String,
}

#[derive(Serialize)]
pub struct AudioSearchResponse {
    source: Option<String>,
    preview_url: Option<String>,
    external_url: Option<String>,
    search_queries: Vec<String>,
    deezer_id: Option<u64>,
    itunes_id: Option<u64>,
}

#[derive(Deserialize)]
struct ItunesResponse {
    results: Vec<ItunesTrack>,
}

#[derive(Deserialize)]
struct ItunesTrack {
    #[serde(rename = "trackId")]
    track_id: u64,
    #[serde(rename = "trackName")]
    track_name: String,
    #[serde(rename = "artistName")]
    artist_name: String,
    #[serde(rename = "previewUrl")]
    preview_url: Option<String>,
    #[serde(rename = "trackViewUrl")]
    track_view_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Returns true if the host is in the allowlist for the audio proxy.
fn is_allowed_host(host: &str) -> bool {
    // Deezer CDN
    if host == "dzcdn.net" || host.ends_with(".dzcdn.net") {
        return true;
    }
    // Apple CDN
    if host == "audio-ssl.itunes.apple.com" {
        return true;
    }
    if host == "mzstatic.com" || host.ends_with(".mzstatic.com") {
        return true;
    }
    false
}

/// Builds a proxied URL for the local `/api/audio/proxy` endpoint.
fn build_proxy_url(raw_url: &str) -> String {
    let encoded: String = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("url", raw_url)
        .finish();
    format!("/api/audio/proxy?{}", encoded)
}

// ---------------------------------------------------------------------------
// Deezer field-specific search helper
// ---------------------------------------------------------------------------

/// Searches Deezer using field-specific syntax: `artist:"X" track:"Y"`.
/// Returns `(deezer_id, preview_url)` on success, `None` otherwise.
async fn deezer_field_search(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
    strict: bool,
) -> Option<(u64, String)> {
    let query = format!(r#"artist:"{}" track:"{}""#, artist, title);
    let mut params: Vec<(&str, String)> = vec![("q", query), ("limit", "5".to_string())];
    if strict {
        params.push(("strict", "on".to_string()));
    }

    let resp = timeout(
        Duration::from_secs(2),
        client
            .get("https://api.deezer.com/search")
            .query(&params)
            .send(),
    )
    .await
    .ok()?
    .ok()?;

    let data = resp.json::<DeezerResponse>().await.ok()?;

    data.data
        .into_iter()
        .find(|t| {
            !t.preview.is_empty()
                && crate::services::match_scoring::is_acceptable_match(
                    title,
                    artist,
                    &t.title,
                    &t.artist.name,
                )
        })
        .map(|t| (t.id, t.preview))
}

// ---------------------------------------------------------------------------
// iTunes search helper
// ---------------------------------------------------------------------------

/// Searches iTunes for a preview URL, returning `(itunes_id, preview_url, track_view_url)`.
async fn itunes_search(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
) -> Option<(u64, String, String)> {
    let _permit = itunes_semaphore().acquire().await.ok()?;
    let term = format!("{} {}", artist, title);

    let resp = timeout(
        Duration::from_secs(2),
        client
            .get("https://itunes.apple.com/search")
            .query(&[
                ("term", term.as_str()),
                ("media", "music"),
                ("entity", "song"),
                ("limit", "5"),
                ("country", "US"),
            ])
            .send(),
    )
    .await
    .ok()?
    .ok()?;

    let data = resp.json::<ItunesResponse>().await.ok()?;

    data.results
        .into_iter()
        .filter(|t| t.preview_url.is_some())
        .find(|t| {
            crate::services::match_scoring::is_acceptable_match(
                title,
                artist,
                &t.track_name,
                &t.artist_name,
            )
        })
        .map(|t| {
            let preview = t.preview_url.unwrap();
            let view_url = t.track_view_url.unwrap_or_default();
            (t.track_id, preview, view_url)
        })
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Unified audio search: tries Deezer (strict → fuzzy) then iTunes.
async fn audio_search(Query(params): Query<AudioSearchParams>) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let title = &params.title;
    let artist = &params.artist;

    let deezer_query = format!(r#"artist:"{}" track:"{}""#, artist, title);
    let mut search_queries = vec![deezer_query];

    // 1. Try Deezer strict
    if let Some((deezer_id, preview_url)) = deezer_field_search(&client, artist, title, true).await
    {
        return Json(AudioSearchResponse {
            source: Some("deezer".to_string()),
            preview_url: Some(build_proxy_url(&preview_url)),
            external_url: Some(format!("https://www.deezer.com/track/{deezer_id}")),
            search_queries,
            deezer_id: Some(deezer_id),
            itunes_id: None,
        })
        .into_response();
    }

    // 2. Try Deezer fuzzy
    if let Some((deezer_id, preview_url)) = deezer_field_search(&client, artist, title, false).await
    {
        return Json(AudioSearchResponse {
            source: Some("deezer".to_string()),
            preview_url: Some(build_proxy_url(&preview_url)),
            external_url: Some(format!("https://www.deezer.com/track/{deezer_id}")),
            search_queries,
            deezer_id: Some(deezer_id),
            itunes_id: None,
        })
        .into_response();
    }

    // 3. Try iTunes
    let itunes_query = format!("{} {}", artist, title);
    search_queries.push(itunes_query);

    if let Some((itunes_id, preview_url, view_url)) = itunes_search(&client, artist, title).await {
        return Json(AudioSearchResponse {
            source: Some("itunes".to_string()),
            preview_url: Some(build_proxy_url(&preview_url)),
            external_url: Some(view_url),
            search_queries,
            deezer_id: None,
            itunes_id: Some(itunes_id),
        })
        .into_response();
    }

    // 4. No match found
    Json(AudioSearchResponse {
        source: None,
        preview_url: None,
        external_url: None,
        search_queries,
        deezer_id: None,
        itunes_id: None,
    })
    .into_response()
}

async fn deezer_search(Query(params): Query<DeezerSearchParams>) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let mut query_params = vec![
        ("q".to_string(), params.q),
        ("limit".to_string(), params.limit.to_string()),
    ];
    if let Some(ref strict) = params.strict {
        query_params.push(("strict".to_string(), strict.clone()));
    }
    match client
        .get("https://api.deezer.com/search")
        .query(&query_params)
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

    // Validate that the host is in the allowlist
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

    if !is_allowed_host(host) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {
                    "code": "INVALID_HOST",
                    "message": "Host not in proxy allowlist"
                }
            })),
        )
            .into_response();
    }

    // Fetch the audio data from the upstream CDN
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

            // Capture Content-Type before consuming the response body
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .cloned()
                .unwrap_or_else(|| HeaderValue::from_static("audio/mpeg"));

            // Stream the response body with forwarded Content-Type
            match resp.bytes().await {
                Ok(body_bytes) => {
                    let body = Body::from(body_bytes);
                    let mut response = (StatusCode::OK, body).into_response();
                    response
                        .headers_mut()
                        .insert(header::CONTENT_TYPE, content_type);
                    response
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

#[derive(Debug, Serialize)]
struct EnrichDeezerResponse {
    enriched: usize,
}

async fn enrich_deezer_handler(
    State(pool): State<SqlitePool>,
) -> Result<Json<EnrichDeezerResponse>, DeezerEnrichError> {
    let enriched = crate::services::deezer::enrich_tracks_with_deezer(&pool).await?;

    Ok(Json(EnrichDeezerResponse { enriched }))
}

#[derive(Debug)]
struct DeezerEnrichError(anyhow::Error);

impl From<anyhow::Error> for DeezerEnrichError {
    fn from(e: anyhow::Error) -> Self {
        DeezerEnrichError(e)
    }
}

impl axum::response::IntoResponse for DeezerEnrichError {
    fn into_response(self) -> axum::response::Response {
        let msg = self.0.to_string();

        let body = serde_json::json!({
            "error": {
                "code": "DEEZER_ENRICH_ERROR",
                "message": msg,
            }
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

pub fn audio_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/audio/search", get(audio_search))
        .route("/audio/deezer-search", get(deezer_search))
        .route("/audio/proxy", get(audio_proxy))
        .route("/audio/enrich-deezer", post(enrich_deezer_handler))
        .with_state(pool)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_allowed_host_deezer() {
        assert!(is_allowed_host("dzcdn.net"));
        assert!(is_allowed_host("cdns-preview-a.dzcdn.net"));
        assert!(is_allowed_host("e-cdns-proxy-76.dzcdn.net"));
    }

    #[test]
    fn test_is_allowed_host_apple() {
        assert!(is_allowed_host("audio-ssl.itunes.apple.com"));
        assert!(is_allowed_host("mzstatic.com"));
        assert!(is_allowed_host("a1.mzstatic.com"));
        assert!(is_allowed_host("audio.mzstatic.com"));
    }

    #[test]
    fn test_is_allowed_host_blocked() {
        assert!(!is_allowed_host("evil.com"));
        assert!(!is_allowed_host("itunes.apple.com"));
        assert!(!is_allowed_host("spotify.com"));
        assert!(!is_allowed_host("soundcloud.com"));
    }

    #[test]
    fn test_build_proxy_url_encodes_url() {
        let raw = "https://audio-ssl.itunes.apple.com/foo.m4a?bar=baz";
        let proxied = build_proxy_url(raw);
        assert!(proxied.starts_with("/api/audio/proxy?url="));
        assert!(proxied.contains("%3A%2F%2F")); // encoded ://
    }

    #[test]
    fn test_audio_search_response_serializes() {
        let resp = AudioSearchResponse {
            source: Some("itunes".to_string()),
            preview_url: Some("/api/audio/proxy?url=https%3A%2F%2Ffoo.com%2Fbar.m4a".to_string()),
            external_url: Some("https://music.apple.com/track/123".to_string()),
            search_queries: vec!["Avicii Levels".to_string()],
            deezer_id: None,
            itunes_id: Some(123456789),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["source"], "itunes");
        assert_eq!(json["itunes_id"], 123456789);
        assert!(json["deezer_id"].is_null());
    }

    #[test]
    fn test_audio_search_response_no_match() {
        let resp = AudioSearchResponse {
            source: None,
            preview_url: None,
            external_url: None,
            search_queries: vec![
                r#"artist:"Avicii" track:"Levels""#.to_string(),
                "Avicii Levels".to_string(),
            ],
            deezer_id: None,
            itunes_id: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json["source"].is_null());
        assert_eq!(json["search_queries"].as_array().unwrap().len(), 2);
    }
}
