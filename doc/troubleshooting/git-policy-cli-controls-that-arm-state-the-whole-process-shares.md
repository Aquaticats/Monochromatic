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
