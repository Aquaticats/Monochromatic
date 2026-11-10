# Tests of the Git wrapper that pass without their race

## Purpose

This is the record of an audit of the native Git wrapper's tests for controls that prove nothing.
It exists because a reviewer found one such control,
the place control of
`package/git-policy/cli/src/native/testing/parallel_test_support.rs`,
which gave its place back after a fixed wait whether or not the work under test had tried to take one.
The owner's requirement for this work is "Fix flaky tests properly.",
and the rule that decides what counts as proof is `GFP` in `AGENTS.md`:
a guard test proves nothing until it is shown to fail,
by committing it,
removing the guard,
rebuilding,
running,
and restoring.

Nothing was changed by the audit itself.
No production code and no existing test was different after it:
every plant was removed again and `git status --short` was empty after each one.
The findings were acted on afterwards,
in
"What became of each finding (2026-11-09T03:13:07-05:00)" below,
which quotes the failing output of every control that changed.

## What became of each finding (2026-11-09T03:13:07-05:00)

Branch `fix/git-policy-cli-controls-witness-their-race-by-ordering`,
one commit per finding,
each pushed as it landed.
No production behaviour changed:
every control below waits on a fact instead of on a guess at how fast this host is,
and where wall time is still the only witness,
the figure beside it says what was measured here at a load average of 47.
Every plant was removed again before the next one,
and no production file is different after any of them.
One rename stands outside the findings:
`Concurrency`'s lock guard is `locked` and what `wait_timeout_while` hands back is `released`,
because Clippy's `shadow_reuse` refuses the branch while both were `held` (commit `44f610591e`).

### Finding 1: the probe says when it is inside its hold

`package/git-policy/cli/src/native/push/push_progress_tests.rs`,
`a_tree_in_disk_wait_is_progress_at_every_look_until_it_ends`.
Commit `1d7b694f70`.

The probe's `vfork` child now sleeps `ANNOUNCE_AFTER`,
creates the file that `PROBE_ANNOUNCE_VARIABLE` names,
and then sleeps out the hold,
so that file exists only once the calling thread is suspended in the wait.
The control waits for it,
takes one look whose only job is to record the tree's counters,
and then looks four times.
The 700 ms start-up sleep is gone.

The plant is the one this audit used,
`is_working` without the `D` state.

```console
$ mise run //package/git-policy/cli:native:test:host -- a_tree_in_disk_wait_is_progress_at_every_look_until_it_ends
    thread '...a_tree_in_disk_wait_is_progress_at_every_look_until_it_ends' panicked at
      src/native/push/push_progress_tests.rs:529:9:
    look 0: a thread in uninterruptible sleep is alive
     Summary [   0.525s] 1 test run: 0 passed, 1 failed, 4894 skipped
```

Four planted runs of four failed at look 0.
The audit's own plant showed only at look 1,
and its first look passed because it landed in the probe's start-up.

### Finding 2: the burst holds its places until the control has counted it

`package/git-policy/cli/src/native/foundation/blocking_jobs_within_the_budget_tests.rs`,
`no_more_jobs_run_at_once_than_the_budget_allows`.
Commit `dc0f7bc750`.

Each job of the burst now holds its place until the control has read the peak,
and the burst runs on a thread of its own so the control can count it while it runs.
The control waits for the budget to be reached,
which the burst cannot fail to do,
then for a place beyond it,
reads the peak,
and only then lets the jobs go.
A job's place is given back because the control counted it,
never because time passed,
so the fixed 40 ms hold is gone.

The plant is this audit's,
the semaphore's size decoupled from the reported bound.

```console
$ mise run //package/git-policy/cli:native:test:host -- no_more_jobs_run_at_once_than_the_budget_allows
    thread '...no_more_jobs_run_at_once_than_the_budget_allows' panicked at
      src/native/foundation/blocking_jobs_within_the_budget_tests.rs:182:5:
    the burst of 6 jobs held more than the budget of 2 places at once
     Summary [   0.009s] 1 test run: 0 passed, 1 failed, 4894 skipped
```

Four planted runs of four failed the same way,
the slowest in 1.4 s.
Here the audit's plant showed 3 of 6 jobs overlapping inside 40 ms;
with the hold in place the peak is the whole burst's.

### Finding 3: a bound control keeps the rendezvous answer

