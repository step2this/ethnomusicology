use std::collections::HashMap;
use std::sync::Arc;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{NaiveDateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::sync::RwLock;

use crate::db::tokens;

// ---------------------------------------------------------------------------
// App state for OAuth
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AuthState {
    pub pool: SqlitePool,
    pub csrf_states: Arc<RwLock<HashMap<String, (String, NaiveDateTime)>>>,
    pub encryption_key: [u8; 32],
    pub spotify_client_id: String,
    pub spotify_redirect_uri: String,
    /// Injectable token exchange function for testability.
    /// Takes (code, redirect_uri, client_id) and returns (access_token, refresh_token, expires_in_secs).
    pub token_exchanger: Arc<dyn TokenExchanger>,
}

/// Trait for exchanging an OAuth code for tokens, allowing test mocking.
#[async_trait::async_trait]
pub trait TokenExchanger: Send + Sync {
    async fn exchange(
        &self,
        code: &str,
        redirect_uri: &str,
        client_id: &str,
    ) -> Result<TokenExchangeResult, anyhow::Error>;
}

pub struct TokenExchangeResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub scope: String,
}

/// No-op exchanger used as a placeholder until the real Spotify client is wired in.
pub struct NoOpExchanger;

#[async_trait::async_trait]
impl TokenExchanger for NoOpExchanger {
    async fn exchange(
        &self,
        _code: &str,
        _redirect_uri: &str,
        _client_id: &str,
    ) -> Result<TokenExchangeResult, anyhow::Error> {
        Err(anyhow::anyhow!(
            "Token exchange not configured — wire in SpotifyClient"
        ))
    }
}

// ---------------------------------------------------------------------------
// Encryption helpers
// ---------------------------------------------------------------------------

pub fn encrypt_token(key: &[u8; 32], plaintext: &str) -> Result<Vec<u8>, anyhow::Error> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {e}"))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

    // Prepend nonce to ciphertext: nonce (12 bytes) || ciphertext
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt_token(key: &[u8; 32], data: &[u8]) -> Result<String, anyhow::Error> {
    if data.len() < 12 {
        return Err(anyhow::anyhow!("Ciphertext too short"));
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {e}"))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {e}"))?;

    String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {e}"))
}

// ---------------------------------------------------------------------------
// Route handler types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct StatusResponse {
    connected: bool,
}

#[derive(Serialize)]
struct RedirectResponse {
    redirect_url: String,
}

#[derive(Serialize)]
struct CallbackSuccessResponse {
    success: bool,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

// ---------------------------------------------------------------------------
// Helper: extract user_id from X-User-Id header
// ---------------------------------------------------------------------------

fn extract_user_id(headers: &HeaderMap) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing X-User-Id header".to_string(),
                }),
            )
        })
}

// ---------------------------------------------------------------------------
// GET /api/auth/spotify/status
// ---------------------------------------------------------------------------

async fn spotify_status(
    State(state): State<AuthState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let user_id = extract_user_id(&headers)?;

    let connected = match tokens::get_tokens(&state.pool, &user_id).await {
        Ok(Some((_access, _refresh, expires_at, _scopes))) => expires_at > Utc::now().naive_utc(),
        _ => false,
    };

    Ok(Json(StatusResponse { connected }))
}

// ---------------------------------------------------------------------------
// GET /api/auth/spotify — initiate OAuth
// ---------------------------------------------------------------------------

async fn spotify_authorize(
    State(state): State<AuthState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let user_id = extract_user_id(&headers)?;

    // Generate 32-byte random state, base64url-encode it
    let mut state_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut state_bytes);
    let state_token = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        state_bytes,
    );

    // Store in CSRF map with timestamp
    {
        let mut csrf = state.csrf_states.write().await;
        csrf.insert(state_token.clone(), (user_id, Utc::now().naive_utc()));
    }

    let scopes = "playlist-read-private playlist-read-collaborative user-library-read";
    let mut auth_url =
        url::Url::parse("https://accounts.spotify.com/authorize").expect("valid base URL");
    auth_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &state.spotify_client_id)
        .append_pair("scope", scopes)
        .append_pair("redirect_uri", &state.spotify_redirect_uri)
        .append_pair("state", &state_token);

    Ok(Json(RedirectResponse {
        redirect_url: auth_url.to_string(),
    }))
}

