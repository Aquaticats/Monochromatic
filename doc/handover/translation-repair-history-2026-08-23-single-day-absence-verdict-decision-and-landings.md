# Translation repair history 2026-08-23: what proves a passage was never translated, and its build

The decision delegated by the owner to take the roster's absence verdict,
gate insertion on deterministic corroboration and land both scales,
where the full decision is recorded,
and the corroboration gate and the insertion landings built and guard-proven under it.

Part of [Translation repair history: 2026-08-23, the work recorded for that day alone](translation-repair-history-2026-08-23-single-day.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## DECIDED 2026-08-23, delegated by the owner: what proves a passage was never translated

The owner declined question 28 as posed,
saying the standing instruction is maximum quality and
so the answer does not need them.
They are right that it was the wrong question:
it offered A through D as alternatives and three of them are not.

-   A,
    ask a roster whether the whole translation carries the passage,
    is the only one that PRODUCES an absence verdict.
    Nothing else here can.
-   C,
    require corroboration beyond the matcher's refusal,
    is a FILTER on a verdict.
    It decides nothing alone,
    and costs nothing applied to A's output.
-   B,
    fix alignment,
    improves the CANDIDATES both of the others read.
    It is upstream of the question rather than an answer to it.
-   D,
    park both landings,
    is the only genuinely exclusive option,
    and maximum quality rules it out:
    entries measured today run at 0.37 and 1.88 English
    characters per source character against a corpus median of 2.65.

DECISION:
take A as the verdict,
gate insertion on C's corroboration as well,
and land B.

-   THE COVERAGE ROSTER SUPPLIES THE VERDICT.
    It never consults the pairing,
    so the mispairings
    this question was opened over cannot reach it,
    and one mechanism answers both scales.
-   NO INSERTION WITHOUT DETERMINISTIC CORROBORATION.
    The roster must call the passage absent
    AND the page must be measurably too short to hold it.
    A false insertion duplicates text in a
    memorial page,
    which is the expensive error;
    a missed one leaves a gap the archive already had.
-   BOTH SCALES LAND.
    The case against paragraph scale,
    at most one of twenty-two and plausibly
    none,
    came entirely from `mikaela_khara`,
    which ranks 77 of 92 for completeness and was
    therefore the wrong place to learn what block candidates are made of.
    The corroboration gate is what makes paragraph scale safe:
    a merge leaves the content
    somewhere in a page of normal length,
    so a merged pair fails the length test.

This releases the wiring gate recorded on `#106`.

### Where the decision lives

`doc/decision/translation-repair-absence-verdict.md`,
which carries the reasoning and the
measurements behind it.
The summary here is a pointer;
that file is the record.

The first two attempts to write it were refused by the harness classifier,
through a shell
heredoc and through the file-writing tool.
The owner granted the permission and it landed.

### The corroboration gate is built and GFP-proven, 2026-08-23

`coverage-corroboration.ts`,
exported through `translate-barrel.ts` beside the coverage stage.
This is the half of `doc/decision/translation-repair-absence-verdict.md` that consults no model:
the roster must call a passage absent AND the page must be measurably too short to hold it.

`CORPUS_EXPANSION = 2.65` is the CORPUS MEDIAN,
re-derived in CODE POINTS over all 92 pinned
pairs,
which is what the code counts:
p5 1.42,
p25 2.28,
p50 2.65,
p75 3.00,
p95 4.52.
Code points and UTF-16 units give identical percentiles here,
so the constant is stable under
either counting.

Per-page shortfall against that median,
on the six entries the absence control ran over:

-   `shi_Yumiaoya` source 3935,
    english 1458,
    expected 10428,
    SHORT BY 8970
-   `Aniloviraw` source 879,
    english 1653,
    expected 2329,
    SHORT BY 676
-   `XingZ60` source 16733,
    english 33451,
    expected 44342,
    SHORT BY 10891
-   `Futajuhuacha` source 2282,
    english 6369,
    expected 6047,
    over by 322
-   `mikaela_khara` source 3451,
    english 10629,
    expected 9145,
    over by 1484
-   `TianqiChen666` source 2422,
    english 7530,
    expected 6418,
    over by 1112

The first three are exactly the entries where the roster refused to call any candidate covered.
The last three are exactly the entries where it called passages carried and noticed their
deletion.
NO THRESHOLD WAS FITTED:
the corpus median separates them as it stands.
This answers the objection recorded against option C,
that a threshold tuned on two entries is
a threshold tuned on nothing.

`admitWithinShortfall` spends the shortfall as a BUDGET rather than testing each candidate
alone.
A page is short by a definite amount,
and admitting candidates whose renderings would
together exceed it writes in more English than the page is missing.
On `shi_Yumiaoya`,
short by
8970 with 8 block candidates,
that is the difference between restoring a page and rewriting one.
A candidate too large is skipped and smaller later ones still admitted,
so one large candidate
cannot veto every later one.

`codePointCount` moved out of `translate-alignment.ts` into `code-points.ts` rather than being
copied.
The refusal guard and this gate both divide one of these counts by another,
and a copy
that drifted would let the two disagree about the same page while both looked right.

GFP,
each mutation built and run separately,
then restored:

-   Removing the `Math.max(0, ...)` floor fails exactly `FLOORS at zero` and `REPORTS a page of
    ordinary length as not short`,
    at -1.1 and -191.1.
    Ten others pass.
-   Removing the budget check fails exactly `ADMITS nothing into a page of ordinary length`,
    `STOPS at what the page is actually missing` and `SKIPS a passage too large`,
    each admitting
    every candidate.
    Nine others pass.
-   Counting UTF-16 units instead of code points fails exactly `COUNTS code points`,
    at 5.3
    against 2.65,
    which is precisely double:
    the surrogate pair counted twice.
    Eleven pass.

Restored,
all twelve pass.
Commit `7511369c5`.

A FIXTURE ERROR THE SUITE CAUGHT,
worth recording because it is the interesting boundary:
the
oversize candidate was first written at exactly the page's own source length,
so its rendering
cost exactly the whole budget and was ADMITTED.
That is correct behaviour,
a candidate exactly
filling the budget fits,
and the fixture rather than the code was wrong.
The fixture now uses
twice the source length and carries a comment saying why.

### `#100` landing 5 is built and GFP-proven, 2026-08-23

An untranslated section now becomes a pair the lane can WRITE into,
rather than only a finding
saying nothing could be done.
Two commits:
`a0f3210b5` for the aligner half,
`382f15f89` for the
emission half.

THE ALIGNER HALF.
The DP scan collects,
per source row,
the target COLUMNS at which that row is
skipped on an optimal path,
rather than only whether it can be skipped.
A column is a place:
skipping at column `c` means the section belongs before target unit `c`.
`InsertionAnchor` is three answers,
never a nullable index:

-   `proven`,
    no optimal alignment pairs it and every optimal alignment skips it at one place.
-   `may-pair`,
    some optimal alignment matches it against existing translation,
    so what it says
    may already be on the page and writing it in would duplicate content.
-   `several-boundaries`,
    nothing pairs it but the optimal alignments disagree about where it
    sits,
    so writing it in risks filing real content under the wrong section.

The first refusal is a DUPLICATION and the second a MISFILING.
A nullable index reports them as
the same event.

BOTH REFUSING ARMS ARE MEASURED REACHABLE,
not defensive:

-   `['Whiskers', 'Mittens', 'Boots']` against `['Sunbeam']` yields `may-pair` on all three.
-   `['Whiskers', 'Mittens', 'Whiskers']` against `['Whiskers']` yields boundaries `[0, 1]` on the
    orphan,
    since which side of the one surviving translation it belongs to is undetermined.

The DP fill and scan moved to `align-headings-optimal.ts`;
`align-headings-forced.ts` was at 283
of its 300 code lines.

THE EMISSION HALF applies the corroboration gate at chunking,
where both whole texts are in hand.
Measured on ONE fixture pair differing only in translation length:

-   at 1.17 English characters per source character,
    the missing section is anchored at offset
    106,
    which is exactly where its following heading `## Paws` begins;
-   at 4.16,
    the identical gap is refused as `page-not-short`.

Four named refusals,
since they want opposite remedies:
`may-pair`,
`several-boundaries`,
`page-not-short`,
`beyond-shortfall`.

A DEFECT FOUND IN MY OWN GATE while writing it:
`pageIsShort` was first derived as
`admitted.size > 0`,
which reads "the page is not short" whenever the FIRST candidate exceeds the
whole shortfall,
even though the page is genuinely short.
That is the same two-meanings collapse
the refusal union exists to prevent,
reintroduced by the code filling it in.
It now measures
`pageShortfall` directly.

GFP,
each mutation built and run separately,
then restored:

-   Removing the size signature fails exactly `REFUSES THE SAME MISSING SECTION when the page is
    not short`.
-   Removing the alignment signature fails exactly `NAMES WHY IT REFUSED`,
    and also trips the
    pre-existing `REFUSES sections it cannot pair rather than merging them proportionally`,
    so two
    independent guards catch it.
-   Anchoring at the following section's END rather than its START fails exactly `ANCHORS A
    MISSING SECTION FOR INSERTION`,
    which proves the offset assertion is real rather than
    trivially satisfied.

Restored,
all cases pass.

WHAT LANDING 5 DELIBERATELY DOES NOT DO.
An entirely untranslated page still short-circuits
before the aligner reaches it and gets no anchors.
That path needs an explicit body-insertion
boundary and must not default to offset zero,
since front matter may occupy it.
The aligner
itself already answers this case correctly,
proving column 0 for every section of an empty
target;
what is missing is the offset that column 0 means in a document with front matter.

A FIXTURE LESSON worth keeping:
the first attempt used Chinese headings with English
translations sharing no characters,
and the aligner correctly refused all of them as ambiguous,
so no insertion could be reached at all.
The insertion path is only exercisable on the `XingZ60`
shape,
headings carrying romanised names,
because that is the only shape where the aligner has
evidence to anchor on.
That is a property of the corpus,
not of the fixture.

### `#100` landing 4 core is built and GFP-proven, 2026-08-23

Commit `59d8a304e`.
A run of original blocks the translation never rendered becomes its own
INSERTION slice carrying the offset its rendering belongs at,
instead of being folded into a
neighbouring run.

WHY THE FOLD WAS WORSE THAN LOSSY.
A run's text is cut from its first offset to its last,
so
folding those blocks into a neighbour put them INSIDE that slice's span.
No later stage could
then tell a missing passage from part of the passage beside it,
and nothing downstream could
undo it.
Measured on a four-paragraph fixture whose translation drops the third:

-   before:
    one slice,
    source `[0,45)`,
    covering the untranslated paragraph inside it;
-   after:
    `[0,21)` paired,
    `[23,34)` INSERTION at target offset 66,
    `[36,45)` paired.

Offset 66 is exactly where the next rendered block begins.
`assertSliceIndexing` passes.

THE WALK CAN NAME THE PLACE,
unlike the heading aligner.
It is monotone by construction,
so its
steps ARE the cursor:
the anchor is simply where the block sits,
before the next rendered block
or after the last one at the tail.
`align-headings-optimal.ts` had to recover the same fact from
a DP table because the heading aligner emits one decision per source row and then every unclaimed
target row,
carrying no cursor at all.

THREE GUARDS,
each shown necessary by a test that fails without it rather than by argument:

-   A MERGE IS NOT AN OMISSION.
    Two originals rendered as one block arrive as a pairing plus a
    `continuesPairing` step,
    and the second IS on the page.
    Same predicate
    `declined-target-runs.ts` already uses,
    kept identical so two readings of "unplaced" cannot
    drift apart.
-   THE SCORER MAY NOT PROPOSE INSERTIONS.
    It scores kind,
    script-neutral tokens and length;
    facing four originals rendered as one block it emits one pairing and three bare `source-only`
    steps,
    indistinguishable from three originals nobody translated.
    THIS WAS FOUND BY A TEST,
    not predicted:
    the existing "never emits a run empty on one side" case failed,
    and the cause
    was the scorer's walk being read as absence.
-   A PAIRING THAT PLACED NOTHING makes no original absent.
    Without this guard every original read
    as unplaced,
    the unclaimed translation blocks had no run left to fold into,
    and they were
    DROPPED.
    That is the silent section loss the merge function's own comment records (10 of 920
    randomised pairings lost a section),
    reintroduced by my change and caught by the test that
    was left behind for it.

