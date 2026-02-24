# Claude Code Best Practices: From Blank Repo to Production-Ready Monorepo

**Presentation notes for a 30-minute talk on requirements-driven AI development**

*Based on building the Ethnomusicology project — a music playlist app for Muslim families planning occasions (Nikah, Eid, Mawlid) featuring African and Middle Eastern musical traditions.*

---

## Table of Contents

1. [The Forge Pattern](#1-the-forge-pattern)
2. [Multi-Agent Team Architecture](#2-multi-agent-team-architecture)
3. [Prompt Engineering Best Practices](#3-prompt-engineering-best-practices)
4. [Tooling Setup](#4-tooling-setup)
5. [Working Model: Sprint 0 Case Study](#5-working-model-sprint-0-case-study)
6. [Key Principles](#6-key-principles)
7. [Anti-Patterns to Avoid](#7-anti-patterns-to-avoid)

---

## 1. The Forge Pattern

### What Is "The Forge"?

The Forge is a `.claude/` directory that acts as a **requirements-driven development framework** for AI agents. It's not just configuration — it's the institutional memory, quality enforcement, and process guardrails that make Claude Code behave like a disciplined engineering team rather than a code-generating autocomplete.

```
.claude/
├── agents/                    # Team definitions (who does what)
│   ├── implementation-team.md # Builder, Reviewer, Documentation roles
│   ├── requirements-team.md   # Architect, Devil's Advocate, Test Designer
│   └── ux-team.md            # UX Architect, Interaction Designer, A11y, Visual QA
├── commands/                  # Slash commands (workflow automation)
│   ├── uc-create.md          # Interactive Cockburn use case creation
│   ├── uc-review.md          # Use case gap analysis
│   ├── task-decompose.md     # Break UCs into implementable tasks
│   ├── wireframe.md          # ASCII wireframe generation
│   ├── grade-work.md         # Blind review against acceptance criteria
│   ├── session-handoff.md    # Cross-session continuity
│   └── ...13 commands total
├── skills/                    # Reusable reference documents
│   ├── cockburn-template.md  # The canonical use case structure
│   ├── grading-rubric.md     # A-F scoring with weighted categories
│   ├── design-system.md      # Full design token reference
│   └── pre-implementation-checklist.md
└── settings.json             # Hooks for automated quality enforcement
```

> **Speaker note**: Think of the Forge as the `.github/` directory, but for AI agent behavior. It's checked into the repo and versioned alongside the code.

### The Self-Reinforcing Quality Loop

The Forge creates a development cycle where each step feeds the next:

```
┌─────────────┐     ┌──────────────┐     ┌───────────────┐
│  /uc-create  │────→│  /uc-review   │────→│/task-decompose │
│ (Write spec) │     │ (Find gaps)   │     │ (Plan work)    │
└─────────────┘     └──────────────┘     └───────┬───────┘
       ↑                                          │
       │                                          ▼
┌──────┴──────┐     ┌──────────────┐     ┌───────────────┐
│/retrospective│←────│ /grade-work  │←────│  Implement    │
│(Learn & fix) │     │ (Blind review)│     │ (Agent teams) │
└─────────────┘     └──────────────┘     └───────────────┘
                            ↑
                    ┌───────┴───────┐
                    │  /verify-uc    │
                    │(Check postcon.)│
                    └───────────────┘
```

Each command in the loop produces artifacts that the next command consumes:

| Step | Command | Input | Output |
|------|---------|-------|--------|
| 1 | `/uc-create` | Goal description | `docs/use-cases/uc-NNN-slug.md` |
| 2 | `/uc-review` | Use case doc | Gap analysis, completeness score |
| 3 | `/task-decompose` | Reviewed use case | `docs/tasks/uc-NNN-tasks.md` |
| 4 | `/agent-team-plan` | Task file | `docs/teams/uc-NNN-team.md` |
| 5 | Implementation | Task list + team plan | Working code on feature branch |
| 6 | `/verify-uc` | Use case postconditions | Pass/fail verification |
| 7 | `/grade-work` | Use case + implementation | Grade report (A-F) |
| 8 | `/retrospective` | All of the above | Process improvements |

### Why Cockburn Use Cases Work So Well with AI Agents

Alistair Cockburn's fully-dressed use case format was designed for humans in the early 2000s. It turns out to be *even more valuable* for AI agents, for reasons Cockburn never anticipated:

**1. Structured, parseable format**

The template has named sections with predictable content. An AI agent can mechanically extract preconditions, find postconditions to verify, and walk through the Main Success Scenario step by step. No ambiguity about what "done" means.

```markdown
## Conditions
- **Preconditions** (must be true before starting):
  1. Backend server is running
  2. Spotify API credentials are configured in .env
- **Success Postconditions** (true when done right):
  1. All 54 tracks from "Salamic Vibes" exist in the database
  2. Each track record contains: title, artist(s), album, duration, Spotify URI
- **Failure Postconditions** (true when it fails gracefully):
  1. Partial import is rolled back — no orphan records
  2. Error details are logged with Spotify API response codes
```

**2. Testable postconditions map directly to assertions**

Every postcondition in a Cockburn use case becomes a test assertion. The agent doesn't have to *infer* what to test — the spec tells it explicitly:

```rust
// Postcondition: "All 54 tracks from 'Salamic Vibes' exist in the database"
#[tokio::test]
async fn test_all_tracks_imported() {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tracks")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(count, 54);
}
```

**3. Extensions map to error handling**

The Extensions section ("What Can Go Wrong") at each step is essentially a **pre-written error handling specification**. Without it, AI agents tend to implement only the happy path and leave error cases as `todo!()` or `unwrap()`:

```markdown
## Extensions
- **3a. Spotify API returns 429 (rate limit)**:
  1. System waits for Retry-After header duration
  2. System retries the request (max 3 retries)
  3. If still failing, use case fails with rate limit error
- **3b. Spotify API returns 401 (token expired)**:
  1. System refreshes the OAuth token
  2. Returns to step 3 with new token
```

**4. Goal levels map to agent delegation**

Cockburn's three goal levels naturally correspond to agent team structure:

| Goal Level | Icon | Agent Role |
|-----------|------|------------|
| Summary (☁️) | High-level business goal | Team Lead coordinates |
| User Goal (🌊) | What user sits down to do | Teammate implements |
| Subfunction (🐟) | Step within larger goal | Subagent handles |

### The Key Insight

> **"AI agents need structured specs more than humans do, not less."**

A senior engineer can look at a vague ticket like "add Spotify import" and fill in dozens of implicit requirements from experience. An AI agent will implement *exactly what you ask for* — nothing more, nothing less. If you don't specify error handling, you won't get error handling. If you don't specify rollback behavior, partial failures will corrupt your data.

The Cockburn template forces you to think through:
- What must be true before we start? (Preconditions → setup code)
- What must be true when we're done? (Postconditions → test assertions)
- What must stay true throughout? (Invariants → continuous assertions)
- What can go wrong at each step? (Extensions → error handling)
- How do we know it worked? (Verification command → CI/CD)

This level of spec rigor would feel like overkill for a human team. For AI agents, it's the minimum viable spec.

---

## 2. Multi-Agent Team Architecture

### The Team Patterns

The Forge defines three specialized agent teams, each with distinct roles and workflows:

#### Requirements Team
*Purpose: Create, review, and refine specifications before any code is written.*

```
Requirements Team
├── Lead: Requirements Architect
│   └── Owns the use case document, asks clarifying questions, scores completeness
├── Devil's Advocate
│   └── Finds missing extensions, challenges assumptions, pushes back on vague postconditions
├── Test Designer
│   └── Writes test skeletons from postconditions, identifies untestable requirements
└── Architecture Scout
    └── Researches feasibility, maps to modules, estimates complexity
```

#### Implementation Team
*Purpose: Build, review, and document features from use cases.*

```
Implementation Team
├── Lead: Implementation Coordinator
│   └── Manages task list, routes work, resolves blockers, runs final verification
├── Builder (can have multiple)
│   └── Writes code following TDD, commits per completed use case
├── Reviewer
│   └── Reviews against postconditions, runs quality gates, approves or requests changes
└── Documentation
    └── Updates README, CLAUDE.md, use case registry, sprint tracker
```

#### UX Team
*Purpose: Design, review, and validate user experience.*

```
UX Team
├── Lead: UX Architect
│   └── Owns design system, creates wireframes, defines design tokens
├── Interaction Designer
│   └── State diagrams, animation specs, gesture definitions, flow design
├── Accessibility & Localization Specialist
│   └── WCAG 2.1 AA compliance, RTL/Arabic layout, screen reader semantics
└── Visual QA
    └── Golden tests, responsive breakpoints, theme consistency, design token compliance
```

### Why the Devil's Advocate Agent Matters

The Devil's Advocate role is one of the most valuable innovations in the team structure. Here's why:

**A single agent has confirmation bias.** When one agent writes a use case and then implements it, it tends to assume its own spec is correct and complete. It won't question its own assumptions.

**The Devil's Advocate is structurally adversarial.** Its entire job is to find what's missing:

> For EVERY step in the Main Success Scenario, asks "what if this fails?"
> Challenges assumptions in preconditions ("is this really guaranteed?")
> Looks for implicit dependencies that aren't documented
> Pushes back on vague postconditions ("how would you test this?")

In the tech stack decision phase, this pattern was used as a debate format:

| Concern | Mitigation |
|---------|-----------|
| Spotify API is shrinking | Design around source-agnostic internal catalog |
| Occasion classification can't be automated | Accept it. 54 tracks is small enough to curate by hand |
| Cultural sensitivity | Taxonomy includes sacred/devotional flag |
| "Muslim Africa" is not a monolith | Granular regional/tradition taxonomy |
| Cold start: only 54 tracks | Frame as "curated collection," use Last.fm to expand |

This Devil's Advocate table caught real issues that would have become bugs or architectural debt later.

### The Lead/Builder/Reviewer Pattern

This mimics how a real engineering team operates:

```
Lead (coordinates, doesn't code)
  │
  ├──assigns──→ Builder 1 (backend routes)
  ├──assigns──→ Builder 2 (frontend screens)     ← parallel execution
  ├──assigns──→ Builder 3 (database schema)
  │
  └──triggers──→ Reviewer (after each task group)
                    │
                    ├── Runs: cargo fmt --check
                    ├── Runs: cargo clippy -- -D warnings
                    ├── Runs: cargo test
                    ├── Checks: postcondition coverage
                    └── Verdict: APPROVE or REQUEST CHANGES
```

**Key rule: The lead coordinates but does not implement directly.** This prevents the lead from getting lost in implementation details and losing track of the overall task graph.

### Team Coordination with Claude Code Tools

Teams coordinate using three mechanisms:

**1. TeamCreate — Set up the team**
```
TeamCreate: team_name="sprint-0", description="Project scaffolding"
```

**2. Task tools — Shared work queue**
```
TaskCreate: subject="Scaffold Rust backend", description="...", activeForm="Scaffolding backend"
TaskUpdate: taskId="1", status="in_progress", owner="backend-builder"
TaskUpdate: taskId="1", status="completed"
TaskList: // check what's available next
```

**3. SendMessage — Direct communication**
```
SendMessage: type="message", recipient="reviewer",
  content="Backend scaffolding complete, ready for review",
  summary="Backend ready for review"
```

### Parallel Execution

The `/parallel-sprint` command sets up multiple agents working on independent tracks simultaneously. The key insight is **identifying which tasks are truly independent**:

```
Sprint 0: Project Setup
├── Track 1: Backend (independent)      → backend-builder agent
├── Track 2: Frontend (independent)     → frontend-builder agent
├── Track 3: UX Research (independent)  → ux-researcher agent
└── Track 4: Forge Tooling (independent)→ ux-architect agent

Dependency rule: Only ONE agent edits a given file at a time.
If two tasks touch the same file, they MUST be sequential.
```

---

## 3. Prompt Engineering Best Practices

### CLAUDE.md as Persistent Memory

`CLAUDE.md` is the most important file in any Claude Code project. It persists across sessions, is read automatically at the start of every conversation, and acts as **standing orders** for the AI.

Structure it with mandatory directives at the top:

```markdown
# Ethnomusicology Project - Claude Code Directives

## Mandatory: The Forge
ALWAYS use the Forge (`.claude/` directory) for ALL development work:
- ALWAYS create use cases via `/uc-create` before implementation
- ALWAYS validate use cases via `/uc-review` before coding
- ALWAYS decompose tasks via `/task-decompose` before starting work
- ALWAYS run quality gates before committing
- ALWAYS verify implementations via `/verify-uc`

## Architecture
- **Backend**: Rust (Axum 0.8) in `backend/`
- **Frontend**: Flutter (Dart) in `frontend/`
- **Database**: SQLite (dev) / PostgreSQL (prod) via SQLx

## Quality Gates
- Backend: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- Frontend: `flutter analyze && flutter test`
- Both must pass before any commit
```

> **Speaker note**: Use ALL CAPS for "ALWAYS" and "NEVER" — Claude Code weights these as strong instructions. The `##` headers create scannable sections. Keep CLAUDE.md under 200 lines; link to detailed docs for depth.

**Key patterns for CLAUDE.md:**
- **Mandate behaviors, don't suggest them.** "ALWAYS create use cases" not "consider creating use cases"
- **Specify the tech stack.** Claude will guess if you don't tell it, and it often guesses wrong
- **Include the quality gate commands verbatim.** Copy-pasteable commands prevent drift
- **Reference the plan document.** `Full plan: docs/project-plan.md` tells Claude where to look for details

### The Session Handoff Pattern

Claude Code sessions have context windows. When you hit the limit, you lose everything the agent "knows" about your project state. The session handoff pattern solves this:

**Before ending a session:**
```
/session-handoff
```

This command auto-gathers:
- Git state (status, recent commits, branches, worktrees)
- Task state (incomplete tasks from all task files)
- Test state (last test run results)
- Active context (previous handoff, current CLAUDE.md)

And writes a structured document to `docs/session-handoff.md`:

```markdown
# Session Handoff — 2026-02-24

## What Was Accomplished
- Project plan finalized and saved to docs/project-plan.md
- Forge tooling adapted from rust-term-chat template
- Backend scaffolded (Axum hello-world compiles and passes tests)
- Frontend scaffolded (Flutter counter app runs on Chrome)

## Current State
- Branch: main (initial commit)
- Backend: cargo test passes (1 test)
- Frontend: flutter test passes (1 test)
- Blockers: None

## Next Session Should
1. Read this file: `docs/session-handoff.md`
2. Begin Sprint 1: UC-01 (Import Seed Catalog from Spotify)
3. Run `/uc-create Import Seed Catalog from Spotify Playlist`
```

**Starting the next session:**
> "Read `docs/session-handoff.md` and continue from where the last session left off."

This single sentence restores full context.

### Structured Plan Documents

Don't give Claude a one-liner like "build me a music app." Give it a complete plan document with:

1. **Context** — What are we building and why?
2. **Tech stack decision table** — What, Why columns for every layer
3. **Architecture diagram** — Even ASCII art helps Claude understand system boundaries
4. **Use cases with dependencies** — Numbered, prioritized, with dependency chains
5. **Implementation sequence** — Sprint-level breakdown
6. **Key dependencies** — Exact versions in Cargo.toml/pubspec.yaml format
7. **Project structure** — Full directory tree
8. **Verification strategy** — How to know each sprint is done

The plan document for this project is 265 lines and covers all of the above. The result: Claude Code can scaffold the entire monorepo from it without asking clarifying questions.

> **Speaker note**: The upfront investment in the plan document pays for itself 10x. Every minute spent on the plan saves 10 minutes of debugging wrong assumptions.

### Devil's Advocate Prompting

When making technology decisions, use the **3-option debate format**:

```
We need to choose a frontend framework. Evaluate these options:

| Criteria | Next.js (SSR) | Flutter (Hybrid) | SvelteKit |
|----------|-------------|-----------------|-----------|
| Learning curve | ... | ... | ... |
| Mobile story | ... | ... | ... |
| RTL support | ... | ... | ... |
| Audio playback | ... | ... | ... |

For each option, give:
1. Three strongest arguments FOR
2. Three strongest arguments AGAINST
3. Recommended choice with reasoning
```

Then follow up with a Devil's Advocate concerns table:

```
| Concern | Mitigation |
|---------|-----------|
| [Concern 1] | [How we address it] |
| [Concern 2] | [How we address it] |
```

This format forces Claude to argue both sides rather than just confirming your initial preference.

### Task Decomposition

The `/task-decompose` command breaks a use case into implementable units. The output format is designed for agent consumption:

```markdown
## Tasks for UC-001: Import Seed Catalog

### Phase 1: Foundation (sequential)
| # | Task | Size | Module | Depends On | Test |
|---|------|------|--------|-----------|------|
| 1 | Create tracks table migration | S | backend/migrations/ | - | migration runs |
| 2 | Define Track model struct | S | backend/src/db/tracks.rs | T1 | compiles |
| 3 | Implement track CRUD queries | M | backend/src/db/tracks.rs | T2 | unit tests |

### Phase 2: Spotify Integration (parallelizable after Phase 1)
| # | Task | Size | Module | Depends On | Test |
|---|------|------|--------|-----------|------|
| 4 | Spotify OAuth client setup | M | backend/src/api/spotify.rs | T1 | auth test |
| 5 | Playlist fetch endpoint | M | backend/src/api/spotify.rs | T4 | mock test |
| 6 | Import route handler | L | backend/src/routes/tracks.rs | T3,T5 | integration |
```

Key properties:
- **Size estimates** (S/M/L/XL) help the lead allocate work
- **Module paths** tell the builder exactly where to write code
- **Dependencies** reveal what can be parallelized
- **Test column** tells the builder what "done" means

### Quality Gates as Hooks

The `settings.json` file configures automated quality enforcement:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash(git commit.*)",
        "hooks": [
          {
            "type": "command",
            "command": "cd backend && cargo fmt --check 2>&1 && cargo clippy -- -D warnings 2>&1 && cargo test 2>&1",
            "description": "Quality gate: format + lint + test before commit",
            "timeout": 180000
          }
        ]
      }
    ]
  }
}
```

This hook runs **automatically before every `git commit`**. The agent can't bypass it — if tests fail, the commit is blocked. This is the AI equivalent of CI/CD, but it runs locally and instantly.

> **Speaker note**: Hooks are the most underused feature of Claude Code. They're how you enforce standards without relying on the agent's "willpower." The agent *will* try to skip tests if you let it. Hooks don't let it.

---

## 4. Tooling Setup

### The Slash Command System

Slash commands (`.claude/commands/*.md`) are workflow automations that Claude Code executes when you type `/command-name`. They're markdown files with frontmatter and a structured prompt.

**Anatomy of a slash command:**

```markdown
---
description: Interactively create a Cockburn-style use case document
allowed-tools: Read, Glob, Write, AskUserQuestion
---

# Create a Use Case: $ARGUMENTS

You are a **Requirements Architect** creating a fully-dressed Cockburn use case.
The use case goal is: **$ARGUMENTS**

## Step 1: Determine the Next UC Number
Scan `docs/use-cases/` for existing files matching `uc-*.md`. Increment the highest.

## Step 2: Validate the Goal
Title MUST be an **Active Verb Phrase Goal**. Rephrase if needed.

## Step 3: Walk Through Each Section Interactively
[...8 groups of questions...]

## Step 6: Next Steps Reminder
1. Review: `/uc-review`
2. Decompose: `/task-decompose <UC-number>`
```

Key features:
- `allowed-tools` restricts what the command can do (principle of least privilege)
- `$ARGUMENTS` is replaced with whatever the user types after the command name
- Numbered steps give Claude a clear execution sequence
- The reminder at the end chains to the next command in the workflow

**The full command catalog:**

| Command | Purpose | Inputs | Output |
|---------|---------|--------|--------|
| `/uc-create` | Create Cockburn use case | Goal description | `docs/use-cases/uc-NNN-slug.md` |
| `/uc-review` | Review UC for gaps | UC number | Gap analysis with completeness score |
| `/task-decompose` | Break UC into tasks | UC number | `docs/tasks/uc-NNN-tasks.md` |
| `/agent-team-plan` | Design team for tasks | UC number or "all" | `docs/teams/uc-NNN-team.md` |
| `/parallel-sprint` | Set up parallel agents | Sprint name | Worktree branches + agent assignments |
| `/wireframe` | Generate ASCII wireframes | Screen name or UC | `docs/wireframes/screen-name.md` |
| `/ux-review` | Review UI implementation | UC number | UX review report (CRITICAL/WARNING/SUGGESTION) |
| `/verify-uc` | Check postconditions | UC number | Pass/fail verification |
| `/grade-work` | Blind review against spec | UC number | Grade report (A-F with scoring breakdown) |
| `/session-handoff` | Preserve session state | Optional summary | `docs/session-handoff.md` |
| `/retrospective` | Capture lessons learned | Milestone name | `docs/retrospectives/slug.md` + action items |
| `/code-quality` | Run quality checks | - | Lint, format, test results |
| `/prd-from-usecases` | Synthesize PRD from UCs | UC range | Product requirements document |

### Agent Definitions

Agent definitions (`.claude/agents/*.md`) specify team structure, roles, workflows, and coordination rules. They have frontmatter that tells Claude Code the agent type:

```markdown
---
description: Implementation Team — builds, reviews, and documents features
agent_type: general-purpose
---
```

**What makes a good agent definition:**

1. **Clear role descriptions** — Each role has a name, responsibilities, and boundaries
2. **Explicit workflow** — Numbered steps showing the team's process
3. **Quality gates table** — What must pass, what command to run, who owns it
4. **Coordination rules** — How agents avoid conflicts (file locking, sequential constraints)
5. **Key references** — Links to templates, rubrics, and project docs

Example quality gates table from the Implementation Team:

| Gate | Check | Command | Owner |
|------|-------|---------|-------|
| 1 | Backend Format | `cd backend && cargo fmt --check` | Automated |
| 2 | Backend Lint | `cd backend && cargo clippy -- -D warnings` | Automated |
| 3 | Backend Tests | `cd backend && cargo test` | Automated |
| 4 | Frontend Analyze | `cd frontend && flutter analyze` | Automated |
| 5 | Frontend Tests | `cd frontend && flutter test` | Automated |
| 6 | UC Verification | Verification command from use case | Reviewer |
| 7 | Blind Review | Grade against acceptance criteria | Reviewer (fresh context) |

### Skills as Reusable Reference Documents

Skills (`.claude/skills/*.md`) are reference documents that agents consult during work. They're not commands — they're templates, rubrics, and checklists that provide consistency.

**The Cockburn template** (`cockburn-template.md`) — 113 lines defining the exact structure every use case must follow. Includes field reference tables for goal level, priority, complexity, and common actors/invariants specific to this project.

**The grading rubric** (`grading-rubric.md`) — 156 lines defining:
- Grade scale (A-F with percentage ranges)
- 5 scoring categories with weights:
  - Postcondition Coverage (30%)
  - Extension Handling (25%)
  - Invariant Enforcement (15%)
  - Code Quality (15%)
  - Test Quality (15%)
- Bonus/penalty adjustments (e.g., `-10% per unwrap() in Rust production code`)
- Completeness scoring for use case documents

**The design system** (`design-system.md`) — 423 lines covering:
- Color palette (core + occasion-specific + sacred/devotional)
- Typography (font families, type scale, Arabic typography notes)
- Spacing scale (4dp grid)
- Elevation/shadow levels
- Border radius tokens
- Component patterns (Track Tile, Playlist Card, Mini Player, etc.) with ASCII specs
- Motion/animation specs with duration tokens and easing curves
- Responsive breakpoints
- Flutter implementation code examples

> **Speaker note**: The design system as a skill is particularly powerful. It means Claude Code produces *consistent UI* across all screens without a human designer reviewing every widget. The agent references the skill and uses the correct tokens, spacing, and patterns every time.

**The pre-implementation checklist** (`pre-implementation-checklist.md`) — A gate that must be verified before any implementation work begins:

```markdown
## Required (must be YES to proceed)
- [ ] UC doc exists
- [ ] UC reviewed (`/uc-review` passed)
- [ ] Task file exists (`/task-decompose` done)
- [ ] Feature branch created (NOT on main)
- [ ] Handoff read (if continuing from prior session)
- [ ] Dependencies added (before spawning parallel agents)
```

### Settings.json Hooks

The `settings.json` file in `.claude/` configures automated behavior:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash(git commit.*)",
        "hooks": [
          {
            "type": "command",
            "command": "cd backend && cargo fmt --check 2>&1 && cargo clippy -- -D warnings 2>&1 && cargo test 2>&1",
            "description": "Quality gate: format + lint + test before commit",
            "timeout": 180000
          }
        ]
      }
    ]
  }
}
```

The `matcher` uses regex against tool calls. `PreToolUse` fires *before* the tool executes. If the hook command returns a non-zero exit code, the tool call is blocked.

**Practical implications:**
- Agent tries to `git commit` → hook runs `cargo fmt + clippy + test` → if any fail, commit is rejected
- Agent must fix the issue and retry — no way to skip
- 180-second timeout accommodates compilation time
- This is real enforcement, not a suggestion in CLAUDE.md that the agent might ignore

---

## 5. Working Model: Sprint 0 Case Study

### From Blank Repo to Scaffolded Monorepo

Sprint 0 took the project from an empty git repository to a fully scaffolded monorepo with:
- Rust backend (Axum) compiling and serving JSON
- Flutter frontend compiling and running in Chrome
- Static landing page
- Complete Forge tooling (13 slash commands, 4 skills, 3 agent definitions)
- Design system with color palettes, typography, spacing, and component specs
- Project plan with 8 use cases, dependency graph, and sprint breakdown

### Team Composition

Four agents worked in parallel:

| Agent | Role | Track | Key Outputs |
|-------|------|-------|-------------|
| **backend-builder** | Implementation | Backend scaffold | `backend/Cargo.toml`, `src/main.rs`, hello-world endpoint, tests |
| **frontend-builder** | Implementation | Frontend scaffold | `frontend/pubspec.yaml`, `lib/main.dart`, Flutter project structure |
| **ux-researcher** | Research | UX & Design | Design system skill, color palettes, typography, component patterns |
| **ux-architect** | Tooling | Forge setup | All 13 slash commands, agent definitions, templates, hooks |

### What the Lead Agent Does vs What Gets Delegated

**Lead agent responsibilities:**
- Reads the project plan and identifies parallelizable work
- Creates the team and task list
- Assigns tasks to agents based on their specialization
- Monitors for conflicts (two agents editing the same file)
- Resolves blockers when agents get stuck
- Performs final integration verification
- Runs session handoff at the end

**What gets delegated:**
- All actual code writing (to builders)
- All research and design work (to researchers/architects)
- Quality reviews (to reviewers)
- Documentation updates (to documentation agents)

> **Speaker note**: The lead agent is like a tech lead in a standup. They don't write code during the standup — they unblock others and make sure work flows.

### How Research Agents Inform Implementation Agents

The information flow in Sprint 0:

```
UX Researcher                    UX Architect
     │                                │
     │ Researches:                    │ Creates:
     │ - Color palettes               │ - /uc-create command
     │ - Typography for Arabic        │ - /wireframe command
     │ - Component patterns           │ - /ux-review command
     │ - Accessibility requirements   │ - Agent definitions
     │                                │ - settings.json hooks
     ▼                                ▼
Design System Skill              Forge Commands & Agents
(.claude/skills/design-system.md)     (.claude/commands/*.md)
     │                                │
     │ Referenced by:                 │ Used by:
     ▼                                ▼
Implementation Team              All Future Sessions
(Sprint 1+: builders use          (Every session starts
design tokens, wireframes)        with CLAUDE.md → Forge)
```

The research agents' output becomes the implementation agents' input. The design system skill means that when a builder creates a Flutter screen in Sprint 1, it can reference exact color values, spacing tokens, and component patterns — without a human designer in the loop.

### Handling Failures

When agents get blocked, the system has defined escalation paths:

**Missing tools or permissions:**
- Agent flags the blocker immediately (coordination rule: "don't wait silently")
- Lead agent adapts — either provides the missing resource or reassigns the task
- Example: If Flutter SDK isn't installed, the frontend-builder can't scaffold. The lead either installs it or assigns the task to a different agent that can run shell commands.

**Conflicting file edits:**
- Coordination rule: "Only ONE agent edits a given file at a time"
- Builder claims files by listing them in the task when starting work
- If two tasks touch the same file, they're made sequential, not parallel

**Test failures:**
- Reviewer runs quality gates and sends specific, actionable feedback with line numbers
- Builder reworks the code
- Reviewer re-checks — this loop repeats until quality gates pass

**Session context limits:**
- `/session-handoff` captures state before context is lost
- Next session starts by reading the handoff document
- No institutional knowledge is lost between sessions

---

## 6. Key Principles

### "Measure Twice, Cut Once": Plan Mode Before Implementation

Claude Code has a Plan mode (EnterPlanMode) where it explores the codebase and designs an approach *before* writing any code. Use it for anything non-trivial.

**The cost of planning is low.** Reading files and thinking about architecture takes seconds.
**The cost of wrong implementation is high.** Refactoring, debugging, and re-testing takes minutes to hours.

The Forge enforces this with the pre-implementation checklist: UC must exist, must be reviewed, must be decomposed into tasks, must have a feature branch — all before a single line of code is written.

### "The Forge Remembers": Session Handoffs and Institutional Memory

Three layers of memory in the Forge:

| Layer | File | Scope | Lifespan |
|-------|------|-------|----------|
| **Standing orders** | `CLAUDE.md` | Every session | Permanent (versioned) |
| **Session state** | `docs/session-handoff.md` | Next session | Until overwritten |
| **Process learnings** | `docs/retrospectives/*.md` | All future work | Permanent (versioned) |

The retrospective command feeds improvements back into the Forge itself. If a sprint reveals that agents keep forgetting to run `cargo fmt`, the retrospective proposes adding a hook — and the next sprint benefits automatically.

### "Agents Need Specs More Than Humans Do"

A human engineer has:
- Years of experience with similar systems
- Intuition about edge cases
- Ability to ask a colleague in real-time
- Understanding of organizational context

An AI agent has:
- The spec you gave it
- The code it can read
- The tools you configured

If the spec is vague, the agent's output will be vague. If the spec is precise (Cockburn postconditions, testable acceptance criteria, explicit extensions), the agent's output will be precise.

**Concrete example:** Without extensions, an agent implementing Spotify import might write:

```rust
let tracks = spotify.get_playlist_tracks(&playlist_id).await?;
// Happy path only — what about rate limits? Token expiry? Empty playlist?
```

With Cockburn extensions documented, the same agent writes:

```rust
let tracks = match spotify.get_playlist_tracks(&playlist_id).await {
    Ok(tracks) if tracks.is_empty() => return Err(AppError::EmptyPlaylist),
    Ok(tracks) => tracks,
    Err(SpotifyError::RateLimit { retry_after }) => {
        tokio::time::sleep(retry_after).await;
        spotify.get_playlist_tracks(&playlist_id).await?
    }
    Err(SpotifyError::TokenExpired) => {
        spotify.refresh_token().await?;
        spotify.get_playlist_tracks(&playlist_id).await?
    }
    Err(e) => return Err(e.into()),
};
```

### "Parallelize Everything"

If tasks are independent, run them simultaneously. The `/parallel-sprint` command sets up worktrees so agents don't conflict:

```
main branch (protected)
├── worktree: feature/backend-scaffold  → backend-builder
├── worktree: feature/frontend-scaffold → frontend-builder
├── worktree: feature/ux-research       → ux-researcher
└── worktree: feature/forge-tooling     → ux-architect
```

**Rules for safe parallelization:**
1. Each agent works in its own git worktree (isolated copy of the repo)
2. No two agents edit the same file
3. Dependencies between agents are explicit in the task graph
4. The lead agent merges results at review gates

### "Quality Gates as Culture"

Quality isn't something you check at the end — it's enforced at every step:

| When | Gate | How |
|------|------|-----|
| Before implementation | Pre-implementation checklist | Manual check by lead |
| Before every commit | Format + lint + test | Automated via settings.json hook |
| After each task group | Reviewer quality checks | Reviewer agent runs all gates |
| After implementation | Use case verification | `/verify-uc` command |
| After verification | Blind grading | `/grade-work` command |
| After sprint | Retrospective | `/retrospective` command |

The grading rubric is particularly effective because it's a **blind review** — the grading agent evaluates the code *only* against the spec, without knowledge of implementation difficulty or effort. This prevents the sunk cost fallacy ("we spent so long on this, it must be good enough").

### "The AI Is Your Team, Not Your Tool"

The mental model shift:

| Tool Mindset | Team Mindset |
|-------------|-------------|
| "Claude, write me a function" | "Backend-builder, implement task #3 from the task list" |
| "Fix this bug" | "Reviewer, investigate the test failure in tracks.rs and create a fix task" |
| "Make the UI look good" | "UX team, create wireframes for the browse screen before the implementation team starts" |
| One agent does everything | Specialized agents with clear roles and handoffs |
| Fix problems when they appear | Pre-implementation checklist prevents problems from starting |
| Review at the end | Review gates at every stage |

---

## 7. Anti-Patterns to Avoid

### Anti-Pattern 1: Single Agent for Everything

**What it looks like:** One Claude Code session that plans, researches, implements, tests, and documents.

**Why it fails:**
- Context window fills up with research that's no longer needed during implementation
- No adversarial review — the agent won't challenge its own decisions
- Serialized execution — backend can't be built while frontend research is happening
- No specialization — a "jack of all trades" agent does nothing excellently

**Fix:** Use teams. Even a minimal team (lead + builder + reviewer) outperforms a single agent on any task that takes more than 15 minutes.

### Anti-Pattern 2: Vague Prompts Without Structure

**What it looks like:**
> "Build me a music app that helps people create playlists for different occasions."

**Why it fails:**
- Agent has to guess the tech stack, architecture, data model, and UI patterns
- No definition of "done" — when is it good enough?
- No error handling spec — agent implements happy path only
- No verification criteria — you'll manually test everything

**Fix:** Use Cockburn use case templates. The structure forces completeness:
- Preconditions tell the agent what to assume
- Postconditions tell the agent what "done" looks like
- Extensions tell the agent what to handle when things go wrong
- Acceptance criteria tell the agent how it will be graded

### Anti-Pattern 3: No Quality Gates

**What it looks like:** Agent writes code, commits, and moves on. No tests, no lint, no review.

**Why it fails:**
- Agents *will* cut corners without enforcement. They'll use `unwrap()` instead of proper error handling. They'll skip tests for "simple" functions. They'll hardcode values.
- Technical debt accumulates invisibly
- Bugs compound — later features build on broken foundations

**Fix:** Quality gates at every stage, enforced by hooks:
- Pre-commit hook runs `cargo fmt + clippy + test`
- Reviewer agent runs full quality gate after each task group
- Grading rubric penalizes specific anti-patterns (-10% per `unwrap()`)
- Design system skill prevents hardcoded visual values

### Anti-Pattern 4: Skipping Research

**What it looks like:** Jumping straight to implementation without evaluating tools, libraries, or approaches.

**Why it fails:**
- You might pick a library that's deprecated, or a framework that doesn't support your requirements
- The Spotify API, for example, has changed significantly — code written for the old API doesn't work
- Cultural/domain-specific requirements (like RTL Arabic layout) need research before design

**Fix:** Use the Requirements Team pattern:
- Architecture Scout researches technical feasibility
- Devil's Advocate challenges assumptions
- Test Designer identifies untestable requirements early
- Research happens *before* implementation, not during

In this project, UX research revealed that Arabic text renders ~20% larger than Latin at the same font size, that `TextOverflow.ellipsis` breaks mid-word in Arabic (use `.fade` instead), and that mixed-direction text needs Unicode bidi markers. These would have been painful bugs to discover during implementation.

### Anti-Pattern 5: Not Documenting Decisions

**What it looks like:** Making a tech stack decision in one session, then starting a new session that doesn't know *why* that decision was made.

**Why it fails:**
- Next session might reverse the decision without understanding the trade-offs
- New team members (human or AI) can't understand the reasoning
- When something breaks, there's no record of what was considered and rejected

**Fix:** Three documentation mechanisms:
1. **Decision tables in the project plan** — Tech stack choices with "Why" column
2. **Devil's Advocate concerns table** — What was argued against and how it was mitigated
3. **Session handoff** — Captures decisions made in each session
4. **Retrospectives** — Captures what worked and what didn't after each milestone

### Anti-Pattern 6: Over-Engineering Initial Prompts

**What it looks like:** Spending hours crafting the "perfect" CLAUDE.md before starting any work.

**Why it fails:**
- You don't know what you need until you've done a sprint
- Requirements emerge from implementation, not from planning alone
- The Forge itself should evolve — the retrospective command exists precisely for this

**Fix:** Start with a minimal CLAUDE.md (tech stack, quality gates, project context). Run a sprint. Run `/retrospective`. Update the Forge based on what you learned. Iterate.

The Forge for this project was adapted from a previous project (rust-term-chat) and then customized during Sprint 0. The design system skill, UX team agent, and wireframe command were all created *during* the session, not before it.

---

## Appendix: Quick Reference Card

### Starting a New Project
```
1. Create CLAUDE.md (tech stack, quality gates, project context)
2. Write docs/project-plan.md (architecture, use cases, sprints)
3. Copy/adapt .claude/ directory (or create from scratch)
4. Run Sprint 0: scaffold, verify, handoff
```

### Starting a New Feature
```
1. /uc-create <feature description>
2. /uc-review
3. /task-decompose <UC-number>
4. /agent-team-plan <UC-number>
5. Pre-implementation checklist ✓
6. Implement (on feature branch, with agent team)
7. /verify-uc <UC-number>
8. /grade-work <UC-number>
```

### Starting a New Session
```
1. Read docs/session-handoff.md
2. "Continue from where the last session left off"
```

### Ending a Session
```
1. /session-handoff
2. Verify docs/session-handoff.md was written
```

### The Forge Directory Structure
```
.claude/
├── agents/          # WHO does the work (team definitions)
├── commands/        # HOW work is done (workflow automations)
├── skills/          # WHAT standards to follow (reference docs)
└── settings.json    # WHEN checks run (automated hooks)
```

---

*This document was generated from the Ethnomusicology project build session. All examples, file paths, and code snippets reference actual artifacts in the repository.*
