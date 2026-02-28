# Use Case: UC-014 Import Tracks from SoundCloud

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P0 Critical
- **Complexity**: 🟡 Medium

## Actors
- **Primary Actor**: App User (authenticated)
- **Supporting Actors**:
  - SoundCloud API (OAuth 2.1, app-level credentials)
  - Database (SQLite/PostgreSQL via SQLx)
- **Stakeholders & Interests**:
  - DJ User: Wants to import tracks from SoundCloud playlists/sets for inclusion in setlists; understands SoundCloud tracks lack BPM/key and will need analysis
  - Developer: Wants `SoundCloudClient` as a second `MusicSourceClient` implementation, validating the trait design from UC-013
  - Curator: Wants access to SoundCloud's deep underground catalog (bootlegs, edits, unreleased mixes)

## Conditions
- **Preconditions** (must be true before starting):
  1. User is authenticated in the app
  2. Backend has valid SoundCloud API credentials (client ID + secret) configured via environment
  3. Database is initialized with tracks/artists/track_artists schema and DJ metadata columns (migrations 001-003 applied)
  4. `MusicSourceClient` trait is defined (established in UC-013)

- **Success Postconditions** (true when done right):
  1. All tracks from the SoundCloud playlist/set exist in the `tracks` table with: title, artist, duration_ms, soundcloud_urn, soundcloud_permalink_url, artwork_url, stream_access_level, source='soundcloud'
  2. All artists from imported tracks exist in the `artists` table with: name, soundcloud_urn
  3. All track-artist relationships exist in the `track_artists` junction table
  4. Duplicate tracks (same soundcloud_urn) are upserted — metadata updated, no duplicate rows created
  5. BPM, musical_key, and camelot_key are set to `null` for all SoundCloud imports (no native DJ metadata)
  6. All imported tracks have `needs_analysis = true` flag set, marking them for UC-015 audio analysis
  7. The import operation returns a summary: total tracks found, new tracks inserted, existing tracks updated, failed, tracks needing analysis
  8. An import record is stored linking the SoundCloud playlist URN to the user and import timestamp
  9. `SoundCloudClient` implements the `MusicSourceClient` trait

- **Failure Postconditions** (true when it fails gracefully):
  1. If the import fails mid-way, all tracks successfully fetched up to the failure point are persisted (per-track transactions)
  2. User receives a clear error message describing what went wrong
  3. Rate limit errors are communicated with a retry-after suggestion
  4. Tracks with `blocked` stream access are skipped with a count in the summary

- **Invariants** (must remain true throughout):
  1. SoundCloud API credentials are never exposed to the frontend
  2. All SoundCloud API calls happen on the backend — frontend never calls SoundCloud directly
  3. Database remains in a consistent state throughout (per-track transactions)
  4. Existing tracks from other sources (Spotify, Beatport) are never modified by SoundCloud import
  5. The `urn` field is used as the primary identifier (not `id` — per SoundCloud migration deadline)

## Main Success Scenario
1. User navigates to the "Import" screen in the Flutter app
2. User selects "SoundCloud" as the import source
3. User pastes a SoundCloud playlist or set URL (e.g., `https://soundcloud.com/username/sets/playlist-name`)
4. System validates the URL format and extracts the playlist/set identifier
5. Backend authenticates with SoundCloud API using app-level OAuth 2.1 credentials
6. Backend resolves the URL to a playlist URN via SoundCloud's resolve endpoint (`GET /resolve?url=...`)
7. Backend calls SoundCloud API to fetch the playlist tracks, paginating with cursor-based pagination until all tracks are retrieved
8. For each track in the response, Backend extracts: title, user (artist), duration, urn, permalink_url, artwork_url, stream_access_level, genre (SoundCloud user-tagged, not DJ-grade), created_at
9. Backend filters out tracks with `stream_access_level = 'blocked'` and counts them separately
10. For each importable track, Backend performs a single transaction: upserts the track into `tracks` (keyed on soundcloud_urn with source='soundcloud', needs_analysis=true), upserts the artist into `artists` (keyed on soundcloud_urn), and upserts track-artist relationships. If any part fails, the transaction rolls back for that track only.
11. System displays an import summary to the User: "Imported 48 tracks (40 new, 8 updated, 0 failed, 2 blocked/skipped). 48 tracks queued for BPM/key analysis."
12. User sees the imported tracks in their catalog with BPM and key columns showing "Pending analysis"

