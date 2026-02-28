# Use Case: UC-013 Import Tracks from Beatport

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P0 Critical
- **Complexity**: 🟡 Medium

## Actors
- **Primary Actor**: App User (authenticated)
- **Supporting Actors**:
  - Beatport API v4 (OAuth, app-level credentials)
  - Database (SQLite/PostgreSQL via SQLx)
- **Stakeholders & Interests**:
  - DJ User: Wants accurate BPM, key, genre, and label metadata imported directly from Beatport — the gold standard for DJ track data
  - Developer: Wants the `MusicSourceClient` trait established here as the pattern for all future source integrations (SoundCloud, etc.)
  - Curator: Wants Beatport charts and playlists importable for building DJ setlists

## Conditions
- **Preconditions** (must be true before starting):
  1. User is authenticated in the app
  2. Backend has valid Beatport API v4 credentials (client ID + secret) configured via environment (requires approved developer application)
  3. Database is initialized with tracks/artists/track_artists schema (migrations 001-002 applied)
  4. Migration `003_dj_metadata.sql` is applied (see Key Implementation Details for full column spec)

- **Success Postconditions** (true when done right):
  1. All tracks from the Beatport chart/playlist exist in the `tracks` table with: title, mix_name, album/release, duration_ms, beatport_id, bpm, musical_key, camelot_key, genre, sub_genre, label, remixer, isrc, source='beatport'
  2. All artists from imported tracks exist in the `artists` table with: name, beatport_id
  3. All track-artist relationships exist in the `track_artists` junction table
  4. Duplicate tracks (same beatport_id) are upserted — metadata updated, no duplicate rows created
  5. The import operation returns a summary: total tracks found, new tracks inserted, existing tracks updated, failures (if any)
  6. Musical keys are stored in both standard notation (e.g., "A minor") and Camelot notation (e.g., "8B")
  7. An import record is stored linking the Beatport chart/playlist ID to the user and import timestamp
  8. The `MusicSourceClient` trait is defined and `BeatportClient` implements it

- **Failure Postconditions** (true when it fails gracefully):
  1. If the import fails mid-way, all tracks successfully fetched up to the failure point are persisted (per-track transactions)
  2. User receives a clear error message describing what went wrong
  3. Rate limit errors are communicated with a retry-after suggestion
  4. If Beatport API is unreachable, user is told to try again later

- **Invariants** (must remain true throughout):
  1. Beatport API credentials are never exposed to the frontend
  2. All Beatport API calls happen on the backend — frontend never calls Beatport directly
  3. Database remains in a consistent state throughout (per-track transactions)
  4. Existing tracks from other sources (Spotify) are never modified by Beatport import

## Main Success Scenario
1. User navigates to the "Import" screen in the Flutter app
2. User selects "Beatport" as the import source
3. User pastes a Beatport chart or playlist URL (e.g., `https://www.beatport.com/chart/top-100-techno/12345`)
4. System validates the URL format and extracts the chart/playlist type and ID
5. Backend authenticates with Beatport API v4 using app-level OAuth credentials (client credentials flow)
6. Backend calls Beatport API v4 to fetch the chart/playlist tracks, paginating with `page` and `per_page` parameters until all tracks are retrieved
7. For each track in the response, Backend extracts: name, mix_name, artists[], remixers[], bpm, key (musical key object), genre, sub_genre, label, release_date, length (duration), isrc, beatport track id
8. System converts each track's musical key to Camelot notation using the key-to-Camelot mapping table
9. For each track, Backend performs a single transaction: upserts the track into `tracks` (keyed on beatport_id with source='beatport'), upserts each artist into `artists` (keyed on beatport_id), and upserts track-artist relationships. If any part fails, the transaction rolls back for that track only.
10. System displays an import summary to the User: "Imported 100 tracks (85 new, 15 updated, 0 failed) — BPM and key data included"
11. User sees the imported tracks in their catalog with BPM and Camelot key columns visible

## Extensions (What Can Go Wrong)

- **2a. Import source selector not available (feature flag off or UI error)**:
  1. System shows only previously available sources (Spotify)
  2. User cannot proceed with Beatport import
  3. Use case fails

- **3a. User enters an invalid URL (not a Beatport chart/playlist)**:
  1. System displays "That doesn't look like a Beatport chart or playlist URL. Expected format: https://www.beatport.com/chart/... or https://www.beatport.com/playlist/..."
  2. Returns to step 3

- **3b. User enters a Beatport track URL (single track, not chart/playlist)**:
  1. System detects it's a single track URL, extracts track ID
  2. Backend fetches just that one track
  3. Continues from step 7 with a single track
  4. Summary: "Imported 1 track"

