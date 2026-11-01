# Translation repair history 2026-08-07 to 2026-08-12: the introduced-defect probe and the naturalness audit

The probe's trigger rate and where its canonical figures live,
the probe judging wording the naturalness lane can replace,
the audit of the naturalness lane validated live against injected damage,
and the first live numbers from run 012.

Part of [Translation repair history: 2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Probe trigger rate: the decision doc is canonical now

Issue #53 is decided:
the probe stays in shadow mode,
recorded in
`doc/decision/introduced-defect-probe-gating.md` with the rejected gating
designs and the condition that reopens it.
The follow-up measurement,
comparing
corroborated regions against the round-three human repair grades,
is tracked
separately and is blocked on those grades existing.

STOP ADDING A SECTION PER SETTLED ENTRY.
This file accumulated one at five
entries and another at seven,
which makes the series hard to read and easy to
quote stale.
The current figures live in the decision doc;
refresh them there
with `mise run //package/module/translation-repair:score-probe` (moved on 2026-10-18 to
`//package/translation-repair/probe:score-probe`),
which costs no
quota and reads only local artifacts.

THE SERIES SO FAR,
for whoever wants the shape without rerunning anything:
`majorityIntroduced` has been 1 at every checkpoint (1,
5,
6,
7,
8,
9 entries)
while regions went 13,
67,
68,
72,
79, 83.
The numerator has never moved.
`minorityIntroduced` was 1 at one entry,
5 at five,
and 6 at both eight and
nine;
the six- and seven-entry checkpoints recorded only the majority column,
so
treat the minority series as four measured points,
not six.
`contradicted` and
`unanchored` are still zero,
and at nine entries `unprobedRecords` is zero
across 202 shipped records,
so the probe is reaching everything rather than
skipping quietly.

## The probe judges wording the naturalness lane can replace

Found while sizing #58,
and it changes what #60 can conclude.

Ordering,
from the source rather than from memory:
 `repair-chunk.ts:299` runs `runIntroducedDefectProbe` inside the accuracy
 stage,
and `repair-translation.ts:429` runs `runRefinePhase` afterwards over those
 outcomes.
So on a slice the lane rewrote,
the probe's before/after pair is the accuracy
 stage's,
and the text that reached the reader is `finalSliceText`.

`repair-sheet.ts:175` handles its side of this correctly and always did.
For a refined slice it prints
 "a later naturalness pass rewrote this slice,
so the wording above is not
 final",
 shows "the slice as actually returned",
and instructs "grade the RETURNED wording".
So the human grades post-refinement text while the probe judged
 pre-refinement text,
and joining them treats the two as one.

Measured across 28 entries:
 151 refined records,
 90 of them shipped,
 out of 1214 shipped records,
and 10 of the 50 positions in the drawn round-three gate sample.

Two consequences that must not be conflated:

-   GATING is unaffected.
    A gate would act during candidate selection,
    which is also before the lane
     runs,
    so the probe judges exactly the text such a gate would judge.
-   VALIDATION against human repair grades is affected,
    on those positions only.
    `score-probe` now reports `refinedJoined` so they can be excluded and the
     exclusion reported.

DETECTION PRECISION IS NOT AFFECTED AT ALL,
and this is worth stating plainly
 because it is the number currently out for grading.
The detection sheet asks whether an accepted issue is a real defect in the
 ORIGINAL translation.
It shows the original's wording and no correction,
so nothing the repair or the
 lane did afterwards can reach that question.

The remaining work is #58 proper:
 nothing probes whether the naturalness lane itself introduces defects,
 and it rewrote 90 shipped records here.
That is the same blind spot the introduced-defect probe was built to close,
 one stage later.

## The naturalness lane is audited now, and the audit was checked before trusting it

#58 is built,
wired,
read,
reported,
and validated live.

An accepted refinement runs the same introduced-defect probe against the pair
 that actually matters for it:
 `baselineText` is the repaired text and the region is the rewritten slice.
One region per slice,
because `RefineStageResult` exposes only whole-slice text
 and because `retainsResolvedIssues` already rolls back per slice,
so the audit's unit matches the lane's own unit of decision.
The roster is the checkers,
whom `assertCheckerIndependence` has already proved
 disjoint from the refiners.

THE PROMPT NEEDED A SECOND FRAMING,
and this was the part worth being careful
 about.
The probe tells reviewers the editor was "trying to fix defects that were
 ALREADY THERE",
 which is false of a lane that rewrites already-correct text for fluency.
The rule that saves it,
"Stylistic preference is NOT a defect",
was already
 there and is shared.
A first attempt neutralised that rule's neighbour into kind-agnostic wording,
 which would have silently reworded the ACCURACY prompt too;
the accuracy prompt is byte-identical to the one every artifact was produced
 under,
and a test pins it.

VALIDATED LIVE rather than assumed,
all three at 3/3 heard:

```text
SENSITIVITY refinement/clean          noneFound=3   (no claims)
SENSITIVITY refinement/omitting       removal=3
SENSITIVITY refinement/contradicting  corroborated=3
```

The control is the one that mattered.
This lane exists to rephrase,
so a prober reading rephrasing as damage would
 flag every refinement the pipeline ships,
and in a shadow-mode stage nobody reads,
that failure looks exactly like a clean
 run.
It reported nothing on the clean rewrite and caught both injected damages.

READING IT:
`score-probe` prints a REFINEMENT line,
kept separate from the
 accuracy figures because the two audit different edits against different
 baselines and their region counts are different units (rewritten slices against
 replaced envelopes).
`rewrittenSlices=0` prints a note saying so,
because every artifact before run
 012 predates the audit and a bare zero there would read as "the lane broke
 nothing" when it means "nothing asked".

## Run 012 settled, and the naturalness audit has its first live numbers

Run 012 ended on its SOFT budget,
not on a fault.

```text
DONE processed=4 of pending=54; artifacts=42/92 elapsed=43529854ms
SOFT budget reached after 43529854ms; not starting new entries
```

Two entries settled (40 to 42 artifacts) out of 4 processed.
`Y1Ran` burned its full 3-hour per-entry deadline and ended `status=ERROR
 aborted=true`.
Per #61 that costs a restart rather than an entry:
 `attempts.json` shows `Y1Ran: 1`,
so its banked chunks resume on the next
 attempt.
Throughput was poor and the log says why,
in the drain lines rather than in any
 pipeline stage:
 first-byte times of 43s,
112s,
143s,
161s,
191s,
217s and 262s on a single
 stage,
and one critic round losing 6 of 6 voices to the 360s deadline before
 the retry recovered all six.
That is provider latency,
recorded here as an observation with its evidence and
 NOT as a trend:
it is one run.

RUN 013 IS RUNNING,
launched from the same task at the same tip,
log
 `pass8-run-013.log`.

### The first REFINEMENT line, and how not to misread it

`score-probe` at 42 entries:

```text
PROBE       entries=42 shippedRecords=1791 unprobedRecords=0 regions=583
            majorityIntroduced=12 minorityIntroduced=67 noneIntroduced=504
CLAIMS      added=60 dropped=32 contradicted=1 unanchored=2
            degradedRosterRegions=0
REFINEMENT  rewrittenSlices=9 majorityIntroduced=1 minorityIntroduced=3
            noneIntroduced=5 added=3 dropped=2 contradicted=0 unanchored=1
```

The audit built in #58 is producing output on live corpus data.
Three artifacts carry it (`Toka_ls`,
`SS3B_0016`,
`TianqiChen666`),
being the
 ones settled since the lane was instrumented.

WHAT THESE NINE SLICES DO NOT SUPPORT:
any rate,
and any comparison against the
 accuracy line.
n IS 9.
The two lines count different units,
rewritten slices against replaced
 envelopes,
which `score-probe.ts` states at its own REFINEMENT summary;
 setting 1 in 9 beside 12 in 583 would be a ratio between incompatible
 denominators AND a rate from single digits.
That is the same move withdrawn from
 `doc/decision/introduced-defect-probe-gating.md`,
which asserted a stable rate
 on two events.
Report the counts with their denominator and stop there until the sample grows.

What IS supportable at n=9:
the lane's audit is wired end to end,
it reports
 non-zero,
and it does not report zero everywhere (which would have been the
 signature of a probe that never fires).
The `noneIntroduced=5` cell matters as much as the flagged one:
a probe reading
 every rephrasing as damage would have flagged all nine.
