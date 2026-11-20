# Step 5, the second executable: what this round built and measured

Written 2026-11-20T05:00:00-05:00 in
`.claude/worktrees/zfs-step-5-second-executable`
(branch `feat/git-policy-cli-zfs-second-executable`,
from `main`'s `dd752b10cf`),
for `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 5.
**Unapproved and not final:**
the owner has not reviewed it,
and nothing here was merged.

## What landed

- The `[[bin]]` `mount-git-worktree-clone-datasets`
  (`src/native/mount_git_worktree_clone_datasets/main.rs`),
  whose decisions are the library's `src/native/worktree_clone_mount_handler/`:
  `attaching_sweep_mode` (no argument,
  plan step 8),
  `boot_reconcile_mode` (step 6),
  `shutdown_unmount_mode` (step 9)
  and `adopt_worktree_parent_mode`,
  the one mode this build performs end to end in the guest
  (`worktree_clone_mount::parent_adoption_decision`,
  step 4),
  whose slot read,
  unresolved-binding refusal and surviving-slot compare have guest-free controls too;
  the arms still owed a guest are stated under "What this round did not do".
  The two installed paths,
  `/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets`
  and `/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets`,
  are constants in `handler_install_paths.rs` and are stated in `SPEC.md` and `README.md`.
- The wrapper's path dependency on
  `monochromatic-zfs-request-to-openzfs-kernel-device`,
  pinned `=0.1.0-alpha.1`.
- One archive carrying both executables:
  the workflow builds one `--bin` each,
  packages both,
  and `cargo-binstall`'s `bin-dir` names each declared bin;
  the source guard,
  `bin/native-release-bins.unit.test.ts`,
  `releasable-crates.json`,
  `toolchain.plan.jsonc` with its lock,
  the fuzz sidecar's lock and `native:install` all name both.
- `SPEC.md`'s `## Single executable and shipped policies` became
  `## The two executables and shipped policies`,
  and the README's install,
  requirements and source-layout text names both.
- The adoption arm's end-to-end driver,
  `e2e/mount-git-worktree-clone-datasets-guest.ts` with the task
  `e2e:mount-git-worktree-clone-datasets`,
  and the guest-only control `binary_zfs_adoption_guest_tests.rs`.

## Each new control, red first on committed code

- `source_guard::release_bins_tests::the_crate_declares_only_the_executables_the_release_archive_holds`:
  red:
  gate 2,
  image `zfs-step-5-red2-20261120T032300`:
  `left: ["monochromatic-git-policy-cli", "mount-git-worktree-clone-datasets"] right: ["monochromatic-git-policy-cli"]`;
  green:
  final gate:
  `PASS ... release_bins_tests`.
- `bin/native-release-bins.unit.test.ts`:
  red:
  four disagreements,
  one per list,
  each naming `mount-git-worktree-clone-datasets`;
  green:
  4 of 4 pass.
- `native-wrapper-snapshot.unit.test.ts`,
  the snapshot's path-crate list:
  red:
  gate 1:
  `cargo metadata failed ... failed to read .../package/zfs/request-to-openzfs-kernel-device/Cargo.toml`;
  green:
  `PASS` for the list,
  the copy and both snapshots.
- crate-release's `release-workflow.unit.test.ts`,
  the binding's pin files:
  red:
  `expected [ …(2) ] to deeply equal [ Array(1) ]` for `monochromatic-zfs-request-to-openzfs-kernel-device`;
  green:
  no failure.
- `toolchain lock check`:
  red:
  `the toolchain plan and lock disagree: cargo:monochromatic-git-policy-cli differs between the plan and the lock in: bins`;
  green:
  `toolchain lock is current: 66 tools, digest verified`.
- `adoption arm, the store write`:
  red:
  guest run `2026-11-20T08:45`:
  `Missing { path: "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities" }`;
  green:
  `ADOPTION recorded device=64769 inode=318343`.
- `adoption arm, another folder`:
  red:
  guest run `2026-11-20T09:05`:
  `assertion left != right failed: the replacement is another folder, so the arm proves a write; left: 318339 right: 318339`;
  green:
  the six `ADOPTION` markers of the passing guest run.
- `the handler's report on standard output`:
  red:
  install-path container run `2026-11-20T09:29`:
  `mount-git-worktree-clone-datasets 0.6.1-alpha.44mount-git-worktree-clone-datasets 0.6.1-alpha.44`;
  green:
  `--version` on stdout is exactly `mount-git-worktree-clone-datasets 0.6.1-alpha.44\n`,
  and the zedlet copy passes a `grep` pipe.

The two new binary controls that need no guest,
`neither_executable_links_an_openzfs_cddl_library` and
`both_executables_start_without_the_zfs_device`,
passed in the final gate image (`/dev/zfs` is absent there,
so the second really ran and printed no `SKIPPED`).
On this machine the second arm prints `SKIPPED`,
because the host kernel has `/dev/zfs`.

## The release checks and the unpublished binding

The release tooling's own check,
`src/crates-io-dependency-check.ts` of `package/crate-release/cli`,
run through `dispatch-release.ts` and `release.ts` before a run is spent,
reads each registry crate the wrapper depends on and refuses a version that goes to crates.io
while one of them never does:

```text
$ node --input-type=module-typescript -e '<assertDependenciesReachCratesIo for three versions>'
0.6.1-alpha.45: PASSED the crates.io dependency check (no refusal)
0.6.1: REFUSED ->
monochromatic-git-policy-cli 0.6.1 would go to crates.io, which refuses a crate that uses a crate it does not hold, and these dependencies are at a version that never goes to crates.io (only a plain number and -rc.N do):
- monochromatic-lint: package/linter/monochromatic-lint/Cargo.toml holds 0.5.2-alpha.8, and package/git-policy/cli/Cargo.toml pins 0.5.2-alpha.8
- monochromatic-zfs-request-to-openzfs-kernel-device: package/zfs/request-to-openzfs-kernel-device/Cargo.toml holds 0.1.0-alpha.1, and package/git-policy/cli/Cargo.toml pins 0.1.0-alpha.1
Release the dependencies as -rc.N or a plain number first, or release monochromatic-git-policy-cli as -alpha.N or -beta.N, which stays on GitHub.
```

What this means for the project:

- An alpha wrapper release is unaffected:
  the GitHub release path builds with `--locked` in a checkout where the path dependency exists,
  one archive carries both executables,
  and no crates.io step runs.
- A wrapper release that goes to crates.io is refused before any run,
  twice over:
  by that check,
  and by `cargo package`/`cargo publish`,
  because crates.io holds no `0.1.0-alpha.1` of the binding.
  The offline index on this host cannot isolate the binding in `cargo package`
  (it fails first on `forbidden-strings`,
  a version the offline index does not hold),
  so the release tooling's check is the evidence that names it.
  The wrapper was already refused for `monochromatic-lint` at `0.5.2-alpha.8`,
  so the binding adds one more dependency to a refusal the project had,
  not a new class of problem.
- Releasing the binding now moves the wrapper's exact pin and refreshes the wrapper's lock:
  the binding's `pinFiles` and `lockDirs` name `package/git-policy/cli`,
  its `lockDirs` also name `package/git-policy/cli.fuzz`,
  and the crate-release workflow test holds both lists to the repository's own files.
  So the binding must be released before the wrapper's pin moves,
  and a wrapper crates.io release must wait for the binding to be published at a plain or `rc` version.

## Checks and their summary lines

- Host:
  `mise run //package/git-policy/cli:native:clippy` ends
  `Finished 'dev' profile [unoptimized] [debuginfo] target(s) in ...` with no warning;
  `mise run //package/git-policy/cli:lint:types` ends with its one `TSFILE:` line and no error.
- `cargo package --list --offline --locked --allow-dirty`:
  2302 paths,
  none whose path has a `uses` segment
  (the control `source_guard::packaged_uses_links_tests` passed in the final gate).
- `bin/native-install.unit.test.ts` passes,
  including the new refusal when the archive moved only the wrapper;
  its dry run prints
  `and unless what mount-git-worktree-clone-datasets --version prints holds mount-git-worktree-clone-datasets <version>`.
- Final whole gate,
  `node bin/test-native-container.ts` in `gpc-test-zfs-step-5-final-20261120T0942.service`
  (CPUQuota 1000%,
  MemoryMax 20G,
  TasksMax 4000,
  IOWeight 10,
  image tag `zfs-step-5-final-20261120T0942`):
  `Summary [ 798.611s] 9272 tests run: 9270 passed (6 slow), 2 failed, 28 skipped`.
  The two failures are the cause-H pair of
  `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`.
  The gate stops before its Clippy step when the suite fails,
  so the Clippy evidence of this round is the host run above.
- The adoption arm in the throwaway guest (`git-policy-adoption-20261120T0905`,
  Ubuntu 24.04,
  kernel `6.8.0-147-generic`,
  OpenZFS `2.4.4-1`),
  step `run` against the kept guest:
  `1 passed, 1242 skipped`,
  with the six markers
  `ADOPTION guest=`,
  `recorded device=64769 inode=318343`,
  `refused=wrong-device`,
  `refused=link`,
  `refused=mode-widened`,
  `refused=slot-differs`,
  `done`.
  The guest was destroyed afterwards and `mvm list` no longer names it.
- The two install paths,
  in a throwaway container from the gate image as root:
  both files installed `0755 root root`,
  `/usr/libexec/...` printed the version,
  the zedlet copy's `--version` passed `grep`,
  and its no-argument invocation refused with status 2.

## What this round did not do

- The adoption arm runs one of the four refusals the plan's acceptance names.
  `adopt-worktree-parent` is refused with a link,
  a boundary,
  an ACL mismatch or an unresolved live or staged binding;
  the guest arm's markers are `refused=wrong-device`,
  `refused=link`,
  `refused=mode-widened` and `refused=slot-differs`,
  so the link arm is the only one of the four that runs there.
  `mode-widened` is a policy field and not an ACL mismatch;
  no arm puts a mount at or above the parent,
  plants an ACL,
  stands a mount at a slot
  or removes a surviving slot folder.
  (Amended 2026-11-20T06:55:38-05:00,
  in the round that took the integration pass's findings:
  that round added guest-free controls over a fixture store and a fixture checkout,
  covering a recorded slot whose folder is gone,
  a slot name that is no folder,
  the slot that agrees with its record and the one that differs
  (`adopt_worktree_parent_mode_tests.rs`).
  Still owed to a guest:
  the standing-mount arm,
  which reads as unresolved only where a mount can be made
  (`slot_folder_reason`'s `EXDEV` sentence),
  and the boundary and ACL refusals against a real mount.
  `adopt_at`'s `NoParentRecord`,
  `Store` and `Write` branches are reached by no control either.
  Step 4's decision table has its own supplied-fact controls for every refusal kind.)
- No release was dispatched and nothing was published.
- `mise run //package/git-policy/cli:lint:monochromatic-lint` could not run here:
  `sh: line 1: monochromatic-lint: command not found`,
  because no released tool of this repository is installed on this host
  (the plan's step 1 choice).
  The layout rules the linter enforces were followed by hand:
  the new folder lists its files in dependency order,
  its `uses/` links name the folders its code uses,
  and `cargo package --list` carries no `uses` entry.
- The first commit of the branch,
  `aea55bd922`,
  left the handler's module files out of its pathspec and does not
  compile;
  `da0612a297` completes it.
- The two cause-H failures still fail;
  no change of this round touches them.
