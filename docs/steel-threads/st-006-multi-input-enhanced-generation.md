# Steel Thread: ST-006 Multi-Input Seeding + Enhanced Generation

## Classification
- **Goal Level**: 🧵 Thread — thin end-to-end proof of multi-input seeding, energy profiles, and enhanced generation
- **Scope**: System (black box)
- **Priority**: P0 Critical (core product experience — the MVP milestone)
- **Complexity**: 🔴 High

## Cross-Cutting References

- **UC-016**: Steps 3, 5-8 — proves energy profile selection influences LLM prompt construction and output quality; proves catalog filtering by source playlist; proves seed tracklist passed as LLM context; proves creative mode alters generation behavior
  - Postcondition 5: coherent energy arc (via energy profiles)
  - Postcondition 7: BPM transitions >±6 flagged
  - Postcondition 9: <30% catalog tracks triggers warning
  - Invariant 5: prompt caching via `cache_control: ephemeral`
- **UC-017**: Step 5a — proves energy arc scoring is parameterized by profile, not hardcoded
  - Postcondition 1: weighted transition score with profile-aware energy component
- **UC-015**: Steps 1-4 (indirectly) — proves enriched BPM/key/energy data flows through to enhanced system prompt and arrangement
- **UC-001**: Step 14 (indirectly) — proves auto-enrich trigger fires after import completes

This thread proves the **core product experience**: multiple ways to seed a setlist + DJ-intelligent generation with energy profiles. After ST-006, a DJ can import a Spotify playlist, describe their vibe, and get a harmonically arranged setlist with real BPM/key data and intelligent transition notes.

## Actors

- **Primary Actor**: App User (DJ)
- **Supporting Actors**:
  - Claude API (Sonnet — enhanced setlist generation with energy profiles)
  - Database (SQLite — catalog queries, setlist persistence, energy profile storage)
  - Spotify API (playlist import for playlist-seed flow)
- **Stakeholders & Interests**:
  - DJ User: Wants multiple ways to start a set (describe vibe, seed from playlist, paste tracklist). Wants energy arcs that match the gig (warm-up vs peak-time vs journey).
  - Developer: Wants clean extension of existing generate/arrange APIs. Parameterized energy profiles, not hardcoded scoring.
  - Business: This is the MVP milestone — the "magic set" experience that differentiates the product.

## Conditions

### Preconditions
1. Tracks exist in the database with BPM/key/energy populated (ST-005 enrichment pipeline operational)
2. Backend has a valid `ANTHROPIC_API_KEY` environment variable
3. Migration 005 already applied (`energy_profile` column on `setlists` table, `user_usage` table)
4. ST-003 generate/arrange pipeline operational (POST /api/setlists/generate, POST /api/setlists/{id}/arrange)
5. ST-005 POST /api/tracks/enrich endpoint operational
6. Migration 006 applied (import_tracks junction table — see Migration section below)

### Success Postconditions

**Enhanced Generation API:**
1. `POST /api/setlists/generate` accepts optional `energy_profile` parameter (one of: "warm-up", "peak-time", "journey", "steady")
2. `POST /api/setlists/generate` accepts optional `source_playlist_id` (DB ID) to filter catalog to that playlist's tracks
3. `POST /api/setlists/generate` accepts optional `seed_tracklist` (free text) passed as LLM context
4. `POST /api/setlists/generate` accepts optional `creative_mode` boolean for unexpected but compatible combinations
5. `POST /api/setlists/generate` accepts optional `bpm_range` object (`{min, max}`) to constrain BPM selection
6. `energy_profile` is stored on the `setlists` table after generation
7. Enhanced system prompt includes DJ expertise, transition techniques, energy arc guidelines, and real BPM/key/energy from enriched catalog

**Parameterized Arrangement:**
8. `POST /api/setlists/{id}/arrange` accepts optional `energy_profile` parameter
9. `energy_arc_score()` is parameterized by profile (warm-up: 3→7, peak-time: 7→9→7, journey: 3→9→4, steady: 6→6)
10. If no explicit profile passed to arrange, reads from setlist's stored `energy_profile`

**Setlist Quality Validation:**
11. Post-generation: adjacent BPM jumps >±6 are flagged with `bpm_warning: true` on the transition
12. Post-generation: if <30% of tracks are from catalog, response includes `catalog_warning` message
13. When `seed_tracklist` is provided, LLM response includes `seed_match_count` in notes

