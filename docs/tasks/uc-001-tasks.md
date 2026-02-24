# Task Decomposition: UC-001 Import Seed Catalog from Spotify Playlist

**Source**: `docs/use-cases/uc-001-import-seed-catalog-from-spotify.md`
**Review Status**: Reviewed and fixed (12 fixes applied, all CRITICAL/WARNING/SUGGESTION resolved)
**Total Tasks**: 10 implementation tasks + 1 integration test task

---

## Dependency Graph

```
T1 (DB migration)
├── T2 (Spotify API client) ──┐
│   └── T4 (retry/resilience) │
├── T3 (OAuth routes) ────────┤
│                              ├── T6 (import service)
├── T5 (DB queries) ──────────┤   ├── T7 (import route)
│                              │   │   └── T11 (integration tests)
│                              │   └── T8 (frontend: import screen)
│                              │       ├── T9 (frontend: progress/summary)
│                              │       └── T10 (frontend: provider)
└──────────────────────────────┘
```

---

## Tasks

### T1: Database Migration — Add Spotify Import Tables

**Module**: `backend/migrations/002_spotify_imports.sql`
**Covers**: Precondition 3, Postcondition 6 (encrypted tokens), Postcondition 7 (import provenance)
**Size**: S | **Risk**: Low | **Agent**: Teammate:Builder (backend)

**Description**:
Create migration 002 adding two tables missing from the current schema:

1. `user_spotify_tokens` — stores encrypted OAuth tokens per user
   - `user_id TEXT PRIMARY KEY REFERENCES users(id)`
   - `access_token_encrypted BLOB NOT NULL` — encrypted access token (never plaintext)
   - `refresh_token_encrypted BLOB NOT NULL` — encrypted refresh token (never plaintext)
   - `expires_at TIMESTAMP NOT NULL`
   - `scopes TEXT NOT NULL`
   - `created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP`
   - `updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP`

2. `spotify_imports` — import provenance tracking
   - `id TEXT PRIMARY KEY`
   - `user_id TEXT NOT NULL REFERENCES users(id)`
   - `spotify_playlist_id TEXT NOT NULL`
   - `spotify_playlist_name TEXT`
   - `tracks_found INTEGER NOT NULL DEFAULT 0`
   - `tracks_inserted INTEGER NOT NULL DEFAULT 0`
   - `tracks_updated INTEGER NOT NULL DEFAULT 0`
   - `tracks_failed INTEGER NOT NULL DEFAULT 0`
   - `status TEXT NOT NULL DEFAULT 'in_progress'` — 'in_progress', 'completed', 'failed'
   - `error_message TEXT`
   - `started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP`
   - `completed_at TIMESTAMP`

Also add UNIQUE index on `tracks.spotify_uri` and `artists.spotify_uri` to support upsert.

**Acceptance**:
- [ ] Migration applies cleanly: `sqlx migrate run`
- [ ] Tables exist with correct schema
- [ ] UNIQUE constraints on spotify_uri columns

**Blocked by**: Nothing
**Blocks**: T3, T5, T6

---

### T2: Spotify API Client — Core HTTP Wrapper

**Module**: `backend/src/api/spotify.rs`
**Covers**: MSS steps 10-11, Extensions 10a-10d
**Size**: M | **Risk**: Medium | **Agent**: Teammate:Builder (backend)

**Description**:
Create a Spotify API client struct wrapping `reqwest::Client` (not `rspotify` — we want full control over error handling and retry logic). Implement:

1. `SpotifyClient::new(client_id, client_secret)` — from env config
2. `exchange_code(code: &str, redirect_uri: &str) -> Result<TokenResponse>` — POST to `/api/token` (MSS step 6)
3. `refresh_token(refresh_token: &str) -> Result<TokenResponse>` — POST to `/api/token` with `grant_type=refresh_token` (Extension 10b)
4. `get_playlist_tracks(access_token: &str, playlist_id: &str, offset: u32, limit: u32) -> Result<PlaylistTracksResponse>` — GET `/v1/playlists/{id}/tracks` (MSS step 10)
5. Parse response into internal types: `SpotifyTrack`, `SpotifyArtist` (MSS step 11)
6. Handle rate limiting: read `Retry-After` header, respect it (Extension 10a)

