# btrfs (Linux 7.2) refuses to destroy a subvolume that holds another one, and an unprivileged user cannot list a subvolume's children, so a worktree that holds a linked worktree could only be walked

## Symptom

`git worktree remove <worktree>` through this repository's wrapper,
for a worktree that holds a linked worktree of its own
(`git worktree add <worktree>/.claude/worktrees/inner`),
printed:

```text
monochromatic-git-policy-cli: removed the btrfs subvolume of the worktree in one step, and the commit service deletes the
files after this command returns (normally within a second); this cannot be undone, and anything another process wrote or
locked in the moments since the last check went with it (the risk of git worktree remove, here faster).
```

and exited 0,
while the worktree stayed under the service's own name with every file of it:

```text
$ ls .claude/worktrees
cli-git-deleting-000000000001bc25
$ git worktree list
.../repo                                                         b3c33cc [main]
.../repo/.claude/worktrees/W/.claude/worktrees/B                  b3c33cc [sub] prunable
$ find .claude -name notes.txt
.../cli-git-deleting-000000000001bc25/.claude/worktrees/B/notes.txt
```

The commit service's log held the only word about it:

```text
monochromatic-git-policy-cli-service: the subvolume .../cli-git-deleting-000000000001bc25 could not be deleted: the
subvolume of .../cli-git-deleting-000000000001bc25 cannot be destroyed (BTRFS_IOC_SNAP_DESTROY on ...
```

The commands that touch a directory rather than the ioctl answer the same way:

```text
$ btrfs subvolume delete /var/home/user/temp/agent/sonnet-findings-removal/probe/parent
ERROR: Could not destroy subvolume/snapshot: Directory not empty
Delete subvolume 114575 (no-commit): '.../probe/parent'
exit=1
$ rmdir .../probe/parent
rmdir: failed to remove '.../probe/parent': Directory not empty
```

`rm --recursive` is not a way out,
and it is worse than the ioctl:

```text
$ rm -rf .../probe/parent
exit=0
$ ls .../probe/parent/child
ls: cannot access '.../probe/parent/child': No such file or directory
```

It descends into the inner subvolume,
unlinks the files there,
and removes each root with `rmdir`,
which this mount allows (`user_subvol_rm_allowed`),
so the files of the inner worktree go with it.

## Root cause

### The kernel refuses before it deletes anything

