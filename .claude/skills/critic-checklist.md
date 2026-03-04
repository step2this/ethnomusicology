# Critic Review Checklist

Reference for **Critic Agents** running fresh-context code reviews. The critic reads the diff cold — it has NOT watched the code being written. This breaks the builder's blind spot.

## Purpose

The critic is the last quality gate before `/verify-uc`. It catches what the builder missed because the builder was too close to the code. ST-003 proved that in-session review is theater — only a fresh-context agent reads with genuinely new eyes.

## Inputs

Before starting, gather:
1. **Diff**: `git diff main...HEAD` — the full set of changes
2. **Plan**: The task doc (`docs/tasks/st-NNN-tasks.md`) or use case (`docs/use-cases/uc-NNN.md`)
3. **Test output**: `cd backend && cargo test 2>&1` and `cd frontend && flutter test 2>&1`
4. **Quality gates**: Confirm all pass before reviewing logic

```bash
cd backend && cargo fmt --check
cd backend && cargo clippy -- -D warnings
cd backend && cargo test
cd frontend && flutter analyze
cd frontend && flutter test
```

If any quality gate fails, the review stops — builder must fix before critic proceeds.

## Checklist

### 1. Plan Compliance (HIGHEST PRIORITY)

> ST-005 lesson: `auto-enrich` was planned but dropped during task decomposition. No one caught it until critic review. ST-006 lesson: `compute_seed_match_count` was defined and tested but never called — postcondition 13 shipped unmet.

- [ ] Every postcondition in the plan has a corresponding implementation
- [ ] Every postcondition has at least one test (unit or integration)
- [ ] API endpoints in the plan match the actual routes wired in `main.rs`
- [ ] All task items (T1, T2, ...) from the task doc are implemented — none silently dropped
- [ ] Response schemas match what the plan specifies (field names, types, nesting)

**How to check**: Read each postcondition. Find the code that satisfies it. If you can't find it, it's missing.

### 2. Dead Code and Unused Imports

- [ ] No functions defined but never called
- [ ] No structs/enums defined but never used
- [ ] No `use` statements that reference unused items
- [ ] No commented-out code blocks left in (unless explicitly marked TODO with ticket)
- [ ] `cargo clippy -- -D warnings` passes (this catches most unused items automatically)

### 3. Naming Consistency

- [ ] New names follow existing conventions (snake_case Rust, camelCase Dart)
- [ ] New error codes follow the `SCREAMING_SNAKE_CASE` pattern in existing error types
- [ ] New DB table/column names follow `snake_case` pattern of existing schema
- [ ] New route paths follow the `/api/resource/{id}/action` pattern
- [ ] Struct field names match the JSON keys in the API contract (or use `#[serde(rename)]`)

### 4. Security

- [ ] No `unwrap()` in production code paths (only in tests and one-off scripts)
  - Exception: `expect()` with a message is acceptable for truly infallible cases
- [ ] SQL queries use parameterized binds (`.bind(value)`) — no string interpolation
- [ ] No secrets, tokens, or credentials in code or comments
- [ ] User input is validated at the route handler before reaching service layer
- [ ] Error messages don't leak internal implementation details to callers

### 5. Error Handling

- [ ] Errors propagate with `?` — no swallowed errors (silent `let _ = ...` on fallible ops)
- [ ] All error variants in custom error enums are actually reachable
- [ ] HTTP status codes match error semantics: 400 (bad input), 404 (not found), 429 (rate limit), 500 (server fault)
- [ ] Retry logic is present where external APIs are called (Claude, Spotify, Deezer)
- [ ] Background tasks (`tokio::spawn`) log errors — spawn errors never silently vanish

### 6. Transaction Safety

- [ ] Operations that must be atomic (e.g., create version + create version tracks) use a DB transaction
- [ ] Transactions commit/rollback on error — no partial writes
- [ ] Bulk inserts within a transaction use a single `tx` parameter (not a new pool connection)

### 7. Test Gaps

- [ ] Happy path is tested for every new endpoint
- [ ] Error paths are tested: 400, 404, and the most likely 500 scenario
- [ ] Edge cases: empty inputs, boundary values (e.g., turn limit at 19 vs 20 vs 21)
- [ ] Integration tests create their own DB state — no test depends on another test's data
- [ ] `create_test_pool()` in integration tests runs all migrations (`sqlx::migrate!()`)
  - Check BOTH `db/mod.rs` and `tests/*.rs` — they have separate pool helpers (recurring trap)
