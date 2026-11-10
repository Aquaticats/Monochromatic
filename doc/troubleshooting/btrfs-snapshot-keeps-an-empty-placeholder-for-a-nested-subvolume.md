# btrfs (Linux 7.2) snapshots show a nested subvolume as an empty inode-2 folder that a send never carries, so a copy compared with its snapshot differs

## Symptom

The first-pass verification of the move to the 860
(`p4f-verify-first`,
`package/repository-move/cli/src/verify-pass.ts`)
stopped on 2026-11-08T10:54:19Z with:

```text
STOP: the copy s109 of /var/home/user/Monochromatic/.claude/worktrees/clear-stale-ref-locks differs from its snapshot (1 lines; the first 20 follow): cd+++++++++ .claude/worktrees/.cli-git-worktree-copy-844fe6ae-d350-49d2-950d-2b54480f7024/
```

`rsync --itemize-changes` says it would create that folder in the received copy.
The folder exists in the live worktree and in its read-only snapshot,
and not in the copy received from `btrfs send`.
In the live tree it is an empty directory with inode number 2 on a device number of its own,
and `btrfs subvolume show` says it is not a subvolume:

```text
inode=2 type=directory          (the folder)
dev=38 vs dev=1048596 ino=258   (the folder vs its parent)
ERROR: Not a Btrfs subvolume: Invalid argument
```

A census of the live checkout and the external worktrees on 2026-11-08
(`find /var/home/user/Monochromatic /var/home/user/worktrees -inum 2 -type d`)
found five such folders,
all named `.claude/worktrees/.cli-git-worktree-copy-<id>` inside a worktree.
The same search over the main checkout's snapshot found 267,
one for each nested worktree,
none of them stray.
An earlier run of the census inside a capped scope with standard error thrown away printed none;
the plain run found the five,
and a search limited to the one known folder found it,
so that empty result was a broken probe,
not an absence.

## Root cause

### btrfs makes the placeholder

