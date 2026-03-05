# Track Verification Skill

You MUST verify every track you suggest. The #1 failure mode is confidently naming tracks that don't exist or crediting them to the wrong artist.

## Why You Hallucinate Tracks

Your training data contains DJ mix tracklists, compilation liner notes, and forum posts where artist names appear near track titles — but proximity ≠ production credit. You pattern-match "Richie Hawtin" + "Detroit techno 1996" and construct "Richie Hawtin - Cyclotron" — but no such track exists. "Cyclotron" is a scene term, not a release. Similarly, you might output "Jeff Mills - Mind Games" because Mills is associated with that sound, but "Mind Games" is actually by DJ Hell.

## Three Rules

**1. Production credit, not association.** Only attribute a track to an artist who PRODUCED it. Not the DJ who played it in a mix, not the label owner, not a scene associate. If you know the track from a mix tracklist or compilation context, verify the actual producer.

**2. Real titles, not constructed ones.** If the title sounds like a genre description ("Motor City Acid", "Deep Space Techno") rather than a creative work, you are probably inventing it. Real track titles are specific and often unexpected.

**3. Null over wrong for metadata.** If unsure about BPM or key, set to null. But do NOT omit tracks you believe are real just because you can't cite the label. A deep cut at medium confidence is valuable — only omit tracks you actively suspect are fabricated.

## Confidence Field (REQUIRED)

Every track MUST include a `"confidence"` field:

- **"high"**: You can name the label/year/EP, OR you have strong recall of this track existing across multiple independent sources.
- **"medium"**: You believe it's real — the artist plausibly made this, the title feels like a real creative work, but you can't cite the release.
- **"low"**: You are constructing a plausible track. Be honest — low-confidence suggestions are creative inspiration.

Reserve "high" for genuine confidence. But don't default to "medium" just because you lack a citation — strong recall counts.

## Diversity Guidance
A great setlist showcases range: mix well-known tracks with deep cuts. Include 2-3 tracks that would surprise a knowledgeable listener. Do NOT play it safe with only obvious hits — medium-confidence deep cuts are valuable.

## Creative Mode Note

When creative mode is active, prioritize discovery. Dig into back catalogs, side projects, aliases. A setlist of obvious choices is a failure. Medium-confidence deep cuts are the soul of a great DJ set.
