# What a worktree add pays for the directories its snapshot leaves out

Status:
the coordinator picked the route below under the owner's "Please make any non-destructive options
(when they are the ones I would likely have picked) yourself from now on for this session.",
and the owner may veto any part of it.
Implemented on branch `fix/git-policy-cli-worktree-add-does-not-wait-for-its-trash`;
`package/git-policy/cli/SPEC.md`,
"Leaving out of a snapshot",
and its README say what the code does.

The owner's words this route answers:
"never discard user data",
"nothing should ever wait forever or indefinitely",
"Use as few locks as possible.",
"Agents should never wait on each other on commits or merges.",
and, for worktrees,
"60s is fine for adding worktrees".

## What was measured on 2026-11-10

Ten `git worktree add` runs took more than ten minutes each on this machine on 2026-11-10.
The samples are `${HOME}/temp/agent/pi-runs/why-worktree-adds-are-slow-sample-1.txt` and
`why-worktree-adds-are-slow-sample-2.txt`,
made by `sample-why-worktree-adds-are-slow.ts` beside them.

Across every thread of those adds over 92 s,
the second sample counts 54 samples on a CPU and 2,796 blocked,
of which 2,182 are inside `unlink` or `rmdir`:

- `btrfs_unlink` under `filename_unlinkat`, waiting for a transaction: 981 samples,
- the same `unlink` waiting to reserve metadata: 555,
- `btrfs_rmdir` under `filename_rmdir`: 160,
- the same `rmdir` waiting to reserve metadata: 143.

The folders the processes held open were each add's own `.claude/cli-git-trash/<id>`,
the drive was 99% busy,
and the first sample counts 12,638 B-tree operations from `unlink` and `rmdir` over its window.
Nine adds ran at once in that window and each of them wrote about 270 MB.

Why the trash exists at all:
a snapshot of the main checkout carries everything,
so the leave-out sets the caches,
the other checkouts and the run records aside into that trash by rename
(`src/native/worktree/worktree_trash.rs`),
which costs a tenth of a millisecond per directory.
Emptying it is what costs minutes,
and every add emptied its own trash before it returned.

## Measured on 2026-11-11: the change, on this volume

TBD-ADD-MEASUREMENT

## The four steps the owner set

The route is four steps,
and each one is a fact about where the deletions are:

1. The add returns once the worktree is in place,
   without waiting for the trash to be emptied.
2. A standing trash cannot be staged whatever the target commit ignores.
3. The emptying runs off every command's waiting path,
   one trash at a time across the repository,
   at the lowest I/O priority Linux offers.
4. A trash left by a killed add or a stopped service is still emptied later,
   is never lost track of,
   and is still reported when it cannot be emptied,
   and a later add never carries a trash into its own copy or snapshot.

## A standing trash cannot be staged

The repository's own `info/exclude` holds one line,
`/.claude/cli-git-trash/`,
written before the leave-out asks Git whether that place is ignored
(`src/native/worktree/worktree_trash_exclude_line.rs`).
No commit writes that file,
so no target commit can decide whether another command's state is stageable.

Measured with `/usr/bin/git` 2.56.0 on 2026-11-11,
in a throwaway repository with a linked worktree
(`${HOME}/temp/agent/trash-exclude-probe/probe.ts` and `probe2.ts`):

- Before any line,
  `git check-ignore --no-index` in the linked worktree names nothing,
  and `git status --porcelain` says `?? .claude/`.
- With the line in the **common** `info/exclude`,
  `check-ignore` names that same file and the paths below it,
  `git status --porcelain` says nothing,
  and `git add --all` stages nothing of a trash standing there.
- With the line only in `$GIT_DIR/worktrees/<name>/info/exclude`,
  nothing is ignored and the trash is untracked:
  a linked worktree reads the common directory's file and never its own.
- A target commit whose `.gitignore` ignores nothing under `.claude/`,
  and one that writes `.claude/` then `!.claude/cli-git-trash/`,
  both leave the trash out of `git add --all`.
  Only the `.gitignore` itself is staged.

The controls that hold this are
`real_git_keeps_a_standing_trash_out_of_the_index_of_a_linked_worktree` and
`a_target_commit_that_stops_ignoring_the_trash_place_cannot_stage_it`.

## One trash at a time, and no new lock

The record a trash is handed over with is written into the repository's **common** work directory,
`<common>/monochromatic-git-policy-cli/work/pending/trash-<name>`,
and the service is woken for that common directory.
A service round drains one lane per repository
(`src/native/landing/service_round.rs`),
and the drain of one work directory empties its trashes one after another
(`src/native/landing/service_work_drain_pending_claims.rs`).
So concurrent adds enqueue beside each other and never delete beside each other,
whichever worktree holds each trash.

No lock is added for this:
the claim the drain takes is the lease every other kind of work already uses,
and a record whose holder died is taken back when its lease runs out.
The emptying itself is the face that already existed for a work item of an earlier release,
`empty_trash` (`src/native/worktree/worktree_trash_dispose.rs`),
with the same identity checks:
the worktree's `.git` must point into the record's Git directory or one of its `worktrees` folders,
the trash must be the object the leave-out made,
and each entry removed must be the object moved there.

## The lowest I/O priority Linux offers

Each removal of a trash entry,
and the removal of the trash directory itself,
runs inside I/O priority class `idle`
(`src/native/worktree/idle_io_priority.rs`,
one `ioprio_set` before the work and one after it to put the thread back).
The class is set on the thread that unlinks,
because the service pushes a branch and reads the repository on its other threads while a trash is emptied.
A thread whose class cannot be read or set is left as it is,
and the emptying is correct either way.

## What the trash divides up into

TBD-ADD-LEAVE-OUT-COUNTS

## What would disappear if the largest folder were a subvolume

TBD-ADD-SUBVOLUME-ANSWER

## What the add now takes

TBD-ADD-WHAT-CHANGED

## What is left for the owner

TBD-ADD-OWNER
