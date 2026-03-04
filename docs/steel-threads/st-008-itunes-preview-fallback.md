# Steel Thread: ST-008 Add iTunes Search API as Preview Fallback Source

## Classification
- **Goal Level**: 🧵 Thread — prove iTunes Search API → backend proxy → frontend playback chain
- **Scope**: System (backend + frontend)
- **Priority**: P1 High (Deezer misses ~10-20% of electronic tracks; iTunes has 100M+ catalog)
- **Complexity**: 🟡 Medium

## Cross-Cutting References

- **UC-019**: Steps 3-5 — preview playback with multi-source fallback
  - Postcondition: tracks not found on Deezer play via iTunes preview
  - Postcondition: preview source indicated to user (Deezer vs iTunes)
- **UC-020**: Steps 1-2 — Apple Music purchase links with affiliate token
- **ST-006**: Precondition — setlist generation returns tracks with title/artist
- **Phase 4**: Precondition — Deezer field-specific search operational

This thread proves: when Deezer search returns no preview for a track, the backend automatically tries iTunes Search API and returns a working 30s preview from Apple's CDN.

## Actors

- **Primary Actor**: App User (DJ)
- **Supporting Actors**:
  - Deezer API (primary preview source)
  - iTunes Search API (fallback preview source — `itunes.apple.com/search`)
  - Apple CDN (`audio-ssl.itunes.apple.com` — AAC audio hosting)
- **Stakeholders & Interests**:
  - DJ User: wants to hear every track in the setlist, not just the ones Deezer has
  - Product: Apple affiliate links are a near-term revenue opportunity

## Conditions

### Preconditions
1. Setlist generation returns tracks with title and artist fields
2. Deezer search endpoint operational (Phase 4)
3. Backend can proxy external audio (existing `/api/audio/proxy` for Deezer CDN)

### Success Postconditions
1. Backend `/api/audio/search` returns preview URL from Deezer OR iTunes (unified endpoint)
2. iTunes preview audio plays correctly in browser via backend proxy
3. Frontend displays preview source (Deezer/iTunes/none) per track
4. Apple Music link available for tracks found via iTunes
5. Tracks not found on either source show "No preview available"

### Failure Postconditions
1. iTunes API timeout → Deezer-only results returned (graceful degradation)
2. iTunes API returns no match → track marked as no-preview
3. iTunes preview URL expired/broken → playback error handled cleanly

### Invariants
1. Existing Deezer-only playback continues to work unchanged
2. No regression in Deezer search accuracy (field-specific queries preserved)
3. Preview chain latency < 3s per track for prefetch

## API Contract

| Method | Path | Description | Schema Ref | Status |
|--------|------|-------------|------------|--------|
| GET | /api/audio/search | Unified preview search — tries Deezer then iTunes | — | Draft |
| GET | /api/audio/proxy | Proxy audio from Deezer CDN or Apple CDN | — | Extend |

### `GET /api/audio/search`

Replaces the current `/api/audio/deezer-search` with a unified endpoint that tries multiple sources.

**Query params:**
- `title` (required) — track title
- `artist` (required) — artist name

**Response:**
```json
{
  "source": "deezer" | "itunes" | null,
  "preview_url": "/api/audio/proxy?url=...&source=deezer" | null,
  "external_url": "https://music.apple.com/..." | "https://www.deezer.com/track/..." | null,
  "search_query": "artist:\"X\" track:\"Y\"",
  "deezer_id": 12345 | null,
  "itunes_id": 67890 | null
}
```

### `GET /api/audio/proxy` (extended)

Add `source` param to whitelist Apple CDN hosts alongside Deezer CDN:
- `*.dzcdn.net` (existing)
- `audio-ssl.itunes.apple.com` (new)

## Main Success Scenario

