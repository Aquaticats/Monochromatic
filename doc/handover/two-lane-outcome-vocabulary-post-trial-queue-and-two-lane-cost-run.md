# Lane outcome vocabulary handover: the post-trial queue and the two-lane cost run

The run queue decided in advance after the window trial,
the two-lane cost run and its sublinear cost finding,
the sweep owed when that run exited and the work that followed it,
the lifting of the launch gate,
and the afternoon sitting of 2026-08-17 that closed several queue items.

Part of [Handover: the lane outcome vocabulary and what is mid-flight](two-lane-outcome-vocabulary.md),
which lists every file of this record.

## Where the 2026-08-17 afternoon sitting got to

Newest first.
Everything named here is COMMITTED and pushed unless it says otherwise.

-   `ad76cc8da` records that queue item 3 is BLOCKED and that its census already exists.
    Read that
    item before touching the aligner;
    it cost this sitting an hour before the task itself was read.
-   `3c070bde8`,
    `cffd4c376` the retry's missing test (raised-then-decided) and a count correction.
-   `3ab0a2d15` re-reads the straggler-grace re-measure on the AT-RISK population and prices the
    retry from settled artifacts.
    Both are in `doc/audit/straggler-grace-remeasure.md`.
-   `fcc6a4b57`,
    `3a667a187` the re-measure write-up and `#105`'s retry,
    from the morning.

THE RETRY IS GFP'D AND THE STRIP IS RESTORED.
`judgeSlateWithRetry` was replaced in
`translate-stage.ts` by a direct `judgeTranslateSlate` call,
rebuilt,
and the suite reported
`AggregateError: 4 children failed in suite "runTranslateStage"`,
naming exactly the four retry
cases:
the twice-decline restamp,
the second-ask accept,
the evidence-into-refusal absent case,
and
the new raised-then-decided one.

THE FIFTH RETRY CASE STAYED GREEN ON PURPOSE.
"BUYS NO SECOND ROUND when the first one decided"
passes with the wiring stripped,
because a decision buys no retry either way,
so it is NOT strip
evidence and must not be counted as any.
Worth keeping straight:
a guard suite where every case
fails on a strip is usually a suite that is testing the wiring rather than the behaviour.

ITEM 6 IS BUILT,
persisting probe output into the runs directory,
which was decided and unblocked
once item 3 turned out not to be.
It landed as `src/corpus-run/probe-store.ts` with six tests,
and
`coverage-probe.ts` now writes through it while still printing to standard output.

TWO THINGS ABOUT ITS DESIGN worth not re-litigating:

-   IT IS SHARED,
    not coverage-specific,
    because `audit-sensitivity.ts` had the identical defect and
    its three runs of each arm on 2026-08-17 also survive only in a transcript.
    BOTH CALLERS ARE NOW
    WIRED (`d8dd1172c`),
    and the second one turned out to carry a worse defect than the one it was
    being fixed for:
    it had NO `import.meta.main` guard,
    so both arms ran at module scope and any
    import of that file bought a full checker roster twice.
    Verified from git rather than memory:
    the previous revision ends in two bare `await auditOne(...)` calls and contains no guard at all.
    Only the bundler's entry list stood between an ordinary import and the spend.

    THE GUARD IS DEMONSTRATED ONE WAY ONLY,
    and that is deliberate.
    Importing the built `.mjs` with
    `TRANSLATION_REPAIR_SYNTHETIC_API_KEY` cleared returns in 16 ms with no output,
    so the guard
    holds.
    The negative half,
    stripping it to watch the arms fire,
    is NOT run:
    it would spend a full
    roster,
    and the git evidence establishes the before-state without paying for it.
    Recorded here
    the same way `assertDerivationsAgree`'s missing strip demonstration was.

    `audit-sensitivity` RECORDS NO CORPUS PIN,
    deliberately,
    since it reads invented fixtures.
    That
    is what moved the corpus commit out of `ProbeRun` and into `subject`:
    the second caller could not
    fill the field honestly,
    which is the signal a field is in the wrong place.

    VERIFIED BY RUNNING IT FOR REAL,
    6 calls and about two minutes,
    which is both the boundary check
    and a fourth stability sample.
    It wrote
    `audit-sensitivity/2026-08-17T13-41-58.852Z-c93f970d.json`,
    18435 bytes,
    carrying three
    checkers,
    the subject naming its fixtures and arms,
    and BOTH ARMS' REPORTS WHOLE.
    It is the
    first `audit-sensitivity` result in this project's history that survives the terminal that ran
    it.

    WHAT THAT FOURTH SAMPLE SAYS,
    and it is not nothing:

    ```text
    flipped   heard 3/3   oracleVoices 3   agreed 1 (3 voices)   corroborated 0   near 3
    clean     heard 3/3   oracleVoices 0   agreed 0              corroborated 0   near 0
    ```

    THE CONTROL IS STILL CLEAN and the auditors still find the planted defect unanimously.
    THE
    STRICT TIER STILL MISSES IT:
    all three voices pointed at the oracle span and `corroborated` is
    zero,
    while the loose `agreed` tier caught it with all three.
    That is the fourth run in a row
    where the strict count would have reported a unanimous defect as nothing,
    which is exactly why
    the second tier was added,
    and it is now on disk rather than in a transcript.

