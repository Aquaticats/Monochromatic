# btrfs and ZFS, each given its best configuration, on the repository's own workload

## Status

Measured inside one disposable VM made with `package/mvm/cli`
(`mvm create --image cachyos-72-zfs244 btrfs-zfs-best`),
on this machine.
Owner's question, 2026-11-10:

> What if we give both btrfs and zfs benefit of doubt and give the best possible configs they can get?
> For example, zfs supports ARC.

This file holds the settings,
their sources,
every round's numbers,
the waits each filesystem still has,
and what these runs cannot tell.
Numbers are filled in by the campaign described under "How to repeat it".

## The guest

- Built as a CachyOS root filesystem by `pacman --root` inside a privileged
  container, from the CachyOS and Arch repositories as they stood at
  2026-11-11T03:32:00+00:00.
- Kernel `7.2.11-2-cachyos-hardened`, measured with `uname -r`.
  The host runs `7.2.15-ogc1.1.fc45.x86_64`,
  and `zfs-2.4.4`'s `META` lists `Linux-Maximum: 7.2`,
  so this is the same kernel line and the newest one this ZFS supports.
- OpenZFS `2.4.4`: `zfs version` reports `zfs-2.4.4-1` and `zfs-kmod-2.4.4-1`,
  userspace and kernel module from the same release.
- btrfs: `btrfs-progs v7.3`, kernel `fs/btrfs` from `7.2.11`.
- 4 virtual CPUs, 7931 MiB of memory (`free -m`), no balloon.
- One system disk (ext4, 20 GiB, `cache=none`) and three subject disks,
  all virtio-blk:
  `/dev/vdb` 24 GiB raw for btrfs,
  `/dev/vdc` 24 GiB raw for ZFS,
  `/dev/vdd` 512 MiB raw read-only payload.
  The two subject images live on the NVMe (`/var/home/user/nvme-agent`),
  never on the encrypted SATA volume,
  and both are attached with `--cache none`,
  so a guest flush is an `O_DIRECT` flush of the host file:
  both arms pay the same durability price and neither can hide a flush.
- Nothing else ran in the guest: no timers of its own,
  no package manager, no network traffic during a round.

## Why both filesystems are encrypted

The repository volume is LUKS plus btrfs, and a ZFS install would use native
encryption,
so each arm pays for encryption the way an install of it would:
btrfs inside LUKS2 (`aes-xts-plain64`, 512-bit key,
`--perf-no_read_workqueue --perf-no_write_workqueue`),
ZFS with `encryption=aes-256-gcm`, `keyformat=raw`, `keylocation=file://`.

`man 4 zfs` on the installed 2.4.4 says the default suite for
`encryption=on` is `aes-256-gcm`,
and `keyformat=raw` needs exactly 32 random bytes
(`man 7 zfsprops`, `keyformat`),
which is what `keylocation=file:///root/exp/zfs.key` holds.

The dm-crypt options come from
`Documentation/admin-guide/device-mapper/dm-crypt.rst:129-134`
in the 7.2 kernel tree:
they "bypass dm-crypt internal workqueue and process read requests
synchronously",
which removes a thread hop and a context switch per I/O on a single-device
volume. With one device there is nothing for a workqueue to overlap,
so the option costs nothing and saves two dispatches per request.

## The workload

One round is the repository's own shape at the host's concurrency:

- 20 workers start together.
- Each snapshot or clone the base tree:
  `btrfs subvolume snapshot` of a writable subvolume, or
  `zfs clone` of `tank/base@base`.
- Each runs `git worktree add` inside its copy,
  which checks out the tracked files of this repository,
  with plain Git defaults (`git version 2.56.0`, no `core.fsync` setting).
- Each makes three commits: append a file, `git add`, `git commit`.
- Each deletes a build tree shaped like a `cargo target` directory,
  the way `cargo-shadow lean` empties one:
  8000 files of 400 bytes in 80 directories.
- Each removes the worktree (`git worktree remove --force`)
  and destroys its copy (`btrfs subvolume delete --recursive`,
  `zfs destroy -r -f`).

