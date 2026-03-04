# Steel Thread: ST-007 Conversational Setlist Refinement

## Classification
- **Goal Level**: 🧵 Thread — thin end-to-end proof of multi-turn conversational refinement of setlists
- **Scope**: Backend API (frontend deferred to follow-on sprint)
- **Priority**: P0 Critical (core DJ experience — refine a setlist with natural language)
- **Complexity**: 🔴 High

## Cross-Cutting References

- **UC-023**: Primary use case — conversational refinement via natural language and quick commands
  - Postcondition 1: User sends natural language refinement message, setlist updated accordingly
  - Postcondition 2: Quick commands (shuffle, sort-by-bpm, reverse, undo) bypass LLM
  - Postcondition 3: Version history maintained with full snapshots
  - Postcondition 4: Revert to previous version creates a new version (non-destructive)
  - Postcondition 5: >50% change warning surfaced to caller
  - Postcondition 6: 20-turn limit enforced per conversation
- **ST-006**: Precondition — enhanced generation pipeline with energy profiles operational
- **ST-005**: Precondition — enrichment pipeline operational (BPM/key/energy on tracks)

This thread proves **multi-turn conversational refinement**: a DJ generates a setlist (ST-006), then iteratively improves it via natural language ("add a harder drop after track 5") or quick commands ("sort by bpm") with full version history and undo support.

## Actors

- **Primary Actor**: App User (DJ)
- **Supporting Actors**:
  - Claude API (Sonnet — natural language refinement interpretation)
  - Database (SQLite — version snapshots, conversation history)
- **Stakeholders & Interests**:
  - DJ User: Wants to iterate on a generated setlist via chat without losing history. Wants instant feedback on simple operations (shuffle, sort) without waiting for LLM.
  - Developer: Clean separation between quick commands (pure functions) and LLM-mediated refinement. Version history as append-only snapshots.
  - Business: Conversational UX is the key differentiator vs static playlist tools.

## Conditions

### Preconditions
1. ST-003 generate/arrange pipeline operational
2. ST-005 enrichment pipeline operational (tracks have BPM/key/energy)
3. ST-006 enhanced generation operational (energy profiles, multi-input)
4. Backend has a valid `ANTHROPIC_API_KEY` environment variable
5. Migration 008 applied (`setlist_versions`, `setlist_version_tracks`, `setlist_conversations` tables)

### Success Postconditions

**Refinement API:**
1. `POST /api/setlists/{id}/refine` accepts `{message: string}` and returns updated setlist
2. Natural language messages are interpreted by Claude via multi-turn `converse()` call
3. Quick commands (shuffle, sort-by-bpm, reverse, undo) bypass LLM — pure function transforms
4. First refinement on a setlist auto-bootstraps version 0 (snapshot of current `setlist_tracks`)
5. Each refinement creates a new version with full track snapshot
6. Response includes `change_warning: true` when >50% of tracks are modified
7. Response includes `version_number` of the created version
8. 20-turn limit enforced — returns `TurnLimitExceeded` error on the 21st message

**Revert API:**
9. `POST /api/setlists/{id}/revert/{version_number}` restores to that version's track snapshot
10. Revert creates a new version (non-destructive — old versions are preserved)
11. Returns 404 if version_number doesn't exist for that setlist

**History API:**
12. `GET /api/setlists/{id}/history` returns all versions and conversation messages in chronological order
13. Each version includes: version_number, created_at, track_count, change_summary
14. Each conversation entry includes: role (user/assistant), content, created_at

**Quick Commands:**
15. `shuffle` — randomizes track order
16. `sort-by-bpm asc/desc` — sorts tracks by BPM field
17. `reverse` — flips track order
18. `undo` — reverts to previous version (version N-1)
19. `revert to version N` — reverts to specific version

**Claude API:**
20. `ClaudeClientTrait::converse(system, messages, model, max_tokens)` method implemented
21. Multi-turn: full conversation history (user + assistant messages) passed in each call
22. Reuses existing `send_with_retries()` infrastructure

### Failure Postconditions
1. Empty message returns 400 `INVALID_REQUEST`
2. Non-existent setlist returns 404 `NOT_FOUND`
3. Turn limit exceeded returns 429 `TURN_LIMIT_EXCEEDED`
4. LLM returns malformed response → retry once; on second failure return 500 `GENERATION_FAILED`
5. Hallucinated track IDs in LLM response are reclassified as suggestions (not hard failures)
6. Invalid version_number for revert returns 404 `VERSION_NOT_FOUND`

### Invariants
1. Version history is append-only — no version is ever deleted or mutated
2. Revert creates a new version; it does not roll back the version counter
3. Quick commands never call the LLM — they are pure, deterministic transforms
4. All LLM calls happen on the backend; API key never exposed
5. Conversation history is preserved even if refinement produces no changes

## API Contract