GFP'D AND VERIFIED AT THE BOUNDARY,
both:

-   THE STRIP was the filename,
    replaced by a fixed `rows.json`.
    Exactly three cases failed,
    and
    they are the three loss-guards:
    keeping both runs,
    separating two builds of one instant,
    and the
    stamped name.
    The other three pass under a fixed name,
    correctly,
    and are not strip evidence.
    Run per file with `node src/corpus-run/probe-store.unit.test.ts`,
    which takes seconds against
    the full suite's ten minutes.
-   THE BOUNDARY CHECK was the real task against a throwaway `TRANSLATION_REPAIR_RUNS_DIR` at
    `--cap 0`,
    which short-circuits before any candidate and so spends NO quota.
    Exit 0,
    one file
    written,
    carrying both instants,
    a real six-model roster,
    the corpus commit,
    93 entries walked
    and a pipeline digest computed over built output.

AND IT FOUND SOMETHING,
which is why that check is worth running rather than reasoning about:
THE
REDIRECT THE PROBE WAS BUILT AROUND NEVER WORKED.
The tagged logger writes to STANDARD OUTPUT,
the
same stream the rows JSON goes to,
and the probe logs a line per candidate,
so
`coverage-probe > rows.json` yields progress lines wrapped around a JSON document that no parser
accepts.
The rows were never recoverable that way for any run that actually probed anything.
That
is not a regression from this change;
it is why a file is the answer rather than better discipline.
-   A PER-RUN FILENAME,
    carrying the instant and the pipeline digest,
    because both probes are rerun
    against the same subject ON PURPOSE to see whether a verdict is stable,
    and one fixed `rows.json`
    would let each rerun destroy the run it was bought to be compared against.

NOT the reason,
though an earlier draft of this section said so:
`coverage-probe.ts` is nowhere near
the line cap.
It measures about 209 CODE lines against a cap of 300,
which counts code only;
the 396
physical lines are mostly TSDoc.
The split is justified by the second caller,
not by size.

ALSO FOUND,
and not yet fixed:
`audit-sensitivity.ts` has NO `import.meta.main` guard,
so importing
it runs both arms and spends quota.
`coverage-probe.ts` carries that guard with a comment saying
exactly why.
Nothing imports it today except the bundler entry list,
so the hole is latent.

## The post-trial run queue, decided in advance

THE GATE HAS LIFTED.
The window trial finished 2026-08-16T23:58Z,
all 327 arms,
zero refusals.
Its result is in `doc/planning/translation-repair-window-trial.md`:
widening the judge's context did
not move selection,
and in the larger reading the effect is SMALLER than the run-to-run noise band.
The negative control also measured something nobody asked for,
and it constrains items 2 and 4
below:
the per-slice preserve-or-replace decision is about 19 percent unstable between identical
runs,
so no single pass settles a per-slice question.
Recorded on `#105` and `#108`.

