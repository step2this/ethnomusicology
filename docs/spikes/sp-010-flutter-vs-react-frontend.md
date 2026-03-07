# SP-010: Flutter Web vs React/Next.js Frontend Analysis

**Date**: 2026-03-07
**Status**: COMPLETE
**Time-box**: Research spike (no code changes)

## Context

The Ethnomusicology project (tarab.studio) is a DJ-first music platform with a Rust/Axum backend and a Flutter web frontend. The user's primary frustrations are:

1. **Build times**: ~4 minutes for production web builds
2. **E2e testing**: No e2e tests yet; Playwright is hard with Flutter's canvas rendering
3. **Web-first audience**: DJs use laptops at gigs; no mobile deployment exists yet

This spike evaluates whether to stay with Flutter web, migrate to React/Next.js, or pursue a hybrid approach.

---

## 1. Current Frontend Inventory

### Codebase Size

| Metric | Count |
|--------|-------|
| Dart source files (`lib/`) | 42 |
| Lines of code (`lib/`) | 5,668 |
| Test files (`test/`) | 17 |
| Test LOC | 3,686 |
| Total tests | 156 |
| Screens | 7 (Home, SpotifyImport, TrackCatalog, SetlistGeneration, SetlistLibrary, CrateLibrary, CrateDetail) |
| Widgets | 10 (SetlistTrackTile, SetlistInputForm, SetlistResultView, TransportControls, PurchaseLinkPanel, etc.) |
| Providers (state) | 8 (setlist, audio, deezer, refinement, spotifyImport, trackCatalog, crateProvider, setlistLibrary) |
| Models | 7 (Track, Setlist, SetlistTrack, Crate, Refinement, PurchaseLink, TrackListResponse) |
| Routes | 8 |

### Dependencies (minimal)

- `dio` — HTTP client
- `flutter_riverpod` — state management
- `go_router` — routing
- `google_fonts` — typography
- `url_launcher` — external URL opening
- `web` — Web Audio API (dart:js_interop)

### Flutter-Specific Code

| Feature | Complexity | React Equivalent |
|---------|-----------|-----------------|
| Web Audio API (`audio_service_web.dart`, 140 LOC) | Medium — uses `dart:js_interop` + `package:web` with conditional imports | Trivial in JS/TS — native Web Audio API, no interop needed |
| Conditional import (web/stub) | Flutter-specific pattern for platform targeting | Not needed in React (web-only) |
| Riverpod state management (8 providers, ~1,200 LOC) | Medium — Notifier + NotifierProvider pattern | Direct mapping to Zustand/Jotai stores or TanStack Query |
| Material 3 theme (`theme.dart`) | Low — standard theme config | shadcn/ui or Radix UI + Tailwind |
| GoRouter (8 routes) | Low | Next.js file-based routing or React Router |

**Verdict**: No Flutter-specific features that would be hard to replicate. The Web Audio code is actually *simpler* in native JS/TS. The most complex screen (`SetlistGenerationScreen`, 241 LOC) is straightforward CRUD + state management.

---

## 2. Flutter Web: Current State (March 2026)

### What's Improved Since Project Start

| Feature | Status | Details |
|---------|--------|---------|
| **Hot reload on web** | STABLE (Flutter 3.35+) | Graduated from experimental in Q3 2025. Stateful hot reload now works on web, matching mobile DX. Our Flutter 3.41.2 has this. |
| **WebAssembly (Wasm)** | Production-ready | Skwasm renderer: 2-3x faster graphics than JS-CanvasKit, 40% faster load times, 30% less memory |
| **HTML renderer** | Deprecated | Removed in favor of CanvasKit/Skwasm — all rendering is canvas-based |
| **Semantics tree** | 80% faster build | Flutter 3.32+ optimized semantics compilation, 30% frame time reduction when semantics enabled |

### Build Times

- **Dev server start**: ~15-30 seconds (acceptable with hot reload now working)
- **Production build** (`flutter build web`): **~4 minutes** (this is the pain point)
- **Wasm build**: Comparable or slightly slower than JS build, but produces faster runtime
- **Incremental dev builds**: Fast with hot reload (sub-second for widget changes)

