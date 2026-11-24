# Installed system and Monochromatic repo AppArmor inventory

## Status at 2026-11-23T16:09:36-05:00

The owner approved the enforcement result and requested a complete inventory.
That inventory is now
complete.
Current evidence has no additional AppArmor denial requiring a policy change,
but this is
not a guarantee against future application changes or unexercised operations.

The system remains default enforce with documented per-profile exceptions,
not universal
enforcement.
Package `fsp no` deliberately leaves unprofiled development and repo processes
unconfined.
An unconfined label is a coverage boundary,
not itself an AppArmor error.

## Installed package and executable inventory

- Installed packages:
  2,276.
- Package-owned executable candidates:
  21,975.
- Executable and symlink entries retained after inspection:
  9,273.
- Complete names and versions:
  `keep/installed-system-and-repo-inventory-installed-packages.json`.
- Complete executable metadata and symlink targets:
  `keep/installed-system-and-repo-inventory-installed-executables.json`.

## Profile inventory

The selected cache has 809 files.
Their declared source modes are:

- Enforce: 560.
- Complain: 189.
- Unconfined: 55.
- No `profile` keyword:
  5,
  classified below.

The generator owns 710 profiles.
The loaded kernel map at the snapshot contains 982 entries because
it also includes nested and dynamic profiles:

- Enforce: 675.
- Complain: 252.
- Unconfined: 55.

The complete source/cache profile and loaded-mode lists are in
`keep/installed-system-and-repo-inventory-profiles.json`.
The complete exception and
active-unconfined lists are in `keep/installed-system-and-repo-inventory-exceptions.json`.

### Files without a profile keyword

- `lxc-containers` is an include aggregator for per-container policy,
  not a missing profile.
- `usr.bin.akonadiserver`,
  `usr.bin.lxc-copy`,
  and `usr.bin.lxc-start` are valid attachment-style
  profile bodies without a `profile` keyword.
- `usr.lib.apache2.mpm-prefork.apache2` is valid but explicitly permissive.
  It is a compatibility
  exception,
  not an enforcement boundary.

### Compatibility and unconfined exceptions

The 55 explicit unconfined source profiles are enumerated in the exceptions report.
They include
browser,
container,
development,
and compatibility programs.
Their presence does not imply that
another workload should be unconfined or that an enforcing profile should be weakened.

At the snapshot,
25 active service metadata entries have unconfined main processes.
One is the
transient inventory unit itself.
Other examples include the Incus controller,
firewall and hardware
services,
and desktop helpers.
This is not a list of failed services.
No failed services were found.

## Systemd inventory

- Unit files across system and user managers: 782.
- Active service metadata entries: 234.
- Complete unit-file and service metadata:
  `keep/installed-system-and-repo-inventory-systemd-units.json`.
- Running processes:
  741,
  comprising 632 unconfined,
  90 enforce,
  and 19 complain.
- Complete PID,
  UID,
  parent,
  executable,
  label,
  and mode observations:
  `keep/installed-system-and-repo-inventory-running-processes.json`.

The two root worktree-mount handler paths are absent at the snapshot:
`/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets` and
`/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets`.
There is no installed handler at
those paths to profile or invoke.
Do not infer a missing unit from a source design alone.

## Monochromatic repo inventory

- Manifests and specifications:
  570,
  including `package.json`,
  `Cargo.toml`,
  `mise.toml`,
  and
  `SPEC.md`.
- Tracked files:
  24,756.
- Untracked non-ignored files:
  zero.
  The four unrelated modified files are tracked and preserved.
- Complete manifest metadata,
  declared binaries,
  script names,
  and task names:
  `keep/installed-system-and-repo-inventory-repository-manifests.json`.
- Complete tracked-file types,
  sizes,
  modes,
  executable flags,
  and symlink targets:
  `keep/installed-system-and-repo-inventory-repository-files.json`.

Four repo-related processes were observed.
They are intentionally unconfined under `fsp no`:

- The long-running `monochromatic-git-policy-cli` service using the exact current owner-managed
  wrapper path.
- The Node CPU-watch process.
- The key-helper shell and its `key-helper-service` child.

The exact wrapper inherit-exec rule applies when Git invokes that wrapper.
It does not attach the
Git profile to a long-running service started by an unconfined parent.
That distinction is a
coverage boundary,
not a denied operation.

## Evidence and limits

The complete reports are private under
`/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/keep/installed-system-and-repo-inventory/`.
The paths in this document are relative to that directory.

The full audit initially showed zero new denials after 2026-11-23T20:49:00Z.
A later root-run
`git ls-files` probe was correctly rejected for `dac_read_search` and `dac_override`;
it was a
harness error,
not a reason to grant bypass capabilities.
The corrected desktop-user inventory
passed.
The final direct audit-window check has zero new denials after 2026-11-23T21:12:16.815Z.
DMI
denials remain intentional.
No failed units or managed background processes remain.
`zfs-nvme-gen3.service` remains successful at 8.0 GT/s x4.

Exact-name matching is only a lookup aid.
It does not prove that a profile attaches to a process or
that a workload is confined.
Process labels,
measured operations,
and workload results decide that.
The inventory does not exercise every binary or every operation and cannot guarantee that no future
AppArmor issue will appear.