`src/native/testing/parallel_test_support.rs`,
`worktree/worktree_trash_entries_tests.rs`,
`subvolume/subvolume_index_parallel_tests.rs`,
`worktree/worktree_convert_parallel_tests.rs`
and `landing/service_round_tests.rs`.
Commit `ec2ae30f16`.

Four controls called `point.arrive()` and discarded the answer.
`Rendezvous::arrive_at_the_bound` keeps it,
so work that never brings that many of its jobs together inside the same work fails the control
instead of passing it,
and its patience is `PATIENCE`,
whose own doc states its role:
how long an arrival waits for the others when the control expects them to come.
It bounds how long a failing run waits,
never whether the claim is witnessed.

The plant:
one worker thread for every ordered run,
so no two items can overlap.

```console
$ mise run //package/git-policy/cli:native:test:host -- the_budget_of_the_host_bounds_the_overlap
    thread '<unnamed>' panicked at src/native/testing/parallel_test_support.rs:74:9:
    the work under test never brought 2 of its jobs together inside the same work,
    so the overlap this control measures against a bound of 2 was not reached
     Summary [  17.944s] 4 tests run: 0 passed, 4 failed, 4891 skipped
```

All four failed,
each on its own message.

### Finding 4: a second drain is an event the first one reports

`src/native/landing/service_round_tests.rs`,
`the_directories_of_one_repository_are_drained_one_after_another`,
and `Rendezvous::arrive_and_give_back`.
Commit `60de4af92c`.

The 150 ms sleep in the fixture is gone.
Each drain arrives at a two-place point and gives its arrival back when no second drain comes,
so the count is of the drains inside the work now:
a drain that leaves only after a second one arrived proves the two were inside together,
and a drain that waits alone is the rule holding,
which it proves before the next drain can begin.

The plant is this audit's,
every directory in a lane of its own.

```console
$ mise run //package/git-policy/cli:native:test:host -- the_directories_of_one_repository_are_drained_one_after_another
    thread '...the_directories_of_one_repository_are_drained_one_after_another' panicked at
      src/native/landing/service_round_tests.rs:232:5:
    a drain of one repository began while another was inside its push
      left: 2
     right: 0
     Summary [   0.024s] 1 test run: 0 passed, 1 failed, 4894 skipped
```

Three planted runs of three failed the same way,
the slowest in 0.34 s.

### Finding 5: the two controls that could not fail, and the elapsed ceilings

Commit `710d99ae4a`.

`a_wait_that_has_ended_is_announced_no_more`
offered the watcher nothing but time after the wait ended,
so a host slow enough to starve the watcher passed the control and would have passed it with the
watcher still counting.
It now offers the watcher turns after the end:
each is an end mark it reads in order and that resets nothing,
and the next wait's notice is the proof that every one of them was read before it.
A watcher that went on counting is due a notice on one of those turns however slowly the host runs it.
The plant is `Mark::End` no longer ending the wait.

```console
$ mise run //package/git-policy/cli:native:test:host -- a_wait_that_has_ended_is_announced_no_more
    thread '...a_wait_that_has_ended_is_announced_no_more' panicked at src/native/push/child_watch_tests.rs:205:9:
    assertion `left == right` failed: turn 2 after the wait ended: a finished wait went on being announced: ...
      left: 3
     right: 1
     Summary [   0.197s] 1 test run: 0 passed, 1 failed, 4894 skipped
```

Four planted runs of four failed,
at turn 1 or turn 2.

`the_reporter_prints_nothing_inside_the_grace`
had an hour of grace,
so nothing it could do would make a notice due in its life.
It now watches a grace of 500 ms pass:
the polls timed from after the reporter started are inside half of it on any host,
and the first line must then come,
so neither half rests on a duration the control cannot outlive.
The plant is `Ticker::due_count` ignoring the grace.

```console
$ mise run //package/git-policy/cli:native:test:host -- the_reporter_prints_nothing_inside_the_grace
    thread '...the_reporter_prints_nothing_inside_the_grace' panicked at
      src/native/push/wait_progress_tests.rs:157:13:
    a notice was due 6.92µs into a grace of 500ms
     Summary [   0.006s] 1 test run: 0 passed, 1 failed, 4894 skipped
```

#### The elapsed ceilings of finding 5, one by one

Each is judged by progress where a progress witness exists,
and where none does,
the reason and the measured figure are written beside the value.
Every figure is a single run on this host at a load average of 47,
so none of them is called a spread.