**Auto-Enrich on Import:**
14. After `import_playlist()` completes, enrichment fires automatically via `tokio::spawn`
15. Import endpoint returns immediately; enrichment runs in background

**Prompt Caching:**
16. System prompt + catalog context use `cache_control: { type: "ephemeral" }` content blocks
17. Claude API response includes `cache_read_input_tokens > 0` on subsequent calls

**Frontend:**
18. Unified "Create Set" screen with tabs: "Describe a Vibe" / "From Spotify Playlist" / "From Tracklist"
19. Energy profile selector with visual mini-curve
20. Creative mode toggle
21. Set length slider (5-30 tracks)
22. Spotify playlist tab: URL input → import progress → "Generate from this playlist" button
23. BPM warning badges on transitions with >±6 BPM jump
24. Catalog percentage indicator on generated setlists

### Failure Postconditions
1. If energy_profile is invalid, returns 400 with `INVALID_ENERGY_PROFILE` error code
2. If source_playlist_id doesn't exist, returns 404 with `PLAYLIST_NOT_FOUND` error code
3. If auto-enrich fails after import, import result is unaffected (enrichment failure is independent)
4. If prompt caching is unavailable, generation still works (just costs more)

### Invariants
1. Existing generate/arrange endpoints remain backward-compatible (all new params are optional)
2. All LLM calls happen on the backend; API key never exposed to frontend
3. Energy profile is a hint, not a hard constraint — LLM and arrangement use it as guidance
4. Auto-enrich does not block import response

## API Contract

| Method | Path | Description | Changes | Status |
|--------|------|-------------|---------|--------|
| POST | /api/setlists/generate | Generate setlist from prompt | Add: energy_profile, source_playlist_id, seed_tracklist, creative_mode, bpm_range to request. Add: catalog_warning, bpm_warnings to response | Draft |
| POST | /api/setlists/{id}/arrange | Arrange by harmonic compatibility | Add: energy_profile to request. Energy arc scoring parameterized. | Draft |
| GET | /api/setlists/{id} | Get setlist details | Response includes energy_profile, bpm_warnings, catalog_percentage | Draft |
| POST | /api/import/spotify | Import Spotify playlist | Auto-triggers enrichment after import (no API change, behavior change) | Draft |
| POST | /api/tracks/enrich | Trigger enrichment | No change (used by auto-enrich internally) | Implemented |

### Extended Generate Request Schema
```json
{
  "prompt": "Deep house warm-up set, starting mellow",
  "track_count": 15,
  "energy_profile": "warm-up",
  "source_playlist_id": "uuid-of-imported-playlist",
  "seed_tracklist": "1. Kerri Chandler - Rain\n2. Larry Heard - Can You Feel It",
  "creative_mode": true,
  "bpm_range": { "min": 118, "max": 128 }
}
```

### Extended Generate Response Schema
```json
{
  "id": "uuid",
  "prompt": "...",
  "model": "claude-sonnet-4-20250514",
  "energy_profile": "warm-up",
  "tracks": [...],
  "notes": "...",
  "harmonic_flow_score": null,
  "score_breakdown": null,
  "catalog_percentage": 65.0,
  "catalog_warning": null,
  "bpm_warnings": [
    { "from_position": 5, "to_position": 6, "bpm_delta": 8.5 }
  ],
  "created_at": "..."
}
```

## Main Success Scenario

