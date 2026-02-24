pub mod artists;
pub mod imports;
pub mod models;
pub mod tokens;
pub mod tracks;

use sqlx::SqlitePool;

/// Create an in-memory SQLite pool with all migrations applied. For tests only.
pub async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let migration_001 = include_str!("../../migrations/001_initial_schema.sql");
    sqlx::raw_sql(migration_001).execute(&pool).await.unwrap();

    let migration_002 = include_str!("../../migrations/002_spotify_imports.sql");
    sqlx::raw_sql(migration_002).execute(&pool).await.unwrap();

    // Enable foreign keys for SQLite
    sqlx::raw_sql("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .unwrap();

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
