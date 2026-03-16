-- Migration 004: Setlists and setlist tracks for LLM-powered setlist generation

CREATE TABLE IF NOT EXISTS setlists (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    prompt TEXT NOT NULL,
    model TEXT NOT NULL,
    notes TEXT,
    harmonic_flow_score DOUBLE PRECISION,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS setlist_tracks (
    id TEXT PRIMARY KEY,
    setlist_id TEXT NOT NULL REFERENCES setlists(id),
    track_id TEXT REFERENCES tracks(id),
    position INTEGER NOT NULL,
    original_position INTEGER NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    bpm DOUBLE PRECISION,
    key TEXT,
    camelot TEXT,
    energy DOUBLE PRECISION,  -- H5: matches tracks.energy type (DOUBLE PRECISION/f64); LLM returns integer 1-10 but DOUBLE PRECISION avoids precision loss
    transition_note TEXT,
    transition_score DOUBLE PRECISION,
    source TEXT NOT NULL DEFAULT 'suggestion',
    acquisition_info TEXT  -- L2: reserved for future use (Beatport/SoundCloud purchase links, affiliate URLs)
);

-- H4: Index for efficient setlist_tracks lookup by setlist_id
CREATE INDEX IF NOT EXISTS idx_setlist_tracks_setlist_id ON setlist_tracks(setlist_id);
