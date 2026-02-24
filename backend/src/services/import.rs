use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::api::retry::{retry_with_backoff, RetryConfig};
use crate::api::spotify::{SpotifyClient, SpotifyError};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("Invalid playlist URL: {0}")]
    InvalidUrl(String),

    #[error("Spotify error: {0}")]
    SpotifyError(#[from] SpotifyError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),
}

// ---------------------------------------------------------------------------
// DB record types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackRecord {
    pub id: String,
    pub title: String,
    pub album: Option<String>,
    pub duration_ms: Option<i64>,
    pub spotify_uri: String,
    pub spotify_preview_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistRecord {
    pub id: String,
    pub name: String,
    pub spotify_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertResult {
    Inserted,
    Updated,
}

// ---------------------------------------------------------------------------
// Import summary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub import_id: String,
    pub total: u32,
    pub inserted: u32,
    pub updated: u32,
    pub failed: u32,
    pub status: String,
}

// ---------------------------------------------------------------------------
// Repository trait (T5 will provide the real implementation)
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait ImportRepository: Send + Sync {
    async fn create_import(
        &self,
        user_id: &str,
        playlist_id: &str,
        playlist_name: Option<&str>,
    ) -> Result<String, ImportError>;

    async fn upsert_track(&self, track: &TrackRecord) -> Result<UpsertResult, ImportError>;

    async fn upsert_artist(&self, artist: &ArtistRecord) -> Result<UpsertResult, ImportError>;

    async fn upsert_track_artist(&self, track_id: &str, artist_id: &str)
        -> Result<(), ImportError>;

    async fn complete_import(
        &self,
        import_id: &str,
        summary: &ImportSummary,
    ) -> Result<(), ImportError>;
}

// ---------------------------------------------------------------------------
// URL validation
// ---------------------------------------------------------------------------

/// Extract a Spotify playlist ID from a URL or URI.
///
/// Accepted formats:
/// - `https://open.spotify.com/playlist/{id}`
/// - `https://open.spotify.com/playlist/{id}?si=...`
/// - `spotify:playlist:{id}`
pub fn validate_playlist_url(input: &str) -> Result<String, ImportError> {
    let input = input.trim();

    // Try spotify URI format first: spotify:playlist:{id}
    if let Some(id) = input.strip_prefix("spotify:playlist:") {
        let id = id.trim();
        if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric()) {
            return Ok(id.to_string());
        }
        return Err(ImportError::InvalidUrl(format!(
            "Invalid playlist ID in URI: {input}"
        )));
    }

    // Try HTTPS URL: https://open.spotify.com/playlist/{id}[?...]
    let re = Regex::new(r"^https://open\.spotify\.com/playlist/([a-zA-Z0-9]+)(\?.*)?$").unwrap();
    if let Some(caps) = re.captures(input) {
        let id = caps.get(1).unwrap().as_str();
        return Ok(id.to_string());
    }

    Err(ImportError::InvalidUrl(format!(
        "Unrecognised playlist URL or URI: {input}"
    )))
}

// ---------------------------------------------------------------------------
// Import orchestration
// ---------------------------------------------------------------------------

pub async fn import_playlist(
    repo: &dyn ImportRepository,
    spotify: &SpotifyClient,
    access_token: &str,
    user_id: &str,
    playlist_id: &str,
) -> Result<ImportSummary, ImportError> {
    let import_id = repo.create_import(user_id, playlist_id, None).await?;

    let retry_cfg = RetryConfig::default();
    let mut offset: u32 = 0;
    let limit: u32 = 100;
    let mut total_tracks: u32 = 0;
    let mut inserted: u32 = 0;
    let mut updated: u32 = 0;
    let mut failed: u32 = 0;
    let mut first_page = true;

    loop {
        let page_result = {
            let at = access_token;
            let pid = playlist_id;
            let off = offset;
            let lim = limit;
            retry_with_backoff(&retry_cfg, || async move {
                spotify.get_playlist_tracks(at, pid, off, lim).await
            })
            .await
        };

        let page = match page_result {
            Ok(p) => p,
            Err(e) => {
                // API failure mid-import: commit what we have, mark import as failed.
                let summary = ImportSummary {
                    import_id: import_id.clone(),
                    total: total_tracks,
                    inserted,
                    updated,
                    failed,
                    status: "failed".to_string(),
                };
                let _ = repo.complete_import(&import_id, &summary).await;
                return Err(e.into());
            }
        };

        if first_page {
            total_tracks = page.total;
            first_page = false;
        }

        for item in &page.items {
            let raw_track = match &item.track {
                Some(t) => t,
                None => {
                    // Local/unavailable track — skip
                    failed += 1;
                    continue;
                }
            };

            // Build track record
            let track_id = deterministic_id(&raw_track.uri);
            let track_record = TrackRecord {
                id: track_id.clone(),
                title: raw_track.name.clone(),
                album: Some(raw_track.album.name.clone()),
                duration_ms: Some(raw_track.duration_ms as i64),
                spotify_uri: raw_track.uri.clone(),
                spotify_preview_url: raw_track.preview_url.clone(),
            };

            match repo.upsert_track(&track_record).await {
                Ok(UpsertResult::Inserted) => inserted += 1,
                Ok(UpsertResult::Updated) => updated += 1,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            }

            // Upsert each artist and link
            for raw_artist in &raw_track.artists {
                let artist_id = deterministic_id(&raw_artist.uri);
                let artist_record = ArtistRecord {
                    id: artist_id.clone(),
                    name: raw_artist.name.clone(),
                    spotify_uri: raw_artist.uri.clone(),
                };
                let _ = repo.upsert_artist(&artist_record).await;
                let _ = repo.upsert_track_artist(&track_id, &artist_id).await;
            }
        }

        offset += limit;
        if offset >= total_tracks {
            break;
        }
    }

    let summary = ImportSummary {
        import_id: import_id.clone(),
        total: total_tracks,
        inserted,
        updated,
        failed,
        status: "completed".to_string(),
    };

    repo.complete_import(&import_id, &summary).await?;

    Ok(summary)
}

