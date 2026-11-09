# The wrapper's unit-test watchdog ended tests that were waiting for the disk

`package/git-policy/cli`'s unit tests run one process per test (`cargo nextest`),
and the first fixture a test makes arms a watchdog
(`package/git-policy/cli/src/native/testing/test_support.rs`).
Until 2026-11-09 that watchdog was a wall clock:
it slept 120 s from the moment it was armed and ended the process with
`the unit-test run exceeded 120s: a test is stuck`
(`SIGABRT`),
whatever the test was doing.
The tests that copy a checkout into btrfs subvolumes spend almost all of their time waiting in the kernel,
so the wall clock ended tests that were working,
and which tests it ended followed the host's load.

## Symptom

Two host runs of the library suite on 2026-11-08:

- one ran 4,842 tests,
  4,821 passed,
  18 failed and 3 timed out.
  Every failure was the abort named above,
  and every one of them was in the `subvolume::*` and `worktree::worktree_convert*` families;
  the three timeouts were `subvolume_migrate` tests stopped at the runner's own per-test limit
  (`.config/nextest.toml`,
  `slow-timeout` 60 s times `terminate-after` 5,
  310 s in the log).
  The same families passed 84 of 84 alone in 42 s.
- the other aborted 4 tests with the same line,
  while the wrapper logged a `git index-pack` that showed no sign of work for 50 s.

## What was measured

Method:
`mise run //package/git-policy/cli:native:test:host -- --no-fail-fast subvolume:: worktree::worktree_convert`
in a scope of four processors,
with a scratch sampler reading every task's `stat`,
`io` and `wchan` for the whole process tree every 250 ms
(not part of the repository).
Host:
btrfs on dm-crypt with `compress=zstd:3`,
8 processors,
load average 62 to 107 from other sessions,
the fixture root on the build volume
(`<target>/debug/build/monochromatic-git-policy-cli/test-fixtures`).

- 347 tests matched the two families,
  193 s of wall time,
  exit 0:
  at that load no test reached the wall clock,
  so the run is the shape of the failure without the failure.
  The durations of the 347 add up to 729.5 s across eight test processes.
- Of the 9,050 thread-samples of the test processes themselves,
  2,820 (31.2 percent) were state `D`,
  223 (2.5 percent) runnable and 6,005 (66.4 percent) sleeping.
- The disk waits,
  as shares of those 2,820 samples:
  `wait_log_commit` 52.5 percent,
  `wait_for_commit` 12.6,
  `wait_current_trans` 11.6,
  `btrfs_btree_wait_writeback_range` 7.5,
  `folio_wait_bit_common` 5.5,
  `blk_mq_get_tag` 2.9,
  `btrfs_commit_transaction` 2.4,
  `btrfs_sync_log` 1.4,
  `barrier_all_devices` 1.3,
  and under one percent each for `btrfs_mksubvol`,
  `btrfs_delete_subvolume`,
  `btrfs_tree_read_lock_nested`,
  `read_extent_buffer_pages`,
  `switch_commit_roots` and `kernel_clone`.
- Per test,
  the duration and the share of its life in `D`:

  - `subvolume::subvolume_btrfs_tests::a_real_migration_resumes_after_a_kill_on_each_side_of_the_exchange`
    69.01 s,
    98 percent;
  - `subvolume::subvolume_migrate::tests::a_rerun_after_a_simulated_kill_at_every_step_completes`
    64.00 s,
    94 percent;
  - `subvolume::subvolume_btrfs_tests::a_real_rollback_removes_the_subvolumes_whatever_the_mount_option_is`
    31.80 s,
    98 percent;
  - `subvolume::subvolume_command::ownership_tests::a_second_command_on_one_checkout_is_refused_whatever_its_mode`
    31.14 s,
    99 percent;
  - `subvolume::subvolume_migrate::tests::the_new_checkout_holds_the_index_locks_until_the_indexes_are_rewritten`
    28.86 s,
    96 percent;
  - three more `subvolume_btrfs_tests` at 26.33 s,
    26.19 s and 26.08 s,
    97 to 99 percent;
  - `worktree::worktree_convert::ownership_tests::a_conversion_an_older_release_left_unfinished_is_finished`
    19.95 s,
    88 percent.

