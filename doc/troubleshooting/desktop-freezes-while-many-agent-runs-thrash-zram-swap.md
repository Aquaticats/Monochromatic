# CachyOS on kernel 7.3.5: many agent runs push memory into compressed swap and the desktop freezes for minutes

Status:
the two kernel settings described under "Workarounds applied at runtime and measured once without verification"
are in force,
and the owner has chosen to keep them for now.
Their effect is **unverified**:
one later reading under a lighter load is recorded here,
and the comparison that would show whether they prevent a repeat has not been made.
Nothing in this document is a verified workaround,
and nothing here is approved or final;
the project's owner declares the work complete.

Every figure in this document was measured on the machine on 2026-11-22 by the coordinating agent session.
That session recorded each one in its findings file,
and the later reading in its own counters file.
The few figures this document adds are each named with the read-only command that produced them,
and a figure derived from others is marked arithmetic.
Times are local,
UTC-05:00,
at the precision the measurement recorded.
Most carry seconds;
the atop samples and the 21:01 `/proc/vmstat` read carry minutes.

## Symptom

The owner,
at about 20:57:
"My system was just stuck for like 5 minutes.
Might be a us problem."
The desktop stopped answering for about five minutes and then came back.
Nothing crashed,
nothing was killed,
and no line in the `dmesg` reads named a failing program.
While it hung,
one coordinating session had this running,
all of it started by that session or its agents:
6 guest machines,
an uncounted number of whole container test gates of `package/git-policy/cli`
(the session recollected 3 to 4,
and the later reading records that the number was not counted),
several Rust builds,
7 Pi rounds and 9 Sonnet passes.

The person meets a machine that ignores input.
The machine shows the following.

### The pressure files

`/proc/pressure/memory`,
read at 2026-11-22T20:56:41-05:00:

```text
some avg10=5.28 avg60=33.46 avg300=47.05
full avg10=4.79 avg60=32.21 avg300=45.76
```

The kernel documentation defines the `full` line as
[the share of time in which all non-idle tasks are stalled on a given resource simultaneously][psi-full],
and calls time spent there thrashing.
It also calls the percentages
[recent trends over ten, sixty, and three hundred second windows][psi-windows],
with the exact stalled time exported in microseconds as `total=`.
So `full avg300=45.76` is a 300-second trend ending at that read,
not an exact fraction of the freeze,
and no pair of `total=` readings brackets the freeze,
so the exact stalled time for it cannot be recovered from what was recorded.

The same second's I/O and CPU files read:

```text
/proc/pressure/io:  some avg300=48.45 full avg300=3.09
/proc/pressure/cpu: some avg300=22.35
```

Those numbers separate the resources only in `full`:
I/O's `full avg300` is 3.09 against memory's 45.76,
while I/O's `some avg300` 48.45 is slightly above memory's 47.05.
CPU `full` is defined as zero at system level,
so the CPU comparison is `some` only.
What the three files support is that the all-task stall was on memory.

`uptime` at the same second reported a load average of 156.76 over one minute,
208.56 over five minutes,
and 126.46 over fifteen.
`free --mebi` at the same second reported memory used 50343,
free 10109,
shared 14014,
available 12809,
and swap used 35687 of 63151.

### The activity log

`atopsar -r /var/log/atop/atop_20261122 -m` and `-s`,
ten-minute samples:

- 2026-11-22T20:50:00-05:00:
  memfree 1966M,
  cached 20899M,
  slab 4643M,
  swap free 21774M of 63151M,
  pagescan 65984/s,
  swapin 28182/s,
  swapout 35512/s,
  and commit space 141908M against a limit of 94728M.
- 2026-11-22T21:00:00-05:00:
  memfree 5108M,
  cached 11434M,
  swap free 24382M,
  pagescan 4681429/s,
  swapin 23063/s,
  swapout 29510/s,
  and commit space 124016M.

The 21:00 sample's page scan rate is in the millions per second,
while the 20:50 sample's is in the tens of thousands.

### The kernel's own "blocked for more than" lines

`journalctl --dmesg`,
at 2026-11-22T20:55:27-05:00:

```text
INFO: task worker:2986026 blocked for more than 123 seconds.
INFO: task worker:301926 blocked for more than 123 seconds.
```

Both are threads of `qemu-system-x86` (tgid 847644 and 3112795),
both in `fdatasync`.
From there the stack runs through `zpl_fsync` and `zfs_fsync` to `zil_commit_flags`,
then `zil_commit_impl`,
and waits in `cv_wait_common`.
So two guests' disk flushes waited over two minutes inside ZFS's intent log.
`systemd-journald` logged the same second:

```text
Under memory pressure, flushing caches.
```

### The swap device

Swap is zram only.
At 2026-11-22T20:56:41-05:00 `free --mebi` reported 35687 MiB of 63151 in use,
which is 57% of the swap (arithmetic).
At 2026-11-22T21:01-05:00 `zramctl --output-all` showed `/dev/zram0` alone:
zstd,
priority 100,
disk size 61.7G,
DATA 37.2G compressed to COMPR 14G,
TOTAL 14.3G,
MEM-USED peak 17.8G,
and a ratio of 2.5979,
which is 60% of the device in DATA (arithmetic).
At 2026-11-22T21:47:55-05:00 it held DATA 41.2G in TOTAL 13.7G,
which is 67% (arithmetic).
The device never filled.

### The signature, in one place

- `/proc/pressure/memory` with `full` tracking `some` closely over minutes:
  every task stalled,
  not a few.
- `journalctl --dmesg` "blocked for more than N seconds" lines for tasks that are not the desktop's own.
- `zramctl` and `/proc/swaps` showing a zram-only swap device carrying most of the machine's swapped data,
  with the machine's own programs' pages among the swapped.

## Root cause

The cause has two layers,
kept apart.
First,
what the running jobs asked of memory at that moment,
and what let the sum fit at all.
Second,
why the shortage turned into a frozen desktop instead of merely slow jobs.

### What the running jobs asked of memory

From atop's per-process records summed by program name at 2026-11-22T20:50:00-05:00,
by the coordinating session's script `sum-memory-by-program-from-the-activity-log.ts`:

- All processes together:
  resident 38383 MiB,
  swapped 14574 MiB.
- `qemu-system-x86`,
  6 guests:
  resident 19888 MiB,
  swapped 147 MiB.
- `rustc`,
  132 processes recorded in the interval:
  resident 4772 MiB,
  swapped 2157 MiB.
- A build script:
  resident 2912 MiB,
  swapped 0 MiB.
- `helium`,
  the browser,
  41 processes:
  resident 2776 MiB,
  swapped 3596 MiB.
- `node-MainThread`:
  resident 1516 MiB,
  swapped 1222 MiB.
- `claude`:
  resident 1351 MiB,
  swapped 625 MiB.
- `pi`,
  6 processes:
  resident 1061 MiB,
  swapped 518 MiB.
- `git`,
  3788 processes recorded:
  resident 803 MiB,
  swapped 0 MiB.
- `mise`:
  631 MiB resident,
  82 MiB swapped.
- `cargo`:
  226 MiB resident,
  43 MiB swapped.
- `plasmashell`:
  166 MiB resident,
  311 MiB swapped.

