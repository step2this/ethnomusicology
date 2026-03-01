# Density & Spacing — Round 1 Critique

Building on all 8 locked facets. This is the first facet rendered with the complete design system: Ember Crate colors, Inter-scale typography, Wire Lift elevation, Crate + Board components — everything together.

The density decision is straightforward for this product. DJs scan large catalogs. Density is a feature, not a tradeoff. The question is how tight.

---

## Option A: Pro Dense

**32px rows · 25 tracks visible · 2px base unit**

Maximum information density. 24x24 art, tighter chips, narrower columns. Fits ~25 tracks in a 900px viewport.

**What works:** More tracks on screen means less scrolling during set-building. Power users scanning 500+ track catalogs will appreciate seeing more at once. Every Rekordbox power user has wished for denser rows at some point.

**Watch out for:** 32px rows are below the 44px touch target minimum — desktop-only. The Camelot chips at 10px/18px height are harder to read. Energy bars at 10px tall lose visual weight. The warm Ember palette's nuances get compressed at this scale — the amber accents have less room to breathe. Album art at 24px is barely a thumbnail.

---

## Option B: Studio

**40px rows · 19 tracks visible · 4px base unit**

The balanced density — matches the component design wireframes the user already approved. 32x32 art, standard chips, comfortable scanning.

**What works:** This is the density the user has been reviewing and liking throughout the entire crit. 19 visible tracks is plenty for scanning. The 32x32 art is just large enough to recognize albums. Camelot chips at 11px/20px are easy to read. Energy bars at 14px have clear visual weight. The Ember Crate palette has room to express its warmth.

**Watch out for:** 40px is close to but below the 44px Material 3 minimum touch target. Fine for desktop (primary platform), may need a tablet density mode later. Not as dense as Rekordbox at its tightest.

---

## Option C: Session

**48px rows · 16 tracks visible · 4px base unit (generous)**

Most spacious option. 36x36 art, wider chips, more padding everywhere. Touch-friendly, approachable.

**What works:** Meets Material 3's 48dp touch target. Comfortable for extended sessions. Larger art thumbnails are more recognizable. All data viz elements (chips, bars) are easy to read. Would work on tablets without modification.

**Watch out for:** 16 tracks visible is noticeably less than 19 or 25. For a DJ scanning a 500-track catalog, that's more scrolling. Feels more like a music streaming app than pro DJ software. The extra whitespace dilutes the "dense professional tool" feel the user consistently values.

---

## My Take

**I recommend Option B (Studio).**

This is not a close call. The user has consistently valued density, called out Rekordbox/Traktor/Serato as reference points, and approved wireframes at exactly this density throughout the crit. Option B isn't a compromise — it's the right density for a desktop-first DJ tool where scanning speed matters.

Option A is too tight — it sacrifices readability of the very data viz elements (Camelot chips, energy bars) that make this product distinctive. Option C is too loose — it loses the professional tool density that distinguishes this from a consumer music app.

The 4px base unit with a 7-token scale (4/8/12/16/24/32/48) provides clean, predictable spacing that aligns with the Inter type scale and the component dimensions already locked.

**Spacing tokens for the locked system:**
- `space-xs`: 4px (inline gaps, chip padding)
- `space-sm`: 8px (row padding, element spacing)
- `space-md`: 12px (section padding, column headers)
- `space-lg`: 16px (panel padding, section gaps)
- `space-xl`: 24px (major section separation)
- `space-2xl`: 32px (page-level spacing)
- `space-3xl`: 48px (page sections, toolbar height)