### Flow A: Enhanced Prompt Generation (core flow)
1. **[Frontend]** User navigates to "Create Set" screen, selects "Describe a Vibe" tab
2. **[Frontend]** User types prompt, selects energy profile ("journey"), enables creative mode, sets track count to 15
3. **[Frontend → API]** App sends POST /api/setlists/generate with `{prompt, track_count: 15, energy_profile: "journey", creative_mode: true}`
4. **[API]** Server validates request: prompt non-empty, energy_profile is valid enum, track_count in range
5. **[API → DB]** Server loads full catalog with enriched metadata (BPM, key, energy)
6. **[API]** Server builds enhanced system prompt: DJ expert persona + energy profile instructions ("journey": start low energy, build to peak, wind down) + creative mode instructions + Camelot rules + catalog context with real metadata
7. **[API]** Server annotates system prompt + catalog blocks with `cache_control: { type: "ephemeral" }`
8. **[API → LLM]** Server calls Claude Sonnet with structured content blocks
9. **[API]** Server parses LLM response, validates track IDs against catalog, flags hallucinations
10. **[API]** Server runs quality validation: counts catalog matches (catalog_percentage), flags BPM jumps >±6
11. **[API → DB]** Server persists setlist with energy_profile, creates setlist_tracks
12. **[API → Frontend]** Server returns 201 with setlist, catalog_percentage, bpm_warnings
13. **[Frontend]** App renders setlist with energy profile indicator, BPM warning badges, catalog percentage
14. **[Frontend]** User taps "Arrange"
15. **[Frontend → API]** App sends POST /api/setlists/{id}/arrange (energy_profile read from stored setlist)
16. **[API]** Server runs arrangement with parameterized energy arc (journey profile targets)
17. **[API → DB]** Server persists arranged order + scores
18. **[API → Frontend]** Server returns arranged setlist with transition scores
19. **[Frontend]** App renders arranged setlist with transition badges, flow score

### Flow B: Spotify Playlist Seed
20. **[Frontend]** User selects "From Spotify Playlist" tab, pastes Spotify URL
21. **[Frontend → API]** App sends POST /api/import/spotify with playlist URL
22. **[API → Spotify]** Server fetches playlist tracks
23. **[API → DB]** Server persists imported tracks
24. **[API → LLM]** Server auto-triggers enrichment via tokio::spawn (background)
25. **[API → Frontend]** Server returns import result immediately
26. **[Frontend]** App shows import success, pre-fills generate form with source_playlist_id
27. **[Frontend → API]** User taps "Generate from this playlist" → POST /api/setlists/generate with `{prompt, source_playlist_id}`
28. **[API → DB]** Server loads catalog filtered to source playlist's tracks
29. Steps 6-19 continue with filtered catalog

### Flow C: Text Tracklist Seed
30. **[Frontend]** User selects "From Tracklist" tab, pastes a list of tracks
31. **[Frontend → API]** App sends POST /api/setlists/generate with `{prompt, seed_tracklist: "1. Artist - Title\n2. ..."}`
32. **[API]** Server includes seed_tracklist in user prompt context for Claude
33. Steps 6-19 continue with seed context

## Extensions

- **3a. Invalid energy_profile value**:
  1. Server returns 400 `INVALID_ENERGY_PROFILE`: "Energy profile must be one of: warm-up, peak-time, journey, steady"
  2. Use case fails

- **3b. source_playlist_id provided but doesn't exist in DB**:
  1. Server returns 404 `PLAYLIST_NOT_FOUND`: "No imported playlist found with that ID"
  2. Use case fails

- **3c. source_playlist_id playlist has 0 enriched tracks**:
  1. Server falls back to full catalog with warning in notes
  2. Continues to step 5

- **5a. Filtered catalog (by source_playlist_id) has 0 tracks**:
  1. Server returns 400 `EMPTY_CATALOG`: "No tracks found in the specified playlist"
  2. Use case fails

- **8a. Claude API rate limited or unavailable**:
  1. Existing retry logic from ST-003 handles this (3 retries for 429, 2 for 500s)
  2. If all retries fail, returns 503 `LLM_UNAVAILABLE`

- **10a. All tracks are suggestions (0% catalog)**:
  1. catalog_warning set to "All tracks in this setlist are suggestions — none match your catalog. Consider importing more tracks."
  2. Continues normally (warning is informational)

- **16a. energy_profile not stored on setlist (pre-ST-006 setlists)**:
  1. Arrangement falls back to default energy arc (gradual build)
  2. Continues normally

- **24a. Auto-enrich fails (Claude API error during background enrichment)**:
  1. Enrichment errors are logged and stored per-track (enrichment_error column)
  2. Import result is unaffected
  3. User can manually trigger POST /api/tracks/enrich later

- **24b. Import completes but enrichment hasn't finished when user tries to generate**:
  1. Generation works with whatever metadata is available (partially enriched)
  2. Notes include "Some tracks are still being enriched. Generate again after enrichment completes for better results."

## Integration Assertions

