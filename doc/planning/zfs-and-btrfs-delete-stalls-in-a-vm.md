# ZFS and btrfs delete stalls measured in one VM

## Status

Measured 2026-11-10 inside one disposable VM,
`mvm create --image ubuntu zfs-btrfs-delete-stalls`,
on this machine.
The VM was destroyed at 2026-11-10T14:45:55+00:00 and no domain,
disk,
or process of it remains.
The branch is `docs/planning-zfs-and-btrfs-delete-stalls-in-a-vm`;
nothing was merged to `main`.

The question the owner asked:

> Test ZFS in a VM (Recommended)
> 2026-11-10,
> in the question tool.

The standing decision it tests:

> Try btrfs first,
> only zfs when btrfs is proven to be the problem.
>
> `doc/decision/cachyos-zfs-desktop.md:9-10`,
> superseded decision of 2026-09-14.

The hypothesis in the brief:

- `git worktree add` here snapshots a btrfs checkout and then deletes hundreds of thousands of files from each new
  subvolume.
- Ten at once were measured with 78 percent of their blocked samples in deletion:
  41 percent in `wait_current_trans`,
  25 percent in `handle_reserve_ticket`,
  22 percent reading metadata.
- OpenZFS's `module/zfs/txg.c` says there is always a transaction group open for new changes while another is written
  out,
  so ZFS might not stall the same way.
- Nobody had measured it for this workload.

