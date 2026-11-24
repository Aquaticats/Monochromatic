# cli-git clone rung: the partial clone of a dirty source, with provisional limits

## Status

Accepted 2026-11-21 in the question tool,
recorded 2026-11-21T22:04:07-05:00.
It answers the proposal `doc/planning/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`
(at `dc7b5e5624` on `docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`),
and it is the end condition of `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`,
section "When it ends".
Nothing of it was built when this was recorded.
(Note of 2026-11-23:
the build is on `feat/git-policy-cli-clone-rung-partial-clone-of-a-dirty-source`,
unmerged,
in a fix round on its integration recheck.)

## What was decided

The project answered "1:
clone with limits (Recommended)":
the clone rung (rung 2 of the worktree add ladder) clones every tracked file the proof shows unchanged
and lets Git write only the changed ones,
with two provisional limits that send an add to Git's own checkout instead:

- a whole-tree size threshold:
  a source whose tracked files sum below it goes to Git's own checkout;
  any value above 26 MB and up to 763 MB fits every measurement taken;
- a changed-file cut:
  a source with more changed files than it goes to Git's own checkout;
  any value from 6,000 to below 20,000 fits every measurement taken,
  and a quarter of the tracked files (6,155 in this repository) is the fence's own decline fraction.

An add the limits decline takes Git's own checkout,
so no add is slower than Git's own checkout.
(Note of 2026-11-23:
that sentence is superseded.
Measured,
a declined add is the source check and then Git's checkout,
and the requirement was restated under
"Four further answers after the recheck,
answered 2026-11-23",
"What a declined add may cost".)

## The measurements it rests on

Against Git's own checkout,
in position-balanced rounds with an identical-binary control
(the proposal's round of `dc7b5e5624` and its recheck,
`~/temp/agent/gate-recheck-partial-clone-proposal-on-the-fenced-rung-2026-11-21/keep/review.md`):

- the usual dirty add:
  0.73 to 0.77 of Git's time;
- 300 changed files:
  0.73;
- 6,000 changed files:
  1.00 to 1.01,
  parity to within about 0.15;
- 20,000 changed files:
  1.74,
  which the cut declines;
