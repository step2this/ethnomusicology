# Session Handoff — 2026-03-04

## Branch: `main` (all work merged directly)

### Status: ST-008 COMPLETE

### What Was Done This Session

**ST-008: iTunes Preview Fallback (commit 6cfcc81):**
- New unified `GET /api/audio/search?title=X&artist=Y` endpoint
- Deezer (strict → fuzzy) → iTunes Search API fallback chain
- Match scoring module (`services/match_scoring.rs`) for fuzzy title/artist validation
- Proxy fix: forwards upstream Content-Type (was hardcoded audio/mpeg)
- Proxy extended: Apple CDN hosts whitelisted (audio-ssl.itunes.apple.com, *.mzstatic.com)
- Rate limiting: tokio::sync::Semaphore(20) for iTunes
- Frontend: source-specific indicators (Deezer checkmark / Apple icon / red X)
- Frontend: Apple Music link when source=itunes
- Backward compat: `/api/audio/deezer-search` still works
- OpenAPI spec updated with AudioSearchResponse schema
- Critic review completed (fresh-context opus agent)

**T0 (commit 10226e0, done by prior session):**
- Renamed Deezer* types to Preview* across all frontend files

### Test Counts
- Backend: 352 tests (was 332, +20 new)
- Frontend: 150 tests (was 148, +2 new)
- **Total: 502 tests, all passing**

### Next Steps

**ST-009: SoundCloud Preview Integration** (depends on ST-008 — now unblocked)
- SoundCloud OAuth 2.1 Client Credentials flow
- Extend unified search: Deezer → iTunes → SoundCloud
- Manual prerequisite: register SoundCloud app at soundcloud.com/you/apps
- Set SOUNDCLOUD_CLIENT_ID and SOUNDCLOUD_CLIENT_SECRET env vars
- See `docs/steel-threads/st-009-soundcloud-preview-integration.md`

**Other backlog:**
- Phase 6: Purchase link panel (UC-020)
- Granular generation progress indicators
- iOS/mobile spike

### Key Files Modified
| Area | Files |
|------|-------|
| Backend search | `routes/audio.rs` (unified search + proxy fix) |
| Backend scoring | `services/match_scoring.rs` (NEW), `services/mod.rs` |
| Frontend API | `services/api_client.dart` (searchPreview method) |
| Frontend provider | `providers/deezer_provider.dart` (source, externalUrl, searchQueries) |
| Frontend UI | `widgets/setlist_track_tile.dart` (source icons, Apple Music link) |
| Frontend wiring | `widgets/setlist_result_view.dart` (new field passthrough) |
| Docs | `docs/api/openapi.yaml` (AudioSearchResponse schema) |
| Tests | `test/providers/deezer_provider_test.dart`, `test/services/api_client_test.dart`, `test/providers/audio_provider_test.dart` |