1. **Frontend → API → LLM**: Energy profile parameter flows from UI selector → request body → system prompt → LLM behavior (verified by checking that "warm-up" profile produces lower starting energy than "peak-time")
2. **API → DB → API**: energy_profile stored on setlists table after generate, read back during arrange (verified by arrange returning profile-aware scores without explicit profile param)
3. **API → LLM → API**: Prompt caching works — second generation with same catalog has `cache_read_input_tokens > 0` in Claude API response
4. **Frontend → API → DB**: source_playlist_id filters catalog correctly — generation only uses tracks from that playlist (verified by all catalog tracks having matching import source)
5. **API → API (background)**: Auto-enrich fires after import — tracks that were needs_enrichment=1 become enriched_at != NULL within reasonable time
6. **API → Frontend**: BPM warnings and catalog_percentage flow through response to UI rendering
7. **API (arrangement)**: Parameterized energy arc produces different arrangements for different profiles on the same setlist
8. **Backward compatibility**: Existing generate/arrange calls without new params continue to work identically

## Does NOT Prove

- **Conversational refinement** — ST-007 handles multi-turn chat and version history
- **Held-Karp optimal arrangement** — Post-MVP, greedy+2-opt is sufficient
- **Large catalog pre-filtering** — Current implementation sends full catalog; chunking for >2000 tracks is deferred
- **Beatport/SoundCloud import** — Only Spotify playlist seed is implemented; other sources are post-MVP
- **Audio-accurate BPM/key** — essentia sidecar is post-MVP; LLM estimation from ST-005 is sufficient
- **Real-time enrichment progress** — Frontend doesn't poll enrichment status; user refreshes manually
- **Generation usage limits** — daily cap for generation_count exists in user_usage table but is not enforced in ST-006 (enrichment cap is enforced)
- **Mix URL seed** — stretch goal, deferred
- **Catalog-only mode toggle** — V4 variation from UC-016, deferred

## Agent Execution Notes

- **Verification Command**: `cd backend && cargo test && cd ../frontend && flutter analyze && flutter test`
- **Test File**: Backend tests inline per module; frontend tests in `frontend/test/`
- **Depends On**: ST-003 (generate/arrange pipeline), ST-005 (enrichment + Camelot), SP-004 (enrichment path decision)
- **Blocks**: ST-007 (conversational refinement builds on enhanced generation)
- **Estimated Complexity**: XL — ~8-12 files modified, ~1500-2500 lines new/changed code
- **Agent Assignment**:
  - Lead: Coordinates, reviews, wires modules, runs quality gates
  - Teammate:prompt-engineer — Enhanced system prompt, energy profile prompt templates, creative mode instructions (`api/claude.rs`)
  - Teammate:generation-api — Extended request validation, catalog filtering, quality validation (BPM warnings, catalog %), seed_tracklist handling (`services/setlist.rs`, `routes/setlist.rs`)
  - Teammate:energy-profiles — Parameterized energy_arc_score, profile definitions, arrange endpoint extension (`services/arrangement.rs`)
  - Teammate:auto-enrich — tokio::spawn trigger after import, prompt caching content blocks (`services/import.rs`, `api/claude.rs`)
  - Teammate:frontend — Unified Create Set screen, energy profile selector, creative mode toggle, BPM warnings, playlist tab

## Implementation Summary

| Component | File | Description |
|-----------|------|-------------|
| Enhanced system prompt | `api/claude.rs` | DJ expertise, energy profile instructions, creative mode, prompt caching content blocks |
| Extended generation | `services/setlist.rs` | New request params, catalog filtering, seed_tracklist, quality validation |
| Generation route | `routes/setlist.rs` | Extended request/response schemas |
| Energy profiles | `services/arrangement.rs` | Parameterized energy_arc_score(), profile definitions |
| Auto-enrich trigger | `services/import.rs` | tokio::spawn enrichment after import_playlist() |
| Prompt caching | `api/claude.rs` | cache_control: ephemeral on system + catalog content blocks |
| Frontend: Create Set | `screens/setlist_generation_screen.dart` | Tabs, energy selector, creative mode, playlist seed |
| Frontend: API client | `services/api_client.dart` | Extended generate request params |
| Frontend: provider | `providers/setlist_provider.dart` | New parameters, warnings state |
| Frontend: models | `models/setlist.dart` | energy_profile, catalog_percentage, bpm_warnings |

## Acceptance Criteria

