-- Migration 006: Import-track junction table for playlist filtering

CREATE TABLE IF NOT EXISTS import_tracks (
    import_id TEXT NOT NULL REFERENCES spotify_imports(id),
    track_id TEXT NOT NULL REFERENCES tracks(id),
    PRIMARY KEY (import_id, track_id)
);

CREATE INDEX IF NOT EXISTS idx_import_tracks_import_id ON import_tracks(import_id);
CREATE INDEX IF NOT EXISTS idx_import_tracks_track_id ON import_tracks(track_id);
