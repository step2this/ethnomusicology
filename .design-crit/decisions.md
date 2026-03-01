# Design Decisions — Ethnomusicology Import & Catalog

**Project:** Multi-source import screen and DJ metadata catalog for an LLM-powered DJ setlist platform
**Platform:** Flutter Web (desktop-first)
**Date:** 2026-03-01
**Areas resolved:** 10 of 10
**Rounds total:** 12 (across all areas)
**Design system:** Ember Crate

---

## Defining the Product (Structural)

### 1. Screen Inventory — Catalog-First

| Field | Value |
|-------|-------|
| Locked option | option-c |
| Decided by | user (1 round) |

1 primary screen (Track Catalog) + 1 secondary (Import Sheet overlay). Catalog is home base. Import is a drawer/sheet action — one of many future sources.

**User signal:** "DJs browse far more than they import. The catalog IS the product."

### 2. Edge States — Context-Rich

| Field | Value |
|-------|-------|
| Locked option | option-b |
| Decided by | user (1 round) |

Tailored edge states per situation. Empty catalog shows source cards. URL errors show valid examples. Errors reassure "your tracks are safe." Partial imports show per-track results. Shaped skeletons.

**User signal:** "Context-rich approach is the right fit."

---

## Arranging the Pieces (Compositional)

### 3. Layout — Full-Width Crate

| Field | Value |
|-------|-------|
| Locked option | option-b |
| Decided by | user (1 round) |

Edge-to-edge track table with sortable columns. 48px collapsed icon rail on left (reserves space for future browser pane). No max-width.

**User signal:** "V1 discipline — ship the dense track table now, expand later."

### 4. Content Hierarchy — Flat Five

| Field | Value |
|-------|-------|
| Locked option | option-c |
| Decided by | user (1 round) |

5 co-primary fields (Artist, Title, BPM, Key, Energy) differentiated by visual treatment not size. Artist (bold), Title (regular), BPM (monospace), Key (Camelot color chip), Energy (bar graph). 3 emphasis levels total.

**User signal:** "All five data points matter equally. Don't pick a winner."

### 5. Component Design — Crate + Board (A+C Merge)

| Field | Value |
|-------|-------|
| Locked option | option-ac |
| Decided by | user (2 rounds) |

14 components merging C's structure (slide-over with scrim, click-to-select accent border, expandable detail strip, multi-column sort with priority numbers, source pills) with A's design language (compact pill Camelot chips 20px, thin 5px vertical energy bars, flat borders, 6px radius, 34px buttons, 320px panel).

- **Round 1:** 3 options (Lean Crate, Studio Deck, Mixer Board). User cut A+B, kept C structure but wanted A's compact language. Requested merge.
- **Round 2:** A+C merge "Crate + Board" presented. Locked immediately.

**User signal:** "Merge, don't compromise. Sortable columns are non-negotiable."

### 6. Elevation & Shape — Wire Lift

| Field | Value |
|-------|-------|
| Locked option | option-b |
| Decided by | user (1 round) |

4-level elevation. Table is flat (levels 0-1, no shadows). Shadows only on floating elements: slide-over (-4px 0 16px), toasts (0 1px 3px), primary CTA (0 1px 3px). Scrim 55% black. 4-token radius scale (0/3/6/full).

**User signal:** "Trusts designer recommendation. Flat table + shadowed overlays makes sense."

---

## Making it Feel Right (Sensory)

### 7. Typography — Inter Tight

| Field | Value |
|-------|-------|
| Locked option | option-b |
| Decided by | user (1 round) |

Inter variable font for UI + JetBrains Mono for BPM/data. 7-level discrete scale (15/13/13/12/11/10/9px). 3 weights (400/500/700). Tabular numbers enabled. ~175KB one-time load.

**User signal:** "Clean sans-serif, Helvetica vibes. Cross-platform consistency over zero-load pragmatism."

### 8. Color System — Ember Crate

| Field | Value |
|-------|-------|
| Locked option | option-d |
| Decided by | user (1 round) |

Warm dark palette. Amber (#E8963A) primary, copper (#C06030) secondary on warm-tinted blacks (#0F0D0B). Warm white text (#F0ECE4). 24 warm-shifted Camelot key colors. Energy bars: teal to amber to copper-red. Roland TR-808 aesthetic.

4 options presented: Mondrian Board (light, primary colors), Indigo Crate (dark monochrome), Neon Deck (cyan/magenta neon), Ember Crate (warm amber/copper).

**User signal:** "It's like a Roland 808. Warm amber is the brand. Subtle hardware nods welcome."

### 9. Density & Spacing — Studio

| Field | Value |
|-------|-------|
| Locked option | option-b |
| Decided by | user (1 round) |

40px rows, ~19 tracks visible, 4px base unit. 32x32 art thumbnails. 7-token spacing scale (4/8/12/16/24/32/48px). Matches wireframe density from all prior areas.

**User signal:** "Not fussed — trusts designer recommendation. Defaults throughout the crit have been fine."

---

## Final Check (Cross-Cutting)

### 10. Accessibility — Enhanced Access

| Field | Value |
|-------|-------|
| Locked option | option-b |
| Decided by | user (1 round) |

WCAG AA, 12/12 issues fixed. Keyboard row navigation (up/down arrows + Enter/Space), focus trap on slide-over with ESC, skip link, 2px amber focus rings, semantic table with aria-sort, live regions for toasts/sort changes, energy bar numeric tooltips, icon rail tooltips, keyboard shortcut hints in toolbar, row count announcements.

**User signal:** "DJs actively seek hotkeys — keyboard shortcuts are a feature, not just compliance."

---

## Emerging User Preferences

Cross-cutting patterns observed across all 10 decisions:

- **Thinks like a working DJ**, not a product manager
- **V1 discipline** — ship what works, expand when content justifies it
- **Rejects false hierarchies** between DJ data points
- **Appreciates merges over compromises** — combine the best parts, don't water down
- **Trusts designer on technical/subtle facets** — density, elevation, spacing
- **Color is the differentiator** — warm amber/copper, TR-808 aesthetic
- **Keyboard access is a feature** for this audience, not just compliance
- **Responds to designs that feel "sick"** — visceral, professional, dense

---

## Deliverables

| File | Purpose |
|------|---------|
| `direction.html` | Visual design direction page |
| `design-tokens.json` | Extracted design tokens for implementation |
| `decisions.md` | This file |
| `overview.html` | Progress dashboard |
| `crit-session.md` | Compressed crit context |
| `facets/` | All option wireframes, compare views, feedback files |
