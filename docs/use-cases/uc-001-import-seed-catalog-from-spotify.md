# Use Case: UC-001 Import Seed Catalog from Spotify Playlist

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P0 Critical
- **Complexity**: 🟡 Medium

## Actors
- **Primary Actor**: App User (authenticated)
- **Supporting Actors**:
  - Spotify API (OAuth 2.0 Authorization Code flow, Web API v1)
  - Database (SQLite/PostgreSQL via SQLx)
- **Stakeholders & Interests**:
  - Curator: Wants the "Salamic Vibes" 54-track seed catalog imported accurately with all available metadata
  - App User: Wants to import their own Spotify playlists to build occasion-specific playlists
  - Developer: Wants a clean, source-agnostic internal track representation that doesn't couple to Spotify's schema

## Conditions
- **Preconditions** (must be true before starting):
  1. User is authenticated in the app
  2. Backend has valid Spotify app credentials (client ID + secret) configured via environment
  3. Database is initialized with the tracks/artists/track_artists schema (migration 001 applied)

- **Success Postconditions** (true when done right):
  1. All tracks from the Spotify playlist exist in the `tracks` table with: title, album, duration_ms, spotify_uri, spotify_preview_url
  2. All artists from the imported tracks exist in the `artists` table with: name, spotify_uri
  3. All track-artist relationships exist in the `track_artists` junction table
  4. Duplicate tracks (same spotify_uri) are upserted — metadata updated, no duplicate rows created
  5. The import operation returns a summary: total tracks found, new tracks inserted, existing tracks updated, failures (if any)
  6. User's Spotify OAuth tokens (access + refresh) are encrypted at rest in the database (not stored as plaintext)
  7. An import record is stored linking the Spotify playlist ID to the user and import timestamp, enabling re-import detection and provenance tracking

- **Failure Postconditions** (true when it fails gracefully):
  1. If the import fails mid-way, all tracks successfully fetched up to the failure point are persisted. No individual track is partially written (per-track transactions ensure atomicity at the track level).
  2. User receives a clear error message describing what went wrong
  3. If Spotify OAuth fails, user is redirected back to the app with an error state (not a blank page)
  4. Rate limit errors are communicated with a retry-after suggestion

- **Invariants** (must remain true throughout):
  1. Spotify API credentials (client secret, user tokens) are never exposed to the frontend
  2. All Spotify API calls happen on the backend — frontend never calls Spotify directly
  3. Database remains in a consistent state throughout (transactions used for multi-row operations)

## Main Success Scenario
1. User navigates to the "Import from Spotify" screen in the Flutter app
2. User clicks "Connect to Spotify" button
3. System generates a cryptographically random `state` parameter, stores it in the user's session, and redirects User to Spotify's OAuth authorization page (Authorization Code flow, scopes: `playlist-read-private playlist-read-collaborative`, including the `state` parameter)
4. User logs into Spotify and grants permission
5. Spotify redirects back to the app's callback URL with an authorization code and the `state` parameter
6. Backend validates the `state` parameter matches the user's session, then exchanges the authorization code for access and refresh tokens via Spotify's token endpoint
7. Backend stores the encrypted tokens associated with the user's account
8. User pastes or selects a Spotify playlist URL/URI in the import form
9. System validates the playlist URL format and extracts the playlist ID
10. Backend calls Spotify API `GET /v1/playlists/{id}/tracks` with the user's access token, paginating with limit=100 and offset until all tracks are retrieved
11. For each track in the response, Backend extracts: track name, artist(s), album name, duration_ms, Spotify URI, preview_url
12. For each track, Backend performs a single transaction: upserts the track into `tracks` (keyed on spotify_uri), upserts each artist into `artists` (keyed on spotify_uri), and upserts track-artist relationships into `track_artists`. If any part fails, the transaction rolls back for that track only.
13. System displays an import summary to the User: "Imported 54 tracks (42 new, 12 updated, 0 failed)"
14. User sees the imported tracks in their catalog

## Extensions (What Can Go Wrong)

- **1a. User is not authenticated**:
  1. System redirects user to login/registration screen
  2. After successful authentication, returns to step 1

- **2a. User has already connected Spotify (tokens exist and are valid)**:
  1. System skips steps 2-7 and proceeds directly to step 8
  2. Returns to step 8

- **3a. User declines Spotify authorization**:
  1. Spotify redirects back with `error=access_denied`
  2. System displays "Spotify access was denied. You need to grant access to import playlists."
  3. Use case fails

- **4a. User abandons Spotify login (closes window or navigates away)**:
  1. Backend times out waiting for callback after 5 minutes
  2. System displays "Spotify connection timed out. Please try again."
  3. Returns to step 2

- **5a. Spotify callback returns an error (invalid state, expired code)**:
  1. System logs the error details
  2. System displays "Something went wrong connecting to Spotify. Please try again."
  3. Returns to step 2

- **6a. Token exchange fails (Spotify returns 400/401)**:
  1. System logs the error response from Spotify
  2. System displays "Failed to connect to Spotify. Please try again."
  3. Returns to step 2

