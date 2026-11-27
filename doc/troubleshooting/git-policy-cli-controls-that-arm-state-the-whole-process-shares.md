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

The record control of
`package/git-policy/cli/src/native/invocation/started_as_git_tests.rs`
joined them for a second reason,
below:
its child inherited the record the test process held,
so running in a child was not enough on its own.

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

That starter is `run_the_body_in_a_child_of_its_own` since 2026-11-26 (issue 6503):
it takes the body to run in the
child,
so the refusal now names a call that runs a body rather than one a control could read as a flag.

With the `refuse_outside_the_child` call taken out of `a_control_holds_the_read`
and the same control still running beside the others,
the run passes (`1 passed; 0 failed`):
the control set a hold of the whole process and nothing said so,
which is what the refusal is for.

The refusal and the child's own answer also have permanent controls,
in `package/git-policy/cli/src/native/testing/process_wide_state_child_test_support_tests.rs`:
one is `#[should_panic]` on the refusal's own sentence,
and the other is a body that runs in its child and asserts that the child saw its marker.

## A control's child read the record the test process held, so the run failed now and then

Issue 6514.
The same family and the same debt ledger item,
with a second mechanism:
the chain record a wrapper leaves for its Git is one per process,
and the child a control runs in inherits that record from the test process.

### Symptom

`cargo test --locked --offline --lib --features test-only-switches started_as_git:: -- --skip guest --skip _arm_tests`
on 2026-11-26,
default threads.
Of the thirteen runs this round made before the change,
eleven ended with a result this round kept:

- seven runs failed with `14 passed; 1 failed`,
  and every one of them failed
  `started_as_git::tests::a_record_of_this_process_is_read_even_when_an_older_marker_sits_above`;
- four runs passed with `15 passed; 0 failed`.

The other two were cut off in the output of the first batch and are counted as neither.

The same control alone (`-- --exact invocation::started_as_git::tests::a_record_of_this_process_is_read_even_when_an_older_marker_sits_above`)
passed three times of three before the change,
which is what issue 6514 reported from split round 1.
The one `invocation::` run this round made before the change failed three controls,
all three in `invocation::git_version_cache::tests` (`Text file busy`),
and the record control passed in it;
split round 1's own four runs of `invocation::` failed the record control in two.

### Root cause

`this_chain` makes the record once per process,
from the descriptor the process inherited
(`package/git-policy/cli/src/native/invocation/chain_starts_record.rs:172`):

```rust
pub fn this_chain() -> Option<&'static ChainRecord> {
    static RECORD: std::sync::OnceLock<Option<ChainRecord>> = std::sync::OnceLock::new();
    return RECORD.get_or_init(ChainRecord::for_this_start).as_ref();
}
```

The record's memory file carries no close-on-exec flag,
because every child of a wrapper has to inherit it
(`chain_starts_record.rs:371`),
so a child of the test binary inherits the record the test process holds.
The lookup answers with the first entry of `/proc/self/fd` whose link carries the record's name
(`chain_starts_record.rs:321`):

```rust
for entry in listing.flatten() {
    let Some(number) = entry.file_name().to_str().and_then(descriptor_number) else {
        continue;
    };
    let Ok(target) = std::fs::read_link(entry.path()) else {
        continue;
    };
    if carries_record_name(target.as_path()) {
        return Some(number);
    }
}
```

The control made a record of its own beside that one and wrote its marker into it
(`started_as_git_tests.rs` at `e2e7df813e`,
lines 656 and 662):

```rust
let inherited: i32 =
    crate::chain_starts_record::create_descriptor().expect("a record is made");
let written: String =
    marker_value(StartMode::Replace, std::process::id(), identity.as_str(), 2)
        .to_string_lossy()
        .into_owned();
assert!(
    crate::chain_starts_record::write_marker_for_controls(inherited, written.as_str()),
    "the record holds the start above"
);
```

So the process held two records with one name,
the lookup answered whichever number came first,
and `record_marker`
(`package/git-policy/cli/src/native/invocation/started_as_git.rs:258`)
read the marker of the test process instead of the control's.
Which number came first was not fixed:
it was 3 for the inherited record and 4 for the control's whenever the test process held one,
and the control's own number when it held none,
so the control passed in the runs where the test process had made no record yet.

### The evidence

A temporary print in the child body,
three lines from the failing run:

```text
MEASURE child 1866279 parent 1866268
MEASURE inherited before: Some(3)
MEASURE own: 4
MEASURE chain descriptor 3 marker Some("1866268\u{1f}linux:24292103\u{1f}replace\u{1f}1")
MEASURE written 1866279linux:24292104replace2
```

The record the lookup answered with named process 1866268,
which is the child's parent and the process that wrote it,
while the control's marker named the child 1866279 and sat at descriptor 4.
The child then failed its own assertion
(`started_as_git_tests.rs:690`):
`the record names this process, which started itself again`.

### The change

The body closes every record it inherited before it makes its own,
so the process holds the record the control writes and the lookup can answer nothing else
(`started_as_git_tests.rs:649`):

```rust
if crate::process_wide_state_child_test_support::started_within_a_child_of_its_own(
    module_path!(),
    "a_record_of_this_process_is_read_even_when_an_older_marker_sits_above",
) {
    return;
}
crate::process_wide_state_child_test_support::refuse_outside_the_child(
    "the chain record this process holds",
);
while let Some(inherited) = crate::chain_starts_record::inherited_descriptor() {
    // SAFETY: `close` takes one descriptor this process holds, reads no memory of it, and cannot fail.
    let _closed = unsafe { libc::close(inherited) };
}
```

It runs in a child of its own through the shared support the 6487 round landed,
whose refusal names the starter,
so an inverted guard fails at its first line
instead of closing the record of a test process that shares it with the whole run.
The control's own variable (`CLI_GIT_RECORD_ABOVE_CHILD`) is gone with it.