THE WHOLE QUEUE IS NOW HANDLED,
as of 2026-08-16.
Item 1 ran,
item 2 was answered from the trial
ledger without building the bench,
item 3 was answered and its trade corrected,
and item 4 is
RUNNING;
see "The two-lane cost run,
launched 2026-08-16" for where to find it.
Nothing in the queue is waiting on a user answer.

What is left on the audit is not a measurement but a decision,
and it is the user's:
the instrument
is WIRED TO NOTHING.
Whether it runs per slice inside the translate lane,
or standalone over settled
version 2 artifacts,
changes what it costs and what it can be compared against,
and neither is
obviously right.
Recorded as Question 8 in `doc/planning/translation-repair-open-decisions.md`,
with
options,
a ranking and the reason for each step of it.
The user has confirmed quota is not a constraint (near full,
regenerating,
one reset in hand),
so
nothing below is held back for cost;
what held them back was the trial's validity,
since every one
of these calls the same six models.

1.  ~~**The two live arms of the rendering audit (`#85`)**~~ DONE,
    three runs of each arm;
    the
    polarity-flip fixture and the control that must not fire.
    They were the GFP for the whole
    instrument,
    and they earned their keep:
    the strict matcher missed a unanimous defect,
    which is
    why a second tier exists.
2.  ~~**`#105`'s decline-rate bench**~~ DONE,
    and the bench was never built:
    the window trial's
    ledger already held 327 real judgings under the production roster,
    which is better evidence than
    a synthetic bench and cost nothing.
    Decline rate 0.171 overall,
    but 0.063 on a full panel against
    0.287 on a panel of five and 0.692 on a panel of four,
    so declines are mostly VOICE LOSS rather
    than passages.
    Retrying the same slate decided 21 of 37 declines.
    Only 8 slices of 109 declined
    under both identical judgings.
    `#105` carries it;
    the policy decision is still the user's.
3.  ~~**The coverage rerun question (`#106`)**~~ ANSWERED,
    in
    `doc/planning/coverage-wire-rerun-trade.md`,
    and the trade recorded here was stated backwards.
    Ranked A > C > B > D:
    MEASURE the false-CARRIED rate first over the UNCHANGED wire,
    rather than
    change the wire or do nothing.
    The correction is that a false CARRIED is what a permissive
    evidence rule manufactures,
    so the block-scale null of ninety-six answers with no vote for
    absence is the measurement MOST exposed to the defect,
    and it is the one question 28 leans on
    against landing four.
    Section-scale verdicts are comparatively safe,
    since absence findings are
    not what the defect produces.
    Correspondence cannot be scored over the existing runs at all:
    the
    probe writes to stdout and nothing persists it.
    Not started,
    and must not start while `#114`
    holds the roster.
4.  **A few two-lane entries end to end**,
    to read the real per-entry cost against `HARD_CAP_MINUTES`
    of 180 before any full pass.
    `#92` measured the lane from bench calls;
    nothing has yet run both
    lanes over one document under the cap.
    LAUNCHED 2026-08-16,
    see "The two-lane cost run".

## The two-lane cost run, launched 2026-08-16

WHERE IT IS,
so a killed session can find it without this transcript:

-   Runs directory,
    throwaway and outside the repo:
    `~/temp/agent/two-lane-cost.l9Fpgk6V`
-   Combined stdout and stderr:
    `~/temp/agent/two-lane-cost.l9Fpgk6V/two-lane-cost.log`
-   Command,
    from the `translation-repair` worktree:
    `TRANSLATION_REPAIR_RUNS_DIR=<that dir> mise run
    //package/module/translation-repair:corpus-pass -- --only
    Aniloviraw,zheermao101,aiyysk,XingZ60` (moved on 2026-10-18 to `//package/translation-repair/corpus-pass:corpus-pass`)
-   Code identity,
    from the run's OWN `START` line rather than from this document:
    `tip=e79bb338 pipeline=sha256-tree-v1:e327ca8b`.
    The tip is the run-record commit,
    one later than
    the `4fed9d7bd` the plan check ran at,
    and the PIPELINE HASH IS THE SAME in both,
    so the delta
    is documentation and the code is unchanged.
    Read the hash,
    not the tip:
    only one of them
    describes what actually ran.

