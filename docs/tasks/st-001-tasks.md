# Task Decomposition: ST-001 Serve Paginated Track Catalog via API

**Source**: `docs/steel-threads/st-001-serve-paginated-track-catalog.md`
**API Contract**: `docs/api/openapi.yaml` (reviewed and locked)
**Review Status**: Steel thread reviewed; API contract finalized
**Total Tasks**: 8 implementation tasks
**Team**: Lead (coordinator) + Backend Teammate + Frontend Teammate

---

## Dependency Graph

```
T1 (migration 003) ──┐
                      ├── T3 (DB query: list_tracks_paginated)
T2 (AppError nested) ─┤   │
                      │   ├── T5 (route: GET /api/tracks)
                      │   │   │
                      │   │   ├── T6 (integration test: round-trip)
                      │   │   │
                      │   │   └── T7 (frontend: track catalog screen)
                      │   │       │
T4 (Track model +     │   │       └── T8 (frontend: widget test)
    API types) ───────┘   │
                          │
                          │
  (T4 also feeds T7 via the API response shape)
```

**Critical path**: T1 → T3 → T5 → T6
**Parallel tracks**:
- Backend Teammate: T1 → T2 → T4 → T3 → T5 → T6
- Frontend Teammate: T7 → T8 (starts after T5 is merged; can stub API client earlier)

---

## Tasks

### T1: Database Migration 003 — DJ Metadata Columns

**Module**: `backend/migrations/003_dj_metadata.sql`
**Covers**: Precondition 4, API fields `bpm`, `key`, `source`, `energy`, `album_art_url`
**Size**: S | **Risk**: Low | **Agent**: Backend Teammate

**Description**:
Create migration 003 that adds DJ metadata columns to the `tracks` table. These columns will be nullable initially (populated later by Beatport import and essentia analysis).

SQL to add:
```sql
-- Migration 003: Add DJ metadata columns to tracks table
-- Supports ST-001 (track catalog API) and UC-013 (multi-source import)

ALTER TABLE tracks ADD COLUMN bpm REAL;
ALTER TABLE tracks ADD COLUMN camelot_key TEXT;
ALTER TABLE tracks ADD COLUMN energy REAL;
ALTER TABLE tracks ADD COLUMN source TEXT NOT NULL DEFAULT 'spotify';
ALTER TABLE tracks ADD COLUMN album_art_url TEXT;
```

Also update the following files to apply the new migration:

1. **`backend/src/main.rs`** — Add `include_str!("../migrations/003_dj_metadata.sql")` and execute it after migration 002 (around line 116).

2. **`backend/src/db/mod.rs`** — Update `create_test_pool()` to also apply migration 003 (add after the migration 002 block, around line 17).

**Acceptance**:
- [ ] Migration applies cleanly on fresh DB and on DB with existing tracks from UC-001
- [ ] Existing tracks retain all data (non-destructive ALTER)
- [ ] New columns default correctly: `bpm` NULL, `camelot_key` NULL, `energy` NULL, `source` = 'spotify', `album_art_url` NULL
- [ ] `cargo test` passes (test pool applies migration 003)
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: Nothing
**Blocks**: T3, T4

---

### T2: Update AppError to Nested ErrorResponse Format

**Module**: `backend/src/error.rs`
**Covers**: Postcondition 6, Integration Assertion 7, Extensions 3a, 4a, 5a
**Size**: S | **Risk**: Medium | **Agent**: Backend Teammate

**Description**:
The current `AppError::into_response` returns a flat `{"error": "message"}`. The OpenAPI contract requires a nested format: `{"error": {"code": "...", "message": "..."}}`.

Update `backend/src/error.rs` to change the `IntoResponse` implementation:

**Current** (line 35):
```rust
let body = serde_json::json!({ "error": message });
```

**New**:
```rust
let (status, code, message) = match &self {
    AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
    AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST", msg.clone()),
    AppError::Database(_) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "Database error".to_string(),
    ),
    AppError::Internal(_) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "Internal server error".to_string(),
    ),
};

let body = serde_json::json!({
    "error": {
        "code": code,
        "message": message,
    }
});
```