- `push_progress_tests.rs`,
  `a_tree_that_moves_nothing_stalls_at_its_window`:
  the report now has to carry the bytes the meter counted,
  which is progress and not a clock.
  The ceiling stays because the detector itself reads the wall clock,
  and nothing about the tree's progress bounds how late a loaded host wakes the poll loop.
  Measured:
  reported at 1.7 s for a 1.2 s window,
  against a ceiling of 4.2 s.
- `bounded_push_tests.rs`,
  `a_push_that_ignores_the_stop_request_is_killed_after_the_grace`:
  kept as a liveness guard.
  What the child does after it stops answering the request is not a sign of progress the control can read.
  Measured:
  0.66 s for a deadline and grace of 0.6 s,
  against a ceiling of 20 s.
- `lfs_output_mask_tests.rs`,
  `a_long_line_is_masked_in_linear_time`:
  kept.
  The claim is a work rate,
  and a count of what the scan looked at would need a seam in the mask.
  Measured:
  0.08 s,
  against a ceiling of 30 s that a quadratic scan is still minutes away from.
- `service/journald_native_tests.rs`,
  `a_full_journal_stream_drops_the_line_instead_of_waiting`:
  removed.
  The witness is the refusal and the count of lines the stream took before it,
  which is progress;
  the 5 s ceiling could not fire for the writer it named,
  a writer that waited for room,
  because such a writer never returns to the loop that reads the clock.
  A writer that blocked instead of refusing hangs the control,
  which the unit-test watchdog reports and which is owned elsewhere.
- `repository_link_check_walk_tests.rs`,
  `a_stalled_drive_is_answered_within_the_deadline`:
  kept.
  The deadline passing is what ends the wait,
  so a slow host only makes the ceiling easier to hold.
  Measured:
  0.31 s for a 0.3 s deadline,
  against a ceiling of 3 s.
- `binary/binary_commit_timing_tests.rs`,
  `exit_of`,
  and `landing/service_index_wait_tests.rs`,
  the release that wakes a wait:
  unchanged,
  for the reasons above them:
  a poll that fails loudly at its patience,
  and an interval wide enough to separate an event from a poll.

### A sixth control from a cross-vendor review: a name read across a drop

`package/git-policy/cli/src/native/landing/service_claim_renewal_tests.rs`,
`a_running_renewal_renews_the_held_claims_until_it_is_dropped`.
Commit `7c9cf78126`.

The control saved the renewing claim's names while the renewal task was still running every 5 ms,
dropped the renewal,
and compared the names after the drop with that earlier save.
A pass the task made between the save and the drop is its own correct work,
and the drop then waits for it,
so the comparison failed although the renewal and its shutdown both behaved.
The ordering that forces the extra pass is 50 ms between the save and the drop,
ten passes of a 5 ms renewal.

```console
$ mise run //package/git-policy/cli:native:test:host -- a_running_renewal_renews_the_held_claims_until_it_is_dropped
    thread '...a_running_renewal_renews_the_held_claims_until_it_is_dropped' panicked at
      src/native/landing/service_claim_renewal_tests.rs:201:5:
    assertion `left == right` failed: a dropped renewal renames nothing
      left: ["push-0123...~324749247~1794213874~92bf..."]
     right: ["push-0123...~324749198~1794213874~92bf..."]
     Summary [   0.082s] 1 test run: 0 passed, 1 failed, 4894 skipped
```

The two names differ only in the pass counter,
which is the correct pass the control compared against.
The names are now read after the drop,
and what is asserted of them is what a returned task leaves:
one claim,
under a renewal's name.
That the task returned is the assertion that was already there,
and the task's own ending is what says nothing can rename after the line.
Six runs of six passed with the reproduction's pause counted as allowed work,
and the pause is gone with it.

## How the branch's runs stand (2026-11-09T05:00:38-05:00)

The package's two gates and one whole-suite run,
each in a scope of its own
and each written to a file rather than to a pipe,
because a command piped into `tail` exits with `tail`'s status and reports success for anything.

```console
$ mise run //package/git-policy/cli:native:clippy
exit=0  Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.34s

$ mise run //package/git-policy/cli:lint:monochromatic-lint
exit=0  Finished in 50.41s

$ mise run //package/git-policy/cli:native:test:host
exit=100  Summary [ 587.487s] 4883 tests run: 4881 passed (1 slow), 2 timed out, 12 skipped
```

