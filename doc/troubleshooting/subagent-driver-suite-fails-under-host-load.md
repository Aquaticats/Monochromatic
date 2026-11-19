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

The stop file ending such a run reports `model-error`, not `stop-file`:
a failure stays until a reply succeeds,
so a run stopped while its failure is unanswered has not finished its task.

## A session note that went missing, seen once in the suite on 2026-11-19

A review of the temporary-folder change ran the whole package suite once and saw one test fail that
this record did not name
(`keep/unit-suite-at-tip.log` in
`/home/user/temp/agent/review-of-the-temporary-folder-a-run-gives-pi-2026-11-19/`):

```text
[drive-pi-subagent] [leaves a session alone that Pi reports and this run did not make]
[FAIL] (3.6s) AssertionError: expected 'undefined' to include 'other-run.jsonl'
@drive-pi-subagent.unit.test.ts:2254 expect(String(noted?.problem,),).toContain('other-run.jsonl',);
```

The test is named here and nowhere by its line,
because the lines move and the name does not:
it is `leaves a session alone that Pi reports and this run did not make`
in `src/drive-pi-subagent.unit.test.ts`.

The test resumes a session and has its stand-in Pi report another run's session file,
which a run must refuse and note as `session_at_rest` with the file's name.
The failing line means no such note was written,
which happens when the run decided Pi named no file at all and masked its own copy,
so the answers that should have named the reported file did not.
That mechanism is **inferred** from the source and from what the test needs,
not observed: no answer of the stand-in was kept from the failing run.

### What each run answered

The same test file was run four times on 2026-11-19 away from the suite,
twice at the branch's tip and twice with `src/supervise-pi-run.ts` put back to `main` in a scratch copy
of the package:

- Tip run one and tip run two:
  the whole file passed,
  no `[FAIL]` line
  (`keep/session-flake-tip-run-1.log` and `keep/session-flake-tip-run-2.log`).
- The control's two runs:
  the only failing test was the new folder test,
  which is what `main`'s supervisor is expected to fail,
  and the session test passed in both
  (`keep/session-flake-control-run-1.log` and `keep/session-flake-control-run-2.log`).

The review measured the same test green six times alone at the tip and three times at `main`,
and measured one test taking 3.6 s inside the suite against 1.6 s for the two tests alone,
so the comparison is one test inside the suite against two tests alone,
not a rate for one test in either place.

The test did not reproduce in the four runs above,
nor in the review's nine runs outside the suite.
The one failing run was inside the suite,
and what is recorded here is that the same test can fail there while passing alone,
which is the contention this record names rather than a defect a run can be pointed at yet.
A suite run that fails on this test alone is this record's subject,
and a suite run that fails on it should keep the failing run's feed and stand-in log before anything
is concluded.

## A store read that is not JSON, seen in the suite on 2026-11-19

Three tests of the package's suite failed this way on 2026-11-19,
each of them building its store with `createSopsFixture`:

```text
SecretEnvironmentUnreadableError: the .env.local.json of … holds values of this machine's secret environment and cannot be read,
so no value of it could be masked before it reached the model, and no run was started:
.env.local.json is not the JSON a sops store is (the read failed with SyntaxError)
```

The three:

- `reads the store, Pi own files, the shell start-up file and the environment, naming each source`
  and `arms an OAuth credential Pi own schema names, a shell value with a comment after it, and a key that begins with a
  slash`,
  both children of `src/secret-values-of-this-machine.unit.test.ts`,
  at the tip of the temporary-folder branch (`keep/full-suite-at-the-branch-head-b0324ec24b.log`)
  and once at `40e906d6de`, an ancestor of `main`
  (`keep/full-suite-at-base-commit-40e906d6de.log`),
  while a later run at that same base passed
  (`keep/full-suite-at-base-with-the-store-probe.log`);
- `refuses to start a run whose secret environment cannot be read, naming the cause and starting no Pi`
  in `src/drive-pi-subagent.unit.test.ts`,
  in the review's one run of the suite at `9f815bc830`
  (`keep/unit-suite.log` in `/home/user/temp/agent/review-answer-at-9f815bc830/`).

Every one of them reads a store the fixture made,
and the failure is the store's own text not being JSON,
so what is short is the file the fixture wrote,
not anything the run did with it.

### The mechanism, inferred

`createSopsFixture` writes its store from what its own `runCommand` captured,
and `runCommand` awaits the child's `exit` event and then joins the chunks it has
(`package/env-local-json/core/src/fixture.ts:215-234`).
Node may still deliver a child's standard output after `exit`,
so a store built from what had arrived can be short,
and a short store does not parse.

That this failure's text is what a cut store gives,
and that the wait for `exit` is where the cut happens, was measured by the env-local-json fixture review of 2026-11-18:
four of eighty waits for `exit` captured nothing,
where eighty of eighty waits for `close` captured the whole document
(`~/temp/agent/env-local-json-fixture-waits-for-close-review-2026-11-18/prompt.md`,
and the entry naming branch `74dbe1200e` under `doc/handover/cli-git-rust-implementation.md`).
Branch `74dbe1200e` changes the wait to `close`,
and that change is not merged here.

For the three failures above the mechanism is **inferred**, not measured:
this branch's probes of the same shape did not reproduce a lost chunk,
and the review's probe did not either.
Nothing in the driver's store reader is implicated:
it parses the file it was given, and the file was short.
A suite run that fails this way should keep the fixture folder the message names before anything is concluded.

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