- Alone in the same tree,
  the heaviest test took 47 s,
  43.25 of them in `D`,
  and the longest stretch in which it showed no sign of work at all was 0.0 s.
- The longest stretch any process of the family showed nothing
  (no bytes,
  no processor time,
  no task runnable and none in `D`)
  was 8.0 s,
  of one test and the `git` it waited for;
  every test that the disk made slow showed 0.0 s.
- The longest uninterrupted disk wait of one process was 24.5 s,
  in that 69 s test;
  next came 20.0 s,
  20.0 s and 17.0 s.

What the waits are:
the migration writes its journal and flushes the file and its directory at each phase
(`package/git-policy/cli/src/native/worktree/worktree_convert_journal.rs`),
the index rewrite flushes the file it installs
(`package/git-policy/cli/src/native/subvolume/subvolume_index_files.rs`),
and the real-platform controls create,
exchange and destroy subvolumes
(`package/git-policy/cli/src/native/subvolume/fs_btrfs_ioctl.rs`).
Each flush waits for a btrfs log commit and each of those ioctls for a transaction commit,
which is what `wait_log_commit` and `wait_current_trans` name.
A `strace --follow-forks -yy -e trace=fsync` of the heaviest migration control shows 90 flushes:
28 journal files,
10 claim files,
8 index locks and 39 directories.

## Root cause

The watchdog judged the wall clock:

```rust
const RUN_LIMIT: std::time::Duration = std::time::Duration::from_secs(120);

pub(crate) fn arm_watchdog() {
    static ARMED: std::sync::Once = std::sync::Once::new();
    ARMED.call_once(|| {
        let _armed = std::thread::Builder::new().spawn(|| {
            std::thread::sleep(RUN_LIMIT);
            eprintln!("the unit-test run exceeded {RUN_LIMIT:?}: a test is stuck");
            std::process::abort();
        });
    });
}
```

`cargo nextest` runs each test in a process of its own,
so this clock is one test's clock,
and a wall clock cannot tell a test that waits for the disk from one that waits for something that never happens.
A test that is 98 percent disk wait needs 120 s when the disk is about a hundred times slower than in an idle minute,
which is what the two reported runs met.
The three tests that reached the runner's own per-test limit instead are the same family:
a process whose task sits in one uninterrupted disk wait cannot finish its exit until that wait returns,
and the longest single uninterrupted disk wait measured here was 24.5 s.

## The watchdog now

`package/git-policy/cli/src/native/testing/run_progress.rs` reads the signs the wrapper already reads for the child
processes it waits for (`package/git-policy/cli/src/native/push/push_progress.rs`):

- the state of every task of the test process and of every process it started
  (`R` can run,
  `D` waits for the disk),
- their processor ticks,
- the bytes each moved,
- and how many tasks and processes there are,
  so a run that starts or ends an entity counts as work.

A run that shows none of them for `QUIET_WINDOW` ends the process;
a run that is moving is left alone however long it takes.

A look that cannot be read (a task that ended while it was read,
or a process out of file descriptors)
is neither a sign of work nor proof that the run stopped:
the quiet clock keeps running through it,
so a run whose counters never come back ends and does not wait for a reading that never comes,
and a later look that reads the run counts again.
The judge says which of the three endings it reached:
`Quiet` (signs read,
nothing moved),
`Unreadable` (no counters could be read for the window,
though earlier looks read them),
and `Unmeasurable` (no look ever read them,
so the wall clock is the bound).