WHICH ENTRIES,
AND WHY THOSE.
Sizes are `page.md` blob bytes at corpus pin
`a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
over 92 pending entries whose distribution is
min 120,
q1 1481,
median 2323,
q3 4557,
max 41720:

-   `Aniloviraw`,
    1481 bytes,
    the first quartile.
-   `zheermao101`,
    2323 bytes,
    the median.
-   `aiyysk`,
    21455 bytes,
    second largest,
    the clean large point.
-   `XingZ60`,
    41720 bytes,
    the largest by a factor of two,
    so the worst case the cap must survive.
    It is also `#71`'s pathological alignment entry,
    so it doubles as a full-scale check of that fix.

WHAT IT IS FOR,
and what it cannot answer.
Per-entry wall time against the 180 minute cap,
the cost
split between the two lanes,
and the voice-loss rate under the current roster.
NOT per-slice
verdicts:
the window trial's negative control measured the preserve-or-replace decision as about 19
percent unstable between identical runs,
so nothing per-slice from a single pass is evidence.

HOW TO READ IT.
Each entry logs one `TALLY <id> status=... ms=<durationMs>` line,
and `durationMs`
also lands in the entry's artifact,
so cost is readable from the log or the runs directory.
An entry
that aborts at the cap is a RESULT,
not a failed run;
the soft budget is 72 hours and absorbs it.

NOTHING IS DECIDED BY THIS RUN.
Whether to widen the deadline for `hf:zai-org/GLM-5.2`,
seat a
replacement,
or accept the voice loss is a user bullet on `#105`,
and the run was deliberately
launched under the CURRENT roster and deadline so it reports that configuration as it stands.

## The cost run finished, and a seven hour cap clears the corpus (`#114`)

DONE 2026-08-17,
written up in `doc/audit/two-lane-entry-cost.md`.
The sweep listed under
"What is owed the moment the cost run exits" HAS BEEN RUN and is clear:
lint 0 warnings 0 errors,
types clean,
359 test groups passing.
That section is kept as the record of why it was deferred.

READ THE AUDIT RATHER THAN THIS SUMMARY IF THE CAP IS THE QUESTION.
The first reading of this run
was wrong by about a factor of three on how many entries the cap stops,
and the audit records both
the corrected numbers and the wrong ones under "What the first reading got wrong".

TWO OF FOUR ENTRIES NEVER FINISHED.
`XingZ60` (41720 bytes) and `aiyysk` (21455) each ran the full
three hours and produced no artifact;
`zheermao101` (2323) settled in 65.3 minutes and `Aniloviraw`
(1481) in 35.4.
Seven and a half hours bought two artifacts,
six of those hours buying nothing.

NEITHER ABORT WAS CLOSE TO HOPELESS,
which is the fact the first reading missed.
The aborted
entries KEEP their slice caches,
so file counts and mtimes recover per-slice timing directly:
`aiyysk` had settled 73 of its 80 slices and `XingZ60` 55 of 83,
both in the repair lane,
at 2.47
and 3.19 minutes per slice.
Whole-entry cost is therefore about 4.7 and 6.4 hours,
not the 10 and
19.5 the per-byte extrapolation predicted.

COST IS SUBLINEAR IN SIZE:
`cost_minutes = 0.2585 * bytes^0.694`,
fitting all four entries within 16
percent.
Per-byte cost FALLS threefold from the smallest entry to the largest,
so a rate taken from
small entries badly over-prices large ones.
That single error produced every wrong number.

A FULL PASS COSTS 119 to 128 HOURS.
At the current cap 4 to 6 entries abort,
wasting 12 to 18 hours.
AT A SEVEN HOUR CAP NONE ABORT,
for about 8 hours more in total,
because an aborting entry is
already paid for in full and thrown away.
The budget conclusion survives unchanged:
a full pass does
not fit the 72 hour soft budget at any cap.

SET ANY CAP WITH MARGIN.
`XingZ60` projects at 385 minutes,
so exactly 420 leaves 35 minutes inside
a fit whose residuals reach 16 percent and whose largest input is itself a projection.
Eight hours
rather than seven,
or set it after the queue lands and the rate is re-measured,
which `#114` already
says is the better order.

