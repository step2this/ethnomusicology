/// Application configuration loaded from environment variables.
pub struct AppConfig {
    pub database_url: String,
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
    pub server_port: u16,
}

impl AppConfig {
    /// Load configuration from environment variables.
    /// Falls back to defaults for development.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:ethnomusicology.db".to_string()),
            spotify_client_id: std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_default(),
            spotify_client_secret: std::env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default(),
            server_port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
        }
    }
}
