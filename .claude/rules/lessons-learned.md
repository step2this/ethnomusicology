---
paths:
  - "docs/**"
  - ".claude/agents/**"
---

# Lessons Learned

## UC-001
- Two backend agents can work in parallel if they own separate directories (e.g., `src/api/` vs `src/db/` or `src/services/`)
- Agents handle `mod.rs` creation and module wiring well when given clear boundaries
- Define shared traits up front in prompts so agents produce compatible interfaces
- Run `cargo check` before spawning agents to warm the dependency cache — saves 30s+ per agent
- Frontend can be done sequentially after backend since it depends on API shapes
- `package:web` fails in Flutter test VM — use `url_launcher` for cross-platform URL handling

## Walking Skeleton
- Spikes before steel threads: SP-001/002/003 revealed constraints (container sizing, CORS, key format) that would have caused rework mid-implementation
- Error response shape must be agreed early: flat `{"error": "msg"}` vs nested `{"error": {"code": "...", "message": "..."}}` spans every endpoint — fix once before building more
- Migration 003 added DJ metadata columns (bpm, camelot_key, energy, source). Migration 005 added enrichment tracking. Tracks are now populated via LLM enrichment (ST-005).
- Artist data is relational: track API must JOIN track_artists + artists tables and concatenate names
- API contract review gate works: both frontend and backend agents must confirm OpenAPI spec before writing implementation code
- Research-only spikes are valid: SP-002 produced actionable decisions without a prototype

## ST-003
- **Lead-as-solo-builder is an anti-pattern**: Lead implemented all 9 tasks (~2,800 lines, 14 new files) solo despite plan specifying 4 parallel agents. By task T8-T9 (frontend), context was exhausted — chasing formatting errors, wrong package names, assertion mismatches that a fresh agent would catch instantly.
- **Pre-commit hook was incomplete**: Only ran backend checks (cargo fmt/clippy/test). Flutter analyze/test was NOT enforced. Fixed: hook now runs both backend and frontend gates.
- **In-session reviewer is theater**: The Reviewer role watches code being written in the same session. It cannot catch what it already "knows" — this is the self-review blind spot. Replaced with Critic Agent pattern: fresh context, reads diff cold, finds what the builder missed.
- **Pure-function modules are perfect subagent targets**: camelot.rs, arrangement.rs — zero IO, deterministic, isolated. These should always be delegated to separate agents.
- **Ralph Wiggum loops fit pure modules**: `while true; do claude; done` with test backpressure works for isolated, well-tested modules.
- **Context rot is real and measurable**: Early tasks (T1-T3) produced clean code on first try. Later tasks (T8-T9) required 5+ fix iterations.
- **Generator-Critic separation is essential**: The agent that writes code should not be the only one reviewing it.

## ST-005
- **Multi-agent teams work when file boundaries are clean**: 5 parallel builders on non-overlapping files = zero merge conflicts.
- **Critic review catches real bugs**: Fresh-context critic found position=0 off-by-one and missing BPM/energy range validation — both would cause silent data corruption.
- **Spike-first saves real time**: SP-004 confirmed Spotify Audio Features is deprecated before building around it.
- **Plan-vs-code compliance gap**: Auto-enrich trigger was in the plan but dropped during task decomposition. Critic should explicitly check plan items against implementation.
- **Merge dependencies before branching**: Merge all pending PRs to main before creating the next feature branch.
- **Git worktrees for cross-branch fixes**: Fixed ST-004 E2E tests on a worktree while ST-005 builders ran undisturbed.

## ST-006
- **Devil's advocate on task plans is highest-ROI quality step**: 3 CRITICAL issues caught before any builder started (ContentBlock name collision, missing test pool migration, scope creep). Cost to fix at plan time: minutes. Mid-implementation: hours of rework across multiple builders.
- **Combine tasks that touch the same files into one builder**: T6+T8+T9 all touched `services/setlist.rs` and `routes/setlist.rs`. One builder handling all 3 sequentially was cleaner than 3 builders fighting over the same files.
- **Plan-vs-code compliance check works**: Adding explicit postcondition checking to the critic prompt caught `compute_seed_match_count` being defined and tested but never called — postcondition 13 would have shipped unmet.
- **Duplicate test helpers are a recurring trap**: `create_test_pool()` exists in both `db/mod.rs` and `tests/setlist_api_test.rs`. Adding a migration to one but not the other creates silent failures. Bitten in ST-005 and ST-006. Need single canonical implementation.
- **Commit per-phase, not monolithic**: 27 files and ~5,600 lines in one commit makes bisect impossible and PR review painful. Each phase or builder should produce its own commit.
- **Shut down idle builders proactively**: Builders that finish their phase and sit idle consume notification bandwidth and cause confusion. Shut them down as soon as their tasks complete.