```rust
pub(crate) fn observe(&mut self, elapsed: Duration, look: Option<Look>) -> Option<Ending> {
    let Some(current): Option<Look> = look else {
        return self.ending_without_a_look(elapsed);
    };
    self.last_read = Some(elapsed);
    let moved: bool = self.previous.is_none_or(|before: Look| {
        return before.bytes != current.bytes || before.ticks != current.ticks || before.members != current.members;
    });
    if moved || current.working {
        self.last_change = elapsed;
    }
    self.previous = Some(current);
    return self.ending_after_the_quiet_stretch(elapsed);
}
```

The window stays at 120 s,
which is fifteen times the longest quiet stretch these families showed (8.0 s)
and four times the longest single disk wait (24.5 s),
and the runner's own per-test limit stays above it.
Where a system counts no task's work,
the window is a fixed deadline as it was before,
and the ending says so instead of claiming the run showed nothing.

The line a stuck test leaves names the figure and the signs:

```text
the unit-test run showed no sign of work for 120.1 s (no task able to run or waiting for the disk, no processor time, no bytes moved, and no process or task started or ended): a test is stuck
```

A test that hangs from its start ends after the window plus up to the runner's own 60-second line:
libtest prints `has been running for over 60 seconds` at 60 s,
which is a write and therefore a sign of work,
and the quiet stretch then begins again.
Measured on a planted stuck test:
`SIGABRT [ 180.905s]` with `no sign of work for 120.1 s`.

## The flushes the controls did not need

With the wall clock gone,
the migration controls still took minutes under load and one of them reached the runner's own 300 s limit
(`TIMEOUT [ 305.127s]`),
so the remaining bound was the next wall clock in line.
The flushes above are not what those controls claim:
a control that simulates a kill in the process loses no page cache,
so its journal and index flushes prove nothing there.

Measured,
the same control twice on each fixture root,
alternating:

- on the btrfs fixture root:
  35.433 s and 32.199 s;
- with `CLI_GIT_TEST_ROOT` on a tmpfs (`/run/user/1000`):
  2.149 s and 2.381 s.

So 94 percent of the test was the flushes.
`Platform::flushes_to_disk` now answers whether a platform's writes are flushed:
`LinuxPlatform` flushes,
and a stand-in platform says no.
`subvolume_command::execute` and `worktree_convert::convert_worktree` install the answer of the platform whose call is
running for that call's whole extent (`subvolume_file_write_steps::WritesKeptInMemory`,
whose first name,
`without_flushing_for_this_thread`,
this record keeps as its original text and which the code renamed
on 2026-11-09):
a stand-in that keeps its writes in the page cache turns the flushes off,
and a platform that flushes turns them back on,
which is what keeps a real-platform call nested inside a control's simulated run durable.
The switch is a thread-local value that goes back however the run ends.
The same control then took 15.4 s.
The rest of its flushes were the claim publishes of
`package/git-policy/cli/src/native/foundation/private_storage.rs`
and of
`package/git-policy/cli/src/native/lock/single_owner_claim.rs`,
which the migration did not reach through the platform.
Those two were measured on 2026-11-09T15:44:00-05:00 and are now the switch's as well:

- one migration command took 79 s at a load average of 119,
  of which `take_claim` was 23.9 s (a record write of 16.2 s and its directory flush of 7.7 s)
  and its release 22.2 s;
- the two index installs took 6.9 s each,
  because a rewrite is a job of the blocking pool and never inherited the calling thread's answer,
  so the run's answer about the disk now travels with the job.

After the change the same claim took 1.0 ms and the same index rewrite 5.0 ms,
the twelve kill points of `subvolume::subvolume_migrate::tests::a_rerun_after_a_simulated_kill_at_every_step_completes`
fell from 384 s to 110 s one after another and 25.2 s side by side,
and the whole `worktree::worktree_convert::ownership_tests` module from 193.1 s to 55.4 s.
What a crash leaves is still the real platform's question:
`LinuxPlatform` flushes,
and only a stand-in that says otherwise keeps its writes in the page cache.

## Verification

