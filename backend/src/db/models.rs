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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrackRow {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<i64>,
    pub bpm: Option<f64>,
    pub camelot_key: Option<String>,
    pub energy: Option<f64>,
    pub source: String,
    pub spotify_uri: Option<String>,
    pub spotify_preview_url: Option<String>,
    pub album_art_url: Option<String>,
    pub deezer_id: Option<i64>,
    pub deezer_preview_url: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpsertResult {
    Inserted,
    Updated,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SetlistRow {
    pub id: String,
    pub user_id: String,
    pub prompt: String,
    pub model: String,
    pub notes: Option<String>,
    pub harmonic_flow_score: Option<f64>,
    pub energy_profile: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SetlistTrackRow {
    pub id: String,
    pub setlist_id: String,
    pub track_id: Option<String>,
    pub position: i32,
    pub original_position: i32,
    pub title: String,
    pub artist: String,
    pub bpm: Option<f64>,
    pub key: Option<String>,
    pub camelot: Option<String>,
    pub energy: Option<f64>,
    pub transition_note: Option<String>,
    pub transition_score: Option<f64>,
    pub source: String,
    pub acquisition_info: Option<String>,
}
