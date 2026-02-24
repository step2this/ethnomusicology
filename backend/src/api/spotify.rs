use reqwest::Client;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SpotifyError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Spotify API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),
}

// ---------------------------------------------------------------------------
// Types – public/normalised
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    #[serde(default)]
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyTrack {
    pub name: String,
    pub uri: String,
    pub album_name: String,
    pub duration_ms: u64,
    pub preview_url: Option<String>,
    pub artists: Vec<SpotifyArtist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyArtist {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTracksResponse {
    pub items: Vec<PlaylistItem>,
    pub total: u32,
    pub next: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub track: Option<SpotifyTrackRaw>,
}

// ---------------------------------------------------------------------------
// Types – raw Spotify JSON shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyTrackRaw {
    pub name: String,
    pub uri: String,
    pub album: SpotifyAlbumRaw,
    pub duration_ms: u64,
    pub preview_url: Option<String>,
    #[serde(default)]
    pub artists: Vec<SpotifyArtistRaw>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyAlbumRaw {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyArtistRaw {
    pub name: String,
    pub uri: String,
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

impl SpotifyTrackRaw {
    pub fn into_track(self) -> SpotifyTrack {
        SpotifyTrack {
            name: self.name,
            uri: self.uri,
            album_name: self.album.name,
            duration_ms: self.duration_ms,
            preview_url: self.preview_url,
            artists: self
                .artists
                .into_iter()
                .map(|a| SpotifyArtist {
                    name: a.name,
                    uri: a.uri,
                })
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct SpotifyClient {
    http: Client,
    client_id: String,
    client_secret: String,
    base_url: String,
    auth_url: String,
}

impl SpotifyClient {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            base_url: "https://api.spotify.com".to_string(),
            auth_url: "https://accounts.spotify.com".to_string(),
        }
    }

    /// Override base URLs for testing (e.g. with wiremock).
    pub fn with_base_url(
        mut self,
        base_url: impl Into<String>,
        auth_url: impl Into<String>,
    ) -> Self {
        self.base_url = base_url.into();
        self.auth_url = auth_url.into();
        self
    }

    /// Exchange an authorization code for tokens.
    pub async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<TokenResponse, SpotifyError> {
        let url = format!("{}/api/token", self.auth_url);

        let resp = self
            .http
            .post(&url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await?;

        self.handle_auth_response(resp).await
    }

    /// Refresh an access token.
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, SpotifyError> {
        let url = format!("{}/api/token", self.auth_url);

        let resp = self
            .http
            .post(&url)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;

        self.handle_auth_response(resp).await
    }

    /// Fetch a page of playlist tracks.
    pub async fn get_playlist_tracks(
        &self,
        access_token: &str,
        playlist_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<PlaylistTracksResponse, SpotifyError> {
        let url = format!(
            "{}/v1/playlists/{}/tracks?offset={}&limit={}",
            self.base_url, playlist_id, offset, limit
        );

        let resp = self.http.get(&url).bearer_auth(access_token).send().await?;

        self.handle_api_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    async fn handle_auth_response(
        &self,
        resp: reqwest::Response,
    ) -> Result<TokenResponse, SpotifyError> {
        let status = resp.status().as_u16();
        if status == 200 {
            let token: TokenResponse = resp.json().await?;
            Ok(token)
        } else if status == 401 {
            let body = resp.text().await.unwrap_or_default();
            Err(SpotifyError::AuthFailed(body))
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(SpotifyError::Api {
                status,
                message: body,
            })
        }
    }

    async fn handle_api_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T, SpotifyError> {
        let status = resp.status().as_u16();
        match status {
            200 => {
                let body: T = resp.json().await?;
                Ok(body)
            }
            401 => {
                let body = resp.text().await.unwrap_or_default();
                Err(SpotifyError::AuthFailed(body))
            }
            403 => {
                let body = resp.text().await.unwrap_or_default();
                Err(SpotifyError::AccessDenied(body))
            }
            404 => {
                let body = resp.text().await.unwrap_or_default();
                Err(SpotifyError::NotFound(body))
            }
            429 => {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(1);
                Err(SpotifyError::RateLimited {
                    retry_after_secs: retry_after,
                })
            }
            _ => {
                let body = resp.text().await.unwrap_or_default();
                Err(SpotifyError::Api {
                    status,
                    message: body,
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Realistic playlist tracks JSON from Spotify API.
    fn sample_playlist_json() -> &'static str {
        r#"{
            "items": [
                {
                    "track": {
                        "name": "Habibi Ya Nour El Ain",
                        "uri": "spotify:track:1abc",
                        "album": { "name": "Best of Amr Diab" },
                        "duration_ms": 234000,
                        "preview_url": "https://p.scdn.co/mp3-preview/abc",
                        "artists": [
                            { "name": "Amr Diab", "uri": "spotify:artist:1xyz" },
                            { "name": "Another Artist", "uri": "spotify:artist:2xyz" }
                        ]
                    }
                },
                {
                    "track": {
                        "name": "Qasida Burda",
                        "uri": "spotify:track:2abc",
                        "album": { "name": "Sacred Songs" },
                        "duration_ms": 312000,
                        "preview_url": null,
                        "artists": [
                            { "name": "Mesut Kurtis", "uri": "spotify:artist:3xyz" }
                        ]
                    }
                },
                {
                    "track": null
                },
                {
                    "track": {
                        "name": "Desert Rose",
                        "uri": "spotify:track:3abc",
                        "album": { "name": "Brand New Day" },
                        "duration_ms": 278000,
                        "preview_url": "https://p.scdn.co/mp3-preview/def",
                        "artists": []
                    }
                }
            ],
            "total": 54,
            "next": "https://api.spotify.com/v1/playlists/123/tracks?offset=4&limit=4",
            "offset": 0,
            "limit": 4
        }"#
    }

    #[test]
    fn test_deserialize_playlist_tracks() {
        let resp: PlaylistTracksResponse =
            serde_json::from_str(sample_playlist_json()).expect("deserialization failed");

        assert_eq!(resp.items.len(), 4);
        assert_eq!(resp.total, 54);
        assert_eq!(resp.offset, 0);
        assert_eq!(resp.limit, 4);
        assert!(resp.next.is_some());

        // First item – normal track
        let t0 = resp.items[0].track.as_ref().unwrap();
        assert_eq!(t0.name, "Habibi Ya Nour El Ain");
        assert_eq!(t0.artists.len(), 2);
        assert_eq!(t0.artists[0].name, "Amr Diab");
    }

    #[test]
    fn test_null_preview_url() {
        let resp: PlaylistTracksResponse =
            serde_json::from_str(sample_playlist_json()).expect("deserialization failed");

        let t1 = resp.items[1].track.as_ref().unwrap();
        assert_eq!(t1.name, "Qasida Burda");
        assert!(t1.preview_url.is_none());
    }

    #[test]
    fn test_empty_artists_array() {
        let resp: PlaylistTracksResponse =
            serde_json::from_str(sample_playlist_json()).expect("deserialization failed");

        let t3 = resp.items[3].track.as_ref().unwrap();
        assert_eq!(t3.name, "Desert Rose");
        assert!(t3.artists.is_empty());
    }

    #[test]
    fn test_null_track_in_playlist_item() {
        let resp: PlaylistTracksResponse =
            serde_json::from_str(sample_playlist_json()).expect("deserialization failed");

        // Third item is a local/unavailable track
        assert!(resp.items[2].track.is_none());
    }

    #[test]
    fn test_track_raw_into_track() {
        let raw = SpotifyTrackRaw {
            name: "Test Track".to_string(),
            uri: "spotify:track:abc".to_string(),
            album: SpotifyAlbumRaw {
                name: "Test Album".to_string(),
            },
            duration_ms: 180000,
            preview_url: Some("https://example.com/preview".to_string()),
            artists: vec![SpotifyArtistRaw {
                name: "Test Artist".to_string(),
                uri: "spotify:artist:xyz".to_string(),
            }],
        };

        let track = raw.into_track();
        assert_eq!(track.name, "Test Track");
        assert_eq!(track.album_name, "Test Album");
        assert_eq!(track.artists.len(), 1);
        assert_eq!(track.artists[0].name, "Test Artist");
    }
}
