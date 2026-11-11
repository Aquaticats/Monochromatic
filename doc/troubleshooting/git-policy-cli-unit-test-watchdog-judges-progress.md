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

Since 2026-11-11 the line ends `and no process or task started`:
a figure that fell is a task that ended or one the look could not read,
and neither is the run working,
so only a figure that grew counts (`Judge::observe`).

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
Those two were measured on 2026-11-09T15:44:00-05:00 and are the switch's as well now:

- one migration command took 79 s at a load average of 119,
  of which `take_claim` was 23.9 s (a record write of 16.2 s and its directory flush of 7.7 s)
  and its release 22.2 s;
- the two index installs took 6.9 s each,
  because a rewrite is a job of the blocking pool and never inherited the calling thread's answer,
  so the run's answer about the disk now travels with the job.

After the change the same claim took 1.0 ms and the same index rewrite 5.0 ms,
the twelve kill points of
`subvolume::subvolume_migrate::tests::a_rerun_after_a_simulated_kill_at_every_step_completes`
fell from 384 s to 110 s one after another and 25.2 s side by side,
and the whole `worktree::worktree_convert::ownership_tests` module from 193.1 s to 55.4 s.
What a crash leaves is still the real platform's question:
`LinuxPlatform` flushes,
and only a stand-in that says otherwise keeps its writes in the page cache.

## The flushes two worktree-copy controls did not need (2026-11-10)

Two controls of `package/git-policy/cli/src/native/worktree_copying/worktree_copy_lifecycle_tests.rs`,
`a_creation_recovers_a_dead_journal_first` and
`a_refused_add_leaves_no_record_for_the_worktree_another_command_completed`,
were ended by the runner's per-test limit at 300 s in a 344-test run and passed alone.
What the time went to was unmeasured then;
this is the measurement.

### Symptom

`nextest` ends a test after `slow-timeout` 60 s times `terminate-after` 5
(`package/git-policy/cli/.config/nextest.toml`),
which reads as `TIMEOUT [ 305.127s]`.
The first control took 0.7 s to 27.3 s alone before this change,
and the second passed alone in 0.67 s.

### What was measured

On this host (btrfs on dm-crypt with `compress=zstd:3`,
the fixture root under the build volume,
load average 20 to 70 from other sessions),
2026-11-10:

- `strace -f -c -w -e trace=fsync` around the control alone:
  35 flushes and 0.67 s of a 1.36 s run for the first control,
  51 flushes and 0.32 s of a 0.67 s run for the second.
- `strace -f -y -e trace=fsync` shows where they go:
  15 of the 35 flush the journal root directory (`<git dir>/monochromatic-git-policy-cli-worktree-copy/v1`),
  the rest are the journal's and the claims' record files,
  4 are the install log,
  and 1 is a registration's owner file.
- With the same control on a tmpfs fixture root (`CLI_GIT_TEST_ROOT=/run/user/1000/...`),
  the same 35 flushes took 0.47 ms in total and the run took 0.31 s:
  the work is not what costs the time.
- A probe writing what the journal writes (one 35-byte record,
  the file flush,
  then the directory flush)
  every 200 ms on the fixture root,
  sampling the flushing task's `wchan`:
  at a load average of 20 to 33 the record flush was p50 6.7 ms,
  p95 640 ms and max 3,863 ms in one minute,
  and p50 17.6 ms,
  p95 3,996 ms and max 20,038 ms in the next;
  the waits named were
  `btrfs_btree_wait_writeback_range` (500 samples),
  `lock_extent_buffer_for_io` (247),
  `read_extent_buffer_pages` (63),
  `btrfs_commit_transaction` (51),
  `handle_reserve_ticket` (49),
  `folio_wait_bit_common` (33),
  `barrier_all_devices` (13) and `blk_mq_get_tag` (11).

So 35 to 51 commits at this disk's worst minutes is the 300 s:
8.6 s per flush at the limit is inside the measured 20.0 s maximum.