- Controls for the judgment,
  the look and the ending (`testing/run_progress.rs`,
  sidecar `run_progress_tests.rs`):
  a run that keeps moving is left alone;
  a run that stops moving is ended at the first look the window covers and the ending counts from the last look that
  moved;
  a task waiting for the disk is work;
  a run whose tasks come and go is left alone;
  a system that counts nothing is bounded by the clock;
  the first look is work;
  the ending names the figure and the signs;
  the look sees the processes the run started;
  and two child-process controls,
  one stuck (the process really ends with `SIGABRT` and the message) and one moving three windows past a one-second
  window (it ends on its own).
  `10 tests run: 10 passed`.
  (The controls of the 2026-11-09 review named below add three;
  the set now runs 13.)
- Controls for the flush switch (`subvolume/subvolume_file_write_steps_tests.rs`,
  4 tests) and for the platform answer
  (`subvolume/subvolume_platform_tests.rs`,
  `the_real_platform_flushes_its_writes_and_a_stand_in_does_not`).
  (The controls of the 2026-11-09 review named below add three;
  the set now runs 7.)
- The guard was shown to fail before it was kept,
  by planting the wall clock back at the top of `Judge::observe`,
  rebuilding and running the controls:
  `10 tests run: 5 passed, 5 failed`,
  with `a_run_that_keeps_moving_is_left_alone` failing
  (`left: Some(Quiet(2s))`),
  `a_run_that_keeps_moving_past_its_window_is_not_ended` failing on the child it starts
  (`the unit-test run showed no sign of work for 1.0 s ...: a test is stuck`,
  in a child that wrote 4 KiB every
  100 ms),
  `a_run_waiting_for_the_disk_is_left_alone` and `a_run_whose_tasks_come_and_go_is_left_alone` failing with
  `left: Some(Quiet(1s))`,
  and `the_first_look_is_work` failing with `left: Some(Quiet(10s))`.
  Restoring the guard returned `10 tests run: 10 passed`.
- A moving run that the old wall clock ended,
  planted as a test that writes 4 KiB every 200 ms:

  - with the wall clock:
    `SIGABRT [ 120.192s]`,
    `the unit-test run exceeded 120s: a test is stuck`;
  - with the judge:
    `PASS [ 236.473s]`.

- A stuck run,
  planted as a test that waits for a message nobody sends:
  `SIGABRT [ 180.905s]`,
  `no sign of work for 120.1 s`,
  and the planted test was removed again.
- The two families and the new controls after both changes:
  `362 tests run: 362 passed (5 slow)`,
  4479 skipped,
  235.327 s,
  no timeout and no abort.
  The heaviest tests of that run were the conversion ones
  (`worktree::worktree_convert::tests::a_kill_at_every_phase_is_finished_by_a_rerun` 168.89 s and three
  `worktree_convert_command::parallel_tests` at 114 to 128 s),
  which still flush through `worktree_copying/worktree_copy_install_log.rs` and through the `Watched` platform of
  those controls,
  and the real-platform control at 43.74 s.
  None of them is near the runner's limit.

## Runs after the changes

- `mise run //package/git-policy/cli:native:test:host -- --no-fail-fast subvolume:: worktree::worktree_convert`
  before the flush change:
  347 tests,
  520 s of wall time,
  every test passed except
  `subvolume_migrate::tests::a_rerun_after_a_simulated_kill_at_every_step_completes`,
  which the runner ended at 305.127 s while the host ran three other sessions' suites;
  that test took 223.20 s,
  234.37 s and 161.81 s in the same family for its siblings,
  against 64 s and less in the run before.
- The same control after the flush change:
  15.423 s.
- The whole library suite,
  `mise run //package/git-policy/cli:native:test:host -- --no-fail-fast`,
  while another session ran the same suite:
  `4831 tests run: 4831 passed (1 slow), 10 skipped` in 197.5 s,
  and again on the last tree of the branch:
  `4831 tests run: 4831 passed (1 slow), 10 skipped` in 215.443 s,
  the same deliberately slow push test.
  The one slow test is `push::push_slow_server_tests::a_push_over_http_waits_past_the_old_low_speed_cutoff_and_lands`,
  which is slow on purpose.
