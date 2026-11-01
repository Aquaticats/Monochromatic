# Lane outcome vocabulary handover: the 2026-08-17 corpus pass, its freeze and the stop at six entries

The sittings around the 2026-08-17 corpus pass,
newest first:
the two settled pools and what six entries measured,
the night after the freeze lifted,
the runaway-call guard built during the freeze,
the evening grace measurement,
the freeze itself and the decision to stop at six entries,
and the state recorded at 23:22Z.

Part of [Handover: the lane outcome vocabulary and what is mid-flight](two-lane-outcome-vocabulary.md),
which lists every file of this record.

## Two pools now exist, and they must not be confused

READ THIS BEFORE ANY DRAW.
There are two settled pools and only one of them may feed a sample.

`~/translation-repair-runs-20260817/artifacts`,
SIX ENTRIES,
NATURAL ACCUMULATION.
Acheron,
AkiraComplex,
AmbeR_the_anpa,
Anilovr,
Arita,
Chinatsu_Suzuki.
Drawn in the order the pass takes
them,
so a draw over it is a draw over the corpus.
`#60`'s damage sheet came from here.

`~/translation-repair-runs-flagged-20260818/artifacts`,
FIVE ENTRIES,
HAND PICKED.
lintong,
wangzihao980,
GLaDOSister,
dogesir_,
saurikissa,
chosen because the displacement probe flags them.
`corpus-pass` warns about this itself when `--only` is used,
in as many words:
run it into a
throwaway directory so a hand-picked entry never enters a pool later draws treat as natural
accumulation.
IT MAY BE USED for the flagged-versus-unflagged comparison `#107` waits on,
and it
MAY NOT be used for any precision,
damage or accumulation draw,
because its entries were selected
for the property being measured.

## What the six-entry pool measured

REPLACEMENT RATE 0.740,
37 of 50 slices,
0 withdrawn,
translate lane.
The three-entry reading was
0.739,
so it held as the denominator doubled.
Pooled figure quotable;
per-entry rows are not,
and
six entries cannot separate an effect from an entry.

ZERO DISPLACEMENT FLAGS across all six,
which is why the flagged pool had to be made.
That is a
real zero rather than a dead instrument:
the same probe run flags 29 of 92 entries,
66 flagged
slices against 1260,
about 5 percent,
which over 50 slices predicts two or three.

## After the freeze lifted, 2026-08-17 night

THE PASS STOPPED CLEANLY at exactly 6 entries,
the watcher reporting
`reached 6 entries, stopping pass 3768707` then `stopped`.
Pool at
`~/translation-repair-runs-20260817/artifacts`:
Acheron,
AkiraComplex,
AmbeR_the_anpa,
Anilovr,
Arita,
Chinatsu_Suzuki.

THE RUNAWAY GUARD IS NOW WIRED AND PROVEN.
Built,
tested and linted once the producing path was
free:
`7aa4866fa` wires `drainBody`,
and the strip test shows it fails without the guard
(`expected 'drained' to equal 'raised'`,
drain reads 3,060,000 characters).
The healthy long call
logs 1,652,574 in both runs,
which is the control.
`lint:types` and `lint:oxlint` clean.

THE SOL REVIEW RETURNED NOTHING.
`timeout 3600 pi --model openai-codex/gpt-5.6-sol` exited 124 with
zero lines after the full hour,
which is the documented "it may never return" case rather than a
failure.
So the three stream modules have had NO second-reviewer pass.
Worth redoing with a smaller
paste,
one file rather than three,
if a reviewer is wanted.

### `#60` redraw: the pool was ready, the draw was not

The draw failed on the settled pool with
`artifact parse failed at artifact Acheron.issues: expected an array`.
It read version 1's
top-level issue list;
these artifacts carry `lanes.repair` and `lanes.translate`.

MOVING THE FIELD READ WOULD HAVE BEEN THE WRONG FIX,
quietly.
Both lanes ship replacement text.
Measured over all six entries the population is 69 regions,
32 repair against 37 translate,
so a
repair-only draw would have covered 46 percent of the regions where this pipeline replaced text
while calling itself a draw over the shipped regions.

