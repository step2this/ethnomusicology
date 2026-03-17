use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::timeout;

use crate::services::soundcloud::SoundCloudClient;

// ---------------------------------------------------------------------------
// Static resources
// ---------------------------------------------------------------------------

/// Cached Spotify Client Credentials token: (access_token, expiry).
type SpotifyCcCache = RwLock<Option<(String, DateTime<Utc>)>>;

static ITUNES_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();
static SOUNDCLOUD_CLIENT: OnceLock<Mutex<Option<SoundCloudClient>>> = OnceLock::new();
static SPOTIFY_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();
static SPOTIFY_CC_TOKEN: OnceLock<SpotifyCcCache> = OnceLock::new();

fn soundcloud_client() -> &'static Mutex<Option<SoundCloudClient>> {
    SOUNDCLOUD_CLIENT.get_or_init(|| Mutex::new(SoundCloudClient::new_from_env()))
}

fn itunes_semaphore() -> &'static Semaphore {
    ITUNES_SEMAPHORE.get_or_init(|| Semaphore::new(20))
}

fn spotify_semaphore() -> &'static Semaphore {
    SPOTIFY_SEMAPHORE.get_or_init(|| Semaphore::new(5))
}

fn spotify_cc_token() -> &'static SpotifyCcCache {
    SPOTIFY_CC_TOKEN.get_or_init(|| RwLock::new(None))
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
    soundcloud_id: Option<String>,
    uploader_name: Option<String>,
    spotify_uri: Option<String>,
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
// Spotify Client Credentials types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SpotifyCCTokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct SpotifySearchResponse {
    tracks: SpotifySearchTracks,
}

#[derive(Deserialize)]
struct SpotifySearchTracks {
    items: Vec<SpotifySearchTrack>,
}

#[derive(Deserialize)]
struct SpotifySearchTrack {
    uri: String,
    name: String,
    artists: Vec<SpotifySearchArtist>,
}

