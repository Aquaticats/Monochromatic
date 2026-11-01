# Translation repair history 2026-08-23 to 2026-08-24: attempting the largest entries one after another in a single invocation

Why the build already stops moving for exactly one invocation,
the first-in first-out queue that replaced the single visit per entry,
the stop condition based on measured progress,
and what the change does not settle.

Part of [Translation repair history: 2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The largest entries: the frozen build `#196` asked for is one invocation wide

Landed 2026-08-24 in `c9fc220a0`.

`#196` recorded three ways out and ranked none,
and its first was to run an oversized entry as a SEQUENCE of attempts against a frozen build,
noted as costing nothing to build and needing the build to stop moving.
It also carried a warning against acting yet:

> DO THIS ONLY AFTER THE CURRENT LANDINGS SETTLE.
> The point of the first option is a stable digest,
> and it is worth nothing while fixes are still going in.

That warning rests on a premise that reading the code refutes.

### The build already stops moving, for exactly as long as one invocation

`corpus-pass.ts` computes `digestPipeline({ dir: import.meta.dirname, },)` once,
before the processing loop,
and every cache the run opens is namespaced by that digest in `slice-cache-namespace.ts`.
Nothing landing in this repository can move it under a running loop.
The slices a capped attempt bought are still this generation's when the next attempt opens the cache.

So the sequence never needed a frozen build in the operator's sense,
and it never needed the landings to settle.
What actually prevented it was scheduling,
and only scheduling:
the loop ran `for (const entry of pending)` over a list built before it started,
so a capped entry was visited exactly once
and the invocation moved on to other entries for up to the three-day soft budget.
XingZ60 got one 420-minute attempt per invocation
and every further attempt required a relaunch,
which re-read `HEAD`,
rebuilt,
and moved the digest if anything had landed in between.
That is the loop `#196` described as needing discipline to escape.
It needs a queue instead.

### What landed

The list became a FIFO queue.
An attempt that did not settle is pushed to the BACK,
so a large entry can never spend the budget
before every other entry has been tried once:
coverage of the corpus is what a first attempt buys.

### The stop condition is the half that needed care

`corpus-pass.ts` had already written the hazard down,
in the note beside `resumableIds`:

> NO PROGRESS GUARANTEE IS CLAIMED HERE,
> and one used to be:
> this said a cap-abort always completes at least one new slice,
> which is false.

An abort can land before the first persistence,
and the slices a lane deliberately leaves uncached,
the unfilled and the unheard,
produce no cache entry however long they took.
A retry-until-settled loop would therefore spend three days on an entry that never moves,
which is worse than the cap it replaced.

So an attempt earns another only on MEASURED progress:
`countCachedSlices` before and after,
across every lane sharing the entry's directory,
since progress is progress whichever lane made it.
Equal counts stop the entry and log `STALLED`.

### Two cases where the obvious reading is backwards

Both are about a cache count that FELL,
and both carry tests.

A SETTLED entry discards its slice cache on the way out,
in `pass-entry.ts`,
after the artifact write.
Inferring progress from the count alone
would read the one outcome the whole pass exists to reach
as the sharpest possible stall.
Settlement is therefore an input to the verdict,
never inferred from it.

A count that FALLS but stays above zero means the lane discarded an older build's cache,
so every slice left was bought by the attempt just finished.
The plain difference reads negative,
and taking it would drop the entry
precisely when it had begun paying for a fresh generation.

GFP-proven both ways.
Returning the plain difference fails the reset case alone;
removing the stall branch fails both stop-condition cases.
607 unit tests pass,
zero lint findings,
types clean.

### What this does not settle

The cap itself is untouched,
and so are `#196`'s other two ways out.
Raising the cap by slice count and cutting the per-slice round count
are still unmeasured against each other,
and the second changes quality,
so it stays last under the standing instruction.
Neither is needed to make an oversized entry settle now:
a single invocation can give one entry attempt after attempt against a build that cannot move.

What is owed is a live run.
The queue is proven at the unit boundary and by mutation,
not yet at the user boundary,
and the entry that would prove it is the one that costs thirteen hours.
