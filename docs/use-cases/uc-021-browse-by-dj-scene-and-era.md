# Use Case: UC-021 Browse by DJ Scene and Era

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P1 Important
- **Complexity**: 🟡 Medium

## Actors
- **Primary Actor**: App User (authenticated, DJ)
- **Supporting Actors**:
  - Claude API (Sonnet — scene/era classification)
  - Database (SQLx — track catalog with enriched metadata)
- **Stakeholders & Interests**:
  - DJ User: Wants to explore their catalog by musical scenes (NYC House, Detroit Techno, UK Garage, Berlin Minimal) and eras (80s, early 90s, late 90s, 2000s, 2010s, current) — the way DJs actually think about music, not just by genre
  - Developer: Wants scene/era to be LLM-derived tags stored on tracks, enabling fast filtering without repeated LLM calls

## Conditions
- **Preconditions** (must be true before starting):
  1. User has tracks in their catalog (at least 10 for meaningful browsing)
  2. Tracks have been enriched with genre/sub-genre (UC-018) — scene/era derivation builds on this
  3. Backend has valid Anthropic API key (for initial scene/era classification)

- **Success Postconditions** (true when done right):
  1. Each track in the catalog has `scene_tags` (e.g., ["NYC House", "Paradise Garage"]) and `era_tag` (e.g., "early-90s") populated
  2. User can browse catalog filtered by scene, era, or scene+era combination
  3. Scene/era tags are derived by LLM from track metadata (artist, genre, label, release year) and stored permanently
  4. Browse view shows track count per scene and per era
  5. User can tap a scene/era to see matching tracks, then generate a setlist from that filtered view

- **Failure Postconditions** (true when it fails gracefully):
  1. If LLM classification fails, tracks retain their existing genre/sub-genre and can still be browsed by those
  2. Tracks without enough metadata for classification show in an "Unclassified" category

- **Invariants** (must remain true throughout):
  1. Scene/era tags are stored on tracks, not computed at query time (performance)
  2. The LLM scene vocabulary is constrained to a predefined set (prevents proliferation)
  3. Users cannot create custom scenes (curated vocabulary only)

## Main Success Scenario
1. Background enrichment worker (same worker as UC-018 — see UC-018 "Shared worker" note) classifies tracks with scene and era tags using Claude API
2. User navigates to "Browse" screen in the Flutter app
3. System displays a scene grid: tiles for each scene (NYC House, Chicago House, Detroit Techno, UK Garage, Berlin Minimal, Balearic, etc.) with track count per scene
4. User taps a scene tile (e.g., "NYC House")
5. System filters catalog to tracks tagged with that scene
6. System displays filtered tracks, grouped by era within the scene (e.g., "Early 90s (12 tracks)", "Late 90s (8 tracks)", "2000s (3 tracks)")
7. User can further filter by era
8. User sees track listing with DJ metadata (BPM, key, energy, mood)
9. User can tap "Generate Setlist from This" to pass the filtered view to UC-016 as context

> **Dependency note**: Filtered catalog from browse view passes track IDs to UC-016's setlist generation endpoint as a `track_filter` parameter, constraining the LLM to only select from the filtered subset.

## Extensions (What Can Go Wrong)

- **1a. Track metadata insufficient for scene classification (no genre, label, or release year)**:
  1. LLM assigns "Unclassified" scene and best-guess era from release date
  2. Track appears in "Unclassified" browse section

- **1b. LLM classification call fails**:
  1. Track retains existing genre/sub-genre
  2. Shows in genre-based browse as fallback
  3. Retried on next enrichment cycle

- **2a. User has 0 classified tracks**:
  1. Browse screen shows "Your tracks are being classified. Check back in a few minutes."
  2. If tracks exist but classification hasn't run, shows genre-based fallback browse

- **3a. Scene has 0 tracks (empty tile)**:
  1. Tile is hidden or grayed out
  2. Only scenes with ≥1 track are shown as active tiles

- **5a. Filter returns 0 tracks (scene exists but user has none)**:
  1. System shows "No tracks in [scene]. Import tracks from Beatport or SoundCloud to build this collection."

- **9a. "Generate Setlist" with very few tracks (< 5)**:
  1. System passes filtered tracks to UC-016 but warns: "Only X tracks in this filter. The setlist may include suggestions for tracks to acquire."

## Variations

- **V1. Era-First Browse**: User browses by era first (grid of decades), then filters by scene within an era.
- **V2. Scene Discovery**: System suggests scenes the user might like based on their catalog composition: "You have 30 NYC House tracks. You might enjoy Paradise Garage or Sound Factory sets."
- **V3. Cross-Scene View**: Show tracks that span multiple scenes (e.g., a track tagged both "UK Garage" and "NYC House").

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test scene_browse`
- **Test File**: `backend/tests/scene_browse.rs`
- **Depends On**: UC-018 (enrichment provides genre/sub-genre as input), UC-013/014 (tracks with metadata)
- **Blocks**: None directly (enriches UC-016 setlist workflow)
- **Estimated Complexity**: M (~1500 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend — Scene/era classification worker (LLM batch), browse/filter API endpoints, constrained scene vocabulary
  - Teammate:Frontend — Scene grid UI, era grouping, filtered track listing, "Generate Setlist from This" button

### Key Implementation Details
- **Scene vocabulary** (constrained set):
  ```
  NYC House, Chicago House, Detroit Techno, UK Garage, Berlin Minimal,
  Balearic, Italo Disco, Acid House, Jungle/DnB, Dubstep, Trance,
  Progressive House, Deep House, Tech House, Afro House, Amapiano,
  Reggaeton/Latin, Hip-Hop/R&B, Afrobeats, Arabic/Middle Eastern, South Asian
  ```
- **Era vocabulary**: `pre-80s`, `80s`, `early-90s`, `late-90s`, `early-2000s`, `late-2000s`, `2010s`, `2020s`
- **Classification**: Batched LLM call (20 tracks), stores scene_tags (JSON array) and era_tag (string) on tracks table
- **Migration**: Add `scene_tags TEXT` (JSON array), `era_tag TEXT` to tracks table
- **API endpoints**: `GET /api/browse/scenes` (scene list with counts), `GET /api/browse/scenes/{scene}` (tracks), `GET /api/browse/scenes/{scene}/eras/{era}` (filtered)

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] Scene/era tags are derived from track metadata via LLM
- [ ] Scene vocabulary is constrained to predefined set
- [ ] Browse view shows scene tiles with accurate track counts
- [ ] Filtering by scene and era returns correct tracks
- [ ] "Generate Setlist from This" passes filtered context to UC-016
- [ ] Tracks without classification show in "Unclassified" section
- [ ] Genre-based fallback browse works when classification hasn't run
- [ ] Frontend renders scene grid and era groupings