1. **[Frontend]** User generates setlist → tracks rendered with preview status "loading"
2. **[Frontend → API]** For each track, call `GET /api/audio/search?title=X&artist=Y`
3. **[API]** Backend tries Deezer field-specific search (strict → fuzzy)
4. **[API → Deezer]** Deezer returns match with preview URL → return `source: "deezer"`
5. **[API]** If Deezer returns no match: try iTunes Search API
6. **[API → iTunes]** `GET https://itunes.apple.com/search?term={artist}+{title}&media=music&entity=song&limit=5`
7. **[API]** Parse iTunes response, find best match, extract `previewUrl`
8. **[API → Frontend]** Return unified response with `source: "itunes"`, proxied preview URL, Apple Music link
9. **[Frontend]** Display preview source indicator (Deezer checkmark / Apple icon / red X)
10. **[Frontend]** User clicks play → audio fetched via `/api/audio/proxy?url=...&source=itunes`
11. **[API → Apple CDN]** Backend proxies AAC audio from `audio-ssl.itunes.apple.com`
12. **[Frontend]** Audio plays via Web Audio API (AAC supported in all modern browsers)

## Extensions

- **3a. Deezer API timeout (>5s)**:
  1. Log warning, skip to iTunes fallback (step 5)
- **5a. iTunes API timeout (>5s)**:
  1. Return `source: null` — track has no preview
- **6a. iTunes returns no results**:
  1. Return `source: null` — track not found on any source
- **6b. iTunes returns results but no previewUrl**:
  1. Some iTunes tracks don't have previews. Return `source: null`.
- **10a. Proxy host validation fails (not Apple CDN)**:
  1. Return 403 Forbidden (existing behavior for non-whitelisted hosts)
- **11a. Apple CDN returns 403/404**:
  1. Preview URL may have expired. Return 502 with error code.

## Integration Assertions

1. **[API → iTunes]** Backend can call iTunes Search API and parse JSON response
2. **[API → Apple CDN]** Backend can proxy AAC audio with correct content-type (`audio/mp4` or `audio/aac`)
3. **[Frontend → API → CDN]** Full round trip: search → get URL → proxy audio → play in browser
4. **[API]** Fallback chain completes in <3s per track (Deezer + iTunes serial)
5. **[Frontend]** Preview source indicator shows correct source for each track
6. **[API]** When Deezer succeeds, iTunes is NOT called (short-circuit optimization)
7. **[API → Frontend]** Apple Music external URL returned for affiliate linking

## Does NOT Prove

- SoundCloud integration (ST-009)
- ISRC-based lookup (future enhancement — requires ISRC in setlist track metadata)
- Apple Music affiliate program registration (manual process)
- Purchase link panel UI (Phase 6, UC-020)
- Rate limiting / quota management for iTunes API
- Batch enrichment of catalog tracks with iTunes IDs (deferred)

## Agent Execution Notes

- **Verification Command**: `cargo test && cd ../frontend && flutter test`
- **Test File**: `backend/tests/audio_search_test.rs` (new), `frontend/test/providers/preview_provider_test.dart` (rename)
- **Depends On**: Phase 4 (Deezer field search), ST-006 (generation)
- **Blocks**: Phase 6 (purchase links need `external_url`)
- **Estimated Complexity**: M / ~600 lines across backend + frontend
- **Agent Assignment**: Lead coordinates, 2 builders (backend + frontend)

## Acceptance Criteria

- [ ] `GET /api/audio/search` returns Deezer results when available
- [ ] `GET /api/audio/search` falls back to iTunes when Deezer misses
- [ ] iTunes preview audio plays in browser via proxy
- [ ] Apple CDN host whitelisted in proxy endpoint
- [ ] Frontend shows source indicator (Deezer/iTunes/none) per track
- [ ] Apple Music external URL returned for affiliate linking
- [ ] No regression in existing Deezer playback
- [ ] All quality gates pass (cargo fmt, clippy, test; flutter analyze, test)
- [ ] Critic agent approves
