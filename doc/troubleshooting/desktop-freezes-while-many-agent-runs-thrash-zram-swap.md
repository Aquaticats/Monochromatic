# CachyOS on kernel 7.3.5: many agent runs fill zram swap and the desktop freezes for lack of kswapd headroom

Status:
the two kernel settings in "Verified workarounds" were applied at runtime on 2026-11-22,
and have **not** been verified under load.
Nothing here is approved or final;
the project's owner declares the work complete.

Every figure in this document was measured on the machine on 2026-11-22 by the coordinating agent session.
That session recorded each one in its findings file.
The few figures this document adds are each named with the read-only command that produced them.
Times are local,
UTC-05:00,
at the precision the measurement recorded.
Most carry seconds;
the atop samples and one `/proc/vmstat` read carry minutes.

## Symptom

The owner,
at about 20:57:
"My system was just stuck for like 5 minutes.
Might be a us problem."
The desktop stopped answering for about five minutes and then came back.
Nothing crashed,
nothing was killed,
and no log line named a failing program.
While it hung,
one coordinating session had this running,
all of it started by that session or its agents:
6 guest machines,
3 to 4 whole container test gates of `package/git-policy/cli`,
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

The `full` line is the one that matters:
it counts the time when every non-idle task was stalled at once.
Over the five minutes before the read that was 46% of the time.
The same second's other files put the stall on memory and not on the disk or the CPUs:

```text
/proc/pressure/io:  some avg300=48.45 full avg300=3.09
/proc/pressure/cpu: some avg300=22.35
```

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

The second sample's page scan rate is far above the first:
by 21:00 the machine was scanning pages for reclaim in the millions per second.

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

### The swap device at 21:01

`zramctl --output-all` and `/proc/swaps` show `/dev/zram0` alone,
with no disk swap:
zstd,
priority 100,
disk size 61.7G,
DATA 37.2G compressed to COMPR 14G,
TOTAL 14.3G,
MEM-USED peak 17.8G,
and a ratio of 2.5979.

### The signature, in one place

- `/proc/pressure/memory` with `full` tracking `some` closely over minutes:
  every task stalled,
  not a few.
- `/proc/vmstat` with direct reclaim dominating background reclaim (the counters are in "Root cause").
- `journalctl --dmesg` "blocked for more than N seconds" lines for tasks that are not the desktop's own.
- `zramctl` showing a nearly full compressed swap device with no disk swap behind it.

## Root cause

Two layers,
kept apart.
First,
what the running jobs asked of memory at that moment,
and what made the sum fit at all.
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

Swap in use system-wide was 41377 MiB,
of which those processes account for 14574 MiB.
The other 26803 MiB is swapped shared memory,
which a process's own swap figure does not count:
guest memory and memory-backed file systems,
among them the test gate's fixture file system.
The split between those two was not measured.

So the footprint was roughly 38.4 GiB of resident process memory,
guests' shared memory included,
40.4 GiB swapped,
and 4.5 GiB of kernel slab,
on 61.7 GiB of RAM,
with zram holding the swapped part in about 16 GiB of that RAM.
The commit accounting did not fit either:
commit space was 141908M against a limit of 94728M.

Two things follow,
and both are measurements rather than opinions.

- The swapped part could not have stayed resident.
  40.4 GiB of anonymous and shared memory was paged out to a device that lives in the same 61.7 GiB,
  and it fitted because that device is compressed swap:
  37.2G of DATA in 14.3G of zram TOTAL,
  at a ratio of 2.5979.
  Compressed swap is what made the workload fit.
- The pressure reached the owner's own programs,
  not only the batch jobs.
  The browser had 3596 MiB swapped,
  `plasmashell` 311 MiB,
  and `opensnitch-ui` 985 MiB.
  A person whose window manager's pages are being reclaimed is looking at a frozen desktop,
  because the programs that would draw the "busy" indicator cannot get their own pages back fast enough.

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

At 0.1% of memory the headroom is tens of MiB at this machine's memory size:
enough for a quiet desktop,
not enough for this workload's allocation bursts.
The counters say that is what happened.
Read at 21:01 on 2026-11-22 (the findings record minutes for that read),
since boot (3 days 9 hours):

- `pgscan_direct 7252497503` against `pgscan_kswapd 3998833981`:
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

When every allocating task must reclaim its own pages,
each task's progress waits on page scans and on zram's compress and decompress path,
so the delay spreads to every process on the machine,
including the ones that did not ask for memory at all.
That is what `full avg300=45.76` measures.
It is the documented allocation-burst case,
and the knob the documentation names for it was at its default.

### Nothing shed load

