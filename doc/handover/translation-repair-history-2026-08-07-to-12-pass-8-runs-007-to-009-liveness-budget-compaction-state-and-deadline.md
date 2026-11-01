# Translation repair history 2026-08-07 to 2026-08-12: how to read corpus-pass progress, why runs 007 to 009 of pass 8 ended as they did, the state at the 2026-08-07 compaction and the per-entry deadline

How to tell a live corpus pass from a stalled one,
why run 007 ended on its budget,
the state at the 2026-08-07 compaction,
how the per-entry deadline hit the large band,
the evidence that a deadline costs a restart and never an entry,
and the outside kill of run 009 that lost nothing.

Part of [Translation repair history: 2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## How to read corpus-pass progress without inventing a stall

Reading the pass log for liveness has one trap,
and it cost this session several
probes before it resolved.

CHUNK INDICES ARE NOT MONOTONIC ACROSS THE LOG.
A tail of `chunk N: repaired`
lines reads `1, 3, 1, 2, 3, 4, 5, 6, 3, 5, 9, 11, 12, 13, 1, 0, 11`.
That is not
corruption and not a restart loop.
Indices are per entry,
and the slice cache
resumes a partial entry by recomputing only its uncached chunks,
so an entry that
already has chunks 0 through 8 cached emits 9,
11,
12,
13 and nothing else.

THE FALSE ALARM THIS PRODUCES:
comparing the newest artifact mtime against the
newest `chunk 1: repaired` line suggests the run sat idle for hours.
It did not.
The interval was full of `selectBestCandidate` ballots,
which are per envelope
inside the editor stage and carry no chunk or stage prefix,
so a grep filtered to
`stage:` or `chunk ` shows an empty window over an hour that logged 120 lines.

WHAT ACTUALLY ESTABLISHES LIVENESS,
in increasing order of cost:

-   Per-hour line counts including `drainBody`.
    A live pass logs a few hundred
    per hour.
    Zero for an hour is the real stall signal.
-   Chunk completion timestamps.
    Steady spacing means healthy;
    this run held ten
    to fifteen minutes per chunk across forty chunks.
-   Stage lines for the entry in flight,
    which show `critic` through `checker`
    advancing rather than one stage repeating.

FIRST-BYTE LATENCY IS NOT A HEALTH SIGNAL AT THIS PROVIDER.
Consecutive calls
reading 58s,
126s,
163s,
193s,
222s look like a provider degrading under a
climbing backlog.
Measured across the whole run the mean is 54.7s over 1488
calls with a maximum of 336s,
so that climb is ordinary variance sampled at a
window boundary.
Do not infer throttling from a handful of adjacent lines;
take
the distribution over the run.

AN ENTRY TAKING MUCH LONGER THAN ITS PREDECESSORS IS USUALLY CHUNK COUNT,
NOT A
HANG.
Entries here range from a handful of chunks to more than thirteen,
and
settle time tracks that count nearly linearly.
Check how many chunks the entry
has emitted before concluding anything is wrong with it.

## Run 007 ended on its budget, not on its target

`pass8-run-007` exited 0,
which reads like success and is not one.
The final
lines are what matter:

```text
SOFT budget reached after 52381272ms; not starting new entries
DONE processed=9 of pending=92; artifacts=9/92 elapsed=52381272ms
```

IT STOPPED BECAUSE TIME RAN OUT,
having settled 9 entries of 92 pending in
14.5 hours.
Exit 0 means the driver shut down cleanly at its own soft budget,
so never read a zero exit from `corpus-pass` as "the corpus was processed".
Read the `DONE` line.

TWO OF ELEVEN ATTEMPTED ENTRIES HIT THE PER-ENTRY DEADLINE.
`Dethelly` and
`Futajuhuacha` each burned the full 10800000ms (3 hours) and produced no
artifact.

CORRECTION TO AN EARLIER READING OF THIS,
recorded because the wrong version was
committed first:
that is NOT 41 percent of the budget wasted.
Each of those
entries banked 15 completed chunks into the slice cache before the deadline
fired,
and run 008 resumed both from there.
No artifact is not the same as no
progress,
and the slice cache is exactly the mechanism that makes the difference.
Judge a timed-out entry by its cached chunk count,
never by the missing
artifact.

WHAT THE 9 SETTLED ENTRIES SUPPORT.
`draw-sample` produced a real round-three
sheet from them:
small 3 entries with 22 accepted,
medium 4 with 108,
large 2
with 72,
pool 202,
drawn 50,
`unrecordedRepairs=0`.
All three bands are
represented,
so the sheet is gradeable.
Entry coverage is still far short of the
~10/10/10 target,
so treat the resulting precision as provisional until more
entries settle.

RUN 008 IS ACCUMULATING ON TOP OF THIS.
Settled entries are not recomputed and
the slice cache preserves partial work,
so the two timed-out entries resume from
their cached chunks rather than restarting from zero.

## State at the 2026-08-07 compaction

WHAT IS RUNNING:
`pass8-run-008`,
started 2026-08-07,
logging to
`pass8-run-008.log` in the runs directory.
It resumed `Dethelly` from cached
chunk 14 and was past chunk 18 an hour in.
Nine artifacts settled,
all from run
007.
The run notifies on exit;
do not poll it.

WHAT TO DO WHEN IT EXITS,
in order:

-   Read the `DONE` line,
    never the exit code.
    Exit 0 means the driver stopped
    cleanly at its soft budget.
-   Re-run `score-probe`.
    Costs no quota,
    reads local artifacts,
    refreshes the
    figures the gating decision doc quotes.
-   Check whether the large band gained entries.
    If it did,
    redraw with
    `draw-sample -- --final` for the gate sheet;
    if not,
    restart the pass.
-   Check whether `Dethelly` or `Futajuhuacha` reached `TALLY status=repaired`.
    That answers the open half of the deadline task.

THE ONE BLOCKER EVERYTHING CHAINS OFF:
the final gate sheet needs more
large-band entries.
Precision re-measure waits on that,
pre-grading waits on the
gate sheet,
the probe false-positive comparison waits behind pre-grading,
and
the recall re-measure,
naturalness probe,
and judge crosscheck want throughput
this run holds while the provider is the measured bottleneck (54.7s mean first
byte over 1488 calls,
336s max).

DECIDED THIS SESSION:
the introduced-defect probe stays in shadow mode,
recorded
in `doc/decision/introduced-defect-probe-gating.md`.
Nothing else is waiting on
a decision.

NOT DONE ON PURPOSE,
so nobody re-opens these as oversights:
no pre-grading (the
preliminary sheet is not the gate sheet,
and items 1 through 5 were sighted);
no
change to the per-entry deadline or soft budget (resume is measured to work,
so
the deadline costs wall time rather than entries,
and both limits are the user's
design call);
no concurrent quota-bound measurement while the pass runs.

## The per-entry deadline is starving the large band specifically

Measured read-only off run 007 at zero quota,
so this needs no rerun.

BAND THE ATTEMPTED ENTRIES BY `page.md` SOURCE BYTES,
using the pipeline's own
cuts from `band-order.ts` (`SMALL_PAGE_BYTES` 1843,
`MEDIUM_PAGE_BYTES` 3686).
That reproduces `draw-sample`'s split exactly (small 3,
medium 4,
large 2),
which is what makes the proxy trustworthy rather than a second,
different
measurement:

