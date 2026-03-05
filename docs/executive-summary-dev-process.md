# Verification Loops: A Force Multiplier for Building with AI

**tarab.studio** is a DJ platform that generates setlists from natural-language prompts, sources music from three streaming services, and arranges tracks for harmonic mixing. One engineer built and shipped it in under four weeks using AI coding agents -- not by moving fast and breaking things, but by moving deliberately and catching mistakes early.

This document is about the methodology behind that result, and why the same pattern that makes the product work also makes the development process work.

---

## The Core Insight

The product asks an AI to generate DJ setlists. Left unchecked, the AI hallucinates tracks that do not exist, ignores tempo constraints, and produces sets that no working DJ would play. The fix is not a better prompt. The fix is a verification loop: generate a setlist, critique it against known constraints, refine based on the critique. The output quality is proportional to the rigor of the loop, not the cleverness of the initial prompt.

The same principle applies to building software with AI agents. An AI agent can write a feature in minutes. Without independent review, that feature ships with silent data corruption, crashes on non-English text, and advertises capabilities that were never wired up. The fix is the same: generate code, critique it with fresh eyes, refine before shipping.

We call this pattern -- generate, critique, refine -- a verification loop, and it operates at every level of the system:

| Level | Generate | Critique | Refine |
|-------|----------|----------|--------|
| **Product** | AI generates a setlist from a DJ's prompt | Validation checks for hallucinated tracks, tempo coherence, energy flow | DJ refines conversationally: "make it darker," "swap track 7" |
| **Code** | AI agents write features from task specs | A separate reviewer reads the code cold, with no prior context | Targeted fixes based on specific findings |
| **Planning** | Feature requirements and task breakdowns | Adversarial review challenges assumptions before any code is written | Plans updated to address gaps before implementation begins |

This is not an analogy. It is the same algorithm applied recursively. The insight is that verification infrastructure matters more than generation quality -- at every level.

---

## How It Works in Practice

Every feature begins with a written use case that defines what "done" looks like in measurable terms -- for example, *"Given a prompt 'deep house for sunset,' the system returns 12-15 tracks with BPM within 118-128 and Camelot-compatible keys for adjacent tracks."* Before implementation starts, an adversarial review challenges the plan: Are there naming conflicts with existing code? Will the test infrastructure support the new feature? Has out-of-scope work crept in? A typical finding looks like: *"CRITICAL: `ContentBlock` is already defined in the Claude API response module. The new request module redefines it, causing a name collision that will fail compilation across all six builder agents."* In one case, this fifteen-minute review caught three such issues that would have blocked an entire team of six agents working in parallel -- saving an estimated four to six hours of rework.

When unknowns exist, we research before building. A thirty-minute investigation confirmed that a critical third-party API had been deprecated months earlier. Without that check, the team would have built an integration, deployed it, received errors in production, and then pivoted -- losing days. Six such investigations were conducted over the project's lifetime, and each one prevented at least one wrong turn.

The reviewed plan is decomposed into tasks with an explicit dependency graph -- for instance, *T1 (database schema) and T2 (Camelot wheel algorithm) can run in parallel; T5 (route handler) blocks on both.* Implementation is then distributed across multiple AI agents working on non-overlapping areas of the codebase. One agent owns `services/camelot.rs`, another owns `services/enrichment.rs`, a third owns `routes/setlist.rs`. They share a trait interface at the boundary so they never block each other and never touch the same file. A coordinating agent reviews boundaries and connects the pieces but does not write implementation code itself. This separation was introduced after the first major feature -- built by a single agent writing 2,800 lines across fourteen files -- degraded in quality as the session progressed. The agent that started the session making zero errors per task was averaging five correction cycles per task by the end. Distributing work across fresh agents eliminated this degradation entirely.

After implementation, a separate critic examines the work from scratch, with no knowledge of how the code was written. A real finding: *"services/setlist.rs:412 -- `compute_seed_match_count` is defined and tested but never called from `generate_setlist`. The enrichment feature described in the plan is dead code."* The critic also checks plan-vs-code compliance: did every planned capability actually get wired up? This reviewer has caught bugs that no amount of self-review would find:

- A text-processing function that would crash on Arabic music titles -- the product's primary use case -- because it split on byte boundaries instead of character boundaries
- A core feature that was defined, tested in isolation, but never connected to the rest of the system, meaning it would have shipped as dead code
- A user-facing input flow that passed raw data where a processed identifier was expected, guaranteeing an error on first use
- A track-ordering bug that silently placed the first track last in every setlist

