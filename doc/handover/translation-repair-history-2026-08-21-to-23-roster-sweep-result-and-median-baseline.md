# Translation repair history 2026-08-21 to 2026-08-23: the roster sweep's result and the median baseline landing

The completed sweep:
production pairing never adds a flag and contamination outlives better pairing,
the first commit,
where the median landed and the file's own history was found wrong about why,
what the exclusion buys,
and the second commit's blocker,
a collision with the policy that protects page-only content.
The sweep behind the partial-prefix figures its opening mentions,
and the warning not to read band and tail endpoints off a partial prefix,
are recorded in `translation-repair-history-2026-08-21-to-23-ratio-guard-instrument-and-estimator-choice.md`.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-22, sweep complete: production pairing never manufactures a flag, and contamination outlives it

All 29 flagged entries were re-paired with the production roster,
the same six
voices a pass asks,
and none refused.
These are whole-population numbers,
so they can be read as final;
the earlier
partial-prefix figures in this file cannot,
since the sweep was ordered by
baseline exposure and delivered its high-exposure head first.

### Pairing quality is worth about a third of the flags

Deterministic pairing against production roster pairing,
summed over the 29:

-   slices 643 to 632
-   flagged 64 to 40
-   eligible 425 to 430
-   contaminating 36 to 35
-   fallback 218 to 202

So roughly 38 percent of the flags measured under deterministic pairing were
artefacts of the pairing rather than displacement in the document.

### Production pairing never ADDS a flag, which settles the open question

Seventeen entries lose flags,
twelve are unchanged,
and **no entry gains one**.
The question recorded earlier,
whether the 63 entries the sweep never re-paired
might sprout flags under roster pairing,
is answered in the direction that
matters:
roster pairing is uniformly equal or better at not manufacturing flags,
so the unswept entries cannot be hiding flags that only production pairing
would raise.

That is evidence about FLAG COUNT only.
It says nothing about whether an unswept entry's baseline moves,
so a false-fire
sample after the guard lands is still owed.

### Contamination is not fixed by better pairing

The total barely moves,
36 to 35,
and that near-equality is OFFSETTING rather
than stable:

-   five entries gain contaminating slices:
    `shihai4h` 1 to 2,
    `windward0032` 2
    to 6,
    `interrgned` 1 to 2,
    `cheonwoomaeng` 0 to 1,
    `lintong` 0 to 1
-   eight lose them:
    `zheermao101`,
    `gqt`,
    `Futajuhuacha`,
    `MizuharaNagisa`,
    `dogesir_`,
    `XingZ60`,
    `Huasheng`,
    `MeowBot233`

Nineteen of 29 entries still carry contamination under production pairing.
Reporting the total as unchanged would be wrong twice over:
it is not unchanged,
and the entries beneath it move in both directions.

This is the finding the guard rests on.
Better pairing removes flags that were never real,
and leaves the ones that are,
so the slices that survive must still be kept out of the baseline they would
otherwise define.

### The band is not what refuses documents

Under production pairing,
24 of 29 documents trust their own expansion,
and
those baselines run from 1.962 to 3.785,
median 2.970,
quartiles 2.431 and
3.166.
Every one of them sits inside `PLAUSIBLE_BASELINE_MIN` to
`PLAUSIBLE_BASELINE_MAX`,
so the band is not the thing turning documents away.
Six documents change baseline source between the two pairings,
three each way.

### What the guard now has to do

1.  Exclude flagged slices from baseline-setting.
    Non-circular,
    since the
    absolute predicates never read a baseline.
2.  Take the centre as a median of per-slice ratios rather than the pooled
    ratio,
    which the longest slices decide.
3.  Add no minimum-count floor,
    which was measured and refuted.
4.  Feed a named fault in `contest-ballot-wire.ts`,
    following the mechanism
    `#155` established,
    because `translate-validate.ts` and
    `consolidate-validity-floor.ts` are both size-blind and `#155` covers only
    the silent-original direction.

## 2026-08-22, `#163` Commit A: the median landed, and the file's own history was wrong about why

Commit `39108d02c`.
`documentBaseline` now reads a document's expansion from the MIDDLE slice
rather than from a pooled character aggregate,
and slices whose sizes are
implausible on their face are kept out of the set that sets the bar.

