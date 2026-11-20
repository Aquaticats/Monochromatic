# Readings a ZFS arm cannot take the way its btrfs sibling does

What:
 Four readings the step 3 guest arm needed,
 and what OpenZFS 2.4.4 on the plan's `6.8.0-147-generic` guest answers instead.
Why:
 Each was found while writing
 `package/git-policy/cli/src/native/binary/binary_worktree_clone_zfs_tests.rs`
 and the guest recipe of
 `package/zfs/request-to-openzfs-kernel-device`,
 and each cost a probe to find;
 a later arm that takes a reading one of these describes should not learn it twice.

The run that measured all of them is the step 3 arm of 2026-11-20 in the throwaway guest of
`package/zfs/request-to-openzfs-kernel-device/e2e/README.md`,
on a file-backed `aes-256-gcm` pool with `recordsize=32K` and `compression=zstd-3`.

## `FIEMAP` answers with no physical extent

A btrfs clone control reads `FS_IOC_FIEMAP` and compares the two files' first physical
address (`binary_support_first_physical_extent.rs`).
On a file of a guest ZFS dataset the same reading has no answer:

```sh
$ filefrag -v /home/zfs-step-3-fixture/checkout/clone.bin
Filesystem type is: 2fc12fc1
File size of /home/zfs-step-3-fixture/checkout/clone.bin is 800000 (25 blocks of 32768 bytes)
/home/zfs-step-3-fixture/checkout/clone.bin: FIBMAP/FIEMAP unsupported
```

So a ZFS arm's proof that two files share blocks is the pool's own counters,
read through `zpool get`:

- `bcloneused`:
 the space the pool's cloned blocks take.
- `bclonesaved`:
 the bytes the pool would have used without those clones.
- `bcloneratio`:
 the pool's ratio of the two.

## The block-clone counters are per transaction group

A reading taken immediately after an arm reads the pool as it was:
`cp --reflink=always` of an 800,000-byte file left `bcloneused` and `bclonesaved` at 0
until the group was committed.
`zpool sync <pool>` commits it,
and then the counters show the clone.
`binary_support_zfs_controls.rs`'s `counters` syncs before every reading for that reason.

`bcloneused` is the **physical** space after compression (a same-dataset reflink of an
800,000-byte compressible file counted 499,200),
and `bclonesaved` is the logical size,
so a control compares each against the right quantity and never the file's size against
`bcloneused`.

Block cloning itself is one module parameter,
`/sys/module/zfs/parameters/zfs_bclone_enabled`,
not a dataset property:
`zfs create -o zfs_bclone_enabled=off` fails with
`invalid property 'zfs_bclone_enabled'`,
and the parameter is writable at runtime (1 by default in 2.4.4).
An arm that must see `FICLONE` fail turns that one switch off for its own length and puts
it back.

## A cross-dataset `copy_file_range` shared no block here

The design proposal's "Rung 3" says a successful `copy_file_range` can share blocks across
datasets that hold the same data key,
and the plan's step 3 asks for a same-key and a different-key arm on that premise.
In this guest it shared nothing in either case:

- `zfs_clone_range` in the guest's `module/zfs/zfs_vnops.c` allows a cross-dataset clone
  when both datasets are in one pool and share the master key,
 and returns `EXDEV`
  otherwise,
 so the kernel permits the same-key case.
- Measured:
 an 800,000-byte and a 4 MiB file copied from the checkout dataset into the
  second dataset (same encryption root) and into the different-key dataset,
  into a fresh destination and into one pre-sized to the source,
  with `bcloneused` unchanged after `zpool sync` in every case.
  A same-dataset `copy_file_range` of the same file counted the blocks,
  so the reading itself works.

So the arms assert the refusal (the pair must not qualify) and print what the copy did as a
`MEASUREMENT` line,
rather than asserting a sharing the guest does not show.
What in `zfs_clone_range` refuses it is not established here;
the candidates are the destination block-size rule and a source block still in the open
group,
and a later arm that needs the sharing should find which.

## `zfs` in 2.4.4 takes no long form of `destroy`'s recursive flag

`zfs destroy --recursive <dataset>` answers
`invalid option '-'` with this release's usage text,
so a script that prefers long flags must write `zfs destroy -r <dataset>` there.
`zfs mount`,
 `zfs clone`,
 `zfs snapshot` and `zpool get` do take their long options.
