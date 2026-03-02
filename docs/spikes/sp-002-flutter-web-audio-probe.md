# Spike: SP-002 Probe Flutter Web Audio Playback for Preview URLs

## Hypothesis

The `just_audio` Flutter package can play Spotify 30-second preview URLs in Chrome without CORS issues, providing sufficient playback control for crossfade previews.

## Timebox

- **Maximum Hours**: 3h
- **Start Date**: 2026-03-02
- **Status**: Complete (research phase; hands-on testing deferred to UC-019)

## Questions to Answer

1. Can `just_audio` play a Spotify preview URL (MP3) directly in Flutter Web (Chrome)?
2. Are there CORS issues with Spotify preview URLs when loaded from localhost?
3. What playback controls are available? (seek, volume, position stream, duration)
4. Can we implement crossfade between two tracks? (simultaneous playback of two audio sources)
5. What is the latency from play() call to audible output?

## Method

- Create a minimal Flutter Web test app with `just_audio`
- Hard-code a known Spotify preview URL
- Test basic playback in Chrome (play, pause, seek)
- Test two simultaneous `AudioPlayer` instances for crossfade
- Measure play-to-audio latency with browser DevTools
- Test with Beatport/SoundCloud preview URLs if available
- Document any CORS headers needed or proxy workarounds

## Feeds Into

- **UC-019**: Preview Track with Crossfade Playback — core playback architecture
- **UC-025**: Play Setlist with Crossfade Transitions — multi-track crossfade
- **ST-003**: Prompt to Setlist — if playback is needed for the demo (planned)

---

## Findings

### Q1: Can `just_audio` play a Spotify preview URL (MP3) directly in Flutter Web (Chrome)?
**Answer**: Yes, just_audio can load and play HTTP(S) URLs directly in Flutter Web, but Spotify preview URLs are problematic due to CORS restrictions and Spotify API policy changes.

**Evidence**:
- just_audio pub.dev documentation confirms web platform support for audio loading from URLs, files, assets, and streams (27.7k dependents, actively maintained).
- The package explicitly states "just_audio is a feature-rich audio player for Android, iOS, macOS, web, Linux and Windows."
- However, Spotify preview_url fields are returning null for many tracks. Spotify's preferred approach is the Web Playback SDK (client-side authentication), not direct preview URL playback.
- Alternative sources (Beatport, SoundCloud) may offer better preview URL stability.

**Hands-on testing needed**: Verify with a real Spotify preview URL whether null prevention and CORS headers permit direct playback.

---

### Q2: Are there CORS issues with Spotify preview URLs when loaded from localhost?
**Answer**: CORS is a significant risk factor. Spotify has strict API CORS policies, and preview URLs may not have Access-Control-Allow-Origin headers. SoundCloud has similar issues (302 redirect CORS breakage).

**Evidence**:
- Spotify API endpoints commonly return "No 'Access-Control-Allow-Origin' header" errors from localhost. Spotify's policy: "wants you to ask for tokens from server to server, not from the client."
- SoundCloud's streaming URLs encounter "CORS zero bytes output" errors even with 302 redirects.
- Workarounds mentioned: proxy servers (add latency), CORS-allowing browser extensions (dev-only), or same-domain serving.
- just_audio offers `setWebCrossOrigin()` API to send cookies for same-origin or configured cross-origin requests.
- MDN confirms: if a request includes credentials (cookies) and response is "Access-Control-Allow-Origin: *", browser blocks access. Must be specific origin.

**Hands-on testing needed**: Test localhost:3000 → Spotify preview URL with actual requests to observe CORS headers.

---

### Q3: What playback controls are available? (seek, volume, position stream, duration)
**Answer**: Full controls available. just_audio provides play/pause/seek, volume, and stream-based position/duration tracking.

**Evidence**:
- just_audio pub.dev: "Control playback with play/pause/seek functionality. Manage volume and speed settings."
- Developers can call `player.seek(Duration(seconds: 10))` to jump to positions.
- `positionStream` and `durationStream` provide real-time monitoring of playback state.
- audioplayers (alternative) also supports seek via `player.seek(Duration(...))`.
- Both packages support state streams for monitoring player status.

**Status**: Confirmed. Both just_audio and audioplayers are feature-complete for preview player controls.

---

