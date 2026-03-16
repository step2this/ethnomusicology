use sqlx::PgPool;

use super::models::UpsertResult;

pub async fn upsert_artist(
    pool: &PgPool,
    id: &str,
    name: &str,
    spotify_uri: &str,
) -> Result<UpsertResult, sqlx::Error> {
    let existing = sqlx::query_scalar::<_, String>("SELECT id FROM artists WHERE spotify_uri = $1")
        .bind(spotify_uri)
        .fetch_optional(pool)
        .await?;

    sqlx::query(
        "INSERT INTO artists (id, name, spotify_uri, updated_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT(spotify_uri) DO UPDATE SET
           name = excluded.name,
           updated_at = NOW()",
    )
    .bind(id)
    .bind(name)
    .bind(spotify_uri)
    .execute(pool)
    .await?;

    Ok(if existing.is_some() {
        UpsertResult::Updated
    } else {
        UpsertResult::Inserted
    })
}

pub async fn upsert_track_artist(
    pool: &PgPool,
    track_id: &str,
    artist_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO track_artists (track_id, artist_id)
         VALUES ($1, $2)
         ON CONFLICT(track_id, artist_id) DO NOTHING",
    )
    .bind(track_id)
    .bind(artist_id)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_test_pool;
    use crate::db::tracks::upsert_track;

    #[tokio::test]
    async fn test_upsert_track_artist() {
        let pool = create_test_pool().await;

        // Create a track and artist first
        upsert_track(
            &pool,
            "t1",
            "Track",
            None,
            None,
            "spotify:track:t1",
            None,
            None,
        )
        .await
        .unwrap();
        upsert_artist(&pool, "a1", "Artist", "spotify:artist:a1")
            .await
            .unwrap();

        // Link them
        upsert_track_artist(&pool, "t1", "a1").await.unwrap();

        // Verify the link exists
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM track_artists WHERE track_id = $1 AND artist_id = $2",
        )
        .bind("t1")
        .bind("a1")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1);

        // Upserting again should not fail (ON CONFLICT DO NOTHING)
        upsert_track_artist(&pool, "t1", "a1").await.unwrap();
    }
}
