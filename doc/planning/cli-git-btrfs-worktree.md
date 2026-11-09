# Cheap `git worktree add` on btrfs and overlayfs

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Status:
implemented in `package/git-policy/cli` (2026-11-02);
the sections "Snapshot flow (rung 1)" and
"Source selection" describe the original proposal,
and "Implemented flow and what superseded the proposal" says what the
code does now.
The measurements and the owner's decisions in this record stand.
Scope:
`package/git-policy/cli` (crate `monochromatic-git-policy-cli`),
Linux only.
The proposal changed no product code;
the implementation is on the branch `btrfs-worktree-ladder` and its successors.
The measurements ran on throwaway copies under `/var/home/user/.cache/agent-btrfs-wt` (same btrfs filesystem and mount
as the repository;
`${HOME}/temp` is a different filesystem,
so a reflink or snapshot from the repository into it fails
with `Invalid cross-device link`).

## Owner words and decisions that bound this design

- 2026-11-01,
  about the policy that new branches use `git worktree add -b`:
  "yes but that's intended and I also intended for this to not be so heavy because we can take advantage of btrfs."
- 2026-11-01:
  "snapshotting isn't possible since the repo root isn't a btrfs subvolume" followed by "so make it one.
  Make the wrapper gracefully degrade.
  Make doctor say this can help with performance."
  The snapshot route is decided;
  this document designs it and measures what it buys.
- 2026-11-01:
  the doctor should also provide the command to set the shown mount option,
  so the person can run it.
- 2026-11-01:
  "Properly support overlayfs too."
- Standing rules used here:
  "Invoke real git as rarely as possible.";
  "embed instead of spawning"
  (the wrapper never runs a third-party program);
  "Use as few locks as possible.";
  Linux only;
  "Fix even pre-existing issues of the pkgs you own."
- The owner migrates this machine's main checkout into a subvolume separately.
  The procedure was proven on a throwaway (the script is `doc/handover/cli-git-subvolume-migration-script.md`)
  and is summarized in "Migration of the main checkout" here;
  the migration itself was done on 2026-11-01.

## Summary

- Today `git worktree add` through the wrapper has two costs:
  real Git's checkout of tracked files,
  and the wrapper's
  copy of the creating worktree's ignored state.
  On a repository like this one with an ignored tree of about 72 thousand entries,
  the median was 28.35 s,
  of which
  the tracked checkout is about 2.1 to 2.7 s.
  With no ignored state the median was 2.1 to 2.4 s.
- A btrfs snapshot of a source worktree that is a subvolume,
  then a two-tree update of only the differing paths,
  took a median of 0.35 s to 1.09 s (0.12 s to 1.42 s range) including the ignored tree,
  spawns no more than
  six processes in the prototype,
  and leaves the next `git status` at 0.08 s to 0.10 s.
  It shares every data block with the source (exclusive bytes 0,
  or 3.9 MB for a target 100 commits older).
- Reflinking each tracked file from a clean source (option B) shares the data blocks too but did not make the
  creation faster on this host (median 2.1 s against 2.1 s to 2.4 s);
  it is a fallback for when no snapshot is possible.
- Recommended order of the degradation ladder:
  snapshot,
  then per-file reflink,
  then Git's own checkout.
  Each rung falls through silently and correctly when its precondition fails,
  and never fails the command.
- The main checkout as a subvolume is not by itself a good snapshot source.
  A snapshot of a root carries `.git` and every ignored tree.
  Linked worktrees that are themselves subvolumes are good sources,
  and the wrapper can make every new worktree a
  subvolume at no cost,
  so the first rung bootstraps itself.
- Overlayfs:
  detection and the cheapest operation are settled and measured in a container;
  overlayfs as a mechanism for
  a new worktree is not usable here without root,
  because an unprivileged overlay mount is private to its mount
  namespace.

## Facts

### This machine

- Mount,
  read from `/proc/self/mounts` at the end of the session:
  `/dev/dm-1 /var/home btrfs rw,seclabel,relatime,compress=zstd:1,ssd,discard=async,space_cache=v2,
  user_subvol_rm_allowed,subvolid=257,subvol=/home`.
- At the start of the session the same mount had neither `compress=zstd:1` nor `user_subvol_rm_allowed`
  (`findmnt --target /var/home` at 15:56).
  The coordinator ran `sudo mount --options remount,user_subvol_rm_allowed /var/home` during the session;
  the remount
  also picked up `compress=zstd:1`,
  which `/etc/fstab` already listed.
  The option is not persistent:
  the first mount of the filesystem comes from the kernel argument
  `rootflags=subvol=root` (`/proc/cmdline`),
  and a later mount of the same superblock does not change its options.
- The repository root `/var/home/user/Monochromatic` is an ordinary directory (inode 2595895);
  `/var/home` is inode 256.
- `${HOME}/temp` resolves to `/var/mnt/encrypted/temp`,
  another btrfs filesystem (`/dev/mapper/crypt_sda`,
  `compress=zstd:3`).
- The host is `rpm-ostree` plus `bootc 1.17.1` (Bazzite),
  kernel `7.2.11-ogc4.1.fc44`,
  Git 2.56.0.
- The repository has 20546 tracked files (49 executable,
  no symlinks,
  no submodules),
  1578 Git LFS files,
  and
  `.gitattributes` with `* text=auto eol=lf`.
  Its `.git/config` sets `core.fsmonitor false` and `extensions.worktreeconfig true`;
  the global config sets
  `core.fsmonitor true`,
  `core.untrackedcache true` and the `filter.lfs.*` process filter.

### Btrfs permissions (kernel source, fetched 2026-11-01)

- Subvolume creation is unrestricted.
  `fs/btrfs/ioctl.c`:
  "Subvolume creation is not restricted,
  but snapshots are limited to own subvolumes only";
  a snapshot needs `inode_owner_or_capable` on the source and returns `EPERM` otherwise;
  the source must be a
  subvolume root (`EINVAL` otherwise) and on the same filesystem (`EXDEV` otherwise).
  Measured as the unprivileged user:
  `btrfs subvolume create` and `btrfs subvolume snapshot` work.
- `BTRFS_IOC_SNAP_DESTROY` for a regular user needs the `user_subvol_rm_allowed` mount option,
  write and execute on
  the subvolume root,
  and that `rmdir(2)` would be allowed.
  The kernel comment:
  "Users who want to delete empty subvols should try rmdir(2)."
- `btrfs_rmdir` in `fs/btrfs/inode.c` deletes a subvolume when its root directory is empty
  (`inode->vfs_inode.i_size > BTRFS_EMPTY_DIR_SIZE` returns `ENOTEMPTY` first),
  with no mount option.
  So `git worktree remove` (which unlinks the files,
  then `rmdir`s) always works on a worktree that is a subvolume.
- Measured before the remount:
  `btrfs subvolume delete` of a non-empty subvolume failed with `EPERM`;
  after it,
  the same call succeeded and took 2 ms for a snapshot.
  Deleting a subvolume that contains a nested subvolume needs `--recursive`
  (`ENOTEMPTY` otherwise);
  unprivileged `--recursive` prints "Could not search B-tree" warnings but deleted the nested
  subvolumes.
