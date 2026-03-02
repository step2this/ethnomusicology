---
description: Grading rubric for evaluating completed use case implementations
---

# Use Case Grading Rubric

Use this rubric when evaluating completed work with `/grade-work` or during blind review of the quality pipeline.

## Grading Scale

| Grade | Range | Meaning | Verdict |
|-------|-------|---------|---------|
| A | 90-100% | Exemplary — exceeds requirements | APPROVED |
| B | 80-89% | Good — meets requirements with minor issues | APPROVED |
| C | 70-79% | Acceptable — meets minimum bar | CONDITIONAL — follow-up tasks required |
| D | 60-69% | Below expectations — significant gaps | REJECTED — rework required |
| F | < 60% | Failing — does not meet requirements | REJECTED — major rework required |

## Scoring Categories

### 1. Postcondition Coverage (30% of total)

How well are success postconditions verified by automated tests?

| Grade | Criteria |
|-------|----------|
| A (100%) | Every postcondition has a dedicated test assertion with clear naming |
| B (80%) | Most postconditions tested; 1-2 verified indirectly through integration |
| C (60%) | At least half of postconditions have explicit tests |
| D (40%) | Some postconditions tested, but major gaps |
| F (0%) | No postcondition-specific tests |

### 2. Extension Handling (25% of total)

How thoroughly are error paths and edge cases implemented?

| Grade | Criteria |
|-------|----------|
| A (100%) | Every extension has code handling AND test coverage |
| B (80%) | All extensions handled in code; most have tests |
| C (60%) | Most extensions handled in code; some tested |
| D (40%) | Some extensions handled; minimal testing |
| F (0%) | Extensions not addressed |

### 3. Invariant Enforcement (15% of total)

Are invariants maintained throughout execution?

| Grade | Criteria |
|-------|----------|
| A (100%) | Invariants enforced by type system or compile-time guarantees |
| B (80%) | Invariants checked at all key points with runtime assertions |
| C (60%) | Invariants checked at entry/exit points |
| D (40%) | Invariants mentioned in comments but not enforced |
| F (0%) | Invariants violated or ignored |

### 4. Code Quality (15% of total)

Does the code meet project standards?

| Grade | Criteria |
|-------|----------|
| A (100%) | Zero warnings, all pub items documented, idiomatic code, proper error types |
| B (80%) | Zero warnings, most pub items documented, minor style issues |
| C (60%) | Few warnings, some documentation, acceptable style |
| D (40%) | Multiple warnings, sparse documentation |
| F (0%) | Fails linting, no docs, panics in production code |

### 5. Test Quality (15% of total)

How good are the tests themselves?

| Grade | Criteria |
|-------|----------|
| A (100%) | Unit + integration + property tests; clear naming; tests document behavior |
| B (80%) | Unit + integration tests; good coverage; clear assertions |
| C (60%) | Integration tests present; basic happy-path coverage |
| D (40%) | Minimal tests; only happy path |
| F (0%) | No tests or tests don't compile |

### 6. Integration Proof (Steel Threads Only — 20% of ST total)

How well does the steel thread prove cross-layer integration?

| Grade | Criteria |
|-------|----------|
| A (100%) | All integration assertions tested end-to-end with real dependencies; API contract validated against actual responses |
| B (80%) | Most integration assertions tested; contract validated; 1-2 minor gaps |
| C (60%) | Integration tests exist but some assertions use mocks at boundaries |
| D (40%) | Minimal cross-layer testing; contract not validated |
| F (0%) | No cross-layer tests; only unit tests with mocked dependencies |

## Steel Thread Scoring Weights

When grading a steel thread (ST-NNN), use these rebalanced weights instead of the UC weights:

| Category | UC Weight | ST Weight |
|----------|-----------|-----------|
| Postcondition Coverage | 30% | 20% |
| Extension Handling | 25% | 15% |
| Invariant Enforcement | 15% | 15% |
| Code Quality | 15% | 15% |
| Test Quality | 15% | 15% |
| **Integration Proof** | — | **20%** |

The grading scale (A-F) and quality adjustments (bonuses/penalties) remain the same as for UCs.

## Quality Adjustments (Bonus/Penalty)

Applied after calculating the base score:

### Bonuses (max +15%)
| Factor | Bonus |
|--------|-------|
| Doc comments on ALL public items | +5% |
| Proper error handling throughout (no panics) | +5% |
| Property-based tests for serialization round-trips | +5% |

### Penalties (no limit)
| Factor | Penalty |
|--------|---------|
| `unwrap()` in Rust production code | -10% per occurrence (max -30%) |
| Missing doc comments on public functions | -5% |
| Dead code or unused imports | -5% |
| Panics on recoverable errors | -15% |
| Hardcoded values that should be configurable | -5% |
| `unsafe` without justification comment | -10% |

### Steel Thread Adjustments

| Factor | Adjustment |
|--------|------------|
| API contract held through implementation without changes | +5% bonus |
| API contract broke during implementation (required mid-sprint update) | -10% penalty |
| Mocking at layer boundaries in integration tests | -15% penalty |
| All integration assertions proven with real dependencies | +5% bonus |

## Completeness Scoring (for Use Case Documents)

Used by `/uc-review` and `/uc-create` to score use case document quality:

| Criterion | Weight |
|-----------|--------|
| Title is active verb phrase | 5% |
| All 4 classification fields filled | 5% |
| Primary actor identified | 5% |
| At least 2 stakeholder interests | 5% |
| At least 2 preconditions | 10% |
| At least 2 success postconditions | 10% |
| At least 1 failure postcondition | 5% |
| At least 1 invariant | 5% |
| MSS has 5+ steps | 10% |
| Extensions cover 50%+ of MSS steps | 15% |
| All postconditions are testable | 10% |
| Verification command specified | 5% |
| Dependencies listed | 5% |
| Acceptance criteria present | 5% |

**Minimum bar**: 70% — use cases below this need more work before implementation.

## Common Extension Checklist

When reviewing extension coverage, check that these common failure modes are addressed:

### Network / API
- [ ] API rate limit exceeded
- [ ] API key invalid or expired
- [ ] Timeout on response
- [ ] Service unavailable (Spotify, YouTube, Last.fm, MusicBrainz)

### Data
- [ ] Empty input
- [ ] Input exceeds size limit
- [ ] Malformed/corrupted data
- [ ] Unexpected data type

### State
- [ ] Precondition not met at runtime
- [ ] Concurrent modification / race condition
- [ ] Resource exhaustion (memory, disk, connections)
- [ ] Stale state / cache invalidation

### Auth
- [ ] Invalid credentials / expired session
- [ ] OAuth token refresh needed
- [ ] Unauthorized access attempt

### UX
- [ ] Operation takes too long (user waits)
- [ ] Partial success (some items processed, some failed)
- [ ] Conflicting user action