Swap in use system-wide was 41377 MiB at that 20:50 sample,
of which those processes account for 14574 MiB.
The other 26803 MiB is in no process's own swap figure;
the findings attribute it to swapped shared memory,
guest memory and memory-backed file systems among them,
including the test gate's fixture file system,
and the split between those was not measured.

Summed,
the readings at 20:50 and 21:01 put physical use near the machine's 61.7 GiB:
resident process memory 38383 MiB,
which is 37.5 GiB,
and a sum of per-process figures,
so a shared page counts once per process that maps it;
kernel slab 4643M,
which is 4.5 GiB;
and zram's TOTAL 14.3G at 21:01 with MEM-USED peak 17.8G.
Taken with TOTAL the sum is about 55 GiB,
and taken with the peak about 59 GiB (arithmetic).
40.4 GiB was swapped at that moment (41377 MiB).

Commit space exceeded its limit at both atop samples:
141908M against a limit of 94728M at 20:50,
and 124016M at 21:00.
The machine runs `vm.overcommit_memory = 0` (`sysctl`,
read on 2026-11-23),
the default mode in which the kernel rejects only obvious overcommits,
so those samples are reported here and are not used as a shortage.

The numbers support one measured statement and one inferred one.

- Measured:
  40.4 GiB of swapped memory occupied 14.3G of zram TOTAL at 21:01,
  a ratio of 2.5979.
  Inferred:
  without compression,
  that data could not have stayed resident beside 37.5 GiB of resident memory and 4.5 GiB of slab on 61.7 GiB,
  so compressed swap is what made the workload fit.
- Measured:
  the browser had 3596 MiB swapped,
  `plasmashell` 311 MiB,
  and `opensnitch-ui` 985 MiB.
  Plausible and unmeasured:
  a person whose window manager's pages are being reclaimed sees a frozen desktop,
  because the programs that would draw a busy indicator need those pages back.

### Why the desktop froze instead of running slowly

`kswapd` is the kernel's background reclaim thread.
It frees memory ahead of allocations so that an allocating task rarely has to reclaim for itself,
and its headroom is set by `vm.watermark_scale_factor`.
The kernel's own documentation defines the unit and the default in the
[`watermark_scale_factor` section][vm-watermark]:

```text
The unit is in fractions of 10,000. The default value of 10 means the distances between watermarks are 0.1% of
the available memory in the node/system. The maximum value is 3000, or 30% of memory.
```

The same section names the signal this machine showed and the knob that answers it,
[under `watermark_scale_factor`][vm-allocstall]:

```text
A high rate of threads entering direct reclaim (allocstall) or kswapd going to sleep prematurely
(kswapd_low_wmark_hit_quickly) can indicate that the number of free pages kswapd maintains for latency reasons is
too small for the allocation bursts occurring in the system. This knob can then be used to tune kswapd
aggressiveness accordingly.
```

The kernel source shows the two paths those counters count.
When an allocation cannot be served from the free lists,
the slow path calls `__alloc_pages_direct_reclaim`,
which marks the task as stalled for PSI and then reclaims synchronously
([`mm/page_alloc.c` 4470 to 4482 at v7.3.5][page-alloc-direct]):

```c
// mm/page_alloc.c:4470-4482, linux v7.3.5
/* The really slow allocator path where we enter direct reclaim */
static inline struct page *
__alloc_pages_direct_reclaim(gfp_t gfp_mask, unsigned int order,
		unsigned int alloc_flags, const struct alloc_context *ac,
		unsigned long *did_some_progress)
{
	struct page *page = NULL;
	unsigned long pflags;
	bool drained = false;
	int reclaim_order = ac->reclaim_compact_order;

	psi_memstall_enter(&pflags);
	*did_some_progress = __perform_reclaim(gfp_mask, reclaim_order, ac);
```

Before that,
the same slow path asks the background thread to catch up with `wakeup_kswapd`
([`mm/page_alloc.c` 4520 to 4528][page-alloc-wakeup]):

```c
// mm/page_alloc.c:4520-4528, linux v7.3.5
	for_each_zone_zonelist_nodemask(zone, z, ac->zonelist, highest_zoneidx,
					ac->nodemask) {
		if (!managed_zone(zone))
			continue;
		if (last_pgdat == zone->zone_pgdat)
			continue;
		wakeup_kswapd(zone, gfp_mask, reclaim_order, highest_zoneidx);
		last_pgdat = zone->zone_pgdat;
	}
```

The synchronous reclaim those allocations enter is `try_to_free_pages`
([`mm/vmscan.c` 6779 to 6793][vmscan-ttfp]):

```c
// mm/vmscan.c:6779-6793, linux v7.3.5
unsigned long try_to_free_pages(struct zonelist *zonelist, int order,
				gfp_t gfp_mask, const nodemask_t *nodemask)
{
	unsigned long nr_reclaimed;
	struct scan_control sc = {
		.nr_to_reclaim = SWAP_CLUSTER_MAX,
		.gfp_mask = current_gfp_context(gfp_mask),
		.reclaim_idx = gfp_zone(gfp_mask),
		.order = order,
		.nodemask = nodemask,
		.priority = DEF_PRIORITY,
		.may_writepage = 1,
		.may_unmap = 1,
		.may_swap = 1,
	};
```

The watermarks those paths compare against were not read before the change.
At 2026-11-22T21:47:55-05:00,
with the factor at 125,
`/proc/zoneinfo` gave min 66 MiB,
low 855 MiB and high 1,645 MiB,
so kswapd wakes under about 855 MiB free and reclaims until about 1,645 MiB is free.
At the factor 10 the same rule gives a step of about 63 MiB,
so low about 129 MiB and high about 192 MiB,
which the reading file marks computed from the kernel's rule.
Whether about 129 MiB of low watermark is enough for this workload's allocation bursts is this document's inference;
it was not measured.

The counters recorded at 2026-11-22T21:01-05:00 are consistent with that reading,
and they are cumulative from boot (3 days 9 hours),
so they cannot show what happened during the five minutes:

- `pgscan_direct 7252497503` against `pgscan_kswapd 3998833981`:
  over the boot,
  allocating tasks scanned more pages themselves than the background thread did.
- `pgsteal_direct 2111178042` against `pgsteal_kswapd 2640168318`.
- `allocstall_normal 67950` and `allocstall_movable 49656`:
  allocation attempts that had to enter direct reclaim.
- `kswapd_low_wmark_hit_quickly 862758` against `kswapd_high_wmark_hit_quickly 96`:
  the background thread woke and went back to sleep almost every time without building a cushion,
  which is the second half of the documented signal.
- `compact_stall 91031` with `compact_fail 55725`,
  and `thp_fault_alloc 11278918` against `thp_fault_fallback 11369066`.
- `pswpin 2496422265`,
  `pswpout 3331029563`,
  and `oom_kill 14`.

That chain is a supported hypothesis rather than a shown cause:
it says direct reclaim dominated this boot,
not that every allocating task reclaimed during the freeze.
The nearest windowed evidence is the atop sample at 2026-11-22T21:00:00-05:00,
whose ten minutes contain the stall and whose page scan rate is 4681429/s.
What would show the mechanism during the incident is a delta of the same counters across it,
or a trace of direct reclaim while it runs,
and neither was taken.