`BTRFS_IOC_SNAP_DESTROY` reaches `btrfs_ioctl_snap_destroy`,
which calls `btrfs_delete_subvolume`
([`fs/btrfs/ioctl.c:2415` at 8d3ae592](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/ioctl.c#L2415)),
and that marks the root dead and then asks `may_destroy_subvol` whether it may go
([`fs/btrfs/inode.c:4801` at 8d3ae592](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/inode.c#L4801)):

```c
	root_flags = btrfs_root_flags(&dest->root_item);
	btrfs_set_root_flags(&dest->root_item,
			     root_flags | BTRFS_ROOT_SUBVOL_DEAD);
	spin_unlock(&dest->root_item_lock);

	ret = may_destroy_subvol(dest);
	if (ret)
		goto out_undead;
```

`may_destroy_subvol` is documented as "Helper to check if the subvolume references other subvolumes or if it's default"
([`fs/btrfs/inode.c:4673-4678`](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/inode.c#L4673-L4678)),
and it answers `ENOTEMPTY` as soon as the root tree holds one `ROOT_REF` whose object id is this subvolume,
which is every subvolume inside it,
a nested one included
([`fs/btrfs/inode.c:4722-4731`](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/inode.c#L4722-L4731)):

```c
	ret = 0;
	if (path->slots[0] > 0) {
		path->slots[0]--;
		btrfs_item_key_to_cpu(path->nodes[0], &key, path->slots[0]);
		if (key.objectid == btrfs_root_id(root) && key.type == BTRFS_ROOT_REF_KEY)
			ret = -ENOTEMPTY;
	}
```

Nothing is deleted when it answers that way,
which is why the renamed directory kept every file.

`rmdir(2)` cannot help either:
`btrfs_rmdir` returns `ENOTEMPTY` for a directory that still holds entries,
and hands a subvolume root to
`btrfs_delete_subvolume`,
which makes the same check
([`fs/btrfs/inode.c:4902-4919`](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/inode.c#L4902-L4919)):

```c
	if (inode->vfs_inode.i_size > BTRFS_EMPTY_DIR_SIZE)
		return -ENOTEMPTY;
	if (btrfs_ino(inode) == BTRFS_FIRST_FREE_OBJECTID) {
		...
		return btrfs_delete_subvolume(dir, dentry);
	}
```

A mount without `user_subvol_rm_allowed` answers `EPERM` before any of this
([`fs/btrfs/ioctl.c:2384-2385`](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/ioctl.c#L2384-L2385)).

### No unprivileged call lists a subvolume's children

Asking the filesystem instead of walking it needs a call a person's own command may not make.
The listing `btrfs subvolume list` starts with is `BTRFS_IOC_TREE_SEARCH`,
and that one requires `CAP_SYS_ADMIN`
([`fs/btrfs/ioctl.c:1604-1605`](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/ioctl.c#L1604-L1605)):

```c
	if (!capable(CAP_SYS_ADMIN))
		return -EPERM;
```

so an unprivileged `btrfs subvolume list -o <subvolume>` fails before it looks at anything:

```text
$ id -u
1000
$ btrfs subvolume list -o .../probe2/parent
ERROR: can't perform the search: Operation not permitted
exit=1
```

`BTRFS_IOC_GET_SUBVOL_ROOTREF` is the one call with no capability check,
and it walks the root tree by this subvolume's own object id
([`fs/btrfs/ioctl.c:2154-2160`](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/ioctl.c#L2154-L2160)),
but it answers child tree ids and directory ids and never a name,
so every child it reports still needs `BTRFS_IOC_INO_LOOKUP` to become a path.

### What the wrapper does about it

`package/git-policy/cli` walks the directories below the worktree before it renames anything,
with the same walk its registration emptying makes
(`src/native/subvolume/subvolume_remove_walk.rs`),
and refuses the removal with exit status 1 when a directory reports a device of its own,
naming the linked worktree and the command that removes it while keeping its branch
(`src/native/worktree/worktree_removal_refuses_a_nested_volume.rs`).
The walk costs one `openat2` and two `getdents64` per directory:
33,075 and 66,189 of them for a worktree of 33,061 directories,
0.45 s of syscall time at a load average of 50,
against the 5,413 ms median btrfs transaction wait the destroy
route already pays (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).

## Verification

Host under test:
`Linux 7.2.15-ogc1.1.fc45.x86_64`,
`btrfs-progs v7.1`,
on a btrfs volume mounted
`rw,noatime,seclabel,compress=zstd:3,ssd,discard=async,space_cache=v2,user_subvol_rm_allowed,subvolid=5,subvol=/`,
fixtures under `/var/home/user/temp/agent/sonnet-findings-removal/`.

The kernel citations are line numbers of the files as they are at commit `8d3ae59288f1e7d58d76558a6ee96d533bc5019f`,
read in full from that commit through the GitHub contents API on 2026-11-11
(`gh api -H 'Accept: application/vnd.github.raw' 'repos/torvalds/linux/contents/fs/btrfs/inode.c?ref=8d3ae592...'`),
so each permalink addresses the lines quoted above.
Not run:
loading the permalinks in `agent-browser`,
which this run could not do
(`✗ Read request failed: ... tcp connect error: Software caused connection abort (os error 103)`),
although the same host reached GitHub through `gh` in the same session.

The harness,
run as uid 1000:

```sh
# probe: what an unprivileged user can ask btrfs about a subvolume's children
P=/var/home/user/temp/agent/sonnet-findings-removal/probe2
rm -rf "$P"; mkdir -p "$P"
btrfs subvolume create "$P/parent" >/dev/null
mkdir -p "$P/parent/keep"
btrfs subvolume create "$P/parent/child" >/dev/null
btrfs subvolume list -o "$P/parent"        # fails: "can't perform the search: Operation not permitted"
btrfs subvolume delete "$P/parent"         # fails: "Directory not empty"
rmdir "$P/parent"                          # fails: "Directory not empty"
rm -rf "$P/parent"                         # succeeds, and takes the child's files with it
```

The catalogs:

- Answers cleanly:
  destroying the inner subvolume alone (`btrfs subvolume delete .../child`),
  and then the parent (`btrfs subvolume delete .../parent`),
  both exit 0 on this mount.
- Fails:
  destroying the parent while the child is there,
  by ioctl (`Directory not empty`),
  by `rmdir` (`Directory not empty`),
  and as the wrapper's own command did before it learned to refuse,
  which reported success and left the whole tree.
- Fails differently:
  the same destroy on a mount without `user_subvol_rm_allowed` answers `EPERM` instead,
  which no probe here could make,
  because this volume carries the option.

## Verified workarounds

- Refuse the one-step removal while the worktree holds a subvolume of its own,
  and name `git worktree remove <inner>` as the way forward
  (`src/native/worktree/worktree_removal_refuses_a_nested_volume.rs`).
  Tradeoff:
  every destroy-route removal pays a walk of the worktree before it starts,
  measured above,
  and a tree the walk cannot read is left to Git,
  whose removal deletes the files inside the nested subvolume.
- Delete or move the inner subvolume first,
  then remove the outer one:
  both commands above exit 0 in that order.
  Tradeoff:
  nothing of the inner tree may be needed any more,
  and a `git worktree remove` of it keeps its branch and refuses while anything there is uncommitted.

## What does not work

- `btrfs subvolume list -o <subvolume>` as an unprivileged user:
  `EPERM` before the search starts,
  for `-o` and for the
  plain form alike.
- `rmdir` of the parent:
  `ENOTEMPTY`,
  and on a mount without `user_subvol_rm_allowed` it would be `EPERM` at the child.
- `rm --recursive` of the parent:
  exits 0 only because this mount lets an unprivileged `rmdir` of an empty subvolume
  through,
  and it unlinks the files of the inner subvolume before it gets there.
- Deciding it from the parent's stat data:
  a subvolume's children are recorded in the root tree,
  not in the parent's inode,
  so `st_dev`,
  `st_size` and `st_nlink` say nothing about them.

## Upstream filing decision

Nothing is filed,
and nothing is drafted:
the refusal is the kernel's design and this repository's command is the thing that was wrong.

1. Is it upstream's fault?
   No. `may_destroy_subvol` exists so that a parent subvolume cannot be removed while a child's own root would be left
   unreachable,
   which is the behavior this repository wants.
2. Can upstream fix it?
   No,
   not without undoing that guarantee,
   and it is not asked to.
3. Does upstream support the use case?
   The use case is a person's nested worktree,
   not a kernel call,
   and the kernel's own answer (`ENOTEMPTY`) is the correct
   one for it.
4. Would upstream welcome a contribution?
   Not applicable,
   because there is nothing to contribute.
5. Would upstream fix it?
   Not applicable.
6. A minimal compatible fix:
   Not applicable.

The one thing this repository could do better is the enumeration:
`BTRFS_IOC_GET_SUBVOL_ROOTREF` would turn the walk into one call per subvolume,
and it is recorded here for the next session rather than built,
because it answers tree ids and directory ids and needs `BTRFS_IOC_INO_LOOKUP` per child to name a path.
