-- Initial schema for Ethnomusicology backend
-- SQLite (dev) / PostgreSQL (prod) compatible via SQLx

-- Artists table
CREATE TABLE IF NOT EXISTS artists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    bio TEXT,
    region TEXT,
    country TEXT,
    tradition TEXT,
    spotify_uri TEXT,
    musicbrainz_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tracks table (seed data from Spotify import)
CREATE TABLE IF NOT EXISTS tracks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    album TEXT,
    duration_ms INTEGER,
    spotify_uri TEXT,
    spotify_preview_url TEXT,
    youtube_id TEXT,
    musicbrainz_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Track-Artist junction (many-to-many)
CREATE TABLE IF NOT EXISTS track_artists (
    track_id TEXT NOT NULL REFERENCES tracks(id),
    artist_id TEXT NOT NULL REFERENCES artists(id),
    role TEXT DEFAULT 'primary',
    PRIMARY KEY (track_id, artist_id)
);

-- Occasions (Nikah, Eid al-Fitr, Eid al-Adha, Mawlid, Sufi gathering, etc.)
CREATE TABLE IF NOT EXISTS occasions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    icon TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Track metadata tags (region, tradition, mood, language, etc.)
CREATE TABLE IF NOT EXISTS track_tags (
    id TEXT PRIMARY KEY,
    track_id TEXT NOT NULL REFERENCES tracks(id),
    category TEXT NOT NULL,  -- 'region', 'tradition', 'mood', 'language', 'instrument'
    value TEXT NOT NULL,
    confidence REAL DEFAULT 1.0,  -- 1.0 = curator-verified, <1.0 = auto-tagged
    source TEXT DEFAULT 'curator',  -- 'curator', 'lastfm', 'musicbrainz'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Track-Occasion suitability scores
CREATE TABLE IF NOT EXISTS track_occasions (
    track_id TEXT NOT NULL REFERENCES tracks(id),
    occasion_id TEXT NOT NULL REFERENCES occasions(id),
    score REAL NOT NULL DEFAULT 0.5,  -- 0.0 to 1.0
    phase TEXT,  -- 'processional', 'ceremony', 'celebration', 'ambient'
    is_sacred BOOLEAN DEFAULT FALSE,
    curator_notes TEXT,
    PRIMARY KEY (track_id, occasion_id)
);

-- Playlists
CREATE TABLE IF NOT EXISTS playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    occasion_id TEXT REFERENCES occasions(id),
    user_id TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Playlist tracks (ordered)
CREATE TABLE IF NOT EXISTS playlist_tracks (
    playlist_id TEXT NOT NULL REFERENCES playlists(id),
    track_id TEXT NOT NULL REFERENCES tracks(id),
    position INTEGER NOT NULL,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (playlist_id, track_id)
);

-- Users (for Sprint 4)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE,
    display_name TEXT,
    password_hash TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Seed occasions
INSERT OR IGNORE INTO occasions (id, name, description) VALUES
    ('nikah', 'Nikah', 'Wedding ceremony and celebration'),
    ('eid-al-fitr', 'Eid al-Fitr', 'Celebration marking the end of Ramadan'),
    ('eid-al-adha', 'Eid al-Adha', 'Festival of Sacrifice'),
    ('mawlid', 'Mawlid', 'Celebration of the Prophet''s birthday'),
    ('sufi-gathering', 'Sufi Gathering', 'Spiritual gathering with devotional music'),
    ('walima', 'Walima', 'Wedding feast and reception'),
    ('aqiqah', 'Aqiqah', 'Celebration for a newborn child'),
    ('family-gathering', 'Family Gathering', 'Casual family get-together'),
    ('ramadan-evening', 'Ramadan Evening', 'Iftar and evening gatherings during Ramadan');