The two timeouts are `testing::run_progress::tests::a_stuck_run_ends_its_process`
and `testing::run_progress::tests::two_watchers_do_not_keep_each_other_alive`,
neither of which this branch's commits touch:
both `run_progress_tests.rs` and the watchdog it drives are byte-identical to the base of this branch,
and neither imports the test support this branch changed.
They are the watchdog's cost,
which another branch owns:
alone on this branch they pass in 1.5 s each,
and `origin/main` already gives the child they start a 45 s deadline
and records this failure mode in `doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`,
so the run to judge them by is the one after main is merged in.

## What was searched for

The tests of `package/git-policy/cli/src/native/`,
including the binary suite (`src/native/binary/binary_tests.rs`,
the `native_binary` test target),
and the end-to-end suite in `e2e/`,
were searched for controls of ordering,
concurrency,
ceilings and recovery that rest on elapsed time:

- `std::thread::sleep` and `tokio::time::sleep` in a control or in its fixture,
- `Duration::from_*` constants used as the only witness of a claim,
- `wait_timeout` or a poll loop whose timeout means "it has had a chance",
- a thread,
  a task or a process started and then assumed to be running,
- an assertion on `Instant::elapsed`.

The scan reached 1638 files under `src/native`,
of which 684 hold tests and 110 call `thread::sleep` at least once,
and the 223 test functions that combine a timing construct with an assertion about something not happening.
Those were read and reduced to the ones below by what the control claims,
not by how much time it spends.

## The host these numbers come from

Every measurement below was taken on this machine as it stands,
6 CPUs,
with other sessions' work on it.
The load average is quoted with each run,
because load is normal here and a control that only works on an idle machine is the defect under audit.
The load average moved between 44 and 94 during the audit.

A sleep probe measured how late this host wakes a sleeping thread,
40 sleeps of 50 ms at a load average of 76:

```console
$ node ${HOME}/temp/agent/pi-audit/lateness.mjs
asked 50 ms, 40 sleeps: median late 0.1 ms, p90 1.2 ms, max 1.2 ms
```

So sleep lateness is not what breaks these controls here.
What changes with host pressure is how long the work takes:
starting a process,
reading a directory,
running a conversion.
Every finding below is about a window made of work,
not of scheduling jitter.

Each control was run alone,
never with its suite,
through the package's own task and in a bounded scope:

```console
$ systemd-run --user --scope --quiet \
    -p CPUQuota=400% -p MemoryMax=16G -p TasksMax=1024 -p IOWeight=10 -- \
    mise run //package/git-policy/cli:native:test:host -- <test name>
```

Nextest ran 1 test and skipped 4834 for each of these runs,
so "one test passed" below always means that one control and nothing else.

## Findings, in rank order

### 1. The disk-wait looks are timed from the probe's start, so the first look can pass for the wrong reason

`package/git-policy/cli/src/native/push/push_progress_tests.rs:427`,
control `a_tree_in_disk_wait_is_progress_at_every_look_until_it_ends`.

What it claims:
a tree whose thread waits for the disk is a sign of life at every look,
with nothing else moving,
and it is idle at last once the hold ends.
The evidence for the first half is the guard in
`package/git-policy/cli/src/native/push/push_progress.rs:167` (`is_working`),
which counts the `D` state as work.
The control starts a probe that holds a thread in uninterruptible sleep for 3000 ms,
sleeps 700 ms "for the probe's start-up" (`push_progress_tests.rs:434`),
then looks 4 times,
400 ms apart (`:436`).

The experiment:
`is_working` was planted to return `state == 'R'`,
so a task waiting for the disk no longer counts as work,
and the control was run.

```console
$ mise run //package/git-policy/cli:native:test:host -- a_tree_in_disk_wait_is_progress_at_every_look_until_it_ends
    thread '...a_tree_in_disk_wait_is_progress_at_every_look_until_it_ends' panicked at
      src/native/push/push_progress_tests.rs:437:9:
    look 1: a thread in uninterruptible sleep is alive
     Summary [   1.559s] 1 test run: 0 passed, 1 failed, 4834 skipped
```

The plant was caught at look 1,
not at look 0:
look 0,
700 ms after the probe started,
found the probe still starting up,
and a process that is able to run is working for the same meter.
So the control passed its first look for a reason that has nothing to do with the guard it names,
and the margin between the plant being caught and being missed is how long the probe's own start-up takes
against 700 ms plus one or two look intervals.
On a host that takes longer to start the probe,
or if the hold were shorter,
all 4 looks can land in the start-up and the guard can be removed with the control still green.
The control is green on this host only because its own fixture was fast enough.

