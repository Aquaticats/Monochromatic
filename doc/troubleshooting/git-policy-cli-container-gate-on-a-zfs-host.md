# The git-policy-cli gate's containers on a ZFS host: five mount behaviours fail 185 native tests, and Clippy's step fails with E0514

Investigated on 2026-11-19T17:00:41-05:00 in
`.claude/worktrees/pi-git-policy-cli-container-gate-on-the-zfs-host` (branch
`docs/git-policy-cli-container-gate-on-the-zfs-host`, from `main` `cff64adfa0`), against the release
`0.6.1-alpha.44` snapshot in the gate's own image, under podman 6.2.0 on CachyOS, kernel 7.3.5.

The gate is `mise run //package/git-policy/cli:native:test:container`
(`bin/test-native-container.ts`), whose substantive step is the whole suite and Clippy in two
containers built from one source image.

## Symptom

The gate's suite fails on this host, and its Clippy step does not compile:

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

On the previous host, a Btrfs machine, the same gate failed six of 9,173 tests
(`doc/handover/cli-git-rust-implementation.md`, the gate on `0f07c4f0ee`). This host fails 185.

The mise task cannot be run as written here either: its first step
`cargo-shadow lean records target/verification` fails, because `cargo-shadow` and `cargo-nextest` are
not installed on this host (`mise ls` prints nothing). Every run below is the task's second step,
`node bin/test-native-container.ts`, or the image it builds, run the way the gate runs it.

## The host, as measured

- `/home` is `zpcachyos/ROOT/cos/home`:
  `utf8only=on`, `normalization=formD`, `casesensitivity=sensitive`, `acltype=posix`, `longname=off`
  (`zfs get -H -o property,value ...`). ZFS 2.4.4.
- `podman info --format '{{.Store.GraphDriverName}} {{.Store.GraphRoot}}'` answers
  `overlay /home/user/.local/share/containers/storage`, both on that dataset. That store's own
  configuration also sets `overlay.mountopt=nodev`.
- Inside a container the repository tree is a **user-space** overlay: `stat -f --format=%T` prints
  `fuse`, `/proc/self/mountinfo` names the type `fuse.fuse-overlayfs`, and `statfs` gives
  `f_type = 0x65735546` (`FUSE_SUPER_MAGIC`). On the Btrfs host the same tree was a kernel `overlay`.
- The gate image (`localhost/git-policy-native-test:development`, built 2026-11-19T19:30Z from
  `localhost/git-policy-native-test-base:cfc6cb4ce2c8-97221c29742b`) carries rustc 1.100.0
  (Arch 1:1.100.0-1), cargo 1.100.0, git 2.56.1, cargo-nextest 0.9.151.
- The host's `rustc --print sysroot` answers
  `/home/user/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu` (rustc 1.102.0-nightly, 5df8c02af),
  whose `bin/` holds `cargo`, `rustc`, `rustdoc` and the gdb wrappers and **no `cargo-clippy` and no
  `clippy-driver`**. `/usr/bin/rustc` is a symlink to `/usr/bin/rustup`.
- The repository's locked toolchain is `nightly-2026-11-16` (`toolchain.lock.json`, "rust"), installed
  here with `cargo-clippy` and `clippy-driver` (rustc 1.102.0-nightly, 5f4effd57).

Four mount facts follow from those two settings, and every cause below is one of them or a
consequence:

- a name that is not UTF-8 is refused by the dataset (`utf8only=on` with `normalization=formD`);
- the fuse mount records no birth time (`stat --format=%w` prints `-`);
- the fuse mount's type is in neither of the product's two "local file systems" tables;
- Git reads a FUSE mount as a remote working tree.

## Cause A: a name that is not UTF-8 is refused (108 tests)

Reproduction, on the host with no container: `mkdir` of a name whose last byte is `0xFF` under the
investigation scratch folder (on `zpcachyos/ROOT/cos/home`) fails
`errno=84 Invalid or incomplete multibyte or wide character`; the same call under `/dev/shm`
succeeds. Inside the gate's image the same pair repeats against `/work/package/git-policy/cli/target/tmp`
(fails, `errno` 84) and `/dev/shm` or a `--tmpfs` mount (both succeed), so the refusing layer is the
dataset, reached through podman's storage, and not FUSE.

The product and its fixtures create names Git itself accepts, such as `not-utf8-\xFF` and
`caf\xe9.md`, and those creations fail. Two shapes of failure appear: the test's own create fails
(`worktree file: Os { code: 84, ... }`), or the fixture's Git command fails
(`git ["branch", "lost-\xFF"] failed: fatal: cannot lock ref ... Unable to create ... Invalid or
incomplete multibyte or wide character`).

The 108 tests this cause covers include every one whose name says so, for example
`candidate::candidate_store::tree_tests::a_pushed_tree_lists_a_name_that_is_not_utf8_exactly`,
`commit::capture_invocation::tests::a_repository_whose_name_is_not_utf8_is_captured`,
`policy::policy_catalog_specifiers::tests::path_cases::a_manifest_below_a_name_that_is_not_utf8_is_still_checked`,
`shadow::shadow_create::tests::a_ref_name_that_is_not_text_does_not_stop_the_shadow`,
`worktree::worktree_trash::safety::a_non_utf8_worktree_is_named_by_a_record_and_emptied_by_its_path_bytes`
and the eleven `event_path_commit` tests. The full list is `cause-A-non-utf8-name.txt`, in the
investigation folder named at the end of this doc.

Positive control: all 108 pass once the fixture root is a tmpfs (the workaround below).

## Cause A2: one test writes through a scratch helper the fixture root does not move (1 test)

`landing::service_claim_watch::tests::a_path_is_kept_byte_for_byte` still fails with `errno` 84 under
the tmpfs workaround, because its own helper bypasses the knob:

```rust
// package/git-policy/cli/src/native/landing/service_claim_watch_tests.rs:77
fn scratch(name: &str) -> PathBuf {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("claim-watch-tests").join(format!("{name}-{}", std::process::id()));
```

Every other test file builds its root from `fixture_root()`
(`src/native/testing/test_support.rs`, `FIXTURE_ROOT_VARIABLE`), or from the binary face's
`fixture_root()` (`src/native/binary/binary_support_fixture_directories.rs`), and both read
`CLI_GIT_TEST_ROOT`. This one writes under the crate's `target/`, which stays on the dataset. It is a
defect of ours, of one line, and no host change removes it.

## Cause B: the mount records no birth time (29 tests)

Inside the image, `stat --format=%w` of a file under `/work/package/git-policy/cli/target/tmp`
prints `-`, and the same call on `/dev/shm` or a `--tmpfs` mount prints a time.

The product proves a left-behind lock is gone by comparing birth times
(`src/native/lock/lock_birth_times.rs`), and the worktree-copy stage decides whether a stage was left
by an interrupted command by the same proof. With no birth time the answer is "unproven", and the
tests assert the proven answer. Two wordings appear:

```text
this file system cannot show that <lock>~pid.lock was born after <lock>.lock: the lock is None and the PID file None
nothing shows what payload is (creation time is not available for the filesystem), so nothing below it was examined
```

The 29 tests are the eleven `lock::abandoned_lock_proving_tests`, the five
`worktree_copying::worktree_copy_stage_claim` reap controls, the eleven `killed_holder_lock` binary
controls, `lock::lock_birth_times::tests::the_hidden_and_coarse_arms_report_what_other_file_systems_do`
and `worktree_copy_concurrent::an_add_says_nothing_about_a_lock_a_live_program_holds_in_another_worktree`
(`cause-B-birth-time.txt`). Three of the five stage controls assert an outcome without printing the
reason; they belong to this group because their module's other two print it.

Positive control: all 29 pass on a tmpfs fixture root.

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

The mount's `/proc/<pid>/mountinfo` type is `fuse.fuse-overlayfs`, which is on neither list, so a lock
whose holder is a live process reads as unproven and is cleared, a live holder is not seen, and a copy
a program holds open is removed:

```text
unresolvable: ["PID 28169 fd 3: its mount is a `fuse.fuse-overlayfs` file system, whose attributes are not in the memory of this kernel, so the file is not read"]
Unproven("it sits on a `fuse.fuse-overlayfs` file system, which a Git on another machine may share")
the scan never saw the holder 50772
```

The 28 tests are the ten `lock::stale_ref_locks`, `lock::index_lock_holders`,
`lock::index_holder_wait`, the three `process` controls (`process::open_file_of_a_descriptor` and
two `process::process_use`), `merge::merge_isolation::repository_tests`, the three
`merge::merge_isolation_recovery`, `merge_never_fails::a_merge_that_meets_a_holder_proven_alive_saves_itself_at_once`,
two `landing::service_index_lock`, `landing::service_index_unheld_removal`, two
`landing::service_land_reference_lock` and the three `wait_notice` binary controls
(`cause-C-mount-type.txt`).

Reading `/proc/<pid>/fd` of a descriptor of a file on this mount works (measured: the link resolves on
the fuse mount, on `/dev/shm` and on a `--tmpfs`), so the refusal is the product's list, not the
kernel's ability.

Positive control: all 28 pass on a tmpfs fixture root, whose mount type is `tmpfs`.

## Cause D: Git refuses `core.fsmonitor` on a FUSE mount (10 tests)

Reproduction inside the image, with a repository whose `core.fsmonitor` names a program that appends
to a marker file, on the fuse mount: `git status` prints
`warning: remote repository '<repo>' is incompatible with fsmonitor` and the program does not run. On
a `--tmpfs` mount the same repository runs the program and prints no warning. On the fuse mount with
`-c fsmonitor.allowRemote=true` the program runs, so the mount and not the program is the cause.

Git's source explains the wording: `check_remote()` refuses a working directory that
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
which returns the bytes of that commit; the numbers above are that read. A rendered-page check was
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
person's monitor and the wrapper's own Git does not, so a monitor that never runs in either arm makes
both halves fail; the two bound controls use a monitor program that never answers to prove a bound,
and Git answers at once because it never starts that program.

Positive control: all ten pass on a tmpfs fixture root.

## Cause E: the copy flow makes no `ioctl` where the file system is neither Btrfs nor overlayfs (2 tests)

```text
copy lists calls its flow never makes, so faulting them would test nothing: ["ioctl"] (measured [("ioctl", 0)])
```

`COPY_KILL_CALLS` (`src/native/binary/binary_fault_trace_tests.rs:1041`) lists `ioctl` because on
Btrfs and on a kernel overlay it carries the `FICLONE` that fills each staged file. The efficient
worktree-copy method is chosen by file system: Btrfs and overlayfs clone, every other file system
lets Git check the worktree out (`src/native/health/doctor_filesystem.rs:255`, and the ladder the
message names). A `fuse.fuse-overlayfs` mount is neither, so the flow never calls `ioctl`, and the
control that faults every call in that list fails. The second test,
`fault_trace::a_worktree_copy_killed_at_each_ioctl_leaves_a_recoverable_repository`, is the same
failure for the `ioctl` entry alone.

A tmpfs fixture root does not fix these two: tmpfs is not on the cloning list either (measured, both
arms still fail). The two ways out are a fixture root on a Btrfs or overlay mount, or a change to what
the kill list demands on such a file system. The two tests are `cause-E-reflink.txt`.

## Cause F: three single behaviours of the mount

Each of these fails alone, for its own reason, and each passes on a tmpfs fixture root.

