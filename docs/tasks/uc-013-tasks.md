# Task Decomposition: UC-013 Import Tracks from Beatport

**Source**: `docs/use-cases/uc-013-import-tracks-from-beatport.md`
**Review Status**: Reviewed (devil's advocate), all blockers fixed
**Total Tasks**: 10 implementation tasks
**Pre-Sprint Infrastructure**: T1–T3 (shared by UC-013/014, must complete first)

---

## Dependency Graph

```
Wave 1 (all independent — start immediately):
  T1 (Migration 003) ──────────────────────────┐
  T2 (Camelot module) ─────────────────────┐    │
  T3 (MusicSourceClient trait) ─────┐      │    │
                                    │      │    │
Wave 2 (after Wave 1):              │      │    │
  T4 (DB models + repo) ───────────[T1]    │    │
  T5 (BeatportClient API) ────────[T3]     │    │
  T6 (AppConfig + creds) ──────────┤       │    │
                                   │       │    │
Wave 3 (after Wave 2):             │       │    │
  T7 (Import service + route) ───[T2,T4,T5,T6]  │
                                   │             │
Wave 4 (after Wave 3):             │             │
  T8 (Frontend import screen) ───[T7]            │
  T9 (BPM/Key catalog display) ────────────────[T4]
  T10 (Integration tests) ────────[T7]
```

**Critical path**: T1 → T4 → T7 → T10
**Parallel tracks**:
- Track A (API): T3 → T5 → T7
- Track B (Data): T1 → T4 → T7
- Track C (Pure): T2 (feeds into T7)
- Track D (Frontend): T8, T9 (after backend ready)

---

## Tasks

### T1: Migration 003 — DJ Metadata + Source-Agnostic Fields

**Module**: `backend/migrations/003_dj_metadata.sql`
**Covers**: Precondition 4, Postconditions 1/2/6/7, UC-013 Key Implementation Details
**Size**: S | **Risk**: Low | **Agent**: Teammate:Backend-2

**Description**:
Create migration 003 adding DJ metadata columns and source-agnostic infrastructure:

1. **Tracks table** — add columns (all `DEFAULT NULL` for backwards compat):
   - `bpm REAL` — beats per minute (Beatport native, essentia-derived, or null)
   - `musical_key TEXT` — standard notation e.g. "A minor"
   - `camelot_key TEXT` — Camelot wheel notation e.g. "8A"
   - `genre TEXT` — primary genre
   - `sub_genre TEXT` — sub-genre
   - `label TEXT` — record label
   - `catalog_number TEXT` — label catalog number
   - `mix_name TEXT` — "Original Mix", "Dub Mix", etc.
   - `remixer TEXT` — remixer name
   - `isrc TEXT` — International Standard Recording Code
   - `beatport_id BIGINT` — Beatport track ID
   - `beatport_slug TEXT` — URL slug for purchase link construction (UC-020)
   - `source TEXT DEFAULT 'spotify'` — origin source identifier

2. **Artists table** — add columns:
   - `beatport_id BIGINT DEFAULT NULL`

3. **Partial unique indexes** (prevent dupes while allowing NULLs):
   ```sql
   CREATE UNIQUE INDEX idx_tracks_beatport_id ON tracks(beatport_id) WHERE beatport_id IS NOT NULL;
   CREATE UNIQUE INDEX idx_artists_beatport_id ON artists(beatport_id) WHERE beatport_id IS NOT NULL;
   ```

4. **Imports table refactor** — make source-agnostic:
   - Rename `spotify_imports` → keep as-is but add `source TEXT NOT NULL DEFAULT 'spotify'`
   - Rename `spotify_playlist_id` → `source_playlist_id` (or add new column + backfill)
   - Rename `spotify_playlist_name` → `source_playlist_name`
   - **Note**: SQLite ALTER TABLE limitations may require a create-copy-drop-rename approach for renames

**Acceptance**:
- [ ] Migration applies cleanly on existing dev database with Spotify data
- [ ] Existing Spotify tracks unaffected (new columns are NULL, source defaults to 'spotify')
- [ ] `cargo test` passes (existing tests unbroken)
- [ ] Partial unique indexes created correctly

**Blocked by**: Nothing
**Blocks**: T4, T9

---

### T2: Camelot Key Conversion Module

**Module**: `backend/src/camelot.rs` (new file)
**Covers**: MSS step 8, Postcondition 6, Extensions 7d/8a
**Size**: S | **Risk**: Low | **Agent**: Teammate:Backend-2

**Description**:
Pure Rust module with a 24-entry lookup table mapping standard musical keys to Camelot wheel notation. No external dependencies.

1. **Function**: `fn to_camelot(key: &str, scale: &str) -> Option<String>`
   - `key`: note name (e.g., "A", "Bb", "F#")
   - `scale`: "major" or "minor"
   - Returns Camelot notation (e.g., "8A" for A minor, "11B" for A major)
   - Returns `None` for unrecognized input

2. **Alternative function**: `fn key_string_to_camelot(key_string: &str) -> Option<String>`
   - Parses combined format like "A minor", "Bb major", "F# minor"
   - Normalizes common variants: "Am" → "A minor", "Abm" → "Ab minor"

3. **Compatibility scoring** (for UC-017):
   - `fn camelot_distance(a: &str, b: &str) -> Option<u8>` — number of steps on the wheel
   - `fn are_compatible(a: &str, b: &str) -> bool` — same key, adjacent, or relative major/minor

4. **Lookup table** (24 keys):
   ```
   1A = Ab minor    1B = B major
   2A = Eb minor    2B = F# major
   3A = Bb minor    3B = Db major
   4A = F minor     4B = Ab major
   5A = C minor     5B = Eb major
   6A = G minor     6B = Bb major
   7A = D minor     7B = F major
   8A = A minor     8B = C major
   9A = E minor     9B = G major
   10A = B minor    10B = D major
   11A = F# minor   11B = A major
   12A = Db minor   12B = E major
   ```

**Acceptance**:
- [ ] All 24 key mappings tested
- [ ] Handles enharmonic equivalents (Db = C#, Bb = A#)
- [ ] Returns `None` for invalid input (not panic)
- [ ] `camelot_distance` returns correct step counts
- [ ] `cargo clippy -- -D warnings` passes
- [ ] ~15 unit tests

**Blocked by**: Nothing
**Blocks**: T5, T7

---

### T3: MusicSourceClient Trait + SourceTrack Definition

**Module**: `backend/src/api/source.rs` (new file)
**Covers**: Postcondition 8, shared infrastructure for UC-014+
**Size**: S | **Risk**: Low | **Agent**: Teammate:Backend-1

**Description**:
Define the source-agnostic interface that all music source clients implement. This is the shared contract between Beatport (UC-013), SoundCloud (UC-014), and potentially future sources.

```rust
use async_trait::async_trait;

/// Auth context — each source fills the variant it needs.
#[derive(Clone)]
pub enum SourceAuth {
    ClientCredentials { access_token: String },
    OAuth { access_token: String, refresh_token: Option<String> },
}

/// Normalized track returned by any music source.
#[derive(Debug, Clone)]
pub struct SourceTrack {
    pub source_id: String,         // Beatport track ID, Spotify URI, SoundCloud URN
    pub source_slug: Option<String>, // URL slug (Beatport)
    pub title: String,
    pub mix_name: Option<String>,
    pub artists: Vec<SourceArtist>,
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
    pub artwork_url: Option<String>,
    pub preview_url: Option<String>,
    pub permalink_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SourceArtist {
    pub source_id: String,
    pub name: String,
}

/// Errors that any source client can return.
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Access denied: {0}")]
    AccessDenied(String),
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("Auth failed: {0}")]
    AuthFailed(String),
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

#[async_trait]
pub trait MusicSourceClient: Send + Sync {
    /// Human-readable source name (e.g., "Beatport", "SoundCloud")
    fn source_name(&self) -> &str;

    /// Source identifier for DB storage (e.g., "beatport", "soundcloud")
    fn source_key(&self) -> &str;

    /// Import all tracks from a playlist/chart URL
    async fn import_playlist(
        &self, url: &str, auth: &SourceAuth
    ) -> Result<Vec<SourceTrack>, SourceError>;

    /// Import a single track by URL
    async fn import_track(
        &self, url: &str, auth: &SourceAuth
    ) -> Result<SourceTrack, SourceError>;
}
```

Also wire the new module: add `pub mod source;` to `backend/src/api/mod.rs`.

**Acceptance**:
- [ ] Trait compiles with `cargo check`
- [ ] All types derive necessary traits (Debug, Clone)
- [ ] SourceError variants cover all error cases from UC-013 extensions
- [ ] Module is wired into `api/mod.rs`

**Blocked by**: Nothing
**Blocks**: T5

---

### T4: Extend DB Models + Repository for Multi-Source Support

**Module**: `backend/src/db/models.rs`, `backend/src/db/tracks.rs`, `backend/src/db/artists.rs`, `backend/src/db/imports.rs`, `backend/src/services/import.rs`, `backend/src/repo.rs`
**Covers**: Postconditions 1-5, MSS step 9, Extensions 9a/9b
**Size**: M | **Risk**: Medium | **Agent**: Teammate:Backend-2

**Description**:
Extend existing DB layer to support DJ metadata and multi-source imports. **Critical**: Must not break existing Spotify import functionality.

1. **Update `db/models.rs`** — extend `Track` struct:
   ```rust
   pub struct Track {
       // ... existing fields ...
       pub bpm: Option<f64>,
       pub musical_key: Option<String>,
       pub camelot_key: Option<String>,
       pub genre: Option<String>,
       pub sub_genre: Option<String>,
       pub label: Option<String>,
       pub catalog_number: Option<String>,
       pub mix_name: Option<String>,
       pub remixer: Option<String>,
       pub isrc: Option<String>,
       pub beatport_id: Option<i64>,
       pub beatport_slug: Option<String>,
       pub source: Option<String>,
   }
   ```
   Extend `Artist`:
   ```rust
   pub struct Artist {
       // ... existing fields ...
       pub beatport_id: Option<i64>,
   }
   ```
   Add/update `Import` model to use source-agnostic fields (source, source_playlist_id, source_playlist_name).

2. **Update `db/tracks.rs`** — extend `upsert_track()`:
   - Add new columns to INSERT and ON CONFLICT UPDATE
   - Add `upsert_track_by_beatport_id()` — upsert keyed on `beatport_id` with `source='beatport'`
   - Keep existing `upsert_track()` (by spotify_uri) working for backwards compat

3. **Update `db/artists.rs`** — extend `upsert_artist()`:
   - Add `upsert_artist_by_beatport_id()` — upsert keyed on `beatport_id`

4. **Update `db/imports.rs`** — source-agnostic:
   - `create_import()` now takes a `source` parameter
   - Queries use renamed columns if migration renames them, otherwise add new columns

5. **Update `services/import.rs`** — extend `ImportRepository` trait:
   - Add methods for Beatport-keyed upserts (or make existing upsert methods source-aware)
   - Extend `TrackRecord` to carry DJ metadata fields
   - Ensure `ImportSummary` works for any source

6. **Update `repo.rs`** — `SqliteImportRepository`:
   - Implement new/updated trait methods

**Acceptance**:
- [ ] Existing Spotify import tests still pass (zero regressions)
- [ ] New upsert_track_by_beatport_id() inserts and updates correctly
- [ ] Per-track transaction isolation works for Beatport imports
- [ ] Import record stores source='beatport'
- [ ] `cargo test` passes — all 49 existing tests still green

**Blocked by**: T1 (needs migration)
**Blocks**: T7, T9

---

### T5: BeatportClient — OAuth + API Fetching

**Module**: `backend/src/api/beatport.rs` (new file)
**Covers**: MSS steps 5-7, Extensions 3a/3b/4a/4b/5a/5b/6a/6b/6c/7a-7d/Xa/Xb/Xc
**Size**: M | **Risk**: Medium | **Agent**: Teammate:Backend-1

**Description**:
Implement the `MusicSourceClient` trait for Beatport API v4. Uses client credentials OAuth (not user-level).

1. **`BeatportClient` struct**:
   ```rust
   pub struct BeatportClient {
       http: reqwest::Client,
       base_url: String,     // https://api.beatport.com/v4/
       auth_url: String,     // https://api.beatport.com/v4/auth/o/token/
       client_id: String,
       client_secret: String,
       token_cache: Arc<Mutex<Option<CachedToken>>>,
   }
   ```

2. **OAuth client credentials flow** (MSS step 5):
   - `POST /v4/auth/o/token/` with `grant_type=client_credentials`
   - Cache token until expiry (1hr default)
   - Auto-refresh on 401 (Extension Xa: mid-import token refresh)

3. **URL validation** (MSS step 4):
   - `validate_url(url: &str) -> Result<BeatportResource>` where `BeatportResource` is:
     - `Chart { id: u64 }` — `/chart/{slug}/{id}`
     - `Playlist { id: u64 }` — `/playlist/{slug}/{id}`
     - `Track { id: u64 }` — `/track/{slug}/{id}` (Extension 3b)
   - Reject invalid URLs with descriptive error (Extension 3a)

4. **Chart/Playlist fetching** (MSS step 6):
   - `GET /v4/catalog/charts/{id}/tracks?page=1&per_page=100`
   - `GET /v4/catalog/playlists/{id}/tracks?page=1&per_page=100`
   - Offset pagination: loop page 1..10 (max 1000 tracks)
   - Use `retry_with_backoff()` from existing `api/retry.rs`

5. **Track fetching** (Extension 3b):
   - `GET /v4/catalog/tracks/{id}`
   - Returns single SourceTrack

6. **Response parsing** (MSS step 7):
   - Map Beatport JSON → `SourceTrack` (via `SourceTrack` from T3)
   - Extract: name, mix_name, artists[], remixers[], bpm, key, genre, sub_genre, label, release_date, duration, isrc, beatport_id, slug
   - Handle missing fields gracefully (Extensions 7a-7d, Xb)
   - Convert musical key to standard format for Camelot conversion

7. **Error handling**:
   - 401 invalid_client → `SourceError::AuthFailed` (Extension 5a)
   - 403 → `SourceError::AccessDenied` (Extension Xc)
   - 404 → `SourceError::NotFound` (Extension 4a)
   - 429 → `SourceError::RateLimited` (Extension 6a)
   - 5xx → `SourceError::ServerError` (Extension 6b)

8. Wire module: add `pub mod beatport;` to `backend/src/api/mod.rs`.

**Testing**: Use `with_base_url()` pattern (same as SpotifyClient) for wiremock tests.

**Acceptance**:
- [ ] Implements `MusicSourceClient` trait
- [ ] OAuth token caching works (1 token request per import, not per API call)
- [ ] URL validation handles chart, playlist, and track URLs
- [ ] Response parsing handles all field presence/absence combinations
- [ ] Retry logic integrated (429, 5xx, network errors)
- [ ] Unit tests with wiremock: auth flow, track fetch, pagination, error cases
- [ ] ~12 unit tests

**Blocked by**: T2 (Camelot for key conversion), T3 (MusicSourceClient trait)
**Blocks**: T7

---

### T6: AppConfig — Beatport Credentials

**Module**: `backend/src/config.rs`, `backend/src/main.rs`
**Covers**: Precondition 2, Invariant 1
**Size**: S | **Risk**: Low | **Agent**: Teammate:Backend-1

**Description**:
Extend `AppConfig` to load Beatport API credentials from environment:

1. Add to `AppConfig`:
   ```rust
   pub beatport_client_id: Option<String>,     // BEATPORT_CLIENT_ID
   pub beatport_client_secret: Option<String>,  // BEATPORT_CLIENT_SECRET
   ```
   Optional because Beatport may not be configured in all environments.

2. Add to `ImportState` (or create new `BeatportState`):
   - Store `BeatportClient` instance (lazily initialized)
   - Handle case where credentials aren't configured → Beatport import returns "Beatport integration not configured"

3. Update `.env.example` with new variables.

**Acceptance**:
- [ ] Config loads from env without breaking existing Spotify config
- [ ] Missing Beatport creds don't crash the server (graceful degradation)
- [ ] Credentials never logged or exposed in error responses

**Blocked by**: Nothing
**Blocks**: T7

---

### T7: Import Service + Route — Beatport Integration

**Module**: `backend/src/services/import.rs`, `backend/src/routes/import.rs`, `backend/src/main.rs`
**Covers**: MSS steps 1-11, Postcondition 5 (summary), Extensions 9a/9b
**Size**: L | **Risk**: Medium | **Agent**: Teammate:Backend-1 (with Backend-2 wiring)

**Description**:
Wire BeatportClient into the import orchestration and expose as an HTTP endpoint. This is the integration task that connects T2–T6.

1. **Import orchestration** (`services/import.rs`):
   - New function: `import_from_beatport(repo, client, user_id, url) -> Result<ImportSummary, ImportError>`
   - OR refactor to generic: `import_from_source(repo, client: &dyn MusicSourceClient, user_id, url, auth) -> Result<ImportSummary, ImportError>`
   - Flow:
     a. Validate URL via `BeatportClient::validate_url()`
     b. Create import record with `source='beatport'`
     c. Authenticate via client credentials
     d. Fetch tracks via `import_playlist()` or `import_track()`
     e. For each `SourceTrack`:
        - Convert key to Camelot via `camelot::key_string_to_camelot()`
        - Build `TrackRecord` with DJ metadata
        - Upsert track + artists in a single transaction
        - Count inserted/updated/failed
     f. Complete import record with final counts
     g. Return `ImportSummary`

2. **Route handler** (`routes/import.rs`):
   - `POST /api/import/beatport` — JSON body: `{ "url": "https://www.beatport.com/chart/..." }`
   - Auth: `X-User-Id` header (same pattern as Spotify)
   - Error mapping: SourceError → AppError → HTTP status + JSON
   - Response: `{ "import_id": "...", "total": 100, "inserted": 85, "updated": 15, "failed": 0 }`

3. **Router wiring** (`main.rs`):
   - Add beatport route to import router
   - Pass BeatportClient via state

4. **Map ImportError ↔ SourceError**:
   - `SourceError::InvalidUrl` → 400
   - `SourceError::NotFound` → 404
   - `SourceError::AccessDenied` → 403
   - `SourceError::RateLimited` → 429
   - `SourceError::AuthFailed` → 503 (server-side config issue)
   - `SourceError::ServerError` → 502

**Acceptance**:
- [ ] `POST /api/import/beatport` with valid chart URL returns 200 + import summary
- [ ] Invalid URL returns 400 with descriptive message
- [ ] Missing Beatport creds returns 503 "Beatport not configured"
- [ ] Partial import committed on mid-way failure
- [ ] Import record created with source='beatport'
- [ ] Camelot keys computed and stored for tracks with musical key data
- [ ] Existing Spotify import endpoint unaffected

**Blocked by**: T2, T4, T5, T6
**Blocks**: T8, T10

---

### T8: Frontend — Multi-Source Import Screen

**Module**: `frontend/lib/screens/import_screen.dart` (refactored from `spotify_import_screen.dart`), `frontend/lib/providers/import_provider.dart`, `frontend/lib/services/api_client.dart`
**Covers**: MSS steps 1-3, 10-11, Extensions 2a/3a
**Size**: M | **Risk**: Low | **Agent**: Teammate:Frontend

**Description**:
Refactor the import screen to support multiple sources with a tab/selector UI.

1. **Source selector** — TabBar or SegmentedButton at top:
   - Spotify (existing flow)
   - Beatport (new)
   - SoundCloud (disabled/coming soon — for UC-014)

2. **Beatport import tab**:
   - Text field: "Paste Beatport chart or playlist URL"
   - Hint text with example URL
   - Client-side URL validation (regex for beatport.com/chart/ or /playlist/ or /track/)
   - Import button → `POST /api/import/beatport`
   - Progress indicator (same pattern as Spotify)
   - Summary display (same ImportSummary widget, but with BPM/key note)

3. **Provider** (`import_provider.dart`):
   - Extend or generalize the Spotify import provider to handle multiple sources
   - `importFromBeatport(url)` method
   - Same state machine: idle → validating → importing → completed/error

4. **API client** (`api_client.dart`):
   - Add `importBeatport(String url)` method → `POST /api/import/beatport`

5. **Routing**: Update `GoRouter` if screen path changes.

**Acceptance**:
- [ ] Source selector renders Spotify and Beatport tabs
- [ ] Beatport URL validation rejects non-Beatport URLs
- [ ] Import flow works: paste URL → import → summary
- [ ] Spotify import still works (no regressions)
- [ ] Widget tests for source selector and Beatport tab
- [ ] `flutter analyze` passes

**Blocked by**: T7 (needs backend endpoint shape)
**Blocks**: Nothing

---

### T9: Frontend — BPM/Key Display in Catalog

**Module**: `frontend/lib/models/track.dart`, `frontend/lib/widgets/track_tile.dart`
**Covers**: MSS step 11 ("BPM and Camelot key columns visible"), Postcondition 6
**Size**: S | **Risk**: Low | **Agent**: Teammate:Frontend

**Description**:
Update the Track model and catalog UI to display DJ metadata.

1. **Update `Track` model** (`models/track.dart`):
   - Add nullable fields: `bpm`, `musicalKey`, `camelotKey`, `genre`, `label`, `mixName`, `source`
   - Update `fromJson()` factory

2. **Update `TrackTile` widget** (`widgets/track_tile.dart`):
   - Show BPM (e.g., "128.0") if available
   - Show Camelot key (e.g., "8A") if available, color-coded by wheel position
   - Show source badge: small chip showing "Spotify" or "Beatport"
   - Gracefully handle null values (show "—" or hide the field)

3. **Camelot color mapping** (client-side):
   - Simple map: 1A/1B = red, 2A/2B = red-orange, ..., 12A/12B = pink
   - Used for visual key compatibility at a glance

**Acceptance**:
- [ ] Track model deserializes new fields from backend JSON
- [ ] BPM displays correctly (1 decimal place)
- [ ] Camelot key displays with appropriate color
- [ ] Null fields shown as "—" (not crash)
- [ ] Source badge renders for both Spotify and Beatport
- [ ] Widget test for TrackTile with DJ metadata
- [ ] `flutter analyze` passes

**Blocked by**: T4 (needs updated Track shape from backend)
**Blocks**: Nothing

---

### T10: Integration Tests — Full Beatport Import Flow

**Module**: `backend/tests/beatport_import.rs`
**Covers**: All MSS steps, all extensions, all postconditions
**Size**: M | **Risk**: Medium | **Agent**: Teammate:Backend-1

**Description**:
Integration tests using `wiremock` to mock Beatport API v4 and in-memory SQLite for database.

1. **Happy path**: Mock 50-track chart → import → verify 50 rows in DB with correct BPM, key, Camelot, genre, label, mix_name, source='beatport'
2. **Re-import (upsert)**: Import same chart twice → verify no duplicates, updated counts
3. **Single track import**: Mock single track URL → verify 1 row imported
4. **Camelot conversion**: Verify musical key → Camelot mapping for imported tracks
5. **Missing BPM/key**: Mock track with null BPM/key → verify stored as NULL, no crash
6. **Rate limiting (429)**: Mock 429 + Retry-After → verify retry, then success
7. **Server error (5xx)**: Mock 500 → verify retry with backoff
8. **Partial failure**: Mock 200 for page 1, 500 for page 2 → verify page 1 tracks persisted
9. **Invalid URL**: Various bad inputs → verify 400 errors
10. **404 chart**: Mock 404 → verify error message
11. **403 private playlist**: Mock 403 → verify error message
12. **OAuth token refresh**: Mock expired token → 401 → refresh → retry → success
13. **Empty chart**: Mock 0 tracks → verify appropriate response
14. **No title**: Mock track with missing title → verify fallback "{artist} - Untitled"

**Acceptance**:
- [ ] All 14 test scenarios pass
- [ ] Tests run in <30 seconds (wiremock + in-memory SQLite)
- [ ] No flaky tests (all deterministic with mocks)
- [ ] `cargo test --test beatport_import` passes
- [ ] Existing `cargo test` (all 49 tests) still passes

**Blocked by**: T7 (needs full backend stack)
**Blocks**: Nothing (final verification)

---

## Summary

| Task | Description | Size | Risk | Blocked By | Blocks |
|------|-------------|------|------|------------|--------|
| **T1** | Migration 003: DJ metadata + source-agnostic fields | S | Low | — | T4, T9 |
| **T2** | Camelot key conversion module (pure Rust) | S | Low | — | T5, T7 |
| **T3** | MusicSourceClient trait + SourceTrack definition | S | Low | — | T5 |
| **T4** | DB models + repository for multi-source | M | Med | T1 | T7, T9 |
| **T5** | BeatportClient: OAuth + API + response parsing | M | Med | T2, T3 | T7 |
| **T6** | AppConfig: Beatport credentials | S | Low | — | T7 |
| **T7** | Import service + route: Beatport integration | L | Med | T2,T4,T5,T6 | T8, T10 |
| **T8** | Frontend: multi-source import screen | M | Low | T7 | — |
| **T9** | Frontend: BPM/key display in catalog | S | Low | T4 | — |
| **T10** | Integration tests: full Beatport import | M | Med | T7 | — |

**Estimated total**: ~20 new backend tests + 14 integration tests + ~5 frontend tests = ~39 tests
**Current count**: 49 backend + 1 frontend = 50
**After UC-013**: ~84 backend + ~6 frontend = ~90 tests

---

## Recommended Agent Team

| Agent | Track | Tasks | Rationale |
|-------|-------|-------|-----------|
| **Backend-1** (API) | API + Integration | T3 → T5 → T6 → T7 → T10 | Owns BeatportClient and wiring |
| **Backend-2** (Data) | Data + Infra | T1 → T2 → T4 | Owns migration, Camelot, DB layer |
| **Frontend** | UI | T8, T9 | After backend ready |

**Wave execution**:
- Wave 1: Backend-1 does T3 + T6, Backend-2 does T1 + T2 (all parallel, ~30 min)
- Wave 2: Backend-1 does T5, Backend-2 does T4 (~45 min)
- Wave 3: Backend-1 does T7 (Backend-2 assists with wiring review) (~45 min)
- Wave 4: Backend-1 does T10, Frontend does T8 + T9 (parallel, ~45 min)

---

## Pre-Implementation Checklist

Before starting implementation:
- [ ] Feature branch: `git checkout -b feature/uc-013-beatport-import`
- [ ] Run `cargo check` to warm dependency cache
- [ ] Verify dev database exists and migrations 001-002 are applied
- [ ] Review Beatport API v4 docs (confirm endpoint paths, response shapes)
- [ ] Prepare wiremock fixtures from sample Beatport API responses
- [ ] **design-crit**: Run before building frontend (T8, T9)

## Next Steps

1. `/agent-team-plan 013` — configure the 3-agent team
2. `design-crit` — run for import screen redesign (before T8)
3. Implement waves 1-4
4. `/verify-uc 013` when all tasks complete
5. `/grade-work 013` to score