THE CACHE DOES NOT SPAN A REBUILD,
verified rather than assumed.
`slice-cache-namespace.ts:463`
resumes only when `cached === generation` and `:470` discards otherwise,
and the two surviving caches
hold 128 slices stamped `sha256-tree-v1:e327ca8b` while the current build digests to
`sha256-tree-v1:d8507690`.
About 6 hours of paid work was orphaned by the sweep that followed the
run.
Any plan that lets an entry span runs has to answer that first.

RECOVERED FOR FREE:
`XingZ60` confirmed the `#71` and `#74` alignment fix at full scale before it
aborted,
because preparation runs before the lanes.
The log reports `12 chunk pairs, 83 slices, 4
alignment findings`,
exactly what `doc/decision/translation-repair-unpairable-section.md` predicted.

THE SWEEP FOUND SOMETHING TOO,
and it is worth knowing the telemetry cost a file split:
the eight
lines of timing pushed `repair-translation.ts` to 301 lines,
so `repairTranslation` now lives in
`repair-entry.ts` (`c2a16470a`).
The boundary is real rather than convenient:
one half takes two
documents and decides how to slice them,
the other takes slices already decided.

## What is owed the moment the cost run exits

DO THIS FIRST,
BEFORE READING THE RUN,
because it is the half that is not yet verified.

`2ff6228d9` added per-slice cost telemetry (`slice-cost-log.ts`,
`slice-cost-read.ts`) and its unit
tests import from `dist/`,
which the cost run is reading.
Building would have swapped code under a
live multi-hour run,
so the build and the full sweep were deferred on purpose:

1.  `mise run //package/module/translation-repair:build` (moved on 2026-10-18 to
    `//package/translation-repair/<package>:build`)
2.  `mise run //package/module/translation-repair:test:unit` (moved on 2026-10-18 to
    `//package/translation-repair/<package>:test:unit`)
3.  `mise run //package/module/translation-repair:lint` (moved on 2026-10-18 to
    `//package/translation-repair/<package>:lint`)
4.  `mise run //package/module/translation-repair:lint:types` (moved on 2026-10-18 to
    `//package/translation-repair/<package>:lint:types`)

WHAT WAS ALREADY VERIFIED,
so this is a sweep rather than an unknown:
the source modules were
exercised directly with node,
outside `dist`,
and every assertion held.
The round trip between
writer and reader,
the early-exit path where a lane leaves a slice by `continue`,
a mixed log with
unrelated lines,
and all three refusals (half-written line,
unknown lane,
non-integer measurement).
What the sweep adds is the built artifact,
the lint rules and the types.

NOTHING ELSE MAY RUN `mise` IN THIS PACKAGE UNTIL THE RUN EXITS.
`lint:types` runs `tsc --build`
and `build` writes `dist/` outright;
`test:unit` reads `dist/` and would test the wrong code.

### The decided work that follows the sweep

THE DECIDED WORK THAT FOLLOWS THE SWEEP,
in order,
all of it now unblocked:

1.  THE VOICE LOSS,
    first because everything else is measured through it.
    135 of 327 judgings lost at
    least one voice of six,
    dominated by `hf:zai-org/GLM-5.2` abandoned 60000 ms after quorum.
    Widen
    that deadline or seat a replacement,
    then re-measure the decline rate.

    DONE 2026-08-17,
    recorded in `doc/decision/translation-repair-straggler-grace.md`.
    The window
    moved from 60 to 180 seconds;
    no model was replaced.

    WIDEN WON ON MEASUREMENT.
    Whole-call latency over 602 bench exchanges puts
    `hf:zai-org/GLM-5.2` at p50 24.0 s,
    p95 74.0 s,
    max 85.5 s and `hf:zai-org/GLM-4.7-Flash` at p50
    30.5 s,
    p95 72.9 s,
    max 88.6 s,
    so 60 seconds sat between those medians and their 95th
    percentiles and cut working voices by construction.
    In the cost run 91 of 96 losses were this
    window firing,
    95 of 96 were the two GLM models,
    and `hf:zai-org/GLM-5.2` was still heard in 220
    full panels.
    Replacing a model that answers in three quarters of its stages would have worked
    around a mis-set constant by discarding a working voice.

    ONE CLAIM I MADE HERE WAS WRONG and is corrected in that decision:
    widening does NOT push the
    wrong way on entry cost.
    The window is only paid when a voice is late,
    so the added wall time is
    about 21 minutes across a seven and a half hour run,
    under half a percent,
    against entries that
    cost hours.

    THE RE-MEASURE FINISHED 2026-08-17 and is written up in
    `doc/audit/straggler-grace-remeasure.md`.
    NO RUN IS IN FLIGHT and NO `mise` EMBARGO IS IN FORCE;
    an earlier version of this section said both,
    and a session reading it would have held itself
    back from a run that had already exited.
    Its artifacts and log survive at
    `~/temp/agent/grace-remeasure.UfE5pP1J`,
    also recorded in the scratchpad as `grace-run-dir.txt`.

    HEADLINE:
    per-stage voice completeness fell from 11 of 80 stages (0.138) to 3 of 77 (0.039) on
    the same two entries.
    One abandonment survived at 180000 ms on `hf:zai-org/GLM-5.2`,
    so the tail
    the decision warned about is real but is one call rather than a class.
    The other two residual
    losses are `schema-mismatch`,
    a different fault that the window never touched.

    COST ROSE,
    by more than the estimate the decision carried:
    +7.1 percent and +28.6 percent on the
    two entries.
    No unchanged-build repeat exists,
    so the run-to-run band is unmeasured and 21
    percent aggregate cannot yet be separated from provider variance.
    The audit says so at length.

    COMPARE ON PER-STAGE VOICE COMPLETENESS and nothing else,
    for any future re-measure.
    The
    old-window baseline on that definition is 63 of 483 stages,
    0.130,
    per entry 5/22,
    6/58,
    30/208,
    22/195.
    The 0.413 quoted elsewhere in this handover counts JUDGINGS and is not comparable;
    the 96
    loss lines in the cost run count retries and are not comparable either.

2.  ~~`#105`'s retry policy~~ DONE 2026-08-17 (`3a667a187`),
    as `src/translate-retry.ts`.
    A declining slate is judged ONCE more against the same panel,
    and what still declines is recorded
    as `no-candidate-backed`.
    The delivery vocabulary already said WHERE such a slice lands,
    so this
    is a reason rather than a new kind:
    with an incumbent it stays `incumbent-retained`,
    without one
    it stays `gap-remains`.
    Both exit doors are covered,
    since a decline RETURNS when an incumbent
    exists and THROWS when none does,
    and covering one would have applied the policy to half the
    slices.

    PRICED FROM THE SETTLED ARTIFACTS,
    since the retry landed after the last run and has never
    executed in production:
    8 of 40 slices declined under the new window and 4 of 40 under the old,
    every one of them `declined-indecision`,
    so the retry buys a second judging round on roughly one
    slice in five rather than the one in sixteen a full-panel rate predicts.
    `no-candidate` never
    occurred,
    so the exclusion in `RETRIED_DECLINES` protects nothing on this evidence.
    Budget
    accordingly;
    the audit carries the working.

    EVERY TIMING COMPARISON FROM HERE IS TWO CHANGES DEEP against the 2026-08-16 baselines,
    the
    window and this retry,
    and no arithmetic separates them afterwards.
    Record the `pipeline=` digest
    from a run's own `START` line beside any number taken from it.