Error codes to use (matching OpenAPI `ErrorResponse.error.code`):
- `INVALID_REQUEST` — 400 Bad Request
- `NOT_FOUND` — 404 Not Found
- `INTERNAL_ERROR` — 500 Internal Server Error

**Important**: The import route (`backend/src/routes/import.rs`) has its own `ErrorBody` struct (line 48) with a flat `{ "error": "..." }` format. This must also be updated to match the nested format, or refactored to use `AppError`. For this steel thread, update the `ImportError::into_response` (line 57) to use the same nested shape:

```rust
let body = serde_json::json!({
    "error": {
        "code": code,
        "message": msg,
    }
});
```

Where `code` is mapped from each variant (e.g., `InvalidUrl` -> `"INVALID_REQUEST"`, `NotFound` -> `"NOT_FOUND"`, `AccessDenied` -> `"ACCESS_DENIED"`, `SpotifyError` -> `"UPSTREAM_ERROR"`, `Database` -> `"INTERNAL_ERROR"`).

**Acceptance**:
- [ ] `AppError::BadRequest` returns `{"error": {"code": "INVALID_REQUEST", "message": "..."}}`
- [ ] `AppError::NotFound` returns `{"error": {"code": "NOT_FOUND", "message": "..."}}`
- [ ] `AppError::Database` returns `{"error": {"code": "INTERNAL_ERROR", "message": "Database error"}}`
- [ ] `ImportError` variants return the same nested format
- [ ] Existing tests in `routes/import.rs` still pass (they only check status codes, not body shape, so they should)
- [ ] Add at least one unit test in `error.rs` that deserializes the response body and verifies the nested structure
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: Nothing
**Blocks**: T5

---

### T3: Database Query — Paginated Track Listing with Artist JOIN

**Module**: `backend/src/db/tracks.rs`
**Covers**: MSS steps 4-6, Extensions 4b, 5a, 6a, Invariant 3
**Size**: M | **Risk**: Medium | **Agent**: Backend Teammate

**Description**:
Add a new function to `backend/src/db/tracks.rs` that fetches a paginated list of tracks with joined artist names.

**New function signature**:
```rust
pub async fn list_tracks_paginated(
    pool: &SqlitePool,
    page: u32,
    per_page: u32,
    sort: &str,
    order: &str,
) -> Result<(Vec<TrackRow>, i64), sqlx::Error>
```

Where `TrackRow` is a new struct (add to `backend/src/db/models.rs`):
```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrackRow {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,       // Concatenated from JOIN, may be NULL if no artists linked
    pub album: Option<String>,
    pub duration_ms: Option<i64>,
    pub bpm: Option<f64>,
    pub camelot_key: Option<String>,
    pub energy: Option<f64>,
    pub source: String,
    pub spotify_uri: Option<String>,   // Used to derive source_id
    pub spotify_preview_url: Option<String>,
    pub album_art_url: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}
```

**SQL query structure** (two queries — count + data):

Count query:
```sql
SELECT COUNT(*) FROM tracks
```

Data query (conceptual — the sort/order must be built dynamically):
```sql
SELECT
    t.id,
    t.title,
    GROUP_CONCAT(a.name, ', ') AS artist,
    t.album,
    t.duration_ms,
    t.bpm,
    t.camelot_key,
    t.energy,
    t.source,
    t.spotify_uri,
    t.spotify_preview_url,
    t.album_art_url,
    t.created_at
FROM tracks t
LEFT JOIN track_artists ta ON t.id = ta.track_id
LEFT JOIN artists a ON ta.artist_id = a.id
GROUP BY t.id
ORDER BY {sort_column} {order} NULLS {LAST|FIRST}, t.id ASC
LIMIT ? OFFSET ?
```

**Sort field mapping** (sort parameter -> SQL column):
- `title` -> `t.title`
- `artist` -> `artist` (the GROUP_CONCAT alias — or use a subquery)
- `bpm` -> `t.bpm`
- `key` -> `t.camelot_key`
- `date_added` -> `t.created_at`