-   large:
    `Dethelly` 6171 TIMEOUT,
    `Arita` 5951 settled,
    `Futajuhuacha` 5448
    TIMEOUT,
    `Chinatsu_Suzuki` 5353 settled.
    Two of four lost.
-   medium:
    `Considerate_cat` 3513,
    `AmbeR_the_anpa` 2122,
    `Anilovr` 1985,
    `Everythings99` 1859.
    Four of four settled.
-   small:
    `Aniloviraw` 1481,
    `Acheron` 938,
    `AkiraComplex` 743.
    Three of three
    settled.

FIFTY PERCENT LOSS IN THE LARGE BAND,
ZERO EVERYWHERE ELSE.
The deadline is not
trimming entries at random.

SIZE CORRELATES BUT DOES NOT DETERMINE,
and this is the part worth not
forgetting.
`Dethelly` is the largest and timed out,
but `Futajuhuacha` at 5448
timed out while the LARGER `Arita` at 5951 settled.
Something beyond size varies,
most plausibly provider latency across the window.
Do not model the deadline as
a pure size threshold and do not predict which large entries will fail.

WHY IT BLOCKS THE MILESTONE RATHER THAN JUST COSTING TIME.
`draw-sample` refuses
a final gate sheet until the large band fills,
and the preliminary draw pulls 16
large-band slots from 2 entries.
The deadline is starving exactly the band the
gate depends on,
so the budget task and the precision re-measure are one problem,
not two.