### Root cause

A `git worktree add` through the wrapper publishes a stage claim,
a journal,
an install log and a registration's
owner file,
and removes the journal when the copy completes,
and each of those publishes and removals flushes its file and its directory
(`worktree_copy_journal.rs:411`,
`worktree_copy_journal_removal.rs:111`,
`worktree_copy_stage_claim.rs:124,235,295`,
`worktree_copy_install_log.rs:257`,
`private_storage.rs`).
The control checks nothing about what a crash leaves,
so none of those flushes proves anything there,
and a filesystem commit is what every other process of the machine is waiting for at the same time.

### The controls now

The three controls of that file whose subject is neither a crash nor the flush itself call
`keep_the_durability_writes_in_the_page_cache()`,
which is `private_storage::skip_the_durability_syncs_here()`:
the answer the migration and conversion controls already give `Platform::flushes_to_disk`.
Two writers keep a handle of their own,
the install log and a registration's owner file,
so they flush through `private_storage::flush_file_when_faithful`,
which reads the same answer.
Every sync is still recorded (`record_file_sync`,
`record_directory_sync`),
so a control whose subject is the order of the flushes reads them as before.

The first control then made no flush at all and ran in 0.22 s,
the second in 0.26 s on the same host,
and 0.092 s and 0.095 s inside the whole library suite.

### Verification

- The new control `a_settlement_whose_control_checks_no_crash_keeps_its_writes_in_the_page_cache`
  reads the process's answer in the middle of a settlement and reads the recorded syncs after it.
  Before the change it failed with
  `assertion `left == right` failed: the settlement's durability writes stay in the page cache for a control that
  checks nothing about a crash: left: [true], right: [false]`,
  and after it passes.
- `strace -f -c -w -e trace=fsync` then reported no `fsync` line for either control,
  where it had reported 35 and 51.
- The 464 tests of `worktree_copying::`,
  `private_storage`,
  `install_log`,
  `worktree_add_binding`,
  `worktree_ladder`,
  `conclusion_copy`,
  `merge_isolation_journal` and `shadow_refs`:
  `464 tests run: 464 passed` in 12.4 s,
  which is the same set that reads the sync records.
- The whole library suite (`mise run //package/git-policy/cli:native:test:host -- --no-fail-fast`),
  on the tree of `8e6e326c8e`:
  `6080 tests run: 6080 passed (1 slow), 14 skipped` in 156.762 s,
  the slow one the push control that is slow on purpose
  (`push::push_slow_server_tests::a_push_over_http_waits_past_the_old_low_speed_cutoff_and_lands`),
  run while another session was using the host.
  The pipe control below was added after that run;
  its own run and the 405 tests of `private_storage`,
  `worktree_copying::`,
  `worktree_add_binding` and `install_log`
  (`405 tests run: 405 passed`) are the runs it had.
- `cargo build --bin monochromatic-git-policy-cli` (`native:build`),
  which compiles the `#[cfg(not(test))]` arm where `syncing_is_faithful` is the constant `true`.
- The two branches of `private_storage::flush_file_when_faithful` are told apart by the flush itself,
  not by its
  record:
  `the_durability_answer_decides_whether_a_file_is_flushed` flushes a pipe's write end,
  which `fsync` refuses with `EINVAL` (`Err(Os { code: 22, kind: InvalidInput, ... })`),
  and asserts the refusal is an error while the answer is faithful and no error once the answer skips it.
  With the helper's skip branch removed (`if false && !syncing_is_faithful()`),
  the control failed with `the answer skips the flush the pipe refuses: Err(Os { code: 22, ... })`;
  restored,
  it passes.
  That closes the reading a reviewer of another vendor made of an earlier tree:
  a record cannot tell a kept flush from one that happened.
  The install log's flush of a torn tail (`worktree_copy_install_log.rs:253`) is still not exercised under the skip
  answer,
  because the controls that open a log always open a whole one.

### The whole library suite on this host

