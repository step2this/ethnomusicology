---
description: Set up parallel Claude sessions on worktrees for a sprint
allowed-tools: Read, Glob, Grep, Bash, Write, Edit
---

# Parallel Sprint Setup: $ARGUMENTS

You are a **Sprint Coordinator** setting up parallel Claude Code sessions for the Ethnomusicology project. Each session works on an isolated git worktree with its own feature branch, preventing merge conflicts.

## Input

`$ARGUMENTS` is either:
- A sprint name (e.g., "sprint-1")
- A list of UC numbers (e.g., "001 002 005")
- A single UC with work packages (e.g., "001" — will be split into parallel tracks)
- Empty — analyze pending work and suggest a split

## Step 1: Analyze Available Work

Read the current sprint doc and backlog to understand what needs to be done.

## Step 2: Design the Multi-Track Split

Divide work into tracks that can run concurrently. Follow these rules:

### File Ownership Rules
- **Track 1 (Lead)**: Root config files, `CLAUDE.md`, `docs/`
- **Track 2 (Builder-A)**: Assigned backend source files (no overlap with Track 3)
- **Track 3 (Builder-B)**: Assigned frontend source files (no overlap with Track 2)

### Dependency Rules
- Shared dependencies MUST be added by Track 1 FIRST
- No two tracks may edit the same file

### Merge Order
Tracks merge in dependency order:
1. Track 1 merges first (shared deps, docs)
2. Track 2 and Track 3 merge in any order (no overlap)
3. Final integration verification on main after all merges

## Step 3: Create Worktrees and Branches

For each track, create a git worktree with isolated branch.

## Step 4: Write the Coordination File

Create `docs/sessions.md` with the session plan including track assignments, file ownership matrix, and merge queue.

## Step 5: Generate Session Launch Instructions

Output clear, copy-pastable instructions for the user to spawn each session.

## Step 6: Prep Commit (Track 1)

If any shared dependencies need to be added before parallel work begins, commit them first.

## Step 7: Merge Protocol

After all tracks complete, provide merge commands with verification at each step.

## Rules for Track Agents

1. **Read `docs/sessions.md` first** — understand your role and file ownership
2. **Never edit files outside your ownership**
3. **Run quality gate before committing**
4. **Commit per-task** — not one monolithic commit
5. **Update your status** in `docs/sessions.md`
6. **Rebase on main** if the Lead has pushed shared dependency changes
7. **When your session ends**, run `/session-handoff` to preserve context
