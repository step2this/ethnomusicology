# UC-019 Phase 2: Deezer Enrichment Pipeline

## Context

Phase 1 of UC-019 (Deezer preview playback) is working on tarab.studio. Currently, the frontend fires N parallel Deezer API searches every time a setlist is displayed. Phase 2 persists Deezer data on the tracks table to avoid redundant API calls.

## Architecture Decision

- **Spotify**: import, metadata, OAuth, album art (unchanged)
- **Deezer**: 30s preview playback via persisted `deezer_id` + `deezer_preview_url` on tracks table
- **LLM suggestions**: still use on-demand Deezer search (no track row to persist on)

## Changes

### Backend
1. Migration 007: `ALTER TABLE tracks ADD COLUMN deezer_id INTEGER; ALTER TABLE tracks ADD COLUMN deezer_preview_url TEXT;`
2. `services/deezer.rs`: enrichment function that searches Deezer for tracks where `deezer_id IS NULL`, persists matches
3. Wire into import pipeline: after Spotify import, fire Deezer enrichment alongside Claude enrichment
4. `POST /api/audio/enrich-deezer`: manual trigger endpoint for existing tracks
5. Track API response includes `deezer_preview_url`

### Frontend (future, not in this phase)
- Prefer persisted `deezer_preview_url` from track data over on-demand search
- Fall back to on-demand search for LLM suggestions and un-enriched tracks

## Critic Review Status

**PENDING** — builder is implementing, critic review required before commit.

## Known Risks
- Adding `deezer_id`/`deezer_preview_url` to `TrackRow` breaks all `SELECT *` queries unless migration runs first
- Test pool uses `sqlx::migrate!()` which auto-applies migration 007 — but `TrackRow` FromRow derive expects the columns to exist
- Deezer API rate limits unknown — 100ms delay between requests as a precaution
