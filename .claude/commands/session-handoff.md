---
description: Write structured session state to docs/session-handoff.md for cross-session continuity
allowed-tools: Read, Glob, Grep, Bash, Write, AskUserQuestion
---

# Session Handoff: $ARGUMENTS

You are writing a **session handoff document** to preserve context for the next Claude Code session.

## Step 1: Auto-Gather Current State

Collect automatically:

### Git State
- `git status`, `git log --oneline -5`, `git worktree list`, `git branch -a`

### Task State
- Scan `docs/tasks/uc-*-tasks.md` for incomplete tasks
- Check `docs/sprints/current.md` for sprint context

### Test State
- Backend: `cd backend && cargo test 2>&1 | tail -5`
- Frontend: `cd frontend && flutter test 2>&1 | tail -5`

### Active Context
- Read `docs/session-handoff.md` if it exists (previous handoff)
- Read `CLAUDE.md` for current milestone context

## Step 2: Ask the User for Session-Specific Context

If `$ARGUMENTS` is provided, use it. Otherwise ask:
1. What was accomplished this session?
2. Are there any blockers?
3. What should the next session do first?

## Step 3: Write docs/session-handoff.md

Write the handoff document with current state, in-progress tasks, blockers, decisions, and next session instructions.

## Step 4: Validate the Handoff

Verify the file exists and has real data.

## Step 5: Remind the User

> Handoff written to `docs/session-handoff.md`.
> **Next session**: Start by telling Claude to "Read `docs/session-handoff.md` and continue from where the last session left off."
