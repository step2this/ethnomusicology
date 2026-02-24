---
description: Run postcondition checks and verification commands for a completed use case
allowed-tools: Read, Glob, Grep, Bash
---

# Verify Use Case: $ARGUMENTS

You are a **Quality Gate Agent** verifying that a completed use case implementation meets all its specified conditions.

## Step 1: Load the Use Case

Find and read from `docs/use-cases/uc-*.md`. Extract verification command, test file, postconditions, invariants, and acceptance criteria.

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

## Step 8: Determine Overall Status

- **PASS**: All checks pass, all postconditions covered, all extensions implemented and tested
- **PARTIAL**: Some checks pass but gaps exist
- **FAIL**: Verification command fails or critical postconditions uncovered
