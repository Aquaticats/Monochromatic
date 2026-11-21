# The slot identity store's write cost

**Status:
measurement record by an agent,
unapproved and non-final.**

It records what a publication into the store costs,
and two durable facts that outlived the lock this file once measured:

- the kernel's `fsync` of a record reaches the pool's device,
  and
- `probe-reaches-storage.py` is kept beside this file,
  so that question can be asked again.

The store takes no lock:
one file per slot is published with an exclusive creation and one link that replaces nothing,
so no write waits for another and no wait's constant has to cite a hold.
This record replaced the lock-hold record
(`slot-identity-store-lock-hold.md`,
renamed on 2026-11-21,
whose subject was the lock and its measured wait),
because rule `VGW` of `AGENTS.md` says a name must not lie about its content,
and rule `DL4` says a record goes only when its durable facts have a new home.

## What is measured

The publication of one slot's record is:

- a staging file created with `O_EXCL` under a dot name,
  mode `0644`,
- the header line and the record line written into it,
- the file's `fsync`,
- one `linkat` that gives the file the slot's name,
- the staging name unlinked,
- and the folder's `fsync`.

The adoption's replacement adds one rename over `.parent` and the same two `fsync` calls,
and it writes no slot file.

What a publication costs is therefore two `fsync` calls and three name operations,
and that cost is what this record measures:

- the pool's partition's written sectors,
  the whole disk's completed flush requests and the dataset's ZIL counters around the whole sequence,
- against controls whose windows are the same length,
- so a figure that says nothing about the store cannot pass for one that does.

## The probe

`probe-reaches-storage.py` in this folder is the time-matched control the pre-merge integration check wrote
(it was run from that check's own scratch folder,
and its review quoted the run);
it is kept here so the question can be asked again.
It stays Python because the run this record quotes for it was made with this file:
a TypeScript port would be a new tool whose output is not compared against that run,
and the counters it reads (`/proc/diskstats` and the pool's `kstat` objset files)
are read by nothing else in the repository.
It is run by hand and by no task,
and the guest driver's `run` step runs it too,
on a folder under `/var/lib`,
which is the dataset the store itself is on.

The dataset comes from `findmnt` at run time;
the pool's partition and whole disk are options,
because a dataset's mount names the dataset and never its vdev,
and reading the vdev from ZFS opens `/dev/zfs`,
which this repository's rules for the host bar.
Its arms,
interleaved over the same rounds:

- an append with the file's and the folder's `fsync`,
- the same append without `fsync`,
  in a window as short as that control's,
- the same append without `fsync` and then a wait as long as the `fsync` sample took,
- nothing but a wait of that length,
- and one publication of a per-slot record:
  the staging name created with `O_EXCL`,
  the record's bytes,
  `fchmod 0644`,
  the file's `fsync`,
  the `linkat` that names it,
  the staging name's `unlink`,
  and the folder's `fsync`.

## Where it ran

### The host's `fsync` control, 2026-11-20

Its re-run on 2026-11-20 by the earlier record's author,
60 rounds,
four arms interleaved,
on the `home` dataset of this host's `zpcachyos` pool:

```text
append with fsync of the file and the folder: n=60 ms 0.091 / 0.134 / 0.668 (min/median/max)
    any device moved 60, nvme1n1p2 moved 60, median nvme1n1p2 sectors 8.0,
    samples with a ZIL commit 60, samples whose ZIL wrote bytes to the pool 60, nvme1n1 flush requests completed 0 of 60
same append without fsync, short window:        n=60 ms 0.008 / 0.018 / 0.100;  nvme1n1p2 moved 0, ZIL commits 0
same append without fsync, then a wait:         n=60 ms 0.144 / 0.193 / 1.559;  nvme1n1p2 moved 0, ZIL commits 0
nothing but a wait as long as the fsync sample: n=60 ms 0.136 / 0.192 / 0.794;  nvme1n1p2 moved 0, ZIL commits 0
```

So each `fsync` sample moved the pool's partition by a median of 8 sectors (4096 bytes,
one 4 KiB log block)
and caused two ZIL commits (the file and the folder),
while three controls of the same window length moved the pool's partition not once.
The kernel's block statistics count 512-byte sectors
(`https://www.kernel.org/doc/html/latest/block/stat.html`).

What the probe does not show is a cache-flush command:
no flush request completed on `nvme1n1` in any sample.
That is what a device the kernel treats as write-through gives:
`/sys/block/nvme1n1/queue/write_cache` reads `write through` and `fua` reads `0`.
On a host whose pool disk has a volatile write cache the flush would appear in that counter.

### The store's own dataset, in the throwaway guest, 2026-11-21

GUEST-FSYNC-MEASUREMENT

## What the numbers say

The publication is two `fsync` calls and three name operations,
and its cost is the two `fsync` calls:
each one reaches the pool's device and commits a ZIL block,
which is what the host's control above shows for the append's two calls,
and the guest's run shows the same for a record's publication on the dataset the store lives on.

Nothing ships a limit that cites these numbers:
the store takes no lock,
no write waits for another,
and the only bound the publication enforces is `RECORD_MAXIMUM_BYTES`,
which is derived from the kernel's own limits on one extended attribute value and one file name
(`src/native/worktree_clone_mount/slot_identity_store_reading.rs`)
and not from a timing.

## What this does not show

- A release-profile build.
  The figures come from test binary runs and from the probe itself,
  and the direction the release build takes is unknown here.
- A busy pool whose `fsync` waits behind a transaction group sync for longer than any sample here.
- The dataset's `sync` property,
  or a device with a volatile write cache;
  reading the property needs `/dev/zfs`.
- The adoption's rename as its own arm:
  the probe's publication arm does not rename over an existing record,
  and the rename costs one name operation and its own folder `fsync`,
  which the arm already includes.

## How to re-run it

```sh
# The publication's cost, inside a capped scope, on a folder of the same dataset the store's siblings live on.
python3 -I package/git-policy/cli/doc/slot-identity-store-write-cost/probe-reaches-storage.py <folder> 60
```

```sh
# The same arm on the store's own dataset, which only the guest can write:
mise run //package/git-policy/cli:e2e:mount-git-worktree-clone-datasets -- all
# whose log directory holds store-fsync.log
```