- `mise run //package/git-policy/cli:native:clippy`:
  clean.
  The package's lint first failed on two of its own rules,
  which were fixed rather than suppressed:
  `rust/avoidable-closure` on the closures the two entry points passed to the switch,
  now a `WritesKeptInMemory` guard whose `Drop` puts it back,
  and 13 `markdown/semantic-line-breaks` findings in the new comments,
  fixed by `mise run //:format:monochromatic-lint -- package/git-policy/cli`.
  `mise run //package/git-policy/cli:lint` then passed:
  Clippy clean,
  `monochromatic-lint` 0 warnings and 0 errors,
  Oxlint 0 warnings and 0 errors.

## The cross-vendor review of 2026-11-09 and what it found

A reviewer of another vendor read this branch and found five defects.
All five held,
each was shown by a control that failed before its fix,
and each is fixed or guarded here.

- `Judge::observe` returned before any deadline when a look could not be read
  (`let current: Look = look?`),
  so a test that exhausted its file descriptors and then blocked was never ended.
  It now keeps the clock running through a look that reads nothing,
  and ends as `Unreadable` when the window passes without a reading.
- the first look decided whether the system counts work at all,
  so one unreadable first sample made every later readable look irrelevant and the wall clock ended a working run.
  Whether a look has ever read the system is now `last_read`,
  and a look that reads again counts again.
- a nested call on the real platform inherited a stand-in's skipped flushes,
  because the entry points installed the switch only for a platform that does not flush.
  Both entry points now install the platform's answer for every call,
  so a call that flushes turns the flushes back on for its own extent.
- two watchers of one process kept each other alive:
  `look` left out only the reading thread,
  so the other watcher's `/proc` reads moved the run's byte counter.
  Every watcher now lists its own thread in `WATCHING_THREADS` when it starts,
  and `look` leaves out every thread in it.
- the file-write control proved only that the bytes arrived,
  and a read of a page-cached file does not flush,
  so it stayed green with the durable flush removed.
  `the_switch_decides_whether_a_write_is_flushed` writes to `/dev/null`,
  whose flush the kernel refuses with `EINVAL`,
  so the branch that skips the flush and the branch that makes it give different answers.

The controls fail as the reviewer predicted when their fix is planted out.
Measured on this host on 2026-11-09:

- with the pre-fix judgment:
  `a_run_whose_counters_stop_being_readable_is_ended` panicked with
  `the run ends rather than wait for a reading that never comes`,
  and `a_first_look_that_cannot_be_read_leaves_the_later_ones_to_judge` failed with
  `assertion left == right failed: step 4 moved`,
  `left: Some(Unmeasurable(2s))`,
  `right: None`;
- with the entry points installing the switch only when the platform says no,
  both flush-scope controls failed with `left: Some(false)`,
  `right: Some(true)`;
- with `look` leaving out the reading thread alone,
  `two_watchers_do_not_keep_each_other_alive` failed with
  `assertion left == right failed: a watcher ended the process:`,
  `left: None`,
  `right: Some(6)`,
  after the child ran its whole five seconds;
- with the durable file flush removed,
  `the_switch_decides_whether_a_write_is_flushed` failed with
  `the write made the flush and the kernel refused it: Ok(())`,
  and with the skip branch removed it failed with
  `the switch skipped the flush, so the refusal was never met: Err(Os { code: 22, kind: InvalidInput, message: "Invalid argument" })`.

The judge and flush controls after the fixes:
`20 tests run: 20 passed`
(`run_progress` 13,
`subvolume_file_write_steps` 7;
 the later controls named in the next subsection make it 21).

