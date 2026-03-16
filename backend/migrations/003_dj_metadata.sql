-- Migration 003: Add DJ metadata columns to tracks table
-- Supports ST-001 (track catalog API) and UC-013 (multi-source import)

ALTER TABLE tracks ADD COLUMN bpm DOUBLE PRECISION;
ALTER TABLE tracks ADD COLUMN camelot_key TEXT;
ALTER TABLE tracks ADD COLUMN energy DOUBLE PRECISION;
ALTER TABLE tracks ADD COLUMN source TEXT NOT NULL DEFAULT 'spotify';
ALTER TABLE tracks ADD COLUMN album_art_url TEXT;
