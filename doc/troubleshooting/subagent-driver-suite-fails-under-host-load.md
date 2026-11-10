# The subagent driver's suite failed on a busy host, and its waits were why

The agent that built the overdue-agent watch reported two failed runs of the suite in
`package/pi-shared/subagent-driver` while the host's load average was 106,
with waits of 60 s,
and called them
"the known contention behaviour".
No failing test name or output was kept,
and a later run on the same branch passed.
This record enumerates every wait the suite had,
the failures that a capped processor reproduced on
2026-11-09T05:20:00-05:00 and later,
the mechanism each one measured,
and what each fix does not claim.

## The waits the suite had

Every wait in the suite is one of these.
The files that hold only stand-ins (`look-at-pi-runs`,
`running-pi-units`,
`checked-runs`,
`tool-call-times-of-a-feed`,
`parse-*`,
`end-reasons`,
`read-pi-fields`,
`environment-without-credentials`,
`split-pi-output`,
`watch-for-overdue-runs`,
`overdue-limits`) hold no clock and no timer.

### `src/drive-pi-subagent.unit.test.ts`

- `waitForLine` polled every 20 ms for a feed or stand-in log line,
  and gave up after `WALL_SECONDS`,
  which was
  60 s:
  the same 60 s as the run's own wall clock.
  On a host slow enough to delay start-up and one event,
  the
  run reached its wall clock first and the line could no longer be written.
- The run's `--wall-seconds` was 60 by default,
  1 s for the two tests whose subject is the wall clock,
  and 4 s
  for the stalled delivery.
  The 4 s one asked a delivery to happen before a wall clock that start-up alone can
  spend.
- `--idle-seconds` was 0.2 s by default and 60 s in the tests that keep the run alive while a test acts;
  the
  60 s idle sat under a 60 s wall clock,
  so neither could be trusted to outlast the other.
- `stopBounded` waits 120 s for the driver to leave after a SIGKILL,
  and the driver ran under
  `execFile`'s own 180 s timeout:
  both are safety nets that no working test reaches.

### `src/connection-to-pi.unit.test.ts`

- `GRACE_MILLISECONDS` (1.5 s) bounded each of the four stages of ending Pi:
  its input closing,
  one signal,
  the last signal,
  and the copy of its standard error.
  A starved host delays Pi's own exit,
  and the test then
  saw a signalled end where it expected `code: 0`.
- `ANSWER_MILLISECONDS` (10 s) bounded an answer that must arrive.
- `SHORT_ANSWER_MILLISECONDS` (50 ms) bounded an answer that must not arrive,
  next to a stand-in that leaves
  1 s after starting;
  the test then asserted Pi had not left yet,
  which is two timers racing.
- `REPORT_DEADLINE_MILLISECONDS` (10 s) bounded a report or a log line the connection writes as the event
  arrives.

### `src/record-of-pi-run.unit.test.ts`

- `FEED_FAILURE_DEADLINE_MILLISECONDS` (5 s) bounded the feed's open failing,
  which is an in-process file
  open in a folder that is not there.

### `src/run-command.unit.test.ts`

- A 100 ms bound kills `sleep 30`,
  and the test asserts only that the whole call took less than 30 s,
  so no
  host schedule can decide it.
  Nothing here needed a change.

## What was measured: a capped processor reproduced these failures

Every command ran in its own `systemd-run --user --scope` with `CPUQuota`,
`MemoryMax`,
`TasksMax` and
`IOWeight=10`;
a quota is a limit,
never load.

```sh
systemd-run --user --scope --property=CPUQuota=50% --property=MemoryMax=8G \
  --property=TasksMax=1024 --property=IOWeight=10 \
  -- mise run //package/pi-shared/subagent-driver:test:unit
```

The whole package's suite at 50 percent of one processor failed on the stalled delivery:

```text
[drive-pi-subagent] [an inbox answer that stalls ends at the wall clock, not on a delivery that never returns]
  [FAIL] (9.0s) AssertionError: expected undefined to equal false
  @drive-pi-subagent.unit.test.ts:619 expect(run.feed.find((entry,) => entry.kind === 'inbox',)?.success,).toBe(false,);
```

