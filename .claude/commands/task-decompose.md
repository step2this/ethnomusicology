---
description: Break a use case into implementable tasks with dependencies and test requirements
allowed-tools: Read, Glob, Grep, Write
---

# Decompose Use Case into Tasks: $ARGUMENTS

You are an **Implementation Coordinator** breaking a Cockburn use case into concrete, implementable tasks for the Ethnomusicology project.

## Input

`$ARGUMENTS` is either:
- A UC number (e.g., "001" or "UC-001")
- A file path to a use case document
- A use case name (e.g., "Import Seed Catalog")

## Step 0: Pre-Check

Verify:
1. UC has been reviewed with `/uc-review`. If not, recommend running it first.
2. UC doc exists. If not, suggest `/uc-create` first.

## Step 1: Load the Use Case

Use Glob and Read to load from `docs/use-cases/uc-*.md`.

## Step 2: Extract Implementable Units

Map MSS steps, extensions, preconditions, and postconditions to tasks.

## Step 3: Determine Task Dependencies

Build a dependency graph.

## Step 4: Assign to Modules

Map each task to the project structure:

| Layer | Path | Typical Tasks |
|-------|------|---------------|
| Backend Routes | `backend/src/routes/` | API endpoints |
| Backend Services | `backend/src/services/` | Business logic |
| Backend API Clients | `backend/src/api/` | Spotify, YouTube, Last.fm, MusicBrainz |
| Backend DB | `backend/src/db/` | Database queries |
| Frontend Screens | `frontend/lib/screens/` | UI pages |
| Frontend Widgets | `frontend/lib/widgets/` | Reusable components |
| Frontend Services | `frontend/lib/services/` | API client, audio |
| Frontend Providers | `frontend/lib/providers/` | State management |

## Step 5: Estimate Complexity

Size (S/M/L/XL), Risk (Low/Medium/High), Agent suitability.

## Step 6: Generate Task List

Write to `docs/tasks/uc-<NNN>-tasks.md`.

## Step 7: Report to User

Summarize and remind about feature branch, pre-implementation checklist, and agent team plan for complex UCs.
