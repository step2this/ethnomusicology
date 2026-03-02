# Steel Thread: ST-003 Generate Setlist from Natural Language Prompt and Arrange by Harmonic Compatibility

## Classification
- **Goal Level**: 🧵 Thread — thin end-to-end proof of architectural connection
- **Scope**: System (black box)
- **Priority**: P0 Critical
- **Complexity**: 🔴 High

## Cross-Cutting References

- **UC-016**: Steps 2, 4, 5, 6, 7, 8, 9, 10, 11 — proves prompt input → catalog loading → Claude API call → JSON parsing → validation → DB persistence → frontend rendering
- **UC-017**: Steps 2, 3, 4, 5, 6, 7, 8, 9 — proves arrangement button → load tracks → scoring matrix → greedy+2-opt algorithm → persist new order → render arranged setlist with scores

This thread proves the **core differentiator feature**: LLM-as-crate-digger. It slices through the two most critical use cases end-to-end, proving that the Claude API integration, structured JSON output, DB persistence, Camelot arrangement algorithm, and frontend rendering all connect.

## Actors

- **Primary Actor**: App User (authenticated, DJ)
- **Supporting Actors**:
  - Claude API (Sonnet — setlist generation)
  - Database (SQLite via SQLx — catalog reads, setlist writes)
- **Stakeholders & Interests**:
  - DJ User: Wants to type a vibe description and receive a playable, harmonically arranged setlist
  - Developer: Wants clean separation between LLM prompt engineering, arrangement algorithm, and frontend rendering
  - Business: This is the product's core differentiator — must work end-to-end before building anything else

## Conditions

### Preconditions
1. Backend API is running and serves `GET /api/tracks` (proven by ST-001)
2. At least 5 tracks exist in the database with BPM and Camelot key data
3. Backend has a valid `ANTHROPIC_API_KEY` environment variable
4. Frontend can reach the backend API

### Success Postconditions
1. User can type a natural language prompt in the frontend and submit it
2. Backend calls Claude API with the prompt and user's catalog context, receives structured JSON
3. Backend parses and validates the LLM response (required fields, track_id existence)
4. A `setlists` row is created with user_id, prompt text, model, and timestamp
5. `setlist_tracks` rows are created with position ordering, track metadata, and source markers
6. Frontend displays the generated setlist with track names, BPM, key, and source indicators
7. User can tap "Arrange" to invoke the harmonic arrangement endpoint
8. Backend computes transition scores using Camelot (50%), BPM (30%), and energy (20%) weights
9. Backend applies greedy nearest-neighbor + 2-opt optimization and persists the new order
10. Frontend displays the arranged setlist with transition score badges and harmonic flow score
11. Original order is preserved as `original_position` and can be toggled back

### Failure Postconditions
1. If Claude API fails, user sees "Couldn't generate a setlist right now. Please try again." — no partial/corrupt output
2. If LLM returns malformed JSON, system retries once; if still bad, shows error
3. If arrangement fails, original order is preserved

### Invariants
1. Anthropic API key is never exposed to the frontend
2. All LLM calls happen on the backend
3. Arrangement algorithm is deterministic — same input produces same output
4. Arrangement is pure reordering — no tracks added or removed

## API Contract

| Method | Path | Description | Schema Ref | Status |
|--------|------|-------------|------------|--------|
| POST | /api/setlists/generate | Generate setlist from prompt via Claude API | — | Draft |
| POST | /api/setlists/{id}/arrange | Arrange setlist by harmonic compatibility | — | Draft |
| GET | /api/setlists/{id} | Retrieve a setlist with tracks | — | Draft |

### POST /api/setlists/generate

**Request body:**
```json
{
  "prompt": "Deep, dubby NYC house from the early 90s, building from 118 to 126 BPM",
  "track_count": 15
}
```

**Response (201 Created):**
```json
{
  "id": "uuid",
  "prompt": "...",
  "model": "claude-sonnet-4-20250514",
  "tracks": [
    {
      "position": 1,
      "title": "Track Name",
      "artist": "Artist Name",
      "bpm": 124.5,
      "key": "A minor",
      "camelot": "8A",
      "energy": 5,
      "transition_note": "Blend low-end, match kick",
      "source": "catalog",
      "track_id": "uuid-or-null"
    }
  ],
  "notes": "This set builds from deep house into peak-time territory...",
  "harmonic_flow_score": null,
  "created_at": "2026-03-02T12:00:00Z"
}
```

