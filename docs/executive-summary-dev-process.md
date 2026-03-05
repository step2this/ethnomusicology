# Building Products with AI Agents: What We Learned

**Project**: tarab.studio -- an LLM-powered DJ setlist generator (Rust + Flutter + Claude API)
**Period**: Feb--Mar 2026 | **Result**: 9 steel threads, 9 PRs, 510 tests, deployed to production
**Audience**: Senior engineers interested in AI-assisted development process

---

## The Thesis

Prompt engineering and verification loops are the highest-leverage skills in AI-assisted development. Not the model. Not the framework. Not how fast you can generate code. **The process you wrap around generation determines whether you ship quality or ship hallucinations.**

We call the verification pattern a "Ralph loop" -- generate, critique, refine -- and it applies at every level: the product (Claude generates DJ setlists), the development process (Claude Code agents build the product), and the quality gates (critic agents review the code). The same pattern, fractally.

---

## The Process: "The Forge"

Every feature follows a fixed pipeline. No exceptions. When we skipped steps, we paid for it (see "What Happens When You Skip" below).

```
Use Case (Cockburn-style)
  -> Devil's Advocate Review (find plan flaws before coding)
  -> Task Decomposition (dependency graph, file ownership)
  -> Spike (if unknowns exist: research before hacking)
  -> Parallel Agent Teams (builders on non-overlapping files)
  -> Critic Review (fresh-context agent reads the diff cold)
  -> Verification (postconditions checked against implementation)
  -> Retrospective (lessons fed back into the process)
  -> Session Handoff (cross-session continuity doc)
```

**Steel threads** prove end-to-end integration through the full stack. Not prototypes -- thin but complete slices. ST-006 proved multi-input setlist generation from Spotify import through LLM prompt construction through harmonic arrangement through the Flutter UI. One thread, touching every layer.

**Spikes** de-risk unknowns before building. SP-004 spent 30 minutes confirming that Spotify's Audio Features API was deprecated before we built an enrichment pipeline around it. Without that spike, we would have built the integration, deployed it, gotten 403 errors, and then pivoted -- losing days.

---

## Why It Works: Concrete Evidence

### 1. Devil's Advocate Reviews Prevent Wasted Work

In ST-006 (our largest thread: 6 builders, 27 files, 5,600 lines), a devil's advocate review of the task plan caught **3 critical issues before any builder started**:

- **`ContentBlock` name collision**: A new struct would shadow an existing response-parsing enum, breaking imports across 4 files. Cost to fix at plan time: rename it. Cost mid-implementation: rebuild all 4 affected modules.
- **Missing test migration**: Integration tests had their own `create_test_pool()` that would be missing migration 006. Every integration test would fail with cryptic column errors. Cost to fix at plan time: add one line. Cost mid-implementation: 14+ thrash cycles (we know because this exact bug hit UC-019 Phase 2 when planning was skipped).
- **Scope creep**: Daily generation limits had leaked into the task list from a "Does NOT Prove" section. Would have wasted a builder's entire session on out-of-scope work.

Time spent on devil's advocate review: ~15 minutes. Time saved: conservatively 4-6 hours of multi-builder rework.

### 2. Fresh-Context Critic Agents Break the Self-Review Blind Spot

The agent that writes code cannot effectively review it. This is not a discipline problem -- it is a structural limitation of shared context. The writer "knows what they meant," so they read past their own mistakes.

What our critic agents actually caught:

| Steel Thread | Finding | Severity | What Would Have Happened |
|---|---|---|---|
| ST-005 | `position=0` off-by-one in track ordering | HIGH | Silent data corruption: first track always placed last |
| ST-005 | Missing BPM/energy range validation | HIGH | LLM returns BPM=999, stored as-is, breaks arrangement |
| ST-006 | `compute_seed_match_count` defined and tested but never called | HIGH | Postcondition 13 ships unmet -- feature advertised but dead |
| ST-006 | Spotify tab passing raw URL instead of playlist ID | HIGH | Every user hits PLAYLIST_NOT_FOUND on first use |
| ST-007 | `truncate_to_length()` slices UTF-8 bytes, not chars | HIGH | Panic on Arabic music titles -- our core user demographic |
| UC-019 | `dart:web_audio` does not exist (use `package:web`) | CRITICAL | Runtime crash on first audio playback attempt |

None of these would have been caught by the builder who wrote the code. The UTF-8 bug in ST-007 is particularly telling: the builder tested with ASCII track titles. A fresh critic immediately asked, "What happens with Arabic text?" -- which is literally the product's primary use case.

### 3. Structured Decomposition Enables Real Parallelism

| Metric | ST-003 (solo) | ST-005 (team) | ST-006 (team) | ST-007 (team) |
|---|---|---|---|---|
| Builder agents | 0 (lead did everything) | 5 | 6 | 5 |
| Max parallel builders | 1 | 3 | 4 | 3 |
| Merge conflicts | Multiple | 0 | 0 | 0 |
| Context rot errors (late-task failures) | 5+ fix iterations | 0 | 0 | 0 |
| Lines per agent | 2,800 | ~250 | ~930 | ~300 |

ST-003 was the "before" picture. One agent wrote 2,800 lines across 14 files. By task 8 of 9, it was making errors it never would have made fresh: wrong package names, constructor mismatches, invalid URLs. Five or more fix iterations per task. Context rot is measurable and predictable.

