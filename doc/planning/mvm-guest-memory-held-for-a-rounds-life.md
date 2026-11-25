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
  and its `smaps_rollup` `Rss` read 7.59, 2.66, 5.06 and 7.91 GiB.
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

## Results

(None recorded yet.)

## Ranking

(None yet: it follows the measurements.)
