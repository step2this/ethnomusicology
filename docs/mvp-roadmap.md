# MVP Roadmap: Magic Set Generator

## Phase 0: SP-004 — Verify Enrichment Path (COMPLETE)
**Decision**: LLM enrichment is primary. Spotify Audio Features deprecated Nov 2024.
See: `docs/spikes/sp-004-enrichment-path.md`

## Phase 1: ST-005 — Track Enrichment (COMPLETE)
Imported tracks get real BPM, Camelot key, energy, album art via Claude estimation.
See: `docs/steel-threads/st-005-track-enrichment.md`

## Phase 2: ST-006 — Multi-Input Seeding + Enhanced Generation (COMPLETE)
Multiple input methods (prompt, playlist seed, tracklist seed). Energy profiles. Creative mode.
Merged via PR #3, 2026-03-03.

## Phase 3: ST-007 — Conversational Setlist Refinement (COMPLETE)
Natural language chat to iterate on generated setlists. Version history. >50% change guard.
Backend + frontend complete. PR #5 merged.

## UC-019: Audio Preview Playback (COMPLETE — sequential, no crossfade)
30s preview clips via Deezer. Transport bar (prev/next/pause/stop), auto-advance, PlaybackStatus enum.
Crossfade removed — too complex for 30-second clips; sequential playback is better UX.
Backend MP3 proxy for CORS. Merged via PR #4, 2026-03-03.

## Phase 4: Audio Search Quality
Improve Deezer hit rate for electronic music (aliases, remixes, obscure tracks).
- Deezer field-specific search: `artist:"X" track:"Y" strict=on` (replaces freeform)
- ISRC lookup fallback: `GET /track/isrc:{ISRC}` — instant exact match when Spotify ISRC present
- Per-track search status indicators + debug tooltips

## Phase 5: Multi-Source Preview Fallback
Add iTunes Search API as Deezer fallback to improve catalog coverage.
- iTunes Search API: free, no auth, 100M+ catalog, 30s AAC previews
- Preview chain: Deezer ISRC → Deezer field search → iTunes Search → SoundCloud → no-preview
- Apple Music affiliate links (add `at=` param to iTunes URLs for ~7% commission)

## Phase 5.1: SoundCloud Preview Integration
OAuth + SoundCloud preview stream for underground/independent electronic catalog.
- App-level OAuth 2.1 credentials required
- SoundCloud migrating to AAC HLS — use new stream endpoint
- Strongest for artists not on Deezer or iTunes

## Phase 6: Purchase Link Panel (UC-020)
Multi-store link panel per track in setlist UI.
- Beatport deep links (requires API v4 access — apply early)
- Apple Music affiliate links (via iTunes Search API match)
- Bandcamp, Traxsource, Juno Download search links
- Apple affiliate registration at apple.com/itunes/affiliates

## Future: Beatport API Integration
Rich DJ metadata (native BPM, key, genre, label, remixer) + preview audio.
Requires application for API v4 access. Apply immediately — approval may take weeks.

## Dependencies
```
SP-004 (done) → ST-005 (done) → ST-006 (done) → ST-007 (done) → Phase 4 → Phase 5 → Phase 6
```

## MVP Milestone
After Phase 3: Import playlist (or zero tracks) → describe vibe → get harmonically arranged setlist → refine with chat.

## Full Vision
After Phase 6: Hear any track in the setlist via multi-source preview, buy it from the best store.
