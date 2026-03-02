# Steel Thread: ST-001 Serve Paginated Track Catalog via API

## Classification
- **Goal Level**: 🧵 Thread
- **Scope**: System (black box)
- **Priority**: P0 Critical — first integration proof
- **Complexity**: 🟢 Low — straightforward CRUD read

## Cross-Cutting References
- **UC-001**: Step 14 — "System displays imported tracks in the user's library" — proves tracks are retrievable after import
- **UC-013**: Step 11 — "User sees imported tracks with BPM and Camelot key columns visible" — partially proves: demonstrates the rendering pipeline and null-field handling; BPM/key display with real data proven only after migration 003

## Actors
- **Primary Actor**: App User (browsing their track catalog)
- **Supporting Actors**: Database (SQLite via SQLx), Frontend (Flutter Web)
- **Stakeholders & Interests**:
  - DJ: Wants to see all imported tracks with BPM, key, and source info for setlist building
  - Developer: Wants to prove the API shape matches the frontend prototype before building more features

## Conditions
- **Preconditions**:
  1. At least one track exists in the database (from UC-001 Spotify import)
  2. Backend server is running on port 3001
  3. Frontend can reach the backend API
  4. Migration 003 (DJ metadata) is applied, adding bpm, camelot_key, source, and energy columns to the tracks table — OR — the thread operates in "null-fields mode" where these fields return null
- **Success Postconditions**:
  1. GET /api/tracks returns a paginated JSON response with track metadata
  2. Response includes all documented fields: id, title, artist, album, duration_ms, bpm, key, energy, source, source_id, preview_url, album_art_url, date_added (nullable fields return null when not yet populated)
  3. Pagination metadata (page, per_page, total, total_pages) is present and correct
  4. Tracks can be sorted by title, artist, bpm, key, or date_added
  5. Frontend renders the track list from the API response
  6. Error responses use nested ErrorResponse format: {"error": {"code": "...", "message": "..."}} matching the OpenAPI spec
- **Failure Postconditions**:
  1. If no tracks exist, API returns empty data array with total=0
  2. If invalid page requested, API returns 400 with ErrorResponse
- **Invariants**:
  1. API keys never exposed to the frontend
  2. Response shape is consistent regardless of number of tracks (0 to 10,000)
  3. Sort results are deterministic — tracks with equal sort keys (e.g., same BPM) have a stable tie-break order (secondary sort by id)

## API Contract

| Method | Path | Description | Schema Ref | Status |
|--------|------|-------------|------------|--------|
| GET | /api/tracks | Fetch paginated track catalog with optional sort | GET /api/tracks → TrackListResponse | Draft |

### Request Parameters
- `page` (integer, default 1) — page number
- `per_page` (integer, default 25, max 100) — items per page
- `sort` (string, enum: title, artist, bpm, key, date_added) — sort field
- `order` (string, enum: asc, desc, default: desc) — sort direction (default desc = newest first for catalog browsing)

### Response Shape (200 OK)
```json
{
  "data": [
    {
      "id": "uuid",
      "title": "string",
      "artist": "string",
      "album": "string | null",
      "duration_ms": 180000,
      "bpm": 128.0,
      "key": "8A",
      "energy": 0.85,
      "source": "spotify",
      "source_id": "spotify:track:abc123",
      "preview_url": "https://p.scdn.co/...",
      "album_art_url": "https://i.scdn.co/...",
      "date_added": "2026-03-01T12:00:00Z"
    }
  ],
  "page": 1,
  "per_page": 25,
  "total": 142,
  "total_pages": 6
}
```

### Error Response (400)
```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Page must be a positive integer"
  }
}
```

### Current Schema vs. API Shape

The current `tracks` table (migration 001) has: `id`, `title`, `album`, `duration_ms`, `spotify_uri`, `spotify_preview_url`, `youtube_id`, `musicbrainz_id`, `created_at`, `updated_at`. Artist data lives in the `artists` table joined via `track_artists`.

The API response fields map as follows:

| API Field | Current DB Source | Notes |
|-----------|------------------|-------|
| `id` | `tracks.id` | Direct |
| `title` | `tracks.title` | Direct |
| `artist` | `artists.name` via `track_artists` JOIN | Concatenate multiple artists with ", " |
| `album` | `tracks.album` | Nullable |
| `duration_ms` | `tracks.duration_ms` | Nullable (INTEGER) |
| `bpm` | `tracks.bpm` | Added by migration 003 (REAL, nullable) |
| `key` | `tracks.camelot_key` | Added by migration 003 (TEXT, nullable) |
| `energy` | Not yet in schema | Future: essentia analysis (UC-015), return null for now |
| `source` | `tracks.source` | Added by migration 003 (TEXT, default 'spotify') |
| `source_id` | `tracks.spotify_uri` / `tracks.beatport_id` | Depends on source; normalize in query |
| `preview_url` | `tracks.spotify_preview_url` | Nullable |
| `album_art_url` | Not yet in schema | Future: add column or derive from Spotify API; return null for now |
| `date_added` | `tracks.created_at` | TIMESTAMP |

