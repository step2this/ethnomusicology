# Crit Session Context

## Emerging Preferences
User thinks like a working DJ, not a product manager. Values the browsing/set-building experience over import mechanics. Sees import as tactical and infrequent — the catalog is the daily workspace. Thinks about future extensibility (multiple sources, AI generation, local files) and wants architecture that accommodates it. Prefers designs where the primary activity gets the primary surface. Quick to decide when options align with DJ workflow intuition — doesn't need multiple rounds when the answer is clear.

DJ software reference points: Rekordbox, Traktor, Serato. Catalog/crate takes most of the screen. Left column browser for genres/artists/labels. Top area for waveforms/decks. The catalog and ways to slice/dice the collection are the core experience.

Favors v1 discipline — ship what works now, expand when content justifies it. Saves ambitious layouts (Browser + Crate) for v2 rather than shipping half-baked.

Rejects false hierarchies between DJ data points. Artist, title, BPM, key, and energy are ALL important — doesn't want the UI to pick a winner. Prefers flat, task-flexible designs over opinionated scan orders. Values rich visual coding (each data type gets its own visual voice) over size-based hierarchy.

Appreciates merges over compromises — when two options have distinct strengths (C's structure + A's design language), the user wants both fully intact, not watered down. Responds well to designs that feel "sick" — visceral, professional, dense. Not interested in pretty-but-empty.

Trusts designer recommendations on technical/subtle facets. Doesn't need to agonize over small differences — makes fast decisions when the distinctions are minor. Values speed through areas that don't differentiate the product.

Typography matters — it's "hard to do anything crazy and not have it look silly." Prefers clean sans-serif (Helvetica vibes), tight and dense. Chose quality + cross-platform consistency over zero-load-cost pragmatism. Values the craft of type even when the differences are subtle.

Color is where the product should "make an impact." Open to experimentation but wants it grounded — not distracting. Immediately connected warm amber to the Roland TR-808 aesthetic. Thinks in terms of physical hardware references (808, vinyl, analog), not abstract color theory. Wants subtle hardware nods woven in, not heavy-handed. The warmth IS the brand.

Keyboard access is a feature, not just compliance. DJs are trained on hotkeys from DJ software — they actively seek key commands and expect them. Accessibility and power-user UX are the same thing for this audience.

## What Exists
- **Screen Inventory** — Catalog-First: 1 primary screen (Track Catalog as home base) + 1 secondary (Import Sheet as overlay/drawer). Import is one of many future actions that feed into the catalog. Locked option: option-c. User signal: "DJs browse far more than they import. The catalog IS the product. Import, AI generation, and future sources are all secondary actions that feed into it."
- **Edge States** — Context-Rich: tailored edge states per situation. Empty catalog shows source cards. URL errors show valid examples. Errors reassure "your tracks are safe." Partial imports show per-track results. Shaped skeletons for loading. Locked option: option-b. User signal: "Context-rich approach is the right fit for this product."

## How It's Arranged
- **Layout** — Full-Width Crate: edge-to-edge track table with sortable columns (Title, Artist, BPM, Key, Genre, Source). 48px collapsed icon rail reserves space for future browser pane. No max-width — DJ software density. Locked option: option-b. User signal: "B to get started, save C (Browser + Crate) for the next round. V1 discipline."
- **Content Hierarchy** — Flat Five: 5 co-primary fields (Artist, Title, BPM, Key, Energy) differentiated by visual treatment not size. Artist bold, Title regular, BPM monospace, Key Camelot color chip, Energy bar graph. 3 emphasis levels total (co-primary, secondary, ambient). Locked option: option-c. User signal: "All five data points matter — artist, title, BPM, key, and energy. Don't pick a winner between them."

## How It's Built
- **Component Design** — Crate + Board (A+C merge): 14 components. C's structure (slide-over with scrim, click-to-select accent border, expandable detail strip, multi-column sort with priority numbers, source pills) dressed in A's design language (compact pill Camelot chips 20px, thin 5px vertical energy bars, flat borders, 6px radius, 34px buttons, 320px slide-over). Full table context view is the primary artifact. Locked option: option-ac. User signal: "Merge, don't compromise. C's structural patterns are critical (slide-over, detail strip, multi-sort). A's compact language keeps it dense. Sortable columns are non-negotiable."
- **Elevation & Shape** — Wire Lift: 4-level elevation (base, content, raised, overlay). Table is completely flat (levels 0-1, no shadows). Shadows only on floating elements: slide-over (directional -4px 0 16px), toasts (0 1px 3px), primary CTA (0 1px 3px). Scrim (55% black) + shadow for overlay depth. 4-token radius scale (0/3/6/full). Locked option: option-b. User signal: "Differences are subtle — trusts designer recommendation. Flat table + shadowed overlays makes sense."

## How It Feels
- **Typography** — Inter Tight: Inter variable font for UI + JetBrains Mono for BPM/data. 7-level discrete scale (15/13/13/12/11/10/9px). 3 weights (400/500/700). Tabular numbers enabled ('tnum'). ~175KB one-time load. Locked option: option-b. User signal: "Likes B best. Values clean sans-serif with Helvetica vibes. Chose cross-platform consistency and small-size optimization over zero-load pragmatism."
- **Color System** — Ember Crate: warm dark palette. Amber (#E8963A) primary, copper (#C06030) secondary on warm-tinted blacks (#0F0D0B base). Warm white text (#F0ECE4). 24 warm-shifted Camelot key colors on tinted dark backgrounds. Energy bars: teal (#5A9EC0) → amber (#E8963A) → copper-red (#D05040). Status: warm green success, yellow warning, copper-red error, teal info. Roland TR-808 design reference — subtle analog hardware nods. Locked option: option-d. User signal: "It's like a Roland 808. Warm amber is the brand identity. Subtle hardware references are welcome — not too much since it's played out, but fun here and there."
- **Density & Spacing** — Studio: 40px rows, ~19 tracks visible in 900px viewport, 4px base unit. 32x32 art thumbnails. 7-token spacing scale: space-xs(4) / space-sm(8) / space-md(12) / space-lg(16) / space-xl(24) / space-2xl(32) / space-3xl(48). Matches wireframe density from all prior facets. Locked option: option-b. User signal: "Not fussed about density — trusts designer recommendation. The defaults throughout the crit have been fine. Go with the balanced option."

## Cross-Cutting
- **Accessibility** — Enhanced Access: WCAG AA, 12/12 issues fixed. Keyboard row navigation (↑↓ + Enter/Space), focus trap on slide-over with ESC, skip link, 2px amber focus rings, semantic `<table>` with `aria-sort`, live regions for toasts/sort changes, energy bar numeric tooltips for color-independence, icon rail tooltips, keyboard shortcut hints in toolbar, row count announcements. Locked option: option-b. User signal: "DJs are used to key commands — they actively seek hotkeys. Keyboard shortcuts are a feature, not just accessibility."
