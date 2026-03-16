use sqlx::PgPool;

use crate::db::models::{SetlistRow, SetlistSummary, SetlistTrackRow, TrackRow};

// ---------------------------------------------------------------------------
// Insert operations
// ---------------------------------------------------------------------------

pub async fn insert_setlist(pool: &PgPool, row: &SetlistRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO setlists (id, user_id, prompt, model, name, notes, harmonic_flow_score, energy_profile) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&row.id)
    .bind(&row.user_id)
    .bind(&row.prompt)
    .bind(&row.model)
    .bind(&row.name)
    .bind(&row.notes)
    .bind(row.harmonic_flow_score)
    .bind(&row.energy_profile)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_setlist_track(pool: &PgPool, row: &SetlistTrackRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO setlist_tracks (id, setlist_id, track_id, position, original_position, title, artist, bpm, key, camelot, energy, transition_note, transition_score, source, acquisition_info, confidence, verification_flag, verification_note) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"
    )
    .bind(&row.id)
    .bind(&row.setlist_id)
    .bind(&row.track_id)
    .bind(row.position)
    .bind(row.original_position)
    .bind(&row.title)
    .bind(&row.artist)
    .bind(row.bpm)
    .bind(&row.key)
    .bind(&row.camelot)
    .bind(row.energy)
    .bind(&row.transition_note)
    .bind(row.transition_score)
    .bind(&row.source)
    .bind(&row.acquisition_info)
    .bind(&row.confidence)
    .bind(&row.verification_flag)
    .bind(&row.verification_note)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Read operations
// ---------------------------------------------------------------------------

pub async fn get_setlist(pool: &PgPool, id: &str) -> Result<Option<SetlistRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistRow>(
        "SELECT id, user_id, prompt, model, name, notes, harmonic_flow_score, energy_profile, created_at FROM setlists WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_setlist_tracks(
    pool: &PgPool,
    setlist_id: &str,
) -> Result<Vec<SetlistTrackRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistTrackRow>(
        "SELECT st.id, st.setlist_id, st.track_id, st.position, st.original_position, \
         st.title, st.artist, st.bpm, st.key, st.camelot, st.energy, \
         st.transition_note, st.transition_score, st.source, st.acquisition_info, \
         t.spotify_uri, st.confidence, st.verification_flag, st.verification_note \
         FROM setlist_tracks st LEFT JOIN tracks t ON st.track_id = t.id \
         WHERE st.setlist_id = $1 ORDER BY st.position ASC",
    )
    .bind(setlist_id)
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------
// Update operations
// ---------------------------------------------------------------------------

