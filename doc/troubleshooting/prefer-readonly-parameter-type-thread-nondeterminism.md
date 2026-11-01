# `prefer-readonly-parameter-type` reports different findings on repeated identical runs under oxlint 1.75.0 multi-threaded linting

Running the same lint twice over the same unmodified sources produces different diagnostics.
The difference is a whole finding appearing or disappearing,
 not a reordering.
Passing `--threads 1` removes it entirely.

## Symptom

`mise run //package/electron-infra/core:lint:oxlint`,
run repeatedly with no edit between runs,
alternates between eight and nine `prefer-readonly-parameter-type` findings.
The ninth is an offer:

```text
x prefer-readonly-parameter-type(prefer-readonly-parameter-types): Parameter "{
  | expected,
  | observed,
  | }" should be readonly: index signature is writable.
   ,-[src/wayland-state.ts:166:3]
```

`stateMatches` in `package/electron-infra/core/src/wayland-state.ts:165`
reads `Object.entries(expected)` and `observed[key]` and writes nothing,
so the offer is correct and the runs that omit it are the wrong ones.
The direction is therefore safe here,
a withheld offer rather than an offer for written state,
but nothing about a scheduling-dependent result guarantees that direction in general.

The same flip is visible at workspace scale.
Two full sweeps of the same commit reported 1837 findings with 27 offers and
1838 findings with 28 offers,
the difference being exactly this one diagnostic.

## Root cause

Pinned,
 and reproducible without threads at all.

A callable's foreign-borrowed verdict depends on how much of the effect graph happened to be
expanded when its summary was first requested.
`effect-demand-index.ts` gates the expensive complete-inbound proof behind a hint:

```ts
const partialForeignParameterIndexes = foreignByCallable.current.get(key,) ?? new Set();
if ((partialForeignParameterIndexes.size > 0) && (!verifiedForeignKeys.has(key,))) {
  // ... completeForeignBorrowedGraph, then verifiedForeignKeys.add(key)
}
const foreignParameterIndexes = verifiedForeignKeys.has(key,)
  ? completeForeignByCallable.get(key,) ?? new Set()
  : partialForeignParameterIndexes;
```

`completeForeignBorrowedGraph` is itself order-independent:
 it walks `indexedSourceFiles`,
 which
is the whole configured scope however little has been reached.
The gate in front of it is not.
`foreignByCallable.current` is recomputed after each expansion over the *reached* graph,
 so
whether the hint is non-empty when a callable is queried is a fact about which files the lint run
happened to visit first.
An empty hint skips the proof and answers "not foreign",
which is not a conservative default:
foreign ownership suppresses the readonly offer,
so the unproven answer emits an offer the proven one withholds.

Measured directly,
 one process,
 one project,
 one callable,
 no threads involved:

```text
before expanding siblings: {"foreign":[],"written":[],"opaque":[]}
after expanding siblings:  {"foreign":[0],"written":[],"opaque":[]}
```

The probe opens `wayland-state.ts`,
 reads `stateMatches`,
 then requests summaries for every
callable in `wayland-test.ts`,
 `wayland-control.ts`,
 `wayland-process.ts` and
`wayland-constants.ts`,
 and reads `stateMatches` again.
`wayland-test.ts` declares `expected: ForeignBorrowed<ExpectedObservedState>` and reaches
`stateMatches` through `waitForObservedState`,
so the marker is real and the second answer is the correct one.

Threads are therefore not the cause but an amplifier.
They decide the order files reach the rule,
 and the order decides the answer.

The earlier hypotheses are recorded as refuted rather than deleted,
 so nobody re-derives them:

- **The in-memory summary cache colliding across scopes.**
   `summariesByProject` is addressed by
   `projectKey` and file name while the persistent layer uses `scopeKey`,
   which looks like a
   missing partition.
   It is not one.
   `scopeKey` is `configFileName + analysisRoot`,
   `projectKey`
   is `configFileName`,
   and the `projectDigest` the memory cache validates already folds in
   `analysisRoot`.
   Two scopes differing only in analysis root therefore differ in
   `projectDigest` and cannot share an entry.
- **The inclusion scope differing per active file.**
   `indexedSourceFileMap` admits every
   non-declaration program source that is not from an external library,
   plus the active file,
   so
   the set and its digest are the same whichever file is active.
- **The fixed-point pass bound being exhausted.**
   That throws `SemanticBridgeError`,
   which
   `prefer-readonly-parameter-types.ts` logs as `semantic rule failed`,
   and no run of either kind
   logs one.
   The suppressed case still reports a diagnostic elsewhere in the same file,
   so
   analysis plainly completed.

What was established by measurement before the cause was found:

- The flip is controlled by oxlint's thread count,
   and by nothing else tested.
- It is not the persistent effect cache.
   Deleting
   `node_modules/.cache/prefer-readonly-parameter-type` before every run leaves the
   multi-threaded flip rate unchanged,
   and leaves single-threaded runs deterministic.
