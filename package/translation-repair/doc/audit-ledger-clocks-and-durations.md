# Audit ledger: clocks and durations

Durations measured on the system clock,
and test bounds on the real clock a loaded machine could outrun,
among them the held review seat that landed before quorum under load (B813).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B78: durations, holds, paces and budgets measured on the system clock

Found in T8's seventeenth batch,
beside B71,
and left open there.
The system clock is the time of day,
which a person or a time daemon can set,
and the package took its durations,
deadlines,
holds,
paces and budgets as differences of `Date.now()` readings.
Set back an hour mid-call,
a duration read negative,
a hold or a pace waited an hour longer than asked,
and a budget lasted an hour more;
set forward,
holds ended early and budgets ran out at once.
Found by reading the code;
no stored log was searched for a run during which the system clock was set.

The scan committed red (`a3f1f2c55`) read 36 reads of the wall clock as a number:
23 in production,
in `corpus-pass.ts`,
`pass-decline.ts`,
`pass-entry.ts`,
`sentinel-probe.ts`,
`provider-budget.ts`,
`provider-router.ts`,
`repair-benchmark.ts`,
`request-pace.ts`,
`slice-cost-log.ts`,
`stage-round.ts` and `stream-idle-guard.ts`,
and 13 in tests,
in `budget-hold-wait`,
`pass-decline`,
`pass-outside-reads`,
`stage-quorum`,
`stage-round` and `transient-retry`.

Each red case (`29608a23b`) failed on its assertion against the unfixed code,
with `wall-clock-stub.test-fixture.ts` holding `Date.now` at a reading the case moved:

- the pacer on its default clock,
  set back an hour after a take,
  asked the next take to wait 3,600,000 ms more than the window;
- a take whose sleep ended a millisecond before its reserved start returned there
  (1,059,999 for 1,060,000):
  the pacer slept once and never read the clock again;
- the idle guard,
  set back an hour before the first byte,
  reported that byte at -3,600,000 ms;
- the slice cost meter,
  set back mid-slice,
  wrote `ms=-3600000`,
  a line `slice-cost-read` drops;
- the gather round,
  set forward two hours as it started,
  logged a round of 7,200,000 ms;
- the default budget view,
  set back an hour after a refusal,
  held the refuser 3,600,300 ms on a 300 ms backoff;
- a budget view whose clock read 0 at its first reading took that reading for none
  and read every meter again inside the window;
- the repair benchmark,
  set forward two hours during its first entry,
  skipped the second on an hour's budget;
- `settleEntry`,
  set back an hour while it settled,
  wrote a `durationMs` of -3600000 into the artifact,
  which the two-lane reader refuses.

Fixed (`8204394fc`):
every difference between two readings in one process reads `monotonicMs` (`monotonic-clock.ts`),
which is `Math.floor(performance.now())`.
Node reads `performance.now` through `uv_hrtime` (`src/node_perf.cc` in v26.10.0),
which on Linux reads `CLOCK_MONOTONIC` (libuv `src/unix/linux.c`);
clock_gettime(2) says that clock is unaffected by jumps in the system time and does not count suspend,
and libuv's timers already run on a monotonic loop clock.
Floored,
every printed and stored duration stays a whole number of milliseconds,
the form the slice-cost reader and the artifact require.
Converted:
the stream idle guard,
the slice-cost log and reader,
the stage round's three marks,
the repair benchmark's run budget,
the corpus pass's soft budget,
stop-before-next check and closing line,
the pass entry and its decline line,
the sentinel probe,
and the default clocks of the provider router,
the provider budget and the request pace,
which the two hold modules' examples name too.
Two behaviours changed beyond the clock:
the pacer sleeps again for what is left whenever a sleep ends before its reserved start,
and the budget's stamp for a view never read is negative infinity rather than zero.
The first is also the likely cause of the request pace's real-clock case
falling short of its 19 ms bound at 18.89 and 18.99 ms (recorded with B73):
a take now returns only once a floored reading has passed its reserved start,
more than 19 ms after the case's own first reading,
an inference from the code,
since the short reads were never reproduced on demand.
`e2b0962fd` added to that stamp's TSDoc the second way zero fails on this clock,
which a guard-off found.
Docs that said "wall time" for these durations say time,
read on `monotonicMs`,
and tests that measured elapsed time with `Date.now` read `performance.now`.

