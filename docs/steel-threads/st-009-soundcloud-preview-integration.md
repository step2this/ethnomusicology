# Steel Thread: ST-009 Add SoundCloud as Preview Source for Underground Catalog

## Classification
- **Goal Level**: 🧵 Thread — prove SoundCloud OAuth → search → preview stream → frontend playback
- **Scope**: System (backend + frontend)
- **Priority**: P2 Medium (underground electronic catalog — Deezer+iTunes miss these)
- **Complexity**: 🟡 Medium (OAuth adds complexity vs free APIs)

## Cross-Cutting References

- **UC-019**: Steps 3-5 — preview playback with multi-source fallback
  - Postcondition: underground tracks not on Deezer/iTunes play via SoundCloud
  - Postcondition: preview source indicated to user
- **ST-008**: Precondition — unified `/api/audio/search` endpoint exists with Deezer+iTunes
- **Phase 4**: Precondition — Deezer field-specific search operational

This thread proves: when both Deezer and iTunes miss (common for underground electronic, white labels, bootleg remixes), the backend tries SoundCloud's API and returns a working preview.

## Actors

- **Primary Actor**: App User (DJ)
- **Supporting Actors**:
  - SoundCloud API (`api.soundcloud.com` — OAuth 2.1 Client Credentials)
  - SoundCloud CDN (audio streaming)
- **Stakeholders & Interests**:
  - DJ User: underground electronic tracks are the core value prop; SoundCloud is where these live
  - Product: SoundCloud skew toward independent artists differentiates us from mainstream tools

## Conditions

### Preconditions
1. ST-008 complete — unified `/api/audio/search` endpoint with fallback chain
2. SoundCloud OAuth 2.1 app credentials registered (manual step)
3. Backend has `SOUNDCLOUD_CLIENT_ID` and `SOUNDCLOUD_CLIENT_SECRET` env vars

### Success Postconditions
1. Backend acquires SoundCloud OAuth token via Client Credentials flow
2. SoundCloud search returns tracks matching artist/title
3. SoundCloud preview audio streams through backend proxy
4. Frontend shows "SoundCloud" as preview source when used
5. Fallback chain: Deezer → iTunes → SoundCloud → no-preview

### Failure Postconditions
1. SoundCloud credentials not configured → SoundCloud step skipped silently
2. SoundCloud API timeout → returns Deezer+iTunes results only
3. SoundCloud token refresh failure → logs error, skips SoundCloud

### Invariants
1. Existing Deezer and iTunes playback unaffected
2. SoundCloud OAuth token cached and refreshed automatically
3. Rate limit: stay under 15,000 stream requests/24h

## API Contract

| Method | Path | Description | Schema Ref | Status |
|--------|------|-------------|------------|--------|
| GET | /api/audio/search | Extend: add SoundCloud as third fallback | — | Extend |
| GET | /api/audio/proxy | Extend: whitelist SoundCloud CDN hosts | — | Extend |

### `GET /api/audio/search` (extended response)

Add `"soundcloud"` as a possible `source` value:
```json
{
  "source": "deezer" | "itunes" | "soundcloud" | null,
  "preview_url": "/api/audio/proxy?url=...&source=soundcloud" | null,
  "external_url": "https://soundcloud.com/artist/track" | null,
  "search_query": "artist:\"X\" track:\"Y\"",
  "soundcloud_id": 123456 | null
}
```

### SoundCloud-specific backend internals (not exposed as API):
- Token management: `POST https://api.soundcloud.com/oauth2/token` (Client Credentials)
- Search: `GET https://api.soundcloud.com/tracks?q={query}&limit=5`
- Audio: use `preview_mp3_128_url` (confirmed NOT deprecated per SP-006). Do NOT use HLS streams.

## Main Success Scenario

1. **[Frontend → API]** Call `GET /api/audio/search?title=X&artist=Y`
2. **[API]** Try Deezer (miss) → try iTunes (miss)
3. **[API]** Check if SoundCloud credentials configured
4. **[API → SoundCloud]** Ensure valid OAuth token (cache or refresh)
5. **[API → SoundCloud]** `GET /tracks?q={artist} {title}&limit=5`
6. **[API]** Parse response, find best match by title/artist similarity
7. **[API]** Extract `preview_mp3_128_url` (confirmed available per SP-006)
8. **[API → Frontend]** Return `source: "soundcloud"`, proxied URL, SoundCloud permalink
9. **[Frontend]** Show SoundCloud icon as source indicator
10. **[Frontend]** User clicks play → proxy fetches from SoundCloud CDN
11. **[API → SoundCloud CDN]** Proxy MP3/AAC audio to browser
12. **[Frontend]** Audio plays via Web Audio API

