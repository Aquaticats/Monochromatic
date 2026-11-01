# Translation repair history 2026-08-21 to 2026-08-23: the ratio guard's instrument, the sweep and the estimator

The discovery that the expansion-ratio instrument was already built,
why the pairing sweep was bought for a better reason than the first,
and the estimator chosen by measurement:
the exclusion shown not to be circular,
the median chosen over the pooled ratio,
and the minimum-count floor proposed,
measured and refuted.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-22, later still: `#163` found its instrument already built, and its sweep was re-aimed

`#163` asked for an expansion-ratio guard.
Its own closing note proposed the sharper form:
"a per-slice comparison against the archive's own ratio may be sharper than any corpus-wide band,
and costs nothing more".
That instrument already exists in production source,
and the task never mentioned it.

### The instrument was already built, and its header is a catalog of the traps

`src/displacement-ratio.ts`,
253 lines,
and `src/displacement-class.ts` beside it:

-   `sliceRatios` measures every slice and filters nothing,
    because an earlier version discarded short originals and threw away the strongest evidence there is.
-   `documentBaseline` returns the document's own expansion when it lands between
    `PLAUSIBLE_BASELINE_MIN` 1.9 and `PLAUSIBLE_BASELINE_MAX` 4.5,
    and `CORPUS_REFERENCE_EXPANSION` 2.86 otherwise,
    saying which of the two it used.
-   `classifyDisplacement` classifies each slice,
    computes a residual against that baseline,
    and keeps untranslated,
    target-only,
    relocation candidates and unpaired imbalances apart.
-   `MIN_RATIO_SOURCE_CHARS` 80 stops a short original's ratio being read as evidence at all.

The header already records that a document's own MEDIAN is contaminated by exactly what it detects,
and that the eligible-slice aggregate is only partly invariant under relocation.
A second ratio instrument built beside this one would re-hit both.

### What it measures, and what it decides

Read at the call sites rather than assumed:

-   Both consumers feed it `prepareDocumentPair({ sourceText, targetText, },)`,
    which is the ARCHIVE pair.
    It has never seen a candidate rendering.
-   Both consumers are probes under `corpus-run/`,
    `displacement-probe.ts` and `window-trial-probe.ts`,
    emitting telemetry.
-   No pipeline stage reads it.
    `translate-barrel.ts` only re-exports it.

So the instrument exists and the guard does not.
The neighbouring guards are all size-blind:
`translate-validate.ts` compares block skeleton,
references and code;
`consolidate-validity-floor.ts` refuses a slate with no structurally valid proposal.
A candidate that keeps the skeleton and deletes most of the prose inside it passes both.
`#155` named the complementary fault,
deleting what the archive carries and the Chinese does not,
bounded to a SILENT original.
An original that says something and a rendering that renders almost none of it is still unnamed.

### The sweep looked unnecessary, and the probe that could refute that did

The sweep's product is a cleaned outlier list,
which is worth buying only if the guard reads a parameter those outliers can move.
A per-slice guard comparing a rendering against the archive's own value for that slice draws no corpus-wide band,
so at first reading the 28 pairing calls bought nothing.
That reading was wrong,
and measuring it rather than asserting it is what showed the difference:

-   Only 854 of 1259 slices,
    67.8 percent,
    can serve as their own reference.
    The other 405 must read the document baseline.
-   Of the 64 flagged slices,
    36 are currently eligible to SET their document's baseline.
-   Removing them changes the baseline source in 7 documents,
    and moves it by up to 0.725 against a corpus value of 2.86.
-   170 of the 405 baseline-reading slices sit in a document whose baseline moves.

### Why the sweep is bought anyway, which is a better reason than the first one

Every number this task ever recorded,
the 1259 slices,
the 64 flags,
and both contamination probes,
was measured with `prepareDocumentPair`,
the DETERMINISTIC pairing.
Production preparation is `prepareDocumentPairWithRoster`,
at `corpus-run/pass-entry.ts:266`,
per `#131`.
The two slice a document differently.
Calibrating a guard on the deterministic numbers would calibrate it on slices the pipeline never produces,
so the sweep is not "clean the outlier list" but "measure on the population the guard will read".
That justifies it whether or not the contamination result had come back null.