**Types to define** (in `backend/src/api/spotify.rs` or separate `types.rs`):
```rust
struct TokenResponse { access_token, refresh_token, expires_in, scope }
struct SpotifyTrack { name, uri, album_name, duration_ms, preview_url, artists: Vec<SpotifyArtist> }
struct SpotifyArtist { name, uri }
struct PlaylistTracksResponse { items: Vec<PlaylistItem>, total: u32, next: Option<String> }
```

**Acceptance**:
- [ ] Unit tests for response parsing (happy path + missing fields)
- [ ] Handles null preview_url (Extension 11a)
- [ ] Handles missing artist info (Extension 11b)
- [ ] Compiles with `cargo clippy -- -D warnings`

**Blocked by**: Nothing (no DB dependency for the client itself)
**Blocks**: T4, T6

---

### T3: OAuth Routes — Spotify Authorization Code Flow

**Module**: `backend/src/routes/auth.rs`
**Covers**: MSS steps 2-7, Extensions 1a, 2a, 3a, 4a, 5a, 6a, 6b, 7a
**Size**: L | **Risk**: High | **Agent**: Teammate:Builder (backend)

**Description**:
Implement three Axum route handlers:

1. `GET /api/auth/spotify` — Initiate OAuth
   - Generate cryptographically random `state` (32 bytes, base64url)
   - Store `state` in session/temp store (keyed by user ID, TTL 5 minutes)
   - Return redirect URL to Spotify `/authorize` with:
     - `response_type=code`
     - `client_id`
     - `redirect_uri` (configured via env)
     - `scope=playlist-read-private playlist-read-collaborative`
     - `state`
   - Extension 2a: If user already has valid tokens, return early with status 200 + `{ already_connected: true }`

2. `GET /api/auth/spotify/callback` — Handle OAuth callback
   - Validate `state` matches stored value (Extension 5a: reject if mismatch)
   - Exchange `code` for tokens via `SpotifyClient::exchange_code` (T2)
   - Extension 6a: Handle 400/401 from Spotify token endpoint
   - Extension 6b: Detect `invalid_client` → log critical, return 503
   - Extension 4a: If `state` has expired (>5 min), return error

3. Token storage (via T5 DB layer):
   - Encrypt tokens before storage (use `aes-gcm` or `ring` — add to Cargo.toml)
   - Extension 7a: If storage fails, discard tokens, return error
   - Postcondition 6: Tokens NEVER stored as plaintext

**New dependency**: Add `aes-gcm = "0.10"` or `ring = "0.17"` to Cargo.toml for token encryption.

**Acceptance**:
- [ ] State parameter generated and validated (CSRF protection)
- [ ] Token exchange works with mocked Spotify (wiremock)
- [ ] Invalid state returns 400
- [ ] Expired state returns 400
- [ ] Token storage failure doesn't leave plaintext tokens
- [ ] Already-connected user skips OAuth flow

**Blocked by**: T1 (needs token table), T2 (needs SpotifyClient)
**Blocks**: T6, T8

---

### T4: Retry and Resilience Middleware

**Module**: `backend/src/api/retry.rs` (or integrated into `spotify.rs`)
**Covers**: Extensions 10a (rate limit), 10b (token refresh), 10c (server errors), 10d (network timeout)
**Size**: M | **Risk**: Medium | **Agent**: Teammate:Builder (backend)

**Description**:
Implement retry logic for Spotify API calls:

1. **Rate limiting (429)**: Read `Retry-After` header, sleep, retry. Max 3 consecutive 429s before giving up. (Extension 10a)
2. **Token expiry (401)**: Trigger token refresh via `SpotifyClient::refresh_token`, retry with new token. If refresh fails, return auth error. (Extension 10b)
3. **Server errors (500/502/503)**: Exponential backoff (1s, 2s, 4s), max 3 retries. (Extension 10c)
4. **Network timeout**: Same exponential backoff as server errors. Configure `reqwest` timeout to 10s. (Extension 10d)

Can be implemented as:
- A wrapper function `retry_spotify_request<F, T>(f: F, token_refresher: &dyn TokenRefresher) -> Result<T>`
- Or a Tower middleware layer

**Acceptance**:
- [ ] Unit test: 429 with Retry-After → waits and retries
- [ ] Unit test: 3x consecutive 429 → gives up with descriptive error
- [ ] Unit test: 401 → refreshes token → retries
- [ ] Unit test: 500 → exponential backoff → retries 3x
- [ ] Unit test: Network timeout → retries 3x

**Blocked by**: T2 (needs SpotifyClient)
**Blocks**: T6

