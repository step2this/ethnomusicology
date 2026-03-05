# ST-010 Task Decomposition: Wire Verification Loop and Confidence UI

## Dependency Graph

```
T1 (migration) ──┐
                  ├──► T3 (DB persistence) ──► T5 (integration wiring) ──► T7 (integration tests)
T2 (structs)   ──┘                                    │
                                                       ▼
T4 (verify propagation) ──────────────────────► T5 (integration wiring)
                                                       │
T6 (frontend model + badges) ◄─────────────────────────┘
                  │
                  ▼
T8 (frontend widget tests)
```

## Tasks

---

### T1: Add DB migration for confidence and verification columns
**Module**: `backend/migrations/009_verification.sql`
**Size**: S | **Risk**: Low | **Agent**: Backend Builder
**Depends on**: Nothing
**Files**: `backend/migrations/009_verification.sql` (new)

Add three nullable TEXT columns to `setlist_tracks`:
```sql
ALTER TABLE setlist_tracks ADD COLUMN confidence TEXT;
ALTER TABLE setlist_tracks ADD COLUMN verification_flag TEXT;
ALTER TABLE setlist_tracks ADD COLUMN verification_note TEXT;
```

**Test**: `cargo test` — all existing tests pass (columns are nullable, no breakage).

**IMPORTANT**: After creating this migration, ALL `create_test_pool()` functions in `backend/src/db/mod.rs` AND `backend/tests/*.rs` must include this migration. This is a recurring footgun (ST-006 lesson).

---

### T2: Add verification_flag and verification_note to Rust structs
**Module**: `backend/src/services/setlist.rs`, `backend/src/db/models.rs`
**Size**: S | **Risk**: Low | **Agent**: Backend Builder
**Depends on**: Nothing
**Files**: `backend/src/services/setlist.rs`, `backend/src/db/models.rs`

1. Add to `SetlistTrackResponse`:
   ```rust
   pub verification_flag: Option<String>,
   pub verification_note: Option<String>,
   ```
2. Add to `SetlistTrackRow`:
   ```rust
   pub confidence: Option<String>,
   pub verification_flag: Option<String>,
   pub verification_note: Option<String>,
   ```
3. Update `From<SetlistTrackRow> for SetlistTrackResponse` to map all three fields.
4. Update ALL constructor sites for `SetlistTrackResponse` (grep for `SetlistTrackResponse {`) — add `verification_flag: None, verification_note: None` to each. There are ~5 sites in setlist.rs.

**Test**: `cargo clippy -- -D warnings && cargo test` — compilation clean, all tests pass.

---

### T3: Update DB INSERT/SELECT for new columns
**Module**: `backend/src/db/setlists.rs`
**Size**: S | **Risk**: Medium | **Agent**: Backend Builder
**Depends on**: T1, T2
**Files**: `backend/src/db/setlists.rs`

1. Update `insert_setlist_track()` SQL to include `confidence`, `verification_flag`, `verification_note` columns and bind values.
2. Update `get_setlist_tracks()` SELECT to include `st.confidence`, `st.verification_flag`, `st.verification_note`.
3. The `sqlx::query_as::<_, SetlistTrackRow>` will now map these columns automatically since T2 added them to the struct.

