-- Migration 008: Setlist versioning + conversations for refinement (ST-007)

CREATE TABLE IF NOT EXISTS setlist_versions (
    id TEXT PRIMARY KEY,
    setlist_id TEXT NOT NULL REFERENCES setlists(id),
    version_number INTEGER NOT NULL,
    parent_version_id TEXT REFERENCES setlist_versions(id),
    action TEXT,
    action_summary TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(setlist_id, version_number)
);
CREATE INDEX IF NOT EXISTS idx_sv_setlist ON setlist_versions(setlist_id);

CREATE TABLE IF NOT EXISTS setlist_version_tracks (
    id TEXT PRIMARY KEY,
    version_id TEXT NOT NULL REFERENCES setlist_versions(id),
    track_id TEXT REFERENCES tracks(id),
    position INTEGER NOT NULL,
    original_position INTEGER NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    bpm DOUBLE PRECISION,
    key TEXT,
    camelot TEXT,
    energy DOUBLE PRECISION,
    transition_note TEXT,
    transition_score DOUBLE PRECISION,
    source TEXT NOT NULL DEFAULT 'suggestion',
    acquisition_info TEXT
);
CREATE INDEX IF NOT EXISTS idx_svt_version ON setlist_version_tracks(version_id);

CREATE TABLE IF NOT EXISTS setlist_conversations (
    id TEXT PRIMARY KEY,
    setlist_id TEXT NOT NULL REFERENCES setlists(id),
    version_id TEXT REFERENCES setlist_versions(id),
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_sc_setlist ON setlist_conversations(setlist_id);
