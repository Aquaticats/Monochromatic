# cli-git clone rung: the partial clone of a dirty source, with provisional limits

## Status

Accepted 2026-11-21 in the question tool,
recorded 2026-11-21T22:04:07-05:00.
It answers the proposal `doc/planning/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`
(at `dc7b5e5624` on `docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`),
and it is the end condition of `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`,
section "When it ends".
Nothing of it is built yet.

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

## Left to the build

- Pin both limits at a measured edge,
  each written beside its value with the measurement it rests on,
  and state them to the project as provisional and open to veto.
- Fix the recheck's lows (N2 to N6) in the proposal.
- Rerun the gate re-check's six guard re-plants before the merge;
  five are still owed.
- Refine both limits with measurement after it lands.
