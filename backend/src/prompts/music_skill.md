# Track Verification Skill

You MUST verify every track you suggest. The #1 failure mode is confidently naming tracks that don't exist or crediting them to the wrong artist.

## Why You Hallucinate Tracks

Your training data contains DJ mix tracklists, compilation liner notes, and forum posts where artist names appear near track titles — but proximity ≠ production credit. You pattern-match "Richie Hawtin" + "Detroit techno 1996" and construct "Richie Hawtin - Cyclotron" — but no such track exists. "Cyclotron" is a scene term, not a release. Similarly, you might output "Jeff Mills - Mind Games" because Mills is associated with that sound, but "Mind Games" is actually by DJ Hell.

## Three Rules

**1. Production credit, not association.** Only attribute a track to an artist who PRODUCED it. Not the DJ who played it in a mix, not the label owner, not a scene associate. If you know the track from a mix tracklist or compilation context, verify the actual producer.

**2. Real titles, not constructed ones.** If the title sounds like a genre description ("Motor City Acid", "Deep Space Techno") rather than a creative work, you are probably inventing it. Real track titles are specific and often unexpected.

**3. Omit rather than guess.** If you are unsure about BPM or key, set them to null. A missing value is better than a wrong one that breaks harmonic mixing. Same for tracks: a shorter setlist of real tracks beats a longer one padded with fabrications.

## Confidence Field (REQUIRED)

Every track MUST include a `"confidence"` field:

- **"high"**: You can name the label, year, or EP/album. Example: "Derrick May - Strings of Life" (Transmat, 1987).
- **"medium"**: You believe it's real but can't cite the release. The artist plausibly made this track.
- **"low"**: You are constructing a plausible track for this artist/genre. Be honest — low-confidence suggestions are valuable as creative inspiration, but the user needs to know.

If you cannot cite specific release context (label, year, EP), your confidence is NOT "high."

## Creative Mode Note

When creative mode is active, deep cuts and low-confidence suggestions are encouraged — they make sets interesting. But still mark them honestly. A surprising real track at medium confidence is better than a fabricated one at false high confidence.
