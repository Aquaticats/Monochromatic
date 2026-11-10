# What a btrfs transaction commit costs the commands a person waits on

Status:
measured and partially changed on branch `fix/git-policy-cli-never-waits-on-btrfs-commits`,
from the owner's question of 2026-11-10:
"How about making our wrapper not wait on btrfs-cleaner and btrfs-transactions?"
The rest of what this document proposes is not built:
the snapshot taken ahead of an add,
the hand-over of subvolume deletion,
and the bound on a stuck btrfs call.

The owner's words this work answers:
"nothing should ever wait forever or indefinitely",
"Agents should never wait on each other on commits or merges.",
"Commits should never fail.",
"commits vs native git should not take 5s longer.",
and, for worktrees,
"60s is fine for adding worktrees".

The sync rule applied here is the coordinator's, set with the advisor on 2026-11-10 under the owner's
"I think you can decide yourself on which fsync to remove and not have to ask me. Or do it with the advisor.":
native Git is the baseline,
a plain `git commit` on this machine makes no sync,
so a commit's answer waits on no sync of the wrapper's own;
and every sync removed needs a test that leaves its file empty,
torn or absent.

## The volume this was measured on

`uname -r` is `7.2.15-ogc1.1.fc45.x86_64`;
`/var/mnt/encrypted` is btrfs on `/dev/mapper/crypt_sda`,
a SATA SSD under dm-crypt,
3.7 TB with 31 GB free at 2026-11-10T18:03:00-05:00.
The kernel sources quoted below are the stable tag `v7.2.15` of `fs/btrfs/`,
fetched at 2026-11-10T18:08:19-05:00 from
`https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/plain/fs/btrfs/<file>?h=v7.2.15`
(sha256 of each file is in `${HOME}/temp/agent/pi-runs/btrfs-waits-20261110T180803/kernel/kernel-sources.sha256`).

## Every call on a waiting path that forces, joins or waits for a commit or a log sync

Each line names the wrapper file and function,
the system call,
and the kernel lines that decide what it waits for.
`kind` is what the call costs:
**commit** forces a full transaction commit on the caller's thread,
**log** commits the tree log and waits for it,
**join** attaches to the running transaction and waits only for it to accept writers.

### A worktree add

- `src/native/worktree/worktree_snapshot.rs:337`, `take_snapshot`: `BTRFS_IOC_SNAP_CREATE_V2`
  through `src/native/subvolume/fs_btrfs_ioctl.rs:365-373`, `snapshot_create`.
  **kind: commit**, and a wait for all writeback of the source first.
  Kernel: `btrfs_ioctl_snap_create_v2` (`ioctl.c:1235`) reaches `__btrfs_ioctl_snap_create` (`ioctl.c:1145`),
  then `btrfs_mksnapshot` (`ioctl.c:913`), which runs
  `btrfs_start_delalloc_snapshot(root, false)` and `btrfs_wait_ordered_extents(root, U64_MAX, NULL)`
  (`ioctl.c:931-940`) before it takes `down_read(&fs_info->subvol_sem)` (`ioctl.c:890`).
  `btrfs_mksubvol` (`ioctl.c:866`) calls `create_snapshot` (`ioctl.c:742`), which does
  `trans = btrfs_start_transaction(root, 0);` (`ioctl.c:808`),
  `trans->pending_snapshot = pending_snapshot;` (`ioctl.c:821`)
  and `ret = btrfs_commit_transaction(trans);` (`ioctl.c:823`).
  The snapshot itself is made inside that commit,
  `ret = create_pending_snapshots(trans);` (`transaction.c:2475`),
  so the snapshot's own work is the caller's commit.
- `src/native/worktree/worktree_subvolume.rs:100`, `prepare_subvolume_destination`:
  `BTRFS_IOC_SUBVOL_CREATE` through `fs_btrfs_ioctl.rs:337-343`, `subvolume_create`.
  **kind: join**.
  Kernel: `create_subvol` (`ioctl.c:537`) runs `btrfs_start_transaction(root, 0)` (`ioctl.c:598`)
  and ends it with `btrfs_end_transaction(trans)` (`ioctl.c:726`),
  which only decrements the writer count (`__btrfs_end_transaction`, `transaction.c:1085-1135`);
  `btrfs_end_transaction` does not commit.
- `src/native/worktree/worktree_ladder_run.rs:360`, `discard_stage`,
  and `src/native/worktree_copying/worktree_copy_journal_removal.rs:94`, `remove_stage`:
  `BTRFS_IOC_SNAP_DESTROY` or `BTRFS_IOC_SNAP_DESTROY_V2` through `fs_btrfs_ioctl.rs:450-484`.
  **kind: join**, under a filesystem-wide write lock.
  Kernel: `btrfs_ioctl_snap_destroy` (`ioctl.c:2225`) calls `btrfs_delete_subvolume` (`inode.c:4790`),
  which takes `down_write(&fs_info->subvol_sem)` (`inode.c:4801`),
  runs `btrfs_start_transaction(root, 0)` (`inode.c:4848`) and `btrfs_end_transaction(trans)` (`inode.c:4902`).
  The write lock is taken against every snapshot's `down_read` (`ioctl.c:890`),
  so one add's committing snapshot holds off every subvolume deletion of the whole filesystem.