- `index::index_reset_file::tests::the_file_mode_under_a_default_acl_equals_git`: a directory with a
  default ACL of `u::rwx,g::---,o::---` gives a file mode `0644` on this mount and `0600` on tmpfs
  (measured with `setfacl --default --modify ...` and one create on each). The dataset has
  `acltype=posix`, so the ACL is accepted and then not applied to the new file, and the test sees
  Git's index at `0664` where it expects `0660`.
- `worktree::worktree_completion::tests::an_error_that_is_not_a_conflict_fails_the_inspection`: the
  test builds a 300-byte name and expects an error that is not a conflict. `lstat` of that name
  answers `ENOENT` on this mount and `ENAMETOOLONG` on tmpfs (measured from a Rust probe running the
  two calls the product makes), and `is_conflict_error`
  (`src/native/worktree/worktree_completion_file_inspection.rs:99`) reads `NotFound` as a conflict.
- `transaction::transaction_git::tests::input_files_leave_nothing_behind`: the input file is created,
  opened and unlinked, and the product then lists the scratch directory and finds one entry
  (`left: 1`). `strace` shows the `unlink` returns 0 before that listing, and the same directory is
  empty when listed afterwards (measured with `CLI_GIT_KEEP_FIXTURES=1`). A file created, opened,
  unlinked and listed in one process leaves nothing on this mount in a Python probe, and its link
  count reads 1 where tmpfs reads 0; which layer keeps the entry in the product's listing (the
  kernel's FUSE directory cache or fuse-overlayfs's own) was not isolated.

## Cause G: tests that fail only in a busy full run

