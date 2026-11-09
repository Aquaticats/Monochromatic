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

```rust
pub(crate) fn observe(&mut self, elapsed: Duration, look: Option<Look>) -> Option<Ending> {
    if self.measured.is_none() {
        self.measured = Some(look.is_some());
    }
    if self.measured == Some(false) {
        return if elapsed >= self.window { Some(Ending::Unmeasurable(elapsed)) } else { None };
    }
    let current: Look = look?;
    let moved: bool = self.previous.is_none_or(|before: Look| {
        return before.bytes != current.bytes || before.ticks != current.ticks || before.members != current.members;
    });
    if moved || current.working {
        self.last_change = elapsed;
    }
    self.previous = Some(current);
    let quiet: Duration = elapsed.saturating_sub(self.last_change);
    return if quiet >= self.window { Some(Ending::Quiet(quiet)) } else { None };
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
`subvolume_command::execute` and `worktree_convert::convert_worktree` keep the calling thread's writes in the page cache
for the run when their platform says no (`subvolume_file_write_steps::without_flushing_for_this_thread`),
which is a thread-local switch that goes back however the run ends.
The same control then took 15.4 s.
The rest of its flushes are the claim publishes of
`package/git-policy/cli/src/native/foundation/private_storage.rs`,
which the migration does not reach through the platform,
and they are left as they are.

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
- Controls for the flush switch (`subvolume/subvolume_file_write_steps_tests.rs`,
  4 tests) and for the platform answer
  (`subvolume/subvolume_platform_tests.rs`,
  `the_real_platform_flushes_its_writes_and_a_stand_in_does_not`).
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
  so the reading thread is left out (`Look` never includes it).
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