- **6b. Backend's Spotify client credentials are invalid or expired**:
  1. Token exchange returns 401 with `invalid_client`
  2. System logs a critical error (admin notification)
  3. System displays "Spotify integration is temporarily unavailable. Please try later."
  4. Use case fails

- **7a. Token storage fails (encryption error, database error)**:
  1. System logs the error with full context
  2. System displays "Failed to save Spotify connection. Please try again."
  3. Tokens are discarded (not persisted in an unencrypted state)
  4. Returns to step 2

- **8a. User enters an invalid URL/URI (not a Spotify playlist)**:
  1. System displays "That doesn't look like a Spotify playlist URL. Expected format: https://open.spotify.com/playlist/..."
  2. Returns to step 8

- **9a. Playlist ID is valid format but playlist doesn't exist (404)**:
  1. Spotify API returns 404
  2. System displays "Playlist not found. It may have been deleted or the URL may be incorrect."
  3. Returns to step 8

- **9b. Playlist exists but user doesn't have access (403)**:
  1. Spotify API returns 403
  2. System displays "You don't have access to this playlist. It may be private. Ask the owner to make it public or collaborative."
  3. Returns to step 8

- **10a. Spotify API returns 429 (rate limited)**:
  1. System reads the `Retry-After` header
  2. System waits the specified duration, then retries the request
  3. If rate-limited more than 3 times consecutively, display "Spotify is temporarily limiting requests. Please try again in a few minutes."
  4. Returns to step 10 (retry) or use case fails (exceeded retries)

- **10b. Spotify API returns 401 (token expired mid-import)**:
  1. Backend uses the refresh token to obtain a new access token
  2. Backend retries the failed request with the new token
  3. If refresh also fails, returns to step 2 (re-authorize)

- **10c. Spotify API returns 500/502/503 (server error)**:
  1. System retries up to 3 times with exponential backoff (1s, 2s, 4s)
  2. If all retries fail, system displays "Spotify is experiencing issues. Imported X of Y tracks so far. You can retry to continue."
  3. Partial import is committed (tracks fetched so far are saved)
  4. Returns to step 8 (user can retry for remaining tracks)

- **10d. Network timeout or connection failure**:
  1. System retries up to 3 times with exponential backoff
  2. System displays "Network error while contacting Spotify. Check your connection and try again."
  3. Partial import is committed
  4. Returns to step 8

- **11a. Track has no preview_url (Spotify removed previews for some tracks)**:
  1. System stores `null` for preview_url
  2. Track is still imported — preview fallback to YouTube will be handled by UC-06
  3. Continues to next track (step 11)

- **11b. Track has no artist information**:
  1. System creates track with artist set to "Unknown Artist"
  2. Continues to next track (step 11)

- **11c. Artist name contains non-Latin characters (Arabic, Tifinagh, etc.)**:
  1. System stores the name as-is in UTF-8 — no transliteration or normalization
  2. Frontend renders using appropriate font (Noto Sans Arabic, etc.)
  3. Continues to next track (step 11)

- **12a. Database transaction fails for a track (disk full, connection lost)**:
  1. Transaction rolls back for that track only
  2. Track is counted as "failed" in the import summary
  3. System continues with next track
  4. If all tracks fail, system displays "Failed to save imported tracks. Database error." and logs with full context

- **13a. Playlist is empty (0 tracks)**:
  1. System displays "This playlist has no tracks. Nothing to import."
  2. Returns to step 8

- **14a. Catalog view fails to load after import**:
  1. System displays "Import completed successfully but we couldn't load the catalog. Pull to refresh or go back and try again."
  2. Data is persisted — only the display failed

## Variations

- **1a.** User may paste a Spotify playlist URL directly on the home screen (quick import) → skips navigation, proceeds to step 8 (after OAuth if needed)
- **8a.** User may enter a Spotify URI (`spotify:playlist:xxxxx`) instead of a URL → System accepts both formats
- **10a.** For very large playlists (>100 tracks), System shows a progress indicator with "Importing track X of Y..." → same flow, just UI feedback during pagination

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test spotify_import`
- **Test File**: `backend/tests/spotify_import.rs`
- **Depends On**: None (first use case, requires only Sprint 0 scaffold)
- **Blocks**: UC-002 (Enrich Track Metadata), UC-003 (Browse Catalog), UC-006 (Preview/Play Track)
- **Estimated Complexity**: L / ~2000 tokens implementation budget
- **Agent Assignment**: Teammate:Builder (backend OAuth + import), Teammate:Builder (frontend import screen)

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] Code passes quality gates (`cargo fmt --check && cargo clippy -- -D warnings && cargo test` + `flutter analyze && flutter test`)
- [ ] Reviewer agent approves
- [ ] Spotify OAuth Authorization Code flow works end-to-end (mock in tests, real in manual smoke test)
- [ ] Importing the 54-track "Salamic Vibes" playlist produces exactly 54 track rows (manual smoke test)
- [ ] Re-importing the same playlist upserts without creating duplicates
- [ ] Rate limiting is handled gracefully (wiremock test with 429 responses)
- [ ] Token refresh works when access token expires mid-import (wiremock test)
- [ ] Frontend displays import progress and summary
