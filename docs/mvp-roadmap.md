# MVP Roadmap: Magic Set Generator

## Phase 0: SP-004 — Verify Enrichment Path (COMPLETE)
**Decision**: LLM enrichment is primary. Spotify Audio Features deprecated Nov 2024.
See: `docs/spikes/sp-004-enrichment-path.md`

## Phase 1: ST-005 — Track Enrichment (COMPLETE)
Imported tracks get real BPM, Camelot key, energy, album art via Claude estimation.
See: `docs/steel-threads/st-005-track-enrichment.md`

## Phase 2: ST-006 — Multi-Input Seeding + Enhanced Generation
Multiple input methods (prompt, playlist seed, tracklist seed). Energy profiles. Creative mode.

## Phase 3: ST-007 — Conversational Setlist Refinement
Natural language chat to iterate on generated setlists. Version history.

## Dependencies
```
SP-004 (done) → ST-005 (done) → ST-006 → ST-007
```

## MVP Milestone
After Phase 2: Import playlist → describe vibe → get harmonically arranged setlist with real BPM/key.

## Full Vision
After Phase 3: Iterate with natural language and watch the set evolve.
