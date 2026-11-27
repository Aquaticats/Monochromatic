# The prestage controls shared one serial lock and one blocking pool, so one failure failed the twenty-one after it

`package/git-policy/cli`'s tests run as threads of one process under `cargo test --lib`,
and the controls of `landing/service_index_prestage_tests.rs` were written as if each had the process to itself.
Two facts of the whole process broke them beside the run's other tests:
the serial lock they take in turn,
and the blocking pool their markers are made on.
Issue 6565.

## Symptom

Issue 6524's round ran the whole `landing::` filter in two shapes with more test threads than processors
while it looked for a failure of its own.
Its logs are in that round's scratch,
which expires on 2026-11-28;
the figures below are quoted from them.

`landing-24-threads-run-1.log`,
24 test threads on ten processors:

```text
test result: FAILED. 1136 passed; 22 failed; 6 ignored; 0 measured; 9607 filtered out; finished in 66.76s
```

Every one of the 22 failures is a control of `landing::service_index_prestage::tests`,
and that module holds 22 controls,
so all of them failed.

`landing-ten-threads-one-cpu-run-1.log`,
ten test threads under `CPUQuota=100%`:

```text
test result: FAILED. 1132 passed; 26 failed; 6 ignored; 0 measured; 9607 filtered out; finished in 119.92s
```

22 of those 26 are the same module,
the same 22 controls.

## The lock: one real failure, twenty-one poisoned readings

Of the 22 failures in the 24-thread log,
one is a failure of its own subject:

```text
---- landing::service_index_prestage::tests::a_commit_that_comes_during_the_flush_takes_the_lock_without_waiting_for_it stdout ----

thread 'landing::service_index_prestage::tests::a_commit_that_comes_during_the_flush_takes_the_lock_without_waiting_for_it' (3666139) panicked at src/native/landing/service_index_prestage_tests.rs:83:9:
no marker was made ahead
```

The other 21 read the lock:

```text
thread 'landing::service_index_prestage::tests::a_commit_that_comes_to_a_failing_flush_stages_its_own_without_joining_it' (3666152) panicked at src/native/landing/service_index_prestage_tests.rs:731:44:
serial: PoisonError { .. }
```

`serial` is the control's own `.expect("serial")` on `ONE_AT_A_TIME.lock()`,
so each of the 21 panicked at its first statement,
before it reached anything it means to check.
The ten-thread run under one processor reads the same first failure at `83:9` and 21 more
`serial: PoisonError { .. }` lines from the same module.

Rust poisons a mutex whose holder panicked,
and every later `lock()` answers an error for the rest of the process.
So one control's real failure failed every control after it,
and the run's report named 22 failures for one cause.

## What the first failure waited for

`until_staged` is the control's witness that its marker's making has begun:
it polls the index's directory for the marker's staging file and fails at ten seconds with
`no marker was made ahead`.
The making is not the control's:
`prestage::start_with` hands it to `spawn_on_runtime(run_blocking_job(..))`,
so the file appears only when the process's blocking pool gives that job a thread.

The runtime is built with one blocking-pool thread per worker
(`foundation/runtime_plan.rs`,
`plan_with_workers`:
`blocking_threads: workers`),
and `workers_for_this_process` derives the workers from the host's budget,
so the pool is a handful of threads for the whole process.
A 24-thread run has 24 test threads starting their own jobs on that one pool.

A planted delay shows it without the rest of the filter,
in this process alone.
Twelve jobs were started on the blocking pool,
each holding its thread until the plant ends,
and the plant waited until the pool stopped taking them:

```text
PLANTED the pool took 7 jobs before it stopped taking them
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10771 filtered out; finished in 10.72s
thread 'landing::service_index_prestage::tests::__planted_a_full_pool_holds_a_making_back_past_the_ten_seconds' panicked at src/native/landing/service_index_prestage_tests.rs:94:9:
no marker was made ahead
```

That is the log's first failure,
at the same line of the same helper,
ten seconds after a making that had a thread
to wait for.
The plant is test-only and uncommitted;
it is not in the change.

## The change