After ST-003, we mandated multi-agent teams. Non-overlapping file ownership means zero merge conflicts. Each builder stays fresh (small context). The lead coordinates but never writes implementation code. This is not optional -- it is the single most impactful process change we made.

### 4. Spikes Prevent Expensive Wrong Turns

| Spike | Time Spent | What It Prevented |
|---|---|---|
| SP-004 (Enrichment Path) | 30 min | Building an entire integration against Spotify's deprecated Audio Features API |
| SP-005 (Audio Playback) | 2 hours | Proved Deezer previews work + CORS needs backend proxy, before writing production code |
| SP-006 (SoundCloud) | 2 hours | Confirmed OAuth flow + preview URL format before committing to integration |
| SP-002 (Flutter Audio) | Research only | Identified CORS risk and crossfade complexity before choosing audio architecture |

SP-007 (LLM Self-Verification Loop) is the most meta example: it is a spike to test whether adding a second-pass "fact-checker" prompt reduces the rate at which Claude hallucinates track attributions (e.g., suggesting "Jeff Mills - Cyclotron" -- a release that does not exist). The spike applies the same generate-verify-refine pattern to the LLM's own output. Verification loops all the way down.

### 5. Session Handoffs Survive Catastrophic Failures

During ST-007, two parallel Claude Code sessions hit an EC2 out-of-memory crash. All changes were uncommitted. The `session-handoff.md` document -- updated after each milestone -- was the only record of what each session was building, which files each owned, and what was complete. Recovery took 10 minutes instead of hours. Without it, we would have been reading every modified file trying to reconstruct intent.

This led to two immediate process changes: (1) commit after every completed task (not just at phase boundaries), and (2) written file-ownership protocol before starting parallel sessions.

---

## What Happens When You Skip the Process

UC-019 Phase 2. The builder skipped task decomposition ("I can just code it") and went straight to implementation. Result: **14 thrash cycles** on a column-mismatch cascade. Adding columns to `TrackRow` broke `SELECT *` queries in 5+ places and integration test pools in 2+ places. A 15-minute pre-implementation analysis would have flagged every affected file.

The human had to intervene multiple times: "This is thrashing -- stop and think." The builder was trying config tweaks, workarounds, and debug prints instead of root-cause analysis. One question -- "What error are you actually seeing?" -- identified the root cause immediately.

The lesson: the Forge process costs 15-30 minutes upfront. Skipping it costs hours. Every time.

---

## The Meta-Insight: Ralph Loops at Every Level

The product and the process use the same pattern:

| Level | Generate | Critique | Refine |
|---|---|---|---|
| **Product** (LLM setlist generation) | Claude generates a DJ setlist from a natural language prompt | Verification pass checks for hallucinated tracks, BPM coherence, energy arc | Conversational refinement: "make it darker," "swap track 7" |
| **Development** (agent teams) | Builder agents write code from task specs | Fresh-context critic reads the diff cold, checks plan compliance | Lead assigns fixes, builders apply, critic re-reviews |
| **Planning** (the Forge) | Use case + task decomposition | Devil's advocate finds gaps, scope creep, dependency risks | All critical/high findings fixed before any builder starts |

This is not an analogy. It is the same algorithm. The quality of the output -- whether it is a setlist or a codebase -- is determined by the quality and frequency of verification loops. More loops, tighter loops, independent loops. That is the entire insight.

**Prompt engineering is not about writing better prompts.** It is about building the verification infrastructure around the generation step. A mediocre prompt with a rigorous verification loop produces better results than a brilliant prompt with no verification.

---

## By the Numbers

- **9 steel threads** shipped end-to-end (ST-001 through ST-009)
- **6 spikes** completed, each preventing at least one wrong turn
- **510 tests** (360 backend, 150 frontend), all passing
- **9 pull requests** merged to main, deployed to production at tarab.studio
- **0 merge conflicts** after adopting non-overlapping file ownership (ST-005 onward)
- **0 context rot errors** after mandating multi-agent teams (ST-005 onward)
- **5 retrospectives** with lessons fed back into the process after each milestone
- **14 use cases** documented in Cockburn style before implementation
- **3 audio preview sources** integrated (Deezer, iTunes, SoundCloud) with automatic fallback chain
- Critic agents caught **6+ HIGH-severity bugs** that builders missed across 4 steel threads
- Devil's advocate reviews caught **3 CRITICAL issues** in a single plan review (ST-006)

All of this was built by one human directing AI agents, following a process that enforces verification at every step.

---

## What This Means for How We Build

1. **Invest in process, not speed.** The teams that ship the fastest are not the ones generating the most code. They are the ones that catch errors earliest. A 15-minute devil's advocate review has higher ROI than any tooling improvement.

2. **Separate generation from verification.** The agent (or person) who writes code should not be the only one reviewing it. Fresh context is not a luxury -- it is a structural requirement for catching a class of bugs that self-review cannot reach.

3. **Spike unknowns before building.** Thirty minutes of research prevented days of building against deprecated APIs. "Research before hacking" sounds obvious. It is also the step most often skipped when deadlines are tight.

4. **Make the process fractal.** The same generate-critique-refine loop works at every scale: individual functions, features, architecture decisions, product strategy. If you only verify at one level, bugs leak through at the others.

5. **Write things down.** Session handoffs, retrospectives, file-ownership protocols -- these artifacts are not bureaucracy. They are the difference between a 10-minute recovery and starting over from scratch.

The prompt is not the product. The loop is the product.
