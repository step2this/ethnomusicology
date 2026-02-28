# Use Case: UC-025 Full Browser-Based DJ Mix Playback (Aspirational)

**Status: Aspirational Beta** — This feature is experimental and may ship as a beta with known limitations.

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P3 Aspirational
- **Complexity**: 🔴 Very High (may be implemented as an extension of UC-019 crossfade rather than a standalone feature)

> **Implementation note**: Start with UC-019's crossfade engine and incrementally add tempo adjustment. If beat-matching proves too unreliable in browser, ship as "Extended Crossfade" (sequential crossfades through the entire setlist without beat-matching).

## Actors
- **Primary Actor**: App User (authenticated, DJ)
- **Supporting Actors**:
  - Browser Web Audio API (client-side audio processing, beat-synced mixing)
  - Audio source APIs (SoundCloud streams, Spotify previews)
  - Database (setlist, track metadata, BPM/key data)
- **Stakeholders & Interests**:
  - DJ User: Wants to hear the entire setlist played back as a continuous DJ mix with beat-matched transitions — the ultimate validation that the setlist works as a real set
  - Developer: This is the most technically ambitious feature — real-time beat-matching in the browser is non-trivial
  - Business: Full mix playback is a strong differentiator if achievable; acceptable to ship as "beta"

## Conditions
- **Preconditions** (must be true before starting):
  1. User has an arranged setlist (UC-016 + UC-017) with at least 3 tracks
  2. Tracks have BPM data (essential for beat-matching)
  3. Tracks have audio source URLs (SoundCloud stream or Spotify preview)
  4. Browser supports Web Audio API with sufficient performance
  5. UC-019 crossfade preview is implemented (this UC extends it)

- **Success Postconditions** (true when done right):
  1. User hears the full setlist played as a continuous mix with beat-synchronized transitions
  2. Each transition uses BPM-matched crossfading: tempo of outgoing track is adjusted (pitch-shifted or time-stretched) to match incoming track's BPM within ±2 BPM
  3. Transitions occur at musically appropriate points (phrase boundaries, ideally every 16 or 32 bars)
  4. Mix sounds listenable — not perfect DJ quality, but demonstrates the setlist's flow
  5. User has playback controls: play/pause, skip forward/back (by track), seek within track, volume
  6. Visual feedback: current track, BPM readout, waveform, progress through setlist, upcoming transition countdown

- **Failure Postconditions** (true when it fails gracefully):
  1. If beat-matching fails for a transition, fall back to simple crossfade (UC-019 style) with no pitch adjustment
  2. If audio for a track is unavailable, skip to next track with audio
  3. If browser performance is insufficient, degrade to crossfade-only mode with a notice

- **Invariants** (must remain true throughout):
  1. All audio processing is client-side (no server-side mixing)
  2. Preview-length limitations apply: max 30s of each track for copyright compliance (unless full streaming is authorized by the platform)
  3. No audio is recorded or saved — real-time playback only
  4. CPU usage stays below 50% on a modern laptop (performance budget)

## Main Success Scenario
1. User views an arranged setlist and taps "Play Full Mix"
2. System checks audio availability for all tracks, warns if some tracks lack audio
3. System pre-loads audio for the first 2 tracks (prefetch strategy — load N+1 while playing N)
4. Mix begins: Track 1 plays from a configurable start point (default: 10s from beginning to skip intros)
5. System calculates transition point for Track 1 → Track 2:
   a. Analyzes waveform to find a phrase boundary near the end of the playable region
   b. Computes BPM ratio between tracks
   c. If BPM difference ≤ 6: applies gradual tempo adjustment on the outgoing track over 8 bars
   d. If BPM difference > 6: uses simple crossfade without beat-matching (too jarring to tempo-shift)
6. At the transition point, system begins the mix:
   a. Outgoing track: tempo-adjusted, volume fading via equal-power curve
   b. Incoming track: starts at a phrase boundary, volume rising
   c. Transition duration: 16-32 beats (depending on BPM and genre)
7. Mix continues through Track 2, pre-loading Track 3
8. Repeat steps 5-7 for each subsequent transition
9. Final track plays to completion (or configured endpoint)
10. System displays: "Mix complete — [setlist name], [total duration], [track count] tracks"
11. Mix timeline shows all transitions with quality scores (from UC-017)

## Extensions (What Can Go Wrong)

- **1a. Setlist has fewer than 3 tracks**:
  1. Still playable — just 1-2 transitions (or 0 for a single track)
  2. System plays available tracks with transitions

- **2a. >50% of tracks lack audio URLs**:
  1. System warns: "Only X of Y tracks have audio. The mix will skip tracks without audio."
  2. User can proceed or cancel

- **2b. No tracks have audio URLs**:
  1. System displays: "No audio available for any tracks in this setlist. Import tracks from SoundCloud for playable audio."
  2. Use case fails

- **3a. Audio pre-loading fails (network error)**:
  1. System retries once
  2. If still failing, skips that track in the mix
  3. If first track fails, tries starting from track 2

- **4a. Track audio is very short (<10 seconds, e.g., Spotify preview clip)**:
  1. System plays from the beginning
  2. Transition starts earlier (at 50% of track length)
  3. Note: short previews may produce choppy mixes — this is a known limitation