Fixed in `b71ea7328` and `0b6d9afb0`:
`corpus-run/damage-region-v2.ts` reads both lanes' delivery
ledgers,
keeps `replacement-shipped` rows,
records the lane per row,
and counts rows that filled a
passage with no incumbent rather than dropping them (0 such rows in this pool).
Uses the artifact's
own text rather than re-slicing,
per `#115`,
which also removed the old unplaceable-region failure.
Draw domain bumped to `damage-sample/v2` so the two populations cannot claim one draw.

## Evening sitting, 2026-08-17: runaway calls

WHY THIS EXISTS.
The user reports that the provider does not auto-terminate token degeneration,
so
runaway calls must be ended by us.
Decision recorded in
`doc/decision/translation-repair-runaway-call-termination.md`;
work tracked in `#119`.

THE USER'S THREE CONSTRAINTS,
all of which shaped the design:

-   DO NOT SEND `max_tokens`.
    The provider is a two-person shop and a cap only helps if implemented
    correctly;
    relying on one turns "runs too long" into "truncates silently",
    which is worse.
    Keep the request body lean.
    No production stage sets it today,
    so this is a decision to keep
    current behaviour deliberately rather than by omission.
-   CATCH DEGENERATION IN THINKING TRACES,
    "I will output."
    repeating without end.
    This is the case
    that escapes everything else,
    because such a model emits no answer and a content-only scan sees
    an empty string,
    which reads as a short reply.
-   SOME MODELS LEGITIMATELY WRITE A GREAT DEAL,
    and that is not bad.
    Length never condemns.

BUILT AND COMMITTED,
none of it wired into the drain yet,
so nothing in the running pass changed:

-   `stream-degeneration.ts` `watchForDegeneration`,
    `8eb62c0ac` and `bf15afaf1`
-   `stream-delta-scan.ts` `scanStreamDeltas`,
    `8517282c1`
-   `stream-runaway-watch.ts` `watchRunaway` plus `StreamDegenerateError`,
    `fd62d7b37`

NOT BUILT OR LINTED YET.
The worktree used for it had no `node_modules`,
so everything was exercised
by running `node` against the TypeScript source directly:
9 detector checks,
8 scanner checks driven
end to end,
6 watch checks.
Unit test files exist beside each module and import `dist`,
so they have
never run.
RUN `mise run //package/module/translation-repair:test:unit` (moved on 2026-10-18 to
`//package/translation-repair/<package>:test:unit`) AND `lint:types` ONCE THE
FREEZE LIFTS,
and expect to fix things.

THE ONE REAL FALSE POSITIVE FOUND AND FIXED:
verse carrying a refrain scored 0.036 distinct,
which
the ratio alone condemns,
and this corpus contains verse.
The fix was to refuse to speak below about
131000 characters of generated text,
roughly fifteen times the longest model output in any of the 56
settled artifacts.
Do not lower that bar without re-checking verse.

MISREAD ONCE,
DO NOT REPEAT IT:
`chars` in the drain progress line counts RAW server-sent-event
bytes,
envelope included,
not generated text.
A median of 324,995 is an ordinary reply.

## What the evening measurement said, and what it did not

`doc/decision/translation-repair-straggler-grace.md` now carries a second reading from the evening
pass.
IT DOES NOT REPLACE the morning's `doc/audit/straggler-grace-remeasure.md`,
which already ran
the named control and is the re-measure of record.
An earlier draft of that section wrongly said the
re-measure was still owed;
corrected in `d2573ed7a`.

Genuinely new from the evening data:

-   THE DEFINITION CANNOT SEE THE TRANSLATE LANE.
    `translate-document.ts:391` logs selection only
    when ZERO translators were heard,
    so partial loss in `select` leaves no `heard` line.
    Of 30
    voices lost in the snapshot,
    17 went in `select` and 1 in `refiner`.
    The instrument counts 10
    of 30.
    Both sides of every comparison are measured the same way,
    so nothing is invalidated,
    but
    the figures understate loss and the definition should be extended before the next comparison.
-   EVERY ABANDONED VOICE IS A GLM,
    29 of 29 attributed,
    and none is Kimi-K3,
    which appears more
    often in the log than any other model.
    Supersedes `#77`.
-   THE RESIDUAL IS CENSORED.
    An abandoned call is killed at quorum plus 180 s,
    so its duration is
    unknown and strictly greater.
    It is not shown to be a latency finding,
    and if these calls are
    degenerating then no window would ever have helped them.

