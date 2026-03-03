use sqlx::SqlitePool;

use crate::db::models::{SetlistRow, SetlistTrackRow, TrackRow};

// ---------------------------------------------------------------------------
// Insert operations
// ---------------------------------------------------------------------------

pub async fn insert_setlist(pool: &SqlitePool, row: &SetlistRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO setlists (id, user_id, prompt, model, notes, harmonic_flow_score, energy_profile) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&row.id)
    .bind(&row.user_id)
    .bind(&row.prompt)
    .bind(&row.model)
    .bind(&row.notes)
    .bind(row.harmonic_flow_score)
    .bind(&row.energy_profile)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_setlist_track(
    pool: &SqlitePool,
    row: &SetlistTrackRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO setlist_tracks (id, setlist_id, track_id, position, original_position, title, artist, bpm, key, camelot, energy, transition_note, transition_score, source, acquisition_info) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Read operations
// ---------------------------------------------------------------------------

pub async fn get_setlist(pool: &SqlitePool, id: &str) -> Result<Option<SetlistRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistRow>(
        "SELECT id, user_id, prompt, model, notes, harmonic_flow_score, energy_profile, created_at FROM setlists WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_setlist_tracks(
    pool: &SqlitePool,
    setlist_id: &str,
) -> Result<Vec<SetlistTrackRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistTrackRow>(
        "SELECT id, setlist_id, track_id, position, original_position, title, artist, bpm, key, camelot, energy, transition_note, transition_score, source, acquisition_info FROM setlist_tracks WHERE setlist_id = ? ORDER BY position ASC",
    )
    .bind(setlist_id)
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------
// Update operations
// ---------------------------------------------------------------------------

pub async fn update_setlist_harmonic_score(
    pool: &SqlitePool,
    id: &str,
    score: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE setlists SET harmonic_flow_score = ? WHERE id = ?")
        .bind(score)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_setlist_track_position(
    pool: &SqlitePool,
    track_id: &str,
    new_position: i32,
    transition_score: Option<f64>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE setlist_tracks SET position = ?, transition_score = ? WHERE id = ?")
        .bind(new_position)
        .bind(transition_score)
        .bind(track_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Catalog loading (all tracks, no user_id filter for ST-003)
// ---------------------------------------------------------------------------

pub async fn load_catalog_tracks(pool: &SqlitePool) -> Result<Vec<TrackRow>, sqlx::Error> {
    sqlx::query_as::<_, TrackRow>(
        r#"SELECT
            t.id, t.title, GROUP_CONCAT(a.name, ', ') AS artist,
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
    async fn test_insert_and_get_setlist() {
        let pool = crate::db::create_test_pool().await;

        let row = SetlistRow {
            id: "sl-1".to_string(),
            user_id: "user-1".to_string(),
            prompt: "Chill house vibes".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            notes: Some("A relaxing set".to_string()),
            harmonic_flow_score: None,
            energy_profile: None,
            created_at: None,
        };

        insert_setlist(&pool, &row).await.unwrap();

        let fetched = get_setlist(&pool, "sl-1").await.unwrap().unwrap();
        assert_eq!(fetched.id, "sl-1");
        assert_eq!(fetched.prompt, "Chill house vibes");
        assert_eq!(fetched.notes.as_deref(), Some("A relaxing set"));
    }

    #[tokio::test]
    async fn test_get_setlist_not_found() {
        let pool = crate::db::create_test_pool().await;
        let result = get_setlist(&pool, "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_insert_and_get_setlist_tracks() {
        let pool = crate::db::create_test_pool().await;

        // Insert parent setlist first
        let setlist = SetlistRow {
            id: "sl-1".to_string(),
            user_id: "user-1".to_string(),
            prompt: "Test".to_string(),
            model: "test".to_string(),
            notes: None,
            harmonic_flow_score: None,
            energy_profile: None,
            created_at: None,
        };
        insert_setlist(&pool, &setlist).await.unwrap();

        let track = SetlistTrackRow {
            id: "st-1".to_string(),
            setlist_id: "sl-1".to_string(),
            track_id: None,
            position: 1,
            original_position: 1,
            title: "Desert Rose".to_string(),
            artist: "Sting".to_string(),
            bpm: Some(102.0),
            key: Some("A minor".to_string()),
            camelot: Some("8A".to_string()),
            energy: Some(5.0),
            transition_note: Some("Open with pads".to_string()),
            transition_score: None,
            source: "suggestion".to_string(),
            acquisition_info: None,
        };

        insert_setlist_track(&pool, &track).await.unwrap();

        let tracks = get_setlist_tracks(&pool, "sl-1").await.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Desert Rose");
        assert_eq!(tracks[0].bpm, Some(102.0));
    }

    #[tokio::test]
    async fn test_update_harmonic_score() {
        let pool = crate::db::create_test_pool().await;

        let setlist = SetlistRow {
            id: "sl-1".to_string(),
            user_id: "user-1".to_string(),
            prompt: "Test".to_string(),
            model: "test".to_string(),
            notes: None,
            harmonic_flow_score: None,
            energy_profile: None,
            created_at: None,
        };
        insert_setlist(&pool, &setlist).await.unwrap();

        update_setlist_harmonic_score(&pool, "sl-1", 82.5)
            .await
            .unwrap();

        let fetched = get_setlist(&pool, "sl-1").await.unwrap().unwrap();
        assert_eq!(fetched.harmonic_flow_score, Some(82.5));
    }

    #[tokio::test]
    async fn test_update_track_position() {
        let pool = crate::db::create_test_pool().await;

        let setlist = SetlistRow {
            id: "sl-1".to_string(),
            user_id: "user-1".to_string(),
            prompt: "Test".to_string(),
            model: "test".to_string(),
            notes: None,
            harmonic_flow_score: None,
            energy_profile: None,
            created_at: None,
        };
        insert_setlist(&pool, &setlist).await.unwrap();

        let track = SetlistTrackRow {
            id: "st-1".to_string(),
            setlist_id: "sl-1".to_string(),
            track_id: None,
            position: 1,
            original_position: 1,
            title: "Test".to_string(),
            artist: "Artist".to_string(),
            bpm: None,
            key: None,
            camelot: None,
            energy: None,
            transition_note: None,
            transition_score: None,
            source: "suggestion".to_string(),
            acquisition_info: None,
        };
        insert_setlist_track(&pool, &track).await.unwrap();

        update_setlist_track_position(&pool, "st-1", 3, Some(0.85))
            .await
            .unwrap();

        let tracks = get_setlist_tracks(&pool, "sl-1").await.unwrap();
        assert_eq!(tracks[0].position, 3);
        assert_eq!(tracks[0].transition_score, Some(0.85));
    }

    #[tokio::test]
    async fn test_load_catalog_tracks() {
        let pool = crate::db::create_test_pool().await;

        sqlx::query(
            "INSERT INTO tracks (id, title, source, bpm, camelot_key) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("t1")
        .bind("Track One")
        .bind("spotify")
        .bind(128.0)
        .bind("8A")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO artists (id, name) VALUES (?, ?)")
            .bind("a1")
            .bind("DJ Test")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES (?, ?)")
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
    }

    #[tokio::test]
    async fn test_insert_setlist_with_energy_profile() {
        let pool = crate::db::create_test_pool().await;

        let row = SetlistRow {
            id: "sl-ep".to_string(),
            user_id: "user-1".to_string(),
            prompt: "Peak time techno".to_string(),
            model: "test".to_string(),
            notes: None,
            harmonic_flow_score: None,
            energy_profile: Some("peak-time".to_string()),
            created_at: None,
        };

        insert_setlist(&pool, &row).await.unwrap();

        let fetched = get_setlist(&pool, "sl-ep").await.unwrap().unwrap();
        assert_eq!(fetched.energy_profile.as_deref(), Some("peak-time"));
    }

    #[tokio::test]
    async fn test_insert_setlist_with_none_energy_profile() {
        let pool = crate::db::create_test_pool().await;

        let row = SetlistRow {
            id: "sl-none".to_string(),
            user_id: "user-1".to_string(),
            prompt: "Whatever vibes".to_string(),
            model: "test".to_string(),
            notes: None,
            harmonic_flow_score: None,
            energy_profile: None,
            created_at: None,
        };

        insert_setlist(&pool, &row).await.unwrap();

        let fetched = get_setlist(&pool, "sl-none").await.unwrap().unwrap();
        assert!(fetched.energy_profile.is_none());
    }
}