- Hard links and `rename(2)` do not cross a subvolume boundary (`EXDEV`):
  measured with `ln` and in
  `fs/btrfs/inode.c` (`btrfs_rename`:
  "we only allow rename subvolume link between subvolumes").
  The wrapper's current ignored-copy stage is "a rename or a link" beside the destination,
  so a stage outside the
  destination subvolume could not be installed into it.
- `FICLONE` works across subvolumes of one filesystem (measured:
  `cp --reflink=always` between two subvolumes).
  `ioctl_ficlone(2)`:
  "Both files must reside within the same filesystem.";
  `EXDEV` otherwise.
- `libc` 0.2.191 has `FICLONE`,
  `BTRFS_SUPER_MAGIC` and `OVERLAYFS_SUPER_MAGIC`,
  and no `BTRFS_IOC_*` constants.
  `strace` of `btrfs subvolume snapshot` shows `ioctl(parent_fd, BTRFS_IOC_SNAP_CREATE_V2, {fd=source_fd, flags=0,
  name="b"})` and of `btrfs subvolume delete` `ioctl(parent_fd, BTRFS_IOC_SNAP_DESTROY, {name="b"})`.
  `include/uapi/linux/btrfs.h`:
  `BTRFS_IOCTL_MAGIC` is 0x94,
  `BTRFS_IOC_SNAP_CREATE_V2` is `_IOW(0x94, 23,
  struct btrfs_ioctl_vol_args_v2)`,
  `BTRFS_IOC_SUBVOL_CREATE_V2` is number 24,
  and the structure is 4096 bytes
  (`fd`,
  `transid`,
  `flags`,
  a 32-byte union,
  then a 4040-byte name).

### Git 2.56.0 behavior that matters (sources fetched from the `v2.56.0` tag)

- `builtin/worktree.c`:
  `git worktree add` runs `git branch` (for `-b`) and then `git reset --hard
  --no-recurse-submodules` as child processes (`checkout_worktree`),
  then the `post-checkout` hook.
  With `--no-checkout` neither the reset nor the hook runs.
  It copies the sparse-checkout patterns when the current worktree has sparse checkout enabled,
  and the filtered
  worktree config when `extensions.worktreeConfig` is set (this repository has it),
  before the checkout.
- `worktree.c` lines 119 to 120:
  the main worktree path is `strbuf_add_real_path` of the common directory with
  `/.git` stripped.
  So a symlinked `.git` or a symlinked `.claude/worktrees` changes recorded paths (measured,
  see the migration section).
- `statinfo.c` lines 91 to 97:
  `st_dev` is compared only under `USE_STDEV`,
  which the `Makefile` leaves undefined by
  default.
  So an index copied into a snapshot (same inode numbers,
  times and sizes,
  different `st_dev`) stays valid
  (measured:
  first `git status` in a snapshot worktree 0.08 s to 0.10 s).
- `Documentation/technical/racy-git`:
  entries whose modification time equals the index timestamp are re-hashed;
  Git zeroes the cached size of such entries when it writes the index.
  After a plain checkout,
  the first `git status` re-hashes the racy entries,
  which with Git LFS runs the clean
  filter:
  measured 0.88 s to 1.05 s median against 0.07 s to 0.10 s for options B and C.
- `Documentation/config/checkout.adoc`:
  `checkout.workers` set to less than one uses one worker per logical core.
  `technical/parallel-checkout`:
  entries that need external filters are not parallelized,
  but this repository's
  checkout was faster (see Measurements).
- `git worktree add` into an empty directory is accepted,
  and an empty subvolume is an empty directory:
  measured
  `btrfs subvolume create` then `git worktree add --no-checkout` then `git worktree remove --force`.
  A plain `git worktree add` creates an ordinary directory (inode 48936 in the probe),
  not a subvolume.

### Precedent (fetched 2026-11-01)

