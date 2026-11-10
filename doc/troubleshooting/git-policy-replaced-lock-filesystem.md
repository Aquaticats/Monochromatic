# Git-policy CLI at e02be6c8 accepts replacement locks in the measured ordinary CI directories

Note 2026-11-09T16:55:17-05:00:
these citations in this record name paths that no longer exist.
`package/git-policy/cli/src/native/recovery_files.rs` is now
`package/git-policy/cli/src/native/transaction/recovery_files.rs`,
moved 2026-11-03 in `9d630d998e`.
`package/dev-script/task-util` is now `package/monorepo-task/core-exec`,
moved 2026-10-29 in `592d78b143`.

## Symptom

The frozen source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The real archived Git-policy tests reproduce replacement-lock failures on native Depot,
GitHub-hosted Ubuntu 24.04 and Depot GitHub runners.
The emitting tool is `cargo-nextest 0.9.148` running the existing Rust test executable.
The distinct assertions are:

- `real_index_lock::tests::a_replaced_lock_is_never_released`:
  `a replaced lock must not be released: ()`.
- `real_index_lock::tests::an_install_through_a_replaced_lock_is_refused`:
  `a replaced lock must not be written: ()`.
- `real_index_lock::tests::releasing_after_a_landing_leaves_a_replaced_lock`:
  `replacement kept: Os { code: 2, kind: NotFound, message: "No such file or directory" }`.

The corresponding ordinary-directory repeats pass 32,
32 and 32 cases on native Depot;
32,
32 and 31 on Depot GitHub;
and 31,
32 and 31 on standard GitHub-hosted Ubuntu.
Each repeat runs the same selected 34 cases from an enumerated total of 1,956,
with 1,922 filtered skips.
Every repeat on the owned tmpfs fixture passes all 34 cases.

## Root cause boundary

The source requires a replacement to have a different identity,
but the measured unlink/recreate operations can reuse the recorded device and inode.
This is a source identity boundary demonstrated on every measured provider;
filesystem type alone is not the explanation.
No kernel defect or universal ext4 allocation behavior is established.

At `package/git-policy/cli/src/native/real_index_lock.rs:257`,
exclusive creation returns a boolean rather than retaining the opened file:

```rust
//! package/git-policy/cli/src/native/real_index_lock.rs:257,
//! frozen e02be6c8
let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
options.write(true).create_new(true).mode(0o600);
match options.open(path) {
    Ok(_) => return Ok(true),
```

The paths this record names under `package/git-policy/cli/src/native/` were removed on 2026-11-01
in `72e0534cca` (refactor(git-policy-cli):
delete the transaction landing,
recovery,
registry,
replay,
gate and index-writer coordination;
binary tests of the old route retired,
push
expectations still to restate);
no file in the working tree carries them now.
`package/git-policy/cli/src/native/recovery_files.rs` moved on 2026-11-03 to
`package/git-policy/cli/src/native/transaction/recovery_files.rs` in `9d630d998e`
(refactor(git-policy-cli):
move the transaction record modules into a transaction folder).
`mise run //package/monorepo-task/core-exec:research:reuse:lock-portability` is not declared today:
the task lived in `package/dev-script/task-util`,
which became `package/monorepo-task/core-exec` on 2026-10-29 in `592d78b143`,
and that package declares no `research:*` task.

The `RealIndexLock` fields at that file's line 57 retain paths and a `LockIdentity`,
with no open `File` field.
The identity is subsequently read from path metadata at line 300.
The direct release path at line 138 calls `assert_identity()` before removing that path.
Its deciding Unix comparison is at line 218:

```rust
//! package/git-policy/cli/src/native/real_index_lock.rs:218,
//! frozen e02be6c8
if !metadata.is_file()
    || metadata.dev().to_string() != identity.file.device
    || metadata.ino().to_string() != identity.file.inode
{
    return Err(format!("the index lock identity changed: {}", path.display()));
}
return Ok(());
```

The recovery path additionally compares the filesystem identity,
as `package/git-policy/cli/src/native/recovery_files.rs:382` shows:

```rust
//! package/git-policy/cli/src/native/recovery_files.rs:382,
//! frozen e02be6c8
if filesystem != journal.fs_id || device != journal.file.device || inode != journal.file.inode {
    return Ok(Some(format!(
        "Index lock identity changed: {}",
        lock_path.display()
    )));
}
```

