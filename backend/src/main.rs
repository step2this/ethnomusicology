use std::collections::HashMap;
use std::sync::Arc;

use axum::{routing::get, Json, Router};
use base64::Engine;
use serde::Serialize;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod repo;
pub mod routes;
pub mod services;

use api::spotify::SpotifyClient;
use config::AppConfig;
use repo::SqliteImportRepository;
use routes::auth::{AuthState, TokenExchangeResult, TokenExchanger};
use routes::import::ImportState;

// ---------------------------------------------------------------------------
// Real Spotify token exchanger
// ---------------------------------------------------------------------------

struct RealTokenExchanger {
    client: SpotifyClient,
}

#[async_trait::async_trait]
impl TokenExchanger for RealTokenExchanger {
    async fn exchange(
        &self,
        code: &str,
        redirect_uri: &str,
        _client_id: &str,
    ) -> Result<TokenExchangeResult, anyhow::Error> {
        let resp = self
            .client
            .exchange_code(code, redirect_uri)
            .await
            .map_err(|e| anyhow::anyhow!("Spotify token exchange failed: {e:?}"))?;

        Ok(TokenExchangeResult {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token.unwrap_or_default(),
            expires_in: resp.expires_in as i64,
            scope: resp.scope,
        })
    }
}

// ---------------------------------------------------------------------------
// Health / info endpoints
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Serialize)]
struct ApiInfo {
    name: String,
    description: String,
    endpoints: Vec<String>,
}

async fn api_info() -> Json<ApiInfo> {
    Json(ApiInfo {
        name: "Ethnomusicology API".to_string(),
        description:
            "Music playlist curation for occasions, featuring African and Middle Eastern traditions"
                .to_string(),
        endpoints: vec!["GET /api/health".to_string(), "GET /api".to_string()],
    })
}

fn api_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/", get(api_info))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cfg = AppConfig::from_env();

    // --- Database ---
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await?;

    // Run migrations
    let migration_001 = include_str!("../migrations/001_initial_schema.sql");
    sqlx::raw_sql(migration_001).execute(&pool).await?;
    let migration_002 = include_str!("../migrations/002_spotify_imports.sql");
    sqlx::raw_sql(migration_002).execute(&pool).await?;
    sqlx::raw_sql("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    tracing::info!("Database migrations applied");

    // Ensure dev-user exists (temporary until UC-008 adds real auth)
    sqlx::query(
        "INSERT OR IGNORE INTO users (id, email, display_name) VALUES ('dev-user', 'dev@local', 'Dev User')",
    )
    .execute(&pool)
    .await?;

    // --- Spotify client ---
    let spotify_client = SpotifyClient::new(&cfg.spotify_client_id, &cfg.spotify_client_secret);

    // --- Encryption key ---
    let encryption_key: [u8; 32] = if cfg.token_encryption_key.is_empty() {
        tracing::warn!(
            "TOKEN_ENCRYPTION_KEY not set, generating ephemeral key (tokens won't survive restart)"
        );
        let mut key = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key);
        key
    } else {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&cfg.token_encryption_key)
            .expect("TOKEN_ENCRYPTION_KEY must be valid base64");
        let mut key = [0u8; 32];
        key.copy_from_slice(&decoded);
        key
    };

    // --- Auth routes state ---
    let auth_state = AuthState {
        pool: pool.clone(),
        csrf_states: Arc::new(RwLock::new(HashMap::new())),
        encryption_key,
        spotify_client_id: cfg.spotify_client_id.clone(),
        spotify_redirect_uri: cfg.spotify_redirect_uri.clone(),
        token_exchanger: Arc::new(RealTokenExchanger {
            client: SpotifyClient::new(&cfg.spotify_client_id, &cfg.spotify_client_secret),
        }),
    };

    // --- Import routes state ---
    let import_state = Arc::new(ImportState {
        spotify: spotify_client,
        repo: Arc::new(SqliteImportRepository::new(pool.clone())),
        pool: pool.clone(),
        encryption_key,
    });

    // --- Router ---
    let app = Router::new()
        .nest("/api", api_router())
        .nest("/api", routes::auth::auth_routes(auth_state))
        .nest("/api", routes::import::import_router(import_state))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = format!("127.0.0.1:{}", cfg.server_port);
    tracing::info!("Backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = Router::new().nest("/api", api_router());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_info_endpoint() {
        let app = Router::new().nest("/api", api_router());

        let response = app
            .oneshot(Request::builder().uri("/api").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
