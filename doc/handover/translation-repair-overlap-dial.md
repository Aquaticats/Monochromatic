# Handover: the overlap dial for the corpus pass (`#261`)

Written 2026-08-26 for whoever picks this up next,
including an agent that has read no transcript.
Everything needed to understand the legacy overlap work is here or in the files it names.

## Current status, 2026-09-01

The overlap implementation remains part of the legacy pipeline,
but its pending measurements are not an authorized next action.
The finite replacement redesign stopped after Candidate M failed.
Read
[`translation-repair-redesign-failure-2026-09-01.md`](../audit/translation-repair-redesign-failure-2026-09-01.md)
and the current section of [`translation-repair.md`](translation-repair.md) first.

No corpus pass is running.
Do not launch overlap measurements,
resume an old run,
or treat overlap completion as production readiness until the owner explicitly authorizes a new phase.

## What this task is

The corpus pass runs its slices one at a time.
Every per-slice loop in the pipeline carries the same comment,
that "aggregate concurrency beyond one stream per model collapses throughput on this plan".
That premise was measured on 2026-08-26 and is wrong:
`doc/decision/translation-repair-calibration-overlap.md` records five matched arms
where four slices in flight took the same total call time in 0.23 of the wall clock
that one slice at a time took in 0.41,
which is six run-to-run bands apart.

The owner decided the editor calibration defaults to four slices in flight under a 300000 ms straggler window.
The corpus pass was deliberately left alone until it is measured there,
which is this task.
The dial exists already:
`readOverlap({ fallback, })` in `package/module/translation-repair/src/corpus-run/slice-overlap.ts` (moved on 2026-10-18
to `package/translation-repair/slice-cache/src/corpus-run/slice-overlap.ts`)
reads `TRANSLATION_REPAIR_SLICE_OVERLAP` and refuses anything that is not a whole number of at least one.
The pass's fallback stays `1` until the measurement says otherwise.

## What has landed

Both commits are on branch `translation-repair-rebased` in worktree `/var/home/user/worktrees/translation-repair`,
and auto-push is on.

-   `5ad32cc5d`:
    `src/overlapped-map.ts` plus `src/overlapped-map.unit.test.ts`,
    exported from `src/stage-barrel.ts`.
    `mapOverlapped({ items, overlap, oneItem, })` opens at most `overlap` lanes,
    each taking the next unclaimed item and running it to the end before taking another.
    Items therefore START in item order,
    results come back in item order however they finished,
    a failure stops further items from starting while the ones in flight finish,
    and the error thrown is the LOWEST position's,
    which is the one a sequential loop would have thrown.
    `overlap: 1` reproduces the loop it replaces.
    Not `p-limit`,
    whose queue keeps draining after a failure;
    `p-limit` is still what `editor-calibrate.ts` uses,
    which is fine because that command has no entry to abandon.
    Three guards are GFP-proven
    (script kept at `~/temp/agent/gfp-overlapped-map.py`):
    the failure short circuit,
    the lowest-position throw,
    and the lane count.

-   `748b54841`:
    `src/twin-memo.ts` plus its suite,
    and `src/translate-slice-settle.ts` plus `src/translate-slice-buy.ts`.
-   `e7e017a36` through `9f1c843d6`:
    `translateDocument` now calls `mapOverlapped` over `settleTranslateSlice`,
    defaults `overlap` to `1`,
    aggregates records,
    unfilled passages and findings in slice order,
    counts only records resumed from disk,
    and runs both twin cases at overlap `1` and `2`.
    A successful-call instrument proves `overlap: 2` puts two slices in flight.
-   `1c4a8b78e`:
    the first whole-suite run exposed three expectations left stale by earlier committed changes.
    The message inventory now names `OverlapRefusedError`,
    the one-provider router case uses the still Synthetic-only Nemotron seat rather than the newly dual-routed Qwen seat,
    and spend-cost expectations follow the 2026-08-26 Hyper rates.
