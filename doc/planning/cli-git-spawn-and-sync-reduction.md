# cli-git: fewer processes and fewer syncs per commit, and retry on process exhaustion

Status: accepted for implementation by the user (2026-10-29), scope below.
Owner package: `package/git-policy/cli`. Tracking issue: 1804 (follow-up work found while stabilizing it).

## Why

A wrapped `git commit` of one explicit path starts about 45 real Git processes and makes about 41 sync calls,
against one process and no syncs for direct Git. Measured on an idle host with the debug and release builds alike:

- wall time about 180 ms per commit, of which about 77 ms is inside sync calls (strace, same fixture);
- the container gate takes 12 to 32 seconds for the commit tests under host load, and the same suite takes about 67
  seconds when the host is quiet;
- a host-wide fork-failure episode (the user cgroup recorded 598 limit events) failed commits with
  `Resource temporarily unavailable (os error 11)`.

Every process start and every sync is multiplied by load, so the fix is to do less of both and to ride out a short
refusal to start a process.

## Decisions (user)

- Process-exhaustion safeguard: retry failed starts everywhere, and also retry read-only and shadow-repository Git
  commands that fail inside themselves (`unable to create thread`, `cannot fork`), with an allow-list.
- Spawn reduction: save as many processes as possible, engineering the replacement in the wrapper where the result is
  provably identical to Git's, in the manner of `package/forbidden-strings/core` (own engine, an
  `OPTIMIZATIONS.md`-style log of measured wins and rejections, oracle-checked parity).
- Journal syncs: option B of the earlier question (sync at the barriers before side effects), after a crash
  harness exists that tears, drops and reorders records.

## Method

- Parity first: every in-process replacement is tested against real Git (`git` is the oracle) over generated cases,
  and falls back to spawning Git for any shape it does not handle exactly. A fallback is never a failure.
- Every replacement shows its win with the same scripts: `GIT_TRACE2_PERF` process counts and `strace` sync counts.
  A permanent budget test per measure ratchets down after each slice (a regression fails the host and gate suites).