Suggestion,
not a decision:
the probe should announce that it has entered the hold,
with a file it writes or a byte it sends on a pipe just before the `clone` in `hold_in_disk_wait`,
and the control should wait for that announcement instead of sleeping 700 ms.
The first look then happens after a fact,
not after a guess at how fast the machine is.

### 2. The blocking bound is witnessed inside a 40 ms window, and six jobs showed only three at a time under load

`package/git-policy/cli/src/native/foundation/blocking_jobs_within_the_budget_tests.rs:81`,
control `no_more_jobs_run_at_once_than_the_budget_allows`.

What it claims:
no more jobs run at once than the budget allows,
however many are started,
and the bound is reached.
The control holds the budget to 2 places (`BOUND_HELD_TO`),
starts 6 jobs at once (`BURST`),
and each job counts itself in,
sleeps 40 ms (`HELD_MILLIS`),
and counts itself out.

The experiment:
the places were planted to be decoupled from the bound that is reported,
so `budgeted_blocking_jobs()` still answers 2 while the semaphore holds 66
(`src/native/foundation/blocking_jobs_within_the_budget.rs:87`),
and the control was run three times.

```console
$ mise run //package/git-policy/cli:native:test:host -- no_more_jobs_run_at_once_than_the_budget_allows
    assertion `left == right` failed: the burst of 6 jobs held 3 places at once, and the budget is 2
      left: 3
     right: 2
```

All three runs of the planted binary printed `held 3 places at once`,
at load averages of 93.5,
72.9 and 65.3,
in 0.08 s each;
the unplanted control passes in 0.14 s.

So the plant was caught,
and that is the whole of the margin:
with the bound gone,
6 jobs never overlapped,
only 3 did.
The silence of the other three is the machine,
not the code,
and 40 ms is the only reason any of them overlapped at all.
A control whose unfair reading is "the observed peak happened to equal the bound" has no witness left
when the peak is half the burst.

Suggestion,
not a decision:
each job should tell the control that it has started and wait for the control's release,
so the overlap is an event the control drives rather than a race inside 40 ms.
That is also the shape the fix on
`feat/git-policy-cli-tokio-pure-file-sites-on-the-awaited-runner` uses for the place control
(`jobs_waiting_for_a_place`,
commit `51e5958ff6`),
which is not on main.

### 3. Four place controls discard a helper's answer, and the helper is the shape the reviewer found

`package/git-policy/cli/src/native/testing/parallel_test_support.rs:45`,
`Rendezvous::impatient(target, patience)`,
used at `src/native/worktree/worktree_trash_entries_tests.rs:98`,
`src/native/subvolume/subvolume_index_parallel_tests.rs:111`,
`src/native/landing/service_round_tests.rs:134`
and `src/native/worktree/worktree_convert_parallel_tests.rs:238`.

What they claim:
the host's budget bounds the overlap,
the overlap reaches the budget and no more.
In all four the work calls `point.arrive()` and throws the answer away,
so the rendezvous only holds the early jobs for up to `SERIAL_PATIENCE * 10`,
2 s.
The held job gives its place back when that wait ends,
whether or not the target number of jobs ever arrived,
which is the reviewer's defect one step removed:
a place that is given back on a clock.

The experiment:
the helper itself was planted to give every arrival back at once,
`wait_while_short` returning `true` unconditionally,
and its own control was run.

```console
$ mise run //package/git-policy/cli:native:test:host -- one_worker_cannot_pass_a_rendezvous_of_two
    thread '...one_worker_cannot_pass_a_rendezvous_of_two' panicked at
      src/native/foundation/parallel_runner_tests.rs:77:5:
    ["begin 0", "end 0 true", "begin 1", "end 1 true"]
     Summary [   0.011s] 1 test run: 0 passed, 1 failed, 4834 skipped
```

The control for the control fails,
as it must,
which is why the four sites above are not dead controls:
the rendezvous is not their witness.
Their witness is the count of jobs inside the work,
`busiest` against `allowed`,
which the discarding call cannot falsify,
and a plant that removes the bound moves that count
(the plant in the sound list below moved that count to 8 against a bound of 1).

They are still the same code shape that was already wrong once,
and they are fragile for a different reason:
2 s is the longest the early jobs are held,
so the count can only be wrong for 2 s,
and a work path slower than 2 s to start its own jobs would be counted at its own pace.

