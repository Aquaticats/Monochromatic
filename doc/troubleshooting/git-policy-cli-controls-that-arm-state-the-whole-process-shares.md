# A cli-git module's controls armed state of the whole process, so a threaded run failed in a different set each time

`package/git-policy/cli`'s tests run as threads of one process under
`cargo test --lib`,
and the controls of `worktree/worktree_clone_conversion_size_read_tests.rs` were written for one process per test
(nextest's shape,
which the container gate gives them).
This is debt ledger item 3 (`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`)
and issue 6493.

## Symptom

One module of 36 controls,
run once with default threads on 2026-11-26:
`cargo test --locked --offline --lib --features test-only-switches worktree::worktree_clone_conversion_size_read:: -- --skip guest --skip _arm_tests`.

Before the change,
three such runs in a row and a set of recording controls on their own:

- 34 passed,
  2 failed:
  `an_abandoned_read_ends_its_child_and_its_record_is_swept`,
  `a_process_that_ends_with_a_running_read_leaves_no_child_behind`.
- 33 passed,
  3 failed:
  those two and `the_read_beside_the_check_leaves_another_steps_note_alone`,
  whose note was gone (`left: None`,
  `right: Some(FailedStep { name: "git add", status: Some(1) })`).
- 34 passed,
  2 failed:
  the record sweep and `the_abandon_frees_the_answered_map_off_the_adding_thread`,
  which read `left: 5, right: 6`.
- the ten controls that write an add timing record,
  run as one set at eight threads:
  8 passed and 2 failed,
  then
  10 passed,
  then 9 passed and 1 failed,
  then 10 passed.

Issue 6493's own readings on `main` and on the formatted branch were 32 passed with 4 failed and 33 with 3 failed,
and a different set failed each time.
Every one of those controls passed alone (`--exact`).

After the change:
three runs of the same module,
36 passed and 0 failed each time,
in 5.26 to 5.31 seconds;
three more after `origin/main` was merged at `73f1091ce0`,
36 passed and 0 failed each time,
in 5.27,
5.36 and 5.39 seconds;
and three more after the second merge of `origin/main` at `0aa42fdb9f`,
36 passed and 0 failed each time,
in 5.25,
5.29 and 5.30 seconds;
and three more after the third merge of `origin/main` at `556b3a6118`,
36 passed and 0 failed each time,
in 5.24,
5.25 and 5.27 seconds.

## Root cause

Six pieces of state belong to the whole process,
and six controls of the module armed or read them beside the rest:

- the add timing record (`worktree/add_timing.rs`):
  `WORK` is one `Mutex<Option<Vec<...>>>`,
  `begin` replaces it with an empty list and `finish` takes it.
  Two controls recording at once shared one list,
  so a control's own work entry went into another's record
  and another's went into its own.
- the failed Git step note (`commit/git_step_failure.rs`):
  `LAST` is one `Mutex<Option<FailedStep>>`,
  and any Git run of any thread writes or clears it.
- the repository whose record every Git start is noted in (`lock/lock_holder_records.rs`):
  `REPOSITORY` is a `OnceLock<PathBuf>` and the first caller wins,
  and the record file itself is named for the process.
  Once one control remembered its repository,
  every other control's Git child was noted into that control's record file,
  so its sweep met holders that were still alive and kept the record.
- the size read's holds (`worktree/worktree_clone_conversion_size_read.rs`):
  `THE_READ_IS_HELD`,
  `THE_READ_DIES` and `THE_SIZES_ARE_HELD_AFTER_THE_WAIT` are consulted by every read of the
  process,
  and `THE_ANSWER_WAS_SENT`,
  `ANSWERED_MAPS_FREED` and `ANSWERED_MAPS_FREED_ON_THE_ADDING_THREAD`
  (`worktree_clone_conversion_size_read_free.rs`) count every abandoned answer of the process.
  A control that armed a hold held other controls' reads,
  and `THE_ANSWER_WAS_SENT` was never cleared,
  so a control could abandon before its own read answered and then see no free at all.
- the hold before a watched run's clear (`push/child_watch_one_shot_run.rs`):
  `THE_CLEAR_IS_HELD` is one flag
  every watched Git run of the process consults.
- the process itself:
  `a_process_that_ends_with_a_running_read_leaves_no_child_behind` forks,
  and a fork of a process whose runtime threads already run loses the runtime's wake-up in the child;
  `a_read_and_the_fence_complete_on_the_current_thread_stand_in` sets a variable and reads the runtime flavour the
  whole process built;
  and `a_read_with_the_promisor_remote_from_the_environment_fetches_nothing` set five `GIT_CONFIG_*` variables in the
  process,
  which every other control's Git child inherited.

## The evidence

The add timing one,
from a record a control wrote beside another recording control:

```text
the read's own cost is the record's work: {"schemaVersion":1,...,"steps":{},"otherMs":0.102,"work":{},"notes":{},"quiet":{}}
```

and the other direction,
a control that states a read still running and must state no cost:

```text
a read still running has no cost to state: {"schemaVersion":1,...,"work":{"fenceSizesBesideTheCheck":{"ms":7.000,"calls":1}},"notes":{"fenceSizesBesideTheCheck":"started and not joined"},"quiet":{}}
```

The record sweep one:
`the line a killed read's child left is swept once every holder it names is gone`,
with the record folder still holding a line for a child of another control that was running.

The failed-step note one:
`the read took no note of its own`,
`left: None`.

The free counters one:
`the abandoned answer was freed, not leaked`,
`left: 5`,
`right: 6`.

## The change

`testing/process_wide_state_child_test_support.rs` starts one control's body in a child process of this test binary
through `testing/child_test_process.rs`,
where it is the only test that process runs,
and the parent's assertion is the child's one passing test.
That helper landed with issue 6487's round (branch `fix/git-policy-cli-issue-6487-one-process-wide-room-for-blocking-jobs`),
and this module's controls now call it too.
The twenty-two controls that armed or read one of the six pieces of state above call it first and return in the parent:
the ten that write an add timing record,
the six that arm a size read hold or read its counters,
the clear hold's control,
the failed-step note's control,
the record sweep's control,
the fork's control,
and the two that change the process's environment or runtime.

`refuse_outside_the_child` fails a body that sets one of the size read holds or the clear hold in a process that holds
other tests,
naming the starter,
so a control that forgets its child fails every run at its first line instead of making the rest of a threaded run fail
now and then.
The same refusal already stood in front of the room and the blocking bound that helper was written for.

