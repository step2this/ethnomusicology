# Plan: UC-019 Deezer Preview Playback in Flutter
# Also saved to: docs/tasks/uc-019-tasks.md (on ExitPlanMode)

## Context

Audio crossfade PoC works at `tarab.studio/audio-poc.html`. Backend Deezer search proxy exists. Now wire into Flutter app. Critic review found 3 critical issues that reshape the approach.

## Critical Fixes from Critic Review

1. **`dart:web_audio` doesn't exist in Flutter Web** → Use `package:web` + `dart:js_interop` instead
2. **Deezer preview MP3 CORS may fail** → Add backend audio proxy endpoint to stream MP3s
3. **`dart:html` breaks non-web platforms** → Abstract audio service behind interface with web-only implementation
4. **PoC uses linear crossfade** → Implement equal-power curves (cos/sin) per UC-019 spec

## Scoped IN (today)

- Play button on each track tile
- Single-track preview playback
- Crossfade between adjacent tracks
- Crossfade duration slider
- Singleton AudioContext (user-gesture-gated)
- Backend audio proxy for CORS-safe MP3 streaming
- Prefetch Deezer URLs for all tracks on setlist display (reduce tap-to-play latency)

## Explicitly scoped OUT (future iteration)

- Waveform visualization (UC-019 step 7)
- Skip to next/previous transition navigation (UC-019 step 8)
- Purchase link fallback (UC-019 extensions 2a-2c)
- Retry on audio load failure (UC-019 extension 4a)
- ISRC-based Deezer lookup (accuracy improvement)

## Tasks

### T0: Define interfaces and add dependencies (Lead)
- Add `web: ^1.1.0` to `frontend/pubspec.yaml`
- Define `AudioPlaybackService` abstract class with method signatures
- Define `DeezerPreviewProvider` and `AudioPlaybackProvider` state classes
- This unblocks all builders to work in parallel against the interface

### T1: Backend audio proxy endpoint
**File:** `backend/src/routes/audio.rs`
- Add `GET /api/audio/proxy?url=<encoded-url>` that fetches and streams MP3 bytes
- Validate URL is from `*.dzcdn.net` (don't become an open proxy)
- Stream response with `Content-Type: audio/mpeg`
- Existing Deezer search endpoint stays as-is

### T2: ApiClient + DeezerPreviewProvider
**Files:** `frontend/lib/services/api_client.dart`, `frontend/lib/providers/deezer_provider.dart` (NEW)
- Add `searchDeezerPreview(String title, String artist)` to ApiClient
- `DeezerPreviewProvider`: manages map of `{trackId -> previewUrl}`
- On setlist display, prefetch all Deezer URLs in background (batch, not lazy)
- Never cache expired URLs — Deezer URLs have `exp=` param, re-fetch if stale

### T3: AudioPlaybackService (web implementation)
**Files:** `frontend/lib/services/audio_service.dart` (NEW — abstract), `frontend/lib/services/audio_service_web.dart` (NEW — web impl)
- Abstract interface: `loadAndPlay(url)`, `playCrossfade(urlA, urlB, duration)`, `stop()`, `dispose()`
- Web impl uses `package:web` AudioContext, GainNode, AudioBufferSourceNode
- Singleton AudioContext, created on first user gesture
- Equal-power crossfade: pre-computed cos/sin gain curves via `setValueCurveAtTime()`
- Fetch MP3 via backend proxy (`/api/audio/proxy?url=...`) to avoid CORS
- Non-web stub returns "Audio preview requires a web browser"

### T4: AudioPlaybackProvider
**File:** `frontend/lib/providers/audio_provider.dart` (NEW)
- State: `isPlaying`, `isLoading`, `currentTrackIndex`, `crossfadeDuration`, `error`
- Depends on `DeezerPreviewProvider` (URLs) and `AudioPlaybackService` (playback)
- Methods: `playTrack(index)`, `playCrossfade(indexA, indexB)`, `stop()`
- Stops playback on `dispose()` and when setlist is reset

### T5: UI — play button + crossfade controls
**Files:** `frontend/lib/widgets/setlist_track_tile.dart`, `frontend/lib/screens/setlist_generation_screen.dart`
- SetlistTrackTile: trailing play/stop IconButton, loading spinner while fetching
- "No preview" text if Deezer URL is null
- Playback bar between header and track list: Play/Stop, crossfade slider (1-8s), status text
- Stop audio in `_resetAll()`

### T6: Tests
- ApiClient: mock Deezer search response
- DeezerPreviewProvider: batch prefetch, cache, expiry
- AudioPlaybackProvider: state transitions with mocked AudioPlaybackService
- SetlistTrackTile widget test: play button renders, tap triggers provider
- Note: actual Web Audio untestable in headless runner — manual browser + Playwright E2E

## Agent Assignment

| Builder | Tasks | Files (exclusive) |
|---------|-------|-------------------|
| **backend-builder** | T1 | `backend/src/routes/audio.rs` |
| **audio-builder** | T3, T4 | `frontend/lib/services/audio_service*.dart`, `frontend/lib/providers/audio_provider.dart` |
| **api-builder** | T2, T6 (api + provider tests) | `frontend/lib/services/api_client.dart`, `frontend/lib/providers/deezer_provider.dart`, tests |
| **ui-builder** | T5, T6 (widget tests) | `frontend/lib/widgets/setlist_track_tile.dart`, `frontend/lib/screens/setlist_generation_screen.dart`, tests |
| **Lead** | T0 (interfaces), wiring, deploy | `frontend/pubspec.yaml`, abstract class defs, main.rs route wiring |

Dependencies: T0 first (unblocks all). T1 and T2 parallel. T3 parallel with T1+T2. T4 after T2+T3. T5 after T4.

## Verification

1. `cargo fmt --check && cargo clippy -- -D warnings && cargo test` — backend passes
2. `flutter analyze && flutter test` — frontend passes
3. Deploy to tarab.studio
4. Manual browser test: generate setlist → see play buttons → tap one → hear 30s preview
5. Manual browser test: tap crossfade between adjacent tracks → hear smooth equal-power transition
6. Manual test: track with no Deezer result → "No preview" shown gracefully
7. Manual test: reset setlist → audio stops
