/// Application configuration loaded from environment variables.
pub struct AppConfig {
    pub database_url: String,
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
    pub spotify_redirect_uri: String,
    pub token_encryption_key: String,
    pub anthropic_api_key: String,
    pub server_port: u16,
}

impl AppConfig {
    /// Load configuration from environment variables.
    /// Falls back to defaults for development.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        if anthropic_api_key.is_empty() {
            tracing::warn!("ANTHROPIC_API_KEY not set — setlist generation will fail");
        }

        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:ethnomusicology.db?mode=rwc".to_string()),
            spotify_client_id: std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_default(),
            spotify_client_secret: std::env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default(),
            spotify_redirect_uri: std::env::var("SPOTIFY_REDIRECT_URI")
                .unwrap_or_else(|_| "http://127.0.0.1:3001/api/auth/spotify/callback".to_string()),
            token_encryption_key: std::env::var("TOKEN_ENCRYPTION_KEY").unwrap_or_default(),
            anthropic_api_key,
            server_port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
        }
    }
}