The multi-gen LRU's thrash valve was nearly shut.
`/sys/kernel/mm/lru_gen/min_ttl_ms` read 100.
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

`systemd-oomd` had nothing to act on either.
Its `oomctl` output records a swap used limit of 90.00%,
memory pressure monitored only for `/system.slice` (80%,
30 s),
and every user slice monitored for swap use only.
Swap use never neared 90% (35687 of 63151 MiB,
with zram holding 37.2G of its 61.7G),
so the daemon never selected a user cgroup.

The findings also record two guests' flushes waiting over two minutes inside ZFS's intent log,
the `dmesg` lines quoted in "Symptom".
Whether memory reclaim or the disk made ZFS that slow is not established,
so this document does not use those waits as cause.
The ZFS ARC was not the cause either:
at 21:00 its `size` was 6123719120 against `c_max` 65146339328 and `c_min` 2069377536.

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
- Resident process memory at 20:50 was 38.4 GiB and kernel slab 4.5 GiB,
  and 40.4 GiB more was swapped out and held compressed in about 16 GiB of the same RAM.
- Commit space exceeded its limit (141908M against 94728M),
  so the shortage was in committed memory rather than in installed memory.
- Compressed swap is what carried the difference,
  so the owner's second sentence,
  "We might have to change our swap or zram or whatever conf",
  describes what was then changed:
  the two kernel settings in "Verified workarounds".

## Verification

### Versions under test

- Kernel `7.3.5-1-cachyos` (`uname --kernel-release`).
- `systemd 262.2-1` and `zram-generator 1.2.1-1.1` (`pacman --query systemd zram-generator`).
- `cachyos-settings 1:1.4.5-1`,
  built 2026-11-20 (`pacman --query --info cachyos-settings`).
- Swap is zram only:
  `/proc/swaps` lists `/dev/zram0` alone,
  and `zramctl` reports zstd on a 61.7G device.

### The reads

```sh
# doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md
cat /proc/pressure/memory /proc/pressure/io
cat /proc/vmstat
free --mebi
uptime
zramctl --output-all
oomctl
journalctl --dmesg | rg 'blocked for more than'
atopsar -r /var/log/atop/atop_20261122 -m
atopsar -r /var/log/atop/atop_20261122 -s
sysctl vm.watermark_scale_factor vm.watermark_boost_factor
cat /sys/kernel/mm/lru_gen/min_ttl_ms
```

### Catalog: the values across the stall

The discriminating values,
all from the reads above at their recorded times:

- memory pressure `some avg300=47.05` and `full avg300=45.76` at 20:56:41;
- page scan 4681429/s and commit space 124016M at 21:00:00;
- `pgscan_direct` above `pgscan_kswapd`,
  with `kswapd_low_wmark_hit_quickly` 862758 against `kswapd_high_wmark_hit_quickly` 96,
  since boot at 21:01;
- two "blocked for more than 123 seconds" lines at 20:55:27;
- zram holding 37.2G of DATA in 14.3G of TOTAL,
  with no disk swap.

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

### The comparison that would verify the change

No such comparison has been made.
This is the one that would settle it.

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
- memory `full avg300` stays far below 45.76,
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

## Verified workarounds

Status:
applied at runtime on 2026-11-22,
**not verified**.
No load comparison exists yet,
so neither setting is known to prevent a repeat.
Both are runtime values:
a reboot returns them to the stock values,
so a fix that survives a reboot needs the draft in "The persistent form".

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
The documented unit makes 125 equal to 1.25% of memory against the stock 0.1%:
an orderly reclaim thread with a cushion,
at the price of a permanently larger free pool.
That is the cost the CachyOS settings package measured when it shipped this exact value and then removed it,
and it is named in "Upstream filing artifact" below.

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
and a reboot starts it again from whatever the kernel boots with.
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
nothing has been measured after it.

### The persistent form, as a draft for the owner

Draft only.
Nothing here is installed,
and this document installs nothing.
Two files,
one per setting,
for the owner to review and apply.

`/etc/sysctl.d/90-vm-watermark-scale-factor.conf`:

```ini
# kernel reclaim headroom: 1.25% of memory instead of the stock 0.1%
vm.watermark_scale_factor = 125
```

The `sysctl.d` page defines this form:
files are read at boot by `systemd-sysctl.service`,
hold `key = value` assignments,
and the entry in the file with the
[lexicographically latest name takes precedence][sysctl-precedence],
which is why a 90-prefixed file in `/etc/` wins over the vendor's `70-cachyos-settings.conf`.
A `sysctl.d` file cannot carry the second setting,
because it writes `/proc/sys` keys only:
the page's own example is the assignment `kernel.domainname=foo`,
which it says will cause "foo" to be written to `/proc/sys/kernel/domainname`
([same page][sysctl-format]).

