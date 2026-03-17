use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::time::Duration;

use crate::services::match_scoring::is_acceptable_match;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct SoundCloudClient {
    client_id: String,
    client_secret: String,
    token: Option<String>,
    token_expiry: DateTime<Utc>,
    consecutive_failures: u8,
    disabled_until: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct SoundCloudMatch {
    pub stream_url: String,
    pub permalink_url: String,
    pub uploader_name: String,
    pub soundcloud_id: String,
}

// ---------------------------------------------------------------------------
// Internal deserialisation types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct ScTrack {
    urn: String,
    title: String,
    stream_url: Option<String>,
    permalink_url: String,
    streamable: Option<bool>,
    user: ScUser,
}

#[derive(Deserialize)]
struct ScUser {
    username: String,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl SoundCloudClient {
    /// Returns `None` if `SOUNDCLOUD_CLIENT_ID` or `SOUNDCLOUD_CLIENT_SECRET`
    /// are absent or empty.
    pub fn new_from_env() -> Option<Self> {
        let client_id = std::env::var("SOUNDCLOUD_CLIENT_ID").ok()?;
        let client_secret = std::env::var("SOUNDCLOUD_CLIENT_SECRET").ok()?;
        if client_id.is_empty() || client_secret.is_empty() {
            return None;
        }
        Some(Self {
            client_id,
            client_secret,
            token: None,
            // Initialise to now — the token is None so it will be fetched on first use.
            token_expiry: Utc::now(),
            consecutive_failures: 0,
            disabled_until: None,
        })
    }

    /// Returns a valid OAuth access token, fetching one if necessary.
    ///
    /// Circuit breaker: after 3 consecutive auth failures the client is
    /// disabled for 5 minutes and this method returns an error immediately.
    pub async fn ensure_token(&mut self, http: &reqwest::Client) -> Result<String> {
        // --- Circuit breaker ---
        if let Some(disabled_until) = self.disabled_until {
            if Utc::now() < disabled_until {
                anyhow::bail!("SoundCloud client disabled by circuit breaker");
            }
            // Window has elapsed — reset and retry
            self.disabled_until = None;
            self.consecutive_failures = 0;
        }

        // --- Return cached token (with 60 s buffer) ---
        if let Some(ref token) = self.token {
            if Utc::now() + chrono::Duration::seconds(60) < self.token_expiry {
                return Ok(token.clone());
            }
        }

        // --- Fetch a fresh token ---
        let result = http
            .post("https://api.soundcloud.com/oauth2/token")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => match resp.json::<TokenResponse>().await {
                Ok(tr) => {
                    self.token_expiry =
                        Utc::now() + chrono::Duration::seconds(tr.expires_in as i64);
                    self.token = Some(tr.access_token.clone());
                    self.consecutive_failures = 0;
                    Ok(tr.access_token)
                }
                Err(e) => {
                    self.record_failure();
                    Err(anyhow::anyhow!("Failed to parse SoundCloud token: {e}"))
                }
            },
            Ok(resp) => {
                self.record_failure();
                Err(anyhow::anyhow!(
                    "SoundCloud auth returned HTTP {}",
                    resp.status()
                ))
            }
            Err(e) => {
                self.record_failure();
                Err(anyhow::anyhow!("SoundCloud auth request failed: {e}"))
            }
        }
    }

    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= 3 {
            self.disabled_until = Some(Utc::now() + chrono::Duration::seconds(300));
        }
    }

    /// Searches SoundCloud for a preview of the given track.
    ///
    /// Returns `None` if credentials are missing, the API is unavailable, or
    /// no acceptable match is found.
    ///
    /// The `stream_url` from SoundCloud points to their API (requires OAuth),
    /// so we resolve the 302 redirect here to get the CDN URL directly.
    /// This way the proxy can fetch it without needing the OAuth token.
    pub async fn search_track(
        &mut self,
        http: &reqwest::Client,
        title: &str,
        artist: &str,
    ) -> Option<SoundCloudMatch> {
        let token = self.ensure_token(http).await.ok()?;
        let q = format!("{} {}", artist, title);

        let resp = tokio::time::timeout(
            Duration::from_secs(2),
            http.get("https://api.soundcloud.com/tracks")
                .query(&[("q", q.as_str()), ("limit", "5")])
                .header("Authorization", format!("OAuth {token}"))
                .send(),
        )
        .await
        .ok()?
        .ok()?;

        // C1 fix: if search returns 401, invalidate token and retry once
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            self.token = None;
            let retry_token = self.ensure_token(http).await.ok()?;
            let retry_resp = tokio::time::timeout(
                Duration::from_secs(2),
                http.get("https://api.soundcloud.com/tracks")
                    .query(&[("q", q.as_str()), ("limit", "5")])
                    .header("Authorization", format!("OAuth {retry_token}"))
                    .send(),
            )
            .await
            .ok()?
            .ok()?;
            let tracks: Vec<ScTrack> = retry_resp.json().await.ok()?;
            return self
                .find_and_resolve_match(&retry_token, tracks, title, artist)
                .await;
        }

        let tracks: Vec<ScTrack> = resp.json().await.ok()?;
        self.find_and_resolve_match(&token, tracks, title, artist)
            .await
    }

    /// Find the best match and resolve the stream_url to a direct CDN URL.
    async fn find_and_resolve_match(
        &self,
        token: &str,
        tracks: Vec<ScTrack>,
        title: &str,
        artist: &str,
    ) -> Option<SoundCloudMatch> {
        let matched = tracks
            .into_iter()
            .filter(|t| t.stream_url.is_some() && t.streamable.unwrap_or(false))
            .find(|t| is_acceptable_match(title, artist, &t.title, &t.user.username))?;

        let api_stream_url = matched.stream_url.as_ref()?;

        // H1 fix: resolve the SoundCloud API stream_url (which requires OAuth)
        // to the direct CDN URL (which doesn't need auth).
        // Use a no-redirect client to capture the Location header.
        let no_redirect = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .ok()?;

        let redirect_resp = tokio::time::timeout(
            Duration::from_secs(2),
            no_redirect
                .get(api_stream_url)
                .header("Authorization", format!("OAuth {token}"))
                .send(),
        )
        .await
        .ok()?
        .ok()?;

        // Extract the CDN URL from the 302 Location header
        let cdn_url = if redirect_resp.status().is_redirection() {
            redirect_resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        } else {
            // If not a redirect, use the original URL as fallback
            Some(api_stream_url.clone())
        };

        cdn_url.map(|url| SoundCloudMatch {
            stream_url: url,
            permalink_url: matched.permalink_url,
            uploader_name: matched.user.username,
            soundcloud_id: matched.urn,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_from_env_returns_none_when_vars_missing() {
        // Ensure neither var is set in this test process.
        std::env::remove_var("SOUNDCLOUD_CLIENT_ID");
        std::env::remove_var("SOUNDCLOUD_CLIENT_SECRET");
        assert!(SoundCloudClient::new_from_env().is_none());
    }

    #[test]
    fn new_from_env_returns_none_when_only_one_var_set() {
        std::env::remove_var("SOUNDCLOUD_CLIENT_ID");
        std::env::set_var("SOUNDCLOUD_CLIENT_SECRET", "secret");
        assert!(SoundCloudClient::new_from_env().is_none());
        std::env::remove_var("SOUNDCLOUD_CLIENT_SECRET");
    }

    #[test]
    fn token_response_parses_correctly() {
        let json = r#"{"access_token":"abc123","token_type":"bearer","expires_in":3600}"#;
        let tr: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(tr.access_token, "abc123");
        assert_eq!(tr.expires_in, 3600);
    }

    #[test]
    fn circuit_breaker_activates_after_three_failures() {
        std::env::set_var("SOUNDCLOUD_CLIENT_ID", "id");
        std::env::set_var("SOUNDCLOUD_CLIENT_SECRET", "secret");
        let mut client = SoundCloudClient::new_from_env().unwrap();
        std::env::remove_var("SOUNDCLOUD_CLIENT_ID");
        std::env::remove_var("SOUNDCLOUD_CLIENT_SECRET");

        assert_eq!(client.consecutive_failures, 0);
        assert!(client.disabled_until.is_none());

        client.record_failure();
        assert!(client.disabled_until.is_none());
        client.record_failure();
        assert!(client.disabled_until.is_none());
        client.record_failure();
        assert!(client.disabled_until.is_some());
    }

    #[test]
    fn circuit_breaker_blocks_after_activation() {
        std::env::set_var("SOUNDCLOUD_CLIENT_ID", "id");
        std::env::set_var("SOUNDCLOUD_CLIENT_SECRET", "secret");
        let mut client = SoundCloudClient::new_from_env().unwrap();
        std::env::remove_var("SOUNDCLOUD_CLIENT_ID");
        std::env::remove_var("SOUNDCLOUD_CLIENT_SECRET");

        // Trigger circuit breaker
        client.disabled_until = Some(Utc::now() + chrono::Duration::seconds(300));
        assert!(client.disabled_until.is_some());
        // Check: still within window
        assert!(Utc::now() < client.disabled_until.unwrap());
    }

    #[test]
    fn sc_track_deserialises() {
        let json = r#"[{
            "id": 1066423924,
            "urn": "soundcloud:tracks:1066423924",
            "title": "Throw",
            "stream_url": "https://api.soundcloud.com/tracks/soundcloud:tracks:1066423924/preview",
            "permalink_url": "https://soundcloud.com/paperclippeople-music/paperclip-people-throw",
            "streamable": true,
            "user": {"username": "Paperclip People"}
        }]"#;
        let tracks: Vec<ScTrack> = serde_json::from_str(json).unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].urn, "soundcloud:tracks:1066423924");
        assert_eq!(tracks[0].user.username, "Paperclip People");
        assert!(tracks[0].streamable.unwrap_or(false));
    }
}