Kept on the system clock,
since each outlives the process that wrote it:
ISO stamps (`new Date().toISOString()`),
the runs lock,
the lookup caches' `fetchedAt`,
`wallClock()` in `corpus-run/pass-outside-reads.ts`,
and `LAUNCH_STAMP`.
`corpus-run/run-timing-read.ts` takes its durations between the logger's stamps,
which are all a stored log keeps,
so a run during which the clock was set still reads wrong there.

Calls made here are open to veto:

- every duration inside one process is read on `monotonicMs`,
  the soft budget and the repair budget among them,
  so both now count only time the machine is awake,
  as the hard cap's timer already did;
- readings are floored to whole milliseconds rather than kept fractional,
  so no log or artifact format changes;
- the pacer sleeps again until its reserved start rather than trusting a timer's early end;
- the budget's never-read stamp is negative infinity rather than zero or an absent value;
- the scan reads tests as well as source,
  with the stub and its own test the two files it skips;
- docs keep "wall clock" for the system clock,
  while idiomatic speed phrases and the file name `where-a-round-spends-its-wall-clock.md` stay.

Guard-offs on the fixed tree,
each restored with `git diff` printing nothing after:

- `monotonicMs` reading `Date.now`:
  the first run hung `request-pace.unit.test.ts` after its FAIL line (ledger M100);
  after the stub was scoped to its case (`b1604a2f8`),
  all eight files exited 1,
  each failing only its B78 case,
  with the red values
  (1790809200000,
  3660000,
  -3600000,
  `'ms -3600000'`,
  7200000,
  3600300,
  `[ 'error', 'skipped' ]` and -3600000);
- the pacer's loop cut to one sleep:
  the sleeps-again case failed (1059999 for 1060000);
- the budget stamp's exact revert,
  zero plus the check reading zero as never:
  only the reads-once case failed,
  with two reads of each meter;
  zero without that check failed nine of the file's cases,
  since on this clock zero is the process's start
  and every call in the first freshness window took the pre-read view as fresh;
- the scan with `Temporal.Now` unrecognised:
  its fixture case failed;
- the stub replacing `Date.now` for the whole process:
  ledger M100.

The full suite on `3a6f92f13`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1,500 PASS lines and no FAIL line.

Out of the scan's reach,
and named in its module note:
`Date`,
`performance` or `Temporal` reached through `globalThis` or another name,
a date compared with `<` or `>`,
and a duration taken between stamps a process wrote.
TSDoc examples are comments,
which the scan does not read,
and were fixed by hand.

Recurrence:
a difference of two readings in one process reads `monotonicMs`,
and the system clock is read only as a stamp;
`wall-clock-reads.unit.test.ts`,
which `source-scans` runs,
fails on `Date.now` called,
handed on or destructured,
`getTime` or `valueOf` called,
a unary plus or `Number` on a date,
`performance.timeOrigin` and `Temporal.Now`,
in source and tests alike.

## B318: two cases kept a late voice inside a grace window a slow timer could outrun

Fixed in `f3c2bae13`.

Found on 2026-10-06 (UTC) by the agent of B316,
searching the tests for bounds on the clock.
`lane-contest-stage.unit.test.ts`,
"KEEPS DELAYED ELIGIBLE VOICES that arrive inside bounded grace",
and `absolute-naturalness-review-stage.unit.test.ts`,
"KEEPS REJECTION that arrives inside bounded post-quorum grace",
each expected a voice delayed 30 ms to land inside a 100 ms grace window,
which a loaded machine could outrun.

The fix:
both give a window of 60,000 ms,
which the round never waits for,
since it ends at the last answer.
By the agent's measurement,
each case fails at a window of 0,
so the window is what keeps the late voice.

Left by the agent,
each with its reason and none shown to fail:
peak-concurrency instruments that wait 10 ms,
meters that wait 20 ms,
a bound of 5,000 ms over about 50 ms of work,
and a lower bound only.
A bound of that kind has since been shown to fail:
the consolidation driver test's 5,000 ms hang bound,
outrun on a loaded machine (B340);
whether it is the bound listed here is not established.
In the census of B358 the meters and instruments of that kind were measured and kept.

Open to the owner's veto:
a window of 60,000 ms and no gate in the scripted clients.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B340: the consolidation driver test's 5,000 ms hang bound outran on a loaded machine

Fixed in `5086b6c6f`.

