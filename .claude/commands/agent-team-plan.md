---
description: Design an agent team configuration for implementing a set of tasks
allowed-tools: Read, Glob, Grep, Write
---

# Plan Agent Team: $ARGUMENTS

You are a **Team Architect** designing an agent team configuration to implement tasks for the Ethnomusicology project.

## Input

`$ARGUMENTS` is either:
- A UC number (e.g., "001") — load tasks from `docs/tasks/uc-001-tasks.md`
- A task file path — load directly
- "all" — plan a team for all pending tasks across all use cases
- A sprint name — plan a team for tasks in that sprint

## Step 1: Load Tasks

Use Glob and Read to load the relevant task file(s) from `docs/tasks/`.

If no task files exist, suggest running `/task-decompose` first.

## Step 2: Analyze Task Graph

From the loaded tasks, determine:
- **Parallelizable groups**: Tasks with no dependencies between them that can run concurrently
- **Sequential chains**: Tasks that must happen in order
- **Review gates**: Points where work should be checked before continuing
- **Risk hotspots**: High-risk tasks that need extra attention

## Step 3: Design Team Roles

Based on the task analysis, assign roles from the Implementation Team pattern:

### Standard Roles

| Role | Responsibility | When to Include |
|------|---------------|-----------------|
| **Lead (Implementation Coordinator)** | Manages task list, routes work, monitors conflicts | Always |
| **Builder** | Writes code, follows TDD | When there are implementation tasks |
| **Reviewer** | Reviews code against postconditions, runs checks | When there are 3+ implementation tasks |
| **Documentation** | Updates docs, CLAUDE.md, use case registry | When there are doc-affecting changes |

For larger task sets, consider multiple Builders working in parallel on independent task groups.

### Role Assignment Rules
- Each task gets exactly one **owner** (primary agent)
- High-risk tasks should have the Reviewer assigned as a **gate** (must approve before dependent tasks start)
- Test tasks can be assigned to the Builder (TDD: write test first) or a dedicated Test Writer
- Documentation tasks are batched and assigned after implementation tasks complete

## Step 4: Define Review Gates

Insert review checkpoints in the task flow:

1. **After prerequisite tasks**: Verify foundations before building on them
2. **After each MSS task group**: Verify the happy path works incrementally
3. **After extension tasks**: Verify error handling is correct
4. **After all implementation**: Full integration check
5. **Before marking UC complete**: Run verification command from use case

Each gate specifies:
- What the Reviewer checks
- What commands to run (backend: `cargo test`, `cargo clippy`; frontend: `flutter analyze`, `flutter test`)
- Pass/fail criteria
- What happens on failure (rework task assigned back to Builder)

## Step 5: Generate Team Configuration

Write the team plan to `docs/teams/uc-<NNN>-team.md` (create `docs/teams/` if needed).

## Step 6: Generate Claude Code Team Spawn Commands

Provide the actual commands to set up the team using TeamCreate, TaskCreate, and Task tools.

## Step 7: Report to User

Summarize:
- Team size and roles
- Number of phases and review gates
- Parallelization opportunities
- Ask if the user wants to adjust team composition or task assignments before spawning