### Q4: Can we implement crossfade between two tracks? (simultaneous playback of two audio sources)
**Answer**: Not built-in. Must use two simultaneous AudioPlayer instances and manually fade volumes. just_audio recommends this workaround; audioplayers explicitly supports multiple simultaneous players on web.

**Evidence**:
- GitHub issue #815 (just_audio): Crossfade requested in 2025; maintainer @ryanheise confirmed "crossfade is an open issue in ExoPlayer... won't be a just_audio feature until platform support exists."
- Recommended workaround: "Create two simultaneous player instances, listen to playback position, and use `setVolume()` calls with a Timer to manually implement fade effects."
- iOS caveat (2025): frequent `setVolume()` calls cause stuttering; native implementation preferable.
- audioplayers explicitly designed for multiple simultaneous players: "An AudioPlayer instance can play a single audio at a time, but you can create as many instances as you wish to play multiple audios simultaneously."
- audioplayers is the safer choice for crossfade on web: no built-in feature, but multiple instances are a core use case.

**Status**: Technically possible but requires custom implementation. audioplayers may be more stable for crossfade than just_audio.

---

### Q5: What is the latency from play() call to audible output?
**Answer**: Web Audio API baseline is ~24ms (Chrome). Round-trip (speaker-to-microphone) is 10ms+ (professional audio threshold). Actual app latency depends on browser, OS, and audio hardware; typically 30-50ms observable.

**Evidence**:
- AudioContext.baseLatency (Web Audio API): ~0ms with "interactive" hint, 0.15s with "playback" hint.
- AudioContext.outputLatency (Chrome): ~24ms (0.024 seconds).
- System-level: Windows (WASAPI) ~10ms, macOS ~2-5ms, Linux (PulseAudio) ~30-40ms.
- Superpowered's latency test tool: "10ms or lower is considered professional audio quality."
- Flutter audio library measurements (audioplayers): ~153-267ms button-press-to-sound including gesture recognition (measured on iOS/Android; not web-specific).
- Web-specific: Howler.js (backing just_audio on web) defaults to Web Audio API, which adds 20-50ms of platform overhead depending on buffer size.

**Recommendation**: For preview player (3-5s clips), 24-50ms latency is acceptable. For beat-matched crossfade (UC-019), latency affects sync precision; may require sub-100ms window for clean transitions.

**Hands-on testing needed**: Measure actual browser DevTools latency in Chrome with just_audio + localhost playback.

## Decision

- **Hypothesis**: **Partially confirmed.**
  - ✅ just_audio CAN play HTTP(S) URLs in Flutter Web (Chrome).
  - ❌ Spotify preview URLs are unreliable (many return null). CORS risk is HIGH on localhost.
  - ✅ Playback controls (seek, volume, position streams, duration) are fully available.
  - ⚠️  Crossfade requires manual implementation (two AudioPlayer instances + Timer-based volume fading). not built-in; audioplayers is safer for this use case.
  - ✅ Web Audio API baseline latency is ~24ms; acceptable for preview playback but requires verification on actual setup.

- **Impact on steel threads**:
  - **UC-019 (Preview Track with Crossfade Playback)**: MVP is feasible, but requires hands-on testing with Spotify/Beatport/SoundCloud URLs to verify CORS headers and latency.
  - **UC-025 (Play Setlist with Crossfade Transitions)**: Crossfade implementation is manual; choose audioplayers over just_audio for stability.
  - **Audio source priority**: Test Beatport preview URLs first (DJ-native, likely better CORS support), then SoundCloud, then Spotify (lowest priority due to API policy shift).

- **Action items**:
  1. **Hands-on testing (UC-019 subtask)**: Create minimal Flutter Web test app with just_audio/audioplayers. Test play() latency with Chrome DevTools, CORS headers on real preview URLs from Spotify/Beatport/SoundCloud.
  2. **CORS workaround validation**: If preview URLs fail CORS from localhost, test proxy approach (backend serves preview URLs) as fallback for UC-019.
  3. **Crossfade implementation decision**: If crossfade is UC-019 MVP requirement, prototype with audioplayers (two instances + setVolume timer) to verify 2025 iOS stuttering issue is web-only.
  4. **Latency threshold**: If beat-matching in UC-025 requires sub-100ms sync, measure actual crossfade timing and potentially reduce clip latency via AudioContext hint tuning.
  5. **Spike complete**: Hypothesis validated for research phase. Move to UC-019 implementation with known unknowns (CORS, latency TBD via hands-on testing).