#[derive(Deserialize)]
struct SpotifySearchArtist {
    name: String,
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
    // SoundCloud CDN (stream_url is resolved to CDN URL server-side in soundcloud.rs)
    if host == "cf-preview-media.sndcdn.com" || host.ends_with(".sndcdn.com") {
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
// Spotify Client Credentials token cache + search
// ---------------------------------------------------------------------------

/// Fetches a Spotify Client Credentials access token, using a cached value
/// when still valid (with 60 s buffer before expiry).
async fn spotify_cc_token_get(http: &reqwest::Client) -> Option<String> {
    let client_id = std::env::var("SPOTIFY_CLIENT_ID").ok()?;
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET").ok()?;
    if client_id.is_empty() || client_secret.is_empty() {
        return None;
    }

    // Fast path: read lock to check cached token.
    {
        let guard = spotify_cc_token().read().await;
        if let Some((ref token, expiry)) = *guard {
            if Utc::now() + chrono::Duration::seconds(60) < expiry {
                return Some(token.clone());
            }
        }
    }

    // Slow path: write lock, double-check, then fetch.
    let mut guard = spotify_cc_token().write().await;
    if let Some((ref token, expiry)) = *guard {
        if Utc::now() + chrono::Duration::seconds(60) < expiry {
            return Some(token.clone());
        }
    }

    let resp = timeout(
        Duration::from_secs(5),
        http.post("https://accounts.spotify.com/api/token")
            .basic_auth(&client_id, Some(&client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send(),
    )
    .await
    .ok()?
    .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let tr: SpotifyCCTokenResponse = resp.json().await.ok()?;
    let expiry = Utc::now() + chrono::Duration::seconds(tr.expires_in as i64);
    *guard = Some((tr.access_token.clone(), expiry));
    Some(tr.access_token)
}

/// Searches Spotify for a track URI using Client Credentials.
/// Returns `spotify:track:<id>` on a match, `None` otherwise.
async fn spotify_search(artist: &str, title: &str) -> Option<String> {
    let _permit = spotify_semaphore().acquire().await.ok()?;
    let client = reqwest::Client::new();

    let token = spotify_cc_token_get(&client).await?;
    let q = format!(r#"track:"{}" artist:"{}""#, title, artist);

    let resp = timeout(
        Duration::from_secs(3),
        client
            .get("https://api.spotify.com/v1/search")
            .query(&[("q", q.as_str()), ("type", "track"), ("limit", "5")])
            .bearer_auth(&token)
            .send(),
    )
    .await
    .ok()?
    .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let data: SpotifySearchResponse = resp.json().await.ok()?;

    data.tracks
        .items
        .into_iter()
        .find(|t| {
            let result_artist = t.artists.first().map(|a| a.name.as_str()).unwrap_or("");
            crate::services::match_scoring::is_acceptable_match(
                title,
                artist,
                &t.name,
                result_artist,
            )
        })
        .map(|t| t.uri)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Unified audio search: tries Deezer (strict → fuzzy) then iTunes, then SoundCloud.
/// Spotify URI lookup runs in parallel with the first Deezer search.
async fn audio_search(Query(params): Query<AudioSearchParams>) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let title = &params.title;
    let artist = &params.artist;

    let deezer_query = format!(r#"artist:"{}" track:"{}""#, artist, title);
    let mut search_queries = vec![deezer_query];

    // Run Deezer strict and Spotify URI search in parallel.
    let (deezer_strict, spotify_uri) = tokio::join!(
        deezer_field_search(&client, artist, title, true),
        spotify_search(artist, title)
    );

    // 1. Try Deezer strict
    if let Some((deezer_id, preview_url)) = deezer_strict {
        return Json(AudioSearchResponse {
            source: Some("deezer".to_string()),
            preview_url: Some(build_proxy_url(&preview_url)),
            external_url: Some(format!("https://www.deezer.com/track/{deezer_id}")),
            search_queries,
            deezer_id: Some(deezer_id),
            itunes_id: None,
            soundcloud_id: None,
            uploader_name: None,
            spotify_uri,
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
            soundcloud_id: None,
            uploader_name: None,
            spotify_uri,
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
            soundcloud_id: None,
            uploader_name: None,
            spotify_uri,
        })
        .into_response();
    }

    // 4. Try SoundCloud
    {
        let mut sc_guard = soundcloud_client().lock().await;
        if let Some(ref mut sc) = *sc_guard {
            let sc_query = format!("{} {}", artist, title);
            if let Some(m) = sc.search_track(&client, title, artist).await {
                search_queries.push(format!("SoundCloud: {sc_query}"));
                return Json(AudioSearchResponse {
                    source: Some("soundcloud".to_string()),
                    preview_url: Some(build_proxy_url(&m.stream_url)),
                    external_url: Some(m.permalink_url),
                    search_queries,
                    deezer_id: None,
                    itunes_id: None,
                    soundcloud_id: Some(m.soundcloud_id),
                    uploader_name: Some(m.uploader_name),
                    spotify_uri,
                })
                .into_response();
            }
            search_queries.push(format!("SoundCloud: {sc_query}"));
        }
    }

    // 5. No match found
    Json(AudioSearchResponse {
        source: None,
        preview_url: None,
        external_url: None,
        search_queries,
        deezer_id: None,
        itunes_id: None,
        soundcloud_id: None,
        uploader_name: None,
        spotify_uri,
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
    State(pool): State<PgPool>,
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

pub fn audio_router(pool: PgPool) -> Router {
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
    fn test_is_allowed_host_soundcloud() {
        // api.soundcloud.com NOT whitelisted — stream_url resolved to CDN server-side
        assert!(!is_allowed_host("api.soundcloud.com"));
        assert!(is_allowed_host("cf-preview-media.sndcdn.com"));
        assert!(is_allowed_host("a1.sndcdn.com"));
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
            soundcloud_id: None,
            uploader_name: None,
            spotify_uri: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["source"], "itunes");
        assert_eq!(json["itunes_id"], 123456789);
        assert!(json["deezer_id"].is_null());
        assert!(json["soundcloud_id"].is_null());
        assert!(json["uploader_name"].is_null());
        assert!(json["spotify_uri"].is_null());
    }

    #[test]
    fn test_audio_search_response_with_spotify_uri() {
        let resp = AudioSearchResponse {
            source: Some("deezer".to_string()),
            preview_url: Some("/api/audio/proxy?url=https%3A%2F%2Fdzcdn.net%2Ffoo.mp3".to_string()),
            external_url: Some("https://www.deezer.com/track/12345".to_string()),
            search_queries: vec![r#"artist:"Avicii" track:"Levels""#.to_string()],
            deezer_id: Some(12345),
            itunes_id: None,
            soundcloud_id: None,
            uploader_name: None,
            spotify_uri: Some("spotify:track:3qT4bUD1MaWpGrTwcvguhb".to_string()),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["source"], "deezer");
        assert_eq!(json["spotify_uri"], "spotify:track:3qT4bUD1MaWpGrTwcvguhb");
    }

    #[test]
    fn test_audio_search_response_soundcloud() {
        let resp = AudioSearchResponse {
            source: Some("soundcloud".to_string()),
            preview_url: Some("/api/audio/proxy?url=https%3A%2F%2Fcf-preview-media.sndcdn.com%2Fpreview%2F0%2F30%2Ftest.128.mp3".to_string()),
            external_url: Some("https://soundcloud.com/artist/track".to_string()),
            search_queries: vec!["SoundCloud: Paperclip People Throw".to_string()],
            deezer_id: None,
            itunes_id: None,
            soundcloud_id: Some("soundcloud:tracks:1066423924".to_string()),
            uploader_name: Some("Paperclip People".to_string()),
            spotify_uri: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["source"], "soundcloud");
        assert_eq!(json["soundcloud_id"], "soundcloud:tracks:1066423924");
        assert_eq!(json["uploader_name"], "Paperclip People");
        assert!(json["deezer_id"].is_null());
        assert!(json["itunes_id"].is_null());
        assert!(json["spotify_uri"].is_null());
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
            soundcloud_id: None,
            uploader_name: None,
            spotify_uri: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json["source"].is_null());
        assert_eq!(json["search_queries"].as_array().unwrap().len(), 2);
        assert!(json["spotify_uri"].is_null());
    }

    #[test]
    fn test_spotify_search_response_deserializes() {
        let json = r#"{
            "tracks": {
                "href": "https://api.spotify.com/v1/search",
                "items": [
                    {
                        "uri": "spotify:track:3qT4bUD1MaWpGrTwcvguhb",
                        "name": "Levels",
                        "artists": [{"name": "Avicii"}]
                    }
                ],
                "limit": 5,
                "total": 1
            }
        }"#;
        let resp: SpotifySearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.tracks.items.len(), 1);
        assert_eq!(
            resp.tracks.items[0].uri,
            "spotify:track:3qT4bUD1MaWpGrTwcvguhb"
        );
        assert_eq!(resp.tracks.items[0].name, "Levels");
        assert_eq!(resp.tracks.items[0].artists[0].name, "Avicii");
    }

    #[test]
    fn test_spotify_cc_token_response_deserializes() {
        let json = r#"{"access_token":"BQD...","token_type":"Bearer","expires_in":3600}"#;
        let tr: SpotifyCCTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(tr.access_token, "BQD...");
        assert_eq!(tr.expires_in, 3600);
    }
}
