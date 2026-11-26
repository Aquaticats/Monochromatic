# Wrapper controls that wait on a guessed wall clock or on flushes the service defers

Two families of `package/git-policy/cli` controls passed alone and failed inside a larger run on 2026-11-09
(load average about 48 on eight processors,
twelve gigabytes of swap in use,
about twenty agents committing):

- `landing::service_land::adoption_tests::a_socket_that_cannot_start_the_earlier_helper_does_not_hold_the_adoption`,
  `an_open_batch_of_the_earlier_helper_is_finished_by_the_new_service` and
  `the_adoption_says_what_it_did_with_each_batch` reached the runner's per-test bound
  (`TIMEOUT` at 306 to 337 s in a run of 341 unit tests that took 932 s).
  Alone the whole 22-test module passed in 26.7 s.
- `package_repair::a_running_repair_does_not_hold_another_add` failed once at 58.5 s in a run of 36 tests,
  passed alone in 2 s,
  and passed in a rerun of its set.

Both are arrangement defects,
not product waits:
neither control claimed what it measured.
The product's own bound on each of them is below,
and both fixes keep it.

A third family joined them on 2026-11-10
(a library control of the Markdown notice guard,
recorded under "The Markdown notice controls judged the host's scheduler"):
it too rested on a wall clock the host decides,
and it too passed alone.

## The adoption controls measured the disk

The three controls that timed out are the three that finish a batch,
and each lands a setup batch first.
The setup landing ran outside a landing task,
so every flush of a record it wrote was made on the control's own thread,
while the service queues those flushes for the background task
(`package/git-policy/cli/src/native/landing/service_durable.rs`,
"Force at leaving,
else background",
and the control `a_landing_that_publishes_waits_for_no_wrapper_sync`).

Measured on 2026-11-09,
one control under `strace -f -tt -T`,
`landing::service_land::adoption_tests::the_adoption_says_what_it_did_with_each_batch`:

- 79.63 s in all;
- 10 `fsync` calls,
  all on the control's own thread,
  summing 66.358 s:
  5.921 s and 3.252 s for the intent file and its directory,
  5.894 s for the index-lock marker,
  12.913 s and 2.945 s for a fabricated intent of a landing in progress,
  3.545 s and 27.521 s for two more lock markers the adoption took,
  and 3.781 s and 0.560 s for the index it installed.

The same module passed in 0.784 s on the same host later,
which is the same shape as the original report:
the control's time is the disk's,
not the code's.

## The repair control measured its own guesses

`package_repair::a_running_repair_does_not_hold_another_add`
held a fixture `pnpm` for a guessed 15 s and waited a guessed 30 s for that repair to start.
Under the load above the first add reached its repair after more than 30 s,
so the control failed with

```text
# cargo nextest run --test native_binary package_repair
thread 'package_repair::a_running_repair_does_not_hold_another_add' (244830) panicked at src/native/binary/binary_package_repair_tests.rs:250:5:
the first add's repair never started
```

after 45.571 s,
and the five other controls of the same selection were `SLOW` past 60 s and passed at 63.8 to 65.4 s.
The same guessed hold failed the other way as well:
a healthy second add that took longer than 15 s expired the hold,
and the control reported a repository-wide lock that is not there.

## The Markdown notice controls judged the host's scheduler (2026-11-10T15:40:00-05:00)

`markdown::markdown_linter_guard::tests::audit_a_blocked_notice_write_does_not_hold_a_finished_result`
failed in the whole library run on `main` at `b4ed7deb73`:

```text
# mise run //package/git-policy/cli:native:test:host
        FAIL [   1.642s] (3007/6209) monochromatic-git-policy-cli markdown::markdown_linter_guard::tests::audit_a_blocked_notice_write_does_not_hold_a_finished_result
    thread 'markdown::markdown_linter_guard::tests::audit_a_blocked_notice_write_does_not_hold_a_finished_result' (615614) panicked at src/native/markdown/markdown_linter_guard_tests.rs:149:5:
    assertion `left == right` failed
      left: Err(Ceiling(50ms))
     right: Ok(7)
```

The control gave the guard a 50 ms ceiling
and waited `ceiling * 10`,
500 ms,
for the answer,
so its verdict asked the host to start the job's thread inside the ceiling it gave the guard.
`wait_for` (`markdown_linter_guard.rs`) judges the ceiling in its loop and takes the answer after it,
so a host that starts that thread late reaches the ceiling first
and the guard then reports a ceiling for a job that has answered.
The same shape sat in the sibling control,
which waited `ceiling * 10` for a guard that must reach the ceiling.

Measured on 2026-11-10 in a scope of three processors:
with the job's body sleeping 200 ms
(a scratch change,
reverted),
the control failed with those same two lines
(`Err(Ceiling(50ms))` against `Ok(7)`),
and the whole module passed 9 of 9 in 2.5 s without that sleep.

