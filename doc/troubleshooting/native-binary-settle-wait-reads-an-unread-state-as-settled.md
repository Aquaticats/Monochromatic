# The binary suite's wait for the commit service answered "settled" from a queue it never read (monochromatic-git-policy-cli 0.6.1-alpha.42)

## Symptom

`package_repair::quiet_runs_the_repair_and_says_nothing_about_success` fails inside a whole
`native_binary` suite,
on this machine and in the container gate,
and passes when it runs alone:

```text
FAIL [   0.862s] ( 716/1215) monochromatic-git-policy-cli::native_binary package_repair::quiet_runs_the_repair_and_says_nothing_about_success
thread 'package_repair::quiet_runs_the_repair_and_says_nothing_about_success' (1133848) panicked at src/native/binary/binary_package_repair_tests.rs:311:5:
assertion `left == right` failed
  left: 0
 right: 1
monochromatic-git-policy-cli-service: the package state of .../one was relinked relink_ms=2 at_ms=1794741978650
```

The stand-in `pnpm` left no record when the control looked (`left: 0`),
while the service's own line in
the same window says the package state was relinked:
the tool ran after the control had already read
its result.
The gate's copy of the same failure shows the same shape with a service that lived 47 ms.

## Root cause

The control reads the state the service leaves,
so its wrapped command first waits for the service's
after-commit work (`package/git-policy/cli/src/native/binary/binary_support_observation.rs`),
and that
wait was:

```rust
let work: PathBuf = git_dir.join(git_policy_cli::work_item::WORK_DIRECTORY);
let open = |state: &str| -> usize { return std::fs::read_dir(work.join(state)).map_or(0, Iterator::count); };
while open("pending") + open("claimed") + open("recovering") > 0 { ... sleep(20ms) ... }
```

Two readings answered "settled" without evidence.

`map_or(0, ...)` reads a listing it could not make as "no record there":
the count of every unread
state directory is zero,
so the loop ends at once.
The doc above the function says it "waits only
where a work directory exists",
but nothing checks that,
so an absent or unreadable queue is
indistinguishable from an empty one.

The folder is spelled by the join alone,
while the wrapper reads a Git directory's data folder under
whichever of the two spellings exists
(`package/git-policy/cli/src/native/foundation/kept_names_in_the_git_directories.rs`):

```rust
let prefix: String = format!("{}/", DATA_FOLDER.current);
if let Some(rest) = relative.strip_prefix(prefix.as_str()) {
    return data_folder_for_reading(directory).join(rest);
}
```

and `data_folder_for_reading` answers `DATA_FOLDER.earlier` when only that spelling exists.
A
repository whose queue an earlier release made keeps its records under `<earlier>/work`,
which the
wait never listed.

A third reading is closed by the same change:
a record moves between the three states
(`package/git-policy/cli/src/native/landing/work_item.rs`,
`take_entry` for a claim,
`requeue_with` for the return of an expired claim),
so one pass over the three
states,
listed one after another,
can see none of them while the service holds the record.
The states are listed in the order a claim carries a record through them,
so a claim cannot be missed by one pass at all,
and a record an expired claim returns to `pending` is caught by a second reading in a row.

The two failures were not reproduced.
What is established is that these were the only ways this wait
could answer "settled" with the record present,
and that the record was present:
the service's own
line says it relinked the worktree,
and the control found no record before it.

## Verification