- pnpm `package-import-method`:
  values `auto`,
  `hardlink`,
  `clone-or-copy`,
  `copy`,
  `clone`;
  `clone` is "copy-on-write or reference link";
  `auto` tries hardlink before clone on Linux
  (<https://pnpm.io/settings/node-modules>).
- uv `link-mode`:
  default `clone` on macOS and Linux (<https://docs.astral.sh/uv/reference/settings/#link-mode>).
  Source `crates/uv-fs/src/link.rs` (shallow clone 2026-11-01) reflinks per file with `reflink-copy`,
  preserves
  permissions itself because `ioctl_ficlone` clones data only,
  and falls back per file:
  to a temporary file,
  then to
  hardlink or copy,
  remembering the failure for the later files.
- Jujutsu (`jj-vcs/jj`) and gitoxide (`GitoxideLabs/gitoxide`,
  including `gix-worktree-state`):
  a search of every `.rs`
  file for `reflink`,
  `FICLONE` and `clonefile` found none;
  both write files from objects.
  The same search found matches in uv,
  so the search ran.
- Sapling's documentation page on differences from Git does not mention reflinks or copy-on-write (not read further).
- `git-cow-worktree` (<https://engineered.at/articles/copy-on-write-git-worktrees>) runs `git worktree add --no-checkout`,
  reflinks tracked files from a similar worktree,
  and lets Git finish the checkout;
  the summary says it "is slower than
  native `git worktree` with many files".
  That agrees with the option B measurement here.
- Git itself has no reflink option for checkout;
  `git worktree` documentation does not mention copy-on-write.
- `cp --reflink=always` is the coreutils equivalent (9.10 here).

### The wrapper today

- `package/git-policy/cli/src/native/worktree_copy_stage.rs` copies selected ignored entries one by one with `FICLONE`
  (`try_clone`) and falls back to a byte copy;
  `binary_support.rs` forces `cp --reflink=never` for its own fixture.
- The creating worktree is the copy source;
  a main checkout is never copied as a whole ("its ignored trees are the
  repository's heaviest",
  `worktree_copy_registry.rs`),
  only named repository paths.
- Git's checkout is not changed:
  the wrapper forwards `git worktree add`,
  then observes the created worktrees and runs
  the copy under a per-repository settlement lock with a journal and recovery.
- In-process reads exist:
  `repo_read_index.rs` (`parse_index`),
  `repo_read_tree.rs` (`flatten_tree`,
  `parse_commit`),
  `repo_read_odb.rs`,
  `repo_read_pack.rs`,
  `repo_read_config_*`;
  there is no index writer and no blob-to-file writer.
- `doctor.rs`:
  `git cli-git doctor` prints findings and notes on standard error and keeps its exit status (0,
  1 or 2)
  for hooks;
  `filesystem_identity*.rs` already read `/proc/self/mountinfo`.
- Another agent is moving the ignored copy into the resident service (`service_copy*.rs`).
  This design adds a step before the copy and removes the need for it when the snapshot carries the ignored state.

## Assumptions and limits

- The host load was high throughout (one-minute load average between 8 and 58,
  median about 33 on a 28-thread machine);
  I did not wait for it to fall.
  Every measurement ran inside `systemd-run --user --scope` with `CPUQuota=400%`,
  `MemoryMax=8G`,
  `TasksMax=512`,
  `IOWeight=50`;
  the benchmark driver ran as a unit with `CPUQuota=500%`,
  `MemoryMax=10G`,
  `TasksMax=1024`,
  `IOWeight=50`.
- Arms ran interleaved in shuffled order,
  five rounds per arm (four for the ignored-state arms),
  after one discarded
  warm-up of each family.
  The page cache was warm;
  a cold first checkout took 36 s to 40 s.
- Same-binary controls:
  `A1` and `A2` are the same command,
  `B0` and `B0b` the same prototype,
  `C0` and `C0b` the same
  prototype.
  Their medians differ by 0.30 s,
  0.07 s and 0.50 s,
  so differences smaller than that are noise.
- The clone is a `git clone --local` of the repository at one commit (20546 files);
  its linked-worktree list and
  ignored state differ from the real checkout.
  The first benchmark run used the global `core.fsmonitor true` by mistake
  (the clone lacked the repository's `core.fsmonitor false`),
  which starts a `git fsmonitor--daemon` per worktree;
  it was discarded and rerun with `core.fsmonitor false` and `extensions.worktreeConfig true` as in the repository.
- `compress=zstd:1` became active during the session.
  Data is written compressed;
  `btrfs filesystem du` reports
  logical bytes,
  so the exclusive and shared columns are comparable,
  and checkout wall times include compression.
- Options B and C are prototypes in Node (`proto.ts`):
  `fs.copyFileSync` with `COPYFILE_FICLONE_FORCE`,
  a hand-written
  version 2 index writer,
  `btrfs subvolume snapshot` as a child process,
  and `git` for the two-tree update.
  A Rust implementation would replace the `btrfs` child with the ioctl and most `git` children with in-process reads.
  The prototype's process counts are therefore upper bounds.
- The snapshot source in the benchmark is a linked worktree (a `.git` file).
  A snapshot of a main checkout carries a
  `.git` directory;
  that case was measured only for its pieces (see "Snapshot source").
- The wrapped commit path (scoped `git commit`) goes through the production commit service and was not run.
- Container and user-namespace overlay numbers are single runs.

## Measurements

All values are wall seconds as median (minimum to maximum) unless stated.
"Status" is the first `git status --short` after creation;
the second one took 0.01 s to 0.05 s in every arm.

### Tracked files only (source worktree has no ignored files), five rounds

- `A1` real `git worktree add -b` (control `A2`):
  2.40 (2.06 to 6.44) and 2.10 (2.06 to 2.81).
  CPU 1.1 s user and 1.4 s system,
  494 MB peak memory,
  30 processes (27 `git`,
  2 `git-lfs`,
  one shell,
  the LFS hook).
  Status 1.05 (0.23 to 1.16) and 1.01 (0.91 to 1.21).
  Bytes:
  total 830279680,
  exclusive 830279680,
  shared 0.
  Removal with `git worktree remove --force`:
  0.75 (0.65 to 2.10).
- `Aw` the same command through the installed wrapper (no ignored files to copy):
  2.11 (2.04 to 3.01),
  23 processes,
  status 0.88 (0.14 to 1.28).
  The wrapper adds no measurable cost when there is nothing to copy.
- `Apar` real Git with `-c checkout.workers=-1`:
  1.44 (1.35 to 1.61),
  63 processes,
  188 MB,
  status 1.12 (0.19 to 2.66).
  About 35 percent faster than `A1` and `A2` with a tight spread,
  at the price of one worker process per core.
- `B0` reflink each tracked file from a clean source,
  own index writer,
  source timestamps kept (control `B0b`):
  2.15 (1.44 to 2.60) and 2.08 (1.33 to 3.12).
  CPU 0.39 s user and 1.8 s system,
  127 MB,
  four `git` processes plus the
  prototype itself.
  Status 0.07 (0.06 to 0.08).
  Bytes:
  total 842739712,
  exclusive 0,
  shared 842739712.
  `B1` (target 100 commits older,
  298 differing paths,
  207 files written from objects by `git checkout-index`):
  2.24 (1.59 to 2.45),
  status 0.09,
  exclusive 3964928.
- `C0` snapshot of a subvolume worktree,
  then index copy (control `C0b`):
  0.35 (0.12 to 0.85) and 0.85 (0.16 to 1.03).
  CPU 0.05 s user and 0.3 s to 0.8 s system,
  81 MB,
  five `git` processes plus `btrfs` and the prototype.
  Status 0.08 (0.07 to 0.09) and 0.09 (0.08 to 0.13).
  Bytes:
  total 830279680,
  exclusive 0,
  set shared 665608192.
  `C1` (target 100 commits older,
  two-tree update with `git read-tree -m -u`):
  0.47 (0.19 to 1.05),
  six `git`
  processes,
  status 0.10,
  exclusive 3964928.
  Removal with `git worktree remove --force`:
  0.72 (0.68 to 0.75);
  with `btrfs subvolume delete` then
  `git worktree prune`:
  0.04 (0.03 to 0.12).
- Content check:
  `diff --recursive --brief --exclude=.git` of every arm against a reference checkout of the same
  commit printed nothing,
  and the first `git status` showed no change,
  in every arm.

### With an ignored tree in the creating worktree (72333 ignored entries, four rounds)

- `Ai` real `git worktree add` from that worktree (copies no ignored files):
  2.73 (1.94 to 3.57).
- `Awi` the installed wrapper from that worktree (reflinks the ignored tree):
  28.35 (25.87 to 125.21),
  22 processes,
  12.99 s system CPU,
  status 1.23,
  removal 3.95.
- `Ci` snapshot,
  which carries the ignored tree:
  1.09 (0.43 to 1.42),
  status 0.09,
  removal by file unlinking 3.64
  (by subvolume deletion about 0.04).
- The wrapper's per-entry ignored copy costs about 350 microseconds per entry here,
  so the ignored tree,
  not the
  tracked checkout,
  is the dominant cost.
  The owner's 7.88 s is consistent with a smaller ignored tree.

### Snapshot of a whole root (a main checkout that is a subvolume), probe of 20546 files

- `btrfs subvolume snapshot` of the migrated root:
  1.08 s wall (0.63 s system);
  nested worktree subvolumes appear as
  empty directories with inode number 2;
  deleting the snapshot took 2 ms.
- The snapshot carries `.git` in full:
  removing `.git` (4739 entries) from the snapshot took 0.12 s,
  removing an
  18083-entry ignored tree 0.41 s (about 23 microseconds per entry).
  For millions of ignored entries that is minutes.

## Design

### Degradation ladder

Applies to `git worktree add` forms the wrapper recognizes (a new branch with `-b`,
or a commit-ish;
`--detach`;
`--quiet`;
`--lock`).
Any option it does not know,
`--orphan`,
`--no-checkout`,
`--force`,
`--track` and every `-B` form
go straight to rung 3.

- Rung 1,
  snapshot:
  needs btrfs,
  a clean source worktree that is a subvolume root on the destination's filesystem,
  and
  a destination parent on the same filesystem.
- Rung 2,
  reflink:
  needs a filesystem with `FICLONE` (btrfs,
  XFS with reflink,
  overlayfs over either) and a clean
  source worktree on the same filesystem.
  The measurement says it saves disk,
  not time,
  so it is optional.
- Rung 3,
  real Git:
  today's behavior,
  plus the existing ignored copy.
- (Changed 2026-11-03T02:57:19-05:00:
  the settlement lock no longer exists;
  no rung takes a lock,
  and each add's stage,
  claim and journal belong
  to that add alone.
  See `package/git-policy/cli/SPEC.md`.)
  Every rung runs inside the existing per-repository settlement
  lock and journal.
  A failure before the point of
  no return (the destination path exists with content) rolls the rung back and tries the next;
  a failure after it is
  reported as today's copy failures are.
  No rung changes the exit status of the command the person ran when a lower rung
  can complete it.
- Precondition probes are cheap and reuse what the wrapper reads anyway:
  `statfs` magic for the destination parent,
  inode 256 of the source root,
  `/proc/self/mountinfo` for the filesystem and its options.

### Source selection

- The creating worktree (the one Git was invoked in) is the first candidate,
  as the ignored copy does today.
- A main checkout is never the source of its whole ignored state;
  today only named paths are copied.
  A snapshot cannot
  copy named paths,
  so from a main checkout the candidates are the most recently used clean linked worktree that is a
  subvolume,
  then rung 3.
  This is an owner decision (see "Decisions").
- The source must be clean,
  checked without spawning Git where possible:
  no `index.lock`,
  `MERGE_HEAD`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `BISECT_LOG`,
  `rebase-merge`,
  `rebase-apply` or
  `sequencer` in its administrative directory;
  its index equals the tree of its `HEAD` (in-process `parse_index` and
  `flatten_tree`);
  every entry's size,
  modification time and inode equal a fresh `lstat`;
  and no untracked file that is
  not ignored.
  The last check needs Git's ignore rules;
  one `git ls-files --others --exclude-standard` (0.5 s on a clean
  worktree) is the only spawn,
  and the wrapper already runs the ignored listing for the copy.
- After the snapshot the same stat comparison runs against the snapshot's own files,
  so a source that changed during the
  snapshot is detected on the copy and the snapshot is deleted (no time-of-check gap on the source).

### Snapshot flow (rung 1)

This is the proposal as written on 2026-11-01.
It is kept as the reasoning the measurements rest on;
where the
implementation differs,
"Implemented flow and what superseded the proposal" is the description to trust.

- Real Git runs once with `--no-checkout`,
  `--lock` and the person's arguments,
  inside the settlement lock (changed 2026-11-03T02:57:19-05:00:
  there is no settlement lock any more;
  Git runs with no lock around it).
  This keeps branch
  name validation,
  `post-checkout`-less admin creation,
  worktree config copying and every Git message;
  it spawns
  `git worktree add` and its child `git branch` (two processes).
- The destination path now holds an empty directory and a `.git` file.
  The wrapper removes both,
  snapshots the source
  to a stage name beside the destination (`.cli-git-worktree-copy-<id>`,
  the reserved prefix),
  writes the `.git` file
  for the new administrative directory,
  copies the source index into the new administrative directory (a reflink),
  and renames the stage to the final path.
  Rename within one parent subvolume works (measured).
  Taking the snapshot first at the stage name,
  not at the final name,
  keeps the final path absent or complete.
  Between removing Git's empty directory and the rename,
  the destination path does not exist,
  and a concurrent
  `git worktree prune` (another session,
  or `gc --auto`,
  which runs it) would delete the new administrative directory
  because its `gitdir` points at a missing location;
  the probe printed exactly that message for a moved worktree.
  The `--lock` flag prevents it.
  The wrapper removes the `locked` file after the final rename,
  and keeps it when the
  person passed `--lock` themselves.
  Recovery must remove `locked` or the orphan administrative directory itself,
  because `git worktree prune` skips locked entries.
- If the target tree equals the source tree nothing else is needed (measured 0.35 s).
  Otherwise the differing paths are
  applied with a two-tree update.
  First version:
  one `git read-tree -m -u <source-tree> <target-commit>` in the new
  worktree (0.47 s for 298 paths).
  Later:
  in-process,
  with the missing blob-to-file writer and index writer.
- Fall back to rung 3 without error if the source or target has any of:
  sparse checkout enabled in the creating
  worktree (Git copies the patterns),
  a gitlink entry (submodule working directories in a snapshot would point at the
  source's module directory),
  a differing `.gitattributes` blob between the source tree and the target tree,
  or
  differing `core.autocrlf`,
  `core.eol`,
  `core.safecrlf`,
  `filter.*` or `core.symlinks` between the two worktrees'
  effective config (unchanged paths keep the source's bytes,
  so conversion rules must agree),
  a `post-checkout`
  hook that the person configured,
  or a `-c` option on the command line that Git would pass to the checkout.
- Correctness edges and how each rung treats them:
  - Modes and symlinks:
    a snapshot keeps them exactly;
    the two-tree update and option B create files with the
    index mode and umask,
    as Git does.
    This repository has no symlinks,
    so symlink cases need a fixture (not measured).
  - Git LFS:
    the source holds smudged bytes;
    a snapshot or reflink carries them;
    changed paths go through Git's filter.
    Measured:
    1578 LFS files,
    content equal,
    status clean.
  - `core.autocrlf` and `* text=auto eol=lf`:
    see the attribute and config conditions in Snapshot flow.
  - Dirty or mid-operation source:
    not eligible,
    fall to the next source or rung.
  - Files the source modified:
    not eligible (stat comparison),
    never copied.
  - Ignored selection:
    a snapshot copies everything in the source,
    including entries the current copy excludes
    (excluded roots,
    the reserved stage prefix,
    sockets,
    FIFOs).
    Owner decision,
    see "Decisions".
  - Crash at any instruction (normal here):
    the journal records stage creation,
    admin creation,
    rename and index
    copy.
    Recovery deletes a leftover stage subvolume,
    and a leftover administrative directory with no `.git` file is
    pruned by Git (`git worktree prune` reported "gitdir file points to non-existent location" in the probe).
    A half-written index cannot occur because the index is copied by reflink or renamed from `index.lock`.
  - Removal:
    `git worktree remove` works unchanged on a worktree that is a subvolume (measured 0.72 s).
    It is a
    subvolume delete of 0.04 s if the wrapper deletes the subvolume itself,
    which needs `user_subvol_rm_allowed` and
    should only follow Git's own cleanliness check,
    so it is optional (see "Decisions").
    `git worktree prune` only
    touches administrative directories.

### Creating new worktrees as subvolumes (bootstrap)

- When the destination parent is on btrfs,
  the wrapper creates the destination as an empty subvolume
  (`BTRFS_IOC_SUBVOL_CREATE_V2`,
  unrestricted) before real Git runs,
  so the plain checkout of rung 3 fills a subvolume.
  Measured:
  Git accepts the empty subvolume and `git worktree remove` removes it.
- Every worktree the wrapper creates is then a valid source for the next snapshot,
  with no owner action and no
  migration.
  Failure to create the subvolume falls back to the ordinary directory Git makes.
- Cost:
  the nested subvolume stays an empty directory in a snapshot of its parent,
  which is wanted.
  Tools that stay on one filesystem by `st_dev` (for example `rsync -x`,
  `du -x`,
  `find -xdev`,
  a backup policy
  such as Kopia's "one file system") treat each subvolume as a separate device.
  Whether the machine's backup sets
  that policy is not verified;
  check before turning this on.

### Migration of the main checkout (measured separately)

- Result in short (full numbers in the handover):
  the procedure creates a subvolume beside the root,
  nested
  subvolumes for linked worktrees,
  a reflink `cp`,
  `rsync --delete` deltas,
  then under `.git/index.lock` a last delta,
  an atomic `mv --exchange`,
  a catch-up and an index stat rewrite.
  68149 entries took copy 8.46 s and three `rsync`
  passes of about 2.4 s each;
  the exchange took under 0.01 s;
  the root is inode 256 and shares 2.89 GiB with the old
  tree (exclusive 0.00 B).
  One real worktree (`integ`,
  22921 entries) copied in 1.94 s.
- `.git` cannot stay behind a symlink:
  with a symlinked `.git` the main worktree in `git worktree list` became the
  store directory,
  and with a symlinked `.claude/worktrees` a new worktree was recorded under the target path.
  The copy window is closed instead by the lock and the deltas.
- A reflink copy gives every file a new inode and change time,
  so the first `git status` in the new root re-hashed all
  files (22.1 s);
  rewriting the index stat fields takes about 0.2 s per index and the first status then took 0.17 s.
- A root snapshot is a poor source of a worktree:
  it carries `.git` and all ignored trees (see "Measurements").
  Making `.git` a nested subvolume would leave an empty stub in the snapshot;
  that was not evaluated and is not
  recommended before the stub behaviour of `git` operations with a nested-subvolume `.git` is tested.

### Doctor

`git cli-git doctor` gains a filesystem section on standard error,
informational,
never changing the exit status.
It reads only `statfs`,
`stat` inode numbers,
`/proc/self/mountinfo` and `/run/ostree-booted`;
it spawns nothing.

- Neither btrfs nor overlayfs:
  one line saying worktree creation uses Git's checkout and why nothing faster applies.
- Btrfs,
  repository root not a subvolume (inode not 256):
  "Making `<root>` a btrfs subvolume lets `git cli-git` create
  `git worktree add` worktrees from a snapshot:
  measured median 0.35 s to 1.09 s (range 0.12 s to 1.42 s) against
  2.1 s to 2.7 s for Git's checkout without ignored files and 28.35 s with a 72 thousand entry ignored tree,
  and
  with no extra disk (exclusive bytes 0)."
  Then the exact steps for this host,
  taken from the proven procedure:
  stop agent sessions that have a working directory in the checkout;
  run the dry run
  `node <migrate.ts> --root <root>`;
  then the bounded migration

  ```sh
  # file: doctor output, migration step
  systemd-run --user --scope --property=IOWeight=10 --property=CPUQuota=200% --property=MemoryMax=2G \
    --property=TasksMax=64 node <migrate.ts> --root <root> --apply
  ```

  restart sessions;
  remove `<root>.before-subvolume` after checking.
  The doctor prints the path of the script
  the repository ships (an owner decision:
  where it lives,
  or whether it becomes a `git cli-git` subcommand).
- Btrfs,
  `user_subvol_rm_allowed` absent from the super options of the repository's mount:
  "worktree removal is
  fast only with this mount option;
  it already works without it,
  file by file."
  Runtime command:
  `sudo mount --options remount,user_subvol_rm_allowed <mountpoint>` (verified on this host).
  Persistent on an ostree or bootc host (`/run/ostree-booted` exists,
  kernel command line has `rootflags=`):
  the printed primary command is `sudo rpm-ostree kargs --editor`,
  with the instruction to append
  `,user_subvol_rm_allowed` to the `rootflags=subvol=root` line,
  save,
  and reboot.
  The one-line alternate is `sudo rpm-ostree kargs --replace=rootflags=subvol=root=subvol=root,user_subvol_rm_allowed`;
  its value contains `=`,
  which is the parse in doubt,
  so it is secondary.
  `rpm-ostree kargs --help` on this host lists `--editor` and `--replace=KEY=VALUE=NEWVALUE`;
  neither command was run
  (another `rpm-ostree` transaction was active) and both are unverified.
  bootc documents `kargs.d` only for image builds and says it offers no API for local kernel arguments
  (<https://bootc.dev/bootc/building/bootc-kernel-arguments.7.html>).
  Persistent on a conventional host:
  add `user_subvol_rm_allowed` to every `/etc/fstab` btrfs entry that shares the
  filesystem UUID and reboot.
  Measured here:
  fstab listed `compress=zstd:1` for `/home`,
  yet the first mount decided
  the options until a remount.
  The doctor shows the one line that matches what it detected,
  and re-reading `/proc/self/mountinfo` confirms it.
- Overlayfs (see the Overlayfs section):
  report the detection,
  which operation is cheap,
  and the fallback.
- A nested-subvolume note when worktrees exist as ordinary directories:
  they stay ordinary;
  new ones become subvolumes.

### Overlayfs

Reading 1,
the repository lives on overlayfs (container root filesystems):

- Detect:
  `statfs` `f_type` 0x794c7630 (`OVERLAYFS_SUPER_MAGIC`,
  measured in a container).
  The mount options in `/proc/self/mountinfo` list `lowerdir`,
  `upperdir`,
  `userxattr` and others;
  inside a rootless
  container they name host paths that cannot be `statfs`ed,
  so the upper filesystem is not readable from inside.
  Do not infer it;
  probe.
- `Documentation/filesystems/overlayfs.rst` and `fs/overlayfs/file.c` (fetched 2026-11-01):
  `ovl_remap_file_range`
  passes `FICLONE` to `vfs_clone_file_range` on the real files (`ovl_copyfile`,
  operation `OVL_CLONE`);
  a lower-layer
  source is read in place (no copy-up) and the destination is copied up or created in the upper layer,
  so a clone
  works when lower and upper are on the same filesystem.
  `copy_file_range` passes through the same way.
- Measured in `podman run --cpus 4 --memory 6g --pids-limit 1024 --blkio-weight 50` (rootless,
  overlay storage with
  `Backing Filesystem: btrfs`,
  mount options `redirect_dir=nofollow,uuid=on,userxattr`,
  image
  `docker.io/cachyos/cachyos:latest`,
  Git 2.56.0,
  LFS filters set to `cat` because `git-lfs` is absent):
  - `FICLONE` from a bind mount into the overlay root failed with `Invalid cross-device link`
    (the kernel compares superblocks of the two files:
    the overlay's,
    not the real one's).
    So a source outside the overlay cannot be cloned in;
    the wrapper's source and destination must both be inside it.
  - `cp --reflink=always` of the 20546 tracked files,
    upper to upper:
    1.36 s;
    lower to upper:
    1.55 s (no `EXDEV`);
    byte copy:
    1.85 s and 1.73 s.
    The lower-to-upper clone succeeded,
    which proves the layers share a filesystem.
    Free-space deltas were too noisy (other sessions write) to quantify sharing.
  - Git's own checkout on the overlay:
    `git reset --hard` 15.27 s and `git worktree add` 9.55 s and 7.65 s,
    against
    about 2.1 s on btrfs directly with a different CPU cap.
    These are single runs and not like for like,
    but the
    direction is that overlay makes the plain checkout several times slower.
  - `git status` printed "untracked cache is disabled on this system or location".
- Consequence:
  on overlayfs the cheapest operation is the existing per-file `FICLONE`,
  with rung 3 as the fallback;
  rung 1 does not apply (a snapshot needs btrfs semantics,
  and the layer's btrfs is not addressable from inside).
  Rung 2 (reflink from a clean source) applies and is the useful rung there because the checkout is slow.
  The ladder probes by attempting `FICLONE` on a temporary file pair and caches the result per mount.

Reading 2,
overlayfs as the mechanism of a cheap worktree (shared lower layer,
per-worktree upper layer):

- Measured:
  `unshare --user --map-root-user --mount` followed by `mount -t overlay ... -o userxattr` works without
  privileges,
  and `FICLONE` inside it works from a lower file to an upper file.
- The mount exists only in that mount namespace.
  From outside,
  the mount point listed zero entries while the upper
  directory held the files written inside.
  Other sessions,
  editors and the commit service would see an empty
  directory,
  so a worktree cannot be a user-namespace overlay mount.
- Crash behavior:
  the mount disappears with the last process in the namespace,
  leaving an empty-looking worktree;
  `git worktree prune` would then treat it as present (the directory exists) and nothing would clean the upper layer.
- Inside the namespace,
  the first `git status` over a lower layer that is a clean worktree took about 12 s
  (a single run)
  because the user namespace maps the owner to uid 0 and the index records uid 1000,
  so every entry looks changed;
  the second status took 0.036 s.
- Nothing usable remains that is both unprivileged and embedded:
  `fuse-overlayfs` is an external program
  (excluded),
  mounting FUSE without `fusermount3` is again private to a namespace,
  and a FUSE server inside the
  resident service would be a new filesystem implementation with its own crash and permission surface.
  A system mount needs root (a systemd mount unit installed once with `sudo`) and was not evaluated.
  Overlayfs as a mechanism is therefore not proposed;
  snapshots cover the same need on btrfs.

## Options and ranking

- Option C,
  snapshot of a subvolume source plus a two-tree update (decided by the owner):
  fastest by far with an
  ignored tree (1.09 s against 28.35 s),
  cheap even without one (0.35 s to 0.85 s against 2.1 s to 2.4 s),
  no extra
  disk,
  cheap next status,
  five to six processes in the prototype.
  Cons:
  btrfs only,
  a source that is a subvolume,
  carries all ignored state of the source,
  needs hand-defined ioctls,
  a clean-source check,
  and subvolume-aware cleanup.
- Option B,
  reflink each tracked file:
  shares disk (exclusive 0),
  cheap next status (0.07 s),
  no subvolume or mount
  option needed,
  works on XFS and overlayfs.
  Cons:
  no wall-time gain here (2.1 s),
  needs an index writer and
  per-file conversion checks,
  fewer than five processes but 20 thousand syscalls.
- Option A,
  today's checkout:
  nothing to build,
  always correct.
  Cons:
  2.1 s to 2.7 s plus 350 microseconds per ignored
  entry,
  792 MiB exclusive per worktree,
  a racy first status of 0.9 s to 1.1 s,
  23 to 30 processes.
- A tuning of A,
  `checkout.workers`:
  1.44 s with one flag,
  tight spread.
  Cons:
  63 processes,
  which runs against
  "invoke real git as rarely as possible" even though it is one invocation.
- Overlay-based worktrees:
  not viable unprivileged (see the Overlayfs section).

Ranking:
C > B > A.
C ranks higher than B because it is 2 to 7 times faster without ignored state and 26 times faster with it,
the only
option that removes the ignored-copy cost,
and it shares disk equally.
B ranks higher than A because it shares all data blocks,
leaves the next status 12 times cheaper,
and is the only
accelerated rung on filesystems with reflink but no subvolumes (XFS,
overlayfs),
at no loss of wall time.
Within A,
the `checkout.workers` flag is optional polish and ranks last among the changes because of the process count.

## Dependencies

- No new crate.
  `libc` (already a dependency) supplies `ioctl`,
  `FICLONE`,
  `statfs`,
  `BTRFS_SUPER_MAGIC` and
  `OVERLAYFS_SUPER_MAGIC`.
- Hand-defined:
  `#[repr(C)] struct btrfs_ioctl_vol_args_v2` (4096 bytes),
  request numbers
  `BTRFS_IOC_SNAP_CREATE_V2` (`_IOW(0x94, 23, 4096)`),
  `BTRFS_IOC_SUBVOL_CREATE_V2` (24) and
  `BTRFS_IOC_SNAP_DESTROY` (`_IOW(0x94, 15, 4096)`,
  name only),
  proven by one real call each in a test on a btrfs
  fixture (the `strace` evidence in Facts shows the argument shapes).
  `rustix` 1.1.x is in the local registry but is not a
  dependency of this crate and is not needed.
- Missing in-process pieces:
  an index writer (stat fields only,
  version 2,
  preserving extensions except the untracked
  cache and the fsmonitor token),
  a blob-to-file writer for the two-tree update (conversion filters make this the
  risky one,
  so the first version keeps `git read-tree -m -u`),
  and a cleanliness check built from `parse_index`.
- Tools:
  none spawned.
  `btrfs`,
  `rsync`,
  `cp`,
  `mv --exchange` are used only by the migration script and the manual
  steps the doctor prints,
  never by the wrapper at run time.

## Decisions for the owner

- Where the migration script lives and whether it becomes a wrapper subcommand (the wrapper may not spawn `rsync` or
  `cp`,
  so a native subcommand would be a reimplementation of the copy and the deltas).
- Snapshot source policy:
  snapshot only from a linked worktree that is a subvolume (recommended),
  or also from the
  main checkout after pruning `.git` and ignored trees.
  A snapshot carries all of a worktree's ignored state;
  today's copy carries the selected roots and never the main
  checkout's.
  Either accept "everything the source has" for linked-worktree sources,
  or add a prune step (cheap only
  where the unwanted trees are nested subvolumes).
- Whether the wrapper creates every new worktree as an empty subvolume (recommended;
  needs a check that the backup
  policy does not skip other devices),
  and whether it deletes the subvolume itself on removal when
  `user_subvol_rm_allowed` is active (saves 0.7 s to 3.6 s,
  needs the mount option on every machine).
- Whether to persist `user_subvol_rm_allowed` through the kernel argument on this host (affects every mount of the
  filesystem and needs a reboot;
  the command is unverified).
- Whether to pass `-c checkout.workers=-1` on rung 3.
- Whether rung 2 ships (it is only useful on XFS and overlayfs,
  and the fallback there is the slow checkout).

## What I could not measure

- A wrapped scoped commit and any wrapped `git worktree add` against the real repository (production service,
  real worktree list).
- The real repository's entry count and a full migration;
  the extrapolation is in the handover.
- Symlink,
  submodule,
  sparse-checkout,
  `core.autocrlf` and `post-checkout` behavior with fixtures (the repository has no
  symlinks or submodules);
  these edges are specified in Snapshot flow from the Git sources,
  not measured.
- Snapshot from a main checkout with `.git` as a directory end to end (pieces only),
  and `.git` as a nested subvolume.
- A cold-cache comparison;
  a first checkout of the clone took 36 s to 40 s while the warm medians are 2.1 s to 2.4 s.
- Overlay timings with spread,
  overlayfs on a host mount rather than in a container,
  and a system overlay mount.
- Kopia or any backup policy's treatment of subvolume boundaries.

## Implemented flow and what superseded the proposal

Historical in "Snapshot flow (rung 1)":
the removal of Git's destination before the snapshot,
the temporary worktree
lock,
the claim that a reflink copy of the index is enough,
and the recovery by pruning.
What the code does:

- Real Git runs once with `--no-checkout`.
  Before it runs,
  the wrapper records the add durably (a stage claim and a
  record of the planned destination under the repository's journal root).
  That record authorizes nothing.
- After Git returns,
  the add is bound:
  the destination's `.git` pointer must name,
  by a path that resolves without any
  symbolic link,
  an administrative directory directly below this repository's `worktrees`,
  that directory's `gitdir` must name the pointer back,
  and the identities (device,
  inode,
  birth time) of both directories and the pointer bytes are written to the record with both directories held open.
  A pair that cannot be proven ends the ladder for this add and leaves what Git made as it is.
- The wrapper never deletes or unregisters a worktree or a registration,
  and recovery never deletes a stage.
  The binding
  is not authority to remove:
  it is what the fast rungs and the fallback checkout verify (identities at their paths,
  the
  pointer bytes,
  the registration's back-link) before every write,
  and what the report of an interrupted add names.
  The only
  deletions are of objects the running process made:
  the stage it took and never exchanged (removed only while the name
  still holds the object it made) and its own `index.lock` (while the name still holds the file it made,
  as Git does).
  The entries its clones created are not removed:
  no way to check a name and unlink it in one step exists,
  a private
  quarantine directory is no protection against a process of the same user,
  so a failed clone run leaves them,
  the checkout
  that completes the worktree overwrites the tracked ones,
  and the report counts the rest.
  Anything else,
  including every
  doubt,
  is kept and reported.
- The snapshot rung takes the snapshot at a stage name beside the destination,
  removes the stubs below a held handle of
  the stage without following a link,
  writes Git's own `.git` pointer,
  installs the index bytes that were proven clean
  (never a second read of the file) through the administrative directory's handle under `index.lock`,
  publishing the open
  file itself under the name of the index with a link that cannot replace anything and releasing the lock name only while it
  still holds that file,
  applies the tree difference with one `git read-tree -m -u`,
  and swaps stage and destination with
  `RENAME_EXCHANGE` after checking that the destination is still the directory Git made.
  A swap that finds another
  directory is undone;
  when it cannot be undone the stage name is never destroyed.
  The displaced directory is settled
  without a recursive removal:
  Git's pointer and an empty directory go,
  and anything added meanwhile stays and is named.
- The clone rung opens every source file below a held handle of the source root,
  compares the open descriptor's metadata
  with the frozen index entry before and after the clone,
  creates its entries below held handles of the destination,
  records
  the identity of everything it creates and the stat fields of the objects it made (the index takes those,
  not a fresh
  look at the paths;
  a created link keeps the source's stat fields,
  because nothing can pin a link while its metadata is
  read),
  and on a failure removes nothing and reports what stays.
- Every file operation below a root uses `openat2` with `RESOLVE_BENEATH`,
  `RESOLVE_NO_SYMLINKS`,
  `RESOLVE_NO_XDEV` and
  `RESOLVE_NO_MAGICLINKS` on one plain name at a time,
  and a root is accepted only when the path read back from its
  descriptor equals the path asked for.
- Both rungs require the new worktree's effective conversion settings (`filter.*`,
  `extensions.*`,
  `core.autocrlf` and its
  relatives),
  read in the new worktree's own context,
  to be the same sequence as the source's,
  in the same order.
  A
  filter selected by an attribute (tracked,
  `info/attributes` or a global file) declines the fast rungs,
  except the
  standard Git LFS definition;
  a `core.attributesFile` that is not an absolute path,
  LFS extensions,
  and a destination
  that is a symbolic link decline too.
- A step that cannot finish falls to Git's own checkout (`git reset --hard`),
  run in the bound worktree after the binding is
  verified and verified again after.
  When the checkout fails,
  or the binding no longer holds,
  the worktree and its
  registration stay as they are,
  the command fails,
  and one report names the path and the two commands:
  `git -C <path>
  reset --hard` to retry and `git worktree remove --force <path>` to remove it.
  This is the cost of the rule:
  Git's own
  `worktree add` removes what it registered when its checkout fails;
  the wrapper leaves it,
  because no check made before
  a removal is safe against another process changing the path,
  the registration or the contents afterwards.
- Success needs nothing durable except the completion marker,
  which only keeps a later report from calling a whole worktree
  interrupted;
  a marker that cannot be written is a warning.
- Recovery changes nothing.
  For an add that was bound and has no marker it prints the recovery report once;
  for one that was
  never bound it says that nothing proves a leftover at the recorded path is the add's;
  a stage named by an add is
  printed with the commands that remove it and is not removed.
  A record that cannot be read,
  or a marker that cannot be
  looked up,
  keeps the record and says the state is unknown.

### Cross-vendor reviews of the add path (2026-11-02)

The first review found four data-loss or false-clean problems and four concurrency or recovery problems.
Each was fixed with
a control that was shown to fail without the fix:
stub removal followed a link where a registered directory had moved;
clone
verification checked the source path after cloning;
the conversion settings were never compared;
a failed clone run deleted
files it never created;
the installed index was a second read of the file;
a destination that is a link was exchanged as a
link;
an interrupted add left a registered worktree with no record of how to finish it;
the index was written in place
before its lock was taken.

The second review found that the fixes still deleted objects identified by a pathname after an earlier check,
and that a
pathname in a record authorized deletion.
The common thread is now the rule under the `Implemented flow and what superseded the proposal` heading:
nothing is deleted that the add cannot
show it created.
Its findings were:
recovery deleted a worktree someone else made at the recorded path;
rollback validated
one resolution and deleted another;
clone cleanup unlinked replacements of created names;
the displaced destination was
removed recursively;
a failed swap back handed foreign data to the stage cleanup;
the index refresh certified a destination
modified after cloning;
a pointer edited to name another worktree's registration redirected the index install;
the
conversion comparison used a set;
equal configuration strings were taken for equal conversions;
a failed marker write still
reported success;
the install removed the lock after publication and overwrote an index written meanwhile;
a failed
rollback dropped its record;
and the primitives followed ancestors,
accepted `.` and `..`,
and missed bind mounts.

Third review:
automatic recovery and rollback could still delete or unregister what they could not prove the add made (an
adopted worktree,
a moved worktree's registration,
a replacement reset by the fallback,
stages identified by name,
quarantine names replaced after the check,
files inside an owned root,
a failed marker lookup read as incomplete).
The rule that nothing is deleted that the add cannot show it created
replaced all of that code:
recovery and rollback no longer delete anything.

Fourth review (part B of round three):
the quarantine directory was no protection against a process of the same user,
the
stat fields of a created link came from a name that can be replaced,
the index lock name could be replaced between the check
and the rename that published it,
a refresh deleted the next command's lock after its own rename,
the held-lock refresh
followed and truncated an existing staging name,
and a pointer through a link could be retargeted after the binding.
Each is
fixed as described in the bullets under the `Implemented flow and what superseded the proposal` heading.

Not covered by a control:
the arm of `finish_snapshot` that skips the stage cleanup when the stage name is not the
wrapper's is exercised through the primitives (`btrfs_a_failed_swap_back_does_not_claim_the_stage`) and the mapping of the
error,
not through a whole add;
a bind mount cannot be made without privilege,
so `RESOLVE_NO_XDEV` is relied on from the
kernel documentation;
a process that replaces the destination between Git's exit and the binding is not detected,
since the
binding is what first captures the identities.

## Implementation measurement (2026-11-02)

The ladder is implemented in `package/git-policy/cli` (commits on branch `btrfs-worktree-ladder`).
Measured with `git worktree add --quiet -b <name> ../<name>` and `git worktree remove ../<name>` through two wrapper
builds (0.5.11 installed as "before",
this branch's release build as "after") on a throwaway clone on `/var/home` that is a
subvolume,
with the real root `node_modules` (about 122 thousand entries) reflinked in as ignored state.
Four rounds per arm after one discarded warm-up,
arms shuffled each round,
inside a systemd unit with `CPUQuota=500%`,
`MemoryMax=10G`,
`TasksMax=1024`,
`IOWeight=50`;
the host load was high throughout.
`Ma2` and `La2` are the same binary as `Ma` and `La`,
so the spread between them is the noise.

- `M` is run in the clone's main checkout;
  `L` in a linked worktree that carries the ignored state.
- Add,
  median (minimum to maximum) seconds:
  `Mb` 15.15 (2.65 to 35.60);
  `Ma` 2.43 (1.23 to 17.38),
  `Ma2` 2.65 (1.10 to 5.22);
  `Lb` 80.70 (77.03 to 97.81);
  `La` 1.43 (0.82 to 2.00),
  `La2` 2.40 (0.82 to 13.83).
- CPU of the add (user plus system,
  median):
  `Mb` 3.4 s,
  `Ma` 1.2 s;
  `Lb` 30.9 s,
  `La` 1.2 s.
  Peak memory 489 MB before,
  22 MB after.
  Processes started:
  20 or 21 before,
  5 after.
- The first `git status` of the new worktree:
  0.18 to 0.34 s before from the main checkout and 0.12 to 1.66 s from the
  linked worktree;
  0.06 to 0.08 s after.
- Remove,
  median seconds:
  `Mb` 2.69,
  `Ma` 0.51,
  `Ma2` 0.49;
  `Lb` 6.92,
  `La` 0.49,
  `La2` 0.46.
- A worktree made after is a subvolume in every run;
  before it is an ordinary directory.
- From the main checkout the before arm copies only the named rules file (22 thousand entries);
  the snapshot carries the
  main checkout's whole ignored state (144 thousand entries),
  as the owner's decision of 2026-11-01 states.
- The wall-time spread of the same binary is large here (`Ma2` 1.10 to 5.22 s,
  `La2` 0.82 to 13.83 s),
  so the
  medians of `Ma`,
  `Ma2`,
  `La` and `La2` do not differ from each other,
  and only the difference to `Lb` (about 30 times)
  and the CPU,
  memory and process counts are clear of the noise.
  The remove medians of 0.5 s include the clean check of
  20 thousand tracked files and one Git run for untracked files,
  against 0.04 s for the bare ioctl.

### Measurement after the second review (2026-11-02)

Same method as above on a new throwaway clone (the real root `node_modules` reflinked in,
except the unreadable log files of
`node_modules/.monochromatic`;
197857 entries in a worktree made by the ladder):
four rounds per arm after a discarded
warm-up,
arms shuffled,
in a unit with `CPUQuota=500%`,
`MemoryMax=10G`,
`TasksMax=1024`,
`IOWeight=50`.
The host load average was
39 to 41 throughout,
higher than in the first measurement.
`Mb` is the released 0.5.14 wrapper,
`Mr1` and `Lr1` the build
before the reviews,
`Ma`,
`Ma2`,
`La` and `La2` this branch (`Ma2` and `La2` are the same binary as `Ma` and `La`).
- Add,
  median (minimum to maximum) seconds:
  `Mb` 24.95 (10.46 to 36.75),
  `Mr1` 8.56 (6.44 to 19.63),
  `Ma` 9.42 (7.83 to 26.98),
  `Ma2` 5.43 (4.20 to 9.28),
  `Lr1` 10.72 (2.47 to 36.08),
  `La` 37.03 (2.24 to 50.03),
  `La2` 6.37 (2.99 to 40.18).
  The spread of the same binary (`La` against `La2`) is larger than any difference between the builds,
  so wall time says
  nothing about the cost of the second review's changes;
  only the difference to `Mb` in process count and memory is clear.
- User CPU,
  median seconds:
  `Mb` 1.15,
  `Mr1` 0.12,
  `Lr1` 0.10,
  `Ma` 0.20,
  `Ma2` 0.20,
  `La` 0.18,
  `La2` 0.17.
  The extra 0.06 to
  0.1 s is the per-file descriptor work (the index refresh and the check of the snapshot are done through handles) and
  the configuration read of the new worktree.
  System CPU is dominated by the snapshot ioctl and varied from 0.6 s to 5.3 s with the
  load;
  its medians (`Mr1` 2.14,
  `Ma` 3.00,
  `Ma2` 3.55,
  `Lr1` 1.33,
  `La` 2.31,
  `La2` 2.57) do not separate the builds.
- Peak memory:
  `Mb` 495 MB,
  the ladder builds 19 to 22 MB.
  Processes started:
  21 for `Mb`,
  5 for every ladder build.
- The first `git status` of the new worktree:
  0.06 to 0.08 s for the ladder builds,
  0.25 to 0.48 s for `Mb`.
- Remove,
  median seconds:
  `Mb` 1.35,
  ladder builds 0.49 to 0.57.
  A worktree made by the ladder is a subvolume in every run.
- The measured repository applies the standard Git LFS filter by attribute;
  the ladder serves it because the definition is the
  standard one in both contexts.
