use sqlx::PgPool;

use super::models::{Track, TrackRow, UpsertResult};

#[allow(clippy::too_many_arguments)]
pub async fn upsert_track(
    pool: &PgPool,
    id: &str,
    title: &str,
    album: Option<&str>,
    duration_ms: Option<i64>,
    spotify_uri: &str,
    preview_url: Option<&str>,
    album_art_url: Option<&str>,
) -> Result<UpsertResult, sqlx::Error> {
    // Check if a track with this spotify_uri already exists
    let existing = sqlx::query_scalar::<_, String>("SELECT id FROM tracks WHERE spotify_uri = $1")
        .bind(spotify_uri)
        .fetch_optional(pool)
        .await?;

    sqlx::query(
        "INSERT INTO tracks (id, title, album, duration_ms, spotify_uri, spotify_preview_url, album_art_url, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
         ON CONFLICT(spotify_uri) DO UPDATE SET
           title = excluded.title,
           album = excluded.album,
           duration_ms = excluded.duration_ms,
           spotify_preview_url = excluded.spotify_preview_url,
           album_art_url = excluded.album_art_url,
           updated_at = NOW()",
    )
    .bind(id)
    .bind(title)
    .bind(album)
    .bind(duration_ms)
    .bind(spotify_uri)
    .bind(preview_url)
    .bind(album_art_url)
    .execute(pool)
    .await?;

    Ok(if existing.is_some() {
        UpsertResult::Updated
    } else {
        UpsertResult::Inserted
    })
}

pub async fn list_tracks_paginated(
    pool: &PgPool,
    page: u32,
    per_page: u32,
    sort: &str,
    order: &str,
) -> Result<(Vec<TrackRow>, i64), sqlx::Error> {
    // Count total tracks
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tracks")
        .fetch_one(pool)
        .await?;

    // Map validated sort param to SQL column
    let sort_col = match sort {
        "title" => "t.title",
        "artist" => "artist",
        "bpm" => "t.bpm",
        "key" => "t.camelot_key",
        _ => "t.created_at",
    };

    let order_dir = if order == "asc" { "ASC" } else { "DESC" };

    // Build query with NULLS LAST for both directions (populated values first)
    let query = format!(
        "SELECT
            t.id,
            t.title,
            STRING_AGG(a.name, ', ') AS artist,
            t.album,
            t.duration_ms,
            t.bpm,
            t.camelot_key,
            t.energy,
            t.source,
            t.spotify_uri,
            t.spotify_preview_url,
            t.album_art_url,
            t.deezer_id,
            t.deezer_preview_url,
            t.created_at
        FROM tracks t
        LEFT JOIN track_artists ta ON t.id = ta.track_id
        LEFT JOIN artists a ON ta.artist_id = a.id
        GROUP BY t.id
        ORDER BY ({sort_col} IS NULL), {sort_col} {order_dir}, t.id ASC
        LIMIT $1 OFFSET $2",
        sort_col = sort_col,
        order_dir = order_dir,
    );

    let offset = ((page - 1) * per_page) as i64;

    let rows = sqlx::query_as::<_, TrackRow>(&query)
        .bind(per_page as i64)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((rows, total))
}

pub async fn get_track_by_spotify_uri(
    pool: &PgPool,
    uri: &str,
) -> Result<Option<Track>, sqlx::Error> {
    sqlx::query_as::<_, Track>(
        "SELECT id, title, album, duration_ms, spotify_uri, spotify_preview_url, youtube_id, musicbrainz_id, created_at, updated_at FROM tracks WHERE spotify_uri = $1",
    )
    .bind(uri)
    .fetch_optional(pool)
    .await
}

