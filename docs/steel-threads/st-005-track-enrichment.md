# Steel Thread: ST-005 Track Enrichment via LLM Estimation

## Classification
- **Goal Level**: 🧵 Thread — thin end-to-end proof of enrichment pipeline
- **Scope**: System (black box)
- **Priority**: P0 Critical (data foundation for arrangement scoring)
- **Complexity**: 🟡 Medium

## Cross-Cutting References

- **UC-015**: Steps 1-4 — proves track enrichment populates BPM/key/energy via LLM estimation
- **UC-016**: Daily usage limits — proves cost cap enforcement via user_usage table
- **UC-018**: enriched_at timestamp — proves enrichment tracking metadata

This thread proves the **data quality foundation**: without real BPM/key/energy, the Camelot arrangement scoring from ST-003 is meaningless (all neutral scores on NULL values). ST-005 makes the arrangement algorithm meaningful.

## Actors

- **Primary Actor**: System (enrichment triggered by dev endpoint or import)
- **Supporting Actors**:
  - Claude API (Sonnet — BPM/key/energy estimation)
  - Database (SQLite — track metadata updates, usage tracking)
- **Stakeholders & Interests**:
  - DJ User: Wants accurate BPM/key/energy so arrangement scoring works
  - Developer: Wants clean enrichment pipeline with cost controls
  - Business: LLM calls cost money — daily cap prevents runaway costs

## Conditions

### Preconditions
1. Tracks exist in the database (imported via UC-001 / Spotify import)
2. Backend has a valid `ANTHROPIC_API_KEY` environment variable
3. Migration 005 has been applied (enrichment columns + user_usage table)

### Success Postconditions
1. `POST /api/tracks/enrich` triggers LLM enrichment for unenriched tracks
2. Tracks receive BPM (validated 0-300), Camelot key (validated via parse_camelot), energy (validated 1-10)
3. `needs_enrichment` flag set to 0, `enriched_at` timestamp set after enrichment
4. Failed tracks get `enrichment_error` set, not retried automatically
5. Daily enrichment cap of 250 tracks per user enforced via `user_usage` table
6. Cost cap exceeded returns 429 with `RATE_LIMITED` error code
7. Album art URL extracted from Spotify API during import
8. `from_spotify_key()` converts Spotify pitch_class/mode to Camelot notation
9. `from_notation()` converts essentia-style note/scale to Camelot notation

### Failure Postconditions
1. If Claude API fails, tracks get `enrichment_error` set — not silently skipped
2. If LLM returns malformed JSON, all tracks in batch get error marked
3. If daily cap exceeded, endpoint returns 429 before any LLM calls

## API Contract

### POST /api/tracks/enrich
**Request**: Empty body, `X-User-Id` header (defaults to "dev-user")

**Response 200**:
```json
{"enriched": 3, "errors": 0, "skipped": 0}
```

**Response 429**:
```json
{"error": {"code": "RATE_LIMITED", "message": "Daily enrichment limit of 250 tracks reached"}}
```

## Implementation Summary

| Component | File | Description |
|-----------|------|-------------|
| Migration | `migrations/005_enrichment.sql` | enrichment columns + user_usage table |
| Enrichment service | `services/enrichment.rs` | Batch LLM estimation, cost cap, error handling |
| DB functions | `db/tracks.rs` | get_unenriched, update_metadata, mark_error, usage tracking |
| Route | `routes/enrich.rs` | POST handler + error mapping |
| Camelot conversion | `services/camelot.rs` | from_spotify_key(), from_notation() |
| Album art | `api/spotify.rs` | SpotifyImageRaw, largest image picker |

## Test Coverage

- 176 total backend tests (23 new in ST-005)
- Enrichment service: 6 tests (success, empty, cost cap, malformed, usage, flags)
- Enrich route: 3 tests (success, empty, rate limited)
- Camelot conversion: 11 tests (spotify key, notation, edge cases)
- Album art: 3 tests (largest, empty, no dimensions)

## Status: COMPLETE
- **Branch**: `feature/mvp-magic-set-generator`
- **PR**: #2
- **Critic review**: 0 CRITICAL, 4 HIGH (2 fixed, 2 MVP-acceptable), 3 LOW, 4 INFO