Found on 2026-10-06 (UTC)
when the gate on `9b028204d` failed one case of the whole suite.
`consolidate-driver.unit.test.ts` bounded each call,
and every drive passing no signal of its own,
by `CALL_TIMEOUT_MS`,
5,000 ms,
and its TSDoc said the bound was never reached because nothing bought a call.
The case "runs two consolidation slices at once at overlap 2,
after a serial positive control ..."
buys calls through `recordingClient`,
and failed with a `TimeoutError` its name never states:
in that run (6.0 s,
the logger's own 5 s sink check timing out just before)
and in another agent's suite run (13.9 s).
The twin memo logged a buy abandoned,
"refused by TimeoutError".
The case passed in every other gate.

The fix:
the bound is 60,000 ms,
as the grace windows of B318 are:
a stop for a hang,
which no case waits for,
since every scripted client answers at once.
The one case of the file that stops a run brings its own controller,
and the TSDoc says what the bound covers and why it is that size.
No run at a bound of 1 ms was made:
the failure already shows the bound on the case's path.

Open:
the same shape,
a real-clock bound of a few seconds over scripted work,
stands at 604 sites in 138 test files
(by the lead's search for `AbortSignal.timeout`,
`perCallTimeoutMs`,
a `TIMEOUT_MS =` declaration
and `timeoutMs:` with a number),
and a census of them waits for the merges in flight:
each is either a stop for a hang,
moved to one documented bound,
or a bound a case means to reach,
kept and gated.
B318 left "a bound of 5,000 ms over about 50 ms of work" not shown to fail;
whether that is this site is not established.
Closed since by B358 in `4cca8c301`.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B358: bounds on the real clock across the tests, which a loaded machine could outrun

Red in `1f983c322`,
fixed in `4cca8c301`.

Found on 2026-10-06 (UTC) by the census B340 left open,
after a 5,000 ms signal on a whole drive of scripted consolidation work ran out twice under load.
By the agent's census at its base `a7eca7f`,
the tests and fixtures set 976 bounds on the real clock in 194 files
(the argument of `AbortSignal.timeout`,
the delay of a timer or a sleep under any local name,
and twenty option keys production arms a timer with or compares elapsed time against):
948 outside seven files deferred to the changes in flight on them,
and 28 inside.
The lead's search for B340 had counted 604 sites in 138 files;
the agent could not reproduce that count,
and its closest pattern gave 683 lines in 144 files.
`real-clock-bounds.unit.test.ts` fails on each such bound that is not `HANG_STOP_MS` of `hang-stop.test-fixture.ts`,
unless it is listed with its class and reason or sits in a deferred file.
The agent's red at `a7eca7f`,
with an empty list,
gave 948 findings in 442 keys;
on main at `1f983c322` the package case fails with findings differing from the list,
which names the values the fix leaves.

The fix:
787 of the 948 bounds,
in 152 files,
read `HANG_STOP_MS`,
60,000 ms:
a stop for a case that hangs,
never a limit a passing case approaches.
158 stay,
each listed with a class and a reason:
33 a case reaches,
86 a case reads back
and 39 other.
Three sleeps that stood for ordering became gates.
The scan joins the source-scans task,
and "Tests on the real clock" carries the census and its classes.
By the agent's measurement,
no describe of the moved files grew by more than 0.8 s;
with the drive signal of `absolute-naturalness-review-stage` at the old 5,000 ms and every seat slowed by 5.1 s,
its cases fail with `TimeoutError`,
and at the hang stop they pass.
The agent measured and kept the meters and instruments that wait 10 or 20 ms:
with a hold of 1,000 microtask turns they all pass and with none they all fail,
so the overlap they hold for is in place within one turn of the event loop.
The holds in the budget's coalescing cases and the router's slot cases decide nothing,
since those cases pass with no hold,
and the windows moved to the hang stop in `stage-recovery-round`,
`stage-windowed-rounds`,
`naturalness-quorum-roundtrip` and two `stage-quorum` recovery cases pass at a window of 0.
The cases whose outcome depended on the clock are B359 to B363,
and B364 records what the census found about per-call deadlines.
Run on the merged tree before the commit:
lint clean,
49 scans passing,
and the suite at 2,027 passing with four cases failing,
the four B352 leaves open.

Open:
the seven deferred files,
28 reads,
wait for the changes in flight on them,
with each read's class in the agent's report;
two upper bounds on elapsed time in `stage-round.unit.test.ts`,
which the scan does not read and which load can break,
are left for the lead to restructure;
and the logger package's own sink verification bound of 5,000 ms timed out 34 times in the agent's suite run
without failing a case.
Closed since by B378 in `b7244804f`,
except the logger package's sink check,
which stays open.

Open to the owner's veto:
The name `HANG_STOP_MS`,
and a fixture file of its own for it.
Five drive signals of `CALL_TIMEOUT_MS * 8` collapse into one hang stop.
Bounds of 120,000 to 600,000 ms come down to 60,000.
The runner `timeout:` of `wall-clock-stub` rises from 10,000 and that of `pass-footnote-lifecycle` from 30,000,
both to 60,000.
Windows of 0,
50 and 60 ms move to the hang stop although they decide nothing at 0,
so a seat that ever stopped answering would hold its case for a minute.
`stage-call` keeps its 1,000 ms,
as a value read back.
Files are deferred whole rather than site by site.
Waits and overrides through environment strings are left out of the guard.
Guard keys use the nearest named function,
so identical forms in one function share a key and one class,
and the key of `SHORT_BOUND_MS` in `stream-bound` covers both a bound a case reaches and a record field.
The 150 ms pause of `gated-git` stays,
since a slow machine can only let its case pass without exercising the order it scripts,
never fail it.
The upper bounds of 880 s and 10 s in `transient-retry` stay.
The agent's report names no alternative for these choices beyond the one a "rather than" states.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B359: stalled seats in the quorum and refill cases raced a sleep against the clock

The guard's red is `1f983c322`,
and the cases changed in `4cca8c301`;
their red is the agent's control under a forced stall,
since the old cases pass when nothing stalls.

Found on 2026-10-06 (UTC) by the census of B358.
In `stage-quorum.unit.test.ts` the stalling seat slept 30 s,
and the cases "BOUNDS the recovery round" and "ABANDONS a voice" asserted elapsed time under 1,000 and 5,000 ms;
"still hears a voice INSIDE the grace" relied on a window of 2,000 ms.
In `stage-round-refill.unit.test.ts` the slow seat slept 1.5 s before the quorum,
for a round that must close on the others.
By the agent's controls,
with a synchronous busy loop standing in for a stalled process:
the old stalling case fails after a stall of 1.1 s,
the old refill case after a stall of 1.6 s before the quorum,
since the slow seat is then heard,
and the old grace case with its voice 2.1 s late;
each new shape passes under the same stall.

The fix:
the stalled seat waits on the abort,
records its reason and throws it,
and the cases assert what cut it
(`cuts` equal to `['AbortError: This operation was aborted']`,
and `cut` equal to `{ aborted: true, reason: 'AbortError: This operation was aborted' }`),
not the time spent;
the refill's slow seat waits on the abort and throws;
and the grace case's window is the hang stop,
which by the agent's measurement fails at a window of 0,
so the window is what keeps the voice.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B360: the lane-contest overlap case could pass without the finishing order it names

The guard's red is `1f983c322`,
and the case changed in `4cca8c301`;
its red is the agent's control.

Found on 2026-10-06 (UTC) by the census of B358.
`lane-contest-driver.unit.test.ts` ordered its overlapping slices by sleeps of 20 and 5 ms,
and asserted no finishing order.
By the agent's control,
a stall of 40 ms before the second slice's timer lets the first slice finish first,
and the old case still passed without exercising its order;
the old shape with the order assertion added fails,
and the new shape passes.

The fix:
the first slice's calls wait on a gate the second slice opens when every one of its calls has finished,
and the case asserts the finishing order,
the second slice's calls ahead of the first's.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B361: the bedrock per-attempt bound case raced each attempt against its own bound

The guard's red is `1f983c322`,
and the case changed in `4cca8c301`;
its red is the agent's control.

Found on 2026-10-06 (UTC) by the census of B358.
In `bedrock-client.unit.test.ts`,
"GIVES EACH ATTEMPT THE WHOLE STREAM BOUND",
each attempt slept 200 ms under a stream bound of 300 ms.
By the agent's control,
a stall of 150 ms fails the old case,
and the new one passes.

The fix:
both attempts answer at once,
and the time passes in the backoff between them,
whose base of 1,000 ms sleeps at least 500 ms,
longer than the bound,
so a bound armed once around the retry ladder has run out before the retry starts.
The case's name now says the retry is not cut by the time spent before it,
in the failed attempt or the backoff.

Open to the owner's veto:
The case waits in the backoff it names,
and its name changed.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B363: the nesting differential timed its parses on the real clock

The guard's red is `1f983c322`,
and the case changed in `4cca8c301`;
its red is the agent's control.

Found on 2026-10-06 (UTC) by the census of B358.
`nesting-fence-differential.unit.test.ts` held each parse of a text the scan passes to 1,000 ms of real time,
which a process taken off the CPU mid-parse stretches without bound.
By the agent's control,
a wait of 1.1 s on `Atomics.wait` inside one parse fails the old case,
and the new one passes.

The fix:
the parse time is read with `process.cpuUsage`,
user and system time together,
and the limit's TSDoc says why.

Open to the owner's veto:
The parse limit is measured on CPU time.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B364: a per-call deadline handed to a stage over a hand-scripted client is never armed

The values moved in `4cca8c301`;
no case holds the finding.

Found on 2026-10-06 (UTC) by the agent of the census of B358.
Only the provider clients arm a per-call `exchangeTimeoutMs` or `perCallTimeoutMs`,
through `armCallDeadline`;
besides them,
only the pass entry's hard cap,
`model-health-probe`,
`provider-listing` and `translate-probe-run` arm a timer in production.
By the agent's controls,
`document-lanes` at its old per-call bound of 50 ms,
with every scripted call slowed by 60 ms,
still passed;
and with the deadline at 200 ms and the grace window at the hang stop,
a case ran 60.4 s and passed on the grace alone.
What fires in a test is a signal,
a cap,
a window or a stream bound that the test arms itself
or hands to production code that arms it;
by the agent's reading,
the failure of B340 was the whole drive's `AbortSignal.timeout(5_000)`.
So a per-call value a test hands to a stage over a scripted client bounds nothing.

The change:
those values moved to the hang stop anyway,
so a stage that comes to arm them waits a minute.

Open to the owner's veto:
Per-call deadlines that are never armed moved to the hang stop anyway.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B378: the seven files the clock census deferred, and two ceilings on elapsed time in the round test

Red in `37c3419ea`,
fixed in `b7244804f`.

Found on 2026-10-10 (UTC) by the follow-up the census of B358 left open.
With `DEFERRED` emptied and no other edit,
`real-clock-bounds.unit.test.ts` fails with 34 findings in 15 keys,
all in the seven files
(`coverage-probe-run`,
`recall-benchmark-run`,
`sentinel-probe-run`,
`settled-carve`,
`translate-probe-run`,
`window-trial-probe-run` and `stage-round`),
by the measurement of the agent that wrote the change:
six more than the census counted at its base,
from cases added to `stage-round.unit.test.ts` in `d1e6af41c` and `154da3c68`.
That file held the round's quorum mark under 125 ms past its first answer,
which failed in 2 of 4 runs beside the stop cases (B365),
and held the grace of a round whose roster all answered under its 250 ms window.
By the agent's controls,
a busy loop of 300 ms between the first answer and the quorum mark fails the first,
and the same stall as the round cuts its seats fails the second.

The fix:
22 reads that only stopped a hang read `HANG_STOP_MS`;
12 stay listed with a class and a reason:
one window the case that loses a voice reaches,
seven values read back
(`recall-benchmark-run`,
`sentinel-probe-run`,
`settled-carve`),
and four other.
Six of the round test's grace windows moved to the hang stop,
since those rounds end before their window:
by the agent's probe,
every case passes at a window of 0,
where at the old shape only the case that loses a voice fails there.
The quorum mark is held to the span from the round's start to the next turn of the event loop,
which the microtasks between the first answer and the mark cannot outlast;
the whole-roster round's line is read at a timer armed after its last voice's own and as long,
and its grace is held to the span from that answer to that turn.
By the agent's controls,
each new shape passes under the same stall,
and fails when quorum waits for the slow voice,
or when the round waits out a window.
`DEFERRED` is empty,
with TSDoc saying why;
`DEFERRED_REASON`,
which nothing read,
and `QUORUM_MARK_SLACK_MS` are gone.
The lower bounds and the round's bound of an hour stay as the census left them.
Run on the merged tree before the fix commit:
lint clean,
49 scans passing,
and the suite at 2,030 passing with no failure.

Found on the way:
the per-call deadline of `translate-probe-run.ts` is armed as the whole slice round's signal,
and no case reaches it;
the other per-call deadlines in these files go to scripted stages or clients that never arm them.

Open:
the logger package's own sink verification bound of 5,000 ms timed out 21 times in the agent's suite run,
without failing a case.

Open to the owner's veto:
`concurrency: 1` stays on the round test's top-level describe,
its comment rewritten to give its history;
the alternative is to drop it,
now that the timing case no longer breaks beside other work.
The second case's ordering instrument is a `wait(SLOW_MS)` the case arms,
listed as class 4;
the alternative is a `setImmediate` started from the slow seat's answer through a hook on `scheduledClient`,
which adds no bound for the guard to read.
The second case's window moved from 250 ms to `HANG_STOP_MS`;
the alternative is to keep 250 as a value of class 1,
since the ordering check makes the window's length irrelevant there.
`CALL_TIMEOUT_MS` of `sentinel-probe-run`,
1,234 ms,
stays a distinctive value listed as read back;
the alternative is `HANG_STOP_MS`,
which the read-back would still match but which no longer marks the value as the test's own.
Two reason constants are new,
`RECALL_CALL_CONFIG` and `SENTINEL_DEADLINE`;
the alternative is reason strings written inline.
The round test's file TSDoc gains a paragraph,
"NO CEILING ON THE CLOCK";
the alternative is to leave the rule to `mistake-prevention.md` alone.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B383: the fidelity probe read its start after its references

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers,
from the code,
since the lead's brief named it with no source text (M142).
`runFidelityProbe` read the start it records in its plan after it had read every reference,
so the time those reads took was left out of the run the plan and the kept run date.
By the agent's run against the base build,
the case
"READS THE START OF THE RUN BEFORE IT READS ANY REFERENCE and the finish after the last row,
so the plan and the kept run date the whole run and not the end of the corpus read"
fails;
it counts the reference reads made by each clock reading,
and expects `[0, 1]`.

The fix:
`startedAt` is read once,
after the request check and the return for a cap of 0,
before `readReferences`.
By the agent's census of the other runners,
`audit-sensitivity`,
`recall-benchmark`,
`coverage-probe` and the settled audit's drive read their start before any work;
`sentinel-probe` reads each entry's start before that entry's reads;
`roster-bench` reads its start before each stage call;
and `pass-entry.ts` reads it at the entry's start.
`corpus-pass` starts its loop clock after selection and republish,
which its documentation calls loop time shared with the soft budget,
and is unchanged.

Recurrence:
`mistake-prevention.md`,
"Durations on the system clock".

## B402: the real clock scan did not read the windows a stage settings value carries

Found and closed in `7285b3c98c55a4654832943f3c444569cb536182`,
the red commit of B401:
the scan lives in its own test file,
so it passes against any production code,
and `e749bca51e21079a2ee4289ad20ae73206900c7e` does not touch it.

Found on 2026-10-10 (UTC) by the agent working the open items B398 left.
`real-clock-bounds.unit.test.ts` keyed `graceMs` but not `roundMs` or `writerMs`.
Production hands every round its window from `stageSettings.grace.roundMs` or `stageSettings.grace.writerMs`
as `graceMs`,
so those are the other names the straggler window goes by.
`CalibrationGrace.effectiveMs` and the `builtInMs` of `graceOverrideNote` are only printed,
and stay out of the keys,
as the TSDoc of `BOUND_KEYS` says.
The red is the scan's own fixture case:
by the agent's control,
a fixture text with two new `grace` literals fails
"FINDS a bare or local bound ..."
with 12 findings against the 14 it expects until both keys are added.
With both keys added,
the package-wide case lists 30 values not listed:
2 in `stage-settings.test-fixture.ts#scriptedStageSettingsWaiting`,
4 in `stage-settings.unit.test.ts#<module>`
and 24 in `writer-grace-override.unit.test.ts#<module>`.
The 28 in the two unit test files are windows a case reads back,
and no round runs under them.
The 2 in the fixture hand on its `graceMs` parameter,
whose callers' `graceMs:` values the scan already reads where they stand.
So none was a bound on the real clock the scan had missed,
and none moved to `HANG_STOP_MS`;
each is listed in the scan's `LISTED` with its reason:
`HANDED_ON` for the fixture's two;
`READER_OUTPUT`,
the output of `readStageSettings` asserted whole,
for the four of `stage-settings.unit.test.ts`;
and `WRITER_GRACE_READ_BACK`,
an output of `readWriterGrace` asserted whole,
or a grace handed to `writerGraceOverrideNote` and read back in its line,
for the 24.
The example B398 gave,
the `WriterGrace` literals of `editor-calibrate-run.unit.test.ts`,
cannot fire:
they are already `HANG_STOP_MS`.
By the agent's controls,
each reverted:
deleting one listed entry fails the scan with the finding
"found and not listed" for the `roundMs: 90_000` value of `stage-settings.unit.test.ts`;
and dropping `roundMs` from `BOUND_KEYS` fails the fixture case
and reports each listed `roundMs` key as "listed and not found".
One of the agent's controls set a literal bound in the tree for one run and put it back,
against the brief;
the lead found that file's `git diff` empty.
This closes the second open item of B398.

Open to the owner's veto:
one reason covers both the note inputs and the reader outputs of `writer-grace-override.unit.test.ts`,
where splitting the cases into named functions,
so the keys tell them apart,
is the alternative;
`effectiveMs` and `builtInMs` stay out of the keys,
where keying them and listing every printed value is the alternative;
and no value the scan found moved to `HANG_STOP_MS`,
where moving them is the alternative.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B416: tests armed a model card's stream bound on the real clock, which the real clock scan could not see

Red `0f8847e91f`,
fixed in `5b0b8fef14`.

Found on 2026-10-11 (UTC) as the open item B401 recorded.
The Bedrock client arms each card's `streamBoundMs` around every attempt unless handed `streamBoundMsOverride`,
and the real clock scan keyed neither the card's field nor a client built without the override.
By the agent's probe over one whole suite run,
`bedrock-client.unit.test.ts`,
`client-log-context.unit.test.ts` and `owner-body-rules.unit.test.ts` armed the Gemma cards' 60,000 ms bound in 14 attempts,
the same length as the hang stop,
so no case was cut;
their timing rested on a production measurement that no scan or test showed.
No test or fixture wrote `streamBoundMs`.

The fix:
the scan keys `streamBoundMs`,
reads a `createBedrockClient` from the built package whose options name no override,
and leaves a stream bound of `'unbounded'`;
`streamBoundMsOverride` takes `'unbounded'`;
the three files hand it,
and one listed case reads the catalog for the gpt-oss card,
which carries no bound,
so the client's fallback to the card's bound is covered.
The change is red on the base build only to the type checker,
since the client already passed `'unbounded'` through.
By the agent's control,
setting the test helper's override back to a bound fails the case that asserts no timer is armed.
Two same-family fields were read and left:
`waitOutHold` takes a real hold wait in milliseconds that `budget-hold-wait.unit.test.ts` sets without the scan seeing it,
and `holdMs` arms no timer.

Read and left:
`budget-hold-wait.unit.test.ts` calls `waitOutHold` with a real wait of 6 milliseconds
and asserts only a lower bound of 5 on the time it took,
and with a wait of 50 milliseconds under a signal that has already aborted,
which ends before any wait;
both follow the real clock rule in `mistake-prevention.md`.
The scan does not key the `ms` parameter,
since keying it would catch many unrelated record fields.

Open to the owner's veto:
`'unbounded'` in the tests,
where the alternative is the hang stop,
which needs no production change but keeps a real 60 second timer armed;
the scan flags every test client built with no override,
where the alternative reads only the two keys and misses the measured sites;
and the scan leaves `'unbounded'`,
where the alternative lists each such site.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock".

## B713: the archive block review round waited the readers' window for replies that are writer voices

Status:
fixed on 2026-10-21 (UTC),
red `a38bc52e29`,
fix `96d9d0be32`.
Found in the investigation of the letter block of `zheermao20`.
No cache version moved.

What the run showed:
the review round asks each reviewer for the complete corrected block,
so a `revise` reply is a whole candidate,
and the owner's reason for the 180000 ms writer window is that a cut writer voice is a whole candidate lost
(`doc/decision/translation-repair-straggler-grace.md`,
decision of 2026-09-06).
The stage passed `grace.roundMs` (120000 ms),
and no decision had classified the round either way.
The letter block's Qwen reply was cut 120000 ms after quorum,
243 s into a 360 s call deadline,
and the block shipped without it.

The fix:
the gather passes `grace.writerMs`,
`archive-block-review` joins `WRITER_STAGE_LABELS`,
and the `WRITER GRACE built in` line names it and said it joined on 2026-10-21 and was open to the owner's veto.
That note was removed in `29f86a2863`:
a run log states what the build does,
and whether the owner has confirmed a classification is the status of a decision,
which the straggler grace decision record and `configuration.md` already hold;
left in the line it would sit in every run log after the owner has answered.
The line keeps the stages,
both windows,
the decision date of 2026-09-06 and the dial that moves it.
Only two files carry the line's text:
`writer-grace-override.ts` and the `WRITER_GRACE_BUILT_IN` constant of `corpus-pass-built.test-fixture.ts`.
This entry first said five files pinned it and the fix commit's message said four;
`corpus-pass.unit.test.ts` and `corpus-pass-refusals.unit.test.ts` import the constant,
and the bench and stage units pin only the overridden line or the words "built in".
`configuration.md`,
`seats-and-calibration.md`
and a dated addendum of 2026-10-21 in the straggler grace decision record it.
The selection round after the review keeps the readers' window:
its judges read text that exists.

This session made the classification under the owner's rule and the owner has not confirmed it.
The addendum says how to veto it.

The red case holds a reviewer past quorum under `writerMs` of the hang stop and `roundMs` of 0,
and failed on the previous tree with one `stage-voice-lost (archive-block-review ...)` finding.
A second case holds a judge the same way in the selection round and expects it cut and said lost;
it passed before and keeps passing,
so it pins the readers' window rather than shows a change.
Both use a straggler delay of 25 ms past peers that answer at once,
so the red case could pass by accident only if the held reply landed inside a zero window,
and the green case cannot fail on load.

Not measured:
whether the extra 60 seconds turns the cut into a rewrite or into a cap-cut empty reply
(the same block in `zheermao19` ran its Qwen stream to the cap with no content),
and how often a review round reaches the window at all.
The cost is at most 60 seconds on a review round with a straggler past 120 seconds,
bounded by the 360000 ms call deadline.

## B813: the held review seat landed before quorum once under the sweep's load, and a bounds entry said it never could

Status:
fixed on 2026-10-22 (UTC),
fix `94b50fca0f`,
cache versions none.
A member of the family of B318 and B713,
found by the rotation sweep.

What the sweep showed:
the case "KEEPS THE READERS' WINDOW IN THE SELECTION ROUND (ledger B713)" of
`package/translation-repair/archive/src/archive-block-review-windows.unit.test.ts`
was red once with no shift (salt 0) in a full rotation sweep,
and it passed alone.

The cause:
the scripted review client (`archive/src/scripted-review-client.test-fixture.ts`)
held the straggler for `HELD_REPLY_MS` = 25 ms
counted from the straggler's own request.
The stage's own work between the three peers' answers and the quorum tally is real time,
and under the sweep's load it took longer than 25 ms.
The held seat therefore landed before quorum,
and the readers' window had no straggler to cut.
The real-clock bounds entries
(`package/translation-repair/source-scan/src/real-clock-bounds.unit.test.ts`)
claimed the held seat lands after quorum "on any machine",
which was false:
the claim covered the client's own timer and left out the stage's work after the client returns.

The fix:
the hold is 1000 ms,
still far inside the 60000 ms hang stop that the writers' window waits,
and both bounds entries now say why.
After the fix the archive sweep was clean over salts 0 to 9.

What was tried and dropped before the commit:
a client-side barrier that holds the straggler until every unheld seat has answered.
The unheld seats answer synchronously,
so the barrier holds nothing;
the slow part is the stage after the client returns,
which no client-side signal can see.

Why B713 did not catch it:
B713 used a straggler delay of 25 ms "past peers that answer at once"
and said of its cases that the green case "cannot fail on load".
That reasoning covers the delay between the peers and the held seat,
and leaves out the work the stage does between the last peer's answer and the tally.
The second case of B713,
the readers' window in the selection round,
is the one that failed.

Not measured:
how long the stage's own work takes under the sweep's load.
The hold of 1000 ms is a size chosen to be far above 25 ms,
and no run measured the longest stage span.

Cache versions:
none moves.
The change is in a test fixture and a test file.

Recurrence:
`mistake-prevention-tests-clocks-and-arrival-order.md`,
"Tests on the real clock":
a real-clock hold in a test fixture is sized against the stage's own work under load,
and no bound entry claims "on any machine" for it.