The mechanism the documentation describes,
and this document infers,
is that when the background thread keeps too small a cushion,
allocations take the synchronous path,
and each such task's progress waits on page scans and on zram's compress and decompress path.
PSI's `full` line measures the resulting all-task stall directly,
and the slow path marks the stall at `psi_memstall_enter`,
but the readings taken do not tie a particular allocation to a particular stall.

### Nothing shed load

The multi-gen LRU's thrash valve was nearly shut.
`/sys/kernel/mm/lru_gen/min_ttl_ms` read 100.
No file under `/usr/lib` or `/etc` sets that value:
searching the `tmpfiles.d`,
`udev` and `systemd` files for `min_ttl` on 2026-11-22 finds nothing.
So the value the kernel boots with is the documented default of 0,
and 100 was already a runtime departure of unknown origin.
The kernel documentation's [thrashing-prevention section][lru-thrash] says what the number protects:

```text
Users can write N to min_ttl_ms to prevent the working set of N milliseconds from getting evicted. The OOM killer
is triggered if this working set cannot be kept in memory. In other words, this option works as an adjustable
pressure relief valve, and when open, it terminates applications that are hopefully not being used.

Based on the average human detectable lag (~100ms), N=1000 usually eliminates intolerable janks due to
thrashing. Larger values like N=3000 make janks less noticeable at the risk of premature OOM kills.

The default value 0 means disabled.
```

At 100 milliseconds the valve protected a tenth of a second of working set,
which is nothing against a five-minute stall,
and the associated kill never came:
`oom_kill` read 14 at 21:01 and 14 again at 21:07:19.

`systemd-oomd` never matched its thresholds on the samples taken.
Its `oomctl` output records a swap used limit of 90.00%,
memory pressure monitored only for `/system.slice` (80%,
30 s),
and every user slice monitored for swap use only.
Swap use never neared 90% (35687 of 63151 MiB at 2026-11-22T20:56:41-05:00),
and memory pressure was monitored for the system slice rather than the user's,
so no oomd action was selected on those samples.

The findings also record two guests' flushes waiting over two minutes inside ZFS's intent log,
the `dmesg` lines quoted in "Symptom".
Whether memory reclaim or the disk made ZFS that slow is not established,
so this document does not use those waits as cause.
The ZFS ARC was far below its ceiling at 21:00:
`size` 6123719120 against `c_max` 65146339328 and `c_min` 2069377536.
Its peak and its reclaim behaviour were not measured,
so that reading rules out a full cache rather than a cache contribution.

### The owner's position, and the measurements beside it

The owner,
in the session's question tool:
"I am confident we have enough memory.
We have 64G of physical RAM.
There's no way that isn't enough.
We might have to change our swap or zram or whatever conf."

Beside that position the same session's measurements put these:

- Physical RAM really is 64G:
  `/proc/meminfo` reads `MemTotal: 64668048 kB`.
- Resident process memory at 2026-11-22T20:50:00-05:00 was 38383 MiB (37.5 GiB) and kernel slab 4643M (4.5 GiB),
  and 40.4 GiB was swapped out and held compressed in 14.3G of zram TOTAL.
- Commit space exceeded its limit at both atop samples (141908M against 94728M at 20:50,
  124016M at 21:00),
  in the default overcommit mode.
- Compressed swap is what carried the difference (inferred),
  so the owner's second sentence,
  "We might have to change our swap or zram or whatever conf",
  describes what was then changed:
  the two kernel settings in "Workarounds applied at runtime and measured once without verification".

## Verification

### Versions under test

- Kernel `7.3.5-1-cachyos` (`uname --kernel-release`,
  read on 2026-11-22).
- `systemd 262.2-1` and `zram-generator 1.2.1-1.1` (`pacman --query systemd zram-generator`).
- `cachyos-settings 1:1.4.5-1`,
  built 2026-11-20 (`pacman --query --info cachyos-settings`).
- Swap is zram only:
  `/proc/swaps` lists `/dev/zram0` alone,
  and `zramctl` reports zstd on a 61.7G device.

### The reads

```sh
# doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md
cat /proc/pressure/memory /proc/pressure/io /proc/pressure/cpu
cat /proc/vmstat
cat /proc/zoneinfo
free --mebi
uptime
zramctl --output-all
oomctl
journalctl --dmesg | rg 'blocked for more than'
atopsar -r /var/log/atop/atop_20261122 -m
atopsar -r /var/log/atop/atop_20261122 -s
sysctl vm.watermark_scale_factor vm.watermark_boost_factor vm.overcommit_memory
cat /sys/kernel/mm/lru_gen/min_ttl_ms
```

### Catalog: what the stall looked like

There is no clean catalog in this round:
no reading was taken with the machine at rest under a workload comparable to the freeze.
The catalogs here are the stall,
a baseline taken after it,
and one later reading under a lighter load.
The stall's discriminating values are:

- memory pressure `some avg300=47.05` and `full avg300=45.76` at 20:56:41,
  as 300-second trends;
- page scan 4681429/s and commit space 124016M at 21:00:00;
- `pgscan_direct 7252497503` above `pgscan_kswapd 3998833981`,
  with `kswapd_low_wmark_hit_quickly` 862758 against `kswapd_high_wmark_hit_quickly` 96,
  cumulative since boot at 21:01;
- two "blocked for more than 123 seconds" lines at 20:55:27;
- zram holding DATA 37.2G in TOTAL 14.3G at 21:01,
  60% of a 61.7G device (arithmetic),
  with no disk swap configured.

### Catalog: the baseline snapshot, taken at 2026-11-22T21:07:19-05:00

Recorded immediately before the two settings were applied,
so the next comparison subtracts it:

```text
pgsteal_kswapd 2652454685
pgsteal_direct 2115564415
pgscan_kswapd 4016504058
pgscan_direct 7259376568
pswpin 2509028963
pswpout 3344818944
allocstall_normal 68002
allocstall_movable 49708
pgmajfault 3330732813
kswapd_low_wmark_hit_quickly 865287
kswapd_high_wmark_hit_quickly 96
pageoutrun 891970
oom_kill 14
compact_stall 91079
some avg10=4.20 avg60=4.51 avg300=9.20 total=5378263857
full avg10=4.18 avg60=4.33 avg300=8.82 total=5254545044
some avg10=2.82 avg60=2.10 avg300=8.06 total=6510863605
full avg10=1.20 avg60=0.64 avg300=1.03 total=4813209115
```

The last four lines are the pressure counters in the order the file records them:
`/proc/pressure/memory` (some,
then full),
then `/proc/pressure/io` (some,
then full).

This snapshot is after the stall,
so it is a floor rather than a quiet baseline:
memory `full avg300` still reads 8.82 there,
and `allocstall_normal` rose from 67950 to 68002 between 21:01 and 21:07:19.
That is why it is the right number to subtract and a poor number to call "before".

### Catalog: the reading 2,436 seconds after the settings, at 2026-11-22T21:47:55-05:00

The coordinator read the same counters again 2,436 seconds after the baseline.
Differences against the 21:07:19 snapshot,
from `counters-forty-minutes-after-the-two-settings.txt`:

- kswapd did 80.9% of the scanning in this window:
  `pgscan_kswapd` +106,248,316 and `pgscan_direct` +25,149,865,
  against 35.6% cumulative before the change;