### POST /api/setlists/{id}/arrange

**Request body:** (empty — uses existing setlist data)

**Response (200 OK):**
```json
{
  "id": "uuid",
  "harmonic_flow_score": 82,
  "score_breakdown": {
    "key_compatibility": 90,
    "bpm_continuity": 78,
    "energy_arc": 70
  },
  "tracks": [
    {
      "position": 1,
      "title": "Track Name",
      "artist": "Artist Name",
      "bpm": 118.0,
      "key": "C minor",
      "camelot": "5A",
      "energy": 3,
      "transition_note": "...",
      "transition_score": 0.87,
      "original_position": 5,
      "source": "catalog",
      "track_id": "uuid-or-null"
    }
  ]
}
```

### GET /api/setlists/{id}

**Response (200 OK):** Same shape as generate response, with `harmonic_flow_score` populated if arranged.

## Main Success Scenario

1. **[Frontend]** User navigates to the "Generate Setlist" screen and types a prompt: "Deep house, 120-126 BPM, Camelot keys around 8A"
2. **[Frontend → API]** App sends `POST /api/setlists/generate` with `{ prompt, track_count: 15 }`
3. **[API]** Server validates prompt is non-empty and ≤2000 chars
4. **[API → DB]** Server loads user's track catalog with DJ metadata (title, artist, BPM, key, Camelot, energy)
5. **[API → Claude]** Server constructs Claude API request: system prompt (DJ persona + JSON schema + Camelot rules) + catalog context + user prompt. Calls `POST /v1/messages` with `model: claude-sonnet-4-20250514`
6. **[Claude → API]** Claude returns structured JSON: array of tracks with position, title, artist, BPM, key, camelot, energy, transition_note, source, track_id
7. **[API]** Server parses JSON, validates required fields, checks catalog track_ids against DB
8. **[API → DB]** Server creates `setlists` row + `setlist_tracks` rows with position ordering
9. **[API → Frontend]** Server returns 201 with the full setlist response
10. **[Frontend]** App renders the setlist: track list with BPM, key, and catalog/suggestion markers
11. **[Frontend]** User taps "Arrange by Key" button
12. **[Frontend → API]** App sends `POST /api/setlists/{id}/arrange`
13. **[API → DB]** Server loads setlist tracks with BPM, Camelot key, and energy data
14. **[API]** Server computes transition score matrix (Camelot 50% + BPM 30% + energy 20%), applies greedy nearest-neighbor from lowest-energy track, then 2-opt local optimization
15. **[API → DB]** Server persists new positions, transition scores, and harmonic_flow_score
16. **[API → Frontend]** Server returns 200 with arranged setlist + scores
17. **[Frontend]** App displays arranged setlist with transition score badges (green/yellow/red) and harmonic flow score

## Extensions (What Can Go Wrong)

- **2a. Empty prompt submitted**:
  1. API returns 400: `{ "error": { "code": "INVALID_REQUEST", "message": "Prompt cannot be empty" } }`
  2. Frontend shows validation message, returns to step 1

- **2b. Prompt exceeds 2000 characters**:
  1. API returns 400: `{ "error": { "code": "INVALID_REQUEST", "message": "Prompt exceeds 2000 character limit" } }`
  2. Frontend shows validation message, returns to step 1

- **4a. User has 0 tracks in catalog**:
  1. API returns 400: `{ "error": { "code": "EMPTY_CATALOG", "message": "Import tracks before generating a setlist" } }`
  2. Frontend shows helpful guidance to import tracks

- **5a. Claude API returns 429 (rate limited)**:
  1. Server retries after delay, up to 3 attempts
  2. If still limited, returns 503: `{ "error": { "code": "SERVICE_BUSY", "message": "AI service is busy. Please try again in a few minutes." } }`

- **5b. Claude API returns 500/502/503 (service error)**:
  1. Server retries up to 2 times with exponential backoff
  2. If all retries fail, returns 503