- [ ] All success postconditions verified by automated test
- [ ] All integration assertions pass end-to-end
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] API contract matches implementation (request/response shapes)
- [ ] Existing generate/arrange calls without new params produce identical results (backward compat)
- [ ] Energy profile selection influences both LLM generation and arrangement scoring
- [ ] BPM warnings correctly flag transitions >±6 BPM
- [ ] Catalog percentage accurately reflects catalog vs suggestion ratio
- [ ] Source playlist filtering returns only tracks from specified playlist
- [ ] Auto-enrich fires after import without blocking import response
- [ ] Prompt caching reduces token usage on subsequent calls
- [ ] Frontend renders all three input tabs with appropriate controls
- [ ] Code passes quality gates (cargo fmt, clippy, cargo test, flutter analyze, flutter test)
- [ ] Critic agent approves implementation

## Migration 006

```sql
-- Migration 006: Import-track linkage for playlist seed filtering
-- Supports ST-006 source_playlist_id catalog filtering

CREATE TABLE IF NOT EXISTS import_tracks (
    import_id TEXT NOT NULL REFERENCES spotify_imports(id),
    track_id TEXT NOT NULL REFERENCES tracks(id),
    PRIMARY KEY (import_id, track_id)
);

CREATE INDEX IF NOT EXISTS idx_import_tracks_import_id ON import_tracks(import_id);
CREATE INDEX IF NOT EXISTS idx_import_tracks_track_id ON import_tracks(track_id);
```

## Critic Findings & Resolutions

Devil's advocate review found 3 CRITICAL, 5 HIGH, 6 MEDIUM issues. All Critical and High are resolved below.

| # | Severity | Finding | Resolution |
|---|----------|---------|------------|
| 1 | CRITICAL | No track-to-import join table — `source_playlist_id` unimplementable | Migration 006 adds `import_tracks` junction table. `import_playlist()` updated to record linkage. |
| 2 | CRITICAL | `ClaudeClientTrait::generate_setlist()` incompatible with prompt caching (needs structured content blocks) | Add new method `generate_with_blocks()` that accepts `Vec<ContentBlock>` for system + user. Keep existing method for backward compat. |
| 3 | CRITICAL | `SetlistRow` model missing `energy_profile` field | Add `energy_profile: Option<String>` to `SetlistRow`. Update `insert_setlist()` and `get_setlist()` queries. Use explicit column list (not `SELECT *`) to avoid SQLx issues. |
| 4 | HIGH | `generate_setlist()` has 8+ params — needs request struct | Create `GenerateSetlistRequest` struct at service layer. All new params are `Option<T>`. |
| 5 | HIGH | `arrange_setlist()` takes no energy_profile param | Add `energy_profile: Option<&str>` param. Read from setlist DB row if not passed explicitly. |
| 6 | HIGH | `energy_arc_score()` lives in `camelot.rs` not `arrangement.rs` — not parameterizable | Add `EnergyProfile` enum to `camelot.rs`. New `energy_arc_score_with_profile()` function. `arrange_tracks()` accepts profile. |
| 7 | HIGH | UC-016 daily generation limits not enforced | Add generation_count check + increment in `generate_setlist()`. Default 20/day. |
| 8 | HIGH | `SetlistResponse` missing 4 new API fields | Add `energy_profile`, `catalog_percentage`, `catalog_warning`, `bpm_warnings` to response. Define `BpmWarning` struct. |
| 9 | MEDIUM | Auto-enrich needs `ClaudeClientTrait` in `ImportState` | Add `claude: Arc<dyn ClaudeClientTrait>` to `ImportState`. Update `main.rs` wiring. |
| 10 | MEDIUM | BPM warnings: compute on-the-fly | Compute during generation and GET, not persisted. Avoids migration. |
| 11 | MEDIUM | Backward compat testing gap | All new request fields are `Option<T>` with `#[serde(default)]`. Existing tests must pass unmodified. |
| 14 | MEDIUM | Frontend scope large | Frontend scope reduced: tabs + energy selector + BPM warnings. Visual polish deferred. |
| 15 | LOW | `seed_match_count` fragile | Compute on backend by matching seed tracklist against response tracks, not relying on LLM. |
| 18 | LOW | `bpm_range` validation missing | Add validation: min >= 60, max <= 200, min <= max. Return 400 `INVALID_BPM_RANGE`. |

## Status: IN PROGRESS
- **Branch**: `feature/st-006-enhanced-generation`
- **Depends on**: ST-003 ✅, ST-005 ✅