No shipped behaviour changed:
`this_chain`,
the lookup,
`create_descriptor` and the guard are as they were,
and the record still continues the chain when its start is this process or a live process above it,
as `package/git-policy/cli/SPEC.md` says
("every start of real Git writes the marker it gives that Git into the file,
and the file records the path it chose,
and every child inherits the descriptor").

Eight runs of the same module after the change answered `15 passed; 0 failed`,
and so did four more after the guard was taken out and put back,
and three runs of `invocation::` answered `394 passed; 0 failed`.

### What the guard is worth

With the close loop taken out and the control running beside the others,
the run fails in the runs where the test process already holds a record:
seven of eleven,
failing in the child at the assertion that names the precondition
(`started_as_git_tests.rs:680`):

```text
assertion `left == right` failed: this process holds the record it made and no other, which is the one the guard reads
```

With the loop put back,
four runs answered `15 passed; 0 failed`,
as nine runs had before it was taken out.
The control asserts the precondition on every run,
so a future change that leaves a second record in the child fails at that line instead of
failing the guard's own assertion now and then.

## The child starter answered a flag a control could read inverted (issue 6503)

`testing/process_wide_state_child_test_support.rs` answered
`started_within_a_child_of_its_own(module, name) -> bool`,
and each of its callers held that answer in an `if`.
A caller that wrote `if !started_within_a_child_of_its_own(...)` returned at once in the child and ran the body in the
parent,
which is the process the body was written to keep its state out of.
Nothing in the run said so:
measured on 2026-11-26 on this host,
with one control of `foundation/lazy_runtime_tests.rs` read inverted,
`cargo test --locked --offline --lib --features test-only-switches foundation::lazy_runtime::tests:: -- --skip guest --skip _arm_tests`
answered `test result: ok. 15 passed; 0 failed; 1 ignored` in both of two runs,
and the same control alone with `--exact` answered `1 passed; 0 failed`,
while its body built the runtime every other test of the run then shared.
With every guard of that module inverted the run answered `FAILED. 14 passed; 1 failed; 1 ignored`,
at `this control is the first need of a runtime in its own process`,
which is where two bodies in one process meet.
The bodies that set the room or the bound fail loudly instead:
`foundation::blocking_jobs_within_the_budget::tests` with one guard inverted answered
`FAILED. 10 passed; 1 failed`,
at the refusal of the room a control gives the process.

The starter takes the body now,
as `run_the_body_where_no_thread_can_start` and `run_the_body_under_umask` have since issue 6494,
so no caller holds an answer to invert,
and `this_process_is_the_child` is private to the starter's file.
`source_guard/only_the_process_wide_starter_reads_its_marker_tests.rs` refuses a source that names the marker variable
outside that file,
and its planted fixture shows the scan still finds a read.
With a read of the marker planted in `foundation/lazy_runtime_tests.rs`,
that guard's ban answered `FAILED. 2 passed; 1 failed` at `foundation/lazy_runtime_tests.rs:37`,
and with the line gone again it answered `ok. 3 passed; 0 failed`.
After the change,
each module the round touched answered its own run with no failure except
`worktree::worktree_trash_entries::tests`,
whose `the_removals_take_places_of_the_processs_bound_on_blocking_jobs` answers
`FAILED. 7 passed; 1 failed` on the tree before the change as well
(issue 6477's shape,
and that control guards no child either way),
and the four changed controls run alone answered `1 passed; 0 failed` each.
Issues 6494,
6503 and the three above are one shape:
a process-wide fact belongs to a process of the control's own,
and the control hands over the body that sets it.

## Not covered

- The other modules of this crate hold the same debt
  (`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`,
  item 3,
  items 3a to 3g);
  this change moves one module's controls,
  and the rounds that fixed 6487,
  6486 and 6484 moved others.
- Issues 6483 and 6485 are still open on `origin/main` at `73f1091ce0`.
- The `invocation::git_version_cache::tests` failures are a separate fault,
  in a module this change does not touch,
  and they are issue 6516.
  They are not a flake of the wider filter:
  twelve runs of `invocation::git_version_cache::` alone,
  in two batches of six,
  failed four of them
  (three times `12 passed; 1 failed`,
  and once with three failing fixtures named)
  and answered `13 passed; 0 failed` in the other eight.
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

That is the sentence as the refusal gave it on 2026-11-26 at the measurement above.
The starter is `run_the_body_in_a_child_of_its_own` since the same day (issue 6503):
it takes the body to run in the child,
so the refusal names a call that runs a body rather than one a control could read as a flag,
and each of this round's controls hands its body to it.

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

### The same runs after `origin/main` moved the starter

`origin/main` was 47 commits ahead when this round's second half began,
and issue 6503 had replaced the starter:
`started_within_a_child_of_its_own` answered a yes or a no,
and every caller held that answer in an `if`,
so the ten guards of this round called a function that no longer existed.
Each now hands its body to `run_the_body_in_a_child_of_its_own(module_path!(), name, || { .. })`,
and no test's body changed.

The host carried other work while these ran (load average 7.78 on 10 cores).
Measured on the merged tree:

- two threaded runs of `push::child_watch::`:
  `test result: ok. 47 passed; 0 failed; 1 ignored`,
  in 21.38 s and 21.38 s;
- the same module with `--test-threads=1`:
  `test result: ok. 47 passed; 0 failed; 1 ignored`,
  in 41.35 s;
- `push::child_watch_output_watched_on_the_runtime::`:
  `test result: ok. 7 passed; 0 failed`,
  in 0.58 s;
- `foundation::lazy_runtime::`:
  `test result: ok. 15 passed; 0 failed; 1 ignored`;
- `service::service_runtime_build_report::`:
  `test result: ok. 5 passed; 0 failed`;
- `source_guard::`:
  `test result: ok. 145 passed; 0 failed`,
  in 5.20 s;
- each of the ten controls this round changed,
  alone with `--exact`:
  `test result: ok. 1 passed; 0 failed` each.

The two guard proofs were made again on the merged tree,
with one control of `service::service_runtime_build_report::tests` taken out of its child:

- the body in the parent fails at its first line,
  `test result: FAILED. 0 passed; 1 failed`,
  with the refusal's sentence,
  which now ends at `run_the_body_in_a_child_of_its_own`;
- with `refuse_outside_the_child` also taken out of `a_control_stands_in_for_a_late_runtime`,
  the same run passes,
  `test result: ok. 1 passed; 0 failed`:
  the control set a stand-in of the whole process and nothing said so,
  and the refusal is what catches it.

After the last merge of `origin/main` (`a2f41fc43e`),
the module answered `test result: ok. 47 passed; 0 failed; 1 ignored` in 21.36 s threaded,
and in 41.41 s with one test thread;
`source_guard::` answered `test result: ok. 145 passed; 0 failed`.

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

## Third round, 2026-11-27T03:16:37-05:00: the add timing record's own controls (issue 6532)

### Symptom

`package/git-policy/cli`,
two threaded runs of the record's own module,
default threads,
`CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1 cargo test --locked --offline --lib --features test-only-switches worktree::add_timing::tests -- --skip guest --skip _arm_tests`:

- both runs answered `test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 10758 filtered out`,
  failing `worktree::add_timing::tests::work_on_other_threads_is_summed_by_name`
  at `src/native/worktree/add_timing_tests.rs:235`
  with `left: Null`,
  `right: 2`,
  which is issue 6532's line.

The module alone had been reported passing by the round that first saw the failure in a whole `worktree::` run,
where the control failed once in 14 runs.

### Root cause

`WORK` (`src/native/worktree/add_timing_steps.rs`) is one `Mutex<Option<Vec<(String, Duration)>>>` for the whole process,
which `begin` replaces with an empty list and `finish` takes.
Six controls of the module's own tests begin a recording,
so the one that sums the work of other threads reads the list its two worker threads pushed into,
and any recording another control begins or ends between its `begin` and its `finish`
leaves its line without that work:
the other's `begin` replaces the control's list and the other's `finish` takes the replacement.

### The evidence

A window planted between that control's `begin` and its `finish`
(test-only and uncommitted,
reverted before the change),
with one line printed where a recording begins,
one where `finish` names the work it took,
and two around the window:

```text
PROBE begin by "worktree::add_timing::tests::work_on_other_threads_is_summed_by_name"
PROBE window opens for "worktree::add_timing::tests::work_on_other_threads_is_summed_by_name"
PROBE begin by "worktree::add_timing::tests::work_without_a_recording_is_forgotten"
PROBE finish by "worktree::add_timing::tests::work_without_a_recording_is_forgotten" took 0 work entries: Some([])
PROBE window closes for "worktree::add_timing::tests::work_on_other_threads_is_summed_by_name"
PROBE finish by "worktree::add_timing::tests::work_on_other_threads_is_summed_by_name" took 0 work entries: None
thread 'worktree::add_timing::tests::work_on_other_threads_is_summed_by_name' panicked at src/native/worktree/add_timing_tests.rs:244:5:
assertion `left == right` failed
  left: Null
 right: 2
test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 10758 filtered out
```

So the control that moved the record is named:
`work_without_a_recording_is_forgotten`.
Before the window was planted the same control failed twice in two runs,
so the failure did not need the window;
the window only made the interleaving visible and named the other control.

### The change

Six controls of `src/native/worktree/add_timing_tests.rs` now run their body in a child of their own
(`run_the_body_in_a_child_of_its_own`,
which is in `src/native/testing/process_wide_state_child_test_support.rs`):
every one that begins a recording,
which is `steps_nest_through_the_thread_recorder`,
`a_missing_common_directory_is_not_created`,
`records_append_as_lines`,
`work_on_other_threads_is_summed_by_name`,
`work_without_a_recording_is_forgotten` and
`an_add_the_ladder_does_not_serve_records_the_route_it_took`.
The three controls that only drive a `Recorder` by hand,
and `a_step_without_a_recorder_returns_its_body`,
set nothing that belongs to the process,
so they stay beside the module's other threads.

That the child really runs the body is measured and not assumed:
`strace --follow-forks --trace=execve` on one of them shows this test binary started twice,
and the second start carries `--exact` with the control's path in this module
and `--nocapture` beside `--test-threads=1`.

No release code changed:
`WORK` is still one list for the process,
and `begin` and `finish` still replace and take it.
The notice and the record are as they were,
and one process still runs one add's record,
which is what the shipped wrapper does.

After the change the same filter answered
`test result: ok. 13 passed; 0 failed`
in each of three runs,
taking 0.04 s then 0.09 s then 0.07 s.

### Not covered

- No `source_guard` ban covers this record yet.
  The scan every such ban is built on reads each test-only source of the crate,
  and seven controls of four other files still begin a recording in the run's own process:
  three in `worktree/add_standing_notice_tests.rs` (issue 6533),
  two in `worktree/worktree_clone_tests.rs`,
  one in `worktree/worktree_ladder_finish_tests.rs`
  and one in `worktree/worktree_ladder_target_tree_skip_tests.rs`.
  A ban is worth adding when those own their processes,
  and until then the guide states the rule for a new control
  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).