Four tests failed in one full run and passed in another run of the same build:
`package_repair::pnpm_runs_in_the_new_worktree_only` (failed in the full suite, passed in both
filtered runs of the same list), `package_repair::quiet_runs_the_repair_and_says_nothing_about_success`
and `fault_trace::a_disk_that_fills_during_a_commit_leaves_a_recoverable_repository` (failed on the
LFS branch's build, passed on `main`'s) and
`push::bounded_push::tests::a_push_that_floods_and_ends_keeps_its_status_and_says_its_output_was_cut`
(failed on `main`'s full suite, passed in three filtered runs). Two of these run `pnpm`, and the
flood control decides a process's ending from a short look; both are the shape that a loaded host
moves. They are not causes of the gate's failure on this host, and none of them names a mount fact.

## Cause H: the two standing failures

`repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone`
and `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing` fail here
and failed on the Btrfs host (`doc/handover/cli-git-rust-implementation.md`, the standing three, of
which `owed_push` passed on this host). Both complain that a scratch directory named
`cli-git-lock-holders` is left behind. They fail with the fixture root on tmpfs as well, so no mount
change touches them, and they are not this host's doing.

## Verification

Versions: the image above (rustc 1.100.0, git 2.56.1, cargo-nextest 0.9.151), podman 6.2.0,
ZFS 2.4.4, kernel 7.3.5-1-cachyos, `main` at `cff64adfa0` (this branch changes docs only, so its tree
is `main`'s build).

The harness is the gate's own image, run exactly as the gate runs it, with a nextest filter
expression that names the failing tests:

```sh
podman run --init --network=none --memory=2g --cpus=4 --pids-limit=1024 --blkio-weight=10 \
  --cgroup-conf=io.weight=10 <image> \
  cargo nextest run --offline --locked --all-targets --no-fail-fast --success-output=never \
  -E 'test(/^(<names separated by | >)$/)'
```

The tests the gate's whole suite failed on `main`'s build are the 185 in
`main-failing-names.txt`. Running that list as its own arm, twice, and with the workaround, twice,
gave:

- as-is: 184 then 183 of 185 failed; 103.9 s and 103.8 s; the container's `memory.peak` 430 MB and
  409 MB;
- `--tmpfs /gate-fixtures` with `CLI_GIT_TEST_ROOT=/gate-fixtures`: 5 of 185 failed, twice; 40.9 s
  and 42.0 s; `memory.peak` 681 MB and 680 MB;
- a host `/dev/shm` directory mounted there with the same variable: 5 of 185 failed, twice; 43.9 s
  and 42.8 s; `memory.peak` 687 MB and 601 MB.

The residual five are cause E (two), cause A2 (one) and cause H (two).

The per-cause name lists in the investigation's `scratch/` hold 184 of the 185 between them, with no
duplicate: cause A 108, A2 1, B 29, C 28, D 10, E 2, F 3, G 1 and H 2. The one name no list holds is
`package_repair::pnpm_runs_in_the_new_worktree_only`, which the cause G passage above names: it failed
in the full suite and passed in both filtered arms. It is also the only name of the 185 absent from
the as-is arm's first run's 184 failures, and one of the two absent from its second run's 183 (the
flood control is the other), so it moves with the host's load as cause G says.

The full suite ran once on `main`'s build, as the control: 185 of 9,173 failed, listed above. The
earlier session's LFS branch failed 187 of 9,189; comparing the two lists leaves four differences and
every one of them is a test that failed in one run and passed in another of the same tree
(`package_repair::quiet_runs_the_repair_and_says_nothing_about_success`,
`fault_trace::a_disk_that_fills_during_a_commit_leaves_a_recoverable_repository` and
`merge::merge_isolation_carry_apply::tests::a_save_whose_bytes_accept_the_whole_merge_is_not_replaced_by_the_captured_layer`
on the LFS build; the flood control on `main`'s). So this host's failures are the same list
whichever build of the wrapper runs them.

What does not run here, and is not a failure of the gate: `cargo-shadow` and `cargo-nextest` are
absent, so the mise task's first step fails and only its second step runs.

## Verified workarounds that change no host setting

Both move the tests' fixture root off the fuse mount, which removes causes A, B, C, D, F and the
flaky arm of G: 108 + 29 + 28 + 10 + 3 = 178 tests, plus one flaky one. Neither touches the two
standing failures, cause E or cause A2, so the gate would still be red at 5.

The knob already exists for this and is documented as such (`FIXTURE_ROOT_VARIABLE` in
`src/native/testing/test_support.rs`: "Environment variable that moves the fixture root, for example
onto a tmpfs for speed"). Neither way can be passed through `bin/test-native-container.ts` as it
stands: the gate accepts only `--memory`, `--cpus` and `--pids-limit` and passes no `--env` and no
`--tmpfs` to `podman run` (`bin/native-gate-container.ts`, the two `runGateContainer` calls in
`bin/test-native-container.ts`). So each is a change to the gate, not to the host.

### A tmpfs the container makes for itself

The gate's `podman run` adds `--tmpfs /gate-fixtures:rw,size=1500m,mode=1777 --env
CLI_GIT_TEST_ROOT=/gate-fixtures`.

- Measured: 5 of the 185 fail (180 fixed); 2.5 times faster than the as-is arm (103.9 s against
  41 s, both arms run twice, interleaved); about 250 MB more peak memory in the container's cgroup
  (430 MB against 680 MB of a 2 GiB cap), which is the fixtures' own bytes.
- What it changes about what the tests test: the fixture root becomes a tmpfs, so the mount-dependent
  controls exercise the "birth time present, mount on the local list, Git may run a monitor" arms and
  no longer exercise the absent-birth-time and FUSE arms; `lock::lock_birth_times::tests::
  the_hidden_and_coarse_arms_report_what_other_file_systems_do` remains the synthetic control for
  those arms. In the measured arm two location controls examined nothing about the moved root, because
  each returned early when the variable was set (`src/native/binary/binary_fixture_tests.rs` and
  `src/native/testing/test_support_tests.rs`); the branch makes both examine it, and the section "Two
  location controls checked nothing in the gate" says what each now asserts. The worktree-copy ladder
  then reads `tmpfs` instead of `fuse.fuse-overlayfs`, which is a different "other file system" branch,
  and the `fault_trace` kill list still names a call the flow does not make (cause E).
- The `size=1500m` is a ceiling under the container's 2 GiB memory cap; a run that outgrows it fails
  with `ENOSPC` rather than with a mount cause. No run here came near it (peak growth 250 MB). That
  arm's container peaked at 680 MB with about 250 MB of fixtures in it, which leaves roughly 1618 MB of
  the cap for fixtures. The earlier whole-suite run, whose sampler record this branch does not keep,
  sampled 994 MiB of process pages (the history below), which left only about 1054 MiB of that cap for
  fixtures, so the `1500m` those figures suggested was not reachable in the gate; the branch ships
  `768m` instead (the section "The gate's fixture root is a tmpfs the container makes").

### A host tmpfs directory, bind-mounted

The same variable and mount, with `--mount type=bind,source=<a directory on a host tmpfs>,
destination=/gate-fixtures`, the host directory made `0777` first.

- Measured: the same 5 failures, 43.9 s and 42.8 s, `memory.peak` 687 MB and 601 MB.
- It costs the same and needs a host directory to exist and be world-writable, which is a change to
  the host's state (though not to any configuration), so the container-made tmpfs is the better of
  the two.

## Ways that change the host (not run here)

Each needs the owner's authorization; none was run.

- A ZFS dataset with `utf8only=off` and `normalization=none` for podman's storage would remove cause
  A and A2 (109 tests) and nothing else: the fuse mount stays FUSE, so B, C, D, F and E stay.
  ```sh
  sudo zfs create -o utf8only=off -o normalization=none -o mountpoint=/var/lib/containers-zfs zpcachyos/ROOT/cos/containers
  sudo systemctl stop podman.socket 2>/dev/null; podman system reset --force
  ```
  and a `~/.config/containers/storage.conf` naming `driver = "overlay"` with
  `graphroot = "/var/lib/containers-zfs/storage"`, then `podman system reset` again so the store is
  rebuilt. `podman system reset` deletes every image and container of this account, the gate's own
  images included.
- A kernel `overlay` or Btrfs mount under the fixture root would remove causes A to F at once, and Btrfs
  or an overlay would also make the copy flow clone, removing cause E. Reaching either from a ZFS host
  means a loop device or a dataset of another type mounted by root, for example
  ```sh
  sudo truncate --size=40G /var/lib/gate-btrfs.img && sudo mkfs.btrfs /var/lib/gate-btrfs.img
  sudo mount --options loop /var/lib/gate-btrfs.img /var/lib/gate-btrfs
  ```
  and then `--mount type=bind,source=/var/lib/gate-btrfs,destination=/gate-fixtures` with
  `CLI_GIT_TEST_ROOT=/gate-fixtures`. If the device stays Btrfs, the mount type the product and Git
  see is `btrfs`, which is on both of the product's lists, has birth times, takes any byte in a name,
  and clones. Whether a bind mount of Btrfs inside a `fuse-overlayfs` container reports `btrfs` as
  the mount type was not measured, since mounting it needs root.
- Changing podman's storage driver to `vfs` would put every layer on the same dataset, so it changes
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

`cargo clippy` is not part of cargo: it looks for `cargo-clippy` on `PATH`, which sets
`clippy-driver` as the wrapper for the workspace's own crates. On this host the mounted sysroot is
`nightly-x86_64-unknown-linux-gnu`, whose `bin/` has no `cargo-clippy` and no `clippy-driver`, so both
are found later in `PATH`, in the image:

```text
toolchain bin: cargo rust-gdb rust-gdbgui rust-lldb rustc rustdoc
cargo-clippy -> /usr/sbin/cargo-clippy
clippy-driver -> /usr/sbin/clippy-driver, version clippy 0.1.100
rustc -> /toolchain/bin/rustc, rustc 1.102.0-nightly (5df8c02af 2026-11-16)
```

The dependencies are then compiled by rustc 1.102.0-nightly, and the wrapper itself by the image's
clippy 0.1.100, which cannot read their metadata: E0514, whose help line names the 1.100 compiler
that is failing. Thirteen errors for the library and fifteen for its test target, none of them about
the sources. The same command succeeds where both compilers come from one toolchain:

- with the image's own toolchain (no mount, no `RUSTC`): `cargo clippy --offline --locked
  --all-targets -- -D warnings` exits 0 in 83 s;
- with the repository's locked toolchain mounted (the same command with
  `--volume /home/user/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu:/toolchain:ro`
  and the same three variables): exits 0 in 130 s, after recompiling the graph with 1.102.0-nightly.

Either is a fix. The second keeps the Clippy the repository pins; it needs the host to name a sysroot
whose toolchain has the clippy component, which this host already has. Running the gate as
`RUSTUP_TOOLCHAIN=nightly-2026-11-16 systemd-run ... node bin/test-native-container.ts` gives
`rustc --print sysroot` that toolchain and changes no host file; `mise run toolchain:install` would
make the same toolchain the default instead. The first needs the gate to stop mounting the host's
sysroot, and then Clippy is the image's pacman clippy (0.1.100) rather than the pinned one.

Every list of test names above is a file beside this investigation's logs, in
`/home/user/temp/agent/git-policy-cli-container-gate-on-the-zfs-host-investigation-2026-11-19/scratch/`:
`cause-<cause>.txt` holds the names, `main-failing-names.txt` the 185 the full suite failed, and
`main185-a1.log`, `main185-b1.log` and their siblings the arms' own nextest output.

## What does not work

- Supplying the variable through `mise`, a shell or the image: the gate's `podman run` passes no
  `--env`, and the image sets no `ENV CLI_GIT_TEST_ROOT`; nothing inside the container sees a host
  variable.
- Moving only `target/tmp` (`CARGO_TARGET_TMPDIR`) or only `target/test-fixtures`: the unit face and
  the binary face use different roots, and both read the same variable, so one `--tmpfs` with the
  variable covers both where the helper honours it (cause A2 does not).
- `podman run`'s `--uidmap` or a different user: the refusal is the dataset's, not the account's; the
  host's own `mkdir` fails with the same errno.
- `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED`, which the image sets: it silences the btrfs-only controls, and
  none of the 185 is a btrfs control.
- Checking the Git citation in a browser: `agent-browser` could not launch Chrome in this session
  (`Socket path too long: <run folder>/org.chromium.Chromium.XXXX/SingletonSocket`), and passing
  `--profile`, `TMPDIR` and `XDG_RUNTIME_DIR` did not move that socket. The two Git files were read
  through the GitHub REST API at the pinned commit instead.

## Upstream filing decision

Nothing to file, by the six constraints:

1. Not upstream's fault: the five mount behaviours are each documented or deliberate. ZFS refuses a
   name with `utf8only=on` by design; Git refuses `core.fsmonitor` on a FUSE working tree by design,
   with a setting to allow it; the product's two file-system lists are ours.
2. Upstream could not fix what is left: a FUSE mount has no kernel-held birth time to report and no
   kernel clone, and fuse-overlayfs cannot make its mount type `overlay`.
3. `fsmonitor.allowRemote` is upstream's supported answer for the Git part, and we do not set it.
4. Not reached: constraints 1 to 3 already decide it.
5. Not reached.
6. Not reached.

The gate is ours, so what needs a change is here: the gate's `podman run` (the fixture root or the
Clippy toolchain), one test helper (cause A2) and possibly the kill list's `ioctl` entry on a file
system that does not clone (cause E).

## What was fixed, and what was measured after

Fixed on the branch `fix/git-policy-cli-container-gate-on-a-zfs-host`, with no change to the host:
nothing on the ZFS dataset, nothing in podman's storage, and nothing in the installed toolchains.

### The gate's fixture root is a tmpfs the container makes

`bin/native-gate-fixture-root.ts` holds the mount and the knob, and `bin/test-native-container.ts`
spreads them into the suite's `podman run` on every host:

```text
--tmpfs /work/package/git-policy/cli/target/gate-fixtures:rw,size=768m,mode=1777
--env CLI_GIT_TEST_ROOT=/work/package/git-policy/cli/target/gate-fixtures
```

The root is a child of the image's `/work` tree rather than the `/gate-fixtures` this investigation
measured with, because a fixture crate that runs Cargo takes the configuration of the directories above
it, and the image's vendored-source mapping is `/work/.cargo/config.toml`. The root it replaces
(`target/tmp` and `target/test-fixtures`) was below `/work` too. The first full run with
`/gate-fixtures` failed two `package_repair_prewarm_job` controls whose fixture workspaces build
offline with `serde = "1"`, for want of that mapping; the root below `/work` keeps their ancestry.

`CLI_GIT_TEST_ROOT` is the variable both faces of the suite already read
(`FIXTURE_ROOT_VARIABLE` in `src/native/testing/test_support.rs` and in
`src/native/binary/binary_support_fixture_directories.rs`), so no test code changed for this.

The tmpfs answers the four mount facts the way the kernel overlay of the btrfs host did: it takes a
name that is not UTF-8, records birth times, is `tmpfs` on both of the product's lists, and Git reads it
as a local working tree. It also gives the copy flow no clone, which is cause E below.

Those four are the facts the btrfs host's kernel overlay gave, so the birth-time, mount-type and monitor
controls passed in the gate's run on that host: a tmpfs root gives that coverage back rather than
trading it away, which is why this workaround needed no owner's decision.

The size is a ceiling under the container's 2 GiB memory bound, because tmpfs pages count against that
bound: a fixture tree that outgrows the ceiling fails with `ENOSPC` on the path being written below the
fixture root, where a tree that instead exhausted the memory bound would end the run with an
out-of-memory kill. The integration pass of 2026-11-19 ran a whole suite on this root in the image
`localhost/git-policy-native-test:integrate-git-policy-cli-container-gate-on-a-zfs-host`, 211 samples
taken with a 3-second delay between iterations across 780 s, and that record is kept in this package
beside the other raw measurements (`perf/gate-resource-peaks-2026-11-19-integration-pass.jsonl`, with the
figures its reader printed, `perf/gate-resource-peaks-2026-11-19-integration-pass-summary.json`, beside
it): the kernel's `memory.peak` was 1123.8 MiB of the 2048 MiB bound, while the highest charge any sample
read was 953.1 MiB, 170.7 MiB below that peak, so a sampler that slow missed the true high point and the
figures taken from it are the samples' and not the run's. The samples' highest fixture bytes were
195.1 MiB, and their highest process pages, a sample's charge minus the fixture bytes in the same sample,
868.3 MiB; those are the maxima of two components and need not have been read in one sample. The
conservative bound is the one beside the kernel's peak: the 2048 MiB bound leaves 924.2 MiB for fixtures,
and the 768 MiB ceiling sits 156.2 MiB below that, so a run whose fixtures pass the ceiling while its
other pages hold to the peak that bound came from meets `ENOSPC` before the memory bound; the ceiling is
also 3.94 times the sampled fixture peak. The summary's own `headroomForFixturesMiB` and
`ceilingBelowHeadroomByMiB` are the sampled process pages' arithmetic (1179.7 and 411.7 MiB), which the
bound beside the kernel's peak replaces. A gate review's run on the same code apart from the repaired location
controls, whose sampler record is not kept here, read a 916.4 MiB `memory.peak` and a 221.4 MiB sampled fixture
peak, of which the ceiling is
3.47 times. No sample of that run counted an out-of-memory kill, and its only failures are the standing
pair of cause H, so every control that builds a fixture workspace built it.
The earlier whole-suite run, whose sampler record this branch does not keep, peaked at 1192 MiB of the
bound and 222 MiB of fixture bytes, with 994 MiB of process pages over 235 samples across 740 s; its
fixture peak came from a run whose two `package_repair_prewarm_job` controls failed before their
fixture workspaces built, so their bytes were not in it, and those controls hold under 64 MiB of
fixtures, since all 23 of that module's controls pass in the gate image on a 64 MiB tmpfs. The 768 MiB
ceiling is 3.5 times that peak and 286 MiB below that headroom too.
The earlier 1500 MiB ceiling, cited from the fixture peak alone, sat above that headroom: the memory
bound would have ended a run there before the tmpfs answered `ENOSPC`. The mechanism was checked in
the gate image: a 64 MiB tmpfs under a 512 MiB bound took the first 64 MiB of a 96 MiB write and
answered `No space left on device`, its cgroup reporting `oom_kill 0`, while a 128 MiB tmpfs beside it
took all 96 MiB. A caller's smaller `--memory` bound moves the memory bound back below the ceiling,
which is the caller's own choice of bound. The two probes' logs are in
`~/temp/agent/git-policy-cli-container-gate-on-a-zfs-host-review-findings-fixed-2026-11-19/scratch/`:
`prewarm-module-on-a-64m-tmpfs.log` and `tmpfs-size-returns-enospc-probe.log`.
The gate's manifest now records the root and the mount options beside the image and the limits, so a
future failure can read where the tests ran.

### Clippy mounts the repository's locked toolchain

`bin/native-gate-clippy-toolchain.ts` reads `toolchain.lock.json` through the repository's owner of
that lock (`readLockedRustToolchain` in `package/test-harness/core-matrix`), asks
`rustup which --toolchain <locked> rustc` for that toolchain's own sysroot, and mounts it where
`rustc --print sysroot` used to answer. `rustup which` answers about the named toolchain whatever the
caller's `RUSTUP_TOOLCHAIN` or rustup default says, so the gate no longer depends on the environment
that started it.

Before it runs Clippy it checks the mounted sysroot's `bin` directory for `cargo`, `cargo-clippy`,
`clippy-driver`, `rustc` and `rustdoc`, and refuses when one is missing, naming the program, the
toolchain, the Clippy component, and both ways to install it (`rustup component add clippy
--toolchain <locked>` and `mise run toolchain:install`). A sysroot without Clippy can therefore no
longer fall through to the image's older `clippy-driver`, which is what produced the E0514 errors.

`package/git-policy/cli/package.json` gains a `workspace:*` dependency on
`@monochromatic-dev/test-harness-core-matrix` for that reader, and `pnpm-lock.yaml` gains its importer
entry; nothing else in the lock changed.

### Cause A2: the one helper that wrote under `target/`

`scratch()` in `src/native/landing/service_claim_watch_tests.rs` now builds its directory under
`crate::test_support::fixture_root()`, which is the root the suite's knob moves, instead of under the
crate's `target/`. Every other control in that file already used the fixture root through the same
function.

### Two location controls checked nothing in the gate

`the_fixture_root_is_not_the_system_temporary_directory` (`src/native/testing/test_support_tests.rs`)
and `fixtures_are_created_outside_the_system_temporary_directory`
(`src/native/binary/binary_fixture_tests.rs`) each returned early when `CLI_GIT_TEST_ROOT` was set. The
gate always sets it, so the two controls that keep a fixture tree out of the system temporary directory
(issue 1809) passed there, and they passed with the root under `/tmp` as well: a control that returns
early cannot tell the two roots apart, and the times they took say nothing either way.

Both now always compare the root they use with `/tmp`, `/var/tmp` and the run's `TMPDIR`
(`system_temporary_directories`, in `src/native/testing/test_support.rs` and in
`src/native/binary/binary_support_fixture_directories.rs`): an empty `TMPDIR` names no directory and is
left out, so `TMPDIR=` no longer matches every path; each entry is the path it resolves to where it
resolves, an entry that does not resolve keeping its own path; and the unit face's root is resolved
before the comparison, so a root linked into `/tmp` and a `TMPDIR` linked to the root are both reported
(the binary face's fixture root is canonical already, because `fixture_in` canonicalizes it, and a root
it cannot resolve cannot exist). The two shape assertions stay on the path `fixture_root` returned, so a
build output that is itself a link keeps its own answer. Each still checks the shape of the root it uses:
the unit face's root always ends in `process-<pid>`, and with nothing moved it is also
`test-fixtures/process-<pid>` beside the build output, while the binary face's fixtures are under Cargo's
per-target scratch directory (`CARGO_TARGET_TMPDIR`). In the gate the unit face's root is
`/work/package/git-policy/cli/target/gate-fixtures/process-<pid>` and the binary face's is
`/work/package/git-policy/cli/target/gate-fixtures`, so both controls now assert that the fixture tmpfs
is not a system temporary directory.

Each was shown red on a moved root under `/tmp` and green on the gate's root. In the integration pass's
image, its source tree replaced by the branch's and nextest filtered to the two names, with
`CLI_GIT_TEST_ROOT=/tmp/moved-root` both failed, naming the root below `/tmp/moved-root` and `/tmp` as
its system temporary directory, and both passed with the gate's own tmpfs root and with no variable
set. That `/tmp` arm passed before the change, which is exactly the gap the two early returns left.

The repair round ran further arms of the same two controls in the same image with the same kind of
mount. With `TMPDIR=` both now pass on the gate's root, where both had failed and named the root as
being under an empty temporary-directory path. A root that is a link to `/tmp/real-root` now fails on
both faces (`/tmp/real-root/process-<pid> is under the system temporary directory /tmp`, and the binary
face's own root the same way), where the unit face had passed; and a `TMPDIR` that is a link to the gate
root now fails on both faces, where both had passed. The arms that must keep their answer did: the gate
root with `TMPDIR=/tmp/` still passed on both faces, and a moved root under `/tmp` and a `TMPDIR` set to
the gate root still failed on both.

### Cause E: the `ioctl` entry follows the fixture root's file system

The copy flow clones each staged file only where the file system may share data blocks between files;
every other file system has Git check the worktree out, and then the flow makes no `ioctl`. The
product's answer to that question is now `may_clone` in `src/native/subvolume/fs_kind.rs` (moved there
from `worktree_snapshot_source`, where it stays the same function with one more reader).

The binary control's `Flow` records which kill-list calls the file system decides, the copy flow
records `ioctl` with `may_clone`, and `require_the_kill_list_is_made` counts only the calls that hold
for the fixture root the run measures. The `ioctl` sweep says it did not run, and why, on a fixture
root that does not clone, as the `strace`-less arms already do; on one that does it sweeps up to twelve
positions of the `ioctl` calls, as every other kill control of that file does. It skips only when the
run states that the absence of a cloning root is known, by the same
`CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED` the btrfs controls read (the gate image sets it), and fails with
the fixture root it found otherwise, so no run passes the sweep as coverage it does not have.

### Five controls that assumed the old fixture root

The first full run on the tmpfs root (with `/gate-fixtures`) failed five controls that the overlay had
let pass. Each failed for its own reason, and each was a dependency on the root rather than a product
defect:

- `conflict::discarded_prune::guard_tests::a_removal_that_fails_part_way_is_reported_and_audited_as_partial`
  read the audit line's file names in the order the removal walk made them and compared them with a
  sorted list. tmpfs lists a directory in another order, so the control now sorts both sides: what it
  holds is that the line names the files that went and no other.
- `repair::package_repair_prewarm_job::tests::the_filling_puts_the_lock_back_as_checked_out` and
  `the_filling_publishes_a_clean_copy_and_leaves_the_source_alone` build a fixture workspace offline
  with `serde = "1"` and resolved it from the image's `/work/.cargo/config.toml`, which a root outside
  `/work` does not reach. The root below `/work` keeps that ancestry, and the controls are unchanged.
- `worktree::worktree_add_binding::tests::a_relative_pointer_without_links_is_bound` climbed a fixed
  seven levels with `..` and expected to land on the file system root. `/gate-fixtures` is shallower
  than the old root, so the climb left the root and the pointer was refused. The climb is now derived
  from the destination's own depth, and the refused one climbs past it by one.
- `worktree::worktree_convert::safety_tests::a_difference_keeps_the_old_tree_and_nothing_is_copied`
  expected the contents difference to be the one reported, but the comparison reports the first
  difference its walk meets, and tmpfs meets the file only the old tree holds first. The control now
  checks the promise (old tree kept, installed edit untouched, nothing copied), settles that file, and
  asks again, so the contents difference itself is the one named.

### Measured after the changes

Two whole suites ran on this host against the branch's tree, each in the gate's own containers
(`--memory=2g --cpus=4 --pids-limit=1024`).

- The first run, on a tmpfs at `/gate-fixtures`, before the five controls above were fixed:
  `9173 tests run: 9166 passed (5 slow), 7 failed, 28 skipped` in 743 s. Its two failures besides
  those five are the standing pair below, and its Clippy step compiled clean from the mounted locked
  toolchain (the gate's refusal named only the test run). Its output is kept beside the second run's
  under `scratch/keep/` of `~/temp/agent/git-policy-cli-container-gate-on-a-zfs-host-fix-2026-11-19/`,
  read back from the system journal of the two containers it ran (`tender_blackburn`, cgroup
  `libpod-c731bc55...`, and `great_maxwell`, cgroup `libpod-1418d739...`):
  `gate-run-1-whole-output-from-the-journal.log` holds the seven failures with their process output,
  the two `package_repair_prewarm_job` panics among them, and the summary line, and
  `clippy-run-1-from-the-journal.log` holds the Clippy step's whole stream, which ends with the
  toolchain's own `Finished` line, 2m 06s after it started, and no warning or error line, the ending a
  refused Clippy does not have (`clippy-run-2-final-tree.log` ends `error: could not compile` instead).
  The journal dropped that run's final success-output block after test 1087 of 9173 and every
  `BTRFS CONTROL NOT RUN` line, the same drops as the second run's log. The gate's own refusal text
  went to the driver's stdout and survives only in that driver's session record, `run-1/feed.jsonl` of
  the same folder, which is not kept as evidence here; the two logs above are what the claim rests on.
- The second run, on the root below `/work`, after the fixture-root, toolchain and five-control changes:
  `9173 tests run: 9171 passed (5 slow), 2 failed, 28 skipped` in 775 s. The two failures are cause H,
  the standing pair: `repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone`
  lists `cli-git-lock-holders` in the `.git` directory it finds and expects it absent, and
  `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing` fails with
  `no scratch directory or record is left: ["cli-git-lock-holders"]`. Both fail on a btrfs host too
  (cause H above), so no mount change touches them. The 28 skipped are nextest's ignored tests, the
  same count this investigation's own full run reported; the btrfs controls are not skipped,
  each prints its line and passes. The harness that ran this suite was stopped by a fault of its own
  while the gate's container ran on, so this run's evidence directory holds no `passed.json`, and its
  output was read back from the journal, which drops some of a burst this size (the btrfs lines and the
  passing controls' lines among them).
- The suite container's cgroup peaked at 1192 MiB of its 2 GiB bound and the fixture tmpfs at 222 MiB,
  with no OOM kill, over 235 samples across the first run's 740 s; its process pages, the container's
  charge minus the fixture bytes in the same sample, peaked at 994 MiB. That sampler's record is no
  longer kept, so those three figures stay here as history; the tmpfs size now cites the integration
  pass's record (`perf/gate-resource-peaks-2026-11-19-integration-pass.jsonl`), whose kernel
  `memory.peak` is 1123.8 MiB of the bound while its 211 samples read 195.1 MiB of fixture bytes and
  868.3 MiB of process pages. The 1192 MiB measurement is what showed the 1500 MiB ceiling then shipped
  to sit above the 1054 MiB that measurement left for fixtures. The second run's sampler ended with its
  harness and still read the first root's path, so its recorded peak of 992 MiB covers only its first
  570 s.
- The first run's gate took 1136 s including a fresh base image (a `pacman` transaction with network)
  and the layered test image; the second reused that base. The investigation's arm of the 185 tests
  took 41 s against 104 s on the container's own tree; this whole-suite run's 743 s and 775 s are not
  comparable with the 1053 s the same suite took on the dataset, because that run's tree was also the
  failing one.
- Clippy: the first run's gate step exited zero from the mounted locked toolchain (the gate's refusal
  named only the test run). The Clippy step over the second run's tree failed on
  `clippy::shadow_unrelated`, which this gate denies: the two-step assertion the conversion control
  gained above bound `message` twice in one function. That step was run on its own, with the gate's
  image, mounted toolchain and flags, because the gate's own Clippy container was still running when
  the harness that started it was stopped and its captured output went with the client
  (`scratch/clippy-run-2-final-tree.log`). Commit `8d1122a6ae` names the second binding apart, and the
  same step over the fixed tree then finished with `Finished dev profile [unoptimized] [debuginfo]
  target(s) in 2m 13s` and no warning or error line (`scratch/clippy-run-2-fixed.log`); the two logs'
  differing outcomes are what shows that step reads this tree rather than a stale image.
- No third gate run was started, as the task set for this fix allows two. The second run's image and
  the final tree differ in exactly one source file, `src/native/worktree/worktree_convert_safety_tests.rs`
  (commit `8d1122a6ae`): the SHA-256 of every file under `src` in the image and in the worktree agree
  otherwise, and that file's only change is one local binding's name.

### What these changes do not cover

- The two standing failures (cause H) still fail, and no mount change touches them.
- The memory bound the gate gives each check also bounds its image build: `buildLimitArguments` with the
  gate's own limits (`bin/container-limits.ts`, whose implementation lives in the mutation-test package,
  called at `bin/test-native-container.ts:234`) turns that bound into the build's `--memory`. The image
  build met that 2 GiB bound during these runs; it predates this fix, which changes neither the bound nor
  the build, and the run that met it reported it.
- The cloning arm of cause E was not exercised here: this host has no btrfs and no overlay mount, so
  the runs above printed that the `ioctl` sweep did not run. The list control still requires every
  listed call on a cloning root, and `may_clone` is the product's own answer to which file systems
  those are; a btrfs host is where the sweep itself runs again, and
  `mise run //package/git-policy/cli:native:test:host:btrfs` is that run: it unsets the acknowledgement
  and runs every target on a btrfs root, so the sweep runs there, and a wrapper release waits on it
  unless the version is an alpha (`bin/native-release-host-controls.ts`).
- The mutation runner copies its work tree onto a tmpfs of its own
  (`package/mutation-test/cli/src/host/podman.ts`), so its tests already met a tmpfs fixture root. That
  root shares no data blocks between files either, which is the arm the `ioctl` control now skips; the
  gate and a campaign therefore agree about the fixture file system, which a gate pass reused as a
  campaign baseline now relies on.