## The notice sink control rested on the notice thread's own wakeups (2026-11-26T16:52:50-05:00)

`commit::native_commit_waiting_notice::tests::a_notice_whose_sink_stopped_reading_does_not_hold_the_commit`
failed in three threaded runs of the whole `commit::` module on 2026-11-26
(`463 passed; 11 failed` and `473 passed; 1 failed` among them),
passed alone three times out of three (0.90 s each),
and passed in its own module twice out of two (19 passed,
5.28 s) (issue 6519).
It panicked at its assertion that the finish answered within its two-second window.

That window held on this host,
where `commit::` at the default threads passed (`474 passed; 0 failed`,
24.44 s).
At 24 test threads the same module failed (`473 passed; 1 failed`,
20.69 s)
on the control's other assertion,
and the record named what was missing:

```text
the record says a line was dropped because the reader was not taking it:
ScratchStepFacts { ceiling: Some(60s), armed: true, notice: 500.062751ms, wrote: 0ns,
lines_dropped: 0, looks: 3, looks_found: 0, ... }
```

### What was measured

A probe in the control
(a scratch change,
reverted)
recorded how its two-second window was spent,
over six runs:

- `commit::` at the default threads:
the finish answered in 500.212 ms,
and the thread that ran it started 0.075 ms after the window opened;
- its module:
501.622 ms,
the thread starting 0.985 ms after the window opened;
- alone,
`--exact`,
three runs:
500.210,
  500.198 and 500.360 ms,
the thread starting 0.072,
  0.080 and 0.205 ms after the window opened.

`finish` waits no longer than twice the notice's own bound (`NOTICE_BOUND`,
500 ms),
once for the notice's answer and once for the line it handed over,
so the product's own maximum is one second
and the two-second window was twice that and nothing more.

### Root cause

Both verdicts of that control rested on wall clocks the host decides,
and neither was a promise of the notice.

The first was the arrangement:
the control slept three cadences and expected the notice thread to have offered three lines by then,
so that the third was the one a reader that stopped reading costs.
The notice hands over one line per look,
and the pause between its looks is the cadence's interval,
so the third line needed the third look after the first.
At 24 threads the notice thread made three looks in the 400 ms the control gave it (`looks: 3`),
the third line was never offered,
and the record counted no drop.
Nothing outside the notice says that a line was dropped,
so the control could not wait for that event and waited on the notice thread's scheduling instead.

The second was the window over the thread that calls `finish`:
it covered a thread start and the one second of bounded waits `finish` owes,
which leaves a second and a half on a quiet host and nothing on a host that wakes a thread late.

## The fixes

The adoption controls' setup batch now lands inside a landing task,
and the intents they fabricate are written the way a landing writes them
(`earlier_service_journal_adopted_tests.rs`,
`write_as_a_landing_in_progress`),
so their flushes are queued for the background task exactly as the service queues them.
A control fails if one is ever waited for again:

```text
# cargo nextest run --lib the_batch_these_controls_leave_open_is_landed_without_waiting_for_a_flush
the setup landing waited for a flush: ["sync-waited:intent-file", "sync-waited:intent-dir", "sync-waited:lock-marker"]
```

The repair control's hold now ends at the release the control writes,
at the end of its fixture,
or with the control process (`while_test_lives`),
and it can end no later than the repair's own quiet window
(`REPAIR_TOOL_CEILING` in `package/git-policy/cli/src/native/service/wait_ceilings.rs`).
The wait for the repair to start ends at the event:
the repair starting,
or the first add ending,
which is reported with the first add's own output:

```text
# the fixture pnpm exiting before it marks the repair started
the first add ended (exit status: 0) before its repair started, so its worktree was never repaired: monochromatic-git-policy-cli: copied 3 ignored filesystem entries ...
# the fixture pnpm marking it started and ending at once
the first add ended before the second returned, so the second waited for it: monochromatic-git-policy-cli: copied 3 ignored filesystem entries ...
```

One backstop is left,
200 s,
and it names what it waits for
("the first add's repair had not started after 200 s and the first add was still running,
so the add is stuck before pnpm").
It is below the runner's per-test bound (`.config/nextest.toml`,
60 s times 5),
so a stuck repair is a named failure and not a timeout,
and it is not what decides a healthy run:
the wait ends at the event,
however slow the host is.

The finished-result Markdown notice control now gives the guard a ceiling no wait reaches
(`ROOMY`,
3,600 s),
because the ceiling is not what it is about;
its sibling keeps the 50 ms ceiling,
which is its subject.
Both wait `ANSWER_WITHIN` (60 s) for the guard's answer
while a notice write is held for `WRITE_HELD_FOR` (240 s),
so only a guard that waits for the held write reaches the wait,
and the write ending can never be the answer such a guard owes.
A broken guard is still found in 60 s,
under the runner's per-test bound.