**NULL handling** (Extension 4b):
- `ASC` -> `NULLS LAST` (populated values first)
- `DESC` -> `NULLS FIRST` (populated values first — but this is DESC, so NULLS go to the end of the result set)

Wait — the steel thread says:
> "NULLS LAST for ascending sorts, NULLS FIRST for descending sorts"

This means: in both directions, tracks with NULL values for the sort column appear at the END of the result set. For SQLite this translates to:
- `ORDER BY col ASC NULLS LAST` — nulls at end
- `ORDER BY col DESC NULLS LAST` — actually, for DESC, putting nulls at end means NULLS LAST too

Re-reading the steel thread Extension 4b more carefully:
> "Server uses NULLS LAST for ascending sorts, NULLS FIRST for descending sorts"
> "Tracks with populated values always appear before tracks with NULL in the sorted direction"

For DESC order, the "sorted direction" is high-to-low. Populated values should come first (high values), then NULLs at the bottom. That is `NULLS LAST` in SQL.

For ASC order, populated values come first (low values), then NULLs at the bottom. That is also `NULLS LAST`.

SQLite does not natively support `NULLS LAST`/`NULLS FIRST` syntax (added in 3.30.0+, which should be available). If not available, use `ORDER BY (col IS NULL), col {ASC|DESC}` as a workaround.

**Tie-break** (Invariant 3): Always add `, t.id ASC` as secondary sort for deterministic ordering.

**Important implementation note**: Since the sort field and order are validated enum values from the route layer, this function should accept them as validated strings. Do NOT interpolate user input directly into SQL. Use a `match` statement to select the correct pre-built query string for each valid sort/order combination (6 combinations: 5 sort fields x is small enough to enumerate, but use `format!` with validated values from a match).

**Tests** (add to the `#[cfg(test)] mod tests` block in `backend/src/db/tracks.rs`):

1. `test_list_tracks_paginated_empty` — No tracks, returns `([], 0)`
2. `test_list_tracks_paginated_basic` — Insert 3 tracks with artists, verify all fields returned correctly
3. `test_list_tracks_paginated_pagination` — Insert 5 tracks, request page=1 per_page=2, verify 2 returned and total=5; request page=3, verify 1 returned
4. `test_list_tracks_paginated_sort_bpm_nulls` — Insert tracks with and without BPM, sort by bpm ASC, verify NULLs come last
5. `test_list_tracks_paginated_multi_artist` — Insert track with 2 artists, verify artist field is "Artist1, Artist2"

**Acceptance**:
- [ ] Returns correct pagination (offset = (page - 1) * per_page)
- [ ] Artist names joined with ", " via GROUP_CONCAT
- [ ] NULL fields (bpm, camelot_key, energy, album_art_url) handled correctly
- [ ] Sort by each of the 5 fields works correctly
- [ ] NULLS appear at end of results regardless of sort direction
- [ ] Deterministic tie-break on `t.id`
- [ ] All 5 tests pass
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: T1 (migration 003 must be applied for bpm/camelot_key/energy/source/album_art_url columns)
**Blocks**: T5

---

### T4: Backend API Response Types — Track and TrackListResponse

**Module**: `backend/src/routes/tracks.rs` (new file, top section)
**Covers**: Postconditions 1-3, the response shape from `openapi.yaml`
**Size**: S | **Risk**: Low | **Agent**: Backend Teammate

**Description**:
Define the Serde serializable types that form the JSON response for `GET /api/tracks`. These types must exactly match the `TrackListResponse` schema in `docs/api/openapi.yaml`.

**Types to define** (in the new `backend/src/routes/tracks.rs` file):

```rust
use serde::Serialize;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize)]
pub struct TrackResponse {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_ms: Option<i64>,
    pub bpm: Option<f64>,
    pub key: Option<String>,       // maps from camelot_key
    pub energy: Option<f64>,
    pub source: String,
    pub source_id: Option<String>,  // derived from spotify_uri (or future beatport_id)
    pub preview_url: Option<String>,
    pub album_art_url: Option<String>,
    pub date_added: Option<String>, // ISO 8601 formatted string from created_at
}

#[derive(Debug, Serialize)]
pub struct TrackListResponse {
    pub data: Vec<TrackResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: i64,
}
```

