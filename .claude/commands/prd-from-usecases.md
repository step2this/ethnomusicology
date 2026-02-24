---
description: Generate a PRD by synthesizing multiple use cases into a coherent product requirements document
allowed-tools: Read, Glob, Grep, Write
---

# Generate PRD from Use Cases: $ARGUMENTS

You are a **Product Architect** synthesizing Cockburn use cases into a coherent Product Requirements Document (PRD) for the Ethnomusicology project.

## Input

`$ARGUMENTS` is optional and can be:
- A list of UC numbers (e.g., "001 002 005") — include only these
- A sprint name (e.g., "Sprint 1") — include use cases tagged for that sprint
- Empty — include ALL use cases in `docs/use-cases/`

## Step 1: Load All Relevant Use Cases

Use Glob to find all `docs/use-cases/uc-*.md` files. Read each one and extract key data.

## Step 2: Build the Dependency Graph

Construct a dependency graph from Depends On / Blocks fields.

## Step 3: Group by Goal Level

Organize into hierarchy: Summary Goals (Epics), User Goals (Features), Subfunctions (Requirements).

## Step 4: Extract Cross-Cutting Concerns

Scan for recurring themes: shared invariants, common extensions, shared actors, common preconditions.

## Step 5: Generate the PRD

Write the PRD to `docs/prd.md` with overview, feature map, dependency graph, epics/features, system-wide requirements, implementation order, risks, and acceptance criteria.

## Step 6: Report Summary

Summarize: total use cases, dependency overview, coverage gaps, suggested next actions.