- memory `full` pressure over the window was 3.95% of the time,
  a `total=` delta of 96,200,010 microseconds,
  and `some` was 4.12%;
- `allocstall_normal` +1,015 and `allocstall_movable` +1,628,
  `kswapd_low_wmark_hit_quickly` +16,583 and `kswapd_high_wmark_hit_quickly` +3,
  and `oom_kill` +0;
- swap traffic stayed heavy:
  `pswpin` +59,812,740 and `pswpout` +74,167,231,
  24,554 and 30,446 pages a second,
  with zram holding DATA 41.2G in TOTAL 13.7G (67% of the device,
  arithmetic) and swap at 43,589 of 63,151 MiB;
- free memory 16,241 MiB,
  load average 24.68,
  30.11 and 33.62,
  and five guests and one gate running at the reading.

The watermarks in force at factor 125,
from `/proc/zoneinfo` at that reading:
min 66 MiB,
low 855 MiB,
high 1,645 MiB.
The same rule at factor 10 gives low about 129 MiB and high about 192 MiB,
a step of about 63 MiB,
which the file marks computed.

This is not the comparison in "The comparison that would verify the change".
The load is not shown comparable:
free memory was 16,241 MiB here against 1,966 MiB at 20:50,
and one gate ran here against six guests and an uncounted number of gates at the freeze.
The two settings were applied together,
so nothing here separates them.
What it shows is that the settings were in force under heavy swap traffic without a reported stall;
what it does not show is that they prevent a repeat.

### The comparison that would verify the change

No such comparison has been made.
This is the one that would settle it.
A start read of a lighter load was taken at 2026-11-23T19:26:55-05:00,
with both settings in force:
four guests,
one whole container gate,
and Rust builds inside several Pi rounds
(`counters-at-the-start-of-the-load-of-2026-11-23-evening.txt` in
`~/temp/agent/desktop-froze-under-agent-runs-memory-stall-2026-11-22/keep/`).
Its end read was taken at 2026-11-23T19:42:03-05:00,
just after the gate ended,
with both settings still in force
(`counters-at-the-end-of-the-load-of-2026-11-23-evening.txt` beside the start read;
the rates by `compare-two-reads-as-rates.ts` in the folder above,
kept as `rates-between-the-two-reads-of-2026-11-23-evening.txt`).
Over the 908 seconds:

- kswapd did 72.4% of the scans
  (`pgscan_kswapd` 37,438 per second against `pgscan_direct` 14,249);
- `allocstall_normal` rose by 45 and `allocstall_movable` by 77;
- `kswapd_low_wmark_hit_quickly` rose by 7,109 and `kswapd_high_wmark_hit_quickly` by 1;
- `pgsteal_kswapd` ran at 24,440 per second against `pgsteal_direct` 9,472;
- memory `full` stalled 3.02% of the interval and `some` 3.17%;
- `pswpin` and `pswpout` ran at 19,862 and 27,012 per second;
- `oom_kill` did not move,
  and none of the kernel log's 8 lines in the window is a "blocked for more than" line;
  no positive control was possible,
  since the journal keeps only the current boot (`journalctl --list-boots` lists one,
  from 2026-11-23T16:38:50-05:00),
  so 2026-11-22's lines,
  which held such lines,
  are gone.

This is not the comparison that would verify the change.
The load was lighter than 2026-11-22's
(four guests,
not six),
no arm without the settings ran beside it,
and the baseline's figures are totals since that boot,
not rates over a comparable load.
It shows the settings held under this load without a stall.
It does not show they caused it.

A heavier load began at 2026-11-23T20:33:24-05:00,
when the owner allowed up to 64 rounds at once
(`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`)
and five more build rounds started beside seven,
with six Claude agents building in their own worktrees.
Both settings were in force
(`min_ttl_ms` 1000,
`watermark_scale_factor` 125).
Its start read is `counters-before-the-build-ahead-wave.txt`,
and a read 137 seconds later is `counters-after-twelve-rounds-and-six-claude-agents.txt`,
both in the folder above.
Over those 137 seconds,
with three guests up and no whole gate:

- kswapd did 98.1% of the scans
  (`pgscan_kswapd` 32,420 per second);
- `allocstall_normal` rose by 13 and `allocstall_movable` by 1,048;
- `kswapd_low_wmark_hit_quickly` rose by 446 and `kswapd_high_wmark_hit_quickly` by 0;
- `pgsteal_kswapd` ran at 29,226 per second against `pgsteal_direct` 497;
- memory `full` stalled 3.31% of the interval and `some` 3.38%;
- `pswpin` and `pswpout` ran at 4,792 and 24,662 per second;
- `oom_kill` did not move,
  and the kernel log since 20:30 holds no "blocked for more than" line.

The interval is short and the load had not reached six guests or a whole gate,
so it is not the comparison either.
The end read of this load,
taken once its guests and builds have run,
is the one still owed.

Take the same reads at the start and at the end of a load comparable to 2026-11-22's:
6 guests,
whole container gates of `package/git-policy/cli`,
Rust builds.
Express every counter as a rate over that interval,
and subtract the 21:07:19 snapshot to get the segment the change is measured on.
A reboot clears both settings,
so a comparison across a reboot compares two arms and cannot subtract counters:
both reads of one arm must have the settings in force.

If the change works,
these must move:

- the `pgscan_direct` rate falls,
  and `pgscan_kswapd` takes a larger share of the total scan (baseline `pgscan_direct` 7259376568 against
  `pgscan_kswapd` 4016504058);
- the `allocstall_normal` and `allocstall_movable` rates fall (baseline 68002 and 49708,
  both above their 21:01 values);
- the `kswapd_low_wmark_hit_quickly` rate falls,
  and its ratio to `kswapd_high_wmark_hit_quickly` narrows (baseline 865287 against 96);
- `pgsteal_direct` falls relative to `pgsteal_kswapd` (baseline 2115564415 against 2652454685);
- memory `full` stays far below the 45.76 trend of the stall,
  and no "blocked for more than N seconds" line appears for a guest thread.

These would not count as success:

- a lower pressure figure bought by running fewer jobs;
- a rise in `pswpin` and `pswpout`,
  which is expected because moving pages to zram faster is what the change is for;
- a rise in `oom_kill`,
  because a one-second `min_ttl_ms` valve may kill an application to shed load,
  which is relief rather than a fix;
- one read at one moment.
  Rates need two reads,
  and a single low `avg10` after a load has ended says nothing.

## Workarounds applied at runtime and measured once without verification

Nothing in this document is a verified workaround.
The two settings were applied at runtime on 2026-11-22.
The machine's restarts on 2026-11-23 returned them to their boot values
(the last boot began at 2026-11-23T16:38:50-05:00,
by `journalctl --list-boots`):
at 2026-11-23T19:2x-05:00 `sysctl vm.watermark_scale_factor` read 10 and `cat /sys/kernel/mm/lru_gen/min_ttl_ms` read 100.
The coordinating session applied both again with the commands under each heading,
since the owner's answer keeps both in force until a load like 2026-11-22's has been seen,
and read them back at 2026-11-23T19:26:27-05:00:
125 and 1000.
They have been measured once,
under a lighter load,
and the comparison that would show their effect does not exist.
Both were changed together,
so even a comparable-load success would not separate their effects.
Both are runtime values:
a reboot returns them to the stock values,
so anything that survives a reboot needs the draft under "The persistent form as a draft for the owner".