- It is not the analysis budget in `effect-analysis-budget.ts`.
   That fails closed through
   `SemanticBridgeError`,
   which `prefer-readonly-parameter-types.ts:204` logs as
   `semantic rule failed`,
   and no run of either kind emitted one.
- Raw output order differs between multi-threaded runs,
   so files reach the rule in a
   different order each time.
   Under `--threads 1` that order is fixed.

The rule holds process-local state that outlives a single file:
`finalIndexesByProject` in `effect-final-index-cache.ts:29`,
keyed by the TypeScript project snapshot and a file-list digest,
and `summariesByProject` in `effect-summary-cache.ts:59`.
An answer that depends on which files were analyzed first would explain everything measured,
and those are the places such a dependence could live.
That is a hypothesis,
 not a trace:
nothing here yet demonstrates which piece of that state changes the answer.

## Verification

oxlint 1.75.0,
 from the pnpm catalog.
Sixteen logical CPUs,
 so the default thread count is sixteen.
Every run is the same command over the same unmodified working tree at commit `54a1d06d4`:

```bash
mise run //package/electron-infra/core:lint:oxlint 2>&1 \
  | rg --count 'should be readonly'
```

The single-threaded form injects `--threads 1` through the wrapper's
`OXLINT_THREADS` environment variable,
 read at
`package/monorepo-task/core-exec/src/oxlint-wrapper.ts:50`:

```bash
OXLINT_THREADS=1 mise run //package/electron-infra/core:lint:oxlint 2>&1 \
  | rg --count 'should be readonly'
```

Measured,
 counting runs that reported the offer:

- Default threads,
   cache left warm:
   four of twelve.
- Default threads,
   `node_modules/.cache/prefer-readonly-parameter-type` deleted before
   each run:
   three of five.
- `--threads 1`,
   cache left warm:
   six of six.
- `--threads 1`,
   cache deleted before each run:
   four of four.

So seventeen multi-threaded runs split seven to ten,
and ten single-threaded runs agreed unanimously on the answer that is correct by inspection.

Every other finding in the package was identical across all runs.
Comparing sorted `location + message` pairs rather than raw output is necessary,
because multi-threaded runs also emit the same findings in a different order.

## A larger divergence, observed and not yet explained

The parallel multi-threaded root sweep reported 1837 findings with 27 offers.
A sequential single-threaded sweep of the same 128 package tasks reported 1741 findings with 33
offers.

Fewer findings alongside more offers is the signature of more analysis completing:
an unresolved call produces a report,
 and resolving it removes the report and can produce an
offer instead.
That points the same way as everything else here,
and it is a far larger effect than the single flipping offer characterized above.

Recorded as an observation.
Nothing has established which of the two sweeps is closer to right,
and the comparison the slot work depends on is single-threaded against single-threaded,
which this does not affect.

## A deterministic reproduction, no threads needed

Once the cause is known the flakiness reproduces as a plain function of the file set,
single-threaded,
 from `package/electron-infra/core`:

```bash
W=package/monorepo-task/core-exec/dist/final/node/oxlint-wrapper.mjs
OXLINT_THREADS=1 node "$W" --type-aware src/wayland-state.ts               # 1 offer
OXLINT_THREADS=1 node "$W" --type-aware src/wayland-state.ts src/wayland-test.ts   # 1 offer
OXLINT_THREADS=1 node "$W" --type-aware src/                              # 0 offers
```

Every four-file subset of the wayland cluster still offers;
only the whole cluster suppresses.
So no single companion file is the trigger,
which is what a cumulative expansion threshold looks like rather than a missing dependency.

Passing the target paths through `mise run ... -- <paths>` does not work:
the task's argument forwarding fails with `ERR_INVALID_TYPESCRIPT_SYNTAX` and oxlint never runs,
which reads as a clean zero-offer result.
Invoke the wrapper directly.

## Fixed, and what it cost

The gate is gone:
 the complete proof now decides every foreign question,
 and the optimistic
per-expansion pass that fed the gate went with it,
 nothing having read its answer any more.

Measured after,
 on the same package that produced the symptom:
eight of eight multi-threaded repeats agree,
where seven of seventeen disagreed before,
and all four file sets that used to disagree now report the same thing.
Across the workspace the fix withdraws exactly one offer,
`package/electron-infra/core/src/wayland-state.ts:166:3`,
the flaky one,
and moves nothing else.

It cost 57 percent of sweep wall time when it landed,
 616 seconds to 966.
Running the proof unconditionally cost 110 percent;
a pre-scan skipping any scope whose sources name neither marker identifier recovers the
difference,
and is an equivalence rather than a trade,
since `isForeignBorrowedType` can only detect a marker written under one of those names.
The two sweeps are byte-identical,
 which is the check rather than the argument.

