---
description: Review a use case or steel thread document for gaps, missing extensions, and untestable postconditions
allowed-tools: Read, Glob, Grep
---

# Review Use Case: $ARGUMENTS

You are a **Devil's Advocate** reviewing a Cockburn-style use case for the Ethnomusicology project.

## Step 1: Load the Artifact

Determine the artifact type from `$ARGUMENTS`:
- If argument matches UC/uc pattern or is a plain number: find and read from `docs/use-cases/uc-*.md`
- If argument matches ST/st pattern: find and read from `docs/steel-threads/st-*.md`

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

## Step 9.5: Steel Thread Specific Review (ST Only)

If the artifact is a steel thread, additionally review:

### 9.5a: Cross-Cutting References
- Are specific UC step numbers cited (not just UC numbers)?
- Do the referenced UCs exist?
- Are the referenced MSS steps plausible for what this thread claims to prove?

### 9.5b: API Contract Section
- Is the endpoint table present with Method, Path, Description columns?
- Are endpoints RESTful and following project conventions (`/api/` prefix, plural resources)?
- Does each endpoint have a plausible Contract Status?

### 9.5c: Integration Assertions
- Does each assertion span at least 2 layers?
- Are assertions measurable and testable (not vague)?
- Are there enough assertions to justify the thread's existence?

### 9.5d: Does NOT Prove
- Is the scope boundary specific (not generic placeholders)?
- Does it reference other STs or UCs that cover the excluded scope?
- Is the boundary tight enough to prevent scope creep?

## Step 10: Generate Review Report

Output structured report with overall score, strengths, issues (CRITICAL/WARNING/SUGGESTION), missing extensions, untestable postconditions, and recommended actions. For steel threads, also include: Cross-Cutting References quality, API Contract readiness, Integration Assertion coverage, and Does NOT Prove specificity.
