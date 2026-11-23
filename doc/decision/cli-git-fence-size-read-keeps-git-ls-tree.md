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

## The overlap, answered 2026-11-22

Recorded 2026-11-22T00:40:29-05:00,
after the measurement round's proposal
(`doc/planning/cli-git-fence-size-read-beside-source-check.md` on `docs/planning-cli-git-fence-size-read-beside-source-check`)
and its gate review
(`~/temp/agent/gate-review-fence-size-read-beside-source-check-2026-11-22/keep/review.md`).

The project answered "B:
start before the check (Recommended)":
on rung 1 only,
the fence's `git ls-tree` starts as a background task before the source check,
the fence waits for it,
and the read never fetches from a server (`GIT_NO_LAZY_FETCH=1`),
because on a partial clone the read otherwise downloads missing blobs to learn their sizes,
which the gate review measured on a source the check then refused.
Rung 2 is unchanged.
Measured with an identical-binary control:
217 ms of a 927 ms snapshot add on loose objects (23 percent),
22 ms of 523 ms on packed objects (4 percent).

It also answered "Yes,
record the cost (Recommended)":
an add the source check refuses,
or the fence declines,
may pay that read in the background.
The rule that a refused rung costs no extra Git run,
and the decline's stated price,
are amended to say so,
and the cost goes into the crate's `SPEC.md`.

## The recheck's two questions, answered 2026-11-22

Recorded 2026-11-22T17:33:43-05:00,
after the integration recheck of the build said it was not fit
(`~/temp/agent/integration-recheck-fence-size-read-beside-source-check-2026-11-22/keep/review.md`).

The build had set `GIT_NO_LAZY_FETCH=1` only when a test saw a promisor remote,
and the recheck made the read fetch into a source the check had refused where that test misses:
the remote's settings in an included configuration file,
set through the environment,
or in a `--reference` clone.
The project answered "A:
variable on every read (Recommended)":
the variable is set on every early read,
with no detection and no `--no-lazy-fetch` option.
A Git older than the releases that honour the variable ignores it and can fetch;
that Git is below the tested minimum of 2.56.0,
and the limit is written down.
Not taken:
"B:
A,
and skip partial clones",
and "C:
keep as built".

Where the source check refuses at once,
the add gave up the early read before its `git ls-tree` had started,
so nothing was stopped and the read ran to its end in the background
(80 of 80 measured adds).
The project answered "B:
really stop the read (Recommended)":
a read given up before it starts never starts its Git child,
or the child is ended the moment it appears.
Not taken:
"A:
reword the spec",
which the recheck had ranked first,
and "C:
leave both".

The fix round had been told both before the answers,
as picks open to veto;
the answers confirm them.

## Left open

The proposal branch also changed shipped code:
`ObjectDatabase::open` refuses any reftable store for its shipped callers,
with its only control in a module the default build does not compile.
That change is not part of this decision;
whether `main` answers a reftable repository's replace refs correctly is a separate question for its own branch.