- `src/native/subvolume/subvolume_index_install_into_new_worktree.rs:90`, `install_index_in_with`,
  through `src/native/subvolume/subvolume_file_write_steps.rs:135`, `write_all_then_sync`.
  **kind: log or commit**, one `fsync` of the new worktree's index.
  Kernel: `btrfs_sync_file` (`file.c:1660` and following) takes `btrfs_start_transaction(root, 0)`
  (`file.c:1685`), then either `btrfs_sync_log(trans, root, &ctx)` (`file.c:1730`),
  whose wait is `wait_log_commit` (`tree-log.c:3335`, `tree-log.c:3346`, `tree-log.c:3184`),
  or the full `btrfs_commit_transaction(trans)` (`file.c:1778`).
- The records of the add itself: `src/native/worktree_copying/worktree_copy_stage_claim.rs:125`, `:237`, `:300`
  and `src/native/foundation/private_storage.rs:351`, `flush_file_when_faithful`, and `:712`, `sync_directory`.
  **kind: log or commit** each,
  so one add makes several of them.

### A commit

- `src/native/landing/service_index_placement.rs:102`, `sync_file`: the flush of the index lock marker.
  Before this change **kind: log or commit**, on the commit's path;
  now queued in the service's background task.
- `src/native/landing/service_index_prestage.rs:197`, `start_with`:
  the marker made ahead was flushed by a job the landing waited for
  (`made_marker`, `service_index_prestage.rs:266`), so a commit waited for another command's flush.
  Now the job hands the flush to the background task and returns.
- `src/native/landing/service_durable.rs:430`, `flush_index`:
  **kind: log or commit** only when `core.fsync` names the index.
  `src/native/config/config_fsync.rs:44-47` reads Git's default set as the objects and the derived metadata,
  not the index, so a repository that sets nothing writes its index unflushed, as native Git does.
- `src/native/landing/service_durable.rs:114`, `sync_all_at_once`, and `:157`, `sync_one_pending`:
  the landing's records, **kind: log or commit**, made by the background task one thread per handle
  unless the landing publishes nothing, in which case `force` (`service_durable.rs:318`) makes them before the answer.
- `src/native/landing/service_land_objects_of_batch.rs:101`: the `core.fsync` value handed to `git fast-import`
  as `-c core.fsync=...`, so the import's own syncs are Git's, not the wrapper's.

### A merge, a landing and a push

- `src/native/conflict/conflict_side_source.rs:343`, `copy_stream`: `to.sync_all()` on the aside copy of a
  conflicting side. **kind: log or commit**.
- `src/native/index/index_reset_publish.rs:99`, `sync_all`, and `src/native/foundation/earlier_log_lines_*.rs`:
  records and logs; **kind: log or commit** each.
- The landing's records and the push hand-over go through the same service syncs as a commit,
  and the landing's own record flush is `src/native/landing/service_durable.rs:396`, `flush_file`.

### What the wrapper never calls

No `BTRFS_IOC_SYNC` (`ioctl.c:5560-5570`, which runs `btrfs_sync_fs`, `super.c:1048-1053`, a full commit),
no `BTRFS_IOC_START_SYNC` or `BTRFS_IOC_WAIT_SYNC` (`ioctl.c:3022-3065`),
no `syncfs`, and no `btrfs` program.
The wrapper's only btrfs ioctls are the five in `src/native/subvolume/fs_btrfs_ioctl.rs`.

## What the live volume spends, from the records the wrapper already keeps

The repository's own add timing log,
300 successful adds ending 2026-11-10T23:09:44.166Z
(`.git/monochromatic-git-policy-cli-worktree-add-timing.jsonl`):

