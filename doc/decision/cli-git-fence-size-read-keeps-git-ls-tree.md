# cli-git conversion fence: the size read keeps `git ls-tree`

## Status

Accepted 2026-11-21 in the question tool,
recorded 2026-11-21T23:35:36-05:00.
It closes the follow-up recorded in `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 10's notes
(the answer "A:
merge,
then faster reader"),
and answers the proposal `doc/planning/cli-git-fence-faster-size-reader.md`
(at `4ec8aa27f6` on `docs/planning-cli-git-fence-faster-size-reader`)
and its gate review and recheck
(`~/temp/agent/gate-review-fence-faster-size-reader-2026-11-21/keep/review.md`).

## What was decided

The project answered "A:
keep git ls-tree (Recommended)":
the clone rung's conversion fence keeps reading recorded sizes with one `git ls-tree`,
and no faster reader is built.

It also answered "A:
measure it (Recommended)":
a round prototypes running the size read at the same time as the source check,
times it in a position-balanced order on loose and packed sources,
and comes back before anything merges.

## What it rests on

Measured on one tree in one guest,
with an identical-binary control:

- packed objects:
  the read is 24 ms of a 517 ms add;
  98.1 percent of this repository's `HEAD` blobs are packed;
- loose objects:
  the read is 341 ms of an 868 ms add;
  step 10's 338 ms came from a fixture built without `git gc`.

The options weighed,
with the gate recheck's corrected figures:

- `gix` 0.90:
  about 15 ms saved on a packed add,
  slower on loose objects,
  47 added packages,
  licences including MPL-2.0,
  Zlib and BSD-3-Clause,
  `unsafe` in its pack reader,
  and the wrong size for a reftable replace ref;
- `gix-odb` alone:
  15 added packages,
  the wrong size for a replaced object;
- the in-process prototype reader:
  faster on loose objects,
  about 13 ms slower on packed ones;
- `git cat-file`:
  slower than `git ls-tree` in every case measured.

## Consequences the project accepted

A source with many loose objects keeps paying the read's full cost
(about 340 ms in the guest)
until `git gc` packs it,
or until the overlap measured next hides it.

## Left open

The proposal branch also changed shipped code:
`ObjectDatabase::open` refuses any reftable store for its shipped callers,
with its only control in a module the default build does not compile.
That change is not part of this decision;
whether `main` answers a reftable repository's replace refs correctly is a separate question for its own branch.