/// Get tracks that need enrichment (needs_enrichment = TRUE, no enrichment_error).
pub async fn get_unenriched_tracks(
    pool: &PgPool,
    limit: usize,
) -> Result<Vec<TrackRow>, sqlx::Error> {
    sqlx::query_as::<_, TrackRow>(
        r#"SELECT
            t.id, t.title, STRING_AGG(a.name, ', ') AS artist,
            t.album, t.duration_ms, t.bpm, t.camelot_key, t.energy,
            t.source, t.spotify_uri, t.spotify_preview_url, t.album_art_url,
            t.deezer_id, t.deezer_preview_url, t.created_at
        FROM tracks t
        LEFT JOIN track_artists ta ON t.id = ta.track_id
        LEFT JOIN artists a ON ta.artist_id = a.id
        WHERE t.needs_enrichment = TRUE AND t.enrichment_error IS NULL
        GROUP BY t.id
        ORDER BY t.created_at DESC
        LIMIT $1"#,
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await
}

/// Update a track's DJ metadata after enrichment.
pub async fn update_track_dj_metadata(
    pool: &PgPool,
    id: &str,
    bpm: Option<f64>,
    camelot_key: Option<&str>,
    energy: Option<f64>,
    album_art_url: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE tracks SET bpm = COALESCE($1, bpm), camelot_key = COALESCE($2, camelot_key), energy = COALESCE($3, energy), album_art_url = COALESCE($4, album_art_url), needs_enrichment = FALSE, enriched_at = NOW() WHERE id = $5",
    )
    .bind(bpm)
    .bind(camelot_key)
    .bind(energy)
    .bind(album_art_url)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark a track's enrichment as failed.
pub async fn mark_enrichment_error(
    pool: &PgPool,
    id: &str,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tracks SET enrichment_error = $1, needs_enrichment = FALSE WHERE id = $2")
        .bind(error)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get today's enrichment count for a user.
pub async fn get_daily_enrichment_count(pool: &PgPool, user_id: &str) -> Result<i64, sqlx::Error> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let count: Option<i32> = sqlx::query_scalar(
        "SELECT enrichment_count FROM user_usage WHERE user_id = $1 AND date = $2",
    )
    .bind(user_id)
    .bind(&today)
    .fetch_optional(pool)
    .await?;
    Ok(count.unwrap_or(0) as i64)
}

/// Reset errored tracks so they can be re-enriched.
/// Clears enrichment_error and sets needs_enrichment = TRUE for all tracks
/// where enrichment_error IS NOT NULL.
/// Returns the number of tracks reset.
pub async fn retry_errored_tracks(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE tracks SET enrichment_error = NULL, needs_enrichment = TRUE WHERE enrichment_error IS NOT NULL",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Get today's generation count for a user.
pub async fn get_daily_generation_count(pool: &PgPool, user_id: &str) -> Result<i64, sqlx::Error> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let count: Option<i32> = sqlx::query_scalar(
        "SELECT generation_count FROM user_usage WHERE user_id = $1 AND date = $2",
    )
    .bind(user_id)
    .bind(&today)
    .fetch_optional(pool)
    .await?;
    Ok(count.unwrap_or(0) as i64)
}

/// Increment the generation usage counter for a user.
pub async fn increment_generation_usage(pool: &PgPool, user_id: &str) -> Result<(), sqlx::Error> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO user_usage (id, user_id, date, generation_count) VALUES ($1, $2, $3, 1)
         ON CONFLICT(user_id, date) DO UPDATE SET generation_count = user_usage.generation_count + 1",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&today)
    .execute(pool)
    .await?;
    Ok(())
}