-   `bb6883548` through `84ba6eecb`:
    `repairPreparedDocument` now calls `mapOverlapped` over `settleRepairSlice`,
    defaults `overlap` to `1`,
    and exposes the optional argument through standalone `repairTranslation` without reading the environment there.
    It aggregates accuracy outcomes and cache-refusal findings in slice order,
    shares one eligibility-aware twin memo across the document,
    and normalizes torn-down in-flight calls to the caller's abort reason.
    Its twin tests run at overlap `1` and `2`,
    and a successful-critic instrument proves two slices are active at overlap `2`.
-   `e65693c20` through `755b22612`:
    `runRefinePhase` now calls `mapOverlapped` over `settleRefinePhaseSlice`,
    defaults `overlap` to `1`,
    and receives the repair document's overlap through `refineSettledSlices`.
    It returns outcomes and findings in input order,
    reports `askedRewriters` when any fresh slice asked,
    and persists only quorum-complete settlements under a live signal.
    Direct tests pin overlap activity,
    order,
    any-asked aggregation,
    silent-stage cache refusal,
    abort-safe persistence and disabled-lane validation.
    A repair-driver test separately pins the overlap threading into this phase.
-   `c4b2d60d1` through `c52d4d6ab`:
    `contestDocumentLanes` now filters eligible comparison rows before calling `mapOverlapped`,
    defaults `overlap` to `1`,
    and returns artifact rows in comparison order.
    It resumes without writing back,
    persists only quorum-complete outcomes under a live signal,
    and shares cache-eligible identical questions through the promise twin memo.
    Twin tests at overlap `1` and `2` prove a settled question is asked once and an unheard question is asked again.
    A mixed resume-and-buy case pins ordered aggregation and one fresh persistence at overlap `2`.
-   `e8f6e78e9` through `31d61e495`:
    `consolidateDocument` now pairs comparison rows with their contest records before calling `mapOverlapped`,
    defaults `overlap` to `1`,
    and returns artifact rows in comparison order.
    Fresh slate production moved to `buyConsolidationSlice`.
    Stable settlements persist only under a live signal,
    and identical position-free questions share one cache-eligible purchase through the promise twin memo.
    Tests pin overlap activity and order,
    settled and unsettled twins at overlap `1` and `2`,
    a mixed resume-and-buy run,
    and abort-safe persistence.
-   `973bf4016` through `0ebcbf53b`:
    `settleEntry` now calls `readPassOverlap` once before entry work,
    logs `OVERLAP <entry> value=<n> source=<fallback-or-variable>`,
    and passes that one value through both document lanes,
    contest and consolidation.
    Corpus fallback remains `1`.
    Environment input now refuses fractional and non-canonical numeric spellings instead of truncating or canonicalizing them.
    Runtime instruments prove overlap `2` reaches repair,
    refinement,
    translation,
    contest and consolidation from the entry boundary.
    Separate document-lane instruments pin both lane handoffs.

## The twin memo, which is the part that is easy to get wrong

Two slices carrying identical source,
incumbent and governance ask ONE question,
and since the key stopped naming the slice index they hash the same.
Each sequential driver memoized what it PERSISTED so the second twin reused it,
which is what makes a cold run settle what a warm run resumes.
The memo deliberately held persisted records only:
a record the driver refused to store (nobody heard,
an unfilled passage,
a stage under quorum)
was not memoized,
so the twin asked again,
exactly as a warm run would.
`#238` is the defect that rule came from,
and both `translate-document.unit.test.ts` and `repair-translation.unit.test.ts`
carry an "ASKS AGAIN for a twin of a slice no translator answered" case that pins it.

Under overlap the twin may arrive while the first is still buying,
so the memo holds a PROMISE of what the buy left behind:

-   `TwinStored<Settled>` is `{ kind: 'stored', record, }` or `{ kind: 'nothing', }`.
    Tagged rather than nullish,
    which the repository requires
    (`no-restricted-syntax(no-nullish-union)` refuses `Settled | undefined`).
-   A twin waiting on `nothing` buys its own,
    which is the sequential behaviour.
-   The buyer withdraws its entry BEFORE resolving `nothing`,
    so a third twin waking up finds either the second twin's entry to wait on or no entry at all.
-   A failed buy withdraws its entry,
    warns,
    and resolves `nothing`:
    the buyer throws on its own path,
    and the waiter goes and asks for itself,
    or throws under the same abort when it tries.
-   The lookup and the registration are SYNCHRONOUS with respect to each other.
    Two twins that both looked,
    both found nothing,
    and both yielded before registering would both buy.
    The buy is started and the entry is set in one synchronous run,
    with no await between.

One path changed deliberately with the move,
and it is written into `translate-slice-settle.ts`'s module note:
a slice whose CACHED record was refused used to bypass the memo,
so two twins with the same refused record each bought and each persisted under one key.
Both twins now reach the memo after refusing the cache,
so the second reuses what the first persisted,
which is what the warm run does.

Contest now follows the same rule even though its former loop did not.
Its position-free key names exactly what the roster sees,
and `LaneContestOutcome` carries no slice index requiring restamping.
Without the memo,
two identical contests could buy contradictory ballots and race to overwrite one cache file at higher overlap.
The new memo makes one cold-run answer match the one answer a warm run resumes.
The pre-persistence abort check is also new:
a gather that retained quorum before caller abort can no longer make an abandoned contest look complete or become warm-run evidence.

Consolidation now follows that same contest rule even though its former loop did not.
Its position-free key names everything the producer,
judges and gate see,
and `ConsolidationSettlement` also carries no slice index requiring restamping.
One settled twin is shared and one unsettled twin is re-bought.
The new pre-persistence abort check keeps a stable settlement returned after caller abort out of both disk cache and twin memo.
`pass-entry.ts` still describes every settled slice as persisted before the next begins;
update that comment when the dial is threaded because overlap and the new abort check both make it stale.

Refinement is the deliberate exception.
Its former loop had no in-run memo,
and the disk cache's `resumed` map is a snapshot that persistence does not mutate.
Two identical uncached refinement questions therefore both buy at overlap one and may buy concurrently at higher overlap.
The concurrency test uses such a pair and pins two simultaneous refiner calls.
When both persist one key,
overlap one deterministically leaves the later positional write while higher overlap leaves the later completion.
No such pair is known in the measured corpus,
but corpus measurement must check this before raising the fallback or refinement needs its own memo and record-restamping design.

## Bounded failure cost

At overlap greater than one,
a lower-position failure stops new slices from starting but lets already active slices finish.
Up to `overlap - 1` later slices may therefore spend after a sequential run would have stopped.
Independent later slices may also persist answers that a sequential run would never have reached.
This is a deliberate consequence of bounded parallelism,
not semantic equivalence with overlap one after the failure point.
Measurement arms and production readings must report failures and spend beside wall-clock results.

## What is left to do

Order matters only in that each driver should be green and committed before the next one starts,
so the tree is shippable at any moment (see "Do not land a driver into a live pass launch").

1.  `translateDocument`:
    DONE in `e7e017a36` through `9f1c843d6`.
    Build,
    focused suites,
    `lint:oxlint` and `lint:types` pass.
    Removing the overlap argument made the driver suite fail,
    and replacing the shared twin memo with one memo per slice made it fail too;
    both mutations were restored and rebuilt.
    The post-handover whole suite passed with 832 PASS,
    0 FAIL and exit 0
    after its first run found and fixed the three stale expectations above.