The specific waits that brief names,
with their measured durations on this host,
are in `doc/planning/cli-git-commit-latency-under-many-agents.md`
(section "What waits,
sampled from the host without adding load (2026-11-09T22:51:40-05:00)"):
`linkat` in `wait_current_trans` at 112 268 ms,
`linkat` in `handle_reserve_ticket` at 112 686 ms,
`fsync` in `wait_for_commit` at 12 680 ms.

## Answer in one paragraph

ZFS does not avoid the stall.
Mass deletion on ZFS is throttled by design:
`dmu_free_long_range_impl` stops adding freed blocks to a transaction group once it reaches a dirty-frees threshold,
waits for the next transaction group to open,
and repeats.
Every deleter in this experiment blocked there,
98.8 and 99.6 percent of their blocked samples in `txg_wait_open`,
and the same 2.7 million deletions that took btrfs 24 to 28 seconds took ZFS 788 to 871 seconds,
with 7 to 9 times the bytes written to the device.
btrfs instead ran the unlinks at memory speed and paid in its commit (`wait_for_commit`,
`wait_current_trans`)
and in btree lock contention,
and its own filesystem stalled unrelated `fsync` calls for seconds in one round while ZFS mostly did not.
The hypothesis that ZFS would smooth this workload is not supported.
What the VM cannot show about the owner's Samsung 860 EVO is listed in the section on limits;
the short version is that this measured filesystem code,
not that drive.

## The workload the VM reproduces

The real shape,
as the owner described it,
is a snapshot of a large checkout plus a mass deletion inside the copy,
repeated concurrently.

Each filesystem therefore got the same three shapes:

1.  **Arm A,
    the baseline delete**:
    a fresh tree of 30 000 small files,
    deleted by one process,
    followed by the filesystem's own commit.
1.  **Arm B,
    the real shape**:
    a source tree of 300 000 small files in a subvolume or dataset,
    a read-only base snapshot of it,
    ten workers that each take a writable snapshot or clone of that base and then delete nine of every ten files
    (270 000 deletions each,
    2.7 million per run),
    then commit.
1.  **Arm C,
    the same shape with destroy instead of delete**:
    the same ten snapshots or clones,
    each destroyed whole with `btrfs subvolume delete` and `zfs destroy -r`,
    started concurrently.

Arm A ran 3 rounds per filesystem,
arms B and C ran 2,
and the order was reversed every round:
btrfs then ZFS on odd rounds,
ZFS then btrfs on even rounds.

Files are 400 bytes of text.

## The VM and every limit placed on it

- Created with `mise run //package/mvm/cli:run -- create --image ubuntu zfs-btrfs-delete-stalls`.
- Guest kernel:
  `6.8.0-106-generic`,
  measured from `uname -r`.
- OpenZFS kernel module:
  `2.2.2-0ubuntu9.4`,
  measured from `/sys/module/zfs/version`,
  with `spl` `2.2.2-0ubuntu9.4`.
- OpenZFS userland:
  `zfs-2.2.2-0ubuntu9.5` with `zfs-kmod-2.2.2-0ubuntu9.4`,
  measured from `zfs version`.
  The package archive carries no 9.4 userland any more,
  so userland and module differ in the last revision inside the same 2.2.2 series.
- btrfs userland:
  `btrfs-progs v6.6.3`.
- Virtual CPUs:
  4.
- Memory:
  7941 MiB,
  measured from `/proc/meminfo`.
- Subject disks:
  `/dev/vdb` 24 GiB for btrfs and `/dev/vdc` 24 GiB for the ZFS pool,
  both virtio-blk,
  both qcow2 files in `${HOME}/temp/agent/zfs-btrfs-vm/disks` on the host.
- Control disk:
  `/dev/vdd` 4 GiB ext4 on the same host directory,
  mounted at `/mnt/ctrlext4`.
- Caching mode:
  `cache=unsafe` on all three,
  set with `virsh attach-disk ... --cache unsafe`.
  See the section on the caching decision for why,
  and the section on limits for what it costs.
- btrfs construction:
  `mkfs.btrfs -f -L exp /dev/vdb`,
  mounted `compress=zstd:3,noatime,ssd`.
- ZFS construction:
  `zpool create -f -O compression=zstd-3 -O atime=off -O mountpoint=/mnt/zfs exp /dev/vdc`,
  one whole disk as the vdev,
  recordsize left at the default 128K,
  `logbias=latency`,
  `sync=standard`.
- Compression therefore matches the host's `zstd:3` on both filesystems,
  and btrfs and ZFS had no tunable changed from their defaults.
- Guest timers:
  every `systemd` timer was disabled before the runs
  (`fstrim`,
  `apt-daily`,
  `logrotate`,
  `sysstat-collect`,
  and the rest),
  so nothing but the experiment wrote.

### The host these numbers were taken on

- The two subject disks are qcow2 files on `/var/mnt/encrypted`,
  which is btrfs on `dm-crypt` over `sda`,
  a Samsung SSD 860 EVO 4TB SATA drive,
  mounted `noatime,compress=zstd:3,ssd,space_cache=v2`,
  95 percent used.
- That is the same volume the repository lives on and the same volume whose `wait_current_trans` and
  `handle_reserve_ticket` waits the owner reported.
- Host load average while the runs were in progress,
  sampled between 09:12 and 09:45 local time (-05:00):
  65,
  76,
  87,
  98,
  100,
  111,
  157, 165.
- Host I/O pressure in the same window:
  `some avg10` between 30 and 40 percent.
- Swap in use:
  about 30 GiB.
- A 128 MiB `O_DIRECT` write to that volume measured 9.1 s and 14.1 s before the runs,
  about 7 to 13 MiB/s.

### Why the disks were not cache=none

The first attempt used `cache=none`,
which makes every guest write an `O_DIRECT` write to the host file,
and it failed at the first setup step:
`zpool create` sat in `txg_wait_synced` from `spa_create` for 862 seconds,
having written about 2 MiB to the guest's disk,
while the guest's own `/proc/diskstats` showed the device nearly idle.
The host volume could not absorb the write burst at the time.
`cache=unsafe` moves the guest's writes into the QEMU page cache instead,
which decouples the guest's filesystem timing from the host's frame of mind.
That is what all the reported runs used.

## How stalls were attributed

Three instruments ran during every arm:

1.  A same-filesystem probe:
    a 400-byte overwrite plus `fsync` in a loop in a separate subvolume or dataset of the subject,
    recording every iteration's duration.
1.  A control probe:
    the same loop on the ext4 disk that has the same qcow2 backing and the same cache mode.
    A stall that appears on both is the host's,
    not the filesystem's.
1.  A stack sampler:
    every 100 ms,
    `/proc/<pid>/stack` and `/proc/<pid>/wchan` for the ten deleter processes and for the filesystem's kernel
    threads (`btrfs-cleaner`,
    `txg_sync`,
    `z_wr_iss`,
    and the like).
    The reported wait site is the first frame past the scheduler and SPL condition-variable helpers,
    so `cv_wait_common` becomes the ZFS function that called it.

Every run also recorded the guest's own `steal` time,
its `/proc/pressure` lines,
its load average,
and the bytes written to the subject device taken from `/proc/diskstats`.

The probe statistics are reported for the interval that overlaps the delete plus commit window.
A sample counts when its interval overlaps the window,
so one long stall that spans the whole window still appears.

## Arm A: one process deleting 30 000 files

Runs between 2026-11-10T14:12:33+00:00 and 2026-11-10T14:13:54+00:00,
three rounds each,
btrfs first on rounds 1 and 3,
ZFS first on round 2.

btrfs:

- Delete window:
  0.79 s in round 1,
  0.79 s in round 2 and 0.72 s in round 3;
  median 0.79 s.
- Commit after the delete:
  0.60 s in round 1,
  7.87 s in round 2 and 2.03 s in round 3;
  median 2.03 s.
- Bytes written to `/dev/vdb` during the run:
  52 MiB in round 1,
  2 MiB in round 2 and 22 MiB in round 3.
  The fresh tree's own writeback can land inside the window,
  so this is not a per-deletion cost at this size.
- Blocked deleter samples:
  almost none,
  the workers were running.
- Kernel-thread wait sites:
  `btrfs_commit_transaction` in the fast round,
  and `wait_current_trans` with `zio_wait` and `wait_for_commit` in the slow one.

ZFS:

- Delete window:
  5.05 s in round 1,
  9.88 s in round 2 and 14.75 s in round 3;
  median 9.88 s.
- Commit after the delete:
  2.08 s in round 1,
  0.32 s in round 2 and 11.22 s in round 3;
  median 2.08 s.
- Bytes written to `/dev/vdc` during the run:
  9 MiB in round 1,
  22 MiB in round 2 and 22 MiB in round 3,
  with the same caveat as btrfs:
  the fresh tree's writeback can land inside the window.
- Blocked deleter samples:
  100 percent of them at
  `txg_wait_open <- dmu_free_long_range_impl <- dmu_free_long_range`.
- Kernel-thread wait sites:
  `zio_wait` at 52 to 82 percent,
  `zil_lwb_flush_wait_all` at 16 to 46 percent.

Worst probe stall inside the window,
subject and control:

- btrfs:
  8 641.7 ms on the subject,
  8 639.8 ms on the control.
- ZFS:
  7 374.0 ms on the subject,
  7 523.2 ms on the control.

The subject and the control stall by the same amount in the same round,
so those worst stalls are the host's,
not either filesystem's.

## Arm B: ten workers, 300 000-file snapshots and clones

Runs between 2026-11-10T14:13:54+00:00 and 2026-11-10T14:43:03+00:00,
two rounds each,
btrfs first on round 1,
ZFS first on round 2.
Each round deletes 2 700 000 files in total.

btrfs:

- Delete window:
  23.72 s and 27.57 s.
- Commit after the delete:
  6.84 s and 6.84 s.
- Total from start of deletion to end of commit:
  30.6 s and 34.4 s.
- Bytes written to `/dev/vdb`:
  494 MiB and 636 MiB,
  16.1 and 18.5 MiB/s over the run.
- Worker wait sites:
  `__btrfs_tree_read_lock` 79.8 percent and 54.6 percent,
  `wait_current_trans` 0 percent and 22.9 percent,
  `handle_reserve_ticket` 10.1 percent and 8.8 percent,
  `btrfs_delete_delayed_dir_index` 5.5 percent and 3.1 percent,
  `__wait_on_buffer` 0 percent and 6.5 percent.
- Worker call chains:
  `__btrfs_tree_read_lock <- btrfs_read_lock_root_node <- btrfs_search_slot` 76.2 and 53.1 percent,
  `wait_current_trans <- start_transaction <- btrfs_start_transaction_fallback_global_rsv` 22.9 percent in round 2,
  `handle_reserve_ticket <- __reserve_bytes <- btrfs_reserve_metadata_bytes` 10.1 and 8.8 percent.
- Kernel-thread wait sites:
  `wait_for_commit` 96.3 percent in round 1,
  `__rq_qos_throttle` 96.9 percent in round 2.
- Same-filesystem probe:
  p99 11 462.8 ms and 82.0 ms,
  worst 13 319.4 ms and 8 430.4 ms,
  iterations over 200 ms:
  5 of 127 and 19 of 2 738.
- Control probe:
  p99 2.8 ms and 3.8 ms,
  worst 4 502.9 ms and 4 419.4 ms.
- Guest I/O pressure at the end of the run:
  `full avg10` 36.8 and 32.3 percent.
- Steal:
  1.2 s and 5.0 s of a 30.6 s and 34.4 s window.

ZFS:

- Delete window:
  871.01 s and 787.97 s.
- Commit after the delete:
  0.07 s and 0.07 s.
- Total:
  871.1 s and 788.1 s.
- Bytes written to `/dev/vdc`:
  4 481 MiB and 4 563 MiB,
  5.1 and 5.8 MiB/s over the run.
- Worker wait sites:
  `txg_wait_open` 98.8 percent and 99.6 percent,
  `zio_wait` 1.1 and 0.2 percent.
- Worker call chains:
  `txg_wait_open <- dmu_free_long_range_impl <- dmu_free_long_range` 98.8 and 99.6 percent.
- Kernel-thread wait sites:
  `zio_wait` 93.3 and 92.9 percent,
  `dmu_objset_sync` 5.5 and 5.7 percent.
- Same-filesystem probe:
  p99 166.7 ms and 151.9 ms,
  worst 15 734.7 ms and 20 933.0 ms,
  iterations over 200 ms:
  423 of 45 078 and 396 of 43 779.
- Control probe:
  p99 1.5 ms and 1.2 ms,
  worst 13 560.7 ms and 16 613.4 ms.
- Guest I/O pressure at the end of the run:
  `full avg10` 63.0 and 44.2 percent.
- Steal:
  89.3 s and 48.9 s of an 871.1 s and 788.1 s window.

The ten workers agree with each other inside each round,
which is why the ZFS figure is not host weather:
862.7,
870.3 and 862.8 s in round 1,
762.4,
777.6 and 783.5 s in round 2,
against 22.5 to 27.1 s for the btrfs workers.

Per deleted file,
ZFS wrote 1.7 KiB to its device where btrfs wrote 0.19 KiB,
and the delete rate was 3 100 files per second for ZFS against 98 000 to 114 000 for btrfs.

## Arm C: destroying the snapshots and clones instead of deleting inside them

Runs between 2026-11-10T14:43:03+00:00 and 2026-11-10T14:44:20+00:00,
two rounds each.

btrfs:

- Each `btrfs subvolume delete` returned in 0.003 s;
  ten of them,
  started together,
  took 0.03 s.
- The commit after them took 0.55 s and 0.16 s.
- Bytes written to `/dev/vdb` during the window:
  48 MiB and 23 MiB.
- Same-filesystem probe:
  p99 5.7 ms and 15.1 ms,
  worst 1 518.4 ms and 1 791.1 ms.
- Control probe:
  p99 1.1 ms and 2.4 ms,
  worst 4 277.3 ms and 530.4 ms.
- Kernel-thread wait sites in round 1:
  `__btrfs_tree_read_lock` 100 percent,
  from a single blocked sample.

ZFS:

- `zfs destroy -r` of the ten clones took 1.5 s in round 1 and 0.1 s in round 2,
  measured as the slowest of the ten.
- The commit after them took 0.04 s and 0.07 s.
- Bytes written to `/dev/vdc` during the window:
  13 MiB and 15 MiB.
- Same-filesystem probe:
  p99 4.7 ms and 4.1 ms,
  worst 1 745.8 ms and 323.2 ms.
- Control probe:
  p99 1.1 ms and 1.3 ms,
  worst 447.9 ms and 4.5 ms.
- Kernel-thread wait sites:
  `zil_lwb_flush_wait_all` 93.3 percent and `zio_wait` 100 percent.

Both filesystems return from the destroy command before doing the work,
and in the window measured neither probe stalled far beyond the control's own worst stall:
btrfs's subject saw 1 518.4 ms against a control worst of 4 277.3 ms in round 1 and 1 791.1 ms against 530.4 ms in
round 2,
and ZFS's subject saw 1 745.8 ms against 447.9 ms in round 1 and 323.2 ms against 4.5 ms in round 2.

Arm C as built is the weakest of the three,
and the section on what it does not show says why:
a clone of an unmodified source frees almost no space,
so the background reclamation had nothing to reclaim,
and the settle detector,
which keys on free space returning,
finished in 2.0 s on both filesystems without observing the background work at all.

## What the two kernels were waiting for

### btrfs

The single-process delete rarely blocks:
the unlinks land in the running transaction,
and the cost appears at commit as `btrfs_commit_transaction` and,
on a slow round,
as `wait_current_trans` and `wait_for_commit`.

With ten workers the blocking moves into the delete itself:

- `__btrfs_tree_read_lock` under `btrfs_search_slot` dominates at 54 to 80 percent,
  which is ten deleters contending on the same btree locks.
- `wait_current_trans` at 0 to 22.9 percent and `handle_reserve_ticket` at 8.8 to 10.1 percent,
  the same two functions the owner's host measurement named,
  `wait_current_trans <- start_transaction` and
  `handle_reserve_ticket <- __reserve_bytes <- btrfs_reserve_metadata_bytes`.
- `btrfs_delete_delayed_dir_index` and `__wait_on_buffer` account for the rest.

So on btrfs the wait is shared between lock contention,
transaction commit,
and metadata reservation,
and the reservation wait is the one that grows with a full filesystem.

### ZFS

Every ZFS arm blocks in one place:

```text
txg_wait_open <- dmu_free_long_range_impl <- dmu_free_long_range
```

That is the free throttle in `module/zfs/dmu.c` of OpenZFS 2.2.2:

```c
/* module/zfs/dmu.c:918-930, OpenZFS tag zfs-2.2.2 */
/*
 * To avoid filling up a TXG with just frees, wait for
 * the next TXG to open before freeing more chunks if
 * we have reached the threshold of frees.
 */
if (dirty_frees_threshold != 0 &&
    long_free_dirty >= dirty_frees_threshold) {
	DMU_TX_STAT_BUMP(dmu_tx_dirty_frees_delay);
	dmu_tx_commit(tx);
	txg_wait_open(dp, 0, B_TRUE);
	continue;
}
```

with the threshold derived from `zfs_per_txg_dirty_frees_percent`,
default 5,
as `zfs_dirty_data_max * 5 / 100`
(`module/zfs/dmu.c:855-930` at tag `zfs-2.2.2`).

The transaction-group documentation the brief cites is accurate and is the reason this throttle exists,
not a reason it disappears:

> There is always a txg in the open state so that ZFS can accept new changes (though the txg may refuse new changes
> if it has hit some limit).

`module/zfs/txg.c:60-62` at tag `zfs-2.2.2`.
The next transaction group opens on the `zfs_txg_timeout` timer,
5 s by default,
so mass deletion advances in transaction-group-sized steps and every deleter waits for each step.
The kernel-thread side of that is `zio_wait` at 92 to 93 percent,
waiting for the writes the throttle admitted.

## What these numbers do not show

They do not rank the two filesystems for the owner's own disk,
and they do not predict the desktop stalls.
Specifically:

1.  **The guest's `fsync` never reaches the physical drive.**
    With `cache=unsafe`,
    QEMU acknowledges flushes from its page cache.
    Absolute commit and `fsync` times here describe filesystem code,
    not the 860 EVO.
1.  **The host leaked into every arm.**
    The subject disks are files on the same 95-percent-full btrfs the owner reports stalling,
    behind `dm-crypt`.
    The control probe exists because of that,
    and it earned its place:
    in arm A the subject and the control stalled by the same 8.6 s in the same round,
    which is host,
    and in arm B the control stalled for 4.4 to 16.6 s,
    which is also host.
    Where a subject's worst stall exceeds the control's the difference is the filesystem's,
    and that is the only reading I trust from the probe.
1.  **The guest's filesystems were nearly empty.**
    1 to 2 percent used,
    where the owner's is 95 percent used with 91 percent metadata.
    btrfs's `handle_reserve_ticket` is a space-reservation wait,
    so the VM understates it,
    and the owner's host shows it while this VM mostly does not.
1.  **There were no qgroups.**
    The stall investigation's first suspect,
    btrfs qgroup accounting during snapshot cleanup
    (`doc/troubleshooting/bazzite-desktop-input-stalls.md`),
    does not exist in this VM at all,
    because no qgroup was ever created.
    ZFS has no equivalent accounting.
1.  **The drive's behaviour is absent.**
    No SATA link,
    no NCQ queue depth,
    no drive scheduler,
    no `discard`/TRIM timing,
    no bad-block retries,
    no DRAM-less controller,
    no SMART.
    The host volume is the 860 EVO over `dm-crypt`,
    and both subject disks were two files on it,
    so nothing here isolates the drive from the host's own btrfs layer between them.
1.  **The files are synthetic.**
    400-byte uniform text in 300 directories of 1 000.
    A real checkout has object files,
    packed and loose objects,
    renames,
    and a size distribution.
    ZFS's 128K recordsize and btrfs's inline extents treat those differently.
1.  **The desktop is absent.**
    A `fsync` probe stands in for input latency,
    which is what the owner actually feels.
    The probe is a fair witness for filesystem-internal serialization and a poor one for compositor and input
    scheduling.
1.  **This ZFS is not the ZFS the plan would install.**
    Ubuntu's in-kernel 2.2.2 module with default tunables,
    not CachyOS's packaged 2.4.x,
    and `zfs_per_txg_dirty_frees_percent` was left at its default.
1.  **Arm C did not measure background reclamation.**
    A clone of an unmodified source frees almost nothing,
    so the free-space settle test finished in 2 s on both filesystems.
    The device counters show 48 MiB written on btrfs and 13 to 15 MiB on ZFS during the window,
    but how long the cleaner and the async destroy threads kept working afterwards is not measured here.

## What the numbers do show

- ZFS converts a mass deletion into a free-throttle wait that every deleter shares,
  `txg_wait_open` in 98.8 to 99.6 percent of their blocked samples.
  For this workload ZFS is 29 to 37 times slower than btrfs and writes 7 to 9 times as many bytes,
  reproducibly,
  with ten workers agreeing to within 3 percent inside a round.
- btrfs runs the unlinks at memory speed and moves the cost into lock contention and its commit,
  which is where the owner's `wait_current_trans` and `handle_reserve_ticket` samples fit.
- Destroying a whole snapshot or clone is cheap and asynchronous on both,
  and neither filesystem made ordinary same-filesystem work wait for it in the window measured.
- On this VM,
  ordinary work on the same btrfs stalled for seconds during arm B in one round (13.3 s worst,
  higher than the control's 4.5 s),
  and ZFS's ordinary work stayed around 160 ms at p99 with maxima inside the control's own range.
  That is the one probe result that favours ZFS,
  and it is exactly the symptom the owner reports for btrfs.

## What would settle it

- The ZFS trigger test already in the decision:
  install on the same NVMe with btrfs qgroups off,
  run the retained NVMe latency histogram from the stall investigation,
  and see whether the stalls stop.
- If a filesystem comparison is still wanted afterwards,
  it belongs on the physical drive with the real checkout,
  ten concurrent `git worktree add` and `git worktree remove`,
  the input latency recorded by the compositor rather than a probe,
  and the drive's own latency histogram beside it.

## What this changes in the decision

Nothing that removes btrfs,
and nothing that promotes ZFS.
The numbers run against ZFS for this workload,
and the standing decision already says btrfs first.
No operating-system swap is earned by these measurements,
and the next step they support is the qgroup-off trigger test on the existing install,
not a ZFS install.

## How to repeat it

The harness is scratch,
not repository code,
and lived at `${HOME}/temp/agent/zfs-btrfs-vm/experiment.py` as one Python file with these subcommands:
`init`,
`all`,
`one`,
`state`,
`resume`.
The raw per-run JSON,
including every sampler tally and every probe file,
is under `${HOME}/temp/agent/zfs-btrfs-vm/results/` and is not committed.

The host conditions matter more than any flag,
so a repeat should record the host's load average,
`/proc/pressure/io`,
and one raw write measurement before starting,
as this record does.