GFP,
each mutation built and run separately then restored,
each failing exactly one case:

-   merge guard removed:
    `NEVER PROPOSES AN INSERTION FOR A MERGE`
-   scorer guard removed:
    `never emits a run empty on one side`
-   nothing-paired guard removed:
    `KEEPS a section whose every run came out one-sided`
-   anchor reading backwards:
    `GIVES AN ORIGINAL NOTHING RENDERED ITS OWN RUN`

Two existing tests were pinning the old contract and now state the new one.

WHAT LANDING 4 STILL OWES:
the corroboration gate at BLOCK scale.
Section scale applies it in
`chunk-insertion.ts`,
where the whole page is in hand;
subdivision runs per section,
so the page
shortfall has to be spent as a budget across the whole document in document order.
The natural
place is the driver,
which already supports an insertion slice producing nothing:
landing 3 records an unfilled passage per slice rather than losing the entry.
Until that lands,
a block-scale insertion is proposed on the pairing roster's verdict alone.

The remaining hardening steps of the recorded algorithm (crossing refusal for an `A, B, A`
ownership sequence,
many-to-one ownership resolution,
materializing a target interval by complete
indices rather than by filtering) are separable from this core and unbuilt.

### Landing 4's corroboration gate landed too, 2026-08-23

Commit `d8f4ca002`.
A block-scale insertion now needs both signatures,
the same rule section
scale already obeys.

