# btrfs and ZFS, each given its best configuration, on the repository's own workload

## Status

Measured inside one disposable VM made with `package/mvm/cli`
(`mvm create --image cachyos-72-zfs244 btrfs-zfs-best`),
on this machine.
Owner's question,
2026-11-10:

> What if we give both btrfs and zfs benefit of doubt and give the best possible configs they can get?
> For example,
> zfs supports ARC.

This file holds the settings,
their sources,
every round's numbers,
the waits each filesystem still has,
and what these runs cannot tell.
Every number below came from the campaign described under "How to repeat it".

## The guest

- Built as a CachyOS root filesystem by `pacman --root` inside a privileged
  container,
  from the CachyOS and Arch repositories as they stood at
  2026-11-11T03:32:00+00:00.
- Kernel `7.2.11-2-cachyos-hardened`,
  measured with `uname -r`.
  The host runs `7.2.15-ogc1.1.fc45.x86_64`,
  and `zfs-2.4.4`'s `META` lists `Linux-Maximum: 7.2`,
  so this is the same kernel line and the newest one this ZFS supports.
- OpenZFS `2.4.4`:
  `zfs version` reports `zfs-2.4.4-1` and `zfs-kmod-2.4.4-1`,
  userspace and kernel module from the same release.
- btrfs:
  `btrfs-progs v7.3`,
  kernel `fs/btrfs` from `7.2.11`.
- 4 virtual CPUs,
  15 983 MiB of memory (`free -m`),
  no balloon.
  The first attempt ran at mvm's default of 8 192 MiB and the guest's out-of-memory
  killer ended a round:
  twenty concurrent checkouts inside ZFS's ARC pressure
  left nothing for them (`dmesg`,
  2026-11-11T04:02:00+00:00,
  `Out of memory: Killed process 419 (systemd)`).
  16 GiB is the smallest size that kept both arms out of the killer at this
  concurrency,
  and both arms got the same total.
- One system disk (ext4,
  20 GiB,
  `cache=none`) and three subject disks,
  all virtio-blk:
  `/dev/vdb` 24 GiB raw for btrfs,
  `/dev/vdc` 24 GiB raw for ZFS,
  `/dev/vdd` 512 MiB raw read-only payload.
  The two subject images live on the NVMe (`/var/home/user/nvme-agent`),
  never on the encrypted SATA volume,
  and both are attached with `--cache none`,
  so a guest flush is an `O_DIRECT` flush of the host file:
  both arms pay the same durability price and neither can hide a flush.
- Nothing else ran in the guest:
  no timers of its own,
  no package manager,
  no network traffic during a round.

## Why both filesystems are encrypted

The repository volume is LUKS plus btrfs,
and a ZFS install would use native
encryption,
so each arm pays for encryption the way an install of it would:
btrfs inside LUKS2 (`aes-xts-plain64`,
512-bit key,
`--perf-no_read_workqueue --perf-no_write_workqueue`),
ZFS with `encryption=aes-256-gcm`,
`keyformat=raw`,
`keylocation=file://`.

`man 4 zfs` on the installed 2.4.4 says the default suite for
`encryption=on` is `aes-256-gcm`,
and `keyformat=raw` needs exactly 32 random bytes
(`man 7 zfsprops`,
`keyformat`),
which is what `keylocation=file:///root/exp/zfs.key` holds.

The dm-crypt options come from
`Documentation/admin-guide/device-mapper/dm-crypt.rst:129-134`
in the 7.2 kernel tree:
they "bypass dm-crypt internal workqueue and process read requests
synchronously",
which removes a thread hop and a context switch per I/O on a single-device
volume.
With one device there is nothing for a workqueue to overlap,
so the option costs nothing and saves two dispatches per request.

## The workload

One round is the repository's own shape at the host's concurrency:

- 20 workers start together.
- Each snapshot or clone the base tree:
  `btrfs subvolume snapshot` of a writable subvolume,
  or
  `zfs clone` of `tank/base@base`.
- Each runs `git worktree add` inside its copy,
  which checks out the tracked files of this repository,
  with plain Git defaults (`git version 2.56.0`,
  no `core.fsync` setting).
- Each makes three commits:
  append a file,
  `git add`,
  `git commit`.
- Each deletes a build tree shaped like a `cargo target` directory,
  the way `cargo-shadow lean` empties one:
  8000 files of 400 bytes in 80 directories.
- Each removes the worktree (`git worktree remove --force`)
  and destroys its copy (`btrfs subvolume delete --recursive`,
  `zfs destroy -r -f`).

The base tree is this repository's tracked files at most 32 KiB each:
20 357 files,
143.9 MiB,
in the repository's own directory layout.
Files above that size were left out so that twenty concurrent checkouts fit
inside a round's time budget;
the count,
the names and the directory fan-out are the repository's.

Three instruments run during every round:

- A same-filesystem probe writes 400 bytes and `fsync`s once a second in its
  own subvolume or dataset,
  recording every duration.
  This is both the `fsync` latency figure and the longest stall figure.
- A sampler reads `/proc/<pid>/stack` and `wchan` for each filesystem's
  background threads and for the 20 workers every 200 ms,
  and tallies the first frame past the scheduler,
  so `cv_wait_common` becomes the ZFS function that called it.
- Before and after each round the harness records the filesystem's own
  counters:
  btrfs `commit_stats` from
  `/sys/fs/btrfs/<uuid>/commit_stats`
  (`fs/btrfs/sysfs.c:1143-1191` in 7.2),
  ZFS `txgs`,
  `dmu_tx` and `zil` kstats and `zpool get freeing`.

Between rounds both arms are unmounted and the page cache is dropped,
so each arm starts cold and neither inherits the other's cache.

## Settings, and why each one

Every setting below is named with the document or source line that carries it.
Nothing here trades durability:
no `sync=disabled`,
no barrier off.
Where a setting only lengthens the window for data no program asked to sync,
it is marked as such.

### btrfs

Mounted:
`noatime,compress=zstd:3,space_cache=v2,discard=async`,
`commit=` left at its default of 30 seconds.

- `noatime`:
  no access-time update on every read of every file.
  `fs/btrfs/super.c` passes it to the generic VFS parser,
  and `man 5 btrfs` lists it among the generic mount options.
  A checkout reads tens of thousands of files;
  `relatime` still writes one inode update per newly read file,
  which is pure write amplification for this workload.
- `compress=zstd:3`:
  `man 5 btrfs` documents it as a mount option
  (`fs/btrfs/super.c:223-224` parses it),
  and level 3 is the level `btrfs-compress` docs and the host volume both use.
  Source text is mostly text;
  compression cuts both the bytes to the device and the writeback stalls.
