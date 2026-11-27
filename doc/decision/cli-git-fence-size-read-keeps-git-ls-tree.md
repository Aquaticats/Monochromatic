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
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

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
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

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
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

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

## Moved from the old handover

From lines 386 to 391 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The size read's reviewed branch merges now and its measured step cost is fixed next:
  "A:
  merge now,
  fix next (Recommended)",
  recorded in the size read's entry under "Build and diagnosis runs".

From lines 1603 to 1640 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The size read
  (`feat/git-policy-cli-fence-size-read-beside-source-check` at `f752fe3cab`):
  its Sonnet recheck found nothing high and no add slower
  (`~/temp/agent/integration-recheck-fence-size-read-never-fetches-2026-11-22/keep/review.md`;
  integration branch `integrate/git-policy-cli-fence-size-read-never-fetches-and-abandon-sticks` at `9a6fb83af0`,
  made on `main` at `f4746ec6a0`,
  before issue 6344's merge).
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
  It called the branch fit if one cost is accepted:
  on a declined add the step that chooses the rung takes 5.91 ms against `main`'s 0.67 ms,
  about half of it the adding thread freeing a map of 24,634 entries,
  with the add's total not slower.
  The owner chose "A:
  merge now,
  fix next (Recommended)" in the question tool on 2026-11-22,
  worded as a merge after a build and the touched tests pass on the merged result.
  The same reviewer merged `main` into that integration branch (`568f9ff721`),
  ran the build,
  both clippy tasks,
  `lint:types` and the touched modules' host tests,
  whose failures are the same 29 names as before
  (the btrfs controls and cause A),
  and wrote `keep/reverify-on-current-main.md` beside its review:
  fit by the owner's wording.
  Merged into `main` as `e37566df9c` at 2026-11-22T20:46:57-05:00;
  the wrapper handed the integration worktree's removal to the commit service.
  `native:check` on `main` after the merge ended with status 0
  (unit `check-main-after-the-size-read-merge`,
  56 s).
  The follow-up is running meanwhile:
  `pi-fence-size-read-recheck-findings-declined-add-step-cost`,
  on `continuation-recheck-findings.md` in
  `~/temp/agent/fence-size-read-never-fetches-and-abandon-sticks-2026-11-22/`
  (the step's cost found whole and removed,
  the release text corrected,
  the two reaped-id windows closed or reworded,
  the recheck's six new controls taken in).
  (scratch,
  deleted on 2026-11-27;
  not kept).
  Then a recheck of the follow-up alone,
  on a new integration branch.