- The container gate was not run:
  this round ran the module on this host and the whole `source_guard` module,
  and a whole-library threaded run is not run here
  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).
- A whole `worktree::` threaded run was not made either:
  the module's own filter shows the failure and its fix,
  and four split rounds' 14 kept runs of that filter are what the issue reports from.

## Fourth round, 2026-11-27T05:24:15-05:00: a control's copy of a file was held open by another thread's child (issue 6542)

### Symptom

`merge::merge_isolation_recovery::tests::saves_and_holders::a_copy_a_program_still_holds_open_is_not_removed`
failed once in the first threaded run of the `merge::` filter
on the untouched tree of the round for branch `refactor/git-policy-cli-split-wrapper-merge-files-over-the-line-budget-1`
(`test result: FAILED. 349 passed; 1 failed; 0 ignored; 0 measured; 10403 filtered out; finished in 86.37s`),
and passed in that round's three later runs of the same code (`350 passed; 0 failed` each).
The container gate cannot see this shape:
nextest runs every test in a process of its own.

The line that failed was the control's last assertion,
"the copy goes once nothing holds it open",
and the notice it quoted said "a program still holds the copy open" without naming one,
so the report left out which process kept the copy.

### Root cause

The control opens the copy with `std::fs::File::open` and hands that descriptor to the program that holds it open
(`sleep 120`) as its standard input.
Between that open and the holder's `exec` the descriptor is one of the test process's,
so a `fork` of any thread of a threaded run duplicates it into the forking child,
where it stays open until that child execs.
The holder scan the recovery makes (`lock/proc_fd_holder_scan.rs`) reads `/proc/<pid>/fd` of every process and
matches the copy's device and inode,
so such a child is a program holding the copy,
and the copy stays.

