use sqlx::SqlitePool;
use uuid::Uuid;

use crate::db::{artists, imports, tracks};
use crate::services::import::{
    ArtistRecord, ImportError, ImportRepository, ImportSummary, TrackRecord, UpsertResult,
};

/// Production implementation of ImportRepository backed by SQLite.
pub struct SqliteImportRepository {
    pool: SqlitePool,
}

impl SqliteImportRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ImportRepository for SqliteImportRepository {
    async fn create_import(
        &self,
        user_id: &str,
        playlist_id: &str,
        playlist_name: Option<&str>,
    ) -> Result<String, ImportError> {
        let id = Uuid::new_v4().to_string();
        imports::create_import(&self.pool, &id, user_id, playlist_id, playlist_name)
            .await
            .map_err(|e| ImportError::Database(e.to_string()))?;
        Ok(id)
    }

    async fn upsert_track(&self, track: &TrackRecord) -> Result<UpsertResult, ImportError> {
        let result = tracks::upsert_track(
            &self.pool,
            &track.id,
            &track.title,
            track.album.as_deref(),
            track.duration_ms,
            &track.spotify_uri,
            track.spotify_preview_url.as_deref(),
        )
        .await
        .map_err(|e| ImportError::Database(e.to_string()))?;

        Ok(match result {
            crate::db::models::UpsertResult::Inserted => UpsertResult::Inserted,
            crate::db::models::UpsertResult::Updated => UpsertResult::Updated,
        })
    }

    async fn upsert_artist(&self, artist: &ArtistRecord) -> Result<UpsertResult, ImportError> {
        let result =
            artists::upsert_artist(&self.pool, &artist.id, &artist.name, &artist.spotify_uri)
                .await
                .map_err(|e| ImportError::Database(e.to_string()))?;

        Ok(match result {
            crate::db::models::UpsertResult::Inserted => UpsertResult::Inserted,
            crate::db::models::UpsertResult::Updated => UpsertResult::Updated,
        })
    }

    async fn upsert_track_artist(
        &self,
        track_id: &str,
        artist_id: &str,
    ) -> Result<(), ImportError> {
        artists::upsert_track_artist(&self.pool, track_id, artist_id)
            .await
            .map_err(|e| ImportError::Database(e.to_string()))?;
        Ok(())
    }

    async fn complete_import(
        &self,
        import_id: &str,
        summary: &ImportSummary,
    ) -> Result<(), ImportError> {
        imports::update_import_counts(
            &self.pool,
            import_id,
            summary.total as i32,
            summary.inserted as i32,
            summary.updated as i32,
            summary.failed as i32,
        )
        .await
        .map_err(|e| ImportError::Database(e.to_string()))?;

        imports::complete_import(&self.pool, import_id, &summary.status, None)
            .await
            .map_err(|e| ImportError::Database(e.to_string()))?;

        Ok(())
    }
}
