# How mvm guests could hold less of the host's memory

**Non-final draft by an agent,
unapproved.**
It proposes;
it changes no code and no `mvm` default,
and the owner chooses.
Measurements are added arm by arm as they land,
and a section heading's stamp is the commit time of the commit that landed that section.

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
`MemAvailable` plus the ARC's `size` minus its `c_min` is at least 16 GiB and memory pressure's `avg10` is not above `avg60`.
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

## Ranking

(None yet:
it follows the measurements.)

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
