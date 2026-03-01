# Accessibility — Round 1 Critique

Auditing the locked design: a dense, dark-themed DJ track catalog with warm amber/copper palette (Ember Crate), 40px rows at studio density, flat table with 14 interactive components, Inter/JetBrains Mono typography, and Wire Lift elevation. Desktop-first, Flutter Web.

This is the final facet — it audits everything accumulated across 9 locked decisions.

---

## Audit Summary

**12 issues found:** 2 critical, 6 major, 4 minor.

| Category | Status |
|---|---|
| Keyboard Navigation | 3 issues (1 critical, 1 major, 1 minor) |
| Screen Reader Support | 4 issues (all major) |
| Touch Targets | 1 issue (minor — desktop-first, deferred) |
| Color & Contrast | 1 issue (minor — energy bars color-only) |
| Motion | Pass (no motion facet locked, minimal animation) |
| Cognitive | Pass (consistent patterns, clear wayfinding) |

### Critical Issues

1. **No keyboard access to row selection** — Track rows are mouse-click only. A keyboard user cannot select a track, expand the detail strip, or navigate between rows. This blocks the core use case. (WCAG 2.1.1)

2. **Slide-over panel lacks focus trap** — When the slide-over opens over the track table with a scrim, keyboard focus is not constrained to the panel. Users can Tab behind the scrim. No ESC key handler to close. (WCAG 2.1.2)

### Major Issues

3. **No skip link** — The 48px icon rail sits before the main content in DOM order. No skip link exists to bypass it. (WCAG 2.4.1)

