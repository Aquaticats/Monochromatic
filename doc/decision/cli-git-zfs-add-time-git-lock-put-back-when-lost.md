# cli-git ZFS add-time Git lock: a lost lock is written again by the wrapper

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T20:21:19-05:00.
It answers the first question of the design
`doc/planning/cli-git-zfs-add-time-git-lock-design.md`
(on its own branch,
still unapproved),
as its gate review put it
(`~/temp/agent/gate-review-zfs-add-time-git-lock-and-binding-record-designs-2026-11-22/keep/review-diff-check.md`,
section "The questions for the owner",
Q1).
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
It decides what plan step 13 builds;
plan step 10 does not wait for it.

When the wrapper makes a ZFS clone worktree it writes Git's own worktree lock,
the one `git worktree lock` writes,
so that `git worktree prune` and a removal by plain Git cannot delete the registration of a mounted clone.
A person can take that lock off with `git worktree unlock`,
and a kill can leave it missing.

## What was decided

### The wrapper writes a lost lock again and says so

The project chose "A:
put it back,
say so (Recommended)":
any wrapped command that already holds that worktree's slot and has proved its registration writes the lock again
and prints one line saying it did.

The cost the question stated with this option:
it undoes a deliberate `git worktree unlock` without asking.

### What a removal does when the lock is already gone

The review's second question,
whether a removal stops when it finds the lock gone,
was to be asked only under another answer.
Under this one the removal is such a command:
it writes the lock again,
says so,
and then releases it as every removal does.

## What was not chosen

- Writing the lock again only through a new repair command,
  with `doctor` naming it and a removal refusing until it is run.
- Never writing it again,
  reporting only.

## What this record does not decide

What a reader does with a record a later release cannot read,
and what happens to a slot whose recorded clone is gone with its absence unprovable,
are the binding record design's questions and are asked together with the removal-side record designs' questions.
