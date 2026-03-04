// ST-007: Refinement DB operations (versioning, conversations)

use sqlx::SqlitePool;

use crate::db::models::{SetlistConversationRow, SetlistVersionRow, VersionTrackRow};

pub async fn insert_version(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &SetlistVersionRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO setlist_versions (id, setlist_id, version_number, parent_version_id, action, action_summary) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&row.id)
    .bind(&row.setlist_id)
    .bind(row.version_number)
    .bind(&row.parent_version_id)
    .bind(&row.action)
    .bind(&row.action_summary)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn insert_version_tracks(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tracks: &[VersionTrackRow],
) -> Result<(), sqlx::Error> {
    for track in tracks {
        sqlx::query(
            "INSERT INTO setlist_version_tracks \
             (id, version_id, track_id, position, original_position, title, artist, bpm, key, camelot, energy, transition_note, transition_score, source, acquisition_info) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&track.id)
        .bind(&track.version_id)
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
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn insert_conversation(
    pool: &SqlitePool,
    row: &SetlistConversationRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO setlist_conversations (id, setlist_id, version_id, role, content) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&row.id)
    .bind(&row.setlist_id)
    .bind(&row.version_id)
    .bind(&row.role)
    .bind(&row.content)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_versions_by_setlist(
    pool: &SqlitePool,
    setlist_id: &str,
) -> Result<Vec<SetlistVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistVersionRow>(
        "SELECT id, setlist_id, version_number, parent_version_id, action, action_summary, \
         CAST(created_at AS TEXT) as created_at \
         FROM setlist_versions WHERE setlist_id = ? ORDER BY version_number",
    )
    .bind(setlist_id)
    .fetch_all(pool)
    .await
}

pub async fn get_latest_version(
    pool: &SqlitePool,
    setlist_id: &str,
) -> Result<Option<SetlistVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistVersionRow>(
        "SELECT id, setlist_id, version_number, parent_version_id, action, action_summary, \
         CAST(created_at AS TEXT) as created_at \
         FROM setlist_versions WHERE setlist_id = ? ORDER BY version_number DESC LIMIT 1",
    )
    .bind(setlist_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_version_tracks(
    pool: &SqlitePool,
    version_id: &str,
) -> Result<Vec<VersionTrackRow>, sqlx::Error> {
    sqlx::query_as::<_, VersionTrackRow>(
        "SELECT id, version_id, track_id, position, original_position, title, artist, bpm, key, camelot, energy, \
         transition_note, transition_score, source, acquisition_info \
         FROM setlist_version_tracks WHERE version_id = ? ORDER BY position",
    )
    .bind(version_id)
    .fetch_all(pool)
    .await
}

pub async fn get_version_by_number(
    pool: &SqlitePool,
    setlist_id: &str,
    version_number: i32,
) -> Result<Option<SetlistVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistVersionRow>(
        "SELECT id, setlist_id, version_number, parent_version_id, action, action_summary, \
         CAST(created_at AS TEXT) as created_at \
         FROM setlist_versions WHERE setlist_id = ? AND version_number = ?",
    )
    .bind(setlist_id)
    .bind(version_number)
    .fetch_optional(pool)
    .await
}

pub async fn get_conversations_by_setlist(
    pool: &SqlitePool,
    setlist_id: &str,
) -> Result<Vec<SetlistConversationRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistConversationRow>(
        "SELECT id, setlist_id, version_id, role, content, \
         CAST(created_at AS TEXT) as created_at \
         FROM setlist_conversations WHERE setlist_id = ? ORDER BY created_at",
    )
    .bind(setlist_id)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_test_pool, models::{SetlistConversationRow, SetlistVersionRow, VersionTrackRow}};

    async fn insert_test_setlist(pool: &SqlitePool) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind("test-user")
            .bind("test prompt")
            .bind("claude-test")
            .execute(pool)
            .await
            .unwrap();
        id
    }

    fn make_version(setlist_id: &str, version_number: i32) -> SetlistVersionRow {
        SetlistVersionRow {
            id: uuid::Uuid::new_v4().to_string(),
            setlist_id: setlist_id.to_string(),
            version_number,
            parent_version_id: None,
            action: Some("generate".to_string()),
            action_summary: Some(format!("Version {} summary", version_number)),
            created_at: None,
        }
    }

    fn make_track(version_id: &str, position: i32) -> VersionTrackRow {
        VersionTrackRow {
            id: uuid::Uuid::new_v4().to_string(),
            version_id: version_id.to_string(),
            track_id: None,
            position,
            original_position: position,
            title: format!("Track {}", position),
            artist: "Test Artist".to_string(),
            bpm: Some(120.0),
            key: Some("A".to_string()),
            camelot: Some("8A".to_string()),
            energy: Some(0.8),
            transition_note: None,
            transition_score: None,
            source: "suggestion".to_string(),
            acquisition_info: None,
        }
    }

    #[tokio::test]
    async fn test_insert_and_get_version() {
        let pool = create_test_pool().await;
        let setlist_id = insert_test_setlist(&pool).await;
        let version = make_version(&setlist_id, 1);

        let mut tx = pool.begin().await.unwrap();
        insert_version(&mut tx, &version).await.unwrap();
        tx.commit().await.unwrap();

        let versions = get_versions_by_setlist(&pool, &setlist_id).await.unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].id, version.id);
        assert_eq!(versions[0].version_number, 1);
        assert_eq!(versions[0].action.as_deref(), Some("generate"));
    }

    #[tokio::test]
    async fn test_get_latest_version() {
        let pool = create_test_pool().await;
        let setlist_id = insert_test_setlist(&pool).await;

        let v1 = make_version(&setlist_id, 1);
        let v2 = make_version(&setlist_id, 2);

        let mut tx = pool.begin().await.unwrap();
        insert_version(&mut tx, &v1).await.unwrap();
        insert_version(&mut tx, &v2).await.unwrap();
        tx.commit().await.unwrap();

        let latest = get_latest_version(&pool, &setlist_id).await.unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version_number, 2);
    }

    #[tokio::test]
    async fn test_version_tracks_roundtrip() {
        let pool = create_test_pool().await;
        let setlist_id = insert_test_setlist(&pool).await;
        let version = make_version(&setlist_id, 1);

        let mut tx = pool.begin().await.unwrap();
        insert_version(&mut tx, &version).await.unwrap();
        let tracks = vec![make_track(&version.id, 1), make_track(&version.id, 2)];
        insert_version_tracks(&mut tx, &tracks).await.unwrap();
        tx.commit().await.unwrap();

        let fetched = get_version_tracks(&pool, &version.id).await.unwrap();
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].position, 1);
        assert_eq!(fetched[1].position, 2);
        assert_eq!(fetched[0].title, "Track 1");
        assert_eq!(fetched[0].bpm, Some(120.0));
    }

    #[tokio::test]
    async fn test_conversations_roundtrip() {
        let pool = create_test_pool().await;
        let setlist_id = insert_test_setlist(&pool).await;

        let user_msg = SetlistConversationRow {
            id: uuid::Uuid::new_v4().to_string(),
            setlist_id: setlist_id.clone(),
            version_id: None,
            role: "user".to_string(),
            content: "Make it more energetic".to_string(),
            created_at: None,
        };
        let assistant_msg = SetlistConversationRow {
            id: uuid::Uuid::new_v4().to_string(),
            setlist_id: setlist_id.clone(),
            version_id: None,
            role: "assistant".to_string(),
            content: "I've increased the energy levels.".to_string(),
            created_at: None,
        };

        insert_conversation(&pool, &user_msg).await.unwrap();
        insert_conversation(&pool, &assistant_msg).await.unwrap();

        let convos = get_conversations_by_setlist(&pool, &setlist_id).await.unwrap();
        assert_eq!(convos.len(), 2);
        assert_eq!(convos[0].role, "user");
        assert_eq!(convos[1].role, "assistant");
    }

    #[tokio::test]
    async fn test_empty_setlist_returns_none() {
        let pool = create_test_pool().await;

        let latest = get_latest_version(&pool, "nonexistent-setlist-id").await.unwrap();
        assert!(latest.is_none());

        let versions = get_versions_by_setlist(&pool, "nonexistent-setlist-id").await.unwrap();
        assert!(versions.is_empty());
    }

    #[tokio::test]
    async fn test_unique_constraint() {
        let pool = create_test_pool().await;
        let setlist_id = insert_test_setlist(&pool).await;

        let v1 = make_version(&setlist_id, 1);
        let v1_dup = SetlistVersionRow {
            id: uuid::Uuid::new_v4().to_string(),
            ..make_version(&setlist_id, 1)
        };

        let mut tx = pool.begin().await.unwrap();
        insert_version(&mut tx, &v1).await.unwrap();
        tx.commit().await.unwrap();

        let mut tx2 = pool.begin().await.unwrap();
        let result = insert_version(&mut tx2, &v1_dup).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_version_by_number() {
        let pool = create_test_pool().await;
        let setlist_id = insert_test_setlist(&pool).await;
        let v1 = make_version(&setlist_id, 1);
        let v2 = make_version(&setlist_id, 2);

        let mut tx = pool.begin().await.unwrap();
        insert_version(&mut tx, &v1).await.unwrap();
        insert_version(&mut tx, &v2).await.unwrap();
        tx.commit().await.unwrap();

        let found = get_version_by_number(&pool, &setlist_id, 2).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, v2.id);

        let not_found = get_version_by_number(&pool, &setlist_id, 99).await.unwrap();
        assert!(not_found.is_none());
    }
}