- **4a. Chart/playlist ID is valid format but doesn't exist (404)**:
  1. Beatport API returns 404
  2. System displays "Chart not found. It may have been removed or the URL may be incorrect."
  3. Returns to step 3

- **4b. Chart/playlist is empty (0 tracks)**:
  1. System displays "This chart has no tracks. Nothing to import."
  2. Returns to step 3

- **5a. App-level OAuth credentials are invalid or expired**:
  1. Beatport returns 401 with `invalid_client`
  2. System logs a critical error (admin notification)
  3. System displays "Beatport integration is temporarily unavailable. Please try later."
  4. Use case fails

- **5b. OAuth token request fails (network error)**:
  1. System retries up to 3 times with exponential backoff (1s, 2s, 4s)
  2. If all retries fail, system displays "Could not connect to Beatport. Please check your connection and try again."
  3. Use case fails

- **6a. Beatport API returns 429 (rate limited)**:
  1. System reads the `Retry-After` header
  2. System waits the specified duration, then retries the request
  3. If rate-limited more than 3 times consecutively, display "Beatport is temporarily limiting requests. Please try again in a few minutes."
  4. Partial import is committed
  5. Returns to step 6 (retry) or use case fails (exceeded retries)

- **6b. Beatport API returns 500/502/503 (server error)**:
  1. System retries up to 3 times with exponential backoff (1s, 2s, 4s)
  2. If all retries fail, system displays "Beatport is experiencing issues. Imported X of Y tracks so far. You can retry to continue."
  3. Partial import is committed
  4. Returns to step 3 (user can retry for remaining tracks)

- **6c. Network timeout or connection failure during pagination**:
  1. System retries up to 3 times with exponential backoff
  2. System displays "Network error while contacting Beatport. Check your connection and try again."
  3. Partial import is committed
  4. Returns to step 3

- **7a. Track is missing BPM data**:
  1. System stores `null` for bpm
  2. Track is flagged for audio analysis in UC-015
  3. Continues to next track

- **7b. Track is missing musical key data**:
  1. System stores `null` for musical_key and camelot_key
  2. Track is flagged for audio analysis in UC-015
  3. Continues to next track

- **7c. Track has no artist information**:
  1. System creates track with artist set to "Unknown Artist"
  2. Continues to next track

- **7d. Track has unrecognized key format**:
  1. System stores the raw key string in musical_key but sets camelot_key to `null`
  2. Logs a warning for investigation
  3. Continues to next track

- **8a. Key-to-Camelot conversion encounters an unknown key**:
  1. System stores `null` for camelot_key, preserves raw musical_key
  2. Logs warning
  3. Continues to next track

- **9a. Database transaction fails for a track (disk full, connection lost)**:
  1. Transaction rolls back for that track only
  2. Track is counted as "failed" in the import summary
  3. System continues with next track
  4. If all tracks fail, system displays "Failed to save imported tracks. Database error." and logs with full context

- **9b. Duplicate track exists from a different source (same ISRC, different source)**:
  1. System creates a new track row with source='beatport' (tracks are keyed per-source)
  2. Future: UC-018 or a dedup UC can merge cross-source duplicates using ISRC
  3. Continues normally

- **11a. Catalog view fails to load after import**:
  1. System displays "Import completed successfully but we couldn't load the catalog. Pull to refresh or go back and try again."
  2. Data is persisted — only the display failed

- **Xa. OAuth token expires mid-import (large catalog)**:
  1. Client credentials token has 1-hour lifetime
  2. System detects 401 response, refreshes token, retries the failed request
  3. Import continues without losing progress (per-track transactions protect committed work)

- **Xb. Track has no title in Beatport response**:
  1. System uses `"{artist} - Untitled"` as fallback title
  2. Track is still imported with a warning logged

- **Xc. Beatport URL points to another user's private playlist**:
  1. API returns 403 Forbidden
  2. System displays: "This playlist is private. Only public Beatport playlists can be imported."

## Variations