4. **Focus indicators not defined** — No focus ring specified anywhere in the locked design. Need visible focus rings against all Ember Crate surfaces (#0F0D0B, #141210, #1A1714). (WCAG 2.4.7)

5. **Table lacks semantic markup** — Track table needs `<table role="grid">` with `<th scope="col">` and `aria-sort` attributes on sortable headers. Currently implied as a div-based layout. (WCAG 1.3.1)

6. **Sort state changes not announced** — Clicking a column header to sort gives no auditory feedback. Needs `aria-sort` on `<th>` elements and a live region announcement: "Sorted by BPM, ascending." (WCAG 4.1.3)

7. **Toast notifications need live region** — Import success/error toasts appear visually but don't announce to screen readers. Need `aria-live="polite"` and `role="status"`. (WCAG 4.1.3)

8. **Album art missing alt text** — 32x32 thumbnails have no alt attribute. Need `alt="[Album] by [Artist]"` or `alt=""` if decorative (but album art aids recognition, so it's informational). (WCAG 1.1.1)

### Minor Issues

9. **Energy bars convey info by color alone** — The teal→amber→copper-red gradient encodes energy level visually. No text alternative for color-blind users. A numeric tooltip (e.g., "8/10") would resolve this. (WCAG 1.4.1)

10. **9px source badges at readability limit** — "SPOTIFY" / "BEATPORT" at 9px uppercase is the smallest text in the system. It passes contrast but stresses readability for users with mild vision impairment. (WCAG 1.4.12)

11. **Icon rail buttons need labels** — Icon-only navigation buttons in the 48px rail have no text labels or aria-labels. Screen readers would announce "button" with no context. (WCAG 4.1.2)

12. **Slide-over scrim dismiss needs keyboard** — Clicking the scrim to close the slide-over has no keyboard equivalent. (WCAG 2.1.1)

### Color Contrast Audit

The Ember Crate palette is well-designed for contrast:

| Pair | Ratio | Pass? |
|---|---|---|
| #F0ECE4 on #0F0D0B (primary text) | ~15.2:1 | AA ✓, AAA ✓ |
| #E8963A on #0F0D0B (amber accent) | ~6.4:1 | AA ✓ |
| #8B8580 on #0F0D0B (muted text) | ~4.6:1 | AA ✓ |
| #C06030 on #0F0D0B (copper) | ~4.1:1 | AA ✓ (large text) |
| Camelot chip text on tinted bg | Varies, all >4.5:1 | AA ✓ |

The warm palette works well for accessibility. No critical contrast failures.

### Color Blindness Simulation

- **Protanopia (red-blind):** Amber and copper shift toward yellow-brown. Still distinguishable from teal. Energy gradient loses red→amber distinction but teal remains clearly different.
- **Deuteranopia (green-blind):** Similar to protanopia. Camelot greens (#4A9B6E, #6E9B4A) may become indistinguishable from each other. The text labels (6A vs 7A) differentiate.
- **Tritanopia (blue-blind):** Teal (#5A9EC0) shifts toward green. Blue Camelot keys shift. Text labels remain readable.

All Camelot chips include text labels, so color-blindness does not prevent key identification. The energy bar gradient is the main concern — hence finding #9.

---

## Option A: Minimum Compliance

**Fixes 8 of 12 issues · WCAG AA achieved · Minimal visual disruption**

Addresses all critical and major issues. The locked design changes very little — you add focus rings, semantic markup, a skip link, and keyboard handlers. The visual design the user approved is virtually unchanged.

**What works:** Least disruption to the locked design. Ships accessible. Implementation is straightforward — mostly HTML attributes and keyboard event handlers. The user who locked Ember Crate would recognize the design without noticing the changes.

**Watch out for:** Defers 4 minor issues including energy bar color-independence (finding #9). That's a real usability gap for color-blind users who use energy level for set-building. Also defers icon rail labels, which means screen reader users can't navigate using the rail.

---

## Option B: Enhanced Access

**Fixes 12 of 12 issues · WCAG AA+ · Keyboard QoL additions**

Everything from A plus: energy bar numeric tooltips, slightly larger source badges (9→10px), icon rail tooltips with aria-labels, ESC-to-close on scrim. Also adds keyboard shortcut hints in the toolbar and row count announcements.

**What works:** Complete coverage — no deferred issues. The QoL additions (keyboard shortcut bar, row count) make the keyboard experience genuinely good, not just compliant. The energy bar tooltips solve the color-independence problem elegantly without changing the visual design. Icon rail tooltips help ALL users, not just screen reader users.

**Watch out for:** Slightly more implementation work than A. The keyboard shortcut bar adds a visual element that wasn't in the locked design — but it's subtle and fits the toolbar area.

---

## Option C: Inclusive by Default

**Fixes 12 of 12 issues + 6 proactive features · Exceeds WCAG AA**

Everything from B plus: high contrast mode toggle, reduced motion mode, screen reader optimized track descriptions (with compatible key info), density toggle (40px→48px for tablets), command palette (Ctrl+K), and :focus-visible for mouse users.

**What works:** The density toggle is genuinely useful for future tablet support. High contrast mode could help DJs in bright environments (outdoor gigs, sun-lit prep areas). Command palette is a power-user feature that benefits everyone. The screen reader track descriptions with harmonic compatibility info is thoughtful.

**Watch out for:** This is a lot of implementation work for v1. The command palette alone is a significant feature. High contrast mode requires maintaining a second color variant. The density toggle partially re-opens a decision we just locked. Some of these features (command palette, density toggle) feel like product features masquerading as accessibility — they should be scoped and prioritized independently.

---

## My Take

**I recommend Option B (Enhanced Access).**

Option A is tempting for its minimalism, but deferring the energy bar color-independence issue (finding #9) is hard to justify when the fix is a simple tooltip. Energy level is a core data point — the "Flat Five" hierarchy says so. Making it color-only fails the product's own design principles.

Option C is impressive but over-scoped. A command palette, high contrast mode, and density toggle are product features, not accessibility fixes. They should be planned and prioritized like any other feature — not bundled into an accessibility audit. The user has consistently preferred v1 discipline.

Option B hits the sweet spot: complete issue coverage, keyboard QoL that makes the product genuinely better for power users (DJs ARE keyboard power users during set prep), and no scope creep. The visual impact is minimal — the user would recognize their locked design immediately. The keyboard shortcut bar is the only visible addition, and it fits naturally in the toolbar.

Implementation effort is moderate — mostly HTML attributes, ARIA roles, keyboard event handlers, and a few tooltips. All changes are additive and component-scoped. No global architecture changes.
