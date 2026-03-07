# Implementation Plan: SP-009 Purchase Link Spike + Phase 7

## Status Legend
- [ ] Pending
- [x] Complete
- [!] Blocked (see notes)

## Pre-flight
- [x] **P0: Update stale artifacts** — Update MEMORY.md (ST-010 COMPLETE, Phase 8 COMPLETE PR #14, test counts 365+156, current date Mar 7), session-handoff.md (rewrite for current session), mvp-progress.md (Phase 8 postconditions done). Acceptance: all three files reflect current state.
  - Files: `~/.claude/projects/-home-ubuntu-ethnomusicology/memory/MEMORY.md`, `docs/session-handoff.md`, `docs/mvp-progress.md`

## SP-009: Purchase Link Store Viability Spike

- [x] **S1: URL pattern verification** — Test search URL templates for Beatport, Traxsource, Juno Download, Bandcamp using WebFetch. For each store: (a) construct search URL for "Derrick May Strings of Life", (b) fetch the page, (c) confirm results appear. Also check Traxsource API at docs.api.traxsource.com — is it public or gated? Record working URL templates. Acceptance: documented URL template per store (or "BLOCKED" with reason).
  - Files: `docs/spikes/sp-009-purchase-link-store-viability.md` (CREATE)
  - Output: Write findings to spike doc

- [x] **S2: Coverage test (5 underground tracks)** — Search these tracks on each store via WebFetch: (1) Objekt - Theme from Q, (2) Shackleton - Blood on My Hands (Villalobos Remix), (3) Omar S - 002, (4) re:ni - Ciste, (5) DJ Stingray 313 - Molecular Enhancement. Record HIT/PARTIAL/MISS per store (HIT = correct track visible). Acceptance: 5x4+ coverage matrix in spike doc.
  - Files: `docs/spikes/sp-009-purchase-link-store-viability.md` (APPEND)

- [x] **S3: Coverage test (5 varied tracks)** — Search: (1) Derrick May - Strings of Life, (2) Peggy Gou - Starry Night, (3) DJ Rashad - Feelin, (4) Lena Willikens - Phantom Delia, (5) Identified Patient - Body Clock. Record HIT/PARTIAL/MISS. Acceptance: complete 10x coverage matrix, aggregate hit rates per store, store recommendation.
  - Files: `docs/spikes/sp-009-purchase-link-store-viability.md` (APPEND)

- [x] **S4: Affiliate assessment + spike conclusion** — Research affiliate programs for each store (web search). Write spike conclusion: ordered store recommendation, decision on which stores to include, go/no-go on Apple affiliate registration. Update spike status to COMPLETE. Acceptance: spike doc has hypothesis, findings, decision, and feeds-into section.
  - Files: `docs/spikes/sp-009-purchase-link-store-viability.md` (APPEND/FINALIZE)

## Phase 7: UC-020 Update + Review

- [x] **U1: Update UC-020 with spike findings** — Read current UC-020 doc. Update with: validated store list, confirmed URL templates, source-attribution vs purchase-link distinction (devil's advocate finding #2: these are DIFFERENT concerns — keep attribution icons, add purchase panel). Update postconditions. Acceptance: UC-020 reflects spike findings, has clear postconditions.
  - Files: `docs/use-cases/uc-020-generate-purchase-links-for-tracks.md` (EDIT)

- [x] **U2: Devil's advocate review of updated UC** — Read UC-020 cold. Find gaps, missing extensions, untestable postconditions. Write findings inline or in a review section. Fix all CRITICAL/HIGH findings. Acceptance: UC-020 passes review with no CRITICAL/HIGH issues remaining.
  - Files: `docs/use-cases/uc-020-generate-purchase-links-for-tracks.md` (EDIT)

## Phase 7: Task Decomposition

- [x] **D1: Task decomposition** — Break UC-020 into implementable tasks with dependencies. Key architectural decisions to encode:
  - Purchase links are computed ON-DEMAND from title+artist (not persisted) — works for fresh and saved setlists
  - Backend endpoint `GET /api/purchase-links?title=X&artist=Y` returns ordered store links with affiliate tags
  - Frontend: PurchaseLinkPanel widget ALONGSIDE existing source attribution icons (NOT replacing them)
  - Empty state design is critical — underground tracks may not be on any store
  - Acceptance: `docs/tasks/uc-020-tasks.md` created with dependency graph, file ownership, acceptance criteria per task.
  - Files: `docs/tasks/uc-020-tasks.md` (CREATE)

## Phase 7: Design-Crit (MANDATORY)

- [x] **DC1: Design brief for purchase link panel** — Run design-crit brief for the purchase panel component. Key questions: expandable panel vs bottom sheet, store icon treatment, empty state (majority case for underground), visual separation from source attribution icons, how many stores is too many. Acceptance: design brief captured in `.design-crit/`.
  - Files: `.design-crit/` (CREATE files as needed)

- [x] **DC2: Component design facet** — Crit the purchase panel component: interaction pattern, store ordering, icon sizing, mobile responsiveness. Lock the design. Acceptance: design direction locked with component specs. **LOCKED: Option A (Chip Strip)** — horizontal pill-shaped store chips, inline expansion below source attribution, +32px height. User decision.
  - Files: `.design-crit/` (EDIT/CREATE)

## Phase 7: Implementation

- [x] **T1+T2: Purchase link service + affiliate config** — Create `backend/src/services/purchase_links.rs` with `build_purchase_links()` function and `AffiliateConfig`. URL templates: Beatport `/search?q=`, Bandcamp `/search?q=`, Juno `/search/?q[all][0]=`, Traxsource `/search?term=`. Affiliate tags from env vars. Add `pub mod purchase_links;` to `services/mod.rs`. Acceptance: unit tests for full query, title-only, artist-only, empty, special chars, affiliate tags, store order. `cargo fmt --check && cargo clippy -- -D warnings && cargo test` all pass.
  - Files: `backend/src/services/purchase_links.rs` (CREATE), `backend/src/services/mod.rs` (EDIT)

- [x] **T3: Purchase links route + wiring** — Create `backend/src/routes/purchase_links.rs` with `GET /api/purchase-links?title=X&artist=Y` handler. Wire into `main.rs`. Acceptance: endpoint returns 4 store links for valid query, empty links for no params. `cargo test` passes.
  - Files: `backend/src/routes/purchase_links.rs` (CREATE), `backend/src/routes/mod.rs` (EDIT), `backend/src/main.rs` (EDIT)

- [x] **T4: Frontend ApiClient method + model** — Create `frontend/lib/models/purchase_link.dart` with `PurchaseLink` model. Add `getPurchaseLinks()` to `ApiClient`. Acceptance: compiles, `flutter analyze` passes.
  - Files: `frontend/lib/models/purchase_link.dart` (CREATE), `frontend/lib/services/api_client.dart` (EDIT)

- [x] **T5: Backend integration tests** — Add comprehensive tests in `backend/src/services/purchase_links.rs` `#[cfg(test)]` module: full query, title-only, artist-only, both-empty, special chars, affiliate tags, no affiliate tags, store order. Acceptance: all tests pass, clippy clean.
  - Files: `backend/src/services/purchase_links.rs` (EDIT)

- [x] **T6: PurchaseLinkPanel widget** — Create `frontend/lib/widgets/purchase_link_panel.dart`. Collapsed by default (shopping bag icon). On expand: calls API, shows loading, then horizontal row of store buttons. Tap opens URL via `url_launcher`. Empty state when no title/artist. Theme tokens only. Acceptance: renders collapsed, expands with links, `flutter analyze` passes.
  - Files: `frontend/lib/widgets/purchase_link_panel.dart` (CREATE)

- [x] **T7: Wire PurchaseLinkPanel into SetlistTrackTile** — Import and add PurchaseLinkPanel below track info, visually separate from source attribution icons. Acceptance: panel visible on every track tile, existing playback/attribution unchanged, `flutter analyze` passes.
  - Files: `frontend/lib/widgets/setlist_track_tile.dart` (EDIT)

- [x] **T8: Frontend widget tests** — Create `frontend/test/widgets/purchase_link_panel_test.dart`. Tests: renders collapsed, expands with store links, correct store order, URL launch on tap, empty state, API error handling. Acceptance: all tests pass, `flutter analyze` passes.
  - Files: `frontend/test/widgets/purchase_link_panel_test.dart` (CREATE)

## Phase 7: Quality Gates

- [x] **Q1: Two-pass critic review** — 7a: 0 CRITICAL, 1 HIGH (fixed), 3 MEDIUM (1 fixed, 2 known debt). 7b: 0 CRITICAL, 2 MEDIUM (both fixed). All findings addressed. — Run 7a (security/arch) and 7b (code quality) critic passes on the full diff. Fix all CRITICAL/HIGH findings. Acceptance: both critics approve.

- [x] **Q2: Verify UC-020** — ALL PASS. 7/7 postconditions covered, 3/3 invariants verified, 4/4 extensions implemented, 9/9 acceptance criteria pass. 573 total tests. — Run /verify-uc against UC-020 postconditions. All must pass. Acceptance: verification report shows all postconditions met.

## Post-Milestone

- [x] **PM1: Retrospective** — Written: `docs/retrospectives/phase-7-purchase-links.md`
- [x] **PM2: Update progress docs** — Updated: `docs/mvp-progress.md`, `MEMORY.md`, `docs/mvp-roadmap.md`. Phase 7 marked complete.
- [ ] **PM3: Session handoff** — Write `docs/session-handoff.md` for next session continuity.
