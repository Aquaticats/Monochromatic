# OpenZFS master 58d73c90 can smooth one-device latency but cannot hide sustained or synchronous stalls

## Symptom

A source-only migration score placed CachyOS with ZFS and ZFSBootMenu behind three Btrfs finalists.
That appears inconsistent with OpenZFS’s reputation for making storage performance more consistent through RAM caching,
transaction groups,
and its own I/O scheduler.

The first diagnosis misread “mask bad drives” as fault tolerance.
The concern is instead whether ZFS can keep applications responsive while the dedicated 2 TB DRAM-less NVMe has slow
successful I/O.

OpenZFS does have stronger explicit latency-smoothing mechanisms than the audit score made visible.
Their effect is bounded by cache residency,
write durability,
dirty-data capacity,
and whether a request has already reached the physical device.

## Root cause

### ARC hits avoid physical reads

OpenZFS commit `58d73c90dcdd77cdd07fc0b53d12d1b7339d2fe7` implements reads through the Adaptive Replacement Cache.
`module/zfs/arc.c:5876-5894` states:

```c
/*
 * "Read" the block at the specified DVA (in bp) via the
 * cache.  If the block is found in the cache, invoke the provided
 * callback immediately and return.  Note that the `zio' parameter
 * in the callback will be NULL in this case, since no IO was
 * required.  If the block is not in the cache pass the read request
 * on to the spa with a substitute callback function, so that the
 * requested block will be added to the cache.
 */
```

The cache-hit branch at `module/zfs/arc.c:5947-6079` increments `arcstat_hits`,
sets `ARC_FLAG_CACHED`,
and returns without issuing device I/O.
This can completely hide storage latency for a resident block.
A miss still reaches the NVMe.

Current OpenZFS documentation describes ARC as balancing recently and frequently used blocks and says it can outperform
a simple LRU cache:
<https://openzfs.github.io/openzfs-docs/Basic%20Concepts/Pool%20Structure/Caching.html>.

Linux filesystems are not uncached by comparison.
The Linux VFS page cache also keeps file data in RAM and writes dirty pages back later:
<https://www.kernel.org/doc/html/latest/filesystems/vfs.html>.
ZFS’s differentiator is its ARC policy and integration with ZFS metadata and block semantics,
not the mere existence of a RAM cache.
This investigation did not trace current Btrfs transaction and writeback source to equal depth and ran no comparative
workload benchmark.
It therefore establishes ZFS mechanisms and limits,
not a measured performance margin over Btrfs.

### ZIO scheduling protects queued foreground work

OpenZFS has a filesystem-aware scheduler before I/O reaches the block layer.
`module/zfs/vdev_queue.c:31-55` prioritizes classes in this order:

```c
/*
 * I/O scheduler divides operations into five I/O classes
 * prioritized in the following order: sync read, sync write, async read,
 * async write, and scrub/resilver.  Each queue defines the minimum and
 * maximum number of concurrent operations that may be issued to the device.
 */
```

`module/zfs/vdev_queue.c:64-76` explains that async-write concurrency changes with dirty-data volume
to reduce burstiness and stabilize other queues,
particularly synchronous I/O.
The current architecture guide documents the same policy and says non-interactive scrub,
resilver,
removal,
initialization,
and rebuild work stays at minimum concurrency until the vdev is idle:
<https://openzfs.github.io/openzfs-docs/Performance%20and%20Tuning/ZIO%20Scheduler.html>.

This is a real advantage over relying only on process-level I/O priority for filesystem maintenance.
It can keep queued foreground reads and writes ahead of queued background work.
It cannot preempt an operation already submitted to a device that has stopped completing requests.

### Transaction groups absorb bursts and then apply backpressure

OpenZFS accumulates asynchronous changes as dirty data and writes them through transaction groups.
For this 64 GB host,
the source defaults allow at most 4 GiB of dirty data per pool:
10 percent of physical memory,
capped at 4 GiB.
`module/zfs/arc.c:8163-8185` contains:

```c
/*
 * Otherwise, use a percentage of physical memory defined by
 * zfs_dirty_data_max_percent (default 10%) with a cap at
 * zfs_dirty_data_max_max (default 4G or 25% of physical memory).
 */
#ifdef __LP64__
if (zfs_dirty_data_max_max == 0)
    zfs_dirty_data_max_max = MIN(4ULL * 1024 * 1024 * 1024,
        allmem * zfs_dirty_data_max_max_percent / 100);