**Key insight**: The 4-minute build time is a *production build* issue, not a dev-loop issue. With hot reload now stable on web, the dev iteration speed is no longer the bottleneck it was when the project started.

### E2e Testing with Flutter Web

| Approach | Viability | Notes |
|----------|-----------|-------|
| **Playwright + semantics tree** | Possible but fragile | Flutter generates invisible ARIA tree alongside canvas. Playwright can query it via `getByRole()`, `getByLabel()`. Requires `SemanticsBinding.ensureInitialized()` and semantic labels on key widgets. |
| **Flutter integration_test** | Works but limited | Runs in Flutter's own test runner, not a real browser. Can't test cross-origin, real network, or browser-specific behavior. |
| **Playwright + screenshot comparison** | Crude but works | Visual regression testing. No semantic interaction. |
| **AI-driven test tools (Autonoma, DevAssure)** | Emerging | Computer vision approaches that "see" the canvas. Experimental in 2026. |

**Verdict**: Playwright with Flutter web is *possible* via the semantics/ARIA layer, but it's an indirect, non-standard approach. You're testing an accessibility shadow DOM, not the actual UI. Every widget you want to test needs explicit semantic labels. This is fundamentally worse than testing a real DOM.

---

## 3. React/Next.js: What You'd Get

### Development Velocity

| Metric | Next.js (Turbopack) | Flutter Web (3.41) |
|--------|---------------------|---------------------|
| **Dev server cold start** | 1-3 seconds | 15-30 seconds |
| **HMR (hot module replacement)** | <100ms (instantaneous) | Sub-second (hot reload, stable since 3.35) |
| **Production build** | 10-30 seconds (typical) | ~4 minutes |
| **Type safety** | TypeScript (excellent) | Dart (excellent) |
| **Rendering** | Real DOM | Canvas (CanvasKit/Skwasm) |

### E2e Testing

| Aspect | React + Playwright | Flutter + Playwright |
|--------|-------------------|---------------------|
| **DOM selectors** | Native — `getByText()`, `getByRole()`, CSS selectors | Indirect — ARIA shadow tree only |
| **Interaction fidelity** | Click, type, drag on real elements | Click on canvas coordinates or ARIA proxy |
| **Setup complexity** | `npm init playwright` — done | Need semantics labels, special configuration |
| **CI integration** | First-class (GitHub Actions template exists) | Manual setup, less documented |
| **Community examples** | Thousands | Handful |
| **Debugging** | Browser DevTools, DOM inspection | Canvas — no element inspection |

### Component Libraries

For a DJ-focused music platform, the React ecosystem offers:

- **shadcn/ui + Radix**: Copy-paste components, full code ownership, Tailwind styling. Highly customizable — ideal for a distinctive DJ aesthetic (dark themes, waveforms, etc.)
- **TanStack Query**: Server state management (caching, refetching, optimistic updates) — replaces most of the manual provider logic
- **Zustand or Jotai**: Client state — simpler than Riverpod, similar concepts
- **Framer Motion**: Animations — far richer than Flutter web's animation support in browsers

### The React Ecosystem Advantage for This Project

The DJ use case benefits from:
1. **Web Audio API is native** — no interop layer, full browser API access
2. **Canvas/WebGL for waveforms** — if you later want waveform visualization, React + canvas is the standard approach
3. **SEO** (if ever needed) — Next.js SSR/SSG, Flutter web has none
4. **Browser DevTools** — inspectable DOM, network tab, performance profiling work naturally

---

## 4. Migration Feasibility

### What Would Change

