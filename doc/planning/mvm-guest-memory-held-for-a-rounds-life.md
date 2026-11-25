# How mvm guests could hold less of the host's memory

**Non-final draft by an agent,
unapproved.**
It proposes;
it changes no code and no `mvm` default,
and the owner chooses.
Every reading below was taken on 2026-11-25 on this host,
and the logs,
samples and scripts behind them are in the round folder's `keep/`.

## The question

On this host one `mvm` guest holds about 6.6 GiB of the machine's memory for as long as its round lives,
even when the guest is idle,
and four live rounds' guests held 25 GiB of the machine's 61.7 GiB at 2026-11-25T16:05:48-05:00.
The memory is not the host's build cache and not QEMU's own heap:
it is the guests' own RAM,
kept on the host as shared memory because of how `mvm` backs a guest that has a virtiofs share.
This document measures every way the host could get that memory back while rounds still build in their guests,
and ranks them by how much each returns.

## What the host held on 2026-11-25

Read from the host at 2026-11-25T16:05:48-05:00,
with four rounds' guests running (`zfs-step18-doctor-removal`,
`zfs-step-17-cancel`,
`zfs-clone-mountpoint-probe`,
`zfs-removal-finish-pass`):

- The `mvm` session daemon's unit `user@1000.service/app.slice/mvm-libvirt-session.service` reported
  `memory.current` 24,782,053,376 bytes (23.1 GiB) and `memory.stat` `shmem` 24,469,475,328 bytes (22.8 GiB),
  with `anon` 78,569,472 bytes (75 MiB).
- `/proc/meminfo` read `Shmem` 24,024,432 kB and `AnonPages` 15,334,804 kB.
- Each guest's QEMU process had the whole of its RAM in one mapping,
  `/memfd:memory-backend-memfd (deleted)`,
  `rw-s` (mapped shared),
  and its `smaps_rollup` `Rss` read 7.59,
  2.66,
  5.06 and 7.91 GiB.
  One guest's roll-up also read 5.36 GiB of `Swap`:
  the host had already written that guest's RAM into the zram swap device,
  which is why its `Rss` was the smallest of the four.
- The zram swap device held 21 GiB of the 61.7 GiB of swap,
  so the machine was already reclaiming guest memory under pressure,
  and still held 22.8 GiB of it.

So the memory the machine gives to guests is charged,
byte for byte,
to shared memory the guests' own pages live in,
and the host's only existing way to take it back is to swap it,
under pressure,
at zram's cost in CPU and in the guests' latency.

## How a guest's memory is backed and sized

### The domain `mvm` writes

`package/mvm/cli/src/config.ts` gives a guest 8,192 MiB and 4 vCPUs,
and no command-line flag changes either:
`DEFAULT_MEMORY_MIB` and `DEFAULT_VCPUS` are read by `package/mvm/cli/src/domain-xml.ts`,
which writes `<memory unit="MiB">8192</memory>` and `<vcpu>4</vcpu>`.

When the share decision is `virtiofs`,
`domain-xml.ts` also adds the share's device and,
beside the memory element,
the backing that device needs:

```xml
<memoryBacking>
  <source type="memfd"/>
  <access mode="shared"/>
</memoryBacking>
```

`package/mvm/cli/src/domain-xml-builders.ts` builds those two elements
(`sharedMemoryBacking`),
and its documentation says why:
a virtiofs share needs guest memory that `virtiofsd` can map,
so `access mode="shared"` is the mode,
and `source type="memfd"` the backing.
The guest-agent route,
taken only when libvirt finds no `virtiofsd`,
writes neither the share nor the backing,
which is the one case where a guest's memory is private.

`mvm` writes no `memballoon` element,
so libvirt adds its default one:
the running guests' libvirt-written XML holds
`<memballoon model='virtio'>` with no attributes.

### What QEMU then does with that memory

