//! One-time helper: extracts `_sqlx_migrations` rows from a fresh DB
//! so they can be inserted into an existing production DB that was
//! migrated manually (no tracking table).
//!
//! Run with: cargo test --test bootstrap_migrations -- --nocapture
//! Pipe to sqlite3: cargo test --test bootstrap_migrations -- --nocapture 2>/dev/null | sqlite3 /path/to/prod.db

#[tokio::test]
async fn print_bootstrap_sql() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let rows = sqlx::query_as::<_, (i64, String, String, bool, Vec<u8>, i64)>(
        "SELECT version, description, installed_on, success, checksum, execution_time FROM _sqlx_migrations ORDER BY version"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    println!("-- Bootstrap: mark existing migrations as applied");
    println!("CREATE TABLE IF NOT EXISTS _sqlx_migrations (");
    println!("    version BIGINT PRIMARY KEY,");
    println!("    description TEXT NOT NULL,");
    println!("    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,");
    println!("    success BOOLEAN NOT NULL,");
    println!("    checksum BLOB NOT NULL,");
    println!("    execution_time BIGINT NOT NULL DEFAULT 0");
    println!(");");
    for (version, description, _installed_on, success, checksum, execution_time) in &rows {
        let hex = checksum
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        println!(
            "INSERT OR IGNORE INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) VALUES ({}, '{}', datetime('now'), {}, X'{}', {});",
            version, description, success, hex, execution_time
        );
    }
}