## Extensions (What Can Go Wrong)

- **3a. User enters an invalid URL (not a SoundCloud playlist/set)**:
  1. System displays "That doesn't look like a SoundCloud playlist URL. Expected format: https://soundcloud.com/username/sets/..."
  2. Returns to step 3

- **3b. User enters a single SoundCloud track URL**:
  1. System detects it's a single track URL
  2. Backend fetches just that one track via resolve endpoint
  3. Continues from step 8 with a single track
  4. Summary: "Imported 1 track. Queued for BPM/key analysis."

- **3c. User enters a SoundCloud user profile URL (not a playlist)**:
  1. System displays "That looks like a user profile, not a playlist. Please paste a playlist or set URL. (Tip: find playlists at soundcloud.com/username/sets)"
  2. Returns to step 3

- **4a. URL format is valid but playlist doesn't exist (404)**:
  1. SoundCloud API returns 404
  2. System displays "Playlist not found. It may have been deleted or set to private."
  3. Returns to step 3

- **4b. Playlist is private and app credentials lack access**:
  1. SoundCloud API returns 403
  2. System displays "This playlist is private. Only public playlists can be imported."
  3. Returns to step 3

- **5a. App-level OAuth credentials are invalid**:
  1. SoundCloud returns 401
  2. System logs a critical error (admin notification)
  3. System displays "SoundCloud integration is temporarily unavailable. Please try later."
  4. Use case fails

- **5b. OAuth token request fails (network error)**:
  1. System retries up to 3 times with exponential backoff (1s, 2s, 4s)
  2. If all retries fail, displays "Could not connect to SoundCloud. Please check your connection."
  3. Use case fails

- **6a. Resolve endpoint returns unexpected response**:
  1. System logs the response for debugging
  2. System displays "Could not process this SoundCloud URL. Please try a different playlist."
  3. Returns to step 3

- **7a. SoundCloud API returns 429 (rate limited)**:
  1. System reads `Retry-After` header
  2. System waits, then retries
  3. If rate-limited more than 3 times consecutively, displays "SoundCloud is temporarily limiting requests. Imported X tracks so far."
  4. Partial import is committed
  5. Returns to step 7 (retry) or use case fails

- **7b. SoundCloud API returns 500/502/503 (server error)**:
  1. System retries up to 3 times with exponential backoff
  2. If all retries fail, displays "SoundCloud is experiencing issues. Imported X of Y tracks."
  3. Partial import is committed

- **7c. Network timeout during pagination**:
  1. System retries up to 3 times with exponential backoff
  2. Partial import is committed
  3. Returns to step 3

- **8a. Track has no artwork**:
  1. System stores `null` for artwork_url
  2. Frontend shows placeholder artwork
  3. Continues to next track

- **8b. Track artist name contains non-Latin characters**:
  1. System stores as-is in UTF-8
  2. Continues to next track

- **9a. All tracks in playlist are blocked**:
  1. System displays "All tracks in this playlist are blocked from streaming and cannot be imported."
  2. Returns to step 3

- **9b. Some tracks are 'preview' access level (partial stream)**:
  1. System imports them normally — preview access is sufficient for metadata
  2. Marks in summary: "X tracks have preview-only access"

- **10a. Database transaction fails for a track**:
  1. Transaction rolls back for that track only
  2. Track counted as "failed" in summary
  3. Continues with next track

- **10b. Track URN conflicts with existing track from a previous import**:
  1. Upsert updates metadata (normal behavior)
  2. `needs_analysis` flag preserved if analysis hasn't run yet; cleared if analysis already completed

- **12a. Catalog view fails to load after import**:
  1. System displays "Import completed but catalog couldn't load. Pull to refresh."
  2. Data is persisted

- **Xa. Stream URL is HLS format (.m3u8) instead of direct audio**:
  1. System downloads the HLS stream segments and concatenates to a temporary audio file
  2. Temporary file is used for analysis, then deleted
  3. If HLS download fails, track is flagged `analysis_error = 'hls_download_failed'`

- **Xb. Track is from an underground/unlisted artist with restricted API access**:
  1. SoundCloud's public API may not return all tracks — some artists restrict API access
  2. System imports what's available and notes: "X tracks from this playlist could not be accessed"
  3. This is a known limitation of app-level OAuth (no user auth = no private content)