| Layer | Current (Flutter) | Target (React/Next.js) | Effort |
|-------|-------------------|----------------------|--------|
| **API client** | Dio (276 LOC) | fetch/axios + TanStack Query | ~200 LOC, straightforward |
| **Models** | Dart classes (7 files, ~500 LOC) | TypeScript interfaces | ~300 LOC, mostly mechanical |
| **State management** | Riverpod providers (8 files, ~1,200 LOC) | TanStack Query + Zustand | ~800 LOC, conceptual mapping |
| **Screens** | 7 screens (~1,500 LOC) | React components | ~1,200 LOC |
| **Widgets** | 10 widgets (~2,000 LOC) | React components | ~1,500 LOC |
| **Audio playback** | dart:js_interop Web Audio (140 LOC) | Native Web Audio API | ~80 LOC (simpler) |
| **Routing** | GoRouter (65 LOC) | Next.js file-based or React Router | ~0 LOC (convention-based) |
| **Theme** | Material 3 theme.dart | Tailwind config + shadcn theme | ~50 LOC |
| **Tests** | 156 widget/unit tests (3,686 LOC) | Jest/Vitest + React Testing Library + Playwright | Need to rewrite |

**Estimated total effort**: ~4,000-5,000 LOC of new React/TS code to replace ~5,700 LOC of Dart. The React version would likely be slightly smaller due to less boilerplate (no conditional imports, no explicit state classes with copyWith).

### Migration Strategy (if chosen)

**Option A: Big bang rewrite** (1-2 weeks for this codebase size)
- Build React frontend from scratch against the same `/api/*` endpoints
- Run both frontends during transition (Caddy can route by path or subdomain)
- Cut over when React frontend reaches feature parity
- Risk: 1-2 weeks of no new features

**Option B: Parallel development** (ongoing)
- Start new features in React, keep Flutter for existing screens
- Gradually migrate screens as they need changes
- Risk: maintaining two frontends increases complexity

**Recommendation if migrating**: Option A. The codebase is small enough (42 files, 5.7k LOC) that a focused rewrite takes less time than maintaining two frontends. At this size, a rewrite is a 1-week sprint, not a multi-month project.

---

## 5. The Hybrid Option

**Keep Flutter for mobile, React for web.**

| Aspect | Assessment |
|--------|-----------|
| **Feasibility** | Yes — both talk to the same Rust REST API |
| **Maintenance cost** | High — two UIs to maintain, two sets of tests, two build pipelines |
| **When it makes sense** | When mobile is a real, active target with specific native needs (push notifications, offline, app store presence) |
| **Current relevance** | Low — no mobile deployment exists, no iOS testing done, DJ audience is web-first |

**Verdict**: Premature optimization. If mobile becomes a real requirement, evaluate then. A responsive React web app covers most "mobile" use cases for a DJ tool (phone browser at gigs). Native mobile only matters for push notifications and offline setlists — both are post-MVP concerns.

---

## 6. Risks of Each Path

### Risk: Stay with Flutter Web

| Risk | Severity | Mitigation |
|------|----------|------------|
| E2e tests remain hard to write | **High** | Invest in semantics labels + Playwright ARIA approach (fragile but workable) |
| 4-min production builds slow CI | **Medium** | Caching, Wasm builds may improve. Not a dev-loop blocker since hot reload works. |
| Smaller web talent pool | **Medium** | Dart/Flutter web developers are harder to find than React/TS developers |
| Canvas rendering limits browser tooling | **Low** | Acceptable for current team size |
| No SSR/SEO path | **Low** | Not needed for a DJ tool |

### Risk: Migrate to React/Next.js

| Risk | Severity | Mitigation |
|------|----------|------------|
| Re-introduce bugs during rewrite | **Medium** | Port tests alongside code. The existing 156 tests define expected behavior. |
| 1-2 week feature freeze | **Medium** | Acceptable at current stage (pre-launch, no paying users) |
| Lose mobile optionality | **Low** | React Native or responsive web covers most cases. Flutter mobile was always theoretical. |
| New framework learning curve | **Low** | React/TS is well-documented; team has JS/TS experience |

---

## 7. Decision Matrix

