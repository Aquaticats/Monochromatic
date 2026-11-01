# Does the pipeline replace the archive more often where content looks displaced?

The reading `#107` waited on.
Answered 2026-08-18 over `~/translation-repair-runs-flagged-20260818/artifacts`,
five entries hand picked because `displacement-probe` flags them,
42 slices,
5.3 hours of production traffic.

THE ANSWER IS NO EFFECT THIS DATA CAN RESOLVE,
and the two readings disagree about which way the small difference points.

## The pre-registered reading

Union of every flag kind the probe emits,
over the delivery ledger,
fixed before any number existed:

```text
             flagged            unflagged
repair       12 of 16  0.7500   18 of 26  0.6923
translate    14 of 16  0.8750   20 of 26  0.7692
pooled       26 of 32  0.8125   38 of 52  0.7308
```

Flagged slices replace more often,
by about eight points pooled.

## The amended reading, which points the other way

A slice can retain the archive because judges preferred it,
or because nothing was offered against it.
Over the six naturally accumulated entries,
eleven of thirteen retentions are the second kind,
so the ledger mixes two events.
Read on slices that actually held a vote:

```text
             flagged            unflagged
translate    14 of 16  0.8750   20 of 21  0.9524
```

Flagged slices replace LESS often,
by about seven points.

Stated as archive survival,
which is the quantity `#107` cares about:

```text
flagged slices, this pool         2 of 16   0.125
unflagged slices, this pool       1 of 21   0.048
all slices, natural six-entry pool 2 of 39   0.051
```

THE AMENDMENT WAS RECORDED WITH ITS REASON BEFORE THE POOL SETTLED,
in the reader and in the task,
rather than chosen after seeing a number.
Both cuts are reported here for the same reason.

## Why this is a null rather than a small effect

THE SIGN FLIPS WITH THE DENOMINATOR,
which is what a difference smaller than the instrument looks like.

THE COUNTS ARE TINY.
The contested reading rests on two archive survivals against one.
Moving a single slice from one column to the other changes the direction.

THE UNFLAGGED HALF MATCHES THE CORPUS.
Unflagged slices in this hand-picked pool survive at 0.048
against 0.051 in the natural six-entry pool,
so the pool's ordinary slices behave ordinarily
and there is no pool-wide shift to attribute anything to.

## The null is worth something, because the instrument was shown able to move

Per `QPC`.
Run with `--control shipped`,
which replaces the flag list with exactly the slices that shipped,
the same reader reports 1.0000 against 0.0000 in both lanes.
It can display a difference.
It does not display one here.

The join also refuses rather than guesses:
it throws when the probe's slice count disagrees with the artifact's delivery rows,
when a lane carries duplicate slice indices,
and when a lane is missing,
each shown to fire on a throwaway fixture.

## What DID differ, and it is not about judging

CONTEST AVAILABILITY.
Every one of the 16 flagged slices held a vote,
against 21 of 26 unflagged.
The difference is whether a candidate arrived at all,
not how the judging went once one did.

CRITIC ATTENTION RUNS THE OTHER WAY.
Counted over the first two entries,
flagged slices drew 12.7 claims per hundred source characters against 15.2 for unflagged ones.
Whatever distinguishes a flagged slice,
it is not that the critic stage complains about it more.

## What this does not close

`#107`'s question was whether per-slice judging condemns the archive
for a relocation it cannot see.
This measures FREQUENCY and finds nothing.
It says nothing about whether the replacements at those slices are GOOD,
which needs a human and is what `relocation-sheet.md` asks:
30 items,
15 flagged against 15 unflagged controls,
blinded and stratified.