## Pass state as of 2026-08-17 evening

-   Pass PID `3768707`,
    alive,
    cwd `package/module/translation-repair` (moved on 2026-10-18 to the packages under
    `package/translation-repair`),
    runs dir
    `/home/user/translation-repair-runs-20260817` via `TRANSLATION_REPAIR_RUNS_DIR`.
-   Watcher PID `3953196` polls that artifacts dir every 60 s and kills the pass at 6 entries.
    VERIFIED to watch the same path the pass writes to.
-   4 artifacts settled,
    5th in flight.
    Frozen snapshot of the log at
    `~/temp/agent/grace-remeasure-snapshot.log`.
-   AUTO-PUSH IS FAILING with GitHub 500s as of 00:00Z;
    `d2573ed7a` is committed locally and not
    pushed.
    Retry `git push` from this worktree.

## The 2026-08-17 corpus pass: freeze, interim state and the stop at six entries

### A PRODUCING-PATH FREEZE IS IN FORCE while the corpus pass runs

Started 2026-08-17 18:00Z:
`mise run //package/module/translation-repair:corpus-pass` (moved on 2026-10-18 to
`//package/translation-repair/corpus-pass:corpus-pass`) over 92
entries,
`TRANSLATION_REPAIR_RUNS_DIR=~/translation-repair-runs-20260817`,
log
`~/temp/agent/corpus-pass-20260817.log`.
This is `#108`,
and it is also the only source of the
more ENTRIES that `#115` proved the rendering audit needs.

DO NOT CHANGE ANYTHING THE PIPELINE RUNS,
and do not rebuild after doing so,
until it finishes or
is abandoned.
The running process is safe either way,
because Node loaded its code at startup and
`corpus-pass` reads its digest before settling anything.
THE RESUME IS NOT SAFE:
`assertResumableGeneration` throws `GenerationDriftError` when the settled entries in a pool record
a build other than the one resuming,
and the driver's own plan output says why in as many words,
that continuing "would stamp a second pipeline into one pool and every reader that computes a rate
would then refuse the lot".
At up to 7 hours an entry over 92 entries,
this run will want resuming.

WHAT THAT RULES OUT for now,
each otherwise ready:

-   `#91`'s half-weight self-certification.
    Settled and authorized,
    and it changes how the checker
    stage tallies,
    which is the producing path.
-   Anything in `#90`,
    `#100` or `#106` that touches slicing.

WHAT IS STILL SAFE:
telemetry readers,
reporting entry points,
documentation,
decision write-ups,
and anything under `corpus-run/` that only READS.
`slice-cost-report.ts` was built under this
freeze and is the model:
it reads a log and prints.

THE FREEZE IS NOT A REASON TO IDLE.
Re-read the cost telemetry as the pass accumulates slices;
the
answer it owes `#92` improves on its own:

```sh
mise run //package/module/translation-repair:slice-cost-report -- ~/temp/agent/corpus-pass-20260817.log
```

Moved on 2026-10-18 from `//package/module/translation-repair:slice-cost-report`
to `//package/translation-repair/slice-cache:slice-cost-report`.

### Where things stand at 18:45Z, with the pass still running

THE PASS:
153 lane starts,
7 slices priced,
no aborts,
no entry settled yet.
First artifacts land in
`~/translation-repair-runs-20260817/artifacts/`.
The freeze above is still in force.

LANDED UNDER THE FREEZE,
all of it read-only or tooling:

-   `slice-cost-report.ts` and its task.
    Nothing had ever read the per-slice cost telemetry,
    which
    is `#71`'s failure exactly.
    Interim reading on `#92`:
    `ms/char` falls fourfold across three size
    bands while `min/slice` only doubles,
    which is the shape a FIXED per-slice overhead makes.
    Seven
    slices,
    all repair lane,
    so it is a shape and not yet a finding.
-   `runner-closure.ts`,
    `#116` closed.
    Runs now record which chunks the entry imports,
    so two runs
    can be compared for "same code" by string equality.
    The whole-tree digest could never say that.
-   Sample manifests record the pipeline that settled their pool,
    `#60`'s last named mechanic.
