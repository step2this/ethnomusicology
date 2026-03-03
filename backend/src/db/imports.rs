use sqlx::SqlitePool;

use super::models::{SpotifyImport, TrackRow};

pub async fn create_import(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    playlist_id: &str,
    playlist_name: Option<&str>,
) -> Result<SpotifyImport, sqlx::Error> {
    sqlx::query(
        "INSERT INTO spotify_imports (id, user_id, spotify_playlist_id, spotify_playlist_name, status)
         VALUES (?, ?, ?, ?, 'in_progress')",
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
    pool: &SqlitePool,
    id: &str,
    found: i32,
    inserted: i32,
    updated: i32,
    failed: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE spotify_imports
         SET tracks_found = ?, tracks_inserted = ?, tracks_updated = ?, tracks_failed = ?
         WHERE id = ?",
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
    pool: &SqlitePool,
    id: &str,
    status: &str,
    error_msg: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE spotify_imports
         SET status = ?, error_message = ?, completed_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(status)
    .bind(error_msg)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_import(pool: &SqlitePool, id: &str) -> Result<Option<SpotifyImport>, sqlx::Error> {
    sqlx::query_as::<_, SpotifyImport>("SELECT * FROM spotify_imports WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

// ---------------------------------------------------------------------------
// Import-track linkage operations
// ---------------------------------------------------------------------------

pub async fn insert_import_tracks(
    pool: &SqlitePool,
    import_id: &str,
    track_ids: &[String],
) -> Result<(), sqlx::Error> {
    for track_id in track_ids {
        sqlx::query("INSERT OR IGNORE INTO import_tracks (import_id, track_id) VALUES (?, ?)")
            .bind(import_id)
            .bind(track_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn insert_import_track(
    pool: &SqlitePool,
    import_id: &str,
    track_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO import_tracks (import_id, track_id) VALUES (?, ?)")
        .bind(import_id)
        .bind(track_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_tracks_by_import_id(
    pool: &SqlitePool,
    import_id: &str,
) -> Result<Vec<TrackRow>, sqlx::Error> {
    sqlx::query_as::<_, TrackRow>(
        r#"SELECT
            t.id, t.title, GROUP_CONCAT(a.name, ', ') AS artist,
            t.album, t.duration_ms, t.bpm, t.camelot_key, t.energy,
            t.source, t.spotify_uri, t.spotify_preview_url, t.album_art_url, t.created_at
        FROM import_tracks it
        INNER JOIN tracks t ON it.track_id = t.id
        LEFT JOIN track_artists ta ON t.id = ta.track_id
        LEFT JOIN artists a ON ta.artist_id = a.id
        WHERE it.import_id = ?
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
    async fn test_import_roundtrip() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        // Create an import
        let import = create_import(&pool, "imp1", &user_id, "playlist123", Some("My Playlist"))
            .await
            .unwrap();
        assert_eq!(import.status, "in_progress");
        assert_eq!(import.spotify_playlist_name.as_deref(), Some("My Playlist"));
        assert_eq!(import.tracks_found, 0);

        // Update counts
        update_import_counts(&pool, "imp1", 54, 50, 3, 1)
            .await
            .unwrap();

        let import = get_import(&pool, "imp1").await.unwrap().unwrap();
        assert_eq!(import.tracks_found, 54);
        assert_eq!(import.tracks_inserted, 50);
        assert_eq!(import.tracks_updated, 3);
        assert_eq!(import.tracks_failed, 1);

        // Complete
        complete_import(&pool, "imp1", "completed", None)
            .await
            .unwrap();

        let import = get_import(&pool, "imp1").await.unwrap().unwrap();
        assert_eq!(import.status, "completed");
        assert!(import.completed_at.is_some());
        assert!(import.error_message.is_none());
    }

    #[tokio::test]
    async fn test_import_with_error() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        create_import(&pool, "imp2", &user_id, "playlist456", None)
            .await
            .unwrap();

        complete_import(&pool, "imp2", "failed", Some("API rate limited"))
            .await
            .unwrap();

        let import = get_import(&pool, "imp2").await.unwrap().unwrap();
        assert_eq!(import.status, "failed");
        assert_eq!(import.error_message.as_deref(), Some("API rate limited"));
    }

    #[tokio::test]
    async fn test_get_import_not_found() {
        let pool = create_test_pool().await;

        let result = get_import(&pool, "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_insert_and_get_import_tracks() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        // Create import
        create_import(&pool, "imp-link", &user_id, "pl1", Some("Test Playlist"))
            .await
            .unwrap();

        // Insert tracks
        sqlx::query("INSERT INTO tracks (id, title, source) VALUES (?, ?, ?)")
            .bind("t1")
            .bind("Track One")
            .bind("spotify")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO tracks (id, title, source) VALUES (?, ?, ?)")
            .bind("t2")
            .bind("Track Two")
            .bind("spotify")
            .execute(&pool)
            .await
            .unwrap();

        // Link tracks to import
        insert_import_tracks(&pool, "imp-link", &["t1".to_string(), "t2".to_string()])
            .await
            .unwrap();

        // Retrieve tracks by import ID
        let tracks = get_tracks_by_import_id(&pool, "imp-link").await.unwrap();
        assert_eq!(tracks.len(), 2);

        let titles: Vec<&str> = tracks.iter().map(|t| t.title.as_str()).collect();
        assert!(titles.contains(&"Track One"));
        assert!(titles.contains(&"Track Two"));
    }

    #[tokio::test]
    async fn test_insert_import_track_single() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        create_import(&pool, "imp-single", &user_id, "pl2", None)
            .await
            .unwrap();

        sqlx::query("INSERT INTO tracks (id, title, source) VALUES (?, ?, ?)")
            .bind("t3")
            .bind("Track Three")
            .bind("spotify")
            .execute(&pool)
            .await
            .unwrap();

        insert_import_track(&pool, "imp-single", "t3")
            .await
            .unwrap();

        let tracks = get_tracks_by_import_id(&pool, "imp-single").await.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Track Three");
    }

    #[tokio::test]
    async fn test_insert_import_track_duplicate_ignored() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        create_import(&pool, "imp-dup", &user_id, "pl3", None)
            .await
            .unwrap();

        sqlx::query("INSERT INTO tracks (id, title, source) VALUES (?, ?, ?)")
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

    #[tokio::test]
    async fn test_get_tracks_by_import_id_empty() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        create_import(&pool, "imp-empty", &user_id, "pl4", None)
            .await
            .unwrap();

        let tracks = get_tracks_by_import_id(&pool, "imp-empty").await.unwrap();
        assert!(tracks.is_empty());
    }
}