Suggestion,
not a decision:
either the four sites should keep the answer and assert that every early job arrived,
or the helper should be replaced by one that waits on the jobs' own arrival
and never on a duration,
so that no caller can ignore a clock it did not ask for.

### 4. A lane bound is witnessed through a 150 ms sleep in the work

`package/git-policy/cli/src/native/landing/service_round_tests.rs:193`,
control `the_directories_of_one_repository_are_drained_one_after_another`.

What it claims:
two directories of one repository are drained one after another,
never at once,
while the two lanes of a budget large enough for both run side by side.
The fixture's push sleeps 150 ms (`:214`) so that two handlers would overlap if the lane rule were gone.

The experiment:
`drain_directories` in `package/git-policy/cli/src/native/landing/service_round.rs:234`
was planted to give every directory a lane of its own,
and the control was run.

```console
$ mise run //package/git-policy/cli:native:test:host -- the_directories_of_one_repository_are_drained_one_after_another
    monochromatic-git-policy-cli-service: pushed branch="refs/heads/main" oid=...beef items=1 outcome=done push_ms=150
    thread '...the_directories_of_one_repository_are_drained_one_after_another' panicked at
      src/native/landing/service_round_tests.rs:222:5:
    assertion `left == right` failed
      left: 2
     right: 1
     Summary [   0.178s] 1 test run: 0 passed, 1 failed, 4834 skipped
```

The plant was caught,
and the log line says why:
`push_ms=150`.
The sleep is what made the second handler overlap the first,
so the control's power comes from a fixed duration in its fixture,
not from anything the code under test emits.
The control is honest here because a drain of both directories starts both handlers within 150 ms,
which is generous for a handoff between two threads,
but a slower start is exactly a green run with the lane rule removed.

Suggestion,
not a decision:
the handler should report that it has started on a channel and wait for the control to release it,
so the two handlers are provably both inside the drain,
as the fix on the branch named in finding 2 does for the place control.

### 5. Elapsed-time upper bounds fail only on a slow machine

These are the controls whose assertion is a ceiling on how long something took.
The direction of the error is the opposite of the findings above:
they cannot pass without their behaviour,
they can fail because the machine was slow.
The margins are listed so that a future failure can be read against them.
The width of each bound comes from its own constants,
not from a measurement of run-to-run spread,
so none of them can be called safe or unsafe from this audit alone.

- `package/git-policy/cli/src/native/push/push_progress_tests.rs:204`,
  `a_tree_that_moves_nothing_stalls_at_its_window`:
  at `:219` the reported stall must be under `stall + SAMPLE_INTERVAL * 6`,
  which is 4200 ms for a window of 1200 ms.
  Measured:
  the same control,
  unplanted,
  passed in 1.805 s while its scope held `CPUQuota=10%`,
  one tenth of one core,
  at a load average of 85.7.
  So it survives a tenth of a core.
- `package/git-policy/cli/src/native/push/bounded_push_tests.rs:217`,
  `a_push_that_ignores_the_stop_request_is_killed_after_the_grace`:
  at `:230` the whole attempt must be under 20 s,
  for a deadline of 200 ms and a grace of 400 ms.
- `package/git-policy/cli/src/native/lfs/lfs_output_mask_tests.rs:152`,
  `a_long_line_is_masked_in_linear_time`:
  at `:156` the mask of 2.4 MB of text must be under 30 s.
  The claim is that the scan is linear and not quadratic,
  which is minutes,
  so the bound discriminates even though its value is arbitrary.
- `package/git-policy/cli/src/native/service/journald_native_tests.rs:244`,
  `a_full_journal_stream_drops_the_line_instead_of_waiting`:
  an elapsed bound of 5 s at `:261` on filling a queue and falling back.
- `package/git-policy/cli/src/native/repository_link_check/repository_link_check_walk_tests.rs:55`,
  `a_stalled_drive_is_answered_within_the_deadline`:
  the answer must come within ten times its own 300 ms deadline,
  checked at `:63`.
- `package/git-policy/cli/src/native/binary/binary_commit_timing_tests.rs:130`,
  the helper `exit_of`:
  four controls poll it and every one of them fails loudly at its patience
  (`assert!(began.elapsed() < limit)` at `:136`) instead of hanging,
  which is the good shape for a poll a test needs.
- `package/git-policy/cli/src/native/landing/service_index_wait_tests.rs:335`,
  `a_watch_that_ended_is_set_again_and_the_release_that_follows_is_seen`:
  the release must wake the wait before the 30 s poll interval.
  This one is a property of the wait and not a machine-speed claim,
  and 30 s is wide enough to separate an event from a poll.