### `vm.watermark_scale_factor`, from 10 to 125

With the owner's decision,
after the baseline snapshot at 2026-11-22T21:07:19-05:00,
the coordinating session ran as root:

```sh
# doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md
sudo --non-interactive sysctl --write vm.watermark_scale_factor=125
```

Read back at 2026-11-22T21:07:36-05:00 with `sysctl vm.watermark_scale_factor`:
`125`.
Revert with the same command and `=10`.

Tradeoff:
this buys kswapd's headroom with memory that the jobs cannot use,
and the value chooses how much.
The documented unit makes 125 equal to 1.25% of memory against the stock 0.1%,
and the watermarks read at 2026-11-22T21:47:55-05:00 put low at 855 MiB and high at 1,645 MiB,
a step of about 790 MiB,
against about 129 and 192 MiB at the stock factor (computed).
This distribution shipped this exact value once and removed it for its memory use:
#55's message is "This causes an unnecessarily large increase in RAM consumption",
and a maintainer reported on [pull request #138][pr-138] that it "increased the ram usage (cache) like x2",
which covers the watermark pair rather than 125 alone.
Neither is a measurement of the effect here.
The owner was told that history and chose the option labelled "keep 125 for now (Recommended)",
which the question tool numbered A,
with both settings staying as they are until a load like 2026-11-22's has been seen again,
and the decision on persisting them following that comparison.

### `/sys/kernel/mm/lru_gen/min_ttl_ms`, from 100 to 1000

At the same time,
after the same baseline snapshot:

```sh
# doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md
echo 1000 | sudo --non-interactive tee /sys/kernel/mm/lru_gen/min_ttl_ms
```

Read back at 2026-11-22T21:07:36-05:00 with `cat /sys/kernel/mm/lru_gen/min_ttl_ms`:
`1000`.
Revert with `echo 100 | sudo --non-interactive tee /sys/kernel/mm/lru_gen/min_ttl_ms`.

This is a runtime value under `/sys` rather than a sysctl key,
and a reboot starts it again from the kernel's boot value.
That boot value is 100 on this kernel,
not the documented default of 0:
it read 100 after the restarts of 2026-11-23.
What sets it is not found.
The kernel command line names no `lru_gen` parameter.
`/proc/config.gz` carries no default for it:
its only matching lines are `CONFIG_LRU_GEN=y`,
`CONFIG_LRU_GEN_ENABLED=y`
and `CONFIG_LRU_GEN_WALKS_MMU=y`.
A search for `min_ttl_ms` found no file in
`/etc`,
`/usr/lib/systemd`,
`/usr/lib/udev`,
`/usr/share`
or `/usr/bin`,
though it skipped files it could not read.
A patch in the distribution's kernel is the place not yet read.
So reverting to 100 is reverting to the boot value.
At 100 the machine's valve was a tenth of a second;
at 1000 the kernel documentation's sentence applies:
"The OOM killer is triggered if this working set cannot be kept in memory",
with "Larger values like N=3000 make janks less noticeable at the risk of premature OOM kills."

Tradeoff:
this trades a stall for a possible kill.
The protected working set is one second of activity,
so a program that cannot get memory back can now be ended instead of the whole machine thrashing.
`oom_kill` read 14 at 21:01 and 14 again at 21:07:19,
so no kill had happened up to the moment of the change;
the later reading shows `oom_kill` +0 over its window.

### The persistent form as a draft for the owner

Draft only.
Nothing here is installed,
and this document installs nothing.
A file per setting,
for the owner to review and apply after the comparison in "The comparison that would verify the change".
Both take the mode and ownership of the vendor files they mirror,
`0644 root root` (`stat`,
read on 2026-11-22).

`/etc/sysctl.d/90-vm-watermark-scale-factor.conf`:

```ini
# /etc/sysctl.d/90-vm-watermark-scale-factor.conf
# kernel reclaim headroom: 1.25% of memory instead of the stock 0.1%
vm.watermark_scale_factor = 125
```

The `sysctl.d` page defines this form:
files are read at boot by `systemd-sysctl.service`,
hold `key = value` assignments,
and the entry in the file with the
[lexicographically latest name takes precedence][sysctl-precedence],
which is why a 90-prefixed file in `/etc/` is read after the vendor's `70-cachyos-settings.conf`.
Precedence is decided per key:
the machine also carries `/usr/lib/sysctl.d/99-splitlock.conf`,
which sorts after that name but sets no watermark key,
so this assignment stands.
A `sysctl.d` file cannot carry the second setting,
because it writes `/proc/sys` keys only:
the page's own example is the assignment `kernel.domainname=foo`,
which it says will cause "foo" to be written to `/proc/sys/kernel/domainname`
([same page][sysctl-format]).

`/etc/tmpfiles.d/lru-gen-min-ttl.conf`:

```ini
# /etc/tmpfiles.d/lru-gen-min-ttl.conf
# one second of protected working set, applied once at boot
w! /sys/kernel/mm/lru_gen/min_ttl_ms - - - - 1000
```

The `tmpfiles.d` page defines the line type:
[`w` writes the argument parameter to a file, if the file exists][tmpfiles-w],
and "The argument parameter will be written without a trailing newline."
The `!` modifier makes the line boot-only.
The page's own sysfs example pairs the type with the `-` modifier,
`w- /proc/sys/vm/swappiness - - - - 10`,
which suppresses a failing line,
and a reader on a kernel without the node should use that form.
This distribution already writes a `/sys` node exactly this way,
in its `usr/lib/tmpfiles.d/thp.conf` [line 3][thp-conf],
so the pattern is one the machine's own boot already runs.
This draft waits,
by the owner's choice to keep 125 for now,
until the comparison in "The comparison that would verify the change" exists.

## What does not work

### Disk swap today

The owner asked for disk swap in the same decision that authorized the two settings:
"Apply both now,
and 2 also make some disk act as additional swap.
If you think 2 is a bad idea because of zfs,
then ask again."
The coordinator asked again,
as instructed,
and the owner chose option A,
"no disk swap for now (Recommended)".

The reasons,
with what each rests on:

- The fast drive is the pool.
  `lsblk` shows `nvme1n1` at 3.5T,
  holding `nvme1n1p1` (4G vfat,
  mounted at `/boot`) and `nvme1n1p2` (3.5T,
  `zfs_member`);
  `zpool list` shows `zpcachyos` at 3.48T size,
  read on 2026-11-22.
  There is no free partition to give to swap.
- A ZFS volume as swap is the documented deadlock risk.
  The OpenZFS FAQ's swap section warns:
  ["**CAUTION:** for now swap on zvol may lead to deadlock, in this case please send your logs
  here"][openzfs-swap],
  pointing at `openzfs/zfs` issue 7734.
- A swap file on ZFS has no support in its source.
  The kernel's swap-file path needs the `swap_activate` member of `struct address_space_operations`
  ([`include/linux/fs.h` 437 to 440 at v7.3.5][fs-aops]):

  ```c
  // include/linux/fs.h:437-440, linux v7.3.5
  	/* swapfile support */
  	int (*swap_activate)(struct swap_info_struct *sis, struct file *file,
  				sector_t *span);
  	void (*swap_deactivate)(struct file *file);
  ```

  ZFS's `zpl_address_space_operations` initializer sets a read member,
  a read-folio or readpage member,
  `.writepage`,
  `.writepages`,
  `.direct_IO`,
  `.set_page_dirty`,
  `.dirty_folio`,
  and `.migrate_folio`,
  and sets neither `.swap_activate` nor `.swap_deactivate`
  ([`module/os/linux/zfs/zpl_file.c` 1394 to 1421 at commit `a1daebfe`][zpl-aops]),
  and a search of that file for `swap` returns nothing.
- The other drives are held read-only by the owner's rule until the migration is done:
  `sda` (3.6T,
  `crypto_LUKS`) and `nvme0n1` (1.9T,
  `crypto_LUKS` partitions) are the old drives,
  kept unwiped,
  so nothing on them may be repartitioned now.
- It would not have been used that night.
  zram carries swap priority 100 and held DATA 37.2G of a 61.7G device at 2026-11-22T21:01-05:00,
  so a lower-priority disk swap would have stayed untouched while zram still had room.
  Disk swap added behind a higher-priority zram changes nothing until zram fills.

### Limiting the concurrent runs

The project rejected limiting how many agent runs go at once,
and treats the freeze as a matter of swap configuration.
The owner's words:
"I am confident we have enough memory.
We have 64G of physical RAM.
There's no way that isn't enough.
We might have to change our swap or zram or whatever conf."
No limit is in force.
This is a rejected approach rather than a measured failure:
the record holds no measurement of a lower job count,
and a lower pressure figure bought by running fewer jobs would answer a different question from the one
"The comparison that would verify the change" asks.

## What is planned for disk swap when an old drive is released

From the findings:
a raw swap partition on the released drive,
with zram writing its idle pages back to it,
or zswap in front of it.
Neither exists today.

## What is not known

Each of these is a question with the measurement that answers it.

1.  Do the two settings prevent a repeat?
    Measure:
    the two-read rate comparison in "The comparison that would verify the change",
    on a load comparable to 2026-11-22's.
2.  What did the counters do during the five minutes themselves?
    Measure:
    a pair of `/proc/vmstat` and `/proc/pressure` reads bracketing a stall,
    or a trace of direct reclaim while one runs,
    since every counter quoted here is cumulative from boot.
3.  Why did ZFS's intent log make two guests wait over two minutes?
    Measure:
    `zpool iostat -v` for the pool and per-dataset ZIL commit latency during a whole-gate run,
    with `/proc/pressure/io` read at the same seconds.
    The recorded stack shows `zil_commit_impl` waiting,
    not what it waited on.
4.  How does the 26803 MiB of swapped shared memory split between the guests and the gates' fixture file systems,
    and how much does one whole gate hold?
    Measure:
    run one gate alone with the guests stopped,
    reading `memory.swap.current` per cgroup and zram's DATA before and after.
    Partly answered on 2026-11-23:
    a gate's fixture file system is one `tmpfs` per suite container,
    mounted `rw,size=768m,mode=1777`
    (`package/git-policy/cli/bin/native-gate-fixture-root.ts`,
    whose own comment says its pages count against the container's memory),
    so one gate's fixtures hold at most 768 MiB,
    resident or swapped.
    One reading of a whole gate on `main` (run `native-RxZa1T`,
    at test 9,320 of 9,957,
    other sessions' guests and rounds running)
    found the suite container's cgroup at 277 MiB,
    77 MiB of it swapped and 76 MiB of it `shmem`
    (`~/temp/agent/apparmor-rules-for-the-wrapper-work-2026-11-23/keep/gate-container-memory-native-RxZa1T-reading-1.jsonl`,
    read by `read-a-gate-runs-container-memory.ts` beside it).
    That is one reading late in a run,
    not the gate's peak,
    and how many gates ran at the freeze is not recorded here.
    Read across a whole run on 2026-11-23
    (run `native-klGxVR` on `main` at `25d719bc6f`,
    from 2026-11-24T00:20:14Z to 00:41:19Z,
    every 30 seconds,
    30 reads,
    with four guests and the Pi rounds' builds running beside it):
    the gate's two containers peaked at 2,184 MiB resident together,
    1,855 MiB of it anonymous,
    169 MiB file and none `shmem`,
    with 956 MiB more in swap at that read;
    the most swapped at once was 1,724 MiB,
    at 2,165 MiB resident
    (`gate-container-memory-native-klGxVR-across-the-run.jsonl` in the same `keep/`,
    written by `sample-a-gate-runs-container-memory-until-it-ends.ts` beside it).
    So one gate asks for about 3.9 GiB at its peak,
    resident and swapped,
    and its fixtures' `tmpfs` was not a visible part of it at the peak.
    The guests stopped is the half of the measurement still owed.
5.  Where did `min_ttl_ms = 100` come from?
    Measure:
    search every source that can set it (`/usr/lib` and `/etc` `tmpfiles.d`,
    udev rules,
    sysctl files,
    units;
    a search of those on 2026-11-22 found no writer),
    then the shell history of the session that wrote it.
    Partly answered on 2026-11-23:
    it read 100 again after the restarts of 2026-11-23,
    so it is this kernel's boot value,
    and nothing a session wrote;
    a wider search found no writer
    (under the heading that names `/sys/kernel/mm/lru_gen/min_ttl_ms`).
    The distribution's kernel patch set is the place not yet read.
6.  Where did `vm.watermark_boost_factor = 0` come from,
    when the kernel [documents a default of 15,000][vm-boost]?
    Measure:
    the distribution's kernel patch set (`gh search code --repo CachyOS/kernel-patches watermark_boost_factor`
    finds patches for 5.15 through 6.17 and none under `7.x`),
    the kernel command line (read on 2026-11-22,
    with no watermark parameter),
    and the history of whichever session wrote the runtime value.
7.  What does transparent huge pages set to `always` cost here?
    Measure:
    `thp_fault_alloc` against `thp_fault_fallback`,
    and `compact_stall` against `compact_fail`,
    over a comparable load and again with `/sys/kernel/mm/transparent_hugepage/enabled` at `madvise`.

## Upstream filing artifact

### Upstream filing decision

The settings package is `cachyos-settings` (`pacman --query --owns` names it for both files),
whose upstream is [`CachyOS/CachyOS-Settings`][cachyos-repo].
What it ships,
read at commit `7de0e214f4fa0daa697f8133c29f5f67acdf95fd`:

- `usr/lib/sysctl.d/70-cachyos-settings.conf` sets [`vm.swappiness = 100`][sysctl-swappiness] and
  [`vm.page-cluster = 0`][sysctl-page-cluster],
  the two zram values that are sysctl keys.
  Its 47 lines carry no `vm.watermark_scale_factor` line.
- `usr/lib/udev/rules.d/30-zram.rules` [lines 13 and 14][zram-rules] set `SYSCTL{vm.swappiness}="150"` when
  `zram0` initialises,
  and disable zswap.
  The rule's comment lines 1 to 12 give the reason:
  with zram,
  only anonymous pages should be paged out,
  and the page cache should stay hermetic because reading it back from disk is expensive.
  A `SYSCTL` assignment here runs after `systemd-sysctl`,
  which is why the live value is 150 and not the file's 100.
- `usr/lib/systemd/zram-generator.conf` [lines 1 to 5][zram-generator-conf] set `zstd`,
  `zram-size = ram`,
  `swap-priority = 100`,
  and `fs-type = swap`.
- The package used to set the value this document applied,
  at commit `fb5130d0d671c51fc0601d66fed83dc312eb1a9c`
  ([the rule at that commit][zram-rules-before-55]).
- It was removed on purpose,
  merged as [commit `1e65d4696d`][drop-commit],
  for pull request [`CachyOS/CachyOS-Settings#55`][pr-55].

The rule's former lines are quoted from that commit:

```text
# Optimal value for games, so sets not too aggressive, but also not too weak
# kswapd behavior, as described there:
# https://www.reddit.com/r/linux_gaming/comments/vla9gd/comment/ie1cnrh/
SYSCTL{vm.watermark_scale_factor}="125"
SYSCTL{vm.watermark_boost_factor}="0"
```

The removal commit's message,
with the middle of it omitted:

```text
udev/zram: Drop tuning for watermark values  (#55)
...
This causes an unnecessarily large increase in RAM consumption.
```

The owner was told that history and chose the option labelled "keep 125 for now (Recommended)",
which the question tool numbered A,
with both settings staying as they are until a load like 2026-11-22's has been seen again.
The tradeoff that choice accepts is recorded under
"Workarounds applied at runtime and measured once without verification".

The value is proposed again and not merged.
[`CachyOS/CachyOS-Settings#138`][pr-138] proposes using the Arch wiki's zram rules and has been open since
2025-03-10 with no merge.
At its head commit `429f0379b00d3acc3970ce3e129244ed36bd674b` the rule adds
[`SYSCTL{vm.swappiness}="180"` on line 12 and `SYSCTL{vm.watermark_scale_factor}="125"` on line 15][pr-138-diff],
citing the Arch wiki's zram page,
whose four values are `vm.swappiness = 180`,
`vm.watermark_boost_factor = 0`,
[`vm.watermark_scale_factor = 125`][arch-zram],
and `vm.page-cluster = 0`,
described there as "what Pop!_OS uses".
Of those four the machine already ran page-cluster 0 from the vendor file and swappiness 150 from the udev rule,
and it read 0 for the boost factor (see "What is not known");
125 was the one not set.

The Arch wiki and the kernel documentation agree that a high swappiness is right for in-memory swap,
[in the `swappiness` section][swappiness]:

```text
For in-memory swap, like zram or zswap, as well as hybrid setups that have swap on faster devices than the
filesystem, values beyond 100 can be considered.
```

Three positions are on record on #138.
1Naim wrote on 2025-03-10:

```text
I think I'm generally fine with it, considering some other distro is shipping it. Can you move it to the sysctl file tho?
```

ptr1337 replied the same day:

```text
We have removed this since this increased the ram usage (cache) like x2
```

ventureoo wrote on 2025-03-12:

```text
Probably we just need to pick a some reasonably balanced value, as we have done with other non-obvious parameters.
```

Searches run on 2026-11-22,
recorded so the next session does not repeat them:

- `gh search code --repo CachyOS/CachyOS-Settings watermark_scale_factor`:
  0 hits on the default branch at commit `7de0e21`,
  so no file there sets it
  (the rule at `fb5130d0` and the head of pull request #138 do,
  as quoted in this section).
- `gh api 'search/issues?q=repo:CachyOS/CachyOS-Settings+watermark'`:
  3 results,
  the open pull request [#138][pr-138],
  the closed pull request [#55][pr-55],
  and closed pull request [#30][pull-30],
  which tuned the ZRAM rules for performance and where the value came from a Reddit comment recommending it
  for games.
  The queries `zram` and `swappiness` return 25 and 9;
  #30,
  #55 and #138 were read in full,
  including their review threads.
- Open work in progress:
  `gh api 'repos/CachyOS/CachyOS-Settings/pulls?state=open&per_page=30'` returned 10 open pull requests on
  2026-11-22,
  including #138,
  so the change this document applied is already proposed upstream and stalled.
- Maintainer churn on the files:
  `gh api 'repos/CachyOS/CachyOS-Settings/commits?path=usr/lib/udev/rules.d/30-zram.rules&per_page=5'`
  returns a latest commit on 2025-06-07 that reworks the ZRAM rule,
  and the same query for `usr/lib/sysctl.d/70-cachyos-settings.conf` returns 2026-11-09,
  which changes dirty ratios and writeback limits,
  neither touching a watermark key.
- Trust and contribution policy:
  the repository tree holds `CONTRIBUTING.md` and `README.md` and no `.github/` directory
  (the contents API returns 404 for it);
  a path filter over the whole tree for `vouch`,
  `trust`,
  `block`,
  `deny` and `allow` names finds nothing.
  `CONTRIBUTING.md` [asks contributors to discuss a change via issue or email first][contributing] and sets no
  AI-assistance rule,
  so no policy bans a filing,
  and none permits one unasked either.
- `.out-of-scope/` in this repository was read:
  no entry covers kernel or distribution configuration
  (the nearest are package managers,
  harnesses and build tools),
  so no exemption applies.

Walking the six constraints:

1.  **Is it really upstream's fault?**
    No.
    The shipped files do what their authors chose,
    the 0.1% headroom is the kernel's documented default rather than a defect,
    the freeze is an interaction between this workload and a configuration the project keeps on purpose,
    and upstream removed this exact value deliberately,
    with its cost written down.
2.  **Can upstream fix it?**
    Yes,
    and it did once:
    two `SYSCTL` lines,
    or one line in the sysctl file that a reviewer asked #138 to use instead.
    Capability is not the obstacle.
3.  **Are they supporting this use case?**
    Only partly.
    The package tunes zram for a desktop and gaming,
    and nothing in it addresses bursty many-guest,
    many-build pressure;
    no doc,
    example or test covers that load.
4.  **Would the repo welcome our contribution?**
    Yes,
    by its own written process:
    `CONTRIBUTING.md` asks for a discussion via issue or email before a change,
    and sets no template,
    no AI ban and no trust list.
    Absence of a policy is not a fail;
    it would require disclosing that an agent wrote the report and naming what a human checked.
5.  **Will they likely fix it?**
    Mixed,
    so this constraint does not decide it.
    #55 removed the value as merged policy with "an unnecessarily large increase in RAM consumption" as the
    reason.
    The three positions quoted here from 1Naim,
    ptr1337 and ventureoo are mixed:
    one is fine with the value if it moves to the sysctl file,
    one reports the RAM cost as the reason it went,
    and one asks for a balanced value.
    The pull request is still open and unmerged on 2026-11-23.
    A thread leaning two ways with no merge is not a clean won't-do,
    so this constraint is uncertain rather than failed.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Not reached,
    because constraint 1 fails on its own.
    The fix is a configuration line rather than code,
    and no patch is kept.

Decision:
**do not file**.
Constraint 1 fails on its own:
the shipped files do what their authors chose,
0.1% is the kernel's documented default rather than a defect,
and the freeze is a workload interaction with a configuration this project keeps on purpose.
Constraint 5 does not settle it either way,
and the skill's default is not to file.
No issue,
comment or draft is furnished here,
and nothing was posted anywhere.

No fenced artifact is kept.
The skill keeps a draft whatever the decision,
and this document says explicitly that no artifact exists yet:
the only artifact a later session would need is a comment on #138,
and it cannot be drafted until the comparison in "The comparison that would verify the change" exists.

There is also nothing to add to the open #138 today.
That thread already carries the Arch wiki values,
the claim about kswapd and page cache,
and the RAM cost objection;
what this session could add is the before measurement,
and the after reading covers a lighter load.
If a future session finishes the comparable-load comparison,
the additive comment goes on #138 and its content is that comparison,
not the wiki quote again.

Risk to the poster,
stated next to the decision:
filing puts the owner's GitHub account in front of maintainers who have already declined this value once.
A maintainer can read an agent-written re-request as low effort,
and this repository's contributors have no trust-list standing to spend.
Filing is the owner's decision,
and this document recommends against it.

[vm-watermark]: https://docs.kernel.org/admin-guide/sysctl/vm.html#:~:text=The%20default%20value%20of%2010%20means%20the%20distances%20between%20watermarks%20are%200.1%25%20of%20the%20available%20memory
[vm-allocstall]: https://docs.kernel.org/admin-guide/sysctl/vm.html#:~:text=A%20high%20rate%20of%20threads%20entering%20direct%20reclaim
[vm-boost]: https://docs.kernel.org/admin-guide/sysctl/vm.html#:~:text=The%20default%20value%20of%2015%2C000
[swappiness]: https://docs.kernel.org/admin-guide/sysctl/vm.html#:~:text=values%20beyond%20100%20can%20be%20considered
[psi-full]: https://github.com/gregkh/linux/blob/v7.3.5/Documentation/accounting/psi.rst?plain=1#L48-L58
[psi-windows]: https://github.com/gregkh/linux/blob/v7.3.5/Documentation/accounting/psi.rst?plain=1#L60-L62
[lru-thrash]: https://github.com/gregkh/linux/blob/v7.3.5/Documentation/admin-guide/mm/multigen_lru.rst?plain=1#L71-L82
[page-alloc-direct]: https://github.com/gregkh/linux/blob/v7.3.5/mm/page_alloc.c?plain=1#L4470-L4482
[page-alloc-wakeup]: https://github.com/gregkh/linux/blob/v7.3.5/mm/page_alloc.c?plain=1#L4520-L4528
[vmscan-ttfp]: https://github.com/gregkh/linux/blob/v7.3.5/mm/vmscan.c?plain=1#L6779-L6793
[fs-aops]: https://github.com/gregkh/linux/blob/v7.3.5/include/linux/fs.h?plain=1#L437-L440
[zpl-aops]: https://github.com/openzfs/zfs/blob/a1daebfec3f5ca46ac261ef339efdf0dbeae997c/module/os/linux/zfs/zpl_file.c#L1394-L1421
[sysctl-precedence]: https://www.freedesktop.org/software/systemd/man/latest/sysctl.d.html#:~:text=the%20entry%20in%20the%20file%20with%20the%20lexicographically%20latest%20name%20will%20take%20precedence
[sysctl-format]: https://www.freedesktop.org/software/systemd/man/latest/sysctl.d.html#:~:text=kernel.domainname%3Dfoo
[tmpfiles-w]: https://www.freedesktop.org/software/systemd/man/latest/tmpfiles.d.html#:~:text=Write%20the%20argument%20parameter%20to%20a%20file%2C%20if%20the%20file%20exists
[openzfs-swap]: https://openzfs.github.io/openzfs-docs/Project%20and%20Community/FAQ.html#:~:text=swap%20on%20zvol%20may%20lead%20to%20deadlock
[arch-zram]: https://wiki.archlinux.org/title/Zram#:~:text=vm.watermark_scale_factor%20%3D%20125
[cachyos-repo]: https://github.com/CachyOS/CachyOS-Settings
[sysctl-swappiness]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/sysctl.d/70-cachyos-settings.conf#L4
[sysctl-page-cluster]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/sysctl.d/70-cachyos-settings.conf#L20
[zram-rules]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/udev/rules.d/30-zram.rules#L13-L14
[zram-generator-conf]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/systemd/zram-generator.conf#L1-L5
[zram-rules-before-55]: https://github.com/CachyOS/CachyOS-Settings/blob/fb5130d0d671c51fc0601d66fed83dc312eb1a9c/etc/udev/rules.d/30-zram.rules#L16-L20
[drop-commit]: https://github.com/CachyOS/CachyOS-Settings/commit/1e65d4696d0836b4b727ce61f3a29376a11e99a7
[pr-55]: https://github.com/CachyOS/CachyOS-Settings/pull/55
[pr-138-diff]: https://github.com/CachyOS/CachyOS-Settings/blob/429f0379b00d3acc3970ce3e129244ed36bd674b/usr/lib/udev/rules.d/30-zram.rules#L12-L15
[contributing]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/CONTRIBUTING.md?plain=1#L3-L4
[pr-138]: https://github.com/CachyOS/CachyOS-Settings/pull/138
[pull-30]: https://github.com/CachyOS/CachyOS-Settings/pull/30
[thp-conf]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/tmpfiles.d/thp.conf#L3

## Moved from the old handover

From lines 43501 to 43503 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  facts read at 2026-11-22T15:50-05:00, for whoever reads a guest timing: swap is zram (61.7 G, 36 G used) at
  `vm.swappiness` 150; three guests held 0.9 G resident and 23 G in zram (unit peak 23.7 G of 33.1 G); the ZFS cache was at
  2.1 G of 60.7 G; memory pressure about 4 percent; 30 processors at a load of 17 to 25. Three notes went to running rounds

From lines 43553 to 43559 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  newer in its guest, since every guest figure of this branch so far is from 2.43.0. MORE RUNS AT ONCE: the owner asked "Can
  we parallelize more?" after the 6344 round's start. Measured between then and 2026-11-22T17:16:54-05:00: eight Pi runs and
  one Sonnet check going against a standard pool that takes 20; 29 GiB of memory available; three guests of 8 GiB each
  (mvm's default) holding 2.5 GiB resident and 30 GiB of swap in a unit capped at 33 GiB; swap 44 of 63 GiB used (zram held
  35.1 GB in 12.7 GB); load 19 to 27 on 30 processors. What held work back was the order of work and not the machine: none
  of the designs the plan marks as blocking steps 10 to 19 was written. Step 10's "Depends on" (steps 2, 3, 6 and 8) is
  met, but its "Open items" block it on the add-time Git lock, and its binding record has no home (plan lines 1479 and 2864

From lines 1364 to 1372 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The freeze's troubleshooting document:
  its revision after the short review is merged.
  A reading taken at 2026-11-23T04:20:03-05:00 while these rounds ran is in the freeze's scratch folder,
  `keep/counters-during-the-rounds-of-the-early-morning-of-2026-11-23.txt`:
  over the night kswapd did 73.1% of the scanning and tasks were stalled on memory 1.97% of the time,
  with free memory at 6,572 MiB at the read.
  It is an average over a night nobody watched,
  not the comparison under a load like the freeze's,
  which is still owed (task 463).