**Test**: `cargo test` — verify existing setlist tests still pass (new columns are nullable, so old INSERT paths that don't provide them will get NULL).

---

### T4: Update verify_setlist() to propagate flag and correction
**Module**: `backend/src/services/setlist.rs`
**Size**: M | **Risk**: Medium | **Agent**: Backend Builder
**Depends on**: T2
**Files**: `backend/src/services/setlist.rs`

The existing `verify_setlist()` function uses `VerificationEntry.flag` and `VerificationEntry.correction` to adjust confidence, but does NOT propagate them to the returned `SetlistTrackResponse`. Fix this:

1. In the verification merge loop, after processing each track:
   ```rust
   track.verification_flag = v.flag.clone();
   track.verification_note = v.correction.clone();
   ```
2. When a flag is `"wrong_artist"`, `"no_such_track"`, or `"constructed_title"`, the verification_note should contain the correction text.
3. When a track is `"replaced"`, set verification_flag to `"replaced"` and verification_note to describe what changed.

**Test**: Unit test in setlist.rs that calls `verify_setlist()` with a mock Claude response containing flags, verifies they appear on the returned tracks.

---

### T5: Wire verify flag through integration boundary (LEAD-OWNED)
**Module**: `backend/src/routes/setlist.rs`, `backend/src/services/setlist.rs`
**Size**: L | **Risk**: High | **Agent**: Lead (integration boundary)
**Depends on**: T3, T4
**Files**: `backend/src/routes/setlist.rs`, `backend/src/services/setlist.rs`

This is the critical integration task. It touches the request/response boundary that connects route handlers to service logic.

1. **Add `verify` field to `GenerateRequest`**:
   ```rust
   #[serde(default)]
   pub verify: Option<bool>,
   ```

2. **Add `verify` field to `GenerateSetlistRequest`**:
   ```rust
   pub verify: bool,
   ```

3. **Update handler mapping**: In `generate_setlist_handler()`, pass `req.verify.unwrap_or(false)` to the service request.

4. **Restructure `generate_setlist_from_request()`** — this is the key change:
   - Current flow: parse LLM response → persist each track to DB → build response
   - New flow: parse LLM response → build track responses in memory → IF verify THEN call `verify_setlist()` → persist all tracks to DB → return response
   - The track persistence loop (currently lines ~467-535) must be split: first collect `SetlistTrackResponse` in memory, then optionally verify, then persist.

5. **Graceful degradation**: If `verify_setlist()` returns an error, log it and proceed with unverified tracks. Add note: "Verification unavailable, confidence scores are self-reported."

**Test**: Route-level tests: POST with `verify: true` (mock Claude for both passes), POST without `verify` (unchanged behavior), POST with `verify: true` when verification fails (graceful degradation).

---

### T6: Frontend model + confidence badge widget
**Module**: `frontend/lib/models/setlist_track.dart`, `frontend/lib/widgets/setlist_track_tile.dart`
**Size**: M | **Risk**: Low | **Agent**: Frontend Builder
**Depends on**: T5 (API contract must be stable)
**Files**: `frontend/lib/models/setlist_track.dart`, `frontend/lib/widgets/setlist_track_tile.dart`

**Model changes** (`setlist_track.dart`):
1. Add `verificationFlag` and `verificationNote` fields (nullable String).
2. Add `fromJson` parsing: `json['verification_flag'] as String?`, `json['verification_note'] as String?`.
3. Add helper: `bool get isFlagged => verificationFlag != null;`

**Widget changes** (`setlist_track_tile.dart`):
1. Add `_confidenceDot()` method following the `_previewStatusDot()` pattern:
   - `confidence == "high"` → green dot (8px)
   - `confidence == "medium"` → yellow/amber dot
   - `confidence == "low"` → orange dot
   - `confidence == null` → no dot (SizedBox.shrink)
2. Place confidence dot in the track tile header row (near position number).
3. If `isFlagged`, show `verificationNote` as a subtitle text below the track title (same style as `transitionNote`). Use a distinct color (e.g., `colorScheme.error` or amber).

**No new tap interaction** — verification notes are shown inline, keeping the widget complexity manageable.

---

### T7: Backend integration tests
**Module**: `backend/tests/verification_integration.rs` (new)
**Size**: M | **Risk**: Medium | **Agent**: Backend Builder
**Depends on**: T5
**Files**: `backend/tests/verification_integration.rs` (new)

**CRITICAL**: The `create_test_pool()` in this file MUST include ALL 9 migrations (001 through 009). This is a recurring footgun.

Tests:
1. **verify=true happy path**: Mock Claude to return generation + verification responses. Assert response has adjusted confidence and verification flags.
2. **verify=false/omitted**: Mock Claude to return generation only. Assert no verification call made. Confidence comes from generation only.
3. **verify=true, verification fails**: Mock Claude to return generation successfully, verification returns error. Assert graceful degradation — tracks returned with original confidence, note about verification unavailable.
4. **DB round-trip**: Generate with verify=true, then GET the setlist. Assert confidence and flags are persisted and returned.
5. **backward compatibility**: Request body without `verify` field parses successfully and behaves as before.

---

### T8: Frontend widget tests
**Module**: `frontend/test/widgets/confidence_badge_test.dart` (new)
**Size**: S | **Risk**: Low | **Agent**: Frontend Builder
**Depends on**: T6
**Files**: `frontend/test/widgets/confidence_badge_test.dart` (new)

Tests:
1. Track with `confidence: "high"` → green dot rendered
2. Track with `confidence: "medium"` → amber dot rendered
3. Track with `confidence: "low"` → orange dot rendered
4. Track with `confidence: null` → no dot
5. Flagged track → verification note text displayed
6. Non-flagged track → no verification note

---

## Summary

| Task | Module | Size | Risk | Agent | Depends On |
|------|--------|------|------|-------|------------|
| T1: DB migration | migrations/ | S | Low | Backend Builder | — |
| T2: Rust structs | services, models | S | Low | Backend Builder | — |
| T3: DB INSERT/SELECT | db/setlists.rs | S | Med | Backend Builder | T1, T2 |
| T4: verify propagation | services/setlist.rs | M | Med | Backend Builder | T2 |
| T5: Integration wiring | routes + services | L | High | **Lead** | T3, T4 |
| T6: Frontend model + badge | models, widgets | M | Low | Frontend Builder | T5 |
| T7: Backend integration tests | tests/ | M | Med | Backend Builder | T5 |
| T8: Frontend widget tests | test/widgets/ | S | Low | Frontend Builder | T6 |

**Total**: 8 tasks, 2 builders + lead
**Parallelism**: T1+T2 parallel → T3+T4 parallel → T5 (lead, sequential) → T6+T7 parallel → T8

## Pre-Implementation Checklist

- [x] Steel thread reviewed (`/uc-review` equivalent: devil's advocate done)
- [x] Devil's advocate findings addressed (all CRITICAL and HIGH fixes applied)
- [ ] Feature branch created
- [ ] Agent team planned (`/agent-team-plan`)
- [ ] Quality gates verified on clean main before starting

## File Ownership (Parallel Safety)

| Builder | Files (exclusive) |
|---------|-------------------|
| Backend Builder | `migrations/009_verification.sql`, `db/models.rs`, `db/setlists.rs`, `tests/verification_integration.rs` |
| Frontend Builder | `frontend/lib/models/setlist_track.dart`, `frontend/lib/widgets/setlist_track_tile.dart`, `frontend/test/widgets/confidence_badge_test.dart` |
| **Lead** | `routes/setlist.rs`, `services/setlist.rs` (integration boundary) |

No overlapping files between builders. Lead owns integration files per CLAUDE.md rules.
