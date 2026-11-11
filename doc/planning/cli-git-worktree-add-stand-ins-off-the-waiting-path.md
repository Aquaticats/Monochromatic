# Taking a worktree add's stand-ins off its waiting path

Status:
implemented on branch `fix/git-policy-cli-worktree-add-placeholders-off-the-waiting-path`;
`package/git-policy/cli/SPEC.md`,
"Leaving out of a snapshot",
and its README say what the code does.

The owner's words this answers:
"never discard user data",
"nothing should ever wait forever or indefinitely",
"Agents should never wait on each other on commits or merges.",
and,
for worktrees,
"60s is fine for adding worktrees".

## What was measured on the owner's machine

About 50 agents started at once,
and after 40 minutes only nine more of 47 `git worktree add` runs had finished.
A sample of the 34 that were still running
(`what-are-the-stuck-worktree-adds-waiting-on.ts` in `${HOME}/temp/agent/pi-runs`)
found 33 with a thread in state `D` in `wait_current_trans` inside `unlinkat` (syscall 263)
and one inside `mkdir`,
with 357 open descriptors on `<worktree>/.claude/worktrees` across the adds.

That folder is where the stand-ins are removed:
a btrfs snapshot shows an empty stand-in (inode 2) wherever its source holds a nested subvolume
(`doc/troubleshooting/btrfs-snapshot-keeps-an-empty-placeholder-for-a-nested-subvolume.md`),
and every add removed one of them for every other worktree of the repository
(`worktree/worktree_snapshot_placeholders.rs`,
`worktree/worktree_snapshot.rs` for the registered worktrees,
which went one by one as well).
Each removal is one btrfs metadata change,
and under that load each one waited for a transaction to commit.

The counts below confirm it on a fixture:
with no nested subvolume the add removed 23 directories,
and with 100 of them 123,
one more for each.

## The fixture and how the syscalls were counted

`${HOME}/temp/agent/wt-placeholder-measure/measure.ts`.
One disposable repository whose source is a btrfs subvolume,
with `.claude/worktrees` holding the nested subvolumes,
and the add's destination inside that folder,
as this repository makes its worktrees.
Every add runs under
`strace --follow-forks --trace unlinkat,rename,renameat,renameat2,mkdir,mkdirat,execve,execveat`.
Only the wrapper's own processes are counted:
the wrapper is the one process the trace shows an `execve` for,
and every process that exec'd anything else (real Git,
`systemctl`) is not the wrapper's work.

Both arms of each pair ran on the same binary shape and the same fixture,
alternating before and after,
on 2026-11-10 with the host's load as it was (load average about 105),
so the counts are the same whatever the load:
they are syscall counts,
not times.

## The counts

The wrapper's own calls in one add (`node summarize.ts` over the traces).
`rmdir done` counts the directory removals that changed the tree;
`rmdir refused` counts the ones the kernel answered with `ENOENT`,
which cost a lookup and no metadata change:

```text
before | source with no nested subvolume               | unlinkat   60 | rmdir done  23 | rmdir refused   0 | rename 11 | mkdir 17
after  | source with no nested subvolume               | unlinkat    3 | rmdir done   1 | rmdir refused   0 | rename 12 | mkdir 17
before | source with 100 nested subvolumes             | unlinkat  160 | rmdir done 123 | rmdir refused   0 | rename 11 | mkdir 17
after  | source with 100 nested subvolumes             | unlinkat    3 | rmdir done   1 | rmdir refused   0 | rename 13 | mkdir 18
before | source with 100 registered subvolume worktrees | unlinkat 1060 | rmdir done 423 | rmdir refused   0 | rename 11 | mkdir 17
after  | source with 100 registered subvolume worktrees | unlinkat  103 | rmdir done   1 | rmdir refused 100 | rename 13 | mkdir 18
```

What the three pairs say:

- The directory removals no longer grow with the worktrees the repository has:
  one is left in every arm (Git's own directory for the new worktree,
  which the swap settles),
  where 123 and 423 were.
- The two renames and one directory made that the fold costs are the only growth,
  and the fold and the `.git` copy are the two renames.
- The 100 refused calls in the last row are the per-worktree stub step of `remove_stub`:
  it looks for a name that the fresh folder no longer holds,
  so each costs a lookup and nothing else.
  They are left in place because a folder that keeps someone else's entry still needs that step.

## The snapshot's own `.git`

The largest term that grew with the number of worktrees was not a stand-in at all.
A snapshot of a main checkout carries the whole common Git directory,
with one administrative directory under
`.git/worktrees` for every worktree of the repository,
and `replace_git_pointer` used to remove that copy inside the add:
three directory removals and a handful of unlinks per worktree,
on the waiting path.
It is a copy,
its source is untouched,
and the new worktree needs nothing from it once the index is installed,
so it goes into the same trash behind one rename,
where the service deletes it after the add returns.
A source whose Git directory is a pointer file has nothing to take:
that one file is unlinked as before.

## What the add does now

A folder that holds nothing but stand-ins goes into the snapshot's trash behind one rename,
and a fresh empty folder with the mode of the one it replaced takes its place.
The snapshot's own `.git` copy goes into the same trash behind one more rename.
The stand-ins are then deleted after the add returns,
by the same commit service and in the same work item as the rest of the trash,
one trash at a time for the repository and at the lowest I/O priority
(`worktree_trash_hand_over.rs`).

The leave-out usually makes that trash already;
when it set nothing aside,
the stand-in step makes it,
so the move is available to every source whose Git ignores the trash place.

What keeps its old shape,
and why:

- A folder that holds one entry that is not such a stand-in stays where it is,
  and only the stand-ins in it are removed one by one.
  Every other entry may be someone's data,
  and the only other way to take the stand-ins off the waiting path would be to carry that entry through the
  trash and move it back,
  which puts someone's directory inside a folder a kill and the service both delete
  (`never discard user data`).
- A source whose Git does not ignore the trash place removes its stand-ins in place:
  a trash inside the new worktree that Git does not ignore can be staged,
  and one worktree's ignored state must never reach another worktree's index.
- The source root itself is never taken:
  it is the snapshot,
  and no fresh folder can take its place.

## Removing a tree that holds stand-ins

The service's emptying opens every directory below the trash entry on the entry's own device and refuses a
directory on another one.
A stand-in reports a device of its own,
yet it holds nothing and belongs to the tree being removed,
so `worktree_trash_remove.rs` allows it through,
as `subvolume_remove_walk.rs` already did for the same reason and with the same inode number.
A link,
a mount point (`openat2` with `RESOLVE_NO_XDEV`) and a nested subvolume stay errors.

## What was not measured

- The time a saving is worth.
  The counts above are syscalls;
  the time these adds take moves with the host's load far more than this change does,
  and no paired timing run was made.
- How long the service takes to empty a trash that holds a worktree folder of stand-ins.
- A repository whose worktree folder holds an entry that is not a stand-in,
  which keeps the old per-stand-in removal.
