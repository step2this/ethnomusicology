# Use Case: UC-023 Refine Setlist with Conversational Feedback

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P2 Nice to Have
- **Complexity**: 🟠 High

## Actors
- **Primary Actor**: App User (authenticated, DJ)
- **Supporting Actors**:
  - Claude API (Sonnet/Opus — conversational refinement with setlist context)
  - Database (SQLx — setlists, tracks, conversation history)
- **Stakeholders & Interests**:
  - DJ User: Wants to iterate on a generated setlist through natural conversation — "swap track 5 for something darker", "increase the energy in the middle", "add a Kerri Chandler track after track 8"
  - Developer: Wants conversation history managed efficiently (not re-sending full catalog every turn)

## Conditions
- **Preconditions** (must be true before starting):
  1. User has an existing setlist (from UC-016)
  2. Backend has valid Anthropic API key
  3. User has not exceeded daily LLM usage limit

- **Success Postconditions** (true when done right):
  1. The setlist is modified according to the user's conversational feedback
  2. Changes are specific: swap a track, reorder, add, remove, adjust energy curve — not a full regeneration
  3. Conversation history is preserved: user can make multiple rounds of refinement
  4. Each refinement is persisted as a new version of the setlist (version history maintained)
  5. The user can undo/revert to any previous version
  6. LLM understands the current setlist state (track order, BPM, keys, energy) and the conversation context

- **Failure Postconditions** (true when it fails gracefully):
  1. If LLM call fails, the setlist remains unchanged (current version preserved)
  2. If LLM misinterprets the request, user can say "undo that" or "that's not what I meant"
  3. Conversation context is preserved even after failures

- **Invariants** (must remain true throughout):
  1. Refinement never silently deletes the original setlist — all versions are preserved
  2. Each refinement uses the current setlist state as context (not the original generation)
  3. Refinement turns count toward the daily LLM usage limit, but at a reduced rate: 3 refinement turns = 1 generation equivalent. This reflects the lower token cost of refinement (no catalog context needed after first turn).
  4. Anthropic API key never exposed to frontend

## Main Success Scenario
1. User views a generated setlist and taps "Refine" (or a chat icon)
2. A conversational panel opens alongside the setlist view
3. User types a refinement request (e.g., "The energy drops too much at track 7. Replace it with something that keeps the momentum going.")
4. System constructs a Claude API request:
   - System prompt: DJ assistant persona, setlist modification rules, output JSON diff format
   - Current setlist state: Full track listing with positions, BPM, key, energy
   - Conversation history: All previous refinement turns
   - User message: Verbatim input
5. Claude returns a structured JSON response describing the modification:
   ```json
   {
     "action": "replace",
     "position": 7,
     "removed": { "title": "...", "artist": "..." },
     "added": { "title": "...", "artist": "...", "bpm": 126, "key": "A minor", "source": "catalog", "track_id": "..." },
     "explanation": "Replaced with a higher-energy track that maintains the 8B Camelot key..."
   }
   ```
6. Backend validates the modification (track_id exists if catalog, positions valid)
7. Backend applies the modification and creates a new setlist version
8. Frontend updates the setlist display, highlighting the change (added track glows, removed track fades)
9. Conversation panel shows the LLM's explanation
10. User can continue refining ("now swap track 3 too") or accept the setlist

## Extensions (What Can Go Wrong)

- **3a. User request is ambiguous ("make it better")**:
  1. LLM interprets broadly — may adjust energy curve, swap weak transitions, or suggest additions
  2. Explains what it changed and why
  3. User can undo if they disagree

- **3b. User request references a track by position that doesn't exist ("swap track 20" in a 15-track setlist)**:
  1. LLM catches the error and responds: "This setlist only has 15 tracks. Did you mean track 15?"
  2. Returns to step 3

- **3c. User requests a track that doesn't exist in catalog or LLM's knowledge**:
  1. LLM responds: "I don't have a track called [X]. Did you mean [similar track]? Or I can suggest something similar."
  2. Returns to step 3

- **3d. User says "undo" or "revert"**:
  1. System restores the previous setlist version
  2. Conversation continues from the restored state

- **4a. Conversation history exceeds context window**:
  1. System summarizes older turns and keeps recent 5 turns in full
  2. Current setlist state is always included in full

- **5a. LLM returns a full regeneration instead of a targeted modification**:
  1. Backend detects >50% of tracks changed
  2. Warns user: "The AI suggested replacing most of the setlist. Apply changes? (This is closer to a new generation than a refinement.)"
  3. User can accept or reject

- **5b. LLM returns malformed JSON**:
  1. Backend retries with stricter prompt
  2. If still malformed, displays LLM's natural language explanation without applying changes
  3. User can rephrase

- **6a. Suggested catalog track doesn't exist (hallucinated track_id)**:
  1. Reclassify as suggestion (same as UC-016 extension 8c)
  2. Apply modification with suggestion marker

- **7a. Version creation fails (DB error)**:
  1. Display modification in UI (in memory) but warn it's not saved
  2. Offer retry

- **10a. User abandons refinement without accepting**:
  1. All versions are preserved — user can return to any version later
  2. The "active" version remains the last explicitly accepted one

## Variations

- **V1. Quick Commands**: Shortcuts like "shuffle middle section", "sort by BPM", "reverse order" that don't need LLM — handled by backend logic directly.
- **V2. Voice Refinement**: User speaks refinement request via speech-to-text (future, requires browser microphone API).
- **V3. Collaborative Refinement**: Two users refine the same setlist in real-time (far future).

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test setlist_refinement`
- **Test File**: `backend/tests/setlist_refinement.rs`
- **Depends On**: UC-016 (generated setlist to refine), UC-017 (arrangement scores updated after refinement)
- **Blocks**: None
- **Estimated Complexity**: H (~3000 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend — Conversation management, Claude API with multi-turn context, setlist versioning (version table), modification validation and application
  - Teammate:Frontend — Conversational panel (chat UI), setlist diff visualization (highlight changes), version history sidebar, undo button

### Key Implementation Details
- **Conversation storage**: `setlist_conversations` table: id, setlist_id, role ('user'|'assistant'), content, version_id, created_at
- **Setlist versioning**: `setlist_versions` table: id, setlist_id, version_number, created_at. Each setlist version snapshots the full `setlist_tracks` state. When a new version is created, the current track positions are copied to a `setlist_track_snapshots` table linked to the version_id. Reverting loads the snapshot back as the active track list.
- **LLM output format**: JSON diff — `{ action: "replace"|"add"|"remove"|"reorder"|"adjust_energy", position, track, explanation }`
- **Context management**: Always send current setlist + last 5 conversation turns. Summarize older turns.
- **Quick commands**: Regex-matched client-side before sending to LLM: "shuffle", "sort by bpm", "reverse" → direct backend operations, no LLM call.
- **Migration**: `006_setlist_versions.sql` — `setlist_versions`, `setlist_conversations`, `setlist_track_snapshots` tables

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] Track replacement modifies only the targeted position
- [ ] Track addition inserts at correct position and shifts others
- [ ] Track removal closes the gap
- [ ] Conversation history persists across multiple turns
- [ ] Setlist versioning creates new version on each modification
- [ ] Undo restores previous version correctly
- [ ] LLM hallucinated tracks reclassified as suggestions
- [ ] Malformed LLM responses handled without crashing
- [ ] Context window managed (summary of old turns)
- [ ] Quick commands handled without LLM call
- [ ] Frontend highlights changes (added/removed tracks)
- [ ] Daily LLM usage limits enforced (refinement counts)