| Method | Path | Description | Status |
|--------|------|-------------|--------|
| POST | /api/setlists/{id}/refine | Refine setlist via natural language or quick command | Implemented |
| POST | /api/setlists/{id}/revert/{version_number} | Revert to a previous version (creates new version) | Implemented |
| GET | /api/setlists/{id}/history | Get version history and conversation log | Implemented |

### Refine Request Schema
```json
{
  "message": "Remove the slower tracks and add a harder drop after position 5"
}
```

### Refine Response Schema
```json
{
  "setlist_id": "uuid",
  "version_number": 3,
  "tracks": [...],
  "explanation": "Removed tracks at positions 4 and 7 (slower BPM), added suggestion 'Artist - Title' at position 6",
  "change_warning": false,
  "harmonic_flow_score": 0.82,
  "quick_command": false
}
```

### History Response Schema
```json
{
  "setlist_id": "uuid",
  "versions": [
    {
      "version_number": 0,
      "created_at": "...",
      "track_count": 12,
      "change_summary": "Initial snapshot"
    }
  ],
  "conversations": [
    {
      "role": "user",
      "content": "Make it more energetic",
      "created_at": "..."
    },
    {
      "role": "assistant",
      "content": "Replaced 3 tracks with higher-energy alternatives...",
      "created_at": "..."
    }
  ]
}
```

## Main Success Scenario

1. **[Frontend]** User has generated a setlist via ST-006 flow
2. **[Frontend → API]** User sends `POST /api/setlists/{id}/refine` with `{message: "add a harder drop after track 5"}`
3. **[API]** Server validates: non-empty message, setlist exists, turn count < 20
4. **[API]** `parse_quick_command()` returns `None` — this is LLM territory
5. **[API → DB]** Server bootstraps version 0 if none exists (snapshot of current `setlist_tracks`)
6. **[API → DB]** Server loads conversation history + current version tracks
7. **[API]** Server builds refinement system prompt: DJ assistant persona + current track listing + catalog context
8. **[API → LLM]** Server calls `claude.converse(system, messages, model, max_tokens)` with full history
9. **[API]** Server parses LLM response → `LlmRefinementResponse { actions, explanation }`
10. **[API]** Server validates actions: positions in range, track IDs exist or reclassified as suggestions
11. **[API]** Server computes change_warning: >50% of tracks modified?
12. **[API]** Server applies actions to track list in memory
13. **[API → DB]** Server creates new version + track snapshot in single transaction
14. **[API → DB]** Server inserts conversation messages (user + assistant)
15. **[API]** Server recomputes harmonic_flow_score on new track order
16. **[API → Frontend]** Server returns 200 with updated tracks, version_number, explanation, change_warning

### Quick Command Flow (steps 4 onward differ)
4. **[API]** `parse_quick_command("shuffle")` returns `Some(QuickCommand::Shuffle)`
5. **[API → DB]** Bootstrap version 0 if needed
6. **[API]** Apply pure function: `apply_quick_command(Shuffle, tracks)` → shuffled tracks
7. **[API → DB]** Create new version + snapshot (no conversation messages for quick commands)
8. **[API → Frontend]** Return updated tracks, version_number, `quick_command: true`

## Extensions

- **2a. Setlist not found**:
  1. Returns 404 `NOT_FOUND`
  2. Use case fails

- **2b. Empty message**:
  1. Returns 400 `INVALID_REQUEST`: "Message cannot be empty"
  2. Use case fails

- **2c. Turn limit reached (20 messages)**:
  1. Returns 429 `TURN_LIMIT_EXCEEDED`
  2. Use case fails — user must start a new refinement session

- **9a. LLM returns malformed JSON**:
  1. Server retries `converse()` once with error feedback in prompt
  2. If second attempt also malformed, returns 500 `GENERATION_FAILED`

- **9b. LLM hallucinates track IDs not in catalog**:
  1. Actions referencing unknown track IDs are reclassified as suggestions
  2. Refinement proceeds with warning in explanation

- **10a. Actions reference out-of-range positions**:
  1. Invalid actions are dropped; valid actions proceed
  2. Explanation notes dropped actions

- **11a. >50% tracks modified**:
  1. `change_warning: true` in response
  2. Refinement still proceeds — warning is informational

- **Revert — version not found**:
  1. Returns 404 `VERSION_NOT_FOUND`: "No version N found for this setlist"

## Integration Assertions

1. **API → LLM → API**: `converse()` sends full message history in correct alternating user/assistant format; verified by wiremock test checking request body structure
2. **API → DB → API**: Version snapshots roundtrip correctly — revert to version N loads exactly those tracks
3. **API → DB**: Transaction atomicity — version + tracks always created together; never partial
4. **Quick commands**: Pure transforms (Shuffle, SortByBpm, Reverse) produce deterministic results on test fixtures
5. **Turn limit**: 20th refinement succeeds; 21st returns `TURN_LIMIT_EXCEEDED`
6. **Bootstrap**: First refinement auto-creates version 0 matching current `setlist_tracks` exactly
7. **Backward compat**: Existing setlists (no versions) work correctly — bootstrap creates version 0 on first refine

