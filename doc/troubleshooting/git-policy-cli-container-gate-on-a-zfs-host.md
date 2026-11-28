# The git-policy-cli gate's containers on a ZFS host: five mount behaviours fail 185 native tests, and Clippy's step fails with E0514

Investigated on 2026-11-19T17:00:41-05:00 in
`.claude/worktrees/pi-git-policy-cli-container-gate-on-the-zfs-host` (branch
`docs/git-policy-cli-container-gate-on-the-zfs-host`,
from `main` `cff64adfa0`),
against the release
`0.6.1-alpha.44` snapshot in the gate's own image,
under podman 6.2.0 on CachyOS,
kernel 7.3.5.

The gate is `mise run //package/git-policy/cli:native:test:container`
(`bin/test-native-container.ts`),
whose substantive step is the whole suite and Clippy in two
containers built from one source image.

## Symptom

The gate's suite fails on this host,
and its Clippy step does not compile:

```text
Summary [1053.402s] 9173 tests run: 8988 passed (8 slow), 185 failed, 28 skipped
...
Clippy failed with status 101: Compiling proc-macro2 v1.0.107
...
error[E0514]: found crate `monochromatic_jsonc_edit` compiled by an incompatible version of rustc
   = note: the following crate versions were found:
           crate `monochromatic_jsonc_edit` compiled by <unknown rustc version>: /work/package/git-policy/cli/target/debug/build/monochromatic-jsonc-edit/89d1ae7d2fcfb4cc/out/libmonochromatic_jsonc_edit-89d1ae7d2fcfb4cc.rmeta
   = help: please recompile that crate using this compiler (rustc 1.100.0 (0ef1bc386 2026-11-09) (Arch Linux rust 1:1.100.0-1)) (consider running `cargo clean` first)
```

On the previous host,
a Btrfs machine,
the same gate failed six of 9,173 tests
(`doc/handover/cli-git-rust-implementation.md`,
the gate on `0f07c4f0ee`).
This host fails 185.

The mise task cannot be run as written in the main checkout either:
its first step
`cargo-shadow lean records target/verification` fails with status 127,
because `cargo-shadow` is not on the
checkout's `PATH` (`mise ls` prints nothing).
Every run below is the task's second step,
`node bin/test-native-container.ts`,
or the image it builds,
run the way the gate runs it.
As read on 2026-11-19 at about 23:20 (-05:00),
the locked tools are present in the toolchain root
`~/.local/share/monochromatic-dev/toolchain` (`cargo-monochromatic-cargo-shadow` among them):
what the main checkout lacks is its `.cache/toolchain-bin` link folder.
`mise run toolchain:install` in a worktree reported `0 installed, 66 unchanged, 0 failed` and made that folder there.
Whether the main checkout gets the same folder waits for the project,
because the plan's step 1 records that nothing is installed on this host until the mpm-shadow change.

## The host, as measured

- `/home` is `zpcachyos/ROOT/cos/home`:
  `utf8only=on`,
  `normalization=formD`,
  `casesensitivity=sensitive`,
  `acltype=posix`,
  `longname=off`
  (`zfs get -H -o property,value ...`).
  ZFS 2.4.4.
- `podman info --format '{{.Store.GraphDriverName}} {{.Store.GraphRoot}}'` answers
  `overlay /home/user/.local/share/containers/storage`,
  both on that dataset.
  That store's own
  configuration also sets `overlay.mountopt=nodev`.
- Inside a container the repository tree is a **user-space** overlay:
  `stat -f --format=%T` prints
  `fuse`,
  `/proc/self/mountinfo` names the type `fuse.fuse-overlayfs`,
  and `statfs` gives
  `f_type = 0x65735546` (`FUSE_SUPER_MAGIC`).
  On the Btrfs host the same tree was a kernel `overlay`.
- The gate image (`localhost/git-policy-native-test:development`,
  built 2026-11-19T19:30Z from
  `localhost/git-policy-native-test-base:cfc6cb4ce2c8-97221c29742b`) carries rustc 1.100.0
  (Arch 1:1.100.0-1),
  cargo 1.100.0,
  git 2.56.1,
  cargo-nextest 0.9.151.
- The host's `rustc --print sysroot` answers
  `/home/user/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu` (rustc 1.102.0-nightly,
  5df8c02af),
  whose `bin/` holds `cargo`,
  `rustc`,
  `rustdoc` and the gdb wrappers and **no `cargo-clippy` and no
  `clippy-driver`**.
  `/usr/bin/rustc` is a symlink to `/usr/bin/rustup`.
- The repository's locked toolchain is `nightly-2026-11-16` (`toolchain.lock.json`,
  "rust"),
  installed
  here with `cargo-clippy` and `clippy-driver` (rustc 1.102.0-nightly,
  5f4effd57).

Four mount facts follow from those two settings,
and every cause below is one of them or a
consequence:

- a name that is not UTF-8 is refused by the dataset (`utf8only=on` with `normalization=formD`);
- the fuse mount records no birth time (`stat --format=%w` prints `-`);
- the fuse mount's type is in neither of the product's two "local file systems" tables;
- Git reads a FUSE mount as a remote working tree.

## Cause A: a name that is not UTF-8 is refused (108 tests)

Reproduction,
on the host with no container:
`mkdir` of a name whose last byte is `0xFF` under the
investigation scratch folder (on `zpcachyos/ROOT/cos/home`) fails
`errno=84 Invalid or incomplete multibyte or wide character`;
the same call under `/dev/shm`
succeeds.
Inside the gate's image the same pair repeats against `/work/package/git-policy/cli/target/tmp`
(fails,
`errno` 84) and `/dev/shm` or a `--tmpfs` mount (both succeed),
so the refusing layer is the
dataset,
reached through podman's storage,
and not FUSE.

The product and its fixtures create names Git itself accepts,
such as `not-utf8-\xFF` and
`caf\xe9.md`,
and those creations fail.
Two shapes of failure appear:
the test's own create fails
(`worktree file: Os { code: 84, ... }`),
or the fixture's Git command fails
(`git ["branch", "lost-\xFF"] failed: fatal: cannot lock ref ... Unable to create ... Invalid or
incomplete multibyte or wide character`).

The 108 tests this cause covers include every one whose name says so,
for example
`candidate::candidate_store::tree_tests::a_pushed_tree_lists_a_name_that_is_not_utf8_exactly`,
`commit::capture_invocation::tests::a_repository_whose_name_is_not_utf8_is_captured`,
`policy::policy_catalog_specifiers::tests::path_cases::a_manifest_below_a_name_that_is_not_utf8_is_still_checked`,
`shadow::shadow_create::tests::a_ref_name_that_is_not_text_does_not_stop_the_shadow`,
`worktree::worktree_trash::safety::a_non_utf8_worktree_is_named_by_a_record_and_emptied_by_its_path_bytes`
and the eleven `event_path_commit` tests.
The full list is `cause-A-non-utf8-name.txt`,
in the
investigation folder named at the end of this doc.

Positive control:
all 108 pass once the fixture root is a tmpfs (the workaround below).

### Cause A is answered by the controls themselves

Since 2026-11-24 a control that writes such a name no longer depends on the root the run happened to
give it.
It asks its own fixture root whether the name can be made there
(`src/native/testing/fixture_root_taking_any_name.rs`),
and where the answer is no it starts the same
test inside a user and mount namespace of its own,
mounts a tmpfs at that fixture root,
and runs there.
The root stays the run's own,
nothing is written under `/tmp` or `/dev/shm`,
and no mount of this run
reaches the host's namespace,
because the mount lives in the namespace the control made for itself.
Where the root already takes the name,
which is the gate's answer,
the control runs as it always did
and starts no namespace at all.
A host that lets no unprivileged user namespace fails such a control
unless the run acknowledges it with `CLI_GIT_TEST_NO_USER_NAMESPACES=1`,
and then prints one line
naming the control that did not run,
so a run in which it never ran is visible.