No shipped behaviour changed:
every seam and every refusal is `#[cfg(test)]`,
and what the size read does is what `package/git-policy/cli/SPEC.md` says it does
("The fence's size read starts in this rung before the source check",
and "when the read has already answered,
its time is the record's `fenceSizesBesideTheCheck` work").

## What the guard is worth

With the refusal in place and one control's child start taken out,
that control fails at its first line on every run
(`0 passed; 1 failed`):

```text
THE_READ_IS_HELD, the hold every read of this process waits at before its Git run belongs to the whole process,
and `cargo test` runs a crate's tests as threads of one process, so setting it here reaches every other test of that
run: set it in a child of the control's own,
which `crate::process_wide_state_child_test_support::started_within_a_child_of_its_own` starts
```

With the `refuse_outside_the_child` call taken out of `a_control_holds_the_read`
and the same control still running beside the others,
the run passes (`1 passed; 0 failed`):
the control set a hold of the whole process and nothing said so,
which is what the refusal is for.

The refusal and the child's own answer also have permanent controls,
in `package/git-policy/cli/src/native/testing/process_wide_state_child_test_support_tests.rs`:
one is `#[should_panic]` on the refusal's own sentence,
and the other is a body that runs in its child and asserts that the child saw its marker.

## Not covered

- The other modules of this crate hold the same debt
  (`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`,
  item 3,
  items 3a to 3g);
  this change moves one module's controls,
  and the rounds that fixed 6487,
  6486 and 6484 moved others.
- Issues 6483 and 6485 are still open on `origin/main` at `73f1091ce0`.
- The container gate was not run for this change:
  the round ran the module on this host,
  as issue 6493 measured it,
  and the whole library on this host is not run (a whole-library threaded run hangs,
  see
  `package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).

## Second round, 2026-11-26: the watcher module's controls (issue 6511)

`push::child_watch::` failed the same way on `main` at `67b47a612a`,
as issue 6493's round measured it while a whole container gate and four writer rounds ran:
7 of 48 controls failed in one run,
8 in the next,
and the branch that round was merging failed 8 in both of its own,
though it changed nothing these controls reach.
Every one of those controls passed alone.

This round measured the same module on this host,
with `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`:

```text
cargo test --locked --offline --lib --features test-only-switches push::child_watch:: -- --skip guest --skip _arm_tests

test result: FAILED. 42 passed; 5 failed; 1 ignored; 0 measured; 10636 filtered out; finished in 32.28s
```

The five,
with what each read:

- `one_shot_through_pipes_tests::a_run_leaves_no_descriptor_open`:
  `assertion left == right failed`,
  `left: 27`,
  `right: 24`.
- `tests::a_step_that_is_never_ended_for_being_quiet_is_ended_at_its_ceiling`
  and `tests::a_step_that_keeps_working_is_ended_at_its_ceiling`:
  the error was not the ceiling's but
  "Nothing could watch it:
  the system refused a thread for the watcher".
- `tests::a_step_whose_watch_is_held_back_is_still_ended_at_its_ceiling`:
  "the watch reached the write of its first notice:
  Disconnected",
  because no watcher existed to write it.
- `tests::a_wait_past_the_bound_is_announced_then_the_child_is_stopped`:
  `watch.expired()` was false,
  and the run took 32 s because that control waited out its whole sleeper.

The same module with `--test-threads=1` before the change:
`test result: ok. 47 passed; 0 failed; 1 ignored`,
in 41.38 s.
Each of the five alone with `--exact`:
`1 passed; 0 failed` each,
in 0.33 s,
0.73 s,
0.73 s,
19.86 s and 0.51 s.
So neither the host's speed nor a control's own bound was the cause;
which controls shared the process decided the set that failed.

The module beside it,
`push::child_watch_output_watched_on_the_runtime::tests`,
was red the same way,
and its two failures name both facts at once:
its 7 controls threaded ran `5 passed; 2 failed` twice,
in 0.53 s each,
and `7 passed; 0 failed` with `--test-threads=1`,
in 1.13 s.
`a_step_awaited_on_the_runtime_leaves_no_descriptor_open` read `left: 16`,
`right: 23`,
the run having added 7 descriptors another control's child held,
and `a_step_left_running_on_the_runtime_is_collected_where_the_build_of_the_runtime_was_late` read
"the runtime this process built can run a task,
so the late build has workers",
because another control's stand-in had refused every task of the process at that moment.
After the change that module ran `7 passed; 0 failed` twice,
in 0.56 s each,
and each of its seven controls alone passed.

### Root cause

Two facts of the whole process:

- the two stand-ins `foundation/lazy_runtime.rs` holds for a system that refused every thread
  (`REFUSED`,
  armed by `a_control_refuses_every_task`) and for a runtime build that outlasted its deadline
  (`THE_BUILD_WAS_LATE`,
  armed by `a_control_stands_in_for_a_late_runtime`).
  `tasks_can_run` and `built_flavor` answer for the whole process,
  and each stand-in's comment said the flag cannot reach another control,
  because nextest gives every control a process of its own.
  A threaded run gives them one process:
  a control that armed one made every other control's `ChildWatch` unwatched,
  so no notice was written,
  no quiet child was ended,
  and the ceiling came from the wait rather than from the watch.
- `/proc/self/fd` lists every thread's descriptors,
  so the two controls that compare a count before a run against a count after it measured the whole process:
  another control's thread opened and closed descriptors between the two counts.

### The change

Every control that arms one of the two stand-ins now runs its body in a child of this test binary,
through `testing/process_wide_state_child_test_support.rs`,
the helper issue 6487's round landed for the room and the bound a process makes once,
and returns in the parent:

- `push::child_watch::tests`:
  three controls;
- `push::child_watch_output_watched_on_the_runtime::tests`,
  the module whose file is `push/child_watch_output_watched_on_the_runtime_tests.rs`:
  two;
- `service::service_runtime_build_report::tests`:
  three,
  which is the only other module of this crate that arms them.

The two descriptor controls,
`a_run_leaves_no_descriptor_open` and `a_step_awaited_on_the_runtime_leaves_no_descriptor_open`,
run their body in such a child too,
so the table a count reads is one test's own.

Both stand-ins now call `refuse_outside_the_child`,
naming the starter,
so a control that forgets its child fails at its own first line on every run,
instead of fixing by chance which of its neighbours fail.
The sentence a forgotten child meets,
in full:

```text
the stand-in for a system that refuses every thread of this process's runtime belongs to the whole process,
and `cargo test` runs a crate's tests as threads of one process, so setting it here reaches every other test of that
run: set it in a child of the control's own,
which `crate::process_wide_state_child_test_support::started_within_a_child_of_its_own` starts
```

No shipped behaviour changed:
every seam and every refusal is `#[cfg(test)]`,
and what ends a watched step,
and when,
is what `package/git-policy/cli/SPEC.md` says it is.

### What the guard is worth

After the change,
three threaded runs of the module:
`test result: ok. 47 passed; 0 failed; 1 ignored` each,
in 22.15 s,
22.12 s and 22.15 s.
With `--test-threads=1`:
the same line,
in 41.37 s.
The same five controls alone after the change:
`1 passed; 0 failed` each.

The refusal was shown failing without its precondition,
and the child start without the refusal:

- with the child start taken out of `a_step_that_finishes_inside_its_ceiling_is_answered`,
  its own run fails at its first line,
  `test result: FAILED. 0 passed; 1 failed`,
  with the sentence above;
- with the `refuse_outside_the_child` call taken out of `a_control_refuses_every_task` and the same child start still out,
  the same run passes,
  `1 passed; 0 failed`,
  in 1.08 s:
  the control armed a flag of the whole process and nothing said so,
  which is the leak the refusal exists to catch.

### Not covered

- The container gate was not run:
  this round ran the module on this host,
  as issue 6493 measured it,
  and a whole-library threaded run is not run here
  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).
- The other modules of this crate still hold the debt of item 3 of
  `package/git-policy/cli/doc/debt-ledger-2026-11-25.md`,
  and issues 6483 and 6485 are still open.
- One fact of the whole process was not measured:
  a runtime build that outlasts `BUILD_DEADLINE` of one second leaves the process with a current-thread stand-in,
  and every watch of that process is then unwatched.
  A build slow enough for that under a gate's load would fail the same controls for a second reason.
  This round's runs never met it,
  and no control of this crate produces it on purpose.
