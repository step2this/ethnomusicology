use sqlx::PgPool;
use uuid::Uuid;

use crate::db::crate_models::{CrateRow, CrateSummary, CrateTrackRow};

// ---------------------------------------------------------------------------
// Insert
// ---------------------------------------------------------------------------

pub async fn create_crate(
    pool: &PgPool,
    id: &str,
    user_id: &str,
    name: &str,
    description: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO crates (id, user_id, name, description) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(description)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Read
// ---------------------------------------------------------------------------

pub async fn list_crates(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<CrateSummary>, sqlx::Error> {
    sqlx::query_as::<_, CrateSummary>(
        r#"SELECT c.id, c.name, c.description, c.created_at,
              CAST(COUNT(ct.id) AS INTEGER) AS track_count
           FROM crates c
           LEFT JOIN crate_tracks ct ON c.id = ct.crate_id
           WHERE c.user_id = $1
           GROUP BY c.id
           ORDER BY c.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_crate(pool: &PgPool, id: &str) -> Result<Option<CrateRow>, sqlx::Error> {
    sqlx::query_as::<_, CrateRow>(
        "SELECT id, user_id, name, description, created_at FROM crates WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_crate_tracks(
    pool: &PgPool,
    crate_id: &str,
) -> Result<Vec<CrateTrackRow>, sqlx::Error> {
    sqlx::query_as::<_, CrateTrackRow>(
        "SELECT id, crate_id, title, artist, bpm, key, camelot, energy, \
         spotify_uri, source_setlist_id, added_at \
         FROM crate_tracks WHERE crate_id = $1 ORDER BY added_at ASC",
    )
    .bind(crate_id)
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

pub async fn delete_crate(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM crate_tracks WHERE crate_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM crates WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn remove_crate_track(
    pool: &PgPool,
    crate_id: &str,
    track_id: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM crate_tracks WHERE crate_id = $1 AND id = $2")
        .bind(crate_id)
        .bind(track_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ---------------------------------------------------------------------------
// Copy tracks from a setlist into a crate
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct SetlistTrackForCrate {
    title: String,
    artist: String,
    bpm: Option<f64>,
    key: Option<String>,
    camelot: Option<String>,
    energy: Option<f64>,
    spotify_uri: Option<String>,
}

pub async fn add_tracks_from_setlist(
    pool: &PgPool,
    crate_id: &str,
    setlist_id: &str,
) -> Result<i64, sqlx::Error> {
    let rows = sqlx::query_as::<_, SetlistTrackForCrate>(
        r#"SELECT st.title, st.artist, st.bpm, st.key, st.camelot, st.energy,
              t.spotify_uri
           FROM setlist_tracks st
           LEFT JOIN tracks t ON st.track_id = t.id
           WHERE st.setlist_id = $1"#,
    )
    .bind(setlist_id)
    .fetch_all(pool)
    .await?;

    let mut tx = pool.begin().await?;
    let mut added: i64 = 0;
    for row in rows {
        let id = Uuid::new_v4().to_string();
        let result = sqlx::query(
            "INSERT INTO crate_tracks \
             (id, crate_id, title, artist, bpm, key, camelot, energy, spotify_uri, source_setlist_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) ON CONFLICT DO NOTHING",
        )
        .bind(&id)
        .bind(crate_id)
        .bind(&row.title)
        .bind(&row.artist)
        .bind(row.bpm)
        .bind(&row.key)
        .bind(&row.camelot)
        .bind(row.energy)
        .bind(&row.spotify_uri)
        .bind(setlist_id)
        .execute(&mut *tx)
        .await?;
        added += result.rows_affected() as i64;
    }
    tx.commit().await?;
    Ok(added)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_crate_not_found() {
        let pool = crate::db::create_test_pool().await;
        let result = get_crate(&pool, "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_crates_returns_track_count() {
        let pool = crate::db::create_test_pool().await;
        create_crate(&pool, "c1", "user-1", "Alpha", None)
            .await
            .unwrap();
        create_crate(&pool, "c2", "user-1", "Beta", None)
            .await
            .unwrap();

        // Add two tracks to c1
        sqlx::query("INSERT INTO crate_tracks (id, crate_id, title, artist) VALUES ($1, $2, $3, $4)")
            .bind("ct1")
            .bind("c1")
            .bind("Track A")
            .bind("Artist A")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO crate_tracks (id, crate_id, title, artist) VALUES ($1, $2, $3, $4)")
            .bind("ct2")
            .bind("c1")
            .bind("Track B")
            .bind("Artist B")
            .execute(&pool)
            .await
            .unwrap();

        let crates = list_crates(&pool, "user-1").await.unwrap();
        assert_eq!(crates.len(), 2);
        // Most recently created first — c2 (created after c1)
        let alpha = crates.iter().find(|c| c.id == "c1").unwrap();
        let beta = crates.iter().find(|c| c.id == "c2").unwrap();
        assert_eq!(alpha.track_count, 2);
        assert_eq!(beta.track_count, 0);
    }

    #[tokio::test]
    async fn test_list_crates_excludes_other_users() {
        let pool = crate::db::create_test_pool().await;
        create_crate(&pool, "c1", "user-1", "Mine", None)
            .await
            .unwrap();
        create_crate(&pool, "c2", "user-2", "Theirs", None)
            .await
            .unwrap();

        let crates = list_crates(&pool, "user-1").await.unwrap();
        assert_eq!(crates.len(), 1);
        assert_eq!(crates[0].id, "c1");
    }

    #[tokio::test]
    async fn test_delete_crate_removes_tracks() {
        let pool = crate::db::create_test_pool().await;
        create_crate(&pool, "c1", "user-1", "Temp", None)
            .await
            .unwrap();
        sqlx::query("INSERT INTO crate_tracks (id, crate_id, title, artist) VALUES ($1, $2, $3, $4)")
            .bind("ct1")
            .bind("c1")
            .bind("Track A")
            .bind("Artist A")
            .execute(&pool)
            .await
            .unwrap();

        delete_crate(&pool, "c1").await.unwrap();

        assert!(get_crate(&pool, "c1").await.unwrap().is_none());
        let tracks = get_crate_tracks(&pool, "c1").await.unwrap();
        assert!(tracks.is_empty());
    }

    #[tokio::test]
    async fn test_add_tracks_from_setlist_skips_duplicates() {
        let pool = crate::db::create_test_pool().await;
        create_crate(&pool, "c1", "user-1", "Alpha", None)
            .await
            .unwrap();

        // Insert a setlist with tracks
        sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES ($1, $2, $3, $4)")
            .bind("sl-1")
            .bind("user-1")
            .bind("test")
            .bind("test")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO setlist_tracks (id, setlist_id, position, original_position, title, artist, source) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind("st1").bind("sl-1").bind(1).bind(1).bind("Track A").bind("Artist A").bind("suggestion")
        .execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO setlist_tracks (id, setlist_id, position, original_position, title, artist, source) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind("st2").bind("sl-1").bind(2).bind(2).bind("Track B").bind("Artist B").bind("suggestion")
        .execute(&pool).await.unwrap();

        let added = add_tracks_from_setlist(&pool, "c1", "sl-1").await.unwrap();
        assert_eq!(added, 2);

        // Second call — all duplicates, should add 0
        let added_again = add_tracks_from_setlist(&pool, "c1", "sl-1").await.unwrap();
        assert_eq!(added_again, 0);

        let tracks = get_crate_tracks(&pool, "c1").await.unwrap();
        assert_eq!(tracks.len(), 2);
    }
}
