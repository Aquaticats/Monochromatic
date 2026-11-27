# The take-index-lock controls shared one serial lock and one blocking pool

`package/git-policy/cli`'s tests run as threads of one process under `cargo test --lib`,
and the two controls of `landing/service_land_take_index_lock_tests.rs` were written as if each had the
process to itself.
Two facts of the whole process broke them:
the blocking pool their markers and their two held flushes are jobs of,
and the serial lock one control's panic left poisoned for the other.
Issue 6569,
measured on 2026-11-27T06:00:00-05:00 in this worktree.

## Symptom

Issue 6569 quotes issue 6524's round,
which ran the whole `landing::` filter with ten test threads under one processor
(`CPUQuota=100%`).
Its log is in that round's scratch,
which expires a day after it was written,
and the two failures are quoted from the round's report of them:

```text
thread 'landing::service_land_take_index_lock::tests::taking_this_batchs_lock_does_not_wait_for_another_indexs_flush' panicked at src/native/landing/service_land_take_index_lock_tests.rs:267:9:
assertion `left == right` failed: taking this batch's lock waited for another index's marker to finish flushing
  left: 1
 right: 0

thread 'landing::service_land_take_index_lock::tests::the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock' panicked at src/native/landing/service_land_take_index_lock_tests.rs:117:44:
serial: PoisonError { .. }
```

The first is a failure of the control's own subject and the second is the lock:
`serial` is the second control's own `.expect("serial")` on `ONE_AT_A_TIME.lock()`,
so it panicked at its first statement,
before it reached anything it means to check.

## The same two failures, without the rest of the filter

The module alone at `CPUQuota=100%`,
on `main`'s code at `138d5b5919`,
with the command the package's guide gives for one module
(`cargo test --locked --offline --lib --features test-only-switches`,
the module's path,
and `-- --skip guest --skip _arm_tests`):

```text
thread 'landing::service_land_take_index_lock::tests::taking_this_batchs_lock_does_not_wait_for_another_indexs_flush' (3991837) panicked at src/native/landing/service_land_take_index_lock_tests.rs:267:9:
assertion `left == right` failed: taking this batch's lock waited for another index's marker to finish flushing
  left: 1
 right: 0

thread 'landing::service_land_take_index_lock::tests::the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock' (3997221) panicked at src/native/landing/service_land_take_index_lock_tests.rs:117:44:
serial: PoisonError { .. }

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 10789 filtered out; finished in 2.32s
```

Both lines of the kept log,
from two controls alone in one process.
The same module at `CPUQuota=200%` was green in 0.05 s,
so the shape that shows it is the one the pool is narrow in.

## What the first failure waited for

`taking_this_batchs_lock_does_not_wait_for_another_indexs_flush` holds another index's two markers flushing,
then takes this index's lock and asserts that neither flush ended while it did.
Both flushes are `service_index_prestage::start_with(..)` jobs,
which `spawn_on_runtime(run_blocking_job(..))` hands to the process's blocking pool,
and `sync_held_by_the_control` holds each for at most `THE_CEILING_OF_A_HELD_SYNC` (two seconds)
when the control has not let it go.

The pool is the whole process's,
with one place per runtime worker
(`foundation/runtime_plan.rs`,
`plan_with_workers`:
`blocking_threads: workers`,
and `foundation/blocking_jobs_within_the_budget.rs`,
`decide_the_budget`).
`HostResources::read` takes the smaller of the affinity mask and the group's CPU quota
(`doc/troubleshooting/container-nproc-ignores-cpu-quota.md`),
so under `CPUQuota=100%` the pool has one place.
The control's two flushes then enter one at a time:
the first ends at its own two-second ceiling before the second has entered,
`until_the_held_flushes_are_running(2)` sees both entered and returns,
and the lock is taken with `HELD_SYNCS_ENDED` at 1.
That is the log's `left: 1`.

`HELD_SYNCS_ENTERED`,
`HELD_SYNCS_ENDED` and `FLUSH_MAY_END` are statics of the test process,
and `ONE_AT_A_TIME` serialized the two controls,
so one control's panic left the other's `.expect("serial")` a `PoisonError` for the rest of the process.

## The plant issue 6565's round used, applied here

Issue 6565's fix planted a control that filled the process's blocking pool and then waited as its failing control waited
(`git-policy-cli-prestage-controls-share-one-lock-and-one-pool.md`).
The same plant here,
test-only and uncommitted,
starts `budgeted_blocking_jobs() + 1` jobs that hold their places,
waits until the pool stops taking them,
and then performs the failing control's own wait.
Run beside the module's control at two processors,
`--test-threads=2`:

```text
PLANTED the pool of 2 places took 2 jobs before it stopped taking them

thread 'landing::service_land_take_index_lock::tests::taking_this_batchs_lock_does_not_wait_for_another_indexs_flush' (46985) panicked at src/native/landing/service_land_take_index_lock_tests.rs:205:9:
0 of 2 held flushes entered

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 10791 filtered out; finished in 10.11s
```

The module's control ran out the same ten-second entry wait its sibling ran out in the kept log,
with nothing else in the process but the plant.
That is the pool:
another test's jobs filling it,
or a host leaving it one place,
decide whether this control's premise holds.

A control made to panic while it held `ONE_AT_A_TIME`,
one thread so its siblings follow it (`--test-threads=1`):

```text
thread 'landing::service_land_take_index_lock::tests::__planted_panics_while_it_holds_the_serial_lock' (63335) panicked at src/native/landing/service_land_take_index_lock_tests.rs:363:5:
PLANTED: a control that panics while it holds the serial lock

thread 'landing::service_land_take_index_lock::tests::taking_this_batchs_lock_does_not_wait_for_another_indexs_flush' (63336) panicked at src/native/landing/service_land_take_index_lock_tests.rs:232:44:
serial: PoisonError { .. }

thread 'landing::service_land_take_index_lock::tests::the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock' (63337) panicked at src/native/landing/service_land_take_index_lock_tests.rs:117:44:
serial: PoisonError { .. }

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 10790 filtered out; finished in 0.00s
```

Both plants are test-only and uncommitted;
neither is in the change.

## The change

Every control of the module now runs its body in a child of this test binary,
`run_the_body_in_a_child_of_its_own` (`testing/process_wide_state_child_test_support.rs`),
where it is the only test that process runs.
The child owns the pool its markers and flushes are jobs of,
and the counters it reads,
so nothing another test does can queue its jobs or move its counters.
`ONE_AT_A_TIME` is deleted rather than kept:
the counters are the child's alone,
so there is nothing left for one lock to keep in turn,
and there is no lock left for a panicking control to poison.

The first control was already host-independent once its jobs had a pool of its own:
it waits up to sixty seconds for two marker files,
and two makings that queue run one after the other in well under that.

The second control's premise is not:
it needs both of the other index's flushes in flight at once,
and a host whose budget leaves the pool one place cannot give it that.
So its body gives its own child the room
(`foundation/room_for_blocking_jobs_in_a_control.rs`,
`for_a_control(service_index_prestage::MARKERS_MADE_AHEAD)`),
one place per flush it holds at once.
The room belongs to the whole process,
which is the second reason the body runs in a child:
`for_a_control` refuses in any other process and names the starter
(issue 6487).

## The red and the green

Before the change,
the module alone at `CPUQuota=100%`,
`test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 10789 filtered out; finished in 2.32s`,
the two failures quoted above.

After the change,
the same module at the same `CPUQuota=100%`:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10789 filtered out; finished in 0.08s
```

And at the guide's default threads under `CPUQuota=200%`:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10789 filtered out; finished in 0.11s
```

With the full-pool plant beside them at two threads,
the pool of the process that started the controls full for the control's whole ten-second witness:

```text
PLANTED the pool of 2 places took 2 jobs before it stopped taking them

thread 'landing::service_land_take_index_lock::tests::__planted_a_full_pool_takes_this_batchs_lock_into_another_indexs_wait' (511282) panicked at src/native/landing/service_land_take_index_lock_tests.rs:241:9:
0 of 2 held flushes entered

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 10791 filtered out; finished in 10.11s
```

The one failure is the plant's own assertion,
and the control that shared its process passed:
each control's jobs now have a pool the plant cannot fill.

With the panic plant written the way every control of the module is written now,
one thread:

```text
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 10790 filtered out; finished in 0.09s
```

The one failure is the plant's own panic in its own child.
Both controls passed,
so one failing control fails no other.

## What is not claimed

- The kept log cannot say which jobs held the pool on 2026-11-26.
  What names the pool is the plant above,
  which reproduces the same wait running out,
  and the pool's width,
  which is the code's own
  (`runtime_plan.rs` and `blocking_jobs_within_the_budget.rs`).
  The one-processor run reproduces the log's exact line without any plant,
  because that shape is the one the pool is narrow in.
- The `1` the first failure read is this module's own two-second ceiling on a held flush,
  not a wait the release code made.
  The release code's rule under test,
  that taking this index's lock does not wait for another index's marker's flush,
  is what the green runs above show;
  the red runs stopped at the control's own premise before they reached the second control's assertions
  on the other index's staged files.
- Only this module was run.
  The whole `landing::` filter was not run again here,
  and no other module's controls were changed.
