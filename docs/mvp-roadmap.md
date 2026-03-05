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

## Phase 4: Audio Search Quality (COMPLETE)
Improve Deezer hit rate for electronic music (aliases, remixes, obscure tracks).
- Deezer field-specific search: `artist:"X" track:"Y" strict=on` (replaces freeform)
- ISRC lookup fallback: `GET /track/isrc:{ISRC}` — instant exact match when Spotify ISRC present
- Per-track search status indicators + debug tooltips

## Phase 5: Multi-Source Preview Fallback (COMPLETE)
iTunes Search API as Deezer fallback to improve catalog coverage.
- iTunes Search API: free, no auth, 100M+ catalog, 30s AAC previews — ST-008
- Preview chain: Deezer ISRC → Deezer field search → iTunes Search → SoundCloud → no-preview
- Apple Music affiliate links (add `at=` param to iTunes URLs for ~7% commission)

## Phase 5.1: SoundCloud Preview Integration (COMPLETE)
OAuth + SoundCloud preview stream for underground/independent electronic catalog — ST-009.
- App-level OAuth 2.1 Client Credentials flow
- `preview_mp3_128_url` (128kbps MP3, not AAC HLS)
- CDN redirect (302) resolved server-side; credentials in /etc/ethnomusicology/env
- Strongest for artists not on Deezer or iTunes

## Phase 6: LLM Verification & Quality (COMPLETE — PR #10 pending merge)
Reduce hallucinated tracks via LLM self-verification and surface confidence in the UI.
- SP-007 spike: skill doc (`music_skill.md`) injected as stable content block[0] for prompt caching
- `verify_setlist()` wired into generation flow via opt-in `verify: true` flag
- `confidence` field (high/medium/low) + `verification_notes` persisted via migration 009
- Frontend: `ConfidenceBadge` widget on track tiles (color-coded chip + tooltip)
- Calibration: high ≈ 90% real tracks, medium ≈ 25%, low = creative suggestion

## Phase 7: Purchase Link Panel (UC-020)
Multi-store link panel per track in setlist UI.
- Beatport deep links (requires API v4 access — apply early)
- Apple Music affiliate links (via iTunes Search API match)
- Bandcamp, Traxsource, Juno Download search links
- Apple affiliate registration at apple.com/itunes/affiliates

## Phase 8: Saved Setlists & Crates (P0 — Core DJ Workflow)
Persistent setlist management and the "crate" concept — a DJ's working library.
- **Save setlists**: Named setlists persist across sessions with full metadata
- **Setlist library**: Browse, search, duplicate, delete saved setlists
- **Crates**: Combine 1+ setlists into a "crate" (like a DJ's milk crate of records)
  - A crate = the 100-200 tracks always in your sets right now
  - Merge tracks from multiple setlists, deduplicate, manage
  - Crate as generation source: "build me a set from my crate"
- **UI**: Setlist list view, crate management screen, drag-to-crate interaction

## Future: Beatport API Integration
Rich DJ metadata (native BPM, key, genre, label, remixer) + preview audio.
Requires application for API v4 access. Apply immediately — approval may take weeks.

## Dependencies
```
SP-004 (done) → ST-005 (done) → ST-006 (done) → ST-007 (done) → Phase 4 (done) → Phase 5 (done) → Phase 5.1 (done) → Phase 6 (done) → Phase 7
```

## MVP Milestone
After Phase 3: Import playlist (or zero tracks) → describe vibe → get harmonically arranged setlist → refine with chat.

## Full Vision
After Phase 7: Hear any track in the setlist via multi-source preview, buy it from the best store.