// ---------------------------------------------------------------------------
// GET /api/auth/spotify/callback?code=...&state=...
// ---------------------------------------------------------------------------

const CSRF_TTL_SECS: i64 = 300; // 5 minutes

async fn spotify_callback(
    State(state): State<AuthState>,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Validate and consume CSRF state
    let user_id = {
        let mut csrf = state.csrf_states.write().await;
        match csrf.remove(&params.state) {
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid state parameter".to_string(),
                    }),
                ));
            }
            Some((uid, created_at)) => {
                let elapsed = (Utc::now().naive_utc() - created_at).num_seconds();
                if elapsed > CSRF_TTL_SECS {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "State parameter expired".to_string(),
                        }),
                    ));
                }
                uid
            }
        }
    };

    // Exchange code for tokens via injected exchanger
    let exchange_result = state
        .token_exchanger
        .exchange(
            &params.code,
            &state.spotify_redirect_uri,
            &state.spotify_client_id,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Token exchange failed: {e}"),
                }),
            )
        })?;

    // Encrypt tokens
    let access_encrypted = encrypt_token(&state.encryption_key, &exchange_result.access_token)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Encryption failed: {e}"),
                }),
            )
        })?;

    let refresh_encrypted = encrypt_token(&state.encryption_key, &exchange_result.refresh_token)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Encryption failed: {e}"),
                }),
            )
        })?;

    let expires_at = Utc::now().naive_utc() + chrono::Duration::seconds(exchange_result.expires_in);

    // Store encrypted tokens in DB
    tokens::store_tokens(
        &state.pool,
        &user_id,
        &access_encrypted,
        &refresh_encrypted,
        expires_at,
        &exchange_result.scope,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to store tokens: {e}"),
            }),
        )
    })?;

    Ok(Json(CallbackSuccessResponse { success: true }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn auth_routes(state: AuthState) -> Router {
    Router::new()
        .route("/auth/spotify/status", get(spotify_status))
        .route("/auth/spotify", get(spotify_authorize))
        .route("/auth/spotify/callback", get(spotify_callback))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_test_pool, create_test_user};
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    /// Mock token exchanger for tests
    struct MockExchanger;

    #[async_trait::async_trait]
    impl TokenExchanger for MockExchanger {
        async fn exchange(
            &self,
            _code: &str,
            _redirect_uri: &str,
            _client_id: &str,
        ) -> Result<TokenExchangeResult, anyhow::Error> {
            Ok(TokenExchangeResult {
                access_token: "mock_access_token".to_string(),
                refresh_token: "mock_refresh_token".to_string(),
                expires_in: 3600,
                scope: "playlist-read-private".to_string(),
            })
        }
    }

    fn test_encryption_key() -> [u8; 32] {
        [0x42u8; 32]
    }

    async fn build_test_app() -> (Router, AuthState) {
        let pool = create_test_pool().await;
        let state = AuthState {
            pool,
            csrf_states: Arc::new(RwLock::new(HashMap::new())),
            encryption_key: test_encryption_key(),
            spotify_client_id: "test_client_id".to_string(),
            spotify_redirect_uri: "http://localhost:3001/api/auth/spotify/callback".to_string(),
            token_exchanger: Arc::new(MockExchanger),
        };
        let app = Router::new().nest("/api", auth_routes(state.clone()));
        (app, state)
    }

    async fn body_to_json(body: Body) -> serde_json::Value {
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    // -----------------------------------------------------------------------
    // Encryption round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_encryption_key();
        let plaintext = "my_secret_token_value";

        let encrypted = encrypt_token(&key, plaintext).unwrap();
        assert!(encrypted.len() > 12); // nonce + ciphertext

        let decrypted = decrypt_token(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32];

        let encrypted = encrypt_token(&key1, "secret").unwrap();
        let result = decrypt_token(&key2, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        let key = test_encryption_key();
        let result = decrypt_token(&key, &[0u8; 5]);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // GET /api/auth/spotify/status — no tokens → connected: false
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_status_not_connected() {
        let (app, _state) = build_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/spotify/status")
                    .header("X-User-Id", "some-user")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_to_json(response.into_body()).await;
        assert_eq!(json["connected"], false);
    }

    // -----------------------------------------------------------------------
    // GET /api/auth/spotify/status — with valid tokens → connected: true
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_status_connected() {
        let (app, state) = build_test_app().await;
        let user_id = create_test_user(&state.pool).await;

        let expires = Utc::now().naive_utc() + chrono::Duration::hours(1);
        tokens::store_tokens(
            &state.pool,
            &user_id,
            b"encrypted_access",
            b"encrypted_refresh",
            expires,
            "scope",
        )
        .await
        .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/spotify/status")
                    .header("X-User-Id", &user_id)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_to_json(response.into_body()).await;
        assert_eq!(json["connected"], true);
    }

    // -----------------------------------------------------------------------
    // GET /api/auth/spotify/status — missing header → 401
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_status_missing_user_id() {
        let (app, _state) = build_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/spotify/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // -----------------------------------------------------------------------
    // GET /api/auth/spotify — returns redirect URL with state
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_authorize_returns_redirect_url() {
        let (app, state) = build_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/spotify")
                    .header("X-User-Id", "user1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_to_json(response.into_body()).await;
        let url = json["redirect_url"].as_str().unwrap();

        assert!(url.starts_with("https://accounts.spotify.com/authorize"));
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("state="));
        assert!(url.contains("response_type=code"));

        // Verify state was stored in CSRF map
        let csrf = state.csrf_states.read().await;
        assert_eq!(csrf.len(), 1);
    }

    // -----------------------------------------------------------------------
    // GET /api/auth/spotify/callback — valid state → success
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_callback_valid_state() {
        let (app, state) = build_test_app().await;
        let user_id = create_test_user(&state.pool).await;

        // Pre-populate CSRF state
        let state_token = "valid_state_token";
        {
            let mut csrf = state.csrf_states.write().await;
            csrf.insert(
                state_token.to_string(),
                (user_id.clone(), Utc::now().naive_utc()),
            );
        }

        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!(
                        "/api/auth/spotify/callback?code=test_code&state={}",
                        state_token
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_to_json(response.into_body()).await;
        assert_eq!(json["success"], true);

        // Verify CSRF state was consumed (one-time use)
        let csrf = state.csrf_states.read().await;
        assert!(csrf.is_empty());

        // Verify tokens were stored
        let stored = tokens::get_tokens(&state.pool, &user_id).await.unwrap();
        assert!(stored.is_some());
    }

    // -----------------------------------------------------------------------
    // GET /api/auth/spotify/callback — invalid state → 400
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_callback_invalid_state() {
        let (app, _state) = build_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/spotify/callback?code=test_code&state=invalid_state")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let json = body_to_json(response.into_body()).await;
        assert!(json["error"].as_str().unwrap().contains("Invalid state"));
    }

    // -----------------------------------------------------------------------
    // GET /api/auth/spotify/callback — expired state → 400
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_callback_expired_state() {
        let (app, state) = build_test_app().await;

        let state_token = "expired_state_token";
        {
            let mut csrf = state.csrf_states.write().await;
            // Insert with a timestamp 10 minutes ago
            let expired = Utc::now().naive_utc() - chrono::Duration::minutes(10);
            csrf.insert(state_token.to_string(), ("user1".to_string(), expired));
        }

        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!(
                        "/api/auth/spotify/callback?code=test_code&state={}",
                        state_token
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let json = body_to_json(response.into_body()).await;
        assert!(json["error"].as_str().unwrap().contains("expired"));
    }
}