- a 2.6 KB dirty tree:
  1.30,
  which the size threshold declines (1.04 with the rule in the proposal's round).

## Consequences the project accepted

- The limits rest on few measured points.
  Adds they send to Git's own checkout miss the clone's gain until more measuring moves them.
- The small-file clean-add cost waived in the waiver record ends where the size threshold declines.

## The size limit on a clean source, answered 2026-11-22

Recorded 2026-11-22T13:18:08-05:00,
after the build's integration pass
(`~/temp/agent/integration-pass-clone-rung-partial-clone-2026-11-22/keep/review.md`,
findings H1,
H2 and M2).

The pass measured what the whole-tree size threshold does to a clean source,
which `main` clones whatever its size:
20 files of 20 MB (400 MB,
below the pinned 762,000,000 bytes) took 952 ms through Git's own checkout against `main`'s 33 ms,
about 30 times slower,
a shape none of this record's measurements held.
The waiver record's clean-add loss is on small files only
(10,001 files of 2.6 KB at 1.33 times `main`,
parity near 26 KB a file,
this repository's 32 KB files at 0.64),
so on a clean source the mean file size decides,
not the total.

The project answered "3:
file-size rule now":
before the merge,
a clean source whose mean tracked file is below a measured edge goes to Git's own checkout,
and a clean source of larger files clones as on `main`.
The whole-tree size threshold stays as decided for a source with changes,
where every measurement in this record was taken.
The edge is measured on the current build,
because the 26 KB figure came from step 3's builds on ZFS.

The pass's other findings are fixed in the same round without a question:
paths whose index entry is stale but whose bytes are unchanged are counted against the changed-file cut,
because Git writes them and the add was 1.39 times `main` on that shape.

## Four further answers after the recheck, answered 2026-11-23

Accepted in the question tool,
recorded 2026-11-23T04:06:17-05:00,
on the integration recheck of the build
(`~/temp/agent/integration-recheck-clone-rung-partial-clone-2026-11-22/keep/review.md`,
"For the owner to decide",
decisions A to D).
Each answer is given as the option's label and its own description,
as the project was shown them.

### No size rule on btrfs

Asked:
the build pinned a mean of 49 bytes a file on btrfs only because that was the smallest size measured,
and no btrfs size was ever measured to lose.
A clean btrfs tree whose files average under 49 bytes is 1.4 to 1.6 times slower than on `main`,
in every round,
because `main` clones it and the branch declines it.

The project chose "A:
no size rule on btrfs (Recommended)":
"Clone as main does on btrfs;
keep the rule on ZFS and on filesystems nobody measured."
The cost stated with it:
trees of a handful of tiny files keep `main`'s small loss against Git,
about 25 ms.

Not chosen:
keeping 49 bytes;
and a pin of about 2,600 bytes.

### What a declined add may cost

Asked:
the requirement was that no add is slower than Git's own checkout,
and the records say an add the rung declines is Git's own checkout.
Measured,
a declined add is the source check and then Git's checkout:
1.07 to 1.24 times Git's own checkout on small clean trees,
and inside the spread on dirty ones.

The project chose "A:
restate the requirement (Recommended)":
"The requirement becomes:
a declined add costs what main paid for the same source (the source check,
then Git's checkout),
with the measured figures written in the specification."
The cost stated with it:
"no add slower than Git's own checkout",
as first put,
is then not met on small clean trees.

Not chosen:
declining before the source check where the index alone decides;
and leaving the sentences as they are.

### No floor on the number of files

Asked:
whether a clean tree of only a few files goes to Git's own checkout whatever its file sizes,
since cloning's fixed cost makes trees of 20 to 500 files 10 to 30 ms slower cloned than written by Git,
which is also `main`'s cost.

The project chose "A:
no floor (Recommended)":
"Leave it as built."
The cost stated with it:
the requirement is missed by 10 to 30 ms on trees of a few files.

Not chosen:
making the clone's fixed cost cheaper as separate work;
and a measured floor per filesystem.

### The ZFS edge is lowered to about 3,000 bytes a file

Asked:
the ZFS edge is 8,799 bytes a file,
and the measurements cannot place the crossing better than between about 3,000 and 9,600 bytes;
across that range the clone ties Git on average and the declined route costs about 13% more than Git.

The project chose "A:
lower it to about 3,000 (Recommended)":
"Clone from about 3,000 bytes a file up,
as main does;
decline only below,
where the clone clearly loses."
The costs stated with it:
it clones at two sizes,
5,199 and 7,999,
whose own measurements read 1.2 times Git,
with the sizes between them winning;
and it is a new pin on the same noisy points.

Not chosen:
measuring first;
keeping 8,799;
and raising it to about 11,000.

The coordinator's reading,
not shown to the project:
the exact value is the build's to set beside the measurement it cites.
The option's fuller wording in the recheck is "about 3,000 (just above 2,599,
the one mean with a clear pooled loss)",
of which the project was shown "about 3,000".
Which edge a filesystem nobody measured takes after this change is not decided either;
the build proposes one,
open to the project's veto.

## Left to the build

- Pin both limits at a measured edge,
  each written beside its value with the measurement it rests on,
  and state them to the project as provisional and open to veto.
- Fix the recheck's lows (N2 to N6) in the proposal.
- Rerun the gate re-check's six guard re-plants before the merge;
  five are still owed.
- Refine both limits with measurement after it lands.

## Moved from the old handover

From lines 42200 to 42209 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  (brief under `~/temp/agent/gate-review-of-the-partial-clone-proposal-on-sonnet-2026-11-20/`).
  The owner then asked
  "Wait,
  why are we cloning every file individually when we have snapshots?":
  the accepted design's rung 1 is the ZFS
  snapshot clone (whole checkout,
  ignored state included,
  a verified restore for a dirty main,
  the proposal's "Rung 1:
  Snapshot clone"),
  and per-file cloning is rung 2,
  the fallback;
  rung 1 needs the managed route (an account cannot mount
  a clone),
  whose add route is step 10 (depends on steps 2,
  3,
  6 and 8) and dirty-main restore step 11.
  Asked whether to
  park the partial clone for steps 6 to 11,
  the owner answered "B:
  finish both":
  the gate review continues and the
  proposal comes to the owner,
  while steps 6 onward proceed.
  Step 6 is split by its own design:
  the start probe runs the
  module gate first and skips itself when the gate refuses,
  so the gate and the layout checker go first
  (`feat/git-policy-cli-zfs-start-time-compatibility-gate` in `.claude/worktrees/zfs-step-6-compatibility-gate`,
  brief
  under `~/temp/agent/zfs-step-6-compatibility-gate-and-layout-checker-2026-11-20/`) and the start probe follows on top.