A directory entry whose target is another subvolume is resolved by `fixup_tree_root_location`;
when the subvolume is not linked at that place
(the entry was copied into a snapshot,
and the snapshot does not contain the nested subvolume),
the lookup builds a stand-in directory instead,
[`fs/btrfs/inode.c:6087-6094` at v7.2](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/inode.c#L6087-L6094):

```c
	ret = fixup_tree_root_location(fs_info, BTRFS_I(dir), dentry,
				       &location, &sub_root);
	if (ret < 0) {
		if (ret != -ENOENT)
			inode = ERR_PTR(ret);
		else
			inode = new_simple_dir(dir, &location, root);
```

The stand-in has inode number `BTRFS_EMPTY_SUBVOL_DIR_OBJECTID` (2 in `linux/btrfs_tree.h`),
only a lookup operation,
mode 0755,
and its parent's owner,
[`fs/btrfs/inode.c:6014-6034` at v7.2](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/inode.c#L6014-L6034):

```c
	set_bit(BTRFS_INODE_ROOT_STUB, &inode->runtime_flags);
	set_bit(BTRFS_INODE_DUMMY, &inode->runtime_flags);

	btrfs_set_inode_number(inode, BTRFS_EMPTY_SUBVOL_DIR_OBJECTID);
	/*
	 * We only need lookup, the rest is read-only and there's no inode
	 * associated with the dentry
	 */
	vfs_inode->i_op = &simple_dir_inode_operations;
	vfs_inode->i_opflags &= ~IOP_XATTR;
	vfs_inode->i_fop = &simple_dir_operations;
	vfs_inode->i_mode = S_IFDIR | S_IRUGO | S_IWUSR | S_IXUGO;
	...
	vfs_inode->i_uid = dir->i_uid;
	vfs_inode->i_gid = dir->i_gid;
```

So the folder holds nothing and nothing can be created in it.

### A send does not carry it

The entry names a subvolume (`BTRFS_ROOT_ITEM_KEY`),
not an inode of the sent subvolume,
and send's lookup treats such an entry as absent,
[`fs/btrfs/send.c:1946-1948` at v7.2](https://github.com/torvalds/linux/blob/8d3ae59288f1e7d58d76558a6ee96d533bc5019f/fs/btrfs/send.c#L1946-L1948):

```c
	btrfs_dir_item_key_to_cpu(path->nodes[0], di, &key);
	if (key.type == BTRFS_ROOT_ITEM_KEY)
		return -ENOENT;
```

Measured:
the received copy `s109` has no entry at that path
(`ls` answers "No such file or directory").

### rsync lists it

The placeholder reports a device number other than its parent's,
so `rsync --one-file-system` treats it as a mount point and,
with the option given once,
copies it as an empty directory
(`man rsync`,
`--one-file-system`).
Against a copy without the folder that is the one `cd+++++++++` line.

### Why a worktree holds one in the live tree

A placeholder is normally only in a snapshot.
These five are in live worktrees because each worktree was itself made as a snapshot of the main checkout
by the Git wrapper's cheap `worktree add`
(`package/git-policy/cli/src/native/worktree/worktree_snapshot.rs`),
at a moment when another add was staging its own snapshot inside the main checkout.
That add claims a stage named `.cli-git-worktree-copy-<id>` beside its destination
(`package/git-policy/cli/src/native/worktree/worktree_ladder_run.rs:210-213`,
`package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim.rs:24`)
and takes its snapshot there,
so a snapshot of the main checkout taken meanwhile shows that stage as a placeholder.
After the snapshot,
the wrapper removes the placeholders of the registered worktrees and the destination's own folder only
(`worktree_snapshot.rs:237-243`):

```rust
fn remove_stubs(stage: &Path, source: &SnapshotSource, plan: &AddPlan) -> Result<(), String> {
    let root: File = subject_failure(open_root(stage), &"the stage", "cannot be opened")?;
    for nested in &source.nested {
        remove_stub(&root, source.root.as_path(), nested.as_path(), false)?;
    }
    return remove_stub(&root, source.root.as_path(), plan.destination.as_path(), true);
}
```

A stage of another add in flight is not registered,
so its placeholder stays in the new worktree.
This part is read from the wrapper's code and matches the five names.
A control reproduced it on 2026-11-08,
by making another add's stage the way an add does:
one snapshot ioctl at the stage name beside its destination,
and then making the worktree through the rung's own snapshot.
The new worktree lists that name at inode 2 on a device of its own and holds nothing,
and an `rmdir` of it removes the snapshot's entry alone:
the subvolume it stands for stays in the source,
with its files.

## Verification

Linux 7.2.15 (`7.2.15-ogc1.1.fc45.x86_64`),
btrfs on the NVMe home and on the 860;
kernel source read at tag `v7.2`
(commit `8d3ae59288f1e7d58d76558a6ee96d533bc5019f`).

Positive control on the real copy,
both sides read-only,
dry run:

```bash
sudo rsync --archive --dry-run --itemize-changes --delete --one-file-system --omit-dir-times \
  /var/home/user/repository-move-snapshots/s109/ /var/mnt/encrypted/repository/received-read-only/s109/
# cd+++++++++ .claude/worktrees/.cli-git-worktree-copy-844fe6ae-d350-49d2-950d-2b54480f7024/

sudo rsync --archive --dry-run --itemize-changes --delete --one-file-system --omit-dir-times \
  '--exclude=/.claude/worktrees/.cli-git-worktree-copy-844fe6ae-d350-49d2-950d-2b54480f7024/' \
  /var/home/user/repository-move-snapshots/s109/ /var/mnt/encrypted/repository/received-read-only/s109/
# (nothing)
```

Cost of finding the placeholders:
`find <snapshot>/ -xdev -inum 2 -type d` over the main checkout's snapshot took 7.4 s.

## Verified workarounds

The move tool excuses a placeholder by what the live tree holds at its path
(`224fc3bbd7`):

- The verification (`verify-pass.ts`) finds every inode-2 folder of the snapshot.
  One the inventory does not list is excluded and named in the output when the live path holds the same placeholder,
  or nothing;
  when the live path holds a real folder or subvolume,
  the pass stops and names it,
  since the inventory missed that subvolume and its files were not copied.
- The content compare in the cutover window (`c13b-rsync`) applies the same rule in shell.
- Tradeoff:
  the moved tree lacks these empty folders.
  They hold nothing and nothing can be stored in them,
  so no data is lost;
  a tool that expected the folder to exist would find it gone.

## What does not work

- Excusing every inode-2 folder without looking at the live tree:
  a placeholder in a snapshot can also stand for a real subvolume the inventory missed,
  whose files were not copied,
  and that must stop the move.
- Removing the five folders from the live worktrees:
  they belong to other sessions' worktrees,
  and the first-pass snapshots already hold them,
  so the first-pass verification would still differ.

## Upstream filing decision

No filing.

1.  Not btrfs's fault:
    the placeholder is how btrfs shows a nested subvolume that a snapshot does not contain,
    and a send carrying no subvolume it was not asked to send is correct.
    The defect is ours:
    the wrapper leaves a placeholder of another add's stage in a new worktree.
2.  Not applicable.
3.  Not applicable.
4.  Not applicable.
5.  Not applicable.
6.  Not applicable.

`.out-of-scope/` was not consulted,
since nothing is filed.

The wrapper removes them after a snapshot since `63dce988a0`
(`package/git-policy/cli/src/native/worktree/worktree_snapshot_placeholders.rs`):
one listing of each folder that holds a worktree,
the parent of each registered worktree and of the destination,
and every entry that is a directory with btrfs's stand-in inode number removed as an empty directory.
A stand-in whose subvolume no worktree names stays,
since the snapshot never carried its files and their absence must stay visible.
The five worktrees made before that commit keep theirs;
only the sessions that own them can remove those.