The same-filesystem replacement can retain all those compared fields.
The install path opens the current path and compares its metadata at
`package/git-policy/cli/src/native/recovery_files.rs:575`:

```rust
//! package/git-policy/cli/src/native/recovery_files.rs:575,
//! frozen e02be6c8
let (device, inode, _) = metadata_identity(&opened);
if device != journal.file.device || inode != journal.file.inode {
    return Err(RecoveryError(format!(
        "Index lock identity changed: {}",
        lock_path.display()
    )));
}
```

The actual replacement fixture at
`package/git-policy/cli/src/native/real_index_lock_tests.rs:54`
removes the original path and writes different replacement bytes before calling `release()`:

```rust
//! package/git-policy/cli/src/native/real_index_lock_tests.rs:54,
//! frozen e02be6c8
std::fs::remove_file(&path).expect("remove the created lock");
std::fs::write(&path, b"another process's lock").expect("replacement");
let refusal: String = lock.release().expect_err("a replaced lock must not be released");
```

The separate field control retains actual before/after device,
inode,
mode,
size and nanosecond timestamps for equivalent unlink/recreate operations.
All three ordinary-directory controls reused device and inode in all 64 measured pairs;
all three tmpfs controls reused them in none of their 64 pairs.
A positive control writes different bytes to an existing file and verifies that the probe observes its unchanged inode.
These controls support the identity-lifetime explanation;
they do not instrument the exact inode of every failed Rust fixture.

## Verification

The reproducible research harness is commit `8a615939f096ba0c4b0f1a28815b759be719368f`
on `research/depot-reuse-20261018`.
Its package-owned entry point is:

```sh
# package/monorepo-task/core-exec/mise.toml, research/depot-reuse-20261018
mise run //package/monorepo-task/core-exec:research:reuse:lock-portability
```

The preceding `research:reuse:lock-prepare` task admits the diagnostic ZIP and binds the exact prerequisites.
The [reuse evidence](../audit/depot-reuse-strategy-2026-10-18.evidence.json)
retains the complete command arguments,
34-case selection inventories,
JUnit identities,
raw failure output,
clock/inode observations,
mount/unmount receipts and ZIP/member hashes.
The native run is `ptxnhf1ppm`;
GitHub Actions run `39113641724` contains the standard Ubuntu and dependent Depot runner jobs.

- The original diagnostic ZIP SHA-256 is
  `019694511127b9d0a3af6b79f62d1560e34d6c76d9e00ee1c8e4d60f04a723cb`.
- The archive SHA-256 is
  `d6b7edef092cad3c81a88deb293777665e4f38e5b9caf27221a5225c4965d596`.
- Bundled Nextest `0.9.148` SHA-256 is
  `53470cfc9b5cbcd96780db2d348700d76af962467503791f1f8a8824d8967e5c`.
- Git `2.56.0` SHA-256 is
  `d42496dde88b0e5bfbdeb8a32f339601635d7007915ce9a9925ba7f7423bc99e`.

All providers verify the same 1,079 source-file hashes,
root policy,
published linter and bundled tools before running tests.
The GitHub mirror also preserves the original inner ZIP hash.
Each cloud job has four CPUs,
16 GB and a ten-minute bound.
Default Nextest concurrency is preserved;
`--no-fail-fast` lets every selected case run despite failures.
The ordinary fixture reports `statfs` type `61267`;
the owned 256-MiB tmpfs reports `16914836`.
All three unmounts succeed and retain the post-unmount filesystem observation.

The ctime test at
`package/git-policy/cli/src/native/index_lock_evidence_tests.rs:437`
passes in every matched repeat.
Its historical failure remains a separate,
unreproduced boundary;
the inode result is not its cause.

## Verified diagnostic control and tradeoffs

Pointing only the disposable tests' `TMPDIR` at the owned 256-MiB tmpfs passes all nine matched tmpfs repeats.
That changes the fixture filesystem,
uses bounded volatile RAM storage and requires successful privileged mounting plus final unmount.
It supplies a matched filesystem control.
It does not repair lock ownership for production repositories on their existing filesystem.

