# Signals the pipeline emits and nobody reads

Measured 2026-08-13 across all 92 entries at pin
 `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
both sides of each.

The section-alignment finding was made because the artifact had been recording
 `alignment structure-mismatch` for weeks and nothing read it.
That is a pattern rather than an incident,
so every signal the deterministic
 core emits was censused at once,
and the census found a second defect.

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Unread signals: census one of the deterministic core, footnotes and alignment, 2026-08-13](translation-repair-unread-signals-deterministic-core-census-footnotes-and-alignment-2026-08-13.md):
  What the deterministic core emitted over the 92-entry corpus,
  and the two defects the census found:
  a footnote graph wrong on 10 of 15 findings,
  and an alignment mismatch finding that was a false alarm 6 times in 7,
  plus the check that alignment never drops content and the four settled repairs that shipped broken footnotes.
  Sections:
  - [Census one: what the deterministic core emits over the 92-entry corpus](translation-repair-unread-signals-deterministic-core-census-footnotes-and-alignment-2026-08-13.md#census-one-what-the-deterministic-core-emits-over-the-92-entry-corpus)
  - [The footnote graph was wrong, and nothing was reading it to notice](translation-repair-unread-signals-deterministic-core-census-footnotes-and-alignment-2026-08-13.md#the-footnote-graph-was-wrong-and-nothing-was-reading-it-to-notice)
  - [The alignment mismatch finding is a false alarm 6 times out of 7](translation-repair-unread-signals-deterministic-core-census-footnotes-and-alignment-2026-08-13.md#the-alignment-mismatch-finding-is-a-false-alarm-6-times-out-of-7)
  - [Alignment never drops content](translation-repair-unread-signals-deterministic-core-census-footnotes-and-alignment-2026-08-13.md#alignment-never-drops-content)
  - [Four settled repairs shipped broken footnotes, and the detector was right there](translation-repair-unread-signals-deterministic-core-census-footnotes-and-alignment-2026-08-13.md#four-settled-repairs-shipped-broken-footnotes-and-the-detector-was-right-there)
- [Unread signals: census two of a run's record and the smaller signal families, 2026-08-13](translation-repair-unread-signals-census-two-run-record-and-smaller-signal-families-2026-08-13.md):
  What a run records across the 56 settled entries,
  and the smaller families checked against it:
  the rejected-value family,
  the refiner that goes silent on whole entries,
  and dropped merge opinions that do not explain the duplicates.
  Sections:
  - [Census two: what a run records across the 56 settled entries](translation-repair-unread-signals-census-two-run-record-and-smaller-signal-families-2026-08-13.md#census-two-what-a-run-records-across-the-56-settled-entries)
  - [The rejected-value family is small, and checked so nobody re-opens it](translation-repair-unread-signals-census-two-run-record-and-smaller-signal-families-2026-08-13.md#the-rejected-value-family-is-small-and-checked-so-nobody-re-opens-it)
  - [The refiner goes silent on whole entries](translation-repair-unread-signals-census-two-run-record-and-smaller-signal-families-2026-08-13.md#the-refiner-goes-silent-on-whole-entries)
  - [Dropped merge opinions do not explain the duplicate issues](translation-repair-unread-signals-census-two-run-record-and-smaller-signal-families-2026-08-13.md#dropped-merge-opinions-do-not-explain-the-duplicate-issues)
- [Unread signals: quote anchoring losses, the closed wrapping hypothesis and the false re-open alarms](translation-repair-unread-signals-quote-anchoring-losses-wrapping-closure-and-re-open-alarms.md):
  The investigation of the 398 critic claims discarded at quote anchoring:
  the soft-wrapping hypothesis,
  its bounds and its closure at 30 misses,
  what the misses are not,
  and the later alarms that tried to re-open it and turned out to be pooling artifacts and one outlier entry.
  Sections:
  - [Quote anchoring discards 398 critic claims, and the cause is still unknown](translation-repair-unread-signals-quote-anchoring-losses-wrapping-closure-and-re-open-alarms.md#quote-anchoring-discards-398-critic-claims-and-the-cause-is-still-unknown)
  - [The quote-anchoring re-open alarm was a pooling artifact, and the closure stands](translation-repair-unread-signals-quote-anchoring-losses-wrapping-closure-and-re-open-alarms.md#the-quote-anchoring-re-open-alarm-was-a-pooling-artifact-and-the-closure-stands)
- [Unread signals: outcomes recorded without causes, and the walls that came down](translation-repair-unread-signals-recorded-outcomes-without-causes-the-four-walls.md):
  The recurring wall,
  where the pipeline records what happened and drops why,
  with the cost of the remedy measured,
  and the three instances fixed so far:
  the quote needle,
  the voice-loss diagnosis,
  and the critic attribution that was discarded in the source after all.
  Sections:
  - [The recurring wall: outcomes are recorded, causes are not](translation-repair-unread-signals-recorded-outcomes-without-causes-the-four-walls.md#the-recurring-wall-outcomes-are-recorded-causes-are-not)
  - [The wall came down on one of the four](translation-repair-unread-signals-recorded-outcomes-without-causes-the-four-walls.md#the-wall-came-down-on-one-of-the-four)
  - [The second wall came down, and the client was never the problem](translation-repair-unread-signals-recorded-outcomes-without-causes-the-four-walls.md#the-second-wall-came-down-and-the-client-was-never-the-problem)
  - [The third wall is the same shape after all, and I said it was not](translation-repair-unread-signals-recorded-outcomes-without-causes-the-four-walls.md#the-third-wall-is-the-same-shape-after-all-and-i-said-it-was-not)
- [Unread signals: the pass13 findings census and two later signal gaps, to 2026-08-26](translation-repair-unread-signals-pass13-findings-census-and-later-signal-gaps-2026-08-26.md):
  A census of the findings recorded in the first settled entries of pass13,
  including the panel parser that degrades gracefully
  and the alignment finding that contradicts its own message,
  and the 2026-08-26 pair:
  a coverage sentence that misread its seat,
  and the link check that did not exist.
  Sections:
  - [Census of the findings nobody reads, pass13](translation-repair-unread-signals-pass13-findings-census-and-later-signal-gaps-2026-08-26.md#census-of-the-findings-nobody-reads-pass13)
  - [A signal that was read and said the wrong thing, and one that did not exist (2026-08-26)](translation-repair-unread-signals-pass13-findings-census-and-later-signal-gaps-2026-08-26.md#a-signal-that-was-read-and-said-the-wrong-thing-and-one-that-did-not-exist-2026-08-26)

## What still reads nothing

`alignment.findings` is turned into scorecard text and recorded,
and reaches no
 stage that could act on it.
`footnoteGraph` was in that position until 2026-08-13 and no longer is:
 `footnoteBreakCount` reads it,
and the chunk-integrity gate refuses a patch
 raising the break count.
The quote-anchoring findings are recorded,
and now carry a diagnosis,
but
 nothing acts on them either.
Whether any of these should is a design question this document does not settle.