- A guard test is shown failing before the change is kept (the repo's rule).
- What is measured and rejected is recorded here, with the number or the counterexample.

## Process-start retry (slice R)

A start that fails with `EAGAIN` (`ErrorKind::WouldBlock`) started nothing, so retrying is always safe.

- One helper starts every child: 10 ms backoff doubling to 500 ms, a 5 s total budget, one warning per retry, then the
  existing error plus the attempt count. It covers the roughly 20 call sites (`run_git`, bounded children,
  forwarding including the `execve` path, hooks, signal relay, post-commit, probes).
- Git commands that start and then fail inside themselves are retried only from an allow-list of commands that are
  harmless to re-run (read-only queries, and the idempotent shadow-repository steps), matched on the exit status and
  the wording of the failure; mutating steps (reference updates, landing) are not retried, and recovery already
  completes or rolls back those.
- Tests: a unit test with a start that refuses K times; the fork-failure sweep gains burst cases
  (`--inject=clone3:error=EAGAIN:when=N..N+K`), which must now succeed for bursts shorter than the budget.

## Candidate replacements, with the exactness condition for each

Measured process lifetimes of one commit (strace): `findmnt` 3 runs about 25 ms, `rev-parse` 12 runs about 32 ms,
`config` 7 runs about 16 ms, `git hook` 4 runs about 14 ms plus the wrapper's own hook shim re-executions.

- Hook shims for events with no hook: the shadow commit points `core.hooksPath` at shims that re-enter the wrapper,
  which takes the hook lock (a synced lock candidate) and runs `git hook run --ignore-missing <event>`. An event with
  no user hook needs no shim. Exact when conservative: the shim is omitted only if the user's hooks directory has no
  entry of that name at all (any mode, because Git prints a hint for a non-executable file) and the configuration has
  no `hook.<name>.event` or `.command` key; any other shape keeps today's behavior. Saves up to four wrapper
  re-executions, four `git hook` runs and four synced lock candidates.
- `findmnt` (3 runs): the filesystem identity of `.git` and `index.lock`. To be replaced by `statx`, `/proc/self/mountinfo`
  and `/dev/disk/by-uuid` when the mapping is unambiguous; otherwise `findmnt` stays. Needs a check of how the
  identity is consumed before choosing (recorded below when read).
- One configuration snapshot (`git config --list -z --show-scope`) in place of the separate `config` runs
  (`core.hooksPath`, the four `hook.<event>.enabled` reads already batched, the maintenance keys, ref storage).
  Last value wins, `-c` and `GIT_CONFIG_*` values come last, bool canonicalized by Git's rules; `--type=path` values
  starting with `~` or `%(prefix)` fall back to a spawn.
- Empty-tree object name: the hash of `tree 0` under the repository's object format (a constant per format) in place of
  `git hash-object -t tree --stdin`.
- `symbolic-ref HEAD` (files ref storage): read `HEAD`; anything but `ref: <name>` or a full object name falls back.
  The landing re-read stays a read of the same file, so the compare-and-swap protection is unchanged.
- Sequencer markers (`MERGE_HEAD`, `CHERRY_PICK_HEAD`, `REVERT_HEAD`) on the files backend: when none of the
  loose files (including the six lookup spellings `rev-parse` tries) exists and `packed-refs` does not contain the
  name, they are absent without a spawn; any hit falls back to `rev-parse --verify`. A dangling marker therefore
  stays a conclusion (pinned by `a_marker_naming_a_missing_object_is_still_a_conclusion`).
- `git version`: cached per binary identity (path, size, mtime, inode) in the repository's private directory.

## Rejected, with evidence

- Reading the three sequencer markers with `cat-file --batch-check`: it reports a marker that names a missing object
  as missing, so a corrupt merge would become a plain commit that drops the merge parent.
- Merging `core.hooksPath` with the maintenance reads by hand: different phases and a `--type=path` value; covered
  instead by the snapshot above.
- Capping Git's threads (`pack.threads=1`, `core.preloadindex=false`): 101 ms against 105 and 100 ms baseline, and
  the container was never throttled (`nr_throttled 0`).
- Re-reading `HEAD` and the target ref once instead of twice: the second reads at landing are the compare-and-swap
  protection.

## Journal syncs (slice J, task 69)

Order from the independent review: skip directory syncs where nothing changed (three `ensure_*` syncs), skip the two
syncs after removing a landed transaction directory, defer directory syncs to barriers, add torn-record tolerance for
pre-landing records with a tearing, dropping and reordering harness (a kill is not a power loss), and only then drop
file syncs of pre-landing records. Lock-candidate syncs stay: a torn blocking `owner.json` would stick every
acquirer. The sync-site table is in `doc/handover/cli-git-rust-implementation.md`.

## Budgets

The host and gate suites carry a spawn budget (from `GIT_TRACE2_PERF`) and the host suite a sync budget (from
`strace`); each slice lowers its budget to the measured count. Starting point: 45 processes, 41 syncs.

## Filesystem identity without `findmnt` (slice S2)

Scope: the three `findmnt` starts of a commit (two on `index.lock`, one on `.git`), all made by
`package/git-policy/cli/src/native/filesystem_identity.rs`. The `stat --file-system` fallback that follows an empty
`findmnt` answer is not part of this slice.

### What must stay identical

Read from `filesystem_identity.rs`, `real_index_lock.rs`, `recovery_files.rs` and the incumbent
`package/fs-id/core/src/platform-resolvers.ts`, `parsers.ts`.

- **The recorded string.** `filesystem_identity(path)` canonicalizes `path`, then on Linux returns
  `fs-uuid_<payload>` where the payload is the `findmnt --target <canonical> --output=UUID --noheadings` standard
  output, trimmed with JavaScript `.trim()` semantics, lowercased, and accepted only if it is 1 to 512 characters of
  `[a-z0-9.-]` and not a lone `-`. Example: a vfat UUID `44C8-C6BA` is recorded as `fs-uuid_44c8-c6ba`.
- **The empty case.** An empty, multi-line or unsafe payload (`findmnt` prints `\n` for `/`, `/tmp`, `/proc`, `/dev`,
  fuse mounts and `\n\n` for two mounts stacked on one mount point) is not an identity. The resolver then runs
  `stat --file-system --format=%i` and records `f-fsid_<hex>`. A string is therefore never empty; `None` only when both
  commands fail.
- **Where it is recorded and compared.** `created_identity` (`real_index_lock.rs`) records it for the directory
  holding the real index into the journal field `fsId`. `owned_lock_mismatch_with` and `release_owned_lock_with`
  (`recovery_files.rs`) compare it, for the lock file, with the recorded `fsId` together with device and inode.
- **How a mismatch is treated.** `None` from `filesystem_identity` is an error, never a mismatch: the landing stops
  ("the filesystem identity of the worktree is unavailable") and recovery returns a `RecoveryError`. A string that
  differs from the journal is "Index lock identity changed", the lock is reported `Foreign` and is left in place.
  A wrong string therefore leaves the owner's own lock in place as foreign and stops recovery from releasing it; the
  in-process path must never produce a string `findmnt` would not.
- **Journals outlive binaries.** The record is read by older builds of this wrapper and by the TypeScript incumbent
  (`fsId` in the journal JSON). The new code must produce the byte-identical text the old code and `fs-id-core`
  produce for the same filesystem: same prefix, same lowercase, same fallback to `f-fsid_`.
- **Mount namespaces and containers.** `findmnt` is a child of the wrapper, so it reads the wrapper's own
  `/proc/self/mountinfo`, root directory and `/run/udev`. The in-process code reads the same files in the same
  process, so every namespace question has the same answer. A container without `/run/udev` data for the device gets
  no UUID in-process, and the code spawns `findmnt` exactly as before; a host-recorded `fs-uuid_` and a
  container-computed `f-fsid_` already differ today, and that does not change.

### How `findmnt` 2.41 gets the text (read from util-linux `misc-utils/findmnt.c`, confirmed by `strace`)

- `--target X` first lists every `/proc/self/mountinfo` line whose mount point equals `X`; if none, it walks `X` up
  one component at a time (`mnt_table_find_mountpoint`, backward, so the last line wins) and then lists every line
  whose mount point equals the one found. Stacked mounts therefore print several lines.
- The UUID column resolves the source field with `mnt_resolve_spec`, canonicalizes it, and asks libudev for the block
  device by sysfs name (`/sys/class/block/<name>`), which reads `/run/udev/data/b<major>:<minor>` and returns
  `ID_FS_UUID_ENC` with `\xHH` sequences decoded. Only when that is absent does it call `libblkid`, which probes the
  device and needs read access. `strace` on this host shows `openat("/run/udev/data/b252:1")` and no access to
  `/dev/disk/by-uuid`, so `/dev/disk/by-uuid` is not consulted by the oracle (the brief named it; the oracle is the
  authority, and for a multi-device Btrfs the link names only one member while every member has the record).

### Exactness rules for the in-process path

- Answer only when exactly one mountinfo line has the winning mount point, the mount ID of that line equals the
  `stx_mnt_id` that `statx` reports for the path, and the source is a `/dev/` block device whose udev record holds
  `ID_FS_UUID_ENC`. Every other shape spawns `findmnt` unchanged.
- A source that is not a path (`overlay`, `tmpfs`, `proc`, `composefs`) has no UUID: `findmnt` prints an empty line.
  That conclusion is drawn without a spawn only if the source also names no existing path and no sysfs block device.
  The `stat` fallback still runs.

### Result

Code: `src/native/filesystem_identity_mount.rs` (pure decisions plus one `statx` call) and the `MountUuid` match in
`linux_fs_id` (`src/native/filesystem_identity.rs`). Assumption recorded: `findmnt` is linked with libudev, as the
Fedora and Debian builds are; the differential tests fail loudly on a host where it prints something else.

Process starts of one wrapped `git commit --allow-empty` (the repository's `attribute-commit.ts`, pointed at the debug
build of each commit through `BINARY`; `BASE` picks the fixture directory), run from `${HOME}/temp/agent/s2`:

- Fixture on Btrfs with a udev record, before (parent commit `e345bba143`): 49 starts, 3 of them `findmnt`.
  After: 46 starts, 0 `findmnt`. Command:
  `BINARY=<worktree>/package/git-policy/cli/target/debug/cli-git-native node attribute-commit.ts`.
- Fixture on tmpfs (`BASE=/tmp/s2-fixture`), before: 52 starts, 3 `findmnt` and 3 `stat`. After: 49 starts, 0
  `findmnt`, 3 `stat`. The `stat --file-system` runs remain; replacing them needs the `f_fsid` word order of GNU
  `stat` and is a separate slice.
- Cost of the removed starts: 5 to 14 ms each under `strace`; the in-process work is one `statx`, one read of
  `/proc/self/mountinfo`, two small sysfs and udev reads.

Differential evidence (`findmnt` is the oracle; every non-`Unknown` verdict is compared with its exact standard
output, and `Unknown` is printed with what `findmnt` printed):

- Host paths: `/`, `/tmp`, `/proc`, `/dev`, `/run`, `/sys`, `/usr` (empty line, `Absent`); `/etc`, `/var`, `/var/home`,
  the repository (Btrfs subvolumes of one device, `1bb3d23e-...`), `/boot` (ext4), `/boot/efi` (vfat, `44C8-C6BA`),
  `/var/mnt/encrypted` (device mapper source `/dev/mapper/crypt_sda`), a nested Btrfs subvolume whose `st_dev` differs
  from its mount's (matched through the mount ID, so subvolumes need no fallback), fuse mounts (`Absent`).
- A private user and mount namespace (`unshare --user --map-root-user --mount`, no host mount touched): bind mounts
  of a directory, of a directory whose name has a space, of a directory whose name has a backslash, of another
  device's directory, of a single file, of a `tmpfs` directory, a `tmpfs`, an unprivileged `overlay` (`Absent`), and
  two mounts stacked on one mount point. All equal `findmnt` or fall back.
- A rootless Podman container (`--memory=2g --cpus=2 --network=none`, Debian trixie, `findmnt` 2.41, no
  `/run/udev`): overlay root, `/proc`, `/dev`, `/etc/hosts` bind and `/dev/shm` are `Absent` and equal `findmnt`; the
  Btrfs bind whose source `/dev/mapper/crypt_sda` does not exist in the container is `Unknown` and spawns.

Classes that fall back to spawning `findmnt` (kept exact by construction, each with its reason):

- **Stacked mounts on one mount point** (`/proc/sys/fs/binfmt_misc` here: `autofs` under `binfmt_misc`): `findmnt`
  prints one line per mount, `\n\n`, which the payload check rejects, so the identity comes from `stat`. Counterexample
  for answering in-process with the last entry: it would claim `Absent` for a two-line output.
- **A mount covered by a later mount on an ancestor**: the name walk finds the covered mount, `statx` reports the
  covering one. `findmnt` prints the covered mount's line (namespace case `cover/y/z`); the mount ID check makes the
  in-process answer `Unknown`.
- **A device with no udev record** (containers, minimal images) or whose record lacks `ID_FS_UUID_ENC`: `findmnt` then
  asks `libblkid`, which probes the device and depends on read access.
- **Sources that are paths but not `/dev/` nodes, `host:/path` network sources, tag sources with `=`, names that
  exist in the working directory or in sysfs**: `libblkid` or libudev would look at them.
- **UUID text outside letters, digits, dot and dash** (spaces, `\xHH` escapes that decode to anything else, a NUL):
  libsmartcols may re-encode on output.
- Older kernels without `STATX_MNT_ID`: no mount ID, no in-process answer.

Rejected: `/dev/disk/by-uuid` as the source of the text (see "How `findmnt` 2.41 gets the text"). Reading
`ID_FS_UUID_SUB_ENC` (the Btrfs device UUID) instead of `ID_FS_UUID_ENC` is the natural mistake and the guard below
catches it: the repository path then records `87fa7a73-...` against `findmnt`'s `1bb3d23e-...`.

Guards shown failing before being kept (each break applied, tests run, restored):

- `ID_FS_UUID_SUB_ENC` in place of `ID_FS_UUID_ENC`: `udev_records_give_one_plain_uuid`,
  `in_process_uuids_equal_what_findmnt_prints` and `identities_equal_the_command_only_resolution` failed.
- Taking the last entry of stacked mounts instead of giving up: `the_decision_gives_up_on_unproven_shapes` and
  `in_process_uuids_equal_what_findmnt_prints` failed.
- Dropping the mount ID comparison: `the_decision_gives_up_on_unproven_shapes` failed (the differential tests cannot
  see it on this host, because the covered and the covering mount are on one filesystem).
- Dropping the working-directory existence check for pseudo sources: `pseudo_sources_need_no_lookup_to_succeed` failed.
- Forcing every path to `Unknown` (the old behavior): `a_pseudo_filesystem_starts_only_stat` and
  `a_device_with_a_udev_record_starts_nothing` failed. They count the commands through `linux_fs_id_with`.

Not done in this slice: replacing `stat --file-system --format=%i`; a permanent per-commit spawn budget test (no such
test exists in this tree yet; the lowered count is 3 for the `findmnt` row).