3.  `#74` and `#98`,
    the aligner.
    **`#74` IS CLOSED.
    `#98` IS BLOCKED,
    and this item as written
    misdirects**,
    which cost a sitting on 2026-08-17 before the task itself was read.
    Corrected here
    so it cannot cost another.

    THE CENSUS THIS ITEM ASKS FOR HAS ALREADY BEEN RUN,
    on 2026-08-15,
    at zero quota,
    WITH a positive
    control.
    Of 92 complete pairs,
    85 take the equal-shape fast path,
    and forced alignment would pair
    NONE of them differently.
    The control was invented headings (`Intro/Life/Notes` against
    `Intro/Notes/Afterword`,
    equal counts,
    one dropped and one added) and `alignHeadingsForced`
    returned a non-identity script with two refusals and one shift,
    so the probe can show the defect
    and the corpus simply does not contain a case it can see.
    Do not rebuild this.

    WHY THE ZERO IS NOT A CLEAN BILL,
    and why routing the fast path through the aligner anyway is
    the wrong move:
    `headingAffinity` scores SHARED LATIN RUNS,
    so it pairs a handle carried across
    (`Mikä`,
    `wing`,
    `Baimao suki`) and has nothing whatever to prefer between two headings with no
    Latin in common,
    which is the common case here.
    Routing the 85 through it would change no pairing
    while looking like a fix,
    and REFUSING every equal-count pairing would discard real repair
    coverage to catch a case nothing here can detect.

    WHAT ACTUALLY UNBLOCKS IT is a heading signal that works on handle-free headings.
    Three
    deterministic substitutes are already measured and REFUTED,
    recorded on `#106` so they are not
    re-derived:
    neighbour length excess,
    distinctive body tokens (which made alignment WORSE,
    raising
    unpaired source sections from 11 to 18,
    because weak plentiful evidence promotes a spurious
    pairing to a trusted anchor),
    and float-equality in the scorer.
    The one instrument measured
    CORRECT on this question is the coverage stage,
    eleven of eleven at section scale,
    and wiring it
    is question 28's subject rather than a free choice.

    `#98` also carries `blockedBy: #106` in the tracker,
    and `#106` waits on question 28.
    Landing
    section-scale insertion behind it is decided (paragraph scale is refused);
    the aligner gate in
    front of it is not.
4.  SECTION-SCALE insertion,
    with the heading defect fixed,
    over the population that then reaches the
    matcher.
    When a source section's counterpart is a heading with no body,
    the body belongs UNDER
    that heading;
    landing it as designed would have produced eight duplicate headings on `XIEPT2`.
    PARAGRAPH-SCALE insertion is REFUSED on the evidence and is not part of this.
5.  The rendering audit,
    standalone over EVERY settled version 2 artifact rather than a sample.
    A
    census has no sampling error and can settle questions about particular entries.
    Nothing in the
    producing path changes,
    so an instrument whose production error rate is still unmeasured cannot
    reach what ships,
    and it can be re-run against those same artifacts when the instrument changes.
6.  Persisting `coverage-probe` output into the runs directory rather than stdout only.
    This is the
    one piece of the parked coverage work that survives on its own account,
    being the reason its
    2026-08-16 numbers exist nowhere but a transcript.

NONE OF IT BEFORE THE SWEEP.
Every item is code,
and there is already one unverified change in the
tree;
three would compound into a sweep that cannot say which change broke what.

### The slice-cost exit field, closed 2026-08-17

ONE KNOWN GAP TO CLOSE IN THE SAME SITTING:
CLOSED 2026-08-17 (`3309ffff8`).
A `SLICE-COST` line
could not say WHY a slice was cheap.
Both lanes leave a slice early on a cache hit and on an
insertion chunk,
and both emitted a row near zero milliseconds at any size,
which on a RESUMED pass
would have scattered near-zero points across every size band and flattened the very slope the line
exists to show.

Every line now carries `exit=`,
one of `computed`,
`resumed`,
`no-translation` or `unfilled`.
Ordinary completion is the default and needs no call,
so the common path cannot be forgotten;
the
other three name themselves before leaving,
which the disposable cannot infer once scope has ended.
The reader REQUIRES the field rather than treating it as optional,
because the telemetry landed
after the only pass that has run,
so no production log carries the older shape.

THE MIRROR HOLE WAS FOUND AND CLOSED IN THE SAME SITTING (`dab4c6946`):
`using` fires dispose on
THROW,
so a slice killed by the entry deadline was logging `exit=computed` with near-cap
milliseconds,
which is one poisoned row per aborted entry sitting inside the very population the
field exists to keep clean,
passing an `exit=computed` filter.
There is now an `aborted` exit,
READ
FROM THE SIGNAL at dispose rather than named at each throw site:
a slice can throw from the abort
check,
from the stages,
or from an assertion after them,
and `Symbol.dispose` is not told which.
A
named exit still wins,
since a cache hit bought nothing either way.
Both lanes now pass `signal`.

