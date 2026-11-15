# The binary suite's wait for the commit service answered "settled" from a queue it never read (monochromatic-git-policy-cli 0.6.1-alpha.42)

## Symptom

`package_repair::quiet_runs_the_repair_and_says_nothing_about_success` fails inside a whole
`native_binary` suite, on this machine and in the container gate, and passes when it runs alone:

```text
FAIL [   0.862s] ( 716/1215) monochromatic-git-policy-cli::native_binary package_repair::quiet_runs_the_repair_and_says_nothing_about_success
thread 'package_repair::quiet_runs_the_repair_and_says_nothing_about_success' (1133848) panicked at src/native/binary/binary_package_repair_tests.rs:311:5:
assertion `left == right` failed
  left: 0
 right: 1
monochromatic-git-policy-cli-service: the package state of .../one was relinked relink_ms=2 at_ms=1794741978650
```

The stand-in `pnpm` left no record when the control looked (`left: 0`), while the service's own line in
the same window says the package state was relinked: the tool ran after the control had already read
its result. The gate's copy of the same failure shows the same shape with a service that lived 47 ms.

## Root cause

The control reads the state the service leaves, so its wrapped command first waits for the service's
after-commit work (`package/git-policy/cli/src/native/binary/binary_support_observation.rs`), and that
wait was:

```rust
let work: PathBuf = git_dir.join(git_policy_cli::work_item::WORK_DIRECTORY);
let open = |state: &str| -> usize { return std::fs::read_dir(work.join(state)).map_or(0, Iterator::count); };
while open("pending") + open("claimed") + open("recovering") > 0 { ... sleep(20ms) ... }
```

Two readings answered "settled" without evidence.

`map_or(0, ...)` reads a listing it could not make as "no record there": the count of every unread
state directory is zero, so the loop ends at once. The doc above the function says it "waits only
where a work directory exists", but nothing checks that, so an absent or unreadable queue is
indistinguishable from an empty one.

The folder is spelled by the join alone, while the wrapper reads a Git directory's data folder under
whichever of the two spellings exists
(`package/git-policy/cli/src/native/foundation/kept_names_in_the_git_directories.rs`):

```rust
let prefix: String = format!("{}/", DATA_FOLDER.current);
if let Some(rest) = relative.strip_prefix(prefix.as_str()) {
    return data_folder_for_reading(directory).join(rest);
}
```

and `data_folder_for_reading` answers `DATA_FOLDER.earlier` when only that spelling exists. A
repository whose queue an earlier release made keeps its records under `<earlier>/work`, which the
wait never listed.

A third reading is closed by the same change: a claim moves a record out of `pending` and into
`claimed` through `recovering`
(`package/git-policy/cli/src/native/landing/work_item.rs`, `take_entry`), so one pass over the three
states, listed one after another, can see none of them while the service holds the record.

The two failures were not reproduced. What is established is that these were the only ways this wait
could answer "settled" with the record present, and that the record was present: the service's own
line says it relinked the worktree, and the control found no record before it.

## Verification

Version: this worktree's build of `0.6.1-alpha.42`.
Harness (the package's own task):

```sh
mise run //package/git-policy/cli:native:test:binary:host -- --no-fail-fast \
  'settle_the_after_commit_work::'
```

Patterns that fail, against the wait as it stood:

```text
FAIL [0.369s] ... settle_the_after_commit_work::a_state_directory_that_cannot_be_listed_is_reported
  a state directory nothing can list must be reported, not read as settled
FAIL [0.670s] ... settle_the_after_commit_work::a_queue_left_under_the_earlier_folder_name_is_waited_for
  the wait answered settled while .../repo/.git/cli-git/work held a record
```

Patterns that pass cleanly, with the change:

```text
PASS [0.492s] settle_the_after_commit_work::a_state_directory_that_cannot_be_listed_is_reported
PASS [0.816s] settle_the_after_commit_work::a_queue_left_under_the_earlier_folder_name_is_waited_for
```

The control that first failed, alone, at `CPUQuota=25%` and at `CPUQuota=100%`: `PASS` in every run
(`7.096s`, `1.506s`, `1.505s` and `0.744s`, `0.472s`, `0.422s`), and the whole `package_repair` module
at `CPUQuota=25%` passed three times in a row (`16 tests run: 16 passed`). The failure needs a
neighbour in a whole suite, so the module was not where it came from.

## Verified workarounds

The wait now reads the folder by the wrapper's own rule, refuses to answer from a listing it could not
make, and takes two readings at once before calling the work settled
(`package/git-policy/cli/src/native/binary/binary_support_observation.rs`, `open_records` and
`settled_by_one_reading`). Tradeoff: a repository whose queue genuinely cannot be listed now fails the
control with the directory named, instead of passing on a state it never saw.

## What does not work

- Waiting for `pending`, `claimed` and `recovering` to be empty by one pass each: the claim's staging
  rename makes that a reading of three instants, not one.
- Counting only `pending`: a claimed record is still work the control must wait for.
- Sleeping a fixed time after the add: the service can take as long as a tool's own bound.

## Upstream filing artifact

Not applicable: this is this repository's own test harness, not an external tool's defect.
