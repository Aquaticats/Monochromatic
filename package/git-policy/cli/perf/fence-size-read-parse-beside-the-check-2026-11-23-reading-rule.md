# Reading rule, written 2026-11-23 before any guest row of this round was taken

Second-pass fix round (rule `IRW`):
not independent.
Unapproved and non-final.

## What this run must show

The pass's Medium 1:
the read's parse must be back off the adding thread,
so that **every served,
dirty and
declined shape is at most `main`'s,
step for step**,
and the declined add keeps the gain the pass measured.

## Arms (one guest, btrfs loop image, Git 2.56.1, release builds)

- `base`:
  `75a8440193`,
  `main` before the feature (the pass's own base).
- `main`:
  `9852f60358`,
  `main` as it stands (it holds the earlier build of this branch,
  the unparsed hand-over).
- `fix`:
  this round's commit on top of the merge of `main` at `2c7ecb9bd4` (the parse back on the read thread,
  the answered map freed on a thread of its own).
- `fix-again`:
  the identical binary of `fix`,
  the control for the method's own spread.

Hashes:
`binaries.txt`.
Every round rotates the four arms one place and reverses every second pass;
one uncounted
warm-up per arm;
`sync --file-system` before every add;
one driver process inside the guest;
no build,
test or gate
of mine on the host while counted rounds run.

## The step compared

- `step(base)` = `chooseRung`;
  `step(main)` = `chooseRung` (the record books own time,
  and `main`'s `chooseRung`
  holds the read's start and the abandon).
- `step(fix)` = `chooseRung + startBesideTheCheck + abandonRead + abandonTakeTheAnswer + abandonEndTheChild`
  (this build carves four steps out of `chooseRung`;
  the sum is the like-for-like quantity).
- The fence is read as `conversionFence` directly,
  and as `conversionFence + fenceSizesWaitingForTheRead`
  (the parse's place on the adding thread shows up in `conversionFence` after the join,
  and in
  `fenceSizesWaitingForTheRead` only while the join waits).
- The read's own work (`work.fenceSizesBesideTheCheck`) is read where an arm states it.

## Statistics

- Per-round paired differences (arm minus arm,
  same round),
  and per-round ratios of `totalMs`.
- The median over rounds,
  with a 95 percent percentile bootstrap interval of the median (10,000 resamples,
  fixed seed).
- The control's spread:
  the same statistic for `fix-again` minus `fix`.
- **Resolved worse** only when the interval of `fix` minus `main` lies entirely above zero AND is not covered by the
  control's interval AND the median difference is at least 0.1 ms for a step or 1 percent for a total.
  Anything else is
  **not resolved**.
- **Positive control**:
  `main` minus `base` on the declined step must be positive with an interval excluding zero
  (the recheck measured +4 to +6 ms).
  If it is not reproduced,
  a null on `fix` minus `main` is not read as "cost gone".
- **Medium 1 met** means:
  in each of served,
  dirty and declined,
  `conversionFence` (and the step) is not resolved worse
  than `main`,
  and `fix` minus `base` on `conversionFence` is not resolved worse either.
- The round author's rule (every round's ratio above the twin's largest) is reported beside,
  not instead.

## Rounds

Served clean,
served dirty and declined:
12 per layout;
loose and packed layouts.
The pass resolved the parse on
12-round data,
so 12 is the round count that can resolve it;
more only if a step lands inside the control's spread
and the spread is wide.
