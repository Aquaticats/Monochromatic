# The slot identity store's lock hold

**Status:
measurement record by an agent,
unapproved and non-final.**
It records what two shipped constants rest on,
`STORE_LOCK_WAIT` and `STORE_LOCK_PAUSE` in
`src/native/worktree_clone_mount/slot_identity_store.rs`,
and it is where the probe that produced them lives,
because a shipped limit cites a workload measurement or an owner decision beside the value
(rule `LQM` of `AGENTS.md`),
and a citation whose probe is not in the repository cannot be repeated.

## What is measured

The lock stands on a file beside the store and covers one write:
an append's read,
check and line,
or the adoption's read and whole-file rename.
The wait must therefore cover a read,
a write,
the file's `fsync`,
the folder's `fsync`
and,
for the adoption,
the rename that publishes the file.
The hold is that whole stretch,
timed from the moment the writer is called to the moment it returns,
because that is how long another writer waits for the lock.

## The probe

`a_store_write_s_hold_is_measured_for_the_lock_wait`,
an ignored test beside the module in
`src/native/worktree_clone_mount/slot_identity_store_tests.rs`,
drives the module's own writers (`append_slot_identity` and `record_parent_identity`),
so what it times is the shipped code and not a copy of its steps.

- Arms:
  an append on a store of 1000 slot records (40 samples),
  an append that lands exactly on `STORE_MAXIMUM_BYTES` (1,048,576 bytes,
  10 samples),
  and that store's adoption (10 samples).
- The widest append starts on a store one record below the bound,
  which is the widest write the store accepts:
  an append to a store of the whole bound is refused before anything is written
  (`StoreRefusal::WouldNotFit`),
  so timing it would time a refusal.
  The two runs below predate that refusal and appended to a store of the whole bound;
  their figure is the same write one record wider (about 250 bytes of 1 MiB),
  and the arms that matter for the wait are the two `fsync`s,
  not the last record.
- Each sample starts from the same store text,
  written just before the timed call,
  so no sample reads a store another sample grew.
- A record of the wide store carries an access ACL of 200 bytes,
  because a stored ACL is what makes one record wide.
  A store padded with comment lines alone measured 0.873 to 1.385 ms per append and
  0.620 to 0.920 ms per adoption,
  about twenty times less than the same bytes of records:
  the reader skips a comment without parsing a line,
  so padding measures nothing about a full store.
- `MONOCHROMATIC_STORE_HOLD_MEASUREMENT_ROOT` names the folder to measure on.
  Unset,
  the probe writes under the package's fixture root.

## Where it ran