- `space_cache=v2` (the free-space tree):
  `fs/btrfs/super.c` `Opt_space_cache_version` sets `FREE_SPACE_TREE`,
  which is the current format and the one the host volume already uses
  (`doc/planning/filesystem-for-the-repository-volume.md`).
  It removes the v1 free-space inode and its lock from the allocation path.
- `discard=async`:
  `fs/btrfs/super.c:231-232` parses `discard=` and its modes;
  `async` queues discards instead of issuing them inline.
  This is the setting the owner chose for the host volume on 2026-11-10
  (`doc/planning/filesystem-for-the-repository-volume.md`,
  "Owner decisions").
- `commit=` default:
  30 seconds,
  `BTRFS_DEFAULT_COMMIT_INTERVAL` in `fs/btrfs/fs.h:349`.
  A shorter interval buys nothing here:
  `git`'s own `fsync` already forces a
  commit on the log path,
  and every round's work is measured while `git` waits.
- `flushoncommit` left off,
  its default:
  `fs/btrfs/super.c:234` parses it.
  Turning it on makes every commit wait for all dirty data,
  which lengthens
  `commit` for work nobody asked to be synced.
- Metadata profile `DUP`,
  which is `mkfs.btrfs`'s default on one device,
  and data `single`.
  Reported as `Metadata,DUP` by `btrfs filesystem usage`.
- Quotas off:
  `btrfs quota disable`,
  the default.
  The host volume's qgroups were the first suspect in the stall
  investigation and are absent here on purpose,
  so this run measures btrfs without them.
- `autodefrag` off,
  its default (`fs/btrfs/super.c:219`).
  It rewrites files in the background,
  which is more of the load being
  measured,
  not less.
- `block-group-tree` not enabled.
  It is a `mkfs` time feature that `mkfs.btrfs` credits with a faster mount,
  and the repository volume does not have it;
  enabling it would have compared two things at once.

### ZFS

Pool created with:
`ashift=12`,
`compression=zstd-3`,
`atime=off`,
`xattr=sa`,
`dnodesize=auto`,
`recordsize=128K`,
`logbias=latency`,
`sync=standard`,
`encryption=aes-256-gcm`,
`keyformat=raw`.

Module parameters changed:
`zfs_per_txg_dirty_frees_percent=100` and `zfs_txg_timeout=1`.
Everything else stayed at its default and is listed with its default below.

- `ashift=12`:
  the smallest shift that matches a 4 KiB device,
  which is what the NVMe and any SATA SSD this pool could land on report.
  `man 7 zpoolprops` (`ashift`) says ZFS cannot change it later,
  and a pool created with 9 on 4 KiB media writes read-modify-write blocks.
- `compression=zstd-3`:
  the same level as the btrfs arm,
  so neither side gets a compression advantage.
- `atime=off`:
  `man 7 zfsprops` documents `atime=off` as not updating access time on read,
  the same reason as `noatime` above.
- `xattr=sa`:
  `man 7 zfsprops` says with `xattr=sa` "no additional internal objects are
  normally required" for extended attributes;
  with `xattr=on` (the default) each file that has one gets extra objects.
  Git and a checkout write modes and ownership,
  not xattrs,
  but the repository has files that carry them and
  `sa` removes an object per such file.
- `dnodesize=auto`:
  `man 7 zfsprops` says to consider `auto` "if the dataset uses the
  `xattr=sa` property setting and the workload makes heavy use of extended
  attributes";
  it needs the `large_dnode` pool feature,
  which 2.4.4 enables by default.
- `recordsize=128K`,
  the default:
  `man 7 zfsprops` says `recordsize` is "designed solely for use with database
  workloads" and "use of this property for general purpose file systems is
  strongly discouraged".
  A repository of small files is the general purpose case,
  so the default stays.
- `logbias=latency`,
  the default:
  `man 7 zfsprops` says `latency` (the default) uses log devices for low
  latency and `throughput` optimises for pool throughput.
  There is no separate log device here,
  so this arm runs with the default that
  a fresh install gets.
- `sync=standard`,
  the default:
  `fsync` and its dependants wait for the intent
  log;
  `disabled` would be a durability trade and is not used.
- `zfs_per_txg_dirty_frees_percent=100` (default 30 in 2.4.4,
  5 in 2.2.2):
  `module/zfs/dmu.c:77` holds the default,
  `module/zfs/dmu.c:1047-1050` turns it into
  `percent * zfs_dirty_data_max / 100`,
  and `dmu_free_long_range_impl` waits for the next transaction group to open
  once the frees it has queued reach that threshold.
  The earlier VM run measured every deleter waiting 98.8 to 99.6 percent of its
  blocked time in exactly that wait
  (`doc/planning/zfs-and-btrfs-delete-stalls-in-a-vm.md`).
  100 is the largest value the code takes before it falls back to
  `zfs_dirty_data_max / 20`,
  so this is the most free work one transaction
  group can absorb without an extra wait.
- `zfs_txg_timeout=1` (default 5,
  `module/zfs/txg.c:116`):
  when the free throttle is reached,
  `dmu_free_long_range_impl` calls `txg_wait_open`,
  so a deleter waits for the open transaction group to close.
  A group closes when it has 20 percent of `zfs_dirty_data_max` dirty
  (`zfs_dirty_data_sync_percent`,
  `module/zfs/dsl_pool.c:120`)
  or when `zfs_txg_timeout` expires,
  whichever is first
  (`module/zfs/txg.c:62`:
  "There is always a txg in the open state").
  A delete-heavy round has almost no dirty data,
  so the timer is what closes the group:
  at the default 5 seconds every free step costs up to 5 seconds of waiting.
  Shortening it to 1 second divides that wait by five
  and pays in five times as many transaction groups,
  each of which syncs metadata.
  This is the setting the run turns into a measurable difference;
  see the sensitivity rounds.
