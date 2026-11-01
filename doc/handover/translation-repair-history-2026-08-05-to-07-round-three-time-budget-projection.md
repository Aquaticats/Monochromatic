# Translation repair history 2026-08-05 to 2026-08-07: whether fifteen entries fit the soft budget

The projection that fifteen entries might not fit the twelve-hour soft budget,
its sharpening by band,
and the correction that the budget caps one invocation,
not the round.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Whether fifteen entries fit the soft budget

### Budget risk: 15 entries may not fit the 12-hour soft budget

Measured off `pass8-run-007` rather than assumed,
at 59 minutes elapsed:

```text
per-chunk minutes: 2.7, 8.2, 11.8, 8.0, 11.0, 12.0
mean 9.80 min/chunk including setup
```

Projected to the fifteen settled entries the draw needs:

```text
3 slices/entry ->  7.4 h   fits
5 slices/entry -> 12.3 h   marginal, over the 12 h soft budget
7 slices/entry -> 17.2 h   does not fit
```

The entry running when this was measured has SEVEN slices.

TWO CAUSES,
and they are not equally fixable.
Provider latency is running high right now:
first-byte times of 126,
176 and
192 seconds appear in this log,
which is most of the per-chunk cost and is
nobody's design decision.
The introduced-defect probe also adds one stage per chunk that has applied
operations,
three parallel calls,
which is real but small beside the six-model
critic and panel stages.

WHAT HAPPENS AT THE BUDGET:
`corpus-pass.ts` stops STARTING new entries once
`SOFT_BUDGET_MS` is reached,
and finishes the one in flight.
It does not crash
and loses nothing already settled.
A short round is a smaller sample,
not a
broken one.

THE DECISION IF IT MATERIALIZES IS THE USER'S,
because both options spend
something they own:
raise `SOFT_BUDGET_MINUTES` and spend more quota and wall
clock,
or draw the sheets from fewer than fifteen entries and accept narrower
page diversity.
Note that the fifty-issue sample size is unaffected either way;
entry count buys diversity across pages,
not statistical power,
which is the
same arithmetic recorded when the cap was set.

DO NOT silently raise the budget.
Measure again at the halfway mark:
the band
interleave means small entries settle faster,
so the mean over fifteen may come
in well under the seven-slice worst case this projection uses.

#### Sharpened: the projection is band-aware, and 15 entries probably will not fit

The first projection spanned 7.4 to 17.2 hours because it guessed at slices per
entry.
Two facts remove that guess.

SLICES ARE SIZE-CAPPED at `SLICE_CHAR_BUDGET = 400` target characters,
so a
slice costs roughly the same wherever it comes from.
Band does not change the
price of a chunk,
only HOW MANY chunks an entry has.
The 9.80 min/chunk mean is
therefore usable across bands rather than only for the large entry it was
measured on.

THE PASS INTERLEAVES BANDS so coverage fills evenly,
so the first fifteen
settled entries are about five per band rather than fifteen of any one.

Estimating slice counts from the band thresholds (`SMALL_BAND_MAX_BYTES` 1843,
`MEDIUM_BAND_MAX_BYTES` 3686,
Chinese at roughly three bytes per character and
an English expansion near 1.6):

```text
small   ~2 slices     medium  ~4 slices     large  ~11 slices
15 entries at five per band ~ 85 slices
85 x 9.80 min = 13.9 h against a 12.0 h soft budget
```

Per-chunk time would have to fall to 8.5 minutes for fifteen entries to fit.

SO THE LIKELY OUTCOME IS A SHORT ROUND,
not a failed one:
the pass stops
starting entries at the budget and finishes the one in flight,
and roughly
twelve to thirteen entries settle instead of fifteen.

THE ONE VARIABLE THAT COULD CHANGE IT is provider latency,
which is the bulk of
the per-chunk cost right now (first-byte 126 to 192 seconds in this log) and is
not stable.
It has been better in earlier runs this session.
Re-measure rather
than trusting this.

STILL THE USER'S CALL,
and now worth raising BEFORE the budget fires rather than
after:
accept twelve or thirteen entries,
or raise `SOFT_BUDGET_MINUTES`.
The
fifty-issue sample is unaffected either way,
so the cost of the short round is
page diversity alone.

#### CORRECTION: the budget caps an INVOCATION, not the round

The previous two sections framed a choice between raising
`SOFT_BUDGET_MINUTES` and accepting a short round.
That choice does not exist,
and I raised it with the user before checking.

`corpus-pass.ts` computes its `done` set by READING THE ARTIFACTS DIRECTORY:

```ts
const done = new Set(
  (await readdir(artifactsDir,))
    .filter(isArtifactFile,)
    .map(toId,),
);
```

So a second invocation skips every entry already carrying an artifact and
continues with the rest.
The soft budget bounds how long ONE run keeps starting
entries;
it does not bound how many entries a round accumulates.
Artifacts
persist,
`attempts.json` persists,
and the per-entry slice cache is discarded on
settle so nothing stale carries over.

THE ACTUAL PROCEDURE when the budget fires:
start the pass again.
It will report
`done=N` for whatever settled and work the remainder.
Nothing is lost and no
configuration changes.

The projection still MATTERS,
just not as a decision:
it says the round needs
roughly one and a half invocations rather than one,
so plan for a second run
rather than being surprised by a short first one.
The pass is a tracked
background task,
so its exit notifies without polling.

WHAT SURVIVES of the earlier sections:
the measured 9.80 min/chunk,
the
band-independence of per-chunk cost,
the ~85 slices for fifteen entries,
and
that provider latency dominates and is unstable.
Only the framing as a
user-facing tradeoff was wrong.