**Conversion function** — `TrackRow` (from T3) to `TrackResponse`:

```rust
impl From<TrackRow> for TrackResponse {
    fn from(row: TrackRow) -> Self {
        TrackResponse {
            id: row.id,
            title: row.title,
            artist: row.artist.unwrap_or_default(),  // empty string if no artists
            album: row.album,
            duration_ms: row.duration_ms,
            bpm: row.bpm,
            key: row.camelot_key,
            energy: row.energy,
            source: row.source,
            source_id: row.spotify_uri,  // For spotify tracks; will generalize later
            preview_url: row.spotify_preview_url,
            album_art_url: row.album_art_url,
            date_added: row.created_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        }
    }
}
```

**Acceptance**:
- [ ] `TrackResponse` serializes to JSON matching the `Track` schema in openapi.yaml
- [ ] `TrackListResponse` serializes to JSON matching the `TrackListResponse` schema
- [ ] Null fields serialize as `null` in JSON (not omitted)
- [ ] `date_added` is ISO 8601 format
- [ ] `artist` is a String (never null — empty string if no artists linked)
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: T1 (needs TrackRow which depends on migration 003 columns)
**Blocks**: T5

---

### T5: Route Handler — GET /api/tracks

**Module**: `backend/src/routes/tracks.rs` (handler section), `backend/src/routes/mod.rs`, `backend/src/main.rs`
**Covers**: MSS steps 2-7, Extensions 3a, 3b, 4a, 4b, 6a, Postconditions 1-6, Failure Postconditions 1-2
**Size**: M | **Risk**: Medium | **Agent**: Backend Teammate

**Description**:
Implement the Axum route handler for `GET /api/tracks` and wire it into the application router.

**1. Query parameter extraction** — Define a struct for the query params:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListTracksParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    #[serde(default = "default_sort")]
    pub sort: String,
    #[serde(default = "default_order")]
    pub order: String,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 25 }
fn default_sort() -> String { "date_added".to_string() }
fn default_order() -> String { "desc".to_string() }
```

**2. Validation** (MSS step 3, Extension 3a):
- `page` must be >= 1, otherwise return `AppError::BadRequest("Page must be a positive integer")`
- `per_page` must be 1-100, otherwise return `AppError::BadRequest("per_page must be between 1 and 100")`
- `sort` must be one of: `title`, `artist`, `bpm`, `key`, `date_added`, otherwise return `AppError::BadRequest("Invalid sort field. Must be one of: title, artist, bpm, key, date_added")`
- `order` must be one of: `asc`, `desc`, otherwise return `AppError::BadRequest("Invalid order. Must be one of: asc, desc")`

**3. Handler function**:

```rust
async fn list_tracks(
    State(pool): State<SqlitePool>,
    Query(params): Query<ListTracksParams>,
) -> Result<Json<TrackListResponse>, AppError> {
    // Validate params
    // Call db::tracks::list_tracks_paginated(...)
    // Map TrackRow -> TrackResponse
    // Calculate total_pages = ceil(total / per_page)
    // Return TrackListResponse
}
```

**4. Router wiring**:

In `backend/src/routes/mod.rs`, add:
```rust
pub mod tracks;
```

In `backend/src/routes/tracks.rs`, add a router function:
```rust
pub fn tracks_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/tracks", get(list_tracks))
        .with_state(pool)
}
```

In `backend/src/main.rs`, wire the new router (after the existing `.nest("/api", ...)` calls, around line 173):
```rust
.nest("/api", routes::tracks::tracks_router(pool.clone()))
```

**5. Edge cases**:
- Extension 3b: Page beyond total_pages → return 200 with empty `data` array, correct `total` and `total_pages`
- Extension 5a: No tracks → return 200 with empty `data`, `total=0`, `total_pages=0`
- Extension 4a: Database error → return 500 via `AppError::Database`
- Postcondition 3: `total_pages` = `(total + per_page - 1) / per_page` (ceiling division), or 0 if total is 0

**6. Unit tests** (add to `#[cfg(test)] mod tests` in `backend/src/routes/tracks.rs`):

