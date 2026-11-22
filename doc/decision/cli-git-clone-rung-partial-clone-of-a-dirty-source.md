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

## Left to the build

- Pin both limits at a measured edge,
  each written beside its value with the measurement it rests on,
  and state them to the project as provisional and open to veto.
- Fix the recheck's lows (N2 to N6) in the proposal.
- Rerun the gate re-check's six guard re-plants before the merge;
  five are still owed.
- Refine both limits with measurement after it lands.