---

### T5: Database Query Layer — Tracks, Artists, Imports

**Module**: `backend/src/db/tracks.rs`, `backend/src/db/artists.rs`, `backend/src/db/imports.rs`
**Covers**: MSS step 12, Postconditions 1-4, 7, Extension 12a
**Size**: M | **Risk**: Low | **Agent**: Teammate:Builder (backend)

**Description**:
Implement SQLx query functions:

1. **`db/tracks.rs`**:
   - `upsert_track(pool, track) -> Result<UpsertResult>` — INSERT ON CONFLICT(spotify_uri) UPDATE (Postcondition 4)
   - `get_tracks_by_import(pool, import_id) -> Result<Vec<Track>>`
   - Return whether row was inserted or updated (for summary counts)

2. **`db/artists.rs`**:
   - `upsert_artist(pool, artist) -> Result<UpsertResult>`
   - `upsert_track_artist(pool, track_id, artist_id, role) -> Result<()>`

3. **`db/imports.rs`**:
   - `create_import(pool, user_id, playlist_id, playlist_name) -> Result<Import>`
   - `update_import_progress(pool, import_id, counts) -> Result<()>`
   - `complete_import(pool, import_id, status, error_msg) -> Result<()>`

4. **`db/tokens.rs`**:
   - `store_encrypted_tokens(pool, user_id, encrypted_access, encrypted_refresh, expires_at, scopes) -> Result<()>`
   - `get_encrypted_tokens(pool, user_id) -> Result<Option<EncryptedTokens>>`
   - `delete_tokens(pool, user_id) -> Result<()>`

5. **Per-track transaction** (MSS step 12): Each track upsert (track + artists + track_artists) wrapped in a single transaction. If one part fails, only that track rolls back.

**Internal model types** (in `backend/src/db/mod.rs` or `models.rs`):
```rust
struct Track { id, title, album, duration_ms, spotify_uri, spotify_preview_url, ... }
struct Artist { id, name, spotify_uri, ... }
struct Import { id, user_id, spotify_playlist_id, ... counts ... status }
enum UpsertResult { Inserted, Updated }
```

**Acceptance**:
- [ ] `#[sqlx::test]` for each query function
- [ ] Upsert on duplicate spotify_uri updates metadata, doesn't create duplicate
- [ ] Transaction rollback on partial failure leaves DB consistent
- [ ] Token encryption/decryption round-trips correctly

**Blocked by**: T1 (needs migration applied)
**Blocks**: T6

---

### T6: Import Service — Orchestrator

**Module**: `backend/src/services/import.rs`
**Covers**: MSS steps 8-12, Postcondition 5 (summary), Postcondition 7 (provenance), Extensions 13a, 12a
**Size**: L | **Risk**: High | **Agent**: Teammate:Builder (backend)

**Description**:
The core business logic that orchestrates the import:

1. `validate_playlist_url(input: &str) -> Result<String>` — Extract playlist ID from URL or URI format (MSS step 9, Variation 8a)
   - Accept: `https://open.spotify.com/playlist/{id}`, `https://open.spotify.com/playlist/{id}?si=...`, `spotify:playlist:{id}`
   - Reject everything else with descriptive error (Extension 8a)

2. `import_playlist(pool, spotify_client, user_id, playlist_id) -> Result<ImportSummary>` — Main orchestrator:
   - Create import provenance record (Postcondition 7)
   - Paginate through all tracks: offset=0, limit=100, loop until all retrieved (MSS step 10)
   - Use retry middleware (T4) for each API call
   - For each track: extract fields (step 11), run per-track transaction (step 12)
   - Count inserted/updated/failed (Postcondition 5)
   - Handle partial failure: if API fails mid-way, commit what we have (Failure Postcondition 1)
   - Update import record with final counts and status
   - Return `ImportSummary { total, inserted, updated, failed }`

3. Extension 13a (empty playlist): Return early with appropriate error if total=0

**Acceptance**:
- [ ] Integration test: mock 54-track playlist → 54 rows in DB
- [ ] Integration test: re-import same playlist → 0 inserted, 54 updated
- [ ] Integration test: partial failure → committed tracks persisted
- [ ] Unit test: URL validation accepts URL and URI formats
- [ ] Unit test: URL validation rejects invalid inputs
- [ ] Import provenance record created with correct counts

**Blocked by**: T2, T4, T5
**Blocks**: T7