## Does NOT Prove

- **Frontend conversational UI** — Chat widget, message input, version history panel: deferred to follow-on sprint
- **Real-time streaming** — Refinement response is synchronous; streaming LLM output is post-MVP
- **Cross-setlist context** — LLM only sees history of the current setlist's conversation
- **Undo/redo UI** — Backend supports version revert; frontend controls are deferred
- **Branching history** — Linear versioning only; branching (revert + branch) is post-MVP
- **Enrichment of LLM-suggested tracks** — Suggested tracks (not in catalog) are returned as-is without enrichment

## Agent Execution Notes

- **Verification Command**: `cd backend && cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- **Depends On**: ST-003 ✅, ST-005 ✅, ST-006 ✅
- **Blocks**: Frontend chat UI (UC-023 frontend phase)
- **Test File**: `backend/tests/refinement_api_test.rs` + inline unit tests per module
- **Agent Assignment** (completed):
  - Lead: Scaffolding, migration, wiring
  - claude-builder: `api/claude.rs` — `ClaudeClientTrait::converse()` method
  - db-builder: `db/refinement.rs`, `db/models.rs` — version/conversation DB layer
  - quickcmd-builder: `services/quick_commands.rs` — pure function transforms
  - service-builder: `services/refinement.rs` — orchestration, LLM integration
  - route-builder: `routes/refinement.rs`, `tests/refinement_api_test.rs`, `main.rs`

## Implementation Summary

| Component | File | Description |
|-----------|------|-------------|
| Multi-turn conversation | `api/claude.rs` | `converse()` method on `ClaudeClientTrait` + `ClaudeClient` impl |
| DB layer | `db/refinement.rs` | Version/conversation CRUD with transaction support |
| DB models | `db/models.rs` | `SetlistVersionRow`, `VersionTrackRow`, `SetlistConversationRow` |
| Quick commands | `services/quick_commands.rs` | Pure-function transforms: Shuffle, SortByBpm, Reverse, Undo, RevertTo |
| Refinement service | `services/refinement.rs` | `refine_setlist()`, `revert_setlist()`, `get_history()`, LLM action parsing |
| Routes | `routes/refinement.rs` | Three handlers wired into Router |
| Integration tests | `tests/refinement_api_test.rs` | Full round-trip: generate → refine → revert → history |
| Migration | `migrations/008_setlist_versions.sql` | 3 tables, 3 indexes |

## Migration 008

```sql
-- Migration 008: Setlist versioning for conversational refinement

CREATE TABLE IF NOT EXISTS setlist_versions (
    id TEXT PRIMARY KEY,
    setlist_id TEXT NOT NULL REFERENCES setlists(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    change_summary TEXT,
    UNIQUE(setlist_id, version_number)
);

CREATE TABLE IF NOT EXISTS setlist_version_tracks (
    version_id TEXT NOT NULL REFERENCES setlist_versions(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    track_id TEXT NOT NULL,
    is_suggestion INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (version_id, position)
);

CREATE TABLE IF NOT EXISTS setlist_conversations (
    id TEXT PRIMARY KEY,
    setlist_id TEXT NOT NULL REFERENCES setlists(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_setlist_versions_setlist_id ON setlist_versions(setlist_id);
CREATE INDEX IF NOT EXISTS idx_setlist_version_tracks_version_id ON setlist_version_tracks(version_id);
CREATE INDEX IF NOT EXISTS idx_setlist_conversations_setlist_id ON setlist_conversations(setlist_id);
```

## Critic Findings & Resolutions

| # | Severity | Finding | Resolution |
|---|----------|---------|------------|
| 1 | HIGH | `truncate()` panic on multi-byte UTF-8 (explanation field) | Fixed: use `chars().take(N).collect()` instead of byte slicing |
| 2 | LOW | `revert_setlist` not covered by integration test | Accepted debt — noted in known-debt.md |
| 3 | LOW | `get_history` response schema not in openapi.yaml | Accepted debt — openapi.yaml update deferred |
| 4 | LOW | Quick commands don't insert conversation messages | Accepted: intentional — quick commands are silent |
| 5 | LOW | `change_warning` threshold (50%) not configurable | Accepted: hardcoded constant, can parameterize later |

## Status: BACKEND COMPLETE / FRONTEND PENDING

- **Branch**: `feature/st-007-conversational-refinement`
- **Backend Tests**: 328 (272 pre-ST-007 + ~56 new)
- **Frontend**: Deferred — Chat UI widget pending Flutter arch refactor completion
- **Depends on**: ST-003 ✅, ST-005 ✅, ST-006 ✅