- **5a. Waveform analysis cannot find a phrase boundary**:
  1. System uses a fixed transition point (5 seconds before end of playable region)
  2. Transition may be less musically clean

- **5b. BPM data is missing for one or both tracks**:
  1. Fall back to simple crossfade (UC-019 style)
  2. No beat-matching attempted

- **5c. BPM data is inaccurate (essentia detection error)**:
  1. Beat-matching may sound off
  2. User can report "bad transition" to flag the track for re-analysis
  3. System adaptively falls back to crossfade if phase detection indicates mismatch

- **6a. Tempo adjustment causes audible pitch artifacts**:
  1. Expected with large tempo shifts — this is why we limit to ±6 BPM
  2. Use time-stretching algorithm (preserves pitch) instead of simple resampling when possible
  3. If Web Audio API doesn't support quality time-stretching, fall back to crossfade

- **6b. Audio buffer underrun (loading too slow)**:
  1. System pauses momentarily while buffer fills
  2. Displays "Buffering..." indicator
  3. Resumes when ready

- **7a. Pre-loading fails for the next track**:
  1. Current track finishes naturally
  2. System attempts to load the track after next (skip one)
  3. If persistent failures, stops the mix gracefully

- **8a. Browser tab is backgrounded (mobile or desktop)**:
  1. Web Audio API may be throttled in background
  2. Mix may stutter or pause
  3. When tab returns to foreground, mix resumes (may need to re-sync)

- **8b. Device runs out of memory (mobile, many tracks loaded)**:
  1. System uses streaming decode (not loading full audio into memory)
  2. Only 2 tracks in memory at any time (current + next)
  3. Previous tracks are garbage-collected

- **9a. User skips to a specific transition**:
  1. System pre-loads the two tracks around that transition
  2. Mix continues from there

- **9b. User pauses and resumes after a long time**:
  1. Audio URLs may have expired (SoundCloud tokens)
  2. System refreshes URLs before resuming
  3. If refresh fails, skips to next available track

## Variations

- **V1. Crossfade-Only Mode**: User opts for simple crossfades (UC-019 style) for the entire mix — no beat-matching. Simpler, more reliable.
- **V2. Record Mix**: System records the mixed audio output to a downloadable file (WAV/MP3). Major copyright implications — may need to be restricted to user-uploaded files only.
- **V3. Live Adjustments**: User can adjust transition length and crossfade curve in real-time during playback.
- **V4. Visualizer**: Full-screen audio visualizer (frequency spectrum, waveform) during mix playback.

## Agent Execution Notes
- **Verification Command**: `cd frontend && flutter test test/dj_mix_test.dart` (unit tests for mix logic) + manual browser testing
- **Test File**: `frontend/test/dj_mix_test.dart`
- **Depends On**: UC-019 (crossfade preview — this extends it), UC-017 (arrangement with transition scores), UC-015 (BPM data for beat-matching)
- **Blocks**: None (terminal feature)
- **Estimated Complexity**: XL (~5000 tokens implementation budget, likely multi-sprint)
- **Agent Assignment**:
  - Teammate:Frontend-1 — Web Audio API beat-matching engine (tempo adjustment, phase alignment, time-stretching)
  - Teammate:Frontend-2 — Mix playback UI (transport controls, waveform, transition timeline, progress indicator)
  - Teammate:Backend — Audio URL refresh endpoint, waveform pre-computation endpoint (optional)

### Key Implementation Details
- **Beat-matching**: Web Audio API `playbackRate` for simple tempo adjustment. For quality time-stretching, consider `SoundTouch.js` or `Tone.js` libraries.
- **Phase alignment**: Detect beat positions using onset detection (AnalyserNode FFT), align incoming track's first beat with outgoing track's beat grid
- **Phrase detection**: Simple approach — assume 4/4 time, 16-bar phrases. Advanced: FFT-based spectral flux onset detection.
- **Memory management**: Use `AudioBufferSourceNode` for decoded audio, `MediaElementAudioSourceNode` for streaming. Only 2 tracks decoded at once.
- **Performance budget**: Beat-matching computation should stay under 10ms per audio frame (at 44.1kHz). Use `AudioWorkletNode` for custom processing if needed.
- **Fallback chain**: Beat-matched mix → simple crossfade → hard cut → skip track
- **No new migration needed** — uses existing setlist and track data

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified (manual browser testing required)
- [ ] Beat-matched transitions sound musically coherent for ±6 BPM differences
- [ ] Crossfade fallback engages automatically when beat-matching isn't possible
- [ ] Pre-loading strategy prevents audio gaps between tracks
- [ ] Mix plays continuously through an entire setlist (≥5 tracks, manual test)
- [ ] Playback controls (play/pause/skip) work correctly during mix
- [ ] Missing audio tracks are skipped without crashing the mix
- [ ] Memory usage stays reasonable (<200MB for a 15-track mix)
- [ ] CPU usage stays below 50% on a modern laptop
- [ ] Visual feedback shows current track, BPM, progress, and upcoming transition
- [ ] Short audio clips (Spotify 30s previews) are handled gracefully
- [ ] Browser backgrounding doesn't permanently break the mix