---

### T7: Import Route — HTTP Endpoint

**Module**: `backend/src/routes/import.rs`
**Covers**: MSS steps 8-13, Extensions 8a, 9a, 9b
**Size**: S | **Risk**: Low | **Agent**: Teammate:Builder (backend)

**Description**:
Axum route handler that exposes the import service:

1. `POST /api/import/spotify` — Accept JSON body: `{ "playlist_url": "..." }`
   - Validate auth (user must be authenticated) — Extension 1a
   - Validate playlist URL format (delegate to T6 validator) — Extension 8a
   - Call import service (T6)
   - Map Spotify 404 → "Playlist not found" (Extension 9a)
   - Map Spotify 403 → "No access" (Extension 9b)
   - Return JSON: `{ "total": 54, "inserted": 42, "updated": 12, "failed": 0, "import_id": "..." }`

2. `GET /api/import/:id` — Get import status/summary (for progress polling)
   - Returns import record with current counts

3. Wire routes into main router in `main.rs`

**Acceptance**:
- [ ] `axum-test` handler tests for happy path
- [ ] 400 for invalid URL
- [ ] 401 for unauthenticated user
- [ ] Response shape matches frontend expectations

**Blocked by**: T6
**Blocks**: T8, T11

---

### T8: Frontend — Import from Spotify Screen

**Module**: `frontend/lib/screens/spotify_import_screen.dart`
**Covers**: MSS steps 1-2, 8-9, Extension 8a
**Size**: M | **Risk**: Low | **Agent**: Teammate:Builder (frontend)

**Description**:
Create the "Import from Spotify" screen with:

1. **Connect to Spotify button** (MSS step 2)
   - If not connected: Opens browser/webview to backend OAuth URL (`/api/auth/spotify`)
   - If already connected: Show green "Connected" badge, skip to import form
   - Handle OAuth callback deep link

2. **Playlist URL input** (MSS step 8)
   - Text field with hint: "Paste Spotify playlist URL or URI"
   - Client-side format validation before sending to backend (Extension 8a)
   - Accept both URL and URI formats (Variation 8a)

3. **Import button** → calls `POST /api/import/spotify`

4. **Error display** — Show user-friendly errors from backend in a SnackBar or inline

5. Add route to GoRouter in `config/routes.dart`

**Acceptance**:
- [ ] Widget test: renders Connect button when not connected
- [ ] Widget test: renders import form when connected
- [ ] Widget test: validates URL format client-side
- [ ] Widget test: displays error messages from backend

**Blocked by**: T3 (needs OAuth endpoint), T7 (needs import endpoint)
**Blocks**: T9, T10

---

### T9: Frontend — Import Progress and Summary Display

**Module**: `frontend/lib/widgets/import_progress.dart`, `frontend/lib/widgets/import_summary.dart`
**Covers**: MSS steps 13-14, Variation 10a (progress indicator), Extension 14a
**Size**: S | **Risk**: Low | **Agent**: Teammate:Builder (frontend)

**Description**:

1. **Progress indicator** (Variation 10a):
   - Poll `GET /api/import/:id` every 2 seconds during import
   - Show "Importing track X of Y..." with LinearProgressIndicator
   - Show spinner while waiting

2. **Import summary card** (MSS step 13):
   - "Imported 54 tracks (42 new, 12 updated, 0 failed)"
   - Color-coded: green for new, blue for updated, red for failed
   - "View Catalog" button → navigate to catalog screen

3. **Error fallback** (Extension 14a):
   - If catalog load fails after import, show refresh prompt

**Acceptance**:
- [ ] Widget test: progress indicator shows correct counts
- [ ] Widget test: summary card renders all states
- [ ] Widget test: error state shows refresh option

**Blocked by**: T8
**Blocks**: Nothing

---

### T10: Frontend — Spotify Import Provider (State Management)

**Module**: `frontend/lib/providers/spotify_import_provider.dart`
**Covers**: State management for the entire import flow
**Size**: M | **Risk**: Low | **Agent**: Teammate:Builder (frontend)

**Description**:
Riverpod providers for import state:

1. `spotifyConnectionProvider` — AsyncNotifier tracking OAuth status
   - States: disconnected, connecting, connected, error
   - Calls `GET /api/auth/spotify` to check connection status

2. `spotifyImportProvider` — AsyncNotifier tracking import progress
   - States: idle, validating, importing(progress), completed(summary), error(message)
   - Triggers `POST /api/import/spotify`
   - Polls `GET /api/import/:id` during import

