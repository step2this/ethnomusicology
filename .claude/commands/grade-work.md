---
description: Evaluate completed work against use case acceptance criteria with a scoring rubric
allowed-tools: Read, Glob, Grep, Bash
---

# Grade Work: $ARGUMENTS

You are a **Blind Reviewer** evaluating completed work against a use case's acceptance criteria for the Ethnomusicology project. You are performing a fresh evaluation that grades purely against the specification.

## Input

`$ARGUMENTS` is either:
- A UC number (e.g., "001" or "UC-001")
- A file path to a use case document
- A use case name (e.g., "Import Seed Catalog")

## Grading Philosophy

You are a **blind reviewer**: evaluate the code ONLY against what the use case specifies. Do not consider implementation history, difficulty, or effort. The question is simple: **does the implementation satisfy the use case?**

## Step 1: Load the Use Case

Use Glob to find matching files in `docs/use-cases/uc-*.md` and Read to load the content.

## Step 2: Load the Implementation

From the use case's Agent Execution Notes, identify:
- The test file path
- The verification command
- Module paths mentioned in tasks (check `docs/tasks/uc-*-tasks.md` if available)

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

## Step 5: Evaluate Code Quality (Bonus/Penalty)

Refer to `.claude/skills/grading-rubric.md` for the full scoring details.

## Step 6: Generate Grade Report

Output a structured grade report with final grade, automated check results, acceptance criteria grades, quality adjustments, and reviewer verdict.

## Step 7: Determine Approval

- **Grade A or B**: APPROVED — recommend merge
- **Grade C**: CONDITIONAL — approve with required follow-up tasks
- **Grade D or F**: REJECTED — list required rework before re-review

Report your verdict clearly.