These are not obscure edge cases. They are the kind of mistakes that erode user trust on day one. The reviewer catches them because fresh perspective identifies what familiarity overlooks. Only after the critic approves does verification run: each postcondition from the original use case is checked mechanically -- *"POST /api/setlist with verify=true returns tracks where every verification_flag is either null or 'replaced'; no 'no_such_track' flags survive."*

---

## The Loop in Code

The verification loop is not just a process diagram -- it compiles. Here is the core of the `verify_setlist` function (Rust, simplified):

```rust
pub async fn verify_setlist(
    claude: &dyn ClaudeClientTrait,   // trait-based: real API or test mock
    tracks: &[SetlistTrackResponse],
) -> Result<Vec<SetlistTrackResponse>> {
    let response = claude.generate_setlist(VERIFICATION_PROMPT, ...).await?;
    let verification: VerificationResponse = serde_json::from_str(&response)?;

    let mut verified = tracks.to_vec();
    for track in &mut verified {
        if let Some(v) = verify_map.get(&track.position) {
            if matches!(v.flag.as_deref(),
                Some("wrong_artist") | Some("no_such_track") | Some("constructed_title")
            ) {
                track.confidence = Some("low".to_string());  // downgrade, never upgrade
                track.verification_note = v.correction.clone();
            }
        }
    }
    Ok(verified)
}
```

Three things make this work. First, `ClaudeClientTrait` means the entire LLM call is mockable -- tests inject a `MockClaude` that returns canned JSON, so verification logic runs in milliseconds without network calls. Second, the confidence field is normalized on input (`filter(|c| matches!(c.as_str(), "high" | "medium" | "low"))`) and only ever downgraded by verification, never upgraded -- a ratchet that prevents the system from inflating its own certainty. Third, a 30-line skill document (`include_str!("../prompts/music_skill.md")`) is compiled into the binary and injected into every LLM system prompt, teaching the model the three rules of track verification before it generates anything. The verification loop is not bolted on after the fact. It is load-bearing architecture.

---

## What Happens Without the Process

Midway through the project, one feature skipped the planning and decomposition steps. The rationale was efficiency: "I can just code it." The result was fourteen correction cycles on a single database change that cascaded across five dependent components. A fifteen-minute analysis beforehand would have identified every affected area. The human operator intervened twice to redirect the work. The feature that was supposed to be faster ended up taking longer than any properly planned feature in the project.

This was not an isolated incident. It was a controlled experiment with a clear result: the process costs fifteen to thirty minutes upfront and saves hours downstream. Every time.

---

## Results

Over four weeks, the methodology produced:

- **Nine end-to-end features** shipped to production, each proving a complete path through the system -- from user input through AI generation through data persistence through the interface
- **510 automated tests** across backend and frontend, all passing
- **Zero merge conflicts** after adopting distributed ownership (across five team efforts)
- **Zero quality degradation** in later tasks after distributing work across focused agents
- **Six pre-build investigations** that each prevented at least one costly wrong turn
- **Six high-severity bugs** caught by independent review that would have shipped otherwise
- **Three critical plan defects** caught in a single fifteen-minute review before any code was written
- **Three audio sources** integrated with automatic fallback, providing broad music catalog coverage
- **One crash recovery** completed in ten minutes using written handoff documentation that would have otherwise required hours of forensic reconstruction

The product is live, serving real requests, with a full preview playback chain and conversational AI refinement.

---

## Takeaways

**Verification is more valuable than generation.** The teams and individuals that ship the highest-quality work are not the ones producing code the fastest. They are the ones catching errors the earliest. A fifteen-minute adversarial review of a plan has higher return on investment than any tooling improvement.

**Fresh perspective is a structural requirement, not a nice-to-have.** The person or system that creates something cannot effectively review it. This is not a discipline problem -- it is a limitation of shared context. Independent review catches a fundamentally different class of defects.

**Research prevents expensive pivots.** Half an hour of investigation before committing to an approach consistently prevented days of wasted implementation. "Look before you leap" sounds obvious. It is also the step most often skipped under deadline pressure.

**Written artifacts are recovery infrastructure.** Session notes, handoff documents, and ownership records are not bureaucracy. When a system failure wiped out an active work session, these documents were the difference between a ten-minute recovery and starting over.

**The pattern is fractal.** Generate, critique, refine works at every scale -- individual functions, features, architecture decisions, product strategy. If verification only happens at one level, defects leak through at the others.

The prompt is not the product. The loop around the prompt is the product.
