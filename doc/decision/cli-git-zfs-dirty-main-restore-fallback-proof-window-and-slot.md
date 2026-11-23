# cli-git ZFS dirty-main restore: the fallback, the proof's window, and what the slot holds while it is filled

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T19:11:12-05:00.
It answers three of the four questions that the gate review of the dirty-main restore design put to the project
(`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review-second-pass.md`,
section "Questions for the owner,
as I would put them",
questions Q1,
Q3 and Q4),
about the open item "Dirty-main restore on rung 1" of `doc/planning/cli-git-zfs-worktree-design-proposal.md`
and plan step 11 of `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.
The design it binds is still unapproved:
`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
on its own branch.

## What was decided

### Whether a failed add may finish with Git's own checkout by itself

Asked:
when an add from a main checkout with uncommitted edits fails part-way through no fault of the person
(another session saves a file while the add runs,
so the proof refuses;
or the snapshot request fails for lack of space),
may the add finish by itself with Git's own checkout,
given the accepted sentence
"No silent fallback over an unresolved managed state.
An explicitly recovered Git-only add may finish at the same slot only after the mount and all outstanding intents are
settled"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
line 430 onward).
The project chose "A:
always finish itself (Recommended)":
at every step,
the add destroys only what this add made and recorded
(the clone,
then its snapshot),
runs Git's own checkout at the same slot,
and says which route made the worktree and why the first was abandoned.

The costs the question stated with this option:
it puts a destroy inside an add,
it automates what the accepted sentence calls "explicitly recovered",
and it needs an unmount and a destroy that an add can call,
which no design offers yet.

This amends the accepted sentence for this route:
the fallback is not silent,
since the add reports it,
and it finishes only after the mount and the add's own intents are settled by the add itself.

### Which writes the frozen proof refuses

Asked:
the plan says the proof refuses "a tracked write between preflight and snapshot"
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
lines 1457 and 1615),
and the design instead records the main checkout's state immediately before the snapshot.
The project chose "A:
from just before snapshot (Recommended)":
the add records the state immediately before the snapshot,
runs every dirty-source refusal again on that record,
and refuses any write after it.
A write made before that record is part of what the snapshot holds and is restored away like any other uncommitted edit.

This amends the plan's sentence in both places.

### What the slot holds while the add fills it

Asked:
while the add fills the new worktree's folder,
the folder is a mounted copy of the main checkout with a full copy of its `.git` directory,
until the add puts the worktree's own pointer file in its place.
The project chose "A2:
keep copy,
swap at end (Recommended)":
the copied `.git` stays until the last step,
and the pointer and the copy then change places in one exchanging rename,
so that no moment and no kill leaves the folder without a `.git`.

The costs the question stated with this option:
a Git command run in the folder during the fill acts through the copy,
which can write files into the tree being restored
and which names the main repository's other worktree folders,
its remotes and its hooks;
and the exchanging rename is read from OpenZFS's source and has not been run.

Still owed:
a guest run of the exchange,
with what a kill before,
at and after it leaves.

## What was not chosen

- Refusing at every step and printing the remedy,
  and falling back only before the mount,
  for the fallback.
- Recording the state at both the first look and before the snapshot and refusing a difference,
  and applying the refusals at the first look only,
  for the proof's window.
- Replacing the copy in two steps,
  discarding the copy first,
  and making the copy inert before the fill,
  for what the slot holds.

## What this record does not decide

Whether rung 1 may decline a very dirty main checkout as rung 2 does
is the same review's question Q2,
which waits for the measurement that would size the cut.