3. Update `ApiClient` with methods:
   - `checkSpotifyConnection() -> bool`
   - `importSpotifyPlaylist(url) -> ImportResult`
   - `getImportStatus(importId) -> ImportStatus`

**Acceptance**:
- [ ] Provider unit test: state transitions for OAuth flow
- [ ] Provider unit test: state transitions for import flow
- [ ] Provider unit test: error states handled correctly

**Blocked by**: T8
**Blocks**: Nothing

---

### T11: Integration Tests — Full Import Flow with Wiremock

**Module**: `backend/tests/spotify_import.rs`
**Covers**: All MSS steps, all extensions, all postconditions
**Size**: L | **Risk**: Medium | **Agent**: Teammate:Builder (backend test)

**Description**:
Integration tests using `wiremock` to mock Spotify API and `#[sqlx::test]` for database:

1. **Happy path**: Mock 54-track playlist → import → verify 54 rows in DB, correct artists, correct track_artists, import summary counts
2. **Re-import (upsert)**: Import same playlist twice → verify no duplicates, updated counts
3. **Rate limiting**: Mock 429 + Retry-After → verify retry behavior, max 3 retries
4. **Token refresh**: Mock 401 on first call, 200 on retry with new token → verify seamless recovery
5. **Partial failure**: Mock 200 for first page, 500 for second → verify first page tracks persisted
6. **Empty playlist**: Mock playlist with 0 tracks → verify appropriate error
7. **Invalid playlist URL**: Various invalid inputs → verify validation errors
8. **404 playlist**: Mock 404 → verify error message
9. **403 playlist**: Mock 403 → verify error message
10. **Non-Latin artist names**: Include Arabic/Tifinagh names → verify UTF-8 storage

**New dev-dependencies** in Cargo.toml:
```toml
[dev-dependencies]
wiremock = "0.6"
axum-test = "16"
tokio-test = "0.4"
```

**Acceptance**:
- [ ] All 10 test scenarios pass
- [ ] Tests run in <30 seconds
- [ ] No flaky tests (all deterministic with mocks)
- [ ] `cargo nextest run --test spotify_import` passes

**Blocked by**: T6, T7
**Blocks**: Nothing (final verification)

---

## Summary

| Task | Description | Size | Risk | Blocked By | Blocks |
|------|-------------|------|------|------------|--------|
| **T1** | DB migration: spotify_imports + tokens tables | S | Low | — | T3, T5, T6 |
| **T2** | Spotify API client: HTTP wrapper + types | M | Med | — | T4, T6 |
| **T3** | OAuth routes: /auth/spotify + callback | L | High | T1, T2 | T6, T8 |
| **T4** | Retry/resilience: rate limit, backoff, refresh | M | Med | T2 | T6 |
| **T5** | DB queries: upsert tracks/artists/imports | M | Low | T1 | T6 |
| **T6** | Import service: orchestrator logic | L | High | T2, T4, T5 | T7 |
| **T7** | Import route: HTTP endpoint | S | Low | T6 | T8, T11 |
| **T8** | Frontend: Import screen | M | Low | T3, T7 | T9, T10 |
| **T9** | Frontend: Progress + summary widgets | S | Low | T8 | — |
| **T10** | Frontend: Riverpod providers | M | Low | T8 | — |
| **T11** | Integration tests: wiremock full flow | L | Med | T6, T7 | — |

**Critical path**: T1 → T5 → T6 → T7 → T11
**Parallelizable tracks**:
- Track A (can start immediately): T1, T2 (in parallel)
- Track B (after T1+T2): T3, T4, T5 (in parallel)
- Track C (after Track B): T6 → T7
- Track D (after T7): T8 → T9 + T10 (in parallel)
- Track E (after T7): T11

**Recommended agent team**:
- **Backend Builder 1**: T1 → T2 → T4 → T6 → T7
- **Backend Builder 2**: T5 → T3 → T11
- **Frontend Builder**: T8 → T9 + T10

---

## Next Steps

1. `git checkout -b feature/uc-001-spotify-import`
2. Run pre-implementation checklist
3. `/agent-team-plan 001` to assign agents
4. Implement in dependency order
5. `cargo fmt --check && cargo clippy -- -D warnings && cargo test` after each task
6. `/verify-uc 001` when all tasks complete