THE DESIGN ALREADY EXPECTS THIS,
AND RESUME IS NOW MEASURED TO WORK.
`band-order.ts` orders the large band first,
commenting that a large entry may
need a second run to settle so starting it earlier lets it resume sooner.

THE EVIDENCE,
observed live:
after run 007 died on both entries,
the slice cache
held `Dethelly` and `Futajuhuacha` only,
at chunks 0 through 14 each,
contiguous
from zero.
Settled entries discard their caches,
which is why nothing else is
there.
Run 008's FIRST chunk completion is `chunk 14`,
roughly eight minutes
after it started,
not `chunk 0`.

WHAT THAT SETTLES:
the deadline costs wall time,
not entries,
as long as the pass
is restarted.
Raising it is therefore not urgent and probably not the right knob.
The remaining cost is only the work in flight when the deadline fires,
which is
at most one chunk,
plus the per-restart overhead of reaching the entry again.

WHAT IT DOES NOT SETTLE:
whether these entries eventually settle at all,
or keep
consuming a deadline per run without reaching their last chunk.
That needs run
008 to carry one of them to a `TALLY ... status=repaired`.

## The deadline costs wall time and a restart, never an entry (#61 answered)

Both entries that hit the 3-hour per-entry deadline in run 007 settled on
resume in run 008:

```text
TALLY Dethelly status=repaired issues=282 accepted=198 resolved=198 ms=6900394
TALLY Futajuhuacha status=repaired issues=229 accepted=167 resolved=165 ms=3685583
```

The slice cache is what makes this true.
A timed-out entry banks every completed chunk,
 and the next pass resumes from the highest cached index rather than from zero.
Dethelly spent 3 hours in run 007 reaching chunk 14,
 then 1.9 hours in run 008 finishing from there:
 about 4.9 hours of real work split across two passes,
not 3 hours thrown away and 4.9 spent again.

Run 008 produced a third casualty,
 `Huasheng` at `ms=10800002 aborted=true`,
which confirms the pattern rather than contradicting it.
It will settle the same way on the next restart.

So the answer to #61 is that the per-entry deadline is NOT the knob.
Raising it would let one pathological entry monopolize the soft budget,
 and the thing it currently costs,
 a restart,
 is already automated by the cache.
What actually bounds throughput is the 12-hour soft budget and provider latency,
 not the 3-hour deadline.
Close #61 on this evidence rather than tuning the deadline.

## Run 009 was killed from outside, and nothing was lost

Four background tasks stopped at the same moment:
 run 009 and three stale `pi` calls left over from an earlier session.
The log ends `ERROR sh exited with non-zero status: no exit status`,
 which is a signal rather than an exit,
and the simultaneity points at a harness-level cleanup of background tasks
 rather than anything the pass did.

Nothing was lost,
 and this is worth checking rather than assuming next time it happens:

-   All four round-three gate files are byte-for-byte intact
     (sheet 22087,
    repair sheet 52756,
    manifest 8213,
    pre-grades 9662).
    The draw is already written,
    so no pass can affect it.
-   All 21 artifacts pass the accepted-count reconcile,
    which is now STRICT,
     so a half-written artifact from the kill would have thrown rather than
     joined the pool silently.
    None did.
-   Run 009 settled 3 entries before dying:
     `Huasheng`,
    `LCG_Akiball`,
    `CuspariaKLSY`.

`Huasheng` is the notable one.
It was run 008's deadline casualty (`ms=10800002 aborted=true`),
 and it came back on resume with 145 accepted issues,
which is a third independent confirmation of the #61 finding that the per-entry
 deadline costs a restart rather than an entry.

Band composition is now 6 small,
7 medium,
7 large contributing,
 over a pool of 939.
That is BETTER than the 5/7/5 the gate sheet was drawn at,
 and it changes nothing about the gate:
 the draw is one-shot and already spent,
so these entries serve round four,
recall (#51),
and the naturalness probe
 (#58).
