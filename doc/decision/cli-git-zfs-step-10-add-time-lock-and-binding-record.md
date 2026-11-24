# ZFS step 10: the add's record and the add-time Git lock are accepted

## Status

Accepted by the project in the question tool on 2026-11-23.
The designs are `doc/planning/cli-git-zfs-add-binding-record-design.md` (at `aa6a5f18f0` or its next commit)
and `doc/planning/cli-git-zfs-add-time-git-lock-design.md` (at `4625537ae5` or its next commit).
The question's text is the lock reviewer's eleventh read,
`~/temp/agent/gate-review-zfs-add-time-git-lock-and-binding-record-designs-2026-11-22/keep/review-eleventh-read-and-step-10-question.md`,
"The question,
as it would be put to the project".

## What was accepted

The project chose "Accept step 10 (Recommended)":
"Step 10's build starts this turn from the two designs."
That means step 10's builders follow these parts of the two documents:
the record's home,
claim,
fields,
two snapshot lists,
phase words and size bound;
the add's writes in one order,
from preflight to the dependency repairs;
the finder and its recipe,
which after a kill inside Git's own `worktree add` recognise what Git left and clear an incomplete registration in a fixed order;
the resume states;
the tests the designs list for the add,
the guest tests among them;
and,
from the lock design,
how Git is invoked so the registration is locked from its first moment,
the sentence the lock holds,
and how the wrapper proves a lock is its own.

Not accepted by this:
the add's self-finishing fallback (step 11),
the lost-lock rewrite (step 13),
removal and cleanup (steps 14 and 16),
the settling command (step 16),
and `doctor`'s lines (step 18).
Where the binding design hands step 10 more of the dirty-main restore than the accepted dirty-main design does,
the accepted design's allocation stands:
the dirty half of the restore is step 11's.

## The plan sentence

The project chose "Reword it (Recommended)",
with the reviewer's candidate words:
"...and the copied Git administration,
kept until the pointer's exchange and then left in the wrapper's trash for the disposal;
the pointer install as that exchange;
...".
The plan's step 10 text now reads so
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 10").
Not chosen:
"Leave it as it is".

## The 21 choices

The project chose "Keep all 21 (Recommended)":
"Every chosen option stands as listed in the preview."
The preview,
as shown:

```text
1  Records live in .git/monochromatic-git-policy-cli/worktree-clones/v1/
2  A record is named by the worktree's folder name
3  Two files per add: a claim and a record
4  The two snapshot lists are separate write-once files
5  The claim uses the wrapper's existing claim code
6  Every write of the record is forced to disk
7  A snapshot list carries its own count; no size cap
8  The wrapper recognises its lock by words recomputed
   from a random number in the record
9  A record line this release doesn't know: refuse
10 A clone's lost starting list: rebuild while the clone
   has no snapshot of its own, else refuse
11 Eight phase words, claimed .. settling
12 Only the add, the removal and doctor read records
13 A failed add keeps what it made; next add resumes
14 One parent folder for the three record designs
15 Records stay in .git, not an account folder
16 Forced writes only after facts nothing can rediscover
17 Lost-lock installer in step 13; until then step 10
   refuses and prints the sentence the lock must hold
18 NEW: an add refuses while a removal's plan names its
   clone, and names the removal that ends it
19 Git locks the registration in the same command
20 The lock's words: one plain line ending with the
   command a person runs
21 Lock words and comparison in step 10; rewrite and
   doctor lines in step 13
```

Not chosen:
"Veto some".
Each choice's full wording,
with what it did not choose and what a person sees,
is in the binding design's and the lock design's veto lists and in the question's "Part 3".

## Where the add's read of the plan folder is built

The project chose "Build it in step 10 (Recommended)":
"Gives:
no window at any release.
Cost:
the add carries a reader of a plan format you haven't accepted yet,
which may change."
Not chosen:
"Build it in step 14".
