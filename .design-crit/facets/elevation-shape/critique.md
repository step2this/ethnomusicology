# Elevation & Shape — Round 1 Critique

Building on the locked component design (Crate + Board): 14 components, mostly flat, slide-over is the main layered surface, detail strip uses background tint not elevation. The component design already established 6px button radius, 3px chip radius, full-round pills. Platform: Flutter Web, Material 3. User preference: dense, professional, DJ-software aesthetic.

---

## Option A: Flat Crate

**Zero shadows. 3 elevation levels. 4-token radius scale.**

### Assessment:

1. **3 levels, all distinguishable** — base (#111), content (#181), overlay (#1E1). The tint steps are small but perceptible on dark backgrounds. No ambiguity about what's on top of what.
2. **Reinforces the locked component design perfectly** — the component design said "mostly flat" and this is maximally flat. Zero rendering overhead.
3. **Shadow values: N/A** — no shadows to tokenize. Simplest possible implementation.
4. **Light source: N/A** — no shadows means no light source inconsistency to worry about.
5. **Shape language matches tone** — 0/3/6/full is tight, sharp, professional. Matches Rekordbox/Serato aesthetic where everything is flat and functional.
6. **Overlay stacking relies entirely on scrim + tint** — the scrim (55% black) does the heavy lifting. This works but the panel itself is distinguishable from the scrim only by its background tint. Could feel like the panel doesn't "float" enough.
7. **Zero performance concern.** Fastest option to render.
8. **Downstream:** Color system needs 3 surface tints. Accessibility needs to verify contrast at all 3 levels (should be straightforward — tint differences are small). No shadow tokens to define.

### Watch out for:
- Sort dropdown appearing at Level 2 over the table may not feel elevated enough without any shadow. Borders alone may not create sufficient separation from the rows beneath.
- Toast at Level 1 may blend with the table if the tint step is too subtle for some monitors.

---

## Option B: Wire Lift

**Flat table + targeted shadows on overlays only. 4 levels. 4-token radius scale.**

### Assessment:

1. **4 levels, the right split** — levels 0-1 are flat (the daily-use table), levels 2-3 add shadows (things that float). The table stays clean; overlays feel physical. This distinction matches how the DJ actually uses the product.
2. **Reinforces component design** — flat where the component design said flat, shadow only on the slide-over and transient elements. The primary CTA button getting a subtle lift is a nice affordance cue.
3. **Shadow values are systematic** — 3 tokens (sm: 1px/3px blur, md: 2px/8px, lg: 4px/16px). Simple doubling progression. Easy to implement, easy to remember.
4. **Light source: consistent** — top-center, all shadows offset downward. The slide-over gets a directional left-cast shadow (-4px x-offset) which is physically correct for a right-edge panel.
5. **Shape language: identical to A** — same 4-token scale. Shape is already locked by component design; this just formalizes it.
6. **Overlay stacking is the strongest** — scrim + directional shadow on the slide-over creates a convincing physical separation. The panel genuinely feels like it slides over the table. Toast gets a small shadow to lift it above the scrim.
7. **Minimal performance impact** — shadows only on 3-4 elements at any given time (panel, toast, maybe a dropdown). No per-row shadows.
8. **Downstream:** Color system needs 4 surface tints. 3 shadow tokens to define. Accessibility audits the same contrast plus shadow-dependent perception (users who can't see shadows still get borders + tint as fallbacks).

### Watch out for:
- The line between "gets a shadow" and "doesn't" needs to be well-understood by developers. Rule: if it overlaps other content, it gets a shadow. If it's inline, it doesn't.

---

## Option C: Soft Stack

**Shadows on everything above L0. 5 levels. 5-token radius scale.**

### Assessment:

1. **5 levels, potentially too many** — L1 (1px/2px) and L2 (1px/4px) are very close on a dark background. The difference between a track row shadow and a header shadow may be imperceptible.
2. **Conflicts with component design intent** — the locked component design explicitly called for "mostly flat." Putting shadows on track rows, buttons, and source pills contradicts that decision.
3. **Shadow values are systematic** — 4 tokens with a doubling progression. Well-structured, but applied too broadly.
4. **Light source: consistent** — top-center throughout.
5. **Shape language adds an 8px token** — the "md" radius for toasts/tooltips is an interesting idea (rounder = floating), but adds complexity. The 4-token scale from component design is already complete.
6. **Overlay stacking is visually rich** — 5 visible layers in the demo look good in isolation, but in daily use, shadows on every row will create visual noise in a 100+ track catalog.
7. **Performance concern: moderate** — shadows on every track row means hundreds of shadow-casting elements during scroll. On a dark theme the impact is less visible but still computed.
8. **Downstream:** Color system needs 5 surface tints. 4 shadow tokens. Accessibility has more contrast surfaces to audit. More complexity for marginal benefit.

### Watch out for:
- Shadow-on-every-row contradicts the DJ software reference points (Rekordbox, Serato, Traktor are all flat). This option moves toward a consumer SaaS aesthetic.
- The 5-token radius scale adds implementation burden without clear visual payoff.

---

## Comparative Take

**I'd go with B (Wire Lift).** Here's why:

A (Flat Crate) is the purest expression of the "mostly flat" directive from component design, and it's a totally valid choice. But the slide-over panel — the most important overlay in the product — benefits from a real shadow. The scrim alone doesn't communicate "this panel slides over your table" as strongly as scrim + directional shadow. A small shadow on the primary CTA also helps affordance.

C (Soft Stack) is over-built for this product. Shadows on track rows contradicts the locked component design and the DJ-software aesthetic. It solves a problem the product doesn't have.

B threads the needle: **flat where you browse, shadowed where you act.** The daily table view is completely shadow-free (fast, dense, clean). Shadows only appear on transient elements that genuinely float above the table. This matches the user's established preference for dense, professional, DJ-software aesthetics while giving the import slide-over the physical presence it needs.

The 4-token radius scale (0/3/6/full) is already fully defined by the locked component design — all three options agree on it. The shape language is a non-decision here; component design already locked it.
