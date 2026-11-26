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

## The other four ledgers, and the window a notice reads (issue 6509)

Four more ledgers keep a commit's figures until its timing record takes them,
and each is one list for the whole process:
`commit/commit_index_builds.rs` (how each build of the private index was made),
`commit/commit_prepare_part_durations.rs` and `commit/commit_build_part_durations.rs` (the stretches of a
preparation and of a build),
and `transaction/transaction_step_durations.rs` (the Git steps a stretch takes out of itself, which
`step_time_between` reads while the stretch is still open).
Two threaded runs of `commit::` on 2026-11-26 answered `test result: FAILED. 464 passed; 10 failed` both
times,
with a varying membership among these controls:

- `commit::commit_index_builds::tests`: five controls read another test's builds, and one read an empty list
  because another control's timing record had taken its build (`left: []`, `right: [IndexBuild { kind: First,
  stat_data_kept: true }]`).
- `commit::commit_prepare_part_durations::tests` and `commit::commit_build_part_durations::tests`: the
  stretch lists, and the step ledger, so a neighbour's step was taken out of the stretch under test
  (`PreparePart { name: "policies", took: 3.079988ms, children: 0ns }`, where the control's own step of
  500 ms should have been found, and `children: 1.465874ms` in a stretch that held no step of its own).
- `commit::commit_preparation::tests`: two controls asserted that this process has no descendant left, and
  every other test's Git child is a descendant of the same process (`a process was left below the command`).
- `commit::native_commit_waiting_notice::tests::the_notice_reports_the_programs_that_ran_below_the_step`,
  which is not a ledger: the control read both control windows (`commit/scratch_step_look_below_control_window.rs`
  and `commit/native_commit_waiting_notice_control_window.rs`) without taking either, and
  `commit_preparation_tests.rs`'s `a_scratch_commit_whose_ceiling_look_failed_lands_when_the_step_ends`
  planted six looks that cannot be made while this control's notice looked.
  A blind look clears the look before it, so 1.2 s of watching credited one 250 ms interval
  (`looks: 5, looks_found: 3, blind_looks: 0, below: [SeenProgram { name: "sleep", seen: 250.130234ms }]`).

Three of the four ledger modules were red in a threaded run of themselves
(`commit::commit_index_builds::tests` `4 passed; 7 failed`,
`commit::commit_prepare_part_durations::tests` `3 passed; 2 failed`,
`commit::commit_preparation::tests` `21 passed; 2 failed`),
and `commit::native_commit_waiting_notice::tests` answered `ok. 19 passed; 0 failed` alone and red only in
the whole `commit::` filter,
which is how the planted window was found.
Every control that failed was `1 passed` alone with `--exact`.

A later threaded run of the same baseline measured the step ledger's own module red too:
`transaction::transaction_step_durations::tests` `5 passed; 1 failed`,
where `a_slow_step_names_its_arguments_and_a_short_one_does_not` read no arguments for its step
(`left: None`) because another control's drain had taken it.
Two further runs of `commit::` on that baseline answered `FAILED. 467 passed; 7 failed` and
`FAILED. 464 passed; 10 failed`,
so the count of failures holds near ten while which controls fail moves from run to run.

## The fix for the other four ledgers

Every control that counts one of those figures runs its body in a child of its own,
as the five controls of the preparation-span ledger do (`testing/process_wide_state_child_test_support.rs`).
For `transaction/transaction_step_durations_tests.rs` and `repo_read/git_metadata_tests.rs` that is the step
ledger they read and write.

`source_guard/no_control_reads_the_commit_ledgers_in_the_run_tests.rs` keeps the cause from coming back, with
one requirement per ledger: a control of a test-only source that names `drain_index_builds(`,
`note_index_build(`,
`commit_index_builds::`,
`drain_prepare_parts(`,
`note_prepare_part(`,
`drain_build_parts(`,
`note_build_part(`,
`drain_steps(` or
`note_step(` in code and does not start a child of its own is a finding,
and each requirement names the files that must still read it,
so a scan that found no reader fails instead of passing every tree.
With one control's child starter taken out, it answered
`commit/commit_prepare_part_durations_tests.rs:177 (a_name_with_delimiters_stays_one_member)`.

The two causes of that run that are no ledger have their own answers:

- the two `commit::commit_preparation::tests` controls assert the state of this process's descendants, so
  they own a process (`started_within_a_child_of_its_own` at the time; `run_the_body_in_a_child_of_its_own`
  since issue 6503 moved the body into the child), whose only descendants are their own;
- every control of `native_commit_waiting_notice_tests.rs` that steps a notice takes the two windows before
  it starts (`take_the_windows_a_notice_control_reads`, which states the one order that cannot deadlock a
  threaded run), and `source_guard/notice_controls_take_their_window_tests.rs` refuses a control that names
  `step_started(` without taking a window or owning a child.
  With the helper's call taken out of that control, it answered
  `commit/native_commit_waiting_notice_tests.rs:331 (the_notice_reports_the_programs_that_ran_below_the_step)`.

All three guards share one scan, `source_guard/process_wide_fact_readers_tests.rs`, and issue 6507's guard for
the preparation's steps uses it too,
so the next ledger of this shape writes a table and no new scanner.

Measured after the change and before the merge of `origin/main`,
on this host and threaded:
two runs of `commit::` answered `test result: ok. 474 passed; 0 failed`,
and each module answered
`commit::commit_index_builds::tests` `ok. 11 passed`,
`commit::commit_prepare_part_durations::tests` `ok. 5 passed`,
`commit::commit_build_part_durations::tests` `ok. 5 passed`,
`commit::commit_timing::tests` `ok. 21 passed`,
`commit::commit_preparation::tests` `ok. 23 passed`,
`commit::native_commit_waiting_notice::tests` `ok. 19 passed`,
`transaction::transaction_step_durations::tests` `ok. 6 passed`,
`repo_read::git_metadata::tests` `ok. 11 passed`,
`shadow::shadow_pool::tests` `ok. 29 passed` and
`source_guard::` `ok. 147 passed`.

## After main made the starter take the control's body (issue 6503)

`origin/main` replaced the starter that answered a yes or a no with
`run_the_body_in_a_child_of_its_own(module_path!(), name, || { .. })`,
because a control that read the answer inverted ran its body in the process that answer was written to keep its
state out of.
The merge at `6f081469d5` therefore broke these 28 guards,
and each now hands its body to that call,
with the three guards excusing its name instead of the removed one.
`commit/native_commit_waiting_notice_tests.rs` needed no child either way:
its controls take the two control windows.

Measured on the merged tree,
on this host and threaded:
`commit::` answered `test result: ok. 474 passed; 0 failed` in both runs,
and the modules answered the counts above,
`source_guard::` `ok. 151 passed`,
each of the 34 controls `1 passed` alone with `--exact`,
and `cargo check --locked --offline --all-targets` and `cargo clippy --locked --offline --all-targets -- -D warnings`
finished with no message.

The three guards were shown failing on that tree again:
with one control's body unwrapped,
so it ran in the run's own process,
`source_guard/no_control_reads_the_commit_ledgers_in_the_run_tests.rs` answered
`commit/commit_prepare_part_durations_tests.rs:177 (a_name_with_delimiters_stays_one_member)`;
with the window helper's call taken out of `the_notice_reports_the_programs_that_ran_below_the_step`,
`source_guard/notice_controls_take_their_window_tests.rs` answered
`commit/native_commit_waiting_notice_tests.rs:330 (the_notice_reports_the_programs_that_ran_below_the_step)`;
and with one control of issue 6507's module unwrapped,
`source_guard/no_control_reads_the_preparation_steps_in_the_run_tests.rs` answered
`commit/prepare_work_spans_tests.rs:80 (a_step_of_no_measurable_time_is_stated_as_one_call)`.

## The failed step's note, measured after the merge (issue 6509)

The run this page's counts come from left one control alone,
and the first threaded run of `commit::` after the last merge of `origin/main` failed it:
`test result: FAILED. 473 passed; 1 failed`,
at `commit::git_step_failure::tests::a_note_never_carries_a_path_or_a_line_break`
with `the note just written`.
That is a sixth fact of the same shape,
and one this page had not listed:
`commit/git_step_failure.rs` keeps the note of the last Git step that failed
in a `static LAST: Mutex<Option<FailedStep>>`,
which every step writes,
every step's start clears and every timing record takes.
The mutex makes each call atomic,
not the three calls a control makes,
so a control beside it cleared the note between this control's write and its read.
`source_guard/no_control_reads_the_commit_ledgers_in_the_run_tests.rs` gains a fifth requirement for it
(`clear_failed_step(`,
`note_failed_step(` and
`drain_failed_step(`),
whose files are `commit/git_step_failure_tests.rs` and
`worktree/worktree_clone_conversion_size_read_tests.rs`,
which notes and takes the note inside a child of its own since issue 6493.

Each of the four controls of `commit/git_step_failure_tests.rs` now hands its body to
`run_the_body_in_a_child_of_its_own`,
and with one of them unwrapped,
so that its body ran in the run's own process,
the guard answered
`commit/git_step_failure_tests.rs:84 (a_note_never_carries_a_path_or_a_line_break)`.
Measured after it,
threaded:
`commit::git_step_failure::tests` `ok. 4 passed; 0 failed`,
`commit::` `ok. 474 passed; 0 failed` in both runs,
and `source_guard/no_control_reads_the_commit_ledgers` `ok. 3 passed; 0 failed`.

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

The other five files of the same run are fixed;
this page keeps their counts and causes in the sections above.
Item 3 of `package/git-policy/cli/doc/debt-ledger-2026-11-25.md` carries those counts.

One site of the same shape is measured beside them and stays open:
`repo_read::git_metadata::tests` reads the step ledger and its control owns a child,
but it was `ok. 11 passed` alone before the change.
The step ledger's own module did fail in a threaded run of itself,
`transaction::transaction_step_durations::tests` `5 passed; 1 failed` on 2026-11-26;
the two parts tests above are the sites that failed in the whole `commit::` filter.