1. `test_list_tracks_default_params` — GET /api/tracks with no params, verify 200 with default pagination
2. `test_list_tracks_invalid_page_zero` — GET /api/tracks?page=0, verify 400 with nested ErrorResponse
3. `test_list_tracks_invalid_per_page` — GET /api/tracks?per_page=200, verify 400
4. `test_list_tracks_invalid_sort` — GET /api/tracks?sort=invalid, verify 400
5. `test_list_tracks_empty_catalog` — No tracks in DB, verify 200 with `data: [], total: 0, total_pages: 0`
6. `test_list_tracks_page_beyond_total` — Request page=999, verify 200 with empty data but correct total

These tests should use `create_test_pool()` and build the router with `tracks_router(pool)` + `tower::ServiceExt::oneshot()` pattern (matching the existing test patterns in `routes/import.rs`).

**Acceptance**:
- [ ] GET /api/tracks returns 200 with correct `TrackListResponse` shape
- [ ] Default params: page=1, per_page=25, sort=date_added, order=desc
- [ ] Invalid page/per_page/sort/order returns 400 with nested ErrorResponse
- [ ] Empty catalog returns 200 with `data: [], total: 0`
- [ ] Page beyond total returns 200 with empty data
- [ ] All 6 tests pass
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: T2 (nested ErrorResponse), T3 (DB query), T4 (response types)
**Blocks**: T6, T7

---

### T6: Integration Test — Full Round-Trip (API -> DB -> Response)

**Module**: `backend/tests/tracks_api_test.rs` (new file — integration test outside src/)
**Covers**: Integration Assertions 1-7, all postconditions, all failure postconditions
**Size**: M | **Risk**: Low | **Agent**: Backend Teammate