- **Xc. SoundCloud resolve endpoint returns unexpected resource type**:
  1. User pastes a URL that resolves to a user profile, not a playlist/track
  2. System checks the `kind` field in the resolve response
  3. If kind is not 'playlist', 'system-playlist', or 'track', displays: "This URL is a [kind]. Please paste a playlist or track URL."

## Variations

- **V1. Single Track Import**: User pastes a direct SoundCloud track URL. System detects it's a track, fetches and imports it. (Handled by extension 3b.)
- **V2. Re-import**: User imports the same playlist again. All tracks are upserted — metadata updated, no duplicates. Analysis flags preserved.
- **V3. Large Playlist (500+ tracks)**: Same flow with progress indicator: "Importing track X of Y..." — pagination handles automatically.

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test soundcloud_import`
- **Test File**: `backend/tests/soundcloud_import.rs`
- **Depends On**: UC-001 (shared infrastructure), UC-013 (MusicSourceClient trait, migration 003)
- **Blocks**: UC-015 (needs tracks flagged for analysis), UC-016 (setlist needs multi-source catalog)
- **Estimated Complexity**: M (~2000 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend — Implement `SoundCloudClient` (MusicSourceClient trait), resolve endpoint, pagination, Axum route handlers
  - Teammate:Frontend — Update import screen with SoundCloud source option, "Pending analysis" display for BPM/key

### Key Implementation Details
- **MusicSourceClient trait**: Reuse trait from UC-013, implement `SoundCloudClient`
- **SoundCloud API base**: `https://api.soundcloud.com/`
- **Auth**: OAuth 2.1 client credentials, `Authorization: OAuth ACCESS_TOKEN` on all requests
- **Resolve**: `GET /resolve?url={user_url}` → returns resource with URN
- **Pagination**: Cursor-based via `linked_partitioning=true` and `next_href` field. Maximum 50 pages (5000 tracks) per import to prevent runaway imports. If a playlist exceeds this limit, system imports the first 5000 tracks and warns the user.
- **Identity**: Use `urn` field as primary identifier (NOT `id` — deprecated by June 2025). SoundCloud is migrating from numeric `id` to `urn` format (e.g., `soundcloud:tracks:123456`). Store both `soundcloud_id` (legacy numeric) and `soundcloud_urn` (new format) during the transition period. The urn field takes precedence for API lookups when available.
- **Stream access**: Check `access` field — values: `playable`, `preview`, `blocked`. Store `stream_access_level` on each track ('full', 'preview', 'none') from SoundCloud's `access` field. This tells UC-015 and UC-019 what audio quality is available without re-checking the API.
- **Migration**: Add `soundcloud_urn TEXT UNIQUE` and `needs_analysis BOOLEAN DEFAULT false` to tracks table (can be part of migration 003 or 004)
- **HLS Stream Handling**: SoundCloud is migrating to HLS (HTTP Live Streaming) with AAC format. Stream URLs may be `.m3u8` playlist files, not direct MP3s. For UC-015 analysis: system must download the HLS stream to a temporary file before sending to essentia.
- **needs_analysis Conditional Upsert**: When upserting a track that already exists (same `soundcloud_urn`), only set `needs_analysis = true` if the track doesn't already have valid BPM/key data. This prevents re-analysis of tracks that were previously analyzed successfully.

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] Code passes quality gates (`cargo fmt --check && cargo clippy -- -D warnings && cargo test` + `flutter analyze && flutter test`)
- [ ] `SoundCloudClient` implements `MusicSourceClient` trait
- [ ] SoundCloud playlist URL import works end-to-end (wiremock tests with SoundCloud API response fixtures)
- [ ] Tracks are imported with `needs_analysis = true` flag
- [ ] Blocked tracks are skipped with accurate count in summary
- [ ] `urn` field is used as primary identifier (not `id`)
- [ ] Duplicate tracks (same soundcloud_urn) are upserted without creating duplicates
- [ ] Rate limiting (429) and server errors (5xx) handled with retry + exponential backoff
- [ ] Partial imports committed on failure (per-track transactions)
- [ ] Frontend shows "Pending analysis" for BPM/key columns on SoundCloud tracks
- [ ] Import summary shows accurate counts including analysis queue count