### What is running

`~/temp/agent/163-roster-sweep.mjs`,
sequential by entry:
29 entries,
110 paid pairing rounds,
each asking all 6 roster voices.
Sections are already asked in order inside the preparation,
so one entry at a time holds the whole sweep at six concurrent streams,
under the roughly seven the aggregate-concurrency stall was measured at.
Entries are ordered by baseline exposure,
so a stall still leaves the decisive ones settled.
It stores counts only,
never text,
and reports errors by class name,
because pairing and parse errors quote the passage they failed on.

The guard's shape is settled and does not reopen:
per-slice archive reference where usable,
document baseline fallback,
and a named fault in the `contest-ballot-wire.ts` shape both deciders read,
which is `#155`'s mechanism.
The sweep decides its calibration inputs only,
and the guard is built after it rather than beside it.

## 2026-08-22, still later: `#163`'s estimator was chosen by measurement, and the floor it implied was refuted

Three things were settled here,
all at zero quota,
because deterministic pairing
is pure local computation and the question is about the ESTIMATOR rather than
about the pairing.

### The exclusion is not circular, which the source had to say

`displacement-class.ts` computes its baseline over slices that are class
`translated` and at least `MIN_RATIO_SOURCE_CHARS` long,
then derives both
`residual` and the high-slice flags FROM that baseline (`slice.ratio >=
baseline.expansion * HIGH_FACTOR`).
So excluding `highIndices` from the baseline would be circular and would need a
fixpoint.

The contamination actually measured is a different set.
It uses the absolute predicates:
block disparity above one,
ratio below 0.8,
or
ratio above 10.
None of them reads a baseline,
so excluding what they flag is a plain filter and
terminates in one pass.
This distinction decides the design and is the reason the guard can be written
at all.

### The centre should be a median, measured on 89 entries

The shipped estimator is a POOLED ratio,
the sum of translated characters over
the sum of original characters,
so the longest slices decide it.
That is why `Zha_Ke` reads 16.85 at document aggregate,
and why the
minimum-length filter had to be added.

Scored by how far each centre moves when the flagged slices are removed,
over
the 22 entries that carry contamination:

-   pooled:
    median move 0.168,
    seven documents change baseline source
-   median of per-slice ratios:
    median move 0.086,
    two documents change baseline source

The median is about half as sensitive and flips its fallback decision two times
instead of seven.
It also leaves 82 of 89 documents able to trust their own expansion,
against 80
for the pooled ratio.

Switching from shipped (pooled,
uncleaned) to proposed (median,
cleaned) moves 7
of 89 documents across the document/corpus-reference boundary,
five of them
gaining a document-specific baseline they were previously denied.
Median baseline shift is 0.110.

### The minimum-count floor was proposed, measured, and refuted

Instability falls monotonically with the number of clean eligible slices:
median
move 0.334 at one to three slices,
0.271 at four to six,
0.132 at seven to
twelve,
and 0.017 at thirteen or more.
That reads as an argument for refusing a document its own baseline when it has
too few slices.

It is wrong,
and the split-half probe says why.
Splitting each document's clean slices into alternating halves and comparing the
two centres estimates the estimator's own noise without needing a ground truth
the corpus does not have.
Against that,
the corpus reference costs whatever `|own - 2.86|` is.

-   two to three slices:
    own error 0.315,
    reference error 0.605
-   four to five:
    0.164 against 0.583
-   six to eight:
    0.243 against 0.333
-   nine to twelve:
    0.165 against 0.240
-   thirteen or more:
    0.092 against 0.215

The document's own centre wins at EVERY count,
and 60 to 70 percent of
individual documents are better served by it in every band.
Thin documents are noisy,
but 2.86 is more wrong than they are,
so a floor would
trade a noisy estimate for a biased one.
No floor is added.

### What is not yet settled

The calibration NUMBERS still wait on the roster sweep,
which is ordered by
baseline exposure and so has delivered its high-exposure head first.
Band and tail endpoints must not be read off a partial prefix.

The sweep does NOT pass a `pairingCache`,
so its roster pairings are not
persisted and re-reading them would re-buy every round.
Any later probe that needs roster-paired per-slice numbers has to be folded into
a sweep that caches,
rather than run beside this one.