-   The audit reader accepts the FLAT layout a pass writes,
    not only the hand-built nested archive.
    This one mattered:
    pointing the audit at a pass's own output found nothing and refused with "no
    artifacts under",
    which reads like an empty pass rather than a layout it cannot see.
    The four
    archived artifacts only ever read because somebody copied them into run-set directories by hand.
-   `doc/planning/artifact-archive-text.md`,
    `#96`:
    its premise was measured and is wrong.
    Storing
    the whole target text costs 0.6 to 0.9 percent of an artifact,
    not the "doubles" the task
    assumed,
    because an artifact is judge evidence rather than text.

WHEN THE PASS LANDS,
in order:

1.  Re-read the cost telemetry;
    by then it will include translate-lane slices and the larger bands.
2.  Re-run the settled rendering audit over the new artifacts.
    It reads the flat layout directly
    now,
    so point it at `~/translation-repair-runs-20260817/artifacts` with no copying:
    `mise run //package/module/translation-repair:rendering-audit-settled -- --archive <that path>` (moved on 2026-10-18
    to `//package/translation-repair/probe:rendering-audit-settled`).
    More ENTRIES is the one thing `#115` proved the audit needs.
3.  `#108`'s replacement rate,
    which is what `#107`'s remaining decision waits on.
4.  Lift the freeze:
    `#91`'s half-weight first,
    since it is settled,
    authorized and located.

### The pass is being STOPPED AT 6 ENTRIES, on the user's instruction

Decided by the user 2026-08-17 22:50Z,
who noted 6 is low and defensible at this stage.
A watcher
polls the artifact directory and stops the pass by PID once six exist;
the seventh entry's slices
stay in the cache,
so nothing is lost and a later run resumes them.

