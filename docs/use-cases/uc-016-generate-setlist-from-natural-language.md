# Use Case: UC-016 Generate Setlist from Natural Language Prompt

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P0 Critical
- **Complexity**: 🟠 High

## Actors
- **Primary Actor**: App User (authenticated, DJ)
- **Supporting Actors**:
  - Claude API (Sonnet default, Opus for complex refinements)
  - Database (SQLite/PostgreSQL via SQLx — user's track catalog)
- **Stakeholders & Interests**:
  - DJ User: Wants to describe a vibe, scene, era, or occasion in natural language and receive a playable, harmonically coherent setlist drawn from their catalog — plus discovery suggestions for tracks they don't own yet
  - Developer: Wants clean separation between LLM prompt engineering and application logic; structured JSON output for downstream processing (UC-017 arrangement, UC-019 preview)
  - Business: This is the core differentiator — the LLM-as-crate-digger experience

## Conditions
- **Preconditions** (must be true before starting):
  1. User is authenticated in the app
  2. User has at least 1 track in their catalog (ideally 20+ for meaningful setlists)
  3. Backend has valid Anthropic API key configured via environment
  4. At least some tracks have BPM and key data (from Beatport native or UC-015 analysis)
  5. **Pre-implementation spike required**: Test Claude Sonnet's music knowledge accuracy by prompting it with 20 known DJ setlist scenarios and validating track suggestions against real catalogs. Acceptance threshold: >80% of suggested tracks should be real, commercially released tracks. If accuracy is too low, consider adding MusicBrainz lookup as a validation layer.

- **Success Postconditions** (true when done right):
  1. A setlist is generated containing 10-20 tracks (default ~15 for a 1-hour set) ordered for DJ playback
  2. Each track in the setlist includes: title, artist, BPM, musical_key, camelot_key, energy_level (1-8), transition_note (to next track), and source ('catalog' or 'suggestion')
  3. Tracks sourced from the user's catalog are linked to their database track IDs
  4. Suggested tracks (not in catalog) include enough metadata for the user to find and purchase them (artist, title, label, release year, suggested source: Beatport/SoundCloud)
  5. The setlist has a coherent energy arc (not random — builds, peaks, breathes)
  6. The setlist is persisted in the database with: user_id, prompt text, generated tracks, timestamp, model used
  7. BPM transitions between adjacent tracks are within ±6 BPM (or flagged if larger)
  8. Key transitions favor Camelot-compatible keys (scored but not strictly enforced — UC-017 handles arrangement)
  9. If the LLM returns a setlist where fewer than 30% of tracks match the user's catalog, the system warns: "Most tracks in this setlist are suggestions — you may not own them yet. Consider importing more tracks matching this style." This prevents the system from silently generating setlists entirely from hallucinated tracks.

- **Failure Postconditions** (true when it fails gracefully):
  1. If the LLM call fails, user receives "Couldn't generate a setlist right now. Please try again." with no partial/corrupt output
  2. If the user's catalog is too small for the prompt, system explains: "Your catalog has X tracks. Import more tracks from Beatport or SoundCloud for better results."
  3. If the LLM returns malformed JSON, system retries once with a stricter prompt; if still malformed, displays error

- **Invariants** (must remain true throughout):
  1. Anthropic API key is never exposed to the frontend
  2. All LLM calls happen on the backend
  3. The user's full catalog is never sent to external services other than the Anthropic API (and only as structured metadata, not audio)
  4. Per-user daily LLM usage limits are enforced to control costs (configurable, default 20 setlists/day)
  5. Prompt caching is used to minimize API costs (system prompt + catalog context cached)
  6. LLM responses are NOT cached at the application level. Each generation produces a fresh result. Caching is handled at the Anthropic API level via prompt caching (cache_control on system prompt + catalog context).

## Main Success Scenario
1. User navigates to the "Generate Setlist" screen
2. User types a natural language prompt describing the desired setlist (e.g., "Deep, dubby NYC house from the early 90s, Sound Factory vibes, building from 118 to 126 BPM")
3. User optionally specifies: target duration (default 1 hour), number of tracks (default ~15), or occasion context
4. System validates the prompt is non-empty and within length limits (max 2000 chars)
5. Backend loads the user's full track catalog from the database with all DJ metadata (title, artist, BPM, key, Camelot, genre, energy, source)
6. Backend constructs the Claude API request:
   - **System prompt** (~2K tokens, cached): DJ expert persona, output JSON schema, Camelot rules, energy arc guidelines, instruction to draw from catalog first and suggest external tracks to fill gaps
   - **Catalog context** (variable, cached per user): Serialized track list with DJ metadata
   - **User prompt**: Verbatim user input + any specified constraints (duration, track count)
7. Backend calls Claude API (Sonnet by default) with prompt caching enabled
8. Claude returns a structured JSON response: ordered array of tracks, each with title, artist, bpm, key, camelot_key, energy_level, transition_note, source ('catalog'|'suggestion'), and for catalog tracks the track_id
9. Backend validates the LLM response: parses JSON, verifies required fields, checks catalog track_ids exist in the database
10. Backend persists the setlist: creates a `setlists` record with user_id, prompt, model, timestamp, and a `setlist_tracks` join with position ordering
11. System displays the generated setlist to the User with: track listing, BPM flow visualization, key compatibility indicators, energy arc graph, and catalog vs. suggestion markers
12. User can see which tracks they own (catalog) and which are suggestions to acquire

## Extensions (What Can Go Wrong)

- **2a. User submits an empty prompt**:
  1. System displays "Describe the vibe, genre, era, or occasion for your setlist."
  2. Returns to step 2

- **2b. User submits a non-music prompt (gibberish or off-topic)**:
  1. LLM responds with best-effort interpretation or a clarification request
  2. If LLM returns an empty setlist, system displays "Couldn't interpret that as a music request. Try describing a genre, era, mood, or DJ style."
  3. Returns to step 2

- **3a. User requests an unreasonably long setlist (>50 tracks / >4 hours)**:
  1. System caps at 50 tracks and informs: "Maximum setlist length is 50 tracks (~4 hours). Generating 50 tracks."
  2. Continues to step 4

- **3b. User requests a very short setlist (<3 tracks)**:
  1. System allows it — even a 2-track transition is valid for practice
  2. Continues to step 4

- **4a. Prompt exceeds 2000 characters**:
  1. System displays "Prompt is too long (max 2000 characters). Try being more concise."
  2. Returns to step 2

- **5a. User has 0 tracks in catalog**:
  1. System displays "You need to import some tracks first. Go to Import to add tracks from Spotify, Beatport, or SoundCloud."
  2. Use case fails (redirects to import)

- **5b. User has tracks but none have BPM/key data yet (all pending analysis)**:
  1. System proceeds with available metadata (title, artist, genre)
  2. LLM generates setlist without BPM/key constraints
  3. Summary notes: "Some tracks are still being analyzed for BPM and key. Setlist arrangement will improve once analysis completes."

- **5c. Catalog is very large (>5000 tracks) and exceeds context window**:
  1. Backend pre-filters catalog by relevance: uses genre, BPM range, and keywords from the prompt to select a subset
  2. Includes up to 2000 tracks in context (fits in 200K window with metadata). Token math: each track serialized as `"ID | Title - Artist | BPM | Key | Energy"` ≈ 50-80 chars ≈ 20-30 tokens. 500 tracks × 25 tokens = ~12,500 tokens (fits easily). 2000 tracks × 25 tokens = ~50,000 tokens (still fits). For catalogs > 1000 tracks, group by genre/BPM range and send only the relevant subset based on the prompt. Full catalog serialization is viable up to ~2000 tracks within Claude's 200K context window.
  3. Continues to step 6

- **6a. Per-user daily limit reached**:
  1. System displays "You've reached the daily limit of 20 setlist generations. Try again tomorrow, or refine an existing setlist."
  2. Use case fails

- **7a. Claude API returns 429 (rate limited)**:
  1. System retries after the specified delay
  2. If still rate-limited after 3 attempts, displays "Our AI service is busy. Please try again in a few minutes."
  3. Use case fails

- **7b. Claude API returns 500/502/503 (service error)**:
  1. System retries up to 2 times with exponential backoff
  2. If all retries fail, displays "AI service is temporarily unavailable. Please try again shortly."
  3. Use case fails

- **7c. Claude API request times out (>30 seconds)**:
  1. System displays "Setlist generation is taking longer than expected. Please try again."
  2. Use case fails (no partial output from LLM)

- **7d. Claude API returns content that triggers safety filters**:
  1. System displays "Couldn't generate that setlist. Try rephrasing your request."
  2. Returns to step 2

- **8a. LLM returns malformed JSON (not parseable)**:
  1. Backend retries with a stricter system prompt emphasizing JSON format
  2. If second attempt also fails, displays "Had trouble formatting the setlist. Please try again."
  3. Use case fails

- **8b. LLM returns valid JSON but missing required fields**:
  1. Backend fills in defaults where possible (e.g., missing energy_level → estimate from BPM)
  2. If track title or artist is missing, skip that entry
  3. If >50% of entries are invalid, treat as failure and display error

- **8c. LLM hallucinates catalog tracks (returns track_ids that don't exist)**:
  1. Backend validation catches non-existent track_ids
  2. Those tracks are reclassified as 'suggestion' (they exist in LLM's knowledge, not user's catalog)
  3. Continues with corrected data

- **8d. LLM returns fewer tracks than requested**:
  1. System displays the setlist as-is with a note: "Generated X tracks (fewer than requested). Your catalog may not have enough matching tracks."

- **9a. JSON validation passes but data quality is poor (all same BPM, random key jumps)**:
  1. Backend does NOT reject — UC-017 handles arrangement optimization
  2. System displays the raw LLM setlist with a suggestion: "Tap 'Optimize' to arrange by harmonic compatibility"

- **10a. Database persistence fails**:
  1. System still displays the generated setlist to the user (it's in memory)
  2. Displays warning: "Setlist generated but couldn't be saved. It won't appear in your history."
  3. User can still view and use the setlist in the current session

- **11a. Frontend fails to render the setlist visualization**:
  1. System falls back to a simple text list (track name, artist, BPM, key)
  2. Visualization features degrade gracefully

## Variations

- **V1. Occasion-Based Prompt**: User specifies an occasion (e.g., "Nikah ceremony, 2 hours, starting mellow ending upbeat"). LLM understands cultural context and selects appropriate tracks.
- **V2. Artist-Anchored Prompt**: User says "Build a set around Kerri Chandler and Louie Vega". LLM uses those artists as anchors and fills in complementary tracks.
- **V3. BPM-Constrained Prompt**: User says "techno set, 130-138 BPM, never drop below 130". LLM strictly respects BPM bounds.
- **V4. Catalog-Only Mode**: User toggles "catalog only" — LLM only uses tracks from their library, no suggestions.
- **V5. Regenerate with Tweaks**: User says "like the last one but more upbeat" — system includes the previous setlist as context for refinement. (Expanded in UC-023.)

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test setlist_generation`
- **Test File**: `backend/tests/setlist_generation.rs`
- **Depends On**: UC-001/013/014 (tracks in catalog), UC-015 (BPM/key data), Anthropic API access
- **Blocks**: UC-017 (harmonic arrangement), UC-019 (crossfade preview), UC-023 (conversational refinement)
- **Estimated Complexity**: H (~3500 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend-1 — Claude API client, prompt construction, response parsing, setlist persistence (setlists + setlist_tracks tables)
  - Teammate:Backend-2 — Catalog serialization, pre-filtering for large catalogs, usage limit enforcement
  - Teammate:Frontend — Setlist generation screen (prompt input, loading state, setlist display with BPM/key/energy visualization)

### Key Implementation Details
- **Claude API**: `POST /v1/messages` with `model: claude-sonnet-4-20250514`, prompt caching via `cache_control: { type: "ephemeral" }` on system + catalog blocks
- **System prompt structure**:
  ```
  [DJ expert persona + rules] (~1K tokens, cached)
  [Output JSON schema] (~500 tokens, cached)
  [Camelot compatibility rules] (~300 tokens, cached)
  [User's catalog] (variable, cached per-user)
  [User prompt] (not cached)
  ```
- **Output JSON schema**:
  ```json
  {
    "setlist": [{
      "position": 1,
      "title": "...",
      "artist": "...",
      "bpm": 124.5,
      "key": "A minor",
      "camelot": "8B",
      "energy": 5,
      "transition_note": "Blend low-end, match the kick pattern",
      "source": "catalog",
      "track_id": "uuid-if-catalog",
      "acquisition": null
    }],
    "notes": "This set builds from deep house into peak-time territory...",
    "suggested_acquisitions": [{
      "title": "...", "artist": "...", "label": "...", "year": 1993,
      "find_on": "beatport"
    }]
  }
  ```
- **DB migration**: `004_setlists.sql` — `setlists` (id, user_id, prompt, model, created_at) + `setlist_tracks` (id, setlist_id, track_id nullable, position, title, artist, bpm, key, camelot, energy, transition_note, source, acquisition_info)
- **Claude API max_tokens**: `max_tokens: 4096` for setlist generation. A 15-track setlist with metadata and transition notes fits in ~2000 tokens. The extra headroom handles verbose explanations.
- **Estimated cost per setlist generation**: ~$0.01-0.03 with prompt caching enabled (input: ~15K tokens cached + ~500 tokens new, output: ~2K tokens). Without caching: ~$0.08-0.15. Daily limit of 20 generations = ~$0.20-0.60/user/day max.
- **Cost control**: Count API calls per user per day in `user_usage` table. Default limit: 20/day.

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] Code passes quality gates
- [ ] Natural language prompt produces a structured JSON setlist via Claude API (mock in tests)
- [ ] Setlist includes both catalog tracks (with track_ids) and suggestions (with acquisition info)
- [ ] Catalog tracks are validated against the database
- [ ] LLM hallucinated track_ids are caught and reclassified as suggestions
- [ ] Energy arc in generated setlist shows variation (not flat or random)
- [ ] BPM transitions between adjacent tracks are reasonable (flagged if >±6 BPM)
- [ ] Prompt caching is enabled (verified by cache_read_input_tokens in response)
- [ ] Per-user daily usage limits are enforced
- [ ] Setlist is persisted in database and retrievable
- [ ] Frontend displays setlist with BPM flow, key indicators, and energy visualization
- [ ] Malformed LLM responses trigger retry and graceful fallback
- [ ] Empty or too-small catalog shows helpful guidance message
