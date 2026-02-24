use sqlx::SqlitePool;

use super::models::{Track, UpsertResult};

pub async fn upsert_track(
    pool: &SqlitePool,
    id: &str,
    title: &str,
    album: Option<&str>,
    duration_ms: Option<i64>,
    spotify_uri: &str,
    preview_url: Option<&str>,
) -> Result<UpsertResult, sqlx::Error> {
    // Check if a track with this spotify_uri already exists
    let existing = sqlx::query_scalar::<_, String>("SELECT id FROM tracks WHERE spotify_uri = ?")
        .bind(spotify_uri)
        .fetch_optional(pool)
        .await?;

    sqlx::query(
        "INSERT INTO tracks (id, title, album, duration_ms, spotify_uri, spotify_preview_url, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(spotify_uri) DO UPDATE SET
           title = excluded.title,
           album = excluded.album,
           duration_ms = excluded.duration_ms,
           spotify_preview_url = excluded.spotify_preview_url,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(id)
    .bind(title)
    .bind(album)
    .bind(duration_ms)
    .bind(spotify_uri)
    .bind(preview_url)
    .execute(pool)
    .await?;

    Ok(if existing.is_some() {
        UpsertResult::Updated
    } else {
        UpsertResult::Inserted
    })
}

pub async fn get_track_by_spotify_uri(
    pool: &SqlitePool,
    uri: &str,
) -> Result<Option<Track>, sqlx::Error> {
    sqlx::query_as::<_, Track>("SELECT * FROM tracks WHERE spotify_uri = ?")
        .bind(uri)
        .fetch_optional(pool)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_test_pool;

    #[tokio::test]
    async fn test_upsert_track_insert() {
        let pool = create_test_pool().await;

        let result = upsert_track(
            &pool,
            "t1",
            "Test Track",
            Some("Test Album"),
            Some(210_000),
            "spotify:track:abc123",
            Some("https://preview.url"),
        )
        .await
        .unwrap();

        assert_eq!(result, UpsertResult::Inserted);

        let track = get_track_by_spotify_uri(&pool, "spotify:track:abc123")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(track.title, "Test Track");
        assert_eq!(track.album.as_deref(), Some("Test Album"));
        assert_eq!(track.duration_ms, Some(210_000));
    }

    #[tokio::test]
    async fn test_upsert_track_update() {
        let pool = create_test_pool().await;

        upsert_track(
            &pool,
            "t1",
            "Original Title",
            Some("Album"),
            Some(200_000),
            "spotify:track:abc123",
            None,
        )
        .await
        .unwrap();

        let result = upsert_track(
            &pool,
            "t1-new",
            "Updated Title",
            Some("New Album"),
            Some(220_000),
            "spotify:track:abc123",
            Some("https://new-preview.url"),
        )
        .await
        .unwrap();

        assert_eq!(result, UpsertResult::Updated);

        let track = get_track_by_spotify_uri(&pool, "spotify:track:abc123")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(track.title, "Updated Title");
        assert_eq!(track.album.as_deref(), Some("New Album"));
    }

    #[tokio::test]
    async fn test_get_track_not_found() {
        let pool = create_test_pool().await;

        let result = get_track_by_spotify_uri(&pool, "spotify:track:nonexistent")
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