| Criteria (weighted) | Flutter Web | React/Next.js | Notes |
|---------------------|:-----------:|:-------------:|-------|
| **Dev iteration speed** (25%) | 8/10 | 9/10 | Flutter hot reload closed the gap; Next.js still faster cold starts and builds |
| **E2e testability** (25%) | 4/10 | 9/10 | This is the biggest gap. Canvas-based testing is fundamentally harder. |
| **Production build speed** (15%) | 4/10 | 9/10 | 4 min vs 10-30 sec |
| **Component ecosystem** (10%) | 7/10 | 9/10 | shadcn/Radix + Tailwind > Material 3 for custom DJ aesthetic |
| **Web Audio / browser APIs** (10%) | 5/10 | 10/10 | Native JS vs dart:js_interop — no contest |
| **Mobile optionality** (5%) | 9/10 | 6/10 | Flutter's mobile story is excellent; React Native exists but is a separate framework |
| **Migration effort** (10%) | 10/10 | 6/10 | Staying = zero effort; migrating = 1-2 weeks |

**Weighted scores**:
- Flutter Web: **6.2/10**
- React/Next.js: **8.5/10**

---

## 8. Recommendation

**Migrate to React/Next.js**, prioritized by the e2e testing gap and production build speed.

### Rationale

1. **E2e testing is the decisive factor.** The project is at a stage where e2e tests are critical for confidence in deployments. Flutter web's canvas rendering makes Playwright testing an indirect, fragile exercise. React gives you first-class DOM-based testing with standard selectors. This alone justifies the migration.

2. **The codebase is small enough to migrate cleanly.** At 42 files and 5.7k LOC, this is a 1-2 week rewrite, not a multi-month project. The API contract doesn't change — only the UI layer moves.

3. **Production build speed matters for CI/CD.** 4-minute builds add up across CI runs. Next.js builds in 10-30 seconds.

4. **Flutter's web hot reload improvement reduces but doesn't eliminate the DX gap.** Hot reload on web is now stable (good news), but the canvas rendering model, browser tooling limitations, and indirect JS interop remain structural disadvantages for a web-first product.

5. **Mobile is theoretical, web is actual.** Flutter was chosen for cross-platform optionality, but no mobile deployment has been attempted. The DJ audience uses laptops. If mobile becomes real, a responsive React web app or React Native is a viable path.

### What NOT to do

- Don't pursue the hybrid approach (two frontends for one developer is a maintenance trap)
- Don't attempt a "gradual migration" — at this codebase size, a clean rewrite is faster and less risky than maintaining two frameworks
- Don't block on this decision — the Flutter frontend works and ships features. This migration is a strategic improvement, not an emergency.

### Suggested Tech Stack (if migrating)

```
Framework:     Next.js 16 (App Router)
Language:      TypeScript (strict mode)
Styling:       Tailwind CSS 4
Components:    shadcn/ui (Radix primitives)
State:         TanStack Query (server state) + Zustand (client state)
Audio:         Native Web Audio API
Testing:       Vitest (unit) + React Testing Library (component) + Playwright (e2e)
Build:         Turbopack (dev) + default Next.js (prod)
```

---

## Sources

- [Flutter Web Renderers](https://docs.flutter.dev/platform-integration/web/renderers)
- [Flutter Web Hot Reload — Flutter 3.32](https://blog.flutter.dev/whats-new-in-flutter-3-32-40c1086bab6e)
- [Flutter 3.35 — Hot Reload Graduated](https://blog.flutter.dev/whats-new-in-flutter-3-35-c58ef72e3766)
- [State of Flutter 2026](https://devnewsletter.com/p/state-of-flutter-2026/)
- [Flutter Web Performance: CanvasKit vs Wasm](https://coldfusion-example.blogspot.com/2026/01/flutter-web-performance-2025-canvaskit.html)
- [Playwright Flutter Web Testing Guide](https://www.getautonoma.com/blog/flutter-playwright-testing-guide)
- [Flutter Web Accessibility (Semantics)](https://docs.flutter.dev/ui/accessibility/web-accessibility)
- [Next.js 15 Features](https://nextjs.org/blog/next-15)
- [Turbopack in 2026](https://dev.to/pockit_tools/turbopack-in-2026-the-complete-guide-to-nextjss-rust-powered-bundler-oda)
- [shadcn/ui vs Material UI Comparison](https://djangostars.com/blog/shadcn-ui-and-material-design-comparison/)
- [Best React Component Libraries 2026](https://designrevision.com/blog/best-react-component-libraries)
- [Flutter Web Playwright Automation](https://www.devassure.io/blog/flutter-web-automation-devassure/)