pub async fn update_setlist_harmonic_score(
    pool: &PgPool,
    id: &str,
    score: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE setlists SET harmonic_flow_score = $1 WHERE id = $2")
        .bind(score)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_setlist_track_position(
    pool: &PgPool,
    track_id: &str,
    new_position: i32,
    transition_score: Option<f64>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE setlist_tracks SET position = $1, transition_score = $2 WHERE id = $3")
        .bind(new_position)
        .bind(transition_score)
        .bind(track_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// List / count / delete / rename / duplicate
// ---------------------------------------------------------------------------

pub async fn list_setlists(
    pool: &PgPool,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<SetlistSummary>, sqlx::Error> {
    sqlx::query_as::<_, SetlistSummary>(
        "SELECT s.id, s.name, s.prompt, s.created_at, \
         COUNT(st.id) as track_count \
         FROM setlists s \
         LEFT JOIN setlist_tracks st ON st.setlist_id = s.id \
         WHERE s.user_id = $1 \
         GROUP BY s.id \
         ORDER BY s.created_at DESC \
         LIMIT $2 OFFSET $3",
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_setlists(pool: &PgPool, user_id: &str) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM setlists WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn delete_setlist(pool: &PgPool, id: &str) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM setlist_conversations WHERE setlist_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "DELETE FROM setlist_version_tracks WHERE version_id IN \
         (SELECT id FROM setlist_versions WHERE setlist_id = $1)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM setlist_versions WHERE setlist_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM setlist_tracks WHERE setlist_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let result = sqlx::query("DELETE FROM setlists WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_setlist_name(pool: &PgPool, id: &str, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE setlists SET name = $1 WHERE id = $2")
        .bind(name)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn duplicate_setlist(
    pool: &PgPool,
    id: &str,
    new_name: Option<&str>,
) -> Result<Option<String>, sqlx::Error> {
    // Load original
    let original = sqlx::query_as::<_, SetlistRow>(
        "SELECT id, user_id, prompt, model, name, notes, harmonic_flow_score, energy_profile, created_at FROM setlists WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(orig) = original else {
        return Ok(None);
    };

    let new_id = uuid::Uuid::new_v4().to_string();
    let resolved_name = new_name
        .map(|s| s.to_string())
        .or_else(|| orig.name.as_ref().map(|n| format!("{n} (copy)")))
        .or_else(|| {
            Some(format!(
                "{} (copy)",
                &orig.prompt[..orig.prompt.len().min(40)]
            ))
        });

    // Load tracks before starting the transaction (read-only)
    let tracks = sqlx::query_as::<_, SetlistTrackRow>(
        "SELECT st.id, st.setlist_id, st.track_id, st.position, st.original_position, \
         st.title, st.artist, st.bpm, st.key, st.camelot, st.energy, \
         st.transition_note, st.transition_score, st.source, st.acquisition_info, \
         t.spotify_uri, st.confidence, st.verification_flag, st.verification_note \
         FROM setlist_tracks st LEFT JOIN tracks t ON st.track_id = t.id \
         WHERE st.setlist_id = $1 ORDER BY st.position ASC",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    // Wrap all writes in a transaction
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO setlists (id, user_id, prompt, model, name, notes, harmonic_flow_score, energy_profile) \
         SELECT $1, user_id, prompt, model, $2, notes, harmonic_flow_score, energy_profile FROM setlists WHERE id = $3",
    )
    .bind(&new_id)
    .bind(&resolved_name)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    for track in tracks {
        let new_track_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO setlist_tracks \
             (id, setlist_id, track_id, position, original_position, title, artist, bpm, key, camelot, \
             energy, transition_note, transition_score, source, acquisition_info, confidence, \
             verification_flag, verification_note) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
        )
        .bind(&new_track_id)
        .bind(&new_id)
        .bind(&track.track_id)
        .bind(track.position)
        .bind(track.original_position)
        .bind(&track.title)
        .bind(&track.artist)
        .bind(track.bpm)
        .bind(&track.key)
        .bind(&track.camelot)
        .bind(track.energy)
        .bind(&track.transition_note)
        .bind(track.transition_score)
        .bind(&track.source)
        .bind(&track.acquisition_info)
        .bind(&track.confidence)
        .bind(&track.verification_flag)
        .bind(&track.verification_note)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Some(new_id))
}

// ---------------------------------------------------------------------------
// Catalog loading (all tracks, no user_id filter for ST-003)
// ---------------------------------------------------------------------------

pub async fn load_catalog_tracks(pool: &PgPool) -> Result<Vec<TrackRow>, sqlx::Error> {
    sqlx::query_as::<_, TrackRow>(
        r#"SELECT
            t.id, t.title, STRING_AGG(a.name, ', ') AS artist,
            t.album, t.duration_ms, t.bpm, t.camelot_key, t.energy,
            t.source, t.spotify_uri, t.spotify_preview_url, t.album_art_url,
            t.deezer_id, t.deezer_preview_url, t.created_at
        FROM tracks t
        LEFT JOIN track_artists ta ON t.id = ta.track_id
        LEFT JOIN artists a ON ta.artist_id = a.id
        GROUP BY t.id
        ORDER BY t.title ASC"#,
    )
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::SetlistRow;

    #[tokio::test]
    async fn test_get_setlist_not_found() {
        let pool = crate::db::create_test_pool().await;
        let result = get_setlist(&pool, "nonexistent").await.unwrap();
        assert!(result.is_none());
        pool.close().await;
    }

    #[tokio::test]
    async fn test_load_catalog_tracks() {
        let pool = crate::db::create_test_pool().await;

        sqlx::query(
            "INSERT INTO tracks (id, title, source, bpm, camelot_key) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind("t1")
        .bind("Track One")
        .bind("spotify")
        .bind(128.0)
        .bind("8A")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO artists (id, name) VALUES ($1, $2)")
            .bind("a1")
            .bind("DJ Test")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ($1, $2)")
            .bind("t1")
            .bind("a1")
            .execute(&pool)
            .await
            .unwrap();

        let tracks = load_catalog_tracks(&pool).await.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Track One");
        assert_eq!(tracks[0].artist.as_deref(), Some("DJ Test"));
        assert_eq!(tracks[0].bpm, Some(128.0));
        assert_eq!(tracks[0].camelot_key.as_deref(), Some("8A"));
        pool.close().await;
    }

    #[tokio::test]
    async fn test_list_and_count_setlists() {
        let pool = crate::db::create_test_pool().await;

        for i in 1..=3u32 {
            let row = SetlistRow {
                id: format!("sl-{i}"),
                user_id: "user-1".to_string(),
                prompt: format!("Prompt {i}"),
                model: "test".to_string(),
                name: if i == 1 {
                    Some("Named Set".to_string())
                } else {
                    None
                },
                notes: None,
                harmonic_flow_score: None,
                energy_profile: None,
                created_at: None,
            };
            insert_setlist(&pool, &row).await.unwrap();
        }

        let count = count_setlists(&pool, "user-1").await.unwrap();
        assert_eq!(count, 3);

        let list = list_setlists(&pool, "user-1", 10, 0).await.unwrap();
        assert_eq!(list.len(), 3);

        // user-2 should see nothing
        let count2 = count_setlists(&pool, "user-2").await.unwrap();
        assert_eq!(count2, 0);
        pool.close().await;
    }

    #[tokio::test]
    async fn test_delete_setlist() {
        let pool = crate::db::create_test_pool().await;

        let row = SetlistRow {
            id: "sl-del".to_string(),
            user_id: "user-1".to_string(),
            prompt: "To be deleted".to_string(),
            model: "test".to_string(),
            name: None,
            notes: None,
            harmonic_flow_score: None,
            energy_profile: None,
            created_at: None,
        };
        insert_setlist(&pool, &row).await.unwrap();

        let deleted = delete_setlist(&pool, "sl-del").await.unwrap();
        assert!(deleted);

        let fetched = get_setlist(&pool, "sl-del").await.unwrap();
        assert!(fetched.is_none());

        // Deleting non-existent returns false
        let deleted2 = delete_setlist(&pool, "sl-del").await.unwrap();
        assert!(!deleted2);
        pool.close().await;
    }

    #[tokio::test]
    async fn test_duplicate_setlist() {
        let pool = crate::db::create_test_pool().await;

        let row = SetlistRow {
            id: "sl-orig".to_string(),
            user_id: "user-1".to_string(),
            prompt: "Original prompt".to_string(),
            model: "test".to_string(),
            name: Some("Original".to_string()),
            notes: None,
            harmonic_flow_score: None,
            energy_profile: None,
            created_at: None,
        };
        insert_setlist(&pool, &row).await.unwrap();

        // Add a track
        let track = SetlistTrackRow {
            id: "st-orig-1".to_string(),
            setlist_id: "sl-orig".to_string(),
            track_id: None,
            position: 1,
            original_position: 1,
            title: "Track A".to_string(),
            artist: "Artist X".to_string(),
            bpm: Some(128.0),
            key: None,
            camelot: None,
            energy: None,
            transition_note: None,
            transition_score: None,
            source: "suggestion".to_string(),
            acquisition_info: None,
            spotify_uri: None,
            confidence: None,
            verification_flag: None,
            verification_note: None,
        };
        insert_setlist_track(&pool, &track).await.unwrap();

        let new_id = duplicate_setlist(&pool, "sl-orig", None)
            .await
            .unwrap()
            .unwrap();

        assert_ne!(new_id, "sl-orig");

        let dup = get_setlist(&pool, &new_id).await.unwrap().unwrap();
        assert_eq!(dup.prompt, "Original prompt");
        assert_eq!(dup.name.as_deref(), Some("Original (copy)"));

        let dup_tracks = get_setlist_tracks(&pool, &new_id).await.unwrap();
        assert_eq!(dup_tracks.len(), 1);
        assert_eq!(dup_tracks[0].title, "Track A");
        // IDs must differ
        assert_ne!(dup_tracks[0].id, "st-orig-1");
        pool.close().await;
    }
}