`/etc/tmpfiles.d/lru-gen-min-ttl.conf`:

```ini
# one second of protected working set, applied once at boot
w! /sys/kernel/mm/lru_gen/min_ttl_ms - - - - 1000
```

The `tmpfiles.d` page defines the line type:
[`w` writes the argument parameter to a file, if the file exists][tmpfiles-w],
and "The argument parameter will be written without a trailing newline."
The `!` modifier makes the line boot-only.
This distribution already writes a `/sys` node exactly this way,
in its `usr/lib/tmpfiles.d/thp.conf` [line 3][thp-conf],
so the pattern is one the machine's own boot already runs.

Tradeoff,
named rather than hidden:
with `!` alone,
a boot on a kernel without that node makes the boot-time run report a failure for this line.
The page documents a `-` modifier that suppresses a failing line,
and this draft leaves that combination out because it has not been tried here.
Both files also only act at boot;
on a running system the owner applies the runtime commands above,
or reads the values back with `sysctl vm.watermark_scale_factor` and
`cat /sys/kernel/mm/lru_gen/min_ttl_ms`.

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
  `zpool list` shows `zpcachyos` at 3.48T size with 993G allocated.
  There is no free partition to give to swap.
- A ZFS volume as swap is the documented deadlock risk.
  The OpenZFS FAQ's swap section warns:
  ["**CAUTION:** for now swap on zvol may lead to deadlock, in this case please send your logs
  here"][openzfs-swap],
  pointing at `openzfs/zfs` issue 7734.
- A swap file on ZFS has no support in its source.
  `gh search code --repo openzfs/zfs swap_activate` returns nothing,
  so the address-space operation the kernel's swap-file path needs is not implemented there.
- The other drives are held read-only by the owner's rule until the migration is done:
  `sda` (3.6T,
  `crypto_LUKS`) and `nvme0n1` (1.9T,
  `crypto_LUKS` partitions) are the old drives,
  kept unwiped,
  so nothing on them may be repartitioned now.
- It would not have been used that night.
  zram carries swap priority 100 (`/proc/swaps`) and held 37.2G of its 61.7G,
  so a lower-priority disk swap would have stayed untouched while zram still had room.
  Disk swap added behind a higher-priority zram changes nothing until zram fills.

### What is planned when an old drive is released

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
2.  Why did ZFS's intent log make two guests wait over two minutes?
    Measure:
    `zpool iostat -v` for the pool and per-dataset ZIL commit latency during a whole-gate run,
    with `/proc/pressure/io` read at the same seconds.
    The recorded stack shows `zil_commit_impl` waiting,
    not what it waited on.
3.  How does the 26803 MiB of swapped shared memory split between the guests and the gates' fixture file systems,
    and how much does one whole gate hold?
    Measure:
    run one gate alone with the guests stopped,
    reading `memory.swap.current` per cgroup and zram's DATA before and after.
4.  Where did `min_ttl_ms = 100` come from?
    Measure:
    search every source that can set it (`/usr/lib` and `/etc` `tmpfiles.d`,
    udev rules,
    sysctl files,
    units;
    a search of those on 2026-11-22 found no writer),
    then the shell history of the session that wrote it.
5.  Where did `vm.watermark_boost_factor = 0` come from,
    when the kernel [documents a default of 15,000][vm-boost]?
    Measure:
    the distribution's kernel patch set (`gh search code --repo CachyOS/kernel-patches watermark_boost_factor`
    finds patches for 5.15 through 6.17 and none under `7.x`),
    the kernel command line (read on 2026-11-22,
    with no watermark parameter),
    and the history of whichever session wrote the runtime value.
6.  What does transparent huge pages set to `always` cost here?
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
- `usr/lib/udev/rules.d/30-zram.rules` [line 13][zram-rules] sets `SYSCTL{vm.swappiness}="150"` when `zram0`
  initialises,
  and disables zswap.
  The comment above it gives the reason:
  with zram,
  only anonymous pages should be paged out,
  and the page cache should stay hermetic because reading it back from disk is expensive.
  A `SYSCTL` assignment here runs after `systemd-sysctl`,
  which is why the live value is 150 and not the file's 100.
- `usr/lib/systemd/zram-generator.conf` [lines 1 to 5][zram-generator-conf] set `zstd`,
  `zram-size = ram`,
  and `swap-priority = 100`.
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

