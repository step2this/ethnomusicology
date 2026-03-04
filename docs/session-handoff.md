# Session Handoff — 2026-03-04

## Branch: `main` (all work merged directly)

### Active Sessions

| Session | Steel Thread | Status | Files Owned |
|---------|-------------|--------|-------------|
| Session A | ST-008 (iTunes Preview Fallback) | IN PROGRESS | backend/src/routes/audio.rs, backend/src/services/match_scoring.rs, backend/src/services/mod.rs, backend/src/main.rs, frontend/lib/services/api_client.dart, frontend/lib/providers/deezer_provider.dart, frontend/lib/widgets/setlist_track_tile.dart, docs/api/openapi.yaml |
| Session B | SP-006 Spike + Process | COMPLETE | docs/spikes/, docs/steel-threads/, docs/tasks/, docs/research/ |

### T0 Status: COMPLETE
- Commit `10226e0`: Renamed Deezer* types to Preview* across all frontend files
- All consuming files updated (audio_provider, refinement_provider, setlist_result_view, setlist_track_tile, screens, tests)

### ST-008 Execution Plan
- **Phase 1** (parallel): T1 (backend unified search + iTunes + proxy fix) + T2 (match scoring utility)
- **Phase 2**: T3 (frontend provider + tile updates for multi-source)
- **Phase 3**: T4 (tests)
- **Phase 4**: C1 (critic review)

### SP-006 SoundCloud Spike: CONDITIONAL PASS
- `preview_mp3_128_url` confirmed NOT deprecated — our audio path for ST-009
- HLS AAC migration only affects full streaming, not previews
- Client Credentials flow sufficient (no user OAuth needed)
- **Manual step required**: User must register SoundCloud app at `soundcloud.com/you/apps`
- Set `SOUNDCLOUD_CLIENT_ID` and `SOUNDCLOUD_CLIENT_SECRET` env vars before ST-009

### Forge Process Completed This Session
- ST-008 + ST-009 steel threads written with devil's advocate review
- SP-006 spike researched and documented
- Task decompositions for both steel threads
- T0 frontend rename complete
- PRD, roadmap, progress, research docs all updated

### Previous Session Summary

**ST-007 Frontend (PR #5 — merged):**
- Conversational refinement UI: chat input, conversation history, version history panel
- Quick commands (!shuffle, !sort-by-bpm, !reverse, !undo)

**Playback Simplification (PR #6 — merged):**
- Removed crossfade, added admin wipe endpoint, Deezer search status indicators

**Phase 4: Deezer Search Quality (direct to main):**
- Field-specific search with 3-step fallback chain

### Test Counts
- Backend: 332 tests
- Frontend: 148 tests
- **Total: 480 tests, all passing**

### Current Deployment
- `tarab.studio` — latest code deployed (Phase 4 Deezer search fix)
- Catalog is EMPTY (wiped) — user needs to re-import Spotify playlist
