# Use Case: UC-018 Enrich Track with DJ Metadata

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P1 Important
- **Complexity**: 🟡 Medium

## Actors
- **Primary Actor**: System (background enrichment, triggered after UC-015 analysis completes)
- **Secondary Actor**: App User (views enriched metadata, can manually trigger re-enrichment)
- **Supporting Actors**:
  - Claude API (Sonnet — metadata inference from track context)
  - Database (SQLx)
- **Stakeholders & Interests**:
  - DJ User: Wants energy levels, mood tags, and refined genre classification to enable intelligent setlist building and energy arc planning
  - Developer: Wants enrichment to be idempotent and cacheable — run once, store permanently

## Conditions
- **Preconditions** (must be true before starting):
  1. Track exists in database with at least: title, artist, and ideally BPM/key (from UC-013 Beatport native or UC-015 analysis)
  2. Backend has valid Anthropic API key
  3. Track has not been enriched yet (or user explicitly requests re-enrichment)

- **Success Postconditions** (true when done right):
  1. Track has `energy_level` (integer 1-8) populated
  2. Track has `mood_tags` (array of strings: e.g., ["dark", "groovy", "hypnotic"]) populated
  3. Track has `genre` and `sub_genre` refined by LLM (may differ from source-provided genre if source was user-tagged)
  4. `enriched_at` timestamp is recorded
  5. Enrichment is persisted and survives re-imports (not overwritten unless explicitly requested)

- **Failure Postconditions** (true when it fails gracefully):
  1. If LLM call fails, track retains whatever metadata it had before — no data loss
  2. `enrichment_error` field captures the failure reason
  3. Track can be retried later

- **Invariants** (must remain true throughout):
  1. Enrichment never modifies BPM, key, or Camelot data (those come from UC-015/Beatport)
  2. LLM calls are batched (10-20 tracks per request) to minimize API costs
  3. Anthropic API key never exposed to frontend

## Main Success Scenario
1. Background worker identifies tracks with `enriched_at = null` and `bpm IS NOT NULL` (prioritize tracks that have audio analysis complete)
2. Worker batches up to 20 tracks and constructs a Claude API request:
   - System prompt: Music metadata expert, output JSON schema for energy/mood/genre
   - Track context: Title, artist, BPM, key, genre (if available), label, release year
3. Worker calls Claude API (Sonnet) requesting structured enrichment for the batch
4. Claude returns JSON array with energy_level, mood_tags, and refined genre/sub_genre for each track
5. Worker validates response: energy in 1-8 range, mood_tags are strings, genre/sub_genre are non-empty
6. Worker updates each track in the database with enrichment data and `enriched_at` timestamp
7. User sees enriched metadata appear in their catalog (energy badges, mood tags, refined genre)

## Extensions (What Can Go Wrong)

- **1a. No tracks need enrichment**:
  1. Worker sleeps and checks again later

- **1b. Tracks exist but lack BPM/key (analysis pending)**:
  1. Worker can still enrich based on title/artist/genre alone — lower quality but still useful
  2. Re-enrichment triggered after analysis completes for improved accuracy

- **3a. Claude API rate limited or unavailable**:
  1. Worker retries with backoff
  2. If persistent, logs error and tries next cycle

- **4a. LLM returns malformed JSON**:
  1. Worker retries with stricter prompt
  2. If still malformed, marks batch as `enrichment_error` and continues

- **4b. LLM returns energy levels outside 1-8 range**:
  1. Worker clamps to valid range (min 1, max 8)
  2. Logs warning

- **5a. LLM returns empty mood_tags**:
  1. Worker stores empty array — not a failure
  2. Track still enriched with energy and genre

- **6a. Database update fails for a track**:
  1. Skip that track, continue with rest of batch
  2. Track retried on next cycle

## Variations

- **V1. Manual Enrichment**: User selects tracks and clicks "Enrich" to trigger immediate LLM enrichment (bypasses queue).
- **V2. User Override**: User manually sets energy level or mood tags, overriding LLM values. Manual values are preserved on re-enrichment.
- **V3. Bulk Re-enrichment**: User triggers re-enrichment for all tracks (e.g., after model upgrade for better quality).

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test track_enrichment`
- **Test File**: `backend/tests/track_enrichment.rs`
- **Depends On**: UC-015 (BPM/key data improves enrichment quality), UC-016 (uses enriched data for setlists)
- **Blocks**: UC-016 (energy levels used in setlist generation), UC-017 (energy scoring in arrangement)
- **Estimated Complexity**: M (~1500 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend — Enrichment worker, Claude API batch requests, DB updates
  - Teammate:Frontend — Energy badges (1-8 color scale), mood tag chips, refined genre display

### Key Implementation Details
- **Shared worker**: The enrichment worker handles both DJ metadata (energy, mood, genre) and scene/era classification (UC-021) in the same processing loop. Each batch of 20 tracks gets both enrichment and classification in a single LLM call.
- **Batch size**: 20 tracks per LLM call (fits comfortably in context)
- **LLM cost**: ~$0.003 per batch of 20 tracks (Sonnet pricing with caching)
- **Energy scale**: 1=ambient, 2=chill, 3=warm-up, 4=grooving, 5=driving, 6=peak, 7=intense, 8=maximum
- **Mood vocabulary**: Constrained set — dark, euphoric, groovy, melancholic, hypnotic, uplifting, aggressive, deep, soulful, funky, minimal, atmospheric
- **Migration**: Add `energy_level INTEGER`, `mood_tags TEXT` (JSON array), `enriched_at TIMESTAMP`, `enrichment_error TEXT` to tracks table

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] Energy levels are integers in 1-8 range
- [ ] Mood tags are from the constrained vocabulary
- [ ] Enrichment is batched (20 tracks per LLM call)
- [ ] Enrichment doesn't overwrite BPM/key data
- [ ] Re-import doesn't overwrite enrichment data
- [ ] LLM failures don't crash the worker
- [ ] Frontend displays energy badges and mood tags