2.  `repairPreparedDocument`:
    DONE in `bb6883548` through `84ba6eecb`.
    Build,
    focused suites,
    `lint:oxlint` and `lint:types` pass.
    Forcing the driver to overlap one failed the successful-critic concurrency case.
    Replacing the shared twin memo with one map per slice failed the overlap-two ASKS ONCE case.
    Returning the torn-down exchange instead of `signal.reason` failed the abort identity case.
    All mutations were restored and rebuilt.
    Logs are `~/temp/agent/gfp-repair-overlap-ignored.log`,
    `~/temp/agent/gfp-repair-twin-memo.log`,
    and `~/temp/agent/gfp-repair-abort-normalization.log`,
    with matching restored logs beside them.
    Whole-package `buildAndTest` passed with 832 PASS,
    0 FAIL and exit 0;
    log:
    `~/temp/agent/buildAndTest-repair-overlap-20260827T062031Z.log`.
3.  `runRefinePhase`:
    DONE in `e65693c20` through `755b22612`.
    Build,
    focused suites,
    `lint:oxlint` and `lint:types` pass.
    Five direct mutations failed the intended guards:
    forcing phase overlap one,
    replacing any-asked with every-asked,
    caching a silent stage,
    removing abort-safe persistence,
    and removing disabled-lane overlap validation.
    Forcing the repair-to-refine handoff to overlap one failed its integration guard too.
    Logs begin `~/temp/agent/gfp-refine-` and name each mutation;
    matching restored logs sit beside them.
    Whole-package `buildAndTest` passed with 832 PASS,
    0 FAIL and exit 0;
    log:
    `~/temp/agent/buildAndTest-refine-overlap-20260827T064814Z.log`.
4.  `contestDocumentLanes`:
    DONE in `c4b2d60d1` through `c52d4d6ab`.
    Build,
    focused suites,
    `lint:oxlint` and `lint:types` pass.
    Forcing overlap one failed the successful-call concurrency guard.
    Removing the shared twin memo failed the overlap-two ASKS ONCE case.
    Caching an unheard roster and removing abort-safe persistence each failed its own guard.
    Every mutation was restored and rebuilt;
    logs begin `~/temp/agent/gfp-contest-` and name each mutation.
    Whole-package `buildAndTest` passed with 832 PASS,
    0 FAIL and exit 0;
    log:
    `~/temp/agent/buildAndTest-contest-overlap-20260827T072222Z.log`.
5.  `consolidateDocument`:
    DONE in `e8f6e78e9` through `31d61e495`.
    Build,
    focused suites,
    `lint:oxlint` and `lint:types` pass.
    Forcing overlap one and replacing the shared twin memo each failed the corresponding concurrency or ASKS ONCE guard.
    Caching an unsettled gate and removing abort-safe persistence each failed its own guard.
    Every mutation was restored and rebuilt;
    logs begin `~/temp/agent/gfp-consolidate-` and name each mutation.
    Whole-package `buildAndTest` passed with 832 PASS,
    0 FAIL and exit 0;
    log:
    `~/temp/agent/buildAndTest-consolidate-overlap-20260827T075438Z.log`.
    A transient `commit_refs` auto-push rejection on `78215439d` recovered on a later commit;
    measured upstream divergence is zero in both directions.
6.  Corpus-pass overlap dial:
    DONE in `973bf4016` through `0ebcbf53b`.
    `readPassOverlap` reads and logs one value per entry with fallback `1`.
    `runDocumentLanes` hands it to repair,
    refinement and translation;
    the entry hands it to contest and consolidation.
    Build,
    focused suites,
    `lint:oxlint` and `lint:types` pass.
    Forcing either document lane or any of the entry's three driver handoffs to overlap one failed its runtime guard.
    Every mutation was restored and rebuilt;
    logs begin `~/temp/agent/gfp-lanes-` or `~/temp/agent/gfp-pass-` and name each handoff.
    Whole-package `buildAndTest` passed with 832 PASS,
    0 FAIL and exit 0;
    log:
    `~/temp/agent/buildAndTest-overlap-dial-20260827T083706Z.log`.
