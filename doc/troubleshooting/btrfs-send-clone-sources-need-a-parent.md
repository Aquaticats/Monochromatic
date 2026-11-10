# `btrfs send` needs a parent for clone sources, and a parent from another subvolume can pair the wrong inodes

## Symptoms

The first real fixture run of the guided runner failed twice,
in order.

First,
the snapshot of a worktree sent with clone sources only:

```text
btrfs send -c .../s000 .../s001 | btrfs receive .../recv
At subvol .../s001
ERROR: parent determination failed for 260
ERROR: empty stream is not considered valid
```

Then,
with `-p` on the snapshot of the main checkout (the fix I wrote for the first failure,
now withdrawn):

```text
btrfs send -p .../snap/s000 .../snap/s001 | btrfs receive .../dest/recv
ERROR: link big.copy -> o257-10-0/worktrees failed: Operation not permitted
```

## Source trace, `-c` without `-p`

btrfs-progs v7.1 (the installed `btrfs --version`),
`cmds/send.c` at tag `v7.1`,
read on 2026-11-07,
and `btrfs-send(8)`,
which says "The `-p <parent>` option can be omitted when `-c <clone-src>` options are given,
in
which case btrfs send will determine a suitable parent from among the clone sources."

- With clone sources and no `-p`,
  `find_good_parent` runs for each subvolume.
  It takes the subvolume the snapshot was taken from (its `Parent UUID`).
- It succeeds only when that subvolume is itself a clone source,
  or when a clone source is another snapshot of the same subvolume.
  Otherwise `btrfs send` prints "parent determination failed".
- With `-p`,
  `send.c` skips `find_good_parent` and adds the parent to the clone sources.
- With no `-p` and no `-c` the send is full:
  no search.

In a first pass every snapshot is of a different subvolume,
so clone sources alone never work.

## Source trace, `-p` on a snapshot of another subvolume

Linux v7.2 `fs/btrfs/send.c` (the running kernel is 7.2.15),
`changed_inode`:

- Two inodes are the same inode only when their number and their generation (the creating transaction id,
  which is
  filesystem-wide) are equal in the sent root and the parent root.
  If the generations differ the kernel treats the inode as deleted and recreated.
  If both are equal it diffs them as one inode.
  Nothing checks that they have one origin.
- A snapshot `a` of `main` keeps allocating inode numbers from the same counter as `main`.
  Inodes created in both in one transaction get equal (number,
  generation) pairs.
- The measured case (coordinator,
  fixture `send-fixture.GXGz2I`):
  inode 313 in the snapshot of `main` is `.git/worktrees` (a directory),
  inode 313 in the snapshot of `a` is `big.copy`
  (a file),
  and the stream asked the receiver to hard-link the file to the directory,
  which `link(2)` refuses.
  A file paired with a file would not fail:
  it would send a diff against the wrong file.
- The kernel only needs the parent and each clone source to be read-only and not dead (`btrfs_ioctl_send`),
  so it accepts such a parent without complaint.

`-p` is safe only on the sent subvolume's own history:
another snapshot of the same live subvolume
(the first-pass and final snapshots of one subvolume),
or a parent that holds nothing to pair.
`find_good_parent` itself only accepts those relatives.

## Clone reach

`SEND_MAX_EXTENT_REFS` is 1024 in `fs/btrfs/send.c` (`check_extent_item`):
an extent with more references than that is written in full instead of cloned.
`clone_roots_cnt` is only bounded by `SZ_8M / sizeof(struct clone_root)` in the ioctl.
A first pass on the real repository must therefore be measured for extents near the cap
(the count of references of a shared extent,
for example `btrfs inspect-internal logical-resolve` of a sample).

## The pick: coordinator's, owner's veto open

2026-11-07,
after the ranking sent to the coordinator:

1.  Every first-pass send is `btrfs send -p <empty parent> -c <every earlier copy> <snapshot>`.
    The empty parent is one empty subvolume (`empty-parent-base`) snapshotted read-only as `empty-parent` and sent first.
    Comparing a snapshot with an empty root can only create inodes,
    so no (number,
    generation) pair is ever compared,
    and the kernel finds shared extents by their physical backreferences,
    so sharing with every earlier copy is kept.
    Measured by the coordinator on the fixture:
    `empty`,
    `s000 -p empty` and `s001 -p empty -c s000` were received and
    `rsync --checksum` found no difference for `s001`.
2.  The final pass sends `-p` the first-pass snapshot of the same subvolume (one history).
3.  Before any send the copy tool refuses a destination that holds two subvolumes with one received UUID
    (measured:
    `btrfs receive` resolved a clone source to an older copy with the same received UUID) or a subvolume in
    the received folder that no record knows.
4.  Every first-pass copy is verified against its snapshot before the cut-over (`--pass=verify-first`,
    `rsync --dry-run`
    by path,
    type,
    size,
    time and mode,
    and `--checksum` on request),
    excusing exactly the inventoried nested subvolumes,
    which a snapshot shows as empty folders and a send does not carry.
5.  Rejected:
    full sends with no clone sources (3833 GiB apparent against 263.6 GiB with sharing,
    so it does not fit),
    and a related subvolume as parent (can pair the wrong inodes).
    Not built:
    a root-run client for `BTRFS_IOC_SEND` with clone sources and no parent,
    which has the same safety
    as the empty parent and needs code the CLI makes unnecessary.

## Cost of a send on the empty parent

A full-tree walk per subvolume,
the same work as a full send,
plus clone commands for extents shared with an earlier copy.
An extent shared with no earlier copy,
or with more than 1024 references,
is written again on the destination.