Suggestion,
not a decision:
these bounds should keep their wide margin and say beside each what was measured,
as `wait_ceilings.rs` already does for the shipped limits.
A bound that a test cannot fail is not a bound,
and a bound that a slow host can break is a flake,
so the useful figure is the ratio between the margin and the spread.

### 6. Two controls say in their own comment that they cannot fail in the direction they check

These are labelled read,
not experimented.

- `package/git-policy/cli/src/native/push/child_watch_tests.rs:175` (comment above the control at `:141`),
  `a_wait_that_has_ended_is_announced_no_more`:
  the comment says "The check cannot fail on a slow host,
  which only prints nothing,
  while a watcher that goes on counting a finished wait prints at every interval."
  That is true,
  and it means the control's negative half depends on the watcher thread being given a turn:
  a host slow enough to starve it passes the control and would also pass it with the watcher still counting.
- `package/git-policy/cli/src/native/push/wait_progress_tests.rs:143`,
  `the_reporter_prints_nothing_inside_the_grace`:
  the grace here is an hour,
  so no notice can be due in the life of the control,
  and only a report printed synchronously by `notice_due` itself could appear.
  The positive half is the control beside it,
  `the_reporter_prints_at_the_pace_of_its_ticker` at `:161`,
  which drives a 20 ms cadence until three lines are out and is not timed.

## Tests checked and found sound

Each of these was read,
and the reason it is sound is stated.
None was planted,
because the plant for each is a change in a subsystem whose own controls
(the ones in this list) would have to be rebuilt and re-read to tell a caught plant from a broken fixture.

- `package/git-policy/cli/src/native/push/wait_progress_tests.rs`,
  the whole file:
  `Ticker` takes the elapsed time as an argument
  (`a_notice_is_due_after_the_grace_and_at_every_interval` at `:49`),
  so the clock is the input and there is no real time in the claim.
  This is the shape the other findings should move towards.
- `package/git-policy/cli/src/native/push/push_limit_held_for_an_ordering_tests.rs`,
  the whole file:
  `limit_has_passed(elapsed, limit)` takes its clock as an argument,
  for the same reason.
- `package/git-policy/cli/src/native/push/child_watch_tests.rs:141`,
  `a_wait_that_ends_inside_the_grace_prints_nothing`:
  an hour of grace with a watcher that wakes every `SAMPLE_INTERVAL` of 500 ms,
  so a grace check removed altogether makes `until_notice` zero and the notice prints after the shortest sleep.
  The control would then fail,
  which is the right direction.
- `package/git-policy/cli/src/native/service/wait_ceilings.rs`,
  the shipped ceilings:
  constants with their measurements beside them,
  checked as plain arithmetic in
  `wait_ceilings_tests.rs`.
  No clock is read.
- `package/git-policy/cli/src/native/landing/service_index_claim_tests.rs:453`,
  `a_landing_waits_for_the_reclaimer_that_holds_a_claim`,
  and its siblings `service_index_machine_tests.rs:268`,
  `service_index_reclaim_tests.rs:188`:
  each asserts the failure code and that `elapsed >= ceiling`,
  and a wait that ended early fails the second assertion,
  while a slow host only makes it pass more easily.
- `package/git-policy/cli/src/native/push/bounded_query_tests.rs:74`,
  `a_stalled_query_is_stopped_and_reported`:
  the window is 300 ms,
  the assertion is `elapsed >= 300 ms`,
  and the reported seconds must be 0,
  so a clock that never fires fails at the first assertion rather than hanging.
- `package/git-policy/cli/src/native/push/bounded_push_tests.rs:257`,
  `the_grace_is_over_at_its_length_exactly`:
  `grace_remains` is a pure function of two durations.
- `package/git-policy/cli/src/native/service/service_busy_tests.rs:34`,
  `a_step_is_recorded_only_on_a_thread_with_a_board_and_only_while_it_runs`:
  the board's threshold is `Duration::ZERO`,
  so the 20 ms sleep is not what makes the step stuck,
  and a step that is not recorded fails the control.
- `package/git-policy/cli/src/native/foundation/parallel_runner_tests.rs`,
  the rendezvous controls:
  `items_overlap_in_time` at `:61` and `the_report_keeps_input_order_when_completion_order_inverts` at `:83`
  each need every item to arrive,
  and `one_worker_cannot_pass_a_rendezvous_of_two` at `:74` is the control that shows they can fail,
  which experiment 3 above exercised by breaking the helper.
