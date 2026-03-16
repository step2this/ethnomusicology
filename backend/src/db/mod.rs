pub mod artists;
pub mod crate_models;
pub mod crates;
pub mod imports;
pub mod models;
pub mod refinement;
pub mod setlists;
pub mod tokens;
pub mod tracks;

use sqlx::PgPool;

/// Run all migrations using sqlx's built-in migration system.
/// Tracks applied migrations in `_sqlx_migrations` table — never re-runs them.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Create a Postgres pool with migrations applied and tables cleaned. For tests only.
/// Uses TEST_DATABASE_URL env var, falling back to DATABASE_URL.
///
/// IMPORTANT: Each test MUST call `pool.close().await` at the end to release
/// connections back to Neon's connection pooler before the next test starts.
pub async fn create_test_pool() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for tests");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&url)
        .await
        .unwrap();
    run_migrations(&pool).await.unwrap();

    // Delete all data from application tables for test isolation.
    // Order matters: delete from child tables before parent tables to respect FK constraints.
    for table in &[
        "crate_tracks",
        "crates",
        "setlist_conversations",
        "setlist_version_tracks",
        "setlist_versions",
        "setlist_tracks",
        "setlists",
        "import_tracks",
        "spotify_imports",
        "track_occasions",
        "track_tags",
        "track_artists",
        "playlist_tracks",
        "user_spotify_tokens",
        "user_usage",
        "tracks",
        "artists",
        "occasions",
        "playlists",
        "users",
    ] {
        sqlx::query(&format!("DELETE FROM {}", table))
            .execute(&pool)
            .await
            .unwrap();
    }

    pool
}

/// Create a test user (required for FK constraints on tokens/imports tables).
pub async fn create_test_user(pool: &PgPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id, email, display_name) VALUES ($1, $2, $3)")
        .bind(&id)
        .bind("test@example.com")
        .bind("Test User")
        .execute(pool)
        .await
        .unwrap();
    id
}