- Defaults left alone,
  with their source lines:
  - `zfs_dirty_data_max` is 10 percent of memory,
    capped at 25 percent
    (`module/zfs/dsl_pool.c:103-108`).
  - `zfs_delay_min_dirty_percent` 60 and `zfs_delay_scale` 10^9/2000
    (`module/zfs/dsl_pool.c:127,142`) are ZFS's own write throttle:
    above 60 percent dirty,
    every new transaction is delayed
    (`dmu_tx_delay`,
    `module/zfs/dmu_tx.c`).
    Lowering them trades latency for throughput and is not a durability
    question,
    but nothing here shows it would help this workload.
  - `zfs_free_min_time_ms` 500 and `zfs_async_block_max_blocks` unlimited
    (`module/zfs/dsl_scan.c:198,209`):
    the background free pacer.
    Unlimited means no artificial block count cap.
  - vdev queue depths `zfs_vdev_sync_write_max_active=10`,
    `zfs_vdev_async_write_max_active=10`,
    `zfs_vdev_max_active=1000`
    (`module/zfs/vdev_queue.c:125,148,152`).
    One device is the bottleneck;
    deeper queues only lengthen it.
  - ARC size:
    `zfs_arc_max` is set to 8 589 934 592 bytes,
    half the guest's
    memory.
    Without it,
    the measured default `c_max` was 7 242 817 536 bytes of
    8 316 559 360 bytes of memory (87 percent),
    and at 8 GiB of RAM that
    default ended a round in the out-of-memory killer with twenty concurrent
    checkouts.
    Half the memory is the documented way to keep ZFS's cache from crowding
    out the workload's own memory on a workstation
    (`man 4 zfs`,
    `zfs_arc_max`),
    and it is a page-cache equivalent for the
    other arm:
    btrfs's page cache is reclaimable on demand by the same
    memory-pressure path that left ARC unbounded.
    This is the one setting where the two arms do not get the same cache
    ceiling,
    and it is listed again under what these runs cannot tell.

### What both were given that neither defaults to

- Equal memory,
  no ARC limit,
  no page cache limit.
- Equal compression level.
- Equal encryption presence.
- Equal fault injection at the device:
  both disks are throttled by the same
  mechanism to the same profile,
  or neither is.
- A fresh filesystem for each phase,
  so no round inherits fragmentation from
  an earlier round's deletes.

## Throttling the subject disks to the 860's profile

The repository's volume measures 99 percent busy at about 6 200 operations a
second,
about 3 ms each,
mean queue 20.6 of 32
(`doc/planning/filesystem-for-the-repository-volume.md`,
section "The drive").
The limited phase caps both subject disks at that operation rate with
libvirt's own throttle:

```bash
# host, per subject disk, from the same libvirt that runs the VM
virsh blkdeviotune mvm-btrfs-zfs-best vdb --total_iops_sec 6200 --live
virsh blkdeviotune mvm-btrfs-zfs-best vdc --total_iops_sec 6200 --live
```

QEMU's token bucket sits below the guest's block layer,
so the cap applies to reads and writes from every process at once,
and the guest's own `/proc/diskstats` shows the rate it actually got.
`iotune` cannot add the drive's 3 ms per operation,
only cap the rate;
with twenty writers keeping the queue full,
the average operation still waits
about queue depth divided by 6 200 per second.
That is an approximation and is listed again under what these runs cannot tell.

The unlimited phase resets the cap:

```bash
virsh blkdeviotune mvm-btrfs-zfs-best vdb --total_iops_sec 0 --live
virsh blkdeviotune mvm-btrfs-zfs-best vdc --total_iops_sec 0 --live
```

## Answer in one paragraph