`mise exec -- cargo nextest run --locked --offline --lib --no-fail-fast` in one 600% CPU,
12 GiB cgroup took 1311 s and printed
`6607 tests run: 6603 passed (10 slow), 3 failed, 1 timed out, 19 skipped`.

Four problems came out of it.

- `testing::run_progress::tests::a_stuck_run_ends_its_process` hit nextest's `300.010 s`
  timeout,
  because it waited for its child with `run_child`,
  which had no bound at all.
  Every control's child is now waited for inside `CHILD_DEADLINE`
  (both `run_child` callers use `run_child_with_stderr_until`).
  With that child arming no watcher the control fails after `45.123 s` with
  `the watchdog ended the child that stopped moving`,
  where the same runaway child held the test to the runner's limit before.
- `a_stuck_run_ends_its_process_though_its_report_cannot_be_written` failed at 45.071 s
  and `a_wait_does_not_keep_the_watchdog_alive` at 45.068 s:
  both waited out `CHILD_DEADLINE`,
  so no watcher ended those children in 45 s.
  The same controls pass alone (1.61 s each),
  in the `testing::` family (`31 tests run: 31 passed`),
  and beside the `subvolume::` family at 2 CPUs and at 6 CPUs,
  where six traced children all ended at 1.5 s with `working: false` and one unchanged figure.
  What held them in that one run is not measured.
  A look that is never taken cannot end anything,
  so a child the host never schedules is one candidate;
  the accounting change cannot be the cause,
  because it removes one term from the figure judged and leaves every other read where it
  was,
  so a run can only be judged quiet sooner than before.
- `repair::package_repair_prewarm::tests::a_worktree_added_again_at_the_same_path_gets_its_own_filling`
  failed in 0.028 s and belongs to another family.

### What is not claimed

- The switch is a control's process,
  not its test:
  a run that puts several tests in one process (libtest,
  as an `strace` probe does)
  carries it from one test to the next.
  Every task of this package runs the binary through `nextest`,
  which is one test per process.
- The records of the syncs are per thread (`private_storage_syncs_test_support.rs`),
  so a sync site that moved onto a worker thread would move its record with it and the record assertions here would
  have to follow.