- **V1. Search and Import**: Instead of pasting a URL at step 3, User types a search query (artist name, track title, genre). Backend calls Beatport search API, displays results. User selects tracks to import. Flow continues from step 7 with selected tracks only.
- **V2. Single Track Import**: User pastes a direct Beatport track URL. System detects it's a single track, fetches and imports just that one track. (Handled by extension 3b.)
- **V3. Release Import**: User pastes a Beatport release URL. System imports all tracks from that release.
- **V4. Re-import**: User imports the same chart again. All tracks are upserted — metadata updated from latest Beatport data, no duplicates created.

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test beatport_import`
- **Test File**: `backend/tests/beatport_import.rs`
- **Depends On**: UC-001 (shared DB schema, import infrastructure patterns)
- **Blocks**: UC-016 (setlist generation needs multi-source catalog), UC-018 (DJ metadata enrichment)
- **Estimated Complexity**: M (~2500 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend-1 — Define `MusicSourceClient` trait, implement `BeatportClient`, write migration `003_dj_metadata.sql`, Axum route handlers
  - Teammate:Backend-2 — Camelot key conversion module (pure Rust, no external deps), import repository extensions
  - Teammate:Frontend — Update import screen with Beatport source selector, display BPM/key columns in catalog

### Key Implementation Details
- **MusicSourceClient trait**:
  ```rust
  /// Auth context passed to source methods — each source fills what it needs.
  pub enum SourceAuth {
      ClientCredentials { access_token: String },
      OAuth { access_token: String, refresh_token: Option<String> },
  }

  /// Normalized track returned by any music source.
  pub struct SourceTrack {
      pub source_id: String,       // e.g. Beatport track ID, Spotify track URI
      pub title: String,
      pub mix_name: Option<String>, // "Original Mix", "Dub Mix", etc.
      pub artists: Vec<String>,
      pub album: Option<String>,
      pub duration_ms: Option<i64>,
      pub bpm: Option<f64>,
      pub musical_key: Option<String>,
      pub genre: Option<String>,
      pub sub_genre: Option<String>,
      pub label: Option<String>,
      pub catalog_number: Option<String>,
      pub remixer: Option<String>,
      pub isrc: Option<String>,
      pub release_date: Option<String>,
  }

  #[async_trait]
  pub trait MusicSourceClient: Send + Sync {
      fn source_name(&self) -> &str;
      async fn import_playlist(&self, url: &str, auth: &SourceAuth) -> Result<Vec<SourceTrack>, ImportError>;
      async fn import_track(&self, url: &str, auth: &SourceAuth) -> Result<SourceTrack, ImportError>;
  }
  ```
- **Beatport API v4 base**: `https://api.beatport.com/v4/`
- **Auth**: Client credentials OAuth flow (not user-level)
- **Pagination**: System paginates results using offset pagination (`page=1`, `per_page=100`). Maximum 10 pages (1000 tracks) per import to prevent abuse. Beatport API v4 uses offset pagination with `page` and `per_page` params (not cursor-based).
- **Camelot module**: Pure lookup table, 24 entries, `fn to_camelot(key: &str, scale: &str) -> Option<String>`
- **Migration 003** (`003_dj_metadata.sql`):
  ```sql
  -- Tracks: add DJ metadata columns (all DEFAULT NULL for backwards compat)
  ALTER TABLE tracks ADD COLUMN bpm REAL DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN musical_key TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN camelot_key TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN genre TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN sub_genre TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN label TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN catalog_number TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN mix_name TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN remixer TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN isrc TEXT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN beatport_id BIGINT DEFAULT NULL;
  ALTER TABLE tracks ADD COLUMN source TEXT DEFAULT 'spotify';

  -- Artists: add Beatport ID for dedup on import
  ALTER TABLE artists ADD COLUMN beatport_id BIGINT DEFAULT NULL;

  -- Partial unique indexes: prevent duplicate source IDs while allowing NULLs
  CREATE UNIQUE INDEX idx_tracks_beatport_id ON tracks(beatport_id) WHERE beatport_id IS NOT NULL;
  CREATE UNIQUE INDEX idx_artists_beatport_id ON artists(beatport_id) WHERE beatport_id IS NOT NULL;
  ```
- **Imports table**: The `imports` table must be source-agnostic. Add `source TEXT NOT NULL` column with values: `'spotify'`, `'beatport'`, `'soundcloud'`. Rename any Spotify-specific fields (e.g., `spotify_playlist_id`) to generic equivalents (`source_playlist_id`). This allows all import records to share one table regardless of origin.

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] All extension paths have explicit handling (error types, user messages)
- [ ] No invariant violations detected
- [ ] Code passes quality gates (`cargo fmt --check && cargo clippy -- -D warnings && cargo test` + `flutter analyze && flutter test`)
- [ ] `MusicSourceClient` trait defined with `BeatportClient` implementing it
- [ ] Beatport chart URL import works end-to-end (wiremock tests with Beatport API v4 response fixtures)
- [ ] BPM and musical key are stored correctly and converted to Camelot notation
- [ ] Duplicate tracks (same beatport_id) are upserted without creating duplicates
- [ ] Rate limiting (429) and server errors (5xx) are handled with retry + exponential backoff
- [ ] Partial imports are committed on failure (per-track transactions)
- [ ] Migration 003 adds DJ metadata columns without breaking existing Spotify data
- [ ] Frontend displays BPM and Camelot key in the track catalog view
- [ ] Import summary shows accurate counts (new/updated/failed)
