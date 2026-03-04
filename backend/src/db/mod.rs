pub mod artists;
pub mod imports;
pub mod models;
pub mod refinement;
pub mod setlists;
pub mod tokens;
pub mod tracks;

use sqlx::SqlitePool;

/// Run all migrations using sqlx's built-in migration system.
/// Tracks applied migrations in `_sqlx_migrations` table — never re-runs them.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Create an in-memory SQLite pool with all migrations applied. For tests only.
pub async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();
    pool
}

/// Create a test user (required for FK constraints on tokens/imports tables).
pub async fn create_test_user(pool: &SqlitePool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id, email, display_name) VALUES (?, ?, ?)")
        .bind(&id)
        .bind("test@example.com")
        .bind("Test User")
        .execute(pool)
        .await
        .unwrap();
    id
}
