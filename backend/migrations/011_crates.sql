-- Migration 011: DJ Crates — persistent track collections for setlist composition

CREATE TABLE IF NOT EXISTS crates (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crate_tracks (
    id TEXT PRIMARY KEY,
    crate_id TEXT NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    bpm DOUBLE PRECISION,
    key TEXT,
    camelot TEXT,
    energy DOUBLE PRECISION,
    spotify_uri TEXT,
    source_setlist_id TEXT,
    added_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(crate_id, title, artist)
);

-- NOTE: No REFERENCES/FK constraints — cascading deletes are handled manually in the application layer.

CREATE INDEX IF NOT EXISTS idx_crates_user_id ON crates(user_id);
CREATE INDEX IF NOT EXISTS idx_crate_tracks_crate_id ON crate_tracks(crate_id);