- **5c. Claude API times out (>30s)**:
  1. Returns 504: `{ "error": { "code": "TIMEOUT", "message": "Setlist generation timed out. Please try again." } }`

- **6a. Claude returns malformed JSON**:
  1. Server retries once with stricter prompt
  2. If still malformed, returns 500: `{ "error": { "code": "GENERATION_FAILED", "message": "Had trouble formatting the setlist. Please try again." } }`

- **6b. Claude returns valid JSON with missing required fields**:
  1. Server fills defaults where possible (missing energy → estimate from BPM)
  2. Skips entries missing title or artist
  3. If >50% invalid, returns error

- **7a. Catalog track_ids from Claude don't exist in DB**:
  1. Those tracks are reclassified as `source: "suggestion"` (track_id set to null)
  2. Continues with corrected data

- **8a. Database write fails**:
  1. Returns 500: `{ "error": { "code": "INTERNAL_ERROR", "message": "Setlist generated but couldn't be saved" } }`
  2. Response still includes the setlist data so frontend can display it

- **12a. Setlist not found for arrangement**:
  1. Returns 404: `{ "error": { "code": "NOT_FOUND", "message": "Setlist not found" } }`

- **13a. Setlist has <2 tracks**:
  1. 0 tracks: Returns 400 `INVALID_REQUEST`
  2. 1 track: Returns 200 with `harmonic_flow_score: 100.0` (single track is perfectly arranged). *Implementation deviation from original spec (which returned 400 for all <2): returning the setlist unchanged is more useful than erroring.*

- **14a. All tracks missing BPM/key data**:
  1. Arrangement runs with neutral defaults (0.5 scores), returns 200 with numeric `harmonic_flow_score`
  2. *Implementation deviation: original spec said return null score and original order. Current behavior is acceptable for steel thread — the arrangement algorithm handles missing data gracefully via neutral defaults. Full UC-017 can add metadata-quality warnings.*

- **14b. Arrangement algorithm produces worse score than original**:
  1. *Not implemented in steel thread.* The greedy + 2-opt algorithm is monotonically improving, so this case is unlikely. Deferred to full UC-017 if needed.

## Integration Assertions

1. **[Frontend → API → Claude → API → Frontend]** A natural language prompt submitted from the frontend produces a rendered setlist in the UI — full round trip across all layers
2. **[API → Claude]** Claude API receives a well-formed request with system prompt, catalog context, and user prompt, and returns parseable structured JSON
3. **[API → DB]** Setlist and setlist_tracks rows are correctly persisted and can be retrieved via GET /api/setlists/{id}
4. **[API]** Arrangement algorithm produces deterministic output — same input always yields same arrangement
5. **[API]** Camelot-compatible transitions score higher than incompatible ones in the scoring matrix
6. **[API]** Arrangement completes in <500ms for setlists up to 20 tracks
7. **[Frontend → API → API → Frontend]** Arrange endpoint round trip: frontend sends arrange request, receives arranged setlist with transition scores and harmonic flow score
8. **[API]** Nested error format `{ "error": { "code", "message" } }` is consistent across all error responses (matching ST-001 contract)

## Does NOT Prove

- **Prompt caching optimization** — proves Claude API works, not that caching reduces cost (deferred to full UC-016)
- **Large catalog pre-filtering** — thread uses a small test catalog (<100 tracks), not 5000+ (covered by UC-016 extension 5c)
- **Per-user daily rate limits** — no usage tracking in this thread (covered by full UC-016)
- **Frontend visualizations** — proves setlist renders as a list, not full BPM flow chart / energy arc graph (covered by full UC-016/017)
- **Original/harmonic order toggle** — proves arrangement works, frontend toggle deferred to full UC-017
- **Held-Karp optimal algorithm** — thread uses greedy+2-opt only; Held-Karp optimization deferred to full UC-017
- **Occasion-based or artist-anchored variations** — thread uses a simple genre prompt only (UC-016 variations)
- **Crossfade preview playback** — separate steel thread / UC-019
- **Conversational refinement** — UC-023 scope

## Agent Execution Notes