With both filesystems tuned,
neither wins the time a person waits.
The twenty-worker round took 66 to 131 seconds on btrfs and 64 to 127 on ZFS in
the throttled phase,
and 86 to 122 against 78 to 188 unthrottled;
the paired difference changes sign between rounds,
so that spread is the answer,
not a ranking.
What does separate them is the ordinary `fsync` latency a probe sees while the
round runs:
btrfs's median is 208 to 491 ms in every throttled round and 192 to
380 unthrottled,
ZFS's is 19 to 30 ms and 19 to 78,
against a host control probe whose own median is about 3 ms in the same windows.
Both filesystems still produce multi-second stalls,
and the host's own probe stalls by the same seconds in the same windows,
so the worst-case stalls in this VM cannot be attributed to either filesystem.
ZFS pays for its steadier latency in bytes (3.5 to 4.4 GiB written per round
against btrfs's 2.7 to 3.0),
in clone destruction (2 to 9 seconds against 0.02
to 0.1),
and in a background free queue that is still working after the round
ends when `zfs_per_txg_dirty_frees_percent` and `zfs_txg_timeout` are left at
their defaults.

## Run-to-run spread first

Five rounds per arm per profile,
arms alternating inside each round
(btrfs first on odd rounds,
ZFS first on even ones),
and a fresh filesystem for each profile so no round inherits an earlier round's
fragmentation.

Round length,
in seconds,
in the order the rounds ran:

- Throttled btrfs:
  130.8,
  76.3,
  119.7,
  66.2,
  96.8.
  Median 96.8,
  range 64.7.
- Throttled ZFS:
  79.1,
  80.0,
  126.1,
  64.0,
  127.0.
  Median 80.0,
  range 63.0.
- Unthrottled btrfs:
  97.2,
  86.3,
  103.5,
  100.0,
  121.7.
  Median 100.0,
  range 35.4.
- Unthrottled ZFS:
  126.8,
  87.0,
  78.4,
  81.7,
  188.3.
  Median 87.0,
  range 109.9.

Paired by round,
the throttled difference (btrfs minus ZFS) was
+51.7,
-3.6,
-6.4,
+2.2 and -30.2 seconds:
median -3.6,
mean +2.7,
three rounds one way and two the other.
`git worktree add`,
median of the twenty workers,
paired the same way:
115.4 against 53.3,
51.3 against 52.6,
103.4 against 85.8,
48.0 against 36.1,
75.8 against 91.5 seconds,
median difference +11.9 seconds,
again with rounds on both sides.

A difference smaller than the spread is no difference,
and these differences are smaller than the spread.
The only quantity that separates the arms in every round,
throttled and not,
is the probe's median.

## The rounds

Every round is listed with its own numbers;
nothing is averaged away.
"Steps" are the median and the worst of the twenty workers,
"device" is what the guest's `/proc/diskstats` counted for the subject device
between the start and the end of the round,
and "background threads" is where the sampler found the filesystem's own
threads when it looked every 200 ms.

### Limited profile rounds

#### Round 1, started 2026-11-11T04:16:50+0000

- btrfs:
  round 130.786 s,
  settle 2.001 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.654 s (worst 1.567 s),
    `worktree_add` 115.361 s (worst 116.123 s),
    `commit_0` 1.156 s (worst 1.891 s),
    `commit_1` 1.003 s (worst 7.483 s),
    `commit_2` 0.804 s (worst 1.083 s),
    `delete_build_tree` 1.745 s (worst 6.241 s),
    `worktree_remove` 9.504 s (worst 9.536 s),
    `destroy` 0.051 s (worst 0.096 s).
  - Probe:
    67 samples,
    p50 385.91 ms,
    p99 6566.608 ms,
    worst 12257.552 ms,
    50 over 200 ms.
  - Device:
    2991 MiB written,
    963 MiB read,
    131.6 s busy in the round.
  - Background threads waited in `btrfs-cleaner:cleaner_kthread` 50.0 percent,
    `btrfs-transaction:transaction_kthread` 48.2 percent,
    `btrfs-transaction:wait_for_commit` 1.8 percent.

- zfs:
  round 79.097 s,
  settle 1.524 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.756 s (worst 0.768 s),
    `worktree_add` 53.317 s (worst 58.886 s),
    `commit_0` 2.301 s (worst 5.619 s),
    `commit_1` 1.012 s (worst 3.311 s),
    `commit_2` 0.77 s (worst 3.402 s),
    `delete_build_tree` 3.639 s (worst 4.76 s),
    `worktree_remove` 11.59 s (worst 12.905 s),
    `destroy` 2.375 s (worst 16.374 s).
  - Probe:
    62 samples,
    p50 29.565 ms,
    p99 4666.934 ms,
    worst 8684.593 ms,
    10 over 200 ms.
  - Device:
    3565 MiB written,
    177 MiB read,
    61.6 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 25.5 percent,
    `z_wr_iss:taskq_thread` 24.8 percent,
    `z_rd_int:taskq_thread` 15.3 percent,
    `z_null_iss:taskq_thread` 9.4 percent.

#### Round 2, started 2026-11-11T04:20:40+0000

- btrfs:
  round 76.321 s,
  settle 1.502 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 1.375 s (worst 2.906 s),
    `worktree_add` 51.34 s (worst 60.649 s),
    `commit_0` 1.612 s (worst 9.742 s),
    `commit_1` 1.193 s (worst 9.844 s),
    `commit_2` 1.158 s (worst 9.811 s),
    `delete_build_tree` 2.721 s (worst 16.05 s),
    `worktree_remove` 7.177 s (worst 7.823 s),
    `destroy` 0.025 s (worst 0.077 s).
  - Probe:
    53 samples,
    p50 222.557 ms,
    p99 3114.999 ms,
    worst 8797.464 ms,
    29 over 200 ms.
  - Device:
    2672 MiB written,
    963 MiB read,
    62.9 s busy in the round.
  - Background threads waited in `btrfs-cleaner:cleaner_kthread` 50.0 percent,
    `btrfs-transaction:transaction_kthread` 50.0 percent.

- zfs:
  round 79.968 s,
  settle 1.532 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.252 s (worst 0.268 s),
    `worktree_add` 52.599 s (worst 57.079 s),
    `commit_0` 1.52 s (worst 2.599 s),
    `commit_1` 1.138 s (worst 2.764 s),
    `commit_2` 1.1 s (worst 6.984 s),
    `delete_build_tree` 4.977 s (worst 12.237 s),
    `worktree_remove` 15.605 s (worst 19.122 s),
    `destroy` 2.076 s (worst 13.96 s).
  - Probe:
    57 samples,
    p50 20.537 ms,
    p99 5336.015 ms,
    worst 10475.428 ms,
    9 over 200 ms.
  - Device:
    3619 MiB written,
    122 MiB read,
    58.3 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 24.6 percent,
    `z_wr_iss:taskq_thread` 23.2 percent,
    `z_rd_int:taskq_thread` 21.3 percent,
    `z_null_iss:taskq_thread` 8.7 percent.

#### Round 3, started 2026-11-11T04:23:32+0000

- btrfs:
  round 119.676 s,
  settle 1.501 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.832 s (worst 3.863 s),
    `worktree_add` 103.388 s (worst 103.97 s),
    `commit_0` 1.776 s (worst 5.7 s),
    `commit_1` 1.057 s (worst 5.665 s),
    `commit_2` 1.053 s (worst 7.727 s),
    `delete_build_tree` 2.477 s (worst 12.181 s),
    `worktree_remove` 8.171 s (worst 8.686 s),
    `destroy` 0.024 s (worst 0.121 s).
  - Probe:
    55 samples,
    p50 481.872 ms,
    p99 8241.539 ms,
    worst 9372.944 ms,
    40 over 200 ms.
  - Device:
    2715 MiB written,
    968 MiB read,
    106.0 s busy in the round.
  - Background threads waited in `btrfs-transaction:transaction_kthread` 50.0 percent,
    `btrfs-cleaner:cleaner_kthread` 49.8 percent,
    `btrfs-cleaner:read_extent_buffer_pages` 0.1 percent,
    `btrfs-cleaner:handle_reserve_ticket` 0.1 percent.

- zfs:
  round 126.059 s,
  settle 1.525 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.164 s (worst 0.193 s),
    `worktree_add` 85.828 s (worst 86.177 s),
    `commit_0` 1.934 s (worst 2.489 s),
    `commit_1` 1.354 s (worst 2.085 s),
    `commit_2` 1.285 s (worst 2.477 s),
    `delete_build_tree` 4.58 s (worst 5.933 s),
    `worktree_remove` 20.697 s (worst 28.506 s),
    `destroy` 3.991 s (worst 4.882 s).
  - Probe:
    120 samples,
    p50 28.138 ms,
    p99 790.633 ms,
    worst 3726.831 ms,
    10 over 200 ms.
  - Device:
    4375 MiB written,
    244 MiB read,
    110.9 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 26.0 percent,
    `z_wr_iss:taskq_thread` 25.3 percent,
    `z_rd_int:taskq_thread` 18.6 percent,
    `z_null_iss:taskq_thread` 9.0 percent.

#### Round 4, started 2026-11-11T04:27:58+0000

- btrfs:
  round 66.241 s,
  settle 1.501 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 3.628 s (worst 9.931 s),
    `worktree_add` 47.993 s (worst 51.112 s),
    `commit_0` 0.858 s (worst 3.684 s),
    `commit_1` 0.653 s (worst 1.053 s),
    `commit_2` 0.616 s (worst 2.598 s),
    `delete_build_tree` 2.903 s (worst 14.69 s),
    `worktree_remove` 7.216 s (worst 7.605 s),
    `destroy` 0.015 s (worst 0.025 s).
  - Probe:
    43 samples,
    p50 208.375 ms,
    p99 3859.537 ms,
    worst 3859.537 ms,
    23 over 200 ms.
  - Device:
    3006 MiB written,
    963 MiB read,
    76.2 s busy in the round.
  - Background threads waited in `btrfs-cleaner:cleaner_kthread` 50.0 percent,
    `btrfs-transaction:transaction_kthread` 50.0 percent.

- zfs:
  round 64.0 s,
  settle 1.537 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.811 s (worst 0.838 s),
    `worktree_add` 36.098 s (worst 47.891 s),
    `commit_0` 1.742 s (worst 5.023 s),
    `commit_1` 1.165 s (worst 2.712 s),
    `commit_2` 1.377 s (worst 3.227 s),
    `delete_build_tree` 5.249 s (worst 10.533 s),
    `worktree_remove` 9.285 s (worst 13.003 s),
    `destroy` 7.947 s (worst 14.221 s).
  - Probe:
    60 samples,
    p50 19.063 ms,
    p99 1180.185 ms,
    worst 1964.045 ms,
    10 over 200 ms.
  - Device:
    3536 MiB written,
    63 MiB read,
    37.9 s busy in the round.
  - Background threads waited in `z_wr_iss:taskq_thread` 24.2 percent,
    `z_wr_int:taskq_thread` 24.1 percent,
    `z_rd_int:taskq_thread` 18.5 percent,
    `z_null_iss:taskq_thread` 9.0 percent.

#### Round 5, started 2026-11-11T04:30:40+0000

- btrfs:
  round 96.793 s,
  settle 1.501 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 1.237 s (worst 21.418 s),
    `worktree_add` 75.782 s (worst 77.754 s),
    `commit_0` 1.351 s (worst 2.348 s),
    `commit_1` 0.825 s (worst 1.74 s),
    `commit_2` 0.81 s (worst 1.884 s),
    `delete_build_tree` 5.985 s (worst 9.879 s),
    `worktree_remove` 8.737 s (worst 8.808 s),
    `destroy` 0.078 s (worst 0.135 s).
  - Probe:
    50 samples,
    p50 491.36 ms,
    p99 14271.033 ms,
    worst 14271.033 ms,
    40 over 200 ms.
  - Device:
    2999 MiB written,
    964 MiB read,
    89.2 s busy in the round.
  - Background threads waited in `btrfs-transaction:transaction_kthread` 50.0 percent,
    `btrfs-cleaner:cleaner_kthread` 49.8 percent,
    `btrfs-cleaner:0` 0.1 percent,
    `btrfs-cleaner:read_extent_buffer_pages` 0.1 percent.

- zfs:
  round 127.003 s,
  settle 1.563 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.183 s (worst 0.206 s),
    `worktree_add` 91.534 s (worst 91.844 s),
    `commit_0` 1.95 s (worst 4.157 s),
    `commit_1` 1.238 s (worst 2.432 s),
    `commit_2` 1.107 s (worst 1.86 s),
    `delete_build_tree` 3.391 s (worst 4.619 s),
    `worktree_remove` 17.358 s (worst 19.367 s),
    `destroy` 9.296 s (worst 14.335 s).
  - Probe:
    106 samples,
    p50 24.041 ms,
    p99 7305.882 ms,
    worst 8943.011 ms,
    10 over 200 ms.
  - Device:
    4079 MiB written,
    304 MiB read,
    100.2 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 25.8 percent,
    `z_wr_iss:taskq_thread` 25.4 percent,
    `z_rd_int:taskq_thread` 17.8 percent,
    `z_null_iss:taskq_thread` 9.1 percent.

### Unlimited profile rounds

#### Round 1, started 2026-11-11T04:36:26+0000

- btrfs:
  round 97.197 s,
  settle 1.501 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.271 s (worst 0.504 s),
    `worktree_add` 82.31 s (worst 82.862 s),
    `commit_0` 1.148 s (worst 2.002 s),
    `commit_1` 1.085 s (worst 1.347 s),
    `commit_2` 1.081 s (worst 1.452 s),
    `delete_build_tree` 2.176 s (worst 2.366 s),
    `worktree_remove` 8.497 s (worst 8.813 s),
    `destroy` 0.036 s (worst 0.082 s).
  - Probe:
    62 samples,
    p50 192.285 ms,
    p99 4872.479 ms,
    worst 15328.221 ms,
    31 over 200 ms.
  - Device:
    3074 MiB written,
    963 MiB read,
    82.3 s busy in the round.
  - Background threads waited in `btrfs-cleaner:cleaner_kthread` 50.0 percent,
    `btrfs-transaction:transaction_kthread` 50.0 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 46.6 percent,
    `git:handle_reserve_ticket` 43.2 percent,
    `git:0` 8.3 percent,
    `git:filemap_fault` 1.1 percent.

- zfs:
  round 126.836 s,
  settle 1.533 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.65 s (worst 0.667 s),
    `worktree_add` 63.75 s (worst 68.005 s),
    `commit_0` 1.843 s (worst 4.788 s),
    `commit_1` 1.038 s (worst 3.974 s),
    `commit_2` 1.415 s (worst 3.544 s),
    `delete_build_tree` 4.079 s (worst 7.001 s),
    `worktree_remove` 43.101 s (worst 50.211 s),
    `destroy` 2.388 s (worst 6.41 s).
  - Probe:
    79 samples,
    p50 78.643 ms,
    p99 8490.248 ms,
    worst 16535.44 ms,
    18 over 200 ms.
  - Device:
    3843 MiB written,
    200 MiB read,
    115.3 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 26.0 percent,
    `z_wr_iss:taskq_thread` 25.4 percent,
    `z_rd_int:taskq_thread` 18.2 percent,
    `z_null_iss:taskq_thread` 8.9 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 34.1 percent,
    `git:__cv_timedwait_io` 30.7 percent,
    `git:__cv_wait_io` 18.6 percent,
    `git:0` 11.7 percent.

#### Round 2, started 2026-11-11T04:40:27+0000

- btrfs:
  round 86.279 s,
  settle 1.501 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 6.966 s (worst 9.255 s),
    `worktree_add` 66.591 s (worst 67.275 s),
    `commit_0` 0.908 s (worst 1.727 s),
    `commit_1` 0.775 s (worst 1.185 s),
    `commit_2` 0.731 s (worst 1.362 s),
    `delete_build_tree` 2.788 s (worst 14.344 s),
    `worktree_remove` 6.87 s (worst 8.124 s),
    `destroy` 0.028 s (worst 0.064 s).
  - Probe:
    58 samples,
    p50 202.281 ms,
    p99 1935.386 ms,
    worst 15561.719 ms,
    31 over 200 ms.
  - Device:
    2732 MiB written,
    963 MiB read,
    72.4 s busy in the round.
  - Background threads waited in `btrfs-cleaner:cleaner_kthread` 50.0 percent,
    `btrfs-transaction:transaction_kthread` 50.0 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 45.3 percent,
    `git:handle_reserve_ticket` 41.6 percent,
    `git:0` 8.5 percent,
    `btrfs:btrfs_opendir` 2.6 percent.

- zfs:
  round 87.0 s,
  settle 1.524 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.281 s (worst 0.31 s),
    `worktree_add` 61.891 s (worst 68.738 s),
    `commit_0` 1.201 s (worst 11.105 s),
    `commit_1` 1.091 s (worst 1.706 s),
    `commit_2` 1.419 s (worst 10.281 s),
    `delete_build_tree` 4.087 s (worst 13.894 s),
    `worktree_remove` 11.518 s (worst 13.585 s),
    `destroy` 4.249 s (worst 16.14 s).
  - Probe:
    57 samples,
    p50 21.882 ms,
    p99 8951.633 ms,
    worst 14545.013 ms,
    10 over 200 ms.
  - Device:
    3557 MiB written,
    96 MiB read,
    64.5 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 24.6 percent,
    `z_wr_iss:taskq_thread` 24.0 percent,
    `z_rd_int:taskq_thread` 21.3 percent,
    `z_null_iss:taskq_thread` 8.7 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 41.5 percent,
    `git:__cv_wait_io` 31.3 percent,
    `git:0` 14.7 percent,
    `zfs:__cv_wait_io` 4.7 percent.

#### Round 3, started 2026-11-11T04:43:35+0000

- btrfs:
  round 103.526 s,
  settle 1.501 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.179 s (worst 0.718 s),
    `worktree_add` 90.089 s (worst 91.341 s),
    `commit_0` 0.895 s (worst 1.15 s),
    `commit_1` 0.817 s (worst 1.207 s),
    `commit_2` 0.817 s (worst 1.639 s),
    `delete_build_tree` 2.908 s (worst 6.128 s),
    `worktree_remove` 7.379 s (worst 8.28 s),
    `destroy` 0.026 s (worst 0.076 s).
  - Probe:
    57 samples,
    p50 300.686 ms,
    p99 8582.863 ms,
    worst 11043.075 ms,
    40 over 200 ms.
  - Device:
    3048 MiB written,
    963 MiB read,
    92.1 s busy in the round.
  - Background threads waited in `btrfs-cleaner:cleaner_kthread` 50.0 percent,
    `btrfs-transaction:transaction_kthread` 50.0 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 47.8 percent,
    `git:handle_reserve_ticket` 44.8 percent,
    `git:0` 6.9 percent,
    `git:wait_current_trans` 0.2 percent.

- zfs:
  round 78.393 s,
  settle 1.535 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.119 s (worst 0.14 s),
    `worktree_add` 53.121 s (worst 54.249 s),
    `commit_0` 3.565 s (worst 6.492 s),
    `commit_1` 1.257 s (worst 4.496 s),
    `commit_2` 1.147 s (worst 1.738 s),
    `delete_build_tree` 3.264 s (worst 4.947 s),
    `worktree_remove` 10.796 s (worst 13.661 s),
    `destroy` 4.61 s (worst 10.233 s).
  - Probe:
    71 samples,
    p50 19.36 ms,
    p99 620.321 ms,
    worst 8972.295 ms,
    7 over 200 ms.
  - Device:
    3738 MiB written,
    393 MiB read,
    63.1 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 24.7 percent,
    `z_wr_iss:taskq_thread` 24.4 percent,
    `z_rd_int:taskq_thread` 20.3 percent,
    `z_null_iss:taskq_thread` 8.8 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 42.0 percent,
    `git:__cv_timedwait_io` 22.4 percent,
    `git:0` 17.5 percent,
    `git:__cv_wait_io` 9.7 percent.

#### Round 4, started 2026-11-11T04:46:52+0000

- btrfs:
  round 100.019 s,
  settle 2.001 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.851 s (worst 1.554 s),
    `worktree_add` 85.532 s (worst 86.204 s),
    `commit_0` 0.889 s (worst 1.744 s),
    `commit_1` 0.779 s (worst 1.072 s),
    `commit_2` 0.832 s (worst 0.977 s),
    `delete_build_tree` 2.298 s (worst 5.278 s),
    `worktree_remove` 7.839 s (worst 8.343 s),
    `destroy` 0.015 s (worst 0.031 s).
  - Probe:
    50 samples,
    p50 380.378 ms,
    p99 14422.403 ms,
    worst 14422.403 ms,
    36 over 200 ms.
  - Device:
    3471 MiB written,
    963 MiB read,
    97.9 s busy in the round.
  - Background threads waited in `btrfs-cleaner:cleaner_kthread` 50.0 percent,
    `btrfs-transaction:transaction_kthread` 50.0 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 47.3 percent,
    `git:handle_reserve_ticket` 45.1 percent,
    `git:0` 6.7 percent,
    `git:wait_current_trans` 0.2 percent.

- zfs:
  round 81.736 s,
  settle 1.535 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.224 s (worst 0.238 s),
    `worktree_add` 52.981 s (worst 55.93 s),
    `commit_0` 1.305 s (worst 2.49 s),
    `commit_1` 0.927 s (worst 2.22 s),
    `commit_2` 0.982 s (worst 2.242 s),
    `delete_build_tree` 7.575 s (worst 9.98 s),
    `worktree_remove` 9.544 s (worst 10.482 s),
    `destroy` 6.895 s (worst 20.685 s).
  - Probe:
    64 samples,
    p50 26.846 ms,
    p99 3919.855 ms,
    worst 10820.405 ms,
    9 over 200 ms.
  - Device:
    3555 MiB written,
    114 MiB read,
    55.8 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 25.6 percent,
    `z_wr_iss:taskq_thread` 25.3 percent,
    `z_rd_int:taskq_thread` 19.4 percent,
    `z_null_iss:taskq_thread` 8.9 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 42.1 percent,
    `git:__cv_wait_io` 17.8 percent,
    `git:0` 16.2 percent,
    `git:__cv_timedwait_io` 11.8 percent.

#### Round 5, started 2026-11-11T04:50:12+0000

- btrfs:
  round 121.662 s,
  settle 1.502 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 6.476 s (worst 24.626 s),
    `worktree_add` 95.93 s (worst 104.56 s),
    `commit_0` 1.143 s (worst 4.36 s),
    `commit_1` 0.818 s (worst 23.201 s),
    `commit_2` 0.866 s (worst 20.369 s),
    `delete_build_tree` 2.731 s (worst 36.19 s),
    `worktree_remove` 6.466 s (worst 7.426 s),
    `destroy` 0.027 s (worst 0.093 s).
  - Probe:
    53 samples,
    p50 286.693 ms,
    p99 9734.343 ms,
    worst 23490.476 ms,
    29 over 200 ms.
  - Device:
    2691 MiB written,
    969 MiB read,
    109.8 s busy in the round.
  - Background threads waited in `btrfs-transaction:transaction_kthread` 50.0 percent,
    `btrfs-cleaner:cleaner_kthread` 44.8 percent,
    `btrfs-cleaner:handle_reserve_ticket` 2.7 percent,
    `btrfs-cleaner:read_extent_buffer_pages` 2.5 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:do_wait` 45.2 percent,
    `git:handle_reserve_ticket` 43.2 percent,
    `git:0` 5.6 percent,
    `btrfs:start_creating_killable` 2.8 percent.

- zfs:
  round 188.264 s,
  settle 1.538 s,
  20 of 20 workers finished.
  - Steps,
    median and worst of the twenty workers:
    `clone` 0.292 s (worst 0.32 s),
    `worktree_add` 113.612 s (worst 113.864 s),
    `commit_0` 29.684 s (worst 30.421 s),
    `commit_1` 2.022 s (worst 2.88 s),
    `commit_2` 2.033 s (worst 3.072 s),
    `delete_build_tree` 9.055 s (worst 10.512 s),
    `worktree_remove` 24.814 s (worst 28.979 s),
    `destroy` 2.146 s (worst 17.069 s).
  - Probe:
    164 samples,
    p50 36.956 ms,
    p99 2301.168 ms,
    worst 3181.73 ms,
    24 over 200 ms.
  - Device:
    4403 MiB written,
    445 MiB read,
    177.2 s busy in the round.
  - Background threads waited in `z_wr_int:taskq_thread` 25.7 percent,
    `z_wr_iss:taskq_thread` 25.5 percent,
    `z_rd_int:taskq_thread` 18.7 percent,
    `z_null_iss:taskq_thread` 8.9 percent.
  - The work's own processes (git,
    btrfs,
    zfs) were blocked in `git:__cv_timedwait_io` 46.9 percent,
    `git:do_wait` 39.5 percent,
    `git:0` 10.1 percent,
    `zfs:__cv_wait_io` 1.5 percent.

## Where each arm's time went

- Both arms spend the round inside `git worktree add`:
  a checkout of 20 357 files per worker,
  twenty at a time.
  In every round that single step is 60 to 90 percent of the round.
- btrfs's `btrfs-transaction` and `btrfs-cleaner` threads were waiting in
  `transaction_kthread` and `cleaner_kthread` for about half the samples each.
  Those are the threads the owner's host shows in uninterruptible sleep,
  and here they are mostly waiting,
  not running.
- ZFS's own threads were waiting in `z_wr_iss`,
  `z_wr_int`,
  `z_rd_int` and
  `z_null_iss`,
  the write,
  compression and read taskqs,
  spread across the
  samples rather than concentrated in one thread.
- btrfs's deletions finish before the command returns:
  `btrfs subvolume delete` returned in 15 to 96 ms in every round,
  and the build-tree delete
  (`shutil.rmtree` over 8 000 files) took 1.7 to 6.0 seconds.
- ZFS's `zfs destroy -r -f` of a clone returned in 2.1 to 9.3 seconds,
  and `zpool get freeing` was still non-zero after the round in the rounds
  where the sampler caught it,
  so part of ZFS's delete work is
  after the round has ended.
  That work is not in the round's numbers.

## What each filesystem still waits for

### btrfs

The stall the owner reports is in the commit path,
and this VM reproduces the
shape of it:
ordinary `fsync` from an unrelated probe waited 208 to 491 ms at
the median while twenty workers checked out files.

The source lines those waits come from:

- `git`'s `fsync` enters `btrfs_sync_file`,
  which starts a transaction and,
  when the file has been written in the running transaction,
  waits for that transaction to commit
  (`fs/btrfs/file.c`,
  `btrfs_sync_file`,
  in 7.2).
- A commit that the running transaction cannot join waits for the previous one
  to finish:
  `wait_current_trans` in `fs/btrfs/transaction.c`,
  the function the owner's host samples put 41 percent of blocked time in
  (`doc/planning/filesystem-for-the-repository-volume.md`).
- Metadata reservation waits in `btrfs_reserve_metadata_bytes`
  (`fs/btrfs/space-info.c`),
  which is `handle_reserve_ticket` at 25 percent on
  the owner's host.
  In this VM it shows up in the worker samples of
  throttled round 2 (`handle_reserve_ticket` 4.2 percent).
- The commit interval is the 30 second default,
  and every `git` `fsync` can
  force one:
  `BTRFS_DEFAULT_COMMIT_INTERVAL`,
  `fs/btrfs/fs.h:349`.

btrfs's deletions are cheap and synchronous:
`btrfs subvolume delete` only marks extents free and returns,
and the cleaner finishes the work afterwards
(`fs/btrfs/async-thread.c` holds the cleaner kthread's queue),
which is why its `destroy` step is 0.02 to 0.1 seconds and its
build-tree delete is seconds rather than minutes.

### ZFS

ZFS's ordinary `fsync` is much cheaper in this workload:
`zil_commit` writes to the intent log and returns without waiting for a
transaction group,
except when the log has crashed or is suspended
(`module/zfs/zil.c:4010-4037`) or when a log record's data cannot be read
(`module/zfs/zil.c:2484-2508`).
The measured median of 19 to 30 ms is what that looks like from outside.

Where ZFS still waits:

- Snapshot creation is a sync task that waits for its transaction group to
  reach disk:
  `dsl_dataset_snapshot` calls `dsl_sync_task`
  (`module/zfs/dsl_dataset.c:1990-1994`),
  and `dsl_sync_task_common` ends in
  `txg_wait_synced` (`module/zfs/dsl_synctask.c:101`).
  A `zfs clone` of an existing snapshot is not that path,
  which is why the clone step was 0.16 to 0.75 seconds.
- Mass frees are paced per transaction group:
  `dmu_free_long_range_impl` stops adding frees once
  `zfs_per_txg_dirty_frees_percent * zfs_dirty_data_max / 100` is reached and
  then waits for the next group to open
  (`module/zfs/dmu.c:1041-1050` for the threshold,
  `txg_wait_open` in the same loop).
  This is the wait the earlier VM run measured at 98.8 to 99.6 percent of
  blocked deleter samples
  (`doc/planning/zfs-and-btrfs-delete-stalls-in-a-vm.md`).
  Raising the percent to 100 is what this run did;
  it does not remove the wait,
  it raises how much free work one group absorbs.
- A transaction group closes on the `zfs_txg_timeout` timer or on dirty-data
  pressure,
  and there is always one open (`module/zfs/txg.c:55-70`),
  so the timer is the ceiling on each free step when little data is dirty.
  This run set it to 1 second.
  That choice follows from the source and is not
  measured against the default here:
  no round in this campaign ran with `zfs_txg_timeout=5` and the same workload,
  so what the default costs is argued,
  not shown (see what these runs cannot
  tell).
- Background frees of destroyed datasets run inside each transaction group's
  sync,
  paced by `zfs_free_min_time_ms` (500 ms) with
  `zfs_async_block_max_blocks` unlimited for the block count
  (`module/zfs/dsl_scan.c:198,209`).
  That is why `zfs destroy` returns in seconds and why the free queue is still
  draining after the round.

### One extra pair, sampled where the work itself blocked

The five rounds above sampled the filesystems' own threads.
One extra pair,
run after the campaign on the same throttled disks,
sampled the work's own processes as well,
every `git`,
`btrfs` and `zfs` process every 200 ms.
It is one round per arm,
not five,
and the host was busier while it ran,
so its numbers say where processes waited,
not how long a round takes.

- Throttled btrfs,
  326.5 s:
  probe p50 488.1 ms,
  p99 20.0 s,
  worst 21.8 s.
  - The work's own processes were blocked in `do_wait` 48.0 percent
    (`git` waiting for a child `git`),
    `handle_reserve_ticket` 44.3 percent,
    `btrfs_opendir` 1.0 percent,
    `wait_current_trans` 0.1 percent.
  - `btrfs-transaction` and `btrfs-cleaner` were each waiting in their own
    kthread loop for half the samples.
- Throttled ZFS,
  146.9 s:
  probe p50 59.9 ms,
  p99 14.5 s,
  worst 24.8 s.
  - The work's own processes were blocked in `do_wait` 43.4 percent,
    `__cv_wait_io` 16.9 percent,
    `__cv_timedwait_io` 16.3 percent,
    `txg_hold_open` 1.0 percent.
  - `z_wr_int`,
    `z_wr_iss`,
    `z_rd_int` and `z_null_iss` took about a quarter of
    the thread samples each,
    and `txg_sync` sat in `__cv_timedwait_io` for 8.4 percent of them.

After the parents' own `do_wait`,
btrfs's remaining wait is metadata reservation:
`handle_reserve_ticket` is `btrfs_reserve_metadata_bytes` in
`fs/btrfs/space-info.c`,
the wait the owner's host shows at 25 percent.
ZFS's remaining wait is I/O completion plus holding a transaction group open,
which is its free and sync path rather than a reservation.

### The write throttle both of them have

ZFS has its own delay path:
above `zfs_delay_min_dirty_percent` (60 percent of
`zfs_dirty_data_max`) every transaction is delayed by `dmu_tx_delay`
(`module/zfs/dsl_pool.c:118-142`,
`module/zfs/dmu_tx.c`).
It is a smooth delay,
not a lock,
and with 16 GiB of memory
(`zfs_dirty_data_max` would be 10 percent of it,
capped at 25 percent)
this workload never filled it:
the write path stayed in the microseconds-to-
milliseconds part of that curve.

btrfs's equivalent is not a delay but a wait:
writers either join the running
transaction or wait for a commit,
which is why btrfs's waits are quantised in
commit-sized steps while ZFS's ordinary path is not.

## Host interference, measured

The host ran between load average 66 and 97,
with I/O pressure `some` between
36 and 45 percent during the campaign
(other sessions' work,
not this experiment's),
and the control probe that writes 400 bytes and `fsync`s once a second on the
NVMe directory holding the VM's disk images saw:

- 685 samples before the campaign:
  p50 3.1 ms,
  p99 2 167 ms,
  worst 6 342 ms.

Per round,
the host probe inside the same wall-clock window is listed with the
rounds above.
The reading that matters:
the guest's worst stall exceeds the host's own worst in some rounds and not in
others,
so no single worst-case number here can be attributed to a filesystem.
The medians can:
the host's median is about 3 ms,
and the two arms' medians are
an order of magnitude apart.

## What these runs cannot tell

1.  **A throttled NVMe is not the 860.**
    `blkdeviotune` caps operations per second but adds no per-operation
    latency,
    no SATA queue,
    no NCQ,
    no drive scheduler,
    no TRIM timing and no
    bad-block retries.
    The 6 200 operations a second and its 3 ms each come from a real drive
    under a real workload;
    here only the rate is real.
2.  **The two phases did not differ as much as expected.**
    The unthrottled rounds were not faster than the throttled ones
    (medians 100.0 against 96.8 for btrfs,
    87.0 against 80.0 for ZFS),
    because the host's own storage path was the bottleneck in both:
    with `cache=none` every guest flush is an `O_DIRECT` flush of a file on a
    host volume at 82 percent full under load 66 to 97.
    So the "unlimited" arm measures the host's NVMe through a VM,
    not a
    fast device.
3.  **The two arms do not have the same cache ceiling.**
    ARC was capped at half the guest's memory (8 of 16 GiB) because the
    unbounded default ended a round in the guest's out-of-memory killer;
    btrfs's page cache has no such cap and is reclaimable on demand.
    A machine with more memory might give ARC more and might then behave
    differently for a working set larger than 8 GiB.
4.  **The source tree is the repository's shape,
    not its size.**
    Files above 32 KiB were left out:
    20 357 files and 143.9 MiB per checkout
    where the real checkout is 23 083 files and 856 MiB.
    Compression and record sizes behave differently at the larger size,
    and a real checkout writes about six times the bytes.
5.  **The round is a proxy for a working day.**
    It runs the four operations once each per worker,
    all at once,
    for a
    hundred seconds.
    The owner's machine runs them interleaved with builds,
    editors and package installs for hours,
    and filesystem state accumulates
    in ways a fresh filesystem does not.
6.  **No qgroups,
    no snapshots of the real volume,
    no send/receive.**
    btrfs ran without quotas because the host volume's qgroups are a separate
    suspect;
    ZFS ran one dataset tree with no `zfs send`.
7.  **The background work after a round is bounded,
    not measured.**
    The settle detector waits for free space to stop changing,
    up to 180
    seconds.
    It returned in about 1.5 seconds on both arms in every round,
    which means the metric is too coarse to show ZFS's async destroy draining;
    `zpool get freeing` was non-zero after some rounds,
    so the work continued
    after the numbers were taken.
8.  **One VM,
    one kernel,
    one ZFS release.**
    Nothing here says how either filesystem behaves on another kernel,
    another OpenZFS release,
    or with a real SATA link in the path.

## How to repeat it

The harness is scratch,
not repository code,
and lives at
`/var/home/user/nvme-agent/btrfs-zfs-best/`:

- `build/guest-in-container.sh` builds the CachyOS root filesystem,
  its kernel,
  initramfs and grub into a raw image,
  inside a privileged
  container on an Arch base,
  with no mount of the host's own filesystems.
- `harness/make-payload.py` builds the source tree from this repository's
  tracked files and the payload disk.
- `harness/harness.py` is the in-guest campaign driver
  (`setup`,
  `round`,
  `phase`,
  `state`,
  `teardown`).
- `harness/host-probe.py` is the host control probe.
- `host-driver.sh` creates the VM,
  attaches the disks,
  sets the throttle,
  pushes the harness and runs commands.

Raw per-round JSON,
including every probe duration and every sampler tally,
is under `/var/home/user/nvme-agent/btrfs-zfs-best/results/` in the guest and
is not committed.
