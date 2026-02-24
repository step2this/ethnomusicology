use sqlx::SqlitePool;

use super::models::SpotifyImport;

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
}
