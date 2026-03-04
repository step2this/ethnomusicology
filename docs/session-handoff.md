# Session Handoff — 2026-03-04

## Branch: `main` (all work merged directly)

### Status: PHASE 4 COMPLETE, PHASES 5 + 5.1 NEXT

### What Was Done This Session

**ST-007 Frontend (PR #5 — merged):**
- Conversational refinement UI: chat input, conversation history, version history panel
- Quick commands (!shuffle, !sort-by-bpm, !reverse, !undo)
- Optimistic UI updates, Deezer prefetch caching
- 6-phase multi-agent build with critic review

**Playback Simplification (PR #6 — merged):**
- Removed crossfade (too complex for 30s previews)
- Added admin wipe endpoint: POST /api/admin/wipe-catalog
- Added spotify_uri to track API responses (LEFT JOIN)
- Added per-track Deezer search status indicators (found/notFound/error + tooltip)
- Added track attribution links (clickable title/artist → Google, Spotify icon)
- Wiped stale catalog data, set ADMIN_TOKEN in production

**Bug fixes (direct to main):**
- Allow generation without catalog tracks (remove EMPTY_CATALOG error)
- Fixed frontend deployment (symlink was pointing to old build)
- Added no-cache header for main.dart.js in Caddy
- Made Deezer status indicators larger (8px → 16px icons)

**Phase 4: Deezer Search Quality (direct to main):**
- Field-specific search: `artist:"X" track:"Y"` with `strict=on`
- 3-step fallback chain: strict → fuzzy → freeform
- Backend passes through `strict` parameter to Deezer API

**Documentation updates:**
- PRD: Track Discovery & Acquisition section, multi-source preview chain, competitive landscape
- MVP roadmap: added Phases 4-6, marked ST-007 complete
- MVP progress: updated all rows, added Phase 4-6 backlog
- New research doc: `docs/research/audio-source-landscape-2026.md`

### Test Counts
- Backend: 332 tests
- Frontend: 148 tests
- **Total: 480 tests, all passing**

### Current Deployment
- `tarab.studio` — latest code deployed (Phase 4 Deezer search fix)
- ADMIN_TOKEN configured in `/etc/ethnomusicology/env`
- Caddy has no-cache for main.dart.js, flutter_service_worker.js, index.html
- Catalog is EMPTY (wiped) — user needs to re-import Spotify playlist

### Next Steps: Phases 5 + 5.1 (Parallel)

**Phase 5: iTunes Search API Fallback**
- Add iTunes Search API as second preview source when Deezer misses
- Backend: new endpoint or modify existing deezer-search to try iTunes
- Frontend: extend DeezerPreviewProvider to try iTunes as fallback
- Apple Music affiliate links (append `?at={token}`)
- Research: `docs/research/audio-source-landscape-2026.md`

**Phase 5.1: SoundCloud Preview Integration**
- OAuth 2.1 Client Credentials flow for SoundCloud API
- Search + preview stream for underground/independent catalog
- SoundCloud migrating to AAC HLS — use new stream endpoint
- Backend: new SoundCloud search/proxy endpoints
- Frontend: extend preview chain to include SoundCloud

**Implementation approach:** Parallel worktrees (non-overlapping files)
- Worktree 1: iTunes (backend endpoint + frontend fallback)
- Worktree 2: SoundCloud (backend OAuth + endpoint + frontend integration)

### Key Files Modified This Session
| Area | Files |
|------|-------|
| Backend admin | `routes/admin.rs` (NEW), `main.rs`, `routes/mod.rs` |
| Backend spotify_uri | `db/models.rs`, `db/setlists.rs`, `db/refinement.rs`, `services/setlist.rs` |
| Backend search | `routes/audio.rs` (strict param), `api/claude.rs` (empty catalog prompt) |
| Frontend crossfade removal | `audio_service.dart`, `audio_service_web.dart`, `audio_provider.dart`, `transport_controls.dart`, `constants.dart` |
| Frontend deezer status | `deezer_provider.dart`, `setlist_track_tile.dart`, `setlist_result_view.dart` |
| Frontend attribution | `setlist_track.dart` (spotifyUri), `setlist_track_tile.dart` (links) |
| Frontend search | `api_client.dart` (field-specific fallback chain) |
| Docs | `prd.md`, `mvp-roadmap.md`, `mvp-progress.md`, `research/audio-source-landscape-2026.md` |
