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
- `src/native/landing/work_item_holding.rs:101`,
  `record_verdict`,
  the verdict the post-commit gate leaves in the commit's held push record,
  through `src/native/foundation/private_storage.rs:547`,
  `replace_private_file`.
  Its kind was log or commit,
  made twice,
  once for the record and once for its directory,
  on the commit's path after the landing,
  which every landed commit reaches.
  Now `replace_private_file_unsynced` in `private_storage_replacing_a_record.rs` publishes the record whole
  by one rename and flushes nothing,
  so a kill keeps the verdict,
  and a machine crash keeps the record the gate found,
  which `held_push_holder_and_verdict.rs` settles as a record with no verdict.
  Flushing the verdict alone protected nothing the same record does not,
  because `hold` writes the push obligation itself into it unflushed,
  and the audit recorded in this document traced six concurrent flushes at 8 to 12.5 s each,
  which is the wait the owner's "commits vs native git should not take 5s longer" rules out.
  Controls:
  `landing::service_handoff::tests::the_verdict_a_commit_records_flushes_nothing`,
  `held_push_settled_by_the_verdict::a_commit_whose_command_died_before_its_checks_names_what_sends_it`,
  and `budget::` in `binary_budget_tests.rs`,
  whose constants allow no wrapper sync on a commit's path.

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

## Step 1: a worktree remove hands its subvolume to the service

Built on this branch, with a test shown failing first.

What it does:
`git worktree remove` proves the worktree clean and unlocked as before,
then, instead of the destroy ioctl,
it renames the subvolume inside the parent directory it already holds open
to `cli-git-deleting-<16 hex digits of the subvolume id>`
(`src/native/worktree/worktree_remove_hand_over.rs`, `hand_over_removal`),
writes a durable work item of the new kind `subvolume-delete` into the repository's common work directory,
wakes that repository's commit service, and returns.
The service deletes the subvolume in `delete_subvolume_of`,
by id through the opened parent,
in the round's own time (`service_work_drain_pending_claims.rs`).
A step that fails before the rename puts nothing anywhere and the removal destroys the subvolume in place,
exactly as it did before,
so the hand-over is never what makes a removal fail.
A record whose path is gone deletes nothing,
which is why the record is written before the rename.

Why it matters on this volume:
the destroy takes `down_write(&fs_info->subvol_sem)` and joins the running transaction
(`fs/btrfs/inode.c:4801`, `:4848` at `v7.2.15`),
so it waits for any snapshot another add is committing,
and the cleaner works behind every removal.
This is the delete the wrapper's automatic removal of finished worktrees will call
(branch `fix/git-policy-cli-cleans-up-after-its-own-worktrees`).

The line the command prints now says that the service deletes the files after it returns,
and `package/git-policy/cli/README.md` says what a stranded `cli-git-deleting-<id>` directory means
and which command empties it.

### What the review of the hand-over found, and what it changed

Kimi K3.1 Pro reviewed the branch at `5686c7bb29`
(`~/temp/agent/kimi-reviews/never-waits-on-btrfs-commits-round-133/review.md`)
and named four defects in this step and two in the syncs above.
Each reproduction was run here again at alpha.9,
and each fix has a control that failed at `ee545033ad`:

- The destroy by id read the id from the **parent's** descriptor,
  so the call always named the wrong subvolume and every delete fell through to the destroy by name,
  which deletes whatever stands at the name.
  A subvolume another command had put there lost its user's file.
  `delete_subvolume_of` now reads the id the record holds (`item.oid`),
  opens the directory at the name
  without following a link or crossing a mount,
  and asks **that** descriptor for its id:
  a different id is a failure that leaves the directory as it is and names both ids.
  The destroy by id runs only when the ids agree,
  and the destroy by name only after that.
  The record's id also carries a digest of the directory that held the subvolume
  (`work_item::id`,
  `work_item_ids_and_names::parent_digest`),
  because a subvolume id is unique on one
  filesystem only and two removals on two filesystems collided on one queue entry.
- The record is published before the rename,
  so a live service can claim it in that window,
  find the hidden path absent,
  and settle it as done;
  the rename then left the worktree's files under a name no record named.
  `hand_over_with` now asks,
  after the rename,
  whether a record for the item is still queued or claimed
  (`a_record_awaits`),
  and deletes the renamed subvolume itself when none is,
  so the files still go.
- A parent path that is not valid UTF-8 was written into the record as `U+FFFD`,
  a path that names a different directory,
  and the record then settled as done while the real directory stayed.
  `record_the_removal` refuses such a path,
  the hand-over answers why,
  and the removal destroys the subvolume
  in place exactly as it did before the hand-over existed.
- `Verdict::Retry` was settled where a failure the service gave up on goes,
  and nothing ever retried it.
  `settle_with_the_words` queues a retryable verdict again,
  bounded by `tries` and `MAX_TRIES`
  (`service_work_settling_with_words.rs`),
  so a call the kernel refuses every time cannot keep a round busy.

The two sync findings are recorded under "What was changed" below.

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

