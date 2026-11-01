# Translation repair history 2026-08-21 to 2026-08-23: one duplication reported once

Why one duplicated span arrived as 866 findings,
the rule that merges windows by occurrence rather than adjacency,
two findings the tests turned up,
the re-measurement with a positive control and its own correction,
and the decision to export internals marked as internal.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-23: one duplication reported as 866 findings, now reported as one

`#183`,
closed.
Commit `b0295583c`.

`assembly-repetition.ts` grows repeats from fixed-length windows,
longest first,
and suppresses any phrase contained in one already reported.
That rule handles shorter-inside-longer and nothing else.
Growth stops at twelve words,
so a passage longer than that spans many windows of exactly twelve,
none of which contains another,
so none suppresses any other.

### What the reporting shape was costing

Measured on `Zha_Ke` in `vub171-20260822`:
one duplicated span of about 877 words arriving as 866 findings.
That was 866 of the corpus-wide 947,
so any rate,
trend or threshold read off `introduced-repetition`
described one slice of one entry rather than the corpus.

### The rule, and why it is about occurrences rather than adjacency

Two windows are one passage when the second occurs
in exactly the places the first does,
each advanced by one word.
Adjacency alone is not evidence:
two unrelated repeated passages can abut,
and merging them would report a span the document never said.
Two unrelated repeats do not repeat in the same places,
so the occurrence test cannot join them.

### Two findings the tests turned up that the plan had not

Both were found by a test failing,
not by reading the code.

A PASSAGE SAID THREE OR MORE TIMES MAKES A SECOND ARTIFACT.
In `P P P` the join between copies occurs twice,
so the tail of one copy followed by the head of the next is itself a repeat.
Suppressing it needs the covered ranges merged into a union:
the join straddles the seam between two adjacent ranges
and is contained in neither alone.
The first attempt tested containment against single ranges and did nothing.

SUPPRESSING SUCH A SPAN OUTRIGHT TURNS ONE FINDING INTO ELEVEN.
Its pieces reappear at every shorter length,
because the containment rule can only suppress against a span it was given.
So what SUPPRESSES and what is REPORTED are two different lists.
Every span suppresses;
only spans no earlier span accounts for become findings.
That is now explicit in the code as `covering` beside `found`,
and in the returned `GrownSpan.accountedFor`.

### The re-measurement, with a positive control

Run over the same stored artifacts with the pre-fix code and then the fixed code.

The pre-fix code reproduced the recorded counts exactly,
866,
73 and 6,
which is the control:
without it a lower number afterwards would not have been evidence,
since the harness might simply have been measuring something else.

    vub171-20260822       Zha_Ke  866 -> 2
    vub-run1-20260821     Zha_Ke   73 -> 53
    readable-20260820     Zha_Ke    6 -> 6
    win107-adj-20260820   lintong   1 -> 1
    win107-panel-20260820 lintong   1 -> 1
    corpus-wide total             947 -> 63

CORRECTING THE FIRST CORRECTION MADE IN THIS SECTION,
which was wrong the other way.
That pass scanned three hardcoded run directories,
reported 945 -> 61,
and called the recorded 947 an overcount.
947 was right for the corpus;
only its per-directory split was not.
`readable-20260820` holds 6 rather than 8,
and the two unaccounted findings are one apiece
in `win107-adj-20260820` and `win107-panel-20260820`,
both on `lintong`.
Rescanned over every directory under the scratch root,
22 of which hold artifacts,
the corpus-wide figure is 947 -> 63.

Neither `lintong` finding moves,
for the same reason `readable-20260820` does not:
a lone repeat has nothing to merge with.
The fix only ever collapses a run of windows over one passage,
so a corpus with no runaway in it reads the same before and after,
which is itself the check that the change is narrow.

63 is the number to carry forward.
It describes the corpus rather than the reporting shape.
`readable-20260820` is unchanged because all six of its repeats
are shorter than the window,
so nothing merges;
`vub-run1-20260821` falls by a fifth,
which is real merging
on top of genuinely distinct short repeats.

### On exporting internals

The barrel had been withholding `wordsOf` and `countPhrases`
with a comment arguing that a barrel decides what is public.
The owner overruled that during this work:
export them and mark `@internal`.
Done,
and the new span helpers follow the same rule,
which is what let the span module get direct tests
against the built bundle the way everything else here is tested.
`@internal` is a MODIFIER tag and takes no content;
the explanation goes in prose above it.