- `snapshotIoctl`: median 7,443.8 ms, p90 18,006.2 ms, max 39,828.5 ms.
- `installIndexFile` (write and flush of the index): median 8,238.0 ms, p90 29,997.3 ms, max 68,530.3 ms.
- `recordBinding`: median 10,361.1 ms, p90 27,817.1 ms.
- `recordDestination`: median 18,518.9 ms, p90 39,828.5 ms.
- `replacePointer`: median 35,030.6 ms (it removes the source's `.git` from the snapshot: deletions, not syncs).
- The whole add: median 575.6 s, p90 7,318.5 s, max 8,128.3 s,
  of which the largest named work is the package repair
  (`cargoRepairWorkspace` median 230.3 s, `cargoClean` median 180.0 s).

The commit timing log,
219 commits ending 2026-11-10T18:11-05:00:
`answerWaitMs` median 307 ms, p90 4,004 ms, max 31,008 ms.

## What was changed

`sync_file` for the index lock marker,
and with it the prestage job's flush,
no longer runs on the commit's thread:
`src/native/landing/service_durable.rs` gained `defer_file`,
which hands the flush to the service's background task and returns,
and `src/native/landing/service_index_placement.rs:102` uses it.
Protection kept:
the marker is still flushed, only later.
What a power cut inside that window loses is a marker that names no holder,
and the step before a landing removes exactly such a file at once:
`src/native/landing/service_index_unheld_removal.rs` prints
"removed the empty index lock ... so nothing proved a program was still using it; the command goes on".
Measured at 2026-11-10T18:26:00-05:00 in the disposable fixture:
an empty `.git/index.lock` with its modification time set to 2020 was removed by the next wrapped commit,
which then landed (`[main 2899a92] probe`, exit 0),
so the file's age is not waited out.

### The numbers, alternating the two binaries on one sandbox

The audit's `s01-concurrent-linked-commits.ts` from
`${HOME}/temp/agent/sonnet-audits/jams-on-add-commit-merge/reproductions`,
eight linked worktrees committing one path each at once,
three rounds,
wrapped arm only,
both arms the same release build but for this change
(`${HOME}/temp/agent/pi-runs/btrfs-waits-20261110T180803/binaries/before` and `.../after`):

- before, 2026-11-10T18:24:30-05:00: n=24, min 311 ms, median 2,750 ms, p90 8,764 ms, max 8,780 ms.
- after, 2026-11-10T18:25:29-05:00: n=24, min 94 ms, median 154 ms, p90 255 ms, max 295 ms.

The sandbox is on the `$HOME` volume (`/dev/dm-1`), a different btrfs device from the repository's,
so these are the change's effect under the same load, not this volume's absolute cost.

The same scenario with the after binary traced
(`strace -f -ttt -T -e trace=renameat2,rename,fsync,fdatasync,unlink`, run 2026-11-10T18:29:26-05:00),
to answer whether the placement still stalls behind background syncs:
2,146 traced processes,
776 rename calls,
200 sync calls.

- The 24 `renameat2` calls that placed an `index.lock`:
  median under 20 ms,
  six slowest 717.1, 40.6, 23.1, 8.3, 7.7 and 5.9 ms.
  Only one passed half a second.
  The audit's untraced before trace had 1.1 s, 2.9 s and 0.65 s for the same placement
  with six concurrent flushes each taking 8 to 12.5 s.
- The 200 syncs that remain: median 1,489.6 ms, p90 5,517.1 ms, max 6,319.7 ms.
  They are the landing's records and the marker, made by the background task,
  and no commit waits for them;
  two of them began inside a placement window.
- Traced wall times are not the commit's cost: the same run's median commit was 5,915 ms traced
  against 154 ms untraced,
  so only the counts and the per-call durations are evidence here.

The worktree add, commit and remove of one fixture on the repository's volume,
with each step's kernel wait point sampled every 20 ms
(`${HOME}/temp/agent/pi-runs/btrfs-waits-20261110T180803/measure.ts`),
before arm at 2026-11-10T18:18-05:00:

- add: 49,443 / 14,272 / 2,780 / 7,471 / 1,693 ms,
  with threads asleep in `wait_current_trans`, `wait_for_commit`, `btrfs_sync_log` and `wait_log_commit`.
- remove: 232 / 272 / 147 / 207 / 550 ms, its longest wait `wait_current_trans`.
- commit: not measured on that fixture yet, see "What is not measured".

## What is not changed, and why

- The add's index flush and the add's record syncs stay.
  An index can hold staged changes the remote does not have,
  and the landing records are the only way to finish or undo a commit that has not been pushed,
  so neither falls under the owner's "things that are already on remote".
  The add's own bound is 60 s, not the commit bound,
  so nothing here forces their removal;
  their numbers are above for the next decision.
- The snapshot ioctl stays on the add's path.
  It is the largest btrfs call of an add after the deletions,
  and taking it off the path is the next step below.
- No subvolume deletion was moved, because another agent owns that path in this session.

## The waits that remain unavoidable on this volume

Every write joins the running transaction,
so a commit already in progress delays any write of any process:
`btrfs_start_transaction` waits in `wait_current_trans` (`transaction.c:536-560`)
and, when the running transaction is already committing,
`btrfs_commit_transaction` waits with `wait_for_commit(cur_trans, want_state)`
(`transaction.c:2320-2331`).
The measured add above spent most of its blocked samples there.
Removing a forced commit removes a wait that the command itself asked for;
it cannot remove this one.

## What the next step should be

1. Take the snapshot ahead of the add:
   the service keeps a few snapshots of a proven-clean source root,
   and an add claims one with a rename and brings it to the requested commit,
   as it already keeps private repositories made ahead (`373e8adfe2`).
   The claim must prove the snapshot's files equal the source's index
   before it installs an index over them,
   because a stale file with refreshed stat data would be silently wrong.
2. Hand a subvolume deletion to the service and never wait for it,
   as the trash hand-over already does,
   so that no removal takes the filesystem-wide `subvol_sem` write lock on a person's path.
3. Bound the snapshot ioctl and leave the stuck call in the service:
   a bound in the client alone cannot work,
   because a thread asleep in the ioctl keeps the process from exiting.
