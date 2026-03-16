use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CrateRow {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CrateTrackRow {
    pub id: String,
    pub crate_id: String,
    pub title: String,
    pub artist: String,
    pub bpm: Option<f64>,
    pub key: Option<String>,
    pub camelot: Option<String>,
    pub energy: Option<f64>,
    pub spotify_uri: Option<String>,
    pub source_setlist_id: Option<String>,
    pub added_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CrateSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub track_count: i64,
    pub created_at: Option<chrono::NaiveDateTime>,
}
