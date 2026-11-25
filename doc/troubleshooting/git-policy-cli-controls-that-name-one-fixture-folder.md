# Two cli-git controls named one fixture folder, so a threaded `cargo test` run failed both

`package/git-policy/cli`'s tests ran as threads of one process on a host without `cargo-nextest`,
and two controls that write the same fixture name shared one folder there.

## Symptom

Three controls failed in `cargo test --locked --offline --lib lock::` at the default threads,
and each passed alone:

- `lock::index_writer_forward::tests::a_git_child_that_a_signal_ended_keeps_its_signal_when_no_scratch_place_is_given`:
  `remove stale fixture: DirectoryNotEmpty`.
- `lock::index_writer_forward::tests::a_git_child_that_a_signal_ended_keeps_its_signal_through_the_coordinated_forward`:
  `NotCoordinated` where `Killed(15)` was expected, and in other runs a panic at
  `testing/test_support.rs:332:32` with `fresh fixture: Os { code: 17, kind: AlreadyExists, message: "File exists" }`.

The third is a different cause and is recorded in
`doc/troubleshooting/git-policy-cli-landing-controls-wait-on-order-not-time.md`,
"The count a threaded run read from another control".

## Root cause

Both forward controls call `coordinated_death_of_a_git_killed_by_sigterm`,
which took `fixture("forward-killed")`.
`testing/test_support.rs`'s `fixture_with_prefix` named the folder
`native-unit-<process id>-<the name the test wrote>` and removed a folder of that name before making it,
so inside one process both controls got one folder, and each one's removal of a "stale" fixture deleted the other's
live one: one control lost its repository (`DirectoryNotEmpty`, `NotCoordinated`) while the other rebuilt it.

## The evidence

`strace -f -e trace=mkdir,mkdirat,rmdir` of the two together, with the test binary started directly, shows two threads
calling `mkdir` on one path:

```text
3966920 mkdir(".../test-fixtures/process-3966913/native-unit-3966913-forward-killed", 0777 <unfinished ...>
3966919 mkdir(".../test-fixtures/process-3966913/native-unit-3966913-forward-killed", 0777 <unfinished ...>
```

One call made the folder and the other returned `EEXIST`, which is the `fresh fixture` panic above;
the trace then shows one thread building `repo/` and `scripts/` under that folder while the other panics.

The same two, three runs of three, one process: `1 passed; 1 failed`, each time.
Each alone: `1 passed`.
Each alone under `--test-threads=1`: `1 passed`.

## The change

Every call now takes a number from one counter for the process and names its folder with it:
`fixture(name)` makes `native-unit-<process id>-<the number this call took>-<name>`,
so two controls that write one name cannot share a folder,
and the "remove stale fixture" branch can only meet a folder an earlier run of this process's number left.
The binary test target's own helper in `binary/binary_support_fixture_directories.rs` had the same naming
(`native-binary-<process id>-<name>`) and takes the same number, for the binary controls that repeat a name
(`binary_commit_timing_tests.rs`'s two `timing-signal` controls).

The names repeated in one executable were measured before the change, so the change was made for all of them:

- `registry-observe-main` (`worktree_copying/worktree_copy_registry_tests.rs`, two controls);
- `merge-recovery-other-branch` (`merge/merge_isolation_recovery_tests.rs`, two controls);
- `service-land-recover-alternate` and `service-land-switched` (`landing/service_land_tests.rs`);
- `work-lease` and `work-given-up` (`landing/service_work_tests.rs`, `landing/work_item_tests.rs`);
- `claim-names` and `claim-vanished` (the lock, mount handler, copying and landing claim controls);
- `timing-signal` (`binary/binary_commit_timing_tests.rs`, two controls).

No call site builds one of those names itself, so no call site needed an edit;
`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
"One fixture folder per test", now states the new shape of both names.

## What the change moved in the module runs kept for it

Each module that repeats a fixture name was run threaded once before the change and once after:
`config::` (142 passed both times), `merge::` (`338 passed`), `worktree_copying::`, `lock::index_writer_forward::`
(`10 passed`), `lock::` (`473 passed` three times) and `landing::`.
`worktree_copying::` moved three controls from failing to passing
(`worktree_copy_stage_claim`'s `a_replaced_destination_is_left_whole_and_reported`,
`a_stage_another_object_took_the_name_of_is_left_alone`, and
`worktree_copy_registry`'s `outside_and_main_worktrees_are_not_applicable`),
one from passing to failing
(`worktree_copy_compare::a_source_written_after_the_stamp_fails_the_final_check`,
whose own failure line `written: ()` says the final check passed after the source was rewritten,
so its stamp comparison and not this naming is what it measures),
and left the rest of its pre-existing btrfs failures as they were.

## What is not claimed

- The controls are not made to pass by any lock, and nothing about the release executable changed.
- A folder a killed run leaves is no longer met by a later run of another process number,
  so a fixture root collects those until something sweeps it:
  that was already true of every fixture name that only one control writes.
