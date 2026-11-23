# cli-git ZFS removal over a marked origin: what each refusal leaves, and a look before the unmount

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T19:15:35-05:00.
It answers three questions that the gate review of the deferred-destroy checks design put to the project
(`~/temp/agent/gate-review-zfs-deferred-destroy-checks-design-2026-11-22/keep/review-diff-check.md`,
section "Questions as they now stand",
questions QA-1,
QA-2 and Q1),
about the open item "The removal's deferred-destroy checks" of `doc/planning/cli-git-zfs-worktree-design-proposal.md`
and plan step 16 of `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.
The design it binds is still unapproved:
`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
on its own branch.

An origin snapshot that somebody marked with `zfs destroy -d` is destroyed by ZFS itself once its last clone is gone.
The project had already decided that a removal checks for that mark twice and refuses.

## What was decided

### What a refusal at the first check leaves

Asked:
the first check runs after the clone is unmounted and before anything is deleted.
The project chose "A:
stay pending (Recommended)":
the removal is kept pending,
the clone stays unmounted,
and the worktree is out of use until a person deals with the mark and runs the removal again,
or runs the cancel command to get the worktree back.
The removal does not remount the clone by itself,
although it does so for a holder its scan sees:
a holder goes away by itself and a mark does not.

### A look before the unmount

Asked:
should the removal also look for the mark before it unmounts anything,
inside the lineage proof it makes anyway.
The project chose "Yes,
look before unmounting (Recommended)":
a marked origin is refused there with the worktree still mounted and nothing written.
The state of a refusal at the first decided check then arises only when the mark appears between that look
and the check.

This adds a third refusal point to the two the project decided earlier.
That the look costs no extra read is an inference until the step that makes the lineage proof is built.

### What a refusal at the second check leaves

Asked:
the second check runs after the clone's own snapshots are destroyed and before the clone is,
so a refusal there finds part of the clone's history already gone.
The project chose "A:
stay pending,
keep records (Recommended)":
the removal is kept pending with its record of which snapshots were destroyed,
the clone stays unmounted,
and the cancel command is the way back.

## What was not chosen

- Cancelling and remounting,
  at the first check.
- Keeping exactly the two decided checks.
- Cancelling and remounting with the record kept,
  and cancelling with the record dropped,
  at the second check.

## What this record does not decide

Whether the wrapper may ever finish a removal over a marked origin was not asked:
the decided refusal stands.
