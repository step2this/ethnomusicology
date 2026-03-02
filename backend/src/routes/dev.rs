use axum::{extract::State, routing::post, Json, Router};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Serialize)]
struct SeedResponse {
    seeded: bool,
    tracks: usize,
    message: String,
}

async fn seed_data(
    State(pool): State<SqlitePool>,
) -> Result<Json<SeedResponse>, axum::http::StatusCode> {
    // Insert artists first, then tracks, then associations
    // Use INSERT OR IGNORE for idempotency

    // Artists (4 artists)
    sqlx::raw_sql(
        "INSERT OR IGNORE INTO artists (id, name) VALUES \
         ('seed-artist-1', 'Amr Diab'), \
         ('seed-artist-2', 'Hassan Hakmoun'), \
         ('seed-artist-3', 'Charlotte de Witte'), \
         ('seed-artist-4', 'Black Coffee')",
    )
    .execute(&pool)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // 8 tracks with DJ metadata
    sqlx::raw_sql(
        "INSERT OR IGNORE INTO tracks (id, title, album, duration_ms, bpm, camelot_key, energy, source, spotify_uri) VALUES \
         ('seed-track-1', 'Nour El Ain', 'Best Of', 240000, 128.0, '8A', 6.0, 'spotify', 'spotify:track:seed1'), \
         ('seed-track-2', 'Habibi Ya Nour', 'Classics', 210000, 124.0, '7A', 5.0, 'spotify', 'spotify:track:seed2'), \
         ('seed-track-3', 'Gnawa Blues', 'World Fusion', 195000, 118.0, '9A', 3.0, 'spotify', 'spotify:track:seed3'), \
         ('seed-track-4', 'Fez Medina', 'Moroccan Nights', 220000, 122.0, '8B', 4.0, 'spotify', 'spotify:track:seed4'), \
         ('seed-track-5', 'Acid Techno Rise', 'Rave On', 360000, 138.0, '1A', 8.0, 'spotify', 'spotify:track:seed5'), \
         ('seed-track-6', 'Pipeline Dub', 'Selections', 300000, 140.0, '3B', 7.0, 'spotify', 'spotify:track:seed6'), \
         ('seed-track-7', 'Wish You Were Here', 'Subconsciously', 270000, 120.0, '8A', 5.0, 'spotify', 'spotify:track:seed7'), \
         ('seed-track-8', 'Drive', 'Subconsciously', 285000, 122.0, '9A', 6.0, 'spotify', 'spotify:track:seed8')",
    )
    .execute(&pool)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Track-Artist associations (each artist has 2 tracks)
    sqlx::raw_sql(
        "INSERT OR IGNORE INTO track_artists (track_id, artist_id) VALUES \
         ('seed-track-1', 'seed-artist-1'), \
         ('seed-track-2', 'seed-artist-1'), \
         ('seed-track-3', 'seed-artist-2'), \
         ('seed-track-4', 'seed-artist-2'), \
         ('seed-track-5', 'seed-artist-3'), \
         ('seed-track-6', 'seed-artist-3'), \
         ('seed-track-7', 'seed-artist-4'), \
         ('seed-track-8', 'seed-artist-4')",
    )
    .execute(&pool)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SeedResponse {
        seeded: true,
        tracks: 8,
        message: "Test data seeded successfully".to_string(),
    }))
}

pub fn dev_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/dev/seed", post(seed_data))
        .with_state(pool)
}
