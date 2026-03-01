# Typography — Round 1 Critique

Building on the locked hierarchy (Flat Five, 3 emphasis levels), layout (full-width edge-to-edge table), component design (14 components with specific sizes: 13px artist bold, 12px title, 13px mono BPM, 11px chip, 10px genre, 9px source badge), and elevation (Wire Lift, sharp/flat aesthetic). Platform: Flutter Web, Material 3.

All three options use the same 7-level discrete type scale derived from the component design's exact specifications. The scale is not ratio-based — it's a tool scale optimized for the specific components. The real decision is font family and the trade-offs that come with it.

---

## Option A: System Native

**System font stack. SF Pro / Helvetica Neue / Segoe UI / Roboto. System monospace for BPM. Zero load.**

### Assessment:

1. **7 levels, matches the hierarchy.** Exactly the levels the component design requires. No excess.
2. **Every level distinguishable** — Artist bold 13px vs Title regular 12px is the tightest pair, but bold weight + 1px size creates clear separation. Micro (9px uppercase 600) is distinctly ambient.
3. **Body text readable at column widths.** 13px system sans on a full-width table is comfortable. Line length is controlled by column widths, not paragraph flow.
4. **Scale is discrete, not ratio-based.** Appropriate for a tool UI where sizes are dictated by component needs, not editorial rhythm. ~1.08-1.15 between adjacent levels.
5. **Font matches brand tone perfectly.** "Helvetica vibes" — this is literally Helvetica Neue on Mac. Neutral, professional, DJ-software appropriate. Matches the flat/sharp elevation language.
6. **3 weights, 0 bytes loaded.** Fastest possible option. No FOUT, no layout shift, instant render.
7. **Complements the Wire Lift shape language.** System sans + flat borders + no shadows = pure tool aesthetic.
8. **9px Micro is at the legibility floor.** System fonts generally handle 9px reasonably well, but it's tight. Source badges need good contrast.

### Watch out for:
- Cross-platform inconsistency: SF Pro on Mac, Segoe UI on Windows, Roboto on Android. The table will look subtly different on each OS. For a DJ tool where the DJ uses one machine, this matters less.
- System monospace varies widely: SF Mono is tight, Consolas is wider, Fira Code is wider still. BPM column width may need to accommodate the widest variant.
- Flutter Web doesn't always use SF Pro on Mac — it may fall through to Roboto depending on the Skia/CanvasKit renderer.

---

## Option B: Inter Tight

**Inter for UI. JetBrains Mono for BPM. Loaded via Google Fonts. ~175KB total.**

### Assessment:

1. **Same 7 levels.** Same scale structure, different rendering.
2. **Better small-size legibility than system fonts.** Inter was designed for 11-16px screen display. At 10px (Caption) and 9px (Micro), Inter's tall x-height and open apertures give it an edge over system fonts.
3. **Tabular numbers built in.** `font-feature-settings: 'tnum'` aligns numbers vertically even in non-mono contexts (import result panel, toast messages). System fonts have this too but Inter's implementation is more consistent.
4. **JetBrains Mono is excellent for BPM.** Designed for code readability, numbers are highly distinguishable. Dotted zero. Wider than SF Mono, which means more presence in the BPM column.
5. **Geometric but not cold.** Inter sits between the neutrality of Helvetica and the warmth of humanist sans. It's the most popular UI font on the web for a reason.
6. **~175KB load cost.** Not negligible but one-time, cached aggressively. Flutter's google_fonts package handles this cleanly.
7. **Cross-platform consistency.** Looks identical on Mac, Windows, Linux, Android. The catalog renders the same everywhere.
8. **Slight letter-spacing adjustment at Page Title** (-0.02em) — Inter's geometric forms at 15px benefit from tighter spacing.

### Watch out for:
- 175KB is real load time on first visit. On fast connections it's invisible. On slow connections it's a flash of fallback font.
- Inter is very common. If "looking different from other web apps" matters, Inter won't help. But for a DJ tool, blending in is fine — the content is the differentiator.
- JetBrains Mono is a separate font load. If you want to minimize requests, Roboto Mono (Option C) is a lighter alternative.

---

## Option C: Roboto Default

**Flutter's built-in font. Roboto + Roboto Mono. ~75KB additional load (just the mono).**

### Assessment:

1. **Same 7 levels.** Same scale, Roboto rendering.
2. **Zero load cost for the primary font.** Flutter Web bundles Roboto. No additional network request, no FOUT, no theme overrides. This is the path of least resistance.
3. **Material 3 native.** All Material components (buttons, chips, inputs, sliders) already use Roboto. No TextTheme overrides needed. Everything "just works."
4. **Roboto + Roboto Mono share metrics.** x-height, cap height, and baseline align. The BPM column (mono) next to the artist name (sans) feels seamless. No jarring baseline shifts.
5. **Missing weight 600.** Roboto jumps from 500 to 700. The Micro level (source badge) uses 500 instead of the ideal 600. Slight compromise.
6. **Slightly narrower than Inter at the same size.** This gives the table marginally more breathing room per column — or you could tighten column widths.
7. **"Material" feel.** Anyone who's used a Google product will recognize Roboto. This is either neutral (familiar = invisible) or a negative (looks like every other Material app).
8. **75KB total additional load** (just Roboto Mono + Noto Sans Arabic). Lightest custom option.

### Watch out for:
- Roboto at 9-10px is slightly less legible than Inter. The apertures are tighter, the x-height is lower. Genre pills and source badges will be marginally harder to read.
- The "Material default" feel may work against the professional DJ-tool aesthetic. Rekordbox, Traktor, and Serato all use custom or system fonts — none look like Material.
- Missing 600 weight means the Micro level's weight emphasis is weaker than in Options A and B.

---

## Comparative Take

**This comes down to two real choices: A or B.**

C (Roboto) is the pragmatic path — zero friction with Flutter — but it pushes the product toward a "Google Material app" aesthetic that doesn't match the DJ-software references the user has been pointing at. The missing 600 weight and lower small-size legibility are real if minor downsides. It's the safe choice, not the best choice.

A (System Native) is the purest Helvetica-lineage option and costs nothing to load. On a Mac — which is what most DJs use — it's literally SF Pro, which is gorgeous. The risk is cross-platform inconsistency and Flutter Web's unpredictable system font resolution.

**B (Inter Tight) is my recommendation.** It gives you the clean, tight, Helvetica-adjacent aesthetic the user described loving — but optimized for the exact sizes this product uses (10-15px). Cross-platform consistency means the catalog looks identical everywhere. JetBrains Mono is the best monospace for number-heavy data display. The 175KB load is a one-time cost that buys consistent, legible rendering across every row in a 500-track catalog.

The type scale itself is the same across all three options — it's dictated by the locked component design. The font family is the only real decision.

**Downstream:** This type system uses 7 levels with 3 weights. The color system needs text colors that maintain 4.5:1 contrast at 9px/600 (the smallest/lightest combination). Density should align with the 1.3-1.5 line-height range. Accessibility will audit the 9px Micro level closely.