```

That memory can absorb and combine a short asynchronous burst.
The default transaction-group target is 5 seconds at `module/zfs/txg.c:106`.

The buffer is deliberately finite.
`module/zfs/dsl_pool.c:57-75` says new writes halt after `zfs_dirty_data_max` until dirty space falls.
`module/zfs/dmu_tx.c:964-1008` progressively delays transactions after dirty data crosses the configured threshold.
The current transaction-delay guide says the throttle activates when backend storage cannot accommodate incoming writes:
<https://openzfs.github.io/openzfs-docs/Performance%20and%20Tuning/ZFS%20Transaction%20Delay.html>.

ZFS can therefore smooth a burst,
but sustained writes eventually run at device speed and experience backpressure.
A synchronous `fsync()` or `O_SYNC` request still needs stable storage.
A separate SLOG helps only synchronous writes and must itself be a lower-latency,
power-loss-protected device;
it is not an async write cache:
<https://openzfs.github.io/openzfs-docs/Basic%20Concepts/Pool%20Structure/Caching.html>.

### Slow-I/O detection reports rather than accelerates

OpenZFS records an operation after it exceeds the default 30-second threshold.
`module/zfs/zio.c:5673-5697` contains:

```c
if (zio->io_delay >= MSEC2NSEC(zio_slow_io_ms)) {
    if (zio->io_vd != NULL && !vdev_is_dead(zio->io_vd)) {
        ...
        zio->io_vd->vdev_stat.vs_slow_ios++;
```

`zpool status -s` exposes that evidence,
and vdev properties can enable diagnosis thresholds.
The source says `slow_io_n` and `slow_io_t` have no built-in defaults,
so active slow-device fault diagnosis requires configuration.
See <https://openzfs.github.io/openzfs-docs/man/master/7/vdevprops.7.html>.

Counting a slow operation does not shorten it.
On the planned single-device root,
there is no alternate leaf from which to serve a cold read.

### The migration score mixed filesystem behavior with deployment risk

The ZFS candidate’s low total did not mean OpenZFS had weak caching or I/O control.
Its informal ratings gave ZFS 3 for storage-pressure control,
3 for rollback,
and 3 for encryption.
The large deductions were instead:

- installer integration 1.5 because the CachyOS path is third-party;
- operational burden 1.5;
- rolling kernel and filesystem robustness 2 because OpenZFS is out-of-tree;
- migration and exit cost 2.

The rubric also failed to name latency masking as a separate criterion.
It combined qgroup and snapshot-maintenance exposure,
filesystem robustness,
and foreground I/O isolation across broader criteria.
That made the ARC and ZIO scheduler advantage difficult to see.

## Verification

### Versions and environment

- OpenZFS source:
  commit `58d73c90dcdd77cdd07fc0b53d12d1b7339d2fe7`,
  cloned at `/var/home/user/temp/agent/openzfs-zfs-20260829`.
- Third-party CachyOS ZFS installer:
  commit `9d587de2d34a35ea33094735002d8599afed7eac`,
  cloned at `/var/home/user/temp/agent/cachyos-zfs-installer-20260829`.
- Host:
  Fedora 44 derivative,
  x86-64,
  kernel `7.2.0-ogc6.1.fc44.x86_64`,
  64 GB RAM.
- Runtime limit:
  the host has no `zpool` or `zfs` executable and `modinfo zfs` reports `Module zfs not found`.
  No ZFS latency benchmark was run.

### Source-behavior harness

The following read-only harness checks the decisive source boundaries:

```python
from pathlib import Path

root = Path("/var/home/user/temp/agent/openzfs-zfs-20260829")
arc = (root / "module/zfs/arc.c").read_text()
dsl = (root / "module/zfs/dsl_pool.c").read_text()
dmu = (root / "module/zfs/dmu_tx.c").read_text()
queue = (root / "module/zfs/vdev_queue.c").read_text()
txg = (root / "module/zfs/txg.c").read_text()
zio = (root / "module/zfs/zio.c").read_text()

checks = {
    "ARC hit can avoid I/O": "since no IO was\n * required" in arc,
    "sync I/O precedes async and scrub":
        "sync read, sync write, async read,\n * async write, and scrub/resilver" in queue,
    "dirty data is capped":
        "default 4G or 25% of physical memory" in arc,
    "writes halt at dirty-data limit":
        "new writes are halted until space frees up" in dsl,
    "dirty data causes transaction delay": "dmu_tx_delay" in dmu,
    "transaction target is five seconds": "zfs_txg_timeout = 5" in txg,
    "slow-I/O threshold is 30 seconds":
        "static uint_t zio_slow_io_ms = (30 * MILLISEC);" in zio,
}

for name, passed in checks.items():
    print(f'{name}: {"CONFIRMED" if passed else "NOT_FOUND"}')

if not all(checks.values()):
    raise SystemExit(1)
```

The source inspection confirms the mechanisms.
It does not measure their effect on the user’s NVMe or workload.

### Behavior catalog

Performance that ZFS can mask or smooth:

- repeated reads whose blocks remain in ARC;
- short asynchronous write bursts that fit inside dirty-data capacity;
- queued foreground I/O competing with queued scrub,
  resilver,
  removal,
  initialization,
  or rebuild work;
- some physical I/O through compression and request aggregation.

Performance that ZFS cannot hide indefinitely:

- cold reads that miss ARC;
- synchronous writes waiting for stable storage;
- sustained write rates above device throughput;
- an already-issued command that the only device completes slowly;
- complete NVMe queue or controller stalls;
- memory pressure that evicts the needed ARC working set.

## Verified workarounds

No performance workaround is verified on this host because no ZFS runtime was installed
and no candidate was benchmarked.
The following are source-supported controls rather than measured recommendations.

### Preserve default ARC adaptation first

ARC can use available RAM and release memory through the Linux shrinker.
Monitoring `zarcstat` and ARC kstats can establish hit rate before changing `zfs_arc_max`.
The tradeoff of a larger ARC target is less memory immediately available to applications and other kernel caches.

### Let the ZIO scheduler isolate foreground queues

Default queue classes already prioritize synchronous reads and writes over async writes and background scans.
Changing queue concurrency can trade throughput for latency;
without a workload-specific benchmark,
a lower maximum is not automatically better.

### Use dirty-data limits as a finite shock absorber

The current 64 GB host reaches the default 4 GiB dirty-data cap.
Lowering the cap applies backpressure sooner;
raising it can absorb a longer burst but permits a larger eventual flush and consumes more RAM.
Neither change increases sustained device throughput.

### Add a SLOG only for measured synchronous-write latency

A dedicated power-loss-protected SLOG can reduce latency for sync-heavy workloads.
It does nothing for asynchronous writes,
requires another suitable device,
and does not address cold reads or general NVMe stalls.

## What does not work

- Treating ARC capacity as guaranteed responsiveness for cache misses.
- Treating the 4 GiB dirty-data allowance as an unlimited write cache.
- Setting `sync=disabled` to hide latency;
  it violates application durability requests.
- Adding L2ARC to solve writes;
  L2ARC is read-only and costs RAM for headers.
- Treating `zpool status -s` evidence as latency remediation.
- Assuming the ZIO scheduler can cancel or preempt a command already executing in the NVMe controller.
- Inferring that ZFS prevents the Bazzite stalls;
  no initiating layer or incident-time physical latency was isolated.

## Upstream filing decision

No `.out-of-scope/` entry covers OpenZFS or this behavior.
Tracker searches found performance reports and slow-device work,
but no upstream defect matching the difference between bounded smoothing and guaranteed masking.
The current documentation explains ARC,
transaction delay,
and ZIO scheduling directly.

1.  **Is it upstream’s fault?**
    No.
    The capability is bounded by cache hits,
durability semantics,
and backend throughput as documented.
2.  **Can upstream fix it?**
    Not as a general guarantee.
    Software cannot make every cold or durable operation complete independently of its only storage device.
3.  **Are they supporting this use case?**
    Yes.
    OpenZFS documents ARC,
write throttling,
and foreground/background I/O scheduling as supported mechanisms.
4.  **Would the repository welcome a contribution?**
    Not applicable because no defect or documentation omission was established.
    The repository contains bug and feature templates and no AI-specific prohibition was found.
5.  **Will they likely fix it?**
    Not applicable because no fixable defect was identified.
6.  **Has a minimal fix been prototyped?**
    No.
    The investigation found an evaluation-rubric omission rather than an OpenZFS implementation bug.

Nothing should be filed or posted upstream.