The notice sink control now reaches the state it is about itself:
it offers one line into the queue the writer takes its lines from,
through the sender the notice already hands the wait that owns the step (`lines_for_the_wait`),
so the writer holds a line while the queue cannot hold another,
and the notice's own look then decides only which line is dropped and never whether that state exists.
It waits one `LOOK_INTERVAL` for that look,
the longest the notice goes between two,
and its window over the finish is thirty times `NOTICE_BOUND`:
thirty times the measured answer,
and fifteen times what `finish` may honestly wait.
Its cadence is short (20 ms) as well,
so every line a reader that stopped reading costs is offered within the first few looks.

## Verification

- The control the issue named,
  after the change:
  its module `19 passed; 0 failed` in 5.28 s,
  alone `1 passed; 0 failed` in 0.77 s,
  `commit::` at the default threads `474 passed; 0 failed` in 25.00 s,
  and `commit::` at 24 test threads `474 passed; 0 failed` in 20.76 s.
- The same control shown failing without its arrangement:
  with its file at its parent's version,
  `commit::` at 24 test threads failed twice of two
  (`473 passed; 1 failed` in 20.73 s and 20.65 s,
  `lines_dropped: 0`,
  `looks: 3`),
  and it passed again once the file was restored.
- The Markdown notice controls' constants:
  `markdown_linter_guard::`,
  `Summary [2.459s] 9 tests run: 9 passed, 6218 skipped`.
- `mise run //package/git-policy/cli:native:test:host -- adoption_tests`:
  23 of 23 pass,
  0.784 s (`Summary [0.784s] 23 tests run: 23 passed`).
- `mise run //package/git-policy/cli:native:test:binary:host -- package_repair`:
  11 of 11 pass,
  8.830 s (`Summary [8.830s] 11 tests run: 11 passed`).
- The same traced control after the change:
  0.61 s,
  7 `fsync` calls of 1.081 s in all,
  of which 2 are on the control's own thread (0.260 s and 0.006 s) and 5 on the background task.
- Both new guards were shown to fail before they were kept:
  the flush control above with the landing task removed,
  and the repair control with its fixture `pnpm` exiting before the repair marks itself started,
  then with it marking itself started and ending at once
  (the two quoted messages).
- Both Markdown notice controls were shown failing with
  `collect_returned_sink`'s `recv` in place of `recv_timeout(NOTICE_POLL)`
  (a guard that waits for the write),
  and each named its own subject:

```text
# mise run //package/git-policy/cli:native:test:host -- markdown_linter_guard::tests::audit_a_blocked_notice_write
a notice write must not hold a finished result: Err(Timeout)
```

## What is not claimed

- The failing run's own stall was not measured:
  no trace of that host names the microseconds the job's thread lost,
  and the reproduced failure stands in for it.
  What both show is the control's verdict resting on scheduling,
  not anything the guard did differently.
- The guard still judges the ceiling before it takes an answer that has already arrived.
  Reading such an answer first is a product question,
  not what these controls measured,
  and no control here asks for it.
- No product wait was changed.
  The service already queued the landing's record flushes,
  and the recovery still makes its own on the thread that asks
  (`service_durable_tests.rs`,
  "a flush outside a landing is made now,
  as recovery needs it").
  That is the 2 flushes of 0.266 s left on the control's thread,
  and a disk that stalls those for minutes would still stretch a control that finishes a batch.
  Deferring them is a durability decision for the owner,
  not a test repair.
- The runner's per-test bound is unchanged.
  Raising it would have hidden both defects:
  the adoption controls would still have measured the disk,
  and the repair control would still have reported the guess.
- The module times above were taken at a load average of 8 to 48 on this host,
  not under the load of the failing run.
  What carries over is the attribution (the flushes of the control's own thread,
  and the guessed bounds),
  not the seconds.
- The window failure the issue records was not reproduced here:
  `commit::` at the default threads passed on this host (`474 passed; 0 failed`),
  so the window's size rests on the measured answer above and on the three recorded runs,
  not on a red seen here.
  The red seen here is the control's other assertion,
  which the same arrangement now reaches
  without waiting on the notice thread's wakeups.
- The wider window is a weaker guard than two seconds was:
  a `finish` that waited for the reader for ten seconds would pass it.
  Nothing tighter can hold under load,
  because the notice's own promise is a wall clock as well,
  so the window stands above the product's maximum by a stated multiple and the reader's stall stays far longer.
- No product code changed:
  the control reached the writer's queue through the sender `lines_for_the_wait` already hands a wait,
  and `finish`,
  `NOTICE_BOUND` and the line queue are as they were.

## Records this changes

`doc/handover/cli-git-rust-implementation.md` listed both controls as open under `AGENTS.md` `SUF`;
that entry is gone with this record,
which holds the measured cause and the two arrangement fixes.
The Markdown notice controls were added to this record on 2026-11-10;
that handover was not the place the coordinator kept them,
and the failing run's own log names the control.