The base tree is this repository's tracked files at most 32 KiB each:
20 357 files, 143.9 MiB, in the repository's own directory layout.
Files above that size were left out so that twenty concurrent checkouts fit
inside a round's time budget;
the count, the names and the directory fan-out are the repository's.

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
  counters: btrfs `commit_stats` from
  `/sys/fs/btrfs/<uuid>/commit_stats`
  (`fs/btrfs/sysfs.c:1143-1191` in 7.2),
  ZFS `txgs`, `dmu_tx` and `zil` kstats and `zpool get freeing`.

Between rounds both arms are unmounted and the page cache is dropped,
so each arm starts cold and neither inherits the other's cache.

## Settings, and why each one

Every setting below is named with the document or source line that carries it.
Nothing here trades durability: no `sync=disabled`, no barrier off.
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
  30 seconds, `BTRFS_DEFAULT_COMMIT_INTERVAL` in `fs/btrfs/fs.h:349`.
  A shorter interval buys nothing here: `git`'s own `fsync` already forces a
  commit on the log path, and every round's work is measured while `git` waits.
- `flushoncommit` left off, its default:
  `fs/btrfs/super.c:234` parses it.
  Turning it on makes every commit wait for all dirty data, which lengthens
  `commit` for work nobody asked to be synced.
- Metadata profile `DUP`, which is `mkfs.btrfs`'s default on one device,
  and data `single`.
  Reported as `Metadata,DUP` by `btrfs filesystem usage`.
- Quotas off: `btrfs quota disable`, the default.
  The host volume's qgroups were the first suspect in the stall
  investigation and are absent here on purpose,
  so this run measures btrfs without them.
- `autodefrag` off, its default (`fs/btrfs/super.c:219`).
  It rewrites files in the background, which is more of the load being
  measured, not less.
- `block-group-tree` not enabled.
  It is a `mkfs` time feature that `mkfs.btrfs` credits with a faster mount,
  and the repository volume does not have it;
  enabling it would have compared two things at once.

### ZFS

Pool created with:
`ashift=12`, `compression=zstd-3`, `atime=off`, `xattr=sa`,
`dnodesize=auto`, `recordsize=128K`, `logbias=latency`, `sync=standard`,
`encryption=aes-256-gcm`, `keyformat=raw`.

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
  Git and a checkout write modes and ownership, not xattrs,
  but the repository has files that carry them and
  `sa` removes an object per such file.
- `dnodesize=auto`:
  `man 7 zfsprops` says to consider `auto` "if the dataset uses the
  `xattr=sa` property setting and the workload makes heavy use of extended
  attributes";
  it needs the `large_dnode` pool feature, which 2.4.4 enables by default.
- `recordsize=128K`, the default:
  `man 7 zfsprops` says `recordsize` is "designed solely for use with database
  workloads" and "use of this property for general purpose file systems is
  strongly discouraged".
  A repository of small files is the general purpose case,
  so the default stays.
- `logbias=latency`, the default:
  `man 7 zfsprops` says `latency` (the default) uses log devices for low
  latency and `throughput` optimises for pool throughput.
  There is no separate log device here, so this arm runs with the default that
  a fresh install gets.
- `sync=standard`, the default: `fsync` and its dependants wait for the intent
  log; `disabled` would be a durability trade and is not used.
- `zfs_per_txg_dirty_frees_percent=100` (default 30 in 2.4.4, 5 in 2.2.2):
  `module/zfs/dmu.c:77` holds the default,
  `module/zfs/dmu.c:1047-1050` turns it into
  `percent * zfs_dirty_data_max / 100`,
  and `dmu_free_long_range_impl` waits for the next transaction group to open
  once the frees it has queued reach that threshold.
  The earlier VM run measured every deleter waiting 98.8 to 99.6 percent of its
  blocked time in exactly that wait
  (`doc/planning/zfs-and-btrfs-delete-stalls-in-a-vm.md`).
  100 is the largest value the code takes before it falls back to
  `zfs_dirty_data_max / 20`, so this is the most free work one transaction
  group can absorb without an extra wait.
