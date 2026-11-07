# `btrfs send -c` without `-p` stops with "parent determination failed" for any snapshot that has no related clone source

## Symptom

The first real fixture run of the guided runner sent the snapshot of the main checkout, then the snapshot of a worktree:

```text
btrfs send -c .../s000 .../s001 | btrfs receive .../recv
At subvol .../s001
ERROR: parent determination failed for 260
ERROR: empty stream is not considered valid
```

## Source trace

btrfs-progs v7.1 (the installed `btrfs --version`), read from `cmds/send.c` at tag `v7.1` on 2026-11-07,
and `btrfs-send(8)`, which says "The `-p <parent>` option can be omitted when `-c <clone-src>` options are given, in
which case btrfs send will determine a suitable parent from among the clone sources."

- With clone sources and no `-p`, `send.c` calls `find_good_parent` for each subvolume.
  `get_parent` there returns the subvolume the snapshot was taken from (its `Parent UUID`).
- It succeeds only when that source subvolume is itself one of the clone sources,
  or when a clone source is another snapshot of the same source subvolume (the two snapshots have the same parent).
  Otherwise it returns an error and `btrfs send` prints "parent determination failed".
- With `-p`, `send.c` skips `find_good_parent` and adds the parent to the clone sources.
- With no `-p` and no `-c` the send is full: the whole tree, no search.

The kernel (`fs/btrfs/send.c` at tag `v7.1`, `btrfs_ioctl_send`) requires a parent and every clone source to be
read-only and not dead.
It reads no lineage: `btrfs_compare_trees(send_root, parent_root)` diffs any two read-only roots.
An unrelated parent was therefore not measured against a real send (a send needs root; this session had none), only read.

## What that means for a first pass

Every snapshot of a first pass is of a different subvolume, so no clone source shares its parent,
and clone sources alone never work.
A final pass works because the first-pass snapshot of the same subvolume has the same parent, and it passes `-p` anyway.

`package/repository-move/cli/src/send-plan.ts` therefore sends each subvolume as an increment on the copy of its nearest
ancestor by `Parent UUID` chain that the pass already sent (with the other copies as clone sources),
and in full, with neither `-p` nor `-c`, when no ancestor was sent.
The `Parent UUID` of each subvolume comes from `btrfs subvolume list -u -q`;
`btrfs subvolume show` is refused to users ("Could not search B-tree: Operation not permitted", measured).

## Cost of a full send

A full send carries every byte of the subvolume.
Extents a worktree shares with another subvolume by reflink are written again on the receiving side,
because only the clone sources of an incremental send let the receiver clone them.
A worktree that was made by copying the main checkout with reflinks, rather than by a snapshot of it,
has no `Parent UUID` link to it and is sent in full; the capacity rule of the plan's "Capacity policy" must leave room.
