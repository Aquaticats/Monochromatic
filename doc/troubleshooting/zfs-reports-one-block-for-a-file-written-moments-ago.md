# Zfs reports one block for a file written moments ago, until a transaction group commits

Status:
unapproved and non-final.
Written 2026-11-27 from one run's measurements on this host (issue 6552).
What it rests on:
a shell probe run here (quoted below),
the OpenZFS `zfs-2.4.4` source read at the lines cited,
and the measurement already recorded for issue 6468.
Nothing here is a claim about a pool this host does not have.

## Symptom

On this host's home pool,
`stat` of a file written in the same instant reports one 512-byte block,
whatever the file holds:

```sh
# /home/user/temp/agent/issue-6552-size-controls-hold-on-zfs-scratch/measure-blocks-2026-11-27
head -c 8192 /dev/zero | tr '\0' '\1' > eight-thousand-repeated-bytes
head -c 100000 /dev/urandom > hundred-thousand-random-bytes
mkdir --parents one-inner
head -c 8192 /dev/zero | tr '\0' '\1' > one-inner/file
stat -c 'size=%s blocks=%b name=%n' eight-thousand-repeated-bytes hundred-thousand-random-bytes one-inner one-inner/file
sleep 8
stat -c 'size=%s blocks=%b name=%n' eight-thousand-repeated-bytes hundred-thousand-random-bytes one-inner one-inner/file
```

```text
# at once
size=8192 blocks=1 name=eight-thousand-repeated-bytes
size=100000 blocks=1 name=hundred-thousand-random-bytes
size=3 blocks=1 name=one-inner
size=8192 blocks=1 name=one-inner/file
# eight seconds later
size=8192 blocks=9 name=eight-thousand-repeated-bytes
size=100000 blocks=217 name=hundred-thousand-random-bytes
size=3 blocks=17 name=one-inner
size=8192 blocks=9 name=one-inner/file
```

Two numbers move without any process touching the file,
and they move in different directions from what a length suggests:
`8192` bytes of one repeated byte settle at `9` blocks (`4608` bytes),
which is **less** than the file's length,
because the pool compresses it;
`100000` random bytes settle at `217` blocks (`111104` bytes),
which is more,
because blocks are allocated in whole records.
The length itself (`size=`) is right at both moments.

So a program that reads `st_blocks` of a file it wrote a moment earlier
decides on a number that is neither the length nor the disk usage of that file,
and the number changes on its own within seconds.

## Root cause