No application patch or production routing change was made.
A production repair needs a separate design and correctness test for identity across unlink/recreate,
recovery and the existing concurrent-owner cases.

## What does not establish a fix

Installing the exact Git `2.56.0`,
restoring the original tool hashes
and switching between these providers does not eliminate the ordinary-directory failures.
The failure is present with the same externally admitted compiled archive,
so a successful compilation or compiler-cache hit cannot establish a passing runtime test gate.
Changing only the reported filesystem label would not be a control;
these jobs verify the actual mount,
run the original tests and retain the raw metadata.

## Upstream filing decision

Existing firstparty reports are
[#1136](https://github.com/Aquaticats/Monochromatic/issues/1136),
[#1137](https://github.com/Aquaticats/Monochromatic/issues/1137) and
[#1138](https://github.com/Aquaticats/Monochromatic/issues/1138).
Their earlier sources and symptoms are retained separately from this exact frozen cohort.
All-state repository searches for `"replaced lock"` and `"inode"` and the full issue bodies were inspected.

- External fault is not established:
  the deciding comparison belongs to this repository and reproduces on both providers.
- A vendor fix requirement is not established:
  no failing kernel or Nextest operation is identified.
- The supported-use-case question does not justify a vendor report without such a defect.
- External contribution policies are not evaluated because no external issue is proposed.
- Maintainer willingness cannot supply the missing external fault evidence.
- No production source fix was prototyped in this research-only campaign.

Nothing is filed upstream and no duplicate firstparty issue is opened.

## Where the boundary stands on main (2026-11-10T14:57:44-05:00)

The modules the three reports name are gone.
`package/git-policy/cli/src/native/real_index_lock.rs` (639 lines) and
`package/git-policy/cli/src/native/real_index_lock_tests.rs` (385 lines) were deleted in
`72e0534cca` (2026-11-01T20:58:11-05:00),
with the landing,
recovery,
registry,
replay,
gate and index-writer coordination that released and installed through a real index lock
(`git show --stat 72e0534cca`).

The wrapper now releases no index lock without first opening it and judging it through that descriptor.
What replaces the frozen `RealIndexLock` release is
`package/git-policy/cli/src/native/landing/service_index_unheld_removal.rs`,
which opens the lock at `:243`,
judges it through that descriptor at `:248`,
and compares the name against the open file just before the unlink at `:302`.
That comparison is `same_file`,
at
`package/git-policy/cli/src/native/landing/service_index_reclaim.rs:250` to `:255`:
device and inode of the name against device and inode of the open file,
so a replacement whose inode number the file system reused cannot pass while the descriptor keeps the judged
inode alive.
Its control is
`package/git-policy/cli/src/native/landing/service_index_unheld_removal_tests.rs:198`,
`a_lock_replaced_between_the_scan_and_the_unlink_is_never_removed`,
which replaces the lock between the scan and the unlink,
expects `Unheld::Changed`,
and reads the other program's lock back.

The removal left under `package/git-policy/cli/src/native/lock/` takes a stale reference lock apart at
`package/git-policy/cli/src/native/lock/stale_ref_lock_retire.rs:84` to `:108`:
the file is moved aside with `RENAME_NOREPLACE` and only then compared,
and `package/git-policy/cli/src/native/lock/stale_ref_lock_identity.rs:29` to `:42` defines that comparison
over device,
inode,
size,
birth time and modification time.
A freed inode number handed to a live Git's new lock therefore no longer passes as the judged file,
which was the deciding comparison of the frozen failures above.

Two controls pin that behavior in a disposable repository:
`lock::stale_ref_locks::tests::a_lock_that_kept_the_judged_device_and_inode_is_not_taken` rewrites the judged lock
in place so that its device and inode stay the judged ones,
and expects the removal to refuse it,
and `lock::stale_ref_locks::tests::a_lock_taken_again_after_the_judgement_is_put_back` puts a recreated lock back
whole.
The reference-lock control fails when both identity comparisons of `stale_ref_lock_retire.rs` are narrowed to device
and inode,
which is the comparison the frozen source made.

Both areas run together on 2026-11-10:
`mise run //package/git-policy/cli:native:test:host -- -E 'test(/stale_ref_lock|service_index_unheld_removal/)'`
ran 73 tests with 73 passed,
6133 skipped.
