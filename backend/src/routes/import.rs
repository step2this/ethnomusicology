use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::spotify::SpotifyClient;
use crate::services::import::{self, ImportError, ImportRepository, ImportSummary};

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ImportRequest {
    pub playlist_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImportResponse {
    pub import_id: String,
    pub total: u32,
    pub inserted: u32,
    pub updated: u32,
    pub failed: u32,
    pub status: String,
}

impl From<ImportSummary> for ImportResponse {
    fn from(s: ImportSummary) -> Self {
        Self {
            import_id: s.import_id,
            total: s.total,
            inserted: s.inserted,
            updated: s.updated,
            failed: s.failed,
            status: s.status,
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

// ---------------------------------------------------------------------------
// Error → HTTP mapping
// ---------------------------------------------------------------------------

impl IntoResponse for ImportError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            ImportError::InvalidUrl(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ImportError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            ImportError::AccessDenied(m) => (StatusCode::FORBIDDEN, m.clone()),
            ImportError::SpotifyError(_) => {
                (StatusCode::BAD_GATEWAY, "Spotify API error".to_string())
            }
            ImportError::Database(m) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {m}"),
            ),
        };

        (status, Json(ErrorBody { error: msg })).into_response()
    }
}

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

pub struct ImportState {
    pub spotify: SpotifyClient,
    pub repo: Arc<dyn ImportRepository>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn import_spotify(
    State(state): State<Arc<ImportState>>,
    Json(req): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, ImportError> {
    let playlist_id = import::validate_playlist_url(&req.playlist_url)?;

    // TODO: Extract user_id from auth middleware (UC-008). For now use header or hardcoded.
    let user_id = "dev-user";

    // TODO: Extract access_token from stored user tokens. For now use a placeholder.
    let access_token = "placeholder-token";

    let summary = import::import_playlist(
        state.repo.as_ref(),
        &state.spotify,
        access_token,
        user_id,
        &playlist_id,
    )
    .await?;

    Ok(Json(ImportResponse::from(summary)))
}

async fn get_import_status(
    Path(import_id): Path<String>,
) -> Result<Json<ImportResponse>, ImportError> {
    // TODO: Implement actual DB lookup when T5 is ready
    Err(ImportError::NotFound(format!(
        "Import {import_id} not found"
    )))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn import_router(state: Arc<ImportState>) -> Router {
    Router::new()
        .route("/import/spotify", post(import_spotify))
        .route("/import/{id}", get(get_import_status))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Mutex;
    use tower::ServiceExt;

    use crate::services::import::{ArtistRecord, TrackRecord, UpsertResult};

    // -- Simple mock repo for handler tests --

    struct TestRepo {
        tracks: Mutex<Vec<TrackRecord>>,
    }

    impl TestRepo {
        fn new() -> Self {
            Self {
                tracks: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl ImportRepository for TestRepo {
        async fn create_import(
            &self,
            _user_id: &str,
            _playlist_id: &str,
            _playlist_name: Option<&str>,
        ) -> Result<String, ImportError> {
            Ok("test-import-001".to_string())
        }

        async fn upsert_track(&self, track: &TrackRecord) -> Result<UpsertResult, ImportError> {
            self.tracks.lock().unwrap().push(track.clone());
            Ok(UpsertResult::Inserted)
        }

        async fn upsert_artist(&self, _artist: &ArtistRecord) -> Result<UpsertResult, ImportError> {
            Ok(UpsertResult::Inserted)
        }

        async fn upsert_track_artist(
            &self,
            _track_id: &str,
            _artist_id: &str,
        ) -> Result<(), ImportError> {
            Ok(())
        }

        async fn complete_import(
            &self,
            _import_id: &str,
            _summary: &ImportSummary,
        ) -> Result<(), ImportError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_import_invalid_url_returns_400() {
        let state = Arc::new(ImportState {
            spotify: SpotifyClient::new("id", "secret"),
            repo: Arc::new(TestRepo::new()),
        });

        let app = import_router(state);

        let body = serde_json::json!({ "playlist_url": "not-a-url" });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/spotify")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_import_status_not_found() {
        let state = Arc::new(ImportState {
            spotify: SpotifyClient::new("id", "secret"),
            repo: Arc::new(TestRepo::new()),
        });

        let app = import_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/import/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_import_with_mock_spotify() {
        use wiremock::matchers::{method, path_regex};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "items": [
                {
                    "track": {
                        "name": "Track 1",
                        "uri": "spotify:track:t1",
                        "album": { "name": "Album 1" },
                        "duration_ms": 200000,
                        "preview_url": null,
                        "artists": [{ "name": "Artist 1", "uri": "spotify:artist:a1" }]
                    }
                }
            ],
            "total": 1,
            "next": null,
            "offset": 0,
            "limit": 100
        });

        Mock::given(method("GET"))
            .and(path_regex(r"/v1/playlists/.*/tracks.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&mock_server)
            .await;

        let state = Arc::new(ImportState {
            spotify: SpotifyClient::new("id", "secret")
                .with_base_url(mock_server.uri(), mock_server.uri()),
            repo: Arc::new(TestRepo::new()),
        });

        let app = import_router(state);

        let req_body = serde_json::json!({
            "playlist_url": "https://open.spotify.com/playlist/37i9dQZF1DX0BcQWzuB7ZO"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/spotify")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: ImportResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(resp.total, 1);
        assert_eq!(resp.inserted, 1);
        assert_eq!(resp.failed, 0);
        assert_eq!(resp.status, "completed");
    }
}