The driver's own file alone at 25 percent failed thirteen tests in one run.
The three that name a measured mechanism:

```text
[a run that never settles ends at the wall clock and is aborted] [FAIL] (18.9s)
  AssertionError: expected [ 'get_session_stats' ] to include 'abort'
[an inbox answer that stalls ends at the wall clock, not on a delivery that never returns] [FAIL] (13.7s)
  AssertionError: expected 'pi-exited' to equal 'wall-clock'
[sends the prompt file and each inbox message whole to Pi] [FAIL] (60.1s)
  Error: the file never showed "kind":"settled","n":1
[a Pi that exits while the watch runs ends it as pi-exited] [FAIL] (60.4s)
  Error: the file never showed "message":"hello"
[a rejected first prompt fails at once instead of waiting for the wall clock] [FAIL] (76.1s)
  AssertionError: expected 'wall-clock' to equal 'prompt-rejected'
```

At 15 percent,
the same file failed the abort assertion alone:

```text
[a run that never settles ends at the wall clock and is aborted] [FAIL] (22.3s)
  AssertionError: expected [ 'get_session_stats' ] to include 'abort'
```

The recovered model error failed once in a run at 15 percent:

```text
[a model error that an automatic retry recovers still ends clean] [FAIL] (25.1s)
  AssertionError: expected 1 to equal +0
```

One failure in that 25 percent run was not this defect:
`an inbox entry that is a named pipe is set aside
instead of waiting for a writer` failed with `spawn mkfifo EAGAIN`,
which is the host refusing a fork,
and
the same cap did not reproduce it later.

## Cause and fix

- **A run's wall clock and a test's wait were the same 60 s.**
  The driver counts the wall clock from before
  Pi starts,
  so start-up is inside it;
  a test that waited 60 s for an event was racing the limit that ends
  the run.
  The runs a test ends itself now allow 300 s,
  and a wait for a line ends as soon as the driver has
  ended,
  throwing the lines the file did carry instead of a bare "never showed".
  Removal check:
  with 60 s
  restored and the abort assertion put back,
  the driver's whole file failed at `CPUQuota=15%` with
  `expected [ 'get_session_stats' ] to include 'abort'` (27.3 s) and,
  at 25 percent,
  with the same assertion
  (18.1 s).
  Not claimed:
  that 300 s is enough for every host,
  only that it is far above the events a test
  waits for and that a run which truly hangs still ends either at that bound or at the test's own outer
  deadline.
- **A stalled delivery was asserted through a wall clock shorter than start-up.**
  The event that matters is
  the bound the watch asks for when it delivers,
  which is no more than what the run has left.
  A new test in
  `src/watch-pi-run.unit.test.ts` hands `watchUntilEnded` a connection that stalls for exactly the bound it
  was given and a real feed and inbox,
  so the claim is measured with no process starting.
  Removal check:
  the
  delivery bound restored to `COMMAND_ANSWER_MILLISECONDS` fails it with
  `expected 30000 to be at most 600`.
  Not claimed:
  that the removed end-to-end test covered the claim
  elsewhere,
  only that the claim is now measured at the seam and the failure paths around it (a refused
  message,
  an unreadable entry,
  a named pipe) stay covered end to end.
- **A run whose wall clock ended before Pi's first event was asked to abort.**
  The teardown aborts only a Pi
  the record holds at work,
  and the wall-clock test could not know Pi had started a turn inside a 1 s limit
  on a slow host.
  The abort is now asserted on a run the test ends with its stop file after the feed has
  shown Pi's work (`a message dropped between agent_end and agent_settled goes as a steer`),
  where nothing
  depends on a clock.
  The wall-clock test asserts what a run over its wall clock always ends with:
  the wall
  clock reason and a recorded Pi exit.
  Not claimed:
  that the composition of the two is the same single test.