## Phase 8 Session (2026-03-05)

- **A single critic review step is insufficient for a multi-language project.** Security/architecture review and code quality review are different skills requiring different checklists. One critic cannot be expert in security AND Rust idioms AND Flutter patterns simultaneously — depth suffers across all dimensions. The fix: two mandatory passes (7a security/arch, 7b language quality) with separate checklists and separate fresh-context agents.

- **Flutter routing completeness is a CRITICAL review item.** The `SetlistDetailScreen` was implemented but never registered in `lib/config/routes.dart`. Users could generate and save setlists but could never view them from the library — a dead feature that would have shipped to production. The 7b code quality checklist now mandates: "every new screen class MUST appear in the router." This is the single highest-value Flutter-specific check.

- **"Just cleanup" PRs need review too.** PR #11 (tech debt, 13 items across backend and frontend) had NO critic review — it was treated as "just small fixes." The assumption that small refactors are safe is the same assumption that lets typos become outages. The updated process: every PR gets both passes, no exceptions including tech debt, hotfixes, and doc updates that touch code.

- **Code quality review catches different bugs than security review.** A security critic looks for auth bypass, injection, data leaks. A code quality reviewer looks for unreachable screens, missing error states, broken navigation, and incorrect provider access patterns. These are orthogonal. ST-007 was the only prior session with a separate frontend critic, and it produced the cleanest Flutter code in the project. That was not a coincidence — it was a process win that wasn't codified.

- **The user should not be the quality gate.** The entire point of the Forge workflow is that process steps are automatic. When the user must say "wait, did you do a code quality review?" the process has failed. The checklists in 7a and 7b must be explicit enough that the agent follows them without prompting.

- **Devil's Advocate on Phase 8 plan caught 5 issues pre-build.** SQLite FK enforcement gap, Spotify CC flow needing scratch build, models.rs ownership split, multi-table delete ordering, parallel audio search optimization — all fixed before any builder touched code. The devil's advocate step continues to be the highest-ROI quality gate in the Forge.

## Spike Findings Summary

| Spike | Hypothesis | Result | Key Decision |
|-------|-----------|--------|-------------|
| SP-001 Beatport | v4 API provides BPM/key with usable access | Partially confirmed | OAuth2 w/ public client_id scraping. BPM=integer, key=shortName (needs Camelot map). Rate limits unknown — throttle conservatively |
| SP-002 Flutter Audio | just_audio plays Spotify previews in Chrome | Partially confirmed | CORS high risk (may need backend proxy). Crossfade is manual 2-player impl. `audioplayers` preferred over `just_audio` for stability |
| SP-003 essentia | Sidecar extracts BPM/key with <5s latency | Partially confirmed | Needs 1-2 GB container (not 512 MB). Async queue required. Key = separate note + scale strings. Use TempoCNN for 30s previews |
| SP-004 Enrichment Path | Spotify Audio Features + LLM estimation viable | Confirmed (LLM only) | Spotify Audio Features deprecated Nov 2024 for dev-mode apps. LLM estimation is primary. `from_spotify_key()` + `from_notation()` ready for future sources |
| SP-005 Audio Playback | Deezer previews viable; crossfade feasible | Confirmed (sequential chosen) | Deezer 30s previews work via Web Audio API + backend proxy. Crossfade PoC proven but removed — too complex for 30s clips. Simple sequential playback is better UX. |
| SP-006 SoundCloud API | Client Credentials + preview URL accessible | Confirmed | `preview_mp3_128_url` confirmed on track objects. OAuth 2.1 Client Credentials flow works. CDN redirect (302) must be resolved server-side due to CORS. |
| SP-007 LLM Self-Verification | Second-pass LLM reduces hallucination | Confirmed | Skill doc injection (`music_skill.md`) reduces fabricated tracks. Confidence field has predictive value: high ≈ 90% real, medium ≈ 25% real, low = creative suggestion. Prompt caching on skill doc critical for cost. |