- The flush count after the change is measured by `strace`;
  no control counts them,
  because nothing in the process sees a flush that happened.
  The records are what the controls read,
  and the new control asserts the records of all three writers that keep one (the journal root directory,
  the install log,
  and the registration's owner file).

### What does not work

- Raising `slow-timeout` or `terminate-after`:
  it moves the load at which a working control is ended,
  which is the argument this record already makes against a wall clock.
- Running the controls against a tmpfs fixture root:
  it hides the cost instead of removing it,
  and the fixture root stays on the build volume so a control sees the file system the shipped code meets.
- Leaving the flushes to the controls that keep them:
  with the page-cache switch in place but the install log and the registration's owner file flushing through their
  own handles,
  the first control still took 87.7 s and 42.0 s in two of five runs,
  of which its five remaining flushes were 73.6 s and 39.8 s.

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

## The two-watchers control in the alpha.10 gate (2026-11-11)

### Symptom

The wrapper's 0.6.1-alpha.10 release gate failed one test of 7574.
The unit is `gpc-gate-wrapper-alpha-10-on-b96767962c-20261111T053330-0500`:
the suite in a 2-CPU container of 2 GiB beside other sessions' builds.

```text
FAIL [   5.214s] (5603/7574) monochromatic-git-policy-cli testing::run_progress::tests::two_watchers_do_not_keep_each_other_alive
thread 'testing::run_progress::tests::two_watchers_do_not_keep_each_other_alive' (89663) panicked at src/native/testing/run_progress_tests.rs:386:5:
assertion `left == right` failed: a watcher ended the process:

  left: None
 right: Some(6)
```

That quoted line is `run_progress_tests.rs:386` of the gate's own commit;
on this branch the same assertion sits at `run_progress_tests.rs:455`.

`left: None` is a child that ended with no signal,
so the control's first two assertions had already passed:
the child had printed `watchers: 2`
and then slept its five seconds to the end,
and neither of its two watchers ended it in time.

### What was measured

A standalone probe repeated the same reads:
a thread read its own `/proc/self/task/<tid>/stat` and then `io` every 500 ms,
and took the second figure off the first as the look did.
Its judged figure moved 4 times in 20 looks of a thread doing nothing,
every move exactly one byte,
and every move at a look where that stat line's own length changed:
330 to 331 bytes,
with `minflt` and `utime` gaining a digit.

The arithmetic says why.
Every file a look reads has its length taken off the next look
(`Tally::read_bytes` and `note_a_reader_s_own_readings`),
so the figure a look judged was `R0 + s_k + sum(R_j - a_j)`,
where `s_k` is the length of the caller's own stat line.
That line was read before the `io` read that measures the caller,
so it was the one length that never cancelled.
`R_j - a_j` measured 0:
an admitted read's length equals the bytes the read returned.

The real child ran ten times on this host,
in a 2-CPU cgroup capped at 3 GiB beside the package's own `subvolume::` family,
and never under synthetic load.
A trace of every look named each ending:

```text
abort times: 1.5 1.5 2.0 2.0 2.0 2.5 2.5 2.5 2.7 s
the tenth run exited normally after 5.0 s
```

A passing run's trace,
cut to the figures that matter:

```text
watch tid=2005277 window_ms=1000 own_stat_len=332
look tid=2005277 ms=500 taken=Some(Look { bytes: 8738, ticks: 0, working: false, members: 2 }) own_stat_len=333
look tid=2005277 ms=1001 taken=Some(Look { bytes: 8739, ticks: 0, working: false, members: 2 }) own_stat_len=333
look tid=2005277 ms=1501 taken=Some(Look { bytes: 8739, ticks: 0, working: false, members: 2 }) own_stat_len=333
look tid=2005277 ms=2001 taken=Some(Look { bytes: 8739, ticks: 0, working: false, members: 2 }) ending=Some(Quiet(1.000698561s))
```

The judged figure moved by a byte between two looks of a run that did nothing,
and the probe shows such a move is the stat line's own length change.
Each move restarted the one-second quiet clock.

The run that exited normally shows the rest,
verbatim:

```text
watch tid=1986573 window_ms=1000 own_stat_len=333
look tid=1986573 ms=4108 taken=Some(Look { bytes: 8739, ticks: 0, working: false, members: 2 }) previous=Some(Look { bytes: 8739, ticks: 0, working: false, members: 2 }) ending=None readers=[1986572, 1986573] own_stat_len=333
look tid=1986573 ms=4609 taken=Some(Look { bytes: 8739, ticks: 0, working: false, members: 2 }) previous=Some(Look { bytes: 8739, ticks: 0, working: false, members: 2 }) ending=None readers=[1986572, 1986573] own_stat_len=333
look tid=1986573 ms=5109 taken=Some(Look { bytes: 8738, ticks: 0, working: false, members: 2 }) previous=Some(Look { bytes: 8738, ticks: 0, working: false, members: 2 }) ending=None readers=[1986572, 1986573] own_stat_len=332
```

Both watchers' first look came at 4108 ms,
so the process was not scheduled for four seconds of its five.
Of the two looks that followed,
the last one moved:
the figure 8739 to 8738,
with that thread's stat line 333 to 332.
Without that move the look at 5109 ms would have judged 1.502 s quiet and ended the child.

### Root cause

Two defects,
and the control needed both removed.

- The look judged the length of the reader's own stat line.
  `package/git-policy/cli/src/native/testing/run_progress.rs` read the caller's stat first
  and its `io` second,
  so the stat read sat inside the figure the `io` read measured and was never taken off:

  ```rust
  let Ok(stat) = std::fs::read_to_string(task.join("stat")) else { return 0 };
  self.read_bytes = self.read_bytes.saturating_add(stat.len() as u64);
  ...
  let io: String = std::fs::read_to_string(task.join("io")).unwrap_or_default();
  let work: u64 = bytes_in(io.as_str()).saturating_sub(own_reads);
  ```

  A reader's counters gain digits as it works,
  so the figure moved more often the more loaded the box was,
  and a sleeping run looked like it kept moving.
  `look` now reads the caller's own `io` before anything else it reads
  (`run_progress.rs:381`, through `look`'s first call at `run_progress.rs:573`).
  `add_the_callers_own_work` sat at `run_progress.rs:319` on this branch;
  the merge of this branch with the one-reader-per-thread fix left the same accounting in
  `add_the_readers_own_counters` (`run_progress.rs:380`).
  That is the only moment its counters hold the reader's work with none of its readings,
  and every other read of the look is a known length taken off the next look.
- The control raced its own child.
  `child_that_sleeps_under_two_watchers` slept five seconds and then returned,
  and the parent ran it with `run_child`,
  which has no deadline.
  So the control passed or failed on whether a watcher judged the run quiet inside those five
  seconds,
  never on whether two watchers can keep each other alive.
  The child now waits for ever on a channel nothing sends to,
  and the parent runs it with `run_child_with_stderr_until`,
  whose `CHILD_DEADLINE` (45 s in `run_progress_tests.rs:254`) turns a watcher that never ends it
  into the control's failure.

### The controls now

- `a_readers_own_stat_line_is_not_the_runs_work` (`run_progress_tests.rs:494`) asks the
  accounting directly and needs no load.
  The test thread registers as a reader,
  takes a look,
  renames itself with `prctl(PR_SET_NAME)` (a known shorter `comm` and no read or write at all),
  takes a second look,
  and requires the judged bytes not to move.
  The stat line's own shrink is asserted beside it,
  so a rename that changed nothing cannot pass the control.
- `two_watchers_do_not_keep_each_other_alive` (`run_progress_tests.rs:448`) asks what it always
  asked,
  and only a watcher can answer it now.

### Verification

With the old read order planted back in `look`:

```text
thread 'testing::run_progress::tests::a_readers_own_stat_line_is_not_the_runs_work' panicked at src/native/testing/run_progress_tests.rs:454:5:
assertion `left == right` failed: the reader's own stat line shrank from 333 to 320 bytes between the two looks and the run showed nothing, so the figures judged must not move
  left: 17907
 right: 17920
Summary [   0.015s] 1 test run: 0 passed, 1 failed, 6625 skipped
```

13 bytes were the `comm` field.
The run showed nothing at all.

With `look` leaving out only the calling reader,
so that two watchers count each other's readings:

```text
thread 'testing::run_progress::tests::two_watchers_do_not_keep_each_other_alive' (2830503) panicked at src/native/testing/run_progress_tests.rs:392:46:
a watcher ended the child waiting under two of them
FAIL [  45.057s] (1/1) monochromatic-git-policy-cli testing::run_progress::tests::two_watchers_do_not_keep_each_other_alive
Summary [  45.069s] 1 test run: 0 passed, 1 failed, 6625 skipped
```

Restored and run on the branch:
`16 tests run: 16 passed` (`testing::run_progress`),
`31 tests run: 31 passed` (`testing::`),
and the two-watchers control in 1.610 to 1.615 s.
The same ten runs in the same loaded cgroup aborted 10 of 10 (`status 134`) after the fix,
against 9 of 10 before it.

### What is not claimed

- Which of the two ingredients tipped the gate's own run is not measured.
  The reproduction's failing run had both:
  four seconds without a look,
  and a one-byte move at the look that could still have ended the child.
- A thread that takes a look without registering is judged with its whole `rchar`,
  so its own readings move the figure at every look.
  Every reader in the wrapper registers before it looks:
  a watcher in `watch`,
  and a wait in `test_support`.
  and no control exercises that path's figures.
- The loaded family run also failed
  `subvolume::fs_beneath_directory_chain::tests::a_path_deeper_than_a_chain_may_keep_is_walked_one_descriptor_at_a_time`
  once with `Os { code: 24, kind: TooManyOpenFiles, message: "Too many open files" }`.
  It passed alone in 1.650 s and in a repeat of the same 331-test family (`331 passed`).
  It measures its own descriptor budget while the fixture's watchdog reads `/proc`,
  and that race belongs to whoever owns that test.

### The read-error accounting audit, 2026-11-11T09:15:03-05:00

A cross-vendor audit read the source of `426c4022f4` and `bbee0cd9e8`
in a worktree of its own and confirmed the read order:
everything before the caller's `io` read is memory work.
`the_readers` clones the mutex-protected vector (`run_progress.rs:216`),
and `look` formats paths and copies reader identifiers (`run_progress.rs:561` to `572`)
before the first filesystem read (`run_progress.rs:381`, through `look`'s call at `run_progress.rs:573`).
It found two gaps on read-error paths,
and both held.

#### Root cause

A read whose text cannot be decoded delivered bytes that the decode threw away,
so those bytes were never taken off the next look.
The caller's stat read was written this way (`run_progress.rs:349` as it stood before this fix,
which is the commit the audit read; the file has moved since):

```rust
let Ok(stat) = std::fs::read_to_string(task.join("stat")) else { return };
self.read_bytes = self.read_bytes.saturating_add(stat.len() as u64);
```

A Linux task name may hold a byte that is not UTF-8 (`prctl(PR_SET_NAME)` takes any bytes),
and `read_to_string` then returns an error after consuming the whole stat line.
The same shape sat in `add_a_readers_disk_wait` and in `add_task`,
and in `children_of`.
The second gap was the same family:
a look that judged nothing returned before `note_a_reader_s_own_readings`,
so the reads it had already made were never booked.

#### The fixes

- `read_a_tasks_file` (`run_progress.rs:236`) reads the bytes and decodes them lossily,
  and the length booked is the length of the read,
  including what a read delivered before it failed.
  Every `/proc` read of the look goes through it
  (`run_progress.rs:381`
  and `385`
  and `407`
  and `417`
  and `425`
  and `523`).
- The booking moved before the look is judged readable (`run_progress.rs:591`),
  so no look leaves its own readings behind,
  whatever it could read.

#### Verification

- `a_task_name_that_is_not_utf8_is_still_the_readers_own_reading`
  (`run_progress_tests.rs:529`) renames the reader's own thread to `reader` plus a byte that
  no UTF-8 sequence holds,
  takes two looks,
  renames it back,
  and requires the judged bytes not to move.
  With `read_to_string` planted back for the caller's stat it failed in 0.011 s with
  `both looks read the same undecodable stat line and the run showed nothing, so the figures
  judged must not move`,
  `left: 17579`,
  `right: 17254`:
  325 bytes were the stat line it could not decode.
  With the byte read it passes.
- `testing::run_progress` after both fixes:
  `17 tests run: 17 passed`.
  `testing::`:
  `32 tests run: 32 passed`.
- The audit's own probe patch,
  which it had kept beside this record,
  is deleted:
  its two controls asserted the behaviour of the two defects,
  so applying it after the fixes would have failed for the right reason and been read as a
  regression.
  Its measured outputs are quoted above and its findings live here.

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

## Two watchers under a starved processor

Measured on this host on 2026-11-11.
The two-watchers control fails again when the run gets one processor,
with the signature this record already gives for the judgement before every
reader's own reads were taken off.

### Symptom

In one run of the whole library suite on this host,
the only failure of `6612 tests run: 6611 passed, 1 failed, 19 skipped` was
`testing::run_progress::tests::two_watchers_do_not_keep_each_other_alive`,
and that control passed alone:
`1 test run: 1 passed`.

### What was measured

- The control failed with
  `assertion left == right failed: a watcher ended the process:`,
  `left: None`,
  `right: Some(6)`,
  so the child ran its whole five seconds and no watcher ended it.
- The control's own module passed in `30 tests run: 30 passed` when the run had
  the whole host.
- The same module failed the control with the same three lines in
  `30 tests run: 29 passed, 1 failed`
  when it ran inside
  `systemd-run --user --scope -p CPUQuota=100% -p MemoryMax=6G -p TasksMax=512 -p IOWeight=1`
  around
  `mise run //package/git-policy/cli:native:test:host -- testing:: --no-fail-fast`,
  and passed in `30 tests run: 30 passed` with `CPUQuota=700%`.
  So the control's answer follows how much processor the run gets.

### What is not claimed

The cause is not measured to a line.
`look` takes the bytes each reader recorded for its own readings off the counters
(`Reader::own_readings`,
`note_a_reader_s_own_readings`),
so a reading by another reader that lands between the two counter readings of one look
is not taken off,
and two readers can then look like work at every window.
That reading of `package/git-policy/cli/src/native/testing/run_progress.rs` is a hypothesis
this record leaves to a measurement.
What is measured is the failure,
its signature,
and its dependence on the processor the run is given.

Note,
2026-11-11:
the hypothesis did not hold when it was measured.
Two readers' looks read the same byte figure window after window,
and the two-watcher control of this crate did not show the leftover the paragraph above predicts
(see "The reading thread's own work,
measured" below).

## The reading thread's own work, measured (2026-11-11)

### Symptom

Two release gates on this host left these controls unended:

- the gate on `82829a1696`'s successor `6cfc399d09` failed
  `a_stuck_run_ends_its_process_though_its_report_cannot_be_written` and
  `a_wait_does_not_keep_the_watchdog_alive` at 45 s each,
  and sat 1158 s and 1251 s in `two_watchers_do_not_keep_each_other_alive` and
  `a_stuck_run_ends_its_process` until the gate's own SIGTERM ended the run,
  with `2072/7707 tests were not run due to signal`;
- the gate on alpha-10 failed `two_watchers_do_not_keep_each_other_alive` with
  `assertion left == right failed: a watcher ended the process:`,
  `left: None`,
  `right: Some(6)`,
  after its child ran its whole five seconds.

Neither gate's children can be read afterwards:
the parent test captures its child's standard error,
and the two controls that ended by SIGTERM printed nothing,
so whether a watcher judged at all is not in either log.

### What was measured

The look,
with a temporary trace of every look kept in memory
(so the trace itself was not work the run could see),
taken from the two-watcher child (`child_that_sleeps_under_two_watchers`,
two registered readers):

```text
look reader=2482853 bytes=16225 ticks=0 members=2 working=false read_bytes=1617
look reader=2482852 bytes=15892 ticks=0 members=2 working=false read_bytes=1616
look reader=2482852 bytes=15892 ticks=0 members=2 working=false read_bytes=1618
look reader=2482853 bytes=16225 ticks=0 members=2 working=false read_bytes=1618
look reader=2482852 bytes=15892 ticks=0 members=2 working=false read_bytes=1618
look reader=2482853 bytes=16225 ticks=0 members=2 working=false read_bytes=1618
```

Each reader's figure is constant across its looks,
so the accounting took every reading off,
with one reader and with two.
The same probe written to standard error instead,
one line per look,
kept the child alive for its whole five seconds (`finished in 5.01s`,
no abort),
and the byte figure grew about 104 per look.
That is by design and not the hole:
what a look leaves over is the reader's work that is not a reading,
which is the run's work while a wait's predicate does it,
so the trace has to stay in memory and a watcher must not write while it watches.

Two further holes were then measured at the control level,
against the watchdog of `6cfc399d09`:

- a thread inside two registrations of one wait
  (`Reading::begin` twice,
  as a wait whose event another wait reports)
  is two entries in `READING_THREADS`,
  and the second entry's `own_reads` was never advanced,
  so it counted the thread's whole counter as the run's work:
  `a_thread_inside_two_waits_is_left_out_of_the_run_once` failed with
  `left: 20966`,
  `right: 19680`,
  a figure that grew by 1286 between two looks of a thread that did nothing;
- a figure that fell was work to the judge,
  so ten looks with one task fewer and less moved at each never ended:
  `a_run_whose_figures_only_fall_is_ended` failed with
  `left: None`,
  `right: Some(Quiet(2s))`.

What did not reproduce the gate:
the same image,
the same container limits and the same family passed
`15 tests run: 15 passed` at `--cpus 2`,
`32 tests run: 32 passed` at `--cpus 1`,
`30 tests run: 30 passed` three times under `CPUQuota=100%` on this host,
and the two-watcher child was ended by a watcher in 24 of 24 runs at eight at once under `CPUQuota=100%`,
its control passing 24 of 24.
So the trigger of that gate is not measured,
and nothing here claims it.

### The judgment and the controls now

- one thread is one reader,
  however many waits it is inside (`Reader::waits`),
  so no entry of a thread can be read with its readings uncounted;
- each look counts every reading it takes of the reading thread,
  including the one whose own bytes cannot be in the counter it yields,
  so the next look takes exactly them off (`Tally::add_the_readers_own_counters`);
- only a figure that grew is work,
  so a count that fell at every look cannot keep a run alive;
- every control waits on its child through `CHILD_DEADLINE` (45 s),
  and the child controls start their watcher with `expect`,
  so a watcher that cannot start fails its control in 0.1 s instead of leaving a child blocked;
- `child_that_sleeps_under_two_watchers` waits for a message nobody sends instead of sleeping five seconds,
  so a starved scheduler cannot beat its watcher to it.

### The controls, before and after

Measured on this host on 2026-11-11,
each with the watchdog file it guards put back:

- `a_thread_inside_two_waits_is_left_out_of_the_run_once`:
  `left: 20966`,
  `right: 19680` before,
  `Summary [3.283s] 17 tests run: 17 passed` after,
  that control among them;
- `a_run_whose_figures_only_fall_is_ended`:
  `left: None`,
  `right: Some(Quiet(2s))` before,
  `Summary [3.283s] 17 tests run: 17 passed` after;
- with `run` planted to start and never judge,
  every child control failed in 45 s and the run finished:
  `Summary [45.073s] 3 tests run: 0 passed, 3 failed`,
  where the same plant before this change would have held the run for ever;
- with `watch` planted to answer nothing,
  `a_stuck_run_ends_its_process` failed in 0.108 s with
  `the watchdog thread starts` from the child and `left: None`,
  `right: Some(6)`;
- with the fixes in place the family is
  `Summary [3.283s] 17 tests run: 17 passed`,
  the module `Summary [3.816s] 32 tests run: 32 passed`,
  and the touched families together `Summary [3.246s] 21 tests run: 21 passed`.

### What is not claimed

- The trigger of the gate on `6cfc399d09` is not measured.
  The controls pass in that gate's own image and at a quarter of the processors it had,
  so the cause is either host state this record cannot see or the accounting holes above
  meeting conditions not reproduced here.
- Why the runner's own per-test limit (`slow-timeout` 60 s times `terminate-after` 5)
  did not end the two tests that ran 1158 s and 1251 s is not measured.
  What this change fixes is that a control no longer depends on it to end a hung child.
- The two falling-figure and two-registration controls are driven by made-up looks and by a real thread,
  not by a task that vanished under a load average of 100.

## Records this changes

- `doc/troubleshooting/git-policy-cli-landing-controls-wait-on-order-not-time.md` names the old `RUN_LIMIT` in its
  root cause;
  the constant is now `QUIET_WINDOW` and the record carries a note saying so.
- The open question in `doc/handover/cli-git-rust-implementation.md` about the shape of a fix for a control that
  reached the old `RUN_LIMIT` is answered by this record;
  the handover itself is left to its own session.
