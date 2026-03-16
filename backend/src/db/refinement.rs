// ST-007: Refinement DB operations (versioning, conversations)

use sqlx::PgPool;

use crate::db::models::{SetlistConversationRow, SetlistVersionRow, VersionTrackRow};

pub async fn insert_version(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    row: &SetlistVersionRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO setlist_versions (id, setlist_id, version_number, parent_version_id, action, action_summary) \
         VALUES ($1, $2, $3, $4, $5, $6)",
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
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tracks: &[VersionTrackRow],
) -> Result<(), sqlx::Error> {
    for track in tracks {
        sqlx::query(
            "INSERT INTO setlist_version_tracks \
             (id, version_id, track_id, position, original_position, title, artist, bpm, key, camelot, energy, transition_note, transition_score, source, acquisition_info) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
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
    pool: &PgPool,
    row: &SetlistConversationRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO setlist_conversations (id, setlist_id, version_id, role, content) \
         VALUES ($1, $2, $3, $4, $5)",
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
    pool: &PgPool,
    setlist_id: &str,
) -> Result<Vec<SetlistVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistVersionRow>(
        "SELECT id, setlist_id, version_number, parent_version_id, action, action_summary, \
         CAST(created_at AS TEXT) as created_at \
         FROM setlist_versions WHERE setlist_id = $1 ORDER BY version_number",
    )
    .bind(setlist_id)
    .fetch_all(pool)
    .await
}

pub async fn get_latest_version(
    pool: &PgPool,
    setlist_id: &str,
) -> Result<Option<SetlistVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistVersionRow>(
        "SELECT id, setlist_id, version_number, parent_version_id, action, action_summary, \
         CAST(created_at AS TEXT) as created_at \
         FROM setlist_versions WHERE setlist_id = $1 ORDER BY version_number DESC LIMIT 1",
    )
    .bind(setlist_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_version_tracks(
    pool: &PgPool,
    version_id: &str,
) -> Result<Vec<VersionTrackRow>, sqlx::Error> {
    sqlx::query_as::<_, VersionTrackRow>(
        "SELECT svt.id, svt.version_id, svt.track_id, svt.position, svt.original_position, \
         svt.title, svt.artist, svt.bpm, svt.key, svt.camelot, svt.energy, \
         svt.transition_note, svt.transition_score, svt.source, svt.acquisition_info, \
         t.spotify_uri \
         FROM setlist_version_tracks svt LEFT JOIN tracks t ON svt.track_id = t.id \
         WHERE svt.version_id = $1 ORDER BY svt.position",
    )
    .bind(version_id)
    .fetch_all(pool)
    .await
}

pub async fn get_version_by_number(
    pool: &PgPool,
    setlist_id: &str,
    version_number: i32,
) -> Result<Option<SetlistVersionRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistVersionRow>(
        "SELECT id, setlist_id, version_number, parent_version_id, action, action_summary, \
         CAST(created_at AS TEXT) as created_at \
         FROM setlist_versions WHERE setlist_id = $1 AND version_number = $2",
    )
    .bind(setlist_id)
    .bind(version_number)
    .fetch_optional(pool)
    .await
}

pub async fn get_conversations_by_setlist(
    pool: &PgPool,
    setlist_id: &str,
) -> Result<Vec<SetlistConversationRow>, sqlx::Error> {
    sqlx::query_as::<_, SetlistConversationRow>(
        "SELECT id, setlist_id, version_id, role, content, \
         CAST(created_at AS TEXT) as created_at \
         FROM setlist_conversations WHERE setlist_id = $1 ORDER BY created_at",
    )
    .bind(setlist_id)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_test_pool, models::SetlistVersionRow};

    async fn insert_test_setlist(pool: &PgPool) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES ($1, $2, $3, $4)")
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

    #[tokio::test]
    async fn test_empty_setlist_returns_none() {
        let pool = create_test_pool().await;

        let latest = get_latest_version(&pool, "nonexistent-setlist-id")
            .await
            .unwrap();
        assert!(latest.is_none());

        let versions = get_versions_by_setlist(&pool, "nonexistent-setlist-id")
            .await
            .unwrap();
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

}