Measured on 2026-11-24 on this host,
one process per test:
the 69 unit controls and the 17 binary
controls of this cause that failed before the change passed after it (`keep/per-test-before`,
`keep/per-test-after`,
`keep/binary-per-test-before` and `keep/binary-per-test-after` in
`~/temp/agent/non-utf8-name-controls-on-a-utf8only-home-2026-11-24/`;
scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
One control of the cascade,
`worktree_clone_mount::slot_identity_store::tests::a_name_that_is_not_utf8_refuses_the_listing_as_a_foreign_entry`,
had printed `SKIPPED` and passed without running;
it now runs and proves the refusal.

Twenty-two controls were missed by those rounds and still failed here on 2026-11-25:
two in
`commit::capture_invocation`,
four in `candidate::candidate_store` (three of them beside the
`staged_changes` helper and one through it from `lifecycle_tests`),
three in
`landing::service_land_follow`,
five in `worktree_copying`'s stage-claim and journal controls,
two in
`repo_read::repo_read_query`,
one in `shadow::replay_merge`,
three in `shadow::shadow_ref_write`,
one
in `lock::stale_ref_lock_message`,
and the binary control
`service_commit::a_commit_whose_changed_paths_would_not_fit_one_line_lands_anyway`.
Each now asks
`in_a_fixture_root_taking_any_name` first,
and each was red before and green after on this host,
one
process per test:
`keep/red-before-the-two-controls.log`,
`keep/search-before-fix.log`,
`keep/search-before-fix-helpers.log`,
`keep/module-tests-threaded-failures-alone.log`,
`keep/every-test-one-process-each.log`,
`keep/green-after-the-fix.log`,
`keep/green-after-the-late-two.log`,
`keep/green-after-the-nineteenth.log` and `keep/green-after-the-shadow-three.log` in
`~/temp/agent/non-utf8-capture-invocation-controls-on-a-utf8only-home-2026-11-25/`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

### The scanner's package met the same cause (issue 6491)

`package/forbidden-strings/cli`'s embedding control makes a rule file whose name is not valid UTF-8 under
`CARGO_TARGET_TMPDIR`,
which is on the dataset above,
and met this cause on 2026-11-27.
Its own answer and its red and green runs are recorded in
`package/forbidden-strings/cli/doc/embedding-control-on-a-file-system-that-refuses-a-name-that-is-not-utf8.md`;
this section keeps only the fact that the refusing layer is the same one described above.

### The inner half's Git holds no capability (issue 6459)

Recorded 2026-11-25T07:55:39-05:00 by the round `any-name-fixture-root-git-without-capabilities`,
on branch `fix/git-policy-cli-any-name-fixture-root-git-without-capabilities`.
Not approved and not final.

A control re-executed into a namespace of its own is root there,
and the kernel gives the root of a user namespace the whole bounding set in permitted and effective
at every `exec` (`handle_privileged_root` in `security/commoncap.c`).
A `git commit` that control runs in a fixture repository hardlinks a `0444` loose object,
and with `fs.protected_hardlinks = 1` the hardlink check asks for `CAP_DAC_OVERRIDE`
once the mode bits refuse the write (`may_linkat` and `safe_hardlink_source` in `fs/namei.c`).
That Git held the capability,
so AppArmor's `git` profile was asked,
refused it and logged it,
while the link itself succeeded on the owner check.
That is why the audit log holds `git` `dac_override` denials in these controls' windows and none
in the gate's;
the cause and its records are in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
subsection
`### The git dac_override records are the any-name fixture's Git (issue 6459)`.

Since 2026-11-25 the inner half sets `SECURE_NOROOT` once it has mounted the tmpfs
(`src/native/testing/fixture_root_taking_any_name.rs`),
so each program it starts holds no capability of the namespace,
as an ordinary account's does,
and the inner half keeps its own for the fixtures it makes root's.
The control that holds this is
`testing::fixture_root_taking_any_name_of_unit_tests::tests::a_program_started_from_the_any_name_root_holds_no_capability`,
which reads a child's own `/proc/self/status`:
it is red before the bit and green after it.
No control changes what it tests,
and no mount leaves the namespace the control made.

### How the controls of this cause are found

A grep for the name of a guard finds the controls that already have one,
not the ones that need it.
Three searches find the rest of them,
and each one caught controls the others did not:

- Read every byte-string literal in the crate's sources and decode it as UTF-8:
  a literal that does
  not decode is a byte string the home dataset refuses.
  This found 145 controls that hold one and do
  not ask the guard.
- Read every test body for any byte over 127 at all,
  whether an escape in a text literal,
  a byte-string
  literal or a bare `0x` number pushed into a name.
  This found 193,
  and it is what catches a name built
  by `name.push(0xff)`.
- Find each helper whose own body holds such a byte and makes a path component,
  then each control that
  calls one without a byte of its own.
  This found eleven controls,
  and only this search caught three of
  them (`repo_read::repo_read_query::tests::an_offset_table_version_git_words_is_left_to_git`,
  `repo_read::repo_read_query::tests::the_index_extensions_git_reads_silently_are_answered` and
  `candidate::candidate_store::tests::reader_starts_only_for_blob_content`),
  because their name comes
  from a helper that stages it.

A control on the union of those three lists is then run on the host,
one process per control,
with
nothing set:
a control that makes such a name fails there with `errno` 84 and one that does not passes.
That run is the answer,
and the searches are only how its candidates are found.
A control that needs a
`btrfs` fixture root refuses for that reason first and is no part of this cause.

Two shapes escape all three searches,
and only a run over every control finds them:

- A helper that holds the byte but makes no path itself,
  so nothing marks it and its caller holds no
  byte of its own:
  `awkward_components()` served one control,
  `awkward_names()` served one,
  and the
  name `staged_changes` stages reached a control in another file of the module.
- A `const` at module scope holding the bytes,
  which no search over a control's own body sees:
  `NON_TEXT_TAG` and `NON_TEXT_BRANCH` in `shadow_ref_write_tests.rs` served three controls.

So the last step is every test of the crate's three binaries run once,
one process per test,
with no
standard input,
on this host:
11201 tests,
of which four failed because they made a name this data set
refuses (the one `errno` 84 failure and three Git refusals of `refs/tags/nonutf-\xFF`).
A `cargo test`
of the whole suite in one process is not that run:
it hangs on
`commit::commit_index::tests::interactive_selection_succeeds_without_changes`,
which Git answers through
standard input that `nextest` gives no test.

## Cause A2: one test writes through a scratch helper the fixture root does not move (1 test)

This was a defect of ours,
and it is fixed:
the helper now builds its directory under the run's own
fixture root,
so `CLI_GIT_TEST_ROOT` moves it with every other test's fixtures.
As it stood:

`landing::service_claim_watch::tests::a_path_is_kept_byte_for_byte` failed with `errno` 84 under
the tmpfs workaround too,
because its own helper bypassed the knob:

```rust
// package/git-policy/cli/src/native/landing/service_claim_watch_tests.rs:77
fn scratch(name: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("claim-watch-tests").join(format!("{name}-{}", std::process::id()));
```

Every other test file builds its root from `fixture_root()`
(`src/native/testing/test_support.rs`,
`FIXTURE_ROOT_VARIABLE`),
or from the binary face's
`fixture_root()` (`src/native/binary/binary_support_fixture_directories.rs`),
and both read
`CLI_GIT_TEST_ROOT`.
This one wrote under the crate's `target/`,
which stays on the dataset.
The
tmpfs workaround landed as `ddec538f27` and carried the helper with it,
and the control now runs on
either root,
inside a namespace of its own where the root refuses the name.

## Cause B: the mount records no birth time (29 tests)

Inside the image,
`stat --format=%w` of a file under `/work/package/git-policy/cli/target/tmp`
prints `-`,
and the same call on `/dev/shm` or a `--tmpfs` mount prints a time.

The product proves a left-behind lock is gone by comparing birth times
(`src/native/lock/lock_birth_times.rs`),
and the worktree-copy stage decides whether a stage was left
by an interrupted command by the same proof.
With no birth time the answer is "unproven",
and the
tests assert the proven answer.
Two wordings appear:

```text
this file system cannot show that <lock>~pid.lock was born after <lock>.lock: the lock is None and the PID file None
nothing shows what payload is (creation time is not available for the filesystem), so nothing below it was examined
```

The 29 tests are the eleven `lock::abandoned_lock_proving_tests`,
the five
`worktree_copying::worktree_copy_stage_claim` reap controls,
the eleven `killed_holder_lock` binary
controls,
`lock::lock_birth_times::tests::the_hidden_and_coarse_arms_report_what_other_file_systems_do`
and `worktree_copy_concurrent::an_add_says_nothing_about_a_lock_a_live_program_holds_in_another_worktree`
(`cause-B-birth-time.txt`).
Three of the five stage controls assert an outcome without printing the
reason;
they belong to this group because their module's other two print it.

Positive control:
all 29 pass on a tmpfs fixture root.

## Cause C: the mount's type is in neither local file system table (28 tests)

The product reads a descriptor of another process's file only when the mount's type is on a list of
file systems whose attributes are in the kernel:

```rust
// package/git-policy/cli/src/native/process/open_file_of_a_descriptor.rs:61
const READ_IN_MEMORY_FILE_SYSTEMS: [&str; 7] = ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "zfs", "bcachefs"];
```

and it judges a left-behind reference lock only on a local file system:

```rust
// package/git-policy/cli/src/native/lock/stale_ref_lock_judge.rs:99
pub(crate) const LOCAL_FILE_SYSTEMS: [&str; 8] = ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "overlay", "zfs", "bcachefs"];
```

The mount's `/proc/<pid>/mountinfo` type is `fuse.fuse-overlayfs`,
which is on neither list,
so a lock
whose holder is a live process reads as unproven and is cleared,
a live holder is not seen,
and a copy
a program holds open is removed:

```text
unresolvable: ["PID 28169 fd 3: its mount is a `fuse.fuse-overlayfs` file system, whose attributes are not in the memory of this kernel, so the file is not read"]
Unproven("it sits on a `fuse.fuse-overlayfs` file system, which a Git on another machine may share")
the scan never saw the holder 50772
```

The 28 tests are the ten `lock::stale_ref_locks`,
`lock::index_lock_holders`,
`lock::index_holder_wait`,
the three `process` controls (`process::open_file_of_a_descriptor` and
two `process::process_use`),
`merge::merge_isolation::repository_tests`
(now `merge::merge_isolation_round::repository_tests`,
since the split of 2026-11-27 moved the test file),
the three
`merge::merge_isolation_recovery`,
`merge_never_fails::a_merge_that_meets_a_holder_proven_alive_saves_itself_at_once`,
two `landing::service_index_lock`,
`landing::service_index_unheld_removal`,
two
`landing::service_land_reference_lock` and the three `wait_notice` binary controls
(`cause-C-mount-type.txt`).

Reading `/proc/<pid>/fd` of a descriptor of a file on this mount works (measured:
the link resolves on
the fuse mount,
on `/dev/shm` and on a `--tmpfs`),
so the refusal is the product's list,
not the
kernel's ability.

Positive control:
all 28 pass on a tmpfs fixture root,
whose mount type is `tmpfs`.

## Cause D: Git refuses `core.fsmonitor` on a FUSE mount (10 tests)

Reproduction inside the image,
with a repository whose `core.fsmonitor` names a program that appends
to a marker file,
on the fuse mount:
`git status` prints
`warning: remote repository '<repo>' is incompatible with fsmonitor` and the program does not run.
On
a `--tmpfs` mount the same repository runs the program and prints no warning.
On the fuse mount with
`-c fsmonitor.allowRemote=true` the program runs,
so the mount and not the program is the cause.

Git's source explains the wording:
`check_remote()` refuses a working directory that
`fsmonitor__is_fs_remote()` calls remote unless `fsmonitor.allowRemote` is true
([`fsmonitor-settings.c` lines 45-58](https://github.com/git/git/blob/f7de81456940dd6f3274e461149bba12965de609/fsmonitor-settings.c#L45-L58)),
and on Linux "remote" includes `FUSE_SUPER_MAGIC`
([`compat/fsmonitor/fsm-path-utils-linux.c` lines 42-57](https://github.com/git/git/blob/f7de81456940dd6f3274e461149bba12965de609/compat/fsmonitor/fsm-path-utils-linux.c#L42-L57)):

```c
static int is_remote_fs(unsigned long f_type)
{
	switch (f_type) {
	case CIFS_SUPER_MAGIC:
	...
	case FUSE_SUPER_MAGIC:
		return 1;
```

Both files were read at commit `f7de81456940dd6f3274e461149bba12965de609` through the GitHub REST API,
which returns the bytes of that commit;
the numbers above are that read.
A rendered-page check was
attempted and the browser could not start in this session (see "What does not work").

The ten tests (`cause-D-fsmonitor.txt`) are `invocation::hooks_off::tests::{a_forwarded_start_runs_the_core_fsmonitor_program,
the_wrappers_own_start_never_runs_the_core_fsmonitor_program}`,
`repo_read::repo_read_config_typed::tests::{the_query_readers_equal_git_under_a_user_level_fsmonitor_true,
the_query_readers_equal_git_when_the_monitor_cannot_start}`,
`repo_read::repo_read_index::tests::an_index_with_the_monitor_and_untracked_cache_extensions_is_read`,
`binary_hooks_off_escape::{a_forwarded_command_runs_the_core_fsmonitor_program,
a_wrapped_commit_never_runs_the_core_fsmonitor_program}`,
`native_git_bound::{a_native_git_command_that_never_answers_ends_at_its_bound_and_names_it,
a_native_git_read_ended_at_its_bound_does_not_wait_for_a_hook_s_descendant}` and
`service_landing_state::a_native_git_read_that_never_answers_ends_at_the_harness_bound_and_names_it`.
The four `hooks_off`/`hooks_off_escape` controls exist to prove that the wrapper *does* run the
person's monitor and the wrapper's own Git does not,
so a monitor that never runs in either arm makes
both halves fail;
the two bound controls use a monitor program that never answers to prove a bound,
and Git answers at once because it never starts that program.

Positive control:
all ten pass on a tmpfs fixture root.

## Cause E: the copy flow makes no `ioctl` where the file system is neither Btrfs nor overlayfs (2 tests)

```text
copy lists calls its flow never makes, so faulting them would test nothing: ["ioctl"] (measured [("ioctl", 0)])
```

`COPY_KILL_CALLS` (`src/native/binary/binary_fault_trace_tests.rs:1041`) lists `ioctl` because on
Btrfs and on a kernel overlay it carries the `FICLONE` that fills each staged file.
The efficient
worktree-copy method is chosen by file system:
Btrfs and overlayfs clone,
every other file system
lets Git check the worktree out (`src/native/health/doctor_filesystem.rs:255`,
and the ladder the
message names).
A `fuse.fuse-overlayfs` mount is neither,
so the flow never calls `ioctl`,
and the
control that faults every call in that list fails.
The second test,
`fault_trace::a_worktree_copy_killed_at_each_ioctl_leaves_a_recoverable_repository`,
is the same
failure for the `ioctl` entry alone.

A tmpfs fixture root does not fix these two:
tmpfs is not on the cloning list either (measured,
both
arms still fail).
The two ways out are a fixture root on a Btrfs or overlay mount,
or a change to what
the kill list demands on such a file system.
The two tests are `cause-E-reflink.txt`.

## Cause F: three single behaviours of the mount

Each of these fails alone,
for its own reason,
and each passes on a tmpfs fixture root.

- `index::index_reset_file::tests::the_file_mode_under_a_default_acl_equals_git`:
  a directory with a
  default ACL of `u::rwx,g::---,o::---` gives a file mode `0644` on this mount and `0600` on tmpfs
  (measured with `setfacl --default --modify ...` and one create on each).
  The dataset has
  `acltype=posix`,
  so the ACL is accepted and then not applied to the new file,
  and the test sees
  Git's index at `0664` where it expects `0660`.
- `worktree::worktree_completion::tests::an_error_that_is_not_a_conflict_fails_the_inspection`:
  the
  test builds a 300-byte name and expects an error that is not a conflict.
  `lstat` of that name
  answers `ENOENT` on this mount and `ENAMETOOLONG` on tmpfs (measured from a Rust probe running the
  two calls the product makes),
  and `is_conflict_error`
  (`src/native/worktree/worktree_completion_file_inspection.rs:99`) reads `NotFound` as a conflict.
- `transaction::transaction_git::tests::input_files_leave_nothing_behind`:
  the input file is created,
  opened and unlinked,
  and the product then lists the scratch directory and finds one entry
  (`left: 1`).
  `strace` shows the `unlink` returns 0 before that listing,
  and the same directory is
  empty when listed afterwards (measured with `CLI_GIT_KEEP_FIXTURES=1`).
  A file created,
  opened,
  unlinked and listed in one process leaves nothing on this mount in a Python probe,
  and its link
  count reads 1 where tmpfs reads 0;
  which layer keeps the entry in the product's listing (the
  kernel's FUSE directory cache or fuse-overlayfs's own) was not isolated.

## Cause G: tests that fail only in a busy full run

Four tests failed in one full run and passed in another run of the same build:
`package_repair::pnpm_runs_in_the_new_worktree_only` (failed in the full suite,
passed in both
filtered runs of the same list),
`package_repair::quiet_runs_the_repair_and_says_nothing_about_success`
and `fault_trace::a_disk_that_fills_during_a_commit_leaves_a_recoverable_repository` (failed on the
LFS branch's build,
passed on `main`'s) and
`push::bounded_push::tests::a_push_that_floods_and_ends_keeps_its_status_and_says_its_output_was_cut`
(failed on `main`'s full suite,
passed in three filtered runs).
Two of these run `pnpm`,
and the
flood control decides a process's ending from a short look;
both are the shape that a loaded host
moves.
They are not causes of the gate's failure on this host,
and none of them names a mount fact.

One more control has this shape and an issue of its own:
`fault_trace::a_merge_killed_at_each_pwrite64_leaves_a_recoverable_checkout` failed in the gate of 2026-11-23T04:12-05:00 (run `native-uN7Nqs`) after 78.850 s with
"native Git /usr/bin/timeout --signal=KILL 60 /usr/bin/git rev-list --count --merges HEAD … was ended at its 60 s bound:
the control was waiting for Git to answer".
It is issue 6397.
This document's cause N changes the wait for the commit service's after-commit work,
and that control never calls it:
`binary_fault_trace_tests.rs` starts the wrapper under `strace` and asks Git through `support::git`,
and neither that file nor `binary_fault_run.rs` calls `observe`,
`run_wrapped` or `settle_service_work`.

One more control had this shape,
failed twice,
and has a cause of its own.
`worktree_removal_held_commit::a_merge_that_lands_a_branch_keeps_the_held_commit_of_the_worktree_it_removes`
failed in the gate of 2026-11-21T13:25:00-05:00,
whose other new failures were of one binary control file,
and that control's fixture holds no LFS key.
It passed in the gate of 2026-11-21T13:55:00-05:00 and in four filtered runs of that image,
and it failed again in the gate of 2026-11-22T16:51:52-05:00.
The second failure is cause N's opening quotation.
The two service lines in that journal are the fixture teardown's,
not the failure's,
which cause N's section "The fixture's drop,
and the journal's two service lines" shows;
whether the first failure had the same cause is not shown,
because that gate's journal is not kept.
Cause N holds the evidence,
the fix and the controls for the second.

### The removal of a worktree read another process as a holder (issue 6475)

`worktree::worktree_remove_hand_over::tests::the_service_leaves_a_worktree_git_would_keep_and_says_whose_sentence_refused_it`
failed in the gate on `main` at `132843d9d1` with the holder scan's sentence where it expects Git's own:
"process 155499 belongs to this account,
works elsewhere,
and the files it holds open could not be read".
The service's own removal reads every process of the account to see whether one works in the worktree
(`worktree_process_inside.rs`),
so a process another test of the same container started after this fixture's worktree was made keeps that worktree,
and the control's verdict was decided by whatever ran beside it.
The release scan is not the defect:
keeping the worktree while a young process of this account cannot be read is its documented rule
(`doc/planning/git-policy-cli-cleans-up-after-its-own-worktrees.md`,
"A process can hide its working folder from its own account",
and `SPEC.md`,
"The removal guard leaves out the command's own chain of parents").

The failure was reproduced on this host on 2026-11-25 by starting one process that hides its working folder
(`PR_SET_DUMPABLE` 0 and no exec after it) at the moment a fixture worktree appears,
which the scan refuses in the same way;
the sentence then names the working folder,
because the kernel refuses `/proc/<pid>/cwd` and `/proc/<pid>/fd` with the same check
(`fs/proc/base.c`,
`call_proc_get_link`).
The control now hands the service's call a process table of its own fixture
(`delete_subvolume_of_with`,
`test_support::an_empty_process_table`),
and the landed sweep's own service call does the same,
while `delete_subvolume_of` keeps reading this machine's `/proc`.

## Cause H: the two standing failures

`repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone`
and `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing` fail here
and failed on the Btrfs host (`doc/handover/cli-git-rust-implementation.md`,
the standing three,
of
which `owed_push` passed on this host).
Both complain that a scratch directory named
`cli-git-lock-holders` is left behind.
They fail with the fixture root on tmpfs as well,
so no mount
change touches them,
and they are not this host's doing.

### Cause H diagnosed, and fixed, on 2026-11-25

The gate on `main` at `64a71a4c9b` failed both again (issues 6466 and 6468).
Two causes,
both in the
controls,
and neither needs a mount fact:

- Every wrapper command writes `<common-git-dir>/cli-git-lock-holders/<pid>.<digest>` before it starts
  any Git since `74b1a8c0a0`,
  and a later command's pass removes the record once every process it names
  is gone (`lock/lock_holder_records.rs`,
  `lock_holder_record_reads.rs`'s `sweep`).
  Both controls read
  the whole `.git` directory after one command,
  so both read that record;
  the crate had already decided
  it says nothing about what a command did to a repository,
  in
  `binary_commit_form_refusal_tests.rs`'s `everything_under`,
  which skips the same folder.
  Reproduced on
  this host:
  one `log` in a fixture repository leaves the folder with one `self` line,
  a second `log`
  leaves its own,
  and the refused commit of the `service_no_manager` control leaves one with the `self`
  line alone.
- The naming control also compared `git count-objects -v`'s whole text,
  and its `size` field sums the
  `st_blocks` of the loose objects.
  On this host's ZFS dataset a file reports one 512-byte block until
  the pool's transaction group commits and its real blocks afterwards:
  seven unchanged 40-to-50-byte
  files measured `1` block each and `9` blocks each a moment later,
  which moved the `size` of seven
  loose objects between `3` and `31` with no process touching them.
  `sync` did not settle it reliably
  (three rounds unchanged),
  so the control now compares the facts a commit elsewhere can change and
  leaves those fields out.

Both controls now pass on this host,
and the modules around them were run threaded afterwards
(`repository_naming_commit::` seven tests and `service_no_manager::` two,
both `test result: ok`).
The
record itself still stays in a repository until a later command sweeps it,
which is what `SPEC.md`'s
"The record" describes;
that is not a change either control was measuring.

## The gates run on `main`, in order, with what each failed

A later gate reads these as its baseline:
a failure outside them starts a fix round against `main`,
and the verified fix's tip is the next gate's `main`.
Only a whole gate on `main` counts;
a branch gate's failures are that branch's to read.
Each line names the tip,
the run or unit name where the gate recorded one,
the summary line,
and the failures with the commit that fixed them.

- `a467a1d22c`,
  unit `apparmor-second-gate-on-main-20261123`,
  the second whole gate after the
  restart and the fourth AppArmor batch:
  `9914 tests run: 9904 passed (8 slow), 10 failed, 36 skipped`.
  The cause H pair,
  the signed-commit control,
  three `ssh_terminal` controls,
  three `stale_ref_lock`
  controls and the size read's child control.
  The fifth to eighth batches and `02de1145eb` (the three `stale_ref_lock` controls killed
  `timeout` and not Git) fixed them.
- `25d719bc6f`,
  after the seventh batch and the second batch of merges:
  seven failed,
  all in that
  baseline (the cause H pair,
  the two `ssh_terminal` controls and the three `stale_ref_lock`
  controls);
  the size read's child control passed by then.
- `6f2b9ef708`,
  after the eighth batch and `02de1145eb`:
  `10085 tests run: 10083 passed (7 slow), 2 failed, 37 skipped`,
  the cause H pair alone.
- The two branch gates after the fifth batch,
  the object reader at `cd6f2e1185` and issue 6345 at
  `04b255e004`:
  eight failed each,
  all in that baseline.
- `0576c39994`,
  run `native-TtGef4`,
  2026-11-24T01:57:53Z to 02:51:53Z,
  beside six guests and a dozen rounds:
  `10219 tests run: 10215 passed (10 slow), 4 failed, 37 skipped`.
  The cause H pair;
  the size read's child control (the kernel clears the parent-death signal on an
  AppArmor profile change at exec,
  fixed in `5c84ab3368`);
  and issue 6399's repair control,
  fixed in
  `d1af169091`.
- `8519c599d0`,
  run `native-bNr3O6`,
  2026-11-24T06:18:44Z to 06:46:59Z:
  `10745 tests run: 10741 passed (7 slow), 4 failed, 41 skipped`.
  The cause H pair;
  the guard `the_release_code_keeps_blocking_work_off_the_runtime`,
  on two
  `std::thread::sleep` calls step 10 brought (`add_mount.rs:70`,
  `add_route_plan.rs:265`);
  and the
  `statmount` control,
  whose call answers `ENOSYS` inside the rootless container
  (`doc/troubleshooting/podman-default-seccomp-blocks-statmount-in-a-container.md`),
  fixed as
  `ada7a937b0`.
- `f3b710cbc9`,
  run `native-MgA4LN`,
  2026-11-24T07:45:49Z to 08:09:44Z:
  `10809 tests run: 10807 passed (6 slow), 2 failed, 41 skipped`,
  the cause H pair alone,
  so the blocking-sleep guard,
  the `statmount` control and Clippy all held.
- `adcb708071`,
  run `native-mEmcZt`:
  `10855 tests run: 10843 passed (6 slow), 12 failed, 42 skipped`.
  The cause H pair,
  the blocking-work guard,
  and nine companion tests that read the host's
  `XDG_RUNTIME_DIR`,
  which the gate's container does not set;
  the companion now takes its runtime
  folder from its caller and its controls pass their own (35 of 35 in the gate image).
- `c4b0b123ab`,
  run `native-AoVnL6`:
  `10865 tests run: 10863 passed (5 slow), 2 failed, 42 skipped`,
  the cause H pair alone.
- `a467773fbf`,
  run `native-S1Mngy`,
  for step 12's merge:
  `10931 tests run: 10928 passed (6 slow), 3 failed, 42 skipped`.
  The cause H pair and
  `worktree_clone_add_route::add_preflight::tests::an_uncommitted_file_is_a_source_the_preflight_serves`,
  which asserted the pool's refusal text while the gate's container has no `/dev/zfs` and the
  compatibility gate refuses first;
  the control now derives the refusal it expects from the device
  layer (`c12b45dfc4`).
- `0152f18bb7`,
  run `native-VeMLmC`,
  2026-11-24T21:47:39-05:00 to 22:04:19-05:00,
  for the batch of 6454,
  the GUID follow-up,
  6455,
  6456 and the preflight control:
  `11062 tests run: 11060 passed (5 slow), 2 failed, 43 skipped`,
  the cause H pair alone,
  each with cause H's own message.
- `b97c09746f`,
  run `native-VZ4uUM`,
  2026-11-25T05:14:00-05:00 to 05:33:19-05:00:
  `11157 tests run: 11153 passed (5 slow), 4 failed, 44 skipped`.
  The cause H pair and the one-program build's two `source_guard::typed_ban_configuration_tests`
  controls,
  which read `clippy.toml` beside `Cargo.toml` while the gate snapshot
  (`bin/native-wrapper-snapshot.ts`,
  `crateEntryNames`) did not copy it;
  fixed as `1e562c82c1`.
- `7043fe4a5c`,
  run `native-YA8wZH`,
  2026-11-25T06:28:29-05:00 to 06:49:28-05:00:
  `11158 tests run: 11156 passed (5 slow), 2 failed, 44 skipped`,
  the cause H pair alone,
  with `1e562c82c1`'s two typed-ban controls passing.
- `df7d464a6c`,
  for step 16 and the 6448 fix:
  `11236 tests run: 11234 passed (6 slow), 2 failed, 45 skipped`,
  the cause H pair alone.
- `64a71a4c9b`,
  unit `gate-on-main-64a71a4c9b`,
  holding the per-slot claim,
  usage row,
  settle,
  askpass,
  strace and forced-arm merges:
  the cause H pair again,
  which issues 6466 and 6468 fixed in the
  controls ("Cause H diagnosed,
  and fixed,
  on 2026-11-25").
- `b6d160365e`,
  unit `whole-gate-on-main-b6d160365e`,
  run `native-0bj8nC`,
  started 2026-11-26T05:40:41-05:00,
  holding the reformat with the root `rustfmt.toml`,
  issue 6492's descriptor-limit children,
  the process-wide-state fixes 6483 to 6487 and 6496 to 6498,
  the crate split's rescan and the linter locks to `alpha.13`:
  `11973 tests run: 11972 passed (6 slow, 1 leaky), 1 failed, 49 skipped`.
  The one failure is
  `landing::service_round::tests::a_lane_read_that_outlasted_its_ceiling_does_not_hold_the_drains_next_storage_step`,
  issue 6489's control,
  whose round was open.
  The leaky test is
  `invocation::typed_network_command_relay::git_error_stream_relayed_as_it_arrives::tests::a_process_that_outlives_the_child_does_not_hold_the_relay`,
  which leaves its sleeping grandchild running,
  as the gates at `639f90484d`,
  `6543be0d1a` and `cea8bb92ff` also showed (issue 6501).
- `196b8d0c87`,
  unit `whole-gate-on-main-196b8d0c87`,
  started 2026-11-26T08:01:00-05:00,
  holding issues 6489,
  6494,
  6495,
  6502 and 6504,
  the Ubuntu `setgroups` skip,
  Pi's virtual writer,
  and the linter's `alpha.15` fixes with its lock still at `alpha.14`:
  `11991 tests run: 11991 passed (8 slow, 1 leaky), 49 skipped`,
  in 931 s.
  It failed nothing.
  The leaky test is issue 6501's relay control again.
- `a6c155bea4`,
  unit `whole-gate-on-main-a6c155bea4`,
  started about 2026-11-26T09:47-05:00,
  holding issues 6493,
  6505,
  6507 and 6508,
  the crate split's round 1 and the linter locked to `alpha.15`:
  `11996 tests run: 11996 passed (5 slow, 1 leaky), 49 skipped`,
  in 843 s.
  It failed nothing;
  the leaky test is issue 6501's relay control.
- `42a59e812e`,
  unit `whole-gate-on-main-42a59e812e`,
  started 2026-11-26T12:29:03-05:00,
  holding issues 6500,
  6501,
  6503,
  6509,
  6511,
  6512 and 6514 and the alpha.47 ZFS pass's three rounds:
  `12028 tests run: 12028 passed (6 slow), 49 skipped`,
  in 896 s.
  It failed nothing,
  and no test was leaky:
  issue 6501's relay control no longer outlives its run.
- `beb61ec46b`,
  unit `gpc-gate-wrapper-next-alpha-on-beb61ec46b-20261127T183913-0500`,
  started 2026-11-27T18:39:13-05:00,
  the gate of the wrapper's `0.6.1-alpha.51`:
  `12307 tests run: 12307 passed (7 slow), 51 skipped`.
  It failed nothing.
- `6037c4e0f8`,
  unit `gpc-gate-wrapper-next-alpha-on-6037c4e0f8-20261127T201313-0500`,
  started 2026-11-27T20:13:13-05:00,
  holding issue 6625's twelve controls and the driver's guard of issue 6636:
  `12321 tests run: 12321 passed (5 slow), 51 skipped`,
  in 929 s.
  It failed nothing.
- `229d5310eb`,
  unit `gpc-gate-wrapper-next-alpha-on-229d5310eb-20261127T220658-0500`,
  started 2026-11-27T22:06:58-05:00,
  the first gate of the wrapper's `0.6.1-alpha.52`:
  it ran no test.
  The image build ended with `error: could not compile` for the library's test build and
  `NativeVerificationError: podman build failed (status 101, signal none)`,
  `gate-exit-status=1`,
  at 22:48:48,
  on the night this machine ran out of memory and swap while many rounds compiled at once.
  The gates after it start only once free memory plus free swap reads 40,000,000 kB on three
  readings a minute apart.
- `af9c816379`,
  unit `gpc-gate-wrapper-next-alpha-on-af9c816379-20261128T005704-0500`,
  started 2026-11-28T00:57:04-05:00,
  the gate `0.6.1-alpha.52` was released on:
  `12337 tests run: 12337 passed (8 slow), 51 skipped`,
  in 1243 s.
  It failed nothing.
- `3d64028ef3`,
  unit `gpc-gate-wrapper-next-alpha-on-3d64028ef3-20261128T024847-0500`,
  started 2026-11-28T02:48:47-05:00,
  holding the seven rounds joined on 2026-11-28
  (the removal's switch,
  the management grammar,
  both halves of the lost-event check,
  `doctor`'s checks for two plan steps,
  and issues 6615 and 6642):
  `12474 tests run: 12474 passed (7 slow), 51 skipped`,
  in 1205 s.
  It failed nothing.
  The three gates of that night took between 1205 s and 1243 s,
  against 929 s the evening before,
  with more than twenty rounds and nine guests on the machine.

A gate's window denies what the work meets,
never a failure by itself:
the `umount` `dac_override` refusals and the `git` ones an issue 6448 round made are in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
and only a denial the read maps to a failing test is a finding.
- `05cf4c8820`,
  unit `gpc-gate-wrapper-next-alpha-on-05cf4c8820-20261128T032317-0500`,
  started 2026-11-28T03:23:17-05:00 and ended at 03:55:53,
  on the merge that makes every sentence beside a removal say what the running removal does:
  `12475 tests run: 12471 passed (10 slow), 4 failed, 51 skipped`.
  The four are in `native_binary`,
  and each met a bound of its own:
  `fault_trace::storage_that_fails_during_a_commit_leaves_a_recoverable_repository`
  (native Git's own commit ended at the control's 60 s bound),
  `lfs_pre_push::a_failing_upload_blocks_the_reconcile_push_of_an_auto_push`
  ("commit service has said nothing for 5 seconds"),
  `lfs_pre_push::the_reconcile_push_of_an_auto_push_runs_the_upload_before_the_ref_moves`
  ("service never settled the after-commit work"),
  and `lfs_server::a_numeric_mirror_uploads_the_objects_of_every_branch_it_publishes`.
  At 03:51:23 the machine read `MemAvailable` 0 kB and `full avg10=49.62` in `/proc/pressure/memory`,
  with a load of 101.
  Issue 6656 asks what the wrapper adds over native Git under that shortage and whether each bound is one a starved process misses with nothing wrong.
  No release was made on this gate.

## Verification

Versions:
the image above (rustc 1.100.0,
git 2.56.1,
cargo-nextest 0.9.151),
podman 6.2.0,
ZFS 2.4.4,
kernel 7.3.5-1-cachyos,
`main` at `cff64adfa0` (this branch changes docs only,
so its tree
is `main`'s build).

The harness is the gate's own image,
run exactly as the gate runs it,
with a nextest filter
expression that names the failing tests:

```sh
podman run --init --network=none --memory=2g --cpus=4 --pids-limit=1024 --blkio-weight=10 \
  --cgroup-conf=io.weight=10 <image> \
  cargo nextest run --offline --locked --all-targets --no-fail-fast --success-output=never \
  -E 'test(/^(<names separated by | >)$/)'
```

The tests the gate's whole suite failed on `main`'s build are the 185 in
`main-failing-names.txt`.
Running that list as its own arm,
twice,
and with the workaround,
twice,
gave:

- as-is:
  184 then 183 of 185 failed;
  103.9 s and 103.8 s;
  the container's `memory.peak` 430 MB and
  409 MB;
- `--tmpfs /gate-fixtures` with `CLI_GIT_TEST_ROOT=/gate-fixtures`:
  5 of 185 failed,
  twice;
  40.9 s
  and 42.0 s;
  `memory.peak` 681 MB and 680 MB;
- a host `/dev/shm` directory mounted there with the same variable:
  5 of 185 failed,
  twice;
  43.9 s
  and 42.8 s;
  `memory.peak` 687 MB and 601 MB.

The residual five are cause E (two),
cause A2 (one) and cause H (two).

The per-cause name lists in the investigation's `scratch/` hold 184 of the 185 between them,
with no
duplicate:
cause A 108,
A2 1,
B 29,
C 28,
D 10,
E 2,
F 3,
G 1 and H 2.
The one name no list holds is
`package_repair::pnpm_runs_in_the_new_worktree_only`,
which the cause G passage above names:
it failed
in the full suite and passed in both filtered arms.
It is also the only name of the 185 absent from
the as-is arm's first run's 184 failures,
and one of the two absent from its second run's 183 (the
flood control is the other),
so it moves with the host's load as cause G says.

The full suite ran once on `main`'s build,
as the control:
185 of 9,173 failed,
listed above.
The
earlier session's LFS branch failed 187 of 9,189;
comparing the two lists leaves four differences and
every one of them is a test that failed in one run and passed in another of the same tree
(`package_repair::quiet_runs_the_repair_and_says_nothing_about_success`,
`fault_trace::a_disk_that_fills_during_a_commit_leaves_a_recoverable_repository` and
`merge::merge_isolation_carry_apply::tests::a_save_whose_bytes_accept_the_whole_merge_is_not_replaced_by_the_captured_layer`
on the LFS build;
the flood control on `main`'s).
So this host's failures are the same list
whichever build of the wrapper runs them.

What does not run here,
and is not a failure of the gate:
`cargo-shadow` and `cargo-nextest` are
absent,
so the mise task's first step fails and only its second step runs.

## Verified workarounds that change no host setting

Both move the tests' fixture root off the fuse mount,
which removes causes A,
B,
C,
D,
F and the
flaky arm of G:
108 + 29 + 28 + 10 + 3 = 178 tests,
plus one flaky one.
Neither touches the two
standing failures,
cause E or cause A2,
so the gate would still be red at 5.

The knob already exists for this and is documented as such (`FIXTURE_ROOT_VARIABLE` in
`src/native/testing/test_support.rs`:
"Environment variable that moves the fixture root,
for example
onto a tmpfs for speed").
Neither way could be passed through `bin/test-native-container.ts` as the
investigation found it:
the gate accepted only `--memory`,
`--cpus` and `--pids-limit` and passed no
`--env` and no `--tmpfs` to `podman run` (`bin/native-gate-container.ts`,
the two `runGateContainer`
calls in `bin/test-native-container.ts`).
So each is a change to the gate,
not to the host,
which is
what the fix below does.

### A tmpfs the container makes for itself

The gate's `podman run` adds `--tmpfs /gate-fixtures:rw,size=1500m,mode=1777 --env
CLI_GIT_TEST_ROOT=/gate-fixtures`.

- Measured:
  5 of the 185 fail (180 fixed);
  2.5 times faster than the as-is arm (103.9 s against
  41 s,
  both arms run twice,
  interleaved);
  about 250 MB more peak memory in the container's cgroup
  (430 MB against 680 MB of a 2 GiB cap),
  which is the fixtures' own bytes.
- What it changes about what the tests test:
  the fixture root becomes a tmpfs,
  so the mount-dependent
  controls exercise the "birth time present,
  mount on the local list,
  Git may run a monitor" arms and
  no longer exercise the absent-birth-time and FUSE arms;
  `lock::lock_birth_times::tests::
  the_hidden_and_coarse_arms_report_what_other_file_systems_do` remains the synthetic control for
  those arms.
  In the measured arm two location controls examined nothing about the moved root,
  because
  each returned early when the variable was set (`src/native/binary/binary_fixture_tests.rs` and
  `src/native/testing/test_support_tests.rs`);
  the branch makes both examine it,
  and the section "Two
  location controls checked nothing in the gate" says what each now asserts.
  The worktree-copy ladder
  then reads `tmpfs` instead of `fuse.fuse-overlayfs`,
  which is a different "other file system" branch,
  and the `fault_trace` kill list still names a call the flow does not make (cause E).
- The `size=1500m` is a ceiling under the container's 2 GiB memory cap;
  a run that outgrows it fails
  with `ENOSPC` rather than with a mount cause.
  No run here came near it (peak growth 250 MB).
  That
  arm's container peaked at 680 MB with about 250 MB of fixtures in it,
  which leaves roughly 1618 MB of
  the cap for fixtures.
  The earlier whole-suite run,
  whose sampler record this branch does not keep,
  sampled 994 MiB of process pages (the history below),
  which left only about 1054 MiB of that cap for
  fixtures,
  so the `1500m` those figures suggested was not reachable in the gate;
  the branch ships
  `768m` instead (the section "The gate's fixture root is a tmpfs the container makes").

### A host tmpfs directory, bind-mounted

The same variable and mount,
with `--mount type=bind,source=<a directory on a host tmpfs>,
destination=/gate-fixtures`,
the host directory made `0777` first.

- Measured:
  the same 5 failures,
  43.9 s and 42.8 s,
  `memory.peak` 687 MB and 601 MB.
- It costs the same and needs a host directory to exist and be world-writable,
  which is a change to
  the host's state (though not to any configuration),
  so the container-made tmpfs is the better of
  the two.

## Ways that change the host (not run here)

Each needs the owner's authorization;
none was run.

- A ZFS dataset with `utf8only=off` and `normalization=none` for podman's storage would remove cause
  A and A2 (109 tests) and nothing else:
  the fuse mount stays FUSE,
  so B,
  C,
  D,
  F and E stay.
  ```sh
  sudo zfs create -o utf8only=off -o normalization=none -o mountpoint=/var/lib/containers-zfs zpcachyos/ROOT/cos/containers
  sudo systemctl stop podman.socket 2>/dev/null; podman system reset --force
  ```
  and a `~/.config/containers/storage.conf` naming `driver = "overlay"` with
  `graphroot = "/var/lib/containers-zfs/storage"`,
  then `podman system reset` again so the store is
  rebuilt.
  `podman system reset` deletes every image and container of this account,
  the gate's own
  images included.
- A kernel `overlay` or Btrfs mount under the fixture root would remove causes A to F at once,
  and Btrfs
  or an overlay would also make the copy flow clone,
  removing cause E.
  Reaching either from a ZFS host
  means a loop device or a dataset of another type mounted by root,
  for example
  ```sh
  sudo truncate --size=40G /var/lib/gate-btrfs.img && sudo mkfs.btrfs /var/lib/gate-btrfs.img
  sudo mount --options loop /var/lib/gate-btrfs.img /var/lib/gate-btrfs
  ```
  and then `--mount type=bind,source=/var/lib/gate-btrfs,destination=/gate-fixtures` with
  `CLI_GIT_TEST_ROOT=/gate-fixtures`.
  If the device stays Btrfs,
  the mount type the product and Git
  see is `btrfs`,
  which is on both of the product's lists,
  has birth times,
  takes any byte in a name,
  and clones.
  Whether a bind mount of Btrfs inside a `fuse-overlayfs` container reports `btrfs` as
  the mount type was not measured,
  since mounting it needs root.
- Changing podman's storage driver to `vfs` would put every layer on the same dataset,
  so it changes
  nothing here.

## Clippy's E0514

The gate's Clippy container mounts the host's `rustc` sysroot read-only and runs the host's cargo
from it:

```text
--volume <host sysroot>:/toolchain:ro
--env PATH=/toolchain/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
--env RUSTC=/toolchain/bin/rustc
--env RUSTDOC=/toolchain/bin/rustdoc
/toolchain/bin/cargo clippy --offline --locked --all-targets -- -D warnings
```

`cargo clippy` is not part of cargo:
it looks for `cargo-clippy` on `PATH`,
which sets
`clippy-driver` as the wrapper for the workspace's own crates.
On this host the mounted sysroot is
`nightly-x86_64-unknown-linux-gnu`,
whose `bin/` has no `cargo-clippy` and no `clippy-driver`,
so both
are found later in `PATH`,
in the image:

```text
toolchain bin: cargo rust-gdb rust-gdbgui rust-lldb rustc rustdoc
cargo-clippy -> /usr/sbin/cargo-clippy
clippy-driver -> /usr/sbin/clippy-driver, version clippy 0.1.100
rustc -> /toolchain/bin/rustc, rustc 1.102.0-nightly (5df8c02af 2026-11-16)
```

The dependencies are then compiled by rustc 1.102.0-nightly,
and the wrapper itself by the image's
clippy 0.1.100,
which cannot read their metadata:
E0514,
whose help line names the 1.100 compiler
that is failing.
Thirteen errors for the library and fifteen for its test target,
none of them about
the sources.
The same command succeeds where both compilers come from one toolchain:

- with the image's own toolchain (no mount,
  no `RUSTC`):
  `cargo clippy --offline --locked
  --all-targets -- -D warnings` exits 0 in 83 s;
- with the repository's locked toolchain mounted (the same command with
  `--volume /home/user/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu:/toolchain:ro`
  and the same three variables):
  exits 0 in 130 s,
  after recompiling the graph with 1.102.0-nightly.

Either is a fix.
The second keeps the Clippy the repository pins;
it needs the host to name a sysroot
whose toolchain has the clippy component,
which this host already has.
Running the gate as
`RUSTUP_TOOLCHAIN=nightly-2026-11-16 systemd-run ... node bin/test-native-container.ts` gives
`rustc --print sysroot` that toolchain and changes no host file;
`mise run toolchain:install` would
make the same toolchain the default instead.
The first needs the gate to stop mounting the host's
sysroot,
and then Clippy is the image's pacman clippy (0.1.100) rather than the pinned one.

Every list of test names above is a file beside this investigation's logs,
in
`/home/user/temp/agent/git-policy-cli-container-gate-on-the-zfs-host-investigation-2026-11-19/scratch/`:
`cause-<cause>.txt` holds the names,
`main-failing-names.txt` the 185 the full suite failed,
and
`main185-a1.log`,
`main185-b1.log` and their siblings the arms' own nextest output.
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

## What does not work

- Supplying the variable through `mise`,
  a shell or the image:
  the gate's `podman run` passes no
  `--env`,
  and the image sets no `ENV CLI_GIT_TEST_ROOT`;
  nothing inside the container sees a host
  variable.
- Moving only `target/tmp` (`CARGO_TARGET_TMPDIR`) or only `target/test-fixtures`:
  the unit face and
  the binary face use different roots,
  and both read the same variable,
  so one `--tmpfs` with the
  variable covers both where the helper honours it (cause A2 does not).
- `podman run`'s `--uidmap` or a different user:
  the refusal is the dataset's,
  not the account's;
  the
  host's own `mkdir` fails with the same errno.
- `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED`,
  which the image sets:
  it silences the btrfs-only controls,
  and
  none of the 185 is a btrfs control.
- Checking the Git citation in a browser:
  `agent-browser` could not launch Chrome in this session
  (`Socket path too long: <run folder>/org.chromium.Chromium.XXXX/SingletonSocket`),
  and passing
  `--profile`,
  `TMPDIR` and `XDG_RUNTIME_DIR` did not move that socket.
  The two Git files were read
  through the GitHub REST API at the pinned commit instead.

## Upstream filing decision

Nothing to file,
by the six constraints:

1. Not upstream's fault:
   the five mount behaviours are each documented or deliberate.
   ZFS refuses a
   name with `utf8only=on` by design;
   Git refuses `core.fsmonitor` on a FUSE working tree by design,
   with a setting to allow it;
   the product's two file-system lists are ours.
2. Upstream could not fix what is left:
   a FUSE mount has no kernel-held birth time to report and no
   kernel clone,
   and fuse-overlayfs cannot make its mount type `overlay`.
3. `fsmonitor.allowRemote` is upstream's supported answer for the Git part,
   and we do not set it.
4. Not reached:
   constraints 1 to 3 already decide it.
5. Not reached.
6. Not reached.

The gate is ours,
so what needs a change is here:
the gate's `podman run` (the fixture root or the
Clippy toolchain),
one test helper (cause A2) and possibly the kill list's `ioctl` entry on a file
system that does not clone (cause E).

## What was fixed, and what was measured after

Fixed on the branch `fix/git-policy-cli-container-gate-on-a-zfs-host`,
with no change to the host:
nothing on the ZFS dataset,
nothing in podman's storage,
and nothing in the installed toolchains.

### The gate's fixture root is a tmpfs the container makes

`bin/native-gate-fixture-root.ts` holds the mount and the knob,
and `bin/test-native-container.ts`
spreads them into the suite's `podman run` on every host:

```text
--tmpfs /work/package/git-policy/cli/target/gate-fixtures:rw,size=768m,mode=1777
--env CLI_GIT_TEST_ROOT=/work/package/git-policy/cli/target/gate-fixtures
```

The root is a child of the image's `/work` tree rather than the `/gate-fixtures` this investigation
measured with,
because a fixture crate that runs Cargo takes the configuration of the directories above
it,
and the image's vendored-source mapping is `/work/.cargo/config.toml`.
The root it replaces
(`target/tmp` and `target/test-fixtures`) was below `/work` too.
The first full run with
`/gate-fixtures` failed two `package_repair_prewarm_job` controls whose fixture workspaces build
offline with `serde = "1"`,
for want of that mapping;
the root below `/work` keeps their ancestry.

`CLI_GIT_TEST_ROOT` is the variable both faces of the suite already read
(`FIXTURE_ROOT_VARIABLE` in `src/native/testing/test_support.rs` and in
`src/native/binary/binary_support_fixture_directories.rs`),
so no test code changed for this.

The tmpfs answers the four mount facts the way the kernel overlay of the btrfs host did:
it takes a
name that is not UTF-8,
records birth times,
is `tmpfs` on both of the product's lists,
and Git reads it
as a local working tree.
It also gives the copy flow no clone,
which is cause E below.

Those four are the facts the btrfs host's kernel overlay gave,
so the birth-time,
mount-type and monitor
controls passed in the gate's run on that host:
a tmpfs root gives that coverage back rather than
trading it away,
which is why this workaround needed no decision from the project.

The size is a ceiling under the container's 2 GiB memory bound,
because tmpfs pages count against that
bound:
a fixture tree that outgrows the ceiling fails with `ENOSPC` on the path being written below the
fixture root,
where a tree that instead exhausted the memory bound would end the run with an
out-of-memory kill.
The integration pass of 2026-11-19 ran a whole suite on this root in the image
`localhost/git-policy-native-test:integrate-git-policy-cli-container-gate-on-a-zfs-host`,
211 samples
taken with a 3-second delay between iterations across 780 s,
and that record is kept in this package
beside the other raw measurements (`perf/gate-resource-peaks-2026-11-19-integration-pass.jsonl`,
with the
figures its reader printed,
`perf/gate-resource-peaks-2026-11-19-integration-pass-summary.json`,
beside
it):
the kernel's `memory.peak` was 1123.8 MiB of the 2048 MiB bound,
while the highest charge any sample
read was 953.1 MiB,
170.7 MiB below that peak,
so a sampler that slow missed the true high point and the
figures taken from it are the samples' and not the run's.
The samples' highest fixture bytes were
195.1 MiB,
and their highest process pages,
a sample's charge minus the fixture bytes in the same sample,
868.3 MiB;
those are the maxima of two components and need not have been read in one sample.
The
conservative bound is the one beside the kernel's peak:
the 2048 MiB bound leaves 924.2 MiB for fixtures,
and the 768 MiB ceiling sits 156.2 MiB below that,
so a run whose fixtures pass the ceiling while its
other pages hold to the peak that bound came from meets `ENOSPC` before the memory bound;
the ceiling is
also 3.94 times the sampled fixture peak.
The summary's own `headroomForFixturesMiB` and
`ceilingBelowHeadroomByMiB` are the sampled process pages' arithmetic (1179.7 and 411.7 MiB),
which the
bound beside the kernel's peak replaces.
A gate review's run on the same code apart from the repaired location
controls,
whose sampler record is not kept here,
read a 916.4 MiB `memory.peak` and a 221.4 MiB sampled fixture
peak,
of which the ceiling is
3.47 times.
No sample of that run counted an out-of-memory kill,
and its only failures are the standing
pair of cause H,
so every control that builds a fixture workspace built it.
The earlier whole-suite run,
whose sampler record this branch does not keep,
peaked at 1192 MiB of the
bound and 222 MiB of fixture bytes,
with 994 MiB of process pages over 235 samples across 740 s;
its
fixture peak came from a run whose two `package_repair_prewarm_job` controls failed before their
fixture workspaces built,
so their bytes were not in it,
and those controls hold under 64 MiB of
fixtures,
since all 23 of that module's controls pass in the gate image on a 64 MiB tmpfs.
The 768 MiB
ceiling is 3.5 times that peak and 286 MiB below that headroom too.
The earlier 1500 MiB ceiling,
cited from the fixture peak alone,
sat above that headroom:
the memory
bound would have ended a run there before the tmpfs answered `ENOSPC`.
The mechanism was checked in
the gate image:
a 64 MiB tmpfs under a 512 MiB bound took the first 64 MiB of a 96 MiB write and
answered `No space left on device`,
its cgroup reporting `oom_kill 0`,
while a 128 MiB tmpfs beside it
took all 96 MiB.
A caller's smaller `--memory` bound moves the memory bound back below the ceiling,
which is the caller's own choice of bound.
The two probes' logs are in
`~/temp/agent/git-policy-cli-container-gate-on-a-zfs-host-review-findings-fixed-2026-11-19/scratch/`:
`prewarm-module-on-a-64m-tmpfs.log` and `tmpfs-size-returns-enospc-probe.log`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
The gate's manifest now records the root and the mount options beside the image and the limits,
so a
future failure can read where the tests ran.

### Clippy mounts the repository's locked toolchain

`bin/native-gate-clippy-toolchain.ts` reads `toolchain.lock.json` through the repository's owner of
that lock (`readLockedRustToolchain` in `package/test-harness/core-matrix`),
asks
`rustup which --toolchain <locked> rustc` for that toolchain's own sysroot,
and mounts it where
`rustc --print sysroot` used to answer.
`rustup which` answers about the named toolchain whatever the
caller's `RUSTUP_TOOLCHAIN` or rustup default says,
so the gate no longer depends on the environment
that started it.

Before it runs Clippy it checks the mounted sysroot's `bin` directory for `cargo`,
`cargo-clippy`,
`clippy-driver`,
`rustc` and `rustdoc`,
and refuses when one is missing,
naming the program,
the
toolchain,
the Clippy component,
and both ways to install it (`rustup component add clippy
--toolchain <locked>` and `mise run toolchain:install`).
A sysroot without Clippy can therefore no
longer fall through to the image's older `clippy-driver`,
which is what produced the E0514 errors.

`package/git-policy/cli/package.json` gains a `workspace:*` dependency on
`@monochromatic-dev/test-harness-core-matrix` for that reader,
and `pnpm-lock.yaml` gains its importer
entry;
nothing else in the lock changed.

### Cause A2: the one helper that wrote under `target/`

`scratch()` in `src/native/landing/service_claim_watch_tests.rs` now builds its directory under
`crate::test_support::fixture_root()`,
which is the root the suite's knob moves,
instead of under the
crate's `target/`.
Every other control in that file already used the fixture root through the same
function.

### Two location controls checked nothing in the gate

`the_fixture_root_is_not_the_system_temporary_directory` (`src/native/testing/test_support_tests.rs`)
and `fixtures_are_created_outside_the_system_temporary_directory`
(`src/native/binary/binary_fixture_tests.rs`) each returned early when `CLI_GIT_TEST_ROOT` was set.
The
gate always sets it,
so the two controls that keep a fixture tree out of the system temporary directory
(issue 1809) passed there,
and they passed with the root under `/tmp` as well:
a control that returns
early cannot tell the two roots apart,
and the times they took say nothing either way.

Both now always compare the root they use with `/tmp`,
`/var/tmp` and the run's `TMPDIR`
(`system_temporary_directories`,
in `src/native/testing/test_support.rs` and in
`src/native/binary/binary_support_fixture_directories.rs`):
an empty `TMPDIR` names no directory and is
left out,
so `TMPDIR=` no longer matches every path;
each entry is the path it resolves to where it
resolves,
an entry that does not resolve keeping its own path;
and the unit face's root is resolved
before the comparison,
so a root linked into `/tmp` and a `TMPDIR` linked to the root are both reported
(the binary face's fixture root is canonical already,
because `fixture_in` canonicalizes it,
and a root
it cannot resolve cannot exist).
The two shape assertions stay on the path `fixture_root` returned,
so a
build output that is itself a link keeps its own answer.
Each still checks the shape of the root it uses:
the unit face's root always ends in `process-<pid>`,
and with nothing moved it is also
`test-fixtures/process-<pid>` beside the build output,
while the binary face's fixtures are under Cargo's
per-target scratch directory (`CARGO_TARGET_TMPDIR`).
In the gate the unit face's root is
`/work/package/git-policy/cli/target/gate-fixtures/process-<pid>` and the binary face's is
`/work/package/git-policy/cli/target/gate-fixtures`,
so both controls now assert that the fixture tmpfs
is not a system temporary directory.

Each was shown red on a moved root under `/tmp` and green on the gate's root.
In the integration pass's
image,
its source tree replaced by the branch's and nextest filtered to the two names,
with
`CLI_GIT_TEST_ROOT=/tmp/moved-root` both failed,
naming the root below `/tmp/moved-root` and `/tmp` as
its system temporary directory,
and both passed with the gate's own tmpfs root and with no variable
set.
That `/tmp` arm passed before the change,
which is exactly the gap the two early returns left.

The repair round ran further arms of the same two controls in the same image with the same kind of
mount.
With `TMPDIR=` both now pass on the gate's root,
where both had failed and named the root as
being under an empty temporary-directory path.
A root that is a link to `/tmp/real-root` now fails on
both faces (`/tmp/real-root/process-<pid> is under the system temporary directory /tmp`,
and the binary
face's own root the same way),
where the unit face had passed;
and a `TMPDIR` that is a link to the gate
root now fails on both faces,
where both had passed.
The arms that must keep their answer did:
the gate
root with `TMPDIR=/tmp/` still passed on both faces,
and a moved root under `/tmp` and a `TMPDIR` set to
the gate root still failed on both.

### Cause E: the `ioctl` entry follows the fixture root's file system

The copy flow clones each staged file only where the file system may share data blocks between files;
every other file system has Git check the worktree out,
and then the flow makes no `ioctl`.
The
product's answer to that question is now `may_clone` in `src/native/subvolume/fs_kind.rs` (moved there
from `worktree_snapshot_source`,
where it stays the same function with one more reader).

The binary control's `Flow` records which kill-list calls the file system decides,
the copy flow
records `ioctl` with `may_clone`,
and `require_the_kill_list_is_made` counts only the calls that hold
for the fixture root the run measures.
The `ioctl` sweep says it did not run,
and why,
on a fixture
root that does not clone,
as the `strace`-less arms already do;
on one that does it sweeps up to twelve
positions of the `ioctl` calls,
as every other kill control of that file does.
It skips only when the
run states that the absence of a cloning root is known,
by the same
`CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED` the btrfs controls read (the gate image sets it),
and fails with
the fixture root it found otherwise,
so no run passes the sweep as coverage it does not have.

### Five controls that assumed the old fixture root

The first full run on the tmpfs root (with `/gate-fixtures`) failed five controls that the overlay had
let pass.
Each failed for its own reason,
and each was a dependency on the root rather than a product
defect:

- `conflict::discarded_prune::guard_tests::a_removal_that_fails_part_way_is_reported_and_audited_as_partial`
  read the audit line's file names in the order the removal walk made them and compared them with a
  sorted list.
  tmpfs lists a directory in another order,
  so the control now sorts both sides:
  what it
  holds is that the line names the files that went and no other.
- `repair::package_repair_prewarm_job::tests::the_filling_puts_the_lock_back_as_checked_out` and
  `the_filling_publishes_a_clean_copy_and_leaves_the_source_alone` build a fixture workspace offline
  with `serde = "1"` and resolved it from the image's `/work/.cargo/config.toml`,
  which a root outside
  `/work` does not reach.
  The root below `/work` keeps that ancestry,
  and the controls are unchanged.
- `worktree::worktree_add_binding::tests::a_relative_pointer_without_links_is_bound` climbed a fixed
  seven levels with `..` and expected to land on the file system root.
  `/gate-fixtures` is shallower
  than the old root,
  so the climb left the root and the pointer was refused.
  The climb is now derived
  from the destination's own depth,
  and the refused one climbs past it by one.
- `worktree::worktree_convert::safety_tests::a_difference_keeps_the_old_tree_and_nothing_is_copied`
  expected the contents difference to be the one reported,
  but the comparison reports the first
  difference its walk meets,
  and tmpfs meets the file only the old tree holds first.
  The control now
  checks the promise (old tree kept,
  installed edit untouched,
  nothing copied),
  settles that file,
  and
  asks again,
  so the contents difference itself is the one named.

### Measured after the changes

Two whole suites ran on this host against the branch's tree,
each in the gate's own containers
(`--memory=2g --cpus=4 --pids-limit=1024`).

- The first run,
  on a tmpfs at `/gate-fixtures`,
  before the five controls above were fixed:
  `9173 tests run: 9166 passed (5 slow), 7 failed, 28 skipped` in 743 s.
  Its two failures besides
  those five are the standing pair below,
  and its Clippy step compiled clean from the mounted locked
  toolchain (the gate's refusal named only the test run).
  Its output is kept beside the second run's
  under `scratch/keep/` of `~/temp/agent/git-policy-cli-container-gate-on-a-zfs-host-fix-2026-11-19/`
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here),
  read back from the system journal of the two containers it ran (`tender_blackburn`,
  cgroup
  `libpod-c731bc55...`,
  and `great_maxwell`,
  cgroup `libpod-1418d739...`):
  `gate-run-1-whole-output-from-the-journal.log` holds the seven failures with their process output,
  the two `package_repair_prewarm_job` panics among them,
  and the summary line,
  and
  `clippy-run-1-from-the-journal.log` holds the Clippy step's whole stream,
  which ends with the
  toolchain's own `Finished` line,
  2m 06s after it started,
  and no warning or error line,
  the ending a
  refused Clippy does not have (`clippy-run-2-final-tree.log` ends `error: could not compile` instead).
  The journal dropped that run's final success-output block after test 1087 of 9173 and every
  `BTRFS CONTROL NOT RUN` line,
  the same drops as the second run's log.
  The gate's own refusal text
  went to the driver's stdout and survives only in that driver's session record,
  `run-1/feed.jsonl` of
  the same folder,
  which is not kept as evidence here;
  the two logs above are what the claim rests on.
- The second run,
  on the root below `/work`,
  after the fixture-root,
  toolchain and five-control changes:
  `9173 tests run: 9171 passed (5 slow), 2 failed, 28 skipped` in 775 s.
  The two failures are cause H,
  the standing pair:
  `repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone`
  lists `cli-git-lock-holders` in the `.git` directory it finds and expects it absent,
  and
  `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing` fails with
  `no scratch directory or record is left: ["cli-git-lock-holders"]`.
  Both fail on a btrfs host too
  (cause H above),
  so no mount change touches them.
  The 28 skipped are nextest's ignored tests,
  the
  same count this investigation's own full run reported;
  the btrfs controls are not skipped,
  each prints its line and passes.
  The harness that ran this suite was stopped by a fault of its own
  while the gate's container ran on,
  so this run's evidence directory holds no `passed.json`,
  and its
  output was read back from the journal,
  which drops some of a burst this size (the btrfs lines and the
  passing controls' lines among them).
- The suite container's cgroup peaked at 1192 MiB of its 2 GiB bound and the fixture tmpfs at 222 MiB,
  with no OOM kill,
  over 235 samples across the first run's 740 s;
  its process pages,
  the container's
  charge minus the fixture bytes in the same sample,
  peaked at 994 MiB.
  That sampler's record is no
  longer kept,
  so those three figures stay here as history;
  the tmpfs size now cites the integration
  pass's record (`perf/gate-resource-peaks-2026-11-19-integration-pass.jsonl`),
  whose kernel
  `memory.peak` is 1123.8 MiB of the bound while its 211 samples read 195.1 MiB of fixture bytes and
  868.3 MiB of process pages.
  The 1192 MiB measurement is what showed the 1500 MiB ceiling then shipped
  to sit above the 1054 MiB that measurement left for fixtures.
  The second run's sampler ended with its
  harness and still read the first root's path,
  so its recorded peak of 992 MiB covers only its first
  570 s.
- The first run's gate took 1136 s including a fresh base image (a `pacman` transaction with network)
  and the layered test image;
  the second reused that base.
  The investigation's arm of the 185 tests
  took 41 s against 104 s on the container's own tree;
  this whole-suite run's 743 s and 775 s are not
  comparable with the 1053 s the same suite took on the dataset,
  because that run's tree was also the
  failing one.
- Clippy:
  the first run's gate step exited zero from the mounted locked toolchain (the gate's refusal
  named only the test run).
  The Clippy step over the second run's tree failed on
  `clippy::shadow_unrelated`,
  which this gate denies:
  the two-step assertion the conversion control
  gained above bound `message` twice in one function.
  That step was run on its own,
  with the gate's
  image,
  mounted toolchain and flags,
  because the gate's own Clippy container was still running when
  the harness that started it was stopped and its captured output went with the client
  (`scratch/clippy-run-2-final-tree.log`).
  Commit `8d1122a6ae` names the second binding apart,
  and the
  same step over the fixed tree then finished with `Finished dev profile [unoptimized] [debuginfo]
  target(s) in 2m 13s` and no warning or error line (`scratch/clippy-run-2-fixed.log`);
  the two logs'
  differing outcomes are what shows that step reads this tree rather than a stale image.
- No third gate run was started,
  as the task set for this fix allows two.
  The second run's image and
  the final tree differ in exactly one source file,
  `src/native/worktree/worktree_convert_safety_tests.rs`
  (commit `8d1122a6ae`):
  the SHA-256 of every file under `src` in the image and in the worktree agree
  otherwise,
  and that file's only change is one local binding's name.

### What these changes do not cover

- The two standing failures (cause H) still failed when this section was written,
  and no mount change
  touched them;
  the section "Cause H diagnosed,
  and fixed,
  on 2026-11-25" holds their two causes and the
  controls that changed.
- The memory bound the gate gives each check also bounds its image build:
  `buildLimitArguments` with the
  gate's own limits (`bin/container-limits.ts`,
  whose implementation lives in the mutation-test package,
  called at `bin/test-native-container.ts:234`) turns that bound into the build's `--memory`.
  The image
  build met that 2 GiB bound during these runs;
  it predates this fix,
  which changes neither the bound nor
  the build,
  and the run that met it reported it.
- The cloning arm of cause E was not exercised here:
  this host has no btrfs and no overlay mount,
  so
  the runs above printed that the `ioctl` sweep did not run.
  The list control still requires every
  listed call on a cloning root,
  and `may_clone` is the product's own answer to which file systems
  those are;
  a btrfs host is where the sweep itself runs again,
  and
  `mise run //package/git-policy/cli:native:test:host:btrfs` is that run:
  it unsets the acknowledgement
  and runs every target on a btrfs root,
  so the sweep runs there,
  and a wrapper release waits on it
  unless the version is an alpha (`bin/native-release-host-controls.ts`).
- The mutation runner copies its work tree onto a tmpfs of its own
  (`package/mutation-test/cli/src/host/podman.ts`),
  so its tests already met a tmpfs fixture root.
  That
  root shares no data blocks between files either,
  which is the arm the `ioctl` control now skips;
  the
  gate and a campaign therefore agree about the fixture file system,
  which a gate pass reused as a
  campaign baseline now relies on.

## A gate that ends with status 137 and no memory kill

On 2026-11-19 an integration pass's gate ended with
`NativeVerificationError: the test run failed: podman run failed (status 137, signal none).`
after 8446 of 9199 tests had passed and none had failed.
It was not a memory kill:
the container scope's journal line read `912M memory peak` against its 2 GiB bound,
and `journalctl -u systemd-oomd` had no entry in that window.

The user journal showed the cause.
In the same second as the podman that ran the suite logged `container died`,
a different podman process (`podman[3180925]`) logged `container remove` for that container
and for another session's container:

```text
# journalctl --user --since '2026-11-19 22:34:15' --until '2026-11-19 22:34:30'
podman[3180925]: container remove 7a5c4dce9245... (image=.../git-policy-native-test:..., name=pedantic_bardeen)
podman[3180925]: container remove 5982b11d2ac9... (image=.../git-policy-native-test-base:..., name=strange_mayer)
```

That process was another session's agent cleaning up:
it listed the running containers,
took two that were running `cargo nextest` for its own,
and ran `podman rm --force --time 0 7a5c4dce9245 5982b11d2ac9`.
Many sessions run gates and guests on this machine at once,
so a container's image or command says nothing about who started it.
Briefs now tell every agent to stop or remove only what it started,
known by a name it gave it or the ID its own command printed.

To tell this case apart from a memory kill,
read the user journal around the time the gate stopped for a `container remove` line
from a podman process other than the one that logged the container's start.
The integration pass then reran only the tests that had not passed,
in the same image with the gate's limits and fixture root,
rather than the whole suite a second time:
751 of 753 passed,
and the two failures were the standing cause-H pair.

## Cause I: a joined thread's own file outlives its join (issue 6330)

This is neither cause G nor cause H.
Cause G is the label this document gives failures that one busy full run had and no other run repeats,
with no cause recorded;
this one has a cause decidable from the kernel source,
and its control is fixed.
Cause H is the standing scratch-directory pair,
which fails here and on the Btrfs host.

### The symptom the gate reader saw

The closure gate of 2026-11-20T23:47:51-05:00
(`9367 tests run: 9364 passed, 3 failed`)
failed this control beside the two cause-H tests:

```text
FAIL [ 0.013s] (8870/9367) monochromatic-git-policy-cli::native_binary process_cpu_counters_guard::a_thread_that_ended_between_the_listing_and_the_read_is_not_a_process_that_ended
  stderr ───
    thread 'process_cpu_counters_guard::a_thread_that_ended_between_the_listing_and_the_read_is_not_a_process_that_ended' (275220) panicked at src/native/binary/binary_support_process_cpu_counters_tests.rs:52:5:
    a thread's own file is gone once the thread ended, which is the reading a listing meets
```

That run started each test in a process of its own
(`running 1 test`,
`1252 filtered out`),
so the tid the control named was the thread it started,
and not a sibling test's.

### The cause

The control asserted an order the kernel does not promise.
A join returns when the kernel clears the child tid the join waits on and wakes that futex,
which happens inside the thread's own exit:
`mm_release` does both
(`kernel/fork.c:1617-1627` at v6.12,
`adc218676eef`;
`:1490-1497` at v7.3),
and it is reached from `exit_mm` in `do_exit`
(`kernel/exit.c:926` at v6.12;
`:974` at v7.3).
The thread's `/proc` entry stops answering later,
inside `release_task`
(`kernel/exit.c:241` at v6.12;
`:248` at v7.3),
where `__exit_signal`
(`:258` at v6.12;
`:272` at v7.3)
runs `__unhash_process`
(`:201` at v6.12;
`:211` at v7.3),
whose `detach_pid` on the thread's own number
(`:129` at v6.12;
`:140` at v7.3)
is what leaves nothing to answer.
`proc_flush_pid`
(`:280` at v6.12;
`:297` at v7.3)
flushes the cached dentries after that,
and `exit_notify`
(`:959` at v6.12;
`:998` at v7.3)
is where the same `do_exit` reaches `release_task`.
`clone(2)` documents the first half as `CLONE_CHILD_CLEARTID`,
which is "used by threading libraries".
So a thread's own file can still answer when its join returns.

The window is microseconds for a sub-thread,
which is the inferred reason one loaded gate met it and filtered runs did not:
no run has observed that window directly,
and forcing it would need the load or the spin the rules ban.
It is not bounded for two other tasks,
and both make the same reading:
a thread-group leader whose group still has a thread is not reaped,
because `exit_notify` reaps a leader only when `thread_group_empty`
(`kernel/exit.c:757-762` at v6.12;
`:766-768` at v7.3),
and a traced task is released only when its tracer reaps or detaches it.

What these kernel lines prove is narrower:
the kernel does not promise the order,
and the incident's own assertion was `thread_ticks(pid, ended).is_none()`
(the pre-fix file's `assert!` at `:52:5`),
in a process nextest gave one test.

The same control had a second,
latent defect the gate never met:
it named the ended thread as any tid its second listing had and its first had not,
so a thread a runtime started beside it would have been read as the one that ended.
The thread now reports its own number instead.

### The evidence, forced rather than repeated

The order is forced by a construction whose barrier the probe controls.
The probe is committed beside this document as
[`git-policy-cli-container-gate-on-a-zfs-host-probe.rs`](git-policy-cli-container-gate-on-a-zfs-host-probe.rs),
and builds with `rustc --edition 2021`.
The transcripts it printed were scratch in a run folder whose starter deletes scratch,
so the lines quoted here are the record.
The probe starts a child process that runs one of two arms,
takes the child's `/proc/<pid>/task` listing while the ending thread still waits,
releases it,
and reads that thread's own file as the join returns:

- `--leader`,
  the forced order:
  the child's leader publishes the join word with `set_tid_address`,
  and ends with `exit`,
  which ends only that thread,
  while one other thread waits on the word exactly as a join does.
  The kernel clears the word and leaves the task in `/proc`.
- `--plain`,
  the positive control:
  an ordinary sub-thread is ended and joined.

The probe was run in the gate's own image under the gate's own bounds
(`--cpus 4 --memory 2g --pids-limit 1024`).
Three `--leader` runs printed the same answer,
and the probe's copy of the control's assertion failed in each:

```text
arm --leader listing [3, 4] listed true state-before S joined at-join Some(0) state-at-join Z control-passes false gone-after true total 1.508625ms
arm --leader listing [3, 4] listed true state-before S joined at-join Some(0) state-at-join Z control-passes false gone-after true total 1.199126ms
arm --leader listing [3, 4] listed true state-before S joined at-join Some(0) state-at-join Z control-passes false gone-after true total 1.660545ms
```

The `--plain` control in the same image read the file as already gone at the join
(`at-join None ... control-passes true`),
so the probe can show either answer,
and the forced arm is not the only one it knows.
Those arms show that the kernel can leave a thread's own file answering at a join,
which is the reading the incident's control met;
neither observes which kind of thread that was,
and the `--leader` arm forces a different path from the sub-thread window,
so the incident's sub-thread explanation stays an inference.
Each run ended by the probe's own barrier byte,
reaped its child,
and exited 0 in about a millisecond.

A tracer is the other way to hold a task between the join wake and its release,
and it was measured first.
`PTRACE_SEIZE` on the child's thread showed the same state
(`at-join Some(0)`,
state `Z`),
and it ended badly:
the tracee stops after that wake as a zombie,
`PTRACE_DETACH` on it was refused (`-1`),
and the child process could not die while its traced thread stayed a zombie,
so the probe hung until its wall clock bound killed it.
No control in the tree holds a task at a tracer's stop.

### The fix

`package/git-policy/cli/src/native/binary/binary_support_process_cpu_counters_tests.rs`
is the only file that changed.
The reader it guards,
`package/git-policy/cli/src/native/binary/binary_support_process_cpu_counters.rs`,
needed nothing:
it already sums the threads each moment has,
and answers nothing only for a process that is gone.
The control now:

- has the waiting thread report its own tid,
  so the thread it ends is that thread;
- takes the listing while that thread waits,
  and asserts the listing names it;
- ends and joins the thread,
  then waits,
  bounded to 30 s at 1 ms between looks,
  until the kernel has released the thread's own file,
  so it reaches the state it is named for under either order,
  instead of assuming the moment;
- sums beside a thread that only waits,
  whose clock ticks are improbable to grow between the two readings,
  and asserts that sum is exactly that thread's own ticks,
  so a thread whose own file is gone adds nothing,
  and a sum that counts it again fails.

A planted removal of the reader's tolerance,
a sum that expects a thread's own file,
fails this control on every run,
which is the guard this incident was about.
The sum is one-sided:
a sum that over-counts fails,
and a sum that under-counts is not pinned.
The read of the ticks of the thread that stays is also the positive control
that the reader parses a live thread's own file:
a reader that answered nothing for every file fails there,
before any sum is taken.
The thread that only waits has usually spent no tick to expect,
and a tick it spent between the two reads is improbable,
never impossible,
and would fail the exact sum on a correct reader.
A reader that answered zero everywhere still passed it in the runs that planted it,
so no test in the tree pins that undercount,
which is recorded here rather than left for the next reader to discover.

The image was `localhost/git-policy-native-test:gpc-6330-final2-20261121`,
built 2026-11-21T04:28:08-05:00 from this branch's tree,
and each run was the gate image's own `cargo nextest run --offline --locked --all-targets --no-fail-fast`
narrowed by a name filter under those bounds.
The module's control passed
(`1 test run: 1 passed`,
0.019 s),
and so did the whole `source_guard` set
(`80 tests run: 80 passed`,
4.083 s).

### If this name fails again

Read which assertion failed.
The patience assertion names a joined thread whose own file the kernel still answered after 30 s,
which is a task the kernel is not releasing,
and a different incident from this one.
The sum assertions name the listing's sum and the live threads' sum.
A sum that over-counts fails them,
which is a defect in the reader,
unless a waiting thread spent its one tick between the two reads.
A sum that under-counts is not pinned,
so their passing shows no more than that.

## Cause J: whole gates ended by a SIGTERM no log attributes (issue 6334)

### The symptom

Two of one round's whole gates on 2026-11-21 ended with nextest counting hundreds of tests that never
ran,
and naming no failing one:

```text
9160 passed ... 317 not run due to signal
8880 passed, 598 not run
```

nextest counts a test that a signal ended before it finished,
or before it started,
as "not run due to signal".
Nothing in either log says what sent the signal.
The gate's own deadline sends none:
it removes the container it stops with `podman rm --force`,
and the refusal that reaches the log names the bound it missed
(`bin/native-gate-container.ts`).

### What is known and what is not

On 2026-11-21 one round reported stopping three gate containers it had matched by creation time
(`trusting_poincare`,
`gallant_moore` and
`affectionate_diffie`).
Another round,
running the same gate image in the same hour,
listed `gallant_moore` as its own gate container,
and two of its whole gates ended with nextest counting tests as not run due to signal
(`9160 passed ... 317 not run due to signal`,
`8880 passed, 598 not run`).
`podman events` shows `died` and `remove` for each of those containers and no `kill`.
A `podman stop` also leaves only `died` and `remove`,
as a probe on 2026-11-21 measured,
so the events neither show nor exclude one.
Which session owned `gallant_moore`,
and whether one round's stop ended another round's gate,
is not established.
The gate's deadline is not known to send this signal:
it stops the client with `SIGKILL` (`bin/process-group-stop.ts`)
and removes with `podman rm --force --time 0`.
A unit stop does send it:
stopping a probe's unit sent SIGTERM through the `podman run` client to its container in every shape probed on
2026-11-21
(the gate's suite container itself was not tried),
so a stopped gate unit is a candidate,
and whether any gate unit of that day was stopped is not established.
An earlier reading here said a probe container kept running after its unit was stopped;
that probe read `podman ps`,
which can print `Up` for a container that has already ended,
while `podman inspect` showed it stopped with exit 143 at the moment of the unit stop.

A gate container of that day carried no `--name` and no `--label`,
and the gate printed no container ID,
so the image it ran,
the command in it and the moment it was created were the only things a session could match it by,
and none of those says who started it.

The same shape has a second witness on record.
The section "A gate that ends with status 137 and no memory kill" records another session removing two
containers it had taken for its own,
found the same way.
Its journal shows a podman process that was no client of those containers running `container remove`,
so the stop itself is known there;
whose gate each container belonged to is what neither record says.

### How to tell

Every gate now names itself and each container it starts in its own log,
before that container's output,
and records the same in its evidence directory.
Those lines are what a person compares with the machine's own record:

```text
Gate run id native-fvukQ7: every container this gate starts carries the label dev.monochromatic.git-policy.gate-run=native-fvukQ7, so `podman ps --filter label=dev.monochromatic.git-policy.gate-run=native-fvukQ7` lists exactly this run's containers and no other session's.
Gate container suite: name gpc-gate-native-fvukQ7-suite, label dev.monochromatic.git-policy.gate-run=native-fvukQ7, ID file target/verification/native-fvukQ7/suite.cid
Gate container clippy: name gpc-gate-native-fvukQ7-clippy, label dev.monochromatic.git-policy.gate-run=native-fvukQ7, ID file target/verification/native-fvukQ7/clippy.cid
Gate container suite: ID 53864b147d19baa324bd8246128479c2c0fe575485fb170b0ef848c13ee763f3 (name gpc-gate-native-fvukQ7-suite, label dev.monochromatic.git-policy.gate-run=native-fvukQ7)
Gate container clippy: ID 56fbf1bbc54d1e824fa4ca090dbd9fb9a55aef0049c17db719aa541cc4f45317 (name gpc-gate-native-fvukQ7-clippy, label dev.monochromatic.git-policy.gate-run=native-fvukQ7)
```

The ID file path each line prints is the path the runner gave that client,
which is absolute when the gate runs from the
package directory and relative to it otherwise;
the run id and the container names are the same either way.

`target/verification/<run id>/manifest.json` records the run id,
each container's name,
label and ID,
and the ID file each client wrote them to,
under `run` (`bin/native-gate-container-identity.ts`,
`bin/test-native-container.ts`).
These records name which container died and when.
They show a `kill`,
or a `rm --force` by another podman process,
as the ending it was;
a `podman stop` of a container whose client still runs leaves only `died` and `remove`,
the same as a normal end,
so a stop leaves no event of its own and the stopping session's own report is the record that says it happened.
Since every gate container now carries this run's name and label,
each record also says whose container it was:

- the stopping session's own report:
  what it stopped,
  and the name or ID its own command printed.
- `podman events`:
  `podman events --since <the time the gate started> --stream=false --filter container=gpc-gate-<run id>-suite --format '{{.Time}} {{.Status}} {{.Name}} {{.ID}}'`.
  This host's podman writes its events to journald,
  and this reads a container's events after the container is gone,
  so the question can be answered after the fact,
  for as long as the journal keeps them:
  on 2026-11-21 the user journal's oldest entry was hours old.
  Its `died` and `remove` lines carry the moment,
  the name and the ID:
  a probe on 2026-11-21 that stopped a container of its own read
  `1795299678 died gpc-events-probe-6334 f553f7a85eda...` and
  `1795299688 remove gpc-events-probe-6334 f553f7a85eda...`.
  A `podman stop` from outside the client leaves no event of its own;
  a `podman kill` leaves a `kill`.
- the user journal,
  which keeps podman's own lines,
  labels and all,
  after the fact:
  `journalctl --user --since '<the minute the gate stopped>' --grep 'gpc-gate-<run id>-suite'`.
  While this gate's client still runs,
  a `container kill` or `container remove` line printed by a podman process that is not that
  client is another podman command that ended it,
  the way the section "A gate that ends with status 137 and no memory kill" read one.
  After a unit stop the client is gone,
  so the container's own `died` and `remove`,
  and a later `podman rm` or `podman inspect`,
  are logged under other podman processes,
  and that alone is not evidence of an outside stop.
  A `podman stop` from outside the client logs nothing of its own:
  the client logs `died` and `remove`.
- the gate unit's own `Stopping` and `Stopped` lines in the user journal,
  in the same second as a `container died` line
  (logged by the client or by another podman process),
  which name a unit stop
  (not who asked for it).
  They carry the unit's description,
  not the run's name,
  so the `--grep` above misses them:
  `journalctl --user --user-unit=<unit>.service` finds them.

The label's run id is the one the gate printed before it built anything,
and `manifest.json` records the same name,
label and ID under `run.containers` while the run lasts,
so a line or event naming `gpc-gate-<run id>-suite` is matched to the run that printed it,
and a container carrying another run id was never this run's.

The run whose lines those are ended with the two standing cause-H failures and still recorded both containers:
`Summary [ 835.598s] 9478 tests run: 9476 passed (6 slow), 2 failed, 35 skipped`,
with the gate's own exit status 1,
and its manifest's `run.containers` names the suite's `53864b147d19...` and Clippy's `56fbf1bbc54d...`.
Its unit's journal holds podman's `init`,
`start`,
`died` and `remove` lines for both containers,
and each line names the container and its label,
so the same comparison works after the fact:

```text
container remove 53864b147d19baa324bd8246128479c2c0fe575485fb170b0ef848c13ee763f3 (image=localhost/git-policy-native-test:gpc-6334-containers-carry-a-run-label-20261121T172158-0500, name=gpc-gate-native-fvukQ7-suite, dev.monochromatic.git-policy.gate-run=native-fvukQ7, io.buildah.version=1.46.0, ...)
```

While it ran,
`podman ps --filter label=dev.monochromatic.git-policy.gate-run=native-fvukQ7` listed its suite container
alone,
its Clippy container having ended minutes earlier,
and the same filter with another run's id listed nothing.

A gate stopped by its own deadline says so in its refusal
(`podman run was still running after ... ms, the bound it is given`);
a status 137 with a memory kill names the kernel's own kill,
as the section named above records;
and a signal count with neither is a signal whose sender no record here names.

### The fix

Every container the gate starts now carries this run's own name and label,
`gpc-gate-<run id>-<suite|clippy>` and `dev.monochromatic.git-policy.gate-run=<run id>`,
named after the random name of the run's evidence directory and never after a clock
(`bin/native-gate-container-identity.ts`).
The gate prints the run id before it builds anything,
prints each container's name and label before that container starts,
prints its ID from podman's ID file before the container's own output,
and records all of it in `manifest.json` as each ID is read,
so a gate killed without unwinding still leaves the names,
labels and IDs behind
(`bin/native-gate-container.ts`,
`bin/test-native-container.ts`).
A session therefore stops only a container whose printed label is its own run's,
and the package README says where to read those lines and which command stops one's own gate
without naming a container
(`package/git-policy/cli/README.md`,
"Development").
A container carrying another run's label is another session's gate,
and is never stopped.

## Cause K: a commit-timing control counted the preparation's last steps inside the build (issue 6346)

### The symptom

Two whole container gates failed
`commit_timing::a_build_stopped_in_its_scratch_commit_still_states_its_time`
on the assertion at
`package/git-policy/cli/src/native/binary/binary_commit_timing_tests.rs:611`,
and the control passed alone in both rounds.
The assertion asks the build's stretches plus the Git steps that began inside its window
to fit the window plus the rounding of the figures summed.

The first failure was the second gate of the round that built the per-slot slot identity store
(branch `feat/git-policy-cli-zfs-slot-identity-store-per-slot-files` at `2b3e13f439`,
2026-11-21).
Its journal line was not kept;
what this document recorded then was an excess of 13 ms over the allowed rounding,
so the record's figures passed its `buildMs` by 25 ms.
Nothing can be quoted from that failure,
and this rewrite does not pretend to.

The second was the gate of
`feat/git-policy-cli-zfs-step-9-unmount-staging-and-reattach` at `2120f75dff`,
2026-11-22T17:35:47-05:00,
with 9945 tests run and 9942 passed,
the other two failures the standing cause-H pair.
Its lines,
the journal's own prefix stripped and every other byte as the journal wrote it
(unit `gpc-gate-native-nma30Q-suite[970970]`):

```text
thread 'commit_timing::a_build_stopped_in_its_scratch_commit_still_states_its_time' (175418) panicked at src/native/binary/binary_commit_timing_tests.rs:611:5:
the stretches and steps lie inside the stopped build (34 + 2071 of 2086): {"schemaVersion":1,"time":"2026-11-22T22:35:47.650Z","level":"warn","startMillis":1795386945351,"pid":175566,"release":"0.6.1-alpha.44","instance":"3abd20d1f0f99caa5fa0b83d90e14f6e","gitDir":"/work/package/git-policy/cli/target/gate-fixtures/native-binary-175408-timing-build-stopped/repo/.git","requestId":"a3b14421-1af0-434e-9406-2c420ba30c92","branch":"refs/heads/main","outcome":"stopped","exitCode":1,"stop":{"kind":"git-step","step":"git commit"},"totalMs":2299,"routeMs":101,"prepareMs":111,"attempts":1,"prepareSpans":[{"name":"index read","calls":1,"totalMs":0,"longestMs":0},{"name":"index copy","calls":2,"totalMs":0,"longestMs":0},{"name":"index seed","calls":1,"totalMs":0,"longestMs":0},{"name":"index equals base","calls":1,"totalMs":2,"longestMs":2},{"name":"index stat check","calls":1,"totalMs":2,"longestMs":2},{"name":"policy require-root at pre-forward","calls":1,"totalMs":0,"longestMs":0},{"name":"policy linked-worktree-only at pre-forward","calls":1,"totalMs":0,"longestMs":0},{"name":"policy branch-worktree-only at pre-forward","calls":1,"totalMs":0,"longestMs":0},{"name":"policy add-explicit at pre-forward","calls":1,"totalMs":0,"longestMs":0},{"name":"candidates prepared","calls":1,"totalMs":52,"longestMs":52},{"name":"policy final-newline at pre-forward","calls":1,"totalMs":66,"longestMs":66}],"prepareSpansDropped":0,"indexBuilds":[{"kind":"first","statDataKept":true}],"scratchSteps":[{"ceilingMs":2000,"armed":true,"stoppedAfterMs":2007,"heldAfterCeilingMs":null,"stallCeilingMs":null,"noticeMs":0,"wroteMs":0,"linesDropped":0,"looks":9,"looksFound":9,"below":[{"name":"git","seenMs":2006},{"name":"sleep","seenMs":1755}],"belowCpuMs":0,"belowUnlisted":false,"belowUnended":0,"blindLooks":0,"blockedLooks":0}],"buildMs":2086,"buildParts":{"branch":{"ms":0,"calls":1,"childrenMs":0},"message":{"ms":0,"calls":1,"childrenMs":0},"amend tree":{"ms":0,"calls":1,"childrenMs":0},"fitting request":{"ms":0,"calls":1,"childrenMs":0},"request":{"ms":0,"calls":1,"childrenMs":0},"shadow":{"ms":0,"calls":1,"childrenMs":0},"shadow hand-out":{"ms":8,"calls":1,"childrenMs":0},"shadow pool":{"ms":2,"calls":1,"childrenMs":0},"shadow skeleton":{"ms":6,"calls":1,"childrenMs":0},"shadow config":{"ms":3,"calls":1,"childrenMs":0},"shadow links":{"ms":6,"calls":1,"childrenMs":0},"shadow conclusion":{"ms":3,"calls":1,"childrenMs":0},"shadow refs":{"ms":5,"calls":1,"childrenMs":0},"client build":{"ms":1,"calls":1,"childrenMs":2023}},"buildPartsDropped":0,"provisionMs":0,"submitMs":0,"sends":0,"reachMs":0,"connectTries":0,"refusedConnects":0,"startWaitMs":0,"updateRounds":0,"releasesSeen":[],"answeredByRelease":null,"answerWaitMs":0,"positions":[],"waitingNotices":0,"lockHolderKind":null,"reconnects":0,"afterLandingMs":0,"gateMs":0,"handoffMs":0,"closingMs":0,"otherMs":0,"scratchRemovalMs":24,"removalsThatDidNotFinish":0,"stderrMs":0,"prepareParts":{"layout":{"ms":4,"calls":1,"childrenMs":0},"unmergedBefore":{"ms":0,"calls":1,"childrenMs":0},"capture":{"ms":2,"calls":1,"childrenMs":0},"scratch":{"ms":0,"calls":1,"childrenMs":0},"stagedDeletions":{"ms":3,"calls":1,"childrenMs":0},"index":{"ms":11,"calls":1,"childrenMs":12},"interactive":{"ms":0,"calls":1,"childrenMs":0},"conclusionState":{"ms":0,"calls":1,"childrenMs":0},"unmergedFiles":{"ms":2,"calls":1,"childrenMs":0},"changedPaths":{"ms":6,"calls":1,"childrenMs":0},"policies":{"ms":29,"calls":1,"childrenMs":36},"worktreeRecords":{"ms":0,"calls":1,"childrenMs":0},"treeCheck":{"ms":0,"calls":1,"childrenMs":0},"changes":{"ms":1,"calls":1,"childrenMs":0}},"preparePartsDropped":0,"longestQuietMs":54.802,"quietMs":{"/work/package/git-policy/cli/target/gate-fixtures/native-binary-175408-timing-build-stopped/slow-commit-bin/git rev-parse":29.83,"git add":8.997,"git cat-file --batch":11.755,"git version":54.802},"steps":[{"name":"git version","offsetMs":4,"durationMs":86},{"name":"git add","offsetMs":120,"durationMs":12},{"name":"git rev-parse","offsetMs":153,"durationMs":36},{"name":"git commit","offsetMs":250,"durationMs":2023}],"stepsDropped":0,"landing":null}
```

Its record's figures:
`routeMs` 101,
`prepareMs` 111,
`buildMs` 2086,
`totalMs` 2299,
`otherMs` 0,
`attempts` 1;
`git version` at 4 ms for 86,
`git add` at 120 for 12,
`git rev-parse` at 153 for 36,
`git commit` at 250 for 2023;
14 `buildParts` whose `ms` sum 34,
with `client build`'s `childrenMs` 2023;
`prepareParts` `index`'s `childrenMs` 12 and `policies`' `childrenMs` 36.

### The cause

The record was right and the control's window was wrong.
That is shown for the second failure,
whose record is quoted here;
the first failure's journal line was never kept,
so its cause is inferred from the same control and not shown.

`offsetMs` counts from the command's start
(`package/git-policy/cli/src/native/transaction/transaction_step_durations.rs`,
`figure_of`;
`package/git-policy/cli/SPEC.md`,
"offsetMs from the command's start"),
and the two phases before the build are the route and the preparation
(`CommitTiming::other`,
`package/git-policy/cli/src/native/commit/commit_timing.rs`;
`note_route` at `package/git-policy/cli/src/native/landing/service_commit.rs:181`,
`begin_build` at `:205`).
The build therefore begins at `routeMs + prepareMs`,
here 212 ms,
and the assertion began it at `prepareMs`,
111 ms:
a band as wide as the route,
holding every step that began in the preparation's last `routeMs` milliseconds.
`git add` began 19 ms into the preparation and `git rev-parse` 52 ms into it,
both inside that band and outside the build,
and `prepareParts` already states their 12 and 36 ms as the `childrenMs` of `index` and `policies`.
The assertion counted 48 ms twice.
The build's parts state 34 of its 2086 ms with 2023 under `client build`,
so 29 ms of the build is in no part;
48 - 29 = 19,
and 19 - 12 = 7,
the failure's excess exactly.
Read from 212,
the window holds `git commit` alone,
and 34 + 2023 = 2057 fits 2086,
and so fits 2086 + 1 as well.

The record is right on both sides:
58 + 48 = 106 <= 111 in the preparation,
34 + 2023 = 2057 <= 2086 in the build,
and 101 + 111 + 2086 = 2298 of 2299 with `otherMs` 0.
Nothing of the stopped `git commit`'s 2023 ms lies outside the build either:
`client build` states `childrenMs` 2023,
the step's whole duration,
so its stretch is at least as long as the step,
and the 16 ms between the notice's `stoppedAfterMs` 2007 and the step's 2023
is time the step ran past the ceiling,
consistent with its own end and reap on this host,
where the 17 clean stopped records of the second integration pass,
whose merge was `a33a1a6d16` at 2026-11-22T21:15:49-05:00,
show natural `git commit` steps lasting 2013 to 2091 ms against a 2000 ms ceiling,
and inside `buildMs`,
which is closed after the step's time is read
(`note_step(step, began, began.elapsed())` at
`package/git-policy/cli/src/native/commit/commit_preparation.rs:281`,
after `run_guarded_child` at `:275`;
`close_an_open_build` at `package/git-policy/cli/src/native/commit/commit_command.rs:418`).
No stretch is stated twice:
14 parts,
`calls` 1 each,
2057 under 2086.
No Git child spans a build seam:
the build's one child is stated wholly in one part.

Rounding is not it either,
and neither is counting the figures.
Every figure the accounting sums is a duration cut to whole milliseconds on its own
(`package/git-policy/cli/src/native/commit/commit_prepare_part_durations.rs:180-182`,
`package/git-policy/cli/src/native/transaction/transaction_step_durations.rs:264-265`,
`package/git-policy/cli/src/native/commit/commit_timing.rs:613`),
and a cut never raises a sum:
a stretch's `ms` and `childrenMs` are two cuts of one duration,
and `floor(a) + floor(b) <= floor(a + b)`,
so a right record states the build's parts and the steps that began in them at `buildMs` or under it.
The one cut that can add is the window's origin,
`routeMs + prepareMs`,
two cuts that can begin the window up to two whole milliseconds early;
the step such a band admits is one the preparation had awaited,
so it ended before the preparation did,
ran in under those two milliseconds,
and states at most one whole one.
The controls hold that one whole millisecond (`WINDOW_ORIGIN_ROUNDING_MS`),
and the failure's 2105 passes 2086 + 1 = 2087 by 18,
so no cut decides it.
In the 34 clean records of the second integration pass,
whose merge was `a33a1a6d16` at 2026-11-22T21:15:49-05:00,
the excess was -12 to -3 ms
(stopped control -12 to -3,
landing control -5 to -3),
so the bound leaves 4 to 13 ms of headroom,
and never less than 4.
The fixed 12 ms allowance was larger than any right record needs,
so a step stating 10 or 14 ms too much passed it,
and a count of figures is larger still:
the second integration pass,
whose merge was `a33a1a6d16` at 2026-11-22T21:15:49-05:00,
read the count as 16 on the stopped control and 19 on the landing one,
where a planted step stating 20 ms more than it ran passed both.

Every other way this could be false was checked and excluded:
the build's own figure by `otherMs` 0 and 101 + 111 + 2086 = 2298 of 2299,
a record of several builds by `attempts` 1,
a step list cut short by `stepsDropped` 0,
and the window's own origin,
which can admit only a step that began in the two milliseconds before the build and ran in less than those two,
by the two preparation steps beginning 92 ms and 59 ms before it.
A step that began in the build's last `routeMs` was not counted at all,
so the same off-by-a-route could also hide a defect at the window's end;
the window now ends where the build does.

Why the control passes alone:
the band is as wide as `routeMs`,
and a preparation step it takes in ran only until the preparation ended,
so it adds at most its own few milliseconds to a sum the build leaves 3 to 5 ms uncovered.
The 17 clean stopped records of the second integration pass,
whose merge was `a33a1a6d16` at 2026-11-22T21:15:49-05:00,
read `routeMs` as 3 to 7 ms and stayed green.
The gate's record states 101 ms,
and the band took in both preparation steps,
which had run 12 and 36 ms.
Host load is no cause here (rule `NQH`);
what load changes is how much of the preparation falls inside the band,
because the band is as wide as the route the load stretched.

### The reproduction

The fixture's stand-in Git was made to sleep 400 ms on `version`,
a step of the route,
and 200 ms on `add`,
a step of the preparation.
Neither sleep adds a window:
the route is already before the preparation,
and the band is already as wide as the route.
The control then failed on this host,
at the same assertion:

```text
    thread 'commit_timing::a_build_stopped_in_its_scratch_commit_still_states_its_time' (3702717) panicked at src/native/binary/binary_commit_timing_tests.rs:611:5:
    the stretches and steps lie inside the stopped build (11 + 2243 of 2045): {...}
```

That record states `routeMs` 436,
`prepareMs` 275 and `buildMs` 2045,
with `git add` at 477 ms for 208,
`git rev-parse` at 693 for 6 and `git commit` at 722 for 2029,
and `prepareParts` states the two preparation steps' time as
`index`'s `childrenMs` 208 and `policies`' `childrenMs` 6.
Read from `prepareMs` 275,
the window selects all three steps,
2243 ms,
and 11 + 2243 passes 2045 + 12 by 197 ms
(and 2045 + 1 by 208).
Read from 711 it selects `git commit` alone,
and 11 + 2029 = 2040 fits 2045,
and so fits 2045 + 1 as well.

No whole-suite rerun was used to wait for this failure:
the delay was planted in the stand-in and one run of the control showed it.

### The fix

`package/git-policy/cli/src/native/binary/binary_commit_timing_tests.rs`:

- `build_window` states the window from `routeMs + prepareMs` to
  `routeMs + prepareMs + buildMs`,
  and both controls assert `attempts` 1,
  because a record of several builds is several windows and no single pair states their union;
- `WINDOW_ORIGIN_ROUNDING_MS` is the bound:
  one whole millisecond,
  because every figure summed is a cut that cannot raise a sum,
  and the window's origin,
  two cuts,
  can admit only a preparation step that ran in under two milliseconds;
- both controls assert that the window holds the scratch `git commit`,
  so a frame that moved cannot leave an empty selection that passes by adding nothing;
- `a_preparations_last_steps_are_not_counted_inside_the_build` holds this gate's own record:
  it derives the window,
  the selected step,
  and the sums from that record,
  and asserts that the old reading's 2071 ms passes even the origin's bound.
  With `steps_inside_the_build` reading its window from `prepareMs`,
  the control fails at its step list:
  `left: ["git add", "git rev-parse", "git commit"]` against `right: ["git commit"]`,
  and those three steps state 2071 ms against the record's 2086 ms build,
  the sum the gate printed as `34 + 2071 of 2086`.
  With the window read from the route and the preparation it passes.

`package/git-policy/cli/SPEC.md` now says what the build's window is,
so a later reader does not derive it again.

Nothing in the record changed for the second failure,
whose record is quoted here and was right;
the first failure's record was never kept.

## Cause L: a control wrote over the PID file it had judged, within one millisecond (issue 6342)

### The symptom the gate reader saw

`lock::abandoned_lock_proving_tests::the_pid_file_of_a_git_that_takes_the_freed_name_is_left_alone`
failed as test 3457 of 9768,
after 0.018 s,
in the whole container gate `native-0g2PJW`
(unit `odb-declined-gate-final-20261122`,
2026-11-22T02:02:25-05:00)
of the object-reader round on `fix/git-policy-cli-object-database-reftable-replace-refs`,
a branch that changes nothing under `src/native/lock/`:

```text
thread 'lock::abandoned_lock_proving_tests::the_pid_file_of_a_git_that_takes_the_freed_name_is_left_alone' (…) panicked at src/native/lock/abandoned_lock_proving_tests.rs:655:5:
the new holder keeps its own PID file
```

The same branch's next gate (`native-b9mIYV`) passed it,
and every other gate on record did.
That unit's user journal has since left retention:
`journalctl --user --user-unit=odb-declined-gate-final-20261122.service` answers `-- No entries --`,
so the run's own numbers are the issue's and the reproduction's measured in section The reproduction and what it measured,
and not a kept log.

### What the assertion needed

`a_git_directory` makes the fixture,
`plant` writes the `index.lock`,
the PID file beside it and the record of a dead holder,
and the control arms the fault point `UNLINKED_THE_LOCK`
(`src/native/lock/abandoned_lock_proving.rs`,
`reach`)
with code that acts as a Git that takes the freed name:
it writes `index.lock` and then writes `pid <new holder>` over the file at `index~pid.lock`.
The assertion at `:655` needs a regular file at `index~pid.lock` once `clear_one` has reported a clearance.

`clear_one` unlinks a PID file in one place only:
the block under `reach(UNLINKED_THE_LOCK)`,
and only when what is read there equals what was read before the lock's unlink:
the same device,
the same inode,
the same `ctime_ms`,
and the same contents,
which entail the number the file names
(section The fix says why and what it costs).
So the file could be gone in one of these ways:

- the removal unlinked it.
  That is the only code in the tree that removes this name,
  and it needs the hook's write to leave the judged inode where it was and the change time in the judged millisecond.
  `std::fs::write` opens the existing file with `O_TRUNC`,
  so the inode and the device stay the judged ones,
  and `read_lock_metadata` floors the change time to whole milliseconds
  (`src/native/lock/index_lock_evidence.rs`,
  `metadata.ctime() * 1000 + metadata.ctime_nsec() / 1_000_000`),
  so a write that close behind the plant is not told apart from it;
- the hook wrote nothing.
  It cannot have:
  the assertion before it (`lock.exists()`,
  `:654`) passed,
  and the lock exists again only once the hook wrote it,
  and a failed second write inside the hook would have panicked with the hook's own message;
- another process removed it.
  Nothing else names this fixture:
  the fixture root is per test process (`src/native/testing/test_support.rs`,
  `process-<pid>`),
  the directory is named for the test,
  `nextest` gives each test a process of its own,
  the fault hooks are thread-local (`src/native/service/service_fault_hooks_test_support.rs`,
  `HOOK`),
  and no other control makes a fixture called `taken-name`;
- the removal reported nothing,
  or reported before the hook.
  Then `:653` would have failed on a lock that is still there.

The first is the one the assertion took.

### The cause

Four facts make it,
and the first is the control's:

- The hook wrote the new holder's PID file with `std::fs::write`,
  which truncates the very file the removal had judged.
  What stood at that name after the hook was the judged file with other bytes,
  not a file of the new holder's own,
  and the inode and device the removal compared were the judged ones.
- Git never puts a PID file of its own at a name that is taken.
  `create_lock_pid_file` opens `<name>~pid.lock` with `O_WRONLY | O_CREAT | O_EXCL`,
  and writes nothing when that is refused
  (`lockfile.c`,
  Git 2.56.1,
  the release the gate's image reports as `git version 2.56.1`),
  and `lock_file` calls it only after the lock itself was taken.
  A live Git that takes the freed `index.lock` therefore writes no PID file at all,
  and has nothing there to lose.
  That is the control's own account,
  and the integration pass confirmed it in the source.
- Another program does write its own number over that file,
  through the same inode,
  and the pass found it:
  `gitbutlerapp/grit` writes Git's sidecar with an open that creates and truncates
  (`grit-lib/src/lockfile.rs`,
  `write_lock_pid_file`,
  lines 631 to 640 at `d1637255d9`,
  read here in a shallow clone,
  not run),
  after it has taken the lock with an exclusive create
  (`grit-lib/src/index/core.rs`,
  `IndexLockFile::acquire`,
  lines 4215 to 4231),
  and it takes `core.lockfilePid` as on when it is unset
  (`grit-lib/src/lockfile.rs`,
  `grit_lockfile_pid_enabled`,
  lines 602 to 605).
  A grit that takes the freed `index.lock` there truncates the judged `index~pid.lock`:
  same device,
  same inode,
  and only its change time and its bytes to tell it from the judged file.
- The removal's last look compared `device`,
  `inode` and `ctime_ms`,
  and `ctime_ms` is floored to whole milliseconds,
  so a write over the judged file left one millisecond to decide with.

So the control planted a real program's state and the removal could not tell it apart,
which is at once the control's defect and a route in the product.
The branch's first fix treated only the first:
it made the control write a file of its own and said in `SPEC.md`,
in a comment and in this section that a program which wants its own file there removes the judged one first,
which the inode shows.
The pass showed that sentence false with grit,
and the route that follows from it is the product's.

### The reproduction and what it measured

The runs of this round are the one control in a process of its own,
started from the host's build of this crate,
with `CLI_GIT_TEST_ROOT` on a tmpfs this round made under `/dev/shm`,
which the round's own rules forbade for scratch and which changed no number:
the same file system type is the gate's fixture root.
The build is the tree at the incident with a print of the judged and the later metadata beside the removal,
and nothing else changed.

- The control as it stood,
  200 runs:
  0 failures and no pair in one millisecond.
  The measured `ctime_ms` differences were 1 ms in 91 runs,
  2 ms in 98,
  3 ms in 4,
  4 ms in 3,
  5 ms in 2,
  6 ms in 1 and 30 ms in 1.
- 120 more runs with the two change times printed in nanoseconds:
  0 failures and no pair.
  The differences were 1 ms in 50 runs,
  2 ms in 60,
  3 ms in 9 and 30 ms in 1.
- The same control with the plant's own bytes written into the PID file again immediately before the call,
  100 runs:
  10 failures,
  and they were exactly the 10 runs whose two change times shared a millisecond.
  The file,
  the number,
  the birth and the bytes were the judged state's;
  only the change time the removal compares moved to a moment just before the call,
  which is the distance a plant on a fast machine leaves.
  One recorded failure:

```text
PROBE6342 judged ctime=1795382115.181238873 still ctime=1795382115.181937201 delta=0 tie=true

thread 'lock::abandoned_lock_proving_tests::the_pid_file_of_a_git_that_takes_the_freed_name_is_left_alone' (1163160) panicked at src/native/lock/abandoned_lock_proving_tests.rs:659:5:
the new holder keeps its own PID file
```

  The two are 698 µs apart and in one millisecond,
  and their nanoseconds differ,
  which the account under The reproduction and what it measured read as one tick of a coarse clock.
  That reading was wrong:
  every value ending in `…060000` ns is what a stamp taken with no read between two writes looks like,
  and the pair that failed has nanosecond values of its own.

What this host's clocks and file systems really give,
measured by the integration pass of 2026-11-22 with a probe of 1000 rounds of write,
read and rewrite in place
(the pass's numbers,
`kernel 7.3.5-1-cachyos`,
`CONFIG_HZ=1000`):

- On this host's ZFS,
  where every worktree and the scratch of that pass sat,
  the rewrite's change time equalled the judged one to the nanosecond in 957 of 1000 pairs and was later in 43:
  ZFS stamps a change to a jiffy.
  After a remove and a create the inode number came back in 0 of 1000.
- On the gate's fixture tmpfs,
  inside the gate's own base image,
  the rewrite's change time was strictly later in nanoseconds in 1000 of 1000 pairs and in the same whole millisecond in
  967,
  and the inode number came back in 0 of 1000.
- So a finer comparison,
  in nanoseconds,
  closes nothing on this host's ZFS and something on the gate's tmpfs,
  and a comparison of the bytes closes it on both.
  That is the pass's finding,
  and the claim under The reproduction and what it measured that a millisecond of the clock is a window the control enters rarely holds only
  because the untouched control puts one to two milliseconds between the plant and the write after the unlink.

### What the comparison is now, and why

The last look at the PID file compares its device,
its inode,
its change time in whole milliseconds,
and its contents,
which are read again there
(`pid_file_contents`,
`abandoned_lock_proving.rs`,
the block under `reach(UNLINKED_THE_LOCK)`).
A program that wrote anything else over the judged file is told apart by those contents at every clock granularity and
on every file system,
which the metadata alone could not promise.
It compares the file's contents byte for byte with the contents it judged,
and the one case left open is a program that writes the very same bytes back,
and the instant between the last read and the unlink.

The number the file names is no longer parsed at that look:
equal contents entail the judged number,
so the comparison of the number was strictly weaker than the comparison of the contents,
and the judged contents are read once before the lock goes and give both the number the proof was already asked for
and the bytes the look compares.
The removal therefore reads the file as often as before,
once at the judged point and once at the last look.

The project chose this on 2026-11-22,
in the question tool,
as "B:
also compare bytes (Recommended)",
whose words were "Before deleting the PID file,
read it again and delete only if its contents are also unchanged",
over correcting these words alone and leaving the route open,
and the record is `doc/decision/cli-git-lock-cleanup-pid-file-contents-compared-before-unlink.md`.
The limit the owner was shown with that choice is the one case named under What the comparison is now,
and why:
a program that rewrote the very same bytes in place would still lose the file.

What it costs,
from the code:

- one bounded read of the PID file at the judged point and one at the last look:
  `pid_file_contents` opens the name without following a link and without blocking,
  and reads at most `PID_FILE_LIMIT`,
  which is 4096 bytes
  (`stopped_git_locks.rs`,
  `PID_FILE_LIMIT`),
  of a file that holds `pid <number>` and a newline;
- it sits on the abandoned lock's removal,
  not on a commit's waiting path:
  `clear_one` is reached from `clear_abandoned_locks`,
  which the invocation's entry runs when it meets lock files of the working directory
  (`invocation/entry.rs`,
  `meet_the_locks_of_the_working_directory`)
  and the landing runs when it takes the index lock
  (`landing/service_land_take_index_lock.rs`),
  and it runs only once the proof has already decided to unlink the lock;
  the waits a commit makes for a live holder are in `index_lock_wait.rs` and `index_holder_wait.rs`,
  which this does not touch.

Two states stay open,
and both are recorded here so the next reader does not have to find them:

- a program that writes the very same bytes back,
  whether over the file or into a new one,
  is not told apart:
  the file goes,
  and what it costs is that program's PID file,
  which named a process this boot has not.
  Its lock stands,
  because the lock was unlinked before this look;
- the instant between the last read and the unlink is not closed:
  no system call unlinks a name only when it still holds given contents
  (`service_index_unheld_removal.rs` says the same of the lock's own removal).

### The fix

`src/native/lock/abandoned_lock_proving.rs` is the file whose comparison changed,
with `src/native/lock/abandoned_lock_proving_tests.rs`,
the test-only `src/native/lock/lock_metadata_arm.rs`,
`src/native/lock/stopped_git_locks.rs` for the one read that gives the contents,
and the bullet in `SPEC.md` that had said what the control and the product had wrong.

The arm answers the removal's reads of one path,
so each term of the comparison can be made to differ on its own:
a device number a fixture cannot present,
a change time this host's ZFS hands out in jiffies,
an inode number a file system may hand back to the next file,
and one change time both reads report,
so no clock decides what a control holds.
It tells the removal's judged read from its last look by the phase the removal is in,
which a control's hook moves at the lock's unlink,
so a judged read taken after that instant is answered as a later one,
and a removal that read the file in the wrong place fails its control.
The contents are always the file's own,
so a control that writes them writes a state.

- `a_pid_file_whose_number_gained_a_leading_zero_is_left_alone`,
  `a_pid_file_with_trailing_padding_is_left_alone`,
  `a_pid_file_with_a_carriage_return_before_the_newline_is_left_alone` and
  `a_pid_file_with_no_newline_is_left_alone`
  each have the hook write the judged number in other bytes over the judged file:
  every one of those parses to the judged number,
  and none of them is the bytes the removal judged.
  Each file stands,
  which is the pin of the comparison of the contents.
- `a_pid_file_with_the_very_bytes_it_was_judged_with_goes_with_the_lock`
  has the hook write the very bytes the file was judged with,
  under one change time for both reads:
  all four facts equal the judged ones,
  so the removal takes the file,
  and the control says so and says what that costs.
  This is the one case the comparison leaves open.
- `a_pid_file_another_program_wrote_over_the_judged_one_is_left_alone`
  holds grit's state:
  the hook writes the new lock and then writes its own number over the judged PID file,
  through the same inode,
  with the arm reporting one inode for every file and one change time both reads report.
  The file stands,
  and the control says what the real program does.
- `a_pid_file_of_another_inode_is_left_alone`
  has the hook remove the judged file and write the judged number again,
  a new file with the same bytes,
  and the arm reports one inode for every read and another for the read taken after the lock's name went:
  the inode is the term that decides,
  whichever number the file system gives the new file,
  and dropping the arm's answer makes the removal compare one inode number with itself and take the file.
- `a_pid_file_of_another_change_time_is_left_alone`
  arms the later read with a change time one millisecond back and touches nothing else.
- `a_pid_file_of_another_device_is_left_alone`
  arms the later read with the next device number and touches nothing else.
- `a_git_that_takes_the_freed_name_writes_no_pid_file_and_keeps_its_lock`
  holds what a Git does at that instant:
  its exclusive create is refused,
  its lock stands,
  and the abandoned holder's file goes with the abandoned lock.
- `an_arm_is_put_back_when_the_control_panics`
  panics inside the arm's body and finds the arm gone afterwards,
  which holds the drop guard `with_arm` puts it back with.

Each plant was applied to the committed product file and the whole module run,
then the file was put back:

```text
term left out: device            a_pid_file_of_another_device_is_left_alone
panicked at src/native/lock/abandoned_lock_proving_tests.rs:1053:5: a PID file of another device is left alone
term left out: inode             a_pid_file_of_another_inode_is_left_alone
panicked at src/native/lock/abandoned_lock_proving_tests.rs:990:5: a PID file of another inode is left alone
term left out: change time       a_pid_file_of_another_change_time_is_left_alone
panicked at src/native/lock/abandoned_lock_proving_tests.rs:1023:5: a PID file whose change time reads another millisecond is left alone
contents put back to the number  a_pid_file_whose_number_gained_a_leading_zero_is_left_alone
panicked at src/native/lock/abandoned_lock_proving_tests.rs:876:5: a PID file whose number gained a leading zero is left alone
contents put back to the number  a_pid_file_with_trailing_padding_is_left_alone
panicked at src/native/lock/abandoned_lock_proving_tests.rs:903:5: a PID file with spaces before its newline is left alone
contents put back to the number  a_pid_file_with_a_carriage_return_before_the_newline_is_left_alone
panicked at src/native/lock/abandoned_lock_proving_tests.rs:930:5: a PID file with a carriage return before its newline is left alone
contents put back to the number  a_pid_file_with_no_newline_is_left_alone
panicked at src/native/lock/abandoned_lock_proving_tests.rs:957:5: a PID file with no newline is left alone
unlink planted away              a_lock_of_a_dead_recorded_holder_with_a_free_number_is_cleared
panicked at src/native/lock/abandoned_lock_proving_tests.rs:267:5: the PID file of a cleared lock goes with it
unlink planted away              a_git_that_takes_the_freed_name_writes_no_pid_file_and_keeps_its_lock
panicked at src/native/lock/abandoned_lock_proving_tests.rs:1167:5: the abandoned holder's PID file goes with its lock, and the new holder has none to lose
```

Each run exited 100,
every failing control is named,
and the module was green before the first plant and after the last one
(`Summary [0.176s] 22 tests run: 22 passed, 8632 skipped`).

The pass after the merge moved the judged metadata read to after the lock's unlink,
which makes the three metadata terms compare one file with itself,
and every control passed then:
the arm answered a control's reads by counting them,
so a judged read taken in the last look's place was still answered as the judged one.
The arm answers by the phase of the removal now,
and that plant fails the three controls whose whole difference is one of those three terms
(`Summary [0.097s] 22 tests run: 19 passed, 3 failed`):
`a_pid_file_of_another_inode_is_left_alone`,
`a_pid_file_of_another_change_time_is_left_alone` and
`a_pid_file_of_another_device_is_left_alone`.
Dropping the arm's inode answer alone fails `a_pid_file_of_another_inode_is_left_alone` and nothing else
(`Summary [0.102s] 22 tests run: 21 passed, 1 failed`),
which is why that control arms one inode for every file.

### What the product guarantees here

The removal unlinks the PID file beside a lock it cleared only when that file is still the one it judged,
as its device,
its inode,
its change time in whole milliseconds and its contents show.
It compares the file's contents byte for byte with the contents it judged,
and the one case left open is a program that writes the very same bytes back,
and the instant between the last read and the unlink.
Because equal contents entail the judged number,
a program that writes any other bytes there is told apart on every file system and at every clock granularity,
and the number the proof already asked for costs no second read.
For a Git,
nothing else can be at that name while the judged file stands,
because Git creates its PID file with `O_EXCL` and silently writes none when the name is taken,
so a live Git that takes the freed name has no PID file there to lose.
What is not promised is the state where the very same bytes are written back,
which costs that writer the PID file and never its lock,
and the instant between the last read and the unlink,
which no system call closes for a name.


The whole container gate then ran the same tree,
in run `native-a5XyIb` of image
`localhost/git-policy-native-test:gpc-6342-lows-20261122T2350`,
on commit `1e52bbd00b`:

```text
Summary [1220.043s] 9918 tests run: 9916 passed (9 slow), 2 failed, 36 skipped
FAIL [   0.939s] (9484/9918) monochromatic-git-policy-cli::native_binary repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone
FAIL [   0.377s] (9602/9918) monochromatic-git-policy-cli::native_binary service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing
```

Both failures are the cause-H pair,
and every control of this section passed in that run.
Its Clippy step finished clean
(``Finished `dev` profile [unoptimized] target(s) in 4m 53s``),
and the unit ended `Failed with result 'exit-code'`,
which is the cause-H pair's status and not this section's control.

## Cause M: a shadow control counted its starts in a file the cut start writes (issues 6343 and 6344)

This is neither cause G nor cause H.
Cause G is the label this document gives failures that one busy full run had and no other run repeats,
with no cause recorded;
here the failure's own sentence decides the product's side of it.
Cause H is the standing scratch-directory pair,
which fails here and on the Btrfs host.

The failure was fixed in two rounds.
Issue 6343 made both shadow controls count the attempts the wrapper itself made,
instead of a file the started program wrote.
Issue 6344 removed the two timing windows that control had left:
the stand-in decided which start to hang from a file the cut start writes,
and the sibling control's real second start had to finish inside a shortened ceiling.
The first round is described as it was merged,
and the second round follows it under its own headings,
because the two windows are separate failures of one control.

### The symptom the gate reader saw

The whole container gate of 2026-11-22T14:26:57-05:00
(run `native-XHGDqg` of image `localhost/git-policy-native-test:odb-lows-20261122`)
failed this control beside the two cause-H tests,
on a branch that does not touch `src/native/shadow/`,
and its summary was:

```text
Summary [ 900.425s] 9778 tests run: 9775 passed (8 slow), 3 failed, 36 skipped
```

The control's own lines were:

```text
FAIL [ 1.539s] (6057/9778) monochromatic-git-policy-cli shadow::shadow_create::tests::a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing
  stderr ───
    thread 'shadow::shadow_create::tests::a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing' (138952) panicked at src/native/shadow/shadow_create_tests.rs:546:5:
    assertion `left == right` failed: GitFailure("git init --bare --template= --object-format=sha1 --ref-format=reftable /work/…/shadow/0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10, a bulk step ran for 0 seconds, the longest the wrapper lets one step run, and was ended")
      left: "1"
     right: "2"
```

The fixture path is elided at `…`;
every other byte is the journal's
(`journalctl --user --user-unit=odb-lows-gate-20261122.service`).
The control passed alone straight afterwards,
and 1.539 s is the two 500 ms ceilings the control sets plus its fixture.
A control that passes alone and fails in its suite is a contention or ordering defect until measured otherwise,
and host load is no cause here.

### What the assertion compared

`left` was the trimmed content of the file `init-starts` in the control's fixture root
(`shadow_create_tests.rs:546` as it stood then),
and `right` was `SHADOW_CREATION_ATTEMPTS`
(`shadow_create_init.rs:48`,
in the module issue 6344 split out of the creation),
which is 2.
That file had one writer:
the Git shim the control installs as the real Git,
which `reftable_with_hanging_init` built.
It was a `/bin/sh` script that hangs every `init` start,
and before it hangs it reads one number from the file,
adds one and writes it back.
So `"1"` was the number of starts whose program reached that line,
and `"2"` was the number of attempts the control meant to see.
No other test could write it:
the fixture root carries the process id and the test's own name
(`test_support.rs`,
`fixture_with_prefix`),
and the shim was the file's only writer.

### The two attempts, from the failure's own sentence

The sentence in the `GitFailure` is `step_ended_at_its_ceiling_text`
(`child_watch_step_bounds_and_errors.rs:328-335`):
it says neither "left running" nor any reason,
so the watch proved the child and ended it,
and the step it names is a bulk step.
`CarriedCeiling::ending` writes that sentence into an error of kind `TimedOut`
(`step_output_read_whole_through_pipes.rs:720`),
and `checked_from` returns a `TimedOut` outcome's text with no prefix at all
(`transaction_git.rs`,
the arm of `checked_from` that matches `TimedOut`);
any other kind would have read `could not start`.
`start_was_cut_or_refused` (`shadow_create_init.rs:120`) reads `TimedOut` as a cut,
so at attempt 1 the loop took its `continue` and started again
(`shadow_create_init.rs:95`),
which makes the sentence in the failure attempt 2's.
`init` is in none of `RERUNNABLE`
(`SPEC.md`,
the paragraph beginning with "`RERUNNABLE` names neither"),
so one `run_git` call starts it once and never again inside that call.
Two attempts were therefore made.
What the record does not say is which attempt's write the file lost,
or how the first attempt ended;
the measurements this section records settle the second question and not the first:

- The first attempt is known only as cut or refused.
  Those are the two outcomes that make the loop continue,
  and a refusal is not an ending at a ceiling.
- A start the system refuses at its spawn pauses before it returns:
  `spawn_retrying` pauses 10 ms,
  doubling to the longest pause,
  and returns the refusal when the next pause would pass `START_RETRY_BUDGET`
  (`push/start_retry.rs:33`),
  which is fourteen pauses summing to 4630 ms.
  Measured by lowering the test process's own process limit so every spawn was refused:
  the attempt's error read `the start was refused 15 times over 4630 ms`,
  and the control waited 9.52 s for its two attempts
  (issue 6344's integration pass,
  `keep/merge-run-both-spawns-refused-by-process-limit.txt`).
  A first attempt refused that way costs more than five seconds,
  so the gate's 1.539 s total excludes it.
- What the clock does not exclude is a refusal returned as a finished command:
  there the program ran and printed the system's words,
  and the wrapper reads `refused_by_the_system`.
  The stand-in's shell did not produce one when a fork refusal was forced:
  the gate image's `/bin/sh` is bash 5.3.20,
  a stand-in with a one-process limit printed
  `sh: fork: retry: Resource temporarily unavailable` four times
  and returned nothing in twelve seconds,
  and the control's 500 ms ceiling ends such a start at its ceiling
  (issue 6344's integration pass,
  `keep/fork-refusal-limit-1.txt` and `keep/fork-refusal-old-shim-limit-1.txt`).
  So the first attempt was cut,
  and which attempt's write was lost stays inseparable.
  The limits of that:
  one shell version,
  a persistent one-process limit and not one that flickers,
  and a refused thread read in code,
  where the runtime falls back to the current thread and returns no `WouldBlock`,
  and not run.
- An attempt refused at its spawn ran no program at all,
  so the file's `1` would then be the correct number of programs started and no write was lost.
- On a shell that does not retry its own refused fork,
  a stand-in whose `$(cat ...)` is refused a process writes a wrong number,
  `n` empty and `$((n + 1))` writing `1`,
  which the list under "Why a start can reach its ceiling before its program writes" names among the control's own defects.
  bash 5.3.20 retries that fork,
  so on the gate's shell the start is cut at its ceiling and writes nothing.

What the record alone decides:
attempt 1 was not skipped,
and attempt 2 was ended at its ceiling,
which is what the loop's code and the sentence say.
That the first attempt was cut follows with the measurements this section records,
whose limits are named beside them;
which attempt lost the write,
neither the record nor those measurements separate.

### Why a start can reach its ceiling before its program writes

The ceiling is the one the control shortens for its thread,
500 ms
(`STEP_CEILING_FOR_THIS_THREAD`,
`transaction_git_step_bounds.rs:54`),
and its clock begins once the child is spawned:
`command.spawn_retrying()`,
then `watch.begin()`,
both in `child_watch_one_shot_run.rs`,
and then the wait that carries the rest of the ceiling.
It ends the whole process tree wherever the work has got to.
The script reaches its number in three steps:
its first statement is the argument loop,
the shell must start and parse the script,
and `$(cat ...)` forks a `cat` before the number is read or written.
A start whose program has not reached that line by the ceiling is ended without writing,
and the product is right to end it:
a start that shows nothing for the ceiling is exactly the cut this control asks for.

Every way the count can come out one short,
with whose defect each one is:

- the control's:
  the first attempt is ended at its ceiling before its shim writes,
  so the second attempt writes `1`;
- the control's:
  the second attempt is ended at its ceiling before its shim writes,
  so the file keeps the first attempt's `1`;
- the control's,
  on a shell that does not retry its own refused fork:
  the shim's `$(cat …)` is refused a process under the container's process bound,
  and that start ends before its write,
  so `n` is empty and `$((n + 1))` writes `1` in place of `2`.
  bash 5.3.20 retries that fork,
  so on the gate's shell the start is cut instead;
- a product defect,
  ruled out here:
  the loop skips an attempt.
  The failure's sentence is a cut whose `continue` ran,
  so both attempts ran;
- another product defect,
  ruled out:
  a cut is taken for something else.
  The product keeps no count of its own to be taken instead;
- another test's write,
  ruled out:
  the fixture root and the file's single writer are the control's own.

The container promises the control none of this.
The gate runs four tests at once and sizes the container's processes to them,
1024 of them (`bin/native-gate-container-limits.ts`),
whose own comment records the suite peaking at 401 processes,
and three tests failing on `fork` with `Resource temporarily unavailable` at the 512 bound.

### The reproduction, forced rather than repeated

One planted delay widens the window the code already has and changes nothing else:
a start that finds the count file already there sleeps instead of writing its own number,
which is what a start ended before its write does.
The shim's script,
with the planted line marked:

```sh
#!/bin/sh
for argument in "$@"; do last="$argument"; if [ "$argument" = init ]; then is_init=1; fi; done
if [ -n "$is_init" ]; then
  if [ -f '<count>' ]; then exec sleep 600; fi   # planted: a start after the first never writes
  n=$(cat '<count>' 2>/dev/null || echo 0); n=$((n + 1)); echo $n > '<count>'
  if [ $n -le <hang> ]; then mkdir -p "$last"; : > "$last/PARTIAL"; exec sleep 600; fi
fi
exec /usr/bin/git "$@"
```

`mise run //package/git-policy/cli:native:test:host -- shadow::shadow_create::tests::a_shadow_whose_init`
on this host (git 2.56.1) then failed with the journal's own two lines:

```text
    thread 'shadow::shadow_create::tests::a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing' (1035181) panicked at src/native/shadow/shadow_create_tests.rs:546:5:
    assertion `left == right` failed: GitFailure("git init --bare --template= --object-format=sha1 --ref-format=reftable /home/…/shadow/0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10, a bulk step ran for 0 seconds, the longest the wrapper lets one step run, and was ended")
      left: "1"
     right: "2"
```

The window is two attempts wide in this control,
because either attempt's write can be lost.
The round that wrote this section left the sibling control's own window open;
the issue 6344 sections of this cause close both windows of it.

### The fix of issue 6343: the wrapper counts its own attempts

The wrapper now counts its own attempts,
and both controls count that.
`note_init_attempt` (`shadow_init_attempts.rs:157`) adds one to a thread-local before each
`run_git` of the loop (`shadow_create_init.rs:84`),
and `init_attempts` (`shadow_init_attempts.rs:185`) reads the count.
The item is `#[cfg(test)]` and per thread,
as the link-question count of `shadow_pool_member.rs` is,
so no other test's creation reaches it.
`a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing`
asserts that count against `SHADOW_CREATION_ATTEMPTS`
(`shadow_create_tests.rs:615`),
and the sibling control `a_shadow_whose_init_was_cut_is_made_again_in_a_fresh_directory`
asserts the same count for its two starts (`:574`).
The shim kept its file in that round,
because only the shim can know which of its own starts to hang,
and `reftable_with_hanging_init` no longer returned it.
Issue 6344's first window removed that file from the shim altogether,
and the section "Issue 6344's first window" says what replaced it.

What the product guarantees there,
and what the control now holds:
`init` is started at most `SHADOW_CREATION_ATTEMPTS` (2) times for one creation,
a start ended at its ceiling or refused by the system starts again in a fresh directory,
and the partial directory of the ended start is discarded
(`SPEC.md`,
"at most `SHADOW_CREATION_ATTEMPTS` (2) times").
The count is the wrapper's own,
so a start the ceiling ends before its program writes cannot shorten it.

Red and green,
all on this host with the same test binary:

- red before the fix,
  with the planted delay:
  `left: "1"`,
  `right: "2"`,
  at `:546`,
  which is the transcript in the symptom;
- green after the fix,
  without the plant:
  `2 tests run: 2 passed`;
- green after the fix,
  with the same planted delay:
  the every-time control passes
  (`2 tests run: 1 passed, 1 failed`,
  the one failure the sibling's,
  named in "Issue 6344's second window");
- the new assertion is not vacuous:
  with the retry's condition made `false`
  (`if cut && attempt < SHADOW_CREATION_ATTEMPTS`),
  the fixed control fails on `init_attempts()` alone,
  `left: 1`,
  `right: 2`,
  at `:558`.

The branch's own gate then ran the same binaries,
in run `native-QTl6Sw` of image
`localhost/git-policy-native-test:gpc-6343-shadow-attempts-20261122T151023-0500`,
with the gate's four test threads and their 1024 processes,
and both controls passed there
(the sibling at 1.058 s,
this one at 1.197 s),
beside the two cause-H failures and nothing else:

```text
Summary [ 989.694s] 9877 tests run: 9875 passed (8 slow), 2 failed, 36 skipped
FAIL [   0.792s] (9443/9877) monochromatic-git-policy-cli::native_binary repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone
FAIL [   0.348s] (9561/9877) monochromatic-git-policy-cli::native_binary service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing
```

Its Clippy step finished clean
(`Finished `dev` profile [unoptimized] target(s) in 3m 31s`),
and the unit ended `Failed with result 'exit-code'`,
which is the cause-H pair's status and not this control's.

### Issue 6344's first window: the stand-in decided from a file the cut start writes

The stand-in the two controls install as the real Git had to know which of its own `init` starts to hang,
and the only thing it could look at was a file it wrote itself:
`n=$(cat '<count>' ...)`,
`n=$((n + 1))`
and `echo $n > '<count>'`,
with the start hanging when `$n` was at most the control's number to hang
(`reftable_with_hanging_init`,
`shadow_create_tests.rs:528`).
A first start that the ceiling ended before that write leaves no file at all,
so the second start reads the file's absence as the first attempt,
writes `1` and hangs too.
The sibling control
`a_shadow_whose_init_was_cut_is_made_again_in_a_fresh_directory`
then fails at `made` (`shadow_create_tests.rs:573`,
`the second start made the shadow`)
with the cut's own sentence,
although the product had discarded the partial directory and started again exactly as it should.

That is a control depending on how far a program it started had got inside a shortened window,
which no control may do:
a control may depend on what the product guarantees,
never on where a program stands when the ceiling ends it.

The fix hands the stand-in the attempt number from the wrapper,
at the head of the retry loop,
before the start:
`note_init_attempt(&mut init, attempt)` (`shadow_create_init.rs:84`)
counts the attempt,
and in test builds it also sets `MONOCHROMATIC_TEST_SHADOW_INIT_ATTEMPT`
to the loop's `attempt` in the request's environment
(`shadow_init_attempts.rs:157`).
The stand-in hangs the starts whose number it is told (`shadow_create_tests.rs:538`),
and no file a start writes decides anything.
The variable exists in test builds alone:
the constant is `#[cfg(test)]` (`shadow_init_attempts.rs:64`),
the environment entry is added under `#[cfg(test)]`,
and a shipped build neither sets nor reads it,
so nothing a shipped build does changed.

Red,
on this tree before the fix,
with the plant issue 6343's integration pass used
(a start that finds the count file already there hangs instead of writing its own number):
`FAIL ... a_shadow_whose_init_was_cut_is_made_again_in_a_fresh_directory`,
`the second start made the shadow: GitFailure("... a bulk step ran for 0 seconds,
the longest the wrapper lets one step run, and was ended")`.
Red again with the same control's real route forced,
a one second delay before the count write on the first start,
which loses that write exactly as a ceiling does:
the same failure at the same assertion.
Green after the fix,
with a one second delay inside the first start's work:
`2 tests run: 2 passed`.

The seam is not decorative:
with the stand-in reading a variable no build sets,
both controls fail loudly instead of passing.
The sibling fails at the count assertion
(`shadow_create_tests.rs:574`),
`left: 1`,
`right: 2`.
The every-time control fails before that,
at its `expect_err`
(`every start was cut`,
naming the path the start was handed),
because no start hung and the creation succeeded,
so its count assertion is never reached.

### Issue 6344's second window: the real second start had to finish inside the shortened ceiling

The sibling control cut its first start with a 700 ms ceiling set for the whole thread
(`STEP_CEILING_FOR_THIS_THREAD`),
and its second start is real Git making the shadow.
That second start therefore had to finish inside the same shortened ceiling,
so a program slower than 700 ms failed the control at `made`
with the cut's sentence,
even when the product had done everything the control holds.
Window two is independent of window one:
with window one's fix in place,
the stand-in deciding from the variable and no file,
dropping window two's distinction alone still fails the sibling at `made`,
which is the red run this section records.

The fix plants the cut instead of setting the thread's ceiling:
`cut_the_first_init_attempt_at(THE_FIRST_START_IS_CUT_AT)`
(`shadow_create_tests.rs:570`,
the 700 ms at `:499`)
shortens the ceiling of the first attempt alone
(`shadow_init_attempts.rs:82`),
and `apply_the_planted_cut` (`:123`)
gives the thread back its class's ceiling before the attempt after it,
forgets the plant at that attempt,
and `clear_the_planted_cut` (`:100`) does the same once the creation returned,
so no other test on that thread inherits a shortened ceiling.
`a_planted_cut_is_in_force_for_its_attempt_and_given_back_when_cleared`
holds both halves of that clearing:
with the function's body emptied it fails at `Some(700ms)` against `None`,
with only the stored plant left behind it fails at the first attempt made after the clear
(`shadow_create_tests.rs:710`),
and the two shadow controls stay green either way,
and nextest runs each test in a process of its own,
so no gate run could have seen an inherited ceiling.
The start made again is therefore waited for as production waits for a start of `init`,
`PLUMBING_STEP_CEILING` or `OBJECT_STEP_CEILING` (minutes),
and the control holds only what it means to hold:
the cut start's partial directory is discarded,
the start made again really makes the shadow,
and the shadow answers `HEAD` as the real repository does.
The plant is `#[cfg(test)]`,
so nothing a shipped build does changed.

Red,
with a one second delay on the second start before real Git runs,
before the fix:
`FAIL ... a_shadow_whose_init_was_cut_is_made_again_in_a_fresh_directory`,
`the second start made the shadow: GitFailure("... a bulk step ran for 0 seconds,
the longest the wrapper lets one step run, and was ended")`.
The delay is now part of the control's stand-in,
`THE_START_MADE_AGAIN_TAKES` (`shadow_create_tests.rs:490`),
so the suite itself holds the fix of window two:
with the first-attempt-only distinction dropped from `apply_the_planted_cut`,
the sibling fails at `made` (1.597 s) and the every-time control passes.
A build-time check beside the two numbers holds the delay longer than the ceiling it has to outlast
(`const _: () = assert!(...)`),
so a later change of either number stops the build rather than the guard.
It costs the sibling control its second on every run.
Measured on this host in three interleaved rounds,
each round the committed build,
the same tree with the second set to zero,
and the committed build again:
the sibling read 1.847 s to 2.245 s with the second
against 0.876 s to 1.167 s without it,
a difference of 0.71 s to 1.15 s (mean 0.94 s),
while the two committed builds of one round differed by up to 0.38 s
(issue 6344's integration pass,
`keep/fu-delay-arms-interleaved-three-rounds.txt`).
The every-time control,
whose starts hang before the line that waits,
read the same in both arms.
Green with the cut planted for the first attempt alone:
`2 tests run: 2 passed`.

### The witness the count of attempts does not give, and the lower bound that does

The count issue 6343 added counts loop iterations,
not started programs:
a `run_git` that returned a cut or a refusal without starting anything
would leave the count at `SHADOW_CREATION_ATTEMPTS`
and both controls green.
The old count file did witness that each start reached a program,
because only a program could write it.

`a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing`
now also asks how long it waited
(`shadow_create_tests.rs:617`).
Every start of that control hangs,
so each attempt is waited for its 500 ms ceiling and no shorter,
and an attempt that returned sooner ended before its ceiling.
The bound is derived from the control's own named ceiling and from `SHADOW_CREATION_ATTEMPTS`.
Contention only lengthens a wait,
so the bound cannot flake on a busy host.
What it rejects is an attempt that returned before its ceiling,
which is a refusal returned as a finished command,
where the program ran and printed the system's words,
or a cut returned at once without a program behind it,
which is the defect the bound was added to catch.
Measured:
a first start that prints the refusal and exits fails the bound at 548.950692ms
(issue 6344's integration pass,
`keep/merge-run-lower-bound-first-start-refused-as-finished-command.txt`).
What the bound cannot tell apart from a cut:
a spawn refused through the whole `START_RETRY_BUDGET` pauses 4630 ms,
so such an attempt passes the bound without a program having started
(9.52 s for two attempts and 15 refusals over 4630 ms,
`keep/merge-run-both-spawns-refused-by-process-limit.txt`).
It is worth that window because the false green it closes needs no plant:
on `main`,
a `run_git` that returned a cut at once left both controls green.

Red with a plant that makes the second attempt return a cut
without starting a program or waiting for one:
`an attempt ended before its ceiling,
which a cut of this control's plant cannot do: 532.018781ms`
(`shadow_create_tests.rs:616`).
Green with the plant removed:
`2 tests run: 2 passed`.

### This round's gate, on the fix

The whole container gate of the fix ran as its own capped unit,
`ssc-6344-shadow-gate-20261122`,
started 2026-11-22T16:51:52-05:00 with `RuntimeMaxSec=3600`,
on image `localhost/git-policy-native-test:ssc-6344-20261122T165142-0500`
(run `native-2KpJix`),
with the gate's four test threads and their 1024 processes:

```text
Summary [ 917.011s] 9879 tests run: 9876 passed (7 slow), 3 failed, 36 skipped
```

Both controls passed there:
the sibling `a_shadow_whose_init_was_cut_is_made_again_in_a_fresh_directory`
at 0.965 s (test 6050 of 9879),
and `a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing`
at 1.153 s (6054 of 9879),
so neither window failed and the lower bound held under the gate's contention.
Its Clippy step finished clean
(`Finished `dev` profile [unoptimized] target(s) in 3m 21s`).

Two of the three failures are the pair "Cause H" names.
The third is
`worktree_removal_held_commit::a_merge_that_lands_a_branch_keeps_the_held_commit_of_the_worktree_it_removes`,
the control cause G records:
it failed with `the sweep removed the worktree whose branch landed`,
and the product's own words there say the removal went to the commit service
and that the branch `refs/heads/feature` is now on `main`.
Run alone on this host afterwards it passed (5.469 s).
It is the same control cause G records,
failing in a gate and passing alone,
but cause G records it with other words,
where the removal's record "was moved by another look of the same service while the worktree stayed";
in this failure's own words the worktree was removed.
No failure of cause I appeared,
nor of cause J or K,
and nothing of issue 6342 did either:
nothing else failed,
so there was nothing to run alone.
The total is two more than the 9877 of issue 6343's gates
because `473dccebbc` (the subvolume record's branch) landed two controls
between those runs and this one,
not because this branch adds a test.

### The follow-ups' gate

The whole container gate of the follow-up round ran the same way,
as its own capped unit `ssc2-6344-followups-gate-20261123`
with `RuntimeMaxSec=3600`,
on image `localhost/git-policy-native-test:ssc2-6344-followups-20261123T050551-0500`
(run `native-Y34ZI0`):

```text
Summary [1046.619s] 9914 tests run: 9912 passed (11 slow), 2 failed, 36 skipped
FAIL [   1.339s] (9480/9914) monochromatic-git-policy-cli::native_binary repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone
FAIL [   0.359s] (9598/9914) monochromatic-git-policy-cli::native_binary service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing
```

Exactly the cause-H pair,
and nothing else failed.
All three shadow controls passed there:
the sibling at 2.004 s (test 6131 of 9914),
which carries the committed second,
`a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing`
at 1.152 s (6084 of 9914),
and `a_planted_cut_is_in_force_for_its_attempt_and_given_back_when_cleared`
at 0.013 s (6037 of 9914).
Its Clippy step finished clean
(`Finished `dev` profile [unoptimized] target(s) in 3m 46s`),
and the unit ended `Failed with result 'exit-code'`,
the cause-H pair's status.
The total is 9914 because `main` landed new controls between the merge this round started from and the gate,
and because this round adds one.

### What the controls ask of the host now

- The sibling control asks for one cut:
  a start that shows nothing for 700 ms is ended,
  and the attempt after it is waited for the ceiling production gives `init`.
  A program slower than that ceiling fails the transaction as production would fail it,
  not as a shortened control ceiling would.
- The every-time control asks for two cuts at its own 500 ms ceiling,
  and for the two attempts to have been waited for it.
- Neither control reads a file a start writes,
  and neither depends on a program finishing inside a window shorter than the one production allows it.

### If this name fails again

Read which assertion failed and what it printed.

- `left` and `right` from `init_attempts()` are the wrapper's own attempts:
  a `1` against `2` is `init_with_git` no longer starting a fresh directory when the first was cut,
  which is a defect in the product,
  unless `SHADOW_CREATION_ATTEMPTS` itself moved.
  A panic that names a count file cannot happen here any more:
  no control keeps one.
- The sibling's `made` failure names the sentence of the last attempt's start,
  which is the start made again:
  a first attempt's cut makes the loop start again and is never the error the loop returns.
  A `TimedOut` sentence there is that start ended at production's ceiling,
  and the whole step's ceiling and the host's state decide that,
  not this control.
  A planted ceiling that stayed over the start made again gives the same sentence.
  What holds that distinction is the committed delay,
  `THE_START_MADE_AGAIN_TAKES`:
  with the first-attempt-only branch of `apply_the_planted_cut` dropped,
  the sibling fails at `made` and the every-time control passes
  (the section "Issue 6344's second window").
  The planted-cut control holds a different fact:
  that clearing the plant forgets it,
  so the first attempt of the creation after it is not cut either.
- The every-time control's lower-bound failure means at least one attempt returned before its ceiling.
  Two things end an attempt before its ceiling:
  a refusal returned as a finished command,
  where the program ran and printed the system's words,
  so the run did not witness the two programs the control means to count,
  and a product that returned a cut at once without starting a program,
  which is the false green the bound was added to close
  (the section "The witness the count of attempts does not give,
  and the lower bound that does").
  The sentence the failure carries says which:
  the system's own refusal words are the host's,
  so run it again on a host with processes to spare before reading it as a product defect;
  the cut's sentence,
  with no program's words behind it,
  is the product's.

## Cause N: the wait a landed removal's control relies on can return while its record is being moved (issue 6345)

This is neither cause G nor cause H.
Cause G holds the first gate this control failed in and points here.
Cause H is the standing scratch-directory pair.

### The symptom the gate reader saw

`worktree_removal_held_commit::a_merge_that_lands_a_branch_keeps_the_held_commit_of_the_worktree_it_removes` failed in two whole container gates and passes when it runs alone.
The gates were the one of 2026-11-21T13:25:00-05:00,
which cause G holds,
and the one of 2026-11-22T16:51:52-05:00.
The second ran on the branch `fix/git-policy-cli-shadow-sibling-control-two-timing-windows`,
which does not touch worktree removal,
and it failed this control as test 9834 of 9879 after 0.880 s.
Its own words,
from the gate journal of that run as issue 6345 quotes it,
with the fixture paths elided at `…`:

```text
thread 'worktree_removal_held_commit::a_merge_that_lands_a_branch_keeps_the_held_commit_of_the_worktree_it_removes' (352637) panicked at src/native/binary/binary_worktree_removal_held_commit_tests.rs:359:5:
the sweep removed the worktree whose branch landed: monochromatic-git-policy-cli: the commit 297feb89628925c7a416ccf50e3471a4a174426d for feature is held in /work/…/linked behind the lock `/work/…/repo/.git/refs/heads/feature.lock`, … it is kept at `refs/cli-git/kept/7`, feature stays where it is, and no command moves it by itself.
monochromatic-git-policy-cli: handed the removal of the worktree /work/…/linked to the commit service, though its branch refs/heads/feature is now on main: … no merge waits for that. … What became of it is the record /work/…/repo/.git/monochromatic-git-policy-cli/work carries the record subvolume-delete-da570535966155ca3a7a531dc5d37c522cf6e734-1b541ba361ad4bed: `done` under it means the worktree is gone, and `failed` means it was kept, with the reason on your next command's standard error. …
monochromatic-git-policy-cli-service: ready at_ms=1795385529221
monochromatic-git-policy-cli-service: ready at_ms=1795385529230
monochromatic-git-policy-cli-service: the subvolume /work/…/linked could not be deleted: nothing was removed: /work/…/repo/.git/worktrees cannot be resolved (No such file or directory (os error 2)), so this repository has no linked worktree at_ms=1795385529725
monochromatic-git-policy-cli-service: moving subvolume-delete-da570535966155ca3a7a531dc5d37c522cf6e734-1b541ba361ad4bed to failed failed: No such file or directory (os error 2); its lease probably ran out and another instance took the work over at_ms=1795385529726
monochromatic-git-policy-cli-service: exiting reason=signal-15 uptime_ms=508 at_ms=1795385529727
monochromatic-git-policy-cli-service: exiting reason=signal-15 uptime_ms=504 at_ms=1795385529732
```

Two service instances ran for that fixture,
one for each registered worktree Git directory,
and the two exits came after the two error lines.

### What the assertion needed, and each way it could be false

The assertion was `assert!(!scene.linked.exists(), "the sweep removed the worktree whose branch landed")`.
It needed the linked worktree's directory to be gone by the moment the `merge` command had returned.

- The removal had not run yet when the assertion read the worktree.
  The merge only hands it over,
  and three places in the product say so:
  `SPEC.md` ("and then it records the removal and returns"),
  `HandedOverWorktree::worktree` in `src/native/worktree/worktree_landed_removal.rs` ("The worktree directory,
  which is still there when the merge ends"),
  and the sweep's own line in `src/native/worktree/worktree_landed_sweep_notice.rs` ("no merge waits for that",
  "`done` under it means the worktree is gone").
  The test support hides that in every run by waiting for the record to leave `pending`,
  `recovering` and `claimed`.
  That wait reads the three folders one after another,
  and a record being moved between them during a sample is in none of that sample's three readings.
  The wait then returns with the removal about to start and the assertion fails on a worktree that is still there.
  That the two gates hit this window is not shown:
  it is the way the old control fails with the gate's own sentence under plants that only widen windows the code already has.
- The service examined the record and kept the worktree.
  Git refused it,
  an ignored file there is one the merge's worktree does not hold,
  a process works inside it,
  it is locked,
  a held commit reached no reference,
  or the identity the hand-over wrote names another worktree.
  The worktree then stays for good,
  and the next command that reads notices names it with the reason.
- The service was not woken.
  The record stays in `pending` until a commit wakes the service (see "Which commands wake the service"),
  and the worktree stands until then.
- The service could not start Git.
  The removal answers a retry (`worktree_remove_hand_over.rs`,
  the removal that found no Git),
  the record stays in `claimed` until its 120 s lease ends,
  and the support's wait ends first,
  at its own 60 s bound,
  with "the service never settled the after-commit work".
- The sweep kept the candidate instead of handing it over.
  That is a worktree nothing records as the wrapper's,
  one nested inside it,
  an ignored file,
  or a lock,
  and then there is no hand-over line at all.
- A removal took the registration and left the files.
  A person's `git worktree remove --force` that cannot empty a directory does that:
  Git 2.56.1 deletes the registration and an empty `worktrees` folder even when it could not delete the files (`builtin/worktree.c`,
  `remove_worktree`,
  "continue on even if ret is non-zero").
  The service's look then answers `failed` ("nothing was removed:
  …") and writes no kept-worktree notice,
  so no command says anything;
  that limit is tracked as issue 6355.
- The fixture's teardown removed the tree under a look that was still running.
  That is the journal's `.git/worktrees cannot be resolved` line,
  and it needs the registration tree to go between the look's second and third path resolution.
  The section "The fixture's drop,
  and the journal's two service lines" shows it,
  and it is not what made the assertion fail:
  the assertion ran first.

### The cause

Two things,
and the first is what the controls show.
The merge returns before the removal finishes,
by design,
in the three places named under "What the assertion needed,
and each way it could be false".
The wait that hides that reads three folders one after another,
while a record being claimed moves from `pending` into `recovering` and then into `claimed`,
a recovery of an expired claim moves a `claimed` record into `recovering` and then into `pending`,
and a settle moves a claim into `done` or `failed`.
A record moved between two of a sample's reads is missed by that sample.
The wait now reads the folders in the order a record moves through them,
and confirms an empty reading with a second and a third sample (`settle_service_work` in `src/native/binary/binary_support_observation.rs`).
The order closes forward moves,
because a forward move lands in a folder the sample has not read yet;
the second sample closes the moves that go back;
the third closes a recovery whose two renames straddle two samples.
The comment on `settle_service_work` lists every move a record makes between these folders,
from the code.
What three samples do not close,
over schedules of one record's moves placed among the wait's reads,
counted by a reviewer's run kept outside the repository,
and reproduced by the pass after it:
they closed all 3,350 schedules of up to three moves and all but 4 of 18,365 of up to four,
so a chain of four or more moves that a thread's stall stretches across every sample is still missed.
A record renamed inside one folder while that folder is read was not tested.

Shown:

- four controls of the wait (`src/native/binary/binary_support_observation_tests.rs`),
  each red with its own guard removed and green with it:
  `a_read_in_the_order_a_record_moves_catches_what_a_claim_and_a_recovery_move_between_its_reads` is red when the folders are read in the old order with three samples kept;
  `a_second_sample_catches_what_a_recovery_takes_back_between_two_reads` is red with one sample;
  `a_third_sample_catches_a_recovery_whose_two_renames_straddle_two_samples` is red with two samples;
  `a_record_that_moves_between_two_reads_is_not_a_settled_queue` is red on the wait `main` has,
  and green with the order alone and with the second sample alone;
- a reviewer's four plants (the claim's stay in `recovering` 300 ms,
  the first sample 100 ms late,
  400 ms after the second read of a sample,
  a 600 ms removal) made the control as it stood fail with the gate's own sentence on `main`'s wait and pass on this one,
  and the pass that reviewed this round saw it again;
- the wait as it stood before its third sample (two samples),
  with real renames made by the wait's own step,
  returns with a live record on the third control's schedule;
  the wait now in the tree does not return on any of the four controls' schedules.

Inferred,
and not shown:

- that either gate hit this window.
  Neither gate logged the queue when the wait returned.
  The move that makes the window needs a stall of the waiting thread;
  the pass measured the recovery of an expired claim at 227 to 1131 microseconds (median 249,
  300 runs) against about 55 microseconds for a whole call of the wait with three samples,
  so a capped four-thread container can hold a thread that long,
  and no rate is claimed here.
  The next occurrence must capture the queue at the moment the wait returns.

### The reproduction

Each control's window is one record moved between the wait's reads by the seam `run_before_each_folder_read`,
and each is red with its guard removed:

```text
PASS support_observation::tests::a_read_in_the_order_a_record_moves_catches_what_a_claim_and_a_recovery_move_between_its_reads
FAIL support_observation::tests::a_read_in_the_order_a_record_moves_catches_what_a_claim_and_a_recovery_move_between_its_reads
     with A_RECORD_IN_FLIGHT_FOLDERS read as pending, claimed, recovering and three samples kept
PASS support_observation::tests::a_second_sample_catches_what_a_recovery_takes_back_between_two_reads
FAIL support_observation::tests::a_second_sample_catches_what_a_recovery_takes_back_between_two_reads
     with one sample
PASS support_observation::tests::a_third_sample_catches_a_recovery_whose_two_renames_straddle_two_samples
FAIL support_observation::tests::a_third_sample_catches_a_recovery_whose_two_renames_straddle_two_samples
     with two samples
PASS support_observation::tests::a_record_that_moves_between_two_reads_is_not_a_settled_queue
FAIL support_observation::tests::a_record_that_moves_between_two_reads_is_not_a_settled_queue
     with main's order and one sample
```

Each red prints the queue at the wait's return,
with the record still in it,
for example `a read of the wait sees the record while it moves between the folders a claim passes through, and none did: pending=[] claimed=["subvolume-delete-0000000000000000000000000000000000000000"] recovering=[] done=[] failed=[]`.

### The fix

`settle_service_work` reads the three folders in the order a record moves through them and requires three empty samples.
A whole call on an empty queue costs about 55 microseconds against `main`'s 19,
a ratio of 2.9 (range 2.6 to 3.5 over two sets of 36 interleaved rounds,
where identical code ranged 0.91 to 1.14),
measured by a reviewer's run kept outside the repository.
The four controls hold the order and each sample,
and the pair of halves.
The wait keeps its 60 s bound,
its 20 ms pause and its failure message.
No test-level timing was made:
the whole gate that would pair the wait's users has no usable result for this branch.

### The fixture's drop, and the journal's two service lines

`Fixture::drop` (`src/native/binary/binary_support_fixture_service.rs`) bound the service it took to `_stopped` on `main`.
A binding that begins with an underscore keeps its value to the end of the scope,
so the services were stopped after `settle_fixture` removed the tree,
and the comment on that line of `Fixture::drop` said the opposite ("The service goes first").
A look of a service still running when its test ended therefore read a repository being removed,
which is how the journal's two error lines arose.
The drop stops the services at once now,
before `end_processes_working_in` and `settle_fixture`.

Shown with a look held between its second and third path resolution,
and a test that ends while the look waits (a reviewer's runs of it,
kept outside the repository):

- the old order printed the look's line in 13 of 28 runs,
  two of those with the second line too,
  at pauses from 4 to 20 ms and with a pause that ends when the repository is gone,
  and not at all at 400 ms;
- the new order printed it in none of 28 runs,
  and the only service lines were the two `exiting reason=signal-15` lines,
  both before the tree was touched.

### What a person meets

- The service is slow:
  the merge exits 0 with its own line,
  the worktree stands at its return,
  and the removal finishes later.
- The service was not woken:
  the merge prints "the removal of … is recorded and not woken (…)" beside its own line,
  then says that the next `git commit` there finishes the removal and that `git worktree remove …` of the same path
  removes it now,
  the record stays `pending`,
  and the worktree stands until a commit.
  The clause and the correction of the documents' "every commit and every add" are the build of issue 6354
  (`doc/decision/cli-git-worktree-removal-not-woken-warning-names-the-way-out.md`,
  2026-11-23).
- The service hangs inside the removal:
  the record stays `claimed`,
  the worktree stays,
  and no later command says anything about it;
  the merge's line is the only word,
  and it gives the command that removes the worktree.
  This was run once,
  by the first pass that reviewed this control,
  and not since.
- The service keeps the worktree because it is locked:
  the record reaches `failed`,
  and the next command that reads notices prints the kept-worktree line once,
  with the product's own sentence about the lock.
  A reviewer ran this.
- The service keeps the worktree because Git refused the removal:
  the record reaches `failed`,
  and the notice carries Git's own sentence.
  This was not run since the first pass that reviewed this control;
  it stands on the code (`keep_the_worktree(…, "Git refused to remove it: …")`).
- The merge's line names the hand-over,
  where the record is,
  what `done` and `failed` mean,
  and the exact `git worktree remove <path>`.

#### Which commands wake the service

Measured against a removal record parked in `pending`:
thirteen commands,
eight seconds each,
and only `git commit --allow-empty` woke the service.
Two runs put the record's leaving `pending` at 312 ms and at 1652 ms after the command started,
so the wait it took is what one run of a loaded host gives and no bound is claimed.
The other twelve left it `pending`:
`git status`,
`git status --short`,
`git log`,
`git branch`,
`git rev-parse`,
`git worktree list`,
`git diff`,
`git stash list`,
`git add`,
`git worktree add` (which made no trash),
`git worktree prune` and `git worktree remove` (of another worktree).

From the code:

- a commit is a request to the service,
  whose round drains that repository's queue;
- every wrapped command whose output no program reads runs `show_push_failures` before it runs (`src/native/invocation/wrapped_command.rs`),
  and that step wakes the service when it queues a subvolume leaving or queues a refused removal record again.
  The pass measured the second:
  human-read `git status`,
  `git status --short` and `git log --oneline` did it,
  and `git status --porcelain` and `git rev-parse` did not;
- an add wakes the service through its trash hand-over (`worktree_trash::hand_over_to_the_service`) only when the add made a trash;
  that was not run,
  and this document claims it from the code only;
- a repair that relinks a package,
  an add's snapshot ahead,
  and the next merge that hands over a removal were not run either;
  they stand on the code.

A removal left `pending` by a missed wake was issue 6354;
what wakes the service is unchanged,
and the warning a failed wake prints now names the next `git commit` and `git worktree remove <path>`
(`doc/decision/cli-git-worktree-removal-not-woken-warning-names-the-way-out.md`,
2026-11-23).

### A limit the fix does not close

`remove_the_worktree_of` answers `Failed("nothing was removed: …")` when the registration cannot be read,
and that branch returns before `keep_the_worktree`,
which is the only writer of a kept-worktree notice.
The merge's line promises that a `failed` record's reason reaches the next command;
for this state no notice is written,
so no command shows one.
The state is rare (the worktree is already half gone when it is met),
and it is tracked as issue 6355.

### What the next occurrence must capture

The queue listing and whether the worktree and its registration exist at the moment `settle_service_work` returns,
and the record's state at every look of the service,
with the process of that look.
The wait's own step can print that listing (`run_before_each_folder_read`),
and it is the line that decides whether the wait returned early.

## Cause O: a carry control's save landed in the millisecond Git's own write took (issue 6461)

### The symptom this host's reader saw

`merge::merge_isolation_carry_apply::tests::a_save_whose_bytes_accept_the_whole_merge_is_not_replaced_by_the_captured_layer`,
run alone in its own process at `49029c6828`,
failed in 0.23 s:

```text
panicked at src/native/merge/merge_isolation_carry_apply_tests.rs:510:5:
assertion `left == right` failed: the carry says the path was written after the capture and leaves it
  left: []
 right: [[97, 46, 116, 120, 116]]
```

of the round `lock-kill-controls-that-fail-alone-on-this-host-2026-11-25` (`keep/red-merge-carry-save-alone.log`).
The same
commit passes it in the container gate.

### What the assertion needed

`landed_of` reads the entry standing at the path once Git's own update has finished,
and the control then writes the
person's save with one `std::fs::write` of the bytes it just read.
The carry tells the merge's own file from that save by
the entry alone (`the_file_git_landed` over `LandedFile`):
the device,
the inode,
the size,
the mode,
the modification time
and the change time.
A save of the same bytes changes none of the first four,
so the assertion needs the last two to move.

### The cause

This host's file-system clock moves in steps of a millisecond,
so two writes inside one step carry the same modification
and change times,
and the save is then indistinguishable from the write Git made.
The carry rightly reads such a file as
Git's own (`merge_isolation_carry_landed.rs`,
"What Git's own two-way update left at each path"),
and the control's
expectation that the newer save stands is what fails.

That is the same family as Cause L above:
a control that needed two writes to be ordered in time,
on a clock whose step is
far longer than the work between them.

### The reproduction and what it measured

Eight runs of the one control,
instrumented to print both entries (`keep/merge-carry-same-tick-repetitions.log` and
`keep/merge-carry-identity-measurement.log`):

- six runs printed the same `modified_nanos` for the landed entry and the entry after the save (for example
  `855806081` and `855806081`,
  `247802396` and `247802396`),
  and every one of the six failed with `changed: []`;
- two runs printed a step between them (`247802396` and `248802393`,
  `209799849` and `210799846`),
  and both passed.

So the failure follows the entry and not the clock reading:
the same control passes whenever the step moved between Git's
write and the save.

### The fix

`save_as_a_later_write` writes the save and reads the entry back,
and writes again until the entry is provably not the
landed one,
one clock step apart,
bounded by `CLOCK_MOVES_WITHIN`.
A file system whose clock never moves fails the control
with words that name the clock instead of leaving a save untested.
Eight runs of eight passed after the change
(`keep/green-merge-carry-save-alone-repetitions.log`),
and the whole `merge::merge_isolation_carry_apply` module passes
(`keep/merge-carry-module-after-the-fix.log`).

### What the fix does not change

The production rule stays as it was:
where nothing proves the file standing at a path is Git's own,
the carry keeps the
person's bytes and leaves the file alone,
and the sibling control `a_save_that_lands_before_the_reading_keeps_both_sides`
pins that.
This cause is a control's assumption about time,
not a defect of the carry.

The ten-minute wait this control's report once named is not this control's:
`OBJECT_STEP_CEILING` ended the watched stop-lock
control on the same host,
whose cause is recorded in
`doc/troubleshooting/git-policy-cli-cut-control-holds-a-tracer-the-stop-kills.md`,
"A machine without the tracer".