- [ ] Mock objects (MockClaude, etc.) are used in unit tests; real DB pool in integration tests

### 8. API Contract Compliance

- [ ] Request shape matches `docs/api/openapi.yaml` (or openapi.yaml is updated)
- [ ] Response shape matches openapi.yaml (field names, types, optional vs required)
- [ ] New endpoints are added to openapi.yaml
- [ ] Backward compatibility maintained: new optional fields use `#[serde(default)]` or `Option<T>`
- [ ] HTTP method + path combinations are correct (POST vs PUT, path params vs query params)

### 9. LLM Prompt Quality (if applicable)

- [ ] System prompt has clear persona and task description
- [ ] Output format is specified explicitly (JSON schema, field names, constraints)
- [ ] Prompt handles edge cases: empty catalog, missing BPM/key, all-suggestion setlists
- [ ] Prompt includes enough context for the model to act (current tracks, catalog, history)
- [ ] Conversation history is passed in correct alternating user/assistant order

### 10. Data Integrity

- [ ] Action validation rejects out-of-range positions before applying to data
- [ ] String fields are length-capped before DB insert (prevents unbounded growth)
- [ ] UTF-8 truncation uses `chars().take(N).collect()` not byte slicing (panic risk)
- [ ] Numeric casts (`as usize`, `as i32`) are checked for overflow on realistic inputs
- [ ] Foreign key references are validated before insert (or rely on DB FK constraint)

### 11. Migration Safety (if new migration)

- [ ] Migration is idempotent (`CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`)
- [ ] All `TrackRow`/`SetlistRow` consumers updated when columns are added — search for `SELECT *` queries and `FromRow` derives
- [ ] Both `db/mod.rs` `create_test_pool()` AND `tests/*.rs` `create_test_pool()` include the new migration
- [ ] Migration file name follows `NNN_description.sql` convention and number is sequential

## Classification

| Severity | Definition | Examples |
|----------|-----------|---------|
| **CRITICAL** | Blocks the use case from working or causes data corruption | Missing route wiring (endpoint returns 404), missing transaction (partial write possible), panic in production code path |
| **HIGH** | Postcondition unmet, security issue, or will cause test failures in CI | Planned feature not implemented, `unwrap()` on external API response, DB query with no error handling |
| **LOW** | Code quality, naming, or minor gaps that don't block functionality | Unused import, naming inconsistency, missing edge case test for non-critical path |

## Output Format

```
## Critic Review — ST-NNN

### Quality Gates
- [x] cargo fmt --check
- [x] cargo clippy -- -D warnings
- [x] cargo test (328 passed)
- [x] flutter analyze
- [x] flutter test (104 passed)

### Plan Compliance
- [x] All postconditions implemented
- [x] All task items present

### Findings

| # | Severity | File:Line | Finding | Recommended Fix |
|---|----------|-----------|---------|-----------------|
| 1 | HIGH | backend/src/services/refinement.rs:147 | `truncate()` called on String at byte boundary — panics on multi-byte UTF-8 | Use `s.chars().take(200).collect::<String>()` |
| 2 | LOW | backend/src/db/refinement.rs:23 | `#[allow(dead_code)]` on `get_version_by_number` — is it called anywhere? | Remove if truly unused, or call from revert handler |

### Verdict

**APPROVE WITH FIXES** — Finding #1 (HIGH) must be resolved before merge. Finding #2 is accepted debt.
```

Verdict options:
- **APPROVE** — No issues, or only LOW items that are accepted debt
- **APPROVE WITH FIXES** — HIGH items must be fixed; LOW items may be accepted
- **REJECT** — CRITICAL items found; full re-review required after fixes

## Why Fresh Context Matters

> "An in-session reviewer that watches code being written suffers the same blind spots as the builder. ST-003 proved this — self-review missed wrong package names, invalid base URLs, and assertion mismatches that a cold read catches instantly." — ST-003 retro

The critic's value comes from **not knowing what was intended**. When you read `compute_seed_match_count()` defined but never called, you don't know "oh the builder meant to call it but forgot." You just see: this function exists but is never called. That cold observation is the finding.

## References

- `CLAUDE.md` — "Code Review: Critic Agent" section
- `.claude/agents/implementation-team.md` — Teammate 2: Critic role definition
- `.claude/rules/lessons-learned.md` — ST-003, ST-005, ST-006 lessons that shaped this checklist
- `.claude/rules/known-debt.md` — accepted critic findings not requiring immediate fix