## Extensions

- **3a. SoundCloud credentials not configured (env vars missing)**:
  1. Skip SoundCloud, return `source: null`
- **4a. OAuth token expired**:
  1. Request new token via Client Credentials flow
  2. Cache new token with expiry
  3. Retry search
- **4b. OAuth token request fails (invalid credentials)**:
  1. Log error, skip SoundCloud for this request
  2. Set circuit breaker: disable SoundCloud for 5 minutes
- **5a. SoundCloud API returns 429 (rate limited)**:
  1. Log warning, skip SoundCloud
  2. Respect Retry-After header for next attempt
- **5b. SoundCloud search returns no results**:
  1. Return `source: null`
- **6a. Best match has low title/artist similarity (<50%)**:
  1. Discard match, return `source: null` (avoid wrong tracks)
- **7a. Track has no preview URL (streamable: false)**:
  1. Some SoundCloud tracks don't allow streaming. Return `source: null`.

## Integration Assertions

1. **[API → SoundCloud]** Backend can acquire OAuth token via Client Credentials
2. **[API → SoundCloud]** Backend can search tracks and parse response
3. **[API → SoundCloud CDN]** Backend can proxy audio stream with correct headers
4. **[Frontend → API → CDN]** Full round trip: search → proxy → play in browser
5. **[API]** Fallback chain Deezer → iTunes → SoundCloud completes in <5s
6. **[API]** When Deezer or iTunes succeed, SoundCloud is NOT called
7. **[API]** Token refresh works transparently without user intervention
8. **[API]** Fuzzy match filtering prevents wrong-track playback

## API Terms Compliance (MANDATORY)

Per [SoundCloud API Terms of Use](https://developers.soundcloud.com/docs/api/terms-of-use):

### Attribution (required for every SC-sourced track)
1. **Uploader credit**: display uploader username from API response
2. **Source credit**: show "via SoundCloud" label
3. **Backlink**: clickable link to track's `permalink_url` on soundcloud.com

### Architecture Constraints
- **Playback-only**: SoundCloud metadata MUST NOT be included in LLM prompts or catalog. SC is a preview source, not a generation input.
- **Session-only caching**: preview URLs stored in-memory only (Riverpod state). Never persist SC preview URLs to database.
- **No content modification**: play audio as-is via proxy, no remixing or processing.

### Branding
- Display as "SoundCloud" (capital S, capital C)
- Cannot use in app name or suggest endorsement

### Backend Response Must Include
- `uploader_name` (String) — for attribution display
- `external_url` (String) — permalink to soundcloud.com for backlink

## Does NOT Prove

- SoundCloud user authentication (we use app-level Client Credentials, not user OAuth)
- SoundCloud Go+ streaming (paid tier — we use free preview/stream)
- AAC HLS migration (SoundCloud moving from MP3; we use preview_mp3_128_url for now)
- Full SoundCloud library import (separate UC)
- Rate limit management under load (MVP has single-digit concurrent users)

## Agent Execution Notes

- **Verification Command**: `cargo test && cd ../frontend && flutter test`
- **Test File**: `backend/tests/audio_search_test.rs` (extend), `backend/src/services/soundcloud.rs` (new)
- **Depends On**: ST-008 (unified search endpoint), SoundCloud app credentials (manual)
- **Blocks**: Nothing (terminal in current roadmap)
- **Estimated Complexity**: M / ~500 lines (OAuth + search + proxy + frontend)
- **Agent Assignment**: Lead coordinates, 2 builders (backend + frontend)
- **Manual prerequisite**: Register SoundCloud app at developers.soundcloud.com

## Acceptance Criteria

- [ ] Backend acquires SoundCloud OAuth token and caches it
- [ ] SoundCloud search returns tracks for underground electronic queries
- [ ] SoundCloud preview audio plays in browser via proxy
- [ ] SoundCloud CDN hosts whitelisted in proxy
- [ ] Frontend shows SoundCloud source indicator
- [ ] Fallback chain: Deezer → iTunes → SoundCloud → none
- [ ] Graceful degradation when SoundCloud credentials not configured
- [ ] Fuzzy match filtering prevents wrong-track playback
- [ ] SoundCloud attribution: uploader name + "via SoundCloud" + backlink displayed
- [ ] SoundCloud metadata NOT used in any LLM prompt or catalog
- [ ] Preview URLs NOT persisted to database (session-only)
- [ ] All quality gates pass
- [ ] Critic agent approves