Version:
this worktree's build of `0.6.1-alpha.42`.
Harness (the package's own task):

```sh
mise run //package/git-policy/cli:native:test:binary:host -- --no-fail-fast \
  'settle_the_after_commit_work::'
```

Patterns that fail,
against the wait as it stood:

```text
FAIL [0.369s] ... settle_the_after_commit_work::a_state_directory_that_cannot_be_listed_is_reported
  a state directory nothing can list must be reported, not read as settled
FAIL [0.670s] ... settle_the_after_commit_work::a_queue_left_under_the_earlier_folder_name_is_waited_for
  the wait answered settled while .../repo/.git/cli-git/work held a record
```

Patterns that pass cleanly,
with the change:

```text
PASS [0.492s] settle_the_after_commit_work::a_state_directory_that_cannot_be_listed_is_reported
PASS [0.816s] settle_the_after_commit_work::a_queue_left_under_the_earlier_folder_name_is_waited_for
```

The control that first failed,
alone,
at `CPUQuota=25%` and at `CPUQuota=100%`:
`PASS` in every run
(`7.096s`,
`1.506s`,
`1.505s` and `0.744s`,
`0.472s`,
`0.422s`),
and the whole `package_repair` module
at `CPUQuota=25%` passed three times in a row (`16 tests run: 16 passed`).
The failure needs a
neighbour in a whole suite,
so the module was not where it came from.

## Verified workarounds

The wait now reads every folder the wrapper's own reader of a Git directory names,
refuses to answer from a listing it could not
make,
reads the three states in both orders a record moves through them
(`records_between_the_orders`),
and calls the work settled only on two looks in a row,
a pause apart,
with the listing passed in so a control moves a record between two real listings
(`package/git-policy/cli/src/native/binary/binary_support_observation.rs`,
`settle_service_work_list_with`).
Tradeoff:
a repository whose queue genuinely cannot be listed now fails the
control with the directory named,
instead of passing on a state it never saw;
a call that finds nothing makes six listings instead of three;
and its settled case costs one pause.

## A record in motion and a second queue

The review of 2026-11-16 found two more ways this wait answered "settled" without evidence,
each reproduced against the branch's tip
(`/home/user/temp/agent/change-reviews/git-policy-cli-settle-wait-of-the-binary-controls-one-function-from-two-repairs/review.md`).

**One recovery missed by two looks.**
A record an expired claim returns from `claimed` to `pending` through `recovering`
(`package/git-policy/cli/src/native/landing/work_item_recovery.rs`,
`requeue_with`),
and one pass listed the three states at three instants in the order a claim carries a record,
so the return ran the other way:
a record that left `recovering` before that listing and reached `pending` after the next was in none of them.
Two looks a pause apart did not help,
because the same record could be missed one step on in each;
the reproduction returned after exactly two empty looks
and printed `BUG REPRODUCED` with the record still in `pending`
(`reproductions/one_recovery_missed_by_two_consecutive_empty_readings.rs`).

**A queue under the other name never listed.**
A repository can hold both data-folder spellings at once,
and this wait resolved one folder with `path_for_reading`,
which answers the current spelling when both exist,
so a record open under `.git/cli-git/work` was never listed;
the reproduction returned settled in about 20 ms with the earlier queue's record still open
(`reproductions/current_folder_selected_while_earlier_queue_is_open.rs`).

Both are controls now
(`package/git-policy/cli/src/native/binary/binary_settle_the_after_commit_work_tests.rs`),
and both failed against the wait as it stood:

```text
FAIL [ 1.124s] ... settle_the_after_commit_work::a_recovery_moved_at_every_gap_is_never_read_as_settled
  the wait answered settled while its record was still in `pending`: the recovery's renames fell at the gaps after listings 2 and 4 of the 6 this control places them at, and the record's work settles only after listing 7
FAIL [ 1.899s] ... settle_the_after_commit_work::a_queue_under_either_name_is_waited_for_when_both_exist
  the wait answered settled while .../repo/.git/cli-git/work held a record
```

The wait now reads every queue the Git directory holds (`work_directories`),
and reads the three states twice in one look,
once in the order a claim carries a record and once in the order an expired claim returns it
(`OPEN_STATES`,
`RETURNING_STATES`,
`ORDERS_A_LOOK_READS`).
A record that only moves one way is missed by one order and seen by the other,
so an answer no longer turns on which way the service happened to carry the record.
The one case left is a record that changes state three times inside a single look,
which the second look then has to miss as well;
no order of listings can rule that out,
because a record is always in the state a look is not listing at that moment.
The seam moved with it:
`settle_service_work_list_with` gives a control one state directory's listing (`records_in`),
so a control moves a record at the gap between two real listings
instead of standing in for a whole look,
and `LISTINGS_IN_A_LOOK` says how many listings a look takes.

## What does not work

- Waiting for `pending`,
  `claimed` and `recovering` to be empty by one pass each:
  the claim's staging
  rename makes that a reading of three instants,
  not one.
- Counting only `pending`:
  a claimed record is still work the control must wait for.
- Sleeping a fixed time after the add:
  the service can take as long as a tool's own bound.

## Upstream filing artifact

Not applicable:
this is this repository's own test harness,
not an external tool's defect.