- **A recovering retry was written after the settle it answers.**
  The record takes an error as the run's own
  only when Pi settles on it,
  so a settle between the error and the retry leaves the failure standing until
  the retry's reply clears it,
  and a watch that looks in that gap ends the run.
  A probe that feeds the record
  in that order,
  with the retry two looks later,
  ends `model-error` every time.
  The stand-in's recovering
  retry now answers the error inside the same burst,
  and the post-settle clearing stays covered by
  `the record of a run`,
  "a later reply that succeeded clears a model error Pi settled on".
  Not claimed:
  that
  real Pi never settles before an automatic retry;
  no evidence either way was gathered.
- **Pi's leaving got 1.5 s per stage.**
  The tests about Pi leaving on its own terms now leave the bound the
  driver ships (5 s);
  the 1.5 s bound stays only where the stage bound is the subject (a Pi that ignores its
  input closing or its first signal,
  and a standard-error copy that never finishes).
  Not claimed:
  a
  reproduced failure.
  The stand-in left within either bound at every cap tried,
  including 10 percent of one
  processor,
  so this raise is the driver's own shipped bound applied to the tests,
  not a measured defect.
- **An answer that must arrive and a report the connection makes had 10 s.**
  Both are now 60 s;
  they cost
  nothing unless the connection itself is broken.
  No failure was reproduced for either.
- **The feed's open failure had 5 s.**
  It is an in-process open;
  the wait is now 60 s and names what never
  appeared.
  No failure was reproduced for it.
- **A killed driver reported a null exit status.**
  `exitOf` now names the signal in the text a failing test
  prints,
  so a test the outer deadline killed says so.

## Verification

- `mise run //package/pi-shared/subagent-driver:buildAndTest`:
  28 suites pass,
  no failure.
- `mise run //package/pi-shared/subagent-driver:lint`:
  clean.
- The whole suite at `CPUQuota=50%`:
  passes,
  25 s for the driver's file.
- The driver's file alone at `CPUQuota=15%`:
  passes in 276.7 s.
- The connection's file alone at `CPUQuota=10%`:
  passes.

## Two tests that decided their ending on the host's speed, found 2026-11-09

Two tests in the driver's file passed on a quiet host and failed on the one this was written on,
at load average 83 on four processors,
after the retry work of that branch added runs to the file.

- The test that a message dropped during a retry wait goes after the retry prompt
  asserted the message went as a `prompt`.
  The driver chooses `steer` or `prompt` from `record.isRunning()` at the moment the message is delivered,
  and Pi's settle for the retry's own run can be unread while the read of Pi's answer finishes.
  On a loaded host the message then went as a steer to a run Pi had just finished.
  The order of the messages is what the finding was about,
  so the test now reads the messages themselves and not the command that carried each one.
- The test that a retry is held while an inbox-started run is going
  gave the run a 3 s wall clock and let the wall clock end it.
  On that host the stand-in's own start-up spent the 3 s:
  the feed showed `state` at 3003 ms and the first prompt was answered `no answer in 0 ms`.
  The run now has the file's ordinary 300 s backstop,
  and the test ends it with the stop file after waiting out the looks in which a retry that should not go would have gone.

The stop file ending such a run reports `model-error`,
not `stop-file`:
a failure stays until a reply succeeds,
so a run stopped while its failure is unanswered has not finished its task.

## Limits of this record

- The failing test name from the original report was never kept,
  so this record cannot say which of the two
  60 s waits failed then.
  It reproduces that shape at 25 percent instead.
- At `CPUQuota=15%` with the host's load average near 80,
  one whole run of the driver's file took 470 s and
  lost twelve tests to an absent `summary.json` (read as `driver-failed`),
  while the same cap at load near 30
  took 276.7 s and passed everything.
  The harness logged
  `logger internal error: file sink (entry 3) verification had not answered within 5000ms`,
  so that run
  measured the test harness being starved,
  not the driver.
- The driver answers the first prompt within `COMMAND_ANSWER_MILLISECONDS` (30 s),
  a shipped limit whose
  comment cites four real runs on an unloaded host.
  A real run on a host at load 100 could exceed it and
  report `prompt-rejected` for a prompt Pi accepted.
  That limit is shipped behaviour with no measurement of
  its own here,
  so it is not changed;
  the tests only stop depending on it.
