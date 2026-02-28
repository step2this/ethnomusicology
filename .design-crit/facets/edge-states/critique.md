# Edge States — Round 1 Critique

## Option A: Lean & Direct

### WHY THIS OPTION
Treats the DJ as a power user. One-line messages, one action button, no explanations beyond what's necessary. The same pattern repeats across all edge states: heading + action. Skeleton loading with no text. Error messages with no preamble. This option bets that DJs don't need hand-holding — they've seen error states before and just want to get back to work.

### WORKS WELL FOR
- Experienced DJs who import regularly and know the tools. They don't need to be told what Beatport is.
- Fast recovery — no reading required, just click the action button.
- Consistent — same pattern everywhere means no surprises.
- Less design work and less copy to maintain.

### WATCH OUT FOR
- Empty catalog first-use ("No tracks yet") is underwhelming as a first impression. New user has no context about what the app does or what sources are available.
- "3 tracks failed" with no explanation leaves the DJ wondering why. They can't fix what they can't understand.
- URL validation error says what's wrong but doesn't show what right looks like. DJ has to guess the correct URL format.
- "Couldn't load your tracks" — is my data gone? The DJ doesn't know if this is transient or permanent.

---

## Option B: Context-Rich

### WHY THIS OPTION
Each edge state is tailored to the situation. The empty catalog shows available sources as cards. URL validation shows an example of a valid URL. Import errors explain what happened AND reassure that existing tracks are safe. Partial imports show per-track results so the DJ can work with what succeeded.

The bet: during set prep, a DJ who hits an error doesn't want to debug — they want to know their tracks are safe and what to do next. Context prevents the "wait, did I lose my data?" panic.

### WORKS WELL FOR
- First-time users who need to understand what sources are available and what to expect.
- Partial import recovery — per-track breakdown lets the DJ decide whether to retry or work with what they have.
- Reducing support questions — specific error messages ("this looks like an artist page, not a chart") prevent common mistakes.
- Trust-building — "your tracks are safe" reassurance prevents data-loss anxiety.

### WATCH OUT FOR
- More copy to write and maintain per edge state. Each state is custom, not templated.
- Source cards in the empty state could feel like onboarding that gets stale for returning users.
- Per-track import breakdown adds complexity to the import sheet — more scrolling, more visual noise.
- Risk of being patronizing to experienced DJs who know their URL formats.

---

## My Take

I'd go with **B (Context-Rich)**, but with one important adjustment: **tone it down for returning users.**

The empty catalog first-use state with source cards is excellent — it answers "what can I do here?" immediately. But if the DJ clears their catalog or comes back after importing, they should see the lean "No tracks" + "Import" pattern from A, not the onboarding again.

For everything else, B wins:
- **URL validation with examples** prevents the #1 support question ("what URL do I use?")
- **"Your tracks are safe"** on errors is a three-word trust builder that costs nothing
- **Per-track partial import** lets the DJ make informed decisions about what to do next
- **Shaped skeletons** matching the enriched tile layout feel polished and set expectations

The real principle: **edge states should match the stakes.** Empty catalog first-use is high stakes (first impression) — go rich. URL validation is medium stakes — be specific. Auth expired is low stakes — keep it simple. B gets this calibration right.