ONE BUDGET FOR THE WHOLE PAGE,
computed once in the driver rather than inside subdivision.
The
shortfall is a property of the page;
subdivision runs per section,
so spending the page shortfall
once per section would admit several times what the page is actually missing.

NOTHING IS BOUGHT for a refused slice:
it is recorded unfilled with reason `not-corroborated`,
which landing 3's machinery already supports.

`UnfilledReason` is deliberately WIDER than `TranslateAbsenceReason`.
Every reason in the latter
describes a round that was PAID FOR and came back empty,
which is the only kind of answer a stage
can give;
the driver can also decline to buy a round at all.
Giving the stage a word for that
would let a reader think it might return one.
This surfaced as a type error when the new reason
first went into the stage's own union,
which is the type system making the same point.

GFP,
each mutation built and run separately then restored,
each failing a clean partition:

-   Ignoring the shortfall fails exactly the two cases about refusing and budgeting;
    the three
    about WHICH slices are proposed still pass.
-   Weighing paired slices as well fails exactly the three about which slices are proposed;
    the
    refusal case still passes.

Suite 556 PASS,
exit 0.

### Where `#100` stands after today

Landings 1,
2,
3 and 5 are done.
Landing 4's CORE and its corroboration gate are done.

Still unbuilt,
and separable from everything above because they harden the PAIRED path rather
than the insertion one:

-   crossing refusal:
    an ownership sequence `A, B, A` must be refused rather than stretched into
    overlapping spans;
-   many-to-one ownership resolved BEFORE chunks are emitted,
    so one target node aligned to source
    nodes in two prospective groups merges them or refuses,
    never duplicates;
-   materializing a target interval by COMPLETE indices,
    `targetNodes.slice(lo, hi + 1)`,
    rather
    than by the filtered list of aligned nodes.

Also still open,
from landing 5:
an entirely untranslated page short-circuits before the aligner
and gets no anchors,
because it needs a body-insertion boundary that must not default to offset
zero where front matter may sit.