### The comment in the file argued against the change, and it was stale

`displacement-ratio.ts` carried a region comment headed "WHY A DOCUMENT'S OWN
MEDIAN IS NOT THE STATISTIC",
citing `shi_Yumiaoya`'s median of 0.76 as the
failure that motivated pooling.
That failure does not reproduce.
Under both estimators `shi_Yumiaoya` falls back to the corpus reference,
because the eligibility filter,
which was added by a LATER change than the
comment,
removes its untranslated sections before either estimator sees them.
The filter was doing the work the comment credited to pooling,
and nothing had
ever checked which of the two was responsible.
The comment now records all three historical readings in order:
median over all slices,
pooled aggregate,
median over the filtered set.

Read this as a standing warning.
A comment that names a concrete failure is evidence about the code AT THE TIME
IT WAS WRITTEN,
not about the code now.
Re-run the cited failure before letting it veto a change.

### What the exclusion buys, measured

Excluding implausible slices from the baseline set moves 8 documents beyond
what ratio tails alone would move,
and flips 1 outright:
`saurikissa` gaps to 0.87.
That is why `SliceSize` gained required `sourceBlocks` and `targetBlocks`
rather than staying a two-field character record,
and why `sliceSizeOf` became
the single site that counts both.
Both corpus-run probes were repointed at it,
so no caller hand-rolls the shape.

Non-circularity holds and was checked rather than assumed:
every predicate in `slice-implausible.ts` reads FIXED endpoints
(`IMPLAUSIBLE_MIN_RATIO` 0.8,
`IMPLAUSIBLE_MAX_RATIO` 10,
`MAX_BLOCK_COUNT_GAP` 1) and never reads the baseline it helps compute.

### Agreement and downstream effect

Implementation against prototype:
89 of 89 exact agreement,
0 disagreements,
re-confirmed after the barrel split with byte-identical output.

Old build against new build,
over the whole corpus:
the baseline moves by more than 0.01 in 77 of 92 entries and its source flips
in 7,
but `relocationCandidates` changes in only 3 entries,
+6 and -0.
The mechanism was settled rather than inferred:
no high slice is ever GAINED,
`windward0032` loses high slice 17,
and every
gain is donor-side,
because a higher baseline enlarges every ordinary slice's
deficit.

### Structural notes

`translate-barrel.ts` hit 301 lines against the 300 limit.
Fixed by extracting `displacement-barrel.ts`,
NOT by disabling the rule and
NOT by moving lines into whichever neighbouring file had room.

`stylistic(chain-per-line)` rejected `sliceImplausibility({ slice, },).length`.
Splitting the chain across lines did not satisfy it;
naming an intermediate `const reasons` removed the chain and the finding.

GFP performed on the exclusion test:
removing `isPlausibleSlice` from the filter,
rebuilding and running failed it
with `actual: 'corpus-reference', expected: 'document'`.
Restored with `git checkout --` and rebuilt green.
The guard was committed BEFORE the removal,
so the restore discarded nothing.

Gate at the commit:
lint 0 warnings and 0 errors,
types exit 0,
build exit 0,
suite 527 PASS,
0 FAIL,
exit 0.

### Commit B is blocked on a collision, not on effort

The plan was two named faults in `contest-ballot-wire.ts`,
one per tail
direction,
following `#155`'s policy-line mechanism.
The SHORTER direction is safe:
a candidate at 0.4x its source is missing
Chinese content whatever the archive did.
The LONGER direction collides with `#155` itself.
`CONTEST_POLICY` already tells judges that where the Chinese is silent rather
than contradicting,
KEEPING page-only content is correct.
A candidate that preserves a long page-only region is far-longer against the
Chinese AND is the right candidate,
so a bare fault name would instruct the
judge to penalise the exact behaviour `#155` protects.
That is the shape of `#143`,
a criterion licensing the wrong outcome.

Two resolutions,
to be chosen by measurement rather than taste:
gate far-longer on the candidate also exceeding the ARCHIVE's ratio,
or drop
the fault label in that direction and give judges the three numbers plus a
policy paragraph teaching the reading.
The second is what `DNL` prefers,
since a name that fires on correct
preservation asserts a fault that is not one.
