# Task Decomposition: ST-009 SoundCloud Preview Integration

## Prerequisite: SP-006 (SoundCloud API Feasibility Spike)
Must complete SP-006 before building ST-009. The spike confirms whether `preview_mp3_128_url` is still available post-Nov 2025 migration. If HLS-only, ST-009 design changes significantly.

## Devil's Advocate Resolutions

- **C1 (preview URL deprecation)**: SP-006 spike first — confirms audio delivery path
- **M4 (token storage)**: In-memory with lazy init + circuit breaker (5min disable on failure)
- **M5 (CDN hosts)**: Document from spike observations, use pattern-based whitelist
- **L4 (stream_url auth)**: Proxy injects SoundCloud token if needed (design from spike)

## Task Dependency Graph (post-spike)

```
SP-006 (spike) ──→ T1 (backend: SoundCloud service + OAuth) ──→ T2 (backend: extend search chain) ──→ T3 (frontend: source indicator) ──→ T4 (tests) ──→ C1 (critic)
```

All tasks are sequential — each builds on the previous.

## Tasks

### T1: Backend SoundCloud service + OAuth (~120 lines) — `backend-builder`
**Files:**
- `backend/src/services/soundcloud.rs` (NEW):
  - `SoundCloudClient` struct with `client_id`, `client_secret`, `token: Option<String>`, `token_expires: Instant`
  - `async fn ensure_token(&mut self)` — Client Credentials flow, cache token
  - `async fn search_track(&mut self, title: &str, artist: &str) -> Option<SoundCloudTrack>`
  - Circuit breaker: disable for 5min after 3 consecutive auth failures
  - Rate awareness: check for 429 status, respect Retry-After
- `backend/src/services/mod.rs` (MODIFY — add module)

### T2: Backend extend unified search chain (~40 lines) — `backend-builder`
**Files:**
- `backend/src/routes/audio.rs` (MODIFY):
  - After iTunes fallback, try SoundCloud if configured
  - Check `SOUNDCLOUD_CLIENT_ID` env var — skip if not set
  - Use match scoring from ST-008 T2 for result validation
  - Add SoundCloud CDN hosts to proxy whitelist (from spike findings)
  - Response shape: add `"soundcloud"` as source option + `soundcloud_id`

### T3: Frontend source indicator for SoundCloud (~20 lines) — `frontend-builder`
**Files:**
- `frontend/lib/widgets/setlist_track_tile.dart` (MODIFY):
  - Add SoundCloud icon when source="soundcloud"
  - SoundCloud permalink as external_url
- `frontend/lib/providers/deezer_provider.dart` (MODIFY if needed — by now it's preview_provider)

### T4: Tests (~60 lines) — `test-builder`
- `backend/src/services/soundcloud.rs` (inline tests): token management, circuit breaker
- `backend/tests/audio_search_test.rs` (extend): SoundCloud in chain
- Frontend tests: SoundCloud source display

### C1: Critic review

## File Inventory

| File | Action | Owner | Task |
|------|--------|-------|------|
| `backend/src/services/soundcloud.rs` | NEW | backend-builder | T1 |
| `backend/src/services/mod.rs` | MODIFY | backend-builder | T1 |
| `backend/src/routes/audio.rs` | MODIFY | backend-builder | T2 |
| `frontend/lib/widgets/setlist_track_tile.dart` | MODIFY | frontend-builder | T3 |

**Estimated total: ~240 lines across 4 files (post-spike, pending audio path confirmation)**