The moving-run control was then asked the question the review said it could not answer.
With the child's 4 KiB write removed and its sleeps kept,
`a_run_that_keeps_moving_past_its_window_is_not_ended` fails as it must:
`the moving child ended on its own:` and
`the unit-test run showed no sign of work for 1.0 s (no task able to run or waiting for the disk, no processor time, no bytes moved, and no process or task started or ended): a test is stuck`.
Before the fix the same control passed with the write removed,
which is the finding.
The write was restored and the control passed again (`PASS [ 3.031s]`).

What the review did not measure:
no test in this crate was run with an exhausted descriptor table,
so both judge findings are shown by made-up looks and times,
not by a real stuck process without descriptors.

### The later review rounds of 2026-11-09

A second cross-vendor review of the same branch found two more,
and both held.

- `end` wrote its line with `eprintln!`,
  which panics when the write fails,
  so on a stderr that refuses writes (a full disk,
  a closed pipe) only the watcher thread died
  and the test it watched stayed blocked without a watcher.
  The line now goes through `writeln!` on a locked stderr whose error is ignored,
  and the abort follows unconditionally:
  reporting is never a precondition for ending the process.
- `two_watchers_do_not_keep_each_other_alive` passed with the fixture's watcher disconnected,
  because its child arms a one-second watcher of its own,
  which ends the sleeping child either way.
  The child now prints how many watchers it registered,
  and the parent asserts that count beside the ending and the message.

Measured on this host on 2026-11-09:

- `a_stuck_run_ends_its_process_though_its_report_cannot_be_written` runs the stuck child with its stderr on `/dev/full`,
  bounded by `CHILD_DEADLINE` so a child that never ends fails the control instead of hanging it.
  With `eprintln!` it failed after `Summary [  45.040s] 2 tests run: 1 passed, 1 failed` with
  `the watchdog ended the process though its report could not be written`;
  with the fix it passes in 1.911 s;
- with the fixture's `watch(QUIET_WINDOW)` removed from `arm_watchdog`,
  the two-watchers control failed after 6.123 s with `watchers: 1` in the child's output,
  where it passed before that change;
  with the arming restored it passes in 2.374 s.

The watchdog controls after both fixes:
`14 tests run: 14 passed`.

## What does not work

- Raising the wall clock:
  it only moves the load at which a working test dies.
  At load 70 the heaviest test took 69 s,
  and at load 100 or more its siblings took 223 s,
  so no fixed number separates slow from stuck.
- Judging only the test process's own tasks:
  the `git` it runs is where much of the work is
  (one `git` of the family spent 8.25 s of its 8 s life in `D`),
  so a rule that watched only the test's threads would end a test whose Git is working.
  The judge counts the processes the test started.
- Counting the reading thread's own reads:
  they grow the process's `rchar` by about 855 bytes per look,
  so the reading thread is left out (`Look` never includes it),
  and so is every other watcher of the same process (`WATCHING_THREADS`),
  because two watchers would otherwise keep each other alive.
- A per-run processor budget:
  a suite legitimately uses minutes of processor time,
  and a budget no loop could exceed would end working tests.

## What is not claimed

- The two reported runs were not re-run with a sampler;
  their counts are quoted from the report of that day,
  and what was measured here is the same shape at a lower load.
- Why the three tests of the reported run reached 310 s is not measured.
  The mechanism named above (one uninterrupted disk wait holding the exit) is consistent with the 24.5 s longest wait
  measured here,
  but no log of that run shows it.
- The 8.0 s quiet stretch of one test and its `git` child is not explained,
  and it is not what this change is about.
- The claim publishes' flushes are still real for these controls,
  and the real-platform controls keep every flush by design.

## Records this changes

- `doc/troubleshooting/git-policy-cli-landing-controls-wait-on-order-not-time.md` names the old `RUN_LIMIT` in its
  root cause;
  the constant is now `QUIET_WINDOW` and the record carries a note saying so.
- The open question in `doc/handover/cli-git-rust-implementation.md` about the shape of a fix for a control that
  reached the old `RUN_LIMIT` is answered by this record;
  the handover itself is left to its own session.