BOTH GUARDS WERE GFP'D,
each committed before its strip.

-   The round trip,
    which its own header calls the only thing keeping writer and reader agreed about
    a shape neither owns:
    the writer's `sourceChars=` was renamed to `sourceCharacters=`,
    rebuilt,
    and the suite exited 1 with that case failing by name and the reader reporting `sourceChars
    missing`.
-   The abort exit:
    the signal check was replaced with a bare `taken.exit`,
    rebuilt,
    and the suite
    exited 1 with the cut-slice case failing by name.

Restored in both cases,
and the suite passes.

### Reading the cost run once the sweep is done

THEN READ THE RUN,
and only then:
the `TALLY <id> status=... ms=` lines against the 180 minute cap,
the cost split between lanes,
the voice-loss rate,
and `XingZ60` as the full-scale check of `#71`'s
alignment fix.
The `6/6 heard` critic line already in the log is one stage of one entry and is not
a rate;
report what the whole run measures.

THE COST TELEMETRY DOES NOT APPLY TO THIS RUN.
It was added after launch,
so the run in flight
emits no `SLICE-COST` lines.
This run answers the ENTRY-scale slope,
over four entries spanning 28
times in size;
the next run answers the per-slice question in `#92`.

DO NOT RUN THE COVERAGE MEASUREMENT.
Option A of `doc/planning/coverage-wire-rerun-trade.md` exists
to say how much of the BLOCK-scale null to believe,
and block-scale insertion is refused,
so it
informs no decision that remains open.
Section-scale insertion does not need it:
the instrument was
eleven of eleven correct there,
and its false-CARRIED weakness suppresses insertion rather than
causing it.
The trade note stays as the record of why.

ONE PIECE OF IT SURVIVES ON ITS OWN ACCOUNT:
persisting probe output into the runs directory rather
than stdout only.
That is why the 2026-08-16 coverage numbers survive nowhere but a transcript,
and
it is worth doing whatever happens to insertion.

### The four decisions, taken on the best-quality guideline

THE FOUR DECISIONS,
taken on the best-quality guideline and recorded with their reasoning in
`doc/decision/translation-repair-four-answers.md`.
They were first put to the owner as option sets;
the owner answered and then WITHDREW the answers,
because three of the four sets were priced on cost
and cost is not a constraint here,
so they were decided on quality instead.
Two came out differently
from what the owner clicked,
and both differences are flagged in that file for a cheap veto:

-   The rendering audit runs standalone over EVERY settled artifact,
    not a sample.
    A census has no
    sampling error and can settle questions about particular entries;
    the sample won its earlier
    place on price alone.
-   Declines get one retry AND the voice loss gets fixed.
    These were offered as alternatives,
    which
    was the error:
    the retry treats the symptom and the deadline treats the cause.
-   Insertion splits in two.
    PARAGRAPH scale is refused on the evidence.
    SECTION scale is real and
    gets landed,
    but AFTER `#74` and `#98`,
    because 85 of 92 entries never reach the matcher and
    every section-scale number was drawn from the 7 that do.
-   The window trial is dropped,
    unchanged,
    and this one was decided on its merits rather than its
    price:
    a fourth arm would refine a measurement whose practical action is the same either way.

## The launch gate has lifted

The trial finished at 2026-08-16T23:58Z after 25622 seconds,
so the reason for holding the corpus
pass is gone:
nothing else is competing for the same six models.

What the trial cost in voices is worth carrying into whatever runs next,
because it is the honest
rate for this roster and this deadline:
135 of 327 judgings lost at least one voice of six,
a rate
of 0.413,
dominated by `hf:zai-org/GLM-5.2` abandoned 60000 ms after quorum.
Panels that seat six
and hear four still decide,
and the trial's strict reading threw two thirds of its slices away on
that account.

A pass launched now should either widen the deadline for that model,
seat a replacement,
or accept
that a two-thirds strict-population loss is the price of the current configuration.

One plan-mode pass ran against a throwaway runs directory to check the wiring at the
user boundary:
`PLAN ok`,
92 pending entries,
zero quota.
That exercises the scheduler
and stops before any entry settles,
which is why `settleEntry` has its own tests.
