---
description: Evaluate completed work against use case or steel thread acceptance criteria with a scoring rubric
allowed-tools: Read, Glob, Grep, Bash
---

# Grade Work: $ARGUMENTS

You are a **Blind Reviewer** evaluating completed work against a use case or steel thread's acceptance criteria for the Ethnomusicology project. You are performing a fresh evaluation that grades purely against the specification.

## Input

`$ARGUMENTS` is either:
- A UC number (e.g., "001" or "UC-001")
- A ST number (e.g., "ST-001" or "st-001")
- A file path to a use case or steel thread document
- A use case or steel thread name

## Grading Philosophy

You are a **blind reviewer**: evaluate the code ONLY against what the use case specifies. Do not consider implementation history, difficulty, or effort. The question is simple: **does the implementation satisfy the use case?**

## Step 1: Load the Artifact

Determine the artifact type from `$ARGUMENTS`:
- If argument matches UC/uc pattern or is a plain number: search `docs/use-cases/uc-*.md`
- If argument matches ST/st pattern: search `docs/steel-threads/st-*.md`

Use Glob to find matching files and Read to load the content.

**Identify artifact type** — this determines which weight table to use:
- UC artifacts: use standard UC scoring weights from the grading rubric
- ST artifacts: use Steel Thread scoring weights (includes Integration Proof at 20%)

## Step 2: Load the Implementation

From the use case's Agent Execution Notes, identify:
- The test file path
- The verification command
- Module paths mentioned in tasks (check `docs/tasks/uc-*-tasks.md` or `docs/tasks/st-*-tasks.md` if available)

Use Glob and Read to find and load all relevant implementation files.

## Step 3: Run Automated Checks

Execute in order:

1. **Backend format check**: `cd backend && cargo fmt --check`
2. **Backend lint check**: `cd backend && cargo clippy -- -D warnings`
3. **Backend tests**: `cd backend && cargo test`
4. **Frontend analyze**: `cd frontend && flutter analyze`
5. **Frontend tests**: `cd frontend && flutter test`
6. **Specific verification**: Run the use case's verification command

Record pass/fail for each.

## Step 4: Grade Each Acceptance Criterion

For each acceptance criterion in the use case, assign a grade (A-F, 0-100%).

## Step 4.5: Grade Integration Proof (Steel Threads Only)

If the artifact is a steel thread, evaluate the Integration Proof category (20% of ST total):

1. **Integration Assertions**: For each integration assertion, verify a cross-layer test exists that proves it with real dependencies (not mocks). Grade A-F per the rubric.
2. **API Contract Validation**: Check that actual API responses match the OpenAPI spec in `docs/api/openapi.yaml`. Grade A-F.
3. **Apply ST-specific adjustments** from the grading rubric:
   - +5% if API contract held through implementation without changes
   - -10% if API contract broke during implementation
   - -15% for mocking at layer boundaries in integration tests
   - +5% if all integration assertions proven with real dependencies

## Step 5: Evaluate Code Quality (Bonus/Penalty)

Refer to `.claude/skills/grading-rubric.md` for the full scoring details. For steel threads, use the Steel Thread Scoring Weights table and Steel Thread Adjustments from the grading rubric.

## Step 6: Generate Grade Report

Output a structured grade report with final grade, automated check results, acceptance criteria grades, quality adjustments, and reviewer verdict.

## Step 7: Determine Approval

- **Grade A or B**: APPROVED — recommend merge
- **Grade C**: CONDITIONAL — approve with required follow-up tasks
- **Grade D or F**: REJECTED — list required rework before re-review

Report your verdict clearly.