Every control of the module now runs its body in a child of this test binary,
`run_the_body_in_a_child_of_its_own` (`testing/process_wide_state_child_test_support.rs`),
where it is the only test that process runs.
The child owns the pool its making needs and the counters it reads
(`FLUSHES`,
`HELD_FLUSHES_ENDED`,
`COMMIT_WAITING` and `GATED_FLUSH_DONE`,
`SERVICE_ENDS` and the list of the files
made ahead),
so nothing another test does can queue its making or move its counters.
The serial lock is gone with them:
the counters are that child's alone,
so there is nothing left for one lock to keep in turn.
The commit before this one took that lock's guard out of a poison error,
which is what stopped one failing control from failing the twenty-one after it;
with the bodies in children there is no lock left to poison,
so `the_controls_lock` and `ONE_AT_A_TIME` are deleted rather than kept as a lock with nothing to guard.
Shown with a control made to panic while holding the lock,
one thread:
before,
`test result: FAILED. 0 passed; 23 failed; 1 ignored; 0 measured; 10748 filtered out; finished in 0.21s`
with 22 `serial: PoisonError { .. }` readings;
after,
`test result: FAILED. 22 passed; 1 failed; 1 ignored; 0 measured; 10748 filtered out; finished in 1.36s`,
the one failure being the plant's own panic.

The same rule holds for the one control that gives this process a bound:
`evicting_a_marker_that_never_got_a_thread_makes_no_file_and_waits_for_nothing` calls
`lazy_runtime::cap_workers(1)`,
which belongs to the whole process,
and now sets it in its own child.
Before this change that call reached the run's own runtime,
and whether it did anything depended on whether that
runtime was built yet.

## The red and the green

The module alone,
`--test-threads=24`,
on `main`'s code before the change:

- `test result: FAILED. 10 passed; 12 failed; 1 ignored; 0 measured; 10748 filtered out; finished in 4.14s`
- `test result: FAILED. 19 passed; 3 failed; 1 ignored; 0 measured; 10748 filtered out; finished in 28.05s`

The second run's first failure is its own subject and not the lock:

```text
---- landing::service_index_prestage::tests::a_start_that_evicts_a_marker_still_flushing_does_not_wait_for_it stdout ----

thread 'landing::service_index_prestage::tests::a_start_that_evicts_a_marker_still_flushing_does_not_wait_for_it' (2865364) panicked at src/native/landing/service_index_prestage_tests.rs:1029:9:
assertion `left == right` failed: the start waited for the oldest marker's flush to end
  left: 1
 right: 0
```

`HELD_FLUSHES_ENDED` is the process's,
and the control holds two flushes for at most `THE_CEILING_OF_A_HELD_FLUSH` (two seconds) each while it reads it:
a loaded host reaches that ceiling,
and so does a making queued behind another control's job.
So a threaded run failed in a different set each time,
which is the shape of a control that reads the process.

After the change,
the same module at the same 24 threads:

```text
test result: ok. 22 passed; 0 failed; 1 ignored; 0 measured; 10748 filtered out; finished in 6.76s
```

And with the plant beside them,
the pool of the run's own process full for the control's whole ten-second witness,
the same 24 threads:

```text
PLANTED the pool took 7 jobs before it stopped taking them
test result: FAILED. 22 passed; 1 failed; 1 ignored; 0 measured; 10748 filtered out; finished in 10.60s
```

The one failure there is the plant's own assertion:
every control passed while the pool of the process that started
them was full.

## What is not claimed

- The log cannot say which jobs held the pool on 2026-11-27:
  it holds no timing of the module's own makings.
  What names the pool is the planted delay above,
  which reproduces that failure at the same line of the same helper,
  and the pool's size,
  which is the code's own
  (`runtime_plan.rs`).
- `landing::service_land_take_index_lock::tests` carries the same two defects
  (a serial lock of its own,
  and the same markers made ahead),
  and the ten-thread run under one processor failed two of its controls:
  `taking_this_batchs_lock_does_not_wait_for_another_indexs_flush` at `267:9` with
  `taking this batch's lock waited for another index's marker to finish flushing`,
  and
  `the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock` at `117:44` with
  `serial: PoisonError { .. }`.
  That module is not this record's to change.
- The same run failed two controls of `landing::service_work_items_side_by_side::tests`
  (`the two branches were inside the push handler at the same moment`,
  `left: 1`),
  which read the process's room for blocking jobs;
  they are a third incident,
  not read further here.
