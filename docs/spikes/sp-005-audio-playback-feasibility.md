# SP-005: Audio Playback Feasibility

## Status: COMPLETE

## Hypothesis

Can we provide crossfade preview playback in the browser for the DJ use case?

Specifically:
1. Can we get 30-second audio previews for tracks without requiring user authentication?
2. Can we play those previews in the browser without CORS issues?
3. Can we implement smooth crossfade transitions between tracks using browser APIs?
4. Is the audio quality sufficient for DJ preview use?

## Time-Box

**Target**: 1 session / 1 day
**Actual**: ~3 hours (research + PoC + validation)

## Research

### Candidate Sources

| Source | Preview Quality | Auth Required | CORS | Notes |
|--------|----------------|---------------|------|-------|
| Spotify | 30s, good quality | OAuth required | CORS blocked | Preview URLs often null; requires auth even for previews |
| Deezer | 30s, 128kbps MP3 | **None** | CDN allows; backend proxy needed | Free search API; no auth; reliable preview availability |
| SoundCloud | Variable | OAuth required | 302 redirects (problematic) | Complex auth; redirect chain breaks Web Audio API |
| Direct upload | User-controlled | N/A | N/A | Out of scope for MVP |

### Deezer API Discovery

Deezer provides a public search API with no authentication:
- `GET https://api.deezer.com/search?q={artist}+{title}` → returns tracks with `preview` (30s MP3 URL)
- No API key required
- Preview URLs are CDN-hosted MP3s (`cdns-preview-*.dzcdn.net`)
- Coverage: ~98% of popular tracks have previews

**CORS behavior**: Deezer CDN serves `Access-Control-Allow-Origin: *` from the Deezer domain, but direct browser-to-CDN requests may fail depending on CDN configuration. **Solution**: backend proxy at `/api/audio/deezer-search` fetches and pipes the MP3 response, eliminating all CORS issues.

### Web Audio API for Crossfade

The browser's Web Audio API provides `AudioContext`, `GainNode`, and `AudioBufferSourceNode` which support:
- Precise scheduled playback and gain ramping
- Crossfade via simultaneous fade-out on current track + fade-in on next track
- Equal-power or linear crossfade via `setValueAtTime()` + `linearRampToValueAtTime()`

Flutter web access: via `package:web` (not deprecated `dart:html` or non-existent `dart:web_audio`).

## Proof of Concept

**PoC**: `tarab.studio/audio-poc.html` (deployed and working)

The PoC validated:
1. Deezer search via backend proxy returns reliable MP3 URLs
2. `fetch()` + `ArrayBuffer` + `AudioContext.decodeAudioData()` works for buffered playback
3. Linear crossfade over 3 seconds produces smooth DJ-style transitions
4. Latency from search → decode → playback is acceptable (<2s on typical connections)
5. 30-second previews are sufficient to evaluate track feel and transitions

## Findings

### YES — crossfade preview playback is feasible in the browser

**Technical approach**:
1. Backend proxies Deezer search: `GET /api/audio/deezer-search?q={query}` → returns preview URL + track metadata
2. Frontend fetches MP3 via backend proxy (avoids CORS)
3. `package:web` provides `AudioContext`, `GainNode`, etc. in Flutter web
4. Crossfade: schedule gain ramp on current source (fade out) + gain ramp on next source (fade in)
5. Auto-advance: `AudioBufferSourceNode.onended` callback triggers next track load

**Key implementation constraints**:
- `AudioContext` must be created inside a user gesture (browser security policy)
- `dart:html` is deprecated and breaks non-web platforms — use `package:web` with conditional imports (web/stub pattern)
- Deezer CDN requires backend proxy — do not call CDN directly from browser

### Spotify Preview Limitations (Why Not Spotify)

- Spotify preview URLs are often `null` for tracks (not guaranteed)
- Spotify API requires OAuth even for metadata queries from the browser
- CORS on Spotify CDN is inconsistent
- **Decision**: Use Spotify for import and metadata; Deezer for playback

## Decision

**Use Deezer for preview playback. Use Spotify for catalog import and metadata.**

Architecture:
```
Catalog import: Spotify API → tracks table (title, artist, bpm, key, energy)
Deezer match:   Deezer search API (via backend proxy) → deezer_preview_url column
Playback:       Flutter Web Audio API plays proxied Deezer MP3
```

Backend enrichment adds `deezer_id` and `deezer_preview_url` to existing tracks (migration 007).

## Outcome

This spike led directly to **UC-019 (Deezer Preview Playback)** implementation, which delivered:
- Phase 1: Web Audio API crossfade player in Flutter web (package:web, conditional imports)
- Phase 2: Deezer enrichment pipeline — ~98% of tracks matched and preview URLs populated
- Phase 3: Backend MP3 proxy endpoint, transport controls, PlaybackStatus enum

**UC-019 is COMPLETE** (merged via PR #4, branch `feature/uc-019-deezer-playback`).

## Scoped Out (Post-MVP)

- Waveform visualization (Canvas/essentia)
- Equal-power crossfade (linear is sufficient for MVP)
- Purchase links from preview
- ISRC-based matching (fuzzy title+artist search is sufficient)
- Mobile audio (iOS/Android native audio player)

## Related

- SP-002: Earlier Flutter audio spike — confirmed CORS risk, identified `audioplayers` preference (now superseded by Web Audio API direct)
- SP-003: essentia sidecar — audio analysis path (post-MVP, deferred)
- UC-019: Implementation driven by this spike's findings
