# Use Case: UC-019 Crossfade Preview Between Tracks

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P1 Important
- **Complexity**: 🟠 High

## Actors
- **Primary Actor**: App User (authenticated, DJ)
- **Supporting Actors**:
  - Browser Web Audio API (client-side audio processing)
  - Audio source APIs (Spotify preview URLs in v1; SoundCloud streaming deferred to v2)
  - Database (track metadata and audio URLs)
- **Stakeholders & Interests**:
  - DJ User: Wants to hear how two adjacent tracks in a setlist sound together during a crossfade transition — validates whether the harmonic arrangement actually sounds good
  - Developer: Wants client-side audio processing (no server-side mixing) using Web Audio API for low latency

## Conditions
- **Preconditions** (must be true before starting):
  1. User has a setlist with at least 2 adjacent tracks
  2. At least one of the two tracks has an audio source URL (Spotify preview URL in v1)
  3. Browser supports Web Audio API (all modern browsers)

- **Success Postconditions** (true when done right):
  1. User hears a crossfade preview: track A's last 3-5 seconds fade out while track B's first 3-5 seconds fade in, overlapping
  2. Crossfade duration is configurable (default 4 seconds)
  3. Volume curves follow a smooth equal-power crossfade (not linear — prevents volume dip)
  4. Preview plays entirely client-side — no server-side audio processing
  5. Playback controls: play/pause, skip to next transition, adjust crossfade duration

- **Failure Postconditions** (true when it fails gracefully):
  1. If audio for one track is unavailable, play the other track solo with a visual indicator: "Preview unavailable for [track name]"
  2. If both tracks lack audio, display "No preview available for this transition" with a purchase link (UC-020)
  3. Audio loading failures show retry option

- **Invariants** (must remain true throughout):
  1. Audio is fetched from platform-provided URLs (Spotify preview URLs in v1). No audio files are stored on the server.
  2. No audio processing happens on the server
  3. Playback respects copyright: preview-length only (30s max per track), no full track playback

> **V1 scope**: V1 implementation uses Spotify 30-second preview URLs only. SoundCloud streaming support is deferred to v2 (requires CORS proxy infrastructure). Beatport has no streaming API.
>
> **V2 deliverable**: Backend CORS proxy endpoint (`GET /api/audio/proxy?url=...`) for SoundCloud streams that don't support cross-origin requests.

## Main Success Scenario
1. User views an arranged setlist (from UC-016/017) and taps "Preview Transitions"
2. System loads audio source URLs for the first two adjacent tracks (Spotify preview URLs in v1)
3. Frontend creates two Web Audio API AudioContext source nodes, one per track
4. Frontend positions playback: Track A starts at its final 5 seconds, Track B starts at its beginning
5. Frontend applies equal-power crossfade gain curves: Track A fades from 1.0 to 0.0, Track B fades from 0.0 to 1.0, over 4 seconds (configurable)
6. User hears the crossfade transition between tracks A and B
7. System displays: waveform visualization of both tracks, crossfade overlap region highlighted, BPM and key of both tracks, transition score from UC-017
8. User can tap "Next Transition" to preview the next pair (B→C), or tap any transition in the setlist to jump to it
9. User adjusts crossfade duration with a slider (1-8 seconds)

## Extensions (What Can Go Wrong)

- **1a. Setlist has only 1 track**:
  1. "Preview Transitions" button disabled
  2. User can still play a single track preview

- **2a. Track A has no audio URL (Beatport-only, no stream)**:
  1. System plays Track B solo from the start
  2. Shows: "No preview for [Track A]. Hear it on Beatport: [link]"

- **2b. Track B has no audio URL**:
  1. System plays Track A's ending solo
  2. Shows: "No preview for [Track B]. Hear it on Beatport: [link]"

- **2c. Both tracks lack audio URLs**:
  1. System shows "No audio preview available for this transition"
  2. Displays purchase links for both tracks (UC-020)
  3. User can skip to next transition

- **3a. Web Audio API not supported (very old browser)**:
  1. System detects missing API support
  2. Displays: "Audio preview requires a modern browser. Please update your browser."
  3. Falls back to showing track metadata without audio

- **4a. Audio fails to load (CORS error, network failure)**:
  1. System retries once
  2. If still failing, shows "Couldn't load audio. Check your connection."
  3. Offers retry button

- **4b. Audio format not supported by browser**:
  1. System logs the format issue
  2. Falls back to alternative format if available (SoundCloud AAC HLS → Spotify MP3 preview)
  3. If no compatible format, shows unavailable message (SoundCloud format handling deferred to v2)

- **5a. Crossfade sounds bad (key clash despite Camelot compatibility)**:
  1. This is expected sometimes — Camelot is a guide, not a guarantee
  2. User can adjust crossfade duration or skip this transition
  3. UI shows the transition score to set expectations

- **6a. Audio playback interrupted (browser tab backgrounded)**:
  1. Web Audio API may pause in background tabs (browser-dependent)
  2. Resume playback when tab regains focus

- **9a. User sets crossfade to 0 seconds (hard cut)**:
  1. System performs an instant switch — Track A stops, Track B starts immediately
  2. Valid DJ technique, no error

## Variations

- **V1. Single Track Preview**: User taps a track to hear its preview (not a transition). Plays 30 seconds from the middle of the track.
- **V2. Full Setlist Preview**: User hits "Play All Transitions" — system plays the entire setlist as a sequence of crossfade previews, transitioning every 10-15 seconds per track.
- **V3. A/B Compare**: User compares two different arrangement orders by previewing transitions in each.

## Agent Execution Notes
- **Verification Command**: `cd frontend && flutter test test/crossfade_test.dart`
- **Test File**: `frontend/test/crossfade_test.dart` (unit test for crossfade logic), manual browser test for audio
- **Depends On**: UC-016 (setlist), UC-017 (arranged order with transition scores), UC-001 (Spotify preview URLs)
- **Blocks**: UC-025 (full DJ mix playback extends crossfade concept)
- **Estimated Complexity**: H (~2500 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend — (v1: no backend work needed for Spotify preview URLs) (v2: CORS proxy endpoint for SoundCloud streams)
  - Teammate:Frontend — Web Audio API crossfade engine, waveform visualization, playback controls, transition preview UI

### Key Implementation Details
- **Web Audio API**: Use `AudioContext`, `AudioBufferSourceNode`, `GainNode` for crossfade
- **Equal-power crossfade**: `gainA = cos(t * π/2)`, `gainB = sin(t * π/2)` where t goes from 0→1 over crossfade duration
- **Audio loading**: Fetch audio via `fetch()`, decode with `AudioContext.decodeAudioData()`
- **CORS**: Spotify preview URLs are direct (no proxy needed for v1). SoundCloud CORS proxy deferred to v2.
- **Waveform**: Use `AnalyserNode` for real-time frequency/waveform data visualization
- **Preview length**: Max 30 seconds per track (copyright compliance)
- **No new migration needed** — uses existing track audio URLs

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified (manual browser test + unit test)
- [ ] Equal-power crossfade produces smooth volume transition (no dip in the middle)
- [ ] Crossfade duration is adjustable (1-8 seconds)
- [ ] Missing audio URLs handled gracefully with purchase link fallback
- [ ] Web Audio API compatibility checked before attempting playback
- [ ] Waveform visualization renders during playback
- [ ] User can skip between transitions in the setlist
- [ ] CORS handled for cross-origin audio sources
- [ ] Preview respects 30-second max per track
- [ ] Audio resources are properly released after playback (no memory leaks)