7.  Measure,
    then record the result in
    `doc/decision/translation-repair-calibration-overlap.md`
    and in the open-decisions register.

## What is NOT in scope, and why

-   Lane concurrency in `document-lanes.ts`.
    Repair runs,
    then translate.
    Running both at once doubles the roster's concurrent load,
    which is the variable arm A2 showed dominates single-run wall clock (37% band from provider speed alone),
    so landing it beside the slice dial would make the pass measurement unattributable.
    It is a separate task if anyone wants it.
-   `readDocumentPictures` (`src/document-readings.ts`,
    loop at line 163).
    Check whether picture reading is even on the measured path per entry before touching it;
    on the ten-entry pass being read now it barely appears.
    If its cost is trivial,
    leave it sequential and say so in the decision note
    rather than widening the diff.
-   `producer-calibrate.ts`.
    It gets the same dial as follow-up work so both calibrations run under one default;
    until then it runs one slice at a time.

## How to measure it

Arms must be matched the way the calibration arms were,
because a single run resolves nothing:

-   Same entries,
    same build,
    back to back,
    nothing else running.
-   A THROWAWAY runs directory per arm,
    including separate artifact,
    publish and slice-cache roots.
    A shared cache would resume arm one's slices into arm two and measure the cache,
    not the dial.
    Artifacts do not carry overlap metadata,
    so each arm's separate root and its
    `OVERLAP <entry> value=<n> source=<...>` log lines are also the attribution record.
-   Read the result as wall clock over the sum of stream time,
    not as wall clock alone.
    The unnormalized numbers move 37% run to run on provider speed.
    `~/temp/agent/reading-instruments/compare-arms.py` computes exactly this from two run logs;
    its `ARMS` dictionary names the logs to compare.
-   Report cut voices beside the timing.
    Arm D (overlap 4,
    300000 ms window) heard 318 of 320 with 2 cut,
    the best of the five arms,
    so overlap is not obviously paid for in lost voices.

### Measurement status, 2026-08-27

The live `keyword233` smoke pair passed on the same final digest,
with separate roots and explicit attributed settings.
Overlap `1` took 38.50 minutes over 1.68 hours of calls,
normalized `0.382`,
with 9 voices unheard.
Overlap `4` took 31.12 minutes over 1.78 hours of calls,
normalized `0.291`,
with 8 voices unheard.
Both published pages verified.
The overlap arm reduced wall time by 19.2 percent and normalized wall time by 23.9 percent,
but two slices are only the positive control,
not enough to change the fallback.
The matched decision pairs and output reading continue under
`doc/planning/translation-repair-corpus-overlap-measurement.md`.

The matched same-digest `Toka_ls` pair then settled 15 slices in both arms.
Overlap `1` took 313.24 minutes over 12.44 call-hours,
normalized `0.420`,
with 61 voices unheard and peak 10 calls in flight.
Overlap `4` took 104.37 minutes over 11.82 call-hours,
normalized `0.147`,
with same 61 voices unheard and peak 37 calls in flight.
That is 66.7 percent less wall time and 64.9 percent less normalized wall time at overlap `4`.
Call sum fell 5.0 percent,
but metered Hyper spend rose 104.2 percent because concurrency overflowed subscription seats to Hyper.
Both pages remain timing evidence only:
each artifact recorded same omitted linked death paragraph as `gap-remains`.
Current ten-model coverage control then passed with absence votes only on targeted damage,
never equal-size decoys.
Fixed-build overlap-4 `Toka_ls` at 180-second grace restored complete page in 114.72 minutes,
normalized `0.132`.
Strict read still found one inherited major person error:
repair corrected it,
translate retained it,
and contest heard 8 of 10 then tied 4 to 4.
Same-digest overlap-4 300-second-grace arm settled in 127.43 minutes,
normalized `0.140`,
with 35 voices unheard against 72.
Its page is publishable as-is:
first-person major gone,
source destination and protected memorial facts retained,
no invisible bytes.
One pair does not decide grace default;
remaining overlap decision arms continue at built-in grace.
`ArtsEpiphany` null pair then settled one unchanged slice in both arms,
zero unheard voices and byte-identical pages.
Normalized `0.268` at overlap `1` and `0.325` at overlap `4`
measure provider variation because one slice cannot overlap.
Record:
`doc/audit/translation-repair-output-reading-20260826.md`.

