-- Migration 002: Add Spotify import tables
-- Supports UC-001: Import Seed Catalog from Spotify Playlist

-- Encrypted OAuth tokens per user
CREATE TABLE IF NOT EXISTS user_spotify_tokens (
    user_id TEXT PRIMARY KEY REFERENCES users(id),
    access_token_encrypted BLOB NOT NULL,
    refresh_token_encrypted BLOB NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    scopes TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Import provenance tracking
CREATE TABLE IF NOT EXISTS spotify_imports (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    spotify_playlist_id TEXT NOT NULL,
    spotify_playlist_name TEXT,
    tracks_found INTEGER NOT NULL DEFAULT 0,
    tracks_inserted INTEGER NOT NULL DEFAULT 0,
    tracks_updated INTEGER NOT NULL DEFAULT 0,
    tracks_failed INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'in_progress',
    error_message TEXT,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

-- UNIQUE indexes on spotify_uri for upsert support
CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_spotify_uri ON tracks(spotify_uri);
CREATE UNIQUE INDEX IF NOT EXISTS idx_artists_spotify_uri ON artists(spotify_uri);