**Description**:
Create an integration test file that proves the full round-trip: HTTP request -> route handler -> DB query -> JSON response. This file lives in `backend/tests/` (Cargo's integration test directory), not inside `src/`.

**Setup**: Since this is an integration test outside `src/`, it cannot use `crate::db::create_test_pool()` directly. Instead, set up the pool and migrations inline, or make a test helper publicly accessible. The simplest approach is to make the test module use the library's public interface.

Add `[[test]]` target or just put the file in `backend/tests/tracks_api_test.rs` — Cargo discovers it automatically.

The test file needs to:
1. Create an in-memory SQLite pool with all 3 migrations applied
2. Seed test data (tracks, artists, track_artists) using direct SQL inserts
3. Build the router (just `tracks_router(pool)`)
4. Send HTTP requests via `tower::ServiceExt::oneshot()`
5. Assert response status, body shape, and data correctness

**Test cases**:

1. **`test_round_trip_basic`** (Integration Assertion 1, 2):
   - Seed 3 tracks with artists (simulating UC-001 Spotify import data)
   - GET /api/tracks → verify 200, data has 3 items, all fields present
   - Verify nullable fields (`bpm`, `key`, `energy`, `album_art_url`) are `null` for Spotify-imported tracks

2. **`test_round_trip_pagination`** (Integration Assertion 4):
   - Seed 7 tracks
   - GET /api/tracks?page=1&per_page=3 → verify data has 3 items, total=7, total_pages=3
   - GET /api/tracks?page=2&per_page=3 → verify data has 3 different items
   - GET /api/tracks?page=3&per_page=3 → verify data has 1 item
   - Verify no overlap between pages

3. **`test_round_trip_sorting`** (Integration Assertion 5):
   - Seed 3 tracks with different BPMs (120.0, 128.0, NULL)
   - GET /api/tracks?sort=bpm&order=asc → verify order is 120.0, 128.0, null
   - GET /api/tracks?sort=bpm&order=desc → verify order is 128.0, 120.0, null

4. **`test_round_trip_error_format`** (Integration Assertion 7):
   - GET /api/tracks?page=0 → verify 400
   - Parse body, verify `error.code` == "INVALID_REQUEST" and `error.message` is present

5. **`test_round_trip_multi_artist`** (Integration Assertion 3):
   - Seed 1 track with 2 artists
   - GET /api/tracks → verify artist field contains both names separated by ", "

6. **`test_round_trip_empty_catalog`**:
   - No seeds
   - GET /api/tracks → verify 200, data=[], total=0, total_pages=0

**Acceptance**:
- [ ] All 6 integration tests pass
- [ ] Tests run in-memory (no file-system DB)
- [ ] Tests are deterministic (no timing dependencies)
- [ ] `cargo test --test tracks_api_test` passes
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: T5 (route handler must exist)
**Blocks**: Nothing (final backend verification)

---

### T7: Frontend — Track Catalog Screen with API Integration

**Module**: Multiple frontend files (see below)
**Covers**: MSS steps 1, 8-10, Extensions 2a, 5a, 6a, Integration Assertion 3
**Size**: L | **Risk**: Medium | **Agent**: Frontend Teammate

**Description**:
Build the Track Catalog screen that fetches and displays tracks from `GET /api/tracks`. This requires updating the Track model, API client, creating a Riverpod provider, building the screen, and wiring routing.

**File-by-file breakdown**:

**7a. Update Track model** — `frontend/lib/models/track.dart`

Replace the existing `Track` class with one matching the API response:

```dart
class Track {
  final String id;
  final String title;
  final String artist;
  final String? album;
  final int? durationMs;
  final double? bpm;
  final String? key;
  final double? energy;
  final String source;
  final String? sourceId;
  final String? previewUrl;
  final String? albumArtUrl;
  final DateTime dateAdded;

  const Track({
    required this.id,
    required this.title,
    required this.artist,
    this.album,
    this.durationMs,
    this.bpm,
    this.key,
    this.energy,
    required this.source,
    this.sourceId,
    this.previewUrl,
    this.albumArtUrl,
    required this.dateAdded,
  });

  factory Track.fromJson(Map<String, dynamic> json) {
    return Track(
      id: json['id'] as String,
      title: json['title'] as String,
      artist: json['artist'] as String,
      album: json['album'] as String?,
      durationMs: json['duration_ms'] as int?,
      bpm: (json['bpm'] as num?)?.toDouble(),
      key: json['key'] as String?,
      energy: (json['energy'] as num?)?.toDouble(),
      source: json['source'] as String? ?? 'spotify',
      sourceId: json['source_id'] as String?,
      previewUrl: json['preview_url'] as String?,
      albumArtUrl: json['album_art_url'] as String?,
      dateAdded: DateTime.parse(json['date_added'] as String),
    );
  }
}
```

**7b. Add paginated response model** — `frontend/lib/models/track_list_response.dart` (new file)

```dart
class TrackListResponse {
  final List<Track> data;
  final int page;
  final int perPage;
  final int total;
  final int totalPages;

  const TrackListResponse({
    required this.data,
    required this.page,
    required this.perPage,
    required this.total,
    required this.totalPages,
  });

  factory TrackListResponse.fromJson(Map<String, dynamic> json) { ... }
}
```

**7c. Add API method** — `frontend/lib/services/api_client.dart`

Add to `ApiClient`:
```dart
Future<TrackListResponse> listTracks({
  int page = 1,
  int perPage = 25,
  String sort = 'date_added',
  String order = 'desc',
}) async {
  final response = await _dio.get('/tracks', queryParameters: {
    'page': page,
    'per_page': perPage,
    'sort': sort,
    'order': order,
  });
  return TrackListResponse.fromJson(response.data as Map<String, dynamic>);
}
```

**7d. Create track catalog provider** — `frontend/lib/providers/track_catalog_provider.dart` (new file)

A Riverpod `StateNotifier` or `AsyncNotifier` that:
- Holds current page, sort, order, loading state, error state, and list of tracks
- Exposes `loadPage(int page)` — fetches from API, appends or replaces data
- Exposes `setSort(String sort, String order)` — resets to page 1 with new sort
- Handles loading, loaded, error states
- Supports "load more" (infinite scroll): appends to existing list when fetching next page

**7e. Build track catalog screen** — `frontend/lib/screens/track_catalog_screen.dart` (new file)

The screen should:
- Display tracks in a scrollable list using `TrackTile` widgets (update existing `widgets/track_tile.dart` to show BPM, key, and album art)
- Show loading indicator on initial fetch
- Show empty state: "No tracks yet -- import from Spotify to get started" with a button to navigate to `/import/spotify` (Extension 5a)
- Show error state with retry button (Extension 2a)
- Support infinite scroll: detect when user scrolls near bottom, fetch next page (MSS step 9)
- Handle null field display: show "--" for null BPM, key, energy (Extension 6a)

**7f. Update TrackTile widget** — `frontend/lib/widgets/track_tile.dart`

Update to display DJ-relevant fields:
- Album art thumbnail (left side, placeholder if `albumArtUrl` is null)
- Title + artist (primary content)
- BPM (show as "128" or "--"), key (show as "8A" or "--")
- Duration formatted as "3:45"

**7g. Wire routing** — `frontend/lib/config/routes.dart`

Add route for the track catalog:
```dart
GoRoute(
  path: '/tracks',
  builder: (context, state) => const TrackCatalogScreen(),
),
```

**7h. Update home screen** — `frontend/lib/screens/home_screen.dart`

Add a "Track Catalog" / "My Tracks" button that navigates to `/tracks`.

**Acceptance**:
- [ ] Track catalog screen renders track list from API
- [ ] Pagination works (scroll to load more, or explicit "Load More" button)
- [ ] Null fields display as "--" (not "null" or crash)
- [ ] Empty state renders with import CTA
- [ ] Error state renders with retry
- [ ] Sort selector works (at minimum date_added and bpm)
- [ ] `flutter analyze` passes
- [ ] `flutter test` passes

**Blocked by**: T5 (API endpoint must be available for integration; can be stubbed for initial development)
**Blocks**: T8

---

### T8: Frontend Widget Tests — Track Catalog

**Module**: `frontend/test/screens/track_catalog_test.dart` (new file)
**Covers**: Integration Assertion 3, Postcondition 5
**Size**: S | **Risk**: Low | **Agent**: Frontend Teammate

**Description**:
Widget tests for the track catalog screen using mock data. These do not hit the real API — they use a mock `ApiClient` injected via Riverpod overrides.

**Test cases**:

1. **`renders track list with all fields`**:
   - Provide mock response with 2 tracks (one with all fields, one with nulls)
   - Verify titles, artists rendered
   - Verify BPM shows "128" for populated track and "--" for null track
   - Verify key shows "8A" for populated track and "--" for null track

2. **`renders empty state when no tracks`**:
   - Provide mock response with empty data array
   - Verify empty state message appears
   - Verify import CTA button is present

3. **`renders error state on API failure`**:
   - Mock API to throw DioException
   - Verify error message appears
   - Verify retry button is present

4. **`handles pagination - loads more tracks`**:
   - Provide mock response with total=10, per_page=5 on first call
   - Verify initial 5 tracks rendered
   - Trigger load-more (scroll or button)
   - Provide second page response
   - Verify now 10 tracks rendered

5. **`renders multi-artist track correctly`**:
   - Provide mock track with artist "Hassan Hakmoun, Gnawa Diffusion"
   - Verify full artist string renders

**Acceptance**:
- [ ] All 5 widget tests pass
- [ ] Tests use Riverpod overrides for mocking (not real HTTP)
- [ ] `flutter test test/screens/track_catalog_test.dart` passes
- [ ] `flutter analyze` passes

**Blocked by**: T7 (screen must exist to test)
**Blocks**: Nothing (final frontend verification)

---

## Summary

| Task | Description | Size | Risk | Agent | Blocked By | Blocks |
|------|-------------|------|------|-------|------------|--------|
| **T1** | Migration 003: bpm, camelot_key, energy, source, album_art_url | S | Low | Backend | -- | T3, T4 |
| **T2** | AppError nested ErrorResponse format | S | Med | Backend | -- | T5 |
| **T3** | DB query: list_tracks_paginated with JOIN | M | Med | Backend | T1 | T5 |
| **T4** | API response types: TrackResponse, TrackListResponse | S | Low | Backend | T1 | T5 |
| **T5** | Route handler: GET /api/tracks + router wiring | M | Med | Backend | T2, T3, T4 | T6, T7 |
| **T6** | Integration test: full round-trip | M | Low | Backend | T5 | -- |
| **T7** | Frontend: track catalog screen + provider + routing | L | Med | Frontend | T5 | T8 |
| **T8** | Frontend: widget tests for track catalog | S | Low | Frontend | T7 | -- |

## Agent Execution Plan

### Backend Teammate

Executes T1 through T6 sequentially:

```
T1 (migration 003)
 ↓
T2 (AppError update) — can run in parallel with T1 but logically sequence for single agent
 ↓
T4 (response types) — depends on T1 for TrackRow
 ↓
T3 (DB query) — depends on T1 for new columns
 ↓
T5 (route handler) — depends on T2 + T3 + T4
 ↓
T6 (integration test) — depends on T5
```

**Estimated time**: T1(5m) + T2(15m) + T4(10m) + T3(25m) + T5(20m) + T6(20m) = ~95 minutes

### Frontend Teammate

Executes T7 and T8 sequentially. Can start T7 before T5 is merged by stubbing the API response locally, but should wire to the real API before marking complete.

```
T7 (track catalog screen + model + provider + routing)
 ↓
T8 (widget tests)
```

**Estimated time**: T7(40m) + T8(25m) = ~65 minutes

### Lead (Coordinator)

1. Create feature branch: `git checkout -b feature/st-001-track-catalog`
2. Warm dependency cache: `cd backend && cargo check` (saves agent build time)
3. Spawn Backend Teammate with T1-T6 assignment
4. Spawn Frontend Teammate with T7-T8 assignment (tell them to start with model/provider stubs while waiting for backend)
5. Monitor progress, review code, resolve merge conflicts if any
6. Run quality gates after all tasks complete:
   ```bash
   cd backend && cargo fmt --check && cargo clippy -- -D warnings && cargo test
   cd frontend && flutter analyze && flutter test
   ```
7. Run `/verify-uc ST-001` against the steel thread postconditions

---

## Pre-Implementation Checklist

- [ ] Feature branch created: `feature/st-001-track-catalog`
- [ ] `cargo check` run to warm dependency cache
- [ ] OpenAPI contract reviewed and locked (`docs/api/openapi.yaml`)
- [ ] Steel thread reviewed (`docs/steel-threads/st-001-serve-paginated-track-catalog.md`)
- [ ] No conflicting changes on `main` since steel thread was written

## Files Created / Modified

**New files**:
- `backend/migrations/003_dj_metadata.sql`
- `backend/src/routes/tracks.rs`
- `backend/tests/tracks_api_test.rs`
- `frontend/lib/models/track_list_response.dart`
- `frontend/lib/providers/track_catalog_provider.dart`
- `frontend/lib/screens/track_catalog_screen.dart`

**Modified files**:
- `backend/src/error.rs` (nested ErrorResponse)
- `backend/src/routes/mod.rs` (add `pub mod tracks;`)
- `backend/src/routes/import.rs` (update ImportError response format)
- `backend/src/main.rs` (apply migration 003, wire tracks router)
- `backend/src/db/mod.rs` (apply migration 003 in test pool)
- `backend/src/db/models.rs` (add TrackRow)
- `backend/src/db/tracks.rs` (add list_tracks_paginated)
- `frontend/lib/models/track.dart` (update Track model for API shape)
- `frontend/lib/services/api_client.dart` (add listTracks method)
- `frontend/lib/widgets/track_tile.dart` (update for DJ fields)
- `frontend/lib/config/routes.dart` (add /tracks route)
- `frontend/lib/screens/home_screen.dart` (add Track Catalog nav button)