## Do not land a driver into a live pass launch

The `corpus-pass` mise task BUILDS the tree it then runs.
A pass already running is unaffected by a rebuild
(there are no dynamic imports;
this was verified,
not assumed),
but the NEXT launch ships whatever the tree holds.
So either finish a driver completely,
suites and GFP included,
and commit it,
or hold the work and launch from the committed tree.
The reading of the pass output gates the readiness signal;
this task does not.

Since 2026-09-06 the owner's rule closes the other direction as well:
a pass running when source changes is killed by pid and relaunched on the new build into a fresh runs dir,
never left to finish on the superseded one
(package README,
"A source change while a pass is in flight means kill and relaunch").
The pass overlap fallback moved to four the same day
(`doc/decision/translation-repair-pass-overlap.md`),
on the four matched pairs recorded under the 2026-08-27 measurement status and in the package README.

## The verification chain for each landing

Run from the repository root unless stated:

```sh
mise run //package/module/translation-repair:build
mise run //package/module/translation-repair:lint:oxlint   # must print "Found 0 warnings and 0 errors"
mise run //package/module/translation-repair:lint:types
```

Moved on 2026-10-18 from `//package/module/translation-repair:build`
to `//package/translation-repair/<package>:build`.
Moved on 2026-10-18 from `//package/module/translation-repair:lint:oxlint`
to `//package/translation-repair/<package>:lint:oxlint`.
Moved on 2026-10-18 from `//package/module/translation-repair:lint:types`
to `//package/translation-repair/<package>:lint:types`.

Then,
from `package/module/translation-repair` (moved on 2026-10-18 to the packages under `package/translation-repair`),
the suites that cover the driver,
for example:

```sh
node src/translate-document.unit.test.ts
node src/overlapped-map.unit.test.ts
node src/twin-memo.unit.test.ts
```

The runner prints `] PASS ` and `] FAIL ` per case;
count with those exact strings,
never a bare `PASS`,
and let the exit code decide.

Both twin cases must be run at overlap 1 AND at a higher overlap,
since the higher one is the case the refactor can silently break.
The two cases are named
"ASKS ONCE for two slices carrying identical text"
and
"ASKS AGAIN for a twin of a slice no translator answered"
in `src/translate-document.unit.test.ts`,
with a matching pair in `src/repair-translation.unit.test.ts`.

Whole suite,
in the background,
nothing else touching `dist`:

```sh
LOG="${HOME}/temp/agent/buildAndTest-$(date -u +%Y%m%dT%H%M%SZ).log"
mise run //package/module/translation-repair:buildAndTest > "$LOG" 2>&1
echo "exit=$? PASS=$(grep -c '\] PASS ' "$LOG") FAIL=$(grep -c '\] FAIL ' "$LOG")"
```

Moved on 2026-10-18 from `//package/module/translation-repair:buildAndTest`
to `//package/translation-repair/<package>:buildAndTest`.

The last whole-suite run before the repair driver was 832 PASS,
0 FAIL,
exit 0.

Guard-failure proof,
per the repository rule that a guard proves nothing until it is shown to fail:
remove the guard,
rebuild,
run the suite,
restore,
rebuild.
`~/temp/agent/gfp-overlapped-map.py` is a worked example of the shape,
and `~/temp/agent/reading-instruments/gfp-three-landings.py` is the larger one from the previous landings.
