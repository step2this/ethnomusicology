# ST-007: Conversational Setlist Refinement — Backend Tasks

## Context

Backend-only implementation of UC-023 (conversational refinement). Frontend is blocked by arch refactor in another Claude session. This builds the API layer that the frontend will consume later.

Detailed plan: `.claude/plans/rustling-orbiting-dawn.md`

## Phase 0: Lead Setup (no agent)

### T0: Migration + module scaffolding
- Create `backend/migrations/008_setlist_versions.sql` (3 tables, 3 indexes)
- Add `pub mod refinement;` to `db/mod.rs`, `services/mod.rs`, `routes/mod.rs`
- Add `pub mod quick_commands;` to `services/mod.rs`
- Create empty module files so `cargo check` passes
- Verify: `cargo check`

## Phase 1: Parallel Builders (3 agents, worktrees)

### T1: Claude API multi-turn conversation
**Builder:** claude-builder
**Files:** `src/api/claude.rs`
- Add `ConversationMessage { role: String, content: String }` struct
- Add `converse()` with default impl (unimplemented!) to `ClaudeClientTrait`
- Implement `converse()` on `ClaudeClient` — builds Messages API body with full messages array, reuses `send_with_retries()`
- Wiremock test: verify request body has messages array with alternating user/assistant roles
- Verify: `cargo test` (all existing tests still pass — default method means mocks compile unchanged)

### T2: Database layer for versions, conversations, snapshots
**Builder:** db-builder
**Files:** `src/db/refinement.rs`, `src/db/models.rs`
- Add 3 row structs to `models.rs`: `SetlistVersionRow`, `VersionTrackRow`, `SetlistConversationRow`
- Implement DB ops in `refinement.rs`:
  - `insert_version(tx, row)` — within transaction
  - `insert_version_tracks(tx, version_id, tracks)` — bulk insert within transaction
  - `insert_conversation(pool, row)` — user + assistant messages
  - `get_versions_by_setlist(pool, setlist_id)` → ordered by version_number
  - `get_latest_version(pool, setlist_id)` → max version_number
  - `get_version_tracks(pool, version_id)` → ordered by position
  - `get_version_by_number(pool, setlist_id, version_number)` → single version
  - `get_conversations_by_setlist(pool, setlist_id)` → ordered by created_at
- Unit tests: insert/get roundtrips, unique constraint, latest version, empty setlist
- Verify: `cargo test`

### T3: Quick commands (pure functions)
**Builder:** quickcmd-builder
**Files:** `src/services/quick_commands.rs`
- `QuickCommand` enum: `Shuffle`, `SortByBpm { ascending: bool }`, `Reverse`, `Undo`, `RevertTo(i32)`
- `parse_quick_command(message: &str) -> Option<QuickCommand>` — regex matching
- `apply_quick_command(command: QuickCommand, tracks: Vec<VersionTrackRow>) -> Vec<VersionTrackRow>`
  - Shuffle: randomize positions
  - SortByBpm: sort by bpm field (ascending/descending)
  - Reverse: flip order
  - Undo/RevertTo: signal only (no track mutation — caller handles DB revert)
- Unit tests: each command, non-matching input returns None, edge cases (empty list, missing BPM)
- Verify: `cargo test`

## Phase 2: Sequential Builder (depends on Phase 1)

### T4: Refinement service
**Builder:** service-builder
**Files:** `src/services/refinement.rs`
- `RefinementError` enum with `IntoResponse` impl (NotFound, InvalidRequest, LlmError, Timeout, ServiceBusy, GenerationFailed, Database, TurnLimitExceeded)
- LLM types: `LlmRefinementResponse { actions: Vec<LlmAction>, explanation: String }`
- `LlmAction` enum (tagged): Replace, Add, Remove, Reorder
- `refine_setlist(pool, claude, setlist_id, user_id, message)`:
  1. Load setlist (404 if not found)
  2. Check empty message (400) and turn limit (20 max)
  3. Try `parse_quick_command(message)` — if quick cmd, handle without LLM
  4. Bootstrap version 0 if none exists (snapshot current setlist_tracks)
  5. Load conversation history + current version tracks
  6. Build refinement system prompt (tracks + catalog)
  7. Call `claude.converse(system, messages, model, max_tokens)`
  8. Parse response → `LlmRefinementResponse` (retry once on failure)
  9. Validate actions (positions in range, track_ids exist or reclassify)
  10. Compute change warning (>50% tracks modified)
  11. Apply actions to track list in memory
  12. Create new version + snapshot in single transaction
  13. Insert conversation messages (user + assistant)
  14. Recompute harmonic_flow_score
  15. Return response
- `revert_setlist(pool, setlist_id, target_version_number)` — load target snapshot, create new version
- `get_history(pool, setlist_id)` — versions + conversations
- `build_refinement_system_prompt(tracks, catalog)` — DJ assistant persona
- `parse_refinement_response(text)` → `LlmRefinementResponse`
- `apply_actions(tracks, actions)` → modified tracks
- `validate_actions(actions, track_count, catalog)` → validated actions
- Tests with MockClaude: replace/add/remove/reorder, quick command bypass, bootstrap, hallucinated track, malformed retry, change warning, revert, history
- Verify: `cargo test`

## Phase 3: Sequential Builder (depends on Phase 2)

### T5: Routes + wiring + integration tests
**Builder:** route-builder
**Files:** `src/routes/refinement.rs`, `tests/refinement_api_test.rs`, `src/main.rs`
- `RefinementRouteState { pool, claude }` (same shape as SetlistRouteState — could reuse)
- `refine_handler`: POST /setlists/{id}/refine
- `revert_handler`: POST /setlists/{id}/revert/{version_number}
- `history_handler`: GET /setlists/{id}/history
- `refinement_router(state)` → Router
- Wire into main.rs: `.nest("/api", refinement_router(state))`
- Integration tests: full round-trip (generate → refine → verify), revert, quick command, history, error cases
- Verify: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`

## Phase 4: Critic Review
- Fresh context opus agent reads `git diff main...HEAD` cold
- Reviews all new code for: bugs, dead code, test gaps, naming, security, plan compliance
- Fixes applied before commit

## Agent Assignment Summary

| Phase | Builder | Model | Isolation | Files |
|-------|---------|-------|-----------|-------|
| 0 | Lead | — | worktree | migration, mod.rs files |
| 1 | claude-builder | sonnet | worktree | `api/claude.rs` |
| 1 | db-builder | sonnet | worktree | `db/refinement.rs`, `db/models.rs` |
| 1 | quickcmd-builder | sonnet | worktree | `services/quick_commands.rs` |
| 2 | service-builder | sonnet | worktree | `services/refinement.rs` |
| 3 | route-builder | sonnet | worktree | `routes/refinement.rs`, `tests/refinement_api_test.rs`, `main.rs` |
| 4 | critic | opus | — | read-only review |

## Verification

1. `cargo fmt --check` — formatted
2. `cargo clippy -- -D warnings` — no warnings
3. `cargo test` — all pass (~312: 272 existing + ~40 new)
4. Manual curl: generate → refine → revert → history
5. `/ralph-loop` for CI after PR