Two runs of 2026-11-20 by this host's clock,
both before the kept log's last write at `2026-11-20T03:22:46-05:00`
(the log holds four runs:
a first one that could not reach its folder,
a comment-padded one,
and the two quoted here),
inside a container of the package's gate image
`localhost/git-policy-native-test:integrate-git-policy-cli-zfs-descriptor-attach-pre-merge`,
built at `2026-11-20T02:31:04-05:00`,
which is the image the previous branch's integration check left on this host
(the kept log does not record the image's name,
so the tag named is the one the run's command line used),
unoptimized `test` profile build,
with the measured folder bind-mounted from this host's ZFS `home` dataset.
The store itself lives at `SLOT_IDENTITY_STORE_PATH` on this host's `root` dataset,
the same pool and device:
the probe writes on `home` because the store at `/var/lib` is not a measurement's to write,
and both are `zpcachyos` datasets on `nvme1n1p2`.

The lines one run prints,
and the second run beside them:

```text
run 1
MEASUREMENT store-hold append-small-store bytes=40106 samples=40 min_ms=1.600 median_ms=1.647 max_ms=2.159
MEASUREMENT store-hold append-widest-store bytes=1048576 samples=10 min_ms=19.462 median_ms=19.602 max_ms=20.785
MEASUREMENT store-hold adoption-widest-store bytes=1048576 samples=10 min_ms=47.116 median_ms=50.008 max_ms=50.345
run 2
MEASUREMENT store-hold append-small-store bytes=40106 samples=40 min_ms=1.586 median_ms=1.985 max_ms=79.362
MEASUREMENT store-hold append-widest-store bytes=1048576 samples=10 min_ms=20.288 median_ms=22.656 max_ms=39.369
MEASUREMENT store-hold adoption-widest-store bytes=1048576 samples=10 min_ms=48.598 median_ms=50.727 max_ms=52.750
```

Read:

- The adoption of a full store is the arm that holds the lock longest,
  at 47.116 to 52.750 ms over both runs.
- The small store's append keeps a median near 1.6 to 2.0 ms,
  and one sample in the second run took 79.362 ms
  while that run's other samples stayed near 2 ms.
  No kept line measures what the host was doing then,
  so this record attributes the sample to nothing;
  it is the slowest hold either run saw,
  and `STORE_LOCK_WAIT` is cited against it,
  because a slow hold is still a hold whatever caused it.
- The width matters where the records do:
  the same 1 MiB of comment padding measured 20 times cheaper than the 1 MiB of records.

## The `fsync` reaches the pool's device

`probe-reaches-storage.py` in this folder is the time-matched control the pre-merge integration check wrote
(it was run from that check's own scratch folder,
and its review quoted the run);
it is kept here so the question can be asked again.
It stays Python because the run this record quotes was made with this file:
a TypeScript port would be a new tool whose output is not compared against that run,
and the counters it reads (`/proc/diskstats` and the pool's `kstat` objset files)
are read by nothing else in the repository.
It is run by hand and by no task.
The dataset comes from `findmnt` at run time;
the pool's partition and whole disk are options,
because a dataset's mount names the dataset and never its vdev,
and reading the vdev from ZFS opens `/dev/zfs`,
which this repository's rules for the host bar.
It appends the line the module writes and `fsync`s the file and the folder,
against three controls whose windows are as long as the `fsync` arm's own,
and reads the pool partition's written sectors,
the whole disk's completed flush requests
and the `home` dataset's own ZIL counters around every sample.

Its re-run on 2026-11-20 by this record's author,
60 rounds,
four arms interleaved:

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
On a host whose pool disk has a volatile write cache the flush would appear in that counter and add to the hold,
which is the case the shipped wait has to carry.

## The arithmetic the shipped constants rest on

- `STORE_LOCK_WAIT` is 2000 ms;
  2000 / 79.362 = 25.2,
  so the wait is 25 times the slowest hold either run saw.
- 2000 / 52.750 = 37.9 for the adoption arm alone,
  the arm a full store's rewrite takes.
- `STORE_LOCK_PAUSE` is 2 ms;
  2 / 79.362 = 0.025,
  about a fortieth of that slowest hold.
- The wait holds at most 2000 / 2 = 1000 asks.

The pre-merge round of 2026-11-20 measured the same three arms before this record
(1.577 to 2.243 ms,
31.657 to 33.926 ms and 33.483 to 35.976 ms),
on a store of 1,011,406 bytes rather than the whole bound,
and its numbers agree with this record's in magnitude.
The shipped constants stand on the slowest figure this record saw,
which is larger than any the round saw.

## What this does not show

- A release-profile build.
  The release build is faster,
  so the direction is safe.
- A busy pool whose `fsync` waits behind a transaction group sync for longer than any sample here,
  which the wait's tail has to carry and these 60 samples may not have seen.
- The dataset's `sync` property,
  the store's own dataset as a measurement target,
  or a device with a volatile write cache;
  reading the property needs `/dev/zfs`.
- The spread of a longer campaign:
  40 samples for the small arm and 10 for each wide arm,
  two runs.

## How to re-run it

```sh
# In this package, in a container of the gate image, with a folder on the file system to measure mounted, and the
# container's own account given the mounted folder's ownership (rootless podman maps container root to this user, and
# the image's own account cannot write a folder the bind mount shows as root's):
podman run --rm --user 0 --volume <folder>:/measurement:rw \
  --env MONOCHROMATIC_STORE_HOLD_MEASUREMENT_ROOT=/measurement <gate image> \
  sh -c 'cd /work/package/git-policy/cli && cargo nextest run --offline --locked --lib --run-ignored only --no-capture -E "test(a_store_write_s_hold_is_measured)"'
```

```sh
# The `fsync` control, inside a capped scope, on a folder of the same dataset the store's siblings live on.
python3 -I package/git-policy/cli/doc/slot-identity-store-lock-hold/probe-reaches-storage.py <folder> 60
```
