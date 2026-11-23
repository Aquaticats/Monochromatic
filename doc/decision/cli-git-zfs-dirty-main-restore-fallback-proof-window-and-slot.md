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
The fourth,
Q2,
is answered in the section "Three further answers",
with two questions a later review put.
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

## A further answer: an add whose own unmount is refused as busy

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T20:21:19-05:00,
after the design was rewritten to the three answers of this record and reviewed again
(`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review-after-the-owners-answers.md`,
section "What is left for the owner",
question Q-R5).

Asked:
by the first answer of this record an add that fails its proof unmounts and destroys the clone it just made.
When something else already sits in the new folder,
another session's shell for example,
the unmount is refused as busy and the add's unmount request to the root handler is left standing.
The accepted design allows no remount or unmount by itself after a failure,
with one exception,
a removal that cancels itself.

The project chose "A:
yes,
withdraw it (Recommended)":
the add writes its request back to the mounted state,
nothing having been unmounted,
prints that the worktree is pending and the command that removes it,
and the clone stays mounted as it was.
So the exception now covers an add that withdraws its own request,
beside a removal that cancels itself.

Not chosen:
leaving the request standing,
so that the handler unmounts the clone when the holder leaves.

## Three further answers

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T21:37:01-05:00,
on the review of the design's last round
(`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review-check-of-the-high-finding.md`,
section "What is left for the owner",
questions Q-O,
Q-S and Q-B).
Each answer is given as the option's label and its own description,
as the project was shown them.

### Who records what when the add undoes what it made

Asked:
four designs each described part of the failed add's undoing and disagreed on who records what;
the deciding case is a clone already destroyed and a snapshot whose destroy is refused because a backup tool holds it.
The project chose "B:
record the snapshot as owed (Recommended)":
"Before destroying anything,
the add records the snapshot as owed cleanup (the same record a removal uses).
You get a finished worktree made by Git's own checkout,
one line saying the snapshot stays for now,
and the next `git worktree` command destroys it once the hold is gone (your earlier answer)."
The costs stated with it:
the add writes two more records before its first destroy,
and a fallback leaves an owed-cleanup record a person may not expect.

Not chosen:
the add owning the path alone,
with no owed-cleanup record and the worktree pending until the same add is run again.

### The pending text warns against running Git in the folder

Asked:
the pending folder holds a full working copy of the main repository's `.git`,
frozen when the proof failed,
and a person still in that folder can run Git there.
The project chose "B:
add the warning (Recommended)":
"The pending message and `doctor`'s line gain one clause:
do not run Git in this folder until it is removed."
The cost stated with it:
it only warns;
it cannot stop a plain `git` there.

Not chosen:
leaving the text as it is.

### A very dirty main checkout is warned about and still takes this route

Asked:
this route restores each changed file after cloning,
so with very many changed files it can be slower than Git's own checkout;
should it decline above a measured count,
as the other clone route does.
The project selected "A:
decline above a measured count (Recommended)" with the note
"warn about a measured count,
but still do it."
Asked which of the two readings was meant,
the project chose "A:
warn,
still use the fast route (Recommended)":
"Above the measured count the add clones and restores as usual
and prints a warning that Git's own checkout may have been quicker for this many changed files.
It never declines for dirtiness."
The cost stated with it:
an add can be slower than Git's own checkout on a very dirty checkout.
The count comes from a measurement and is not decided here.

Not chosen:
declining above the count,
and no warning at all.

## The merge bar is waived for an add above the warning's count

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T22:00:52-05:00,
on a finding of the review of the design's next round
(`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review-check-before-acceptance.md`,
"What Q-B costs that the document does not say").

Asked:
the project's merge bar says a branch may make nothing `main` does well worse.
On `main`,
as far as the records say,
an add from a dirty main checkout on ZFS ends in Git's own checkout.
Once plan step 11 lands,
an add above the warning's count clones and restores every changed file instead,
and is slower than `main` by an amount not yet measured.
An integration pass would refuse that branch unless a record waives the bar for this case,
as `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` once did for the clone rung.

The project chose "B:
waive for good":
"Taking this route above the count is the intended behaviour and the bar never applies to it."
The cost stated with it:
it ships without anyone having shown the project how slow a very dirty add gets.

Not chosen:
a waiver that ends when the slowdown is measured,
which was the option recommended;
and no waiver,
with the add declining above the count after all.

What the question did not cover,
and this record therefore does not waive:
an add below the count,
and an add that fails its proof and finishes with Git's own checkout.
Each still owes its measured time against `main`.

## The design is accepted

Accepted in the question tool,
the question put on 2026-11-22 and answered on 2026-11-23,
recorded 2026-11-23T03:48:32-05:00.
The design is `doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`
as it stood at commit `23a2afe0bd` on its own branch,
for plan step 11 of `doc/planning/cli-git-zfs-worktree-implementation-plan.md`
and for the proof plan step 10 builds.

The project chose "A:
accept as written (Recommended)":
"All sixteen choices stand."
The sixteen choices,
as the project was shown them,
each with the alternative shown beside it where one was shown,
under the design's own numbers V1 to V16 in the same order:

1.  The state record holds the Git index,
    every tracked file's facts,
    and the content only of files those facts cannot vouch for.
    Not taken:
    hash every file,
    5 to 9 s per add.
2.  Facts compared:
    kind,
    size,
    both times,
    execute bit.
    Not the inode number.
    Not taken:
    also the inode.
3.  The record lives in a folder of its own under the wrapper's records in `.git`.
    Not taken:
    memory only,
    which allows no resume.
4.  A file changed in the same clock tick as the record is re-read by content,
    not trusted.
    Not taken:
    trust it,
    which is free and can pass off another session's edit as clean.
5.  The copied `.git` goes to a hidden trash in the new worktree,
    emptied after the add returns.
    Not taken:
    delete it before returning,
    a slower add.
6.  The proof is built in step 10 for every fast add,
    clean ones too;
    two plan sentences change.
    Not taken:
    only in step 11.
7.  A main checkout with an untracked nested Git repository is refused up front and gets Git's own checkout.
    Not taken:
    run and fail late.
8.  The copied index must match byte for byte,
    so a `git status` in main during the add can send it to Git's own checkout.
    Not taken:
    compare only the entries that matter.
9.  The undo of a failed add is built in step 11,
    inside the add.
    Not taken:
    in the removal's steps.
10. Three small records per add,
    each tied to that add.
11. After the restore,
    file facts are compared to the nanosecond;
    content only in a narrow band.
    Not taken:
    a full content check.
12. A pending folder keeps a live copy of main's `.git` until removed
    (the project chose the warning,
    not making it inert).
13. Another add's leftover record that cannot be read is cleared.
    Not taken:
    refuse and name it.
14. The add withdraws its unmount request on every outcome but success.
    Not taken:
    only on "busy".
15. Before writing the request back it checks whether the clone is already unmounted,
    and if so goes on with the undo instead of remounting.
    Not taken:
    write back first.
16. The slow-add warning prints after the add has finished.
    Not taken:
    before it starts.

The design's own entries name further alternatives the question did not show
(for the first three and the fourth,
a third option each);
the design is the fuller statement of each.

What the project was told with the question:
the design had seven reviews,
the last four finding nothing high;
the coordinator,
and no reviewer,
checked the last fixes against the diff;
nothing in it has been run,
nine measurements being owed by the build,
among them the count for the warning and the timing of the adds the waiver does not cover;
and two orderings still differed from the binding record design,
to be settled by edits there.

Not chosen:
accepting with a veto,
and not accepting yet.

## What this record does not decide

The count above which the warning prints,
which a measurement gives.
