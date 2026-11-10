# What the repository volume's filesystem costs a worktree add, and what is still unknown

## Purpose

The owner asked whether the drive is good enough,
whether deletions can be made asynchronous,
whether to move to another filesystem,
what about ZFS,
and whether btrfs features or parameters are missing.
This file holds the evidence,
the owner's answers,
and what has not been measured.

## Measured cause of slow worktree adds at 2026-11-10T06:05:00-05:00

Ten `git worktree add` commands ran at once.
Across 92 seconds every thread of those commands gave 54 samples on CPU and 2,796 blocked.
Of the blocked samples,
2,182 were deleting files:
each add was emptying its own `.claude/cli-git-trash/<id>` folder,
which `package/git-policy/cli/SPEC.md` requires before the command returns.
The blocked samples divide by the wait:
`wait_current_trans` 41%,
`handle_reserve_ticket` 25%,
metadata reads 22%,
`blk_mq_get_tag` 8%.

Agent `worktree-add-does-not-wait-for-its-trash` is taking that deletion off the waiting path.

## The drive

The Samsung 860 EVO 4TB is healthy:
SMART reports no reallocated sectors and no logged errors,
and `btrfs device stats` is all zero.
Under this load it is the limit:
99% busy,
about 6,200 I/O operations a second at about 3 ms each,
a mean queue of 20.6 of 32,
and I/O pressure from 54 to 72% with CPU and memory pressure from 1 to 3%.

No faster drive was measured,
so how much one would buy is unknown.
The test is to measure busy time again after the deletions leave the waiting path and TRIM is on.

## Asynchronous deletes

`btrfs-subvolume(8)` says of deleting a subvolume:
"The command returns immediately."
On this volume it returned in 3.2 ms without unlinking the 300 files inside.
A plain directory cannot be renamed into another subvolume (`EXDEV`),
so a trash folder cannot be a subvolume of its own.
A snapshot does not carry a nested subvolume,
so folders made into subvolumes in the main checkout would not need deleting from a new worktree.

Deleting one file has no asynchronous form on any filesystem read.
Each path below opens a transaction per file before it returns:

- btrfs:
  `btrfs_unlink` in `fs/btrfs/inode.c`,
  Linux `master` read 2026-11-10.
- ext4:
  `__ext4_unlink` in `fs/ext4/namei.c`,
  same read.
- XFS:
  `xfs_remove` in `fs/xfs/xfs_inode.c`,
  same read.
- ZFS:
  `zfs_remove` in `module/os/linux/zfs/zfs_vnops_os.c`,
  OpenZFS `master` read 2026-11-10.

ZFS has the same whole-object shortcut as btrfs:
`zpool-features(7)` of `zfs-2.4.4` says that with `async_destroy` a file system's data "will be reclaimed by a background process".
ZFS differs in commits:
`module/zfs/txg.c` of `zfs-2.4.4` says "There is always a txg in the open state so that ZFS can accept new changes (though the txg may refuse new changes if it has hit some limit)."
Whether that shortens the `wait_current_trans` share for this workload is what the VM test is for.

## Timing comparisons that could not rank filesystems

`~/temp/agent/pi-runs/compare-deletes-on-three-filesystems.ts` created and deleted 30,000 files of 1 KiB on 8 GiB no-copy-on-write images behind direct-I/O loop devices on the 860,
and in a folder on the real volume,
alternating arms under the agents' normal load.
Each figure is the median of the delete time,
with the smallest and largest round.

The first run,
of three rounds:

- fresh btrfs:
  754 ms (630 to 36,149)
- fresh ext4:
  912 ms (700 to 1,409)
- fresh XFS:
  5,798 ms (895 to 122,198)
- the real volume:
  3,883 ms (1,727 to 4,682)

The second run,
of four rounds:

- fresh btrfs,
  metadata `dup`:
  23,141 ms (813 to 24,274)
