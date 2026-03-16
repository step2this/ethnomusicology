use sqlx::PgPool;

use super::models::{SpotifyImport, TrackRow};

pub async fn create_import(
    pool: &PgPool,
    id: &str,
    user_id: &str,
    playlist_id: &str,
    playlist_name: Option<&str>,
) -> Result<SpotifyImport, sqlx::Error> {
    sqlx::query(
        "INSERT INTO spotify_imports (id, user_id, spotify_playlist_id, spotify_playlist_name, status)
         VALUES ($1, $2, $3, $4, 'in_progress')",
    )
    .bind(id)
    .bind(user_id)
    .bind(playlist_id)
    .bind(playlist_name)
    .execute(pool)
    .await?;

    get_import(pool, id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn update_import_counts(
    pool: &PgPool,
    id: &str,
    found: i32,
    inserted: i32,
    updated: i32,
    failed: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE spotify_imports
         SET tracks_found = $1, tracks_inserted = $2, tracks_updated = $3, tracks_failed = $4
         WHERE id = $5",
    )
    .bind(found)
    .bind(inserted)
    .bind(updated)
    .bind(failed)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn complete_import(
    pool: &PgPool,
    id: &str,
    status: &str,
    error_msg: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE spotify_imports
         SET status = $1, error_message = $2, completed_at = NOW()
         WHERE id = $3",
    )
    .bind(status)
    .bind(error_msg)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_import(pool: &PgPool, id: &str) -> Result<Option<SpotifyImport>, sqlx::Error> {
    sqlx::query_as::<_, SpotifyImport>("SELECT * FROM spotify_imports WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

// ---------------------------------------------------------------------------
// Import-track linkage operations
// ---------------------------------------------------------------------------

pub async fn insert_import_tracks(
    pool: &PgPool,
    import_id: &str,
    track_ids: &[String],
) -> Result<(), sqlx::Error> {
    for track_id in track_ids {
        sqlx::query("INSERT INTO import_tracks (import_id, track_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(import_id)
            .bind(track_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn insert_import_track(
    pool: &PgPool,
    import_id: &str,
    track_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO import_tracks (import_id, track_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(import_id)
        .bind(track_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_tracks_by_import_id(
    pool: &PgPool,
    import_id: &str,
) -> Result<Vec<TrackRow>, sqlx::Error> {
    sqlx::query_as::<_, TrackRow>(
        r#"SELECT
            t.id, t.title, STRING_AGG(a.name, ', ') AS artist,
            t.album, t.duration_ms, t.bpm, t.camelot_key, t.energy,
            t.source, t.spotify_uri, t.spotify_preview_url, t.album_art_url,
            t.deezer_id, t.deezer_preview_url, t.created_at
        FROM import_tracks it
        INNER JOIN tracks t ON it.track_id = t.id
        LEFT JOIN track_artists ta ON t.id = ta.track_id
        LEFT JOIN artists a ON ta.artist_id = a.id
        WHERE it.import_id = $1
        GROUP BY t.id
        ORDER BY t.title ASC"#,
    )
    .bind(import_id)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_test_pool, create_test_user};

    #[tokio::test]
    async fn test_get_import_not_found() {
        let pool = create_test_pool().await;

        let result = get_import(&pool, "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_insert_import_track_duplicate_ignored() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        create_import(&pool, "imp-dup", &user_id, "pl3", None)
            .await
            .unwrap();

        sqlx::query("INSERT INTO tracks (id, title, source) VALUES ($1, $2, $3)")
            .bind("t4")
            .bind("Track Four")
            .bind("spotify")
            .execute(&pool)
            .await
            .unwrap();

        // Insert same link twice — should not error
        insert_import_track(&pool, "imp-dup", "t4").await.unwrap();
        insert_import_track(&pool, "imp-dup", "t4").await.unwrap();

        let tracks = get_tracks_by_import_id(&pool, "imp-dup").await.unwrap();
        assert_eq!(tracks.len(), 1);
    }

}
