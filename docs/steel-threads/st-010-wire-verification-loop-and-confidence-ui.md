# Steel Thread: ST-010 Wire Verification Loop and Confidence UI

## Classification

- **Goal Level**: Thread
- **Scope**: System (black box)
- **Priority**: P1 High
- **Complexity**: Medium

## Cross-Cutting References

- **UC-001**: Steps 5, 7 — proves generation request accepts `verify` flag, response includes confidence per track
- **UC-013**: Steps 3, 11, 14 — proves natural language → generation → verification loop → response with confidence
- **ST-003**: Extends generation endpoint with verification pass
- **SP-007**: Implements findings — skill doc already deployed, this wires `verify_setlist()` into the hot path and adds UI

## Actors

- **Primary Actor**: App User (DJ requesting a setlist)
- **Supporting Actors**: Claude API (generation + verification), Database (persist confidence), Frontend (display confidence badges)
- **Stakeholders & Interests**:
  - DJ User: wants to know which suggested tracks are real vs. likely hallucinated
  - Platform: improved trust and perceived quality of LLM-generated setlists

## Conditions

### Preconditions
1. SP-007 skill doc and confidence field deployed (DONE — in-memory structs only)
2. `verify_setlist()` function exists in `services/setlist.rs` (DONE)
3. `backend/src/prompts/verification_prompt.md` exists (DONE)
4. `confidence` field exists on `LlmTrackEntry`, `SetlistTrackResponse`, and Flutter `SetlistTrack` (DONE — structs only, NOT in DB)
5. DB persistence of confidence/flags does NOT exist yet — no column, no INSERT, no SELECT (must be built)
6. `verification_flag` and `verification_note` fields do NOT exist on response structs or Flutter model (must be added)
7. `verify` field does NOT exist on `GenerateRequest` or `GenerateSetlistRequest` (must be added)

### Success Postconditions
1. POST `/api/setlists/generate` accepts optional `verify: true` field
2. When `verify: true`, response includes verification-adjusted confidence, `verification_flag`, and `verification_note` per track
3. When `verify` is omitted or false, behavior unchanged (single-pass with skill doc)
4. New migration adds `confidence TEXT`, `verification_flag TEXT`, `verification_note TEXT` columns to `setlist_tracks`
5. Confidence and flags survive DB round-trip: GET `/api/setlists/:id` returns them per track
6. Frontend displays colored confidence dot badges on each track tile (green=high, yellow=medium, orange=low)
7. Flagged tracks show verification note inline (like transition_note), not requiring a separate tap interaction
8. Generation latency with `verify: true` is < 30s P95 (aspirational — graceful degradation if exceeded)

### Failure Postconditions
1. If verification API call fails, generation still succeeds with original (unverified) confidence values
2. If verification returns malformed JSON, original tracks returned unchanged (already implemented)
3. Frontend renders tracks without badges if confidence is null (backward compatible)

### Invariants
1. All existing generation behavior preserved when `verify` is not set
2. Confidence values are always one of: "high", "medium", "low", or null
3. Quality gates pass: cargo fmt + clippy + test + flutter analyze + flutter test

## API Contract

| Method | Path | Description | Schema Ref | Status |
|--------|------|-------------|------------|--------|
| POST | /api/setlists/generate | Generate setlist (add `verify` field to request body) | — | Draft |
| GET | /api/setlists/:id | Get setlist (now includes confidence + flags per track) | — | Draft |

### Request Body Addition (POST /api/setlists/generate)
```json
{
  "prompt": "detroit techno 1996",
  "verify": true
}
```

### Response Shape (tracks array element)
```json
{
  "position": 1,
  "title": "Strings of Life",
  "artist": "Derrick May",
  "confidence": "high",
  "verification_flag": null,
  "verification_note": null
}
```

New fields:
- `confidence`: `"high"` | `"medium"` | `"low"` | `null`
- `verification_flag`: `null` | `"wrong_artist"` | `"no_such_track"` | `"constructed_title"` | `"replaced"` | `"uncertain"`
- `verification_note`: `null` | `"Actually by DJ Hell, not Underground Resistance"` (human-readable correction text)

## Main Success Scenario

1. **[Frontend]** User types "detroit techno 1996" and checks "Verify tracks" toggle
2. **[Frontend → API]** App sends POST `/api/setlists/generate` with `{ "prompt": "detroit techno 1996", "verify": true }`
3. **[API]** Server validates request (including new `verify` field), loads catalog, builds system prompt (with skill doc)
4. **[API → Claude]** Server calls Claude to generate setlist (Pass 1: generation)
5. **[API]** Server parses response, validates track IDs, normalizes confidence
6. **[API → Claude]** Server calls Claude with verification prompt and generated tracks (Pass 2: verification)
7. **[API]** Server merges verification results: adjusts confidence, propagates `flag` and `correction` to response struct, replaces tracks where verifier suggests alternatives
8. **[API → DB]** Server persists setlist with confidence, verification_flag, and verification_note in `setlist_tracks` table (NOTE: persist happens AFTER verification, not before — requires restructuring the generate-persist loop)
9. **[API → Frontend]** Server returns full setlist response with confidence, verification_flag, and verification_note per track
10. **[Frontend]** App renders setlist with colored confidence dot badges (green/yellow/orange) and inline verification notes on flagged tracks

## Extensions

