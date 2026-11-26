# The commit index span control counts steps other parallel tests noted

`package/git-policy/cli/src/native/commit/prepare_work_spans.rs` keeps one `static` ledger for the whole process:
every preparation notes its named steps into it, and every timing record empties it.
`cargo test --lib` runs the crate's tests as threads of one process, so the steps one test's preparation notes
land in another test's drain.

## Symptom

`commit::commit_index::tests::every_selection_mode_states_its_one_read_and_its_seed_walk` failed once in a
threaded module run and passed alone (issue 6507), and the same control failed in both threaded runs of its own
filter on 2026-11-26:

```console
$ cargo test --locked --offline --lib --features test-only-switches commit::commit_index::tests:: -- --skip guest --skip _arm_tests
thread 'commit::commit_index::tests::every_selection_mode_states_its_one_read_and_its_seed_walk' panicked at src/native/commit/commit_index_tests.rs:259:9:
assertion `left == right` failed: index: the one read of the real index is stated: [PrepareSpan { name: "index copy", calls: 6, ... }, PrepareSpan { name: "index read", calls: 2, ... }, ...]
  left: Some(2)
 right: Some(1)
test result: FAILED. 31 passed; 1 failed; 0 ignored; 0 measured; 10647 filtered out; finished in 0.24s

$ cargo test --locked --offline --lib --features test-only-switches -- --exact commit::commit_index::tests::every_selection_mode_states_its_one_read_and_its_seed_walk
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10678 filtered out; finished in 0.02s
```

A run of the `index::` filter the issue named answered `ok. 230 passed; 0 failed` twice on the same build, so
the failure is a race and not every run shows it.

## Root cause

The control drains the ledger before and after one `initialize` and counts the `index read`, `index seed` and
`index copy` steps it finds (`commit_index_tests.rs:255,257`). Every other test that runs a commit preparation
notes the same names into the same ledger while this control is between its two drains, so a second `index read`
appears in the set the control counts, and a test that finishes a timing record empties the set before the
control reads it at all.

The one process holds the effect even without threads, which is what makes it measurable:

```console
$ cargo test --locked --offline --lib --features test-only-switches -- --test-threads=1 \
    commit::commit_index::tests::explicit_path_stages_only_the_named_paths \
    commit::prepare_work_spans::tests::
test result: FAILED. 5 passed; 1 failed
```

There `a_step_of_no_measurable_time_is_stated_as_one_call` read five steps of the preparation the other test ran
before it. The same shape failed `the_record_states_the_preparations_own_work_by_name` with four steps of that
neighbour in its record, where the control expects only the two it noted itself.

## The controls this cause reached

- `commit::commit_index::tests::every_selection_mode_states_its_one_read_and_its_seed_walk`
  (`commit/commit_index_tests.rs`).
- `commit::prepare_work_spans::tests::runs_under_one_name_are_added_to_one_entry_in_the_order_first_noted`,
  `commit::prepare_work_spans::tests::a_step_of_no_measurable_time_is_stated_as_one_call` and
  `commit::prepare_work_spans::tests::names_after_the_bound_are_counted_and_not_kept`
  (`commit/prepare_work_spans_tests.rs`), which assert an empty ledger, one step, and a bounded list.
- `commit::commit_timing::tests::the_record_states_the_preparations_own_work_by_name`
  (`commit/commit_timing_tests.rs`), which writes the steps a `Timing::finish` drained into its record.

Seven further controls of `commit_timing_tests.rs` call `finish` and so empty the ledger without asserting any
step; they are harmless to themselves and are left where they are, because a control that owns its process can
no longer lose its steps to them.

## The fix

Each of the five controls runs its body in a child of the test binary
(`testing/process_wide_state_child_test_support.rs`), the same answer issues 6487, 6492, 6495 and 6498 gave for
the process's blocking room, its durability switch and its sync counts. The child's ledger holds nothing but what
that control noted.

`source_guard/no_control_reads_the_preparation_steps_in_the_run_tests.rs` keeps the cause from coming back: a
control of a test-only source that names the ledger (`drain_spans(`, `note_span(`, `prepare_work_spans::` or
`prepareSpans`) in code and does not start a child of its own is a finding. With one control's child starter
taken out, it answered
`commit/prepare_work_spans_tests.rs:80 (a_step_of_no_measurable_time_is_stated_as_one_call)`.

## What this page does not cover

`commit::commit_index::tests::interactive_selection_succeeds_without_changes` runs `git add --interactive`,
which reads the standard input the test run inherits and printed its `What now>` prompt once without ever
ending, so the run's own watchdog aborted the whole module run after 120 s with 229 tests passed and no
`test result` line. That is a second defect: the control needs a process whose standard input is not the run's,
which the same child helper gives. It is recorded here because it hid the counts above, and it is not fixed by
this change.

The child that hang left behind is worth its own line: the watchdog aborted the test process, and the
`git add --interactive` it had started kept the run's output pipe open, so the capped scope around the run stayed
alive for 21 minutes after the abort and was stopped by hand. A control whose body runs in a child of the test
binary ends with that child (`spawn_owned`), which is the other half of why the controls above run there.