The same rule holds when there is no background task to hand the queue to,
which a runtime that could not start one leaves behind:
a rejected hand is a flush nobody makes,
and the write reaches the disk with the operating system's own write-back,
so no `sync_all` runs on the thread that asked for it
(`defer_file_with` and `Deferral::settle_with` in `src/native/landing/service_durable.rs`,
controls `a_flush_no_background_task_takes_is_never_made_on_the_callers_thread` and
`a_published_landing_whose_queue_no_task_takes_waits_for_nothing`).
For the index lock marker the decision above says what a power cut loses there:
a name that points at no holder.
For a landing that published,
a machine crash loses that attempt as it does with native Git,
and neither the marker nor a landing's record is worth a device's whole wait on the person's path.
A landing that published nothing still forces its records before it returns,
as the owner's "Force at leaving,
else background" says.

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

The same fixture under a lighter load, three rounds per arm, alternating arms,
each round an add, two commits in the fixture's own checkout and a remove:

- before arm, 2026-11-10T18:47:31-05:00: adds 554 / 413 / 391 ms,
  commits 227 / 271 / 248 ms, removes 42 / 42 / 43 ms.
- after arm, 2026-11-10T18:48:43-05:00: adds 3,906 / 700 / 699 ms,
  commits 164 / 188 / 268 ms, removes 42 / 44 / 43 ms.
  The 3,906 ms add spent 1.3 s of its samples in `btrfs_commit_transaction`:
  that is the snapshot ioctl, which this change does not touch,
  and it is why an add's cost is dominated by load rather than by the commit path.
- An uncontended commit is not where this change shows:
  three samples per arm cannot separate 227 to 271 ms from 164 to 268 ms on this volume,
  and the concurrency the audit measured is what the s01 numbers above are for.

### The numbers, alternating the two binaries on the repository's own volume

The disposable fixture of `measure.ts` on `/var/mnt/encrypted` (btrfs, `dm-2`),
each round an add, two commits in the fixture's checkout and a plain `git worktree remove`,
the arms alternating,
every run reported,
the load average 111 and `/proc/pressure/io` some avg60 36.02 when the first arm began at
2026-11-10T19:31-05:00 and some avg60 10.43 when the last ended at 2026-11-10T19:42:47-05:00
(the coordinator's removal of 350 worktrees was running through it).

- before, 19:31-19:32 (`binaries/before`, the branch before this step):
  adds 30,982 / 1,946 / 6,234 / 13,673 / 3,666 ms,
  commits 4,448 / 491 / 4,279 / 144 / 230 ms,
  removes 528 / 184 / 871 / 43 / 64 ms,
  the adds' blocked samples in `wait_for_commit`, `wait_log_commit` and `read_extent_buffer_pages`,
  the long removes' in `btrfs_opendir`.
- under the lighter load at 19:42, five runs each, before then after:
  adds 408 / 224 / 389 / 409 / 225 ms before and 571 / 291 / 226 / 186 / 184 ms after,
  commits 22 to 106 ms in both,
  removes 42 / 43 / 42 / 42 / 42 ms before and 64 / 42 / 42 / 62 / 42 ms after.
- Which removals the wrapper served is visible in its line:
  3 of the before arm's 5 printed the notice of a subvolume removal,
  and 2 of the after arm's 5 did,
  the others being left to Git because the worktree was not proven clean at that moment
  (the trash the add set aside is emptied by the service in its own time).
- What this says: on a volume whose transactions are not stalling, the destroy this step moves costs
  milliseconds, and the record and the wake add 0 to 22 ms.
  The destroy's cost is the `down_write(&fs_info->subvol_sem)` and the transaction join behind it
  (`fs/btrfs/inode.c:4801`, `:4848` at `v7.2.15`),
  which the 19:31 arm never reached with a served removal,
  so this change's worth here is that the wait cannot happen,
  not that a measured number fell.

## Step 2: snapshots made ahead, sized from measured arrivals (not built)

Not built on this branch.
What the measurement that must size it says,
from the 404 adds of the repository's own timing log over 23.5 hours:

- the busiest hour holds 53 adds, the busiest five minutes 19, the busiest minute 12;
- the gap between two adds' starts is 20.0 s at the median, 2.8 s at the tenth percentile, and 0 s at the least;
- one add's snapshot costs the kernel work of `snapshotIoctl`, whose median is 7,443.8 ms and worst 39,828.5 ms
  on this volume,
  so a pool that must never make an add wait for a snapshot needs
  at least as many snapshots as adds can arrive in the time one snapshot takes to make:
  at the measured worst, 39.8 s of making against one add every 2.8 s is 14 snapshots;
  at the median cost it is 3.

The pool also has to keep a stale snapshot from pinning `main`'s files:
a snapshot that has fallen behind `main` by more than the pool needs is deleted through step 1.

## Step 3: a bound on a btrfs call that has not returned (not built)

Not built on this branch.
The bound must come from the measurement beside it:
`snapshotIoctl` costs 7,443.8 ms at the median, 18,006.2 ms at p90 and 39,828.5 ms at its worst
(300 adds to 2026-11-10T23:09:44.166Z),
and the owner's bound for an add is 60 s,
which is 1.5 times the worst measured call.
A client cannot bound the call by waiting less:
a thread asleep in the ioctl keeps the process from exiting,
so the call belongs to the service and the client takes the copy route when the service answers nothing in time.

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
