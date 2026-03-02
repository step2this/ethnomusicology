---
description: Create a Cockburn-format steel thread with cross-cutting integration fields
allowed-tools: Read, Glob, Write, AskUserQuestion
---

# Create Steel Thread: $ARGUMENTS

You are a **Integration Architect** creating a steel thread for the Ethnomusicology project. A steel thread is a thin end-to-end vertical slice that proves architectural connections across all system layers. The steel thread goal is: **$ARGUMENTS**

## Step 1: Determine the Next ST Number

Scan `docs/steel-threads/` for existing files matching `st-*.md`. Increment the highest number. If the directory doesn't exist, start at ST-001.

## Step 2: Validate the Goal

Title MUST be an **Active Verb Phrase Goal**. Rephrase if needed.

Goal level is always **Thread** (🧵 Thread — thin end-to-end proof of architectural connection). Steel threads are not use cases — they exist to prove that layers connect, not to deliver complete user value.

## Step 3: Walk Through Each Section Interactively

Work through sections in conversational groups using AskUserQuestion:

### Group 1: Classification
- **Goal Level**: Always 🧵 Thread
- **Scope**: Always System (black box) — threads cross all layers by definition
- **Priority**: P0 Critical | P1 High | P2 Medium | P3 Low
- **Complexity**: 🟢 Low | 🟡 Medium | 🔴 High | ⚫ Spike needed

### Group 2: Cross-Cutting References
This is what makes steel threads different from use cases. Ask:
- Which UCs does this thread slice through? (e.g., UC-001 step 14, UC-013 step 11)
- For each UC, which specific MSS steps and extensions does this thread prove?
- Are there any steps that span multiple UCs?

Format as:
```
- **UC-001**: Steps 3, 5, 14 — proves Spotify OAuth → track fetch → DB write
- **UC-013**: Steps 2, 11 — proves natural language → Claude API → setlist response
```

### Group 3: Actors
- **Primary Actor**: <who initiates>
- **Supporting Actors**: <systems, services, other users involved>
- **Stakeholders & Interests**: <stakeholder>: <what they care about>

Common actors: App User, Spotify API, Beatport API, SoundCloud API, Claude API, essentia Sidecar, Audio Player, Database

### Group 4: Conditions
- **Preconditions** (must be true before starting):
  1. <condition — becomes a setup assertion>
- **Success Postconditions** (true when done right):
  1. <condition — becomes a verification assertion>
- **Failure Postconditions** (true when it fails gracefully):
  1. <condition — becomes a failure-mode test>
- **Invariants** (must remain true throughout):
  1. <condition — becomes a continuous assertion>

### Group 5: API Contract Section
For each endpoint the thread touches:
- **Method**: GET | POST | PUT | PATCH | DELETE
- **Path**: e.g., `/api/tracks`
- **Description**: Brief purpose
- **OpenAPI Schema Ref**: (left blank — populated by `/api-contract`)
- **Contract Status**: Draft → Agreed → Implemented → Verified

Format as a table:
```
| Method | Path | Description | Schema Ref | Status |
|--------|------|-------------|------------|--------|
| GET    | /api/tracks | Fetch track list | — | Draft |
```

### Group 6: Main Success Scenario
Build the happy path step by step. Focus on the **cross-layer journey** — every step should identify which layer it executes in:
1. **[Frontend]** User does something
2. **[Frontend → API]** App sends request to endpoint
3. **[API]** Server validates and processes
4. **[API → DB]** Server reads/writes data
5. **[API → External]** Server calls external service
6. **[API → Frontend]** Server returns response
7. **[Frontend]** App renders result

The MSS must cross at least 3 layers to qualify as a steel thread.

### Group 7: Extensions (What Can Go Wrong)
For EACH MSS step: "What could go wrong?" This is the MOST IMPORTANT section for steel threads because integration failures are the primary risk.

Focus especially on:
- Network failures between layers
- Schema mismatches (API returns unexpected shape)
- Auth failures at integration boundaries
- Timeout/latency issues across service calls

Format:
- **2a. <condition at step 2>**:
  1. System <handles it>
  2. <returns to step X | use case fails>

### Group 8: Integration Assertions
Cross-layer proofs beyond postconditions. These are the **measurable claims** this thread exists to verify:
- "Frontend can call GET /tracks and receive well-formed JSON response"
- "Latency < 200ms P95 for the full round trip"
- "Error response includes machine-readable error code and human-readable message"
- "Auth token is forwarded correctly from frontend through API to external service"
- "Database schema supports the response shape without lossy transformation"

Each assertion should identify the two (or more) layers it spans.

### Group 9: Does NOT Prove
Explicit scope boundary to prevent scope creep. Ask what is deliberately out of scope:
- What features or behaviors are NOT being tested by this thread?
- What will be proven by other threads or full UC implementation?
- What edge cases are intentionally deferred?

Examples:
- "Does NOT prove pagination (covered by ST-003)"
- "Does NOT prove error recovery for partial failures (covered by full UC-013 implementation)"
- "Does NOT prove performance under load (separate spike SP-002)"

### Group 10: Agent Execution Notes
- **Verification Command**: `<shell command to verify integration assertions>`
- **Test File**: `<path to integration test that validates this thread>`
- **Depends On**: ST-<n>, UC-<m>, SP-<k>
- **Blocks**: ST-<x>, UC-<y>
- **Estimated Complexity**: <T-shirt size> / <token budget hint>
- **Agent Assignment**: Lead | Teammate:<role> | Subagent

### Group 11: Acceptance Criteria
Based on postconditions + integration assertions + standard criteria:
- [ ] All success postconditions verified by automated test
- [ ] All integration assertions pass end-to-end
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] API contract matches implementation (request/response shapes)
- [ ] Cross-layer round trip completes without manual intervention
- [ ] Code passes quality gates (cargo fmt, clippy, cargo test, flutter analyze, flutter test)
- [ ] Reviewer agent approves

## Step 4: Score Completeness

Evaluate the steel thread against this scoring checklist (minimum 70%):

| Section | Weight | Score |
|---------|--------|-------|
| Classification | 5% | |
| Cross-Cutting References | 15% | |
| Actors | 5% | |
| Conditions | 10% | |
| API Contract Section | 15% | |
| Main Success Scenario | 15% | |
| Extensions | 10% | |
| Integration Assertions | 15% | |
| Does NOT Prove | 5% | |
| Agent Execution Notes | 5% | |

Integration-specific sections (Cross-Cutting References, API Contract, Integration Assertions) carry extra weight because they are the reason steel threads exist.

## Step 5: Write the Steel Thread File

Write to `docs/steel-threads/st-<NNN>-<slug>.md`. Create the directory if it doesn't exist.

Use this document structure:
```markdown
# Steel Thread: ST-<NNN> <Active Verb Phrase Goal>

## Classification
...

## Cross-Cutting References
...

## Actors
...

## Conditions
...

## API Contract
...

## Main Success Scenario
...

## Extensions
...

## Integration Assertions
...

## Does NOT Prove
...

## Agent Execution Notes
...

## Acceptance Criteria
...
```

## Step 6: Next Steps Reminder

After writing the file, remind the user:

1. **Review**: `/uc-review st-<NNN>` (reuses the devil's advocate review)
2. **API Contract**: `/api-contract ST-<NNN>` (writes OpenAPI specs for endpoints)
3. **Decompose**: `/task-decompose ST-<NNN>` (breaks into implementable tasks)
4. **Design crit**: Run `design-crit` if the thread involves frontend screens
5. **Spike first**: If any section scored poorly due to unknowns, create a spike with `/spike-create` before proceeding