- `access mode="shared"` becomes QEMU's `memory-backend-memfd` with `share=on`,
  and `backends/hostmem-memfd.c` in QEMU v11.1.0 turns that into a RAM block that is `RAM_SHARED` and has a
  file descriptor (`memory_region_init_ram_from_fd`).
  Every page the guest touches becomes a page of that memfd,
  which the host accounts as shared memory,
  and the host's page cache holds it whether or not the guest still uses it.
- Free page reporting,
  the virtio-balloon feature that returns pages the guest has freed,
  is off.
  QEMU's property default is off (`DEFINE_PROP_BIT("free-page-reporting", ..., VIRTIO_BALLOON_F_REPORTING, false)`
  in `hw/virtio/virtio-balloon.c`),
  and libvirt writes the QEMU property only when the domain XML says
  `freePageReporting='on'`
  (`src/qemu/qemu_command.c` passes the XML's tristate straight through).
- If it were on,
  QEMU would discard a reported page through `ram_block_discard_range`,
  which for a RAM block with a file descriptor calls
  `fallocate(FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE)` on the memfd and then
  `madvise(MADV_DONTNEED)` on the mapping (`system/physmem.c`).
  Punching a hole in a memfd frees the page from the host's page cache,
  so reporting is not blocked by `access mode="shared"`:
  nothing in the virtiofs route disables discards.
  `ram_block_discard_disable` is called by vfio,
  vhost-vDPA,
  SEV guests,
  `blkio` block devices and KVM without a synchronised MMU,
  which this guest has none of.
- KSM cannot touch that memory at all.
  QEMU marks a guest's RAM mergeable when merging is on
  (`backends/hostmem.c` calls `qemu_madvise(..., QEMU_MADV_MERGEABLE)`),
  but `Documentation/admin-guide/mm/ksm.rst` states that KSM only merges anonymous private pages,
  never page-cache pages,
  and a memfd is page cache.
  The host's own view agrees:
  the running guests' RAM mappings carry no `mg` flag in `/proc/<pid>/smaps`
  while six smaller mappings in the same processes do.

### What that costs per guest

A guest asked for 8,192 MiB,
so the host may hold up to 8 GiB of shared memory per guest,
and the unit's `MemoryMax` is half of `MemTotal`,
which is 30.8 GiB here,
so four guests at the default exceed the cap:
`package/mvm/cli/README.md` states that the kernel then kills the guest it picks rather than the daemon.

## The options this document measures

Each is measured in one throwaway guest,
reading the host's own view before,
during and after one real build of the wrapper in that guest.

1. Free page reporting on (`<memballoon model="virtio" freePageReporting="on"/>`).
2. The guest dropping its own page cache after an arm (`echo 3 > /proc/sys/vm/drop_caches`),
   with reporting on and with it off.
3. A smaller guest whose arms run a binary built once,
   so the guest compiles nothing.
4. A guest without the virtiofs share,
   whose memory is private,
   which KSM is the only thing that could then merge across guests of one template.
5. Destroying a guest between a round's arms instead of keeping it for the round's life.

Options that were considered and are not measured,
with the reason,
are in their own section below.

## Method

Every guest in this round is a throwaway `mvm` libvirt guest on the Ubuntu template,
started only through `mvm`,
and destroyed through `mvm` as soon as its readings are saved.
One guest runs at a time,
and no guest starts unless the round note's test holds:
`MemAvailable` plus the ARC's `size` minus its `c_min` is at least 16 GiB
and memory pressure's `avg10` is not above `avg60`.
Each create's gate reading is in `keep/memory-gate-log.txt` in the round folder.

For each guest the round reads,
before its arm,
during it and after it:

- `/proc/meminfo`'s `MemTotal`,
  `MemAvailable`,
  `Shmem`,
  `AnonPages`,
  `Cached` and `SwapFree`;
- the session daemon's unit `memory.current` and the `memory.stat` fields `shmem`,
  `anon`,
  `file`,
  `active_file` and `inactive_file`;
- memory pressure's `some` and `full` lines;
- the guest's own QEMU process:
  `smaps_rollup`'s `Rss`,
  `Pss`,
  `Shared_Clean`,
  `Shared_Dirty`,
  `Private_Dirty`,
  `Anonymous` and `Swap`,
  the description and flags of its largest mapping,
  and how many of its mappings carry the mergeable flag;
- KSM's `run`,
  `pages_shared`,
  `pages_sharing`,
  `pages_scanning`,
  `pages_to_scan` and `sleep_millisecs`.

The load in each guest is one real build of the wrapper,
run the way the rounds' arms run it:
`CARGO_BUILD_JOBS=1 cargo build --release --tests --locked` in a 4-vCPU guest with a 6 GiB swap file,
from the repository's own wrapper snapshot,
started detached so the guest agent cannot lose a long command's answer.
The snapshot helper is `package/git-policy/cli/bin/native-wrapper-snapshot.ts`,
whose tarball is 12.6 MB.
The host samples every 10 seconds while the build runs.
The sampler,
the guest scripts and every reading are under `keep/` in the round folder.

Two measurements need a domain attribute or a smaller memory than `mvm`'s constants allow,
and `mvm` has no flag for either.
For those probe guests only,
the worktree's `package/mvm/cli/src/domain-xml.ts` and `package/mvm/cli/src/file-transfer-route.ts`
carry a temporary block that reads `MVM_MEMORY_PROBE`
(`reporting`,
`memory4g`,
`agent`),
run through `mise run //package/mvm/cli:run -- ...`,
which is `mvm` from source;
the block is reverted before any commit and its diff is `keep/probe-patch-for-the-measurement.diff`.
No other code and no constant is changed,
and the guests those probes create are ordinary throwaway guests.

## Results

### The default guest: 8 GiB, the virtiofs share, shared memfd, reporting off

Guest `mem-a` was `mvm`'s own output with no change at all:
8,192 MiB asked for,
7.75 GiB usable in the guest (`MemTotal` 8,131,772 kB),
the virtiofs share,
`<memoryBacking><source type="memfd"/><access mode="shared"/></memoryBacking>`,
and a virtio balloon with no attributes.
The load was one build of the wrapper,
`CARGO_BUILD_JOBS=1 cargo build --release --tests --locked` with a 6 GiB swap file,
which finished in 34 minutes 13 seconds:
longer than the 18 minutes the round note quotes,
because this machine was under such pressure that the guest's pages were swapped out twice while it built.

The readings,
in order,
as the host's charge to the guest (`smaps_rollup` `Rss` + `Swap` of its RAM mapping):

- 76 seconds after `mvm create`:
  0.88 GiB charged,
  all resident.
  `mvm create` itself took 11 seconds.
- After preparing the guest for a build (25 Debian packages and the Rust 1.100.0 toolchain,
  100 seconds):
  2.99 GiB charged.
  The guest's own view was `MemFree` 6.0 GiB and `Cached` 1.47 GiB.
- During the build:
  5.51 GiB resident at the peak (16:44:25-05:00),
  7.23 GiB in swap at the peak (16:51:15-05:00),
  and 8.16 GiB charged at the peak (16:47:19-05:00).
  The host's `MemAvailable` reached 0 MiB at 16:22:18-05:00,
  when memory pressure's `some avg10` read 14.01,
  and again at 16:41 with `some avg10` 22.92.
- After the build,
  at rest:
  7.98 GiB charged,
  of which 1.00 GiB resident and 6.99 GiB in the zram swap device.
  The guest's own view was `MemFree` 6,610,188 kB (6.3 GiB of its 7.75 GiB),
  `Cached` 1.16 GiB,
  `AnonPages` 54 MiB:
  the guest called 6.3 GiB of its memory free,
  and the host could not use any of it.
- After the guest dropped its own page cache
  (`sync; echo 3 > /proc/sys/vm/drop_caches`):
  the guest's `MemFree` rose to 7,887,348 kB and its `Cached` fell to 75,588 kB,
  and the host's charge did not move:
  7.98 GiB charged before,
  7.98 GiB after (1.09 GiB resident,
  6.89 GiB in swap).
  Dropping the guest's cache alone returns nothing.
- `mvm destroy`:
  9 seconds,
  the guest's charge went to zero,
  and the swap device's free space rose by 7.12 GiB.

The charge is not all resident memory,
and what the resident part costs is the interesting half:
by the end,
6.99 GiB of the guest's memory had been written to the zram swap device,
where it compressed to 3 percent of its size
(`orig_data_size` 36.97 GB against `mem_used_total` 1.22 GiB),
because the pages the guest had freed and this build had left behind are mostly zeros.
So on this host the price of an idle default guest is about 1 GiB of RAM plus the zram share,
and the price of a *working* guest is up to its whole 8 GiB resident,
which is what the 22.8 GiB of shared memory in the opening reading was.

What that means for the question:
the host holds the guest's memory whether or not the guest uses it,
because nothing tells the host when a page stops mattering,
and the guest's own free memory (6.3 GiB of 7.75 GiB here) is invisible to it.
The next arms test what makes the host able to take it back.

### Free page reporting on, the same guest with one domain attribute

Guest `mem-b` was the same shape as `mem-a` with one attribute added:
`<memballoon model="virtio" freePageReporting="on"/>` in `devices`,
put there by the probe patch above.
The attribute reached the device:
the guest's QEMU command line carried
`{"driver":"virtio-balloon-pci","id":"balloon0","free-page-reporting":true,...}`,
read from `/proc/<pid>/cmdline` on the host.
Everything else about the guest,
the load and the preparation was the same,
and the guest's own view of its memory matched the default guest's at every step
(`Cached` 1,539,560 kB after preparation in both,
`MemFree` 6.2 to 6.3 GiB of 7.75 GiB at rest in both).

- After boot:
  0.78 GiB charged (the default guest:
  0.88 GiB).
  `mvm create` took 17 seconds in this run.
- After the same preparation:
  2.30 GiB charged against the default guest's 2.99 GiB,
  with identical guest-side readings.
  Reporting had already returned 0.69 GiB of the pages the preparation freed.
- During the build:
  8.20 GiB charged at the peak (the default guest:
  8.16 GiB),
  of which 6.85 GiB resident at its peak.
  A build's pages are live,
  so reporting has nothing to return while it runs.
  The build took 33 minutes 32 seconds.
- At rest after the build:
  5.52 GiB charged,
  0.20 GiB resident and 5.32 GiB in swap,
  against the default guest's 7.98 GiB.
- After the same `sync; echo 3 > /proc/sys/vm/drop_caches` in the guest:
  3.24 GiB charged,
  0.17 GiB resident and 3.07 GiB in swap.
  The drop returned **2.28 GiB** here,
  where in the default guest it returned nothing.
- A probe that allocated,
  touched and freed 3 GiB of the guest's memory raised the host's charge by
  0.1 GiB,
  and the 1.26 GiB of the guest's pages the host had already swapped out came back during it and were
  reported:
  the guest's swap charge fell by that much while the probe ran.
  That is the shape of the limit below.
- `mvm destroy`:
  8 seconds,
  and the charge went to zero.

What reporting returns:
the pages the guest frees **while they are still resident on the host**.
It returned 0.69 GiB of what the preparation freed,
2.28 GiB of what the guest's page cache held after the build,
and about 3 GiB of the 3 GiB a touching probe freed.
What it cannot return:
pages the host has already swapped out.
Those stayed charged as 3.07 GiB of swap after everything else was returned,
because a page that is not resident cannot be discarded;
they come back only when the guest touches them again,
and then the guest's next free reports them,
as the probe showed.
Under this host's pressure that residual is what is left of the option's saving,
and without pressure it would not exist.

What it costs:
nothing in the guest,
no change to what the guest sees,
and no new `mvm` command.
The one cost the sources imply and this round did not measure is a fault:
a reported page is gone from the host,
so the guest's next touch of that address allocates and zeroes a fresh page instead of reusing a
resident one.
The two builds took 34 minutes 13 seconds and 33 minutes 32 seconds,
under different host pressure,
so this round draws no speed conclusion from them.

### A guest without the share: private memory KSM could merge

Guest `mem-c` was `mvm`'s guest-agent route,
which is what a host without `virtiofsd` gets:
no `filesystem` device and no `memoryBacking`,
`meta.json` reading `"fileTransfer": "guest-agent"`.
QEMU then holds the guest's RAM as `memory-backend-ram` instead of the memfd.

What the host sees is different in kind:

- the guest's RAM mapping is `rw-p`,
  anonymous and private,
  where the shared guests' is `rw-s` on `/memfd:memory-backend-memfd`;
- it carries the mergeable flag:
  `VmFlags` included `mg` on that mapping,
  and 7 of the process's 816 mappings carried it
  (the memfd guests:
  6,
  never the RAM mapping);
- the session unit charged its bytes to `memory.stat` `anon` (2.68 GiB) rather than `shmem`.

That flag is the whole of what KSM needs,
and it is as far as this round could measure:
`/sys/kernel/mm/ksm/run` read 0 and `pages_sharing` 0 throughout,
and turning KSM on means writing a root-owned file in the host's kernel,
which this round does not do.
So what the option would return once two guests of one template have identical pages
is **not measured here**,
and the owner would have to turn KSM on before any round could measure it.

What was measured is the price and the shape,
at the preparation stage where all three guests have a
reading:

- 0.89 GiB charged 75 seconds after create,
  3.07 GiB after the same preparation the others ran (2.67 GiB resident,
  0.40 GiB swapped),
  and 3.08 GiB after the guest dropped its own page cache.
  The default guest was charged 2.99 GiB and the reporting guest 2.30 GiB at that same stage.
  Private memory is not free memory:
  it is the same bytes,
  accounted differently,
  and dropping the guest's cache returned nothing here either.
- `mvm push` of the 12.6 MB snapshot took 11 seconds through the guest agent
  where the same file took 3 seconds through virtiofs,
  and the guest has no `/mnt/shared` at all.
  The rounds' arms push scripts and the wrapper snapshot to `/mnt/shared` and read their logs from it
  (`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`,
  `guestStart` and `guestFinish`),
  so this route asks every one of them to change where their files live.
- `mvm create` took 13 seconds and `mvm destroy` 10 seconds.

### A smaller guest whose arms compile nothing

Guest `mem-d` was created with the probe patch's `memory4g,reporting`:
4,194,304 KiB instead of 8,192 MiB,
and free page reporting on.
Its QEMU command line read `-m size=4194304k` and
`"free-page-reporting":true`.
It ran no build and no preparation for one:
the guest's image already had Git 2.43.0,
and the arm ran the wrapper **the host built**:
`package/git-policy/cli/target/release/monochromatic-git-policy-cli`,
27,971,352 bytes,
built on this host at 2026-11-25T16:00,
pushed into the guest in 3 seconds (about 9 MB/s through virtiofs).

- `mvm create`:
  16 seconds.
- 68 seconds after create:
  0.68 GiB charged.
- The arm:
  the wrapper printed `monochromatic-git-policy-cli 0.6.1-alpha.44, shadowing git at /usr/bin/git`,
  the guest made a 200-file repository and a commit,
  the wrapper read its log,
  and a detached worktree was added and removed through the wrapper.
  It took 3 seconds and the charge peaked at **0.76 GiB**:
  0.56 GiB resident and 0.10 GiB in swap.
- A second pass had Git read and stage every file of the wrapper snapshot;
  it went as far as the staging step and stopped at a later one
  (Git's ownership check refused the root-extracted tree,
  and the round did not chase that,
  because the file reading was the load).
- `mvm destroy`:
  3 seconds.

So a 4 GiB guest that compiles nothing stood at **0.66 to 0.76 GiB**,
against 7.98 GiB for the default guest after one build.
What it costs is the whole point of the option:
the guest cannot compile the wrapper at all.
One in-guest build peaks at 7.45 GiB of a 7.75 GiB guest
(`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`),
so the build has to happen somewhere else once,
and the binary has to reach the guest (3 seconds for 28 MB here).
Every round whose subject *is* the in-guest build loses its arm.

### Destroying a guest between a round's arms

A guest returns everything when it is destroyed,
and this round measured both halves of that:

- `mvm destroy` took 9,
  8,
  10 and 3 seconds across the four guests,
  the charge went to zero each time,
  and for `mem-a` the swap device's free space rose by 7.12 GiB as its pages were released.
- `mvm create` from the baked template took 11,
  17,
  13 and 16 seconds.

What it costs is what the guest was keeping:
preparing it for a build took 74 to 100 seconds,
and one in-guest build took 34 minutes 13 seconds and 33 minutes 32 seconds in the two guests that ran one.
So destroying between arms is cheap only for arms that compile nothing
(the condition of the smaller guest above),
and it returns nothing at all *during* an arm:
the saving is proportional to the share of a round spent between arms,
and a round whose arms run back to back returns almost nothing.

## Ranking

Ranked by the memory each returns while rounds still get their guest arms,
with effort stated as a cost rather than a reason to rank lower.
The figures are the host's charge to one guest
(`smaps_rollup` `Rss` + `Swap` of its RAM mapping),
measured above;
"at rest" means after the load,
with the guest idle.

1. **A 4 GiB guest on a binary the host built,
   with reporting on**:
   0.66 to 0.76 GiB charged at every point measured,
   against 7.98 GiB for the default guest at rest.
   Cost:
   every arm must stop compiling in the guest,
   the wrapper has to be built once per commit
   somewhere else,
   and 28 MB has to reach each guest (3 seconds through virtiofs).
2. **Reporting on,
   and the guest dropping its page cache after an arm**:
   3.24 GiB charged at rest,
   of which 0.17 GiB resident,
   against the default guest's 7.98 GiB.
   The drop is what turns 2.46 GiB into 4.74 GiB returned:
   reporting alone got the guest to 5.52 GiB.
   Cost:
   one domain attribute,
   one command per arm,
   and the arm after it starts with a cold page cache.
3. **Reporting on alone**:
   5.52 GiB charged at rest,
   2.46 GiB returned,
   and 0.69 GiB returned during preparation alone.
   Cost:
   one domain attribute and nothing else;
   no round's arms change,
   no `mvm` command is added,
   and the guest's own readings are identical to the default guest's.
4. **Destroying a guest between a round's arms**:
   all 7.98 GiB returned,
   but only in the gaps:
   during an arm the guest holds everything it holds today,
   and each gap costs a create (11 to 17 seconds) plus the guest's preparation (74 to 100 seconds)
   and,
   for arms that compile,
   the whole build again (33 to 34 minutes here).
5. **A guest without the share,
   for KSM**:
   nothing measured and nothing returned yet.
   Its memory is private and carries the mergeable flag,
   but KSM was off and turning it on needs a write to a root-owned host file that no agent round may make.
   It also costs the file route:
   11 seconds for 12.6 MB where virtiofs took 3,
   and no `/mnt/shared` for the rounds' arm drivers.

Why each pair is ordered the way it is:

- 1 above 2:
  0.76 GiB against 3.24 GiB,
  a quarter of what reporting and the drop leave behind,
  and it is the only option that returns memory while an arm is *working*;
  reporting cannot touch a build's live pages,
  so its saving appears at rest and in the gaps.
- 2 above 3:
  3.24 GiB against 5.52 GiB,
  because the guest's page cache is most of what a build leaves behind and the guest has to be told
  to drop it before reporting can return it;
  the drop alone,
  in the default guest,
  returned nothing at all.
- 3 above 4:
  both cost little to add,
  but 4 returns nothing while an arm runs
  and it charges a create and a rebuild for every gap,
  while 3 lowers the standing charge of any guest that exists.
- 4 above 5:
  4 is measured and returns everything;
  5 returns nothing yet,
  and its own return depends on a host
  change outside any round's reach.

Options 1,
2 and 3 compose:
reporting on a 4 GiB guest that also drops its cache would be the smallest standing charge of all,
and the measurements here cover every pair except the drop on the 4 GiB guest.

## What is not measured, and is owed

- **What KSM would return.**
  Turning it on is a write to `/sys/kernel/mm/ksm/run`,
  owned by the host's root account.
  Until the owner turns it on,
  no round can measure the merging,
  and neither can a round measure it honestly with one guest at a time:
  the saving is between two guests of one template,
  and the round note allows one guest.
- **The 6 GiB middle.**
  This round measured 8 GiB and 4 GiB.
  A guest just large enough to build in (7.45 GiB peak) and just small enough to return memory was not
  measured.
- **What reporting costs the guest in time.**
  The two builds took 34 minutes 13 seconds (reporting off) and 33 minutes 32 seconds (reporting on)
  under different host pressure,
  so this round has no speed reading.
  Each reported page is a page the guest must fault back in,
  and the round did not measure that cost.
- **Windows guests.**
  `mvm`'s Windows guests install the VirtIO balloon driver
  (`package/mvm/cli/TROUBLESHOOTING.virtiofs-windows.md`),
  but this round measured Linux guests only.
- **Two guests and KSM at once.**
  The one-guest rule is why.

## Options considered and not measured

### Turning KSM on

KSM cannot be turned on in a round like this one.
`/sys/kernel/mm/ksm/run` is owned by the system's root account,
this round runs no command as root,
and the host read `0` before and during every arm here,
so no page on this machine was ever a merge candidate.
That leaves the guest-agent route's saving unmeasured:
what a private guest's RAM would give back once two guests of one template have identical pages merged
is the owner's call,
and it needs a host change outside any agent's reach.

### Ballooning the guest down from the host

`virsh setmem` (libvirt's balloon) would make the guest give pages back on the host's command.
It reaches the same discard path as free page reporting
(`balloon_inflate_page` calls `ram_block_discard_range`),
so it would work with the shared backing too,
but it needs the guest's own balloon driver to find the pages,
it leaves the inflated pages unusable to the guest until they are deflated,
and `mvm` exposes no command that would carry it:
a probe would have to run `virsh` itself,
which this round does not do.
Free page reporting reaches the same place without asking the guest to surrender pages it might use.

### Huge pages for the guest

`<memoryBacking><hugepages>` would make the guest's RAM 2 MiB pages.
QEMU's discard path handles huge pages with `fallocate` as well,
and the host would then have to hold a preallocated huge-page pool,
which is memory held for guests that no other process can use.
Not measured,
and the direction is away from the goal.

### Spreading the ARC instead of the guests

The ARC was already at its floor when this document's first reading was taken
(`size` 2,019,672,416 against `c_min` 2,069,377,152),
so there was nothing in it to give,
and the round note's own sum treats the ARC's floor as unavailable memory.
Not a lever for guest memory.

### Swapping guest memory out under pressure

The host already does this:
one of the four guests had 5.36 GiB of its RAM in the zram swap device.
It is not an option among the others because it is not free:
it costs zram's compression CPU and the guests' latency,
it happens only under pressure,
and it left 22.8 GiB held anyway.
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md` records what that costs this machine.

## What shipping either choice would change in mvm

Nothing here is done by this round.
Both changes are small and both are decisions for the owner.

- Free page reporting is one element in the domain XML:
written by `package/mvm/cli/src/domain-xml.ts` beside `sharedMemoryBacking()`.
If it were unconditional,
every new guest would have it and every existing guest would have it from its next create.
If it were a flag,
it would be a new option on `mvm create`,
a field in `meta.json`,
and a line in the package README's VM defaults.
- The smaller guest is a different memory constant,
and it is only a safe default together with the reporting above or with the build moved off the guest,
because one in-guest build of the wrapper peaks at 7.45 GiB of the guest's 7.75 GiB
(`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`).
The guest's memory is `DEFAULT_MEMORY_MIB` in `package/mvm/cli/src/config.ts`,
read by `domain-xml.ts`;
a per-guest value would need a flag and a `meta.json` field too.