- `zfs_txg_timeout=1` (default 5, `module/zfs/txg.c:116`):
  when the free throttle is reached,
  `dmu_free_long_range_impl` calls `txg_wait_open`,
  so a deleter waits for the open transaction group to close.
  A group closes when it has 20 percent of `zfs_dirty_data_max` dirty
  (`zfs_dirty_data_sync_percent`, `module/zfs/dsl_pool.c:120`)
  or when `zfs_txg_timeout` expires, whichever is first
  (`module/zfs/txg.c:62`: "There is always a txg in the open state").
  A delete-heavy round has almost no dirty data,
  so the timer is what closes the group:
  at the default 5 seconds every free step costs up to 5 seconds of waiting.
  Shortening it to 1 second divides that wait by five
  and pays in five times as many transaction groups,
  each of which syncs metadata.
  This is the setting the run turns into a measurable difference;
  see the sensitivity rounds.
- Defaults left alone, with their source lines:
  - `zfs_dirty_data_max` is 10 percent of memory, capped at 25 percent
    (`module/zfs/dsl_pool.c:103-108`).
  - `zfs_delay_min_dirty_percent` 60 and `zfs_delay_scale` 10^9/2000
    (`module/zfs/dsl_pool.c:127,142`) are ZFS's own write throttle:
    above 60 percent dirty, every new transaction is delayed
    (`dmu_tx_delay`, `module/zfs/dmu_tx.c`).
    Lowering them trades latency for throughput and is not a durability
    question, but nothing here shows it would help this workload.
  - `zfs_free_min_time_ms` 500 and `zfs_async_block_max_blocks` unlimited
    (`module/zfs/dsl_scan.c:198,209`): the background free pacer.
    Unlimited means no artificial block count cap.
  - vdev queue depths `zfs_vdev_sync_write_max_active=10`,
    `zfs_vdev_async_write_max_active=10`, `zfs_vdev_max_active=1000`
    (`module/zfs/vdev_queue.c:125,148,152`).
    One device is the bottleneck; deeper queues only lengthen it.
  - ARC size: no `zfs_arc_max` was set.
    The measured default was `c_max` = 7 242 817 536 bytes of 8 316 559 360
    bytes of memory (`/proc/spl/kstat/zfs/arcstats`),
    which is what an install gets.
    Both arms had the whole 8 GiB while their round ran;
    ZFS gave most of it to ARC and btrfs gave most of it to the page cache.

### What both were given that neither defaults to

- Equal memory, no ARC limit, no page cache limit.
- Equal compression level.
- Equal encryption presence.
- Equal fault injection at the device: both disks are throttled by the same
  mechanism to the same profile, or neither is.
- A fresh filesystem for each phase, so no round inherits fragmentation from
  an earlier round's deletes.

## Throttling the subject disks to the 860's profile

The repository's volume measures 99 percent busy at about 6 200 operations a
second, about 3 ms each, mean queue 20.6 of 32
(`doc/planning/filesystem-for-the-repository-volume.md`, section "The drive").
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
with twenty writers keeping the queue full, the average operation still waits
about queue depth divided by 6 200 per second.
That is an approximation and is listed again under what these runs cannot tell.

The unlimited phase resets the cap:

```bash
virsh blkdeviotune mvm-btrfs-zfs-best vdb --total_iops_sec 0 --live
virsh blkdeviotune mvm-btrfs-zfs-best vdc --total_iops_sec 0 --live
```

## How to repeat it

The harness is scratch, not repository code, and lives at
`/var/home/user/nvme-agent/btrfs-zfs-best/`:

- `build/guest-in-container.sh` builds the CachyOS root filesystem,
  its kernel, initramfs and grub into a raw image, inside a privileged
  container on an Arch base, with no mount of the host's own filesystems.
- `harness/make-payload.py` builds the source tree from this repository's
  tracked files and the payload disk.
- `harness/harness.py` is the in-guest campaign driver
  (`setup`, `round`, `phase`, `state`, `teardown`).
- `harness/host-probe.py` is the host control probe.
- `host-driver.sh` creates the VM, attaches the disks, sets the throttle,
  pushes the harness and runs commands.

Raw per-round JSON, including every probe duration and every sampler tally,
is under `/var/home/user/nvme-agent/btrfs-zfs-best/results/` in the guest and
is not committed.