- **4a. Claude generation fails (timeout/rate limit)**:
  1. Return error as today (no change)
- **6a. Verification call fails (timeout/rate limit/error)**:
  1. Log warning
  2. Return original tracks with unverified confidence (graceful degradation)
  3. Add note to response: "Verification unavailable, confidence scores are self-reported"
- **6b. Verification returns malformed JSON**:
  1. Log warning with parse error
  2. Return original tracks unchanged (already implemented in `verify_setlist()`)
- **6c. Verification returns position mismatch (fewer/more tracks)**:
  1. Matched positions get verification updates
  2. Unmatched positions keep original values
  3. Log warning with unmatched position list (already implemented)
- **7a. Verification replaces a track title/artist**:
  1. Updated title/artist persisted to DB (since persist is after verify)
  2. Replacement tracks get confidence set to "medium" at most (already implemented)
- **8a. DB write fails**:
  1. Log error
  2. Return response with confidence in JSON (still in memory)
  3. Confidence lost on next GET (acceptable degradation)
- **10a. Confidence is null (old setlists, verification skipped)**:
  1. Frontend shows no badge — track renders normally

## Integration Assertions

1. **Frontend → API**: POST `/api/setlists/generate` with `verify: true` returns 200 with confidence + verification_flag + verification_note on every track
2. **Frontend → API**: POST `/api/setlists/generate` WITHOUT `verify` returns 200 with self-assessed confidence from generation (no second-pass verification), verification_flag and verification_note are null
3. **API → Claude (x2)**: Both generation and verification calls complete within 30s combined (aspirational, graceful degradation on timeout)
4. **API → DB → API**: Confidence + flags persisted via new columns; GET `/api/setlists/:id` returns saved values
5. **DB round-trip**: Generate with verify=true → GET → verify confidence/flags match initial response
6. **Frontend rendering**: Confidence dot badge colors match values (green=high, yellow=medium, orange=low, none=null)
7. **Graceful degradation**: If verification fails, response still includes tracks with unverified confidence, no flags
8. **Backward compatibility**: Existing clients that don't send `verify` field get identical behavior to pre-ST-010

## Does NOT Prove

- Does NOT prove V2 search-result feedback loop (feeding Deezer results back to Claude for grounded verification — future spike/ST)
- Does NOT prove confidence calibration accuracy at scale (needs data collection over time)
- Does NOT prove verification for the refinement/conversation flow (only generation). Refined setlists via conversation will lose verification status — a future ST should extend verification to the refinement flow
- Does NOT prove auto-flagging or auto-removal of low-confidence tracks (user decides)
- Does NOT prove performance under concurrent verification requests (load testing deferred)
- Does NOT prove prompt caching for the verification call (uses `generate_setlist()` not `generate_with_blocks()` — accepted as cost debt, can optimize later)

## Agent Execution Notes

- **Verification Command**: `cargo fmt --check && cargo clippy -- -D warnings && cargo test && cd ../frontend && flutter analyze && flutter test`
- **Test File**: `backend/tests/verification_integration.rs` (new), `frontend/test/widgets/confidence_badge_test.dart` (new)
- **Depends On**: SP-007 (complete), ST-003 (complete)
- **Blocks**: None
- **Estimated Complexity**: Medium / ~2000 tokens
- **Agent Assignment**: Lead coordinates + wires integration boundary (routes/setlist.rs, services/setlist.rs request structs). 2 builders: backend (DB migration + persistence + verify wiring) and frontend (model + badges + tests)

### Critical Implementation Notes
- **Persist AFTER verify**: The current `generate_setlist_from_request()` persists tracks as they are parsed. This must be restructured: collect tracks in memory → verify if requested → persist final state. This is integration-boundary work owned by the lead.
- **New integration test must include ALL migrations**: The new `create_test_pool()` in `backend/tests/verification_integration.rs` must include all 9 migrations. This is a recurring footgun (ST-006 lesson).
- **`verify_setlist()` propagation**: The existing function adjusts confidence and title/artist but does NOT propagate `flag` or `correction` to `SetlistTrackResponse`. Must add `verification_flag: Option<String>` and `verification_note: Option<String>` to: `SetlistTrackResponse`, `SetlistTrackRow`, Flutter `SetlistTrack` model, and `verify_setlist()` return path.

## Acceptance Criteria

- [ ] POST `/api/setlists/generate` accepts optional `verify: boolean` field
- [ ] When `verify: true`, response tracks include verification-adjusted confidence, flags, and notes
- [ ] When `verify` is false/omitted, behavior unchanged (single-pass + skill doc)
- [ ] Verification failure degrades gracefully (returns unverified tracks)
- [ ] New migration adds `confidence TEXT`, `verification_flag TEXT`, `verification_note TEXT` columns to `setlist_tracks`
- [ ] `SetlistTrackRow`, INSERT, SELECT, and `From` impl updated for new columns
- [ ] GET `/api/setlists/:id` returns persisted confidence + flags per track
- [ ] Frontend shows colored dot badges for confidence (green/yellow/orange)
- [ ] Frontend shows inline verification notes on flagged tracks
- [ ] All existing tests still pass
- [ ] New integration tests cover verify=true, verify=false, verification failure
- [ ] New widget tests cover confidence badge rendering for all states (high, medium, low, null, flagged)
- [ ] Quality gates pass (cargo fmt, clippy, cargo test, flutter analyze, flutter test)
