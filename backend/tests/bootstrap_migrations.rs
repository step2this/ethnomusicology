//! One-time helper: verifies that all migrations run cleanly against Postgres.
//!
//! Run with: DATABASE_URL=... cargo test --test bootstrap_migrations -- --nocapture

#[tokio::test]
async fn test_migrations_run_cleanly() {
    let pool = ethnomusicology_backend::db::create_test_pool().await;

    let rows =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true")
            .fetch_one(&pool)
            .await
            .unwrap();

    println!("Successfully applied {} migrations", rows.0);
    assert!(
        rows.0 >= 11,
        "Expected at least 11 migrations, got {}",
        rows.0
    );
    pool.close().await;
}