- **Verification Command**: `cd backend && cargo test --test setlist_api_test && cd ../frontend && flutter test test/screens/setlist_generation_test.dart`
- **Test File**: `backend/tests/setlist_api_test.rs`, `frontend/test/screens/setlist_generation_test.dart`
- **Depends On**: ST-001 (track catalog API, nested error format, DB schema), Anthropic API access
- **Blocks**: UC-016 (full generation), UC-017 (full arrangement), UC-019 (crossfade preview)
- **Estimated Complexity**: XL / ~5000 tokens implementation budget
- **Agent Assignment**:
  - Teammate:Backend-1 — Claude API client (reqwest), prompt construction, JSON response parsing, setlist persistence (migration 004, setlists + setlist_tracks tables), generate endpoint
  - Teammate:Backend-2 — Camelot module, scoring matrix, greedy+2-opt arrangement algorithm, arrange endpoint, GET setlist endpoint
  - Teammate:Frontend — Setlist generation screen (prompt input, loading, setlist list view), arrange button, transition score badges, harmonic flow score display

### Key Implementation Details

- **Migration 004**: `setlists` (id, user_id, prompt, model, harmonic_flow_score, created_at) + `setlist_tracks` (id, setlist_id, track_id nullable, position, original_position, title, artist, bpm, key, camelot, energy, transition_note, transition_score, source, acquisition_info)
- **Claude API**: `POST https://api.anthropic.com/v1/messages` via reqwest, `ANTHROPIC_API_KEY` from env, `model: claude-sonnet-4-20250514`, `max_tokens: 4096`
- **System prompt**: DJ expert persona + output JSON schema + Camelot compatibility rules (~2K tokens)
- **Camelot scoring**: Parse Camelot notation (e.g., "8A"), compute compatibility: same=1.0, ±1 number or same-number-different-letter=0.9, ±2=0.5, else=0.0
- **BPM scoring**: |a-b| mapped: 0-2→1.0, 3-4→0.8, 5-6→0.6, 7-10→0.3, >10→0.0
- **Energy scoring**: Position-aware arc: build (0-40%), peak (40-75%), cooldown (75-100%)
- **Greedy start**: Lowest energy track as opener
- **2-opt**: Swap pairs, keep if total score improves, max 100 iterations
- **Mock Claude in tests**: Tests use a mock HTTP server (or trait-based mock) returning canned JSON — do NOT call real Claude API in automated tests

## Acceptance Criteria

- [ ] All success postconditions verified by automated test
- [ ] All integration assertions pass end-to-end (with mocked Claude for determinism)
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] API contract matches implementation (request/response shapes match the contract above)
- [ ] Cross-layer round trip completes without manual intervention
- [ ] Arrangement algorithm is deterministic (same input → same output)
- [ ] Camelot-compatible transitions score higher than incompatible ones
- [ ] Greedy+2-opt produces arrangement with higher harmonic_flow_score than random order
- [ ] Tracks missing BPM/key are placed at end without crashing
- [ ] Nested error format consistent with ST-001 contract
- [ ] Code passes quality gates (cargo fmt, clippy, cargo test, flutter analyze, flutter test)
- [ ] Reviewer agent approves

## Completeness Score

| Section | Weight | Score | Notes |
|---------|--------|-------|-------|
| Classification | 5% | 5/5 | Thread level, system scope, P0 critical |
| Cross-Cutting References | 15% | 14/15 | Both UC-016 and UC-017 with specific MSS steps |
| Actors | 5% | 5/5 | All actors identified |
| Conditions | 10% | 10/10 | Pre/post/failure/invariants comprehensive |
| API Contract Section | 15% | 14/15 | Three endpoints with request/response shapes |
| Main Success Scenario | 15% | 15/15 | 17 steps crossing 5 layers (Frontend, API, DB, Claude, Frontend) |
| Extensions | 10% | 9/10 | 14 extension paths covering integration failures |
| Integration Assertions | 15% | 14/15 | 8 cross-layer assertions with specific layer callouts |
| Does NOT Prove | 5% | 5/5 | 9 explicit exclusions preventing scope creep |
| Agent Execution Notes | 5% | 5/5 | Verification, dependencies, assignment, implementation details |
| **Total** | **100%** | **96/100** | |
