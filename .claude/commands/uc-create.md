---
description: Interactively create a Cockburn-style use case document
allowed-tools: Read, Glob, Write, AskUserQuestion
---

# Create a Use Case: $ARGUMENTS

You are a **Requirements Architect** creating a fully-dressed Cockburn use case for the Ethnomusicology project. The use case goal is: **$ARGUMENTS**

## Step 1: Determine the Next UC Number

Scan `docs/use-cases/` for existing files matching `uc-*.md`. Increment the highest number.

## Step 2: Validate the Goal

Title MUST be an **Active Verb Phrase Goal**. Rephrase if needed.

## Step 3: Walk Through Each Section Interactively

Work through sections in conversational groups using AskUserQuestion:

### Group 1: Classification
Goal Level, Scope, Priority, Complexity

### Group 2: Actors
Primary Actor, Supporting Actors, Stakeholders & Interests

Common actors: App User, Spotify API, YouTube API, Last.fm API, MusicBrainz API, Audio Player, Database

### Group 3: Conditions
Preconditions, Success Postconditions, Failure Postconditions, Invariants

### Group 4: Main Success Scenario
Build the happy path step by step.

### Group 5: Extensions (What Can Go Wrong)
For EACH MSS step: "What could go wrong?" This is the MOST IMPORTANT section.

### Group 6: Variations
Alternative paths that aren't errors.

### Group 7: Agent Execution Notes
Verification Command, Test File, Dependencies, Blocks, Complexity, Agent Assignment

### Group 8: Acceptance Criteria
Based on postconditions + standard criteria.

## Step 4: Score Completeness

Evaluate against the scoring checklist (minimum 70%).

## Step 5: Write the Use Case File

Write to `docs/use-cases/uc-<NNN>-<slug>.md`.

## Step 6: Next Steps Reminder

1. Review: `/uc-review`
2. Decompose: `/task-decompose <UC-number>`
3. Branch & Worktree
4. Pre-implementation checklist