The control kills its own holder and reaps it,
which is not the whole set of holders:
under host load another test's thread forks inside that window,
and the child it forks still holds the descriptor when the second recovery scans.
The descriptor is a fact the whole process shares,
so the defect is the control's
(`AGENTS.md`'s SUF) and not the host's.

### The evidence

Planted on 2026-11-27,
uncommitted and reverted before the fix:

- a five second sleep between the copy's `File::open` and the holder's `spawn`,
  which is the window a loaded host widens when it preempts the thread that opened the descriptor;
- a second control in the same file that forks a child every second whose `pre_exec` sleeps a second before its
  `exec`,
  so each such child holds every descriptor this process has open across the whole window.

Two runs of one build answered the issue's failure both times:

```text
test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 10763 filtered out; finished in 32.16s
... still holds the copy open: PID 3905138 (merge::merge_is), so the copy stays where it is
test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 10763 filtered out; finished in 30.57s
... still holds the copy open: PID 4024259 (merge::merge_is), so the copy stays where it is
```

A process's `comm` is the name of the thread that forked it,
cut to fifteen bytes,
so each named process is a child of another test's thread and not the control's own `sleep 120`,
whose `comm` is `sleep`,
which the control's own line names on every run.
Without the plant the same module answered
`test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 10763 filtered out; finished in 0.62s`.

### The change

1. The body runs in a child of this test binary that holds this one test
   (`process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own`),
   so no other test's thread forks under the descriptor the control opens.
2. The wait for the holder's end is `test_support::wait_until`,
   judged on the run's own progress instead of a quiet host's clock,
   and it fails naming the process it waited for and the window it gave
   (`STILL_WORKING_WINDOW`),
   where `Child::wait` waited with no bound at all.
3. The notice names every holder by process number and command (`PID 1234 (sleep)`),
   through the lock module's `holder_name`,
   and the control asserts the line names the program it started,
   which is the report issue 6542 asks for first.

### What the guard is worth

With the plant on the tree of the fix,
both runs pass,
since the plant's forks land in the parent that runs no copy:

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10763 filtered out; finished in 32.93s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10763 filtered out; finished in 42.39s
```

The control the fix changed,
nothing planted:
`test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 10763 filtered out; finished in 0.45s`.

The naming that issue 6542 asks for is a guard that fails without its code:
with the holders taken out of `say_the_copy_stays_held_open`
(it said "a program this command can read still holds the copy open" and named nothing),
the control answered
`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10770 filtered out; finished in 0.69s`
at its own assertion,
`the line names the process that holds the copy, which is the one this control started (742769)`,
and the line it quoted named no process.
With the code back the same control passes.

### Not covered

- The container gate was not run for this change:
  a branch merges without one,
  and one gate runs on `main` for each batch of merges
  (`doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`).
- The other controls of this crate that hand a descriptor to a program were read and left as they are:
  `merge_isolation_carry_place_tests.rs` starts such a program and never runs a second removal after it ends,
  so a child that inherited the descriptor cannot change its answer.
- No measurement of a whole-library threaded run was made,
  as no round makes one
  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).
- This section is unapproved and non-final:
  it records what one round read,
  planted,
  measured and changed,
  and the owner declares the work complete.

## Fifth round, 2026-11-27T05:32:05-05:00: the trash removal place control (issue 6534)

### Symptom

`worktree::worktree_trash_entries::tests::the_removals_take_places_of_the_processs_bound_on_blocking_jobs`
failed in 12 of the 14 kept threaded runs of the whole `worktree::` filter
that four split rounds made on 2026-11-26,
and passed in the other two (issue 6534).
The module's own filter alone passed,
so the failure needed a run in which another control had already made the process's bound.

Measured again on this round's tree before the change,
with
`CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1 cargo test --locked --offline --lib --features test-only-switches worktree:: -- --skip guest --skip _arm_tests`,
default threads:

- `test result: FAILED. 1011 passed; 4 failed; 1 ignored; 0 measured; 9755 filtered out`,
  the control failing at `src/native/worktree/worktree_trash_entries_tests.rs:340`
  with `left: [false, false, false, false]` and `right: [true, true, true, true]`;
  the other three failures were `worktree::add_standing_notice::tests`,
  which issue 6533's own round owns.
- the module alone answered `test result: ok. 8 passed; 0 failed`,
  which is why the control's own filter never showed it.

### Root cause

The control calls `hold_the_only_place()` (`testing/parallel_test_support.rs`),
which stores `ONE_PLACE` in `HELD_TO`,
then holds its place with a job of the process's blocking pool.
`BUDGET` is one `OnceLock` for the process,
and it reads `HELD_TO` only where it is first made
(`foundation/blocking_jobs_within_the_budget.rs`),
so a hold that comes after another control's thread has made the bound changes nothing:
the holder takes one place of seven,
and the removals take the others and start at once.
This is the same mechanism issue 6477 measured for `repair::package_repair_toolchain`.

### The evidence

A print at the budget's own creation,
planted uncommitted and reverted before the change,
read in the failing `worktree::` run:

```text
MEASURE the budget is made now: heldTo=18446744073709551615 allowed=7
```

`heldTo` was `usize::MAX`,
so the budget was made before any control held it,
and it had the seven places the CPUs allow.
The same print in the module's own filter read
`the budget is made now: heldTo=1 allowed=1`,
which is the process the control was written for.

### The change

The control's body runs in a child of its own
(`process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own`),
as its four neighbours in the module do,
and it reads `budgeted_blocking_jobs()` once the hold is taken,
failing at its own line when the bound is not the one place it holds.

No release code changed:
`hold_blocking_jobs_to` and the budget are as they were,
and a hold that arrives after the budget is made still changes nothing,
which issue 6477 owns for the shared helper.

### What the guard is worth

With the child start taken out and the read left in place,
the module alone still passed `test result: ok. 8 passed; 0 failed`,
because the control's own hold made the budget first there,
and the whole `worktree::` run failed at the new line:

```text
assertion `left == right` failed: this process's bound on blocking jobs has 7 places, and the control holds one of them, so a removal takes another and starts at once: the body needs a process of its own, whose bound is the one place it holds
```

### After the change

- the module threaded:
  `test result: ok. 8 passed; 0 failed; 0 ignored`,
  in 0.27 s.
- the whole `worktree::` filter threaded:
  `test result: FAILED. 1011 passed; 4 failed; 1 ignored`,
  with all four failures in `worktree::add_standing_notice::tests`
  and this control passing.
- the same filter with that module skipped:
  `test result: ok. 1007 passed; 0 failed; 1 ignored`,
  in 25.12 s.
- after merging `origin/main` at `c0c4eeeb88`,
  with that module's own fix merged in between,
  the whole `worktree::` filter answered
  `test result: ok. 1015 passed; 0 failed; 1 ignored`,
  in 18.23 s.
- `source_guard`:
  `test result: ok. 176 passed; 0 failed`.
- `cargo fmt --check` printed nothing.
- `cargo check --locked --offline --all-targets` finished clean,
  and so did `cargo clippy --locked --offline --all-targets -- -D warnings`.

### Not covered

- The container gate was not run,
  and a whole-library threaded run is not run here
  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).
- The other callers of `hold_the_only_place` are unchanged:
  `lfs`,
  `subvolume`,
  `worktree_copying`,
  `landing`,
  `repair` and
  `worktree`,
  and issue 6477 owns the helper's own fix.
- No `source_guard` ban refuses a control that holds the process's bound outside a child;
  `hold_blocking_jobs_to` is the one setter of that fact that still has none.
