# Session Handoff — 2026-03-04

## Branch: `feature/playback-simplification-and-debugging`

### Status: IN PROGRESS
Playback simplification and debugging — Phases 1-4 of post-ST-007 manual testing fixes.

### Previous Session
- ST-007 backend + frontend: COMPLETE, merged via PR #5
- Deployed to tarab.studio (manual deploy from EC2)
- Manual testing revealed playback issues → this session

### What's Being Built

1. **Phase 1: Data Cleanup** — Admin endpoint to wipe Spotify catalog data (stale tracks)
2. **Phase 2: Simplify Playback** — Remove crossfade, use simple sequential 30s previews
3. **Phase 3: Deezer Debug Infrastructure** — Per-track search status indicators (found/notFound/error) with search query tooltips
4. **Phase 4: Track Attribution Links** — Clickable track titles/artists → Google search, Spotify links for catalog tracks

### File Ownership

| Owner | Files |
|-------|-------|
| backend-builder | `routes/admin.rs` (NEW), `main.rs`, `db/models.rs`, `db/setlists.rs`, `db/refinement.rs`, `services/setlist.rs` |
| frontend-builder | `audio_service.dart`, `audio_service_web.dart`, `audio_provider.dart`, `transport_controls.dart`, `setlist_result_view.dart`, `constants.dart`, `deezer_provider.dart`, `setlist_track_tile.dart`, `setlist_track.dart` |
| test-builder | `audio_provider_test.dart`, `deezer_provider_test.dart`, `admin_wipe_test.rs` (NEW) |

### Test Counts (pre-implementation)
- Backend: 328 tests
- Frontend: 145 tests
- **Total: 473 tests**
