use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub album: Option<String>,
    pub duration_ms: Option<i64>,
    pub spotify_uri: Option<String>,
    pub spotify_preview_url: Option<String>,
    pub youtube_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub tradition: Option<String>,
    pub spotify_uri: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SpotifyImport {
    pub id: String,
    pub user_id: String,
    pub spotify_playlist_id: String,
    pub spotify_playlist_name: Option<String>,
    pub tracks_found: i32,
    pub tracks_inserted: i32,
    pub tracks_updated: i32,
    pub tracks_failed: i32,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpsertResult {
    Inserted,
    Updated,
}
