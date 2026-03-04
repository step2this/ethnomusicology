# UC-019 Phase 3: Audio UX Improvements — Critic Fix Pass

## Context

Phase 1 (Deezer playback in Flutter) and Phase 2 (Deezer enrichment pipeline) are complete and deployed. Phase 3 adds transport controls (prev/next/pause), auto-advance with crossfade, and a `PlaybackStatus` enum replacing boolean flags.

Implementation is complete on branch `feature/audio-ux-improvements` (unstaged, 7 files, +369/-115 lines). Three parallel critic agents reviewed the code cold and found issues that must be fixed before commit.

## Critic Review Summary

| Critic | CRITICAL | HIGH | MEDIUM | LOW |
|--------|----------|------|--------|-----|
| State Management | 2 | 3 | 6 | 4 |
| Audio Service | 0 | 3 | 3 | 4 |
| UI/Widgets | 0 | 3 | 4 | 4 |

## Scoped IN (this pass — CRITICAL + actionable HIGH)

### T1: Fix `previous()` to skip unplayable tracks (CRITICAL)
**File:** `frontend/lib/providers/audio_provider.dart`
- `next()` uses `_findNextPlayableTrack()` but `previous()` blindly decrements by 1
- If prior track has no Deezer URL → error state, user stuck
- **Fix:** Add `_findPreviousPlayableTrack()` that scans backward, use it in `previous()`
- **Test:** Add test in `audio_provider_test.dart`: fixture with tracks [0:url, 1:no-url, 2:url], play track 2, call previous(), verify lands on track 0

### T2: Fix fire-and-forget `_playCurrentTrack` in `_handleTrackEnded` (CRITICAL)
**File:** `frontend/lib/providers/audio_provider.dart`
- The `else` branch at ~line 191 calls `_playCurrentTrack()` without `.catchError()`
- Unhandled Future error can crash or leave status stuck at `loading`
- **Fix:** Attach `.catchError()` matching the crossfade branch pattern
- **Test:** Not easily testable with NoOp stub (error path), but verify code path exists

### T3: Clean up state in `stop()` and `dispose()` (HIGH)
**File:** `frontend/lib/providers/audio_provider.dart`
- `stop()` does not null `_tracks`, `_deezerState`, or clear `_audioService.onTrackEnded`
- Stale `onTrackEnded` callback can trigger ghost playback after stop
- `dispose()` also does not clean up `_tracks`/`_deezerState`
- **Fix:** Add cleanup in both `stop()` and `dispose()`
- **Test:** Add test: play track, stop(), verify triggerTrackEndedForTest is no-op

### T4: Fix pause icon showing in idle state (HIGH)
**File:** `frontend/lib/screens/setlist_generation_screen.dart`
- Transport play/pause button shows `Icons.pause` when `isPaused` is false (includes idle/loading/completed/error)
- **Fix:** Change icon logic: `audioState.isPlaying ? Icons.pause : Icons.play_arrow`
- Also: Disable stop button when status is idle/completed

### T5: Remove dead `onStop` prop from SetlistTrackTile (HIGH)
**File:** `frontend/lib/widgets/setlist_track_tile.dart`, `frontend/lib/screens/setlist_generation_screen.dart`
- `onStop` is declared, passed in, but never wired to any widget after the refactor
- **Fix:** Remove `onStop` from tile constructor + caller

### T6: Web Audio node disconnect + generation counter (HIGH)
**File:** `frontend/lib/services/audio_service_web.dart`
- Source/gain nodes never `disconnect()`'d — memory leak over many plays
- Stale `ended` callbacks from stopped nodes can corrupt `_isPlaying` (rapid play)
- `_stopSources()` calling `.stop()` fires `ended` event, racing with new playback
- **Fix:**
  - Add `disconnect()` calls in `_stopSources()` (with try/catch)
  - Add `_playGeneration` counter; increment on each `loadAndPlay`/`playCrossfade`; stale callbacks check generation before acting
  - Clear `_onTrackEnded = null` at top of `_stopSources()` to prevent stale ended events

### T7: Add HTTP status check in `_loadAudio` (HIGH)
**File:** `frontend/lib/services/audio_service_web.dart`
- `_loadAudio` calls `fetch()` but never checks `response.ok`
- 404/500 responses passed to `decodeAudioData` produce cryptic errors
- **Fix:** Check `response.ok`, throw descriptive error on non-2xx

### T8: Fix misleading equal-power comment (LOW — while we're there)
**File:** `frontend/lib/services/audio_service.dart`
- Interface comment says "equal-power curves (cos/sin)" but impl uses linear ramps
- **Fix:** Update comment to "linear gain ramps (equal-power deferred)"

### T9: Fix leaking ProviderContainers in tests (MEDIUM — while we're there)
**File:** `frontend/test/providers/audio_provider_test.dart`
- Several tests create `ProviderContainer()` for `DeezerPreviewState` but never dispose them
- `DeezerPreviewState()` is a simple const — no need for a container
- **Fix:** Replace with `const DeezerPreviewState()` directly

### T10: Clear stale `statusText` on auto-advance loading (LOW — while we're there)
**File:** `frontend/lib/providers/audio_provider.dart`
- When `_handleTrackEnded` transitions to `loading`, old track's statusText persists
- **Fix:** Add `statusText: () => null` in the loading transition

## Explicitly scoped OUT (acceptable MVP debt)

- Transport bar overflow on narrow screens (web-first MVP, mobile spike later)
- Transport bar widget tests (transport is 4 buttons + slider, tested manually)
- play-from-idle on transport play button (user clicks track tile to start)
- `isPlaying` prop naming → `isActive` (semantic, not functional)
- Re-downloading Track A buffer for crossfade (performance, not correctness)
- `pause()` guard at service level (provider layer already guards)
- `NoOpAudioPlaybackService` state tracking (provider manages state)
- `dispose()` not awaiting `AudioContext.close()` (fire-and-forget is fine)
- Reentrancy guard on concurrent loadAndPlay (edge case, provider manages)

## Agent Assignment

| Builder | Tasks | Files (exclusive ownership) |
|---------|-------|-----------------------------|
| **provider-builder** | T1, T2, T3, T9, T10 | `audio_provider.dart`, `audio_provider_test.dart` |
| **service-builder** | T6, T7, T8 | `audio_service.dart`, `audio_service_web.dart` |
| **ui-builder** | T4, T5 | `setlist_generation_screen.dart`, `setlist_track_tile.dart`, `setlist_generation_test.dart` |

Dependencies: **None** — all three builders work on non-overlapping files and can run fully in parallel.

## Verification

1. `cd frontend && flutter analyze` — no issues
2. `cd frontend && flutter test` — all tests pass (including new tests for T1, T3)
3. Manual: generate setlist → play → previous/next/pause/stop all work
4. Manual: play through to end → "Set complete" shown
5. Manual: rapid-click different tracks → no ghost playback or stuck states