WHY NOT MORE:
measured rate is about 86 minutes an entry (73,
139 and 45 for the first three,
the
spread being #114's size dependence).
All 92 would take about 128 hours against a 72-hour soft
budget,
so the pass would have stopped near 50 anyway,
and the freeze would have held for days.

## The GLM claim retracted, and the order of work when the freeze lifts

### The GLM finding, corrected

A first reading said "GLM models are slow" from abandon counts alone.
THAT IS RETRACTED.
The same
window carries 262 successful GLM answers against 24 abandons,
and the abandons are stage-shaped:
15 `select`,
6 `panel`,
4 `critic`,
0 `checker`,
spread evenly with no clustering.
Selection carries
the largest prompts (`#92`:
12119 tokens against a slate of 7),
so the shape is a tail on the
largest prompts rather than a slow model.
It is a hypothesis,
not a finding.

IT CANNOT BE SETTLED WITH WHAT IS LOGGED,
which is `#118`,
and the reason is better than first
recorded.
Streaming is always on,
so an aborted call HAS ALREADY DELIVERED partial content:
`drainBody` accumulates it in `parts`,
then the catch rethrows and the accumulated text is dropped.
The answer to "was it mid-reasoning" is sitting in memory at the moment of the cut and is thrown
away.

WORSE,
AND IT INVALIDATES A MEASUREMENT:
the `drainBody` progress line is emitted AFTER the
try/catch,
so an aborted stream logs nothing at all.
Every latency figure read off a pass log
therefore covers SURVIVING calls only.
The "1349 streams,
mean firstByte 1822ms" figure recorded
earlier excluded every abandoned call by construction,
which is survivorship bias;
it should not be
quoted,
and it says nothing about the stragglers it was used to describe.

### When the freeze lifts, in order

1.  `#91` half-weight self-certification.
    Settled,
    authorized,
    located at `tallyResolutionChecks`
    in `src/tally-resolution.ts`,
    called from `runCheckerStage`.
2.  `#118`,
    the two diagnostics above.
    Small,
    and they make `#68`'s latency axis answerable.
3.  Re-read the slice cost telemetry,
    which by then includes translate-lane slices.
4.  Re-run the settled rendering audit over the six new artifacts.
    The reader takes the flat layout
    now,
    so point it straight at `~/translation-repair-runs-20260817/artifacts`.

## READ THIS FIRST AFTER A COMPACTION (written 2026-08-17 23:22Z)

### What is running right now

```text
corpus pass      PID 3768707, node dist/final/node/corpus-pass.mjs
runs directory   ~/translation-repair-runs-20260817
artifacts        ~/translation-repair-runs-20260817/artifacts   (FLAT, no run-set subdirectories)
log              ~/temp/agent/corpus-pass-20260817.log
watcher          background task bkfhlg7ya
progress         3 entries settled of the 6 the user asked for, ~86 min an entry
```

THE WATCHER polls the artifact directory and kills the pass by PID once six artifacts exist.
It
exits early and says so if the pass dies first.
If it is gone after a compaction,
do the same by
hand;
there is nothing subtle in it:

```sh
A=~/translation-repair-runs-20260817/artifacts
while [ "$(ls "$A" | wc --lines)" -lt 6 ]; do sleep 60; done
kill 3768707
```

DO NOT write a `pgrep --full` wait loop for this.
It matches the shell running it and never exits;
that trap was documented here at 09:16 today and then walked into at 15:50 anyway.
See
`doc/troubleshooting/pgrep-wait-loop-matches-itself.md`.

Stopping loses nothing:
the in-flight entry's slices stay in `slice-cache/`,
and a later run resumes
them PROVIDED THE BUILD HAS NOT MOVED.

### The freeze, which is the one thing that can be broken by accident

DO NOT CHANGE ANYTHING THE PIPELINE RUNS,
and do not rebuild after doing so,
until the pass has
stopped.
`assertResumableGeneration` throws `GenerationDriftError` when a pool holds entries settled
under another build,
so a rebuild costs this run's resumability.
Reading,
reporting,
documentation
and anything under `corpus-run/` that only reads are all safe;
`slice-cost-report.ts` was built under
this freeze as the model.

Frozen and waiting:
`#91` (half-weight,
settled and located at `tallyResolutionChecks`),
`#118` (the
stream diagnostics),
and anything in `#90`,
`#100` or `#106` touching slicing.

### When the pass stops, in order

1.  `#91` half-weight self-certification.
    Settled,
    authorized,
    located.
2.  `#118`:
    keep the partial text on abort,
    emit the `drainBody` progress sample on the abort path
    too,
    and put the model id on that line.
    Small,
    and it makes `#68`'s latency axis answerable.
3.  Re-read the slice cost telemetry,
    which by then includes translate-lane slices:
    `mise run //package/module/translation-repair:slice-cost-report -- ~/temp/agent/corpus-pass-20260817.log` (moved on
    2026-10-18 to `//package/translation-repair/slice-cache:slice-cost-report`)
4.  Re-run the settled rendering audit over the six artifacts.
    The reader takes the flat layout,
    so:

    ```sh
    mise run //package/module/translation-repair:rendering-audit-settled \
      -- --archive ~/translation-repair-runs-20260817/artifacts
    ```

    Moved on 2026-10-18 from `//package/module/translation-repair:rendering-audit-settled`
    to `//package/translation-repair/probe:rendering-audit-settled`.

    Verified working against the real artifacts at `--cap 0` on 2026-08-17.
5.  `#60`'s redraw,
    now unblocked at the lowered bar of six.
6.  `#108`'s replacement rate ON displacement-flagged slices against off them,
    which is what `#107`'s
    remaining decision waits on.

### Claims of mine that were retracted today, so they are not re-quoted

-   "GLM models are slow."
    NO. 262 successful GLM answers against 24 abandons in the same window,
    and the abandons are stage-shaped:
    15 `select`,
    6 `panel`,
    4 `critic`,
    0 `checker`,
    evenly
    spread.
    Selection carries the largest prompts.
    A tail on large prompts,
    and still a hypothesis.
-   "1349 streams,
    mean firstByte 1822ms." NO. The `drainBody` progress line is emitted after the
    try/catch,
    so aborted streams log nothing and that figure covers survivors only.
    Survivorship
    bias over exactly the population it was describing.
-   "An abandoned call leaves nothing."
    NO. Streaming is always on and `drainBody` accumulates the
    partial text in `parts`;
    the catch rethrows and drops it.
    The evidence exists at the cut and is
    discarded,
    which is `#118` item 1.
-   "#96 storing the target text doubles artifact size."
    NO,
    it is 0.6 to 0.9 percent.
    An artifact
    is judge evidence,
    not text.
    `doc/planning/artifact-archive-text.md`.