Two attempts to memoize the closure across callables were both rejected by the sweep and are
recorded in task #34 so nobody repeats them.
Reusing every key of a closure's result added six reports;
reusing only the callables it enumerated added an offer,
which is the direction that offers `readonly` for written state.
Enumerating a callable's inbound call sites turns out not to be the same as being able to decide
that callable from a closure rooted elsewhere.

### The cost is gone, and the fix was not a cheaper closure

Nothing about the closure changed.
What changed is who asks for it.
The proof left `CallableEffectSummary` for `EffectSummaryIndex.proveForeignBorrowed`,
so the verifier demands it only when one of the four verdicts that read it could move,
and `includeActiveSource` and `verifyOverloadConsistency`,
 which never read it,
 stopped paying.

Measured at one thread from the repository root,
 same tree:
884.8s with the proof eager against 511.1s deferred.
The residual was then measured directly rather than inferred from that difference,
by timing every closure call in an instrumented build:
67 closures across 128 packages,
 5301.7ms in total,
 which is 1.04 percent of the sweep.

The findings are unchanged:
1937 findings and 32 offers on both sides,
3902 warnings and 3299 errors across every rule,
and 7201 diagnostics identical as message-plus-location pairs compared with multiplicity.
Switching the proof off instead adds exactly seven offers,
one of which is the flaky `wayland-state.ts:166:3` characterized here,
which is what the whole mechanism buys in this workspace.
`doc/planning/prefer-readonly-foreign-proof-cost.md` records the design and the other six.

## Sweeps compare analyzers, so everything else has to be held still

Three ways a sweep pair has silently measured something other than the analyzer,
each of which produced a clean-looking wrong answer before it was noticed:

- **Concurrent edits to linted source.**
   Comparing a sweep taken now against one taken hours
   earlier attributed nine moved report locations to a plugin change,
   when
   `package/config/tofu/src` had been edited twice in between by other work.
   The tell was line
   numbers shifting by a constant while the message text stayed identical,
   and
   `git log --since` over the named files confirmed it.
   Both sides of a pair have to be swept
   against the same tree,
   which for a historical plugin state means checking that state's plugin
   sources into the current tree rather than reusing an old log.
- **Committing while a sweep runs.**
   Each package task rebuilds the plugin when its sources are
   stale,
   so packages linted before and after a commit use different builds.
   One such run
   reported 1880 findings where a clean run of the same commit reported 1931.
- **Invoking the wrapper directly.**
   `oxlint-wrapper.mjs` does not rebuild the oxlint config;
   only the mise task's `ensureOxlintConfig()` does.
   Probing through the wrapper after a source
   change silently measures the previous build,
   which made a landed fix read as ineffective.

## Verified workarounds

Set `OXLINT_THREADS=1` for any run whose output is being compared against another run:

```bash
OXLINT_THREADS=1 mise run //:lint:oxlint
```

Tradeoff:
 the workspace sweep is the slowest thing in the repo already,
and this serializes it.
It buys a comparable result and buys nothing else,
so it belongs on measurement runs rather than on ordinary linting.

There is no workaround that keeps parallelism.
Reducing the thread count without reaching one narrows the window rather than closing it.

## What does not work

- Deleting the persistent cache before each run.
   The flip rate is unchanged,
   which is what
   rules the cache out as the cause rather than fixing anything.
- Comparing a single pair of runs.
   Two multi-threaded runs agreeing proves nothing,
   and the
   stage-one slot refactor was checked exactly that way:
   baseline and refactor both reported
   1837 findings and 27 offers with identical offer sets,
   which now reads as two samples from
   a distribution rather than as evidence the refactor changed nothing.
- Bisecting a multi-threaded difference.
   Five single-sample multi-threaded runs were used to
   attribute the 1838-and-28 sweep to a subset of the changed files,
   and every one of those
   samples is void:
   the bisect reported that neither half of the change reproduced the offer
   while the whole change did,
   which is only possible because each sample was a coin flip.
   The bisect is worth recording for one reason,
   that its incoherence is what prompted the
   determinism check.
   No conclusion drawn from it survives.
- Reading the run's warnings.
   The rule's fail-closed path logs
   `semantic rule failed, so <file> has no readonly analysis this run`,
   and neither the runs
   that produce the offer nor the runs that omit it log anything.

## Upstream filing decision

Nothing to file.
The first constraint decides it:
the defect is in this repository's own rule,
not in oxlint.
oxlint's contribution is scheduling files across threads,
which is its documented behaviour and is deterministic in the only sense it promises.
A rule whose answer depends on the order its files arrive in is the rule's bug.

`.out-of-scope/` was checked and holds no entry for oxlint or for this bug class.
The remaining five constraints are not reached,
because a report would be addressed to ourselves.

The repository-side work is tracked as a task rather than an issue,
and this document is the record a future session should read before trusting any sweep
comparison.