/// Deterministic ID from a Spotify URI (e.g. "spotify:track:abc" → "abc").
fn deterministic_id(uri: &str) -> String {
    uri.rsplit(':').next().unwrap_or(uri).to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // ---- URL validation tests ----

    #[test]
    fn test_valid_url() {
        let id = validate_playlist_url("https://open.spotify.com/playlist/37i9dQZF1DX0BcQWzuB7ZO")
            .unwrap();
        assert_eq!(id, "37i9dQZF1DX0BcQWzuB7ZO");
    }

    #[test]
    fn test_valid_url_with_query() {
        let id = validate_playlist_url(
            "https://open.spotify.com/playlist/37i9dQZF1DX0BcQWzuB7ZO?si=abc123",
        )
        .unwrap();
        assert_eq!(id, "37i9dQZF1DX0BcQWzuB7ZO");
    }

    #[test]
    fn test_valid_uri() {
        let id = validate_playlist_url("spotify:playlist:37i9dQZF1DX0BcQWzuB7ZO").unwrap();
        assert_eq!(id, "37i9dQZF1DX0BcQWzuB7ZO");
    }

    #[test]
    fn test_invalid_url() {
        assert!(validate_playlist_url("https://example.com/playlist/abc").is_err());
    }

    #[test]
    fn test_non_playlist_spotify_url() {
        assert!(
            validate_playlist_url("https://open.spotify.com/album/37i9dQZF1DX0BcQWzuB7ZO").is_err()
        );
    }

    #[test]
    fn test_empty_input() {
        assert!(validate_playlist_url("").is_err());
    }

    #[test]
    fn test_whitespace_trimmed() {
        let id =
            validate_playlist_url("  https://open.spotify.com/playlist/37i9dQZF1DX0BcQWzuB7ZO  ")
                .unwrap();
        assert_eq!(id, "37i9dQZF1DX0BcQWzuB7ZO");
    }

    // ---- Mock repository for import orchestration tests ----

    struct MockRepo {
        imports: Mutex<Vec<String>>,
        tracks: Mutex<Vec<TrackRecord>>,
        artists: Mutex<Vec<ArtistRecord>>,
        track_artists: Mutex<Vec<(String, String)>>,
        completed: Mutex<Vec<ImportSummary>>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                imports: Mutex::new(Vec::new()),
                tracks: Mutex::new(Vec::new()),
                artists: Mutex::new(Vec::new()),
                track_artists: Mutex::new(Vec::new()),
                completed: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl ImportRepository for MockRepo {
        async fn create_import(
            &self,
            _user_id: &str,
            _playlist_id: &str,
            _playlist_name: Option<&str>,
        ) -> Result<String, ImportError> {
            let id = "import-001".to_string();
            self.imports.lock().unwrap().push(id.clone());
            Ok(id)
        }

        async fn upsert_track(&self, track: &TrackRecord) -> Result<UpsertResult, ImportError> {
            let mut tracks = self.tracks.lock().unwrap();
            if tracks.iter().any(|t| t.spotify_uri == track.spotify_uri) {
                Ok(UpsertResult::Updated)
            } else {
                tracks.push(track.clone());
                Ok(UpsertResult::Inserted)
            }
        }

        async fn upsert_artist(&self, artist: &ArtistRecord) -> Result<UpsertResult, ImportError> {
            let mut artists = self.artists.lock().unwrap();
            if artists.iter().any(|a| a.spotify_uri == artist.spotify_uri) {
                Ok(UpsertResult::Updated)
            } else {
                artists.push(artist.clone());
                Ok(UpsertResult::Inserted)
            }
        }

        async fn upsert_track_artist(
            &self,
            track_id: &str,
            artist_id: &str,
        ) -> Result<(), ImportError> {
            self.track_artists
                .lock()
                .unwrap()
                .push((track_id.to_string(), artist_id.to_string()));
            Ok(())
        }

        async fn complete_import(
            &self,
            _import_id: &str,
            summary: &ImportSummary,
        ) -> Result<(), ImportError> {
            self.completed.lock().unwrap().push(summary.clone());
            Ok(())
        }
    }

    // ---- Import orchestration test using wiremock ----

    #[tokio::test]
    async fn test_import_playlist_single_page() {
        use wiremock::matchers::{method, path_regex};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock the playlist tracks endpoint
        let body = serde_json::json!({
            "items": [
                {
                    "track": {
                        "name": "Test Track 1",
                        "uri": "spotify:track:t1",
                        "album": { "name": "Album 1" },
                        "duration_ms": 200000,
                        "preview_url": null,
                        "artists": [
                            { "name": "Artist 1", "uri": "spotify:artist:a1" }
                        ]
                    }
                },
                {
                    "track": null
                },
                {
                    "track": {
                        "name": "Test Track 2",
                        "uri": "spotify:track:t2",
                        "album": { "name": "Album 2" },
                        "duration_ms": 180000,
                        "preview_url": "https://preview.example.com",
                        "artists": [
                            { "name": "Artist 1", "uri": "spotify:artist:a1" },
                            { "name": "Artist 2", "uri": "spotify:artist:a2" }
                        ]
                    }
                }
            ],
            "total": 3,
            "next": null,
            "offset": 0,
            "limit": 100
        });

        Mock::given(method("GET"))
            .and(path_regex(r"/v1/playlists/.*/tracks.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let client =
            SpotifyClient::new("id", "secret").with_base_url(mock_server.uri(), mock_server.uri());

        let repo = MockRepo::new();

        let summary = import_playlist(&repo, &client, "token", "user1", "playlist123")
            .await
            .unwrap();

        assert_eq!(summary.total, 3);
        assert_eq!(summary.inserted, 2); // 2 valid tracks
        assert_eq!(summary.failed, 1); // 1 null track
        assert_eq!(summary.status, "completed");

        // Verify repo state
        assert_eq!(repo.tracks.lock().unwrap().len(), 2);
        assert_eq!(repo.artists.lock().unwrap().len(), 2); // Artist 1 + Artist 2 (deduplicated)
        assert_eq!(repo.track_artists.lock().unwrap().len(), 3); // t1-a1, t2-a1, t2-a2
        assert_eq!(repo.completed.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_import_playlist_multi_page() {
        use wiremock::matchers::{method, path_regex, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Page 1
        let page1 = serde_json::json!({
            "items": [
                {
                    "track": {
                        "name": "Track A",
                        "uri": "spotify:track:a",
                        "album": { "name": "Album A" },
                        "duration_ms": 100000,
                        "preview_url": null,
                        "artists": [{ "name": "Artist X", "uri": "spotify:artist:x" }]
                    }
                }
            ],
            "total": 101,
            "next": "next_url",
            "offset": 0,
            "limit": 100
        });

        // Page 2
        let page2 = serde_json::json!({
            "items": [
                {
                    "track": {
                        "name": "Track B",
                        "uri": "spotify:track:b",
                        "album": { "name": "Album B" },
                        "duration_ms": 120000,
                        "preview_url": null,
                        "artists": [{ "name": "Artist Y", "uri": "spotify:artist:y" }]
                    }
                }
            ],
            "total": 101,
            "next": null,
            "offset": 100,
            "limit": 100
        });

        Mock::given(method("GET"))
            .and(path_regex(r"/v1/playlists/.*/tracks"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page1))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path_regex(r"/v1/playlists/.*/tracks"))
            .and(query_param("offset", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
            .mount(&mock_server)
            .await;

        let client =
            SpotifyClient::new("id", "secret").with_base_url(mock_server.uri(), mock_server.uri());

        let repo = MockRepo::new();

        let summary = import_playlist(&repo, &client, "token", "user1", "playlist456")
            .await
            .unwrap();

        assert_eq!(summary.total, 101);
        assert_eq!(summary.inserted, 2);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.status, "completed");
    }

    #[test]
    fn test_deterministic_id() {
        assert_eq!(deterministic_id("spotify:track:abc123"), "abc123");
        assert_eq!(deterministic_id("spotify:artist:xyz"), "xyz");
        assert_eq!(deterministic_id("nocolon"), "nocolon");
    }
}