- fresh btrfs,
  metadata `single`:
  28,090 ms (955 to 37,123)

The same fresh btrfs arm moved from under a second to tens of seconds between runs,
so the spread on this host is larger than any difference between arms.
No ranking of filesystems follows from these runs,
and neither does any claim that the real volume's state costs more than btrfs does.
Metadata `dup` against `single` shows no measurable difference.

## Settings of the volume at 2026-11-10T06:30:00-05:00

- TRIM was off.
  `/sys/block/dm-2/queue/discard_max_bytes` read 0 while `sda` reported 2,147,450,880,
  `cryptsetup status` showed no flags,
  the LUKS header held no flags,
  and neither the unlock nor the mount command in `/etc/systemd/system/mount-860.service` asked for discards.
  `fstrim` logs of the last 60 days never named this volume.
  The NVMe volumes already allow discards.
- `block-group-tree` is absent;
  `mkfs.btrfs(8)` credits it only with reducing mount time.
- The reclaim settings `periodic_reclaim`,
  `dynamic_reclaim` and `bg_reclaim_threshold` are all off.
  No measurement connects them to the waits on metadata space.
- `commit=` is the default.
  The volume made 8 commits in 124 seconds,
  more than the default 30-second timer accounts for.

## Owner decisions at 2026-11-10T06:44:02-05:00

- TRIM:
  "On,
  saved in the service (Recommended)".
  `~/temp/agent/pi-runs/enable-trim-on-the-860.ts` adds `--allow-discards` and `discard=async` to `mount-860.service`,
  applies both live,
  runs one full trim,
  and samples the drive before and after.
- ZFS:
  "Test ZFS in a VM (Recommended)".
  Agent `zfs-and-btrfs-delete-stalls-in-a-vm` is queued,
  and writes `doc/planning/zfs-and-btrfs-delete-stalls-in-a-vm.md`.

## TRIM results at 2026-11-10T07:14:14-05:00

The full trim finished with status 0 after 1,524 seconds and trimmed 223.2 GiB.
Each sample is 120 seconds of `sda` and the volume's commits,
from `~/temp/agent/pi-runs/enable-trim-on-the-860.log`:

- Before enabling:
  1,681 reads and 4,282 writes per second,
  read wait 2.38 ms,
  write wait 3.70 ms,
  busy 98%,
  mean queue 19.9,
  21 commits averaging 4,631 ms.
- After enabling,
  before the full trim:
  1,272 reads and 2,242 writes per second,
  read wait 3.31 ms,
  write wait 7.36 ms,
  busy 99%,
  mean queue 20.7,
  11 commits averaging 10,227 ms.
- After the full trim:
  3,002 reads and 2,176 writes per second,
  read wait 1.36 ms,
  write wait 3.74 ms,
  busy 100%,
  mean queue 12.2,
  8 commits averaging 17,246 ms.

These do not show whether TRIM helped.
The last sample overlapped the start of a batch of new agents,
the workload differed between all three,
and a commit that spans a sample boundary inflates the time counted as committing.

## Earlier decision on ZFS

`doc/decision/cachyos-zfs-desktop.md` records the owner on 2026-09-14:
"Try btrfs first,
only zfs when btrfs is proven to be the problem."
Its runbook,
`doc/runbook/migrate-bazzite-to-cachyos-zfsbootmenu.md`,
moves the operating system on the NVMe and preserves the 4 TB LUKS and btrfs SSD outside `zroot`,
so following it would not put this repository on ZFS.
`zfs-2.4.4`'s `META` lists Linux 7.2 as its maximum,
and the host runs `7.2.15`.
No ZFS module is installed here,
and none was built.
Putting the repository itself on ZFS needs somewhere to hold 3,486 GiB,
and the only other drive has 268 GiB free.
The wrapper's snapshot route for `git worktree add` is written for btrfs subvolumes and would have to move to ZFS clones.
