---
description: Review a use case document for gaps, missing extensions, and untestable postconditions
allowed-tools: Read, Glob, Grep
---

# Review Use Case: $ARGUMENTS

You are a **Devil's Advocate** reviewing a Cockburn-style use case for the Ethnomusicology project.

## Step 1: Load the Use Case

Find and read from `docs/use-cases/uc-*.md`.

## Step 2: Structural Completeness Check

Verify all required sections exist and are non-empty.

## Step 3: Title Quality

Is it an Active Verb Phrase Goal? Does goal level match?

## Step 4: Precondition Analysis

Verifiable? Necessary? Missing preconditions?

## Step 5: Postcondition Analysis

Testable and automatable? Specific enough? Missing postconditions?

## Step 6: MSS Analysis

Clear actors? Atomic steps? Logical ordering? Missing steps?

## Step 7: Extension Coverage (MOST CRITICAL)

For EACH MSS step: What if it fails? Unexpected input? Dependency unavailable? Race condition? Wrong permissions?

List missing extensions with suggested content.

## Step 8: Invariant Check

Continuously verifiable? Common project invariants covered?

## Step 9: Agent Execution Notes Check

Verification command runnable? Test file path correct? Dependencies accurate? Blocks consistent?

## Step 10: Generate Review Report

Output structured report with overall score, strengths, issues (CRITICAL/WARNING/SUGGESTION), missing extensions, untestable postconditions, and recommended actions.
