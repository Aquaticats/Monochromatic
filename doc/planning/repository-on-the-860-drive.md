# Repository on the 860 drive: measurements, inventory, the symlink design and runbook

Status:
preparation work for the move landed 2026-11-07 (section "Status and order of the work items": W1 built,
the copy tool fixed and in the repository,
the guided runner in TypeScript,
the 860 mount option as a runner stage);
the move itself has not run.
Earlier status:
revised 2026-11-07T06:45:00-05:00 to the design the owner picked (a symbolic link at the old path),
and revised again 2026-11-07T07:15:00-05:00 to answer the cross-vendor review (section "Cross-vendor review of the first
symlink revision").
The first version of this plan (automount over a locked stub,
option A) is at commit `93fec99100`
and its option text stays in this file as history.
Nothing real was moved,
copied,
mounted,
unmounted,
unlocked,
stopped or deleted while writing either version,
with one exception:
a read-only flag was set on the real main checkout by mistake and cleared about a minute later
(section "Quiesce and freeze").
Every command in the runbook that needs root is for the owner to run;
agents do not run `sudo`.

## Owner requirements this plan follows

- "Make this repo backed by 860 on disk."
  (2026-11-07)
- On ordering the 860's unlock before the user session with `Before=user@1000.service`:
  "No,
  that would be the wrong design.
  Make ordering don't matter."
  (2026-11-07)
  So the design works whichever starts first,
  the 860's unlock and mount or anything that uses the repository.
  No unit gets `Requires=` or `After=` on the mount.
- Design pick over the plan's recommended automount:
  "Symlink at the old path.
  You're supposed to take care of dangling links."
  (2026-11-07)
- Worktrees:
  "All worktrees (Recommended)".
  (2026-11-07)
- The unlock timeout question:
  "N/A".
  (2026-11-07)
- On the credential in `.git/config`:
  "We control our lfs setup.
  This is the defect in the lfs pkg we control.
  Fix it."
  (2026-11-07;
  another agent is fixing it.
  Once that lands `.git` holds no secret.
  This plan keeps the 0700 directory anyway.)
- "nothing should ever wait forever or indefinitely"
- "never discard user data",
  "Commits should never fail.",
  "Disk pressure being high is not an excuse.",
  "load is not an excuse."
- Nothing starts itself at login:
  "It should never do that!
  Follow Bazel."
  and "Systemd units are fine."
  (2026-11-07).
  The Git wrapper never starts itself at system startup or login.
  A systemd user unit that starts the wrapper's own services is allowed.
- On questions:
  "Overloading me with non-question questions will only make things worse."
  (2026-11-07).
  This plan keeps a question for the owner only when its answer changes what happens,
  no owner decision or measurement settles it,
  and it is destructive or a matter of preference.
  Everything else is adopted with its ranked pick in the section "Adopted decisions" and is open to the owner's veto.
- "Never generate synthetic host load in any session or subagent:
  no `stress-ng`,
  busy loops,
  `--stress-count`,
  or repeated full test runs;
  cap the test's own container (`--cpus` sized to the test) instead."
- `AGENTS.md` rule RBT:
  a reboot can make data on this machine's NVMe permanently unreadable,
  so commit and push before any reboot and never count on uncommitted files surviving one.

## What the plan decides and what it leaves to the owner

The owner picked the design,
so this plan no longer ranks designs for the decision;
it specifies the picked one and makes it safe.

- Decided by the owner:
  `/var/home/user/Monochromatic` becomes a symbolic link into the 860,
  every worktree moves with it,
  and a missing link target ("a dangling link") must be handled by every tool.
- Decided by this plan from measurements,
  open to the owner's veto:
  the place on the 860 (a dedicated 0700 directory),
  the guard on the mountpoint,
  the shared identity check every tool calls,
  the per-tool work items,
  the two-pass btrfs send and receive copy with an enforced freeze,
  the cut-over phases,
  and the reverse migration.
  The section "Adopted decisions" lists each with its reason.
- Left to the owner:
  one question at the end of the file.
- Not run here:
  anything that needs root,
  so the guards on a real mountpoint and the btrfs send and receive were not exercised.
  The user-level fixtures exercised everything a user namespace or the user's own subvolumes on the 860 allow,
  and the section "Fixture test before the real run" gives the exact root test for the rest.

## Cross-vendor review of the first symlink revision

The plan as of commit `837fc9a6eb` was reviewed by `openai/gpt-6.1-sol`
(review text kept at `/var/home/user/temp/agent/860-review/sol-review.txt`).
Its bottom line:
keep the owner's design,
but write the plan as a state-preserving migration with a complete filesystem inventory,
explicit phases,
enforceable final-write exclusion,
identity-aware readiness checks,
fail-closed transfers and a verified reverse migration.
Its verdict was that rollback and verification were not yet safe enough for cut-over.
This revision answers every finding.
Each line says fixed,
rejected or unverified,
the evidence,
and where the text is.

1.  Rollback was not a state-preserving reverse migration:
    fixed.
    "Before any commit" was the wrong boundary;
    the fast path is now limited to the validation window in which every writer is still excluded,
    and any rollback after release is a full reverse migration through the same fail-closed script
    (section "Rollback").
2.  The copy inventory came from Git's registry,
    not the filesystem:
    fixed.
    A root inventory of every nested subvolume is reconciled with the registry by the script,
    and the script refuses to run on a difference.
    Measured at user level:
    at 2026-11-07T06:57:00-05:00 the 248 directories under `.claude/worktrees` equal the 248 registered ones (243 subvolumes,
    5 plain directories),
    and a depth-3 `find -xdev -inum 256` of the main checkout,
    excluding the worktree folders,
    found no other
    subvolume root.
    That search does not descend into the worktree subvolumes (`-xdev` stops at each),
    so it says nothing about subvolumes inside a worktree.
    The complete answer is the filesystem's own list,
    which needs root (step R0 lists the whole filesystem and filters
    by path,
    because `btrfs subvolume list -o` is documented as not recursive):
    unverified until the owner runs it.
3.  "Ready" did not establish identity:
    fixed.
    The check now compares the btrfs filesystem UUID,
    the expected path and the link text with a recorded identity (section "Work items per tool",
    W1).
    Measured:
    `findmnt --target` prints the btrfs UUID at user level
    (`01c308e7-06fa-4737-8a7b-3bb5fcba871d` for the 860,
    `1bb3d23e-ec16-473a-b932-0deb3c450b94` for `/var/home`),
    and `statfs` `f_fsid` differs between subvolumes of one filesystem
    (`a281dfd5d053415f` for the main checkout,
    `a281dfd5d0534dac` for `/var`),
    so `f_fsid` is not used.
4.  Drain was an observation,
    not a write barrier:
    fixed with an enforced freeze,
    and its limit measured.
    `btrfs property set -ts <subvolume> ro true` worked as the subvolume's owner without root on a scratch
    subvolume on the 860:
    a new file,
    an open for write and a rename all failed with `Read-only file system`,
    and `ro false` restored writes.
    A write through a file descriptor opened before the freeze still succeeded and reached the file
    (measured).
    A read-only bind mount has the same limit (measured in a user namespace).
    So the exclusion is the freeze plus two holder scans (before and after the freeze),
    and any holder with a descriptor open in the tree stops the cut-over
    (section "Quiesce and freeze").
    The 93 external worktrees write the common `.git` through paths that the freeze refuses.
5.  Guards installed before the swap rejected the legitimate pre-migration repository:
    fixed.
    The check has a phase,
    `before` or `after`,
    recorded in a small file;
    the new binaries are installed in `before`,
    and the owner's step in the drained window flips it
    (section "Work items per tool",
    W1,
    and "Cut-over").
6.  "All worktrees" left the 16 under `~/worktrees` on the NVMe:
    fixed.
    They move too (adopted,
    former question Q9).
    They are plain directories,
    so they are copied by `rsync` as root into new subvolumes,
    and the reboot test is forbidden until every dirty worktree has a verified copy off the NVMe
    (sections "The copy" and "Cut-over").
7.  Pre-swap Git verification could inspect the old repository:
    fixed.
    Verification runs in a private mount namespace that bind-mounts the staged tree over the real path,
    so every gitfile resolves into the staged copy.
    Measured in a user-namespace fixture:
    a staged worktree's `.git` file still names the source path,
    and with the staged tree bind-mounted over it `git status` and `git worktree list` read the staged tree
    (section "Verification of the real copy").
    Baselines are captured after the freeze.
8.  The `rsync` comparison crossed subvolume boundaries:
    fixed and measured.
    On a scratch subvolume with a nested subvolume on the 860,
    `rsync --dry-run --itemize-changes --delete` of the parent's read-only snapshot against the assembled tree printed
    `*deleting   nested/inner.txt` without `--one-file-system` and printed no deletion with it
    (the nested stub in the snapshot is an empty directory of inode 2).
    The check now uses `--one-file-system --omit-dir-times` and runs once per subvolume.
    The same fixture showed that `rmdir` of the stub and a snapshot placed at its path both succeed on btrfs
    as an ordinary user.
9.  Send pipelines did not propagate failure:
    fixed and tested.
    `/bin/sh` here is bash without `pipefail`:
    `sh -c 'false | true'` exits 0 (measured).
    The script runs each pipeline as `bash -o pipefail -c`,
    stops at the first failure,
    checks that the received copy is read-only with the sent UUID as its received UUID,
    and writes a completion record before a copy may be placed.
    Seven control-flow cases ran against a btrfs test double
    (clean pass,
    failing send with a receiving side that still exits 0,
    a leftover partial receive,
    a copy that is not read-only,
    a transfer past its deadline,
    first then final pass,
    an inventory with a subvolume Git does not know);
    each stopped or finished as specified.
    Real `btrfs send` and `receive` need root:
    unverified.
10. The space bound was unsupported:
    fixed.
    The sentence "at most 263.6 GiB" is withdrawn.
    The section "Capacity policy" gives a measured-as-root estimate that includes the 16 plain-directory worktrees,
    thresholds,
    and what is retained.
11. A startup check does not prevent wrong-disk writes after the check:
    accepted and narrowed.
    For arbitrary commands the check is a diagnostic;
    concurrent replacement of the link during a command is outside the guarantee
    (fixture state 6 shows it cannot be prevented by a mode).
    For tools this repository controls,
    W1 provides an anchored directory descriptor (section "Work items per tool").
12. The hidden-mountpoint procedure lacked propagation isolation:
    fixed.
    R1 and R2 now run in one `unshare --mount --propagation private` shell with an identity check before each mutation.
    The private-namespace bind pattern itself was measured at user level (item 7).
    Root effects are unverified.
13. "Bounded" confused resource limits with time:
    fixed.
    The section "Deadlines" lists a wall-clock deadline,
    containment,
    leftover state and retry safety for each waiting operation,
    and states the limit that a timeout cannot end a task stuck in uninterruptible I/O.
14. MCP startup still depends on order:
    partly adopted.
    The Claude Code documentation says a stdio server whose first connection fails is marked failed and is restarted by
    Reconnect in `/mcp`,
    and `/mcp reconnect all` from v2.1.284
    ([MCP documentation][claude-mcp],
    section on stdio servers that exit or fail to start,
    fetched 2026-11-07;
    the installed `claude` is 2.1.319).
    The launcher waits up to a bounded time for the repository before failing,
    which covers the login race;
    a later start is one `/mcp reconnect all`.
    A proxy that answers before the repository exists is not built (see W7 for the reason).
15. A failed `ExecStop` does not mean the service stays active:
    fixed.
    The reboot test reports service state,
    mount state and mapper state separately,
    drains every consumer of the 860 first,
    uses a drop-in that replaces `ExecStart` instead of a stop,
    and the pcloud claim is withdrawn.
16. The check executable may not be on the manager's search path:
    fixed.
    Units use the absolute path.
    The existing user units already do
    (`claude-cpu-watch.service` and `kopia-snapshot-all.service` name absolute `ExecStart` programs).
17. The root fixture did not test the final-pass operation:
    fixed.
    The new fixture runs the real script with `-p` and `-c` over changed content,
    reassembles the tree,
    compares every subvolume,
    injects an interrupted receive,
    and flips a known byte.
18. The throwaway fixture paths were not safely allocated:
    fixed.
    A fresh `mktemp -d` directory,
    exclusive creation under `noclobber`,
    and cleanup only of what the invocation created.
    Measured:
    a second create under `noclobber` was refused.
19. Harness-state migration was underspecified:
    fixed.
    Backups,
    absent-destination requirement,
    writer exclusion and an atomic replace with a concurrent-change check
    (section "What the canonical path changes").
    Which key the harness uses stays unverified until the first session after cut-over.
20. Requirement conflict with "Commits should never fail.":
    adopted with a question.
    Refusing a commit before any state changes,
    with a message that names the cause,
    when the repository path is unavailable,
    is not a failed commit and acknowledges nothing it did not write;
    a deferred-commit queue is rejected (it would acknowledge before landing).
    The bounded maintenance window of the cut-over is the one real conflict and is the owner's question.
21. Correction in the "sound sections" list:
    `btrfs device add` refuses a device that holds a filesystem unless forced
    (`btrfs-device(8)`:
    "A device with existing filesystem detected by blkid ... will prevent device addition and has to be forced").
    Option D below is corrected.

[claude-mcp]: https://code.claude.com/docs/en/mcp

## Adopted decisions

These were open questions in the first revision.
Each is now a decision with its reason and is open to the owner's veto.
None needs an answer for the plan to proceed.

- `$PATH` entries inside the repository (was Q1):
  leave the fall-through.
  Ranking was leave > global stubs > move the launchers.
  Reason:
  the shadows' policies apply to work inside the repository,
  no work happens there while it is away,
  and a stub in front of the real tool is a hazard for every other project.
- How `claude-cpu-watch` copes at start (was Q2):
  the pre-start check with a long retry delay and no start limit (W8).
  Ranking was retry loop > `ExecCondition` > `.path` unit.
  Reason:
  the live unit already restarts every two seconds without end when the path dangles (measured),
  so a retry loop is the existing behavior at a gentler rate,
  and the other two options break the owner's ordering rule or can wait indefinitely.
- `user_subvol_rm_allowed` on the 860 (was Q3):
  add it.
  Reason:
  the wrapper removes a worktree that is a subvolume with `BTRFS_IOC_SNAP_DESTROY` when the mount allows it
  (`package/git-policy/cli/src/native/subvolume/subvolume_rollback.rs:91` checks `removal_allowed` first),
  and 243 of the nested worktrees are subvolumes,
  so without the option every such removal on the 860 takes the slower path.
  Deletion itself does not need the option:
  `btrfs subvolume delete` as the user fails with `EPERM`,
  but `rmdir` of an empty subvolume works (kernel feature `rmdir_subvol`,
  present here;
  `btrfs(5)`:
  "Since 4.18,
  the rmdir(2) syscall can delete an empty subvolume just like an ordinary directory").
  An earlier version of this item said the user cannot delete a subvolume without the option;
  that was wrong.
- Rule hooks when their script is missing (was Q4):
  non-blocking for the output filter,
  `onFailure: block` for hooks that enforce a rule.
  Reason:
  a skipped rule is a silent loss,
  a blocked session says why.
- Harness history (was Q5):
  rename the old folders with the safe procedure.
  Reason:
  rename keeps history without relying on unmeasured link handling.
- Clone sources (was Q6):
  `all`,
  decided by a measurement step.
  The first twenty worktree sends are timed under both settings;
  the faster one that keeps footprint under the capacity threshold is used.
- The pnpm store (was Q7):
  leave on the NVMe;
  the first install after the move is measured.
- Snapshots and backups of the repository on the 860 (was Q8):
  out of scope for this plan.
  Kopia keeps its source,
  and a snapper configuration for the 860 is a separate request.
- The 16 worktrees under `~/worktrees` (was Q9):
  they move,
  because the owner said "All worktrees (Recommended)" and two of them hold uncommitted files that a reboot can lose.
  The old folder becomes a link so old paths keep working.
- Third spelling `/home/user/Monochromatic`:
  left as it resolves.
  Configuration this plan writes uses `/var/home/user/Monochromatic`.
- Refusing writes with a plain message while the repository is unavailable:
  adopted (review item 20).
- Freeze method:
  `btrfs property set -ts <path> ro true` per subvolume,
  plus holder scans before and after (review item 4).

## Measurements

All sizes were taken on 2026-11-07 between 05:16:00-05:00 and 05:47:00-05:00 while other sessions kept committing,
so each number is a snapshot,
not a quiet-host figure.

### Tools and bounds

- `du --summarize --one-file-system --block-size=1M` ran inside `systemd-run --user --scope`
  with `CPUQuota=100%`,
  `MemoryMax=1G`,
  `TasksMax=16` and `IOWeight=10`.
- `btrfs filesystem du --summarize --raw` ran in the same scope bounds.
- One unwrapped `du` of the pnpm store (`~/.local/share/pnpm/store`) was started by mistake and stopped within
  minutes;
  its size is therefore not reported.

### The trap in `du --one-file-system` on this repository

Every btrfs subvolume has its own device number (`stat --format=%d` printed 65 for the repository,
244 and 110 for two
worktrees).
`du --one-file-system` therefore skips subvolumes.
Of the 246 entries in `.claude/worktrees`,
242 are subvolume roots (inode 256) and 4 are plain directories.
The first `du` over `.claude` (134046 MiB) counted only the 4 plain directories
(`fast-snapshot-add`,
`cargo-shadow-r21`,
`source-layout`,
`follow-ups-059`).
The repository root is itself a subvolume (inode 256),
and the worktree subvolumes are nested inside it.

### Disks

- NVMe pool:
  label `bazzite_bazzite`,
  one btrfs over two LUKS devices (`dm-0` 1T and `dm-1` 881.1G,
  read from `/sys/fs/btrfs/*/devices`).
  `df`:
  1.9T size,
  1.5T used,
  350G free;
  mounted at `/var/home` as subvolume `/home`,
  `compress=zstd:3`.
- 860 (`sda`,
  Samsung SSD 860 EVO 4TB):
  LUKS `crypt_sda`,
  label `encrypted_storage`,
  one btrfs on one device,
  mounted whole (`subvolid=5`) at `/var/mnt/encrypted`,
  `compress=zstd:3,noatime`.
  `df`:
  3.7T size,
  3.0T used,
  662G free.
  Top-level directory is `user:wheel` mode 777;
  its entries include `Archive`,
  `Backup`,
  `cargo-build`,
  `low`,
  `pcloud`,
  `SteamLinux`,
  `temp`,
  `VMs`,
  `win-salvage`.

### Repository, by part

- Tracked content in the main worktree:
  21543 files,
  876042393 bytes apparent
  (`git ls-files | wc -l` and `stat --format=%s` summed).
- `.git`:
  2158 MiB on disk.
  Packs are 709.78 MiB (4 packs,
  413533 objects);
  loose objects 1.55 MiB (184);
  measured split of the 2158 MiB (capped `du --summarize` per part):
  `objects` 728 MiB (packs and loose objects together),
  `worktrees` 881 MiB (340 administrative directories),
  `lfs` 362 MiB,
  `rr-cache` 94 MiB,
  `lost-found` 15 MiB,
  `logs` 8 MiB,
  wrapper records `monochromatic-git-policy-cli` 16 MiB and its `-worktree-copy` 2 MiB,
  the remainder (index,
  refs,
  small files) is the difference to the total.
  `.git/config` carries a credential inside the `lfs.url` value,
  so every copy of `.git` copies that credential;
  the new subvolume's top directory must be mode 0700 (the 860's top level is mode 777).
- Ignored and build output in the main worktree:
  `package` 73372 MiB (almost all of it ignored build output,
  since tracked content is 0.82 GiB in total),
  `node_modules` 3119 MiB,
  `.pnpm-store` 1095 MiB,
  `.serena` 152 MiB,
  `.cache` 51 MiB,
  `target` 4 MiB.
  The main worktree without worktrees is therefore about 80.5 GiB on disk.
- Worktrees nested in `.claude/worktrees`:
  243 measured of 246 listed (3 vanished while `du` ran,
  removed by other
  sessions).
  Sum of per-worktree `du` is 3843069 MiB,
  which is 3753 GiB,
  and it counts shared extents once per worktree.
  Distribution:
  median 1866 MiB,
  minimum 793 MiB,
  maximum 47048 MiB;
  84 worktrees are above 40000 MiB (copies of main's ignored state) and 137 are below 2000 MiB.
- Reflink sharing:
  `btrfs filesystem du --summarize` on one worktree (`add-explains-slow-route`) printed total 43.99 GiB,
  exclusive 383.86 MiB,
  set shared 43.09 GiB.
  The manual (`btrfs-filesystem(8)`,
  btrfs-progs 7.1) defines exclusive as not shared and set shared as the
  overlap-aware sum of shared space in the argument's tree.
- Whole-repository figure with sharing counted once (`btrfs filesystem du --summarize --raw` on the repository root,
  which descends into nested subvolumes):
  see the section "Whole-set figure".
- Worktrees outside the repository tree:
  the repository listed 341 worktrees when this was first written and 340 at 2026-11-07T06:26:00-05:00,
  of which 246 are under `.claude/worktrees`,
  16 are on the NVMe (`~/worktrees`),
  71 are on the 860 (`/var/mnt/encrypted/temp/agent`),
  and 6 are spelled `~/temp/agent/...`,
  which is also the 860 because `~/temp` is a link to `/var/mnt/encrypted/temp`
  (the first version of this plan counted those 6 as NVMe;
  that was wrong),
  plus the main one.
  Of the 246 nested ones,
  242 are subvolume roots and 4 are plain directories.
  None is locked (`git worktree list --porcelain | rg --count '^locked'` printed 0).
- Branch state of the 224 worktrees on a branch:
  180 branches are already merged into `main`;
  116 worktrees are detached.
  This says nothing about uncommitted files;
  those need `git status` per worktree.
- Files the user cannot read:
  `btrfs filesystem du` printed permission errors for root-owned mode 600 log files under
  `node_modules/.monochromatic/` inside worktrees (one sample:
  `root:root 600`).
  A user-level copy cannot read them;
  only a root-level copy (send and receive,
  or `rsync` as root) keeps them.
  The count (126) is in the section "Whole-set figure".

### Free space after each scope

Scopes,
because the nested worktrees decide what can fit:
- Scope one:
  main worktree only,
  about 80.5 GiB.
  860 free:
  662G minus 80.5 GiB leaves about 581 GiB.
  NVMe free:
  350G while the old copy is kept,
  about 430G after the owner removes the old main part
  (nested worktrees are not part of that gain,
  they stay where they are).
- Scope two:
  main plus all 243 worktrees copied file by file (`cp`,
  `rsync`):
  about 3753 GiB,
  which exceeds 662G
  by more than five times.
  This scope cannot be done.
- Scope three:
  main plus all worktrees with extent sharing kept (send and receive with clone sources):
  263.6 GiB of logical bytes measured in "Whole-set figure",
  fewer on disk after compression.
  860 free:
  662G minus 263.6 GiB leaves about 398 GiB before the 16 external worktrees,
  metadata and retained snapshots
  (this is an estimate,
  not a bound;
  see "Capacity policy").
  NVMe free after the owner removes the old copy:
  up to the shared-extent footprint of that copy comes back,
  which this plan did not separate from other data on the NVMe pool and does not estimate.
- Scope four:
  main plus a chosen subset of worktrees (for example the 44 unmerged branches plus the 116 detached
  ones,
  owner's choice):
  between scope one and scope three.

### Whole-set figure

`btrfs filesystem du --summarize --raw /var/home/user/Monochromatic`
(inside `systemd-run --user --scope` with the four bounds,
started 2026-11-07T05:41:25-05:00,
finished before 2026-11-07T05:53:59-05:00;
it descends into nested subvolumes) printed:

- Total 4115920404480 bytes (3833 GiB),
  the sum with every shared extent counted once per reference.
- Exclusive 106125254656 bytes (98.8 GiB).
- Set shared 176884908032 bytes (164.7 GiB).
- Exclusive plus set shared is 283010162688 bytes (263.6 GiB):
  the footprint with sharing preserved,
  as logical bytes before zstd compression.
  The per-worktree `du` sum (3753 GiB) is on-disk compressed bytes per worktree with sharing ignored,
  which is why it
  is close to the Total.
- Unreadable files:
  126 permission errors (all root-owned mode 600 `*.log.jsonl` and `*.probe.tmp` files under
  `node_modules/.monochromatic/`):
  90 in worktree `agent-a6f97f5da6f22de4d` and 36 in `package/wg/cli-quicker`.
  Their bytes are missing from these figures.

## Inventory of what names or depends on the repository path

Spellings seen in the wild:
`/var/home/user/Monochromatic` (the real path) and `/home/user/Monochromatic`
(through the root symlink `/home -> var/home`).
Both resolve to the same directory;
programs that call `realpath` or `canonicalize` see the first.

### Commands and counts

- Tracked files naming the absolute path:
  `git grep --files-with-matches --fixed-strings '/home/user/Monochromatic' | wc -l` printed 191 files
  (420 occurrences),
  mostly `doc/troubleshooting` 80,
  `doc/planning` 31,
  `doc/audit` 15,
  `doc/handover` 14,
  `package/pi-plugin` 11,
  `package/git-policy` 10.
  Historical records stay as written (rule WR6);
  none is a runtime dependency except those listed below.
- User units:
  `rg --follow --files-with-matches --ignore-case 'Monochromatic' ~/.config/systemd/user` printed 4 paths
  (`claude-cpu-watch.service` and its `default.target.wants` link,
  `chromium-issue-581090722-watch.service`,
  `kopia-snapshot-all.service`;
  the last two only name `~/.local/share/monochromatic-dev`,
  the toolchain outside the
  repository).
  Without `--follow`,
  `rg` skips symlinks and misses the first:
  the unit file `claude-cpu-watch.service` is itself a
  symlink to `/home/user/Monochromatic/package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service`,
  so the user manager dereferences the repository path when it loads units,
  before any `ExecStart` runs.
  Under the symlink design the same link is dangling at every login before the 860 is mounted,
  so the unit would not load;
  work item W8 replaces the symlink with a copy of the unit file.
- System units and autostart:
  `rg` over `/etc/systemd`,
  `/usr/lib/systemd`,
  `~/.local/share/systemd`,
  `/etc/xdg/autostart` found nothing;
  `~/.config/autostart/key-helper-service.desktop` names the repository.
- Generated and transient units:
  the wrapper writes `cli-git@.service` and `cli-git@.socket` to
  `/run/user/1000/systemd/user/`;
  12 service instances and 12 socket instances are loaded now.
  `/run/user/1000/systemd/transient` holds 33 units,
  12 of which name the repository (terminals,
  a browser,
  the music
  player,
  `rust-nightly-*` runs,
  `cargo-shadow` scopes);
  they are running work,
  not configuration.
- Shell and mise:
  `~/.bashrc` line 57 exports `ENV_LOCAL_JSON_PATH=/var/home/user/Monochromatic/.env.local.json`;
  `~/.config/mise/config.toml` names only `~/.local/share/monochromatic-dev` paths.
  `$PATH` holds 6 repository directories at the front (`.cache/*-bin`) and a long tail of
  `package/*/node_modules/.bin`;
  `git` resolves first to `.cache/cli-git-bin/git`,
  which is a symlink to
  `.cache/toolchain-launchers/cli-git-native` inside the repository;
  second is `~/.local/bin/git`.
- Toolchain links in `.cache`:
  187 symlinks,
  of which 12 point back into the repository's own
  `.cache/toolchain-launchers`,
  the rest into `~/.local/share/monochromatic-dev`,
  `~/.local/share/pnpm`,
  `~/.cargo`.
  16 launcher files embed the repository path (for example `cli-git-native.launch` holds the absolute lock file path).
- mise trust:
  `~/.local/state/mise/trusted-configs` holds 48642 links;
  18700 point into the repository and 13897 into
  `/var/mnt/encrypted`.
  mise canonicalizes a config path and hashes the canonical path (`trust` code in `src/config/config_file/mod.rs`,
  `path.canonicalize()` then `hash_to_str(&canonicalized_path)`,
  read from the `main` branch on 2026-11-07).
  A different canonical path therefore means every config is untrusted until trusted again.
- Wrapper state:
  `~/.local/state/cli-git/service` holds 591 instance directories and 3194 files,
  all finished
  records (none pending at the time of counting);
  3963 files under `~/.local/state/cli-git` embed the path text.
  The instance name is `instance_of` in `package/git-policy/cli/src/native/service/service_instance.rs`:
  FNV-1a 128-bit over the bytes of `std::fs::canonicalize(git_dir)`.
  Inside `.git` the wrapper keeps `monochromatic-git-policy-cli` (3573 files),
  `...-worktree-copy` (483 files),
  `cli-git-transactions` (3),
  `cli-git-captures` (2);
  they move with `.git`.
- Worktree administrative files:
  340 `.git/worktrees/<id>/gitdir` files and the same number of `.git` files in the
  worktrees,
  all absolute (`git worktree list` printed 341 lines).
- Editor workspaces:
  `rg --files-with-matches` over `~/.config` found 34 JetBrains files
  (`recentProjects.xml`,
  `trusted-paths.xml`,
  `LanguageServersSettings.xml` with 68 matches,
  workspace files),
  21 `.wrangler`,
  18 `net.imput.helium`,
  4 `lapce-stable`,
  2 each in `Codex`,
  `Emdash`,
  `Polypane`,
  `t3code`,
  `browser-os`;
  `~/.local/share/recently-used.xbel` has 35 matches.
  `monochromatic.code-workspace` and `.idea` use relative or repository-relative paths.
- Harness:
  `~/.claude/projects` has 14 folders;
  `-var-home-user-Monochromatic` (134 session entries) is keyed by the
  working directory string.
  `~/.claude/settings.json` has 4 hook,
  statusline and permission lines naming
  `package/claude-code-plugin/...`;
  `~/.claude.json` names the repository 5 times (two MCP servers `mvm` and
  `search-fetch` run from `package/mcp/mvm/dist/` and `package/search-fetch/mcp/dist/`);
  `~/.codex/config.toml` names it 3 times (one MCP server with `--cd /home/user/Monochromatic`).
- pnpm:
  `pnpm-workspace.yaml` sets `storeDir: ${HOME}/.local/share/pnpm/store` (on the NVMe),
  `packageImportMethod: clone-or-copy` and `virtualStoreType: global`.
  A repository on the 860 cannot clone from a store on the NVMe;
  pnpm then copies.
  `doc/troubleshooting/pnpm-symlinked-store-path.md` records that a symlinked spelling of a path in pnpm 12.16.1
  produced broken relative links.
- cargo:
  `.cargo/config.toml` uses a path relative to the config file's real path for `ZIG_GLOBAL_CACHE_DIR`;
  the build output (`target`) is inside the repository.
- Containers:
  19 podman containers;
  5 are in state `created` with bind mounts of worktree paths under
  `.claude/worktrees`;
  1 is running and does not mount the repository.
- Backups:
  `kopia snapshot list --all` lists the source `user@bazzite:/home/user/Monochromatic` among 10;
  the global policy has "Scan one filesystem only:
  false".
  The timer `kopia-snapshot-all.timer` runs one minute after login and hourly.
  Two other sources,
  `/var/mnt/encrypted/Archive` and `/var/mnt/encrypted/low`,
  are already on the 860.
  `btrfs-qgroup-rescan-home.service` documents that snapper snapshots `/var/home` hourly;
  snapshots of a parent
  subvolume do not contain nested subvolumes,
  so the repository (a nested subvolume) is not part of them;
  this is
  btrfs semantics,
  not probed here.
- Running processes:
  30 with the repository root as working directory,
  6 each in `settle-command-review-fixes` and
  `rust-latest-nightly`,
  3 in `rust-baseline-nightly`,
  7 `claude` processes,
  12 `cli-git-service` processes,
  plus
  MCP servers,
  the Codex app server,
  `claude-cpu-watch`,
  an IDE.
  (`find /proc -maxdepth 2 -name cwd -user user -printf '%l\n'`.)

### What happens when the repository is unreachable at the moment the item runs

This section was written for the first version;
its mentions of "the runbook" and "the stub" mean the first version at commit `93fec99100`.
The behavior under the symlink design,
with measurements,
is in the section "Work items per tool".

- `claude-cpu-watch.service`:
  the user manager reads the unit file through a symlink into the repository;
  if the
  target is missing at manager start or at `daemon-reload`,
  the unit is not loaded and nothing retries.
  If it is loaded and `cpu-watch.ts` is missing,
  `Restart=always` with `RestartSec=2` and the default start limit
  (5 starts in 10 s,
  read with `systemctl --user show`) never trips the start limit:
  the unit restarts every few seconds without end
  (measured,
  see W8;
  an earlier version of this plan said it ends in a failed unit,
  which the measurement disproved).
  This is the item most sensitive to ordering.
- Claude Code hooks (`bash-output-filter`,
  statusline) and the two MCP servers:
  each command is `node /var/home/user/Monochromatic/...`;
  with the path missing the command exits with an error,
  so tool calls and MCP servers fail for that session until restarted.
- Codex MCP server with `--cd /home/user/Monochromatic`:
  fails to start.
- `~/.config/autostart/key-helper-service.desktop`:
  already broken today,
  it names `packages/kwin/...` and the
  directory is `package` (`ls` printed "No such file or directory");
  independent of this move.
- `$PATH` entries inside the repository:
  missing directories are skipped by the shell;
  `git` then resolves to
  `~/.local/bin/git`,
  the wrapper outside the repository.
- Wrapper services:
  their program (`~/.local/share/monochromatic-dev/toolchain/cargo/...`),
  journal
  (`~/.local/state/cli-git/service/<instance>`) and sockets (`$XDG_RUNTIME_DIR/cli-git`) are outside the repository;
  they stay up and each request about an unreachable Git directory fails.
  `instance_of` hashes a path that cannot be resolved as given,
  so a dangling link makes a different name.
- kopia:
  an unreachable source path is reported as an error by the hourly job;
  behavior with an empty stub (a snapshot
  of nothing) is inferred,
  not probed,
  and the locked stub below avoids it.
- mise:
  trust links are keyed by canonical path;
  unreachable means the repository's config is not loaded
  (inside the repository only;
  shells elsewhere are unaffected).
- Worktrees on the NVMe outside the repository (22):
  their `.git` file points into the repository's
  `.git/worktrees`;
  unreachable means "not a git repository" errors,
  no loss.
- Worktrees on the 860 (71):
  today,
  with the 860 not yet mounted,
  their paths are missing,
  and a `git worktree prune`
  (also run by `git gc`) removes the administrative directory of a worktree whose directory is missing once
  `gc.worktreePruneExpire` (default three months) has passed.
  None is locked,
  so this hazard exists today and is independent of this plan;
  `git worktree lock` on those 71 removes it.
- pcloud (`pcloud.service`) already uses `RequiresMountsFor=/var/mnt/encrypted` and `After=var-mnt-encrypted.mount`
  in the user manager;
  it is the owner's existing pattern and is unaffected by the repository.
- Containers in state `created`:
  their bind sources are resolved at start;
  a start while the repository is
  unreachable fails with a missing source.

## Options considered (history; the owner picked option B on 2026-11-07)

This section and the next are the first version's analysis,
kept as the record of what was weighed.
References in the history sections to "the runbook",
"Undo",
"the stub",
"the root commands" and "the first open
question" mean the first version's runbook and open questions at commit `93fec99100`;
those headings no longer exist in this file.
The owner picked option B ("Symlink at the old path.
You're supposed to take care of dangling links.").
The sentences that call B a hazard describe what a bare link does;
the section "Dangling-link guards,
measured" and the work items answer them.

All options are judged on the owner's rule:
whichever starts first,
the unlock and mount or anything that uses the
repository,
the result must be correct,
nothing waits forever,
nothing is written to the NVMe by mistake.

### Facts about the mount mechanism, read from source

Read from `src/core/automount.c` and `src/core/job.c` at the `main` branch on 2026-11-07
(installed systemd is 262.1-1.fc45;
the exact tag was not read,
so the fixture test in the runbook repeats these):

- An automount unit mounts autofs at its path,
  and the first access starts the paired mount unit.
- A process that waits on the path is released with an error when the mount unit reaches dead or failed:
  `automount_send_ready(a, a->tokens, -ENODEV)` in `automount_trigger_notify`.
  `job_finish_and_invalidate` calls `unit_trigger_notify` even for a timed-out or dependency-failed job,
  so a
  failed unlock also ends in that error.
- If the mount unit hits its start limit,
  the automount unit fails and unmounts the autofs;
  the waiters get `-EHOSTDOWN` and the path becomes the bare directory underneath.
  So the directory underneath must be safe to expose.
- The automount `Where=` must be a canonical path (`unit_fail_if_noncanonical_mount_path`),
  so the unit is named for
  `/var/home/user/Monochromatic`,
  not `/home/user/Monochromatic`.
- A mount unit gains `Before=local-fs.target` unless `nofail` is among its options
  (`systemd.mount(5)`,
  "Default Dependencies";
  `mount_is_nofail` in `src/core/mount.c`),
  and `mount-860.service` is `After=local-fs.target`,
  so the mount unit needs `nofail` or the pair forms an ordering cycle.
- Idle expiry is off unless `TimeoutIdleSec=` is set;
  the plan leaves it off.
- `mount-860.service` is `Type=oneshot`,
  for which `TimeoutStartSec=` defaults to infinity
  (`systemd.service(5)`);
  `systemctl show` printed `TimeoutStartUSec=infinity`.
  That is a wait without end in any chain that depends on it.
- `JobTimeoutSec=` defaults to infinity and cancels the job without failing the unit (`systemd.unit(5)`).

### Option A: automount of a repository subvolume on the 860 over a locked stub at the old path

- Layout:
  a subvolume `/var/mnt/encrypted/Monochromatic` holds the repository (nested worktree subvolumes inside it).
  `var-home-user-Monochromatic.automount` and `.mount` (bind) serve `/var/home/user/Monochromatic`.
  The mount unit has `Requires=mount-860.service` and `After=mount-860.service`,
  so the first access starts the
  unlock;
  `mount-860.service` gets a start timeout and a bounded notification.
  The old directory is a root-owned,
  mode 000,
  immutable (`chattr +i`) empty stub.
- Before the 860 is mounted:
  the first access blocks only for the time the unlock takes (bounded by the unit
  timeouts) and then either succeeds or returns `ENODEV` ("No such device");
  no process ever holds a handle inside
  the stub,
  because every lookup goes through autofs.
  If the automount itself is absent or failed (for example the start-limit case),
  the stub answers `EACCES`
  and nothing can be created in it.
  The stub is on the NVMe and empty,
  so nothing is hidden by the mount later.
- Reboot:
  `local-fs.target` ordering makes the automount ready before login (`Before=local-fs.target` is a default
  dependency of automount units);
  no process needs the 860 until it touches the path.
  Login-time readers (the user manager reading the symlinked `claude-cpu-watch.service`,
  the hourly kopia job)
  trigger the unlock themselves.
  Hazard to test:
  `mount-860.service` ends with `notify-send` through the user session bus;
  if the user manager is
  itself blocked on the repository path,
  that bus may be unable to start and `notify-send` can wait.
  The fix is `timeout 5` around that call (see the runbook);
  it is a deadlock candidate,
  derived by reading,
  not seen.
- If `mount-860.service` fails:
  three failed attempts end with exit status 1;
  the mount unit fails with a dependency
  error;
  waiters get `ENODEV`;
  the next access retries,
  within the unit's own start limit
  (`StartLimitBurst=5` per 120 s);
  after that the automount fails and the stub (locked) shows `EACCES`
  until `systemctl reset-failed` and a retry.
  Nothing waits without bound.
- Worktree subvolumes,
  reflinks,
  snapshots:
  they stay as subvolumes inside the 860 subvolume;
  `worktree add` snapshots still work (same filesystem;
  the snapshot source is the bind-mounted subvolume root,
  which
  must be probed on the fixture).
  Worktrees on the 860 under `/var/mnt/encrypted/temp/agent` become same-filesystem with main for the first time.
  Worktrees on the NVMe (22) cross filesystems:
  the wrapper's copy fallback applies,
  as it already does for the 71
  on the 860.
- Wrapper:
  the canonical path is unchanged,
  so the 340 instance hashes,
  the 591 state directories,
  the 12 live
  units,
  mise trust links (18700),
  the harness project folder,
  kopia's source path and every `.git` file stay valid.
  The 12 running `cli-git-service` processes hold the NVMe copy open and must be stopped at the cut-over
  (they are socket-activated and start again on the next commit).
- Undo:
  stop and disable the automount,
  `chattr -i` and remove the stub,
  rename the kept NVMe copy back;
  commits made
  on the 860 since the cut-over are carried back by a reverse copy first (runbook,
  "Undo").
- Needs root:
  unit files,
  the drop-in,
  `systemctl`,
  `chattr`,
  the stub,
  and (for the copy) `btrfs send` and
  `btrfs receive`.

### Option A2: plain bind mount unit (no automount) over the same locked stub

- Same as A without the autofs;
  the mount is started by `WantedBy=user@1000.service` or `local-fs.target`.
- Before the mount:
  the stub answers `EACCES`;
  nothing writes to the NVMe.
  Nothing triggers the unlock when a program arrives first,
  so a login-time reader that comes before the mount
  fails and,
  for `claude-cpu-watch`,
  stays failed.
  That is a start-order dependency,
  which the owner rejected.
- A process that holds a handle in the stub before the mount keeps seeing the stub afterwards (a mount does not move
  existing working directories),
  which the locked,
  empty stub makes harmless but not useful.
- Everything else matches A.

### Option A3: mount the 860 subvolume directly at the old path (`What=/dev/mapper/crypt_sda`, `subvol=Monochromatic`)

- Same automount,
  but the mount unit mounts the device instead of binding a path under `/var/mnt/encrypted`.
- It removes the dependency on `/var/mnt/encrypted` being mounted,
  but it mounts the same btrfs a second time with
  different options (`compress`,
  `noatime`);
  how the kernel treats conflicting options on a second mount was not
  probed,
  and it still needs `mount-860.service` to create `/dev/mapper/crypt_sda`.
- Worth a fixture test next to A;
  not recommended before that test.

### Option C: move the canonical path to the 860 and change every reference

- The repository lives at `/var/mnt/encrypted/Monochromatic`;
  the old path is removed or becomes a stub.
- Before the 860 is mounted:
  `/var/mnt/encrypted` is an ordinary NVMe directory,
  so `mkdir --parents` of a path
  under it succeeds on the NVMe and is hidden by the mount later,
  the hazard the owner named.
  Guarding it needs an immutable underlying directory that cannot be set while the 860 is mounted
  (a private mount namespace can reach it;
  see the runbook).
- Reboot and failed unlock:
  every reference fails with "No such file or directory" until the 860 is up,
  with no
  trigger that brings it up;
  start-order dependent.
- Changes:
  340 worktree `.git` and `gitdir` files (`git worktree repair`),
  all 340 instance hashes,
  591 state
  directories orphaned,
  18700 mise trust links,
  the harness project folder,
  kopia's source,
  the units,
  the hooks
  and MCP entries,
  editor workspaces.
- Undo:
  reverse every edit;
  a large manual change set.

### Option B: symbolic link at the old path to the 860

- Before the 860 is mounted:
  a dangling link answers "No such file or directory",
  but a command that creates parents
  through it (`mkdir --parents /var/home/user/Monochromatic/x`) creates `/var/mnt/encrypted/Monochromatic/x` on the
  NVMe,
  hidden by the mount later.
- Canonical path changes for every program that resolves links:
  the wrapper (`canonicalize`),
  mise (hash of the
  canonical path),
  `git` itself.
  Programs that keep the old spelling see a second spelling of one directory;
  the repository already has a record of
  that failure class in `doc/troubleshooting/pnpm-symlinked-store-path.md` (pnpm relative links).
- No ordering mechanism at all;
  undo is `rm` of the link.

### Option D: btrfs device add (the 860 joins the NVMe pool)

- Disqualified.
  `btrfs device add` refuses a device that holds a filesystem unless forced
  (`btrfs-device(8)`),
  and forcing it destroys the 860's 3.0T of data
  (`never discard user data`).
  The pool would then need the 860 present to mount `/var/home`,
  so a late or failed unlock stops the home
  directory from mounting:
  the strongest possible start-order dependency.
  Undo is a full balance and device removal.

### Send and receive is a copy method, not a path option

It is covered in the section "Copy methods",
together with the alternatives.

## Ranking before the owner's pick (history)

Written 2026-11-07T05:47:00-05:00,
before the pick.

Ranking:
A > A2 > C > B > D,
with A3 held out until its fixture result,
because:

- A over A2:
  both keep the canonical path,
  but only A turns the first access into the trigger for the unlock and
  returns an error after a bounded time,
  so it does not depend on which starts first;
  A2 depends on someone starting
  the mount.
- A2 over C:
  A2 keeps the path,
  the 340 instance hashes,
  18700 trust links,
  the harness folder and kopia's source
  unchanged and fails clearly with `EACCES`;
  C changes every reference and has the unguarded
  `mkdir --parents` hazard.
- C over B:
  both change the canonical path,
  but C changes it everywhere and explicitly,
  so a missed reference fails
  loudly;
  B leaves two spellings of one directory alive,
  which is the pnpm failure class,
  and hides the same hazard
  behind a dangling link.
- B over D:
  B is undone with `rm` and loses nothing;
  D destroys the 860's data and makes the home mount depend on
  the 860.
- A3 sits next to A:
  it needs the fixture result on conflicting mount options before it can be ranked.

### The owner's pick, 2026-11-07

The owner picked option B over the recommended A.
The ranking above stays as history.
The current ranking for the picked design is B first,
because it is the owner's decision,
and the two penalties the first version gave it are answered by measurement:

- "Writes through a dangling link land on the NVMe":
  plain tools (coreutils,
  Python,
  Rust,
  Node,
  `rsync`,
  `git`) all fail through a dangling link;
  only a program that resolves with a missing-tolerant call and then creates the result writes,
  and a root-owned mountpoint refuses that (fixture states 1 and 2).
- "The canonical path changes":
  every consequence is listed with its handling in the section "What the canonical path changes".

Two costs of B remain that A did not have:
replacement of the link by a real folder is not preventable by a mode (fixture state 6),
and every tool needs the shared check (W1).

### The owner's go-ahead on timing, 2026-11-07T12:46:40-05:00

Told that the move waited only on the bounded cut-over window with commits refused,
the owner said,
verbatim:
"No need to wait for me.
Do it whenever you see fit."
So the coordinator picks the window:
the moment no agent works in a worktree,
so no commit is refused,
with the stated maximum the runbook gives after timing the first pass.
Every copy step still needs root,
and agents never run `sudo`,
so the owner runs the root commands;
the coordinator hands them over as one guided script in this order:
the root fixture test of `btrfs send` and `receive` on loop images,
the inventory `R0`,
the first pass (no window;
the tree stays writable),
then,
at the chosen moment,
the final pass and the switch to the link.

That guided runner is written
(first as a bash script in `4980c200ea`,
then,
on the owner's word "Wizard writes TypeScript,
and you just do the move yourself,
no need to delegate to me" (2026-11-07,
relayed by the coordinator),
as TypeScript that the coordinator runs):
[the guided runner](../runbook/repository-on-the-860-drive-guided-runner.md)
(`package/repository-move/cli/src/run-repository-move.ts`).
Its `prepare` command runs the fixture test,
R0,
the pre-flight and the first pass and then stops;
its `cutover` command runs the freeze,
the final pass,
the verification,
the swap and the release inside a stated
maximum duration;
its `thaw` and `rollback` commands undo the freeze and the fast path.
The runbook also lists where this plan no longer matches the repository and which commands of the plan it changes.

## Copy methods

### M1: btrfs send and receive with clone sources (needs root)

- `btrfs send` needs read-only snapshots;
  `btrfs-send(8)` says all snapshots in a send must be read-only and a
  clone source must be in exactly the same state on both sides.
- A send of the main subvolume does not include nested subvolumes (btrfs semantics);
  each worktree subvolume needs its
  own send,
  with the already sent main and worktrees as clone sources (`-c`),
  so the receiver can clone shared
  extents instead of writing them again.
- `--compressed-data` (protocol 2) sends compressed extents as they are,
  if both kernels support it.
- The received subvolumes are read-only;
  a writable snapshot of each,
  placed at its final path,
  is instant and keeps
  sharing.
- Reads root-owned files too (the 90 or more unreadable log files).
- Unverified here:
  that the clone sources actually preserve sharing between worktrees that were snapshots or reflink
  copies of main,
  and the run time of the extent lookups;
  the fixture measures both with `btrfs filesystem du` on the
  receiver.

### M2: unprivileged copy plus per-worktree snapshots

- Copy main once with `rsync`,
  then for each worktree create a snapshot of the copy on the 860
  and run `rsync --inplace --no-whole-file` from the old worktree onto it,
  so unchanged files stay shared.
- No root for the data (subvolume create and snapshot on a user-owned top directory are allowed on btrfs),
  but it
  cannot read the root-owned files,
  and whether `rsync` keeps extents shared (modification times,
  in-place writes) is
  unverified.

### M3: plain `cp` or `rsync` of everything

- Cannot fit (scope two in "Free space after each scope").
  Fine for scope one (main only),
  done as root so root-owned files are included.

Ranking for scope three:
M1 > M2 > M3,
because M1 keeps sharing and reads everything,
M2 keeps sharing only if
`rsync` behaves as assumed and skips root-owned files,
M3 does not fit.


## The symlink design

### Layout

- `/var/mnt/encrypted/repository`:
  an ordinary directory on the 860's btrfs top level,
  owner `user:user`,
  mode 0700.
- `/var/mnt/encrypted/repository/Monochromatic`:
  the main checkout,
  a btrfs subvolume with every worktree subvolume nested in it exactly where it is now
  (`.claude/worktrees/<name>`).
- `/var/mnt/encrypted/repository/received-read-only`:
  the read-only copies that `btrfs receive` makes.
  They are the parents of the final increments and stay until the owner removes them;
  they share extents with the live tree when they are made,
  and cost more as either side changes (section "Capacity policy").
- `/var/home/user/Monochromatic`:
  a symbolic link (absolute target) to `/var/mnt/encrypted/repository/Monochromatic`.
  `/var/home/user` is owned by `user` with mode 0750 (measured),
  so the owner can create and remove the link without root.
- `/var/home/user/repository-move-snapshots`:
  mode 0700 directory on the NVMe that holds the read-only source snapshots while the copy runs.
  They pin NVMe space until deleted,
  and the NVMe mount has `user_subvol_rm_allowed`,
  so the owner deletes them as the user.

### Why a dedicated 0700 directory

- The 860's top level is `user:wheel` mode 777 (measured),
  and the repository root directory is mode 755 (measured),
  so a snapshot of it keeps mode 755.
  Only the parent can carry the restriction,
  and `.git` holds LFS objects,
  the wrapper's records and `rr-cache` even after the credential fix.
- The parent `repository` does not exist on the NVMe.
  A program that resolves the link and then creates the resolved path must first create `repository` under the
  unmounted mountpoint,
  which the mountpoint guard refuses (next section).
- The directory groups the repository with its received snapshots,
  so the 860's top level gains one entry,
  not three.
- Rejected:
  `/var/mnt/encrypted/Monochromatic` directly under the 777 top level.
  It has no restriction that survives a snapshot,
  and a program that makes the missing parent has nothing to refuse it.

### The mountpoint guard

The guard is that the directory `/var/mnt/encrypted` on the NVMe,
the one hidden while the 860 is mounted,
is owned by root,
mode 0755,
and immutable.

- `stat` of `/var/mnt` printed `root:root 755` on the NVMe.
  The directory `/var/mnt/encrypted` itself could not be inspected without root:
  while the 860 is mounted `stat` shows the 860's top level (`user:wheel 777`,
  device 108),
  and no user-level route to the hidden directory exists
  (a rootless bind mount of `/var/mnt` failed with "wrong fs type,
  bad option,
  bad superblock";
  `/sysroot/ostree/deploy/default/var` is a different subvolume from the live `/var`,
  which is `dm-1[/var]` in `findmnt`).
- Root-owned mode 0755 stops every user-level program from creating anything under the unmounted mountpoint
  (fixture result in the next section).
- Immutable (`chattr +i`) additionally stops root-run programs and `chmod`.
  Whether the flag survives the mount is the owner's root test below;
  it is not claimed here.
  The trap:
  `sudo chattr +i /var/mnt/encrypted` while the 860 is mounted sets the flag on the 860's top-level inode,
  not on the hidden NVMe directory.
- The commands below reach the hidden directory through a non-recursive bind mount of `/var` made inside a private
  mount namespace.
  Private propagation matters:
  on a shared hierarchy a mount event elsewhere could otherwise appear at the alias and make `chown`,
  `chmod` or `chattr` act on the 860 instead of the hidden directory.
  The pattern (a bind inside `unshare --mount --propagation private`) was exercised at user level in a user namespace;
  the root effects are unverified.

Owner's root test (R1 and R2) in one namespace,
with identity checks before every mutation.
It stops at the first failure and unmounts nothing real:

```bash
# R1 and R2: inspect, then guard, the hidden mountpoint. The shell below sees /var/mnt/encrypted only through a bind.
sudo unshare --mount --propagation private bash -euo pipefail -c '
  alias_dir=$(mktemp --directory /run/underlying-var.XXXXXX)
  mount --bind /var "$alias_dir"
  hidden="$alias_dir/mnt/encrypted"
  # The alias must be the NVMe /var, never the 860: compare btrfs filesystem UUIDs.
  [ "$(findmnt --noheadings --output UUID --target "$alias_dir")" = "$(findmnt --noheadings --output UUID --target /var)" ]
  [ "$(findmnt --noheadings --output UUID --target "$alias_dir")" != "$(findmnt --noheadings --output UUID --target /var/mnt/encrypted)" ]
  stat --format="%U:%G %a %n" "$hidden"
  lsattr -d "$hidden"
  echo "entries under the hidden mountpoint: $(ls -A "$hidden" | wc -l)"
  chown root:root "$hidden"
  chmod 0755 "$hidden"
  chattr +i "$hidden"
  lsattr -d "$hidden"
  umount "$alias_dir"
  rmdir "$alias_dir"
'
lsattr -d /var/mnt/encrypted
```

Expected from the first lines:
owner `root:root`,
mode 755,
no flags,
and zero entries.
If there are entries,
something was written under the mountpoint while the 860 was away;
the shell stops being useful for that case,
so the owner reads and moves them before running the guard lines.
Expected from the last `lsattr`:
the mounted 860,
with no `i` flag.

Whether a mount can be placed over an immutable directory is proven before any login depends on it,
in the loop-file test F7 below.
The next login then repeats the proof with the real mount:
`mount-860.service` mounts at login and notifies on failure.
If the real mount fails with the flag set,
the owner runs `chattr -i` on the hidden directory through the same namespace pattern and keeps root-owned 0755.

### Fixture results: what a process sees in each state

Fixture:
`podman unshare` (a user namespace in which the owner's uid is root and uids 1 to 65536 map to the subuid range),
with tmpfs mounts that exist only inside that namespace.
Programs ran as namespace uid 1000 through `setpriv`.
The "NVMe" is a tmpfs;
the mountpoint is a directory on it;
the "860" is a second tmpfs (mode 0777 like the real top level) mounted over the mountpoint;
the link sits in a directory that uid 1000 owns,
and points to `<mountpoint>/repository/Monochromatic`.
The fixture script is `/var/home/user/temp/agent/dangling/guard-fixture.ts`
and its last output is `guard-result.txt` in the same folder.
Run at 2026-11-07T06:09:00-05:00 inside `systemd-run --user --scope` (`CPUQuota=100%`,
`MemoryMax=1G`,
`TasksMax=128`,
`IOWeight=10`).

#### State 1: 860 not mounted,
mountpoint not guarded (owned by the user)

- Through a dangling link,
  these all failed and created nothing:
  `mkdir -p L/x/y` (`File exists`),
  `touch L/f`,
  `install -D`,
  `cp -r`,
  `tar -x -C`,
  `ln -s`,
  `rsync` (`mkdir ... failed: File exists`),
  `git init L/r`,
  `git -C L status`,
  Node `mkdirSync(..., {recursive: true})` (`ENOENT`),
  Node `writeFileSync`,
  Python `os.makedirs` (`FileNotFoundError` or `FileExistsError`),
  Rust `create_dir_all` (`File exists (os error 17)`).
  The reason:
  the final component is a link whose target is missing,
  so `mkdir(2)` answers `EEXIST` and every other call answers `ENOENT`.
- Resolve-then-create wrote to the NVMe:
  `mkdir -p "$(realpath -m L)/x"` returned success and created `repository/Monochromatic/x` on the NVMe,
  hidden by the 860 mount later.
  (Later probes in that same state then saw an existing target,
  so the rows after it in the raw output are not independent results.)
  The programs that resolve with a missing-tolerant call are `realpath -m`,
  `readlink -m`,
  and Python `Path.resolve()` (non-strict);
  Node `realpathSync` and Rust `canonicalize` throw on a missing target
  (measured:
  `ENOENT` and `No such file or directory`).
  `git grep` over tracked code outside `doc/` finds no use of `realpath -m`,
  `readlink -f` or `canonicalize-missing`
  (positive control:
  `git grep -c realpath` outside `doc/` does match three files,
  so the search ran;
  the first attempt died with "failed to create thread" under a small task limit and was rerun with
  `grep.threads=2` and `TasksMax=128`).

#### State 2: 860 not mounted,
mountpoint root-owned 0755

- The same resolve-then-create command failed:
  `mkdir: cannot create directory '.../repository': Permission denied`,
  and the mountpoint stayed empty.
- All the probes of state 1 failed the same way as before;
  nothing was created.
- Not testable at user level:
  `chattr +i` on the mountpoint (`Operation not permitted`;
  a namespace has no `CAP_LINUX_IMMUTABLE` over the host,
  and tmpfs refuses it).
  The owner's root test R1 and R2 covers it.

#### State 3: 860 mounted,
repository directory present

- Every probe succeeded,
  as the repository needs.
  Node,
  Python,
  Rust,
  `rsync`,
  `git init` and `git -C L status` all worked through the link.

#### State 4: 860 mounted but the `repository` directory is gone

- The plain probes failed again (dangling link),
  but `realpath -m` followed by `mkdir -p` created `repository` on the 860,
  because the 860's top level is mode 777.
  A wrong or fresh drive mounted at the mountpoint would show the same.
  This is why each tool checks that `<link>/.git` exists,
  not only that the link resolves.

#### State 5: lazy detach with a holder

- A plain `umount` of the mountpoint refused while a process had its working directory inside:
  `target is busy`.
- After `umount -l`,
  the holder kept writing into the detached mount
  (`ok-on-detached-mount`),
  its `pwd` failed (`getcwd: cannot access parent directories`),
  and new lookups through the link failed as in state 2.
- `mount-860.service` stops with a plain `umount ... 2>/dev/null` followed by `cryptsetup luksClose`
  (read in `/etc/systemd/system/mount-860.service`),
  so a stop while any process holds the repository refuses and says nothing.
  This plan never uses `umount -l` or `--force` on the 860:
  a lazy detach strands holders on a mount nobody can see,
  which loses their writes.
  The holder list in the section "Drain" exists for this.

#### State 6: the link replaced by a real directory

- The guard does not stop this.
  `rm -f L` then `mkdir -p L/x` as the owner of the containing directory succeeded,
  and the new directory sits on the NVMe.
  The link lives in `/var/home/user`,
  which the user owns,
  so no mode,
  sticky bit or flag on a symbolic link prevents its replacement.
- The only defence is that every tool refuses unless `<link>/.git` resolves,
  and that the shared check reports "a real folder replaced the link" when `lstat` of the path is a directory.
  Work item W15.

### Other measurements

- `~/temp` is itself a symbolic link to `/var/mnt/encrypted/temp` (`ls -l`),
  so all agent scratch and 71 worktrees already depend on the 860,
  and today they dangle whenever the 860 is away.
  The design extends an existing pattern;
  it does not introduce a first one.
- Subvolumes on the 860 and the user:
  the 860 is mounted without `user_subvol_rm_allowed` (`findmnt`),
  so `btrfs subvolume delete` as the user printed `deletion failed with EPERM`.
  Emptying a subvolume and then `rmdir` works without the option (kernel feature `rmdir_subvol`):
  the coordinator deleted the plan's six scratch subvolumes in `/var/home/user/temp/agent/dangling`
  that way as the user,
  after clearing `ro` on the read-only one,
  at 2026-11-07T07:22:58-05:00.
  The option still decides whether the wrapper's one-step subvolume removal works;
  see the adopted decision on `user_subvol_rm_allowed`.
- Mount-time facts:
  the 860 is mounted whole (`subvolid=5`),
  options `compress=zstd:3,noatime,ssd,space_cache=v2`.

## Ordering: nothing depends on the mount

- No unit gets `Requires=`,
  `After=` or `RequiresMountsFor=` on `/var/mnt/encrypted`,
  and no unit is added that orders itself against `mount-860.service`.
  Existing:
  `pcloud.service` has `RequiresMountsFor=/var/mnt/encrypted`;
  it is the owner's existing pattern for pcloud and is not changed by this plan.
- The wrapper's units carry no repository path
  (`/run/user/1000/systemd/user/cli-git@.service` has `ExecStart` in `~/.local/share/monochromatic-dev` and
  `StateDirectory=cli-git/service/%i`,
  read 2026-11-07),
  so they start and run with the 860 away,
  and a request about an unreachable repository is answered with an error.
- Rule for a unit or tool that needs the repository:
  at start it asks the shared check (W1);
  if the answer is "not ready" it fails plainly with the cause and the next use asks again,
  or (for a daemon) it retries on a fixed delay.
  Resource limits (CPU,
  memory,
  tasks,
  I/O weight) are not time limits,
  so each waiting step also has a wall-clock deadline (section "Deadlines").
- `mount-860.service` is left as it is:
  it is `Type=oneshot` with `TimeoutStartSec=infinity`,
  but nothing in the repository's chain waits on it,
  which is why the owner's answer to the unlock timeout question was "N/A".
  A synchronous `systemctl start mount-860.service` would wait on it,
  so every step in this plan that starts it uses `--no-block` and polls with a deadline.
  It gains one change in the root steps:
  `user_subvol_rm_allowed` in its mount options (adopted decision).

## Deadlines

A resource cap such as `CPUQuota` or `MemoryMax` bounds how much a command may use,
not how long it may take.
Every step that can wait has its own wall-clock deadline,
what happens to the child processes,
what it leaves behind,
and whether a retry is safe.

- The shared check (W1):
  a deadline of a few seconds enforced by the caller
  (a worker thread or `timeout` around the check),
  because `stat` on a stalled drive can block in the kernel.
  Containment:
  the worker is abandoned and the caller reports "the drive did not answer" as a failure.
  Leftover state:
  none.
  Retry:
  safe.
- A unit that runs the check (`claude-cpu-watch.service`):
  `TimeoutStartSec` on the unit.
  Leftover:
  none.
  Retry:
  the unit's own retry delay.
- Each `btrfs send | btrfs receive`:
  `timeout --kill-after=30 <seconds>` around a `bash -o pipefail` inside the `systemd-run --scope`,
  so the whole pipeline and its children die together.
  The default is 14400 s per transfer,
  the plan author's choice until the first measured transfer replaces it.
  Leftover:
  a partial received subvolume with no completion record.
  Retry:
  not automatic;
  the script stops and names the exact `btrfs subvolume delete` for the owner to run.
- `git fsck --full`,
  `rsync --checksum` comparisons:
  one deadline per subvolume chosen by the owner at run time;
  leftover none;
  retry safe.
- The drain:
  the owner's deadline;
  at the deadline nothing is signalled and the cut-over does not happen.
- `systemctl start mount-860.service`:
  `--no-block` followed by `systemctl is-active` polled for at most 180 s,
  because the unit can retry its unlock three times with a ten-second pause and a slow key derivation.
- A userspace deadline cannot end a task stuck in uninterruptible kernel I/O (state `D`).
  The controller then returns a bounded failure and treats the unfinished work as not done;
  the stuck process is reported for the owner and is not killed.

## Work items per tool

The shared contract comes first because every other item calls it.
Each item gives what the tool does today
(measured here,
read from source,
or inferred;
the label is in brackets),
the change,
and the test.

The identity the check enforces is recorded once,
by the owner's cut-over step,
in `~/.config/monochromatic-dev/repository-link.json`:
the link path,
the exact link text,
the approved canonical root,
the btrfs filesystem UUID of the 860,
and the phase (`before` or `after`).
The check reads that file and never writes it.

The probe returns one of these answers:

- Ready,
  phase `after`:
  `lstat` of the link path says symbolic link;
  its text equals the recorded text;
  the target resolves to the recorded canonical root;
  `/proc/self/mountinfo` shows the target on the filesystem whose UUID equals the recorded 860 UUID
  (`findmnt --target <path> --output UUID` prints that UUID at user level,
  measured);
  the root is a directory;
  and `<root>/.git` is a directory whose own filesystem UUID equals the root's.
  A link to a directory on another filesystem is not ready (the review's outside-the-860 case).
  A directory named `.git` alone is not identity,
  so the check also requires the recorded repository marker (the first commit hash of `main`,
  read from `.git/` without running Git:
  the file `.git/monochromatic-root-commit` written at cut-over).
- Ready,
  phase `before`:
  `lstat` says directory,
  its filesystem UUID equals the recorded NVMe UUID,
  and `.git` is a directory.
  Phase `before` exists so the new binaries can be installed while the repository is still on the NVMe.
- The 860 is not mounted:
  the target is missing and `/var/mnt/encrypted` is not a mount point in `/proc/self/mountinfo`.
  Message:
  "the 860 drive is not mounted yet (/var/mnt/encrypted is empty),
  so the repository at /var/home/user/Monochromatic is
  not there;
  unlock it (the mount-860 service does that at login) and run this again".
- Mounted but the folder is missing:
  the mountpoint is a mount point and the target is missing.
  Message:
  "the 860 drive is mounted but /var/mnt/encrypted/repository/Monochromatic does not exist;
  nothing was written".
- A real folder replaced the link (phase `after`):
  `lstat` of `/var/home/user/Monochromatic` says directory.
  Message:
  "/var/home/user/Monochromatic is a real folder,
  not the link to the 860;
  nothing was written".
- Wrong filesystem or wrong repository:
  the link resolves but the UUID or the marker differs.
  Message names both values.

Exit status:
128 for the Git wrapper (Git's own fatal status),
75 (`EX_TEMPFAIL`) for every other tool.
Every call is a few `lstat`,
`stat` and one read of `/proc/self/mountinfo`;
none creates anything.

1.  W1,
    the shared check.
    Today:
    nothing shares one.
    Change:
    a small package with a library and a bin,
    `package/repository-link/check` with bin `repository-link-check`
    (name is a proposal;
    it follows `SGD` and `VGW`),
    installed outside the repository (`~/.local/share/monochromatic-dev/toolchain-global`)
    so the check itself never dangles,
    plus a second bin `repository-link-run [--wait-seconds=<n>] <command...>` that checks and then replaces itself with
    the command.
    With `--wait-seconds` it re-asks every second until the deadline,
    then fails (used by launchers of long-lived programs such as MCP servers).
    For programs this repository controls the library also offers `openRepositoryRoot()`:
    it runs the full check once,
    opens the resolved root as a directory descriptor,
    and returns it,
    so later operations use descriptor-relative calls or `/proc/self/fd/<n>/...` and never look the link up again.
    A held descriptor also makes an ordinary `umount` of the 860 refuse
    (a working-directory holder did,
    measured in fixture state 5).
    For arbitrary commands the check is a diagnostic:
    a command that re-resolves the link later can be redirected by a replaced link,
    and the plan excludes that from its guarantee (review item 11).
    The Git wrapper is Rust,
    so it carries the same logic as a function and the same test table.
    Tests:
    one table of fixture states (ready in each phase,
    not mounted,
    mounted without folder,
    replaced by a folder,
    no marker,
    a link to a directory on another filesystem,
    a wrong filesystem mounted at the mountpoint)
    run against the library,
    both bins and the Rust copy,
    with adversarial paths (a target with a newline,
    a space,
    `..` segments,
    a link chain),
    and a stalled-drive case (a path on a hung test mount) asserting the deadline.
2.  W2,
    the Git wrapper client (`cli-git-native`,
    `package/git-policy/cli`).
    Today:
    `~/.local/bin/git` is a link to the global launcher in `toolchain-global/launchers`,
    outside the repository,
    so it still runs when the repository is away [measured];
    the repository-scope entry `.cache/cli-git-bin/git` is skipped by `$PATH` [measured with `which -a git`].
    With a scratch link to a missing target:
    `git -C link status`,
    `git -C link/sub status`,
    `git --git-dir=link/.git status` and `git -C link rev-parse --show-toplevel` each ended in 0.012 to 0.092 s with
    `fatal: cannot change to 'link': No such file or directory` or `fatal: not a git repository`,
    no hang,
    no new file in `~/.local/state/cli-git/service` [measured 2026-11-07T06:01:00-05:00].
    A worktree whose `.git` file names the missing repository:
    `status` and `log` answered `fatal: gitfile does not point to a valid repository`,
    and `commit` answered `fatal: cannot change to 'wt1': No such file or directory`
    for a folder that exists [measured];
    that message is wrong for a commit.
    `instance_of` hashes the path as given when `canonicalize` fails
    (`service_instance.rs`,
    function `instance_of`) [read from source].
    Change:
    run W1 first for any target path that names the repository link,
    for its alternate spellings (`/home/user/Monochromatic`,
    the approved canonical root,
    the stale NVMe path),
    and for any Git directory the wrapper resolves through `GIT_DIR`,
    `GIT_WORK_TREE`,
    `GIT_COMMON_DIR` or a
    worktree `.git` file:
    the test is on the canonical path the wrapper finally uses,
    not on the text of the argument,
    so a command that spells the repository by its physical path is checked too;
    print the plain message with status 128;
    during the freeze window (a write answers `Read-only file system`) print "the repository is frozen for the move to
    the 860;
    nothing was written;
    try again after the move finishes";
    make `instance_of` return an error when `canonicalize` fails,
    so no instance,
    journal directory or socket is ever named for an unreachable path,
    and make the commit path report the real cause.
    Tests:
    the dangling fixture asserts the message,
    status 128,
    and no new directory under the service state folder or the runtime folder;
    a worktree-through-missing-link fixture asserts the commit message names the repository,
    not the working folder.
3.  W3,
    the commit service (`cli-git-service`).
    Today:
    started only by the first connection to a socket the wrapper wrote;
    its unit names no repository path;
    it holds a plan for failed pushes in `push-retries` with doubling pauses [read from source and `README.md`].
    What it does when the repository vanishes mid-request is not measured.
    Change:
    a request whose Git directory is unreachable is answered with the plain message and left for the client to retry
    on its next command;
    a push plan whose directory is unreachable keeps its doubling pause and its record and never runs on another path.
    Tests:
    a service fixture whose repository folder is renamed away between two requests,
    asserting an answer within the existing ceiling,
    no spin (the count of attempts across a fixed interval),
    and the plan record still present after the folder returns.
4.  W4,
    `mpm-shadow` and the launchers in `.cache/toolchain-launchers`.
    Today:
    a launcher is a hard link of the `mpm-shadow` binary plus a `<name>.launch` record;
    `run_launcher` reads the lock file with `.ok()` and then replaces itself with the real binary,
    so an unreadable lock drops the version warning silently and still runs
    (`package/mpm-shadow/cli/src/toolchain/launcher.rs`,
    function `run_launcher`) [read from source].
    `ensure_launcher` calls `fs::create_dir_all` on the launcher folder;
    Rust `create_dir_all` through a dangling link returned `File exists (os error 17)` and created nothing
    [measured].
    The launchers themselves live inside the repository,
    so when it is away the program is not found at all [measured with `which -a`].
    Change:
    when the lock file is unreadable and its path is under the repository link,
    print one line naming the 860 (or the cause W1 returns) and still run the real binary
    (the global launcher is the correct fallback,
    so this must not fail);
    when `ensure_launcher` is asked to install under the link and W1 is not ready,
    fail with the plain message before any write.
    Tests:
    `launcher_tests.rs` gains an unreadable-lock case asserting the line and the exec;
    an install-through-dangling-link case asserting nothing was created.
5.  W5,
    `cargo-shadow`,
    `pnpm-shadow`,
    `search-shadow`.
    Today:
    same launcher scheme;
    each program runs with its working folder inside the repository,
    so a dangling start cannot happen;
    the failure is mid-run (the 860 goes away under a running cargo or pnpm),
    which the operating system reports as `ENOENT` or `EIO` [inferred,
    not measured].
    With the repository away,
    `$PATH` falls through to other programs [measured with `which -a`]:
    `cargo` and `pnpm` to the global `exposure` links (for `cargo`,
    an ELF),
    `rg` to a mise shim,
    and `find`,
    `du`,
    `grep` to `/usr/bin`.
    These are the unshadowed tools,
    so the shadows' policies (lock trust for `cargo`,
    the search limits for `rg`) are skipped silently outside the repository.
    Change:
    none in the programs beyond W1 at start for an explicit repository path;
    the silent fall-through is an adopted decision (leave it).
    Tests:
    a launcher fixture with a missing `.launch` target asserting a plain message.
6.  W6,
    Claude Code hooks and the status line in `~/.claude/settings.json`.
    Today:
    `PreToolUse` on `Bash` runs `node /var/home/user/Monochromatic/package/claude-code-plugin/bash-output-filter/bundle/node/index.mjs`,
    and the status line runs `node .../statusline/src/statusline.ts`.
    Node with a missing script printed a stack with `Error: Cannot find module` and exited 1 [measured].
    Claude Code treats an exit code other than 0 or 2 from a hook as a non-blocking error and lets the action go ahead;
    "when the script path doesn't exist or isn't executable,
    the shell exits with a code like 127";
    `onFailure` set to `block` makes a failing hook block
    ([hooks reference][claude-hooks],
    sections on exit codes and on hook failures,
    fetched 2026-11-07).
    Change:
    the command becomes `<absolute path>/repository-link-run node <script>`,
    so the message is plain and the status is 75 (non-blocking);
    a hook that enforces a rule (the `guardrail` package under `package/claude-code-plugin`,
    if it is configured) sets
    `onFailure: block`
    so a missing script blocks instead of letting the action through.
    The output filter only trims text,
    so it stays non-blocking.
    Tests:
    a hook fixture with a missing script asserts the status and the first line.
7.  W7,
    MCP servers:
    Claude's `mvm` and `search-fetch`
    (`~/.claude.json`,
    `node .../package/mcp/mvm/dist/...` and `.../package/search-fetch/mcp/dist/...`)
    and Codex's server with `--cd /home/user/Monochromatic` (`~/.codex/config.toml`).
    Today:
    a missing script fails the server start with the Node stack [measured for Node with a missing script];
    Codex's `--cd` to a missing folder is inferred to fail the same way [not measured].
    Change:
    commands go through the absolute path of `repository-link-run --wait-seconds=20`,
    which re-asks every second for at most twenty seconds before failing,
    so a server started in the login race still comes up;
    a server that missed even that is restarted by `/mcp reconnect all` in the session (documented behavior,
    see the
    review table,
    item 14).
    A proxy that answers `initialize` before the repository exists and starts its backend later is not built:
    it is a new MCP frontend for two servers,
    and the documented reconnect already recovers a session without restarting it.
    Codex's `--cd` stays and the server's own error is accepted.
    Tests:
    start each configured command with the link dangling and assert the first line;
    start one with the link dangling,
    make the target appear after five seconds (a fixture swap),
    and assert it comes up inside the wait.
    Reconnect after a missed start is not testable without a live harness session and is unverified.
8.  W8,
    `claude-cpu-watch.service`.
    Today:
    the unit file in `~/.config/systemd/user` is a symbolic link into the repository
    (`ls -l` printed `-> /home/user/Monochromatic/package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service`),
    so the user manager reads through the link while loading units;
    if the repository is away at manager start or at `daemon-reload` the unit is not loaded and nothing retries
    [read from the earlier measurement of 2026-11-07T05:47:00-05:00].
    With the unit loaded and `cpu-watch.ts` missing,
    `Restart=always` with `RestartSec=2` and the default start limit
    (5 starts in 10 s;
    `systemctl --user show` at 2026-11-07T06:44:29-05:00 printed `StartLimitIntervalUSec=10s`,
    `StartLimitBurst=5`,
    `Restart=always`,
    `RestartUSec=2s`,
    `NRestarts=0` for the live unit)
    restarts without end and never reaches a failed state.
    Measured after the owner approved the probe that the tool guard had refused:
    a transient unit `dangling-probe-cpu-watch` with the same restart settings,
    running Node on a script behind a link to a missing target,
    showed `activating (auto-restart)` with `NRestarts=8` at 2026-11-07T06:45:28-05:00
    and `NRestarts=12` after a further thirty-second watch,
    and was stopped and cleared at 2026-11-07T06:45:43-05:00.
    Starts two seconds apart never put more than five starts inside a ten-second window.
    An earlier version of this item said the unit ends failed and stays failed;
    that reading was wrong.
    Change:
    the unit file in `~/.config/systemd/user` becomes a regular copy
    (the repository keeps the original;
    a check compares the two);
    its start runs `ExecStartPre=<absolute path>/repository-link-check` (the manager does not read the shell's `PATH`;
    the existing units already name absolute programs) with `TimeoutStartSec=20s`,
    so a dangling path fails with status 75 and the plain message,
    with `Restart=on-failure`,
    `RestartSec=30`,
    and `StartLimitIntervalSec=0`
    so it never ends in a permanent failure and retries every thirty seconds until the mount is there.
    Tests:
    `systemd-analyze verify` on the unit;
    a transient copy with the link dangling,
    run by the owner,
    asserting it stays in `activating (auto-restart)`,
    then reaches `active (running)` after the target appears.
    The alternatives were weighed and the retry loop adopted (section "Adopted decisions").
9.  W9,
    `~/.local/bin/spawn-claude` and `spawn-pi`.
    Today:
    symbolic links into the repository;
    a shell reports `No such file or directory` with no cause [inferred from the link form;
    the shell message for a missing command target is not repository-specific].
    Change:
    replace each with a regular two-line script outside the repository that runs W1 and then the program,
    so the message is plain.
    Tests:
    run each with the link dangling.
10. W10,
    `env-local-json` (`ENV_LOCAL_JSON_PATH=/var/home/user/Monochromatic/.env.local.json` in `~/.bashrc`).
    Today:
    the reader throws a typed error naming the path (`EnvLocalJsonPathError`,
    `EnvLocalJsonReadError`) [read from source];
    the writer writes a temporary file beside the store and renames it,
    and creates no directories [read from source],
    so a dangling path fails without writing.
    Change:
    the error text gains the cause from W1 when the path is under the repository link.
    Tests:
    the reader and writer unit tests gain a dangling-path case.
11. W11,
    `mount-860.service`.
    Today:
    stops with a plain `umount`,
    which refuses with a held mount [measured in the fixture],
    and `cryptsetup luksClose`;
    both silent (`2>/dev/null`).
    A failed stop command does not mean the service stays active:
    systemd may leave the unit failed or inactive while the mount and the mapping remain [from `systemd.service(5)`
    semantics;
    not measured on this unit].
    Change:
    none required for correctness.
    Optional,
    owner's file:
    list holders in the stop path with a message,
    do not run `luksClose` after a failed `umount`,
    and never use `umount -l` or `--force`.
    Tests:
    with a holder (a shell with its working folder in the repository) run the stop and report three states separately:
    the service state (`systemctl is-active`),
    the mount state (`findmnt /var/mnt/encrypted`),
    and the mapper state (`ls /dev/mapper/crypt_sda`).
    The pcloud claim of the earlier revision (that stopping this system unit stops the user unit) is withdrawn:
    `pcloud.service` lives in the user manager and the plan does not rely on any cross-manager effect.
12. W12,
    `kopia-snapshot-all.service`.
    Today:
    the source stays `user@bazzite:/home/user/Monochromatic`.
    kopia resolves a symbolic-link source before reading it:
    `getLocalFSEntry` calls `resolveSymlink` and returns an error if it fails
    (`cli/config.go`,
    line 115 at blob `b1abe8c3`,
    read through `gh api` on 2026-11-07),
    and `getContentToSnapshot` records the source path as given.
    So the source identity is unchanged,
    the data comes from the 860,
    and a dangling link is an error for that source before any upload;
    no empty snapshot is made [read from source].
    `snapshot create --all` reports the error and the unit fails for that hour [inferred from the code path].
    Change:
    none.
    The first snapshot after the move is expected to re-read files whose inode changed;
    kopia compares by path,
    size and modification time,
    so btrfs send and receive (which keeps modification times) should keep it incremental [inferred,
    check the first run's upload size].
    The ten kopia sources do not include `/home/user` or `/var/home/user`
    (`kopia snapshot list --all`,
    2026-11-07T06:50:00-05:00),
    so the renamed old tree `Monochromatic.nvme-before-860-...` beside the link is not backed up
    and is not uploaded by the hourly job.
    Tests:
    a post-cut-over `kopia snapshot create` of the repository source,
    comparing uploaded bytes with the previous snapshot.
13. W13,
    other readers:
    `podman` containers in state `created` with bind mounts of worktree paths (start fails with a missing source;
    after the move the bind source is the link spelling,
    which resolves when mounted);
    editors (JetBrains,
    Helium,
    Lapce,
    Codex,
    Emdash,
    Polypane,
    t3code),
    which only read;
    terminals started with the repository as working folder,
    which fall back to home;
    `mise` tasks,
    which run inside the repository only.
    Change:
    none.
    Tests:
    after cut-over start one `created` container and open the repository in the IDE through the link.
14. W14,
    one spelling for tools that compute relative paths.
    Today and result:
    section "What the canonical path changes".
    Tests:
    that section's pnpm and mise checks.
15. W15,
    the link itself.
    Today:
    nothing watches it;
    the fixture's state 6 shows a real folder can replace it.
    Change:
    W1 reports a real folder at the link path once the phase is `after`;
    the wrapper's `doctor` prints the same line;
    the cut-over checklist runs `repository-link-check` first.
    Tests:
    state 6 of the fixture table.

### Status and order of the work items, 2026-11-07T14:55:39-05:00

W1 is built: `package/repository-link/check` (library, bins `repository-link-check`,
`repository-link-run` and `repository-link-record`, and `openRepositoryRoot()`),
with tests for every answer of the contract (the identity file also records the mount point, so fixtures can vary it).
The install outside the repository copies the built bins to `~/.local/share/monochromatic-dev/repository-link/`
(the plan's `toolchain-global` folder holds launchers that name the repository, so a separate folder keeps the check
free of it).
The stalled drive on a hung mount and a wrong filesystem mounted at the real mount point need root and are untested.
The Rust copy of the check in the wrapper (W2) is not built.
Nothing else of W2 to W15 is built.

What blocks what.
Nothing from W2 to W15 blocks the copy or the swap itself:
the copy is read-only on the old tree,
and with the 860 mounted the link resolves for every tool.
The plan's phase 0 does require one of them before the swap (the `claude-cpu-watch.service` unit copy of W8,
a root-free step in `~/.config/systemd/user`).
The rest block the first reboot or the first login before the drive is mounted,
which the plan's cut-over step 8 tests,
so the order below is the order in which they stop a bad first login.

1.  W8 first,
    owner package `package/claude-cpu-cap/daemon` and the unit link in `~/.config/systemd/user`.
    Today [measured 2026-11-07T14:55:39-05:00]:
    the unit is still a symbolic link into the repository,
    `Restart=always`,
    `RestartSec=2`,
    default start limit,
    `NRestarts=0`,
    and `ExecStart` runs `%h/.local/share/mise/installs/node/latest/bin/node %h/Monochromatic/package/claude-cpu-cap/daemon/src/cpu-watch.ts`.
    Change and test as in item 8.
2.  W2 and W3,
    owner package `package/git-policy/cli` (Rust).
    Today [read from source]:
    `instance_of` in `src/native/service/service_instance.rs` still hashes the path as given when `canonicalize` fails.
    Change as in items 2 and 3:
    the wrapper's copy of the W1 test table,
    status 128,
    `instance_of` returning an error,
    and the freeze message.
    Test as in items 2 and 3.
    This is what a commit does on the first login before the drive is mounted.
3.  W6,
    W7 and W9,
    owners the user's `~/.claude/settings.json`,
    `~/.claude.json`,
    `~/.codex/config.toml` and the links in `~/.local/bin`,
    with `package/claude-code-plugin/bash-output-filter`,
    `package/mcp/mvm`,
    `package/search-fetch/mcp`,
    `package/claude-code-plugin/source` and `package/pi-plugin/spawn`.
    Today [read 2026-11-07T14:55:39-05:00]:
    only the output filter (`PreToolUse` on `Bash`,
    no `onFailure`) and the status line are configured as hooks,
    so no rule-enforcing hook needs `onFailure: block` yet;
    `mvm` is spelled `/home/user/Monochromatic/...`,
    `search-fetch` `/var/home/user/Monochromatic/...`,
    and Codex runs `mise exec --cd /home/user/Monochromatic -- node ...`;
    `spawn-claude` and `spawn-pi` are links into the repository.
    Change and test as in items 6,
    7 and 9,
    through `repository-link-run` at its installed path.
4.  W4,
    owner package `package/mpm-shadow/cli`.
    Today [read from source]:
    `run_launcher` still reads the lock file with `.ok()` (`launcher.rs`,
    line 357) and `ensure_launcher` still calls `fs::create_dir_all` (line 237).
    Change and test as in item 4.
5.  W5,
    owners `package/cargo-shadow`,
    `package/pnpm-shadow` and `package/search-shadow`:
    no change beyond W1 at start,
    the test of item 5.
6.  W10,
    owner `package/env-local-json/core`:
    the cause text from W1 in the two typed errors (`ENV_LOCAL_JSON_PATH` in `~/.bashrc` still names
    `/var/home/user/Monochromatic/.env.local.json`),
    and the dangling-path tests of item 10.
7.  W11,
    owner the file `/etc/systemd/system/mount-860.service` (root).
    Today [read with `systemctl cat`]:
    `ExecStop` runs `umount /var/mnt/encrypted 2>/dev/null` then `cryptsetup luksClose crypt_sda 2>/dev/null`.
    The mount option edit is a stage of the guided runner,
    not an owner change of this item;
    the optional stop-path message stays optional.
8.  W12,
    no owner package:
    `kopia-snapshot-all.service` is a user unit whose `ExecStart` is `kopia snapshot create --all`,
    and the only work is the first snapshot's upload size after the move.
9.  W13 and W14,
    no package:
    checks after the cut-over,
    from the plan's items 13 and 14.
10. W15,
    owners W1 (built:
    it reports a real folder at the link path in phase `after`) and `package/git-policy/cli` (the wrapper's
    `doctor`,
    with W2).

Identity file.
`repository-link-record --phase=before` runs in the `prepare` command (stage `i2-record-before`),
and `--phase=after` in the `cutover` command after the swap (stage `c20b-record-after`),
as the user and never with `sudo`.
The rollback renames the identity file aside and records phase `before` again.

[claude-hooks]: https://code.claude.com/docs/en/hooks

## What the canonical path changes

A process that resolves the link sees `/var/mnt/encrypted/repository/Monochromatic`.
Measured with a shell inside a linked folder (fixture,
2026-11-07T06:19:00-05:00):
`pwd` printed the link spelling,
`pwd -P` printed the physical one,
and Node printed `process.cwd()` as the physical path and `process.env.PWD` as the link spelling.
That pair is the source of every two-spelling effect below.

- Commit-service instance hashes.
  `instance_of` is FNV-1a over `canonicalize(git_dir)`,
  so every worktree's name changes (one per worktree:
  340 at the last count,
  the main `.git` and 339 administrative directories).
  Old state:
  591 folders under `~/.local/state/cli-git/service` (595 when counted again at 2026-11-07T06:03:00-05:00,
  other sessions keep committing),
  3213 `.done` records and no other file,
  `busy/` and `push-retries/` empty everywhere at that count.
  Handling:
  drain first (section "Drain");
  stop the old instances;
  leave the old folders in place as the finished-commit journal (rollback makes them valid again);
  new commits create new instances on first use.
  Hazard:
  a non-empty `push-retries/` at cut-over is a plan for a push that would resume under the old name and never run,
  so the drain refuses to proceed while any exists.
  The wrapper's records inside `.git` (`monochromatic-git-policy-cli`,
  `...-worktree-copy`,
  `cli-git-transactions`,
  `cli-git-captures`) move with `.git` in the copy.
  `cli-git-transactions` holds 3 `.pending` folders (dated 2026-10-03,
  2026-10-04 and 2026-11-01);
  they are carried unchanged and listed at the drain.
- mise trust.
  Trust links are keyed by the canonical path (source read in the first version);
  the 18700 links for the old path stay as stale files.
  Measured in a fixture with `MISE_STATE_DIR` and the other mise folders set to scratch:
  `mise trust` run from a link spelling recorded the physical path;
  `mise trust --all` from the main checkout trusted the root file and two nested package files;
  a linked worktree then showed the same files as trusted (`mise trust --show`).
  The manual says the same ("Trusting a config file in a repository's main checkout also trusts the same path in its
  git worktrees"),
  and `mise run`,
  `mise install` and `mise exec` trust their active config on their own.
  Handling:
  after cut-over run `mise trust --all` once from the repository,
  then `mise trust --show` in the root and in one package and one worktree.
  `trust --all` skips hidden folders and `node_modules`,
  so a config under a hidden folder is trusted by its first `mise run`.
  Re-trusting is through mise itself;
  nothing writes into `~/.local/state/mise/trusted-configs` by hand.
- pnpm.
  `doc/troubleshooting/pnpm-symlinked-store-path.md` records pnpm 12.16.1 computing relative links from a symlinked
  spelling.
  Fixture (pnpm 12.16.1,
  `virtualStoreType: global`,
  a project reached through a link three folders deep,
  the store beside the link):
  an install from a logical working folder,
  `--dir <link spelling>` from elsewhere,
  and `--dir <physical>` all wrote the same eleven-`..` link text and `require('dep-a')` worked in each.
  pnpm used the physical working folder in all three.
  That did not reproduce the failure for a symlinked project root.
  Handling:
  the configuration keeps one spelling,
  the link `/var/home/user/Monochromatic` (not the third spelling `/home/user/Monochromatic`);
  after cut-over run `pnpm install` in a fresh worktree and then
  `find node_modules -xtype l` in it,
  which must print nothing;
  the existing store setting `${HOME}/.local/share/pnpm/store` is unchanged.
  Not measured:
  a real install in the real tree (3 GiB).
- Worktree `.git` and `gitdir` files.
  340 absolute paths spelled `/var/home/user/Monochromatic/...` resolve through the link,
  so the copy works without `git worktree repair`.
  Measured (scratch repository):
  `git worktree repair` run from a linked spelling reported ".git file incorrect" and rewrote the worktree's `.git`
  file from the link spelling to the physical one,
  and after a rename of the main folder `repair <worktree path>` fixed a broken pair.
  Handling:
  `git worktree repair` is not run during the cut-over and not before the owner decides rollback is no longer needed:
  it rewrites the pointers of every worktree,
  including the external ones,
  so a rollback would have to undo it
  (the reverse migration does,
  section "Rollback").
  Afterwards it is optional,
  but it removes the third spelling from every `gitdir` file and from `git worktree list`.
  Git writes the physical spelling for every new `worktree add`.
- Harness project folders.
  Claude Code keys `~/.claude/projects/<folder>` by the working directory;
  Node's `process.cwd()` returns the physical path (measured in the fixture),
  and Claude Code is a Node program,
  so new sessions in the repository are expected to key as `-var-mnt-encrypted-repository-Monochromatic`;
  the first session after cut-over confirms it.
  (Existing folders named `-var-mnt-encrypted-temp-agent-...` fit the same rule but may also come from sessions
  started at the physical path.)
  The old folder `-var-home-user-Monochromatic` holds the history (10716 entries).
  `~/.claude.json` has a `projects` entry for `/var/home/user/Monochromatic` with trust and MCP settings;
  `~/.pi/agent/sessions/--var-home-user-Monochromatic--` is keyed the same way.
  Handling,
  in this order and only after the first session after cut-over has shown which key the harness uses:
  1.  Stop every writer of the shared files (no `claude`,
      Pi or Codex session running anywhere,
      including sessions outside this repository).
  2.  Back up `~/.claude.json` and the old project folders (a reflink copy is enough).
  3.  Require that the destination folder does not exist (`test ! -e <new>`);
      if it exists,
      merge by hand file by file,
      because `mv <old> <existing-dir>` would nest the old folder inside it.
  4.  Rename the old Claude folder and the Pi folder to the new names.
  5.  Copy the `projects` entry in `~/.claude.json` to the new key with a scratch `.ts`
      (read the file,
      compare its modification time and size to the values read at the start,
      write a temporary file
      beside it,
      compare again,
      then rename it over the original;
      abort without writing if either changed).
  Check by key names only.
  Renaming a folder does not rewrite paths embedded inside the session files;
  the old spelling stays in them and still resolves through the link.
  Which spelling the harness uses is unverified until then.
- kopia.
  The source path is unchanged (W12).
- Editor workspaces.
  JetBrains,
  Helium,
  Lapce and others keep the old spelling;
  it resolves through the link.
  Open the project through the link,
  and expect indexes keyed by whichever spelling each IDE is given.
- Containers.
  Bind sources recorded as the link spelling resolve when mounted;
  checked with W13.
- Permission rules.
  `.claude/settings.local.json` has 134 occurrences of the path,
  matching command text and path patterns.
  Whether Claude Code matches path rules against the link spelling or the physical one is not measured;
  the first session after cut-over shows it as new permission prompts.
- Files that name the path inside tracked docs (191 files,
  420 occurrences) stay as written (rule `WR6`).

## Quiesce and freeze

Goal:
nothing changes in the repository between the baseline and the swap,
and nothing is lost.
Agents are not killed.
Whether the quiet state was reached is decided by evidence taken after the freeze,
not by a scan taken before it.

The mechanisms and their measured limits:

- Cooperative drain:
  sessions are asked to finish.
  No signal is sent.
- Freeze of each subvolume:
  `btrfs property set -ts <subvolume> ro true`.
  Measured on a scratch subvolume on the 860 as the owner of that subvolume (no root):
  creating a file,
  opening a file for write and renaming a file all failed with `Read-only file system`,
  and `ro false` restored writes.
  The same property is what the script sets on the main checkout and on every nested subvolume.
  Limit (measured):
  a write through a file descriptor that was opened for write before the freeze still succeeded and reached the file.
  The freeze therefore stops Git's lock-file-and-rename writes,
  new files and new opens,
  but not a long-lived writer such as a log being appended.
- Read-only bind mount for the 16 plain-directory external worktrees,
  which cannot be frozen as subvolumes:
  `mount --bind <dir> <dir>` then `mount -o remount,ro,bind <dir>` as root.
  Measured in a user namespace:
  new files and removals failed with `Read-only file system`,
  and a descriptor opened for write before the bind could still write.
  Same limit.
- Holder scan before and after the freeze:
  because of the limit,
  the scan is the check for writers that already hold a descriptor.
  Any holder blocks the cut-over (below).
- An accident worth recording:
  while probing the property at 2026-11-07T06:52:00-05:00 the agent that wrote this revision ran
  `btrfs property set -ts /var/home/user/Monochromatic ro true` on the real main checkout by mistake and ran
  `ro false` within about a minute (`ro=false` confirmed at 2026-11-07T06:52:28-05:00;
  a write test then succeeded).
  `.git` is inside that subvolume,
  so a commit or any other write to the main checkout in that minute would have been refused with
  `Read-only file system`.
  Measured afterwards:
  the commit service's finished records (`*.done`) between 06:50 and 06:55 start at 06:52:29,
  the first seconds after the flag was cleared,
  so no commit landed while it was set;
  `journalctl --user --grep 'Read-only'` for 06:50 to 06:55 printed no entries;
  a commit that was refused would have been answered to its caller and may not be journaled,
  so a failed attempt in that minute can be neither confirmed nor excluded.
  The agent changed no file.
  This was a mutation of the real repository's flags,
  reverted,
  and it is reported as such.
  The freeze is a destructive-adjacent action and happens only in the owner-run cut-over.

Steps:

1.  List the holders.
    The user can see only its own processes:

    ```bash
    # Holders by working folder and open file, repository and every nested worktree. Not the shadowed find.
    /usr/bin/find /proc -maxdepth 3 \( -name cwd -o -path '/proc/*/fd/*' \) -lname '/var/home/user/Monochromatic*' -printf '%p %l\n'
    ```

    Processes of other users (rootless containers run under subuids) are listed by root with the same command.
    Expect:
    the terminals,
    `claude` sessions,
    MCP servers,
    the Codex server,
    the IDE,
    and `rust-*` runs (30 processes with the repository as working folder at 2026-11-07T05:47:00-05:00).
    The same scan runs for `/var/home/user/worktrees` and for the 77 worktrees on the 860,
    because their processes write the common `.git`.
2.  Ask each session to finish,
    with a deadline of the owner's choosing.
    At the deadline run the scan again.
    If a holder remains,
    the cut-over does not happen and the holder list is the report.
    Nothing is signalled.
3.  Stop the user units that run from the repository
    (`systemctl --user stop claude-cpu-watch.service` and the `cli-git@*` sockets and services).
    They start again after the swap.
    The services are socket-activated,
    and the wrapper never starts itself at login.
    Check the wrapper's own state first:
    for every loaded `cli-git@*.service` (15 at 2026-11-07T06:03:00-05:00),
    its state folder `~/.local/state/cli-git/service/<instance>` must have no file in `busy/` and none in
    `push-retries/`,
    and `git monochromatic-git-policy-cli service status` must answer.
    List `.git/cli-git-transactions/*.pending` (3 on 2026-11-07).
    A non-empty `push-retries/` would resume under the old instance name and never run after the move,
    so the owner pushes those branches by hand and the check is repeated.
4.  Freeze (`move-repository.ts --freeze`,
    or the lines it prints),
    then scan the holders again.
    A holder with a descriptor in the tree means a writer that the freeze cannot stop:
    thaw (`ro false` on every subvolume the script froze,
    printed by the script),
    report the holder,
    and stop.
5.  Capture the baseline now,
    after the freeze,
    with the script below.
    Values taken earlier legitimately differ because sessions commit while they finish.

The baseline script is `/var/home/user/Monochromatic/package/repository-move/cli/src/list-uncommitted-work.ts`.
For each line of `git worktree list --porcelain` it runs
`/usr/bin/git --no-optional-locks -C <worktree> status --porcelain=v1 --untracked-files=all`,
counts changed and untracked files,
reports an unfinished merge,
rebase,
cherry-pick,
revert or bisect (files in the Git directory) and a held `index.lock`,
and records `HEAD` and the SHA-256 of the index file.
`--no-optional-locks` means it writes nothing.
Run:

```bash
systemd-run --user --scope --property=CPUQuota=100% --property=MemoryMax=1G --property=TasksMax=64 --property=IOWeight=10 \
  timeout 3000 node /var/home/user/Monochromatic/package/repository-move/cli/src/list-uncommitted-work.ts \
  /var/home/user/Monochromatic /var/home/user/repository-move-state/wizard/baseline-frozen.tsv 180
```

The copy carries uncommitted files and the old tree is kept,
so nothing is discarded.
The table exists for two checks:
any row with an unfinished operation or a held `index.lock` is settled by its owner before the freeze,
and the table taken on the staged tree must equal this one byte for byte (the paths are equal because the
verification below runs under a bind mount).

Measured scan before the freeze was designed,
run from 2026-11-07T06:15:00-05:00 to 2026-11-07T06:26:00-05:00 in a bounded scope with the first version of the script
(without the `HEAD` and index columns):
340 worktrees listed and none missing or failing.
44 have something a move would strand:
32 of the 246 nested worktrees,
7 of the 71 on the 860,
3 of the 6 reached through `~/temp/agent`,
and 2 of the 16 under `~/worktrees`;
the main checkout is clean.
In total 350 changed and 5188 untracked files.
No worktree has an unfinished merge,
rebase,
cherry-pick,
revert or bisect,
and none has a held `index.lock`.
The largest is 782 files in
`/var/home/user/temp/agent/music-player-theme-compose-prototype` (on the 860,
so it does not move).
The table is `/var/home/user/temp/agent/dangling/real-uncommitted.tsv`.
Only the dirty nested worktrees and the 2 dirty external ones move.
This is a snapshot:
other sessions keep working.

## Capacity policy

The sentence "at most 263.6 GiB" of the earlier revision is withdrawn.
263.6 GiB is a user-level measurement of the nested tree with sharing counted once,
with 126 unreadable files omitted,
taken while sessions kept writing.
It excludes metadata overhead,
sharing that the send strategy does not reproduce,
the 16 external worktrees,
and anything pinned by retained snapshots.
Sharing is reproduced only as far as the clone sources allow,
and a snapshot that is kept pins old extents while the live tree changes.

- Estimate to be measured as root before the copy:
  `btrfs filesystem du --summarize --raw` of the main checkout (counts the root-owned files),
  plus `du --summarize --one-file-system` of each of the 16 external worktrees (they are plain directories on a
  different filesystem,
  so a copy cannot share their extents with anything),
  plus `btrfs filesystem usage` of both filesystems for data and metadata allocation and unallocated space.
  External worktrees measured at user level so far:
  `du --summarize --one-file-system --block-size=1M` over the 16 registered worktrees under `~/worktrees` (run from
  2026-11-07T07:00:00-05:00 to 2026-11-07T07:13:00-05:00 in a bounded scope) summed to 74.3 GiB (76076 MiB):
  `translation-repair` 40.7 GiB,
  `verify-coverage` 3.6 GiB,
  and the fourteen `tr-*` folders 2.0 to 2.7 GiB each.
  The folder `readonly-result-provenance` (3 MiB) is not a registered worktree and is carried as plain data.
  `du` counts shared extents once per path,
  so any sharing among these folders on the NVMe is not visible in the sum;
  on the 860 none is kept,
  so 74.3 GiB is the expected footprint,
  before compression differences.
- Pre-flight rule:
  the destination's free space after subtracting the estimate and a margin of one quarter of the estimate
  (the plan author's choice) must be at least the script's `--min-free-gib` (default 150,
  the plan author's choice);
  otherwise the copy does not start.
  The NVMe's free space (350G at 2026-11-07T05:47:00-05:00) must stay above 100 GiB (author's choice)
  while source snapshots exist.
- During the copy:
  the script checks the destination's free space before every subvolume and stops with every earlier step recorded.
  The owner watches the NVMe with `df` between passes.
- What is retained and for how long:
  the source snapshots `s###` and `f###` on the NVMe,
  the received `s###` and `f###` on the 860,
  and the frozen old tree.
  They pin extents while anything changes.
  The old tree and the source snapshots are frozen or read-only,
  so they pin little after the freeze;
  the received first-pass copies `s###` become unnecessary after the final verification and are deleted by the owner
  once the validation window has closed.
- Cancellation:
  the script stops at a step boundary;
  the state folder says what completed;
  partial receives are named for the owner to delete.
  The source tree is untouched except for the freeze,
  which the script prints the thaw for.
- The earlier claim that retained received copies "cost almost no space" is corrected:
  they share extents with the live tree at the moment of the snapshot and cost more as either side changes.
- "Commits should never fail" is not promised against disk pressure:
  no mechanism reserves space for commits,
  and the plan does not claim one.

## The copy

Method M1 (send and receive with clone sources,
section "Copy methods") in two passes,
as root,
for every subvolume in the reconciled inventory;
plain directories (the 5 plain-directory worktrees inside the main checkout,
and the 16 external worktrees) are handled as described below.

The root steps of this section,
the fixture test and the cut-over run through
[the guided runner](../runbook/repository-on-the-860-drive-guided-runner.md).

The tool `package/repository-move/cli/src/move-repository.ts` replaces the print-only generator of the earlier
revision.
Without `--run` it prints every step and runs nothing;
with `--run` it needs root,
runs the steps in order,
and stops at the first failure.
It was exercised against a btrfs test double for control flow (seven cases listed in the review table,
item 9);
the real `btrfs send` and `receive` are unverified until the root fixture test below.

What the script does for each subvolume (main first,
then the nested ones in sorted order):

1.  A pre-flight:
    the inventory (see below),
    the destination's free space,
    a state folder with mode 0700.
2.  `btrfs subvolume snapshot -r <source> <snapshot folder>/<name>`,
    then a completion record
    (`<name>.snapshot.json` with the subvolume UUID).
3.  Inside `systemd-run --scope` and `timeout --kill-after=30 <deadline>`,
    `bash -o pipefail -c` runs `btrfs send` (with `-p <first-pass parent>` and one `-c <earlier snapshot>` per clone source)
    piped into `btrfs receive <received folder>`.
    The pipeline fails if either side fails.
4.  Verification of the receive:
    `btrfs property get -ts <received> ro` must print `ro=true`,
    and `Received UUID` of the received subvolume must equal `UUID` of the sent snapshot.
    Only then is `<name>.received.json` written (written to a temporary file and renamed).
5.  An existing `<received>/<name>` with no completion record stops the script with the exact
    `btrfs subvolume delete` for the owner to run.
    The script never deletes anything.
6.  Final pass only:
    a copy is placed only if its completion record exists.
    The empty stub of a nested subvolume in the received parent is removed with `rmdir`
    (measured on btrfs as an ordinary user:
    the stub is an empty directory of inode 2,
    `rmdir` succeeds,
    and `btrfs subvolume snapshot` to that path succeeds and gives a root with inode 256),
    then `btrfs subvolume snapshot <received> <final path>`,
    then a check that the placed subvolume is writable.

Inventory (review item 2).
Before anything is generated,
the owner runs as root:

```bash
# R0: every subvolume of the NVMe filesystem; the script keeps those below the main checkout, at any depth.
sudo btrfs subvolume list -u /var/home > /var/home/user/repository-move-state/inventory-nested.txt
sudo findmnt --list --output TARGET,SOURCE,FSTYPE | grep -F /var/home/user/Monochromatic
```

`btrfs-subvolume(8)` documents `list -o` as not recursive,
so the command lists the whole filesystem instead (paths are relative to the top level,
so the checkout appears as
`home/user/Monochromatic`).
The script reads the file with `--inventory`,
keeps only the lines below `--inventory-main` (`home/user/Monochromatic`) at any depth and skips every other line,
maps the `home/` prefix of each kept path to `/var/home/` (the prefix is a flag,
`--inventory-prefix`),
and refuses to run when the filesystem's list and Git's registered subvolumes differ in either direction,
printing the differences.
A subvolume Git does not know needs an owner-approved disposition (copy it with its own entry or exclude it in
writing) before the copy runs.
Mount boundaries are listed by the second command;
any mount inside the tree needs its own disposition.
The path mapping of `btrfs subvolume list` output is unverified (root only);
the script stops on a line it cannot map.

Invocation (print first,
read the output,
then run):

```bash
mkdir -m 0700 /var/home/user/repository-move-snapshots
sudo mkdir -m 0700 /var/mnt/encrypted/repository
sudo chown user:user /var/mnt/encrypted/repository
mkdir -m 0700 /var/mnt/encrypted/repository/received-read-only
node /var/home/user/Monochromatic/package/repository-move/cli/src/move-repository.ts --pass=first --main=/var/home/user/Monochromatic \
  --snapshots=/var/home/user/repository-move-snapshots --received=/var/mnt/encrypted/repository/received-read-only \
  --final=/var/mnt/encrypted/repository/Monochromatic --state=/var/home/user/repository-move-state \
  --inventory=/var/home/user/repository-move-state/inventory-nested.txt --inventory-main=home/user/Monochromatic
sudo /absolute/path/of/node /var/home/user/Monochromatic/package/repository-move/cli/src/move-repository.ts --run --pass=first ...same arguments...
```

Worked example,
the first lines of the first pass in print mode for the main checkout and the first worktree
(each `systemd-run` line is one line in the output):

```bash
btrfs subvolume snapshot -r /var/home/user/Monochromatic /var/home/user/repository-move-snapshots/s000
systemd-run --scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 --property=IOWeight=10 \
  timeout --kill-after=30 14400 bash -o pipefail -c 'btrfs send /var/home/user/repository-move-snapshots/s000 | btrfs receive /var/mnt/encrypted/repository/received-read-only'
btrfs subvolume snapshot -r /var/home/user/Monochromatic/.claude/worktrees/add-explains-slow-route /var/home/user/repository-move-snapshots/s001
systemd-run --scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 --property=IOWeight=10 \
  timeout --kill-after=30 14400 bash -o pipefail -c 'btrfs send -c /var/home/user/repository-move-snapshots/s000 /var/home/user/repository-move-snapshots/s001 | btrfs receive /var/mnt/encrypted/repository/received-read-only'
```

The final pass prints,
for the same two,
the freeze lines when `--freeze` is given,
a new snapshot `f000`,
a send with `-p .../s000`,
the placement of `f000` at `/var/mnt/encrypted/repository/Monochromatic`,
and for the worktree a send with `-p .../s001 -c .../f000`,
an `rmdir` of the stub and the placement of `f001` at
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/add-explains-slow-route`.

The bounds (`CPUQuota=200%`,
`MemoryMax=2G`,
`TasksMax=32`,
`IOWeight=10`) are sized to one stream and chosen by the plan author,
not measured.
No synthetic load is generated;
the copy is real work.

Clone sources:
the default `--clone-sources=all` lists every earlier snapshot,
`main` only the first.
The first twenty worktree sends are timed under both and the faster one that keeps the footprint under the capacity
threshold is used (adopted decision).

External worktrees under `~/worktrees` (adopted,
review item 6).
They are plain directories,
so `btrfs send` cannot move them and nothing of theirs shares extents with anything on the 860.
For each one the owner runs as root,
in the same two passes:

```bash
# Pass one while everything runs: a new subvolume per external worktree, then a root rsync into it.
sudo btrfs subvolume create /var/mnt/encrypted/repository/external-worktrees/<name>
sudo systemd-run --scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 \
  --property=IOWeight=10 timeout --kill-after=30 7200 rsync --archive --hard-links --acls --xattrs --numeric-ids \
  --one-file-system /var/home/user/worktrees/<name>/ /var/mnt/encrypted/repository/external-worktrees/<name>/
```

Pass two repeats the `rsync` after the read-only bind mount of the source.
`/var/mnt/encrypted/repository/external-worktrees` is created with mode 0700 first.
After the swap `~/worktrees` is moved aside like the repository and a link
`~/worktrees -> /var/mnt/encrypted/repository/external-worktrees` takes its place,
so the old paths keep working.
The `.git` files of these worktrees name `/var/home/user/Monochromatic/.git/worktrees/<id>`,
and the administrative `gitdir` file in the main tree names `/var/home/user/worktrees/<name>/.git`;
both resolve through the two links.

### Fixture test before the real run (root, throwaway loop files)

Facts that are still unverified for the real copy and that decide it:
that `-c` keeps sharing between sibling worktrees,
that an incremental `-p` send with clone sources reproduces changed content,
that the reassembled tree equals the source subvolume by subvolume,
that an interrupted receive is detected on retry,
that a corrupted stream fails,
and that a mount can be placed over an immutable directory.
Already measured as an ordinary user on the 860's btrfs:
stub removal and placement (inode 2 stub,
`rmdir`,
snapshot to the path),
the `rsync --one-file-system` boundary behavior,
and the `ro` property freeze.

```bash
# F1: a fresh private folder; nothing is overwritten and only what this invocation creates is removed.
set -o noclobber
fixture=$(mktemp --directory /var/home/user/temp/agent/send-fixture.XXXXXX)
: > "$fixture/source.img"
: > "$fixture/dest.img"
truncate --no-create --size=3G "$fixture/source.img" "$fixture/dest.img"
sudo mkfs.btrfs --quiet "$fixture/source.img"
sudo mkfs.btrfs --quiet "$fixture/dest.img"
sudo mkdir "$fixture/source" "$fixture/dest"
sudo mount -o loop "$fixture/source.img" "$fixture/source"
sudo mount -o loop "$fixture/dest.img" "$fixture/dest"
sudo mkdir "$fixture/dest/recv"
mkdir "$fixture/snap" "$fixture/state"
findmnt --noheadings --output SOURCE,TARGET "$fixture/source" "$fixture/dest"
```

The mount identity line is read before the next step;
a missing line stops the test.

```bash
# F2: a main checkout that is a subvolume, a Git repository, and two worktrees that are subvolumes.
sudo btrfs subvolume create "$fixture/source/main"
sudo chown user:user "$fixture/source/main"
git -C "$fixture/source/main" init --quiet
head --bytes=300M /dev/urandom > "$fixture/source/main/big"
git -C "$fixture/source/main" add big
git -C "$fixture/source/main" -c user.name=t -c user.email=t@t commit --quiet --message base
mkdir --parents "$fixture/source/main/.claude/worktrees"
sudo btrfs subvolume create "$fixture/source/main/.claude/worktrees/a"
sudo btrfs subvolume create "$fixture/source/main/.claude/worktrees/b"
sudo chown user:user "$fixture/source/main/.claude/worktrees/a" "$fixture/source/main/.claude/worktrees/b"
git -C "$fixture/source/main" worktree add --force --quiet "$fixture/source/main/.claude/worktrees/a" -b a
git -C "$fixture/source/main" worktree add --force --quiet "$fixture/source/main/.claude/worktrees/b" -b b
cp --reflink=always "$fixture/source/main/big" "$fixture/source/main/.claude/worktrees/a/big.copy"
cp --reflink=always "$fixture/source/main/big" "$fixture/source/main/.claude/worktrees/b/big.copy"
sudo btrfs filesystem du --summarize "$fixture/source/main" | tee "$fixture/source-baseline.txt"
sudo btrfs subvolume list -o "$fixture/source/main" > "$fixture/inventory.txt"
```

Expected:
each worktree's `big.copy` is a reflink of `big`,
so the source shares one 300 MiB extent three ways,
and `source-baseline.txt` records the source's own total and shared figures for F4 to compare with.

```bash
# F3: the first pass with the real script, then a change, then the final pass with a freeze.
sudo /absolute/path/of/node /var/home/user/Monochromatic/package/repository-move/cli/src/move-repository.ts --run --pass=first --main="$fixture/source/main" --snapshots="$fixture/snap" \
  --received="$fixture/dest/recv" --final="$fixture/dest/final" --state="$fixture/state" --inventory="$fixture/inventory.txt" \
  --inventory-prefix="<path prefix reported by the list>=$fixture/" --deadline-seconds=600 --min-free-gib=1
echo changed >> "$fixture/source/main/.claude/worktrees/b/extra.txt"
git -C "$fixture/source/main" -c user.name=t -c user.email=t@t commit --allow-empty --quiet --message after
node move-repository.ts --run --pass=freeze ...same arguments...   # as the user: the owner of a subvolume sets ro
sudo /absolute/path/of/node move-repository.ts --run --pass=final ...same arguments...
```

Both runs must end `finished`,
and `ls "$fixture/state"` must show a `received` and a `placed` record for each of `f000`,
`f001`,
`f002`.

```bash
# F4: compare every subvolume with the boundary policy, and check sharing and received state.
for path in main main/.claude/worktrees/a main/.claude/worktrees/b; do
  sudo rsync --archive --checksum --dry-run --itemize-changes --delete --one-file-system --omit-dir-times \
    "$fixture/source/$path/" "$fixture/dest/final/${path#main}/"
done
sudo btrfs property get -ts "$fixture/dest/recv/f001" ro
sudo btrfs subvolume show "$fixture/dest/recv/f001" | grep 'Received UUID'
sudo btrfs filesystem du --summarize "$fixture/dest/recv" "$fixture/source/main"
df --block-size=1M "$fixture/source" "$fixture/dest"
```

Each `rsync` must print nothing.
`ro=true` and a `Received UUID` equal to the source snapshot's `UUID` are required.
The destination's total and `Set shared` must match `source-baseline.txt` within metadata overhead;
if the destination holds three separate copies of the 300 MiB extent where the source holds one shared extent,
the clone sources did not keep sharing and the method is wrong for this repository.

```bash
# F5: an interrupted receive is detected, and a corrupted stream fails on a flipped known byte.
sudo /absolute/path/of/node move-repository.ts --run --pass=first ...arguments with --deadline-seconds=1 on a fresh state folder...
sudo /absolute/path/of/node move-repository.ts --run --pass=first ...arguments with the normal deadline, same state folder...
sudo sh -c "btrfs send '$fixture/snap/s001' > '$fixture/stream.bin'"
python3 -I -c "import sys; p=sys.argv[1]; f=open(p,'r+b'); f.seek(100000); b=f.read(1); f.seek(100000); f.write(bytes([b[0]^255]))" "$fixture/stream.bin"
sudo mkdir "$fixture/dest/recv-bad"
sudo btrfs receive "$fixture/dest/recv-bad" < "$fixture/stream.bin"
```

The first run with a one-second deadline must stop;
the second must refuse with the exact `btrfs subvolume delete` for the partial copy;
the owner runs it and the third run must finish.
The last `receive` must fail with a checksum error.

```bash
# F6: the staged-verification pattern: a private namespace with the staged tree bind-mounted over the source path.
sudo unshare --mount --propagation private bash -euo pipefail -c "
  mount --bind '$fixture/dest/final' '$fixture/source/main'
  git -C '$fixture/source/main' worktree list
  git -C '$fixture/source/main' fsck --full
  node /var/home/user/Monochromatic/package/repository-move/cli/src/list-uncommitted-work.ts '$fixture/source/main' '$fixture/staged.tsv' 60
"
```

`worktree list` must show the same paths as before the namespace and `staged.tsv` must be a table of the staged tree.

```bash
# F7: can a mount cover an immutable directory? (the destination image is unmounted first; it is mounted only once)
sudo umount "$fixture/source" "$fixture/dest"
sudo mkdir "$fixture/mp"
sudo chattr +i "$fixture/mp"
sudo mount -o loop "$fixture/dest.img" "$fixture/mp"
lsattr -d "$fixture/mp"
sudo umount "$fixture/mp"
lsattr -d "$fixture/mp"
sudo chattr -i "$fixture/mp"
```

The mount must succeed;
`lsattr -d` while mounted shows the mounted filesystem's flags (no `i`);
after the unmount the directory shows `i` again.

```bash
# F8: cleanup of exactly what F1 created.
sudo rmdir "$fixture/mp" "$fixture/source" "$fixture/dest"
rm -- "$fixture/source.img" "$fixture/dest.img"
```

### Verification of the real copy

All checks run in a private mount namespace in which the staged tree is bind-mounted over the real path,
so every `.git` file,
`gitdir` file and absolute path resolves into the copy and nothing reaches the source by accident
(measured on a fixture in a user namespace:
a staged worktree's `.git` file names the source path;
with the staged tree bind-mounted over it,
`git status` and `git worktree list` read the staged tree):

```bash
sudo unshare --mount --propagation private bash -euo pipefail -c '
  mount --bind /var/mnt/encrypted/repository/Monochromatic /var/home/user/Monochromatic
  # the external worktrees are bound the same way: mount --bind <staged external folder> /var/home/user/worktrees
  ...checks...
'
```

1.  `git fsck --full` on the staged main checkout (its own scope,
    with a deadline).
2.  The table from `list-uncommitted-work.ts` on the staged tree equals `baseline-frozen.tsv` byte for byte
    (`diff` is empty).
    That compares,
    for every worktree,
    the registration,
    `HEAD`,
    the SHA-256 of the index,
    the counts of changed and untracked files,
    and any unfinished operation.
    Paths are equal because of the bind.
3.  `git for-each-ref | sha256sum` equals the value taken on the frozen source.
4.  Per subvolume,
    `rsync --archive --checksum --dry-run --itemize-changes --delete --one-file-system --omit-dir-times` of the
    source's final read-only snapshot against the placed subvolume prints nothing.
    `--one-file-system` keeps the comparison at the subvolume boundary (measured:
    without it `*deleting   nested/inner.txt` is reported for a correctly assembled tree);
    `--omit-dir-times` ignores the directory times that stub removal changes.
    Nested subvolumes are compared by their own run.
    Run it for every subvolume if time allows;
    at minimum for main,
    the six largest and six smallest by size,
    and every worktree that has a non-clean row in the baseline.
    The 5 plain-directory worktrees are inside main's comparison.
5.  Sharing kept:
    `sudo btrfs filesystem du --summarize --raw /var/mnt/encrypted/repository/Monochromatic` is compared with the same
    command on the frozen source,
    both as root.
    A footprint much larger than the source's means sharing was lost and the swap does not happen.
6.  File counts equal for every subvolume
    (`find <path> -xdev -type f | wc -l` on source snapshot and placed copy).
7.  Root-owned files survived:
    `sudo find /var/mnt/encrypted/repository/Monochromatic -user root -name '*.log.jsonl' | wc -l` is not zero in the two
    worktrees that had them.
8.  The external worktrees:
    `rsync --archive --checksum --dry-run --itemize-changes --one-file-system` of each source (read-only bind) against
    its copy prints nothing.
9.  After the swap,
    still before release,
    the same table and `git fsck` run through the real link (second end-to-end validation).

Any difference stops the runbook with the old tree untouched.

## Cut-over

Phases.
Until the swap the NVMe tree is the repository.

1.  Phase 0,
    preparation (no effect on running work).
    Commit and push everything that matters (rule `RBT`).
    Run R0 (inventory),
    R1 and R2 (mountpoint guard),
    F1 to F8 on the loop files,
    and the capacity measurement.
    Create the folders and add `user_subvol_rm_allowed`:

    ```bash
    sudo mount -o remount,user_subvol_rm_allowed /var/mnt/encrypted
    findmnt --target /var/mnt/encrypted --output OPTIONS
    sudo systemctl edit --full mount-860.service
    ```

    In the unit's `mount -o compress=zstd:3,noatime` add `,user_subvol_rm_allowed`.
    Install the work items (they land through the repository and a toolchain update) with the identity file in phase
    `before`.
    Copy `package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service` over the symlinked unit file in
    `~/.config/systemd/user` (remove the link first) and apply W8's edits,
    then `systemctl --user daemon-reload`.
2.  Phase 1,
    first pass while everything runs:
    `move-repository.ts --pass=first` and the external worktrees' first `rsync`.
3.  Phase 2,
    quiesce:
    holder scan,
    wrapper state check,
    stop the repository's user units,
    freeze,
    second holder scan,
    baseline
    (section "Quiesce and freeze").
4.  Phase 3,
    final pass and staged verification:
    `move-repository.ts --pass=final` (with the freeze already done),
    the external worktrees' second `rsync`,
    then every check of the section "Verification of the real copy".
    If anything differs,
    thaw and stop.
5.  Phase 4,
    swap and activation,
    in the same quiet window and as the user:

    ```bash
    mv /var/home/user/Monochromatic /var/home/user/Monochromatic.nvme-before-860-<timestamp>
    ln -s /var/mnt/encrypted/repository/Monochromatic /var/home/user/Monochromatic
    mv /var/home/user/worktrees /var/home/user/worktrees.nvme-before-860-<timestamp>
    ln -s /var/mnt/encrypted/repository/external-worktrees /var/home/user/worktrees
    ```

    Replace the timestamps with real ones.
    Then write the identity file with phase `after` (the owner's step records the link text,
    the canonical root,
    the 860 filesystem UUID read with `findmnt --noheadings --output UUID --target /var/mnt/encrypted`,
    and the root-commit marker `.git/monochromatic-root-commit`)
    and run `repository-link-check`.
    The old trees stay frozen (`ro true`) and read-only,
    so nothing can write to the stale copy through a habit or a stale path.
    The renames are within one filesystem and atomic;
    the commands leave a short window with no path,
    which is why the window is quiet.
    The old tree's worktrees' `.git` files still name `/var/home/user/Monochromatic/.git`,
    which is now the 860 tree,
    so the old tree is cold storage,
    not a second working repository.
6.  Phase 5,
    validation window (writers are still excluded):
    the second end-to-end validation
    (`git fsck`,
    the table equal to the baseline,
    one test commit in a throwaway worktree made by the wrapper,
    `git worktree list`,
    `mise trust --all` followed by `mise trust --show`,
    one `created` container starts,
    `systemctl --user is-active claude-cpu-watch.service`,
    and a fresh worktree's `pnpm install` followed by `find node_modules -xtype l`,
    which must print nothing).
    Rollback inside this window is the exact fast path.
7.  Phase 6,
    release:
    start the user units,
    announce that sessions may start,
    and keep the old trees until the owner removes them.
    One probe session is started in the repository only to see which project key the harness writes;
    then the harness migration (section "What the canonical path changes") is done with no session running,
    and sessions restart.
8.  Reboot test (commit and push first,
    rule `RBT`).
    Forbidden until every piece of NVMe-resident user data that matters has a verified copy off the NVMe:
    the dirty nested worktrees and the 2 dirty external ones are in the verified copy,
    and the old trees on the NVMe are copies,
    not the only data.
    It also needs every consumer of the 860 stopped by its owner (pcloud,
    backups,
    scratch sessions,
    external
    worktrees),
    because "before opening the repository" does not mean nothing holds the 860.
    The test of a locked drive uses a drop-in instead of stopping the unit:
    `sudo systemctl edit mount-860.service` with `[Service]`,
    an empty `ExecStart=` line and `ExecStart=/bin/true`,
    then a reboot.
    After login report three states separately:
    `systemctl is-active mount-860.service` (`active`),
    `findmnt /var/mnt/encrypted` (nothing),
    and `ls /dev/mapper/crypt_sda` (absent).
    Open a terminal in the repository and run `git status`,
    which must print the plain message in well under a second,
    and `systemctl --user status claude-cpu-watch.service` must show `activating (auto-restart)`.
    Then remove the drop-in,
    run `sudo systemctl daemon-reload`,
    and `sudo systemctl restart --no-block mount-860.service`
    (the unit is still `active` from the `/bin/true` run because it has `RemainAfterExit=yes`,
    so `start` would do nothing;
    its stop path runs first and is harmless with nothing mounted),
    poll `systemctl is-active` for at most 180 s,
    and confirm `repository-link-check` prints ready and the unit becomes active within one retry delay.

## Rollback

The two cases are different because writes after the swap change the state.

- Fast path,
  only inside the validation window of phase 5:
  every writer is still excluded,
  and the 860 tree has received only the test commit in a throwaway worktree.
  Steps:
  confirm the table of the old (frozen) tree still equals `baseline-frozen.tsv`;
  remove the link and the `~/worktrees` link (`rm` of a link removes only the link);
  `mv` the two `.nvme-before-860-<timestamp>` folders back;
  set the identity file's phase to `before`;
  thaw the old subvolumes (`btrfs property set -ts <path> ro false`,
  printed by the script);
  run `repository-link-check`.
  The old canonical path is back,
  so the old instance hashes,
  the old trust links,
  the old harness keys and the old state folders are valid again.
  The throwaway worktree is removed.
- Full path,
  after release (or after any pointer repair,
  harness migration,
  or any write to the 860 tree):
  a reverse migration with the same fail-closed script and the same verification,
  because renaming the old tree back would restore stale data.
  1.  Drain and quiesce again with the same holder scans,
      freeze the 860 tree,
      and capture a new baseline of the 860 tree (the table includes worktrees added since the move).
  2.  Run `move-repository.ts` with the 860 tree as `--main`,
      a new NVMe staging folder as the destination
      (`/var/home/user/Monochromatic.rollback-staging`),
      and a new state folder.
      It copies the main checkout,
      every nested subvolume (including worktrees created after the move),
      and the external worktrees.
  3.  Verify the staging tree against the frozen 860 baseline in the private bind namespace
      (the staging tree bind-mounted over the real path) with the same nine checks.
  4.  Undo pointer changes:
      if `git worktree repair` ran after the move,
      run it again from the staging tree under the bind (it rewrites the pointers to the NVMe spelling),
      restore the harness folders and the `~/.claude.json` entry from the backups taken at the migration
      (the backups are kept until rollback is no longer wanted),
      and re-run `mise trust --all`.
  5.  Swap:
      move the link aside,
      move the staging tree to `/var/home/user/Monochromatic`,
      restore the `~/worktrees` folder the same way,
      set the phase to `before`,
      and run the second end-to-end validation.
  6.  Keep both previous trees (the frozen NVMe tree from the first move and the frozen 860 tree) until the owner
      has validated the restored repository.
- The old trees stay until the owner removes them
  (`btrfs subvolume delete` of each nested subvolume then the root;
  the NVMe mount allows it as the user).
  Removal is read-first:
  compare against the baselines,
  and never delete a tree whose table has an unfinished operation that was not carried over.
- Unverified:
  the reverse migration was not exercised (it needs root);
  F3 to F6 exercise the same script and verification in the forward direction.

## Open question for the owner

One question remains.
Everything else is in the section "Adopted decisions".

### Q1. A bounded maintenance window, against "Commits should never fail."

The cut-over needs a window in which the repository accepts no writes
(the freeze,
the final pass,
the staged verification and the swap).
During it a commit is refused with a plain message that names the cause and the time the window began;
nothing is written and no data is lost.
This is the only part of the plan that conflicts with the standing rule "Commits should never fail."
The length of the window is not known:
it is the duration of the final pass,
which only a root run can measure.
The final pass sends increments,
so its length depends on what changed since the first pass.

- Accept a bounded window and refuse commits during it (the plan's choice):
  pros,
  small change,
  nothing is acknowledged that was not written,
  the freeze is enforced by the kernel;
  cons,
  commits are refused during the window,
  which the owner's rule disallows without an exception,
  and the window length is unmeasured until the first full run.
- Accept commits during the window into a durable queue and land them after the swap:
  pros,
  no commit is ever refused;
  cons,
  a new mechanism in the wrapper that must acknowledge before landing,
  which is the thing the rule forbids doing falsely,
  plus ordering and conflict handling between queued commits and the moved tree;
  weeks of work for a one-time event.
- Do the cut-over only at an owner-chosen idle time with a stated maximum window,
  after the first pass has been timed:
  pros,
  limits the number of refused commits to zero in practice,
  because no session is running;
  cons,
  needs the owner to be present and agents to stop for the duration.
- Ranking:
  accept a bounded window at an owner-chosen idle time > accept a window without a time > durable queue.
  The first adds to the plan's choice only the timing and costs nothing;
  the second leaves the length of the refusal unbounded in practice;
  the queue costs the most and breaks the acknowledge-only-after-landing rule if done carelessly.

## What was not verified

- Nothing that needs root was run:
  the mountpoint's real owner,
  mode and flags,
  whether a mount covers an immutable directory (F7),
  the root inventory (R0) and its path mapping,
  the real `btrfs send` and `receive` including `-c` sharing and `-p` increments,
  and the reverse migration.
- The user-level fixtures used tmpfs in a user namespace or the user's own scratch subvolumes on the 860,
  not the real mountpoint and not the real repository.
  Their results for ownership and mode are the kernel's general behavior.
- The test double for `btrfs` proves the script's control flow
  (stop on failure,
  completion records,
  refusal of a partial receive,
  inventory reconciliation),
  not the behavior of the real commands.
- Subvolumes nested deeper inside a worktree:
  the depth-3 search of the main checkout and the equality of the directory list with Git's registry found none,
  but only `btrfs subvolume list -o` as root is complete.
- The commit service's behavior when its repository vanishes mid-request,
  `cargo`,
  `pnpm` and `search-shadow` mid-run failures,
  Codex's `--cd`,
  the harness's handling of a linked project folder and which project key it writes,
  `/mcp reconnect all` recovering a server that missed a start,
  permission-rule matching across the link,
  IDE index keys,
  and kopia's first incremental size after the move.
- A real `pnpm install` in the real tree through the link.
- Uncommitted work changes while other sessions run:
  the scan is one snapshot and is repeated after the freeze.
- The systemd source was read at the `main` branch,
  not at the installed 262.1 tag (history section).
- Scratch subvolumes `svfix`,
  `svfix-snap`,
  `svfix-snap-ro`,
  `svfix-asm` and the nested subvolumes inside `svfix` and `svfix-asm` in `/var/home/user/temp/agent/dangling`
  were deleted without root (emptied,
  then `rmdir`),
  so none remain.
