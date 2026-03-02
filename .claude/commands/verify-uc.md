---
description: Run postcondition checks and verification commands for a completed use case or steel thread
allowed-tools: Read, Glob, Grep, Bash
---

# Verify Use Case: $ARGUMENTS

You are a **Quality Gate Agent** verifying that a completed use case implementation meets all its specified conditions.

## Step 1: Load the Artifact

Determine the artifact type from `$ARGUMENTS`:
- If argument matches UC/uc pattern or is a plain number: search `docs/use-cases/uc-*.md`
- If argument matches ST/st pattern: search `docs/steel-threads/st-*.md`

Find and read the artifact. Extract verification command, test file, postconditions, invariants, and acceptance criteria.

For steel threads, also extract: integration assertions, API contract section, cross-cutting references, and "Does NOT Prove" boundary.

## Step 2: Run the Verification Command

Execute the verification command from Agent Execution Notes. Report exit code, output, and duration.

## Step 3: Run the Full Quality Gate

Backend:
- `cd backend && cargo fmt --check`
- `cd backend && cargo clippy -- -D warnings`
- `cd backend && cargo test`

Frontend:
- `cd frontend && flutter analyze`
- `cd frontend && flutter test`

## Step 4: Check Postcondition Coverage

For each postcondition, search test files for verifying assertions. Report COVERED or UNCOVERED.

## Step 5: Check Extension Coverage

For each extension, search for handling code and tests. Report IMPLEMENTED+TESTED, IMPLEMENTED(untested), or MISSING.

## Step 6: Check Acceptance Criteria

Verify each criterion: PASS, FAIL, or NEEDS REVIEW.

## Step 7: Generate Verification Report

Output structured report with overall status, quality gate results, postcondition coverage, invariant coverage, extension coverage, acceptance criteria, gaps, and recommended actions.

## Step 7.5: Integration Assertion Checks (Steel Threads Only)

If the artifact being verified is a steel thread (file in `docs/steel-threads/st-*.md`):

### 7.5a: Check Integration Assertions
For each integration assertion in the steel thread, verify:
- Cross-layer test exists (e.g., HTTP call from test → API → DB → response validated)
- Assertion is tested with REAL dependencies (not mocks at layer boundaries)
- Report each assertion as PROVEN, PARTIALLY PROVEN, or UNPROVEN

### 7.5b: API Contract Conformance
- Load the OpenAPI spec from `docs/api/openapi.yaml`
- For each endpoint in the steel thread's API Contract Section:
  - Verify the endpoint exists in the OpenAPI spec
  - Verify response schemas match what the implementation returns
  - Check Contract Status progression: Draft → Agreed → Implemented → Verified
- Report CONTRACT CONFORMANT or CONTRACT DEVIATION (with details)

### 7.5c: Parent UC Coverage
- For each UC referenced in the Cross-Cutting References section:
  - Check which MSS steps/extensions are now proven by this steel thread
  - Report coverage delta (what was unproven before, what's now proven)

## Step 8: Determine Overall Status

- **PASS**: All checks pass, all postconditions covered, all extensions implemented and tested
- **PARTIAL**: Some checks pass but gaps exist
- **FAIL**: Verification command fails or critical postconditions uncovered
- **INTEGRATION INCOMPLETE**: (Steel threads only) Postconditions pass but integration assertions are unproven
