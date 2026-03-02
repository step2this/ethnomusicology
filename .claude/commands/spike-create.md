---
description: Create a time-boxed spike with hypothesis, findings, and decision fields
allowed-tools: Read, Glob, Write, AskUserQuestion
---

# Create Spike: $ARGUMENTS

You are a **Technical Investigator** creating a time-boxed spike for the Ethnomusicology project. A spike is a deliberate, short investigation to reduce uncertainty before committing to implementation. Spikes are NOT Cockburn use cases — they are lightweight, hypothesis-driven, and strictly time-boxed. The spike topic is: **$ARGUMENTS**

## Step 1: Determine the Next SP Number

Scan `docs/spikes/` for existing files matching `sp-*.md`. Increment the highest number. If the directory doesn't exist, start at SP-001.

## Step 2: Walk Through Each Section Interactively

Work through sections in conversational groups using AskUserQuestion:

### Group 1: Hypothesis
One sentence stating what we believe and need to verify. The hypothesis must be **falsifiable** — it should be possible to prove it wrong.

Good examples:
- "Beatport v4 API provides BPM and musical key in track metadata responses."
- "essentia can detect BPM within +/- 1 BPM accuracy for electronic music tracks."
- "Claude Sonnet can generate a coherent 10-track setlist with Camelot-compatible key transitions in a single prompt."

Bad examples (not falsifiable):
- "We should use Beatport." (opinion, not hypothesis)
- "The API might work." (too vague)

### Group 2: Timebox
- **Maximum Hours**: How long to spend before stopping, regardless of progress (typical: 2-4h)
- **Start Date**: When the spike begins
- **Status**: Not Started (always at creation time)

Statuses progress: Not Started → In Progress → Complete

### Group 3: Questions to Answer
3-5 specific, answerable questions. Each question should directly help confirm or reject the hypothesis.

Good examples:
- "What is the Beatport v4 authentication flow?"
- "Does the track endpoint return BPM and key fields?"
- "What rate limits apply to the search endpoint?"
- "Is the response format JSON or XML?"
- "Are there sandbox/test credentials available?"

Each question should be something that can be answered with evidence (API response, documentation quote, benchmark result), not opinion.

### Group 4: Method
Brief, practical description of what to try. This is a recipe for the investigation, not a design document.

Include as appropriate:
- curl commands or API calls to make
- Code snippets to write (keep minimal — spikes produce throwaway code)
- What to measure or observe
- Tools to use (Postman, curl, a small Rust/Python script)
- Specific documentation URLs to check

Example:
```
1. Register for Beatport API access at developer.beatport.com
2. curl the /tracks/{id} endpoint with a known track ID
3. Inspect response for bpm, key, and genre fields
4. Try 5 tracks across different genres to check field consistency
5. Measure response time for search queries
```

### Group 5: Feeds Into
Which steel threads and use cases are affected by the findings:
- **Steel Threads**: ST-<NNN> — how findings affect the thread
- **Use Cases**: UC-<NNN> — how findings affect the use case
- **Spikes**: SP-<NNN> — if this spike may trigger follow-up spikes

## Step 3: Write the Spike File

Write to `docs/spikes/sp-<NNN>-<slug>.md`. Create the directory if it doesn't exist.

Use this document structure:
```markdown
# Spike: SP-<NNN> <Descriptive Title>

## Hypothesis

<One falsifiable sentence>

## Timebox

- **Maximum Hours**: <N>h
- **Start Date**: <YYYY-MM-DD>
- **Status**: Not Started

## Questions to Answer

1. <Question 1>
2. <Question 2>
3. <Question 3>
...

## Method

<Practical investigation steps>

## Feeds Into

- **ST-<NNN>**: <how findings affect this thread>
- **UC-<NNN>**: <how findings affect this use case>

---

## Findings

> **Filled in during spike execution. Leave blank at creation time.**

### Q1: <Question 1>
**Answer**: —
**Evidence**: —

### Q2: <Question 2>
**Answer**: —
**Evidence**: —

### Q3: <Question 3>
**Answer**: —
**Evidence**: —

## Decision

> **Filled in after spike completion. Leave blank at creation time.**

- **Hypothesis**: Confirmed | Rejected | Partially confirmed
- **Impact on steel threads**: —
- **Action items**:
  1. —
```

The Findings and Decision sections are **intentionally left blank** at creation time. They are filled in during and after spike execution.

## Step 4: Next Steps Reminder

After writing the file, remind the user:

1. **Execute the spike** — follow the Method section, stay within the timebox
2. **Fill in Findings** — answer each question with evidence as you go
3. **Write the Decision** — confirm or reject the hypothesis, document action items
4. **Update status** — change Status to "Complete" when done
5. **Feed forward** — update the referenced steel threads and use cases with findings
6. **If hypothesis rejected** — consider creating a new spike with a revised hypothesis or flagging the steel thread as blocked