The pool's accounting is one variable,
`dn_used` of the object's **physical** dnode,
`include/sys/dnode.h:229` at tag `zfs-2.4.4`
([selection](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/include/sys/dnode.h#L222-L229)):

```c
/* include/sys/dnode.h */
	uint16_t dn_datablkszsec;	/* data block size in 512b sectors */
	...
	uint64_t dn_maxblkid;		/* largest allocated block ID */
	uint64_t dn_used;		/* bytes (or sectors) of disk space */
```

`st_blocks` is computed from it at the moment of the `stat`,
nowhere else.
`zfs_getattr_fast` fills the length from the inode first and then overwrites the block count
(`module/os/linux/zfs/zfs_vnops_os.c:1751` to `:1764` at the same tag,
[selection](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/os/linux/zfs/zfs_vnops_os.c#L1749-L1764)):

```c
/* module/os/linux/zfs/zfs_vnops_os.c */
#ifdef HAVE_GENERIC_FILLATTR_IDMAP_REQMASK
	zpl_generic_fillattr(user_ns, request_mask, ip, sp);
#else
	zpl_generic_fillattr(user_ns, ip, sp);
#endif
	...
	sa_object_size(zp->z_sa_hdl, &blksize, &nblocks);
	sp->blksize = blksize;
	sp->blocks = nblocks;
```

`sa_object_size` hands the dnode's accounting to `dmu_object_size_from_db`
(`module/zfs/sa.c:2179` to `:2183`,
[selection](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/sa.c#L2179-L2183)),
which turns it into 512-byte blocks
(`module/zfs/dmu.c:2873` to `:2887`,
[selection](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/dmu.c#L2873-L2887)):

```c
/* module/zfs/dmu.c */
	*blksize = dn->dn_datablksz;
	/* add in number of slots used for the dnode itself */
	*nblk512 = ((DN_USED_BYTES(dn->dn_phys) + SPA_MINBLOCKSIZE/2) >>
	    SPA_MINBLOCKSHIFT) + dn->dn_num_slots;
```

and `DN_USED_BYTES` reads that one member
(`include/sys/dnode.h:135` to `:136`,
[selection](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/include/sys/dnode.h#L135-L136)).

The member is written when a block's write completes,
not when the data is handed to the pool:
`dbuf_write_ready` (`module/zfs/dbuf.c:4983`,
[selection](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/dbuf.c#L4998-L5002))
and `dbuf_lightweight_ready` (`module/zfs/dbuf.c:4675`,
[selection](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/dbuf.c#L4687-L4697))
both call `dnode_diduse_space` (`module/zfs/dnode.c:2504` to `:2529`,
[selection](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/dnode.c#L2504-L2527))
with the difference the finished block makes,
and set `dn->dn_phys->dn_maxblkid` to the block they just wrote.

So between the write and the commit of the transaction group that holds the block,
`DN_USED_BYTES(dn_phys)` is zero,
and the count `stat` reports is `dn_num_slots` alone:
**one 512-byte block**,
which is the dnode's own slot and not the file's data.
That is the `blocks=1` above.
Once the group commits,
the member holds the allocated bytes,
and the file's length was right at both moments,
because `zpl_generic_fillattr` reads it from the inode.

What this record does not establish:
which of those two callbacks is always the last to run for one block,
and how long the wait is on a pool under other load.
What was measured is the pair of readings eight seconds apart on this pool
and that a commit is what moves it,
which issue 6468 measured as well
(`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
"the naming control also compared"):
seven unchanged 50-byte files reported
`1` block each and `9` blocks each a moment later,
moving `git count-objects`' `size` between `3` and `31` with no process touching them.

## Verification

- Version under test:
  OpenZFS `2.4.4-1`,
  from `cat /sys/module/zfs/version` on this host,
  2026-11-27.
- Filesystem:
  this host's `/home`,
  on the pool `zpcachyos`
  (`stat -f -c '%T %n' /home/user` answers `zfs`).
- Harness:
  the `sh` block above,
  which needs nothing but `head`,
  `tr`,
  `mkdir`,
  `sleep` and `stat`.
- Passes cleanly:
  a file written,
  then read after a transaction group has committed
  (`8` seconds here);
  its blocks then stay put across later reads.
- Fails:
  the same file read in the instant after the write,
  and a directory whose contents are fresh (`blocks=1` for a directory holding an 8192-byte file,
  `17` once settled,
  because the directory's own blocks are accounted the same way).

## Verified workarounds

- Count the length where the number is not about disk occupancy.
  In `package/cargo-shadow/cli` this is the seam `lean_sizes::ByteCount`:
  the tools pass `allocated_bytes_of` (the disk's own count),
  and a control that sizes a fixture it just wrote passes
  `lean_test_support::length_bytes` (`metadata.len()`),
  which is the same at both moments and on a file system that accounts blocks differently.
  Tradeoff:
  the number stops being a disk figure,
  so it is only right where the question is "how much did this fixture write",
  never for a cap or a report about freed space.
- Leave block-derived numbers out of a comparison,
  which is what issue 6468 did for `git count-objects`' `size`
  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`).
  Tradeoff:
  the comparison no longer covers what those numbers covered.

## What does not work

- Waiting for a sync.
  A `sync` in the same second did not settle the count in three measured rounds for issue 6468
  (recorded in the same doc),
  and even a settled count is not the length:
  the 8192-byte file of one repeated byte settles at `4608` bytes,
  below its own length,
  because the pool compresses it.
  So a control whose assertion is "at least the bytes I wrote" cannot be repaired by syncing,
  only by counting the length.
- Writing incompressible bytes instead.
  `100000` random bytes still report one block at once,
  so the count is still wrong until the group commits;
  randomness only makes the settled number larger than the length instead of smaller,
  which is a second way to be wrong on another pool setting.

## Upstream filing artifact

Nothing to add upstream.
The accounting is where the writes are:
`dn_used` is space the pool has allocated,
and a block that no transaction group has written yet has none,
which is what the code cited here does on purpose.
A file's length is available immediately and is what a caller who wants a stable number should read.
Filed as issue 6552 in this repository,
not against OpenZFS.