/// Increment the enrichment usage counter for a user.
pub async fn increment_enrichment_usage(
    pool: &PgPool,
    user_id: &str,
    count: i64,
) -> Result<(), sqlx::Error> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO user_usage (id, user_id, date, enrichment_count) VALUES ($1, $2, $3, $4)
         ON CONFLICT(user_id, date) DO UPDATE SET enrichment_count = user_usage.enrichment_count + excluded.enrichment_count",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&today)
    .bind(count)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_test_pool;

    #[tokio::test]
    async fn test_get_track_not_found() {
        let pool = create_test_pool().await;

        let result = get_track_by_spotify_uri(&pool, "spotify:track:nonexistent")
            .await
            .unwrap();
        assert!(result.is_none());
        pool.close().await;
    }

    #[tokio::test]
    async fn test_list_tracks_paginated_empty() {
        let pool = create_test_pool().await;
        let (tracks, total) = list_tracks_paginated(&pool, 1, 25, "date_added", "desc")
            .await
            .unwrap();
        assert!(tracks.is_empty());
        assert_eq!(total, 0);
        pool.close().await;
    }

    #[tokio::test]
    async fn test_list_tracks_paginated_basic() {
        let pool = create_test_pool().await;

        // Insert test tracks
        sqlx::query("INSERT INTO tracks (id, title, album, duration_ms, spotify_uri, source) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind("t1").bind("Track A").bind("Album 1").bind(180000i64).bind("spotify:track:a1").bind("spotify")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO tracks (id, title, album, duration_ms, spotify_uri, source) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind("t2").bind("Track B").bind("Album 2").bind(200000i64).bind("spotify:track:b2").bind("spotify")
            .execute(&pool).await.unwrap();

        // Insert artists and link
        sqlx::query("INSERT INTO artists (id, name) VALUES ($1, $2)")
            .bind("a1")
            .bind("Artist One")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ($1, $2)")
            .bind("t1")
            .bind("a1")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ($1, $2)")
            .bind("t2")
            .bind("a1")
            .execute(&pool)
            .await
            .unwrap();

        let (tracks, total) = list_tracks_paginated(&pool, 1, 25, "date_added", "desc")
            .await
            .unwrap();
        assert_eq!(total, 2);
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].artist.as_deref(), Some("Artist One"));
        pool.close().await;
    }

    #[tokio::test]
    async fn test_list_tracks_paginated_pagination() {
        let pool = create_test_pool().await;

        for i in 0..5 {
            sqlx::query("INSERT INTO tracks (id, title, source) VALUES ($1, $2, 'spotify')")
                .bind(format!("t{}", i))
                .bind(format!("Track {}", i))
                .execute(&pool)
                .await
                .unwrap();
        }

        let (page1, total) = list_tracks_paginated(&pool, 1, 2, "title", "asc")
            .await
            .unwrap();
        assert_eq!(total, 5);
        assert_eq!(page1.len(), 2);

        let (page3, _) = list_tracks_paginated(&pool, 3, 2, "title", "asc")
            .await
            .unwrap();
        assert_eq!(page3.len(), 1);
        pool.close().await;
    }

    #[tokio::test]
    async fn test_list_tracks_paginated_sort_bpm_nulls() {
        let pool = create_test_pool().await;

        sqlx::query("INSERT INTO tracks (id, title, bpm, source) VALUES ($1, $2, $3, 'spotify')")
            .bind("t1")
            .bind("Fast")
            .bind(128.0f64)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO tracks (id, title, bpm, source) VALUES ($1, $2, $3, 'spotify')")
            .bind("t2")
            .bind("Slow")
            .bind(100.0f64)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO tracks (id, title, source) VALUES ($1, $2, 'spotify')")
            .bind("t3")
            .bind("No BPM")
            .execute(&pool)
            .await
            .unwrap();

        let (tracks, _) = list_tracks_paginated(&pool, 1, 25, "bpm", "asc")
            .await
            .unwrap();
        assert_eq!(tracks.len(), 3);
        // Populated values first (ASC), NULLs last
        assert_eq!(tracks[0].bpm, Some(100.0));
        assert_eq!(tracks[1].bpm, Some(128.0));
        assert!(tracks[2].bpm.is_none());
        pool.close().await;
    }

    #[tokio::test]
    async fn test_list_tracks_paginated_multi_artist() {
        let pool = create_test_pool().await;

        sqlx::query("INSERT INTO tracks (id, title, source) VALUES ($1, $2, 'spotify')")
            .bind("t1")
            .bind("Collab Track")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO artists (id, name) VALUES ($1, $2)")
            .bind("a1")
            .bind("Artist A")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO artists (id, name) VALUES ($1, $2)")
            .bind("a2")
            .bind("Artist B")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ($1, $2)")
            .bind("t1")
            .bind("a1")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ($1, $2)")
            .bind("t1")
            .bind("a2")
            .execute(&pool)
            .await
            .unwrap();

        let (tracks, _) = list_tracks_paginated(&pool, 1, 25, "title", "asc")
            .await
            .unwrap();
        assert_eq!(tracks.len(), 1);
        let artist = tracks[0].artist.as_deref().unwrap();
        assert!(artist.contains("Artist A"));
        assert!(artist.contains("Artist B"));
        pool.close().await;
    }
}