The error response shape must adopt the nested `ErrorResponse` format from `openapi.yaml` (`{"error": {"code": "...", "message": "..."}}`), which differs from the current flat `{"error": "message"}` in `backend/src/error.rs`. This steel thread should update `AppError::into_response` to match the OpenAPI contract.

## Main Success Scenario
1. **[Frontend]** User navigates to the Track Catalog screen
2. **[Frontend -> API]** App sends GET /api/tracks?page=1&per_page=25&sort=date_added&order=desc
3. **[API]** Server validates query parameters (page > 0, per_page 1-100, sort field valid)
4. **[API -> DB]** Server queries tracks table with LEFT JOIN on track_artists/artists, applies pagination (LIMIT/OFFSET) and sort (ORDER BY), counts total
5. **[DB -> API]** Database returns track rows with joined artist names, and total count
6. **[API]** Server serializes tracks to JSON response with pagination metadata, concatenating artist names for multi-artist tracks
7. **[API -> Frontend]** Server returns 200 OK with paginated track list
8. **[Frontend]** App parses response and renders track tiles (title, artist, BPM, key, album art)
9. **[Frontend]** User scrolls to bottom, app requests next page (page=2)
10. **[Frontend -> API -> DB -> API -> Frontend]** Pagination round-trip completes, new tracks appended

## Extensions
- **2a. Network timeout (frontend -> API)**:
  1. Frontend shows error state with retry button
  2. Returns to step 2 on retry
- **3a. Invalid query parameters (page=0, per_page=999, sort=invalid)**:
  1. Server returns 400 with ErrorResponse containing specific validation message
  2. Use case fails gracefully
- **3b. Page beyond total_pages requested (e.g., page=999 when only 6 pages exist)**:
  1. Server returns 200 with empty data array, requested page number, and correct total/total_pages
  2. Frontend renders empty state for the page
- **4a. Database query fails**:
  1. Server logs error, returns 500 with ErrorResponse
  2. Use case fails
- **4b. Sort on column with NULL values (e.g., sort=bpm when some tracks lack BPM)**:
  1. Server uses NULLS LAST for ascending sorts, NULLS FIRST for descending sorts
  2. Tracks with populated values always appear before tracks with NULL in the sorted direction
- **5a. No tracks in database**:
  1. Server returns 200 with empty data array and total=0
  2. Frontend renders empty state ("No tracks yet -- import from Spotify to get started")
- **6a. Response serialization produces null for optional fields**:
  1. Null fields (album, bpm, key, preview_url, album_art_url, energy) are included as null in JSON
  2. Frontend handles null display gracefully (shows "--" or placeholder)

## Integration Assertions
1. **[Frontend -> API]** Frontend can call GET /api/tracks and receive a well-formed JSON response matching the documented shape
2. **[API -> DB]** Tracks inserted by UC-001 Spotify import are retrievable via the catalog API without data loss
3. **[Frontend rendering]** All track fields (title, artist, BPM, key, source) render correctly including null values
4. **[Pagination]** Requesting page=2 returns different tracks than page=1, and total/total_pages are mathematically consistent
5. **[Sorting]** sort=bpm&order=asc returns tracks in ascending BPM order
6. **[Latency]** Full round trip (frontend request -> API -> DB -> response -> render) completes in < 200ms P95 for pages of 25 tracks
7. **[API error format]** A 400 response returns the nested ErrorResponse format `{"error": {"code": "...", "message": "..."}}` matching the OpenAPI spec, not the legacy flat format

## Does NOT Prove
- Does NOT prove track search/filtering (covered by a future ST)
- Does NOT prove track import (already proven by UC-001)
- Does NOT prove multi-source import (covered by ST-002 Beatport Import)
- Does NOT prove setlist generation (covered by ST-003 Prompt to Setlist)
- Does NOT prove audio playback (covered by SP-002 Flutter Web Audio spike)
- Does NOT prove performance under concurrent load or stress testing (single-request latency IS in scope via Integration Assertion #6)
- Does NOT prove authentication/authorization (UC-008, future)

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test && cd ../frontend && flutter test`
- **Test File**: `backend/tests/tracks_api_test.rs` (integration), `frontend/test/screens/track_catalog_test.dart` (widget)
- **Depends On**: UC-001 (tracks must exist in DB)
- **Blocks**: ST-002 (Beatport import extends the catalog), ST-003 (setlist needs track data)
- **Estimated Complexity**: S-M / ~2000 tokens
- **Agent Assignment**: Lead coordinates, Backend Teammate builds API, Frontend Teammate wires screen

### Implementation Prerequisites
Before building this steel thread, migration 003 (`003_dj_metadata.sql` from UC-013) must be applied to add the `bpm`, `camelot_key`, `source`, and other DJ metadata columns to the `tracks` table. Until then, the API can return null for those fields using the existing schema. The backend error response format (`AppError::into_response` in `backend/src/error.rs`) should be updated to match the nested `ErrorResponse` schema from `docs/api/openapi.yaml`.

## Acceptance Criteria
- [ ] All success postconditions verified by automated test
- [ ] All integration assertions pass end-to-end
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] API contract matches implementation (request/response shapes)
- [ ] Cross-layer round trip completes without manual intervention
- [ ] Code passes quality gates (cargo fmt, clippy, cargo test, flutter analyze, flutter test)
- [ ] Reviewer agent approves
