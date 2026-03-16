use std::collections::HashMap;
use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use base64::Engine;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use ethnomusicology_backend::api::claude::ClaudeClient;
use ethnomusicology_backend::api::spotify::SpotifyClient;
use ethnomusicology_backend::config::AppConfig;
use ethnomusicology_backend::repo::PgImportRepository;
use ethnomusicology_backend::routes;
use ethnomusicology_backend::routes::auth::{AuthState, TokenExchangeResult, TokenExchanger};
use ethnomusicology_backend::routes::enrich::EnrichRouteState;
use ethnomusicology_backend::routes::import::ImportState;
use ethnomusicology_backend::routes::purchase_links::PurchaseLinkRouteState;
use ethnomusicology_backend::routes::refinement::RefinementRouteState;
use ethnomusicology_backend::routes::setlist::SetlistRouteState;
use ethnomusicology_backend::services::purchase_links::AffiliateConfig;

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
            "DJ-first music platform — LLM-powered setlist generation with harmonic mixing"
                .to_string(),
        endpoints: vec!["GET /api/health".to_string(), "GET /api".to_string()],
    })
}

async fn health_ready(
    axum::extract::State(pool): axum::extract::State<sqlx::PgPool>,
) -> axum::response::Response {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => axum::Json(serde_json::json!({
            "status": "ok",
            "db": "ok",
            "version": env!("CARGO_PKG_VERSION")
        }))
        .into_response(),
        Err(e) => (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({"status": "error", "db": e.to_string()})),
        )
            .into_response(),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, draining connections...");
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

    if cfg.dev_mode {
        tracing::warn!("DEV_MODE enabled — dev-only endpoints are active");
    }

    // --- Database ---
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await?;

    // Run migrations (shared with test pool — single source of truth in db/mod.rs)
    ethnomusicology_backend::db::run_migrations(&pool).await?;
    tracing::info!("Database migrations applied");

    // Ensure dev-user exists (temporary until UC-008 adds real auth)
    sqlx::query(
        "INSERT INTO users (id, email, display_name) VALUES ('dev-user', 'dev@local', 'Dev User') ON CONFLICT DO NOTHING",
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
    // --- Claude client (shared) ---
    let claude_client: Arc<dyn ethnomusicology_backend::api::claude::ClaudeClientTrait> =
        Arc::new(ClaudeClient::new(&cfg.anthropic_api_key));

    let import_state = Arc::new(ImportState {
        spotify: spotify_client,
        repo: Arc::new(PgImportRepository::new(pool.clone())),
        pool: pool.clone(),
        encryption_key,
        claude: claude_client.clone(),
    });

    // --- Setlist routes state ---
    let setlist_state = Arc::new(SetlistRouteState {
        pool: pool.clone(),
        claude: claude_client.clone(),
    });

    // --- Enrich routes state ---
    let enrich_state = Arc::new(EnrichRouteState {
        pool: pool.clone(),
        claude: claude_client.clone(),
        in_flight: std::sync::atomic::AtomicBool::new(false),
    });

    // --- Refinement routes state ---
    let refinement_state = Arc::new(RefinementRouteState {
        pool: pool.clone(),
        claude: claude_client.clone(),
    });

    // --- Purchase links state ---
    let purchase_link_state = Arc::new(PurchaseLinkRouteState {
        affiliate_config: AffiliateConfig::from_env(),
    });

    // --- Health readiness router (needs DB pool) ---
    let health_router = Router::new()
        .route("/api/health/ready", get(health_ready))
        .with_state(pool.clone());

    // --- Router ---
    let mut app = Router::new()
        .nest("/api", api_router())
        .merge(health_router)
        .nest("/api", routes::auth::auth_routes(auth_state))
        .nest("/api", routes::import::import_router(import_state))
        .nest("/api", routes::setlist::setlist_router(setlist_state))
        .nest("/api", routes::enrich::enrich_router(enrich_state))
        .nest(
            "/api",
            routes::refinement::refinement_router(refinement_state),
        )
        .nest("/api", routes::tracks::tracks_router(pool.clone()))
        .nest("/api", routes::audio::audio_router(pool.clone()))
        .nest("/api", routes::admin::admin_router(pool.clone()))
        .nest(
            "/api",
            routes::crates::crate_routes(std::sync::Arc::new(routes::crates::CrateRouteState {
                pool: pool.clone(),
            })),
        )
        .nest(
            "/api",
            routes::purchase_links::purchase_link_router(purchase_link_state),
        );

    // Dev routes (conditionally added when DEV_MODE=true)
    if cfg.dev_mode {
        app = app.nest("/api", routes::dev::dev_router(pool.clone()));
    }

    let static_dir = std::path::Path::new("../frontend/build/web");
    if static_dir.exists() {
        tracing::info!("Serving Flutter web build from {:?}", static_dir);
        let serve_dir =
            ServeDir::new(static_dir).fallback(ServeFile::new(static_dir.join("index.html")));
        app = app.fallback_service(serve_dir);
    } else {
        tracing::warn!(
            "Flutter web build not found at {:?} — static file serving disabled",
            static_dir
        );
    }

    let app = app
        .layer({
            use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin};

            let origins = if cfg.dev_mode {
                AllowOrigin::any()
            } else {
                AllowOrigin::predicate(|origin, _| {
                    let origin = origin.as_bytes();
                    origin == b"https://tarab.studio"
                        || origin == b"http://localhost:3000"
                        || origin.ends_with(b".vercel.app") && origin.starts_with(b"https://")
                })
            };
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(AllowMethods::any())
                .allow_headers(AllowHeaders::any())
        })
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", cfg.bind_address, cfg.server_port);
    tracing::info!("Backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

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