The value is proposed again and not merged.
[`CachyOS/CachyOS-Settings#138`][pr-138] proposes using the Arch wiki's zram rules and has been open since
2025-03-10 with no merge.
At its head commit `429f0379b00d3acc3970ce3e129244ed36bd674b` the rule adds
[`SYSCTL{vm.swappiness}="180"` and `SYSCTL{vm.watermark_scale_factor}="125"`][pr-138-diff],
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

Searches run on 2026-11-22,
recorded so the next session does not repeat them:

- `gh search code --repo CachyOS/CachyOS-Settings watermark_scale_factor`:
  0 hits,
  so no source file sets it.
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
  `gh api 'repos/CachyOS/CachyOS-Settings/pulls?state=open&per_page=30'` returns 10 open pull requests,
  including #138,
  so the change this document applied is already proposed upstream and stalled.
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
    and the combination turned bad only under this workload.
    Upstream also removed this exact value deliberately,
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
    No.
    #55 removed the value as merged policy,
    with "an unnecessarily large increase in RAM consumption" as the reason,
    and the maintainer's [reply on #138][pr-138] repeats that the RAM and cache cost is why it went;
    #138 has been open since 2025-03-10 without a merge.
    That is a recorded won't-do for this value rather than silence.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Not reached,
    because constraints 1 and 5 fail.
    The fix is a configuration line rather than code,
    and no patch is kept.

Decision:
**do not file**.
No issue,
comment or draft is produced here,
and nothing was posted anywhere.

There is also nothing to add to the open #138.
That thread already carries the Arch wiki values,
the claim about kswapd and page cache,
and the maintainers' RAM cost objection;
the only material this session could add is the before measurement,
and the after measurement does not exist yet.
A comment resting on half a comparison would be the low-effort filing that the poster's account pays for.
If a future session finishes the two-read comparison in "Verification",
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
[lru-thrash]: https://github.com/gregkh/linux/blob/v7.3.5/Documentation/admin-guide/mm/multigen_lru.rst?plain=1#L71-L82
[sysctl-precedence]: https://www.freedesktop.org/software/systemd/man/latest/sysctl.d.html#:~:text=the%20entry%20in%20the%20file%20with%20the%20lexicographically%20latest%20name%20will%20take%20precedence
[sysctl-format]: https://www.freedesktop.org/software/systemd/man/latest/sysctl.d.html#:~:text=kernel.domainname%3Dfoo
[tmpfiles-w]: https://www.freedesktop.org/software/systemd/man/latest/tmpfiles.d.html#:~:text=Write%20the%20argument%20parameter%20to%20a%20file%2C%20if%20the%20file%20exists
[openzfs-swap]: https://openzfs.github.io/openzfs-docs/Project%20and%20Community/FAQ.html#:~:text=swap%20on%20zvol%20may%20lead%20to%20deadlock
[arch-zram]: https://wiki.archlinux.org/title/Zram#:~:text=vm.watermark_scale_factor%20%3D%20125
[swappiness]: https://docs.kernel.org/admin-guide/sysctl/vm.html#:~:text=values%20beyond%20100%20can%20be%20considered
[cachyos-repo]: https://github.com/CachyOS/CachyOS-Settings
[sysctl-swappiness]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/sysctl.d/70-cachyos-settings.conf#L4
[sysctl-page-cluster]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/sysctl.d/70-cachyos-settings.conf#L20
[zram-rules]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/udev/rules.d/30-zram.rules#L13
[zram-generator-conf]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/systemd/zram-generator.conf#L1-L5
[zram-rules-before-55]: https://github.com/CachyOS/CachyOS-Settings/blob/fb5130d0d671c51fc0601d66fed83dc312eb1a9c/etc/udev/rules.d/30-zram.rules#L16-L20
[drop-commit]: https://github.com/CachyOS/CachyOS-Settings/commit/1e65d4696d0836b4b727ce61f3a29376a11e99a7
[pr-55]: https://github.com/CachyOS/CachyOS-Settings/pull/55
[pr-138-diff]: https://github.com/CachyOS/CachyOS-Settings/blob/429f0379b00d3acc3970ce3e129244ed36bd674b/usr/lib/udev/rules.d/30-zram.rules#L14-L15
[contributing]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/CONTRIBUTING.md?plain=1#L3-L4
[pr-138]: https://github.com/CachyOS/CachyOS-Settings/pull/138
[pull-30]: https://github.com/CachyOS/CachyOS-Settings/pull/30
[vm-boost]: https://docs.kernel.org/admin-guide/sysctl/vm.html#:~:text=The%20default%20value%20of%2015%2C000
[thp-conf]: https://github.com/CachyOS/CachyOS-Settings/blob/7de0e214f4fa0daa697f8133c29f5f67acdf95fd/usr/lib/tmpfiles.d/thp.conf#L3
