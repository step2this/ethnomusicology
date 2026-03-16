-- Migration 005: Enrichment tracking + user usage limits
-- Supports ST-005 (track enrichment) and ST-006 (energy profiles)

ALTER TABLE tracks ADD COLUMN needs_enrichment BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE tracks ADD COLUMN enriched_at TIMESTAMP;
ALTER TABLE tracks ADD COLUMN enrichment_error TEXT;

ALTER TABLE setlists ADD COLUMN energy_profile TEXT;

CREATE TABLE IF NOT EXISTS user_usage (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    date TEXT NOT NULL,
    generation_count INTEGER NOT NULL DEFAULT 0,
    enrichment_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(user_id, date)
);