- `package/git-policy/cli/src/native/service/service_queue_received_on_the_runtime_tests.rs:44`,
  `an_empty_queue_times_out_at_its_bound`:
  the returned variant and `elapsed >= SHORT_BOUND` are both checked,
  and the sender in the control beside it is a thread that sends rather than a sleep that stands in for one.
- `package/git-policy/cli/src/native/markdown/markdown_linter_tests.rs:333`,
  `a_linter_that_keeps_working_is_never_abandoned_however_long_it_runs`:
  The fixture works for 15 rounds of 200 ms against a 1200 ms quiet window,
  so the progress rule is exercised 3 times over,
  and the control beside it at `:351` covers the other direction.
- `package/git-policy/cli/src/native/binary/binary_service_heartbeat_tests.rs:332`,
  `a_waiting_command_keeps_its_scratch_directory_from_being_swept`:
  the directories are aged two stale periods before the wait,
  the client touches one of them while the control sends 8 beats,
  and the sweep then asserts that exactly one was removed.
  The witness is the touched directory's mtime,
  not elapsed time.
- `package/git-policy/cli/src/native/worktree/worktree_convert_parallel_tests.rs:260`,
  `the_free_memory_of_the_host_bounds_the_overlap`:
  planted,
  not read:
  `conversion_workers` was changed to take no memory per item
  (`per_item_memory: 1`),
  and the control failed with `left: 8, right: 1` in 76 s at a load average of 84.
  The window here is a whole conversion,
  not a sleep,
  which is why 8 of 8 were seen.

  ```console
  $ mise run //package/git-policy/cli:native:test:host -- the_free_memory_of_the_host_bounds_the_overlap
      assertion `left == right` failed
        left: 8
       right: 1
       Summary [  76.112s] 1 test run: 0 passed, 1 failed, 4834 skipped
  ```
- `package/git-policy/cli/src/native/testing/test_support.rs:103`,
  `settle_fixture`,
  and `:31`,
  `arm_watchdog`:
  a retry loop with a bound and a run watchdog with a bound,
  neither of which is an assertion.
- the end-to-end suite in `e2e/`:
  its waits return a discriminated answer (`'record' | 'settled' | 'timeout'` in
  `e2e/work-record-fixture.ts:117`,
  `'marker' | 'settled' | 'timeout'` in `e2e/barrier-fixture.ts:120`)
  and the scenarios turn a timeout into a failure
  (`e2e/scenario-model-fixture.ts:413` throws on it,
  `e2e/scenario-baseline-editor-fixture.ts:220` requires `arrival !== 'timeout'`).
  Its sleeps are the workload and not the witness:
  `e2e/scenario-helper-fixture.ts:274` staggers concurrent commits,
  `e2e/scenario-lock-gc-fixture.ts:134` holds a foreign lock.
  The one place a timeout is an observation and not a failure is
  `e2e/scenario-service-kill-fixture.ts:239`,
  which records whether the service had claimed a record before the agent settled;
  the invariants are asserted elsewhere in the scenario.

## What this audit does not establish

- No test was found on main that passes with its named behaviour removed,
  among the five plants below.
  Every plant was caught.
  The two controls that are closest to passing without their guard are findings 1 and 2,
  and finding 1 was one look away.
- The elapsed upper bounds in finding 5 were not measured for run-to-run spread,
  so no bound there is called safe.
  A spread measurement needs the same control run many times on an unchanged binary,
  alternating with the planted one,
  which is a campaign and not an audit step.
- The end-to-end suite was read,
  not run:
  it runs only inside its container,
  and no container was started.
- The mutation campaigns are paused by the repository's current instructions,
  so no mutant list was taken.
  Findings 1 to 4 are the candidates such a campaign would confirm.

The plants,
in the order they were run,
each removed again before the next:

1.  `foundation/blocking_jobs_within_the_budget.rs:87`,
    the semaphore's size decoupled from the reported bound.
2.  `push/push_progress.rs:167`,
    `is_working` without the `D` state.
3.  `worktree/worktree_convert_command.rs:157`,
    `conversion_workers` with no memory per item.
4.  `testing/parallel_test_support.rs`,
    `wait_while_short` returning `true` at once.
5.  `landing/service_round.rs:234`,
    every directory in a lane of its own.
