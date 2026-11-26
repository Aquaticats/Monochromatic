# A cli-git module's controls armed state of the whole process, so a threaded run failed in a different set each time

`package/git-policy/cli`'s tests run as threads of one process under
`cargo test --lib`,
and the controls of `worktree/worktree_clone_conversion_size_read_tests.rs` were written for one process per test
(nextest's shape, which the container gate gives them).
This is debt ledger item 3 (`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`)
and issue 6493.

## Symptom

One module of 36 controls,
run once with default threads on 2026-11-26:
`cargo test --locked --offline --lib --features test-only-switches worktree::worktree_clone_conversion_size_read:: -- --skip guest --skip _arm_tests`.

Before the change,
three such runs in a row and a set of recording controls on their own:

- 34 passed, 2 failed: `an_abandoned_read_ends_its_child_and_its_record_is_swept`,
  `a_process_that_ends_with_a_running_read_leaves_no_child_behind`.
- 33 passed, 3 failed: those two and `the_read_beside_the_check_leaves_another_steps_note_alone`,
  whose note was gone (`left: None`, `right: Some(FailedStep { name: "git add", status: Some(1) })`).
- 34 passed, 2 failed: the record sweep and `the_abandon_frees_the_answered_map_off_the_adding_thread`,
  which read `left: 5, right: 6`.
- the ten controls that write an add timing record, run as one set at eight threads: 8 passed and 2 failed, then
  10 passed, then 9 passed and 1 failed, then 10 passed.

Issue 6493's own readings on `main` and on the formatted branch were 32 passed with 4 failed and 33 with 3 failed,
and a different set failed each time.
Every one of those controls passed alone (`--exact`).

After the change:
three runs of the same module, 36 passed and 0 failed each time, in 5.26 to 5.31 seconds.

## Root cause

Six pieces of state belong to the whole process, and six controls of the module armed or read them beside the rest:

- the add timing record (`worktree/add_timing.rs`): `WORK` is one `Mutex<Option<Vec<...>>>`,
  `begin` replaces it with an empty list and `finish` takes it.
  Two controls recording at once shared one list, so a control's own work entry went into another's record
  and another's went into its own.
- the failed Git step note (`commit/git_step_failure.rs`): `LAST` is one `Mutex<Option<FailedStep>>`,
  and any Git run of any thread writes or clears it.
- the repository whose record every Git start is noted in (`lock/lock_holder_records.rs`):
  `REPOSITORY` is a `OnceLock<PathBuf>` and the first caller wins, and the record file itself is named for the process.
  Once one control remembered its repository,
  every other control's Git child was noted into that control's record file,
  so its sweep met holders that were still alive and kept the record.
- the size read's holds (`worktree/worktree_clone_conversion_size_read.rs`):
  `THE_READ_IS_HELD`, `THE_READ_DIES` and `THE_SIZES_ARE_HELD_AFTER_THE_WAIT` are consulted by every read of the
  process,
  and `THE_ANSWER_WAS_SENT`, `ANSWERED_MAPS_FREED` and `ANSWERED_MAPS_FREED_ON_THE_ADDING_THREAD`
  (`worktree_clone_conversion_size_read_free.rs`) count every abandoned answer of the process.
  A control that armed a hold held other controls' reads, and `THE_ANSWER_WAS_SENT` was never cleared,
  so a control could abandon before its own read answered and then see no free at all.
- the hold before a watched run's clear (`push/child_watch_one_shot_run.rs`): `THE_CLEAR_IS_HELD` is one flag
  every watched Git run of the process consults.
- the process itself: `a_process_that_ends_with_a_running_read_leaves_no_child_behind` forks,
  and a fork of a process whose runtime threads already run loses the runtime's wake-up in the child;
  `a_read_and_the_fence_complete_on_the_current_thread_stand_in` sets a variable and reads the runtime flavour the
  whole process built;
  and `a_read_with_the_promisor_remote_from_the_environment_fetches_nothing` set five `GIT_CONFIG_*` variables in the
  process, which every other control's Git child inherited.

## The evidence

The add timing one, from a record a control wrote beside another recording control:

```text
the read's own cost is the record's work: {"schemaVersion":1,...,"steps":{},"otherMs":0.102,"work":{},"notes":{},"quiet":{}}
```

and the other direction, a control that states a read still running and must state no cost:

```text
a read still running has no cost to state: {"schemaVersion":1,...,"work":{"fenceSizesBesideTheCheck":{"ms":7.000,"calls":1}},"notes":{"fenceSizesBesideTheCheck":"started and not joined"},"quiet":{}}
```

The record sweep one:
`the line a killed read's child left is swept once every holder it names is gone`,
with the record folder still holding a line for a child of another control that was running.

The failed-step note one: `the read took no note of its own`, `left: None`.

The free counters one: `the abandoned answer was freed, not leaked`, `left: 5`, `right: 6`.

## The change

`testing/alone_in_its_process_child_test_support.rs` starts one control's body in a child process of this test binary
through `testing/child_test_process.rs`,
where it is the only test that process runs,
and the parent's assertion is the child's one passing test.
The twenty-two controls that armed or read one of the six pieces of state above call it first and return in the parent:
the ten that write an add timing record,
the six that arm a size read hold or read its counters,
the clear hold's control,
the failed-step note's control,
the record sweep's control,
the fork's control,
and the two that change the process's environment or runtime.

`refuse_outside_the_child` fails a body that arms one of the size read holds or the clear hold in a process that holds
other tests,
naming the starter,
so a control that forgets its child fails every run at its first line instead of making the rest of a threaded run fail
now and then.

No shipped behaviour changed: every seam and every refusal is `#[cfg(test)]`,
and what the size read does is what `package/git-policy/cli/SPEC.md` says it does
("The fence's size read starts in this rung before the source check",
and "when the read has already answered,
its time is the record's `fenceSizesBesideTheCheck` work").

## What the guard is worth

With the refusal in place and one control's child start taken out, that control fails at its first line:

```text
THE_READ_IS_HELD, the hold every read of this process waits at before its Git run belongs to the whole process,
and `cargo test` runs a crate's tests as threads of one process, so arming it here changes every test of this run:
run this control in a child of its own, which
`crate::alone_in_its_process_child_test_support::ran_alone_in_a_child_of_its_own` starts
```

With the `refuse_outside_the_child` call taken out of `a_control_holds_the_read`, the same test no longer fails there,
which is what the refusal is for.

## Not covered

- The other modules of this crate hold the same debt
  (`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`, item 3, items 3a to 3g),
  and issues 6483 to 6487 record five more failures of the class.
  This change moves one module's controls.
- The container gate was not run for this change: the round ran the module on this host, as issue 6493 measured it,
  and the whole library on this host is not run (a whole-library threaded run hangs, see
  `package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).
